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
| OP-8 | 2.4 | Clock write | Open | None |
| OP-10 | 2.5 | Table read | Open | None |
| OP-11 | 2.6 | Table write | Open | None |
| OP-17 | 2.7 | Authentication/logout | Partial | legacy_login_packs_password_little_endian_and_optional_access; login/recovery tests |
| OP-24 | 2.8; Ch. 7 | Store and forward | Open | None |
| OP-50 | 2.9 | I/O layout | Open | None |
| OP-100 | 2.10 | User-defined information | Open | None |
| OP-105 | 2.11 | Min/max | Open | None |
| OP-108 | 2.12 | History tags/index | Open | None |
| OP-118 | 2.13 | Alarms | Open | None |
| OP-119 | 2.14 | Events | Open | None |
| OP-135 | 2.15 | Single history point | Open | None |
| OP-136 | 2.16 | Multiple history points | Open | None |
| OP-137 | 2.17 | Day indices | Open | None |
| OP-138 | 2.18 | Day history | Open | None |
| OP-139 | 2.19 | History information | Open | None |
| OP-166 | 2.20 | Contiguous writes | Open | None |
| OP-167 | 2.21 | Contiguous reads | Partial | opcode_167_validates_count_range_and_echoed_block_header; TCP reads |
| OP-180 | 2.22 | Arbitrary reads | Partial | opcode_180_request_has_count_and_ordered_tlps; opcode_180_response_echoes_tlp_before_each_value; TCP reads |
| OP-181 | 2.23 | Arbitrary writes | Open | None |
| OP-203 | 2.24 | Files | Open | None |
| OP-205 | 2.25 | Network tunnel | Open | None |
| OP-206 | 2.26 | Transactions | Open | None |
| OP-224 | 2.27; Ch. 6 | SRBX signal | Partial | published_srbx_and_ack_vectors_decode_and_encode_exactly; stable receiver test |
| OP-225 | 2.28; Ch. 6 | SRBX acknowledgement | Open | Literal frame only; no typed service |
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
- [ ] OP-8 implement its distinct write payload and empty acknowledgement; validate calendar inputs without transmitting a read-only weekday field.
- [ ] Test boundary calendars, device rejection, successful set/read-back, cancellation and lost acknowledgement with unknown write outcome.
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

- [ ] OP-50 implement both selectors (point types and logical numbers), with separate codecs and correlated layout results.
- [ ] Cover all three reported layouts (160/240/224 entries), absent modules, diagnostics and location mapping.
- [ ] Detect changed layout between the two queries and after reconnect; avoid silently combining incompatible snapshots.
- [ ] Resolve contradictory compatibility labels/examples (DEC-3) before automatically selecting a layout.
- [ ] OP-100 implement command 11, start/count validation (including zero), echoed header and returned point-type classifications.
- [ ] Preserve unknown classifications and distinguish program/user-defined/ROC/absent entries.
- [ ] Bound paging at point-type/range and response limits; test start/end/count errors.
- [ ] Do not infer a complete parameter schema from opcode 100 classification.

## Parameter and table access

### OP-167 and OP-180: reads

- [x] Typed contiguous/arbitrary reads, echo/count/width validation and response budgets.
- [ ] Add schema-aware access and batch planning with both request/response limits.
- [ ] Audit zero counts, maximum ranges, duplicate/reordered TLPs, sparse parameters, unknown types and mixed widths.
- [ ] Verify partial/error behavior and context-specific error offsets through independent fixtures.
- [ ] Test malformed typed data on valid frames and reconnect between planned batches; expose incomplete progress.

### OP-166 and OP-181: writes

- [ ] Implement separately bounded contiguous and arbitrary write codecs/APIs.
- [ ] Validate schema access, conditional access, values, byte counts and acknowledgement shape.
- [ ] Model single and multiple errors, mixed writable/read-only ranges and partial remote changes.
- [ ] Distinguish confirmed rejection/acknowledgement from unconfirmed outcome after send.
- [ ] Provide explicit batch progress; preserve exclusive ownership when a workflow requires it.
- [ ] Test changed simulator state, disconnect before/after send and cancellation; no implicit replay.

### OP-10 and OP-11: configurable tables

- [ ] OP-10 typed selection/range, table version and schema-backed ordered values.
- [ ] Define cache/version invalidation when table definitions change or a replacement device appears.
- [ ] OP-11 typed writes using current table metadata and explicit result/outcome semantics.
- [ ] Integrate PT-99 definitions; preserve version representation rather than treating float versions as ordinary sortable integers.
- [ ] Test empty/invalid ranges, boundary entries, changed table definitions, variable widths and invalid-table errors.

