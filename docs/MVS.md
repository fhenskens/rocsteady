# Multi-variable sensor reads and caller-driven monitoring

Source: Emerson D301180X012, October 2022, §3.4.19/Table 3-20, printed
pp.3-82–89 (PDF pages 141–148). Text audited 2026-10-15. PT-108 adds all
69 read-field definitions, bringing the 0.1.3 catalog to 332 entries. MVS access
is read-only; the separate twelve-field writable subset and history services
are included in the same release. Physical-controller/serial validation
and full point-type conformance remain outstanding.

## Read contract and applicability

Use the existing explicit ROC800 layout profile, `Client::discover_io`,
`IoSnapshot::by_logical(108, ordinal).tlp(parameter)` and
`Roc800Catalog::from_discovery`. The source allows six logical points per module;
installed points and physical TLP locations come from discovery. All three
explicit layouts remain available under their existing applicability rules.

All fields are introduced in 1.00 except /64–65 (2.02), /66 (3.20) and /67–68
(3.60). The catalog's reviewed firmware window remains 1.00–3.60; it never
infers firmware from a successful parse. No PT-108 write is enabled, including
tag, poll-mode and calibration-command fields.

The configuration-change counter /67 is valid for 4088 sensors and explicitly
invalid for MVS205. Firmware and I/O point type alone do not establish sensor
identity. Ordinary catalog lookup therefore rejects /67 until the caller supplies
`catalog.with_mvs_4088(physical_location)`. This consumes the catalog and returns
a new one carrying an explicit per-location assertion that the sensor is 4088 A/B.
It requires firmware 3.60 and a matching discovered PT-108 position. It does not
read /68 or infer identity from bytes. Callers can first read /68 to obtain a
sensor code and then make their own applicability decision using device context.
Undefined, 3095, MVS205 or unknown identities are not automatically admitted.

The assertion is bound to the discovery generation. After replacement, a fresh
catalog has no 4088 assertions; the caller must establish applicability again.
A successful counter read neither proves persistent controller identity nor
establishes an atomic snapshot or a universal configuration-change cache policy.

`read_schema` retains ordered values and decoded-batch progress, using bounded
OP-180 batches. Local validation, healthy device rejection, malformed replies and
transport failures remain distinct. Discovery-derived providers/plans cannot be
reused after generation replacement. Recovery never replays or resumes a failed
read. Custom `SchemaProvider`/`SchemaChain` metadata remains available with
applicable evidence and the same framework guarantees.

## Values, units and source caveats

Pressure/temperature readings, calibration points, limits, reverse DP and fault
values remain raw FLOATs. PT-108/3 selects English (0) or metric (1) units:
DP uses inches of water or kPa, SP uses PSI or kPa, and temperature uses Fahrenheit
or Celsius. /4 selects the source's 60/68 °F water reference; /5 identifies the
pressure tap. These fields are metadata, not instructions to convert values in
this library. Fixed units are supplied only for sensor voltage /8 (`V`).

Unknown byte codes, unused bits and all IEEE float payloads remain readable.
Five BIN fields have separate masks: sensor alarms /10, range status /11 and
DP/SP/temperature alarms /28, /43, /58. Labels describe set bits without locally
computing alarms or assuming the alarm fields are populated when alarming is off.
Sensor SRBX flags /62–63 apply to communication failures according to the table;
no unsolicited subscription or acknowledgement is enabled by reading them.

The following source details stay explicit in metadata and DEC-10.09:

- Poll-mode codes /2 are 0, 1, 2, 4 and 5; no universal inclusive range is selected.
- Calibration-command /59 prints range 0–6 but documents code 7 for set defaults,
  with gaps at 4–5. The labels retain code 7; numeric bounds remain unspecified.
  Reading a command field never executes it.
- SP zero shift /64 names inches of water while the other SP rows use PSI. No
  fixed unit or conversion is guessed for that field.
- Module type /65 is **FLOAT**, despite documented codes 0 standard MVS, 1 smart
  MVS and 2 3095. It is not cast to a byte enum or filtered to those values.
- Sensor type /68 is a separate UINT8 field: 0 undefined, 1 4088 A, 2 4088 B,
  3 3095, 4 MVS205. It is not interchangeable with /65.

```rust,no_run
# async fn read(client: &rocsteady::Client) -> Result<(), Box<dyn std::error::Error>> {
use rocsteady::{FirmwareVersion, Roc800Catalog};
let io = client.discover_io(FirmwareVersion::new(3, 60)?).await?;
let sensor = io.by_logical(108, 0)?;
let catalog = Roc800Catalog::from_discovery(io)?;
let fields = [3, 19, 35, 50, 10, 11, 68].map(|p| sensor.tlp(p)).into_iter()
    .collect::<Result<Vec<_>, _>>()?;
let values = client.read_schema(&catalog, &catalog.context(), &fields).await?;
println!("Units, DP, SP, temperature, alarms and sensor type: {values:?}");
// Only after independently establishing this point is a 4088 A/B sensor:
let catalog = catalog.with_mvs_4088(sensor.location)?;
let counter = client.read_schema(&catalog, &catalog.context(), &[sensor.tlp(67)?]).await?;
println!("4088 configuration counter: {counter:?}");
# Ok(()) }
```

## Polling example

The example-owned `examples/support/monitoring.rs::poll_once` performs exactly
one sample: explicit discovery, logical-to-physical resolution, planning and one
bounded schema-read workflow. It returns a generation, ordered TLPs and values,
or the original `SchemaReadError` including confirmed partial progress. Discovery
and planning errors have empty read progress. No background monitoring task is
created. Read batches and discovery are individually bounded managed calls; this
example does not add a whole-sample deadline spanning both phases.

Each invocation refreshes discovery, so the next sample after reconnect can use
changed physical positions. Replacement between discovery and reading is still
reported as an error; that sample is not retried. Firmware remains a caller
assertion, and discovering again does not prove persistent controller identity.
The helper does not opt into /67 or issue writes. It can be canceled by dropping
its future; existing managed cancellation/retirement guarantees apply.

`temperature_monitoring` calls this helper in a finite loop, sleeping the caller's
interval after each completed sample. It has no catch-up bursts. Failures are
reported with progress and consume an attempt; a later scheduled sample is a new
explicit call. The caller chooses attempt count, interval, stopping and whether
another sample should be requested. The default runs three samples, 100 ms apart.
The controller-free fixture deliberately includes raw values and unknown bits:

```sh
cargo run --locked --example temperature_monitoring -- 5 250
# Inject one failed sample; the following caller-scheduled sample re-discovers:
cargo run --locked --example temperature_monitoring -- 3 1 --demo-reconnect
```

The demo requires at least three attempts and checks one failure with no sample
replay. CI runs this variant. Application code should supply its own endpoint,
independently established identity, targets, cancellation and error-handling policy;
the example is not a production identity/cache or durable-ingestion strategy.

`tests/mvs_catalog.rs` covers all 69 fields, source metadata, enum gaps, all BIN
byte combinations, firmware boundaries, physical locations, the 4088 gate,
unknown/IEEE values, partial progress, rejection and stale plans. Independent TCP
literals and simulator datasets are separate evidence. `tests/monitoring.rs`
checks caller-owned sampling, one failed sample, explicit re-discovery with changed
physical locations and retained partial progress. The extracted package consumer
reads DP/SP/temperature, metadata, FLOAT module type and an explicitly enabled
4088 counter. Hardware evidence: none.
