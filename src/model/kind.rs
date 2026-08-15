//! The vocabulary of canonical entity kinds.

use serde::{Deserialize, Serialize};

/// The kinds of entity the canonical model can represent.
///
/// This is the fixed vocabulary the completeness manifests and the volatile
/// section are keyed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    /// A realm: the top-level configuration container.
    Realm,
    /// An OAuth/OIDC or SAML client (or "application").
    Client,
    /// A reusable bundle of provider configuration (Keycloak client scopes).
    ClientScope,
    /// A group.
    Group,
    /// A role.
    Role,
    /// A user account.
    User,
    /// A credential record owned by a user.
    Credential,
    /// A protocol mapper.
    ProtocolMapper,
    /// An authentication flow (Keycloak: a flow with execution steps).
    AuthFlow,
    /// An external identity provider.
    IdentityProvider,
}

impl EntityKind {
    /// The common field that holds this entity's row identity key.
    ///
    /// Used by the completeness verifier to key volatile records. `Realm` is
    /// identified by its `name`; every other entity by its `id`.
    pub const fn id_field(self) -> &'static str {
        match self {
            EntityKind::Realm => "name",
            _ => "id",
        }
    }

    /// The human-facing identity key this entity carries in common.
    pub const fn identity_key(self) -> &'static str {
        match self {
            EntityKind::Realm => "name",
            EntityKind::Client => "client_id",
            EntityKind::ClientScope => "name",
            EntityKind::Group => "name",
            EntityKind::Role => "id",
            EntityKind::User => "username",
            EntityKind::Credential => "type",
            EntityKind::ProtocolMapper => "name",
            EntityKind::AuthFlow => "alias",
            EntityKind::IdentityProvider => "alias",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_entity_carries_its_identity_key() {
        assert_eq!(EntityKind::Client.identity_key(), "client_id");
        assert_eq!(EntityKind::User.identity_key(), "username");
        assert_eq!(EntityKind::AuthFlow.identity_key(), "alias");
        assert_eq!(EntityKind::Realm.identity_key(), "name");
    }

    #[test]
    fn id_field_falls_back_to_name_for_realm() {
        assert_eq!(EntityKind::Client.id_field(), "id");
        assert_eq!(EntityKind::Realm.id_field(), "name");
    }
}
