# Read-only alpha release checklist

Published: [0.1.0-alpha.1](https://crates.io/crates/rocsteady/0.1.0-alpha.1),
2026-10-09 (local release date). This checklist applies to the experimental read-only
scope, not full EXT-1/4/5 acceptance or the complete specification backlog.

## API and compatibility policy

Rust minimum: 1.88. Etherbird: published 0.3.1 with pool; Tokio: compatible 1.x.
Cargo.lock pins the validated versions. Consumers may resolve newer compatible
dependencies; re-run framework tests before updating the lockfile. All public APIs
are experimental before 1.0: breaking changes require a minor version increase;
patch releases preserve signatures/contracts. Error and ErrorKind are non-exhaustive;
application matches must include a fallback for compatible variant additions.
No stable hardware/firmware compatibility claim follows from fixture coverage.

API review tightened decoder errors, immutable registration, setup-state ownership,
generation handles, required-state diagnostics and authentication rejection policy.
Legacy setup maps valid login rejection to terminal Rejected; custom strategies
can identify Unsupported. Transport failures still recover. Changed credentials
require a new client. Shutdown is local disconnect; on-wire logout is unsupported.

## Candidate gates

- [x] Read-only built-in scope and unsupported services disclosed in README/usage.
- [x] Error/outcome and authentication/recovery policies documented and tested.
- [x] Runnable custom setup, typed operation, workflow and credential-provider fixtures.
- [x] Serial and real-device examples compile; hardware access requires explicit settings.
- [x] README and usage Rust snippets included in library doctests.
- [x] Changelog and pre-1.0 compatibility policy recorded.
- [x] Windows stable and Rust 1.88 default/all-feature/target/doctest checks.
- [x] Linux Rust 1.88 default/all-feature/target/doctest checks in Docker.
- [x] Linux/macOS/Windows hosted CI for the exact published commit.
- [x] Package build/verification and packaged external consumer check.
- [x] Dependency/license and publication payload audit.
- [ ] Record controller model/firmware/security mode and independent read/recovery evidence (follow-up gate for production compatibility claims; not required for this explicitly unverified experimental release).
- [x] Confirm version, clean release commit, CI success and authorized publication.

## Repeatable checks

```sh
cargo fmt --check
cargo test --locked
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo run --locked --example managed_extensions
cargo run --locked --features simulator --example authenticated_setup
cargo run --locked --features simulator --example simulator
cargo package --locked --allow-dirty --all-features
```

The local development package check allows dirty files for review. Publication
must use a reviewed clean commit and `cargo publish --dry-run --locked --all-features`
before an authorized publish. Package guides and fixture support are included;
research/roadmaps, CI, build output and local secrets are excluded. The build verifies
the extracted crate, so missing files are detected. `scripts/check-linux.sh`
also builds an independent consumer against the extracted packaged crate.

## Evidence

Local validation on 2026-10-09:

| Environment | Evidence |
| --- | --- |
| Windows x86_64 MSVC, stable Rust 1.99.0 | Default/all-feature tests, 43 integration/protocol/configuration cases, formatting and strict Clippy; all fixture examples; reformatted README brings the Windows stable/MSRV doctest count to 9 |
| Windows x86_64 MSVC, Rust 1.88.0 | Default/all-feature/all-target tests and doctests; fixture examples compile/run |
| Linux x86_64, Rust 1.88.0 in Docker | Default/all-feature/all-target tests and doctests; serial compilation; all fixture examples |
| Publication package on Windows and Linux | Extracted all-feature crate verifies; independent packaged consumer compiles/runs setup, operation, workflow, state and generation handle |

Linux image: `rust:1.88-bookworm`, digest
`sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0`.
Use a read-only workspace mount and `sh scripts/check-linux.sh`; build outputs and
the consumer live only in the container. Windows/package CI uses
`scripts/check-package-consumer.ps1` after cargo package. See
[dependency/license audit](DEPENDENCIES.md) for the locked inventory, serial MPL
dependency and reviewed publication payload.

The hosted workflow covers stable and 1.88.0 on Linux/macOS/Windows, including
examples and package consumer. All six jobs passed on candidate commit
`d83cd1d59e7b0246f5417dfc904ac55ac60d9138`:
[hosted validation run](https://github.com/fhenskens/rocsteady/actions/runs/37788381127).
This includes macOS runtime checks and optional serial compilation. All six jobs
also passed on the published commit `8ce3ac729e8160571391d5ea6983bd1400e245bb`:
[release-commit CI](https://github.com/fhenskens/rocsteady/actions/runs/37789099537).
The clean-commit `cargo publish --dry-run --locked --all-features` succeeded,
followed by actual publication and the annotated tag `v0.1.0-alpha.1`.
[GitHub prerelease](https://github.com/fhenskens/rocsteady/releases/tag/v0.1.0-alpha.1)
records the release scope and exact commit. macOS was waived as a blocking alpha
gate by the user while queued, then completed successfully before publication.
Cross-platform workflow configuration is not evidence that a hosted run succeeded.
Physical validation is outside this work and remains pending.
