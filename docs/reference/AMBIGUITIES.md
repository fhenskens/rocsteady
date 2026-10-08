# Specification ambiguity research

Research date: **2026-10-08**. Scope: roadmap DEC-1 through DEC-10.
Use the [current decision register](DECISIONS.md) for delivery dispositions and
current conclusions; this document retains the first-pass evidence narrative.
The [systematic search record](SEARCH_RECORD.md) is the current research coverage
ledger for every DEC subquestion, including later findings and uninspected leads.
This records the first pass. [Second-pass findings](SECOND_PASS.md) strengthen
delete-command evidence, introduce a competing occurrence-time interpretation,
establish generated-fixture provenance and add device-capability constraints.
Read both before using any first-pass candidate as a specification decision.
[Source catalog, editions and retrieval limits](SOURCES.md);
[decision checklist](../SPEC_WORK_PLAN.md#specification-decisions-and-unresolved-references).

This is a research record, not a protocol change. Confidence describes the cited
finding; it does not establish compatibility with physical hardware. No bench
tests or captured controller replies were obtained. All DEC parent items remain
open because each contains further requirements.

- **Documented:** explicit applicable vendor statement.
- **Corroborated candidate:** supporting evidence exists, but applicability or wire behavior needs verification.
- **Implementation clue:** another author's code or driver behavior.
- **Unresolved:** the research did not establish an answer.

## Decision summary

| Decision | Research result | Remaining gate |
| --- | --- | --- |
| DEC-1 | Separate frame/service budgets; CRC policy partly documented | Routed limits and controller boundary fixtures |
| DEC-2 | Firmware/opt-in policy documented; wrapping unresolved | Algorithm, test vectors and protected discovery |
| DEC-3 | Layout selection partly clarified | Independent mapping/layout fixtures |
| DEC-4 | Five-byte occurrence time is a candidate; old clock format belongs to another family | ROC800 min/max and minute-index evidence |
| DEC-5 | Delete 5 strengthened; directory/tunnel/transaction clues recorded | Resolve each subcase with applicable wire evidence |
| DEC-6 | Generic error catalog remains insufficient | Per-service catalog and partial-write evidence |
| DEC-7 | Opcode 200 is real; other index-only references unresolved | ROC800 v1 passthrough definition and supplemental scope decision |
| DEC-8 | Family-specific versions documented; parser subtype disagreement found | Device identification and update-mode policy |
| DEC-9 | Broadcast basics and direct-connect driver defaults found | Reply identity, bus ownership and per-op applicability |
| DEC-10 | Concrete PT-177 width correction found | Audit all rows against source and firmware |

## DEC-1 — message budgets and Ethernet CRC

**Findings:** Elecsys documents a 255-byte maximum packet and suggests keeping
retrieved data to 230 bytes. Its older driver guidance is supporting evidence,
not a universal modern limit. [S9: RediGate guide](https://elecsysdocs.atlassian.net/wiki/spaces/IDG/pages/296262/Protocol_ROC).

The baseline's PT-91 parameter 35 says Ethernet ignores CRC checking but still
requires transmitted CRC bytes. [S1: PT-91/35](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf#page=95).

**Assessment:** sizing remains unresolved. CRC transmission is documented, but
this does not prove that device-generated Ethernet reply CRCs are invalid or absent.

**Implementation direction:** maintain separate byte-field capacity, per-service
budget and routed total-frame budget. Retain outbound CRC and explicit incoming
policy; do not silently change the current default from this research.

**Next evidence:** boundary exchanges for ordinary/routed/file operations;
Ethernet captures with validated incoming CRCs and serial line-setting tests.

## DEC-2 — enhanced authentication

**Findings:** Emerson's deployment guide lists ROC800 3.90, ROC800L 1.70 and FB107
2.00 separately. Firmware installation and enhanced-security opt-in are distinct;
opt-in is irreversible. It specifies 3-30 character IDs, 8-32 character passwords,
case-insensitive IDs and case-sensitive passwords, and encrypted credential
transmission. Port security remains separately configured.
[S2: sections 1.1-1.2, 2.1, 2.3-2.4](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301912x012.pdf).

The baseline identifies enhanced enable at TLP (91,0,81).
[S1: opcode 17/PT-91](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf#page=24).

A related FB1000/FB2000 manual mentions five-second session-key validity.
[S5: enhanced opcode 17](https://documentation.emersonprocess.com/intradoc-cgi/groups/public/documents/instruction_manuals/d301828x012.pdf#page=18).
CISA parses the key and credential field sizes but provides no credential
wrapping implementation. [S6: login parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_017.spicy).

**Assessment:** policy is documented; wrapping and ROC800 key lifetime remain
unresolved. A 40-byte wire password field is not a 40-character user-password
policy. Neither field sizes nor “wrapped” identify an algorithm.

**Implementation direction:** distinguish firmware capability, opt-in state and
port access mode. Do not select enhanced login solely from a firmware threshold,
copy the five-second rule across families, or send plaintext padded to the wire
field size.

**Next evidence:** authoritative wrapping/encoding procedure and non-secret test
vectors; key expiration/reconnect behavior; whether enable/identity reads are
available before authentication. AutoSol's inactivity examples are useful for
session tests, but do not prove every request refreshes every firmware's session.
[S8: ROC Security](https://autosoln.atlassian.net/wiki/spaces/ACMDOCS/pages/2534670341/ROC%2BSecurity).

## DEC-3 — compatibility mode and I/O locations

**Findings:** PT-91/50 defines 0 as 16 points/slot and 1 as 8; opcode 6 status
0/1/2 describes 160/240/224-entry layouts. These are different fields.
[S1: PT-91/50 and opcode 50](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf#page=97).

AutoSol documents the same two points-per-module modes and separate conversion
charts. The charts were not inspected, so no independent mapping formula is
claimed from this source. [S7: layout guide](https://autosoln.atlassian.net/wiki/spaces/ACMDOCS/pages/2619932688/ROC%2BPlus%2BProtocol%2Bfor%2BROC800%2BSeries).

**Assessment:** field distinction is clarified; inconsistent diagram labels do
not supply sufficient mapping evidence by themselves.

**Implementation direction:** give the configuration flag and reported layout
status separate types. Select lengths from the latter and verify actual payload
lengths. Keep physical positions distinct from per-type logical numbers.

**Next evidence:** literal selector replies for each layout, first/last slot
mapping, CPU diagnostics and absent modules, then ROCLINK/controller comparison.
Do not mechanically reproduce table diagrams with inconsistent slot labels.

## DEC-4 — extrema times, clock year and history indices

**Findings:** the related older ROC manual consistently describes opcode 105
occurrence times as five calendar bytes. Its opcode 7 uses a one-byte year plus
leap-year information. [S4: sections 2.5 and 2.14](https://documentation.emersonprocess.com/intradoc-cgi/groups/public/documents/instruction_manuals/d301053x012.pdf).

The ROC800L manual repeats the five-byte-versus-UINT32 contradiction in its
min/max table; it therefore does not independently resolve the ROC800 question.
[S3: opcode 105](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301659x012.pdf#page=28).

**Assessment:** five-byte calendar occurrence times are a corroborated candidate,
not confirmed ROC800 behavior. Shared text may be inherited from an older manual.
The old clock format is evidence of a family difference, not grounds to override
the current ROC Plus clock codec.

**Implementation direction:** retain raw occurrence bytes in any provisional
model; do not infer a year/timezone or reinterpret them as epoch seconds.
Keep the existing clock-table interpretation.

**Next evidence:** independent ROC800 opcode 105 reply length/offsets and values
compared to known extrema; minute-history boundary requests at indices 59/60.
The minute-index discrepancy was searched but not resolved.

## DEC-5 — file, tunnel and transaction subcases

### DEC-5a — delete command

**Findings:** ROC800L Table 2-25 explicitly uses delete command 5 for both
directions. CISA's file enumeration also assigns delete 5 and directory 6.
[S3: file table](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301659x012.pdf#page=47),
[S6: enumeration](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_enums.spicy).

**Assessment:** command 5 is the strongest candidate. The related-family manual
and parser support interpreting the baseline's delete-request 6 as an error,
but do not establish a ROC800 hardware result.

**Next evidence:** vendor erratum or opt-in bench test on a disposable file.
Do not try alternate delete command numbers automatically.

### DEC-5b — directory forms

**Findings:** CISA has 8-bit and 16-bit filename-count structures. Its enum labels
64 as a newer response form; commit history includes a correction stating that
64 applies to the response, not the request.
[S6: file parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_203.spicy),
[history, entry 62e105d](https://github.com/cisagov/icsnpp-roc-plus/commits/main/).

The ROC800L directory table uses command 5 too, colliding with its delete row.
That comparison source is internally inconsistent and cannot settle directory
numbering. [S3: printed p. 2-38](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301659x012.pdf#page=48).

**Assessment:** keeping request/response commands separate is supported; exact
newer counts/paging remain unresolved. The parser's own structures are not a
replacement for the wire specification.

**Next evidence:** old/new ROC800 directory request and reply pairs, count widths,
continuation input, no-progress/EOF behavior and null terminators.

### DEC-5c — commissioned tunnel index

**Findings:** CISA parses a one-byte request index and a four-byte response index
sequentially, choosing the documented width rather than overlapping offsets.
[S6: tunnel parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_205.spicy).

**Assessment:** implementation clue only. It may reproduce the spec's mistake.
Do not select this layout based on parser agreement alone.

**Next evidence:** real opcode 205 request/reply with a known commissioned target,
or an applicable vendor erratum. Check both length and embedded-opcode positions.

### DEC-5d — transaction data offset and value framing

**Findings:** CISA reads a two-byte data offset immediately after transaction
number, and parses the reply as type/value data without a mandatory TLP before
every value. [S6: transaction parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_206.spicy).

**Assessment:** implementation clue, not a resolved correction.

**Next evidence:** multi-field and split-page transaction fixtures establishing
request offset location, data type widths, presence/absence of TLPs and continuation
boundaries. Reject ambiguous field framing rather than guessing.

## DEC-6 — error-code catalog and partial writes

**Findings:** CISA's generic error enum omits file/tunnel-specific additions
discussed by service-specific vendor sections. [S6: errors enumeration](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_enums.spicy).

**Assessment:** another parser does not eliminate the need for a per-service
catalog. No independently verified partial-write transcript was found.

**Implementation direction:** retain all code/offset bytes; interpret them only
with operation context. Do not use enum absence to reject an unknown device error
or to infer that every unreported write succeeded.

**Next evidence:** multi-error writes against writable/read-only parameters,
documented offset interpretation and service-specific failure replies.

## DEC-7 — references outside the baseline inventory

**Findings:** Emerson's older ROC manual explicitly describes opcode 200 as HART/
WirelessHART passthrough, says ROC800 supports version 1, and presents an FB107
version 2 layout. Its cross-reference labels a ROCLINK document number as the
protocol manual, another reason to verify applicability.
[S4: section 2.39](https://documentation.emersonprocess.com/intradoc-cgi/groups/public/documents/instruction_manuals/d301053x012.pdf#page=67).

**Assessment:** opcode 200 is real, not merely an obsolete index artifact.
The published version 2 table is not sufficient to implement ROC800 version 1.
Searches for opcode 19/20/80/101 did not establish applicable ROC800 definitions;
that negative result is not proof they never existed.

**Implementation direction:** add a supplemental passthrough research task,
keeping the 28-service baseline inventory unchanged. Require a version-1 source
before adding a codec; avoid importing unrelated-family service numbers.

**Next evidence:** older applicable ROC Plus manual or vendor supplement defining
version 1, then an explicit decision about expanded project scope.

## DEC-8 — firmware/device identity and subtype

**Findings:** S2 establishes family-specific enhanced-security versions, so ROC800L
1.70 must not be treated as ROC800 1.70. CISA's subtype enum assigns Series 1 to 0
and Series 2 to 1, opposite the baseline table.
[S2: firmware list](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301912x012.pdf#page=7),
[S6: subtype enum](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_enums.spicy).

**Assessment:** firmware-family distinction is documented; subtype disagreement
is a new implementation conflict. Preserve the raw value rather than “fixing”
rocsteady from another parser. Firmware-update readiness remains unresolved.

**Next evidence:** device identification strings/version, independent subtype
replies from known Series 1/2 controllers, update-mode service availability and
behavior when a replacement is a different controller.

## DEC-9 — broadcast and direct connect

**Findings:** the baseline documents unit-zero, matching-group broadcasts with
no response, using clock setting as an example. A Kepware serial-driver manual
defaults device address/group to 240/240.
[S1: section 1.3](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf#page=9),
[S11: communication settings](https://ftp.softwaretoolbox.com/demodnld/prod_docs/topserver_help_pdf/v5_11/fisher_rocplus_serial.pdf).

**Assessment:** broadcast completion basics are documented; direct-connect use
has driver evidence but reply address identity is still unresolved. Broad service
support cannot be inferred from one broadcast example.

**Implementation direction:** maintain explicit broadcast and direct-connect
policies; do not relax response correlation or enable 240/240 silently.

**Next evidence:** captures of configured versus direct-connect addressing on each
channel, per-opcode broadcast applicability, shared-bus arbitration and unsolicited
source handling.

## DEC-10 — concrete schema validation lead

**Findings:** PTC release 6.4.321.0 corrects PT-177 parameter 0 from 10 to 40 bytes
in both ROC Plus drivers.
[S10: release notes](https://support.ptc.com/help/kepware/kepserverex/en/kepware/kepserverex/whats_new/whats_new_6_4_321_0.html).

**Assessment:** strong evidence of a real driver-width defect, not proof that every
firmware uses the same width or that the current baseline is itself incorrect.

**Implementation direction:** make PT-177/0 a specific catalog regression case;
audit source width, firmware gates and response budget before implementing the row.
Retain documented conditional-access rules; no universal conditional-write policy
was established by this pass.

**Next evidence:** independently sized PT-177/0 values and firmware identity, then
the same per-parameter audit for every remaining catalog row.

## Search log and follow-up order

Representative queries: “ROC Plus enhanced security GETSESSIONKEY wrapped session
key protocol”; “ROC Plus opcode 203 delete command 5 6 directory 64”; “ROC Plus opcode
205 commissioned index”; “ROC Plus opcode 105 minimum time timestamp 5 bytes”;
“ROC Plus CRC Ethernet ignore”; “ROC Plus maximum message 255”; exact opcode
19/80/101/200 searches; “D301912X012 site:emerson.com”.

Search results led to publisher documents and implementation-owned documentation.
Mirrors/search snippets were used for discovery; the report relies on the original
sources except S11's explicitly labelled indexed excerpt. Some GitHub source/commit
URLs, ROCLINK PDF fetches and PDF screenshot calls failed. Missing reads were not
treated as confirmations. No entire-schema audit is claimed.

Next work: apply the documented security/layout distinctions to design, seek
authoritative wrapping and ROC800 opcode 200 v1 documents, then obtain independent
file/tunnel/history fixtures. Keep code changes and completion checkboxes separate
from this research; online corroboration alone does not close interoperability gates.
