//! Backend/schema version representation used to key manifests and reports.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A simple `major.minor.patch` version.
///
/// Used to key [`FieldManifest`](crate::FieldManifest)s and
/// [`VerificationReport`](crate::VerificationReport)s against the schema
/// version actually detected at the source, so completeness claims are always
/// tied to a concrete backend version.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub struct Version {
    /// Major schema version.
    pub major: u64,
    /// Minor schema version.
    pub minor: u64,
    /// Patch version.
    pub patch: u64,
}

impl Version {
    /// Construct a version from its three components.
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(Error::InvalidVersion {
                input: input.to_string(),
                reason: format!(
                    "expected major.minor.patch, got {} component(s)",
                    parts.len()
                ),
            });
        }
        let parse = |part: &str| -> Option<u64> { part.parse::<u64>().ok() };
        let (major, minor, patch) = (parse(parts[0]), parse(parts[1]), parse(parts[2]));
        let (Some(major), Some(minor), Some(patch)) = (major, minor, patch) else {
            return Err(Error::InvalidVersion {
                input: input.to_string(),
                reason: "each component must be a non-negative integer".to_string(),
            });
        };
        Ok(Version::new(major, minor, patch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_three_part_versions() {
        assert_eq!("26.0.4".parse::<Version>().unwrap(), Version::new(26, 0, 4));
        assert_eq!("0.1.0".parse::<Version>().unwrap(), Version::new(0, 1, 0));
        assert!("26.0".parse::<Version>().is_err());
        assert!("26.0.beta".parse::<Version>().is_err());
        assert!("".parse::<Version>().is_err());
    }

    #[test]
    fn orders_lexically_by_component() {
        assert!(Version::new(1, 10, 0) > Version::new(1, 9, 9));
        assert!(Version::new(25, 0, 0) < Version::new(26, 0, 0));
    }

    #[test]
    fn displays_and_roundtrips() {
        let v = Version::new(24, 8, 1);
        assert_eq!(v.to_string(), "24.8.1");
        assert_eq!(v.to_string().parse::<Version>().unwrap(), v);
    }
}
