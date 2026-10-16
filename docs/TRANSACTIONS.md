# Transaction inventory and bounded retrieval

Source: Emerson D301180X012 October 2022 §2.26/Table 2-29, printed pp.2-42–43
(PDF pages 52–53), text and rendered tables audited 2026-10-16. OP-206 was
introduced in 3.50. The 0.2.0 APIs require `TransactionSegment`, an explicit
caller assertion of segment and reviewed firmware 3.50–3.60. They do not discover
firmware, controller identity, segment installation or retention geometry.
All byte-valued segment numbers can be represented; the controller can reject them.
`DefaultProfile` advertises OP-206 as provisional; raw/custom operations remain available.

## Inventory (command 1)

`TransactionListRequest::new(segment, offset)` encodes command 1, segment and
little-endian UINT16 storage offset. Offset zero denotes the first storage position;
after rollover it need not be the oldest transaction. `list_transactions` returns
one `TransactionListPage`: raw ten-byte description, UINT16 payload size, ordered
transaction-number/creation-TIME pairs and a Boolean continuation flag. A response
has 15 header bytes plus six bytes per entry, so at most 40 entries fit a 255-byte
unicast payload. Command, count, flag and exact length are validated. Text and TIME
are preserved without assuming printable ASCII or a date conversion.

`transaction_inventory(request, max_pages)` retains one exclusive generation and
advances by returned entry count, never transaction number or creation time.
It preserves duplicates and device order. Changed description/payload size stops
with `HistoryChanged`; a continued empty page, offset overflow or exhausted page
budget also stops explicitly. `TransactionInventoryError.progress` retains confirmed
pages and the next offset. `complete` means the final response had `more=false`,
not that inventory is atomic, unchanged or gap-free. Matching metadata does not
prove persistent controller identity, retention or absence of overwrites.

## Data (command 2)

`TransactionDataRequest::new(segment, number, offset)` encodes command 2, segment,
UINT16 transaction number and UINT16 byte offset. The offset is at **full-frame
position 10**, or payload position 4, as printed in Table 2-29. The single-page
`read_transaction_page` API returns raw bytes. Its size byte counts all bytes below
it, including the continuation flag, as the table states. Each response has at most
252 data bytes; command, exact size and flag are checked. Zero data with continuation
is rejected as no progress; zero data without continuation is representable.

`read_transaction(segment, number, payload_size, max_pages)` starts at byte zero,
using the caller-supplied inventory payload size as an explicit length assertion.
It reassembles bytes before field interpretation, so tags, numeric values, TLPs and
strings can cross pages. It rejects overlong data, premature termination and
continuation at/after the declared size. Page budgets must be 1–65,536; payload size
is UINT16, including zero. It checks request/minimum-response profile budgets before
admission and actual reply limits during each exchange.

`TransactionReadError.progress` preserves only previously confirmed valid pages,
raw bytes and the next offset. An invalid new page is excluded. A valid missing/
overwritten-transaction device rejection remains distinct from malformed replies
and transport failures. The library cannot detect a same-length record replacement
when the protocol returns no identity/number/offset echo. The caller must establish
applicable identity and freshness; a supplied payload size does not establish them.

Both workflows retain one exclusive lease. Call/exchange deadlines, cancellation
and awaited stop follow existing ownership rules. No failed inventory or data read
is replayed or resumed on a replacement generation. A budget-limited result's next
offset is diagnostic progress, not permission for automatic resume. Dropping a
future cannot return progress. Single pages are also available on Session and
ManagedContext; request structs implement `Operation` for explicit custom workflows.

## Explicit field interpretation and unresolved source issues

The default data API stays raw because Table 2-29 labels individual fields as
**Data Type / Value**, then says **TLP, Data Type, Value** in its repetition text.
DEC-5.06 remains unresolved. `decode_transaction_fields(bytes, layout)` therefore
requires an explicit `TransactionFieldLayout`:

- `TaggedValues`: type tag followed by value; no implicit TLP.
- `TlpTaggedValues`: a three-byte TLP precedes every tag/value pair.

These are selectable provisional interpretations, not discovered/verified device
formats. Both preserve each field's raw value bytes. Tags are independent of event
vocabulary: 1 U8, 2 S8, 3 U16, 4 S16, 5 U32, 6 S32, 7 FLOAT, 8 DOUBLE,
9 STRING3, 10 STRING7, 11 STRING10, 12 STRING20, 14 STRING30, 15 T_STRING40,
17 one-byte BINARY, 18 three-byte TLP and 20 four-byte TIME. String tags use their
named fixed widths and remain raw, including nonprintable bytes; no additional
semantics are inferred from the source's `T_STRING40` spelling. Numeric values
retain IEEE payloads, including NaNs/infinities. Unknown tags have no established
width and produce an error without guessing another field boundary. The separately
owned raw record remains available even if interpretation fails.

The printed request offset now has explicit table-backed implementation evidence,
but DEC-5.05 is not closed as a physical-interoperability claim. DEC-5.06/07 remain
open for actual TLP framing, split-page behavior and applicable firmware transcripts.
A consumer with a different supported wire contract can implement `Operation` and
`Workflow` through the existing managed extension APIs. This does not establish
built-in conformance or relax mandatory framing/ownership checks.

```rust,no_run
# async fn example(client: &rocsteady::Client) -> Result<(), Box<dyn std::error::Error>> {
use rocsteady::{FirmwareVersion, TransactionSegment, TransactionListRequest};
let segment = TransactionSegment::new(13, FirmwareVersion::new(3, 50)?)?;
let inventory = client.transaction_inventory(TransactionListRequest::new(segment, 0), 16).await?;
for page in inventory.pages {
    for entry in page.entries {
        let record = client.read_transaction(segment, entry.number, page.payload_size, 512).await?;
        println!("{}: {} raw bytes", entry.number, record.bytes.len());
    }
}
# Ok(()) }
```

Run `cargo run --features simulator --example transactions` for a local fixture
with deliberate page splits and explicit field interpretation. The simulator uses
`HistoryMetadataModel::transactions` and `TransactionSegmentModel` with raw records,
explicit payload sizes and configurable page sizes. It preserves supplied storage
order and does not implement rollover, retention, live acquisition or controller
identity. Injected errors/disconnections remain fixture policies, not device evidence.

`tests/transactions.rs` uses `REQ-206-*`/`POL-206-*` annotations and independent
TCP literals, all tags and truncations, maximum sizes, progress/failure matrices,
profile preflight, standalone retirement, exclusive ownership and cancellation.
The extracted package consumer exercises both commands, reassembly and interpretation.
Physical-controller and actual serial evidence remain none.
