# ROC Plus specification work plan

Implementation target: [Emerson D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf),
Chapters 1-7. Source inventory reviewed 2026-10-08. This plan complements
[OUTSTANDING_WORK.md](OUTSTANDING_WORK.md), [current coverage](COVERAGE.md), and
[the parameter inventory](SCHEMA_WORK_PLAN.md). Read the source tables before coding:
these tasks identify work and acceptance criteria, not replacement wire specifications.

Delivery prioritizes [supported extension points](EXTENSIBILITY.md) with verified
defaults. Users may implement behavior whose wire details they know. This inventory
continues to track full built-in conformance; an available extension seam does not
close a service variant or resolve its ambiguity. Extensible-product release gates
are separate from the full-baseline gate below.

## Tracking and closure

IDs `OP-n`, `PT-n`, `CORE-n`, `DEC-n`, and `EXT-n` are stable references for commits,
issues, fixtures and release notes. Split variants beneath the parent ID; do not
close a parent because one variant works. “Partial” means implemented subset,
not conformance. “Open” may still have raw frame access.

For EACH service variant retain an evidence row:

`ID/variant | source section/table | device/firmware | request/response limits | codec/API | simulator | independent fixtures | TCP tests | serial tests | physical evidence | unresolved decisions`

Additionally record the provider/profile and disposition: verified built-in,
explicit provisional codec, user-supplied extension, or unavailable. Keep extension
framework contract tests separate from evidence for a particular device codec.

Initially all physical evidence is **none**, and actual serial exchange evidence
is **none**. Evidence descriptions below identify current tests, not hardware proof.
When adding a variant, supply exact test/file links and either evidence or an explicit
unverified status for every column. API + simulator + independent positive/negative
fixtures + TCP behavior are required for implementation completion. Firmware and
channel compatibility claims additionally require their corresponding evidence.

## Complete opcode inventory

The 28 services in section 2.1 each have a work package below. Reserved/device-originated
services must have the correct direction; raw request access is not typed coverage.

