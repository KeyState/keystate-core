//! The `Client` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A client (application).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Client {
    /// Stable internal identifier — the sort key for deterministic output.
    #[serde(default)]
    pub id: String,
    /// The public client identifier — the cross-backend identity key.
    #[serde(default)]
    pub client_id: String,
    /// Whether the client is active.
    #[serde(default)]
    pub enabled: bool,
    /// Backend-faithful representation, including protocol, secrets and any
    /// field the canonical model has not formalized.
    #[serde(default)]
    pub native: Value,
}
