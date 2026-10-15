# Comprehensive implementation roadmap

Status: open; reconciled 2026-10-15. This document tracks remaining work.
Implemented behavior belongs in [coverage](COVERAGE.md), executable evidence in
[requirements](REQUIREMENTS.md), and delivery history in the [changelog](../CHANGELOG.md).

## Current handoff — 2026-10-15

**0.1.2** is the release represented by this codebase. It adds PT-102/104/105 read coverage
(202 catalog parameters total), OP-105/108 history metadata and OP-135 single-point
history reads, including bounded paging and explicit partial progress.

The managed extension platform, explicit profiles/codecs, schema-backed reads,
discovery, typed logs and opt-in SRBX polling/acknowledgement are implemented.
See [extension acceptance](EXTENSIBILITY.md), [catalog](CATALOG.md),
[history metadata](HISTORY_METADATA.md), [single-point history](HISTORY.md) and
[logs/SRBX](LOGS.md) for contracts and limitations.

Version 0.1.2 local validation passes 168 all-feature/all-target tests, 16 doctests,
strict Clippy, formatting, fixture examples, package verification and an extracted
external consumer. Hosted CI evidence for the published release does not establish
0.1.2 CI success. Physical-controller and actual serial exchanges remain
unvalidated. [Release evidence](RELEASE.md) preserves exact provenance and gates.

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
| M0-E: extension platform | Open EXT-1/2/3 acceptance: protected discovery, authentication/logout, identity/cache policy, framing evidence and write planning | Managed contracts; source-backed procedures |
| M0: discovery/security foundations | Remaining CORE-1/2/3 and OP-6/7/17/50/100 variants/applicability; DEC-1/2/3/8 evidence | Explicit profiles and readiness strategies |
| M1: parameter access | Remaining PT batches, firmware/layout applicability and discovery/bootstrap policies | Schema metadata and explicit device context |
| M2: logs/SRBX | Stronger durable continuity evidence and remaining applicability/schema coverage | Identity, retention and index semantics |
| M3: history | OP-136/137/138/139/206, history PT batch; outstanding OP-105/108/135 interpretation evidence | History geometry, time/schema models; DEC-4/5 |
| M4: configuration/writes | OP-8/10/11/166/181, PT-99 and writable schemas | Access/budget validation and explicit uncertain outcomes |
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
- [ ] Implement OP-136/137/138/139/206 and their source-defined variants: multi-point
  ordering, time models, transaction tags/widths, bounds and retention behavior.
  Use contextual codecs/policies for unresolved forms and explicit progress for
  multi-exchange work; see [service packages](SPEC_WORK_PLAN.md).
- [ ] Resolve remaining OP-105 occurrence semantics and OP-135 minute-index
  interpretation/applicability with source/device evidence. Existing raw defaults,
  provisional alternatives and custom replacements remain usable meanwhile.
- [ ] Establish stronger durable log continuity/deduplication using controller
  identity, rollover and retention evidence. Matching anchors or unchanged history
  indices do not prove gap-free continuity.

## P2: writes, files and remote services

- [ ] Implement OP-8/166/181/11 and schema-aware write planning: access/constraints,
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

- [ ] Add runnable examples for future multi-point history, file and routing APIs,
  including explicit recovery/progress and ownership semantics when implemented.
- [ ] Cross-check every baseline OP/PT/DEC row before claiming full coverage;
  selected catalog manifests and extension seams do not close the baseline.
- [ ] Before publishing 0.1.2, review final scope/changelog, run hosted stable
  Linux/macOS/Windows checks for the release commit, and verify the reviewed clean
  package and publication dry-run. Local 0.1.2 checks are recorded in [RELEASE.md](RELEASE.md).
- [ ] Create the pending published-release GitHub release page when API authentication
  is available. Publication/tag evidence is already recorded separately.
- [ ] Record physical evidence before making production compatibility claims.

Use stable Rust and ordinary experimental 0.x versions. The dependency remains
registry etherbird 0.4.2; no sibling path migration is pending. Publication is a
separate release action, not implied by completing this backlog.

## Recommended next slice

**Implement OP-136 multi-point history reads**, building on OP-135's policies,
bounded exchanges and explicit progress. Review selector/order/size variants and
applicability in the service plan first; retain public policy/codec replacement
where interpretation remains ambiguous. Add spec-referenced independent fixtures,
simulator/TCP coverage, consumer examples and updated requirements together.

After OP-136, continue the remaining history services and history schemas toward
a coherent history capability, then prioritize schema-aware writes for well-defined
parameters with explicit partial/uncertain outcomes. Catalog expansion can proceed
alongside security, file and routing work; this order does not close their gaps.

**PT-106/107 temperature I/O catalog reads** are a useful independent alternative.
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
