//! The canonical JSON writer.

use serde::Serialize;

use crate::{Error, Result};

/// Serialize `value` to deterministic, byte-stable compact JSON.
///
/// Object keys are sorted **explicitly**, recursively, by this function —
/// never by relying on the incidental map backing. That guarantee holds even
/// if a crate anywhere in the dependency graph enables serde_json's
/// `preserve_order` feature, which would otherwise switch object maps from
/// the sorted `BTreeMap` backing to an insertion-ordered `IndexMap` and
/// silently change bytes (Cargo unifies features per build). Array order is
/// preserved as-is: some collections carry meaningful order and must not be
/// normalized away. Floats use serde_json's fixed shortest-round-trip
/// formatting, and no incidental whitespace is emitted. Identical input
/// always yields identical bytes.
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let staged = serde_json::to_value(value).map_err(Error::from)?;
    let sorted = sort_keys(staged);
    serde_json::to_vec(&sorted).map_err(Error::from)
}

/// Recursively sort every object's keys, preserving array order.
fn sort_keys(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries: Vec<(String, serde_json::Value)> = map
                .into_iter()
                .map(|(key, value)| (key, sort_keys(value)))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            serde_json::Map::from_iter(entries).into()
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(sort_keys).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn object_keys_are_sorted_in_serialized_output() {
        let v = json!({ "z": 1, "a": 2, "m": 3 });
        let bytes = canonical_bytes(&v).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn struct_field_order_does_not_change_bytes() {
        #[derive(serde::Serialize)]
        struct Wide {
            a: u8,
            c: String,
            b: u8,
        }
        #[derive(serde::Serialize)]
        struct Narrow {
            a: u8,
            b: u8,
            c: String,
        }
        assert_eq!(
            canonical_bytes(&Wide {
                a: 1,
                b: 2,
                c: "x".into(),
            })
            .unwrap(),
            canonical_bytes(&Narrow {
                a: 1,
                b: 2,
                c: "x".into(),
            })
            .unwrap()
        );
    }

    #[test]
    fn identical_input_produces_identical_bytes() {
        let value = json!({"native": {"b": 1, "a": "x"}});
        assert_eq!(
            canonical_bytes(&value).unwrap(),
            canonical_bytes(&value).unwrap()
        );
    }

    #[test]
    fn nested_object_keys_are_sorted_recursively() {
        let v = json!({
            "outer": {"z": 1, "mid": {"y": 2, "a": 3}, "b": 4},
            "top": 0
        });
        let bytes = canonical_bytes(&v).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            r#"{"outer":{"b":4,"mid":{"a":3,"y":2},"z":1},"top":0}"#
        );
    }

    #[test]
    fn array_order_is_preserved_not_sorted() {
        let v = json!({"list": [{"z": 1}, {"a": 2}], "items": [3, 1, 2]});
        let bytes = canonical_bytes(&v).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            r#"{"items":[3,1,2],"list":[{"z":1},{"a":2}]}"#
        );
    }
}
