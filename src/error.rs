//! Error and result types shared across the crate and its ports.

/// The error type used by every port in `keystate-core`.
///
/// Adapters and sinks wrap their own failures with [`Error::Other`] (for
/// typed errors) or [`Error::Message`] (for simple strings). Successful
/// composition never requires core to know a backend's error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A JSON (de)serialization failure.
    #[error("JSON (de)serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// A backend/schema version string could not be parsed.
    #[error("invalid version string {input:?}: {reason}")]
    InvalidVersion {
        /// The string that failed to parse.
        input: String,
        /// Why parsing failed.
        reason: String,
    },

    /// A [`FieldManifest`](crate::FieldManifest) failed its consistency
    /// checks and cannot be trusted for verification.
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),

    /// A plain-message error, for cases that do not warrant a typed variant.
    #[error("{0}")]
    Message(String),

    /// Any other failure coming from an adapter or sink implementation.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Convenience alias used across `keystate-core` and implemented by adapters.
pub type Result<T> = std::result::Result<T, Error>;