| ID | Manual section | Service | Implementation | Existing automated evidence |
| --- | --- | --- | --- | --- |
| OP-6 | 2.2 | Configuration | Partial | tests/configuration.rs; configuration tests in tests/integration.rs |
| OP-7 | 2.3 | Clock read | Partial | clock literal and independent peer tests |
| OP-8 | 2.4 | Clock write | Typed acknowledged unicast implemented; broadcast/physical open | tests/clock_set.rs; CLOCK_SET.md |
| OP-10 | 2.5 | Table read | Typed mixed-schema batches, bitwise revision checks and generation-bound mapped plans implemented | tests/configurable_tables.rs; CONFIGURABLE_TABLES.md |
| OP-11 | 2.6 | Table write | Schema-opt-in mapped writes, per-batch definition checks and explicit shared write outcomes implemented | tests/configurable_tables.rs; CONFIGURABLE_TABLES.md |
| OP-17 | 2.7 | Authentication/logout | Partial | legacy_login_packs_password_little_endian_and_optional_access; login/recovery tests |
| OP-24 | 2.8; Ch. 7 | Store and forward | Open | None |
| OP-50 | 2.9 | I/O layout | Partial: both selectors and all three explicit lengths; DEC-3 device applicability remains open | tests/discovery.rs; tests/discovery_simulator.rs; per-variant evidence in CATALOG.md |
| OP-100 | 2.10 | User-defined information | Implemented command 11; firmware exchanges unverified | tests/discovery.rs; tests/discovery_simulator.rs; per-variant evidence in CATALOG.md |
| OP-105 | 2.11 | Min/max | Partial: table-offset reply, raw occurrences/default and replaceable provisional codecs; DEC-4.01 open | tests/history_metadata.rs; tests/history_metadata_simulator.rs; HISTORY_METADATA.md |
| OP-108 | 2.12 | History tags/index | Baseline 1–20-point reads implemented; expanded applicability and continuity not inferred | tests/history_metadata.rs; tests/history_metadata_simulator.rs; HISTORY_METADATA.md |
| OP-118 | 2.13 | Alarms | Partial: fixed records, bounded pages and explicit checkpoints; continuity/physical limits remain | tests/logs.rs; tests/logs_simulator.rs; LOGS.md |
| OP-119 | 2.14 | Events | Partial: all enumerated forms/tags, incomplete values retained; continuity/physical limits remain | tests/logs.rs; tests/logs_simulator.rs; LOGS.md |
| OP-135 | 2.15 | Single history point | Partial: all selectors, index-only reads and bounded paging; explicit minute policies; DEC-4 applicability/continuity limits remain | tests/history.rs; tests/history_simulator.rs; HISTORY.md |
| OP-136 | 2.16 | Multiple history points | Partial: all three selectors, time-major rows, bounded paging and explicit progress; DEC-4 applicability/continuity limits remain | tests/multi_history.rs; tests/multi_history_simulator.rs; MULTI_HISTORY.md |
| OP-137 | 2.17 | Day indices | Table-based start/count lookup and raw invalid daily index implemented; DEC-4.05/06 and physical applicability open | tests/day_history.rs; tests/day_history_simulator.rs; DAY_HISTORY.md |
| OP-138 | 2.18 | Day history | Values/point-255 timestamps and explicit indexed alternative implemented; snapshot/physical limits remain | tests/day_history.rs; tests/day_history_simulator.rs; DAY_HISTORY.md |
| OP-139 | 2.19 | History information | Command-0 inventory and command-1 nonempty selected lists implemented, both timestamp shapes, bounded ranges and opt-in inventory checks; applicability/physical limits remain | tests/history_information.rs; tests/history_information_simulator.rs; HISTORY_INFORMATION.md |
| OP-166 | 2.20 | Contiguous writes | Schema-aware bounded batches implemented; 18 audited writable fields; broader applicability remains open | tests/writes.rs; WRITES.md |
| OP-167 | 2.21 | Contiguous reads | Partial | opcode_167_validates_count_range_and_echoed_block_header; TCP reads |
| OP-180 | 2.22 | Arbitrary reads | Partial | opcode_180_request_has_count_and_ordered_tlps; opcode_180_response_echoes_tlp_before_each_value; TCP reads |
| OP-181 | 2.23 | Arbitrary writes | Schema-aware ordered TLP/value batches implemented; explicit conservative request policy and partial outcomes; physical applicability remains open | tests/arbitrary_writes.rs; tests/writes.rs; WRITES.md |
| OP-203 | 2.24 | Files | Open | None |
| OP-205 | 2.25 | Network tunnel | Open | None |
| OP-206 | 2.26 | Transactions | Partial: typed raw inventory/data pages, bounded reassembly, explicit provisional field layouts; DEC-5/physical open | tests/transactions.rs; TRANSACTIONS.md |
| OP-224 | 2.27; Ch. 6 | SRBX signal | Partial: interleaved/stable notifications and explicit bounded idle polling; durable/physical delivery not established | tests/srbx.rs; tests/srbx_simulator.rs; Chapter 6 vector; LOGS.md |
| OP-225 | 2.28; Ch. 6 | SRBX acknowledgement | Partial: consumed-anchor verification, typed index/empty reply and staged outcomes; caller ingestion/identity and physical evidence remain | tests/srbx.rs; tests/srbx_simulator.rs; Chapter 6 vector; LOGS.md |
| OP-255 | 2.29 | Device errors | Partial | error_response_preserves_all_codes_and_offsets_including_unknown_codes; rejection tests |

## Core work packages

### CORE-1: framing, addressing and broadcast — sections 1.2-1.4

- [x] Encode/decode unicast headers, exact lengths and CRC.
- [x] Serialize exchanges and validate configured reply address/opcode.
- [ ] Resolve total-frame versus payload limits for ordinary, routed and special services (DEC-1).
- [ ] Specify legal source/destination units/groups, direct-connect behavior, unused route addresses, wrong-source responses and broadcast group matching.
- [ ] Implement a separate no-response send API for supported broadcasts; distinguish local send completion from remote execution.
- [ ] Validate broadcast applicability for every service; never automatically convert unicast calls to broadcasts.
- [ ] Test boundary sizes, fragmented/coalesced packets, interleaving, corrupt/partial frames and peer closure across each transport.
- [ ] Establish serial resynchronization/reopen policy and bus ownership; do not consume late replies after cancellation.

### CORE-2: values and TLPs — sections 3.1-3.4, Chapters 4-5

