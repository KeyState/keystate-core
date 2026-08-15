//! The extracted envelope: what one extraction run produces, and what one
//! selects.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::entities::{
    AuthFlow, Client, ClientScope, Credential, Group, IdentityProvider, ProtocolMapper, Realm,
    Role, User,
};
use crate::model::kind::EntityKind;
use crate::version::Version;

/// Metadata describing the backend and the schema version detected at the
/// source. Rides inside the extracted envelope so manifests and reports are
/// provably bound to a concrete version.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BackendInfo {
    /// Backend identifier, e.g. `"keycloak"` or `"ferriskey"`.
    #[serde(default)]
    pub backend: String,
    /// The schema version detected at the source.
    pub detected_version: Version,
    /// Any backend-specific discovery detail the canonical model has not
    /// formalized (e.g. distribution/edition). Opened but kept in the
    /// envelope, not in entity rows.
    #[serde(default)]
    pub extra: BTreeMap<String, Value>,
}

/// Selects what one extraction run pulls.
///
/// Release one extracts exactly one realm per run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractScope {
    /// The realm to extract.
    pub realm: String,
}

impl ExtractScope {
    /// Construct a scope for a single realm.
    pub fn new(realm: impl Into<String>) -> Self {
        Self {
            realm: realm.into(),
        }
    }
}

/// The volatile portion of an extraction.
///
/// Everything that changes on its own between runs with zero configuration
/// drift — last-login timestamps, session-derived data — lives here, never in
/// the config body. Fields are keyed by entity kind, row id, then their
/// dotted path, so the completeness verifier can prove, for every existing
/// row, that its volatile fields were captured.
///
/// The content hash covers only the config portion; this section never enters
/// it, so volatile churn cannot pollute drift detection.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VolatileSection(
    /// `entity kind -> row id -> dotted path -> value`.
    pub BTreeMap<EntityKind, BTreeMap<String, BTreeMap<String, Value>>>,
);

impl VolatileSection {
    /// An empty volatile section.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the section carries no data for any entity.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Record one volatile value for a row.
    pub fn insert(
        &mut self,
        entity: EntityKind,
        row_id: impl Into<String>,
        path: impl Into<String>,
        value: Value,
    ) {
        self.0
            .entry(entity)
            .or_default()
            .entry(row_id.into())
            .or_default()
            .insert(path.into(), value);
    }

    /// Look up a recorded volatile value.
    pub fn get(&self, entity: EntityKind, row_id: &str, path: &str) -> Option<&Value> {
        self.0
            .get(&entity)
            .and_then(|by_id| by_id.get(row_id))
            .and_then(|by_path| by_path.get(path))
    }

    /// All row ids recorded for an entity kind, in stable order.
    pub fn row_ids<'a>(&'a self, entity: &'a EntityKind) -> impl Iterator<Item = &'a String> {
        self.0.get(entity).into_iter().flat_map(|m| m.keys())
    }
}

/// One canonical realm's worth of extracted state.
///
/// Collections are the adapter's responsibility to sort by a stable key
/// before being placed here (see `ARCHITECTURE.md` §4); core serializes
/// deterministically on top of that.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CanonicalRealm {
    /// Backend and schema version detected at the source.
    #[serde(default)]
    pub backend: BackendInfo,
    /// The realm itself.
    pub realm: Realm,
    /// Clients.
    #[serde(default)]
    pub clients: Vec<Client>,
    /// Reusable provider-configuration bundles.
    #[serde(default)]
    pub client_scopes: Vec<ClientScope>,
    /// Groups.
    #[serde(default)]
    pub groups: Vec<Group>,
    /// Roles.
    #[serde(default)]
    pub roles: Vec<Role>,
    /// Users.
    #[serde(default)]
    pub users: Vec<User>,
    /// Credential records.
    #[serde(default)]
    pub credentials: Vec<Credential>,
    /// Protocol mappers.
    #[serde(default)]
    pub protocol_mappers: Vec<ProtocolMapper>,
    /// Authentication flows.
    #[serde(default)]
    pub auth_flows: Vec<AuthFlow>,
    /// External identity providers.
    #[serde(default)]
    pub identity_providers: Vec<IdentityProvider>,
    /// Volatile, non-configuration data, keyed by entity/row/path.
    #[serde(default)]
    pub volatile: VolatileSection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_has_default_empty_state() {
        let realm = CanonicalRealm::default();
        assert!(realm.clients.is_empty());
        assert!(realm.volatile.is_empty());
    }

    #[test]
    fn volatile_section_roundtrips_and_is_ordered() {
        let mut section = VolatileSection::new();
        section.insert(
            EntityKind::User,
            "u1",
            "native.last_login",
            serde_json::json!("2026-01-01"),
        );
        section.insert(
            EntityKind::User,
            "u2",
            "native.last_login",
            serde_json::json!("2026-01-02"),
        );
        assert_eq!(
            section
                .get(EntityKind::User, "u1", "native.last_login")
                .unwrap(),
            &serde_json::json!("2026-01-01")
        );
        assert_eq!(
            section.row_ids(&EntityKind::User).collect::<Vec<_>>(),
            vec![&"u1".to_string(), &"u2".to_string()]
        );

        let json = serde_json::to_value(&section).unwrap();
        assert!(json.is_object());
        let back: VolatileSection = serde_json::from_value(json).unwrap();
        assert_eq!(back, section);
    }

    #[test]
    fn backend_info_defaults_to_unknown_backend() {
        let info = BackendInfo::default();
        assert_eq!(info.backend, "");
        assert_eq!(info.detected_version, Version::default());
    }
}
