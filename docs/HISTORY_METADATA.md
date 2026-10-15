# History metadata and explicit occurrence codecs

Baseline: Emerson D301180X012, October 2022, §2.11/Table 2-14 (pp.2-22–23)
and §2.12/Table 2-15 (p.2-24). These APIs read min/max metadata and history
point tags; [OP-135 single-point samples](HISTORY.md) are available separately.
Neither service proves ring continuity.
Implementation and source review recorded 2026-10-15; physical evidence: none.

## Defaults and prebuilt choices

`Client`, standalone `Session` and exclusive `ManagedContext` provide
`read_min_max(MinMaxRequest)` and `read_history_tags(HistoryTagsRequest)`.
Ordinary calls require no custom profile. DefaultProfile and Roc800Profile use
MinMaxCodec(RawOccurrenceCodec) and HistoryTagsCodec respectively.
OP-105 is labelled Capability::Provisional because its occurrence definitions
conflict; OP-108's baseline codec is software-verified, without a hardware claim.

| Choice | Interface | Interpretation and applicability |
| --- | --- | --- |
| RawOccurrenceCodec (default) | ReplyCodec<()> | Retain exactly five bytes; no guessed meaning. Table 2-14 offsets/lengths define the 50-byte reply layout. |
| CalendarOccurrenceCodec | ReplyCodec<()> | Explicitly provisional: second, minute, hour, day, month, following Table 2-14 prose and related-family corroboration. No verified ROC800 firmware applicability. No year/timezone is inferred. |
| PrefixedEpochOccurrenceCodec | ReplyCodec<()> | Explicitly provisional: preserve an opaque first byte, then decode a little-endian UINT32 epoch-seconds candidate. No status/padding meaning or timezone conversion is assigned. No verified ROC800 firmware applicability. |
| Caller codec | ReplyCodec<()> | Any Send output type through MinMaxCodec<C>; or replace the complete reply with ReplyCodec<MinMaxRequest>. |

Selection is per-call with `read_min_max_with_codec(request, MinMaxCodec(codec))`,
or profile-wide by overriding ProtocolProfile::decode_min_max. The same public
interfaces serve built-ins and consumer implementations. Profile::decode_history_tags
can also be replaced. Codec failure runs under the retirement guard, not after
releasing the connection. For other payload sizes/budgets use a custom Operation
or DecodedOperation; this does not change transport framing guarantees.

Calendar validation accepts seconds/minutes 0–59, hours 0–23 and valid month/day
combinations, allowing February 29 because no year is known. Unset/zero or unknown
calendar fields fail under that explicit interpretation; the raw default retains
them. An identical five-byte region can parse under both named interpretations.
Neither successful parsing nor response length selects one automatically.

The epoch candidate follows the pinned [CISA parser](https://github.com/cisagov/icsnpp-roc-plus/blob/802c31814a92355e66ccc05379b77d87886ca381/analyzer/roc_plus_opcode_105.spicy)
(byte plus UINT32), reviewed again on 2026-10-15. That parser and its generated
fixtures are software clues, not controller captures. The calendar candidate and
contradictory manual wording are recorded in [DEC-4 research](reference/AMBIGUITIES.md#dec-4--extrema-times-clock-year-and-history-indices)
and [second-pass evidence](reference/SECOND_PASS.md#opcode-105-an-additional-competing-interpretation--dec-4).
DEC-4.01 remains open. No bare four-byte occurrence preset is supplied: its shifted
reply offsets are not established by the five-byte table layout.

## OP-105 contract

MinMaxRequest::new accepts baseline segment 0–10 and a wire-byte point number.
Table 2-14 supplies no narrower point bound; controller capacity is not inferred.
`with_segment_limit` is an explicit caller applicability assertion for expanded
segments, not automatic firmware discovery or a verified expanded preset.

The request is segment/point (2 bytes). The baseline response is exactly 50 bytes:
segment/point echoes, archival-method byte, TLP, six FLOAT values and four five-byte
occurrence regions at the table's offsets. MinMaxReply retains the entire raw
payload plus current, today/yesterday minima/maxima, occurrence interpretations
and last-completed-period value. Every unknown archival method remains raw;
known labels include hourly average/accumulation/current/totalize and FST forms.
The archival method does not silently reinterpret FLOAT fields as timestamps.
IEEE NaN/infinity/signed-zero bits survive. Min/max ordering is not used to reject
observations, since unset states and device behavior are not inferred.

## OP-108 contract

The default request accepts one segment 0–10 and 1–20 explicit points 0–199.
Order and duplicates survive. Zero-point requests have no established contract
and fail locally. `with_limits` permits independently established caller-supplied
segment/point bounds while retaining the mandatory 1–20 count.

Request payload size is 2+count. The reply is 4+11*count (maximum 224):
segment/count echoes, one common little-endian UINT16 periodic index, and ordered
(point, ten-byte tag) records. Every point echo is validated in request order.
Tags use LogText<10>; exact padding, zero and unknown bytes survive, and
`printable_ascii()` returns None when the bytes are not all printable. No trailing
bytes, short pages or count mismatches are silently accepted.

The periodic index stays a raw u16; no ring capacity, UTC time, sample availability,
rollover count or durable cursor is inferred. It may change between calls,
including 65535→0. Each reply reports its own index. Applications requiring
several requests can use a Workflow to retain one generation; exclusive ownership
does not freeze the controller or make multiple replies an atomic snapshot.
There is no automatic paging, resume or replay after recovery.

## Ownership, limits and evidence

Known request and expected-response budgets are checked against the selected
profile before managed admission and again before exchange. Unavailable capability,
invalid requests and impossible budgets fail without history I/O. Valid device
rejections preserve a usable connection; malformed typed replies, decoder failure,
timeout and active cancellation retire it. A replacement requires an explicit new
application call. The API does not cache device metadata or infer replacement identity.

`tests/history_metadata.rs` uses independent literals/TCP and covers defaults,
all named alternatives, profile overrides and arbitrary consumer output types,
request limits, every fixed field/echo, all truncations, padded tags, changing
indices, tighter budgets, malformed/device-error outcomes, cancellation/deadlines,
standalone sessions, workflows and decoder-failure retirement. The standalone
malformed-response regression in tests/integration.rs now includes both opcodes.

`HistoryMetadataModel` supplies explicit simulator record bodies/tags/indices via
`Simulator::bind_with_history` or Model::respond_with_history, preserving the
existing public Model shape. `tests/history_metadata_simulator.rs` checks valid,
missing and invalid requests. `examples/history_metadata.rs` runs without hardware;
the extracted-package consumer exercises both services and all codec paths.
These fixtures do not establish device/firmware conformance or serial interoperability.
