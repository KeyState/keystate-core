//! Deterministic serialization and content hashing.
//!
//! The guarantee: running an extraction twice against an unchanged source
//! produces byte-identical output, so a git diff on the output reflects real
//! configuration change, never extraction noise.
//!
//! Object keys are sorted **explicitly** by the canonical writer (recursively,
//! independently of the map backing), so the guarantee does not depend on
//! serde_json's `preserve_order` feature being disabled everywhere — Cargo
//! unifies features per build, and any crate could flip it. The content hash
//! covers only the *config* portion — the volatile section is excluded by
//! construction, so last-login churn cannot pollute drift detection.
//!
//! One concern per module: the canonical writer, the config-portion split,
//! the fingerprint, and the lossless roundtrip check.

mod config;
mod hash;
mod roundtrip;
mod writer;

pub use config::config_bytes;
pub use hash::content_hash;
pub use roundtrip::roundtrip;
pub use writer::canonical_bytes;
