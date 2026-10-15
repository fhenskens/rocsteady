# Single-point history (OP-135)

Source: Emerson D301180X012, October 2022, §2.15/Table 2-18, printed
pp.2-31–32 (PDF pages 41–42), reviewed 2026-10-15. Physical/serial evidence: none.
OP-135 retrieves one point's archived values or timestamp column. Metadata
services OP-105/108 are described separately in [HISTORY_METADATA.md](HISTORY_METADATA.md).

## Ordinary calls and selected bounds

`Client`, `Session` and exclusive `ManagedContext` expose
`read_history(HistoryRequest)` and `read_history_with_codec(request, codec)`.
Ordinary reads require no custom profile. `Client::read_history_range(HistoryRange)`
splits a bounded window across pages while retaining one managed generation.

| HistoryKind | Selector | Decoded items |
| --- | --- | --- |
| Minute | 0 | IEEE FLOAT values |
| Periodic | 1 | IEEE FLOAT values |
| Daily | 2 | IEEE FLOAT values |
| PeriodicTimestamps | 3 | Raw UINT32 epoch-second values |
| DailyTimestamps | 4 | Raw UINT32 epoch-second values |

No timestamp-to-timezone conversion, inferred calendar year or timestamp/value
coercion occurs. Exact payload bytes are retained in HistoryPage::raw, including
IEEE NaN, infinity and signed-zero bits. Timestamp words with the same bits stay
u32 values, not floats. Selecting a column does not establish an atomic snapshot.

| Named policy | Defaults/assumptions | Evidence and limits |
| --- | --- | --- |
| TableHistoryPolicy (default) | Segments 0–10; minute indices 0–59 (60 slots); periodic/daily capacities and point counts unknown | Follows Table 2-18 and the corroborating 60-value capacity lead in DEC-4.03. No controller verification. |
| ProseMinuteHistoryPolicy | Explicitly provisional: minute indices 0–60, treated as 61 distinct slots; other bounds as above | Follows competing §2.15 prose. No verified firmware applicability; does not alias index 60 to zero or invent a sentinel. |
| ConfiguredHistoryPolicy | Explicit caller-asserted segment/point bounds and minute/periodic/daily capacities | Use independently known device configuration. This is not a predefined firmware capacity matrix. |
| Consumer HistoryIndexPolicy | Return checked HistoryBounds for each segment/point/selector | Same public interface as built-ins; can reject unsupported applicability. |

Request/range `with_policy` constructors select the policy explicitly and store
its checked geometry, without retaining borrowed state. Ordinary constructors
select TableHistoryPolicy. There is no format probing or fallback to another
index interpretation after rejection. Unknown periodic/daily capacities stay
unknown; the client does not infer them from the returned current index or the
manual's historical 35-day example. Geometry accepts at most 256 points and
65536 slots, reflecting wire address space, not verified device support.

DEC-4.03 remains open: a tested software choice does not establish which minute
interpretation a particular controller uses. Expanded segment applicability
(DEC-4.04) remains caller-asserted; no automatic Series-2 preset is selected.

## Page contract

The six-byte request is segment, point, selector, little-endian starting index,
then count 0–60. Zero count requests the current index without samples. The default
segment range is 0–10; actual point counts remain device configuration, so an
unconfigured point can return a healthy device rejection.

A page never crosses a selected ring boundary. With unknown capacity it also
never crosses the u16 index-space boundary. Use the range helper to split a known
ring window, including a wrap. Invalid request bounds fail locally.

The reply is segment/point echoes, a little-endian u16 current index, returned
count, and exactly four bytes per returned item. Minimum/maximum payload sizes
are 5 and 245 bytes. The decoder validates both echoes, returned count no greater
than requested, exact item widths, no trailing data, and current-index bounds
when capacity is known. The wire reply has **no selector or starting-index echo**;
the client cannot claim correlation fields the device does not send.

A valid short/zero-count reply to a positive request is retained and treated as a
short page. The client does not fill missing values, infer why data is unavailable,
or silently skip to another position. Zero-count probes reject returned samples.

