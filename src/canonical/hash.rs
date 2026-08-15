//! Content fingerprinting.

use sha2::{Digest, Sha256};

use crate::canonical::config::config_bytes;
use crate::{CanonicalRealm, Result};

/// SHA-256 fingerprint of the config portion of an extraction.
///
/// Two runs with the same hash are provably identical in configuration. The
/// volatile section is excluded, so volatile churn between runs never changes
/// the hash.
pub fn content_hash(realm: &CanonicalRealm) -> Result<[u8; 32]> {
    let bytes = config_bytes(realm)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BackendInfo, CanonicalRealm, Client, EntityKind, Realm};
    use crate::version::Version;
    use serde_json::json;

    fn realm() -> CanonicalRealm {
        let mut realm = CanonicalRealm {
            backend: BackendInfo {
                backend: "stub".into(),
                detected_version: Version::new(1, 0, 0),
                extra: Default::default(),
            },
            realm: Realm {
                name: "master".into(),
                enabled: true,
                native: json!({ "registrationAllowed": false }),
            },
            ..CanonicalRealm::default()
        };
        realm.clients.push(Client {
            id: "a".into(),
            client_id: "api".into(),
            enabled: true,
            native: json!({ "publicClient": true }),
        });
        realm
    }

    #[test]
    fn hash_is_stable_for_identical_config() {
        assert_eq!(
            content_hash(&realm()).unwrap(),
            content_hash(&realm()).unwrap()
        );
    }

    #[test]
    fn hash_ignores_volatile_churn() {
        let a = realm();
        let mut b = realm();
        b.volatile.insert(
            EntityKind::User,
            "u1",
            "native.last_login",
            json!("2026-08-15T10:00:00Z"),
        );
        assert_eq!(content_hash(&a).unwrap(), content_hash(&b).unwrap());
    }

    #[test]
    fn hash_changes_when_config_changes() {
        let a = realm();
        let mut b = realm();
        b.clients[0].enabled = false;
        assert_ne!(content_hash(&a).unwrap(), content_hash(&b).unwrap());
    }

    #[test]
    fn hash_is_always_32_bytes() {
        assert_eq!(content_hash(&realm()).unwrap().len(), 32);
    }
}
