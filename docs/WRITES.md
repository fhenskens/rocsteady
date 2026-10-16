# Schema-aware parameter writes

OP-166 and OP-181 follow D301180X012 October 2022 §§2.20/2.23,
Tables 2-23/26, printed pp.2-37–39. Both services return an empty acknowledgement.
Software fixtures verify their fields; physical-controller interoperability remains
unvalidated. Version 0.2.0 includes both services and all 18 audited fields below,
including six additional PT-101 configuration settings.

## Planning and request budgets

`WritePlan::build` builds contiguous OP-166 batches: point type, location, count
and starting parameter precede 1–230 value bytes. Only adjacent parameters at the
same point/location coalesce. Parameter numbers never wrap past 255.

`WritePlan::build_arbitrary` builds OP-181 batches: a count byte precedes each
ordered three-byte TLP followed immediately by its schema-sized value. Parameters
can cross points, locations and types. Order and duplicates survive; no sorting,
deduplication, gap filling or automatic substitution of one service for the other
occurs. `WriteBatch::opcode()` and `payload()` expose the selected wire form.

OP-181 uses `MAX_ARBITRARY_WRITE_REQUEST = 240` as an explicit conservative
**library request-payload policy**, including count, TLPs and value bytes. This
allows at most 59 one-byte values in one batch. A single value can occupy 236 bytes,
independently of OP-166's 230-byte value ceiling. Tighter profile request limits
split batches or reject an indivisible value; the framework's 255-byte payload
ceiling cannot be relaxed. Both services expect a zero-byte reply.

Section 2.23 incorrectly calls the service opcode 180 and describes a 240-byte
*response* error despite Table 2-26's empty reply. Table 2-26 establishes packing;
it does not establish a 240-byte request boundary. The conservative policy is
not a claim that the source defect is resolved or that hardware accepts that
boundary. DEC-1.03 remains open. A caller with different applicable evidence can
supply an `Operation` through the managed extension API; ordinary framing and
ownership guarantees remain mandatory.

## Write-specific schema and built-in manifest

Both constructors check the entire input before admission. `SchemaProvider::lookup_write`
is a separate opt-in contract: read metadata and an R/W column alone do not
authorize writes. The default returns no writable schema. A provider must establish
applicability, access and all supported value constraints before returning metadata,
or reject the lookup. Conditional access must be resolved explicitly. `SchemaChain`
uses its ordered write lookups; provider errors stop fallback. The existing catalog
read behavior is unchanged.

Exact types/widths, logical bounds, printable ASCII and supplied inclusive numeric
bounds are validated locally. Numeric bounds reject NaN and infinities. This schema
shape does not express arbitrary enumerations or cross-field constraints; providers
must withhold fields whose constraints cannot yet be enforced. Discovery-bound
plans require the same connection generation. Execution rechecks the receiving
client's profile before admission and retains one exclusive workflow lease across
batches. It does not rediscover metadata during a write.

The built-in writable catalog contains **18 fields**, all introduced in 1.00 and
limited to the catalog's explicit reviewed firmware window 1.00–3.60. The manifest
records the source-audited subset, not complete PT write conformance:

