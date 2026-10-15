# rocsteady

**An async ROC Plus client with automatic connection recovery.**

rocsteady is an async Rust client for Emerson ROC Plus devices. Discover points and read configuration,
clocks, parameters and alarm/event logs through the same client while failed connections are replaced.
Configured login and protocol readiness are restored before application calls resume.

[etherbird](https://crates.io/crates/etherbird) supplies lifecycle supervision,
bounded admission and exclusive channel access. rocsteady supplies the ROC Plus
framing, typed reads and session rules. TCP, optional serial connections and
application-provided byte streams use the same protocol implementation.

## Useful features

- **A shared managed client:** clones share one managed channel; subsequent calls wait
  for replacement setup after a transport failure.
- **Typed reads:** discover configuration, read the clock, and read individual or
  contiguous parameters using types supplied from your device's schema.
- **Discovery and catalog:** explicit I/O layouts and 202 source-audited ROC800
  parameters, with firmware/location checks and optional enum/bit/unit metadata.
- **Archived samples:** bounded single-point minute/periodic/daily reads, separate timestamp
  selectors, explicit progress and replaceable index policies.
- **History metadata:** min/max values with raw occurrence times, selectable provisional codecs,
  and ordered history tags with a reported periodic index.
- **Alarm/event monitoring:** typed pages, bounded ring windows and caller-consumed
  checkpoints, plus opt-in idle SRBX polling and explicit alarm acknowledgement.
- **Restored setup:** legacy login and a clock readiness probe run on every new
  connection. Custom setup strategies can obtain application credentials and
  establish generation-local state.
- **Bounded work:** a bounded FIFO rejects excess requests; call, exchange and
  lifecycle deadlines limit waiting and recovery.
- **Exclusive workflows:** custom typed operations and multi-exchange workflows
  retain managed ownership, framing validation and cancellation retirement.
- **Explicit outcomes:** each operation runs once. Valid device rejections preserve
  the channel; incomplete exchanges retire it so late replies cannot reach another call.
- **State and shutdown:** optional heartbeats detect idle failure, state notifications
  report changes, and awaited shutdown cancels/drains work and closes the channel.

Use the current stable Rust toolchain. This codebase represents version `0.1.2`.
All releases below 1.0 are experimental
and may have incomplete protocol coverage; see the scope below.

```toml
[dependencies]
rocsteady = "0.1.1"
# From a checkout: rocsteady = { path = "../rocsteady" }
# Add features = ["serial"] for SerialConnector.
tokio = { version = "1", features = ["macros", "rt-multi-thread", "net", "time"] }
```

Below 1.0, breaking API changes use a new minor version; compatible additions and
fixes use patch releases. See the [usage guide](docs/USAGE.md) for credentials,
errors and serial configuration, and the [managed extension contracts](docs/MANAGED_EXTENSIONS.md)
for ownership, cancellation and generation lifetimes.

## Scope and validation

Version 0.1.2 supports configuration discovery, clock/parameter reads, legacy
login and device errors, typed alarm/event retrieval with bounded pages and
explicit checkpoints, plus opt-in idle SRBX polling and explicit consumed-alarm
acknowledgement. Acknowledgement changes controller SRBX status; it is never
automatic. Explicit profiles, contextual codecs, discovery and bounded schema reads
are covered by the [extension contracts](docs/MANAGED_EXTENSIONS.md).
OP-105/108 history metadata and OP-135 single-point history reads include selectable
policies/codecs and bounded paging; see the [history guide](docs/HISTORY.md).
The [catalog](docs/CATALOG.md) contains 202 ROC800 parameters, including every discrete/analog input and output read width and every pulse-input read field, plus clock, port and module fields.
Remaining catalog tables, enhanced login and typed writes, multi-point/other history retrieval, automatic durable alarm delivery, files
and routing remain unavailable. Raw/custom
operations can perform side effects; they are not automatically replayed.

Local macOS/stable validation passes 168 all-feature tests, 16 doctests, strict
Clippy, fixture examples, package verification and an extracted-package consumer.
Hosted stable Windows/Linux/macOS CI passed for the published 0.1.2
release commit `a939ac1`, independently verified through GitHub CLI. See the release record for the evidence source.
Physical ROC hardware and actual serial exchanges have not been validated. The simulator
is a deterministic fault-injection fixture, not a full ROC800 emulator.

Read the [coverage matrix](https://github.com/fhenskens/rocsteady/blob/main/docs/COVERAGE.md)
and [release evidence](docs/RELEASE.md) for exact support and validation limits.
Configuration discovery does not automatically select authentication or establish
firmware applicability; unknown fields are retained rather than guessed.

## Concrete examples

| Example | What it shows |
| --- | --- |
| [Local simulator](examples/simulator.rs) | Configuration, clock and parameter reads over loopback TCP; no hardware required. |
| [Authenticated setup and recovery](examples/authenticated_setup.rs) | An async credential provider, restored login after disconnection, state notifications and interleaved SRBX signals. |
| [Managed extensions](examples/managed_extensions.rs) | Custom setup, a typed operation and a two-exchange workflow against an independent in-memory peer. |
| [Profiles and schemas](examples/schema_profiles.rs) | Explicit profile/applicability and a user-defined mixed-width schema read in bounded batches over an independent TCP peer. |
| [Discovery and built-in catalog](examples/discovery_catalog.rs) | Point classifications and I/O discovery, then system/I/O reads using source-audited metadata and physical locations. |
| [Single-point history](examples/history.rs) | Default sample/timestamp reads, bounded paging, explicit geometry and provisional minute-index choice. |
| [History metadata](examples/history_metadata.rs) | Default raw occurrences, explicit prebuilt alternatives, consumer decoding and ordered history tags. |
| [Alarm/event retrieval](examples/log_retrieval.rs) | Independent record fixtures, bounded log pages, caller-confirmed consumption, checkpoint resume, idle polling and explicit acknowledgement. |
| [Configured TCP device](examples/read_device.rs) | Configuration and clock reads using an explicit endpoint and optional legacy credentials from the environment. |
| [Serial clock read](examples/serial_read.rs) | A serial connector with an explicit port and baud rate; other line settings must match your channel. |

Run the fixture examples from a checkout:

```sh
# Configuration and typed reads; no controller required:
cargo run --locked --features simulator --example simulator

# Fixture credentials and outage recovery; no controller required:
cargo run --locked --features simulator --example authenticated_setup

# Public extension APIs with an independent fixture peer:
cargo run --locked --example managed_extensions

# Explicit profiles and schema-backed batches:
cargo run --locked --example schema_profiles

# Discovery and the built-in catalog:
cargo run --locked --example discovery_catalog

# Alarm/event windows, checkpoint resume and explicit SRBX handling:
cargo run --locked --example log_retrieval

# Min/max metadata, selectable occurrence codecs and history tags:
cargo run --locked --example history_metadata

# Single-point samples, timestamps and bounded ring windows:
cargo run --locked --example history
```

For hardware examples, set `ROC_ENDPOINT` and optionally both `ROC_OPERATOR` and
`ROC_PASSWORD`, or set `ROC_SERIAL_PORT` and `ROC_SERIAL_BAUD`. Then run
`cargo run --example read_device` or `cargo run --features serial --example serial_read`.
The fixture procedures do not implement enhanced-security authentication.

## Usage

These snippets run inside an async function on a Tokio runtime with I/O and time
enabled. Configure addresses, credentials and parameter types for the actual
controller. Defaults do not discover a device or its security configuration.

### 1. Read through a managed client

Construct one client and share clones where needed. Calls wait for protocol
readiness; await `stop()` even when the read fails.

```rust,no_run
use rocsteady::{Client, ClientConfig, DataType, Parameter, Tlp};

# async fn read_device() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::tcp("192.0.2.1:4000", ClientConfig::default())?;
let result = async {
    let configuration = client.read_configuration().await?;
    println!("device type: {}", configuration.device_type());
    println!("clock: {:?}", client.read_clock().await?);

    // Use the TLP and type from your controller's parameter schema.
    let parameter = Parameter::new(Tlp::new(103, 16, 3), DataType::Float);
    let values = client.read_parameters(&[parameter]).await?;
    Ok::<_, Box<dyn std::error::Error>>(values)
}.await;
client.stop().await;
let values = result?;
# let _ = values;
# Ok(())
# }
```

### 2. Configure login and deadlines

Legacy login uses three printable ASCII operator bytes and a numeric u16 password.
Obtain credentials from your application; keep them out of logs and payload dumps.

```rust,no_run
use rocsteady::{Address, Client, ClientConfig, Login};
use std::time::Duration;

# async fn configured_client() -> Result<(), Box<dyn std::error::Error>> {
let operator = std::env::var("ROC_OPERATOR")?;
let password = std::env::var("ROC_PASSWORD")?.parse::<u16>()?;
let login = Login::new(operator.as_bytes().try_into()?, password, None)?;
let config = ClientConfig {
    host: Address::new(1, 0),
    device: Address::new(1, 2),
    login: Some(login),
    response_timeout: Duration::from_secs(3),
    call_timeout: Duration::from_secs(15),
    queue_capacity: 64,
    ..Default::default()
};
let client = Client::tcp(std::env::var("ROC_ENDPOINT")?, config)?;
let result = client.read_clock().await;
client.stop().await;
let clock = result?;
# let _ = clock;
# Ok(())
# }
```

`call_timeout` includes queueing, readiness and the entire call or workflow.
`response_timeout` bounds each exchange after stream acquisition; lifecycle
setup/recovery deadlines are configured through `recovery`.

A valid legacy login rejection latches an authentication error and blocks further
connection/login attempts. Await `stop()` and create a new client with corrected
setup. Transport failures remain recoverable. For an async credential provider,
use `Client::with_setup`; the authenticated fixture demonstrates that pattern.

### 3. Add a typed operation

An operation owns its request context and decodes the raw reply under the stream
retirement guard. `DecodeError` marks malformed data; it cannot fabricate a healthy
device rejection. This example extracts the year using the existing clock decoder.

```rust,no_run
use rocsteady::{Client, DecodeError, Error, Operation};

struct ClockYear;
impl Operation for ClockYear {
    type Output = u16;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> { Ok((7, vec![])) }
    fn decode(&self, payload: &[u8]) -> Result<u16, DecodeError> {
        rocsteady::Clock::decode(payload)
            .map(|clock| clock.year)
            .map_err(|_| DecodeError("invalid clock payload"))
    }
}

# async fn year(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
let year = client.operation(ClockYear).await?;
# let _ = year;
# Ok(())
# }
```

Request encoding and mandatory opcode/size validation happen before admission.
Use `Client::request` only when raw payload interpretation belongs to your caller.

### 4. Keep several exchanges exclusive

A workflow holds one generation across all its exchanges. Competing clones and
heartbeats cannot consume its responses. This builds on the `ClockYear` operation
above; use the same etherbird version as rocsteady for its async-trait re-export.

```rust,no_run
use rocsteady::{Client, Error, ManagedContext, Workflow};
# use rocsteady::{DecodeError, Operation};
# struct ClockYear;
# impl Operation for ClockYear {
#     type Output = u16;
#     fn encode(&self) -> Result<(u8, Vec<u8>), Error> { Ok((7, vec![])) }
#     fn decode(&self, payload: &[u8]) -> Result<u16, DecodeError> {
#         rocsteady::Clock::decode(payload).map(|clock| clock.year)
#             .map_err(|_| DecodeError("invalid clock payload"))
#     }
# }

struct TwoClockReads;
#[etherbird::async_trait]
impl Workflow for TwoClockReads {
    type Output = (u16, u16);
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<Self::Output, Error> {
        let first = context.operation(ClockYear).await?;
        let second = context.operation(ClockYear).await?;
        Ok((first, second))
    }
}

# async fn pair(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
let readings = client.workflow(TwoClockReads).await?;
# let _ = readings;
# Ok(())
# }
```

Setup state belongs to its generation and is destroyed on replacement or shutdown.
Bind reusable remote handles with `context.bind(value)`; `handle.get(context)`
rejects a replacement generation. Interrupted workflows never resume or replay
implicitly. Do not recursively call the same client from inside a workflow.

Calls return `etherbird::Error<rocsteady::Error>`. Local validation and valid device
rejections retain a healthy channel; malformed replies and interrupted exchanges
retire it. A timeout or disconnect does not establish whether a side effect occurred.
See the [error guide](docs/USAGE.md#errors-and-recovery) before deciding to try again.

State watchers can await `subscribe_state().changed()` and inspect `client.state()`.
`subscribe_events()` survives replacements and delivers bounded SRBX signals found
during requests/heartbeats. Handle receiver lag; there is no continuous idle reader
or automatic alarm-log acknowledgement. For idle observation, drive the opt-in
`srbx_poller`; see [log/SRBX contracts](docs/LOGS.md). Outgoing CRC is generated and incoming CRC
is checked by default; ignoring incoming CRC is an explicit compatibility option.

## Development

Start with the [documentation guide](https://github.com/fhenskens/rocsteady/blob/main/docs/README.md)
for architecture, source evidence and work-package status. Keep Cargo.lock tracked
so dependency and lifecycle checks use reproducible versions.

```sh
cargo fmt --check
cargo test --locked
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
cargo clippy --locked --all-features --all-targets -- -D warnings
```

README and usage snippets are compiled as library doctests. Install
[prek](https://prek.j178.dev/) and run `prek install` / `prek run --all-files` for
repository hygiene and Rust checks. The [release checklist](docs/RELEASE.md) records
package/consumer verification and remaining gates; the [dependency audit](docs/DEPENDENCIES.md)
records the locked license inventory. Inspect publication contents with
`cargo package --list --allow-dirty --locked`.

[API documentation](https://docs.rs/rocsteady) ·
[Coverage](https://github.com/fhenskens/rocsteady/blob/main/docs/COVERAGE.md) ·
[Architecture](https://github.com/fhenskens/rocsteady/blob/main/docs/ARCHITECTURE.md) ·
[Hardware and channels](https://github.com/fhenskens/rocsteady/blob/main/docs/HARDWARE.md) ·
[Engineering tasks](https://github.com/fhenskens/rocsteady/blob/main/docs/OUTSTANDING_WORK.md) ·
[Reference specification](https://github.com/fhenskens/rocsteady/blob/main/docs/reference/README.md) ·
[CI](https://github.com/fhenskens/rocsteady/blob/main/.github/workflows/ci.yml)

Licensed under Apache-2.0. Independent project; not affiliated with Emerson.
