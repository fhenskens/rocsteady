# Configurable-table reads and writes

Source: Emerson D301180X012 October 2022 §§2.5–2.6, Tables 2-5/6, printed
p.2-12, and §3.4.10/Table 3-11, printed pp.3-56–57. Text and rendered PDF
pages 22/115/116 audited 2026-10-16. All forms were introduced in 1.00;
managed discovery requires caller-established ROC800 firmware in the reviewed
1.00–3.60 window. Physical validation is deferred outside current software scope.

## PT-99 definitions and snapshots

A table has logical number 0–15. `TableDefinitionRequest::new(table)` reads all
45 PT-99 parameters with OP-167: revision /0 is a four-byte FLOAT; /1–44 are
three-byte TLPs. The request is `[99, table, 45, 0]`; the reply echoes all four
bytes and contains 136 value bytes, for an exact 140-byte payload. Raw TLPs and
all revision bits are retained. Generic OP-180/167 access remains available;
these typed definition reads do not add entries to the 332-entry ROC800 catalog.

`TableRevision` compares exact bits and offers no ordering. NaNs, signed zero and
infinities are retained; the library does not increment, sort or infer revision
semantics. Table 3-11 says revision and all TLP fields are User-updated R/W, with
zero defaults. This interface reads existing configuration; it does not authorize
changing PT-99 revision or mappings. Table configuration writes remain separate work.

`Client::discover_configurable_table(table, firmware)` compares two complete
observations under one exclusive lease and returns a cloneable `ConfigurableTable`.
It is bound to the creating client/generation using an opaque generation token.
A snapshot is not persistent device identity. It cannot be reused on another client
or after connection replacement, and no automatic table cache or rediscovery exists.
Repeated equality is an observation, not an atomic configuration snapshot.

## OP-10 reads

`TableReadPlan::build(snapshot, provider, context, start, count, profile)` resolves
readable mapped parameter schemas before admission. It preserves table order and
duplicates. Table locations 0–43 correspond to PT-99 parameters 1–44; a range must
contain 1–44 slots and end at/before 44. Selected zero TLPs are rejected as
unconfigured; gaps outside the selected range are retained and need not be filled.
The source footnote limits reviewed target point types to 91–98 and 100–135.
Definitions preserve other raw TLPs, but built-in planners reject their selection;
known supplemental applicability can use the public Operation/Workflow interfaces.
Logical/parameter applicability comes from the selected schema provider.

Each OP-10 request contains table/start/count. Its reply echoes those bytes, then
four revision bytes and schema-sized values. The planner accounts for all seven
header bytes and splits contiguous table slots by actual profile/unicast response
budgets. The 255-byte payload-field ceiling is library framing capacity, not a
verified controller/channel boundary; tighter profiles remain available (DEC-1).
Exact echoes and lengths, scalar widths and normal printable-ASCII values
are validated; unknown numeric codes/bits and IEEE values remain readable.
A changed revision is reported as `DiscoveryChanged`, including when widths changed,
without decoding values under the old schema. Valid revision changes preserve the
connection; malformed wire replies retire it. Raw bytes remain in confirmed replies.

`Client::read_configurable_table(plan)` checks the full mapping/revision before
reading and after the final batch, retaining one exclusive generation throughout.
`TableReadError.progress` contains only confirmed batches; `complete` stays false
on a failed postcheck even if all values were received. Matching checks cannot
detect changes that revert, same-width semantic changes between observations or
an external client's concurrent edits. No atomic multi-value snapshot is claimed.

Single definitions/batches are also available on Client, Session and ManagedContext.
`TableDefinitionRequest` and `TableReadBatch` implement `Operation` for explicit
caller workflows. The convenience methods distinguish revision changes after wire
validation; generic Operation decoding reports its decode failure through the normal
extension retirement policy. Custom wire variants can use those same public seams.

## OP-11 writes

`snapshot.write_plan(provider, context, start, values, profile)` returns the existing
`WritePlan`, with opcode 11 batches. Every mapped value must pass the provider's
separate opt-in `lookup_write` contract, exact type/width, location, printable text
and numeric checks. Read metadata alone never authorizes a write. The entire input
is checked before admission; schema-generation mismatch, unconfigured/unsupported
slots, invalid values and indivisible oversized values fail locally.

The request is table/start/count followed directly by ordered values, without a
revision or TLPs. The planner accounts for its three-byte header, splits by the
selected request budget and never reorders, deduplicates or fills table gaps.
The response must be empty. Execution revalidates the receiving profile and checks
the complete PT-99 definition **before each batch**, under the shared write lease.

The existing synchronized write send gate and `WriteProgress` apply. A failed
metadata check leaves that batch `NotAttempted`, even if the check itself reached
the device. Earlier acknowledged batches remain explicit. Once the OP-11 exchange
is permitted, lost/malformed replies, deadline or interrupted I/O are conservatively
`Uncertain`; a valid OP-255 is `DeviceError` with raw codes/offsets and no commit
inference. Dropping a queued future closes the gate. No write is replayed or resumed.

OP-11 has no expected-revision field and no compare-and-set operation. An external
configuration edit can race the preceding check; an acknowledgement cannot prove
which mapping was used, atomicity or durable storage. Callers requiring stronger
coordination must arrange it externally. Read-back is a separate explicit observation;
reconcile uncertain effects before deciding whether to make another write.
The 18-field audited built-in writable target subset is unchanged; custom providers
must establish their own target constraints. PT-99 configuration writes remain blocked.

```rust,no_run
# async fn example(client: &rocsteady::Client) -> Result<(), Box<dyn std::error::Error>> {
use rocsteady::{FirmwareVersion, Roc800Catalog, TableReadPlan, Value};
let firmware = FirmwareVersion::new(3, 60)?;
// Example assumes table 0 slot 0 is configured to the writable station-name TLP.
let catalog = Roc800Catalog::new(firmware)?;
let table = client.discover_configurable_table(0, firmware).await?;
let plan = TableReadPlan::build(&table, &catalog, &catalog.context(), 0, 1, client.profile())?;
println!("{:?}", client.read_configurable_table(plan).await?.batches);
let plan = table.write_plan(&catalog, &catalog.context(), 0,
    &[Value::Ascii("Example station     ".into())], client.profile())?;
println!("{:?}", client.write_plan(plan).await?.batches);
# Ok(()) }
```

Run `cargo run --features simulator --example configurable_tables` for discovery,
mixed reads/writes and explicit read-back through the audited target catalog.
Simulator tables are explicit mutable fixtures in `Model::configurable_tables`.
The model supports PT-99 reads and OP-10/11 mapped values. Its OP-11 fixture policy
applies valid values in order, retaining earlier effects on a later rejection;
error offsets denote table slots in that fixture, not verified hardware semantics.
`Simulator::inject_for(opcode, fault)` holds a FIFO fault until the matching request
and applies it after dispatch, allowing lost-write-ack tests without corrupting the
preceding metadata exchange. It does not establish controller atomicity or retention.

`tests/configurable_tables.rs` annotates `REQ-099-*`, `REQ-010-*`, `REQ-011-*`
and `POL-TABLE-*`. Independent TCP literals, malformed data, bitwise revisions,
range/budget boundaries, map changes, read progress, write outcomes, stale/foreign
snapshots, exclusive ownership and cancellation are separate from simulator evidence.
The extracted package consumer covers public planning, execution and read-back.
