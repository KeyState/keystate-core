//! The `Group` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A group.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Group {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The group's path/name — its identity key.
    #[serde(default)]
    pub name: String,
    /// Backend-faithful representation.
    #[serde(default)]
    pub native: Value,
}
