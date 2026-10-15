# Online research sources

Reviewed: **2026-10-08**. Findings: [ambiguity research](AMBIGUITIES.md).
Baseline and hardware references remain in [README.md](README.md). These records
link to the originals; no vendor PDF or third-party source tree is rehosted.

“Primary” means the publisher describes its own protocol, driver or implementation.
A driver author's documentation is primary evidence of that driver's choices,
not an Emerson protocol guarantee. Related-family manuals require applicability
checks. Search publication/crawl labels were not treated as document revisions.

## S1 — ROC Plus baseline

- Publisher: Emerson; D301180X012; document cover October 2022.
- [Original PDF](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
- Locations reviewed: sections 1.2-1.3, 2.2-2.3, 2.9, 2.11, 2.24-2.26, 2.29; PT-91 parameters 35/50/81; Chapter 7 and index.
- Input catalog audit (2026-10-14): Tables 3-13/3-15, printed pp.3-61–62 and 3-66–69; all read widths, ranges, access columns and enum/bit definitions inspected. Calibration access notes on pp.3-67/69 visually inspected; DEC-10.06 remains unresolved.
- Log audit (2026-10-14): sections 2.13–2.14, Tables 2-16/17 and all record diagrams, printed pp.2-24–31; count/header budgets, packed flags, text/value boundaries, event-specific tags and old/new storage overlap inspected. Fixed record storage and modulo-index continuity limits are explicit in LOGS.md.
- SRBX audit (2026-10-14): Table 2-31, p.2-44 and Chapter 6 p.6-1 visually inspected; consumed-index request, empty reply, equality-based pending clearing and published index-7 CRC vector verified. Source does not make delivery durable or confirm pending clearing in the empty reply.
- Write audit (2026-10-15): §§2.20/2.23, Tables 2-23/26 (pp.2-37–39), and Table 3-5 /2 plus Tables 3-13 /0–2, 3-14 /0–1, 3-15 /0–1, 3-16 /0–1 and 3-17 /0–1 (pp.3-34/61/63/66/70/72), text checked against the publisher PDF. OP-181 prose mislabels the opcode and describes a response-size error despite its empty acknowledgement; the 240-byte request ceiling is explicit library policy, not a source-verified device boundary.
- Temperature audit (2026-10-15): §§3.4.17–18, Tables 3-18/19, printed pp.3-75–81 (PDF pages 134–140), text checked against the publisher PDF. All 38 RTD and 23 TC fields inventoried, including access suffixes/calibration notes, offset/failsafe versions, unit-default change, noncontiguous TC sensor codes and copied alarm/failsafe wording (DEC-10.08). No temperature writes or hardware compatibility inferred.
- MVS audit (2026-10-15): §3.4.19/Table 3-20, printed pp.3-82–89 (PDF pages 141–148), text checked against the publisher PDF. All 69 field definitions inventoried, including source versions, five BIN masks, noncontiguous command/poll codes, FLOAT /65 and UINT8 /68, 4088-only counter context and inconsistent /59 and /64 text (DEC-10.09). No MVS write or physical compatibility inferred.
- Role: normative baseline and origin of the contradictions; see existing reference section/table map.
- Used for DEC-1 through DEC-10; a revised page date does not establish a different cover revision.

## S2 — Security deployment

- Publisher: Emerson; D301912X012; cover March 2023.
- [Original PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301912x012.pdf).
- Locations: sections 1.1-1.2, 2.1, 2.3-2.4.
- Role: primary firmware/opt-in/credential-policy evidence for DEC-2/8; not a cryptographic implementation specification.

## S3 — ROC800L comparison

- Publisher: Emerson; D301659X012; inspected pages identify September 2022.
- [Original PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301659x012.pdf).
- Locations: opcode 105 table on printed pp. 2-18/19; opcode 203 Table 2-25 on pp. 2-37/38.
- Role: related-device evidence for DEC-4/5, not automatic ROC800 applicability.
- This URL returned 523 PDF pages. Search results also offered a 2020 mirror; that mirror was not substituted for this edition.

## S4 — Older ROC-family protocol comparison

- Publisher: Emerson; D301053X012; cover April 2023.
- [Original PDF](https://documentation.emersonprocess.com/intradoc-cgi/groups/public/documents/instruction_manuals/d301053x012.pdf).
- Locations: section 2.5/Table 2-11 (clock); section 2.14/Table 2-20 (min/max); section 2.39/Table 2-49 (HART passthrough).
- Role: related-family evidence for DEC-4/7. “All devices” in this manual must not be read as overriding ROC Plus device-specific definitions.

## S5 — FB1000/FB2000 protocol comparison

- Publisher: Emerson; D301828X012; June 2023.
- [Original PDF](https://documentation.emersonprocess.com/intradoc-cgi/groups/public/documents/instruction_manuals/d301828x012.pdf).
- Location: enhanced opcode 17, printed pp. 13-14.
- Role: related-family session-key timing clue for DEC-2; not proof of ROC800 timing.

## S6 — CISA/Battelle ROC Plus parser

- Publisher: cisagov, ICSNPP project; source read from mutable `main`.
- [Project](https://github.com/cisagov/icsnpp-roc-plus).
- [Commit history](https://github.com/cisagov/icsnpp-roc-plus/commits/main/).
- [File command parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_203.spicy).
- [Tunnel parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_205.spicy).
- [Transaction parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_206.spicy).
- [Login parser](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_opcode_017.spicy).
- [Enumerations](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/analyzer/roc_plus_enums.spicy).
- Role: primary evidence of parser choices for DEC-2/5/8. No controller capture provenance was established.
- The history listing showed head abbreviation `802c318` (May 14, 2026), but source reads were not independently pinned to that commit. Pin and recheck before deriving fixtures.
- Attempts to retrieve the opcode 50/105 files and immutable commit URLs failed; no claims about their contents are made.

## S7 — AutoSol layout guidance

- Publisher: AutoSol, ACM User Guide; page updated July 11, 2023.
- [ROC Plus Protocol for ROC800 Series](https://autosoln.atlassian.net/wiki/spaces/ACMDOCS/pages/2619932688/ROC%2BPlus%2BProtocol%2Bfor%2BROC800%2BSeries).
- Location: Logical Compatibility Mode and conversion sections.
- Role: primary driver guidance for DEC-3. Text was readable; conversion charts were image/button content and were not independently inspected.

## S8 — AutoSol security guidance

- Publisher: AutoSol, ACM User Guide; no stable revision established.
- [ROC Security](https://autosoln.atlassian.net/wiki/spaces/ACMDOCS/pages/2534670341/ROC%2BSecurity).
- Locations: Inactivity Timeouts, ROC Enhanced Security.
- Role: operational/session behavior clue for DEC-2/8.
- Its general ROC800 firmware wording must be checked against S2's separate ROC800/ROC800L versions.

## S9 — Elecsys RediGate packet sizing

- Publisher: Elecsys, RediGate Product Support; page updated April 5, 2018.
- [Protocol_ROC](https://elecsysdocs.atlassian.net/wiki/spaces/IDG/pages/296262/Protocol_ROC).
- Location: message size discussion before poll-table column descriptions.
- Role: primary driver guidance for DEC-1; cites an older 2011 ROC Plus manual.

## S10 — PTC parameter-width correction

- Publisher: PTC/Kepware; release 6.4.321.0.
- [Release notes](https://support.ptc.com/help/kepware/kepserverex/en/kepware/kepserverex/whats_new/whats_new_6_4_321_0.html).
- Location: Fisher ROC Plus Ethernet/Serial entries.
- Role: primary implementation correction evidence for DEC-10/PT-177; release version is the identifier, not a guessed firmware version.

## S11 — Software Toolbox direct-connect defaults

- Publisher: Software Toolbox distribution of Kepware driver help; v5.11-era URL.
- [Fisher ROC Plus Serial PDF](https://ftp.softwaretoolbox.com/demodnld/prod_docs/topserver_help_pdf/v5_11/fisher_rocplus_serial.pdf).
- Location: communication specification, printed p. 8.
- Role: primary driver-setting evidence for DEC-9. Retrieved via indexed PDF excerpt; full PDF edition/cover was not verified.

## Retrieval and preservation policy

These are citation records with original summaries, not archived copies.
If a future implementation depends on a mutable source, record its exact edition
or commit, checksum where available, relevant section and applicable device.
A failed retrieval is not evidence that a document or service does not exist.
No vendor contact, device operation, or external publication occurred during research.

## Second-pass additions — reviewed 2026-10-08

### S12 — archived ROC Plus manual, June 2018

- Author/publisher: Emerson/Remote Automation Solutions; D301180X012.
- [PDF hosted by Yodify](https://pagebuilderassets.yodify.dev/Documentation/Document/aRd2QZ/499286-ROC-Plus-Protocol-Specification-Manual-Pdf-19.pdf).
- Cover explicitly identifies June 2018; 326 PDF pages. This is a vendor-authored primary document on an archival host, not a current publisher download. Byte identity with a publisher copy was not established.
- Read: cover, opcode 105 Table 2-13, opcode 203 Table 2-26, opcode 205 Table 2-27, error references, CRC chapter.
- Use: historical comparison for DEC-4/5/7. Similar wording across editions is not independent device evidence.

### S13 — pinned CISA parser and fixture provenance

- Publisher: cisagov/ICSNPP; revision **802c31814a92355e66ccc05379b77d87886ca381**.
- [Repository tree API](https://api.github.com/repos/cisagov/icsnpp-roc-plus/git/trees/main?recursive=1) returned this full SHA.
- [Opcode 105 parser](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/analyzer/roc_plus_opcode_105.spicy).
- [Opcode 50 parser](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/analyzer/roc_plus_opcode_050.spicy).
- [Opcode 105 payload definitions](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/testing/tools/payloads/opcode_105.py).
- [Opcode 205 payload definitions](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/testing/tools/payloads/opcode_205.py).
- [Opcode 203 payload definitions](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/testing/tools/payloads/opcode_203.py).
- [PCAP builder documentation](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/testing/tools/readme.md), [builder source](https://raw.githubusercontent.com/cisagov/icsnpp-roc-plus/main/testing/tools/pcap-builder.py).
- Read mechanism: the five pinned parser/payload files were fetched directly from raw GitHub after web retrieval failed. Files were inspected as text, not executed.
- Use: DEC-3/4/5 and qualification of S6. Builder documentation describes generated fixtures and output trace paths; it does not establish hardware provenance.
- This supersedes the first pass's failed retrievals of opcode 50/105. The first-pass mutable-source assessment remains historical.

### S14 — earlier security deployment guide

- Publisher: Emerson; cover D301912X012, September 2022.
- [Original PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301912x0112.pdf).
- Filename contains `x0112`; document number was taken from its cover, not inferred from filename.
- Read: sections 1.1-1.2; 14 PDF pages.
- Use: compare firmware/software prerequisites with S2 for DEC-2/8. No wrapping algorithm was located.

### S15 — ROC800 product sheet

- Publisher: Emerson; D301155X012; April 2023; 12 PDF pages.
- [Original PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301155x012.pdf).
- Read: Series 2 feature list, security and history capabilities.
- Use: DEC-2/8, history capability limits. A product-capacity statement does not itself specify opcode field ranges.

### S16 — FBxEdge user guide

- Publisher: Emerson; D301913X012.
- [Original PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301913x012.pdf).
- Read: section 1.1 notes concerning enhanced security; 94 PDF pages.
- Use: DEC-2 interoperability restriction for the edition inspected. Revision is not independently established here; search discovery identified an April 2024 edition, so recheck the cover before using its limitation as current product guidance.

### S17 — abferm ROC client

- Author: abferm; mutable `master`, no commit pinned.
- [Project](https://github.com/abferm/roc), [opcode definitions](https://raw.githubusercontent.com/abferm/roc/master/opcodes.go).
- Read: README scope and opcode constants.
- Use: limited client implementation clue for DEC-7. The author describes minimal scope and uncertain ROC Plus coverage. Reserved constants do not constitute ROC800 service definitions or hardware evidence.
- The linked publisher ROC Plus PDF failed retrieval. This client did not resolve enhanced security or disputed file/history/tunnel forms.

### S18 — Kepware error documentation

- Publisher: PTC/Kepware; mutable online driver help; revision not established.
- [ROC Plus error codes](https://support.ptc.com/help/kepware/drivers/en/kepware/drivers/FISHERROC/roc-plus-error-codes.html).
- Read: error table and opcode 166 read-only exception note.
- Use: DEC-6; code 53 and partial-write implications as described by this driver. No packet transcript or firmware applicability was established.

## Systematic-search additions — reviewed 2026-10-08

### S19 — Kepware logical/location details

- Publisher: PTC/Kepware; online revision not established.
- [Primary driver help](https://support.ptc.com/help/kepware/drivers/en/kepware/drivers/FISHERROC/logical-location-details.html).
- Read: complete short page; legacy I/O location examples and diagnostic positions.
- Use: DEC-3. Its limited location range must not be generalized to every newer layout.

### S20 — deviceWISE troubleshooting

- Publisher: Telit/deviceWISE; displayed topic update August 03, 2026.
- [Primary driver help](https://docs.devicewise.com/Content/Products/GatewayDevelopersGuide/Devices/DeviceTypes/ROCPlus/ROC-Plus-driver-troubleshooting.htm).
- Read: device configuration troubleshooting and error table.
- Use: DEC-6; error descriptions and read-only exception. No partial-write capture was supplied.

### S21 — Series 2 operating-system firmware sheet

- Publisher: Emerson; D301156X012; cover May 2014; seven PDF pages.
- [Original PDF](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301156x012.pdf).
- Read: first-page historical database description and firmware architecture summary.
- Use: DEC-4/8. Describes fixed minute-history capacity and Series 2 segments; does not specify each history opcode's accepted indices.

### S22 — Crimson ROC configuration

- Publisher: HMS/Red Lion; live support article, stable revision not established.
- [Primary implementation guidance](https://support.hms-networks.com/hc/en-us/articles/27220985126034-Crimson-3-x-Emerson-ROC-Communications).
- Read: serial/Ethernet setup, SRBX host settings, identification/security, Ping TLP and driver selection.
- Use: DEC-2/8/9. A driver name containing “Enhanced” is not proof of enhanced credential wrapping. No direct-connect reply layout was found.

### S23 — Kepware Ethernet driver manual

- Publisher: PTC/Kepware; served by its public content hub; document revision not established.
- [Primary PDF](https://ptc-p-001.sitecorecontenthub.cloud/api/public/content/5abb97f509b14c28a57d9005aa9ac4ce?v=1de8eb20).
- 59 PDF pages. Read: Communication Specification/Operator Identification, printed pp. 23-24; initialization error about restricted I/O map access, printed p. 46.
- Use: DEC-2/3/9; direct-connect setting defaults and discovery restrictions. Neither defaults nor troubleshooting text establishes reply identity or universal access rules.

### S24 — Geo SCADA schema-view documentation

- Publisher: Schneider Electric; Geo SCADA 2020 help.
- [Primary help](https://tprojects.schneider-electric.com/geoscadahelp/geo%20scada%202020/content/rocplusdriverguide/SchemaView3.htm).
- Read: complete short page.
- Use: DEC-10 search classification. This describes the application's database schema, not a ROC wire parameter dictionary; excluded as a source for parameter widths.

The exact queries, source inspection ledger, failed retrievals and per-decision
remaining leads are in [SEARCH_RECORD.md](SEARCH_RECORD.md). Indexed hits not
inspected are explicitly separate from sources used as evidence.
