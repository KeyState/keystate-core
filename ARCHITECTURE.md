# Keystate — Architecture & Design

Language: Rust
Pattern: Hexagonal (Ports & Adapters)
Distribution: Multi-repo under a shared GitHub organization, composed at build time

This document is the canonical description of how Keystate is built. It applies
to every repo in the organization. Where a repo's own README or a signed-off
design review says otherwise, the review wins — but only until this document is
amended.

---

## 1. Repository Layout

One organization, multiple repos, each with a single clear responsibility. This
keeps ownership clean, lets adapters version and release independently of the
core, and mirrors the trust story from the project brief — anyone should be able
to audit exactly what code touches their database without wading through
unrelated adapter or CLI logic.

```
keystate/
├── keystate-core           # domain model, ports (traits), verification, output
├── keystate-adapter-keycloak
├── keystate-adapter-ferriskey
├── keystate-cli            # composition root — the actual distributable binary
└── keystate-docs           # (later) documentation site, compatibility matrices
```

`keystate-core` has no knowledge of any specific backend. Each adapter repo
depends on `keystate-core` and implements its ports. `keystate-cli` depends on
`keystate-core` and on whichever adapters it ships with, and is the only repo
that produces a distributable artifact (binary + Docker image).

This structure lets you add `keystate-adapter-<next-backend>` later without
touching core, cli, or existing adapters at all — new backends are additive,
not invasive.

## 2. Architectural Pattern: Hexagonal / Ports & Adapters

The domain core defines ports — Rust traits — that describe what the system
needs, without knowing how any backend provides it. Each adapter is a concrete
implementation of those ports for one backend's database.

```
                    ┌─────────────────────────┐
                    │      keystate-cli        │
                    │  (composition root)      │
                    └───────────┬──────────────┘
                                │ selects adapter via config
                    ┌───────────┴──────────────┐
                    │                           │
        ┌───────────▼─────────┐     ┌───────────▼─────────┐
        │ adapter-keycloak     │     │ adapter-ferriskey    │
        │ implements Extractor │     │ implements Extractor │
        └───────────┬─────────┘     └───────────┬─────────┘
                    │                           │
                    └───────────┬───────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │      keystate-core        │
                    │  Extractor trait (port)   │
                    │  CanonicalModel            │
                    │  CompletenessVerifier      │
                    │  OutputSink (port)         │
                    │  Idempotency/hashing       │
                    └────────────────────────────┘
```

This buys three things directly relevant to the project's goals:

- **Maintainability** — a bug or schema change in the Keycloak adapter cannot
  break the FerrisKey adapter or the core model. Each has its own test suite,
  its own release cadence, its own compatibility matrix against backend
  versions.
- **Modularity** — new backends are a new crate implementing one trait, not a
  fork or a set of scattered conditionals through a monolith.
- **Speed of onboarding contributors** — the port (`Extractor` trait) is the
  entire contract a new adapter needs to satisfy. It's a small, reviewable
  surface.

### 2.1 Dispatch: concrete types, not trait objects

The `Extractor` port intentionally uses native `async fn` in traits and is
**not** designed for dynamic dispatch. `Box<dyn Extractor>` will not work
ergonomically on stable Rust (as of 2026), and the trait is documented as
object-unsafe. This is a deliberate, documented choice:

- At this milestone the CLI calls a single, statically-known adapter type
  directly.
- When a second backend ships, the CLI dispatches via a small enum wrapping
  each concrete adapter type with a `match`, **not** a trait object.

This is a landmine marker, not a workaround: no `async-trait`, no manual boxing
in core, and the CLI must not assume `Box<dyn Extractor>` will drop in for free
later.

## 3. Core Domain (keystate-core)

### 3.1 Canonical model

Plain, serde-serializable Rust structs representing the backend-agnostic shape:
`Realm`, `Client`, `ClientScope`, `Group`, `Role`, `User`, `Credential`,
`ProtocolMapper`, `AuthFlow`, `IdentityProvider`. Each struct carries two kinds
of fields, and the line between them is a rule, not a judgment call made per
adapter:

