# Read-only alpha release checklist

Published: [0.1.0-alpha.1](https://crates.io/crates/rocsteady/0.1.0-alpha.1),
2026-10-09 (local release date). This checklist applies to the experimental read-only
scope, not full EXT-1/4/5 acceptance or the complete specification backlog.

## API and compatibility policy

The architecture refactor uses published **etherbird 0.4.2** from crates.io.
The sibling path has been removed and Cargo.lock records its registry checksum.
The downloaded package's VCS commit matches release commit
`d3431ea84920505a2de3214164b8f8eb82039d70`. Local pre-publication patched checks
remain separate from the subsequent registry/package consumer checks.

Registry adoption verified 2026-10-14 on macOS ARM64 / stable Rust 1.99:
default tests, all 76 all-feature/target tests, 10 doctests, strict Clippy,
formatting, package verification and the extracted-package external consumer
pass with registry etherbird only. All other locked packages are unchanged.
The etherbird release's seven Linux/macOS/Windows/serial jobs passed in
[its hosted CI run](https://github.com/fhenskens/etherbird/actions/runs/38539621339);
that upstream evidence is separate from Rocsteady's pending hosted checks.

The subsequent decoder/template cleanup passes 78 all-feature/target tests,
10 doctests, strict Clippy and the shell workflow's examples, package verification
and generated consumer on macOS/stable. Both consumer runners now use
`scripts/package-consumer.toml.in`; quoted-path substitutions are checked.
PowerShell runtime execution remains a hosted Windows check.

Shared operation preparation/decoding and parameter budgets retain the public
API, managed ownership, local validation, healthy rejections and no replay.
The generic RetireOnDrop helper is also used in etherbird's serial/Modbus examples.
Current development supports stable Rust on Linux/macOS/Windows, with no fixed
older-toolchain compatibility promise. The 1.88 entries below are historical
release evidence. The hosted stable-platform matrix remains a follow-up gate.

The previous migration adopted etherbird 0.4.1; published
0.1.0-alpha.1 used 0.3.1. The evidence below describes that original release.
Migration checks on Windows pass on stable and Rust 1.88: all 43 regression tests plus a shutdown ownership test,
9 doctests, strict Clippy, fixture examples and extracted-package consumer.
Linux/macOS validation of this migration remains pending the existing CI matrix;
the local Docker daemon is unavailable. This change has not been published.

The working tree additionally implements explicit protocol profiles/contextual codecs
and schema providers with bounded opcode-180 reads. Current Windows stable and
Rust 1.88 validation: **72 tests** with all features/targets, **10 doctests**, strict
stable Clippy, formatting, the profile/schema TCP example and an offline-verified
package plus extracted-package consumer exercising the public codec/schema APIs.
The 12 new independent profile/schema tests include confirmed progress after a
second-batch failure or call timeout and no replay after a committed fixture effect
with lost acknowledgement. The package check used cached dependencies because the
registry endpoint was unavailable. Hosted Linux/macOS validation and publication
of these changes remain pending. No hardware evidence has been added.

The discovery/catalog slice adds 14 independent cases in tests/discovery.rs and
two simulator cases, extends standalone malformed-reply checks to opcodes 50/100,
and adds the discovery_catalog example to the hosted stable/MSRV matrix. Every
one of the 30 initial catalog parameters has a source-derived literal and independent
TCP read fixture. The packaged consumer exercises discovery, physical TLP mapping
and the built-in provider using only public APIs. Source audit and final checks are
recorded on 2026-10-10 (local date); hosted results for the unreleased changes remain pending.

The 2026-10-14 PT-136 slice adds 20 clock catalog entries (50 total), with
independent literals/TCP reads for every entry, 1.20/3.40 firmware gates, singleton
location checks and raw TIME values. PT-136/17's disputed end-week range remains
unspecified; full schema conformance, writes and hardware evidence remain open.
The example and extracted-package consumer now exercise clock reads. A pre-existing
setup test's scheduling race is removed by counting connector attempts directly
rather than relying on the peer accept task having run.

Local macOS ARM64 / stable Rust 1.99 validation for this slice: **76 tests** with
all features/targets, **10 doctests**, strict Clippy and formatting, the
discovery/catalog example, offline package verification and extracted-package
consumer. This is local evidence, not hosted CI evidence for the new slice.
Publication and the hosted stable-platform matrix remain pending.

`ClientConfig.profile` is a new required field for exhaustive struct literals;
existing configurations using `..Default::default()` keep the same default behavior.
Review this experimental API addition under the pre-1.0 versioning policy before
selecting the next release version. This working tree does not republish alpha.1.

Rust policy: current stable. Current etherbird: published registry 0.4.2 with pool; Tokio: compatible 1.x.
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
cargo run --locked --example schema_profiles
cargo run --locked --example discovery_catalog
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

The historical hosted workflow covered stable and 1.88.0 on Linux/macOS/Windows, including
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
