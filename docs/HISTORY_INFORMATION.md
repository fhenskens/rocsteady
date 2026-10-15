# History-information commands (OP-139)

Source: Emerson D301180X012, October 2022, §2.19/Table 2-22, printed p.2-36
(PDF page 46), reviewed 2026-10-15.
[Baseline manual](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Physical-controller and actual serial evidence: none.

## Defaults and public extension contracts

Client, Session and exclusive ManagedContext expose read_configured_history and
read_selected_history, with corresponding _with_codec methods. ConfiguredHistoryCodec
and SelectedHistoryCodec are named public ReplyCodec defaults following Table 2-22.
ProtocolProfile::decode_configured_history and decode_selected_history can replace
them, including calls within range workflows. Per-call codecs can return arbitrary
consumer outputs. Ordinary typed results additionally retain request, point/period
shape, timestamp presence and known current-index validation under the exchange
retirement guard. Request-layout alternatives use public Operation/DecodedOperation.

Default applicability uses segments 0–10, table minute indices 0–59 and unknown
periodic/daily capacities or point counts. ConfiguredHistoryRequest::with_segment_limit
and selected request/range with_policy constructors explicitly select alternatives.
HistoryIndexPolicy, TableHistoryPolicy, configured geometry and provisional prose
minute bounds use the same public interfaces as the other indexed services.
All selected points must have coherent policy geometry. No firmware, geometry or
interpretation is selected from successful parsing or a returned cursor. DEC-4.03/04
and device/channel applicability remain open; policy overrides do not close them.

## Command 0: configured-point inventory

ConfiguredHistoryRequest encodes command 0 and segment. The reply contains the
command/segment echoes, a UINT8 count and exactly that many point-number bytes.
ConfiguredHistoryPoints preserves raw bytes and list order, including duplicates;
it does not sort, invent parameter schemas or infer capacities. Empty inventory is
valid. The 255-byte payload field allows at most 252 points after the header.
This is a wire-field budget, not controller interoperability evidence (DEC-1).

Command 0 has no request offset/count or continuation contract. The client does
not invent inventory pagination or silently truncate an oversized reply. Profile
request/header minimums are preflighted; actual reply budgets are checked under
the stream guard. Larger inventories need applicable firmware definitions and an
explicit supported extension rather than an unverified next-page request.

## Command 1: selected-point values and optional timestamps

SelectedHistoryRequest encodes command 1, segment, little-endian start index,
selector, period count, timestamp flag, point count and the explicit point list.
Selectors are Minute (0), Periodic (1) and Daily (2); timestamps are a separate
boolean flag, not OP-135 selectors 3/4. Request order and duplicate points are
retained. Point 255 is an ordinary selected point, not OP-138's timestamp sentinel;
its applicability still depends on the selected geometry/device configuration.

The library API requires 1–247 points, bounded by the eight-byte request header
and 255-byte payload field. Nonempty selection is a local contract: the table does
not establish zero-point semantics. Caller Operations remain available for a
separately evidenced zero-point form. Zero periods is supported as an explicit
header/current-index probe; it does not establish configured membership.

The source limit is at most 60 four-byte elements: points * periods without
timestamps, or (points + 1) * periods with timestamps. One page cannot cross a
selected ring boundary or the unwrapped UINT16 index-space boundary. The response
header is seven bytes, so a full 60-element page is 247 bytes. Both request and
response profile budgets apply; request points are never silently split.

The reply echoes command/segment, reports a UINT16 current index, period count,
timestamp flag and point count, then time-major rows. Each SelectedHistoryPeriod
contains an optional raw TIME/u32 timestamp followed by FLOAT values in the original
point-list order. Timestamp-disabled replies omit the word entirely. IEEE NaN,
infinity and signed-zero bits and exact raw payloads are preserved. There is no
selector, starting-index or point-list echo: matching the point count does not
prove which device configuration generated the values.

The default decoder validates echoes, flag/count agreement, period count no larger
than requested, exact row widths/length and no trailing data. Known current-index
bounds are checked. Empty/short positive replies are exposed without filling values
or guessing why history is unavailable. Valid device errors preserve the channel;
malformed or interrupted exchanges retire it. No automatic replay occurs.

## Bounded ranges and observed inventory changes

Client::read_selected_history_range accepts SelectedHistoryRange. It splits whole
periods by profile budgets and the 60-element limit and preserves the entire point
selection on every page. A period that cannot fit fails locally before admission.
The window covers at most one selected ring; unknown geometry stays within the
remaining unwrapped UINT16 address space. Zero total produces one probe.

The whole read holds one exclusive workflow/generation and overall call deadline,
with ordinary per-exchange timeouts. HistoryContinuity::RequireUnchangedIndex stops
on observed cursor movement, retaining the page that exposed it. ObserveIndexChanges
explicitly continues and flags movement. Equal cursors do not establish a snapshot
or gap-free continuity, identity or absence of a full unseen ring rollover.

with_inventory_check(true) additionally reads command 0 before and after paging.
The initial inventory must contain every selected point. Changed final inventory
returns HistoryChanged and retains all confirmed samples and both inventories;
valid observations preserve the channel. Comparison preserves inventory order, so
a reordered list is conservatively reported as changed. This opt-in check is an
observation, not an atomic controller snapshot or persistent schema cache. A failed
postcheck leaves complete=false even if all periods were read. Short pages stop
without an extra postcheck. No inventory is queried unless explicitly requested.

SelectedHistoryProgress retains decoded pages, remaining periods, next_index,
generation, index_changed, short_page, complete and optional inventory_before/after
plus inventory_changed. Known rings wrap next_index; unknown geometry yields None
after slot 65535. SelectedHistoryReadError carries boxed confirmed progress on
malformed replies, device errors, transport failures, deadline or awaited stop.
Caller abort cannot return progress but retires incomplete work. A caller can select
an explicit new window; failed pages/ranges are never replayed, resumed or combined
across replacement generations automatically. Allocation is bounded by the explicit
window, wire-sized point list and selected budgets.

## Executable evidence

- tests/history_information.rs: stable REQ/POL IDs, independent request/reply
  literals and TCP; both commands, empty/max inventory, all selectors and flags,
  exact budgets, ordering/duplicates, raw bits, every truncation, malformed headers,
  probes, ring boundaries, inventory changes, short replies, cursor movement,
  partial progress, deadline/cancellation/stop, late replies and public extensions.
- tests/history_information_simulator.rs: explicit raw words, every selector/flag,
  point-255 values, sparse/duplicate order, missing points, retention boundaries,
  index-only requests and valid device-error behavior.
- tests/integration.rs and the inventory standalone test: both typed decoders retire
  malformed sessions before reuse.
- scripts/package-consumer.rs: inventory, explicit request Operation, external codec,
  both timestamp shapes and bounded ranges with inventory rechecks through the
  extracted public API.

Run `cargo run --locked --example history_information` for a hardware-free example.
Software fixtures do not establish physical interoperability or full ROC Plus coverage.
