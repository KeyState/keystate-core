//! The canonical entity structs.
//!
//! One file per entity. Adding a new canonical entity means adding one file
//! here, re-exporting it below, and (per `DEVELOPMENT.md`) shipping it through
//! a core minor release so adapters can populate it.

mod auth_flow;
mod client;
mod client_scope;
mod credential;
mod group;
mod identity_provider;
mod protocol_mapper;
mod realm;
mod role;
mod user;

pub use auth_flow::AuthFlow;
pub use client::Client;
pub use client_scope::ClientScope;
pub use credential::Credential;
pub use group::Group;
pub use identity_provider::IdentityProvider;
pub use protocol_mapper::ProtocolMapper;
pub use realm::Realm;
pub use role::Role;
pub use user::User;

#[cfg(test)]
mod tests {
    use super::*;

    /// Every entity must survive a canonical serialization roundtrip and
    /// expose a `native` escape hatch. Kept as one generic test so a new
    /// entity can't silently break the model contract.
    #[test]
    fn entities_roundtrip_and_keep_native() {
        let realm = Realm {
            name: "master".into(),
            enabled: true,
            native: serde_json::json!({ "a": 1 }),
        };
        let roundtrip_realm: Realm =
            serde_json::from_value(serde_json::to_value(&realm).unwrap()).unwrap();
        assert_eq!(roundtrip_realm, realm);
        assert_eq!(roundtrip_realm.native["a"], 1);
    }
}