- **Common fields** — the minimum needed for cross-backend comparison,
  reporting, and audit. This always includes whatever that entity's identity
  key is (a client's client ID, a user's username, a role's name) even where it
  duplicates something also present in native — the apparent redundancy in a
  single-backend view is a cheap price for an unambiguous, explicit declaration
  of which native field is the canonical identity key.
- **`native: serde_json::Value`** — the single, sole escape hatch. The full,
  faithful representation of that entity in the backend's own terms, structured
  however actually matches that IAM's real model. There is no second, parallel
  `extra` field — one mechanism for backend-specific detail, not two.

The common subset per entity:

| Entity | Common fields (identity key in bold) |
|---|---|
| `Realm` | **name**, enabled |
| `Client` | **client_id**, id, enabled |
| `ClientScope` | **name**, id |
| `Group` | **name**, id |
| `Role` | **id**, name |
| `User` | **username**, id, enabled |
| `Credential` | **type**, id |
| `ProtocolMapper` | **name**, id |
| `AuthFlow` | **alias**, id |
| `IdentityProvider` | **alias**, id |

**Promotion path.** Once a field shows up in native for two or more backends,
it's a candidate for promotion into common as an additive minor version bump in
`keystate-core`. This is how the canonical model stays meaningful over time
instead of slowly emptying out as adapters default to native for everything out
of convenience — promotion is a deliberate, documented event (noted in the core
changelog with the fields and backends that justified it), not something that
happens by default.

**Promotion release-train.** Adding a common field is additive in core (minor
bump), but any adapter that has not updated immediately reads as "field
missing" — verification flags version skew as drift. Promotion therefore
requires all live adapters to move in the same release train before any CLI
combine. An unreleased promotion never enters a tagged CLI.

**Determinism inside native.** Because native is opaque `serde_json::Value`,
core cannot sort or canonicalize it — that responsibility sits with the
adapter, enforced by a shared roundtrip contract test (extract twice against
unchanged fixture data, assert byte-identical output). Building on `serde_json`
without the `preserve_order` feature gets object-key ordering for free (its
maps are BTreeMap-backed). Array ordering is not free and is not always "sort by
id" — some collections carry meaningful order (Keycloak's authentication flow
execution steps, ordered by an explicit priority column, being the clearest
case) that must be preserved as-is, not normalized away. The contract is
"stable output across runs," not "everything sorted identically" — how each
adapter achieves that is its own judgment, reviewed per PR.

### 3.2 The Extractor port

```rust
pub trait Extractor {
    /// Identify the backend and the schema/version detected at the source.
    async fn detect(&self) -> Result<BackendInfo>;

    /// Pull one canonical realm's worth of state from the backend.
    async fn extract(&self, scope: &ExtractScope) -> Result<CanonicalRealm>;
}
```

Every adapter implements this and nothing else is required of it. The CLI never
talks to a database directly — it only calls `Extractor` methods.

**Connection ownership.** There is no `DbHandle` in the port, and core carries
no tokio and no sqlx dependency. Each adapter's constructor takes connection
configuration and builds its own connection pool; `detect` and `extract` use
that pool internally. A port signature that referenced a connection handle
would force core to have an opinion about what a database connection looks
like — a contradiction with core being backend-agnostic.

**Async.** Native `async fn` in traits (stable since Rust 1.75, edition 2024).
No `async-trait` crate. Not object-safe by design — see Section 2.1.

### 3.3 Completeness verification

A separate port, deliberately decoupled from extraction itself:

```rust
pub trait CompletenessVerifier {
    fn expected_fields(&self, backend_version: &Version) -> FieldManifest;
    fn verify(&self, extracted: &CanonicalRealm, manifest: &FieldManifest)
        -> VerificationReport;
}
```

