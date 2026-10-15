# Experimental monitoring release checklist

## 0.1.2 I/O and history — 2026-10-15

Version 0.1.2 adds 72 PT-102/104/105 read fields, bringing the catalog to 202 entries,
and includes requirements-to-tests traceability. It is published on crates.io,
with annotated tag `v0.1.2` pushed.
All 27 DO, 17 AO and 28 pulse-input read fields have independent literal/TCP coverage, source
access/range/version assertions, enum/bit/unit metadata and explicit constraints.
Tests cover every layout, firmware boundaries, malformed replies, device errors,
raw unknown values and generation replacement. Simulator values cover every field;
the discovery example and extracted-package consumer include output and pulse reads.
Pulse rates, contract-day totals and accumulation remain controller-reported values;
no local conversion/rollover is inferred. The nonzero conversion constraint is
recorded explicitly without inventing a numeric interval.

OP-105/108 metadata adds usable raw defaults, named provisional occurrence codecs,
per-call choices and caller replacements with no format autodetection. Standalone,
managed/workflow, malformed/device-error, cancellation and budget tests are present.
The history example is run in CI and the package consumer checks all codec choices.
HistoryMetadataModel is optional; existing Model struct literals remain compatible.

OP-135 adds all single-point value/timestamp selectors, index-only probes, bounded
one-generation paging and explicit failure progress. Table minute bounds are the
default; prose and configured bounds use the same public policy seam. Index movement
is reported explicitly; no gap-free, automatic-resume or physical claim is made.

