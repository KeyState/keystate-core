//! Completeness verification: manifests, reports, the shared engine, and the
//! path resolver.
//!
//! Every adapter ships a [`FieldManifest`] derived from its backend's own
//! schema migration history (Liquibase changelogs for Keycloak). The shared
//! [`ManifestVerifier`] engine checks an extraction against that manifest and
//! produces a [`VerificationReport`] that is part of the output — never a
//! side log. The completeness claim is backed by evidence every time it runs.
//!
//! Each concern lives in its own module so the engine can stay generic while
//! the data shapes it works on stay separately documented and testable.

mod engine;
mod manifest;
mod path;
mod report;

pub use engine::ManifestVerifier;
pub use manifest::{EntityExpectation, EntityPresence, FieldExpectation, FieldManifest};
pub use path::resolve_path;
pub use report::{IssueKind, Severity, VerificationIssue, VerificationReport};

pub(crate) use engine::{entity_label, rows_by_entity};