Each adapter ships a `FieldManifest` derived from the backend's own schema
migration history (Liquibase changelogs, for Keycloak) so the verifier can say,
concretely, "field X was introduced in version Y and is present/absent in this
extraction" rather than silently trusting that nothing was missed. The
`VerificationReport` is part of the output, not a side log — completeness is a
claim the tool should back with evidence every time it runs.

**Native is opaque; the manifest is its entire contract.** Because native is
`serde_json::Value`, the manifest is the entire completeness contract for it,
and that only holds if the manifest is honest. Release one enumerates native
fields explicitly as flat, dotted paths per canonical entity (e.g.
`native.protocolMappers[].name`), generated from the same schema-migration
source as the rest of the manifest — not a flat "optional, unenumerated blob"
note. A generic, unenumerated escape hatch inside a completeness tool would
quietly undermine the one claim the whole product is built on. Nested, typed
schemas for native are a reasonable later step if flat paths prove insufficient
for some backend, but are not the release-one default — they're a meaningfully
bigger lift to generate automatically from a schema diff, for validation depth
that isn't a proven need yet.

**Applicability.** Release-one verification semantics are *presence*:
every *existing* entity row must carry its expected paths, while entity types
themselves are `Required` or `Optional`. A freshly created realm with zero users
is complete — absent rows are not incompleteness, and empty structures are not
either. Without this rule the verifier would emit false positives on normal
small realms, which would erode trust in the completeness claim.

**Volatility.** Manifest entries also carry a `volatile: bool`. Fields marked
volatile (last-login timestamps, session-derived data) are fields the verifier
still tracks for completeness, but the adapter places them in the envelope's
`volatile` section rather than the main config body, and the content hash used
for idempotency checking covers only the non-volatile section. Volatility is a
manifest-driven, per-field property; core never special-cases which fields
happen to be volatile.

**Where volatile routing happens.** The *adapter* places volatile content in
the `volatile` section at extraction time — core does not walk opaque native
trees by manifest path (that would mean building a JSON-path engine, complete
with array-indexing semantics, inside the one crate that must stay dumbest).
Core enforces the invariant with the `assert_volatile_segregation` contract
test: every field the manifest marks volatile must appear in the volatile
section, and no volatile field may appear in the config section.

### 3.4 Output

```rust
pub trait OutputSink {
    fn write(&self, canonical: &CanonicalRealm, report: &VerificationReport)
        -> Result<()>;
}
```

Ships with a local-file sink first (encrypted-at-rest by default), with room for
an S3/object-storage sink later without touching extraction logic at all. Where
the at-rest encryption keys come from (env, passphrase, KMS) is a sink-side
decision, documented there; the cipher choice matters less than key management.

## 4. Idempotency

Since Keystate never writes to a live system, "idempotent" here means something
specific and achievable: running Keystate twice against an unchanged source
produces byte-identical output. That property is what makes the tool trustworthy
for version control, drift detection, and audit diffing — a git diff on the
output should reflect real configuration change, never extraction noise.

Three things get you there:

1. **Deterministic ordering.** Databases do not guarantee row order. Every
   collection (clients, users, roles, groups, mappers) must be sorted by a
   stable key — id, or a natural composite key where id isn't semantically
   meaningful — before serialization. Never rely on the order rows arrive from
   a query. The one exception is collections that carry meaningful order in the
   backend (Keycloak auth-flow execution steps), which are preserved as-is.
2. **Deterministic serialization.** Use BTreeMap rather than HashMap anywhere
   key order could vary, fix float formatting, and serialize with a canonical
   JSON writer so semantically identical data always produces byte-identical
   bytes. Core ships `to_canonical_bytes`; adapters use it, and the roundtrip
   contract test enforces that they do.
