//! The `AuthFlow` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An authentication flow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AuthFlow {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The flow's alias — its identity key.
    #[serde(default)]
    pub alias: String,
    /// Backend-faithful representation. Execution-step ordering is
    /// meaningful and must be preserved as-is, not re-sorted.
    #[serde(default)]
    pub native: Value,
}
