# Comprehensive implementation roadmap

Status: open. Baseline: the initial implementation created on 2026-10-08.
This is the implementation backlog; [COVERAGE.md](COVERAGE.md) records current
protocol support and [HARDWARE.md](HARDWARE.md) records channel validation.

## Scope and working documents

The full implementation target is **ROC Plus D301180X012, October 2022,
Chapters 1-7**, including all services and parameter tables defined there.
[Original Emerson manual](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Earlier ROC protocols, other firmware manuals and undocumented extensions require
separate scope/evidence; a stale index reference is not a service definition.
The published Etherbird crate remains the lifecycle dependency.

Delivery now prioritizes an **extensible client with verified defaults**. Users
with additional device knowledge can supply authentication, protocol profiles,
schemas and typed operations/workflows without forking rocsteady. Complete built-in
coverage remains the long-term target; unresolved wire behavior can remain explicitly
user-supplied without blocking the platform or unrelated well-defined services.
An extension point does not establish full conformance or close its DEC/OP/PT item.

All ambiguous device behavior is intended to be customizable through supported
extension contracts. Sufficiently specified alternatives become named predefined
extensions, selectable through the same interfaces used by callers. Users can
replace these presets without forking. Unknown procedures remain user-supplied;
framework guarantees and supplemental scope decisions remain explicit prerequisites.
See [the ambiguity delivery policy](EXTENSIBILITY.md#handling-ambiguity-during-delivery).

Use these documents together:

| Document | Purpose |
| --- | --- |
| This roadmap | Delivery order, dependencies, product workflows, transport and release work |
| [Documentation guide](README.md) | Starting point, reading order and document responsibilities |
| [Current decision register](reference/DECISIONS.md) | Current conclusion, delivery disposition and closure evidence for every DEC subquestion |
| [Extensible client work plan](EXTENSIBILITY.md) | Early extension milestone, framework guarantees and external-consumer acceptance criteria |
| [Specification work packages](SPEC_WORK_PLAN.md) | All 28 opcode services, command/record variants, core protocol requirements and unresolved specification decisions |
| [Parameter schema work packages](SCHEMA_WORK_PLAN.md) | All 62 Chapter 3 point-type sections and the per-parameter completion contract |
| [Coverage](COVERAGE.md) | What is implemented today and its evidence/limitations |
| [Reference baseline](reference/README.md) | Source identity and revision discipline |
| [Ambiguity research](reference/AMBIGUITIES.md) and [sources](reference/SOURCES.md) | Evidence, confidence and follow-up for every DEC item |
| [Second-pass research](reference/SECOND_PASS.md) | Historical corrections, source independence and remaining public research avenues |
| [Systematic search record](reference/SEARCH_RECORD.md) | Exact queries, per-question outcomes, failed retrievals and remaining lead queue |
| [Hardware](HARDWARE.md) | Physical-channel prerequisites and validation sequence |

The detailed work packages are required parts of this roadmap. A broad checkbox
below closes only when its linked OP/PT/CORE variants close. Read the source table
before implementation; the work plan is not a replacement wire specification.
Update implementation, simulator, evidence and documentation together. Track
physical evidence independently from software completion, and keep externally
blocked requirements visible rather than treating them as complete.

Current software evidence after typed-session and layout-applicability fixes:
**32 tests** (14 codec/discovery tests and 18 integration tests), with all
features/targets, Clippy and formatting
passing locally. No physical controller or actual serial fixture has been tested.

The loopback simulator is runnable. Serial is compiled but not exercised through
a serial port. The CI workflow exists; successful cross-platform GitHub runs have
not been established. The crate has not been published.

## Completion rules

An item is complete only when its implementation and relevant checks are present.
For each new protocol service:

- Record the specification section, supported firmware forms, payload sizes and
  ambiguities. Resolve built-in behavior or explicitly assign it to a user extension;
  provisional codecs must state their assumptions rather than guess silently.
- Add independently derived request/response fixtures before the codec. Include
  truncation, wrong counts/echoes, boundary sizes and device-error cases.
- Implement typed requests/results and simulator behavior through the byte-stream
  seam. A raw request escape hatch does not count as typed service coverage.
- Exercise it over actual loopback TCP, including applicable recovery/cancellation.
  Keep at least one independent wire oracle so shared codec bugs cannot pass both
  client and simulator.
- Record automated validation separately from physical-device interoperability.
  Update coverage and examples together with the implementation.

Priority P0 establishes extension contracts and discovery/security foundations. P1 completes a useful
read-only client. P2 expands full protocol services. P3 establishes physical
interoperability and release readiness. Hardware acquisition can proceed alongside
software work; full protocol completion does not require purchasing hardware first.

## Delivery sequence and dependencies

Milestones are capability slices, not a reduction of the full target. Hardware,
cross-platform CI and serial fixtures run alongside service development.

| Milestone | Required work packages | Dependencies | Exit behavior |
| --- | --- | --- | --- |
| M0-E: extension platform | EXT-1 through EXT-5 in EXTENSIBILITY.md | Existing Session/Connector/Etherbird guarantees; explicit extension contracts | External consumers add auth, profiles, schemas and operations/workflows while managed ownership and recovery remain enforced |
| M0: usable discovery and profiles | CORE-1/2/3 foundations; OP-6/7/17/50/100; dispositions for DEC-1/2/3/8 | M0-E; evidence for built-in behavior or explicit custom profile | Identify supported device/port/layout, select configured auth/readiness strategy, restore setup or report a missing capability |
| M1: schema-backed parameter access | OP-167/180; schema infrastructure; foundation and basic I/O PT batches | M0 layout/version policy | Read common parameters without manually guessing types; distinguish missing, unsupported and rejected points |
| M2: logs and spontaneous reporting | OP-118/119/224/225; applicable communication/system schemas | M0; index/error/time models | Consume all alarm/event variants with checkpoints and acknowledge only consumed alarms, including while idle |
| M3: historical access | OP-105/108/135/136/137/138/139/206; history PT batch; DEC-4/5 dispositions | M0; time/schema models; verified or explicit custom codecs | Retrieve selected history/transaction forms with bounded paging, rollover and gap reporting; remaining forms stay visible |
| M4: configuration and writes | OP-8/10/11/166/181; PT-99; remaining writable catalog batches | Schema/access metadata; CORE-3 outcome model | Read/write tables and parameters with explicit partial/unknown results and no implicit replay |
| M5: file lifecycle and transfers | OP-203 supported forms; DEC-5 dispositions | EXT-4 ownership; uncertain-outcome model; verified or custom codecs | Supported file commands use bounded transfers and explicit handle/recovery rules; unsupported commands fail clearly |
| M6: remote and shared-channel operations | OP-24/205; broadcast CORE-1; network PT batch; DEC-1/5/9 dispositions | Framework framing/address/reader contracts; applicable profiles | Route/tunnel and broadcast within selected capabilities and their distinct completion contracts |
| M7: extensible release and continued conformance | EXT acceptance; declared OP/PT scope; hardware/API/packaging gates | Selected M0-M6 capabilities plus accurate evidence | Release verified defaults and supported extension APIs; full built-in baseline gate remains separate |

Full catalog work continues across M1-M7; all six schema delivery batches are
required. Enhanced authentication may remain externally blocked while independent
legacy-device work progresses. Users may supply it through EXT-1 once they know
the procedure; unresolved built-in authentication does not block the extensible
release, but its absence still prevents an unqualified full-baseline claim.
M2 and M3 can proceed independently after their foundations. M4 can start earlier
for fully specified schemas; no milestone permits automatic write replay.

For each slice: select stable work-package IDs, resolve relevant decisions, record
literal fixtures, implement codecs/API and simulator, verify transport/recovery
behavior, then update coverage and evidence. Do not close an entire opcode from
one successful frame exchange or one supported command.

## P0: discovery, authentication and protocol decisions

### Supported extension framework

- [ ] Complete EXT-1 authentication/readiness and generation-local restoration.
- [ ] Complete EXT-2 explicit protocol profiles and contextual codecs/policies.
- [ ] Provide named predefined extensions for sufficiently specified alternatives
  and prove an external consumer can replace one through the public interface.
- [ ] Complete EXT-3 schema providers and bounded schema-aware access.
- [ ] Complete EXT-4 typed custom operations and scoped exclusive workflows.
- [ ] Complete EXT-5 public contracts, external examples and contribution policy.
- [ ] Assign every known DEC subquestion a delivery disposition: verified built-in,
  explicit provisional codec, user extension, or deferred framework work.
- [ ] Verify extensions retain deadlines, bounded admission, cancellation poisoning,
  healthy device-error handling, no implicit replay and awaited shutdown.

Completion: external consumers implement all seams without crate-private access;
the ordinary API keeps usable defaults and missing capabilities fail clearly.
See [EXTENSIBILITY.md](EXTENSIBILITY.md) for required evidence and boundaries.

### Typed configuration and device identification

- [x] Implement opcode 6 request/response parsing with original and extended forms.
- [ ] Represent firmware-update mode, communication port, security mode, layout
  compatibility and device identity without assuming every field is present.
- [x] Preserve unknown extensions and reject truncated known forms.
- [x] Expose configuration/identity through Session and the managed Client.
- [ ] Add simulator configurations for supported response variants and setup failures.

Completion: independent fixtures cover supported variants; the client can identify
its supported target and distinguish TCP connectivity from usable protocol state.

Implemented discovery includes 97-byte pre-1.20, 98-byte original and 215-byte
extended payloads, with unknown suffixes retained. Five additional tests cover
field offsets, truncation, independent TCP bytes, login restoration and malformed
typed responses. Device type and security remain raw codes; subtype and layout
fields carry firmware-version caveats. Automatic supported-target/setup decisions
and firmware-update readiness policy remain open. Physical validation is outstanding.

### Authentication and session restoration

Research: [DEC-2/8 findings](reference/AMBIGUITIES.md). Firmware capability,
enhanced-security opt-in and port security are distinct. The ROC800/ROC800L
firmware thresholds differ; wrapping remains unresolved. Do not infer the protocol
from credential field length alone.

- [ ] Determine how to select legacy versus enhanced security from documented device
  configuration, including cases where initial reads require authentication.
- [ ] Resolve enhanced opcode 17 session-key/password handling from authoritative
  documentation. The current manual's field layouts alone must not be treated as
  a complete cryptographic procedure.
- [ ] Implement documented enhanced login, session-key handling and logout forms
  after that behavior is established.
- [x] Expose authentication/readiness strategies before requiring built-in enhanced
  login; provide a deterministic external strategy example and restoration tests.
- [ ] Detect session expiration on an otherwise healthy channel and define whether
  the affected operation fails or receives an explicitly permitted replay.
- [x] Distinguish explicit rejected/unsupported authentication from transient channel
  failures using terminal Authentication errors rather than repeated setup timeouts.
- [ ] Establish device-specific configuration/busy/expiration classifications beyond
  the explicit strategy error policy; these still need applicable wire evidence.
- [ ] Test rejected, expired and replacement sessions, secret-safe diagnostics, and
  readiness only after successful setup.

Completion: supported security modes are explicit, login restoration is tested,
and unsupported modes fail clearly. Passwords/session keys do not appear in Debug,
tracing, error text or stored fixtures.

### Resolve framing/address compatibility

- [ ] Audit general, per-opcode and routed message-size limits, including opcode 50
  layout variants. Separate the byte length-field capacity from device limits.
- [ ] Establish documented direct-connect address behavior and expected response
  identity. The current session assumes exact configured source/destination echoes.
- [ ] Decide how partial/corrupted serial frames are handled and where resynchronization
  belongs. Current behavior retires the channel rather than scanning for another frame.
- [ ] Verify incoming Ethernet CRC compatibility with an independent implementation
  or captured device responses; keep serial CRC enforcement explicit.
- [ ] Record manual inconsistencies and the resolution used by each codec.

Completion: boundary fixtures cover documented forms and each compatibility choice
has a source or an explicitly unresolved validation status.

## P1: read-only service and schema coverage

### Parameter schemas and I/O discovery

Required detail: [CORE-2, OP-50/100/167/180](SPEC_WORK_PLAN.md) and
[every PT row](SCHEMA_WORK_PLAN.md). The catalog includes access rules, conditional
writes, firmware variants, enums, bit fields, location rules and every parameter;
a list of point-type names alone is insufficient.

- [ ] Implement typed opcode 50 I/O layout and opcode 100 user-defined point metadata.
- [ ] Add a versioned parameter-schema interface/catalog for supported point types,
  retaining caller-defined schemas for user/vendor extensions.
- [ ] Distinguish physical location from logical numbering and use discovery to
  select the appropriate layout.
- [ ] Provide convenient typed parameter access without requiring users to guess widths.
- [ ] Expand fixtures for floating-point special values, fixed ASCII padding,
  mixed widths, maximum response budgets and unsupported schema entries.
- [ ] Interpret device error entries in operation context; preserve all raw entries.
  An error's second byte does not always represent a literal frame byte offset.

Completion: supported device schemas determine correct widths/locations, unknown
points remain representable, and malformed echoes/data cannot silently misalign values.

### Configurable read tables

- [ ] Implement typed opcode 10, including table selection/version and entry ranges.
- [ ] Validate count/range/version and variable-width results.
- [ ] Model table data and invalid-table responses in the simulator.

Completion: fixed fixtures and TCP tests cover successful reads, range failures,
version handling and response-size boundaries.

### Alarm and event logs

- [ ] Implement typed opcodes 118 and 119.
- [ ] Represent record fields, indices and device errors without dropping unknown data.
- [ ] Define bounded pagination, rollover and checkpoint behavior.
- [ ] Model empty logs, wraparound, unavailable indices and partial pages.
- [ ] Test resumption after channel failure and duplicate/repeated records.

Completion: applications can consume logs with explicit checkpoints and know when
continuity cannot be established.

### History and transaction reads

- [ ] Implement typed opcodes 105, 108, 135, 136, 137, 138, 139 and 206.
- [ ] Represent history metadata, time/index selectors, records and transaction types.
- [ ] Bound page/record allocations and make multi-request progress explicit.
- [ ] Handle retention, index rollover and reconnect between pages.
- [ ] Add simulator datasets and independent positive/malformed fixtures per opcode.

Completion: each listed opcode has typed coverage, applicable TCP tests and documented
continuity semantics. A complete wire exchange is not proof of a complete historical dataset.

## P2: additional protocol services

### Writes and partial results

- [ ] Implement typed clock setting (8), contiguous writes (166), arbitrary writes
  (181), and configurable-table writes (11).
- [ ] Validate request size, types and schema access/ranges where known.
- [ ] Represent partial writes and all returned error entries.
- [ ] Distinguish confirmed rejection from an unknown outcome after disconnect.
- [ ] Make multi-call operations retain exclusive channel ownership where required.
- [ ] Test simulator state changes, partial failures, cancellation after send and
  lost acknowledgement; preserve the no-implicit-replay contract.

Completion: each operation exposes meaningful acknowledgement/outcome semantics.
Reconnect must never imply that a write did not occur.

### Complete spontaneous-report workflow

- [ ] Implement typed acknowledgement (225) and connect it to alarm-log consumption.
- [ ] Decide whether full SRBX support needs a dedicated continuous reader; the
  baseline only reads signals during requests/heartbeats.
- [ ] Preserve reply routing with interleaved and repeated unsolicited signals.
- [ ] Define receiver lag, overflow and reconnect behavior without silent loss claims.
- [ ] Test signals while otherwise idle and acknowledgements after log rollover.

Completion: acknowledgement advances only to the last log entry actually consumed.
Stable receiver identity does not imply durable event delivery.

### File services

- [ ] Implement typed opcode 203 commands, firmware-dependent directory forms,
  names/paths, chunk offsets and response validation.
- [ ] Define file-handle lifetime across connection and device changes.
- [ ] Provide explicit transfer progress and resumability/integrity semantics.
- [ ] Model bounded files, chunking and invalid-handle/offset errors.
- [ ] Test partial transfers, uncertain writes and cleanup after cancellation.

Completion: supported commands have fixtures and TCP coverage; resource replacement
does not silently reuse an invalid remote handle.

Required detail: all OP-203 options, read/write/close/delete and both directory
forms in [the specification work plan](SPEC_WORK_PLAN.md). Resolve documented
command-number contradictions before choosing wire values.

### Routed, peer and no-response operations

- [ ] Implement store-and-forward (24), including embedded requests/responses and
  documented address paths.
- [ ] Implement typed peer-network tunnel services (205).
- [ ] Provide a distinct broadcast/no-response API; do not wait for an acknowledgement.
- [ ] Define shared serial-bus ownership and routing beyond the current one-device
  unicast session.
- [ ] Test nested size limits, invalid paths, unexpected source identities and
  no-response completion separately from timeout.
- [ ] Obtain authoritative ROC800 opcode 200 version-1 passthrough definitions,
  then decide supplemental HART/WirelessHART service scope and add command-level
  work packages if included. Related FB107 version-2 tables are insufficient.
  See [DEC-7 research](reference/AMBIGUITIES.md#dec-7--references-outside-the-baseline-inventory).

Completion: these APIs express their different response/correlation contracts and
the simulator exercises multiple logical destinations.

## P1–P3: transport and regression evidence

### Serial integration through the real connector

- [ ] Add a Unix pseudo-terminal fixture that opens through SerialConnector rather
  than substituting a Tokio duplex stream.
- [ ] Exercise Windows serial with an appropriate paired-port or physical fixture;
  document prerequisites and explicit skips.
- [ ] Test fragmented/corrupt frames, silence, reopen, configured line settings and
  cancellation leaving a partial exchange.
- [ ] Identify how RS-485 direction control/termination and shared-bus arbitration
  are supplied by the chosen hardware/driver.

Completion: the actual serial connector exchanges protocol frames in an integration
test. Compilation and in-memory stream tests alone do not establish serial behavior.

### Recovery and simulator hardening

- [ ] Test listener outage/rebind and repeated controller restart, beyond closing an
  accepted socket while the listener remains available.
- [ ] Test refused/stalled connection, stalled setup and late connection success.
- [x] Test malformed typed replies with otherwise valid frame/CRC and healthy-looking
  transport.
- [ ] Test queued cancellation without execution, overall deadline expiration while
  queued/recovering, and teardown with work still in flight.
- [ ] Test event receiver lag and sustained unsolicited traffic under one deadline.
- [ ] Make simulator failure/task errors observable to tests; unexpected fixture
  failures must not be mistaken for intended connection loss.
- [ ] Use deterministic synchronization where feasible and validate current short
  timing thresholds on loaded Windows/macOS/Linux CI.
- [ ] Add a standalone simulator mode useful to external clients, with clear
  configuration and repeatable datasets.
- [ ] Model every OP command/record variant and configured firmware/device form;
  introduce multiple logical devices for routing and shared-channel scenarios.
- [ ] Maintain an independent literal oracle for each new variant; simulator/client
  agreement alone cannot establish the wire format.
- [ ] Expose deterministic barriers for failures before send, after remote effects
  and before acknowledgement, plus connection/setup/operation counters.
- [ ] Test device identity, layout, schema, session key and remote handle changes
  across replacement; invalidate caches whose identity no longer holds.
- [x] Test exclusive ownership across compound workflows, with competing reads,
  heartbeat/unsolicited traffic, cancellation and stop from another clone.
- [x] Keep Etherbird responsible for lifecycle/admission/shutdown and rocsteady
  responsible for protocol/session semantics; validate against its published crate.

Completion: regression tests assert bounded outcomes and operation/setup counts,
including no replay of side effects, and fixture failures are diagnosable.

## P3: physical validation and release readiness

### Hardware validation

- [ ] Obtain access to a supported ROC800 controller and record model, firmware,
  security mode, unit/group addresses and port configuration.
- [ ] Establish Ethernet communication using the same Connector/session path.
- [ ] Compare identification, clock and representative parameter reads with an
  independent reference client/controller software or captured traffic.
- [ ] Validate outage recovery and session restoration on a bench controller.
- [ ] Validate the selected RS-232 or RS-422/485 channel with matching adapter,
  verified cable/pinout and line settings.
- [ ] Validate added services on appropriate firmware before expanding compatibility claims.

Completion: keep a reproducible validation record and a supported-device/firmware
matrix. Hardware-dependent tests should be explicitly opt-in; no-hardware CI remains usable.

### API, documentation and packaging

- [x] Review public API ergonomics, configuration validation and error/outcome types.
- [x] Document how applications should react to transport, protocol, authentication,
  queue and unknown-outcome errors.
- [ ] Add complete examples for credentials, serial, log/history pagination and state/events.
  Credentials, serial and state/events examples are delivered; log/history
  pagination examples remain blocked on their unimplemented typed services.
- [x] Compile README and usage examples as library doctests, including context lifetime checks.
- [x] Verify the declared Rust 1.88 minimum with default/all-feature/target/doctest
  checks on Windows and Linux; see [release evidence](RELEASE.md).
- [x] Establish successful Linux/macOS/Windows CI results, including optional serial.
- [x] Audit dependencies/licenses and packaged contents; validate cargo package.
- [x] Add changelog, explicit compatibility/versioning policy and release checklist.
- [x] Select the advertised release scope and keep unsupported services visible.
- [x] Publish the experimental read-only alpha after the chosen release criteria are
  met and publication is authorized: 0.1.0-alpha.1; see [release record](RELEASE.md).
- [ ] Provide runnable recovery examples for reads, authenticated setup, logs/history,
  files and routing; show state changes and explicit interrupted-operation outcomes.
  Read/authenticated-setup recovery is delivered; logs/history, files and routing
  remain future services. See [current examples](../README.md#concrete-examples).
- [x] Establish dependency upgrade regression checks for the published Etherbird
  crate, without requiring a sibling checkout for users or CI.
- [ ] Document ownership and cancellation for pagers, file handles, subscriptions,
  routed clients and broadcast sends; give bounded defaults and useful errors.
- [ ] Cross-check service variants and all per-parameter manifests against the manual
  before claiming full baseline completion; account for every unresolved decision.

Completion: a user can tell exactly which services, firmware and channels are
supported, reproduce the documented examples, and distinguish simulator coverage
from hardware evidence.

## Recommended next slice

Continue M0-E: the first managed slice is implemented (see
[contracts](MANAGED_EXTENSIONS.md)). Complete remaining authentication applicability/logout,
remote-handle and lifecycle acceptance tasks, then add profiles and schema
providers with external examples and failure/recovery tests. Catalogue each known
ambiguity's extension disposition before committing to disputed built-in behavior.

Then continue OP-50/100 discovery and schema-backed reads using verified defaults
or explicitly supplied profiles. Opcode 6 parsing is already implemented;
automatic compatibility/security/readiness decisions remain open. Research and
promotion of proven community solutions continue alongside service delivery.

A useful read-only release can precede complete protocol coverage if its advertised
scope is explicit. Completing that release does not close the broader backlog.

## Completion of the comprehensive objective

The extensible product release requires EXT acceptance, a useful declared set of
built-ins, explicit missing-capability behavior, external-consumer guides, and the
applicable transport/packaging gates. It does not require implementing every
unresolved behavior ourselves. User-supplied behavior remains separately labelled.

The full-baseline gate in [SPEC_WORK_PLAN.md](SPEC_WORK_PLAN.md), all schema rows
and per-parameter manifests, and the transport/release criteria above must be
satisfied. Keep separate implementation and interoperability states. A scoped
release can ship earlier; unresolved security, missing variants, uncatalogued
parameters or unverified compatibility must remain visible in that release's claims.
