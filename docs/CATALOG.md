# ROC800 catalog and discovery evidence

Source: [Emerson D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Initial slice reviewed 2026-10-10; PT-136 audited 2026-10-14 against printed
pp.3-200–201 (PDF pages 259–260), including visual inspection of the disputed row.
This document records 202 entries as technical metadata, not vendor descriptions.
The foundation slice was audited on 2026-10-14 against printed pp.3-34–37,
3-44–48 and 3-210–211.
Input Tables 3-13/3-15 audited on 2026-10-14 against printed pp.3-61–62 and
3-66–69 (PDF pages 120–121 and 125–128), including visual inspection of
calibration access notes and alarm-bit definitions.
Output Tables 3-14/3-16 audited on 2026-10-15 against printed pp.3-63–65
and 3-70–71 (PDF pages 122–124 and 129–130), using the publisher PDF text.
All 27 DO and 17 AO read fields are accounted for; physical evidence remains none.
Pulse Table 3-17 audited on 2026-10-15 against printed pp.3-72–74
(PDF pages 131–133), using the publisher PDF text; all 28 fields accounted for.
No PT row meets the full schema completion contract.

## Discovery contracts

Opcode 100 command 11 follows Table 2-13, printed p.2-22. Requests contain command,
start and count; replies echo those three bytes, then one classification per requested
type. Counts 0-245 are supported without wrapping past point type 255. User-program,
user-defined, ROC and absent classifications are distinguished; other bytes survive
as Unknown. A classification never supplies a parameter width. `scan_point_types`
splits a requested range by the count limit and selected profile's request/reply budgets,
retains one generation and returns complete pages or an error, with no implicit resume.

Opcode 50 follows Table 2-12, printed p.2-21. `IoPointRequest` selects point types or
logical numbers and explicitly declares Compatibility160, Compatibility240 or
Expanded224. Exact reply lengths are checked. `Roc800Profile` is the named selectable
layout preset; `DefaultProfile` requires an explicit layout argument for discovery.
Firmware is supplied independently, never inferred from opcode-6 revision/length.

`discover_io` reads configuration, both selectors twice, then configuration again,
under one exclusive generation. Observed changes return DiscoveryChanged while
preserving the healthy connection; malformed individual replies retire it. Repetition
detects observed differences, not changes that revert between queries or an atomic
device snapshot. Protected discovery remains a device rejection and can use custom setup.

Configuration status and PT-91/50 are distinct. The default policy checks reported
status against the explicitly selected layout for firmware >=2.00; pre-2.00 uses
Compatibility160 without interpreting the reserved status byte. Expanded diagrams
and compatibility-table labels contain contradictions. The preset uses the explicit
lengths in Table 2-12 and sequential positions in section 3.2. Source status 1 has a
240-entry range despite conflicting 160-entry prose. Expanded224 uses eight positions
per slot despite diagram labels carried over from the 16-position form. These are
explicit source-based choices, not hardware resolution of DEC-3.

Both selector arrays, including absent/reserved positions and unknown point types,
are retained. `IoPoint.location` is the wire physical position, `logical_number` the
per-type ordinal. `by_logical(...).tlp(parameter)` converts only types 101-109, using
the physical position, not that ordinal. CPU diagnostics occupy positions 0-4.
Custom profiles may override `io_location`, layout applicability and contextual
discovery decoders; framing, transport ownership and cancellation remain enforced.

## Catalog applicability and generation

`Roc800Catalog::new(firmware)` provides system-variable and clock metadata for an explicitly
asserted ROC800 identity. The conservative reviewed window is 1.00-3.60; later firmware
requires a caller provider/override. Introduction versions below are enforced.
This source-applicability window is not a tested firmware interoperability matrix.
`from_discovery(snapshot)` additionally validates I/O presence/type at physical
locations and binds the entire provider to the snapshot's connection generation.

Use `catalog.context()` with `read_schema`. Unknown identity, mismatched firmware,
omitted/reserved parameters, incorrect locations and unavailable points fail locally.
Lookup returns None for omitted metadata, permitting explicitly ordered provider
overrides in SchemaChain; unsupported applicability errors do not silently fall back.
An I/O lookup without a discovery snapshot fails rather than guessing module presence.

PT-91 and PT-136 use logical zero and need no I/O map. Static metadata establishes
types, not point availability or read permission; the controller may reject a read.
Discovery-bound catalogs still bind singleton reads to their connection generation.
PT-136/0-8 require 1.00, /9 requires 1.20, and /10-19 require 3.40.
These gates are source applicability, not physical firmware validation.

SchemaProvider now optionally declares its generation. ReadPlan carries it and checks
it before its first exchange, including after waiting for readiness/admission. A
replacement invalidates an old catalog/plan even if the replacement has an identical
configuration. Obtain a fresh discovery snapshot. SchemaChain rejects conflicting
generation requirements. User providers with static metadata remain unbound by default.

## Per-parameter manifest

All rows below: source edition October 2022; metadata `src/catalog/entries.rs`,
provider policy `src/catalog.rs`;
literal fixture `catalog_literal` in `examples/support/discovery_fixture.rs`;
independent TCP evidence `every_catalog_row_reads_independent_literal_values_over_tcp`
in `tests/discovery.rs`; actual serial evidence **none**; hardware evidence **none**.
Access is source metadata; this release only reads. Numeric constraints are metadata
for later writes and never filter observed read values. Fixed input units and enum/bit meanings are optional `CatalogEntry.metadata`
annotations. Defaults and unspecified units are not guessed; dynamic units tags
remain strings and bit fields remain raw bytes.

| PT | Parameters | Types / widths | Access | Introduced | Source |
| --- | --- | --- | --- | --- | --- |
| 91 | 0 | UInt8 / 1 | R/W | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 1 | UInt8 / 1 | R/W | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 2 | ASCII / 20 | R/W | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 3 | ASCII / 20 | R/O | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 4 | ASCII / 20 | R/O | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 5 | ASCII / 20 | R/O | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 6 | ASCII / 20 | R/O | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 8 | UInt16 / 2 | R/O | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 9 | UInt16 / 2 | R/O | 1.00 | 3.4.4, Table 3-5, p.3-34 |
| 91 | 41 | UInt32 / 4 | R/O | 1.20 | 3.4.4, Table 3-5, p.3-37 |
| 91 | 42 | UInt32 / 4 | R/O | 1.20 | 3.4.4, Table 3-5, p.3-37 |
| 91 | 50 | UInt8 / 1 | R/W | 2.00; Series 2: 3.00 | 3.4.4, Table 3-5, p.3-38 |
| 101 | 0 | ASCII / 10 | R/W | 1.00 | 3.4.12, Table 3-13, p.3-61 |
| 101 | 1 | UInt8 / 1 | R/W | 1.00 | 3.4.12, Table 3-13, p.3-61 |
| 101 | 3 | UInt8 / 1 | R/W | 1.00 | 3.4.12, Table 3-13, p.3-61 |
| 101 | 6 | UInt32 / 4 | R/W | 1.00 | 3.4.12, Table 3-13, p.3-61 |
| 103 | 0 | ASCII / 10 | R/W | 1.00 | 3.4.14, Table 3-15, p.3-66 |
| 103 | 1 | ASCII / 10 | R/W | 1.00 | 3.4.14, Table 3-15, p.3-66 |
| 103 | 2 | UInt8 / 1 | R/W | 1.00 | 3.4.14, Table 3-15, p.3-66 |
| 103 | 3 | Float / 4 | R/W | 1.00 | 3.4.14, Table 3-15, p.3-66 |
| 103 | 4 | Float / 4 | R/O | 1.00 | 3.4.14, Table 3-15, p.3-66 |
| 103 | 7 | UInt16 / 2 | R/O | 1.00 | 3.4.14, Table 3-15, p.3-66 |
| 103 | 21 | Float / 4 | R/O | 1.00 | 3.4.14, Table 3-15, p.3-67 |
| 103 | 32 | Binary / 1 | R/O | 1.00 | 3.4.14, Table 3-15, p.3-68 |
| 109 | 0 | ASCII / 10 | R/W | 1.00 | 3.4.20, Table 3-21, p.3-90 |
| 109 | 1 | ASCII / 10 | R/W | 1.00 | 3.4.20, Table 3-21, p.3-90 |
| 109 | 4 | Float / 4 | R/O | 1.00 | 3.4.20, Table 3-21, p.3-90 |
| 109 | 7 | UInt16 / 2 | R/W | 1.00 | 3.4.20, Table 3-21, p.3-91 |
| 109 | 12 | Float / 4 | R/W | 1.00 | 3.4.20, Table 3-21, p.3-91 |
| 109 | 23 | Binary / 1 | R/O | 1.00 | 3.4.20, Table 3-21, p.3-93 |

PT-136 source: section 3.4.47, Table 3-48, printed pp.3-200–201.
Every parameter 0-19 has a literal and independent TCP read; the rows group only
entries with matching widths/access/version. Numeric constraints are in
`CatalogEntry.numeric_range` and audited independently in the clock boundary test.

| PT | Parameters | Types / widths | Access | Introduced |
| --- | --- | --- | --- | --- |
| 136 | 0-4, 6 | UInt8 / 1 | R/O | 1.00 |
| 136 | 5 | UInt16 / 2 | R/O | 1.00 |
| 136 | 7 | Time / 4 | R/O | 1.00 |
| 136 | 8 | UInt8 / 1 | R/W | 1.00 |
| 136 | 9 | UInt32 / 4 | R/O | 1.20 |
| 136 | 10-13, 15-18 | UInt8 / 1 | R/W | 3.40 |
| 136 | 14, 19 | Time / 4 | R/O | 3.40 |

Clock-specific limitations: reads of separate fields are not an atomic clock
snapshot. TIME stays a raw u32; no timezone, UTC conversion or DST computation
is inferred. Calendar bytes and unknown values remain representable. The printed
PT-136/17 end-week range is 1-56, while /12 uses 1-6 for the start week. The former
constraint is deliberately None pending clarification (DEC-10.05); its raw byte is
readable, including 56. No write validation or corrected range is claimed.
Defaults, update ownership and enum meanings remain outside the current schema
model; PT-136 is partial even though all 20 read widths are accounted for.

Additional evidence in `tests/discovery.rs`: clock firmware/range boundaries,
static reads without I/O discovery, local rejection before parameter exchange,
healthy rejection versus malformed-response retirement, and generation invalidation.
`tests/discovery_simulator.rs` supplies all 20 clock values explicitly; the simulator
does not calculate clock/DST parameter values from its opcode-7 clock.
The example and packaged consumer read PT-136 through the public provider.

Unresolved scope: remaining parameters/reserved rows in the original four PT tables, complete
bit/enumeration/default/unit metadata, typed write semantics and firmware/hardware
interoperability. In particular PT-109's measurement is parameter 12, not PT-103's
parameter 21; PT-109/7 is R/W while PT-103/7 is R/O. These distinctions are fixtures.

## Foundation slice: system diagnostics, ports and modules

The additional 36 fields use these independently transcribed source facts:

| PT | Parameters | Types / widths | Access | Introduced | Source |
| --- | --- | --- | --- | --- | --- |
| 91 | 10-12, 22 | UInt8 / 1 | R/O | 1.00 | Table 3-5, pp.3-34–35 |
| 91 | 13-14 | UInt16 / 2 | R/O | 1.00 | Table 3-5, p.3-35 |
| 91 | 23 | Float / 4 | R/O | 1.00 | Table 3-5, p.3-35 |
| 91 | 37-38 | ASCII / 20 | R/O | 1.00 | Table 3-5, p.3-37 |
| 91 | 43-46 | UInt16 / 2 | R/W | 1.20 | Table 3-5, p.3-37 |
| 91 | 47-48 | Time / 4 | R/O | 1.30 | Table 3-5, p.3-37 |
| 95 | 0 | ASCII / 10 | R/W | 1.00 | Table 3-7, p.3-44 |
| 95 | 1 | UInt16 / 2 | R/W | 1.00 | Table 3-7, p.3-44 |
| 95 | 2-4, 6, 44 | UInt8 / 1 | R/W | 1.00 | Table 3-7, pp.3-44–45, 3-48 |
| 95 | 5, 9 | UInt8 / 1 | R/O | 1.00 | Table 3-7, p.3-45 |
| 95 | 7-8 | Float / 4 | R/W | 1.00 | Table 3-7, p.3-45 |
| 95 | 36, 42 | UInt16 / 2 | R/W | 1.00 | Table 3-7, p.3-47 |
| 95 | 37 | Time / 4 | R/O | 1.00 | Table 3-7, p.3-47 |
| 139 | 0-2 | UInt8 / 1 | R/O | 2.02 | Table 3-51, p.3-210 |
| 139 | 3 | ASCII / 10 | R/O | 2.02 | Table 3-51, p.3-210 |
| 139 | 4, 8-9 | ASCII / 20 | R/O | 2.02 | Table 3-51, pp.3-210–211 |

PT-95 uses communication-port ordinals 0-5, independently of the opcode-50 map.
Logical 1 is Ethernet: only /36, /37, /42 and /44 from this slice are valid there.
Other fields on that port fail locally before admission. The introductory restriction
and later /40–41 zero-value statements require further review before those fields
are included. PT-139 uses module ordinals 0-26, rather than physical I/O channel
positions. The selected fields are all documented as valid in boot mode; other
module fields remain omitted. Neither static range establishes installation, module
mode, read authorization or an atomic snapshot. Device rejection remains possible.
A discovery-bound provider retains its generation requirement for these fields too.

Contradictory enum ranges remain unspecified: PT-95/5 describes Ethernet code 19
outside its printed range; PT-139/0 describes code 36 outside its range, and /1
includes failure code 2 outside 0-1. PT-95/44 exists at 1.00, but security value 2
is documented from 1.21. Reads preserve every raw byte; no security selection,
value-level write constraint or login procedure is inferred. PT-91/11 can return
values outside its licensed capacity range; numeric metadata never filters reads.

Every new row joins the independent literal/TCP sweep. Additional tests cover
port/module boundaries, firmware thresholds, rejection before any parameter exchange,
unknown/raw codes, static reads without I/O discovery and explicit simulator values.
Catalog checks reject duplicate keys/names, invalid widths/bounds and missing sources.
There is one variant per key today; future overlapping applicability needs new checks.
The example and packaged consumer read port counters and module boot identity.
Serial and hardware evidence remain **none**; all seven PT tables remain partial.

## Complete input read widths and interpretation metadata

The input slice adds 44 entries: all 16 PT-101 and all 40 PT-103 parameters now
have read metadata and independent literal/TCP evidence. There are no reserved
parameter rows in these two tables. Sub-bits below are annotations of their byte,
not additional TLP parameters. The full schema contract remains open because
writes, defaults, update ownership and complete conditional semantics are outstanding.

| PT | Parameters | Type / width | Table access | Introduced | Source |
| --- | --- | --- | --- | --- | --- |
| 101 | 0 | ASCII / 10 | R/W | 1.00 | Table 3-13, p.3-61 |
| 101 | 1, 3-5, 9-11 | UInt8 / 1 | R/W | 1.00 | Table 3-13, pp.3-61–62 |
| 101 | 2, 7-8, 13 | Float / 4 | R/W | 1.00 | Table 3-13, pp.3-61–62 |
| 101 | 6 | UInt32 / 4 | R/W | 1.00 | Table 3-13, p.3-61 |
| 101 | 12 | Binary / 1 | R/O | 1.00 | Table 3-13, p.3-62 |
| 101 | 14 | Float / 4 | R/O | 1.00 | Table 3-13, p.3-62 |
| 101 | 15 | UInt8 / 1 | R/O | 1.00 | Table 3-13, p.3-62 |
| 103 | 0-1 | ASCII / 10 | R/W | 1.00 | Table 3-15, p.3-66 |
| 103 | 2, 5-6, 22, 29-31, 34-35 | UInt8 / 1 | R/W* | 1.00 | Table 3-15, pp.3-66–69 |
| 103 | 3, 13-19, 23-28 | Float / 4 | R/W* | 1.00 | Table 3-15, pp.3-66–68 |
| 103 | 4, 20-21, 33 | Float / 4 | R/O | 1.00 | Table 3-15, pp.3-66–69 |
| 103 | 7 | UInt16 / 2 | R/O | 1.00 | Table 3-15, p.3-66 |
| 103 | 8-12 | UInt16 / 2 | R/W | 1.00 | Table 3-15, pp.3-66–67 |
| 103 | 32 | Binary / 1 | R/O | 1.00 | Table 3-15, pp.3-68–69 |
| 103 | 36 | UInt8 / 1 | R/W | 3.00 | Table 3-15, p.3-69 |
| 103 | 37 | Float / 4 | R/W | 3.00 | Table 3-15, p.3-69 |
| 103 | 38 | UInt8 / 1 | R/O | 3.00 | Table 3-15, p.3-69 |
| 103 | 39 | Float / 4 | R/O | 3.10 | Table 3-15, p.3-69 |

* PT-103/19, /34 and /35 have R/W table columns but notes suggesting R/O
for external ROC Plus. Reads are unambiguous. `access` preserves the table column;
`metadata.write_caveat` records the unresolved note (DEC-10.06). It is not a verified
write permission. No calibration commands or configuration writes are implemented.
Numeric ranges in `entries.rs` are source write metadata and never filter replies.
PT-101/13's minimum is 0.004 seconds, unlike PT-103/3's 0.05 seconds.
PT-103/5 is a byte percentage 0-99; calibrated raw values /8-12 are UInt16.
Engineering values and limits have no invented finite range or fixed units.

Optional `CatalogMetadata` lives alongside static entries, separate from
`ParameterSchema` and provider read planning. This keeps caller schemas unchanged.
`EnumMeaning` supplies byte codes/keys and `BitMeaning` supplies masks/keys.
Use `enum_meaning(raw)` or `active_bits(raw)` for interpretation, while retaining
raw `Value::UInt8`/`Value::Binary`. `uninterpreted_bits(raw)` returns every set bit
with no supplied meaning, including documented unused bits. An empty collection
means unspecified metadata. Metadata lookup alone does not establish firmware or
location applicability; obtain readable schemas through the provider first.

| Parameters | Enum code meanings |
| --- | --- |
| PT-101/1,4,5,9,10,11; PT-103/2,6,22,29,30,31,36 | 0 disabled; 1 enabled |
| PT-101/3,15 | 0 off; 1 on |
| PT-103/34 | 0 current; 1 start; 2 calibrate; 3 restore_previous; 4 stop |
| PT-103/35 | 0 none; 1 zero; 2 span; 3 midpoint_1; 4 midpoint_2; 5 midpoint_3; 6 offset |
| PT-103/38 | 0 12_bit; 1 16_bit |

| Parameter | Bit masks and meanings | Unused mask |
| --- | --- | --- |
| PT-101/12 | 0x20 status_on; 0x80 scanning_disabled | 0x5f |
| PT-103/32 | 0x01 low; 0x02 low_low; 0x04 high; 0x08 high_high; 0x10 rate; 0x40 point_fail; 0x80 scanning_disabled | 0x20 |

Fixed units are seconds for PT-101/2,7,8,13,14 and PT-103/3,4,33; percent for
PT-103/5; milliamps for PT-103/39. Device engineering-unit tags remain caller-readable
strings. PT-101 counters/time totals retain raw values; no rollover accounting,
input filtering, calibration or alarm calculation is simulated.

Tests read every input field over all three explicit layouts, reject whole batches
locally at the 3.00/3.10 gates, and cover all 256 bytes for enum/alarm interpretation.
The independent peer also supplies unknown codes, unused bits, a NaN payload,
infinity and negative zero; raw bits survive and the connection stays healthy.
The simulator supplies all 56 input fields explicitly. Catalog consistency tests
check enum/bit uniqueness, masks, unused-bit separation and value bounds.
The example reads input configuration/value/alarms together and displays metadata;
the extracted-package consumer exercises new fields and the public helpers.
No atomic snapshot, serial or physical-controller interoperability is claimed.

## Service evidence

| Variant | Source | Limits | Codec/API | Simulator | Independent fixtures/TCP | Serial/hardware | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| OP-100 command 11 | 2.10/Table 2-13 | Request 3; reply 3+count; count 0-245; no range wrap | PointTypeRequest/Codec, read_point_types/scan_point_types | tests/discovery_simulator.rs | table_2_13_literals and independent_tcp_pages tests in tests/discovery.rs | none / none | Verified software codec; introduced 1.20 per manual; actual firmware exchanges unverified |
| OP-50 selector 0 | 2.9/Table 2-12 | Request 1; reply 160/240/224 selected explicitly | IoPointRequest/Codec, read_io_points/discover_io | tests/discovery_simulator.rs | table_2_12_explicit_selector_lengths and all_layouts tests in tests/discovery.rs | none / none | Verified length/selector codec; explicit source-based position policy; DEC-3 interoperability unresolved |
| OP-50 selector 1 | 2.9/Table 2-12 | Request 1; reply 160/240/224 selected explicitly | Same contextual codec; logical numbers retained separately | tests/discovery_simulator.rs | Same independent selector fixtures; observed-change and malformed-reply cases | none / none | Same limitations; never substituted for physical TLP position |

All discovery methods retain selected profile policies, managed deadlines and no
replay. Tests also cover call cancellation, disconnection, healthy device errors,
malformed replies, budget failures, overrides and stale catalog rejection.

Run `cargo run --locked --example discovery_catalog` to discover both services and
read firmware identification, clock year/raw time, CPU load, a port counter, module boot identity, an analog input, a discrete input and battery value
through the built-in provider, without manually supplying widths.

## Output read catalog — PT-102/104

Every row below has independent literal/TCP evidence in `catalog_literal` and
`every_catalog_row_reads_independent_literal_values_over_tcp`, plus every-layout
coverage in `output_all_fields_read_over_tcp_for_every_layout`. Explicit simulator
values cover all fields in `simulator_reads_every_output_field_in_each_layout`.
The extracted-package consumer reads DO type/alarms and AO raw/physical values.
All fields require an I/O snapshot at the matching physical position; logical
ordinals are converted through `IoPoint::tlp`. No output writes are implemented.

| PT | Parameter | Type / width | Source access | Introduced | Source |
| --- | --- | --- | --- | --- | --- |
| 102 | 0 | ASCII / 10 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-63 |
| 102 | 1 | ASCII / 10 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-63 |
| 102 | 2 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-63 |
| 102 | 3 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-63 |
| 102 | 4 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-63 |
| 102 | 5 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-63 |
| 102 | 6 | Binary / 1 | R/O | 1.00 | 3.4.13, Table 3-14, p.3-63 |
| 102 | 7 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 8 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 9 | UInt32 / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 10 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 11 | UInt8 / 1 | R/O | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 12 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 13 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 14 | Float / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-64 |
| 102 | 15 | Float / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 16 | Float / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 17 | Float / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 18 | Float / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 19 | Float / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 20 | Float / 4 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 21 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 22 | UInt8 / 1 | R/W | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 23 | Float / 4 | R/O | 1.00 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 24 | UInt8 / 1 | R/O | 1.10 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 25 | UInt8 / 1 | R/O | 1.10 | 3.4.13, Table 3-14, p.3-65 |
| 102 | 26 | UInt8 / 1 | R/W | 3.01 | 3.4.13, Table 3-14, p.3-65 |
| 104 | 0 | ASCII / 10 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-70 |
| 104 | 1 | ASCII / 10 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-70 |
| 104 | 2 | UInt8 / 1 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-70 |
| 104 | 3 | UInt8 / 1 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-70 |
| 104 | 4 | UInt8 / 1 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-70 |
| 104 | 5 | UInt8 / 1 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-70 |
| 104 | 6 | Binary / 1 | R/O | 1.00 | 3.4.15, Table 3-16, p.3-70 |
| 104 | 7 | UInt8 / 1 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 8 | UInt16 / 2 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 9 | UInt16 / 2 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 10 | Float / 4 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 11 | Float / 4 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 12 | Float / 4 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 13 | UInt16 / 2 | R/O | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 14 | Float / 4 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 15 | Float / 4 | R/W | 1.00 | 3.4.15, Table 3-16, p.3-71 |
| 104 | 16 | Float / 4 | R/O | 1.00 | 3.4.15, Table 3-16, p.3-71 |

DO /24–25 require 1.10; /26 requires 3.01. Other output fields require 1.00.
Scanning /2 exposes disabled/automatic/manual; /3–5 expose disabled/enabled.
Reset /7 exposes last-value/failsafe-value. DO /8, /21–22, /24 are off/on;
/10–13 are disabled/enabled; /25 labels relay=0, solid-state=1, relay-6-point=3;
/26 labels normal/inverted. Unknown values are retained without a label.

DO alarm /6 defines bit 5 scanning-manual and bit 7 scanning-disabled (unused
mask 0x5f). AO /6 additionally defines bit 6 point-fail (unused mask 0x1f).
Every raw byte, including unused bits, is preserved. DO /14–17 and /23 have
fixed seconds units. The /1 engineering-units tags remain device strings.

Numeric ranges are metadata, never filters on read values. DO /2 is 0–2,
/9 is 0–4294967295, /16–17 are 0–43200, /23 is zero through positive finite
f32 maximum; other documented boolean fields are 0–1. AO /2 is 0–2;
/8–9 and /13 are 0–65535; boolean fields are 0–1. Engineering floats have
no invented numeric constraints. Three special DO constraints remain explicit:

- /14 has module-dependent limits: DO 0.002–43200 seconds, DOR 0.05–43200.
  No universal range is selected without module identity.
- /15 has a strict positive lower bound and inclusive 43200 upper bound.
  The existing inclusive range pair cannot represent this, so it stays None.
- /25 prints a 0–1 range but enumerates code 3; the range stays None and code 3
  retains its documented label.

`write_caveat` records those three constraints, DO /26's ACIO-channel wording,
and AO /7's nonexistent /22 failsafe reference and mistaken /13 EU reference
(the table's failsafe-value field is /15 and auto-value field /12). These do not
change read widths or establish write behavior; see DEC-10.07. Defaults and
update ownership are not yet catalog fields. Full PT conformance remains open.

## Pulse-input read catalog — PT-105

Source: §3.4.16, Table 3-17, pp.3-72–74. Every field has independent bytes in
`catalog_literal`, independent typed values and TCP reads in `tests/discovery.rs`
and `tests/pulse_catalog.rs`, and explicit model values in
`simulator_reads_every_pulse_field_in_each_layout`. Requirement IDs appear beside
the tests and in [the requirements index](REQUIREMENTS.md). The example and
extracted-package consumer read counts, rates, alarm bits and contract-day totals.
Every field requires an explicitly selected ROC800 firmware in the reviewed
1.00–3.60 window and a matching discovered physical position. All are introduced
in 1.00; logical ordinal zero is not the wire physical location. The tests cover
1.00 and 3.60 applicability and all three layouts where applicable.

| PT | Parameter | Type / width | Source access | Introduced | Source |
| --- | --- | --- | --- | --- | --- |
| 105 | 0 | ASCII / 10 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 1 | ASCII / 10 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 2 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 3 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 4 | UInt32 / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 5 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 6 | UInt32 / 4 | R/O | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 7 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 8 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 9 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 10 | Float / 4 | R/O | 1.00 | 3.4.16, Table 3-17, p.3-72 |
| 105 | 11 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 12 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 13 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 14 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 15 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 16 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 17 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 18 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 19 | Float / 4 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 20 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 21 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 22 | UInt8 / 1 | R/W | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 23 | Binary / 1 | R/O | 1.00 | 3.4.16, Table 3-17, p.3-73 |
| 105 | 24 | Float / 4 | R/O | 1.00 | 3.4.16, Table 3-17, p.3-74 |
| 105 | 25 | Float / 4 | R/O | 1.00 | 3.4.16, Table 3-17, p.3-74 |
| 105 | 26 | Float / 4 | R/O | 1.00 | 3.4.16, Table 3-17, p.3-74 |
| 105 | 27 | Float / 4 | R/O | 1.00 | 3.4.16, Table 3-17, p.3-74 |

Known ranges: /2, /8 and /20–22 are 0–1; /3 is 0.05–43200 seconds;
/4 is 0–16000000; /5 is 0–23; /6 is 0–4294967295; /7 is 0–3;
/11 is 0–2; /27 is zero through positive finite f32 maximum. Strings use
10 printable ASCII bytes. /9 excludes zero; its `numeric_range` stays None
because a single inclusive interval cannot express that domain. `write_caveat`
records the actual constraint. Other floats have no invented numeric bound.
Ranges are metadata for future writes and never filter observed read values.

/2 and /20–22 label disabled/enabled. /7 labels EU per second/minute/hour/day;
/8 labels EU per pulse or pulses per EU; /11 labels rate, accumulator with maximum
rollover, or accumulator with entered rollover. Unknown codes retain raw bytes.
Alarm /23 defines low, low-low, high, high-high, rate, point-fail and
scanning-disabled at bits 0, 1, 2, 3, 4, 6 and 7; bit 5 is unused (mask 0x20)
and is preserved. Fixed units are seconds for /3, pulses for /4 and /6, and
pulses/second for /27. Engineering units and rate periods remain device context.

The client does not calculate rate, conversion, contract-day boundaries, totals
or rollover. In particular /4 and /6 are UINT32 counts while /24–26 are FLOAT
values. A read does not make a multi-field snapshot atomic or establish durable
pulse accounting across resets/reconnects. Defaults and update ownership are
not catalog fields; writes and full PT conformance remain open. Physical and
actual serial evidence: none.
