//! Shared contract-test suite.
//!
//! Every adapter imports and runs this suite against its own implementation
//! as part of CI. It enforces the parts of the port contract that need no
//! live database — determinism of canonical serialization, lossless
//! roundtrips, volatile segregation, manifest honesty, and completeness of a
//! constructed extraction.
//!
//! Adapters also carry their own integration and determinism tests against a
//! real fixture database (see `ARCHITECTURE.md` §7); this module covers what
//! core can prove without one.

mod assertions;
mod sample;

pub use assertions::{
    assert_canonical_deterministic, assert_manifest_valid, assert_serialization_roundtrip,
    assert_verification_complete, assert_volatile_segregation,
};
pub use sample::{sample_manifest, sample_realm};
