# Multi-point history (OP-136)

Source: Emerson D301180X012, October 2022, §2.16/Table 2-19, printed
pp.2-32–33 (PDF pages 42–43), reviewed 2026-10-15.
[Baseline manual](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Physical-controller and actual serial evidence: none.

## Requests and time-major replies

Client, Session and exclusive ManagedContext expose
`read_multi_history(MultiHistoryRequest)` and
`read_multi_history_with_codec(request, codec)`. Client additionally exposes
`read_multi_history_range(MultiHistoryRange)` for bounded paging.

Requests contain segment, little-endian starting index, selector, first point,
point count and period count: seven bytes. Selectors are Minute (0), Periodic (1)
and Daily (2); OP-135's separate timestamp selectors are rejected locally.
Points form one ascending contiguous block, without wrapping past point 255.
The request limit is `(points + 1) * periods <= 60`: timestamps consume elements.
For example, two points allow 20 periods per page. Zero periods requests only the
header/current index. Zero points is accepted as timestamp-only according to the
table's element formula; physical-device acceptance remains unverified.

The reply contains segment and starting-index echoes, the current index, a byte
count of four-byte elements, then time-major rows. Each row contains a raw TIME/u32
timestamp followed by FLOAT/f32 values in ascending requested point order. No
selector, first-point or point-count echo exists. MultiHistoryPage retains the
original request, decoded periods, current index and exact raw payload. Float NaN,
infinity and signed-zero bits are preserved; timestamp words remain integers.
No timezone or atomic-snapshot semantics are inferred.

The default decoder validates both echoes, exact element widths/length, no trailing
bytes, a count no larger than requested and divisibility by `points + 1`. A reply
must contain whole periods. Known current-index bounds are checked. Header/payload
sizes are 6–246 bytes, distinct from OP-135's 5–245 bytes.

Table 2-19 allows zero returned elements for an invalid request. A zero-element
reply is exposed without inventing a device error or treating it as proof that
history is absent. For a positive request it is an incomplete short page; paging
stops. A shorter integral-period reply is handled the same way. Zero-period probes
complete without advancing, but do not prove that a point block is configured.
Malformed replies retire the stream; valid opcode-255 errors preserve it.

## Explicit geometry and extension contracts

Ordinary constructors use TableHistoryPolicy: segments 0–10, minute indices 0–59,
unknown point counts and periodic/daily capacities. Table 2-19 and §2.16 retain the
same minute table/prose discrepancy as OP-135. ProseMinuteHistoryPolicy explicitly
selects provisional 0–60/61-slot bounds; it does not establish firmware behavior.
ConfiguredHistoryPolicy supplies caller-asserted segment, point and ring bounds.
Custom HistoryIndexPolicy implementations use the same interface. No layout or
capacity is selected by parsing success, response index or presumed firmware.
Every selected point is validated against the policy and must share the same
geometry for the common cursor. DEC-4.03/04 remain open.

MultiHistoryCodec implements public ReplyCodec<MultiHistoryRequest>.
ProtocolProfile::decode_multi_history can replace its decoder. Ordinary page/range
methods also validate the returned request, period/point shape and selected current
index under the exchange retirement guard. Per-call codecs can return arbitrary
consumer output; their decode failures still retire the stream. Custom wire
widths/budgets require an explicit Operation/DecodedOperation and the existing
managed framing, correlation and cancellation contracts.

## Bounded windows and confirmed progress

A range counts periods, preserves its entire requested point block and obeys
request/reply profile budgets, the 60-element limit and known ring boundaries.
It never splits a point block silently. If one period cannot fit, planning fails
locally before admission. Zero total produces one index-only exchange.
A window covers at most one known ring; unknown capacities permit only an unwrapped
u16 index window. Unknown retention is never inferred from the current index.

All pages run in one exclusive Workflow/generation, sharing admission, overall
call deadline, per-exchange timeout and heartbeat ownership with other operations.
MultiHistoryReadProgress records decoded pages, next_index, remaining periods,
generation, index_changed, short_page and complete. A short reply returns successfully
with short_page=true and complete=false and sends no more requests. next_index
advances by whole returned periods; known rings wrap, while unknown geometry yields
None after wire index 65535.

HistoryContinuity::RequireUnchangedIndex stops on observed current-index movement
with Error::HistoryChanged, retaining the page that exposed it and preserving the
healthy channel. ObserveIndexChanges continues explicitly and flags movement.
Equal indices cannot prove snapshot consistency, continuity or controller identity:
a full rollover, rewrite or restart can go unseen.

MultiHistoryReadError carries the confirmed prefix after malformed replies,
transport errors, device rejection, deadlines or awaited stop. No failed page or
range is replayed/resumed automatically, and generations are never combined. Caller
cancellation cannot return progress but retires incomplete exchanges. A caller may
start an explicit new window after checking identity and continuity requirements.

## Executable evidence

- tests/multi_history.rs: requirement-linked independent literals and TCP requests,
  all selectors, timestamp/value order and raw bits, count/point/index boundaries,
  every truncation, corrupt echoes/counts, exact budgets, known-ring splitting,
  short/zero replies, index movement, failure progress, explicit recovery,
  deadline/cancellation/stop, delayed replies on retired streams, coherent custom
  geometry and public profile/codec/Session/workflow contracts.
- tests/multi_history_simulator.rs: explicit independent timestamp/value words,
  each selector, contiguous/missing points, short/zero replies and ring boundaries.
- tests/integration.rs: standalone malformed OP-136 replies retire the session.
- scripts/package-consumer.rs: default reads, external codec and bounded range
  against the extracted package's public API.

Run `cargo run --locked --example multi_history` for a controller-free example.
These fixtures are software evidence, not controller captures or firmware validation.