`ProtocolProfile::decode_history` can replace the reply decoder. The default
HistoryCodec is itself a public ReplyCodec<HistoryRequest>. Per-call codec
selection can return arbitrary consumer output types. Ordinary page/range methods
also validate that a profile's typed output retains the request, count, sample
kind and selected current-index bounds under the exchange retirement guard.
For different payload widths/budgets, use a custom Operation/DecodedOperation;
transport framing, correlation and cancellation rules remain mandatory.

## Bounded windows, movement and partial progress

HistoryRange is limited to one selected ring. If capacity is unknown, its window
is bounded by the remaining unwrapped u16 address space; wrap is never guessed.
The page planner obeys the 60-item limit, selected profile request/reply budgets,
and each known boundary. A total of zero produces one index-only request.

The whole range runs as one exclusive Workflow: competing calls and heartbeats
cannot consume its replies. It does not freeze the controller's archive.
By default HistoryContinuity::RequireUnchangedIndex stops on an observed change
between page headers with Error::HistoryChanged, preserving the healthy connection.
HistoryContinuity::ObserveIndexChanges explicitly continues the bounded window
and sets index_changed. No option proves gap-free or durable continuity: equal
indices can conceal a full wrap, identical rewrites, resets or unobserved updates.

HistoryReadProgress contains only decoded pages, their generation, next_index,
remaining items, complete/short_page flags and index_changed. The page which first
reveals index movement is included and flagged. A strict movement error keeps
complete=false even if the final page happened to exhaust the window. A short
page returns successfully with short_page=true and complete=false; it sends no
further request. A zero-count probe completes without moving the position.

`next_index` advances by actual returned items. It wraps only under a known
capacity; it becomes None after the last u16 slot when capacity is unknown.
Positions are not durable cursors, automatic resume tokens or controller identity.

HistoryReadError exposes the same confirmed prefix when later work fails, times
out or is stopped. An application may select a fresh window using the returned
position/remaining count after checking identity and its continuity requirements.
The client never retries a page, replays a failed window, combines generations,
or silently resumes after replacement. Aborting the calling future cannot return
a progress result, but still cancels/retire active work. Call timeout includes
admission and every page; response timeout bounds each exchange.

Local validation and valid device rejection preserve the channel. Malformed
replies and interrupted exchanges retire it before another call uses it. Profile
budgets are checked before admission and again at exchange; a single page which
cannot fit fails locally, while range reads split into smaller valid pages.

## Executable evidence

`tests/history.rs` maps requirement IDs to independent literal and TCP assertions:
all selectors, count 0/60, 245-byte replies, wire endianness, every truncation,
echo/count/index/width failures, policy boundaries, raw float bits, known-ring
splitting, short replies, observed movement, confirmed failure progress, explicit
new windows, deadline/cancellation retirement, and public policy/codec/profile
replacement. Standalone and exclusive workflow calls use the same codecs.
The standalone malformed-response regression in tests/integration.rs includes 135.

`HistoryMetadataModel` now additionally accepts explicit four-byte sample vectors
and per-selector current indices; the existing Model shape remains compatible.
`tests/history_simulator.rs` covers all selectors, index-only requests, retention
boundaries, missing points and invalid requests. Independent fixture literals are
in examples/support/history_literals.rs. The packaged consumer tests default
reads, timestamp separation, configured paging and an external codec.

Run `cargo run --locked --example history` for a hardware-free example. The
[requirements index](REQUIREMENTS.md) and [coverage matrix](COVERAGE.md) record
scope; software fixtures do not establish physical interoperability or close DEC-4.
OP-136 multi-point reads are described in [MULTI_HISTORY.md](MULTI_HISTORY.md);
OP-137/138 day services are described in [DAY_HISTORY.md](DAY_HISTORY.md).
OP-139 configured inventory and selected lists are described in
[HISTORY_INFORMATION.md](HISTORY_INFORMATION.md). Transactions remain separate work.
