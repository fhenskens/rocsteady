# Managed extension contracts (EXT-1/4/5 first slice)

Implemented against the published Etherbird 0.3.1 registry source, with the `pool`
feature, Rust 1.88 minimum, and Tokio 1. No fork or private dependency APIs.
`Cargo.lock` identifies the tested dependency versions. Public traits are early
0.1 APIs: breaking signature changes require a minor version change while below
1.0; compatible additions may ship in patch releases. Re-run the extension and
lifecycle suites on every dependency upgrade.

## Audit and ownership

Etherbird `Pool::execute` supplies bounded admission and an exclusive lease;
`Lifecycle::setup` runs before resource publication under `Config::setup_timeout`.
Pool operation cancellation drops the operation future, and `Pool::stop` cancels
and drains tracked jobs and awaits supervisor teardown. `execute` does not replay;
rocsteady does not call the retry variants. `Lifecycle::is_expected` affects error
reporting, not a permanent rejection policy.

Session previously locked the stream for one exchange only, so a workflow built
from separate requests could interleave with the heartbeat. Session now has an
ownership mutex acquired for the entire setup/workflow scope. All ordinary Session
requests and heartbeats acquire that same mutex. Its inner stream mutex retains
the existing exchange cancellation/decoding guard. Unsolicited SRBX dispatch occurs
inside this reader; there is no competing reader. Idle reception remains deferred.

`ManagedContext` has private construction, no stream/Session accessor, and cannot
be cloned. Exchanges borrow it mutably. Its lifetime is bounded by the framework's
setup/workflow future. `Workflow::run` uses async-trait's borrowed Send future;
workflow values and outputs are Send + 'static, operations are Send + Sync + 'static,
and shared setup strategies are Send + Sync + 'static. Contexts cannot be returned
as outputs or spawned into detached tasks. Do not recursively call the same Client
from a workflow: it would wait for its own lease until the overall deadline.

## Deadlines, errors and cancellation

`Client::operation` synchronously encodes and validates the opcode/payload before
queueing. Implementers must keep synchronous methods bounded and nonblocking.
Workflow validation happens at each `context.operation` or `context.exchange`,
before wire use; this slice has no separate workflow preflight trait.
`call_timeout` includes queueing, recovery/readiness, ownership acquisition and the
whole operation/workflow. Each exchange uses `response_timeout` from inner stream
acquisition through write, read and decode. Setup additionally has Etherbird's
setup deadline. Dropping queued calls sends cancellation to Etherbird; they do not
run after admission. No remote completion claim follows from a timeout.

| Result | Classification and retirement |
| --- | --- |
| Encode/request validation `InvalidRequest`/`InvalidConfig` | Local; no wire activity; healthy resource retained |
| Valid opcode-255 `Device` | Healthy rejection; original details returned without relogin |
| Decode `InvalidResponse`, bad frame/CRC/correlation | Malformed; exchange guard retires stream before ownership releases |
| I/O, wire timeout, disconnection | Transport failure; stream retired, replacement requested; failed attempt never replayed |
| Dropped active exchange | Guard drops stream and signals disconnection |
| Cancelled workflow between exchanges | Context signals disconnection before releasing ownership; requests reject that generation, watcher and awaited teardown close stream |

Operation decoders return only `DecodeError`, converted to `InvalidResponse` under
the retirement guard. They cannot fabricate a healthy device rejection or local
validation error. Raw exchanges leave typed interpretation to
callers; use `Operation` to decode under the retirement guard. A workflow may handle
healthy rejections deliberately. It cannot convert a retired generation into
successful completion. Errors returned by workflow code outside an exchange still
request Etherbird recovery except for the healthy/local classes above.

## Setup, state and registration

`Client::with_setup` explicitly replaces login/clock readiness. `Client::new` and
`tcp` retain `LegacySetup`. Registration is immutable and shared by Client clones.
There is no live credential/profile replacement API. Application credential providers
can be awaited inside setup; connection replacement calls setup again, with a fresh
context and generation ID. A returned `SessionState` (`Box<dyn Any + Send + Sync>`) is installed once
only after successful setup, then exposed as a borrowed `context.state::<T>()`.
The context locks the state for its scope; disconnect destroys state even if a
stopped supervisor retains the Session object. Awaited stop releases this state.
Replacement never copies this state. Keys/remote handles must be tagged with the
context generation. `context.bind(value)` creates a `GenerationValue` that can
be returned as workflow output; `get(context)` rejects replacement generations
before wire use. `require_state<T>()` reports absent required state. Unchecked
copies of keys/handles must not be reused. The framework cannot revoke copies made by arbitrary extension
code. Destructors and application providers are responsible for secret erasure.

Contexts, strategies and state have no Debug implementation or automatic payload
logging; Login's password remains redacted. Extensions must keep secrets out of
errors, tracing and fixtures. Generation IDs are process-local diagnostic identities.
Failed or cancelled setup is never published ready, even if extension code handles
a transport error. Transport setup failures retain the retry policy. Explicit
`Authentication(Rejected/Unsupported)` latches across Client clones, wakes waiting
calls and prevents further connector/setup invocation. Legacy login maps a valid
opcode-255 rejection to terminal Rejected. The supervisor retains bounded lifecycle
bookkeeping until stop; `state()` need not show a dedicated authentication state.
`authentication_failure()` exposes the latch. Stop takes precedence over the latch.
Changing credentials/strategy requires awaited stop and a new immutable Client.
Stop is local disconnect, not an invented wire logout. Explicit on-wire logout,
enhanced procedures and applicability metadata remain outstanding EXT-1 tasks.

## External implementation and evidence

Run `cargo run --example managed_extensions`. The consumer example implements
custom setup, a typed operation and a two-exchange workflow using only public APIs
and the published Etherbird async-trait re-export. Its independent in-memory peer
checks literal requests and computes reply CRC separately from production codecs.
The setup procedure is a clock fixture, not enhanced authentication.

`tests/extensions.rs` checks setup state/restoration, healthy rejection, malformed
decoding, active and queued call deadlines, competing clones, cancellation between
steps, stop and disconnect after an observed request. Existing `tests/integration.rs`
checks late replies, wire timeouts, rejected setup, frame/correlation failures,
queue bounds, heartbeats, interleaved SRBX and shutdown. No hardware evidence.
No side-effecting operation is retried, and partial remote outcomes remain unknown.
Additional alpha-review cases cover stale handles, required state, pre-admission
validation, terminal unsupported authentication and swallowed decoder errors.
README/usage snippets and the context escape compile-fail case are doctests;
the package is also exercised from an independent extracted-package consumer.

EXT-2 profiles/address policies and EXT-3 schema providers remain planned. No arbitrary
framing seam, unsupported authentication algorithm or disputed codec is introduced.
Contributions should provide source sections, model/firmware applicability,
independent positive/negative fixtures, simulator/integration coverage and review
before promotion to built-in support; user extensions do not close conformance rows.
