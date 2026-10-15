# Comprehensive implementation roadmap

Status: open; reconciled 2026-10-15. This document tracks remaining work.
Implemented behavior belongs in [coverage](COVERAGE.md), executable evidence in
[requirements](REQUIREMENTS.md), and delivery history in the [changelog](../CHANGELOG.md).

## Current handoff — 2026-10-15

The manifest remains **0.1.3**; current development adds unreleased features
to the completed 0.1.3 history baseline. The implemented history services include:

- OP-105/108 metadata and OP-135 single-point reads.
- [OP-136 multi-point reads](MULTI_HISTORY.md), with time-major rows and bounded paging.
- [OP-137/138 day reads](DAY_HISTORY.md), with yearless defaults and explicit indexed fallback.
- [OP-139 inventory and selected reads](HISTORY_INFORMATION.md), with optional timestamps
  and inventory checks.

These build on managed extensions, explicit profiles/codecs, discovery, the
202-parameter read catalog, schema-backed reads, typed logs and opt-in SRBX
polling/acknowledgement. Coverage and feature contracts describe the exact limits;
this roadmap tracks remaining acceptance.

Unreleased development now adds [OP-166/181 writes](WRITES.md), with a 12-field
source-audited writable subset, explicit partial/uncertain outcomes and no replay.
PT-106/107 temperature and PT-108 multi-variable sensor reads now extend the
catalog to 332 entries; see [temperature scope](TEMPERATURE.md) and
[MVS/caller-driven polling contracts](MVS.md). Current development verification passed
243 all-feature/all-target tests, the default-feature gate and 22 doctests;
[release evidence](RELEASE.md) records the checks. The dated 0.1.3 history
evidence remains in that record.
Strict Clippy, formatting, rustdoc, offline package verification and the extracted
consumer also passed for current development. Hosted CI for the final release
commit, version/changelog finalization and publication gates remain pending.
Physical-controller and actual serial exchanges remain unvalidated.
[Release evidence](RELEASE.md) owns validation/publication provenance, including the
preceding 0.1.2 release; historical counts and commit details are retained there.

## Scope and delivery policy

The comprehensive target is ROC Plus D301180X012, October 2022, Chapters 1–7.
Older manuals, other controller families and undocumented services need explicit
supplemental scope and authoritative definitions; an index entry is not a wire definition.

Deliver useful verified defaults and public extension contracts for ambiguous
behavior. Provide named selectable implementations where alternatives are sufficiently
specified, and support caller replacements through those same contracts. Provisional
choices must disclose assumptions; unknown procedures must not be invented.
Unresolved device behavior does not block unrelated well-defined work.

Full built-in conformance remains a separate objective from a scoped experimental
release. Custom implementations do not close built-in conformance rows. Keep software
implementation, source interpretation and physical interoperability evidence separate.
The [decision register](reference/DECISIONS.md) tracks ambiguity dispositions;
[service](SPEC_WORK_PLAN.md) and [schema](SCHEMA_WORK_PLAN.md) plans retain the
complete inventories and acceptance criteria.

## Completion rules

For every selected EXT/CORE/OP/PT slice:

- Identify specification sections, forms, sizes, applicability and relevant DEC rows.
  Record defaults, named alternatives, public replacement seams and any blockers.
- Encode supported requirements in tests, with stable requirement IDs and comments
  identifying the specification sections. Derive independent wire literals and
  cover malformed replies, boundaries, device errors and unknown values.
- Implement typed APIs and simulator behavior; verify actual loopback TCP and the
  applicable ownership, cancellation, deadline, generation and no-replay contracts.
  Client/simulator agreement alone is not an independent wire oracle.
- Update usage, examples, requirements, coverage and work-package statuses together.
  Record physical validation separately with model, firmware and channel evidence.

## Delivery sequence and dependencies

Milestones describe capability groups; partial delivery does not close the group.
Hardware and cross-platform validation can proceed alongside software development.

