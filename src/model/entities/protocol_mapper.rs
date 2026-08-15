//! The `ProtocolMapper` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A protocol mapper.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProtocolMapper {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The mapper's name — its identity key.
    #[serde(default)]
    pub name: String,
    /// Backend-faithful representation.
    #[serde(default)]
    pub native: Value,
}
