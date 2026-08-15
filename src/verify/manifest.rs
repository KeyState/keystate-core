//! The expected-configuration data shapes: [`EntityExpectation`],
//! [`EntityPresence`], [`FieldExpectation`] and [`FieldManifest`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::EntityKind;
use crate::{Error, Result, Version};

/// Whether an entity *type* is expected to have at least one row.
///
/// A freshly created realm with zero users (or zero identity providers) is a
/// valid, complete state — only [`Required`](EntityPresence::Required)
/// entities must have rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityPresence {
    /// At least one row of this entity must exist.
    Required,
    /// Zero rows is a valid, complete state.
    Optional,
}

/// A declaration about how many rows an entity type must have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityExpectation {
    /// The entity kind this expectation covers.
    pub entity: EntityKind,
    /// Whether at least one row is required.
    pub presence: EntityPresence,
}

impl EntityExpectation {
    /// Declare that an entity type must (or must not) have rows.
    pub fn new(entity: EntityKind, presence: EntityPresence) -> Self {
        Self { entity, presence }
    }
}

/// A single expected field, keyed to one entity kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldExpectation {
    /// The entity kind whose rows must carry this path.
    pub entity: EntityKind,
    /// Dotted path relative to the canonical row — e.g. `client_id`, or
    /// `native.alternativeNames[]` for an array-valued native field. Array
    /// segments end in `[]`.
    pub path: String,
    /// When `true`, every *existing* row of `entity` must carry `path`.
    /// When `false`, the field is complete if at least one row carries it.
    pub required: bool,
    /// When `true`, the field is volatile — session/last-login style data
    /// that changes between runs. It must be reported in the volatile
    /// section, never in the config body, and never in the content hash.
    pub volatile: bool,
    /// The backend schema version that introduced the field, so a report can
    /// say concretely "introduced in 26.x and absent here".
    pub introduced_in: Version,
}

/// The complete expected configuration state for one backend version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldManifest {
    /// Backend this manifest describes, e.g. `"keycloak"`.
    pub backend: String,
    /// The backend schema version these expectations were derived from.
    pub for_version: Version,
    /// Entity-type presence requirements.
    pub entities: Vec<EntityExpectation>,
    /// Field-level expectations, including volatility.
    pub fields: Vec<FieldExpectation>,
}

impl FieldManifest {
    /// An empty manifest for a backend version.
    pub fn new(backend: impl Into<String>, for_version: Version) -> Self {
        Self {
            backend: backend.into(),
            for_version,
            entities: Vec::new(),
            fields: Vec::new(),
        }
    }

    /// Check internal consistency before trusting this manifest.
    ///
    /// Rejects duplicate `(entity, path)` field expectations, duplicate
    /// entity expectations, empty paths, and contradictory Realm handling.
    /// The engine refuses to verify against an invalid manifest, and the
    /// contract suite calls this explicitly in adapters.
    pub fn validate(&self) -> Result<()> {
        if self.backend.trim().is_empty() {
            return Err(Error::InvalidManifest(
                "backend must be non-empty".to_string(),
            ));
        }
        let mut seen: BTreeMap<EntityKind, BTreeMap<&str, ()>> = BTreeMap::new();
        for fe in &self.fields {
            if fe.path.trim().is_empty() {
                return Err(Error::InvalidManifest(format!(
                    "empty field path for entity {}",
                    super::entity_label(fe.entity)
                )));
            }
            if seen
                .entry(fe.entity)
                .or_default()
                .insert(fe.path.as_str(), ())
                .is_some()
            {
                return Err(Error::InvalidManifest(format!(
                    "duplicate field expectation {} for entity {}",
                    fe.path,
                    super::entity_label(fe.entity)
                )));
            }
        }
        let mut entities = BTreeMap::new();
        for ee in &self.entities {
            if entities.insert(ee.entity, ee.presence).is_some() {
                return Err(Error::InvalidManifest(format!(
                    "duplicate entity expectation for {}",
                    super::entity_label(ee.entity)
                )));
            }
        }
        if let Some(EntityPresence::Optional) = entities.get(&EntityKind::Realm) {
            return Err(Error::InvalidManifest(
                "Realm must always be present and cannot be declared optional".to_string(),
            ));
        }
        Ok(())
    }
}

use std::fmt;

impl fmt::Display for FieldManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FieldManifest({} @ {}, {} entities, {} fields)",
            self.backend,
            self.for_version,
            self.entities.len(),
            self.fields.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_for(backend: &str) -> FieldManifest {
        FieldManifest::new(backend, Version::new(26, 0, 0))
    }

    #[test]
    fn manifest_validate_rejects_duplicates() {
        let mut manifest = manifest_for("stub");
        let expectation = || FieldExpectation {
            entity: EntityKind::Client,
            path: "client_id".into(),
            required: true,
            volatile: false,
            introduced_in: Version::new(1, 0, 0),
        };
        manifest.fields.push(expectation());
        manifest.fields.push(expectation());
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn manifest_validate_rejects_optional_realm() {
        let mut manifest = manifest_for("stub");
        manifest.entities.push(EntityExpectation::new(
            EntityKind::Realm,
            EntityPresence::Optional,
        ));
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn manifest_validate_rejects_empty_path() {
        let mut manifest = manifest_for("stub");
        manifest.fields.push(FieldExpectation {
            entity: EntityKind::Realm,
            path: "".into(),
            required: true,
            volatile: false,
            introduced_in: Version::new(1, 0, 0),
        });
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn manifest_without_entities_is_valid() {
        let manifest = manifest_for("stub");
        assert!(manifest.validate().is_ok());
    }
}