| Milestone | Remaining scope | Dependencies |
| --- | --- | --- |
| M0-E: extension platform | Open EXT-1/2/3 acceptance: protected discovery, authentication/logout, identity/cache policy, framing evidence and broader write-schema constraints | Managed contracts; source-backed procedures |
| M0: discovery/security foundations | Remaining CORE-1/2/3 and OP-6/7/17/50/100 variants/applicability; DEC-1/2/3/8 evidence | Explicit profiles and readiness strategies |
| M1: parameter access | Remaining PT batches, firmware/layout applicability and discovery/bootstrap policies | Schema metadata and explicit device context |
| M2: logs/SRBX | Stronger durable continuity evidence and remaining applicability/schema coverage | Identity, retention and index semantics |
| M3: history | OP-206, history PT batch; remaining OP-105/135–139 interpretation/applicability evidence | History geometry, time/schema models; DEC-4/5 |
| M4: configuration/writes | OP-8/10/11, PT-99 and writable schemas beyond the delivered OP-166/181 subset | Access/budget validation and explicit uncertain outcomes |
| M5: files | OP-203 forms and lifecycle | Exclusive workflows, handle lifetime and uncertain outcomes; DEC-5 |
| M6: remote/shared channels | OP-24/205, broadcast CORE-1 and network schemas | Framing/address/reader contracts; DEC-1/5/9 |
| M7: release/conformance | Candidate gates, remaining EXT acceptance and declared scope; continued full-baseline work | Applicable software and interoperability evidence |

Full catalog work spans the service milestones. History and catalog expansion can
proceed independently; fully specified writes need not wait for all read services.
Enhanced-security uncertainty does not block legacy-device work, but remains a
limitation on compatibility claims. No milestone permits automatic side-effect replay.

## P0: remaining extension, discovery and security work

- [ ] Complete the open acceptance items in [EXTENSIBILITY.md](EXTENSIBILITY.md).
  Add source-supported presets and external-consumer evidence as new ambiguous
  behaviors become implementable; retain explicit framework/scope blockers in DEC rows.
- [ ] Establish protected discovery and authentication/readiness selection, including
  firmware-update states and explicit device/firmware applicability.
- [ ] Define enhanced authentication/session-key procedures only from adequate
  sources; specify logout, changed credentials, expiry and restoration policy.
  Extend rejection, busy/expiry, cancellation and replacement tests for new variants.
- [ ] Define identity/capability change handling and invalidation before adding
  automatic profile/schema caches. Existing generation-bound snapshots do not
  establish persistent controller identity.
- [ ] Complete remaining configuration/setup variant and failure matrices.
- [ ] Audit general, per-opcode and routed size limits, including OP-50; distinguish
  wire-field capacity from actual device limits.
- [ ] Validate address-correlation applicability. Exact source matching is the default;
  alternatives use the public profile policy while destination/opcode checks remain
  mandatory. Routed framing requires its own framework work.
- [ ] Establish serial corruption/resynchronization policy and independent Ethernet
  CRC/framing captures where required by the decision register.

## P1: remaining read services and catalog

- [ ] Expand all remaining schema batches and per-parameter manifests: widths,
  access, ranges/defaults, update ownership, conditional access, location limits,
  firmware variants and source constraints. Selected read fields are implemented;
  the complete point-type inventory remains open in [SCHEMA_WORK_PLAN.md](SCHEMA_WORK_PLAN.md).
- [ ] Add source-backed automatic layout/firmware/bootstrap selection where possible;
  keep explicit selection available when discovery is inconclusive.
- [ ] Improve device-error context while preserving raw codes/offsets and keeping
  local validation distinct from healthy device rejection.
- [ ] Implement OP-10 configurable table reads, including version/width/range
  variants, independent fixtures and simulator behavior.
- [ ] Implement OP-206 transaction commands: record inventory, tagged field widths,
  offsets/continuation, bounds and retention behavior.
  Use contextual codecs/policies for unresolved forms and explicit progress for
  multi-exchange work; see [service packages](SPEC_WORK_PLAN.md).
- [ ] Resolve remaining history occurrence-time, minute-index and day/year
  interpretation/applicability questions with source/device evidence. Existing raw defaults,
  provisional alternatives and custom replacements remain usable meanwhile.
- [ ] Establish stronger durable log continuity/deduplication using controller
  identity, rollover and retention evidence. Matching anchors or unchanged history
  indices do not prove gap-free continuity.

## P2: writes, files and remote services

- [ ] Extend the implemented OP-166/181 framework and 12-field writable subset;
  implement OP-8/11 and remaining schema constraints: access/constraints,
  request budgets, partial results, uncertain remote effects and explicit no-replay
  behavior. Test failure before send, after effect and before acknowledgement.
- [ ] Implement OP-203 file command variants, handles, chunking, size limits and
  errata with explicit cancellation, close/recovery and resume rules. Bound remote
  handle lifetime to its valid generation and device context.
- [ ] Implement OP-24/205 and broadcast/shared-bus behavior with capability-specific
  framing, correlation, size limits, late replies and no-response completion rules.
  Test multiple logical devices and contention through the appropriate ownership seam.
- [ ] Determine supplemental ROC800 OP-200 v1 scope from authoritative definitions;
  related FB107 v2 material alone does not establish its wire contract (DEC-7).

