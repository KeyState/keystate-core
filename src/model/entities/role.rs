//! The `Role` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A role.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Role {
    /// Stable internal identifier — the sort key for deterministic output and
    /// the identity key where the backend has no separate human-facing name.
    #[serde(default)]
    pub id: String,
    /// The role's name.
    #[serde(default)]
    pub name: String,
    /// Backend-faithful representation.
    #[serde(default)]
    pub native: Value,
}
