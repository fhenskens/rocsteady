# Using the experimental monitoring client

This guide describes published 0.1.0. All releases below 1.0 are experimental.
Built-ins include configuration/clock/parameter reads, legacy login, explicit
profiles, discovery, schema-backed catalog reads, alarm/event retrieval and opt-in
SRBX polling/acknowledgement. Acknowledgement explicitly changes controller status.

Configure addresses, credentials, firmware applicability and parameter types from
the actual controller. Physical device/serial compatibility remains unverified.
Enhanced authentication, typed parameter writes, history, file and routing services
are unavailable. The extension fixture is not an enhanced-security implementation.
See [coverage](COVERAGE.md) and [log/SRBX contracts](LOGS.md) for exact limitations.

## Credentials and TCP

Legacy credentials use three printable ASCII operator bytes and a u16 numeric
password. Obtain secrets from your application provider; do not log environment
contents, request payloads or credential objects. Login Debug redacts the password.

```rust,no_run
use rocsteady::{Client, ClientConfig, Login};

# async fn connect() -> Result<(), Box<dyn std::error::Error>> {
let operator = std::env::var("ROC_OPERATOR")?;
let password: u16 = std::env::var("ROC_PASSWORD")?.parse()?;
let login = Login::new(operator.as_bytes().try_into()?, password, None)?;
let config = ClientConfig { login: Some(login), ..Default::default() };
let client = Client::tcp(std::env::var("ROC_ENDPOINT")?, config)?;
let result = client.read_clock().await;
client.stop().await;
println!("clock: {:?}", result?);
# Ok(())
# }
```

Run `cargo run --example read_device` with ROC_ENDPOINT and optionally both
ROC_OPERATOR and ROC_PASSWORD. It accesses hardware only after explicit endpoint
configuration. Defaults use host unit/group 1/0 and device 1/2: change them in
ClientConfig for your controller. Discovery is not an authentication selector.

For asynchronous providers, implement SetupStrategy and await the provider inside
setup, then delegate to `LegacySetup { login: Some(login) }.setup(context)`. This
runs on every new generation under setup_timeout. The runnable
`cargo run --features simulator --example authenticated_setup` demonstrates
provider refresh, restoration, state notifications and SRBX with fixture credentials.
Provider I/O failures can retry; return Authentication(Rejected/Unsupported) when
trying again without changing setup cannot succeed.

## Errors and recovery

Client methods return `etherbird::Error<rocsteady::Error>`. Use a direct etherbird
0.4.1 dependency if naming that type in your application; the pool feature matches
rocsteady. Matching ErrorKind avoids tying application policy to each new variant.

```rust
use rocsteady::{Error, ErrorKind};

fn next_action(error: &etherbird::Error<Error>) -> &'static str {
    match error {
        etherbird::Error::Operation(error) => match error.kind() {
            ErrorKind::LocalValidation => "correct the request or required capability",
            ErrorKind::DeviceRejection => "inspect device rejection; channel remains healthy",
            ErrorKind::MalformedReply => "check the codec and device applicability",
            ErrorKind::Transport => "remote outcome may be unknown; wait for replacement",
            ErrorKind::Authentication => "stop and recreate with corrected setup",
            _ => "inspect the newly introduced error category",
        },
        etherbird::Error::Timeout => "overall deadline elapsed; remote outcome may be unknown",
        etherbird::Error::Queue(_) => "reduce concurrent work or apply application backpressure",
        etherbird::Error::Stopped => "this client cannot restart",
        etherbird::Error::Lifecycle(_) => "inspect the terminal lifecycle cause",
    }
}
assert_eq!(next_action(&etherbird::Error::Operation(Error::InvalidRequest("fixture"))),
    "correct the request or required capability");
```

An operation is attempted once. Callers may start a new read after recovery; never
blindly repeat side-effecting raw/custom operations. A call timeout may occur while
queued or after send, so the error alone does not establish remote completion.
`response_timeout` is per exchange; `call_timeout` includes admission, setup,
ownership and the whole workflow. A cancelled scope retires its generation.

Setup Authentication errors latch across clones. `authentication_failure()` reports
the reason, and pending/subsequent calls fail promptly. etherbird publishes Failed
and suspends further connector/setup attempts while bounded teardown completes.
Client maps the typed lifecycle authentication cause to the existing
`Error::Operation(Error::Authentication(_))` result for compatibility.
Registration is immutable: await `stop()` and construct a new Client to change
credentials or strategy. Stop takes precedence over the latched failure. Stop
closes the local channel; no unspecified device logout command is invented. A raw
logout on a live client can conflict with automatic restoration: use stop for the
supported shutdown policy. Explicit device logout protocols remain unsupported.

## Typed operations and workflows

An Operation owns original request context. Encode returns local validation errors;
mandatory opcode and size checks occur before admission. Decode receives the raw
reply and can return only DecodeError, which always retires a malformed exchange.

