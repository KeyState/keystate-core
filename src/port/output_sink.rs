//! The output port.

use crate::{CanonicalRealm, Result, VerificationReport};

/// Writes an extraction and its completeness report somewhere durable.
///
/// Ships with a local-file sink first (encrypted-at-rest by default), with
/// room for an S3/object-storage sink later without touching extraction logic.
pub trait OutputSink: Send + Sync {
    /// Persist an extraction and its verification report.
    fn write(&self, canonical: &CanonicalRealm, report: &VerificationReport) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopSink;

    impl OutputSink for NoopSink {
        fn write(&self, _canonical: &CanonicalRealm, _report: &VerificationReport) -> Result<()> {
            Ok(())
        }
    }

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn sinks_are_send_and_sync() {
        assert_send_sync::<NoopSink>();
    }
}
