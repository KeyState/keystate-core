//! # keystate-core
//!
//! Canonical domain model, extraction ports, completeness verification, and
//! idempotent output for **Keystate** — a read-only tool that pulls the
//! complete, verified configuration state of an IAM system out of its
//! database as a single backend-agnostic JSON document.
//!
//! Core is deliberately **backend-agnostic** and carries no runtime or
//! database dependency. Adapters (own repos) implement the [`Extractor`],
//! [`CompletenessVerifier`] and [`OutputSink`] ports against one backend's
//! schema. The canonical structs are a thin set of *common* fields plus an
//! open `native` escape hatch, per the architecture document:
//!
//! ```text
//! {
//!   "id": "a1b2c3d4",
//!   "name": "my-client",
//!   "enabled": true,
//!   "native": { /* backend-faithful representation, adapter-owned */ }
//! }
//! ```
//!
//! The three guarantees this crate provides are:
//!
//! 1. **Completeness** — a [`FieldManifest`] describing every field a backend
//!    version must carry, checked by the [`ManifestVerifier`] into a
//!    [`VerificationReport`] that is part of the output, not a side log.
//! 2. **Idempotency** — [`canonical_bytes`] produces byte-stable output and
//!    [`content_hash`] fingerprints the *config* portion only, so a git diff
//!    on the output reflects real configuration change, never extraction
//!    noise or volatile churn.
//! 3. **Testability** — the [`testing`] module is a shared contract suite
//!    every adapter runs as part of CI.
//!
//! See `ARCHITECTURE.md` in this repository for the full design.
//!
//! # Module map
//!
//! - [`model`] — canonical entities and the extraction envelope.
//! - [`port`] — the traits adapters implement.
//! - [`verify`] — manifests, reports, the verification engine, path resolution.
//! - [`canonical`] — deterministic serialization and content hashing.
//! - [`error`], [`version`] — cross-cutting types.

#![warn(missing_docs)]

pub mod canonical;
pub mod error;
pub mod model;
pub mod port;
pub mod testing;
pub mod verify;
pub mod version;

pub use canonical::{canonical_bytes, config_bytes, content_hash, roundtrip};
pub use error::{Error, Result};
pub use model::*;
pub use port::{CompletenessVerifier, Extractor, OutputSink};
pub use verify::{
    EntityExpectation, EntityPresence, FieldExpectation, FieldManifest, IssueKind,
    ManifestVerifier, Severity, VerificationIssue, VerificationReport, resolve_path,
};
pub use version::Version;
