//! The ports (traits) every backend adapter implements.
//!
//! Core defines what the system needs, knowing nothing about how any backend
//! provides it. Adapters are concrete implementations of these traits against
//! one backend's database. The CLI never talks to a database directly — it
//! only calls these methods, and (as composition root) wires concrete adapter
//! types together.
//!
//! One file per port: adding a capability to the architecture means adding a
//! port here, not widening the ones that exist.

mod completeness_verifier;
mod extractor;
mod output_sink;

pub use completeness_verifier::CompletenessVerifier;
pub use extractor::Extractor;
pub use output_sink::OutputSink;
