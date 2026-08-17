# keystate-core

[![CI](https://github.com/KeyState/keystate-core/actions/workflows/ci.yml/badge.svg)](https://github.com/KeyState/keystate-core/actions/workflows/ci.yml)

Canonical domain model, extraction ports, completeness verification, and
idempotent output for **Keystate** — a read-only tool that pulls the complete,
verified configuration state of an IAM system out of its database as a single
backend-agnostic JSON document.

`keystate-core` is the contract crate. It is backend-agnostic by design and
carries **no tokio and no sqlx dependency**. Adapters (separate repos) depend
on it, implement its ports, and are composed by `keystate-cli`.

> Full design: [`ARCHITECTURE.md`](ARCHITECTURE.md) · workflow:
> [`DEVELOPMENT.md`](DEVELOPMENT.md) · releases: [`RELEASE.md`](RELEASE.md)

## Module map

| Module | Responsibility |
|---|---|
| `model/` | Canonical entities (one file per entity) + extraction envelope |
| `port/` | The traits adapters implement: `Extractor`, `CompletenessVerifier`, `OutputSink` (one file per port) |
| `verify/` | Manifests, reports, the shared verification engine, path resolution |
| `canonical/` | Byte-stable serialization, the config/volatile split, content hash |
| `testing/` | Shared contract-test suite every adapter runs in CI |
| `error/` · `version/` | Cross-cutting types |

## The model

Every entity carries a thin set of **common** fields — enough for cross-backend
comparison, reporting and audit, including the canonical identity key — plus a
single open escape hatch:

```rust
pub struct Client {
    pub id: String,        // stable sort key
    pub client_id: String, // cross-backend identity key
    pub enabled: bool,
    pub native: serde_json::Value, // backend-faithful representation, adapter-owned
}
```

Common fields are the *rule*, not a per-adapter judgment. Once a field shows
up in `native` for two or more backends it becomes a promotion candidate into
common (additive minor bump) — see `ARCHITECTURE.md` §3.1. There is no second
`extra` field.

## Quick use

```rust
use keystate_core::{
    FieldExpectation, FieldManifest, ManifestVerifier, EntityKind, Version,
};

let mut manifest = FieldManifest::new("keycloak", Version::new(26, 0, 0));
manifest.fields.push(FieldExpectation {
    entity: EntityKind::Client,
    path: "native.secret".into(),
    required: true,
    volatile: false,
    introduced_in: Version::new(1, 0, 0),
});

let report = ManifestVerifier.verify(&realm, &manifest)?;
assert!(report.is_complete());
```

An adapter implements `Extractor`, `CompletenessVerifier::expected_fields`, and
optionally picks an `OutputSink`. The completeness claim is part of the output
(`VerificationReport`), and every extraction can be fingerprinted with
`content_hash(realm)` — which ignores the volatile section, so last-login
churn never appears as configuration drift.

## Semver contract

`keystate-core` is what every adapter pins against:

- **Major** — a common-model field/struct removed or changed incompatibly.
- **Minor** — a common field/struct added additively (including promotions).
- **Patch** — bug fix, no model change.

Promotions carry a release-train obligation (all live adapters move together
before any CLI tags) so version skew is never misread as drift. See
`RELEASE.md`.

## Testing

`cargo test` runs unit tests for every module. Adapters additionally import
the [`testing`](src/testing) contract suite (determinism roundtrip, volatile
segregation, manifest self-consistency, verification completeness) and pass
their own integration tests against real fixture databases with `testcontainers`.