# ROC800 catalog and discovery evidence

Source: [Emerson D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Initial slice reviewed 2026-10-10; PT-136 audited 2026-10-14 against printed
pp.3-200–201 (PDF pages 259–260), including visual inspection of the disputed row.
This document records 50 entries as technical metadata, not vendor descriptions.
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
for later writes and never filter observed read values. Units/defaults and textual
descriptions are not guessed; dynamic units tags remain strings and bit fields raw bytes.

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
read firmware identification, clock year/raw time, an analog input, a discrete input and battery value
through the built-in provider, without manually supplying widths.
