# Second public-source research pass

Use [the current decision register](DECISIONS.md) for current conclusions and
delivery dispositions. This document preserves the second-pass evidence narrative.

Date: **2026-10-08**. [Source records S12-S18](SOURCES.md#second-pass-additions--reviewed-2026-10-08).
The subsequent [systematic record](SEARCH_RECORD.md) records exact executed
queries for every DEC item and separates failed retrievals from negative results.
This extends and qualifies [the first pass](AMBIGUITIES.md); it does not change
protocol code or close implementation/interoperability checkboxes.

## Results that change the evidence

### File delete: stronger vendor evidence for command 5 — DEC-5a

The June 2018 ROC Plus manual's Table 2-26 assigns command 5 to delete requests
and responses. This is same-family historical evidence, stronger than the earlier
ROC800L comparison. Its directory rows separately use command 6.
[S12: file table](https://pagebuilderassets.yodify.dev/Documentation/Document/aRd2QZ/499286-ROC-Plus-Protocol-Specification-Manual-Pdf-19.pdf#page=48).

**Assessment:** a likely documentation regression in the 2022 delete-request row.
Use 5 as the proposed specification correction, pending an erratum or applicable
wire evidence; no automatic destructive fallback to other command numbers.

**Counter-evidence qualification:** at pinned CISA revision S13,
`testing/tools/payloads/opcode_203.py` describes delete as 5 but sets its delete
request's command field to 6. The parser enumeration inspected in pass one assigns
delete 5. These are conflicting implementation artifacts, not independent
corroboration. [S13: file payload](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/testing/tools/payloads/opcode_203.py).

### CISA trace provenance: generated fixtures — all disputed layouts

The PCAP builder documentation says it constructs request/response packets and
writes comprehensive files to the test trace paths. The repository includes
payload definitions and generated trace tests.
[Builder documentation](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/testing/tools/readme.md).

**Assessment:** these traces cannot be used as independent controller captures.
Inspecting bytes still helps reproduce an author's assumptions, but does not
resolve the same spec ambiguity from which they were generated. This does not
claim that every possible capture in the repository has been audited.

The pinned opcode 205 fixture uses a four-byte response index with fabricated
distinctive values. It reinforces the parser's interpretation, not device
behavior. [S13: tunnel payload](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/testing/tools/payloads/opcode_205.py).

### Opcode 105: an additional competing interpretation — DEC-4

The pinned CISA parser consumes each occurrence-time region as one temporary byte
followed by a UINT32. Its payload definitions describe epoch-second times.
[Parser](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/analyzer/roc_plus_opcode_105.spicy),
[payload](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/testing/tools/payloads/opcode_105.py).

The June 2018 manual already contains the five-byte/UINT32 contradiction.
[S12: opcode 105](https://pagebuilderassets.yodify.dev/Documentation/Document/aRd2QZ/499286-ROC-Plus-Protocol-Specification-Manual-Pdf-19.pdf#page=30).

**Assessment:** the first pass's five-calendar-byte candidate is not the only
plausible interpretation. The extra-byte-plus-epoch interpretation consumes five
bytes too, so response length alone cannot distinguish it. No verified device
meaning for that extra byte was found.

**Required discriminator:** occurrence bytes from a known date/time, including
days/months that disambiguate packed calendar fields from an epoch value; test
several samples. Do not invent a status/padding meaning for the temporary byte.

### Opcode 50: parser is not a mapping oracle — DEC-3

The pinned CISA response parser consumes raw bytes using the advertised data
length; it does not calculate physical-slot mappings.
[S13: layout parser](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/analyzer/roc_plus_opcode_050.spicy).

**Assessment:** it cannot settle contradictory mapping diagrams. Maintain the
distinction between layout byte counts and actual logical/physical semantics.

### History segment scope: Series 2 capacity differs — DEC-8

The April 2023 ROC800 product sheet describes Series 2 history capacity as
240 points in 13 segments. It separately describes password encryption for
firmware 3.90 or newer.
[S15: product sheet](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301155x012.pdf).

**Assessment:** a universal eleven-segment API would omit advertised Series 2
capacity. Capacity does not prove every history opcode accepts every segment.
The implementation specification must map service-specific segment applicability
to device/firmware, rather than copying the oldest range everywhere.

### Error catalog: another code and a qualified write exception — DEC-6

Kepware documents error 53 as a ROC filesystem error and notes that an opcode 166
range may contain read-only parameters.
[S18: error help](https://support.ptc.com/help/kepware/drivers/en/kepware/drivers/FISHERROC/roc-plus-error-codes.html).

**Assessment:** preserve 53 rather than treating the baseline generic list as
exhaustive. This documents driver semantics; it does not establish which writable
parameters committed in a particular failed multi-write. Partial outcomes still
need explicit representation and independently verified examples.

## Further checks that did not resolve wire behavior

### Security supplements and actual implementations — DEC-2/8

The September 2022 deployment guide lists ROCLINK 2.70, whereas the March 2023 guide
lists 2.71. Edition-specific prerequisite changes must not be confused with a new
wire algorithm. [S14: older guide](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301912x0112.pdf).

The FBxEdge guide inspected explicitly excludes enhanced-security ROC800/ROC800L
connections for that edition.
[S16: section 1.1](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301913x012.pdf#page=8).
A gateway's support for ROC Plus therefore does not prove it implements enhanced
login.

Neither source supplied the wrapping algorithm. No applicable public SDK or
client-side encryption implementation was located in the searches below.
Unrelated search hits about RTP rollover counters and AES were excluded; “ROC”
there denotes something else. No cipher was inferred from key/credential widths.

### Older manuals and missing services — DEC-5c/7

The 2018 tunnel table repeats the commissioned-index width/offset conflict.
That repeated edition is not an independent correction.
[S12: tunnel table](https://pagebuilderassets.yodify.dev/Documentation/Document/aRd2QZ/499286-ROC-Plus-Protocol-Specification-Manual-Pdf-19.pdf#page=50).

Opcode 200 references occur in the older error tables, but no corresponding
ROC800 version-1 wire definition was located there. The abferm client's constants
mark 19 and 80 as reserved; its README limits its ROC Plus claims.
[S17: constants](https://raw.githubusercontent.com/abferm/roc/master/opcodes.go),
[README](https://github.com/abferm/roc).
These constants are not proof of reserved status across all ROC800 firmware.

The desired 2011 manual was referenced by driver guidance, but a verified copy
was not obtained. The archive CDX endpoint failed retrieval. Do not label that
archive avenue exhausted.

### Remaining subjects — DEC-1/9/10

No new device evidence resolved direct-connect reply identity, general versus
routed message limits, per-operation broadcast support, conditional writes or
all-schema firmware applicability. The history capacity finding adds a concrete
capability question; it is not a full catalog audit.

## Search coverage and independence

| Avenue | Searches/inspection | Outcome and next opening |
| --- | --- | --- |
| Earlier manuals | D301180X012 + October 2011, 2010, PDF; archived June 2018 manual; linked client PDF; archive CDX | 2018 comparison read; 2011 and archive retrieval remain open |
| Security/firmware | ROC800 + encrypted password + developer/SDK; GETSESSIONKEY; wrapped session key; ROC Plus + AES; deployment editions and product/gateway guides | Policy and support restrictions, no wrapping procedure |
| Public clients | ROC800 + Python/library/GitHub; ROC Plus + protocol + GitHub; abferm scope/constants | Limited client found; no applicable disputed-service implementation |
| Parser provenance | CISA API tree, pinned parser/payload files, PCAP builder and trace paths | Synthetic-fixture evidence and internal conflicts |
| Driver documentation | PTC error table, vendor service searches and release/support guides | Code 53 and read-only exception; no verified partial-write transcript |
| Supplemental passthrough | ROC Plus + 200 + HART + version 1 + post/get + 2011 | Existing cross-reference only; applicable v1 layout remains missing |
| Public packet data | ROC Plus + Wireshark/capture/packet capture; parser tests | Generated fixtures found; no verified controller captures for disputed layouts |

The CISA tree API returned full revision
`802c31814a92355e66ccc05379b77d87886ca381`. Five files were read at that revision.
No upstream code, generator or downloaded program was executed. Web indexing
failed for some files; direct public-source retrieval succeeded for those five.

## Specification implications and next research

- Treat delete 5 as a stronger proposed correction with an explicit evidence trail.
- List both five-byte occurrence-time interpretations and the exact experiment
  needed to distinguish them.
- Require device/firmware-aware history bounds and cache identity.
- Exclude generated traces from independent interoperability evidence.
- Keep authentication wrapping, tunnel layout, transaction framing and opcode 200
  v1 open; research answers must precede architectural commitments in these areas.
- Continue the failed archive/2011-manual avenue and seek implementation-owner
  documentation for enhanced login, rather than declaring public sources exhausted.

This pass found useful corrections and limitations; it did not produce a complete,
unambiguous specification. Findings are recorded so later design can distinguish
established behavior, proposed errata and unanswered questions.
