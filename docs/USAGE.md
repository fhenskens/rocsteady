# Using the experimental read-only client

The supported built-ins are configuration discovery, clock reads, legacy login,
individual/contiguous parameter reads and device errors. Configure addresses,
credentials and parameter types from the actual controller. Device compatibility
and serial wiring have not been physically validated. Profiles, schema catalogs,
enhanced authentication and typed write/history/alarm/file/routing services are
unavailable. The extension fixture is not an enhanced-security implementation.

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

Client methods return `etherbird::Error<rocsteady::Error>`. Use a direct Etherbird
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
the reason, and pending/subsequent calls fail promptly. Etherbird publishes Failed
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
