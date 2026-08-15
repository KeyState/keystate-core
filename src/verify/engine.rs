//! The shared, backend-agnostic verification engine.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::verify::manifest::{EntityExpectation, FieldManifest};
use crate::verify::path::resolve_path;
use crate::verify::report::{IssueKind, Severity, VerificationIssue, VerificationReport};
use crate::{CanonicalRealm, EntityKind, Result};

/// The shared, backend-agnostic verification engine.
///
/// Presence semantics per the architecture document: every *existing* row must
/// carry its required paths, entity types may require at least one row, and
/// volatile fields must be segregated into the volatile section. Non-required
/// fields are verified at the entity level: the realm is incomplete if no row
/// carries them (a field introduced in a newer schema version, silently
/// missed).
pub struct ManifestVerifier;

impl ManifestVerifier {
    /// Verify an extraction against a manifest. The manifest must have passed
    /// [`FieldManifest::validate`]; an invalid manifest `Err`s.
    pub fn verify(
        &self,
        extracted: &CanonicalRealm,
        manifest: &FieldManifest,
    ) -> Result<VerificationReport> {
        manifest.validate()?;
        let rows = rows_by_entity(extracted);
        let mut issues = Vec::new();

        for ee in &manifest.entities {
            self.check_entity_presence(ee, &rows, &mut issues);
        }
        for fe in &manifest.fields {
            self.check_field_expectation(extracted, fe, &rows, &mut issues);
        }

        Ok(VerificationReport {
            backend: manifest.backend.clone(),
            against_version: manifest.for_version,
            issues,
        })
    }

    fn check_entity_presence(
        &self,
        expectation: &EntityExpectation,
        rows: &BTreeMap<EntityKind, Vec<(String, Value)>>,
        issues: &mut Vec<VerificationIssue>,
    ) {
        use crate::verify::manifest::EntityPresence;

        let has_rows = rows
            .get(&expectation.entity)
            .is_some_and(|rows| !rows.is_empty());
        if expectation.presence == EntityPresence::Required && !has_rows {
            issues.push(VerificationIssue {
                severity: Severity::Error,
                kind: IssueKind::MissingEntity,
                entity: expectation.entity,
                row_id: None,
                path: None,
                introduced_in: None,
                message: format!(
                    "entity {} is required but no rows were extracted",
                    entity_label(expectation.entity)
                ),
            });
        }
    }

    fn check_field_expectation(
        &self,
        extracted: &CanonicalRealm,
        fe: &crate::verify::manifest::FieldExpectation,
        rows: &BTreeMap<EntityKind, Vec<(String, Value)>>,
        issues: &mut Vec<VerificationIssue>,
    ) {
        let Some(entity_rows) = rows.get(&fe.entity) else {
            // Zero rows of a present-but-empty entity is a complete state.
            return;
        };

        if fe.volatile {
            self.check_volatile(extracted, fe, entity_rows, issues);
        } else if fe.required {
            for (row_id, row) in entity_rows {
                if !resolve_path(row, &fe.path) {
                    issues.push(VerificationIssue {
                        severity: Severity::Error,
                        kind: IssueKind::MissingRequired,
                        entity: fe.entity,
                        row_id: Some(row_id.clone()),
                        path: Some(fe.path.clone()),
                        introduced_in: Some(fe.introduced_in),
                        message: format!(
                            "required field '{}' (introduced in {}) missing from {} '{}'",
                            fe.path,
                            fe.introduced_in,
                            entity_label(fe.entity),
                            row_id
                        ),
                    });
                }
            }
        } else if !entity_rows
            .iter()
            .any(|(_, row)| resolve_path(row, &fe.path))
        {
            issues.push(VerificationIssue {
                severity: Severity::Error,
                kind: IssueKind::MissingOptional,
                entity: fe.entity,
                row_id: None,
                path: Some(fe.path.clone()),
                introduced_in: Some(fe.introduced_in),
                message: format!(
                    "field '{}' (introduced in {}) absent from every {} row",
                    fe.path,
                    fe.introduced_in,
                    entity_label(fe.entity)
                ),
            });
        }
    }

    fn check_volatile(
        &self,
        extracted: &CanonicalRealm,
        fe: &crate::verify::manifest::FieldExpectation,
        entity_rows: &[(String, Value)],
        issues: &mut Vec<VerificationIssue>,
    ) {
        for (row_id, row) in entity_rows {
            if resolve_path(row, &fe.path) {
                issues.push(volatile_issue(
                    IssueKind::MisplacedVolatile,
                    fe.entity,
                    Some(row_id.clone()),
                    fe.path.clone(),
                    Severity::Error,
                    format!(
                        "volatile field '{}' leaked into the config body of {} '{}'",
                        fe.path,
                        entity_label(fe.entity),
                        row_id
                    ),
                ));
            }
            if extracted
                .volatile
                .get(fe.entity, row_id, &fe.path)
                .is_none()
            {
                issues.push(volatile_issue(
                    IssueKind::MissingVolatile,
                    fe.entity,
                    Some(row_id.clone()),
                    fe.path.clone(),
                    Severity::Warning,
                    format!(
                        "volatile field '{}' not captured in the volatile section for {} '{}'",
                        fe.path,
                        entity_label(fe.entity),
                        row_id
                    ),
                ));
            }
        }
    }
}

