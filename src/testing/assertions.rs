//! The concrete assertions an adapter's contract tests call.

use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

use crate::canonical::roundtrip;
use crate::verify::{entity_label, rows_by_entity};
use crate::{CanonicalRealm, FieldManifest, ManifestVerifier, Severity, resolve_path};

/// Assert that canonical serialization is byte-stable: two serializations of
/// identical input must produce identical bytes.
///
/// Run this on every collection an adapter builds — a value built the same
/// way must hash the same way.
#[track_caller]
pub fn assert_canonical_deterministic<T: Serialize>(value: &T) {
    let a = crate::canonical_bytes(value).expect("canonical serialization (#1)");
    let b = crate::canonical_bytes(value).expect("canonical serialization (#2)");
    assert_eq!(
        a, b,
        "canonical serialization produced different bytes for identical input"
    );
}

/// Assert that canonical bytes deserialize back to a semantically identical
/// value — the envelope, each entity, and the volatile section must all
/// survive the roundtrip.
#[track_caller]
pub fn assert_serialization_roundtrip<T: Serialize + DeserializeOwned + PartialEq + Debug>(
    value: &T,
) {
    roundtrip(value).expect("value must survive a canonical serialization roundtrip");
}

/// Assert the volatile-segregation contract: every manifest-volatile field
/// must appear in the volatile section for every existing row, and must not
/// leak into the config body.
#[track_caller]
pub fn assert_volatile_segregation(realm: &CanonicalRealm, manifest: &FieldManifest) {
    manifest
        .validate()
        .expect("manifest must be self-consistent before segregation is checked");
    let rows = rows_by_entity(realm);
    for fe in manifest.fields.iter().filter(|f| f.volatile) {
        let Some(entity_rows) = rows.get(&fe.entity) else {
            continue;
        };
        for (row_id, row) in entity_rows {
            assert!(
                !resolve_path(row, &fe.path),
                "volatile field '{}' must not appear in the config body of {} '{}'",
                fe.path,
                entity_label(fe.entity),
                row_id
            );
            assert!(
                realm.volatile.get(fe.entity, row_id, &fe.path).is_some(),
                "volatile field '{}' must be captured in the volatile section for {} '{}'",
                fe.path,
                entity_label(fe.entity),
                row_id
            );
        }
    }
}

/// Assert a manifest passes its own self-consistency checks (unique
/// expectations, sane backend identity, valid paths).
#[track_caller]
pub fn assert_manifest_valid(manifest: &FieldManifest) {
    manifest
        .validate()
        .expect("manifest failed self-consistency checks");
}

/// Assert an extraction verifies against its manifest with zero
/// `Error`-severity findings.
#[track_caller]
pub fn assert_verification_complete(realm: &CanonicalRealm, manifest: &FieldManifest) {
    let report = ManifestVerifier
        .verify(realm, manifest)
        .expect("verification ran without error");
    let errors: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "verification found {} error(s): {:#?}",
        errors.len(),
        errors
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::sample::{sample_manifest, sample_realm};

    #[test]
    fn sample_data_passes_the_whole_suite() {
        let realm = sample_realm();
        let manifest = sample_manifest();

        assert_manifest_valid(&manifest);
        assert_canonical_deterministic(&realm);
        assert_serialization_roundtrip(&realm);
        assert_serialization_roundtrip(&manifest);
        assert_volatile_segregation(&realm, &manifest);
        assert_verification_complete(&realm, &manifest);
    }

    #[test]
    fn segregation_contract_catches_leaks() {
        let mut realm = sample_realm();
        // Re-introduce the volatile field into the config body.
        realm.users[0].native["last_login"] = serde_json::json!("2026-08-15T10:00:00Z");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_volatile_segregation(&realm, &sample_manifest());
        }));
        assert!(
            result.is_err(),
            "volatile leak should trip the contract test"
        );
    }
}
