# Current specification decision register

Updated: 2026-10-09. Evidence baseline: public research reviewed 2026-10-08.
This register introduces no new wire evidence or physical validation. All DEC
parents remain open. It is the current delivery index; historical findings remain
in [first-pass research](AMBIGUITIES.md), [second-pass research](SECOND_PASS.md)
and the [systematic search ledger](SEARCH_RECORD.md).

## Status and disposition rules

Architecture policy: [all ambiguous device behavior is customizable](../EXTENSIBILITY.md#handling-ambiguity-during-delivery).
Known implementable alternatives become named predefined extensions using the
same public interfaces as caller implementations. A row marked user extension
does not exclude a future built-in preset; it means no such preset is selected yet.
Retained defaults must use the corresponding public boundary when that boundary
is delivered. Framework guarantees remain mandatory for presets and custom code.

For each row, implementation work must record its exact public seam, preset names
and evidence (or why none can yet be supplied), replacement example, and framework
prerequisites. Deferred rows require those prerequisites or scope/evidence decisions
before exposing behavior. Extensibility does not imply every row is currently
implementable, resolved, or eligible for unchecked user configuration.

Every row is **open** pending its closure evidence. Delivery dispositions allow
progress without claiming the underlying wire question is resolved:

- **Retained default:** existing behavior remains with its stated limits; other
  transport/firmware variants are not thereby verified.
- **Documented direction:** apply established distinctions without inferring
  missing device behavior.
- **User extension:** planned seam for callers with applicable knowledge. These
  seams are not implemented yet; affected built-in variants remain unavailable.
- **Deferred framework:** define ownership, validation or outcome contracts before
  exposing behavior. Extensions cannot bypass those contracts.
- **Deferred supplemental scope/evidence:** obtain applicable definitions/evidence
  before expanding the baseline or selecting built-in behavior.

No provisional codec is selected here. Each row needs applicable sources/firmware,
chosen behavior and independent fixtures before closure, with physical validation
tracked separately. All rows currently have **physical evidence: none**.
Stable `DEC-n.nn` IDs refine the parent IDs; historical DEC-5a/b/c/d labels remain
in question text. Append new IDs without renumbering existing rows.

## DEC-1 — framing budgets, CRC and serial recovery

Affected work: CORE-1; OP-24/203; EXT-2/4.
Evidence: [search assessment](SEARCH_RECORD.md#dec-1--framing-budgets-crc-and-serial-recovery) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-1.01 | Byte length field versus total ordinary-frame limit | Documented field capacity; actual safe device limit unresolved | Deferred framework | Earlier applicable protocol editions/errata or boundary captures; retain separate limits |
| DEC-1.02 | Routed total size and embedded overhead | Vendor wording inspected; no independent boundary evidence | Deferred framework | Chapter 7 revision comparison and routed client test/capture provenance |
| DEC-1.03 | Per-service limits, including file blocks | Service-specific text exists; do not apply one universal number | Deferred framework | Audit individual request AND response shapes against source/firmware |
| DEC-1.04 | Incoming Ethernet CRC behavior | Outgoing bytes/checking distinction documented; reply validity unverified | Retained default | Ethernet captures with CRC recomputation and model/firmware |
| DEC-1.05 | Partial serial frames and resynchronization | No usable wire recovery procedure located | Retained default | Serial implementation-owner code, timing/resync documentation or captured failures |

## DEC-2 — enhanced security and bootstrap

Affected work: OP-6/17; EXT-1.
Evidence: [search assessment](SEARCH_RECORD.md#dec-2--enhanced-security-and-bootstrap) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-2.01 | Wrapping cipher, key derivation, padding/encoding and credential construction | No applicable public procedure located | User extension: EXT-1 | Developer supplement, implementation source or official non-secret vectors |
| DEC-2.02 | ROC800 key lifetime/refresh after reconnect | Related-family timing clue only | User extension: EXT-1 | ROC800-specific procedure/capture; avoid cross-family deadline inference |
| DEC-2.03 | Firmware capability versus opt-in versus port security | Documented distinctions | Documented direction | Firmware-aware identification and permitted bootstrap reads |
| DEC-2.04 | Discovery before authentication | S23 says I/O-map access can be restricted; not a universal rule | User extension: EXT-1 | Per-opcode access matrix; ROCLINK/developer initialization workflow |
| DEC-2.05 | Inactivity/session expiration | Driver guidance exists; comprehensive ROC800 refresh rules unverified | User extension: EXT-1 | Applicable firmware security/session documentation |
| DEC-2.06 | Legacy/enhanced logout restoration | Field layouts exist; managed-client semantics remain a design task | Deferred framework | Verify device session consequences separately from client policy |

## DEC-3 — layout status and physical mapping

Affected work: OP-6/50; CORE-2; EXT-2/3.
Evidence: [search assessment](SEARCH_RECORD.md#dec-3--layout-status-and-physical-mapping) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-3.01 | Configuration flag versus reported layout status | Documented as distinct fields | Documented direction | Keep separate representations; source/table audit |
| DEC-3.02 | Slot/point mapping for 160/240/224 entries | Candidate mappings; legacy driver examples support only a subset | User extension: EXT-2/3 | Expanded-layout examples and independent selector replies |
| DEC-3.03 | CPU diagnostics/unused slots and end boundaries | Some textual examples; diagram conflicts remain | User extension: EXT-2/3 | First/last valid physical position and absent-module fixtures |
| DEC-3.04 | Driver logical syntax versus wire location | Driver abstraction must not be confused with wire numbering | Deferred framework | Verify conversion before passing a logical alias into a TLP |

## DEC-4 — occurrence times, clock and history indices

Affected work: OP-7/105/135–139; history PT; EXT-2/4.
Evidence: [search assessment](SEARCH_RECORD.md#dec-4--occurrence-times-clock-and-history-indices) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-4.01 | Opcode 105 five calendar bytes versus extra byte + epoch time | Competing candidates; neither controller-verified | User extension: EXT-2/4 | Known occurrence date/time raw bytes with firmware identity |
| DEC-4.02 | Clock year/leap-year disagreement | Family difference/copy-forward illustration; table-based current choice retained | Retained default | Applicable clock vectors; do not import old-family encoding |
| DEC-4.03 | Minute index 0-59 versus prose 0-60 | S21 documents 60 values, supporting 0-59 as candidate | User extension: EXT-2/4 | Per-opcode boundary/zero-count behavior; capacity alone not a command definition |
| DEC-4.04 | Expanded segment ranges | Hardware capability known, service applicability incomplete | User extension: EXT-2/3 | Per-opcode/firmware segment acceptance matrix |
| DEC-4.05 | Timezone/year reconstruction | Not established by absent fields | Documented direction | Explicit semantics in target manual or leave raw/unqualified |

## DEC-5 — command/layout variants

Affected work: OP-203/205/206; EXT-2/4.
Evidence: [search assessment](SEARCH_RECORD.md#dec-5--commandlayout-variants) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-5.01 | DEC-5a delete request command | 5 is strongest historical vendor candidate; fixture conflicts noted | User extension: EXT-2/4 | Applicable erratum or controller transcript; no destructive alternate-command fallback |
| DEC-5.02 | DEC-5b directory request/response command numbers | Distinct directions supported; newer response 64 clue | User extension: EXT-2/4 | Old/new paired requests/replies and firmware |
| DEC-5.03 | DEC-5b filename counts, continuation inputs and EOF | Unresolved | User extension: EXT-2/4 | Paging documentation or verified multi-page directory sequence |
| DEC-5.04 | DEC-5c commissioned response index width and following offsets | Parser chooses four bytes; vendor conflict persists | User extension: EXT-2/4 | Known-target response with exact bytes or erratum |
| DEC-5.05 | DEC-5d transaction request offset location | Sequential two-byte field is implementation clue | User extension: EXT-2/4 | Correct request layout from vendor/capture |
| DEC-5.06 | DEC-5d mandatory TLP versus tagged type/value framing | Implementation clue only | User extension: EXT-2/4 | Multi-field transaction bytes establishing boundaries |
| DEC-5.07 | DEC-5d split fields/unknown tags/continuation | No complete applicable behavior established | User extension: EXT-2/4 | Multi-page transaction transcript and supported type vocabulary |

## DEC-6 — errors and partial commits

Affected work: OP-255/166/167/180/181; CORE-3; EXT-4.
Evidence: [search assessment](SEARCH_RECORD.md#dec-6--errors-and-partial-commits) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-6.01 | Generic versus service-local error codes | Multiple primary driver lists differ; preserve unknown codes | Retained default | Per-service reconciliation including file/tunnel errors |
| DEC-6.02 | Actual parameter versus tuple-position offsets | Vendor context rules exist; applicability needs exact fixtures | User extension: EXT-4 | Independent error replies for 166/167 and 180/181 |
| DEC-6.03 | Which writes committed after multiple errors | No usable independent transcript located | Deferred framework | Documented atomicity/partial semantics or before/after captured exchange |
| DEC-6.04 | Busy/authentication/configuration errors and recovery | Code descriptions are not complete state transitions | Deferred framework | Firmware-specific error/session behavior |

## DEC-7 — absent services and historical editions

Affected work: Supplemental scope; OP-200 candidate.
Evidence: [search assessment](SEARCH_RECORD.md#dec-7--absent-services-and-historical-editions) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-7.01 | Opcode 200 ROC800 v1 | Existence supported; only related v2 wire table obtained | Deferred supplemental scope | Applicable v1 supplement/manual and post/get-response sequence |
| DEC-7.02 | Opcodes 19/80 | Limited client reserves them; applicability unverified | Deferred supplemental scope | Historical vendor inventory, not constants alone |
| DEC-7.03 | Opcodes 20/101 | Searches produced mixed point-type references, not applicable opcode definitions | Deferred supplemental scope | Separate opcode and point-type documentation; confirm current scope |
| DEC-7.04 | 2011 source and obsolete index entries | Referenced but not obtained | Deferred supplemental scope | Retry archive/publisher/distributor document collections |

## DEC-8 — supported family, subtype and update mode

Affected work: OP-6/17; history PT; EXT-1/2/3.
Evidence: [search assessment](SEARCH_RECORD.md#dec-8--supported-family-subtype-and-update-mode) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-8.01 | Family-specific enhanced-security firmware versions | Documented distinctions | Documented direction | Identify actual family/version before selecting behavior |
| DEC-8.02 | Subtype 0/1 interpretation | Vendor/parser conflict remains | User extension: EXT-2 | Applicable erratum or known Series 1/2 replies |
| DEC-8.03 | Firmware-update mode available services | Only broad limitation text found | User extension: EXT-1/2 | Exact safe-service/readiness matrix |
| DEC-8.04 | Series 2 history capacity versus service limits | Capacity corroborated; per-opcode ranges not complete | User extension: EXT-2/3 | Firmware/opcode applicability matrix |
| DEC-8.05 | Replacement identity/cache invalidation | Client design consequence, not a wire ambiguity | Deferred framework | Use verified identity/capability fields; define cache lifetime |

## DEC-9 — broadcast, direct connect and shared bus

Affected work: CORE-1; OP-24/224/225; EXT-2/4.
Evidence: [search assessment](SEARCH_RECORD.md#dec-9--broadcast-direct-connect-and-shared-bus) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-9.01 | Broadcast no-reply/group behavior | Basics documented | Deferred framework | Per-opcode applicability and sending semantics |
| DEC-9.02 | Direct-connect requests/reply address identity | Both serial and Ethernet driver defaults found; response identity unknown | User extension: EXT-2 | Captured direct/configured addressing pairs |
| DEC-9.03 | Shared serial-bus arbitration/unsolicited collisions | No applicable complete rule located | Deferred framework | Serial modem/driver docs and ROC800 collision behavior |
| DEC-9.04 | Unsolicited source identity and routing | Configured SRBX host guidance; reader/correlation policy still open | Deferred framework | Multi-device bus traces or precise vendor receive rules |

## DEC-10 — widths, conditional access and schema applicability

Affected work: All PT rows; OP-166/167/180/181; EXT-3.
Evidence: [search assessment](SEARCH_RECORD.md#dec-10--widths-conditional-access-and-schema-applicability) and its cited
[catalog sources](SOURCES.md).

| ID | Subquestion | Current conclusion | Delivery disposition | Evidence needed to close |
| --- | --- | --- | --- | --- |
| DEC-10.01 | PT-177/0 10 versus 40 bytes | Published driver correction; target firmware gate unverified | User extension: EXT-3 | Vendor table/version and independent value width |
| DEC-10.02 | Conditional writes and configuration mode | Tags/parameter-level conditions found; no universal rule established | User extension: EXT-3 | Each affected parameter's preconditions and device outcomes |
| DEC-10.03 | Firmware-dependent schema differences | Broad audit remains ongoing | User extension: EXT-3 | Per-parameter manifests for all 62 catalog rows |
| DEC-10.04 | Naming/description defects in other drivers | Indexed release-note lead only | Deferred evidence | Read original release notes before adopting corrections |
