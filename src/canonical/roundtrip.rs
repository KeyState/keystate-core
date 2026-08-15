//! Lossless roundtrip checks for the contract suite.

use std::fmt::Debug;

use serde::{Serialize, de::DeserializeOwned};

use crate::canonical::writer::canonical_bytes;
use crate::{Error, Result};

/// Serialization roundtrip helper used by the contract suite: the canonical
/// bytes must deserialize back to a semantically identical value.
pub fn roundtrip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) -> Result<()> {
    let bytes = canonical_bytes(value)?;
    let back: T = serde_json::from_slice(&bytes).map_err(Error::from)?;
    if &back != value {
        return Err(Error::Message(
            "canonical serialization roundtrip changed the value".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn roundtrip_is_lossless_for_values() {
        let value = json!({ "native": { "b": 1, "a": ["x", "y"] } });
        roundtrip(&value).unwrap();
    }
}
