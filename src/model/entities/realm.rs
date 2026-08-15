//! The `Realm` entity.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A realm: the top-level configuration container.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Realm {
    /// The realm's identity key: its name.
    #[serde(default)]
    pub name: String,
    /// Whether the realm is active.
    #[serde(default)]
    pub enabled: bool,
    /// Backend-faithful representation of this realm and its settings.
    #[serde(default)]
    pub native: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realm_serializes_common_and_native() {
        let realm = Realm {
            name: "master".into(),
            enabled: true,
            native: serde_json::json!({ "registrationAllowed": false }),
        };
        let value = serde_json::to_value(&realm).unwrap();
        assert_eq!(value["name"], "master");
        assert_eq!(value["enabled"], true);
        assert_eq!(value["native"]["registrationAllowed"], false);
    }
}
