//! The `IdentityProvider` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An external identity provider.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct IdentityProvider {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The provider's alias — its identity key.
    #[serde(default)]
    pub alias: String,
    /// Backend-faithful representation.
    #[serde(default)]
    pub native: Value,
}