## Logs and spontaneous reporting

### OP-118: alarms

- [ ] Implement index/count header, index-only requests, record budgets and fixed record boundaries.
- [ ] Decode packed SRBX/set-clear/type flags while preserving unknown bits/types.
- [ ] Support no-alarm, parameter, FST, user-text and user-value records.
- [ ] Catalogue parameter-alarm reason codes and contextual TLP/value interpretation.
- [ ] Preserve reserved/raw data and define timestamp/text decoding.
- [ ] Test empty logs, all variants, count mismatches, wraparound, overwritten checkpoints and concurrent log growth.

### OP-119: events

- [ ] Implement index/count header, index-only requests and fixed record boundaries.
- [ ] Support no-event, parameter-change, system, FST, user, power-lost, clock-set and calibration-verification records.
- [ ] Implement event-specific data-type tags, old/new values, operator IDs, system reasons and unknown variants.
- [ ] Reconcile prose claiming five formats with the enumerated event kinds; do not omit kinds based on that prose.
- [ ] Test every variant, malformed tagged values, ring rollover and gaps during recovery.

### OP-224 and OP-225: complete SRBX workflow — Chapter 6

- [x] Route interleaved signals through a receiver stable across replacement.
- [ ] Establish reception while idle: continuous reader or an explicitly bounded polling contract.
- [ ] OP-225 implement alarm-index acknowledgement and validate its empty reply.
- [ ] Link acknowledgement to consumed alarm checkpoints; never acknowledge unseen records.
- [ ] Model device pending status and its index comparison, including alarms arriving during retrieval.
- [ ] Test repeated signals, missed/lagged signals, rollover, lost acknowledgement, recovery and published workflow fixtures.
- [ ] Document that stable subscriptions do not guarantee durable event delivery.

## Historical and transactional data

All history APIs need explicit cursors, bounded allocation, rollover/gap detection,
and documented snapshot limitations. Resolve segment/firmware applicability against
PT-124-145; never silently assume all firmware exposes the same segment range.

### OP-105 and OP-108: point metadata

- [ ] OP-105 decode archival method/TLP/current value and today/yesterday extrema and occurrence times.
- [ ] Resolve conflicting occurrence-time lengths/representations in Table 2-14 (DEC-4).
- [ ] OP-108 implement ordered point/tag results and common periodic index.
- [ ] Test count bounds, echoed points, padded tags, unknown archival methods and index changes during reads.

### OP-135: single-point history

- [ ] Implement minute, periodic and daily value selectors.
- [ ] Implement periodic and daily timestamp selectors.
- [ ] Validate requested/returned counts and current index; support documented index-only behavior.
- [ ] Resolve minute-index prose/table discrepancy and distinguish float samples from timestamps.
- [ ] Test maximum page sizes, missing points, retention boundaries and replacement between pages.

### OP-136: multiple-point history

- [ ] Implement time-major result ordering with timestamp plus point values for each period.
- [ ] Validate point/period ranges, returned element counts and combined response budgets.
- [ ] Model zero-period/index-only and invalid-request zero-element behavior.
- [ ] Test shape/count mismatches, rollover, sparse/unavailable points and bounded pagination.

### OP-137 and OP-138: day selection

- [ ] OP-137 decode periodic start/count and optional-valid daily index/count.
- [ ] OP-138 implement periodic/daily values and special point 255 timestamp requests.
- [ ] Define day/month lookup without assuming a year that is absent from the request.
- [ ] Handle missing day, multiple daily entries, zero daily entries and oversized day responses.
- [ ] Provide a documented fallback to paged indexed reads when a day cannot fit in one reply.

### OP-139: history information commands

- [ ] Command 0: configured-point inventory, count and segment echo.
- [ ] Command 1: explicit point list with minute/periodic/daily selection.
- [ ] Command 1: separate timestamp-disabled and timestamp-enabled response shapes.
- [ ] Validate point-period products, ordering, current index and optional timestamp widths.
- [ ] Test configured-point changes, invalid counts and pagination while the history ring advances.

### OP-206: transaction commands

- [ ] Command 1: list transactions, description, payload size, number/date pairs and continuation flag.
- [ ] Define offset order after rollover without assuming index zero means oldest.
- [ ] Command 2: read a transaction by number and byte offset with length/continuation validation.
- [ ] Implement its own type-tag vocabulary for numeric, string, binary, TLP and time fields.
- [ ] Preserve or explicitly reject unknown type tags whose width cannot be established; never guess the next field boundary.
- [ ] Handle fields split between pages, vanished/overwritten transactions and no-progress continuation.
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
