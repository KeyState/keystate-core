# GitHub Actions — Workflows

This directory contains the CI/CD pipelines for `keystate-core`. There are two
workflows; both run on the same GNU/Linux runner (`ubuntu-latest`) with the
stable Rust toolchain, and both share the caching from `Swatinem/rust-cache`.

## `ci.yml` — Quality gates (every push & PR)

Runs the fast, pure-logic checks that keep `main` green. Triggered on any push
to `main` and on every pull request; running jobs are cancelled when a newer
push supersedes them.

| Job | Purpose |
|---|---|
| `fmt` | `cargo fmt --all -- --check` — enforces the shared formatting. |
| `clippy` | `cargo clippy --all-targets -- -D warnings` — lints all targets, warnings are errors. |
| `test` | `cargo test --all-targets` plus `cargo test --doc` — unit, integration, and doc tests. |
| `msrv` | `cargo check --all-targets` on Rust 1.85 — proves the published MSRV (`rust-version` in `Cargo.toml`) still compiles. |
| `audit` | `rustsec/audit-check@v2` — blocks on known vulnerabilities in the dependency tree. |
| `deny` | `embarkStudios/cargo-deny-action@v2` — enforces the license allowlist and dependency policy in `deny.toml`. |

## `release.yml` — Releases (every push to `main`)

Automates the entire publish flow with [release-plz], driven by Conventional
Commits. Runs only on pushes to `main`; it needs `contents: write` and
`pull-requests: write` because it manages branches, opens PRs, creates tags,
and publishes.

| Job | Purpose |
|---|---|
| `check` | Quality gate: fmt, clippy, tests, and `cargo package`. Nothing ships unless this passes. |
| `release-plz` | `release-plz/action@v0.5` with `command: release-pr`. Opens/updates the release PR (version bump + `CHANGELOG.md`); once merged it creates the `keystate-core-v<version>` tag, publishes to crates.io, and creates the GitHub release. Requires the `CARGO_REGISTRY_TOKEN` secret. |

> The release-plz job opens PRs with the built-in `GITHUB_TOKEN`. If the org
> blocks that, enable **"Allow GitHub Actions to create and approve pull
> requests"** under Settings → Actions → General → Workflow permissions, or
> pass a fine-grained PAT as `GITHUB_TOKEN` (see `RELEASE.md` §2).

## Secrets used

- `CARGO_REGISTRY_TOKEN` — crates.io token with publish rights (`release.yml`).
- `GITHUB_TOKEN` — built-in, used by both workflows (audit + release-plz).

See `RELEASE.md` for the full release process and one-time setup.

[release-plz]: https://release-plz.enyx.fr/