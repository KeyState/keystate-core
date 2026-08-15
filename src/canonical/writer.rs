//! The canonical JSON writer.

use serde::Serialize;

use crate::{Error, Result};

/// Serialize `value` to deterministic, byte-stable compact JSON.
///
/// Object keys are sorted (via the BTreeMap-backed serde_json map), floats
/// use serde_json's fixed shortest-round-trip formatting, and no incidental
/// whitespace is emitted. Identical input always yields identical bytes.
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let staged = serde_json::to_value(value).map_err(Error::from)?;
    serde_json::to_vec(&staged).map_err(Error::from)
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
}
