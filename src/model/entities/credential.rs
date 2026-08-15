//! The `Credential` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A credential record owned by a user.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Credential {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The credential type (e.g. `password`, `otp`) — its identity key.
    #[serde(default)]
    pub r#type: String,
    /// Backend-faithful representation. Credential hashes deliberately
    /// mirror the backend's stored form; extraction must never mask them.
    #[serde(default)]
    pub native: Value,
}