```rust
use rocsteady::{DecodeError, Error, Operation};

struct ClockYear;
impl Operation for ClockYear {
    type Output = u16;
    fn encode(&self) -> Result<(u8, Vec<u8>), Error> { Ok((7, vec![])) }
    fn decode(&self, bytes: &[u8]) -> Result<u16, DecodeError> {
        rocsteady::Clock::decode(bytes)
            .map(|clock| clock.year)
            .map_err(|_| DecodeError("invalid clock payload"))
    }
}
```

`Client::operation(ClockYear)` performs one typed exchange. Use Workflow for
multiple exchanges under one exclusive context; other clones and heartbeats
cannot read inside it. Do not call the same Client recursively from a workflow.

```rust,no_run
use rocsteady::{Error, ManagedContext, Workflow};

struct TwoClockReads;
#[etherbird::async_trait]
impl Workflow for TwoClockReads {
    type Output = (Vec<u8>, Vec<u8>);
    async fn run(self, context: &mut ManagedContext<'_>) -> Result<Self::Output, Error> {
        let first = context.exchange(7, vec![]).await?;
        let second = context.exchange(7, vec![]).await?;
        Ok((first, second))
    }
}
```

Raw exchanges deliberately leave typed interpretation to callers. Prefer Operation
when malformed decoding must retire the stream. Setup returns exclusively owned
`SessionState` (`Box<dyn Any + Send + Sync>`); state is borrowed from context and
never copied to a replacement. `require_state::<T>()` reports missing capability.
Bind reusable remote handles/progress markers with `context.bind(value)` and use
`handle.get(context)`; a replacement returns StaleGeneration before wire use.
Do not extract/copy handles or keys for unchecked reuse. Partial remote progress
does not imply a workflow may resume; recovery always requires an explicit attempt.

Run `cargo run --example managed_extensions` for custom setup, a typed operation
and a two-exchange fixture workflow. See [managed contracts](MANAGED_EXTENSIONS.md)
for lifetimes, retirement, dependency versions and contribution requirements.

## Discovery and the initial catalog

Use an independently established firmware version and explicitly selected I/O layout.
This example assumes ROC800 firmware 3.00 with the 160-entry compatibility layout;
choose settings applicable to your controller. Discovery may require custom authenticated
setup. Opcode 100 classifies point types without revealing widths. Opcode 50 reports
per-type logical ordinals alongside physical locations; I/O TLPs use the physical location.

```rust,no_run
use rocsteady::{Client, FirmwareVersion, IoLayout, Roc800Catalog, Value};

async fn read_first_analog_input(client: &Client)
    -> Result<Vec<Value>, Box<dyn std::error::Error>>
{
    let io = client.discover_io_with_layout(
        FirmwareVersion::new(3, 0)?, IoLayout::Compatibility160,
    ).await?;
    let input = io.by_logical(103, 0)?;
    let catalog = Roc800Catalog::from_discovery(io)?;
    Ok(client.read_schema(&catalog, &catalog.context(), &[
        input.tlp(3)?, input.tlp(21)?, input.tlp(32)?,
    ]).await?)
}
```

The catalog supplies the parameter type and checks its discovered physical location.
It covers 202 parameters, including every PT-101/102/103/104/105 read width; [the manifest](CATALOG.md) records source references,
applicability and omissions. Snapshot-derived catalogs/plans fail with StaleGeneration
after replacement; rediscover before starting a new attempt. The owning application
must await client.stop(). Run `cargo run --example discovery_catalog` for a complete
independent TCP fixture example without controller access.

`Roc800Catalog::entry(point_type, parameter).metadata` supplies optional enum labels,
alarm-bit masks, fixed units and source write caveats. `enum_meaning(raw)` returns
None for an unknown code; `active_bits(raw)` and `uninterpreted_bits(raw)` retain the
entire alarm byte between them. Keep the raw Value alongside any interpretation.
These annotations do not select firmware or establish write permission. Separate
fields and batches are not an atomic device snapshot.

## Alarm and event logs

`read_alarms(LogRequest)` and `read_events(LogRequest)` return typed pages; count
zero returns just the log indices. Range reads split a caller-chosen window by
record budgets and ring end. They hold one generation/deadline and return confirmed
pages in `LogReadError.progress` on failure. Raw records, unknown types and
incomplete large event values remain available.

```rust,no_run
use rocsteady::{AlarmRecord, Client, LogCheckpoint, LogReadError, LogReadProgress};

// The application created/persisted this anchor only after consuming its record.
async fn next_alarm_window(client: &Client, consumed: LogCheckpoint<AlarmRecord>)
    -> Result<LogReadProgress<AlarmRecord>, LogReadError<AlarmRecord>>
{
    client.resume_alarms(consumed, 10).await
}
```

Create checkpoints with `page.checkpoint(consumed)` or
`progress.checkpoint(consumed)` after successful application processing. Explicit
resume compares the complete consumed record before and after paging. A changed
anchor returns healthy `Error::LogChanged`; choose a new window deliberately.
Modulo indices and matching bytes cannot prove absence of rollover/replacement
or duplicate records. Controller identity and durable ingestion remain caller policy.
No implicit replay or alarm acknowledgement occurs. See [log contracts](LOGS.md)
and run `cargo run --locked --example log_retrieval` without hardware.