- [x] Implement basic data widths, little-endian typed values and zero-seed CRC.
- [ ] Audit every source data type and context-specific type tag; event and transaction tags must not share an assumed numbering.
- [ ] Verify signed limits, IEEE float/double special values, signed zero, fixed ASCII padding/termination, binary bit fields and TLP widths.
- [ ] Define time interpretation, epoch/range, device-local clock versus timestamps and unavailable values without inventing timezone behavior.
- [ ] Implement and verify all 62 schema rows in SCHEMA_WORK_PLAN.md.
- [ ] Separate invalid caller requests from malformed replies and unsupported schema/firmware; preserve original bytes where interpretation is unknown.

### CORE-3: error semantics — section 2.29

- [x] Preserve multiple and unknown error code/offset pairs.
- [x] Keep valid device rejections from forcing reconnection.
- [ ] Catalogue every documented error, including service-local additions, with context-specific descriptions and recovery implications.
- [ ] Interpret offsets as actual parameter numbers for 166/167 and tuple positions for 180/181; retain the raw byte.
- [ ] Distinguish authentication expiration, rejected credentials, unsupported service and transient busy/configuration conditions.
- [ ] Represent partial-write results without assuming every non-reported parameter succeeded.
- [ ] Test empty/odd error payloads, multiple errors, unknown codes, contextual offsets and inconsistent error tables (DEC-6).

## Discovery, clock and security

### OP-6: configuration

- [x] Provide original/extended parsing and managed/session access.
- [x] Preserve unknown/reserved bytes and reject truncated known layouts.
- [ ] Audit pre-1.20 forms and unavailable fields against additional evidence; lengths alone do not establish firmware.
- [ ] Define firmware-update/unknown-mode readiness policy and supported-device policy.
- [ ] Connect discovery to port/security and layout selection without assuming reserved bytes are meaningful.
- [ ] Test setup in update mode, unknown device/security, unavailable discovery before login and changed identity after recovery.

### OP-7 and OP-8: clock

- [x] OP-7 typed read with calendar checks and independent wire evidence.
- [ ] OP-7 audit year/day-of-week semantics and document the conflicting older illustration.
- [x] OP-8 implement its distinct write payload and empty acknowledgement; validate calendar inputs without transmitting a read-only weekday field.
- [x] Test boundary calendars, device rejection, successful set/read-back, cancellation and lost acknowledgement with unknown write outcome.
- [ ] Test broadcast clock setting through CORE-1 separately from acknowledged unicast.

### OP-17: authentication forms

- [x] Legacy operator/password payload, optional access level, redacted password Debug and reconnect login replay.
- [ ] Validate the optional legacy access field against configured port security.
- [ ] Implement legacy logout as its own request and define whether explicit logout leaves the managed client stopped, unauthenticated or ready to relogin.
- [ ] Implement session-key request/response validation.
- [ ] Resolve enhanced credential wrapping/encoding before implementing enhanced login and enhanced logout (DEC-2).
- [ ] Define session-key generation lifetime, refresh, secret storage/redaction, timeout/expiration and operator changes.
- [ ] Obtain a documented firmware/security discriminator and handle protected discovery without an authentication deadlock.
- [ ] Return actionable setup/authentication failures; test warm/cold restart, firmware change, timeout, rejection and reconnect.
- [ ] Never replay an application side effect merely because authentication has been restored.

### OP-50 and OP-100: discovery

- [x] OP-50 implement both selectors (point types and logical numbers), with explicit selector request context and correlated layout results.
- [x] Cover all three reported layouts (160/240/224 entries), absent modules, diagnostics and explicitly selected source-based location mapping. Physical interoperability/DEC-3 resolution remains open.
- [x] Detect observed changes across repeated selectors/configuration and invalidate snapshot-derived catalogs after reconnect; avoid silently combining generations. No atomic device snapshot is claimed.
- [ ] Resolve contradictory compatibility labels/examples (DEC-3) before automatically selecting a layout.
- [x] OP-100 implement command 11, start/count validation (including zero), echoed header and returned point-type classifications.
- [x] Preserve unknown classifications and distinguish program/user-defined/ROC/absent entries.
- [x] Bound paging at point-type/range and profile request/response limits; test start/end/count errors.
- [x] Do not infer a complete parameter schema from opcode 100 classification.

