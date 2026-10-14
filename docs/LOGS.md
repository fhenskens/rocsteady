# Alarm and event retrieval

Baseline: Emerson D301180X012, October 2022, sections 2.13–2.14,
[Tables 2-16/17 and record diagrams](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf#page=34),
printed pp.2-24–31 (PDF pages 34–41). Reviewed 2026-10-14; diagrams for field
boundaries, operator IDs, old/new overlap and calibration values were visually
inspected. Both services are introduced in firmware 1.00. Firmware/controller
applicability is caller-established; no physical controller or serial exchange
has been validated. Vendor prose is not redistributed.

## Requests and page boundaries

`LogRequest::new(start, count)` produces a three-byte payload: count followed by
little-endian UInt16 start. Counts are 0–10 and indices 0–449. A single page
cannot cross slot 449; the range reader splits there and explicitly requests
slot zero next. This avoids assuming that a controller wraps one request.
Count zero is an index-only request.

Replies contain count, starting index and current index in five bytes, followed
by exactly count fixed records. The codec checks the start echo, current-index
range, returned count no greater than requested, and exact record boundaries.
Short pages are permitted; range retrieval stops on a short page. Alarm records
are 23 bytes (maximum payload 235); event records are 22 bytes (maximum 225).
Selected profile limits can tighten these budgets. Impossible budgets fail locally.

`Client`, standalone `Session`, and `ManagedContext` expose `read_alarms` and
`read_events`. `ProtocolProfile::decode_alarms/decode_events` receive the typed
request and raw payload; `AlarmCodec` and `EventCodec` are the default codecs.
Decode stays inside the exchange guard. Malformed headers/tagged data retire the
connection; valid device rejections preserve it. Unsupported services fail clearly.

## Records and raw preservation

Every record retains its entire fixed byte array through `raw()`, including spare
and unused bytes. Timestamps stay UInt32 TIME values; no timezone conversion or
clock ordering is inferred. `LogText<N>` retains fixed bytes. `printable_ascii()`
returns an exact string, including spaces, only when every byte is printable ASCII;
it does not guess NUL padding or lossy encoding. Non-printable descriptions remain
available and do not make the record malformed.

| Alarm kind | Specific fields | Source |
| --- | --- | --- |
| 0 none | Raw unused body retained | p.2-25 |
| 1 parameter | Byte reason, three-byte TLP, 10-byte text, Float value | p.2-26 |
| 2 FST | Byte FST number, 13-byte text, Float value | pp.2-26–27 |
| 3 user text | 18-byte text | p.2-27 |
| 4 user value | 14-byte text, Float value | p.2-27 |
| Other | Kind and all 18 body bytes | Raw extension |

Alarm byte zero retains SRBX bit 7, set/clear bit 6 and the six-bit kind.
`AlarmReason` provides labels for source codes 0–33 and retains unknown codes.
Reason meanings can depend on the TLP; some reasons use only its type/location.
The stored alarm value is always Float, regardless of the parameter's schema type.
The library does not substitute a schema-derived width or infer universal reason
semantics from a label.

The event enumeration has eight kinds including none, despite the introductory
prose referring to five formats. All enumerated kinds are accounted for:

| Event kind | Specific fields | Source |
| --- | --- | --- |
| 0 none | Raw unused body retained | p.2-28 |
| 1 parameter change | Three-byte operator ID, TLP, event-specific tag, new/optional old value | pp.2-28–29 |
| 2 system | Byte reason, 16-byte text | p.2-29 |
| 3 FST | Byte FST number, Float value, 10-byte text, raw spare bytes | p.2-29 |
| 4 user | Three-byte operator ID, byte reason, 13-byte text | pp.2-29–30 |
| 5 power lost | Additional UInt32 time, raw unused bytes | p.2-30 |
| 6 clock set | Additional UInt32 time, raw unused bytes | p.2-30 |
| 7 calibration verification | Operator ID, TLP, two Float values, raw spare bytes | pp.2-30–31 |
| Other | Kind and all 17 body bytes | Raw extension |

`SystemEventReason` preserves codes and supplies source labels for system/user
records, including explicitly reserved 151–153. Unknown codes remain readable.
Operator IDs stay three raw bytes; no string/padding interpretation is imposed.

Event parameter-change tags have their own numbering:

| Tags | Types |
| --- | --- |
| 0–8 | Binary, Int8, Int16, Int32, UInt8, UInt16, UInt32, Float, TLP |
| 9–15 | ASCII widths 3, 7, 10, 12, 20, 30, 40 |
| 16–17 | Double, TIME |
| Other | Unknown tag and ten raw value-storage bytes |

New storage begins at record byte 12 and has ten available bytes. Old storage
starts at byte 16. Widths up to four have both decoded values; a larger new value
overlaps the old storage and only the new value is returned. Widths 7, 8 and 10
fit the new storage. ASCII widths 12–40 cannot fit a full value in a 22-byte record:
`LoggedValue::Truncated` preserves the ten available bytes and declared type,
without inventing a complete string or old value. This fixed-record interpretation
is explicit; an applicable device-specific alternative requires a custom codec or
Operation. Known complete ASCII values use the standard printable-ASCII validation;
invalid new/old values retire the exchange. Unknown tags stay raw.

## Bounded windows and checkpoints

`read_alarm_range(start, count)` and `read_event_range` retrieve a caller-chosen
slot window of at most 450 records, splitting by profile budgets and ring end.
One workflow holds exclusive ownership of a single connection generation and one
overall call deadline. Pages retain each observed current index. They are not an
atomic log snapshot, and the reported index is not automatically treated as the
last written or next free slot. No timestamp ordering is assumed.

Successful `LogReadProgress` contains complete pages. `LogReadError.progress`
contains only previously decoded pages, including on deadline/transport failure.
The failed page is never reported as consumed. An abandoned future cannot return
its in-flight progress; use the application's previously persisted checkpoint.
Framework cancellation on expiry is asynchronous; observe replacement/readiness
before a new explicit attempt. There is no automatic read replay or recovery paging.

Only after application processing succeeds, call `page.checkpoint(consumed)` or
`progress.checkpoint(consumed)` with the number of returned records consumed.
Zero, out-of-range consumption and an empty NoAlarm/NoEvent anchor are rejected.
The checkpoint stores the last consumed slot and complete raw record. Persist its
index/raw bytes if needed; `LogCheckpoint::new(index, decoded_record)` restores an
application-persisted consumed anchor. It validates shape, not actual consumption.

`resume_alarms(checkpoint, count)` and `resume_events` are explicit attempts against
the same intended controller. The library reads the consumed anchor before and
after the requested next window and compares raw bytes, so even NaN values compare
correctly. Changed/missing anchors return `Error::LogChanged`, preserve the healthy
connection, and require an explicit new window. Decoded pages on such an error
have uncertain continuity. Resume excludes the previously consumed anchor and is
limited to 449 records. `anchor_verified` is true only after both anchor checks pass.

**Continuity limit:** a 450-slot modulo index is not a durable sequence number.
Identical rewrites, a complete rollover, changes that revert between observations,
or replacement with identical contents can escape anchor checks. Matching an
anchor cannot prove gap-free delivery. Controller identity, polling frequency,
durable ingestion and deduplication remain caller policy. No automatic alarm acknowledgement or durable subscription is introduced.
The explicit acknowledgement and idle polling APIs below are separately opt-in.

## Explicit SRBX acknowledgement and idle polling

Additional source: [Table 2-31](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf#page=54)
(printed p.2-44) and [Chapter 6](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf#page=323)
(printed p.6-1), visually audited 2026-10-14. Opcode 225 is introduced at 1.00.
Its request is the last consumed alarm index as two little-endian bytes; the reply
is empty. Chapter 6's index-7 request and CRC are an independent literal fixture.

`Client::acknowledge_alarms(&checkpoint)` performs one exclusive attempt. It
re-reads and compares the complete consumed alarm anchor, then sends exactly that
checkpoint's index. An unavailable/changed anchor returns healthy LogChanged before
sending opcode 225. Invalid budgets/unsupported services fail locally. Equivalent
Session/ManagedContext methods preserve the same verification and exchange guard.
Standalone Session has per-exchange deadlines; Client adds its overall call deadline.
Event checkpoints cannot be passed to the typed acknowledgement API.

The caller confirms successful processing of **all outstanding alarms through the
checkpoint**, including when restoring a persisted checkpoint. The library validates
shape and an observed anchor; it cannot prove application ingestion completeness or
controller identity. A single-record match is not a proof that earlier alarms were
consumed. These remain explicit caller contracts, together with the continuity limits
above. The raw request/Operation escape hatch remains caller-owned.

The controller clears SRBX only if the supplied index equals its current alarm
index when it handles the acknowledgement. A new alarm after retrieval/verification
can keep SRBX pending; the host must not jump to its unread index. Even a valid empty
reply does not disclose whether SRBX cleared. `AlarmAcknowledgement.index` means
confirmed receipt of the supplied index, not a confirmed cleared state.

`AlarmAcknowledgementError` contains the index, underlying error and outcome:

| Outcome | Meaning |
| --- | --- |
| NotAttempted | No acknowledgement send was permitted in this attempt |
| Rejected | A valid device-error reply to opcode 225 was observed |
| Unconfirmed | Send was permitted, but no valid acceptance/rejection was observed; may have applied |
| Confirmed | Valid empty reply was already observed, even if the enclosing call subsequently lost a deadline/stop race |

A shared send gate closes on caller cancellation, deadline or stop completion before
reporting NotAttempted. A late-canceled framework worker cannot subsequently begin
opcode 225. Interrupted/malformed replies retire the stream. Accepted effects with
lost replies remain unconfirmed, and neither recovery nor polling retries an
acknowledgement. Reconciliation and any new explicit attempt belong to the caller.

`Client::srbx_poller(interval)` offers an explicit bounded polling policy instead
of a competing permanent stream reader. It subscribes to the stable notification
channel and spawns no background task. The caller drives `poll()` continuously:
first poll immediately, then interval after each completion. An index-only opcode
118 request reads queued SRBX through the existing serialized exchange path and
also reports current index changes when a signal is missed.

For a continuously driven healthy poller, the reception cycle is interval plus the
call deadline; queueing/setup/recovery are included in that deadline. Application
processing/scheduling between calls adds delay. This is a polling observation bound,
not guaranteed delivery or an atomic snapshot. The interval must be positive and
representable. Canceling a timer does not touch the stream; canceling active I/O
retains normal poisoning. Client shutdown wakes the timer through etherbird's
existing stopped receiver and drains managed work. No additional lifecycle latch or
reader task is introduced. Dropping the poller ends polling.

`SrbxPoll` reports a current alarm index and `SrbxNotifications`: at most 32 raw
validated frames plus observed broadcast lag. Draining is bounded even during a
producer storm. Poll errors retain notifications already received. Malformed SRBX
is never published; it retires the connection. Repeated signals are advisory; a quiet
poll or unchanged modulo index cannot prove absence of new alarms. Receiver lag,
reconnects, subscription creation and complete ring rollover can lose observations.
Use explicit alarm refresh/consumption policy, not signals alone. No automatic
acknowledgement, newest-index inference, deduplication or durable delivery is implied.

## Evidence and simulator scope

`examples/support/log_literals.rs` independently transcribes all known/unknown
record forms. `tests/logs.rs` covers those literals, all event tags, both overlapping
and incomplete values, raw flags/spares, malformed widths/headers/text values,
independent TCP, budgets, short pages, wrap splitting, index-only requests, anchor
changes/growth, partial progress, explicit recovery, cancellation/deadlines, and
profile/managed/standalone interfaces. Its wire peer uses no production frame/CRC
or record encoders. `tests/logs_simulator.rs` independently exercises simulator
record slots, empty markers, index-only reads, validation and bounded overwrite.

The simulator stores explicit raw slots. Missing slots return empty record markers;
it does not manufacture alarms from parameter values. `append_alarm/append_event`
use a fixture-only last-written cursor convention (initial 449, first append at 0)
and overwrite modulo 450. That convention is not the client's cursor policy or
controller interoperability evidence. Every caller-selected window remains explicit.

Run `cargo run --locked --example log_retrieval` for a complete independent TCP
example with consumption and resume. The hosted workflow includes it; the extracted-package
consumer also exercises log APIs/checkpoint validation, idle polling and explicit acknowledgement. Software, hosted platform
and physical-channel evidence remain separate.

SRBX evidence: `tests/srbx.rs` uses the independent wire peer and Chapter 6 vector
for consumption, current-index races, lost/malformed/rejected replies, pre/post-send
deadlines, cancellation/stop, true idle frames, repeated/missed/lagged notifications,
wire-deadline preservation under sustained SRBX and profile/standalone/managed APIs.
`tests/srbx_simulator.rs` covers idle emission, equality-based pending clearing,
lost acknowledgement effects surviving reconnect, rollover and preserving a partial
request read while emitting an unsolicited signal.

Simulator pending state is one configured SRBX host/port fixture, shared across TCP
replacement. `Model.initial_srbx_pending` and `srbx_host` configure it;
`Simulator.raise_srbx(index)` updates the runtime alarm index/pending flag and emits
an idle signal. Populate raw slots explicitly before binding. `srbx_status()` exposes
fixture state, accepted acknowledgement count and sent-signal count. Empty replies
are returned for older valid indices too; pending clears only on equality. The
LoseAlarmAcknowledgement fault waits for opcode 225 and drops its reply after the
state effect. The standalone Model dispatcher supplies codec responses; Simulator
and serve_stream own runtime pending-state effects. This is not a full device/port
security model or physical interoperability evidence.