## SRBX polling and consumed-alarm acknowledgement

Use `Client::srbx_poller(interval)` for opt-in reception while otherwise idle.
Drive `poll()` repeatedly; it makes an index-only alarm request, routes queued SRBX,
and reports the current index plus bounded notifications and any broadcast lag.
It spawns no background task and never acknowledges or replays an operation.
The first poll is immediate; later polls wait interval after completion. Each exchange
has normal call/admission/wire deadlines. Stop wakes a waiting poller.

```rust,no_run
use rocsteady::{AlarmRecord, Client, LogCheckpoint};
use std::time::Duration;

async fn acknowledge_consumed(client: &Client, consumed: &LogCheckpoint<AlarmRecord>)
    -> Result<(), Box<dyn std::error::Error>>
{
    // Caller has already processed ALL outstanding alarms through this checkpoint.
    let mut poller = client.srbx_poller(Duration::from_secs(1))?;
    let poll = poller.poll().await?;
    println!("reported index: {}, notification lag: {}",
        poll.current_alarm_index, poll.notifications.lagged);
    let receipt = client.acknowledge_alarms(consumed).await?;
    println!("accepted consumed index: {}", receipt.index);
    Ok(())
}
```

Acknowledgement verifies the consumed record in one exclusive attempt, then sends
that checkpoint's index in opcode 225. It never substitutes an unseen current index.
The caller must have processed all outstanding alarms through that checkpoint and
established controller identity; matching bytes cannot prove either condition.
The empty reply confirms receipt, not that SRBX cleared: a newer alarm can keep it
pending. Failure reports NotAttempted, Rejected, Unconfirmed or an already-observed
Confirmed receipt even when the overall call loses a deadline/stop race. Unconfirmed
may have applied; reconnect never resends. Reconcile deliberately before another
explicit attempt. See [full log/SRBX contracts](LOGS.md).

## Serial and notifications

Enable `serial` and construct SerialConnector with the actual electrical channel,
baud/parity/stop bits and flow control. The example requires ROC_SERIAL_PORT and
ROC_SERIAL_BAUD and only reads a clock; hardware authentication can be configured
as in the TCP example. Run `cargo run --features serial --example serial_read`.
Serial is compile/fixture tested, with physical interoperability still unverified.

`subscribe_state()` yields a watch receiver of change notifications, not a sequence
of guaranteed transitions. Await changed(), then inspect `client.state()`.
`subscribe_events()` yields bounded SRBX frames found during exchanges/heartbeats.
Handle broadcast lag explicitly; there is no continuous idle reader or automatic
alarm-log acknowledgement. Await stop() before leaving your runtime.

## History metadata and occurrence choices

Ordinary calls use built-in codecs. Min/max occurrence times stay lossless raw
five-byte values by default; calendar and prefixed-epoch interpretations are
explicitly provisional options. Both use the same public codec interface as
consumer implementations. See [contracts and evidence](HISTORY_METADATA.md).

```rust,no_run
use rocsteady::{Client, MinMaxRequest, MinMaxCodec, CalendarOccurrenceCodec,
    HistoryTagsRequest};

# async fn history(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
let request = MinMaxRequest::new(0, 7)?;
let values = client.read_min_max(request).await?; // No custom profile required.
println!("current: {}, occurrence: {:?}", values.current, values.today.minimum_time);
// Select only with applicable device knowledge; parsing does not establish applicability.
let calendar = client.read_min_max_with_codec(request,
    MinMaxCodec(CalendarOccurrenceCodec)).await?;
let tags = client.read_history_tags(HistoryTagsRequest::new(0, vec![7, 2])?).await?;
println!("calendar: {:?}, reported index: {}", calendar.today.minimum_time, tags.periodic_index);
# Ok(())
# }
```

Read several metadata replies inside a workflow when they must retain one
connection generation. Index changes and controller updates can still occur;
there is no automatic retry, snapshot, year reconstruction or durable continuity.

## Archived single-point reads

OP-135 reads values or raw timestamp words through default codecs, without a
custom profile. Default minute indices are 0–59; known capacities, the provisional
0–60 interpretation, and consumer policies are explicit choices. See
[history contracts and partial-progress semantics](HISTORY.md).

```rust,no_run
use rocsteady::{Client, HistoryKind, HistoryRequest, HistoryRange};

# async fn samples(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
let page = client.read_history(
    HistoryRequest::new(0, 7, HistoryKind::Periodic, 0, 10)?
).await?;
println!("reported index: {}, samples: {:?}", page.current_index, page.samples);
// The known minute ring splits this window into 58–59, then 0–2.
let result = client.read_history_range(
    HistoryRange::new(0, 7, HistoryKind::Minute, 58, 5)?
).await;
match result {
    Ok(progress) => println!("complete: {}, remaining: {}", progress.complete, progress.remaining),
    Err(error) => println!("confirmed pages: {}, next: {:?}, error: {}",
        error.progress.pages.len(), error.progress.next_index, error.error),
}
# Ok(())
# }
```
