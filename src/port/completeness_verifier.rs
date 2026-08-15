//! The completeness-verification port.

use crate::{CanonicalRealm, FieldManifest, VerificationReport, Version, verify::ManifestVerifier};

/// Supplies the completeness contract for a backend version and checks an
/// extraction against it.
///
/// Adapters implement [`expected_fields`](Self::expected_fields), deriving the
/// manifest from the backend's own schema migration history (Liquibase
/// changelogs for Keycloak). [`verify`](Self::verify) has a default
/// implementation backed by the shared [`ManifestVerifier`] engine in core —
/// an adapter only overrides it when its backend needs semantics beyond
/// field-presence checking.
pub trait CompletenessVerifier: Send + Sync {
    /// The fields a given backend schema version is expected to carry.
    fn expected_fields(&self, backend_version: &Version) -> FieldManifest;

    /// Turn an extraction plus its manifest into a report. The report is part
    /// of the output, not a side log.
    ///
    /// The default implementation runs the shared engine. It panics on a
    /// manifest that fails its own consistency checks — that is an adapter
    /// bug, caught in CI via the contract suite, never something to report to
    /// an end user as an incomplete realm.
    fn verify(&self, extracted: &CanonicalRealm, manifest: &FieldManifest) -> VerificationReport {
        ManifestVerifier
            .verify(extracted, manifest)
            .expect("adapter manifest failed self-consistency checks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Verifier;

    impl CompletenessVerifier for Verifier {
        fn expected_fields(&self, _v: &Version) -> FieldManifest {
            FieldManifest::new("stub", Version::new(1, 0, 0))
        }
    }

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn verifiers_are_send_and_sync() {
        assert_send_sync::<Verifier>();
    }

    #[test]
    fn default_verify_runs_and_passes_on_default_expansion() {
        let verifier = Verifier;
        let realm = CanonicalRealm::default();
        let manifest = verifier.expected_fields(&Version::new(1, 0, 0));
        let report = verifier.verify(&realm, &manifest);
        assert!(report.is_complete());
    }
}