fn volatile_issue(
    kind: IssueKind,
    entity: EntityKind,
    row_id: Option<String>,
    path: String,
    severity: Severity,
    message: String,
) -> VerificationIssue {
    VerificationIssue {
        severity,
        kind,
        entity,
        row_id,
        path: Some(path),
        introduced_in: None,
        message,
    }
}

/// A human-readable label for an entity kind.
pub(crate) fn entity_label(entity: EntityKind) -> &'static str {
    match entity {
        EntityKind::Realm => "realm",
        EntityKind::Client => "client",
        EntityKind::ClientScope => "client scope",
        EntityKind::Group => "group",
        EntityKind::Role => "role",
        EntityKind::User => "user",
        EntityKind::Credential => "credential",
        EntityKind::ProtocolMapper => "protocol mapper",
        EntityKind::AuthFlow => "auth flow",
        EntityKind::IdentityProvider => "identity provider",
    }
}

/// Collect all entity rows from an extraction, keyed by kind, as
/// `(row_id, value)` pairs. Rows keep the adapter's emission order; the
/// verifier only reports, it does not reorder.
pub(crate) fn rows_by_entity(realm: &CanonicalRealm) -> BTreeMap<EntityKind, Vec<(String, Value)>> {
    let mut rows: BTreeMap<EntityKind, Vec<(String, Value)>> = BTreeMap::new();

    macro_rules! push {
        ($kind:expr, $iter:expr) => {
            for row in $iter {
                rows.entry($kind)
                    .or_default()
                    .push((row.id.clone(), value_of(row)));
            }
        };
    }

    rows.insert(
        EntityKind::Realm,
        vec![(realm.realm.name.clone(), value_of(&realm.realm))],
    );
    push!(EntityKind::Client, realm.clients.iter());
    push!(EntityKind::ClientScope, realm.client_scopes.iter());
    push!(EntityKind::Group, realm.groups.iter());
    push!(EntityKind::Role, realm.roles.iter());
    push!(EntityKind::User, realm.users.iter());
    push!(EntityKind::Credential, realm.credentials.iter());
    push!(EntityKind::ProtocolMapper, realm.protocol_mappers.iter());
    push!(EntityKind::AuthFlow, realm.auth_flows.iter());
    push!(
        EntityKind::IdentityProvider,
        realm.identity_providers.iter()
    );
    rows
}

