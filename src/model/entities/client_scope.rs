//! The `ClientScope` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A reusable bundle of provider configuration.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ClientScope {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The scope's name — its identity key.
    #[serde(default)]
    pub name: String,
    /// Backend-faithful representation.
    #[serde(default)]
    pub native: Value,
}
