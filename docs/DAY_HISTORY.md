# Day-oriented history (OP-137/138)

Source: Emerson D301180X012, October 2022, §2.17–2.18/Tables 2-20–2-21,
printed pp.2-34–35 (PDF pages 44–45), reviewed 2026-10-15.
[Baseline manual](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Physical-controller and actual serial evidence: none. DEC-4.05/06 remain open.

## Defaults and explicit interpretation

HistoryDay stores only day/month. Defensive calendar validation accepts February 29
and rejects impossible month/day pairs; it never selects a year or timezone.
Month/day may identify different archives after a rollover; successful decoding
cannot establish an archival year. Consumers retain responsibility for any date label.

The named public defaults DayIndicesCodec and DayHistoryCodec follow the tables.
They implement the same ReplyCodec interfaces as caller replacements. Client,
Session and ManagedContext offer read_day_indices, read_history_day and their
_with_codec variants. Per-call codecs can return application-specific types, such
as a reply plus an explicitly supplied archival-year label. Profiles can override
ProtocolProfile::decode_day_indices and decode_history_day, including the codecs
used inside the indexed workflow. They must retain request/sample shape validation
and the daily-index validity invariant under the exchange retirement guard.
Decode failure retires the stream; framing, correlation and ownership remain mandatory.

Table 2-20 defines a three-byte segment/day/month request. Its prose refers to a
point that has no table field. The default follows the table and does not invent
an extra byte. Request-layout replacements use public Operation/DecodedOperation
or Workflow; a reply-only codec cannot change request bytes. No alternative point
layout or automatic year-selection preset is supplied without adequate evidence.
See DEC-4.06 and the external package consumer's explicit Operation example.

Default segments are 0–10. Request with_segment_limit constructors permit explicit
caller assertions for expanded applicability, without claiming firmware support.
For known point/ring geometry, indexed reads use the public HistoryIndexPolicy
seam and ConfiguredHistoryPolicy. TableHistoryPolicy leaves periodic/daily capacities
and point counts unknown. Selecting a replacement is explicit; no parser-success
selection, alternate-request probing or automatic fallback occurs.

## OP-137 index lookup

DayIndicesRequest contains segment and HistoryDay. The fixed nine-byte response
contains a segment echo and four little-endian UINT16 fields: periodic start/count,
daily start/count. There is no point, day or month echo in this reply. DayIndices
preserves exact bytes and raw_daily_start. daily_start is None when daily_count is
zero, even if the raw daily index is nonzero or 65535. A positive daily count may
have more than one entry and makes index zero valid. Counts and starts are not
restricted by a guessed firmware capacity; indexed planning checks selected geometry.
Missing days remain ordinary opcode-255 device errors with their raw code/offset.

## OP-138 whole-day reads

DayHistoryRequest contains segment, target and day/month. DayHistoryTarget::Values
accepts point numbers 0–254; Timestamps explicitly encodes the reserved point 255.
The four-byte reply echo is followed by periodic/daily UINT16 counts, then all
periodic words and all daily words. DayHistoryReply separates both columns using
HistorySamples::Values or Timestamps. FLOAT NaN/infinity/signed-zero bits and raw
TIME/u32 words are retained; no local timestamp conversion occurs.

The default decoder checks every echo, exact count/length agreement and no trailing
bytes. Zero counts and multiple daily entries are valid shapes. The eight-byte
header plus four-byte words allows at most 61 words in the 255-byte wire payload
field (252 used bytes). This is a codec/field budget, not a verified controller
limit; unlike OP-135/136, Table 2-21 supplies no 60-element limit. DEC-1 device
message limits remain open. Profile budgets can tighten the wire budget.

The operation preflights its four-byte request and eight-byte minimum response;
the counts are unknown until the response. An actual reply exceeding the profile
budget retires the stream. OP-138 cannot paginate a day: a controller may reject an
oversized day. The simulator returns a healthy device error for missing/oversized
data and never silently truncates it. Device errors never trigger an automatic
OP-137/135 attempt; applications explicitly select the alternative below.

## Explicit indexed alternative for large days

Client::read_history_day_indexed accepts IndexedDayHistoryRequest with a caller-selected
HistoryIndexPolicy. It reads OP-137, plans both periodic and daily OP-135 ranges,
reads bounded pages, then rereads OP-137. The entire workflow retains one exclusive
generation and call deadline. It does not use OP-138 and can be selected before an
oversized OP-138 call. It reuses the existing single-point planner's profile budgets,
60-item limit and known-ring splitting. Unknown capacities permit only unwrapped
u16 windows; rollover is not guessed. Zero-count columns send no OP-135 request,
and an invalid daily index is never used. Both nonempty columns must plan successfully
before any samples are read. Allocation is bounded by the UINT16 counts and pages.

Values use selectors 1/2 and the requested point. Timestamp fallback uses selectors
3/4 and requires with_timestamp_point with an independently known OP-135 point.
OP-138's special point 255 does not establish that OP-135 accepts that sentinel.
Missing timestamp-point selection fails locally before any day request.

DayHistoryProgress retains initial/final mappings, generation, confirmed periodic
and daily pages, remaining counts and next indices, index_changed, mapping_changed,
short_page and complete. DayHistoryReadError retains boxed progress. Transport,
malformed reply, timeout, device rejection or awaited stop preserves only confirmed
work. Caller abort cannot return progress but retires incomplete exchanges. Nothing
is replayed, resumed or combined with a replacement generation automatically.

HistoryContinuity::RequireUnchangedIndex stops on observed cursor movement within
each column; ObserveIndexChanges explicitly continues and flags it. Cursors are
not compared across periodic/daily columns. A short page stops with complete=false
and no further request. An altered final day mapping always returns HistoryChanged,
including all already-confirmed samples and indices_after; valid changes preserve
the healthy connection. A failed final lookup leaves complete=false even when all
counts were consumed. Unused raw daily-index changes with zero daily count are
ignored by mapping comparison. No complete result proves a snapshot: equal mappings
and cursors can conceal overwrites, full wraps, restarts or a changed archival year.
Separate values/timestamps calls likewise do not establish atomic alignment.

## Examples and evidence

Run `cargo run --locked --example day_history` for default reads, an external
caller-year codec and explicit value/timestamp indexed plans. The example year is
a caller assertion; the library does not infer it. The extracted-package consumer
checks those public seams plus an explicit DecodedOperation using Table 2-20 bytes.

tests/day_history.rs links stable REQ/POL IDs to independent literals and TCP:
all request fields, calendar bounds, every truncation/echo/count failure, index
validity, raw FLOAT/TIME bits, exact budgets, missing days, zero/multiple daily
entries, malformed/late replies, coherent paging, mapping/cursor changes, confirmed
progress and deadline/cancellation/stop. tests/day_history_simulator.rs supplies
explicit words and exercises an oversized day through the opt-in indexed path.
The standalone session regression covers both services. Software evidence does
not close firmware, date interpretation, routing, physical or serial conformance.
