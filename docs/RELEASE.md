# Experimental monitoring release checklist

## Published 0.2.0 — 2026-10-16

This release adds PT-101 write settings, clock setting, transaction retrieval and
configurable-table access. See the [changelog](../CHANGELOG.md#020--2026-10-16),
[migration notes](../README.md#migrating-from-013) and [requirements](REQUIREMENTS.md).
The public simulator struct additions require a minor version under this project's
pre-1.0 policy. The generic catalog retains 332 entries and 18 audited writable fields;
PT-99 has separate typed definition reads. Hardware validation remains deferred.

Local macOS/stable release checks passed 285 all-feature/all-target tests, 231
default-feature tests plus 25 doctests, 25 explicit all-feature doctests, formatting,
strict all-feature/all-target Clippy, rustdoc with warnings denied, all 16 fixture
examples, offline package verification and the extracted-package consumer. The
reviewed 127-file payload contains no build output, secrets or vendor manuals.
The [dated session journal](reference/SESSION_2026_10_16.md) preserves the original
scope, timing and test-count checkpoints.

Release commit `7118d9986ab2b9910174f5d4e654ba978e1602aa` passed
[all three hosted stable Linux/macOS/Windows jobs](https://github.com/fhenskens/rocsteady/actions/runs/38906701797)
and clean-commit `cargo publish --dry-run --locked --all-features`. The reviewed
127-file package was published to [crates.io](https://crates.io/crates/rocsteady/0.2.0).
The downloaded archive's SHA-256 is
`77ae5959416eea07334053075273bff6dc58dd82e4a6e00bfb3db763020fbebf`;
it matches the clean publication archive byte-for-byte, and its VCS metadata
identifies that exact commit without a dirty flag. The pushed annotated `v0.2.0`
tag targets the same commit, and the
[GitHub release](https://github.com/fhenskens/rocsteady/releases/tag/v0.2.0)
is published as an ordinary experimental release.

An independent consumer using registry `rocsteady = "=0.2.0"`, with no source-path
override, passed the packaged public API checks. Its checksum matches the verified
archive and its dependency versions match the reviewed Cargo.lock. Publication,
tag and registry-consumer verification completed 2026-10-16. No hardware claim
is added; the source-defined scope and provisional interpretations above remain.

## Published 0.1.3 — 2026-10-15

This release combines the completed OP-136–139 history work with schema-aware
OP-166/181 writes (12 audited fields), all PT-106/107/108 read definitions
(332 catalog entries), finite caller-driven polling and registry etherbird 0.4.3
adapter helpers. The changelog groups all additions under 0.1.3; earlier dated
development evidence below preserves its original scope and unreleased status
at the time it was recorded.

Local registry-based checks passed 244 all-feature/all-target tests, the default
feature gate, 22 doctests, strict Clippy, formatting, rustdoc, fixture examples,
package verification and the extracted external consumer. Hosted CI includes the
write/read-back example and bounded disconnect/re-discovery monitoring demo.
Release commit `4336323ab032ed3d22861008225b9c741159dabb` passed
[all three stable Linux/macOS/Windows jobs](https://github.com/fhenskens/rocsteady/actions/runs/38727715062)
and the clean `cargo publish --dry-run --locked` before publication. The reviewed
111-file package was published to [crates.io](https://crates.io/crates/rocsteady/0.1.3).
The downloaded archive's SHA-256 is `7ea4bb606fdda07451cd4bf450fffc87f0d016f91f3438094604015b5c613c74`;
its VCS metadata identifies that exact clean release commit. The pushed annotated
v0.1.3 tag targets the same commit and the
[GitHub release](https://github.com/fhenskens/rocsteady/releases/tag/v0.1.3) is published.
An independent consumer using registry `rocsteady = "=0.1.3"` passed
its public API checks without a source-path override.

Physical-controller and actual serial exchanges remain unvalidated. Full OP/PT
conformance, disputed source interpretations, broader write constraints and
controller partial-commit/error-offset behavior remain open. No automatic replay,
resume, temperature conversion or durable controller identity is introduced.

## Etherbird 0.4.3 registry adoption — 2026-10-15

The current checkout adopts registry etherbird 0.4.3 and its opaque generation
bindings and DisconnectOnDrop ownership guard. Rocsteady retains its public
GenerationValue API, live-state checks, numeric generation reporting, per-exchange
RetireOnDrop guard and explicit outcomes/no replay. A cross-client handle
regression and existing cancellation/stale-handle tests exercise the composition;
[requirements](REQUIREMENTS.md#etherbird-adapter-composition-2026-10-15) links them.
[EXTRACTION.md](EXTRACTION.md) records other candidates and their scope boundaries.

Upstream release commit `695b33e0166b0ee63031fe952dcd01abd1d759f3` passed
[all seven hosted platform/serial jobs](https://github.com/fhenskens/etherbird/actions/runs/38725355730)
and the clean publication dry-run before publication. The registry package's VCS
metadata and pushed v0.4.3 tag identify that commit. Earlier dated evidence below
retains the dependency/version used at the time.

Registry-based local verification passed 244 all-feature/all-target tests, the
default-feature gate, 22 doctests, strict Clippy, formatting and rustdoc, plus
package verification, an extracted-package consumer and the bounded polling demo.
No path override remains. Rocsteady's manifest remains 0.1.3; this is dependency
adoption and unreleased application work, not a new rocsteady publication.

## Unreleased MVS and caller-driven polling evidence — 2026-10-15

PT-108 adds 69 read definitions, bringing the catalog to 332 entries. The /67
configuration counter requires a caller-established per-location 4088 assertion;
all MVS writes remain blocked. [MVS contracts](MVS.md), the per-parameter manifest
and DEC-10.09 retain command-range, unit and subtype caveats. The finite monitoring
example explicitly discovers each sample, reports partial errors once and leaves
subsequent attempts/intervals to the caller. Existing temperature/write work,
version 0.1.3 and completed history code remain intact.

Local checks passed: 243 all-feature/all-target tests, the default-feature test
gate, 22 doctests, strict all-feature/all-target Clippy, formatting and rustdoc.
The monitoring demo completed sample 1, reported sample 2's injected disconnect
without replay and completed sample 3 after explicit discovery on generation 2.
Independent TCP tests additionally moved the MVS physical position after reconnect
and verified the next caller sample used the new TLP. Offline package verification
and the extracted consumer passed, including DP/SP/temperature, FLOAT module type,
raw masks and explicit 4088 counter reads. CI/Linux scripts run the bounded demo.

TCP fixtures required execution outside the network sandbox. Offline consumer
resolution added its own package to the copied lockfile. No dependencies, package
version or publication were changed. Software evidence establishes neither
physical/controller interoperability nor persistent identity, full PT conformance
or resolution of DEC-10.09; those acceptance items remain open.

## Unreleased temperature catalog evidence — 2026-10-15

PT-106/107 add all 61 RTD/thermocouple read fields, bringing the catalog to 263
entries. [Temperature contracts](TEMPERATURE.md), the per-parameter catalog
manifest and DEC-10.08 keep source-access/calibration/copy defects explicit.
All temperature writes remain blocked. Existing OP-166/181 development work,
version 0.1.3 and the completed history implementation are retained.

Local verification passed 235 all-feature/all-target tests, the default-feature
`cargo test --locked` gate, 21 doctests, strict all-feature/all-target Clippy,
formatting and rustdoc. The independent `temperature_monitoring` TCP example,
offline package verification and extracted-package consumer passed. The consumer
reads both temperature schemas, unit/sensor metadata, unused alarm bits and
firmware-gated failsafe fields, and checks the temperature-write gate. CI and the
Linux check script now run the monitoring example. Local TCP tests required
execution outside the network sandbox; offline consumer resolution added its own
package to the copied lockfile. No dependency change or publication occurred.

Evidence is software-only: synthetic independent literals, simulator datasets and
framework lifecycle checks do not establish physical Ethernet/serial/controller
compatibility or resolve DEC-10.08. Full PT completion still requires remaining
metadata, write procedures and applicability/interoperability evidence.

## Unreleased OP-181 and writable catalog evidence — 2026-10-15

This slice extends the existing uncommitted OP-166 work; it does not replace the
completed 0.1.3 history implementation or bump the package version. OP-181 has
separate count/TLP/value packing and a documented conservative 240-byte request
policy. The writable catalog now has 12 audited fields; see [the manifest and
outcome contracts](WRITES.md). No physical-controller or publication claim follows.

Local checks passed: 228 all-feature/all-target tests, 20 doctests, strict
all-feature/all-target Clippy, formatting, rustdoc, the combined `parameter_writes`
simulator example, offline package verification and an extracted-package consumer
that executes cross-point OP-181 writes and explicit read-back. Independent TCP
literals cover mixed widths/order/duplicates and all three catalog layouts;
shared write tests exercise both opcodes' interruption, preflight and generation
contracts. Simulator tests cover the declared manifest, explicit partial effects
and lost-ack recovery without replay or implicit continuation. Test TCP listeners
required execution outside the network sandbox. The consumer used offline
resolution to add its own package to the copied lockfile.

Broader parameter constraints, remaining active-cancellation/stop matrices,
controller error-offset/partial-commit evidence, hosted CI and physical validation
remain open. Historical OP-166 and 0.1.3 evidence below is retained as recorded.

## Unreleased OP-166 development evidence — 2026-10-15

The working tree was clean before this slice; completed 0.1.3 history work and
its release evidence below were retained. Version remains 0.1.3; this work is
unreleased and introduces no publication or physical-compatibility claim.

Local verification: all-feature/all-target regression suite passed 221 tests
(existing 217 plus the first four write tests). The final targeted write suite
passed five tests after adding queued-future cancellation. All 20 doctests passed,
strict all-feature/all-target Clippy, formatting and rustdoc passed, and the
`parameter_writes` simulator example completed write/read-back. Offline package
verification passed; an extracted-package consumer compiled and ran successfully,
including the public write-plan assertion. Local TCP checks required execution
outside the network sandbox. The initial online package attempt failed on registry
DNS access; the offline retry verified successfully with cached locked dependencies.
The consumer used offline resolution to add its own package to the copied lockfile.

This verifies software behavior for the [declared write subset](WRITES.md), not
controller atomicity, firmware interoperability, serial exchange or hosted CI.


## 0.1.3 indexed, day-oriented and selected history

OP-136 adds minute/periodic/daily contiguous point blocks with time-major TIME/FLOAT
rows, bounded period paging, explicit confirmed progress and no implicit replay.
Table minute bounds, provisional prose bounds and configured/caller policies use
the same public interfaces. Profile and per-call codecs remain replaceable under
managed ownership. Independent Table 2-19 literals/TCP and simulator evidence,
a runnable multi_history example and extracted consumer cover the public API.
See [exact contracts and limitations](MULTI_HISTORY.md).

OP-137/138 add yearless day/month index lookup, whole-day values or special-point
255 timestamps and an explicit indexed OP-137/135 alternative for oversized days.
Table-based defaults use public codecs/profile seams, and callers can supply their
own interpretation or request operation. Indexed reads preserve confirmed progress,
geometry/ownership rules and final mapping recheck without implicit fallback/replay.
See [day contracts and ambiguity dispositions](DAY_HISTORY.md).

OP-139 adds configured inventory and ordered selected-point reads, with optional
TIME words, bounded period pages, public policy/codec replacements and opt-in
inventory membership/recheck. Changes and failures preserve confirmed progress;
no atomic-snapshot, automatic-resume or physical claim is made. See
[history-information contracts](HISTORY_INFORMATION.md).

Local macOS ARM64 / stable Rust 1.99 validation passes 217 all-feature/all-target
tests and 19 doctests, default tests, strict Clippy, formatting and all-feature
rustdoc. All eleven controller-free fixture examples pass, including history_information.
The 98-file development package verifies and its extracted external consumer passes
against registry etherbird 0.4.2, including OP-136–139 defaults, consumer codecs, explicit request Operations,
configured indexed paging and inventory-checked selections. No dependency versions changed.

Hosted 0.1.3 CI, clean-commit publication checks and registry publication/tag are
not yet recorded. Physical-controller and actual serial exchanges remain
unvalidated. The dated 0.1.2 evidence below retains its original scope.

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
The dependency is published registry **etherbird 0.4.3** with its pool feature;
there is no sibling path override. Cargo.lock pins the validated dependencies.
Its package VCS commit is `695b33e0166b0ee63031fe952dcd01abd1d759f3`.
The release includes the generation/scope helpers and preceding internal cleanup.

`ClientConfig.profile` is a new required field in exhaustive struct literals.
Supply it explicitly or use `..Default::default()` to retain default behavior.
Releases use ordinary `0.x.y` versions without alpha/beta suffixes. All versions
below 1.0 are experimental and may have incomplete protocol coverage or unverified
hardware compatibility. Breaking API changes use a new minor version; compatible
additions and fixes use patch releases. The first ordinary release is
**0.1.0**, following the historical 0.1.0-alpha.1 prerelease. Cargo.toml and
Cargo.lock carry the current version, 0.2.0. Each release requires its own
validation and publication evidence; dated records retain their original versions.
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
cargo run --locked --features simulator --example parameter_writes
cargo run --locked --features simulator --example clock_set
cargo run --locked --features simulator --example transactions
cargo run --locked --features simulator --example configurable_tables
cargo run --locked --example temperature_monitoring -- 3 100 --demo-reconnect
cargo run --locked --example log_retrieval
cargo run --locked --example history_metadata
cargo run --locked --example history
cargo run --locked --example multi_history
cargo run --locked --example day_history
cargo run --locked --example history_information
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
