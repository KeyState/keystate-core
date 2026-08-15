//! The canonical (backend-agnostic) model.
//!
//! Every entity carries a minimal set of *common* fields — enough for
//! cross-backend comparison, reporting and audit, including the entity's
//! canonical identity key — plus a single open escape hatch,
//! `native: serde_json::Value`, which holds the backend's own faithful
//! representation, structured however actually matches that IAM's model.
//! There is deliberately no second parallel `extra` field.
//!
//! See `ARCHITECTURE.md` §3.1 for the common-vs-native rule and the
//! promotion path from native into common.

pub mod entities;
mod envelope;
mod kind;

pub use entities::{
    AuthFlow, Client, ClientScope, Credential, Group, IdentityProvider, ProtocolMapper, Realm,
    Role, User,
};
pub use envelope::{BackendInfo, CanonicalRealm, ExtractScope, VolatileSection};
pub use kind::EntityKind;

/// The type of an entity's open native escape hatch.
pub type JsonValue = serde_json::Value;
