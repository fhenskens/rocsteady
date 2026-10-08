# rocsteady

A Rust client for Emerson ROC Plus devices with automatic reconnection and session recovery.

rocsteady keeps a stable async client while its connection is replaced. It restores
the configured login and checks protocol readiness before admitting application calls.
[Etherbird](https://crates.io/crates/etherbird) supplies lifecycle supervision,
exclusive channel access, a bounded FIFO queue, and awaited shutdown.

**Status:** initial read-only implementation; not yet published or validated against
physical ROC hardware. It implements configuration discovery, clock reads, individual and contiguous parameter
reads, legacy login, frame validation, and device errors. See
[protocol coverage](https://github.com/fhenskens/rocsteady/blob/main/docs/COVERAGE.md) for implemented and outstanding services.
The [remaining-work checklist](https://github.com/fhenskens/rocsteady/blob/main/docs/OUTSTANDING_WORK.md) records priorities,
dependencies and completion criteria.
The [reference specification](https://github.com/fhenskens/rocsteady/blob/main/docs/reference/README.md) links to the original
Emerson PDF and identifies the implementation baseline.

## Example

Run the local fixture without hardware:

```sh
cargo run --features simulator --example simulator
```

Use a configured device endpoint:

```rust,no_run
use rocsteady::{Client, ClientConfig, DataType, Parameter, Tlp};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::tcp("192.0.2.1:4000", ClientConfig::default())?;
let result = async {
    client.connected().await?;
    let configuration = client.read_configuration().await?;
    println!("device type: {}", configuration.device_type());
    let clock = client.read_clock().await?;
    println!("{clock:?}");

    // Supply the TLP and type from the actual device's parameter schema.
    let request = Parameter::new(Tlp::new(103, 16, 3), DataType::Float);
    let values = client.read_parameters(&[request]).await?;
    println!("{values:?}");
    Ok::<_, Box<dyn std::error::Error>>(())
}.await;
client.stop().await;
result
# }
```

The Tokio runtime needs I/O and time enabled. Configure host/device unit/group
addresses, login, deadlines and heartbeat interval to match the endpoint. Defaults
do not discover a device or its security configuration.

## Behavior

- Every application operation executes once; a transport failure returns to its caller.
  Subsequent calls wait for replacement setup. No writes are automatically retried.
- One channel is shared across clones and used exclusively per operation. Pending work
  is bounded by `queue_capacity`; a full queue returns `etherbird::Error::Queue`.
- `call_timeout` covers queueing and readiness; `response_timeout` covers the exchange
  after acquiring the stream. Lifecycle deadlines are configured through `recovery`.
- An incomplete, cancelled or timed-out exchange discards its stream so a late reply
  cannot be consumed by another request.
- Configured legacy login is repeated during recovery, followed by a clock-read probe.
  Enhanced-security login is not implemented.
- Optional clock heartbeats detect idle channel failure. They also collect interleaved
  SRBX signals; `subscribe_events()` survives connection replacements. There is no
  continuous reader while all requests and heartbeats are idle. Lag is reported by
  the bounded event receiver; alarm-log retrieval and acknowledgement remain caller work.
- CRC is generated for outgoing messages. Incoming CRC validation is enabled by default;
  `CrcMode::Ignore` is an explicit Ethernet compatibility option.
- Await `stop()` to cancel/drain managed work and close the channel. Client clones
  remain stopped.

## Transports and tests

`transport::Connector` produces a fresh `AsyncRead + AsyncWrite` stream.
TCP and optional serial connectors, the in-memory test seam, and the simulator share
the same protocol/session implementation.

```sh
cargo test --locked
cargo test --locked --all-features --all-targets
cargo test --locked --doc --all-features
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo fmt --check
```

The simulator is a deterministic fixture model with fault injection, available behind
the `simulator` feature. It is not a full ROC800 emulator. The integration suite
uses actual loopback TCP sockets, independent fixed wire fixtures and in-memory streams.

See [hardware and channels](https://github.com/fhenskens/rocsteady/blob/main/docs/HARDWARE.md), [architecture](https://github.com/fhenskens/rocsteady/blob/main/docs/ARCHITECTURE.md),
and [coverage / development sequence](https://github.com/fhenskens/rocsteady/blob/main/docs/COVERAGE.md).

`read_configuration()` supports original and extended opcode 6 replies and reports
point counts, operating mode, device type, communication port and raw security mode.
Missing counts are distinct from zero counts. Reserved bytes, unknown values and
trailing extensions are retained. Firmware version is not included in this reply:
fields introduced by later firmware cannot establish that firmware's presence.
Discovery currently runs after configured login/readiness; it does not automatically
select authentication, enforce device compatibility, or change setup policy.

## Development

Start with the [documentation guide](https://github.com/fhenskens/rocsteady/blob/main/docs/README.md)
for the implementation reading order, current decisions and first extension slice.

Install [prek](https://prek.j178.dev/) and enable the repository hooks:

```sh
prek install
prek run --all-files
```

The hooks check file hygiene, Rust formatting, Clippy and the all-feature test suite.
Keep Cargo.lock tracked so these checks use reproducible dependency versions.

The published crate includes source, tests, examples, README and license files.
Roadmaps, reference research, CI and local tooling remain in the repository.
Check the publication payload with `cargo package --list --allow-dirty --locked`.

Licensed under Apache-2.0. Independent project; not affiliated with Emerson.