3. **Separate stable config from inherently volatile data.** Fields like
   last-login timestamps or session-derived data are not really "configuration"
   — they change on their own between runs even with zero config drift. Keystate
   segments these into a clearly labeled section of the output, separate from
   the core config model, so the config portion stays truly idempotent and
   volatile fields are tracked without polluting drift detection with false
   positives.

With that in place, each extraction can carry a content hash (SHA-256 over the
canonical serialized config bytes) as a fingerprint — two runs with the same
hash are provably identical, which is a cheap, strong way to detect "nothing
changed" without a full JSON diff. The hash is computed in core and covers only
the config section; volatile content never enters the hash.

This is idempotent extraction (deterministic snapshots), not idempotent writes.
Importing config back into a live system is out of scope for now.

## 5. Performance Design

- Async throughout, using tokio as the runtime and sqlx for database access —
  compile-time-checked queries, native async support, and native drivers for
  Postgres and MySQL, which covers Keycloak's and FerrisKey's most common
  deployments. Note: sqlx does not support Oracle; if Oracle-backed Keycloak
  support is needed later, that adapter will need a different driver strategy —
  a known gap, not a surprise.
- Read from a replica where one is configured, never the primary, to keep
  extraction from competing with production traffic.
- Parallelize independent table reads. Clients, groups, roles, and users are
  largely independent query plans and run concurrently against a connection
  pool, joined into the canonical model afterward.
- Stream large tables. Users and credentials are paginated with cursors to
  bound transient DB buffers. **Honest memory bound:** the canonical model holds
  the full realm in memory before hashing and verification, so peak memory is
  O(realm size) — pagination shrinks transient query buffers, it does not make
  extraction constant-memory. True streaming serialization to the sink is a
  possible later lift, not a Phase-1 requirement, and the docs will not claim
  otherwise.
- CLI-first, not a resident service. Keystate runs on demand — a migration
  snapshot, a scheduled backup, an audit run — and exits. No long-running
  process, no persistent state beyond its output.

## 6. Cross-Repo Composition & Versioning

Since core and adapters live in separate repos, keystate-cli is the only place
they come together:

- `keystate-core` publishes semver releases (crates.io, or a private registry
  initially if you'd rather not expose the canonical model publicly yet).
- Each adapter pins a compatible core version range and publishes its own semver
  releases, tied to the backend versions it's been tested against.
- `keystate-cli`'s `Cargo.toml` pins specific core and adapter versions and is
  the single source of truth for "what combination of versions is known to work
  together." A compatibility matrix (which CLI version supports which
  Keycloak/FerrisKey versions) belongs in keystate-docs and is generated from
  CI test results, not maintained by hand.

## 7. Testing Strategy

- **Contract tests in keystate-core.** A shared test suite that any
  `Extractor`/`CompletenessVerifier` implementation must pass — this enforces
  the port contract beyond the type signature. Core is a testable core with
  zero external I/O: determinism roundtrip, volatile segregation, manifest
  self-consistency, and synthetic-case verifier checks.
- **Version-matrix integration tests per adapter**, using testcontainers to
  spin up real Keycloak (and later FerrisKey) instances across supported
  versions in CI, so schema drift between versions is caught automatically
  rather than discovered in the field.
- **Determinism tests.** Run extraction twice against an unchanged fixture
  database and assert the output hash is identical — the direct, automated
  check on Section 4.
- **Completeness regression tests.** Periodically diff a keystate DB-based
  extraction against an official Keycloak CLI export (the stop-server one) on
  a throwaway instance, to catch cases where the two diverge.

## 8. Distribution

- **Docker: from day one.** Minimal base image (distroless or scratch) around
  the static Rust binary, built in keystate-cli. Small attack surface matters
  more than usual for a tool that connects to identity databases.
- **Helm chart: deferred.** Not needed until there's validated demand for
  running Keystate as a scheduled in-cluster job. Building it earlier would be
  solving a problem that doesn't exist yet — the CLI and Docker image are
  sufficient for the on-demand snapshot/backup/audit use cases Phase 1 targets.