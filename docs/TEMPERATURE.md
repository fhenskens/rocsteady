# RTD and thermocouple catalog reads

Source: Emerson D301180X012, October 2022, §§3.4.17–18, Tables 3-18/19,
printed pp.3-75–81 (PDF pages 134–140). Text audited 2026-10-15. The
0.2.0 catalog includes all 38 PT-106 RTD and 23 PT-107 thermocouple read fields.
Together with the [PT-108 fields](MVS.md), the catalog contains 332 definitions. Physical-controller and actual serial
validation remain outstanding. No full PT conformance claim is made.

Use `Client::discover_io` with independently established firmware and an explicitly
selected I/O layout, then `IoSnapshot::by_logical(...).tlp(...)` and
`Roc800Catalog::from_discovery`. TLPs use discovered physical locations, not the
per-type logical ordinal. The same catalog supports Compatibility160,
Compatibility240 and Expanded224 with their documented applicability. The source
allows two/three RTD points and five/four TC points per module variant; discovery,
not a guessed fixed module size or subtype, determines available positions.

`Client::read_schema` uses bounded OP-180 batches, retaining caller order and
confirmed progress. Plans and providers from discovery are generation-bound.
Malformed replies retire the connection; valid device rejections remain explicit.
After replacement, obtain fresh discovery before constructing another catalog plan.
Multi-batch reads are not an atomic snapshot and never replay or resume implicitly.
Raw `ParameterBlock`/OP-167 reads remain available with caller-supplied widths;
this slice adds no automatic schema-backed OP-167 planner.

`CatalogEntry.metadata` supplies optional enums, alarm masks, fixed units and source
caveats. Unknown byte codes, unused alarm bit 5, out-of-range observations and raw
IEEE float payloads remain readable. Numeric metadata never filters readings.
EU values and alarm limits have **no fixed temperature unit**: read PT-106/37 or
PT-107/3. Codes 0–3 mean Fahrenheit/Celsius/Kelvin/Rankine; RTD adds code 4 for
Ohms. Units tags are independent strings. No conversion, cold-junction correction,
calibration computation or local alarm evaluation is inferred.

All fields are introduced in 1.00 except RTD offset /19 (3.00) and TC failsafe
/21–22 (3.50). Explicit catalog applicability remains 1.00–3.60. Version 3.40
changes the *default* units selection according to the tables; it does not establish
the device's current configured unit. Read the selector rather than infer it from
firmware, tag text or a presumed default.

## Source caveats and write restrictions

Every PT-106/107 write is blocked by `Roc800Catalog::lookup_write`, including tags.
The current eighteen-field OP-166/181 writable subset excludes temperature fields. An R/W source
column or readable calibration field does not authorize a write.

- PT-106/2 and /7 use `R/W_LOG`; /3, /5 and /6 use `R/W_CNDL`. Coarse readable
  access is represented as `ReadWrite`, with the source token retained in
  `write_caveat`; no logging procedure or write precondition is invented.
- PT-106/20 set-value bounds depend on units and alpha. The table suggests external
  R/O despite its R/W column; no universal numeric range is selected. /35 is not
  event logged; /36 also suggests external R/O. Calibration type 6 means **unused**,
  unlike the offset code in the analog-input table.
- TC /4 has sensor codes 0–6 and 8–10; 7 has no supplied label. Original five-point
  hardware only supports J/K. Labels describe source codes without selecting a
  module subtype or claiming every code is valid on every module. A single inclusive
  numeric range is deliberately absent.
- TC /10, /13–15 and /19 contain copied RTD references, including a clipping field
  absent from the TC table. Bit positions, raw values and alarm labels are clear;
  no missing clipping parameter or alarm algorithm is invented.
- RTD /33.7 and TC /19.7 repeat “disabled” in both set/clear prose. The metadata
  labels the set bit `scanning_disabled`; it does not implement the disputed inverse.
- TC /22 repeats the name “Failsafe Mode” but has FLOAT width and a failsafe-value
  description. The technical key `tc_failsafe_value` identifies that description;
  the source naming defect remains explicit.

These items are tracked in DEC-10.08. Callers can use `SchemaProvider`/`SchemaChain`
for applicable alternative metadata; custom schemas do not close built-in source
or hardware questions. Framework budgets, ownership and generation rules remain.

```rust,no_run
# async fn read(client: &rocsteady::Client) -> Result<(), Box<dyn std::error::Error>> {
use rocsteady::{FirmwareVersion, Roc800Catalog};
// Configure the client with its explicit ROC800 I/O layout first.
let io = client.discover_io(FirmwareVersion::new(3, 60)?).await?;
let rtd = io.by_logical(106, 0)?;
let tc = io.by_logical(107, 0)?;
let catalog = Roc800Catalog::from_discovery(io)?;
let values = client.read_schema(&catalog, &catalog.context(), &[
    rtd.tlp(22)?, rtd.tlp(37)?, rtd.tlp(33)?,
    tc.tlp(9)?, tc.tlp(3)?, tc.tlp(4)?, tc.tlp(19)?,
]).await?;
println!("Raw temperature values, selectors and alarms: {values:?}");
# Ok(()) }
```

Run `cargo run --locked --example temperature_monitoring` for three caller-paced
RTD/TC/MVS samples against an independent TCP peer. The [polling guide](MVS.md#polling-example)
describes caller-selected counts/intervals and explicit re-discovery after reconnect. Tests in
`tests/temperature_catalog.rs` cover the full manifest, every documented enum code,
all alarm-byte combinations, firmware boundaries, all layouts, rejected locations,
blocked writes, raw IEEE values, malformed replies, healthy rejection, generation
replacement and confirmed partial progress. `tests/discovery.rs` now reads all 332
catalog fields against independent literals. Simulator datasets cover all 61
new fields; the extracted package consumer reads both schemas and their metadata.
