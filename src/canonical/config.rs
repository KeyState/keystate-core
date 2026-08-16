//! The config-portion split: everything except the volatile section.

use crate::{CanonicalRealm, Error, Result};

/// The deterministic bytes of a realm's **config** portion only — everything
/// except the volatile section. Content hashes cover exactly this.
///
/// Routes through [`canonical_bytes`](crate::canonical::canonical_bytes), so
/// object keys are explicitly sorted and the guarantee holds regardless of
/// any crate enabling serde_json's `preserve_order`.
pub fn config_bytes(realm: &CanonicalRealm) -> Result<Vec<u8>> {
    let mut staged = serde_json::to_value(realm).map_err(Error::from)?;
    let object = staged
        .as_object_mut()
        .ok_or_else(|| Error::Message("a CanonicalRealm must serialize to a JSON object".into()))?;
    object.remove("volatile");
    super::writer::canonical_bytes(&staged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CanonicalRealm, EntityKind};
    use crate::version::Version;

    #[test]
    fn volatile_section_never_appears_in_config_bytes() {
        let mut volatile = crate::model::VolatileSection::new();
        volatile.insert(
            EntityKind::User,
            "u1",
            "native.last_login",
            serde_json::json!("2026-08-15T10:00:00Z"),
        );
        let realm = CanonicalRealm {
            backend: crate::model::BackendInfo {
                backend: "stub".into(),
                detected_version: Version::new(1, 0, 0),
                extra: Default::default(),
            },
            volatile,
            ..CanonicalRealm::default()
        };
        let bytes = config_bytes(&realm).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("last_login"), "volatile leaked into config");
        assert!(!text.contains("volatile"), "volatile section still present");
    }

    #[test]
    fn config_bytes_are_stable_across_calls() {
        let realm = CanonicalRealm::default();
        assert_eq!(config_bytes(&realm).unwrap(), config_bytes(&realm).unwrap());
    }

    #[test]
    fn native_object_keys_are_sorted_in_config_bytes() {
        let realm = CanonicalRealm {
            backend: crate::model::BackendInfo {
                backend: "stub".into(),
                detected_version: Version::new(1, 0, 0),
                extra: Default::default(),
            },
            ..CanonicalRealm::default()
        };
        let bytes = config_bytes(&realm).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        let a = text.find("\"backend\"").expect("backend present");
        let z = text.find("\"clients\"").expect("clients present");
        assert!(a < z, "object keys must be sorted: {text}");
    }
}
