# Systematic public-source search record

Current delivery conclusions and dispositions are indexed in
[DECISIONS.md](DECISIONS.md). This ledger preserves executed searches, evidence
assessments and remaining leads; changing a disposition is not new research.

Recorded **2026-10-08**; public-source audit of **DEC-1 through DEC-10**.
This completes the search record for the currently identified questions; it does
not claim that every question is resolved or that public information is exhausted.

[Decision work packages](../SPEC_WORK_PLAN.md#specification-decisions-and-unresolved-references) ·
[Source catalog S1-S24](SOURCES.md) · [First findings](AMBIGUITIES.md) ·
[Second findings](SECOND_PASS.md).

## Method and interpretation

This pass ran 48 exact web-search queries in 12 batches, listed below. Queries in
a batch returned a combined result set; the outcome is recorded at batch level,
not falsely attributed to an individual query. All returned result sets were
reviewed for relevant hits. Source reads and failed reads are separately recorded.

The earlier two passes are carried forward through their source records and
findings; their representative queries are not relabelled as newly executed exact
queries. Prior retrieval failures are marked as prior. No device access, vendor
contact, private portal access or execution of downloaded software occurred.

Evidence order: applicable vendor wire definition/erratum; vendor device/firmware
guidance; implementation-owner source/docs; related-device manuals; indexed
leads. A mirror of the same vendor text is one evidence lineage, not another
independent confirmation. Generated test bytes and parser agreement do not prove
device-specific deviations.

Outcome vocabulary:

- **Documented part:** applicable text establishes part of the answer.
- **Candidate:** corroboration supports a proposed correction; applicability or behavior remains unverified.
- **Clue:** an implementation choice without controller provenance.
- **No usable answer located:** inspected search results did not supply an applicable answer.
- **Retrieval failed:** the source contents were not obtained; this is not a negative search result.
- **Lead queued:** potentially useful hit was identified but its relevant contents have not been inspected.
- **Out of scope:** unrelated protocol/family, marketing material or application schema cannot establish the requested wire fact.

## Exact query ledger

All queries below were executed on the record date using the available web search
service. Quotation marks and `repo:` syntax are preserved as submitted; no claim
is made that the search engine enforced every operator.

| Query ID | Batch | Decision scope | Exact query |
| --- | --- | --- | --- |
| Q001 | G1 | DEC-1/9 | `"ROC Plus" "maximum" "255" "packet"` |
| Q002 | G1 | DEC-1/9 | `"ROC800" "direct connect" "240" response` |
| Q003 | G1 | DEC-1/9 | `"ROC Plus" "broadcast" supported opcodes` |
| Q004 | G1 | DEC-1/9 | `"ROC Plus" "serial" resynchronization` |
| Q005 | G2 | DEC-2 | `"ROC800" "GETSESSIONKEY" encryption algorithm` |
| Q006 | G2 | DEC-2 | `"ROC Plus" "session key" "expiration"` |
| Q007 | G2 | DEC-2 | `"ROC800" "enhanced security" "before login"` |
| Q008 | G2 | DEC-2 | `"ROC800" "wrapped" "password" SDK` |
| Q009 | G3 | DEC-3 | `"ROC Plus" "opcode 50" "240" "224"` |
| Q010 | G3 | DEC-3 | `"ROC800" "logical compatibility" "physical location"` |
| Q011 | G3 | DEC-3 | `site:support.ptc.com "ROC Plus" "Logical" "Location"` |
| Q012 | G3 | DEC-3 | `"ROC800" "opcode 50" "slot 27"` |
| Q013 | G4 | DEC-4 | `"ROC Plus" "opcode 105" "time" "five"` |
| Q014 | G4 | DEC-4 | `"ROC Plus" "opcode 7" "year" UINT16` |
| Q015 | G4 | DEC-4 | `"ROC Plus" "minute history" "59" "60"` |
| Q016 | G4 | DEC-4 | `"ROC800" "min_time_today"` |
| Q017 | G5 | DEC-5 | `"ROC Plus" "203" "delete" command 5 errata` |
| Q018 | G5 | DEC-5 | `"ROC Plus" "directory" "64" "count"` |
| Q019 | G5 | DEC-5 | `"ROC Plus" "205" "commissioned" index width` |
| Q020 | G5 | DEC-5 | `"ROC Plus" "206" "offset" "TLP"` |
| Q021 | G6 | DEC-6 | `"ROC Plus" "partial writes" errors` |
| Q022 | G6 | DEC-6 | `"ROC Plus" "255" "tuple" offset` |
| Q023 | G6 | DEC-6 | `"ROC Plus" "166" "read only" error` |
| Q024 | G6 | DEC-6 | `"ROC Plus" "error 53" firmware` |
| Q025 | G7 | DEC-7 | `"ROC Plus" "opcode 200" "version 1"` |
| Q026 | G7 | DEC-7 | `"ROC Plus" "opcode 19" "opcode 20"` |
| Q027 | G7 | DEC-7 | `"ROC Plus" "opcode 80" "opcode 101"` |
| Q028 | G7 | DEC-7 | `"D301180X012" "2011" archive` |
| Q029 | G8 | DEC-8 | `"ROC800" "subtype" "Series 1" "Series 2"` |
| Q030 | G8 | DEC-8 | `"ROC800" "firmware update mode" "opcode"` |
| Q031 | G8 | DEC-8 | `"ROC800" "Series 2" "13 segments" history` |
| Q032 | G8 | DEC-8 | `"ROC800" "firmware" "protocol" release notes 3.90` |
| Q033 | G9 | DEC-10 | `"ROC Plus" "Point Type 177" "40"` |
| Q034 | G9 | DEC-10 | `"ROC Plus" "R/W_CNDL"` |
| Q035 | G9 | DEC-10 | `"ROC800" "parameter" "conditional" write` |
| Q036 | G9 | DEC-10 | `"ROC Plus" "schema" errata firmware` |
| Q037 | G10 | DEC-1/9 | `"ROC Plus" "CRC" "Ethernet" replies` |
| Q038 | G10 | DEC-1/9 | `"ROC Plus" "store and forward" "255" maximum` |
| Q039 | G10 | DEC-1/9 | `"ROC800" "RS485" "SRBX" collision` |
| Q040 | G10 | DEC-1/9 | `"ROC Plus" "direct connect" response address` |
| Q041 | G11 | DEC-2/4/5 | `repo:cisagov/icsnpp-roc-plus "205" issue` |
| Q042 | G11 | DEC-2/4/5 | `repo:cisagov/icsnpp-roc-plus "105" timestamp` |
| Q043 | G11 | DEC-2/4/5 | `"ROC800" "enhanced security" github forks` |
| Q044 | G11 | DEC-2/4/5 | `"ROC Plus" "206" capture pcap` |
| Q045 | G12 | DEC-7/10 | `"ROC Plus" "R/W_CNDL" "configuration mode"` |
| Q046 | G12 | DEC-7/10 | `"ROC800" "conditional" "parameter" "write" mode` |
| Q047 | G12 | DEC-7/10 | `"ROC Plus" "opcode 20" licensing` |
| Q048 | G12 | DEC-7/10 | `"ROC Plus" "opcode 101" user defined` |

## Batch outcomes

| Batch | Sources/results reviewed | Outcome |
| --- | --- | --- |
| G1 | S1/S9/S12; Kepware/Schneider setup hits; HMS hit opened as S22 | Address/CRC/broadcast basics and product setup; no verified direct-connect response, bus arbitration or serial resynchronization algorithm |
| G2 | S1/S2/S5/S8 and mirrored family manuals; ROCLINK 2024 indexed lead | No applicable wrapping procedure or ROC800-specific key deadline; pre-login access rules remain incomplete |
| G3 | S1/S7/S12; PTC page opened as S19 | Layout/physical-location guidance; newer mappings not independently verified |
| G4 | S1/S4/S12; ROCLINK history leads | Existing time/year/index contradictions; no applicable captured min/max response |
| G5 | S1/S3/S12/S13; OpenEnterprise and ABB release-note indexed leads | Delete candidate retained; directory/tunnel/transaction ambiguity persists |
| G6 | S1/S18; deviceWISE opened as S20; Kepware PDF opened as S23 | Additional driver error descriptions; no verified partial-commit transcript |
| G7 | S4/S9/S12/S17; AutoSol mixed-family item tables | Passthrough cross-reference retained; no ROC800 v1 definition or verified 2011 copy |
| G8 | S1/S2/S15; Emerson firmware sheet opened as S21; DL8000/ROC800L and distributor hits | Capacity corroboration; subtype/update-mode service matrix not resolved |
| G9 | S1/S10/S18; Schneider schema page opened as S24; release-note leads | Width/access leads; application schema excluded as a wire catalog |
| G10 | S1/S9/S12; mixed-family serial/SRBX guidance | No independent controller reply CRC/identity or collision evidence |
| G11 | S6/S8/S13; Weatherford release and GitHub index hits | Generated fixture lineage remains; direct issue/fork/history retrieval failed; no new verified disputed-layout capture |
| G12 | S1/S3; mirrored ROCLINK and AutoSol tables | Conditional-write and supplemental-service leads; point type 20/101 hits are not opcode definitions |

Generic false positives (geology “roc”, bicycle/pedal products, unrelated SCADA
protocols, and unrelated GitHub issue numbers) were excluded. “Schema View” in S24
describes Geo SCADA database objects, not ROC parameter byte widths.

## Source inspection ledger

S IDs resolve to original URLs, editions and locations in [SOURCES.md](SOURCES.md).
“Carried forward” means inspected in the earlier recorded passes, not freshly
revalidated by this pass. None establishes physical interoperability.

| Sources | Inspection state | Relevant questions |
| --- | --- | --- |
| S1 | Carried forward; publisher PDF reopened this pass; targeted text searches for conditional access | Frame/address/service tables and PT fields |
| S2/S14 | Carried forward vendor security guide editions | Firmware, opt-in, credential policy |
| S3/S4/S5 | Carried forward related-family manuals | Occurrence time, clock, passthrough, key lifetime |
| S6/S13 | Carried forward parser/source/fixture inspection; S13 pinned | Competing layouts and generated-capture provenance |
| S7/S8/S9 | Carried forward implementation-owner pages | Mapping, sessions, packet budget |
| S10/S15/S16/S17/S18 | Carried forward release notes/product/gateway/client/error sources | Catalog correction, capability, implementation limitations |
| S11 | Prior indexed PDF excerpt only | Serial direct-connect setting defaults |
| S19 | Opened and read this pass | Legacy location examples; not all expanded layouts |
| S20 | Opened and read this pass | Error table/read-only exception; no commit transcript |
| S21 | Opened and read first-page history section this pass | 60 minute values and Series 2 capacity |
| S22 | Opened and read configuration text this pass; figures not independently decoded | SRBX host setup, explicit device identity/security |
| S23 | Opened/read relevant PDF sections this pass | Ethernet address defaults, restricted discovery |
| S24 | Opened/read this pass; excluded for wire metadata | Application database schema |

## Per-ambiguity assessment and closure evidence

Every named DEC question and additional subquestion raised by our findings appears
below. Confidence applies to the evidence, not to an implementation decision.

### DEC-1 — framing budgets, CRC and serial recovery

Queries: Q001, Q004, Q037-Q038; sources S1/S9/S12, and S22 for configuration context.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Byte length field versus total ordinary-frame limit | Documented field capacity; actual safe device limit unresolved | Earlier applicable protocol editions/errata or boundary captures; retain separate limits |
| Routed total size and embedded overhead | Vendor wording inspected; no independent boundary evidence | Chapter 7 revision comparison and routed client test/capture provenance |
| Per-service limits, including file blocks | Service-specific text exists; do not apply one universal number | Audit individual request AND response shapes against source/firmware |
| Incoming Ethernet CRC behavior | Outgoing bytes/checking distinction documented; reply validity unverified | Ethernet captures with CRC recomputation and model/firmware |
| Partial serial frames and resynchronization | No usable wire recovery procedure located | Serial implementation-owner code, timing/resync documentation or captured failures |

Closure requires independently sourced limits/behavior for the applicable transport
and firmware. A driver's own maximum request setting is not a device maximum.

### DEC-2 — enhanced security and bootstrap

Queries: Q005-Q008, Q043; sources S1/S2/S5/S6/S8/S14/S16/S23; queued L02/L03.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Wrapping cipher, key derivation, padding/encoding and credential construction | No applicable public procedure located | Developer supplement, implementation source or official non-secret vectors |
| ROC800 key lifetime/refresh after reconnect | Related-family timing clue only | ROC800-specific procedure/capture; avoid cross-family deadline inference |
| Firmware capability versus opt-in versus port security | Documented distinctions | Firmware-aware identification and permitted bootstrap reads |
| Discovery before authentication | S23 says I/O-map access can be restricted; not a universal rule | Per-opcode access matrix; ROCLINK/developer initialization workflow |
| Inactivity/session expiration | Driver guidance exists; comprehensive ROC800 refresh rules unverified | Applicable firmware security/session documentation |
| Legacy/enhanced logout restoration | Field layouts exist; managed-client semantics remain a design task | Verify device session consequences separately from client policy |

New S23 evidence means setup must tolerate protected discovery. Its troubleshooting
text does not establish that opcode 6 is always readable before login.
[S23: initialization/access](https://ptc-p-001.sitecorecontenthub.cloud/api/public/content/5abb97f509b14c28a57d9005aa9ac4ce?v=1de8eb20#page=46).

Closure requires a complete applicable crypto procedure with test vectors and a
bootstrap sequence. Product labels such as “Enhanced” do not establish it.

### DEC-3 — layout status and physical mapping

Queries: Q009-Q012; sources S1/S7/S13/S19/S23; queued L02.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Configuration flag versus reported layout status | Documented as distinct fields | Keep separate representations; source/table audit |
| Slot/point mapping for 160/240/224 entries | Candidate mappings; legacy driver examples support only a subset | Expanded-layout examples and independent selector replies |
| CPU diagnostics/unused slots and end boundaries | Some textual examples; diagram conflicts remain | First/last valid physical position and absent-module fixtures |
| Driver logical syntax versus wire location | Driver abstraction must not be confused with wire numbering | Verify conversion before passing a logical alias into a TLP |

S19 provides legacy field/diagnostic examples but also a limited range; it cannot
settle every newer mapping. [S19: driver location help](https://support.ptc.com/help/kepware/drivers/en/kepware/drivers/FISHERROC/logical-location-details.html).

Closure requires each layout's mapping and boundaries, not just its byte count.

### DEC-4 — occurrence times, clock and history indices

Queries: Q013-Q016, Q042; sources S1/S3/S4/S12/S13/S21; queued L02/L06.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Opcode 105 five calendar bytes versus extra byte + epoch time | Competing candidates; neither controller-verified | Known occurrence date/time raw bytes with firmware identity |
| Clock year/leap-year disagreement | Family difference/copy-forward illustration; table-based current choice retained | Applicable clock vectors; do not import old-family encoding |
| Minute index 0-59 versus prose 0-60 | S21 documents 60 values, supporting 0-59 as candidate | Per-opcode boundary/zero-count behavior; capacity alone not a command definition |
| Expanded segment ranges | Hardware capability known, service applicability incomplete | Per-opcode/firmware segment acceptance matrix |
| Timezone/year reconstruction | Not established by absent fields | Explicit semantics in target manual or leave raw/unqualified |

S21 independently describes fixed 60-value minute storage. This corroborates
capacity, not index-60 behavior in every opcode.
[S21: firmware sheet](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301156x012.pdf).

Closure must discriminate equal-length time interpretations by values, not length alone.

### DEC-5 — command/layout variants

Queries: Q017-Q020, Q041/Q044; sources S1/S3/S6/S12/S13; queued L01/L04/L05.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| DEC-5a delete request command | 5 is strongest historical vendor candidate; fixture conflicts noted | Applicable erratum or controller transcript; no destructive alternate-command fallback |
| DEC-5b directory request/response command numbers | Distinct directions supported; newer response 64 clue | Old/new paired requests/replies and firmware |
| DEC-5b filename counts, continuation inputs and EOF | Unresolved | Paging documentation or verified multi-page directory sequence |
| DEC-5c commissioned response index width and following offsets | Parser chooses four bytes; vendor conflict persists | Known-target response with exact bytes or erratum |
| DEC-5d transaction request offset location | Sequential two-byte field is implementation clue | Correct request layout from vendor/capture |
| DEC-5d mandatory TLP versus tagged type/value framing | Implementation clue only | Multi-field transaction bytes establishing boundaries |
| DEC-5d split fields/unknown tags/continuation | No complete applicable behavior established | Multi-page transaction transcript and supported type vocabulary |

Closure is separate for each row. Generated PCAPs cannot be promoted to empirical
firmware deviations; their generator and parser are the same evidence lineage.

### DEC-6 — errors and partial commits

Queries: Q021-Q024; sources S1/S6/S18/S20/S23.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Generic versus service-local error codes | Multiple primary driver lists differ; preserve unknown codes | Per-service reconciliation including file/tunnel errors |
| Actual parameter versus tuple-position offsets | Vendor context rules exist; applicability needs exact fixtures | Independent error replies for 166/167 and 180/181 |
| Which writes committed after multiple errors | No usable independent transcript located | Documented atomicity/partial semantics or before/after captured exchange |
| Busy/authentication/configuration errors and recovery | Code descriptions are not complete state transitions | Firmware-specific error/session behavior |

S20 repeats the read-only exception, but supplies no proof of a particular
partial commit. [S20: error table](https://docs.devicewise.com/Content/Products/GatewayDevelopersGuide/Devices/DeviceTypes/ROCPlus/ROC-Plus-driver-troubleshooting.htm).

Closure must preserve uncertainty; an absent error is not proof of successful commit.

### DEC-7 — absent services and historical editions

Queries: Q025-Q028, Q047-Q048; sources S1/S4/S9/S12/S17; queued L01/L07.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Opcode 200 ROC800 v1 | Existence supported; only related v2 wire table obtained | Applicable v1 supplement/manual and post/get-response sequence |
| Opcodes 19/80 | Limited client reserves them; applicability unverified | Historical vendor inventory, not constants alone |
| Opcodes 20/101 | Searches produced mixed point-type references, not applicable opcode definitions | Separate opcode and point-type documentation; confirm current scope |
| 2011 source and obsolete index entries | Referenced but not obtained | Retry archive/publisher/distributor document collections |

Closure requires a definition or authoritative exclusion; search silence cannot
justify declaring a service nonexistent. The 28-service baseline stays distinct
from supplemental ROC800 services.

### DEC-8 — supported family, subtype and update mode

Queries: Q029-Q032; sources S1/S2/S6/S15/S21/S22; queued L02/L08.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Family-specific enhanced-security firmware versions | Documented distinctions | Identify actual family/version before selecting behavior |
| Subtype 0/1 interpretation | Vendor/parser conflict remains | Applicable erratum or known Series 1/2 replies |
| Firmware-update mode available services | Only broad limitation text found | Exact safe-service/readiness matrix |
| Series 2 history capacity versus service limits | Capacity corroborated; per-opcode ranges not complete | Firmware/opcode applicability matrix |
| Replacement identity/cache invalidation | Client design consequence, not a wire ambiguity | Use verified identity/capability fields; define cache lifetime |

Closure requires a supported-device profile and explicit update-mode behavior.
Marketing capacity cannot silently enlarge a request range.

### DEC-9 — broadcast, direct connect and shared bus

Queries: Q002-Q003, Q039-Q040; sources S1/S11/S22/S23.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| Broadcast no-reply/group behavior | Basics documented | Per-opcode applicability and sending semantics |
| Direct-connect requests/reply address identity | Both serial and Ethernet driver defaults found; response identity unknown | Captured direct/configured addressing pairs |
| Shared serial-bus arbitration/unsolicited collisions | No applicable complete rule located | Serial modem/driver docs and ROC800 collision behavior |
| Unsolicited source identity and routing | Configured SRBX host guidance; reader/correlation policy still open | Multi-device bus traces or precise vendor receive rules |

S23 extends the earlier serial-setting evidence to Ethernet defaults; it still
does not prove reply source addressing.
[S23: communication settings](https://ptc-p-001.sitecorecontenthub.cloud/api/public/content/5abb97f509b14c28a57d9005aa9ac4ce?v=1de8eb20#page=23).

S22 explains configured SRBX hosts; this is setup guidance, not collision
arbitration evidence. [S22: configuration](https://support.hms-networks.com/hc/en-us/articles/27220985126034-Crimson-3-x-Emerson-ROC-Communications).

### DEC-10 — widths, conditional access and schema applicability

Queries: Q033-Q036, Q045-Q046; sources S1/S10/S18/S24; queued L02/L09/L10.

| Subquestion | Current assessment | Next public lead / evidence needed |
| --- | --- | --- |
| PT-177/0 10 versus 40 bytes | Published driver correction; target firmware gate unverified | Vendor table/version and independent value width |
| Conditional writes and configuration mode | Tags/parameter-level conditions found; no universal rule established | Each affected parameter's preconditions and device outcomes |
| Firmware-dependent schema differences | Broad audit remains ongoing | Per-parameter manifests for all 62 catalog rows |
| Naming/description defects in other drivers | Indexed release-note lead only | Read original release notes before adopting corrections |

Closure is parameter-by-parameter. This search record completes coverage of the
known research questions, not an audit of every parameter in the manual.

## Retrieval failures

| ID | Date/pass | Exact target/action | Result and remaining action |
| --- | --- | --- | --- |
| F01 | Current | [Emerson alternate baseline PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301180x012.pdf) open | Tool internal error; canonical S1 publisher URL succeeded |
| F02 | Current, also prior | [ROCLINK ROC800 PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301250x012.pdf) open | Tool internal error; L02 remains queued |
| F03 | Current | [CISA issue search](https://github.com/cisagov/icsnpp-roc-plus/issues?q=is%3Aissue+105) open | Tool internal error; cannot infer issue absence |
| F04 | Current | [CISA forks](https://github.com/cisagov/icsnpp-roc-plus/forks) open | Tool internal error; forks not inspected |
| F05 | Current | [Opcode 205 file history](https://github.com/cisagov/icsnpp-roc-plus/commits/main/analyzer/roc_plus_opcode_205.spicy) open | Tool internal error; complete history not inspected |
| F06 | Prior second pass | Archive CDX query for S1 URL, JSON/status-200/digest collapse | Failed; not retried this pass; archive avenue remains open |
| F07 | Prior first/second pass | Some raw/immutable GitHub files and publisher-linked PDF | Failed reads; five pinned parser/payload reads later succeeded (S13), others not assumed read |
| F08 | Prior first pass | PDF screenshot requests for disputed file tables | Failed; visual table confirmation remains unavailable |

A failed web tool request does not prove that a source is private or unavailable
to other clients. No retry success is claimed without a recorded read.

## Explicit remaining lead queue

These are **not completed inspections**. Search snippets are leads, not evidence
for adopting a technical correction.

| Lead | Applies to | Target/next action |
| --- | --- | --- |
| L01 | DEC-4/5/7 | Obtain verified 2011/earlier ROC Plus revision; retry archival collection via F06; compare original tables |
| L02 | DEC-2/3/4/8/10 | Retrieve ROCLINK D301250X012 from publisher/distributor; indexed February 2024 edition found, full applicable sections not inspected |
| L03 | DEC-2 | Weatherford CygNet LoginSec documentation/release pages surfaced; inspect implementation-owner details for algorithm/procedure |
| L04 | DEC-5 | ABB SCADAvantage 6.0.1 opcode 203 implementation note surfaced in an indexed mirror; locate/read publisher version |
| L05 | DEC-4/5/8 | CISA issue/fork/per-file history via F03-F05; inspect lineage and any device-provenance reports |
| L06 | DEC-4/8 | Older/alternate history docs and independently captured known-date minute/min/max exchanges; no verified capture found |
| L07 | DEC-7 | Publisher HART passthrough developer/module supplement for ROC800 opcode 200 v1 |
| L08 | DEC-8 | Exact firmware release notes and boot/update protocol supplement; distributors' document collections were indexed, not audited |
| L09 | DEC-10 | Geo SCADA 2023 August 2024 release-note lead concerning pulse-input descriptions; original release notes not inspected |
| L10 | DEC-3/10 | OpenEnterprise ROC/schema reference-guide leads; retrieve publisher original and distinguish model metadata from wire fields |

L03 discovery link:
[Weatherford release page](https://softwaredocs.weatherford.com/cygnet/98/Content/Topics/CygNet%20Software/What%27s%20New.htm).
L04 indexed lead:
[SCADAvantage release-note mirror](https://www.scribd.com/document/809480280/3CCA2024-003416-a-en-SCADAvantage-6-0-1-Release-Notes).
L09 indexed lead:
[Geo SCADA release-note mirror](https://www.scribd.com/document/788711861/Geo-SCADA-2023-Update-August-2024-86-8986-1-Release-Notes).
These links identify the queue; technical claims from uninspected snippets were
not adopted. Publisher originals remain preferred.

## Completion and exhaustion policy

The record is complete for this bounded pass when every DEC subquestion has
executed query IDs, inspected-source lineage, an assessment, and a remaining
lead or concrete closure requirement. That condition is met here.

A question is resolved only by applicable evidence sufficient to choose behavior.
A public-source avenue may be called exhausted only with its searched collections,
revisions, query variants, follow-ups and retrieval limitations explicitly bounded.
No DEC item receives that label in this record. In particular, queued originals,
unread forks/history and failed archives prevent an overall exhaustion claim.

Reopen the relevant row when a new edition, erratum, implementation or captured
exchange becomes available. Link the source and update confidence rather than
silently replacing a previous candidate. Do not resume disputed implementation
work merely because its search record is complete.


## Targeted follow-up — 2026-10-16

[Executed queries R16-01–09 and findings](FOLLOW_UP_2026_10_16.md) extend this
historical ledger for DEC-2/5/9. Source records are S25–28; no exhaustion claim.

- L02: publisher ROCLINK manual obtained and the named TCP/security sections
  inspected. Other layout/history/parameter sections remain future audit work.
- L03: release, ROCPlus data-group and LOGINSEC pages inspected. Support is confirmed;
  credential wrapping and ROC800 key lifetime remain missing. Device-page full read
  is not claimed after browser retrieval failures.
- L04: ABB publisher original obtained; OP-203 support note inspected, no wire details.
- L05: transaction per-file history and one pinned correction diff inspected;
  formatting-only change supplies no new wire/provenance evidence. Other diffs,
  issues and forks remain open leads.
- L01: renewed 2011/part-number searches did not retrieve a verified earlier manual.
  Archive collection remains open; this is not proof of absence.

Hardware validation is deferred outside the current software-completion scope;
source-definition research remains active. See [prepared questions](DEFINITION_REQUEST.md).
