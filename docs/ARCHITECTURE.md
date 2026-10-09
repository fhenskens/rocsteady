# Architecture

```mermaid
flowchart LR
    App[Application] --> Client[Stable Client]
    Client --> Pool[Etherbird: one resource, bounded FIFO]
    Pool --> Setup[Reconnect, login, readiness]
    Pool --> Session[Serialized Session]
    Session --> Codec[Frame and typed operation codecs]
    Session --> Stream[AsyncRead + AsyncWrite]
    Stream --> TCP[TCP connector]
    Stream --> Serial[Optional serial connector]
    Stream --> Memory[In-memory test channel]
    TCP --> Simulator[Loopback simulator]
    TCP --> Device[Physical controller]
```

Protocol encoding and model dispatch contain no socket or supervision logic.
The Connector boundary supplies fresh channels. Session owns one stream generation
and response correlation; Client owns an Etherbird pool with one exclusive resource.

One outstanding operation per stream is intentional: ordinary ROC Plus frames
provide no request identifier for pipelining. Source/destination and opcode are
validated before accepting a response. An interrupted exchange retires the stream;
a valid opcode-255 rejection preserves it.

A cancellation guard owns the stream slot until the exchange completes. Dropping
an incomplete future clears that slot and signals loss. Etherbird's watcher observes
the signal even with periodic heartbeats disabled. This prevents stale reply reuse.

Etherbird's operation failure policy retains valid device/local errors and returns
them unchanged to the application. This distinction prevents a healthy protocol
rejection from triggering unnecessary reconnect/login. Decoder/transport failures
request recovery.

Etherbird's BoundedFifoQueue bounds waiting work; execute_with_timeout and
connected_with_timeout enforce the call deadline with shutdown precedence and
no replay. The call deadline includes admission and lifecycle readiness. The wire deadline
starts after the stream mutex is acquired, so contention with a heartbeat is not
misclassified as peer silence. Configured creation/setup deadlines bound each
recovery attempt. Explicit rejected/unsupported authentication latches in
Etherbird, publishes Failed, fails waiting calls promptly and suspends further attempts.
Client preserves its public authentication error shape. Await stop and create a new
client to change credentials; transport setup failures remain recoverable.

Awaited stop releases framework resource caches and pool entries. Session teardown
still explicitly closes the stream and destroys generation state. External handles,
user captures and abandoned timed-out hooks can retain resources after stop;
object destruction alone does not guarantee secret erasure.

The simulator's pure Model dispatch runs behind the same framing/stream boundary.
Unit tests include published CRC examples and literal typed payloads. An independent
peer integration fixture neither encodes nor decodes through rocsteady, limiting
shared-client/simulator false positives.

The event receiver is stable across generations and bounded. SRBX signals are
processed during active exchanges/heartbeats. No automatic alarm acknowledgement
is emitted because it requires knowledge of the alarm log entries actually consumed.

The first EXT-1/4/5 slice now uses the [managed extension contracts](MANAGED_EXTENSIONS.md).
A Session ownership mutex spans entire setup/workflow scopes and is shared by
ordinary requests and heartbeats; the inner stream guard still retires incomplete
exchanges. Cancelled scopes between exchanges signal retirement before releasing
ownership. Setup state is installed once per generation and accessed by borrowing
the scoped context. Registration is immutable; setup is repeated on replacement.
