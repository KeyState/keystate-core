//! The `User` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A user account.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct User {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The username — the cross-backend identity key.
    #[serde(default)]
    pub username: String,
    /// Whether the user is active.
    #[serde(default)]
    pub enabled: bool,
    /// Backend-faithful representation.
    #[serde(default)]
    pub native: Value,
}
