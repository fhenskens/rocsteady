# ROC800 catalog and discovery evidence

Source: [Emerson D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Initial slice reviewed 2026-10-10; PT-136 audited 2026-10-14 against printed
pp.3-200–201 (PDF pages 259–260), including visual inspection of the disputed row.
This document records 130 entries as technical metadata, not vendor descriptions.
The foundation slice was audited on 2026-10-14 against printed pp.3-34–37,
3-44–48 and 3-210–211.
Input Tables 3-13/3-15 audited on 2026-10-14 against printed pp.3-61–62 and
3-66–69 (PDF pages 120–121 and 125–128), including visual inspection of
calibration access notes and alarm-bit definitions.
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