| PT / parameter | Field | Type and enforced value constraints | Source |
| --- | --- | --- | --- |
| 91 / 2 | Station name | AC20; every byte 0x20–0x7e; logical zero | Table 3-5, p.3-34 |
| 101 / 0 | Point tag | AC10; every byte 0x20–0x7e | Table 3-13, p.3-61 |
| 101 / 1 | Scanning | UINT8; 0 disabled, 1 enabled | Table 3-13, p.3-61 |
| 101 / 2 | Input filter | FLOAT; finite 0–43,200 seconds inclusive | Table 3-13, p.3-61 |
| 101 / 4 | Invert mode | UINT8; 0 disabled, 1 enabled | Table 3-13, p.3-61 |
| 101 / 5 | Latch mode | UINT8; 0 disabled, 1 enabled | Table 3-13, p.3-61 |
| 101 / 9 | Alarming | UINT8; 0 disabled, 1 enabled | Table 3-13, p.3-61 |
| 101 / 10, 11 | SRBX on clear, SRBX on set | UINT8 each; 0 disabled, 1 enabled | Table 3-13, p.3-62 |
| 101 / 13 | Scan period | FLOAT; finite 0.004–43,200 seconds inclusive | Table 3-13, p.3-62 |
| 102 / 0, 1 | Point tag, units tag | AC10 each; every byte 0x20–0x7e | Table 3-14, p.3-63 |
| 103 / 0, 1 | Point tag, units tag | AC10 each; every byte 0x20–0x7e | Table 3-15, p.3-66 |
| 104 / 0, 1 | Point tag, units tag | AC10 each; every byte 0x20–0x7e | Table 3-16, p.3-70 |
| 105 / 0, 1 | Point tag, units tag | AC10 each; every byte 0x20–0x7e | Table 3-17, p.3-72 |

Every row is R/W with User update ownership. I/O fields require a discovery-bound
catalog and an installed matching point type at the **physical TLP location**.
Logical ordinals are converted through `IoSnapshot::by_logical(...).tlp(...)`.
All three explicitly selected layouts remain supported; no layout or firmware is
automatically inferred. Scanning disabled means field inputs are ignored; enabling
it permits device updates again. Filter is the duration for which the input must
remain ON before the state changes. These APIs set configuration values, not an
atomic output-control procedure.

### PT-101 configuration audit — 2026-10-16

Table 3-13, printed pp.3-61–62 (PDF pages 120–121), was rechecked in
text and rendered pages from the publisher's October 2022 PDF. All nine User-updated
R/W fields are now admitted: /0–2, /4–5, /9–11 and /13. No extra cross-field
prerequisite is specified for these six new settings. They retain the explicit
1.00–3.60 firmware and discovered physical-location gates.

Invert mode changes input interpretation; latch mode holds an active transition
until manually cleared. Alarming permits alarm-log generation, and the SRBX flags
request notification on alarm clear/set. Writing these flags does not poll, acknowledge
or subscribe to SRBX. The simulator stores configuration and supports explicit
read-back; it does not simulate field transitions, latch clearing, timing or generated
alarms. Setting a flag is not proof of an alarm or notification.

Both-updated /3 (status), /6 (accumulated value), /7–8 (cumulative times) remain
blocked pending a separate audit of live-value ownership and reconciliation.
System-only /12 (alarm code), /14 (actual scan time) and /15 (physical status)
remain unwritable. Enabling latch mode does not add a manual-clear procedure.
PT-101 full schema conformance, including defaults/ownership metadata and hardware,
remains open.

`REQ-DI-WRITE-*` and `POL-DI-*` in `tests/arbitrary_writes.rs` cover independent
OP-166/181 literals, all byte codes, adjacent representable FLOAT bounds, excluded
fields, preflight applicability, malformed acknowledgements, raw device rejections,
lost acknowledgements, stale plans and explicit read-back. All 18 fields are exercised
through both services at 1.00/3.60 and all three layouts. Existing generic write tests
continue to cover cancellation, deadlines and no-send behavior. The package consumer
also writes and reads back all six new fields through both services.

Supply exact-length strings including caller-selected padding. No truncation,
padding, configuration save or read-back is automatic. All other built-in writes
remain blocked, including disputed calibration, failsafe and module-dependent
fields (DEC-10.05/06/07). Custom providers do not establish built-in conformance.

## Configurable-table plans

`ConfigurableTable::write_plan` additionally creates OP-11 batches through the same
`Client::write_plan` outcome/send-gate machinery. It uses discovered PT-99 mappings,
validates mapped values against separate write opt-in schemas and rechecks the full
definition before every batch. The 18 audited writable target fields are unchanged.
See [table contracts](CONFIGURABLE_TABLES.md) for budgets, definition-read failures,
generation checks and the external-edit race caused by OP-11 having no revision field.