fn value_of<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("the canonical model always serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Version;
    use crate::model::entities::{Client, Realm, User};
    use crate::verify::manifest::{EntityPresence, FieldExpectation};
    use serde_json::json;

    fn realm_master() -> CanonicalRealm {
        CanonicalRealm {
            realm: Realm {
                name: "master".into(),
                enabled: true,
                native: json!({ "registrationAllowed": false }),
            },
            ..CanonicalRealm::default()
        }
    }

    fn manifest_for(backend: &str) -> FieldManifest {
        let mut manifest = FieldManifest::new(backend, Version::new(26, 0, 0));
        manifest.entities.push(EntityExpectation::new(
            EntityKind::Client,
            EntityPresence::Optional,
        ));
        manifest
    }

    fn verify(realm: &CanonicalRealm, manifest: &FieldManifest) -> VerificationReport {
        ManifestVerifier.verify(realm, manifest).unwrap()
    }

    #[test]
    fn empty_realm_without_expectations_is_complete() {
        let report = verify(&realm_master(), &manifest_for("stub"));
        assert!(report.is_complete(), "{:?}", report.issues);
        assert_eq!(report.counts(), (0, 0));
    }

    #[test]
    fn zero_user_rows_with_a_required_entity_is_not_incomplete() {
        let realm = realm_master();
        let mut manifest = manifest_for("stub");
        manifest.entities.push(EntityExpectation::new(
            EntityKind::User,
            EntityPresence::Optional,
        ));
        manifest.fields.push(FieldExpectation {
            entity: EntityKind::User,
            path: "native.email".into(),
            required: true,
            volatile: false,
            introduced_in: Version::new(20, 0, 0),
        });
        let report = verify(&realm, &manifest);
        assert!(report.is_complete(), "{:?}", report.issues);
    }

    #[test]
    fn required_entity_without_rows_is_an_error() {
        let realm = realm_master();
        let mut manifest = manifest_for("stub");
        manifest.entities.push(EntityExpectation::new(
            EntityKind::IdentityProvider,
            EntityPresence::Required,
        ));
        let report = verify(&realm, &manifest);
        assert!(!report.is_complete());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.kind == IssueKind::MissingEntity)
        );
    }

    #[test]
    fn missing_required_field_on_an_existing_row_is_an_error() {
        let mut realm = realm_master();
        realm.clients.push(Client {
            id: "c1".into(),
            client_id: "web".into(),
            enabled: true,
            native: json!({ "protocol": "openid-connect" }),
        });
        let mut manifest = manifest_for("stub");
        manifest.fields.push(FieldExpectation {
            entity: EntityKind::Client,
            path: "native.secret".into(),
            required: true,
            volatile: false,
            introduced_in: Version::new(1, 0, 0),
        });
        let report = verify(&realm, &manifest);
        assert!(!report.is_complete());
        let issue = report
            .issues
            .iter()
            .find(|i| i.kind == IssueKind::MissingRequired)
            .expect("a missing-required issue");
        assert_eq!(issue.row_id.as_deref(), Some("c1"));
        assert_eq!(issue.introduced_in, Some(Version::new(1, 0, 0)));
    }

    #[test]
    fn missing_optional_field_on_every_row_is_an_error() {
        let mut realm = realm_master();
        realm.clients.push(Client {
            id: "c1".into(),
            client_id: "web".into(),
            enabled: true,
            native: json!({ "protocol": "openid-connect" }),
        });
        let mut manifest = manifest_for("stub");
        manifest.fields.push(FieldExpectation {
            entity: EntityKind::Client,
            path: "native.alternativeNames[]".into(),
            required: false,
            volatile: false,
            introduced_in: Version::new(26, 0, 0),
        });
        let report = verify(&realm, &manifest);
        assert!(!report.is_complete());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.kind == IssueKind::MissingOptional)
        );
    }

    #[test]
    fn optional_field_present_on_any_row_passes() {
        let mut realm = realm_master();
        realm.clients.push(Client {
            id: "c1".into(),
            client_id: "web".into(),
            enabled: true,
            native: json!({ "alternativeNames": ["alpha", "beta"] }),
        });
        realm.clients.push(Client {
            id: "c2".into(),
            client_id: "api".into(),
            enabled: true,
            native: json!({ "protocol": "openid-connect" }),
        });
        let mut manifest = manifest_for("stub");
        manifest.fields.push(FieldExpectation {
            entity: EntityKind::Client,
            path: "native.alternativeNames[]".into(),
            required: false,
            volatile: false,
            introduced_in: Version::new(26, 0, 0),
        });
        let report = verify(&realm, &manifest);
        assert!(report.is_complete(), "{:?}", report.issues);
    }

    #[test]
    fn misplaced_volatile_field_is_an_error_and_missing_volatile_a_warning() {
        let mut realm = realm_master();
        realm.users.push(User {
            id: "u1".into(),
            username: "alice".into(),
            enabled: true,
            // last_login should have been segregated; it leaked into native:
            native: json!({ "last_login": "2026-08-15T10:00:00Z" }),
        });
        let mut manifest = manifest_for("stub");
        manifest.fields.push(FieldExpectation {
            entity: EntityKind::User,
            path: "native.last_login".into(),
            required: false,
            volatile: true,
            introduced_in: Version::new(1, 0, 0),
        });
        let report = verify(&realm, &manifest);
        assert!(!report.is_complete());
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.kind == IssueKind::MisplacedVolatile)
        );
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.kind == IssueKind::MissingVolatile)
        );
    }

    #[test]
    fn correctly_segregated_volatile_field_passes() {
        let mut realm = realm_master();
        realm.users.push(User {
            id: "u1".into(),
            username: "alice".into(),
            enabled: true,
            native: json!({}),
        });
        realm.volatile.insert(
            EntityKind::User,
            "u1",
            "native.last_login",
            json!("2026-08-15T10:00:00Z"),
        );
        let mut manifest = manifest_for("stub");
        manifest.fields.push(FieldExpectation {
            entity: EntityKind::User,
            path: "native.last_login".into(),
            required: false,
            volatile: true,
            introduced_in: Version::new(1, 0, 0),
        });
        let report = verify(&realm, &manifest);
        assert!(report.is_complete(), "{:?}", report.issues);
    }

    #[test]
    fn verify_rejects_invalid_manifests() {
        let mut manifest = manifest_for("stub");
        manifest.entities.push(EntityExpectation::new(
            EntityKind::Realm,
            EntityPresence::Optional,
        ));
        assert!(ManifestVerifier.verify(&realm_master(), &manifest).is_err());
    }

    #[test]
    fn rows_by_entity_keeps_realm_keyed_by_name() {
        let realm = realm_master();
        let rows = rows_by_entity(&realm);
        assert_eq!(rows[&EntityKind::Realm][0].0, "master");
    }
}
