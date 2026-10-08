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

A valid device error is nested inside a successful Etherbird operation result, then
returned unchanged to the application. This distinction prevents a healthy protocol
rejection from triggering unnecessary reconnect/login. Decoder/transport failures
request recovery.

The call deadline includes admission and lifecycle readiness. The wire deadline
starts after the stream mutex is acquired, so contention with a heartbeat is not
misclassified as peer silence. Configured creation/setup deadlines bound each
recovery attempt. Bad credentials may continue to fail until the call deadline;
there is no permanent authentication-failure state yet.

The simulator's pure Model dispatch runs behind the same framing/stream boundary.
Unit tests include published CRC examples and literal typed payloads. An independent
peer integration fixture neither encodes nor decodes through rocsteady, limiting
shared-client/simulator false positives.

The event receiver is stable across generations and bounded. SRBX signals are
processed during active exchanges/heartbeats. No automatic alarm acknowledgement
is emitted because it requires knowledge of the alarm log entries actually consumed.
