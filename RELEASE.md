# Keystate — Release Process & Upstream Tracking

This document covers two related but distinct concerns: how Keystate itself
gets released across five repos without drifting out of sync, and how the
project stays current with upstream Keycloak and FerrisKey schema changes
without discovering them from a user's bug report.

---

## 1. Versioning Policy

Every repo follows semver independently, but the *meaning* of a version bump
differs by repo:

| Repo | A "major" bump means | A "minor" bump means | A "patch" bump means |
|---|---|---|---|
| `keystate-core` | A canonical model field/struct removed or changed incompatibly | A new field/struct added, additively | Bug fix, no model change |
| `keystate-adapter-*` | Drops support for a previously-supported backend version, or a breaking core dependency bump | Adds support for a new backend version, or a new field extraction | Bug fix, no behavior change |
| `keystate-cli` | Breaking CLI flag/output-format change | New feature, new adapter bundled | Bug fix, dependency bump only |

`keystate-core` is the contract. Its version is the one everything else is
pinned against, so changes there get the most scrutiny — a major bump
there forces a major bump in every adapter that hasn't absorbed the change
yet, which is expensive across five repos. Prefer additive changes whenever
the model allows it.

**Promotions are a special release case (and a release-train obligation).** A
promotion (same native field seen in two or more backends, moved into common,
per the architecture document §3.1) is an additive minor bump in `keystate-core`
and always gets a changelog entry naming the fields and the backends that
justified the move. But an adapter that has not absorbed the new common field
extracts it missing, and the verifier then flags healthy realms as incomplete
— version skew misread as drift. So a promotion is only tagged once *all* live
adapters ship compatible minor bumps, and `keystate-cli` only tags a combination
after every constituent repo has absorbed it. A promotion is never released
piecemeal across the org.

## 2. Release Mechanics

- **Tag-based releases.** A release is a git tag (`v1.4.0`) on `main`,
  nothing more exotic. No separate release branches.
- **Changelogs generated from Conventional Commits.** Since commit messages
  are structured (`feat:`, `fix:`, etc., per the development document),
  changelog generation and the semver bump itself can be automated with a
  tool like `release-please` or `cargo-release` rather than hand-written —
  removes a manual step that's easy to get wrong or skip under time
  pressure.
- **Publish targets per repo:**
  - `keystate-core` and each adapter publish as crates (crates.io once the
    project is public and stable enough to commit to that namespace; a
    private registry is fine in the meantime).
  - `keystate-cli` is the only repo that builds and publishes the actual
    distributable: the binary release on GitHub Releases, and the Docker
    image, pushed on every tagged release via CI.
- **No release goes out with a failing nightly.** The scheduled backend-matrix
  and completeness-regression tests (from the development document) act as a
  gate — if the nightly run against `main` is red, that's fixed before the
  next tag, not after.

## 3. The Compatibility Matrix

Because Keystate's whole value proposition rests on completeness, "which
version of Keystate supports which version of Keycloak/FerrisKey" needs to
be a documented, tested fact — not a claim. This lives in `keystate-docs` as
a generated table, not a hand-maintained one:

| keystate-cli | keystate-core | adapter-keycloak | Keycloak versions tested | adapter-ferriskey | FerrisKey versions tested |
|---|---|---|---|---|---|
| 1.4.0 | 1.2.0 | 1.3.0 | 24.x – 26.x | 0.2.0 | main @ 2026-06 |

Each row is populated automatically from CI's integration test matrix results
at release time — if a version combination wasn't actually tested green in
CI, it doesn't appear in the matrix as supported. This is also where you'd
publish a deprecation timeline when dropping support for an old backend
version, so users have a documented window rather than a surprise gap.

## 4. Tracking Upstream: Keycloak

Keycloak's schema changes are visible before they hit a release, via its
Liquibase changelogs in the main Keycloak repository. The tracking process:

1. **Automated schema-diff check.** A scheduled CI job in
   `keystate-adapter-keycloak` pulls the latest Keycloak Docker image
   (tracking both the latest stable and the current RC/milestone if one
   exists), runs the adapter's schema introspection against it, and diffs
   the result against the last known schema fingerprint stored in the repo.
2. **On a detected diff, the job opens an issue automatically** — labeled
   `upstream-schema-change` — summarizing what changed (new table, new
   column, altered constraint). This turns "did something change upstream"
   from a manual research task into a notification that shows up on its own.
3. **A human triages the issue**: does this map to something the canonical
   model should represent? If yes, it becomes a `feat:` following the
   standard core → adapter → cli flow from the development document. If it's
   internal to Keycloak and irrelevant to configuration state (e.g. a purely
   operational/cache table), it gets closed with a note explaining why, so
   the reasoning is preserved for the next person who wonders the same
   thing.
4. **Release notes and major Keycloak version announcements** are worth a
   lighter-weight manual skim in addition to the automated diff — schema
   changes are only part of what matters; deprecations or behavioral changes
   to fields Keystate already extracts (e.g. a field being repurposed rather
   than added) won't necessarily show up as a schema diff but could still
   silently change what the extracted data *means*.

## 5. Tracking Upstream: FerrisKey

Same underlying goal, adjusted for a much younger, faster-moving project:

- FerrisKey's schema changes land as normal commits/migrations in its own
  repository rather than a versioned changelog process as mature as
  Keycloak's Liquibase history. The equivalent automated check here watches
  FerrisKey's migrations directory directly (via a scheduled job comparing
  against the last-seen commit hash) rather than diffing a running
  instance's schema, since stable release tags may be sparser early on.
- Given the project's early stage, expect this to fire more often and with
  less predictability than the Keycloak check. Budget triage time for it
  accordingly rather than treating an infrequent Keycloak-style cadence as
  the default assumption.
- Because FerrisKey adoption is still small, it's reasonable — and
  worthwhile — to engage directly with FerrisKey's own maintainers about
  schema stability plans, rather than purely reacting to diffs after the
  fact. A young project's maintainers are often glad to hear from a
  downstream consumer of their schema.

## 6. Deprecation Policy

When a backend version is dropped from support (typically once it's past its
own upstream end-of-life):

- Announce in the adapter's changelog at least one minor release ahead of
  actually dropping it.
- Update the compatibility matrix with an explicit deprecation date, not just
  silent removal.
- Keep the last adapter version that supported it easily discoverable (a
  pinned note in the README) for anyone who needs to stay on it temporarily.
