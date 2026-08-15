//! Path resolution for field expectations.
//!
//! A manifest path is a dotted path against an entity's serialized row. Array
//! segments end in `[]` and match when *any* element carries the remainder.

use serde_json::Value;

/// Resolve a dotted path against a JSON value. Returns whether the path is
/// present.
///
/// Supported path shapes:
/// - `client_id` — a top-level common field.
/// - `native.secret` — a field inside the native subtree.
/// - `native.alternativeNames[]` — an array field present as a key.
/// - `native.executionSteps[].name` — a field present on array elements.
pub fn resolve_path(value: &Value, path: &str) -> bool {
    let segments: Vec<&str> = path.split('.').collect();
    resolve_segments(value, &segments)
}

fn resolve_segments(value: &Value, segments: &[&str]) -> bool {
    let Some((head, rest)) = segments.split_first() else {
        return true;
    };
    if let Some(key) = head.strip_suffix("[]") {
        // `key[]` — the current value must be an object whose `key` is an
        // array, and at least one element must carry the remainder.
        let Value::Object(map) = value else {
            return false;
        };
        let Some(Value::Array(elements)) = map.get(key) else {
            return false;
        };
        if elements.is_empty() {
            return false;
        }
        elements
            .iter()
            .any(|element| resolve_segments(element, rest))
    } else {
        let Value::Object(map) = value else {
            return false;
        };
        map.get(*head)
            .map(|next| resolve_segments(next, rest))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolves_common_native_and_array_paths() {
        let value: Value = json!({
            "client_id": "web",
            "native": {
                "secret": "s3cr3t",
                "alternativeNames": ["alpha", "beta"],
                "steps": [
                    {"name": "Step A", "priority": 1},
                    {"name": "Step B", "priority": 2}
                ]
            }
        });
        assert!(resolve_path(&value, "client_id"));
        assert!(resolve_path(&value, "native.secret"));
        assert!(resolve_path(&value, "native.alternativeNames[]"));
        assert!(resolve_path(&value, "native.steps[].name"));
    }

    #[test]
    fn does_not_report_absent_or_empty_matches() {
        let value: Value = json!({
            "native": {
                "steps": [{"name": "Step A"}],
                "empty": []
            }
        });
        assert!(!resolve_path(&value, "native.nope"));
        assert!(!resolve_path(&value, "native.steps[].nope"));
        assert!(!resolve_path(&value, "native.empty[]"));
        assert!(!resolve_path(&value, ""));
        assert!(resolve_path(&value, "native"));
    }

    #[test]
    fn null_or_scalar_rows_never_resolve() {
        assert!(!resolve_path(&Value::Null, "native.x"));
        assert!(!resolve_path(&json!("text"), "native.x"));
        assert!(!resolve_path(&json!([1, 2]), "native.x"));
    }
}
