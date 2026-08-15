//! Representative sample data for exercising the contract suite and for
//! adapters to bridge their own extraction into the canonical shape.

use crate::model::{BackendInfo, CanonicalRealm, Client, EntityKind, User};
use crate::{EntityExpectation, EntityPresence, FieldExpectation, FieldManifest, Version};

/// A small, representative extraction used to exercise this suite (and useful
/// to adapters for bridging their own data into the canonical shape).
pub fn sample_realm() -> CanonicalRealm {
    let mut realm = CanonicalRealm {
        backend: BackendInfo {
            backend: "sample".to_string(),
            detected_version: Version::new(1, 0, 0),
            extra: Default::default(),
        },
        realm: crate::model::Realm {
            name: "sample".to_string(),
            enabled: true,
            native: serde_json::json!({ "registrationAllowed": false }),
        },
        ..CanonicalRealm::default()
    };

    realm.clients.push(Client {
        id: "a".into(),
        client_id: "api".into(),
        enabled: true,
        native: serde_json::json!({ "protocol": "openid-connect" }),
    });
    realm.clients.push(Client {
        id: "b".into(),
        client_id: "web".into(),
        enabled: true,
        native: serde_json::json!({ "protocol": "openid-connect", "secret": "s3cr3t" }),
    });

    realm.users.push(User {
        id: "u1".into(),
        username: "alice".into(),
        enabled: true,
        native: serde_json::json!({ "email": "alice@example.com" }),
    });
    realm.volatile.insert(
        EntityKind::User,
        "u1",
        "native.last_login",
        serde_json::json!("2026-08-15T10:00:00Z"),
    );
    realm
}

/// A manifest that matches [`sample_realm`] and passes verification.
pub fn sample_manifest() -> FieldManifest {
    let mut manifest = FieldManifest::new("sample", Version::new(1, 0, 0));
    manifest.entities.push(EntityExpectation::new(
        EntityKind::Client,
        EntityPresence::Optional,
    ));
    manifest.entities.push(EntityExpectation::new(
        EntityKind::User,
        EntityPresence::Optional,
    ));

    let expectation =
        |entity: EntityKind, path: &str, required: bool, volatile: bool| FieldExpectation {
            entity,
            path: path.to_string(),
            required,
            volatile,
            introduced_in: Version::new(1, 0, 0),
        };
    manifest
        .fields
        .push(expectation(EntityKind::Client, "client_id", true, false));
    manifest
        .fields
        .push(expectation(EntityKind::Client, "enabled", true, false));
    manifest.fields.push(expectation(
        EntityKind::Client,
        "native.protocol",
        true,
        false,
    ));
    manifest
        .fields
        .push(expectation(EntityKind::User, "username", true, false));
    manifest
        .fields
        .push(expectation(EntityKind::User, "native.email", false, false));
    manifest.fields.push(expectation(
        EntityKind::User,
        "native.last_login",
        false,
        true,
    ));
    manifest
}