## Transport and regression gaps

These extend existing lifecycle/ownership tests; they are not prerequisites for
claiming that already-covered cases are implemented.

- [ ] Exercise the actual SerialConnector through a Unix pseudo-terminal: fragmented
  frames, corruption, silence/timeouts, reopen, line configuration and cancellation.
  Specify Windows paired-port prerequisites and explicit skips. Validate physical
  RS-232/422/485 and shared-bus arbitration separately.
- [ ] Complete listener outage/rebind/repeated restart and refused/stalled connect,
  setup and late-success matrices beyond existing accepted-connection failure tests.
- [ ] Extend queued cancellation/deadline, recovery and teardown cases where gaps
  remain; use deterministic barriers and verify timing under loaded platforms.
- [ ] Make fixture failures observable and add generalized before-send/after-effect/
  before-acknowledgement controls for new state-changing services.
- [ ] Add standalone external-simulator mode and remaining opcode/form datasets;
  include identity/layout/schema/key/remote-handle changes that invalidate state.
- [ ] Preserve dependency responsibility: published etherbird owns supervision,
  admission, leases, recovery and awaited shutdown. Recheck extension contracts on
  upgrades rather than introducing competing lifecycle machinery.

## Physical interoperability

- [ ] Record available controller model, firmware, security mode, addresses and
  channel configuration; hardware acquisition can run alongside software work.
- [ ] Compare Ethernet exchanges with independent reference-client/capture evidence
  through the same Connector used by applications; exercise bench outage/login recovery.
- [ ] Validate serial adapters, pinouts, line configuration and RS-232/422/485 behavior.
- [ ] Extend the per-service/firmware matrix as features are added. Keep hardware
  tests opt-in and avoid compatibility claims beyond recorded evidence.

## Documentation and release gates

Maintain current feature contracts and test traceability as each slice lands;
completed documentation tasks remain in the changelog and release record.

- [ ] Add runnable examples for future transaction, file and routing APIs,
  including explicit recovery/progress and ownership semantics when implemented.
- [ ] Cross-check every baseline OP/PT/DEC row before claiming full coverage;
  selected catalog manifests and extension seams do not close the baseline.
For each subsequent release, review final scope/changelog, run hosted stable
Linux/macOS/Windows checks for the release commit, and verify the reviewed clean
package and publication dry-run. Completed 0.1.2 evidence is in [RELEASE.md](RELEASE.md).
- [ ] Backfill GitHub release pages for historical 0.1.0/0.1.1 if desired; their
  crates.io publication/tag evidence is already recorded. The 0.1.2 page is complete.
- [ ] Record physical evidence before making production compatibility claims.

Use stable Rust and ordinary experimental 0.x versions. The dependency remains
registry etherbird 0.4.2; no sibling path migration is pending. Publication is a
separate release action, not implied by completing this backlog.

## Recommended next slice

**Prepare the current feature set for release:** finalize the candidate version
and changelog, review the complete diff/package, and run hosted checks and the
clean-commit publication dry-run. Publication remains a separate release action.

After that release, **expand the audited writable catalog** beyond the delivered OP-166/181
[12-field subset](WRITES.md). Keep access/value constraints, conservative budgets,
explicit partial/uncertain outcomes and no automatic replay. Prioritize fields
with unambiguous source constraints; extend schema validation before admitting
cross-field or noncontiguous-enumeration fields. Add independent literals,
simulator state checks and external-consumer evidence with each subset.

OP-139 configured inventory and selected-point reads are implemented within the
[declared scope](HISTORY_INFORMATION.md). Continue transaction OP-206 when its
wire ambiguities have adequate evidence; complete remaining history schemas and
firmware/channel validation alongside parameter work.

**PT-106/107 temperature I/O reads are implemented** in the [declared subset](TEMPERATURE.md).
PT-108 reads and finite caller-driven polling are now implemented in the
[MVS subset](MVS.md); further schema work can target PT-109 completion or PT-110.
A Unix pseudo-terminal SerialConnector fixture is another focused validation slice.
Protected authentication/logout and identity/cache policy remain extension priorities
when adequate source evidence is available.

## Completion of the comprehensive objective

A scoped extensible release requires the applicable EXT acceptance, useful declared
built-ins, actionable missing-capability behavior, external-consumer guides and
software/package gates. Full baseline completion additionally requires the service
plan gate, all schema rows/manifests and the applicable transport/interoperability
criteria. Preserve outstanding security, variants, parameters and compatibility
limitations in release claims until their own evidence closes them.