Version 0.1.2 validation: 168 all-feature/all-target tests, 16 doctests, strict Clippy,
formatting, the discovery example, all-feature package verification and an
extracted-package consumer. The user confirmed hosted stable Linux/macOS/Windows
CI passed for release commit `a939ac1b150cd75d76c789b0dc35f7ca9f22c2de` on
2026-10-15. After GitHub CLI authentication became available, all three jobs
were independently verified successful in the
[release-commit CI run](https://github.com/fhenskens/rocsteady/actions/runs/38669475919).

The clean-commit publication dry-run and crates.io upload succeeded. The downloaded
83-file registry archive records that exact clean release commit; annotated tag
`v0.1.2` is pushed. The published
[GitHub release](https://github.com/fhenskens/rocsteady/releases/tag/v0.1.2)
contains the changelog, validation summary and interoperability limitations.
Physical-controller and actual serial interoperability remain unverified.
Output writes, full PT completion and DEC-10.07 closure remain open.
The published-release evidence below is retained with its original scope/counts.

## Published 0.1.1 documentation patch — 2026-10-15

This patch declares `https://docs.rs/rocsteady` as the package documentation URL,
enables all-feature docs.rs builds and adds API documentation generation to CI.
No Rust source, public API or dependency versions change. Existing documentation
cleanup is included; 0.1.0's runtime behavior and physical-validation limits apply.
Local patch checks pass default/all-feature tests (122 all-feature cases), 14
doctests, strict Clippy, all-feature rustdoc, all fixture examples, package
verification and the extracted-package consumer. Hosted CI is triggered by the
release push; its result has not been independently inspected.

The clean-commit publication dry-run and crates.io upload succeeded; `v0.1.1` is
pushed. The downloaded registry package was verified to carry the documentation
URL, all-feature docs.rs configuration and exact clean publication commit. Hosted
docs.rs generation is automatic after publication.

## Published 0.1.0 — 2026-10-15

[Version 0.1.0](https://crates.io/crates/rocsteady/0.1.0) is a substantial feature update over
[0.1.0-alpha.1](https://crates.io/crates/rocsteady/0.1.0-alpha.1): discovery,
schema-backed reads, a 130-parameter ROC800 catalog, typed alarm/event retrieval,
and explicit SRBX polling and consumed-alarm acknowledgement. This experimental
monitoring release does not claim complete ROC Plus coverage or physical-controller interoperability.

Published alpha.1 remains the original read-only release. Version 0.1.0 adds
opcode-225 acknowledgement, which changes controller SRBX status only when the
caller explicitly requests it. Nothing automatically acknowledges or replays work.
See [CHANGELOG](../CHANGELOG.md), [coverage](COVERAGE.md), [catalog](CATALOG.md)
and [log/SRBX contracts](LOGS.md) for exact scope.

## API and compatibility policy

Use current stable Rust on Linux/macOS/Windows; there is no fixed older-toolchain
compatibility promise. Rust 1.88 checks below are historical alpha.1 evidence.
The dependency is published registry **etherbird 0.4.2** with its pool feature;
there is no sibling path override. Cargo.lock pins the validated dependencies.
Its package VCS commit is `d3431ea84920505a2de3214164b8f8eb82039d70`.
The subsequent etherbird cleanup is unreleased and is not required by this update.

`ClientConfig.profile` is a new required field in exhaustive struct literals.
Supply it explicitly or use `..Default::default()` to retain default behavior.
Releases use ordinary `0.x.y` versions without alpha/beta suffixes. All versions
below 1.0 are experimental and may have incomplete protocol coverage or unverified
hardware compatibility. Breaking API changes use a new minor version; compatible
additions and fixes use patch releases. The first ordinary release is
**0.1.0**, following the historical 0.1.0-alpha.1 prerelease. Cargo.toml and
Cargo.lock now carry version 0.1.2.
Error and ErrorKind are non-exhaustive; callers must include fallback match arms.

## Published 0.1.0/0.1.1 software evidence

Local macOS ARM64 / stable Rust 1.99 validation passes:

- Default tests and **122 all-feature/all-target tests**, including independent
  wire literals, loopback TCP, in-memory streams, lifecycle and simulator cases.
- **14 doctests**, including rejection of an event checkpoint for alarm acknowledgement.
- Strict all-feature Clippy and formatting.
- Every runnable fixture example, package verification and the extracted-package
  external consumer against registry etherbird 0.4.2.

The documentation/package refresh also passes all 14 doctests and offline package
verification using cached locked dependencies; registry DNS was unavailable for
that repeat check. No dependency versions changed.

Coverage includes every catalog entry, all three explicit I/O layouts, firmware
and location boundaries, raw floating-point bits, all log record kinds/tags,
bounded paging and partial progress, changed anchors, notification lag/missing
signals, newer alarms, rollover, lost acknowledgements, cancellation and no replay.
Matching anchors do not prove durable gap-free continuity. A confirmed empty
acknowledgement proves receipt, not that SRBX pending status cleared.

The user confirmed hosted stable Linux/macOS/Windows CI passed for exact release
commit `2c43da00b72ef251a94e87ca84425b20ac3740ae` on 2026-10-15. This is user-reported
hosted evidence: the agent could not independently inspect the private Actions
results because GitHub API authentication was unavailable. A clean-commit
`cargo publish --dry-run --locked --all-features` passed, followed by successful
publication and annotated tag `v0.1.0`. The downloaded crates.io archive's VCS
metadata matches that exact clean commit. A GitHub release page remains pending
API authentication; prepared notes are represented by the changelog.

Serial support compiles; actual serial exchanges and physical-controller tests
remain follow-up evidence for compatibility claims.
The simulator is a deterministic fixture rather than a complete controller emulator.

Upstream etherbird's [0.4.2 release CI](https://github.com/fhenskens/etherbird/actions/runs/38539621339)
and [subsequent cleanup CI](https://github.com/fhenskens/etherbird/actions/runs/38547036690)
passed seven jobs each. Those runs do not validate rocsteady 0.1.2.

## Published 0.1.0 release gates

- [x] Scope, unsupported services and physical-validation limitations disclosed.
- [x] Explicit SRBX effects, acknowledgement outcomes, checkpoint limitations and
  caller ingestion/controller-identity responsibilities documented and tested.
- [x] Authentication/recovery, cancellation retirement and no-replay contracts reviewed.
- [x] Runnable discovery, schema/profile, log/SRBX and managed-extension examples.
- [x] Local tests, doctests, Clippy, formatting and packaged external consumer pass.
- [x] Changelog, migration note and dependency/license inventory updated.
- [x] Select ordinary experimental version 0.1.0 and update Cargo.toml/Cargo.lock.
- [x] Review the final 0.1.0 package payload: 70 files, resolving local Markdown
  links and no excluded secret/build paths.
- [x] Commit/push the candidate; user confirmed hosted stable Linux/macOS/Windows
  CI passed for the release commit.
- [x] Clean-commit publication dry-run, crates.io publication, pushed annotated tag
  and downloaded-package commit verification.
- [ ] Create the GitHub release page when API authentication is available.
- [ ] Record model, firmware, security mode and independent Ethernet/serial results
  before making production compatibility claims (experimental-release follow-up).

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
cargo run --locked --example log_retrieval
cargo run --locked --example history_metadata
cargo run --locked --example history
cargo run --locked --features simulator --example authenticated_setup
cargo run --locked --features simulator --example simulator
cargo package --locked --allow-dirty --all-features
```

`scripts/check-linux.sh` also compiles/runs an independent consumer against the
extracted package. Windows CI uses `scripts/check-package-consumer.ps1` and the
same manifest template. Dirty package verification is for development review;
publication uses a reviewed clean commit. Source, tests, fixture examples and
repository documentation are packaged; build output, local secrets, git internals,
CI and developer scripts are excluded. Vendor manuals are not redistributed.
See [dependency/license audit](DEPENDENCIES.md) for the locked inventory.

## Historical evidence: 0.1.0-alpha.1

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
