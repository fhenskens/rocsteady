# Typed unicast clock setting

Source: Emerson D301180X012 October 2022 §2.4/Table 2-4, printed p.2-11
(PDF page 21), text and rendered table audited 2026-10-16. OP-8 was introduced
in 1.00. The 0.2.0 API sends exactly seven bytes: second, minute, hour,
day, month and little-endian UINT16 year. There is no weekday byte. The controller
computes weekday and resets microseconds; the simulator computes weekday but has
no microsecond clock and does not advance time.

`ClockSet::new(year, month, day, hour, minute, second)` validates the same Gregorian
calendar policy as existing clock reads: years 1–65535, normal month/day bounds,
leap-year/century rules, hours 0–23 and minutes/seconds 0–59. This policy rejects
leap seconds and year zero; wire capacity does not establish device year acceptance.
No timezone conversion or host-clock synchronization is implicit.

`Client::set_clock` returns `WriteOutcome::Acknowledged` for an empty reply, or
`ClockSetError { outcome, error }`. It reuses the schema-write synchronized send
gate: preflight/queued failures are `NotAttempted`; valid OP-255 is `DeviceError`
with all raw codes/offsets; malformed/lost acknowledgement, active timeout and
interrupted I/O are conservatively `Uncertain`. An error never proves rollback.
No set is replayed on recovery. Caller cancellation closes the send gate but cannot
return an outcome. Awaited stop and admission follow the existing write framework.

`ClockSet` also implements `Operation`, and Session/ManagedContext expose
`set_clock`; those lower-level calls return ordinary errors, so callers must retain
remote uncertainty themselves. `client.operation(clock)` likewise does not provide
the higher-level outcome wrapper. Read-back requires a separate explicit call;
it is an observation, not a guarantee of durable storage or an atomic set/read.
Only acknowledged unicast is implemented; broadcast remains a separate CORE-1 task.

```rust,no_run
# async fn example(client: &rocsteady::Client) -> Result<(), Box<dyn std::error::Error>> {
use rocsteady::{ClockSet, WriteOutcome};
let value = ClockSet::new(2026, 10, 16, 20, 0, 0)?;
match client.set_clock(value).await {
    Ok(WriteOutcome::Acknowledged) => println!("{:?}", client.read_clock().await?),
    Ok(_) => unreachable!("successful clock set is acknowledged"),
    Err(failure) => eprintln!("{:?}: {}; reconcile before another explicit set", failure.outcome, failure.error),
}
# Ok(()) }
```

`tests/clock_set.rs` links `REQ-008-*` source requirements and `POL-008-*` policies
to independent TCP literals, empty/malformed acknowledgements, raw device errors,
standalone retirement, calendar bounds, simulator read-back, lost acknowledgement,
active deadline/cancellation, queued cancellation and profile rejection.
Physical-controller and actual serial evidence remain none.