## Outcomes, interruption and reconciliation

`Client::write_plan` returns `WriteProgress`, or `SchemaWriteError` containing
progress and the original error. Each batch has one of these outcomes:

- `NotAttempted`: the batch never passed its send gate.
- `Acknowledged`: a valid empty reply for the selected write opcode was observed.
- `DeviceError`: OP-255 was received; every raw error/offset pair remains in the
  error. It does **not** prove rollback, atomic rejection or success of unreported
  parameters. OP-181 tuple-position wording does not establish individual commits.
- `Uncertain`: the exchange was permitted, but no valid acknowledgement or device
  error was observed. This includes lost/malformed replies and interrupted I/O;
  conservative uncertainty can include a failure before actual transmission.

Earlier acknowledged batches remain explicit; later batches stay unattempted after
failure. Acknowledgement establishes neither durable configuration storage nor a
stable read-back value. DEC-6.02/03 remain open: no per-parameter commit inference
is made from offsets. No write is replayed or resumed automatically after recovery.
Callers decide whether and how to reconcile device state before a new explicit call.
Timeout/stop closes a synchronized send gate before returning progress. Dropping a
caller future closes the gate too, but cannot return a result; effects already
permitted may remain uncertain. A replacement connection never continues that plan.

The simulator supports the manifest above for both write services and retains effects
across reconnects. Its deliberate fixture policy applies valid tuples in order,
retaining earlier effects when a later tuple fails. Unsupported widths stop parsing;
the next tuple boundary is never guessed. Error offsets are tuple positions for
OP-181 and parameter numbers for OP-166. This is a partial-effect test policy,
not controller atomicity evidence. Injected disconnects happen after model dispatch,
allowing lost-ack testing. Independent TCP peers supply the wire oracle separately.

```rust,no_run
# async fn example(client: &rocsteady::Client) -> Result<(), Box<dyn std::error::Error>> {
use rocsteady::{FirmwareVersion, Roc800Catalog, Value, WritePlan};
// Client must have the appropriate explicit I/O layout profile.
let io = client.discover_io(FirmwareVersion::new(3, 60)?).await?;
let di = io.by_logical(101, 0)?;
let ai = io.by_logical(103, 0)?;
let catalog = Roc800Catalog::from_discovery(io)?;
let writes = [
    (ai.tlp(0)?, Value::Ascii("FLOW TAG  ".into())),
    (di.tlp(2)?, Value::Float(0.5)),
    (di.tlp(4)?, Value::UInt8(1)), // Invert mode.
    (di.tlp(13)?, Value::Float(0.125)), // Scan period in seconds.
    (ai.tlp(1)?, Value::Ascii("m3/h      ".into())),
];
let plan = WritePlan::build_arbitrary(&catalog, &catalog.context(), &writes, client.profile())?;
match client.write_plan(plan).await {
    Ok(progress) => assert!(progress.batches.iter().all(|outcome|
        *outcome == rocsteady::WriteOutcome::Acknowledged)),
    Err(failure) => {
        eprintln!("{:?}: {}", failure.progress.batches, failure.error);
        // Reconcile device state before deciding whether to issue another explicit write.
    }
}
# Ok(()) }
```

Run `cargo run --features simulator --example parameter_writes` for OP-166 and
OP-181 writes with explicit read-back. The extracted package consumer also executes
cross-point OP-181 writes through the public API (`POL-181-PACKAGE`).

Requirement-linked independent literals/TCP, budgets, all three layouts,
malformed replies, raw errors, partial progress, deadlines, preflight/generation
rejection, queued caller cancellation and simulator effect/read-back are in
`tests/writes.rs` and `tests/arbitrary_writes.rs`. Hardware evidence: none.
Further delivery should audit each writable field's constraints individually,
including noncontiguous enumerations and cross-field conditions before enabling
those schemas. Broader applicability and DEC-6 evidence remain acceptance work.