Exact limits, codec/profile choices, independent fixtures, TCP/simulator evidence,
firmware assumptions and explicit lack of serial/hardware evidence are recorded in
[CATALOG.md](CATALOG.md#service-evidence). Profile/codec extensibility does not resolve
the contradictory DEC-3 prose/diagrams. Full catalog rows remain in SCHEMA_WORK_PLAN.md.

## Parameter and table access

### OP-167 and OP-180: reads

- [x] Typed contiguous/arbitrary reads, echo/count/width validation and response budgets.
- [x] OP-180 schema-aware access and batch planning with both request/response limits; independent TCP, budget and confirmed-progress evidence in tests/profiles_schema.rs.
- [ ] Add schema-aware contiguous OP-167 access and batch planning; current schema plans use OP-180.
- [ ] Audit zero counts, maximum ranges, duplicate/reordered TLPs, sparse parameters, unknown types and mixed widths.
- [ ] Verify partial/error behavior and context-specific error offsets through independent fixtures.
- [ ] Test malformed typed data on valid frames and reconnect between planned batches; expose incomplete progress.

### OP-166 and OP-181: writes

OP-166/181 now have separately bounded packing, schema-aware planning, empty-ack
validation, exclusive batch progress and explicit uncertain/device-error outcomes
with no replay. The built-in writable subset has 18 fields; see
[write contracts and manifest](WRITES.md). Broader parameter constraints and
physical applicability remain open; software evidence does not resolve DEC-6.

- [x] Implement separately bounded contiguous and arbitrary write codecs/APIs.
- [x] Validate access, exact types/widths, numeric bounds, byte counts and empty acknowledgement for the declared subset; unresolved conditional fields remain blocked.
- [x] Preserve raw multiple errors and acknowledged/uncertain/unattempted batch progress without per-parameter commit inference.
- [x] Keep one exclusive workflow lease; exercise lost/malformed acknowledgements, deadlines, queued cancellation, generation/profile rejection and no replay for both opcodes.
- [x] Exercise cross-point ordering/duplicates, independent literals/TCP and simulator state/read-back with the extracted public API.
- [ ] Complete remaining parameter-level conditions, noncontiguous enumerations, before-send/active-cancellation/stop failure matrices and firmware/device applicability evidence.
- [ ] Establish controller partial-commit and error-offset evidence (DEC-6); fixture partial effects do not close this acceptance.

### OP-10 and OP-11: configurable tables

- [x] OP-10 typed selection/range, bitwise table version and schema-backed ordered values.
- [x] Use opaque generation-bound snapshots, compare full definitions/revisions and require explicit rediscovery after changes/replacement; no automatic cache.
- [x] OP-11 typed mapped writes using checked metadata, separate write opt-in and explicit result/outcome semantics; external edits can race because the request has no revision field.
- [x] Integrate all 45 PT-99 definition read fields; preserve raw revision bits without ordering. PT-99 configuration writes and richer catalog metadata remain separate.
- [x] Test empty/invalid ranges, slot 43/parameter 44, changed definitions, mixed widths, budgets, device errors, partial/uncertain effects and lifecycle cases.

## Logs and spontaneous reporting

### OP-118: alarms

- [x] Implement index/count header, index-only requests, record budgets and fixed record boundaries.
- [x] Decode packed SRBX/set-clear/type flags while preserving unknown types and all raw bits.
- [x] Support no-alarm, parameter, FST, user-text and user-value records.
- [ ] Complete contextual parameter-alarm reason interpretation. Source labels 0-33 and raw TLPs are available; stored values always use Float, not schema widths.
- [x] Preserve reserved/raw data and define raw TIME/optional printable text interpretation.
- [x] Test empty markers, all variants, count mismatches, ring-boundary paging, observed anchor changes and concurrent growth.
- [ ] Establish durable rollover/gap detection if controller evidence supplies a stronger identity/sequence contract; modulo indices and matching bytes alone cannot establish it.

### OP-119: events

- [x] Implement index/count header, index-only requests and fixed record boundaries.
- [x] Support no-event, parameter-change, system, FST, user, power-lost, clock-set and calibration-verification records.
- [x] Implement event-specific tags, old/new overlap rules, raw operator IDs, system reasons and unknown variants. Values larger than the ten-byte storage remain explicitly incomplete.
- [x] Follow all enumerated event kinds despite the five-format prose.
- [x] Test every variant/tag, malformed values, ring-boundary paging, changed anchors and partial progress through explicit recovery. Undetectable complete rollovers remain a documented limit.

### OP-224 and OP-225: complete SRBX workflow — Chapter 6

- [x] Route interleaved signals through a receiver stable across replacement.
- [x] Provide opt-in caller-driven index polling with interval plus call-deadline bounds; no permanent competing reader.
- [x] OP-225 implement consumed-index acknowledgement and validate its empty reply.
- [x] Link acknowledgement to caller-consumed checkpoints, verify their raw anchor, and never substitute the unread current index. Caller ingestion completeness/identity remain explicit contracts.
- [x] Model pending status and equality-based index comparison, including newer alarms before acknowledgement and rollover.
- [x] Test repeated/missed/lagged/idle signals, rollover, lost replies, explicit recovery, cancellation/deadlines and Chapter 6 fixtures.
- [x] Document advisory subscriptions, bounded polling, lag and undetectable continuity limits.

## Historical and transactional data

All history APIs need explicit cursors, bounded allocation, rollover/gap detection,
and documented snapshot limitations. Resolve segment/firmware applicability against
PT-124-145; never silently assume all firmware exposes the same segment range.

### OP-105 and OP-108: point metadata

- [x] OP-105 decode archival method/TLP/current value and today/yesterday extrema, preserving raw occurrence times; explicit replaceable provisional interpretations available.
- [ ] Resolve conflicting occurrence-time lengths/representations in Table 2-14 (DEC-4).
- [x] OP-108 implement ordered point/tag results and raw common periodic index for the baseline bounded request.
- [x] Test count bounds, echoed points, padded tags, unknown archival methods and index changes between metadata replies.

See [history metadata contracts](HISTORY_METADATA.md) for exact limits, defaults,
public replacement seams and the source/evidence of each provisional choice.

### OP-135: single-point history

- [x] Implement minute, periodic and daily value selectors.
- [x] Implement periodic and daily timestamp selectors.
- [x] Validate requested/returned counts and selected current-index bounds; support count-zero index-only behavior.
- [x] Distinguish float samples from timestamps; provide TableHistoryPolicy default, explicitly provisional ProseMinuteHistoryPolicy and public caller policies.
- [ ] Resolve the underlying minute-index discrepancy with device/firmware evidence (DEC-4.03); software presets do not close it.
- [x] Test maximum page sizes, missing points, known retention boundaries and interrupted/replaced generations without implicit resume.

### OP-136: multiple-point history

- [x] Implement time-major result ordering with timestamp plus point values for each period.
- [x] Validate point/period ranges, returned element counts and combined response budgets.
- [x] Model zero-period/index-only and invalid-request zero-element behavior.
- [x] Test shape/count mismatches, rollover, sparse/unavailable points and bounded pagination.

See [multi-point history contracts](MULTI_HISTORY.md) for exact limits, policy/codec
replacement, confirmed failure progress and independent evidence. Physical applicability,
minute-index interpretation and durable continuity remain open (DEC-4.03/04).

### OP-137 and OP-138: day selection

- [x] OP-137 decode periodic start/count and optional-valid daily index/count.
- [x] OP-138 implement periodic/daily values and special point 255 timestamp requests.
- [x] Define day/month lookup without assuming a year that is absent from the request.
- [x] Handle missing day, multiple daily entries, zero daily entries and oversized day responses.
- [x] Provide a documented fallback to paged indexed reads when a day cannot fit in one reply.

See [day-history contracts](DAY_HISTORY.md): defaults follow Tables 2-20/21,
ReplyCodec/profile/Operation replacements remain public, indexed reads are opt-in
and preserve confirmed progress. Equal mappings do not prove a snapshot; physical
applicability and DEC-4.05/06 remain open.

### OP-139: history information commands

- [x] Command 0: configured-point inventory, count and segment echo.
- [x] Command 1: explicit point list with minute/periodic/daily selection.
- [x] Command 1: separate timestamp-disabled and timestamp-enabled response shapes.
- [x] Validate point-period products, ordering, current index and optional timestamp widths.
- [x] Test configured-point changes, invalid counts and pagination while the history ring advances.

See [history-information contracts](HISTORY_INFORMATION.md) for named defaults,
public replacements, nonempty-selection policy, exact wire/profile budgets and
partial progress. Command 0 has no paging contract; firmware/physical applicability
and DEC-4 geometry evidence remain open.

### OP-206: transaction commands

- [x] Command 1: list transactions, description, payload size, number/date pairs and continuation flag.
- [x] Define storage-offset advancement without assuming index zero means oldest; durable rollover/retention evidence remains open.
- [x] Command 2: read raw transaction data by number/byte offset with length/continuation validation; explicit caller payload size and bounded reassembly.
- [x] Implement its own type-tag vocabulary for numeric, raw fixed string, binary, TLP and time fields through explicitly selected provisional layouts.
- [x] Preserve raw records and explicitly reject unknown type tags whose width cannot be established; never guess the next field boundary.
- [x] Reassemble split fields, preserve progress on missing-record errors and reject no-progress continuation; same-length overwrites remain undetectable.
- [ ] Resolve command 2 offset/TLP wording inconsistencies (DEC-5) using independent evidence.

## Files and remote destinations

### OP-203: file command variants

- [ ] Open command: implement every documented option (read, write, create, update, truncate/create), fixed path/name fields and returned descriptor.
- [ ] Read command: descriptor, requested offset, returned size/offset/count and EOF behavior.
- [ ] Write command: descriptor, file size, offset, byte count and returned committed offset.
- [ ] Close command: descriptor lifetime, command acknowledgement and explicit cleanup.
- [ ] Delete command: both files/directories, path validation and irreversible outcome semantics; resolve conflicting command number first (DEC-5).
- [ ] Directory command: older firmware form and newer form, including different response command byte, request fields and continuation.
- [ ] Resolve directory paging/count ambiguities and null-delimited filename validation.
- [ ] Model open-file cap, invalid handles/options/paths/offsets, full filesystem and directory creation restrictions.
- [ ] Test zero-length files, EOF/chunk bounds, reconnect, abandoned descriptors and uncertain writes/deletes.
- [ ] Implement streaming transfer helpers with progress, bounded memory, cancellation cleanup and explicit resume/integrity policy.

### OP-24: store and forward — Chapter 7

- [ ] Implement path encoding for each supported hop count, zero-filled unused destinations and embedded opcode/data.
- [ ] Separate no intermediate acknowledgement from the eventual routed application response; do not treat routing as fire-and-forget by default.
- [ ] Validate outer and embedded identities, path, opcode, sizes and routed device errors.
- [ ] Implement an exclusive routed exchange contract and define ownership across multi-device operations.
- [ ] Test a multi-device fixture, every hop count, unreachable destinations, malformed paths and late routed replies.
- [ ] Resolve total-message limit and obsolete embedded clock examples before advertising conformance.

### OP-205: peer-network tunnel

- [ ] Implement network ID, commissioned destination and embedded operation request/response.
- [ ] Resolve commissioned-index width/offset contradiction before coding (DEC-5).
- [ ] Validate embedded sizes, correlation and device errors without adding an embedded CRC.
- [ ] Model absent/booting/unavailable network module, pending tunnel and SPI failure.
- [ ] Test nested errors, cancellation, late replies and writes with unknown outcome.
- [ ] Integrate applicable PT-172-175 discovery while retaining explicit destination access.

## Specification decisions and unresolved references

Use [the current decision register](reference/DECISIONS.md) for per-subquestion
delivery dispositions and closure evidence. Update it when a conclusion changes;
keep the research records as dated evidence history.

The current per-question research ledger is
[reference/SEARCH_RECORD.md](reference/SEARCH_RECORD.md). It covers every DEC
item and its identified subquestions; completing the record does not resolve the
wire ambiguity or establish that public sources are exhausted.

Online research for all DEC items is recorded in
[reference/AMBIGUITIES.md](reference/AMBIGUITIES.md), with sources in
[reference/SOURCES.md](reference/SOURCES.md). Research is separate from closure:
documented facts, candidates and parser clues must not be treated as hardware evidence.
[Second-pass research](reference/SECOND_PASS.md) adds stronger historical delete
evidence and qualifies parser clues: its traces are generated, its opcode 105
interpretation competes with calendar bytes, and some fixtures conflict internally.
Before selecting history ranges, account for documented Series 2 capacity and
verify each opcode's applicability. Public sources are not declared exhausted.

Each decision needs a source, affected variants, chosen behavior, fixture evidence
and hardware status. If evidence is missing, keep the affected behavior explicitly
unsupported, provisional, or explicitly user-supplied through a supported seam;
an unchecked decision must not disappear into code. Assign an EXT delivery path
without marking the DEC resolved. Framework ownership/framing/cancellation behavior
must still be defined before allowing an extension to participate.

| Unresolved area | Intended extension path | Framework requirement still to establish |
| --- | --- | --- |
| Enhanced login and protected discovery | EXT-1 authentication/readiness strategy | Bounded setup, per-generation state, actionable errors |
| Occurrence-time, directory, tunnel and transaction codecs | EXT-2 profile / EXT-4 typed operation | Raw contextual replies, mandatory validation, decoder failure policy |
| Vendor/firmware schemas and conditional metadata | EXT-3 provider | Lookup precedence, widths/budgets, local versus remote errors |
| File/routed compound operations | EXT-4 exclusive workflow with applicable profile | Ownership, heartbeat/reader interaction, handle generation and partial progress |
| Direct-connect address compatibility | Narrow EXT-2 correlation policy | Defined allowed identities; no blanket bypass of reply checks |
| Partial-write outcomes | EXT-4 operation-specific interpretation | Retain uncertainty and no implicit replay; a profile cannot prove remote commits |

- [ ] DEC-1: reconcile byte-field capacity with general and routed total-message limits and service-specific budgets.
- [ ] DEC-2: obtain complete enhanced-security procedure and reliable firmware/security selection; field sizes do not define cryptography.
- [ ] DEC-3: reconcile opcode 50 compatibility labels, slot examples and mapping formulas.
- [ ] DEC-4: reconcile opcode 105 occurrence-time definitions; document clock-year and history-index inconsistencies.
- [ ] DEC-5: resolve opcode 203 delete numbering/directory forms, opcode 205 commissioned index widths/offsets, and opcode 206 data wording.
- [ ] DEC-6: reconcile error tables with service-local errors and context-dependent offsets; record inaccurate cross-references.
- [ ] DEC-7: investigate index/error references to services such as 19, 20, 80, 101 and 200 that are absent from the section 2.1 service inventory.
  Obtain authoritative supplemental definitions before expanding scope; do not invent codecs from stale index entries.
  Research found an applicable-family claim that opcode 200 v1 is supported by
  ROC800, but only a related FB107 v2 layout. Obtain the ROC800 v1 definition,
  identify post/get-response lifecycle and framing, and explicitly decide supplemental
  passthrough scope. Opcode 200 must not be dismissed as merely a stale reference.
- [ ] DEC-8: establish supported device families, firmware feature gates and firmware-update behavior.
- [ ] DEC-9: identify which operations support broadcast and define shared-bus/unsolicited-source handling.
- [ ] DEC-10: audit schema-table inconsistencies and conditional write behavior as each PT row is implemented.
  Include the documented driver correction for PT-177/0 width as a regression lead;
  verify it against the target firmware rather than importing another driver's catalog.

## Current software-completion scope — 2026-10-16

Physical validation is deferred by the project owner until hardware or an external
tester is available. It does not gate current software completion. Required software
closure still includes source-audited APIs, simulator behavior, independent positive/
negative wire fixtures, lifecycle/transport tests, annotated requirements and package
checks. Unresolved definitions remain active research, tracked in the decision register.
The full-conformance/physical-compatibility gate below remains a distinct future claim.

## Full-baseline completion gate

This is the gate for full built-in baseline claims. It does not block a scoped
extensible release meeting [EXT acceptance](EXTENSIBILITY.md) and accurately
identifying supported defaults and user-supplied capabilities.

- [ ] Every OP variant above has an implementation evidence row and satisfies the closure contract.
- [ ] Every PT row and every parameter within its table is accounted for.
- [ ] CORE and DEC items are resolved or explicitly documented as externally blocked; blocked items prevent an unqualified full-conformance claim.
- [ ] Multi-request workflows retain progress and communicate uncertain/gapped outcomes across recovery.
- [ ] The supported-device/firmware/channel matrix is backed by recorded interoperability evidence.
- [ ] Documentation distinguishes implementation completion from firmware/channel validation and lists any remaining restrictions.

PT-106/107 schema reads now use the existing OP-180 service contract; see
[TEMPERATURE.md](TEMPERATURE.md) and the schema work plan. This adds no new
opcode variant or temperature write support.

PT-108 source-backed reads and example-owned caller-driven polling use existing
OP-50/180 contracts; [MVS.md](MVS.md) records source conditions and the explicit
4088 counter assertion. No new opcode or state-changing command is implemented.
