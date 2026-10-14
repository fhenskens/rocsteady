# Managed extension contracts (setup, profiles, schemas and workflows)

Implemented against published etherbird 0.4.2 from crates.io with the `pool`
feature, current stable Rust and Tokio 1. All dependency interfaces are public.
Registry source/commit and package validation are recorded in RELEASE.md.
`Cargo.lock` identifies the tested dependency versions. Public traits are early
0.1 APIs: breaking signature changes require a minor version change while below
1.0; compatible additions may ship in patch releases. Re-run the extension and
lifecycle suites on every dependency upgrade.

Requested upstream improvements and their eventual adoption are tracked in
[the etherbird upgrade work package](OUTSTANDING_WORK.md#adopt-upstream-etherbird-lifecycle-improvements).
etherbird 0.4.1 supplies terminal lifecycle failures, operation retain/recover
classification, checked configuration, post-stop cache release, bounded FIFO and
single-attempt call/readiness timeouts. Rocsteady retains
stricter positive hook deadlines and explicit stream/generation-state cleanup.
Version 0.4.2 adds the optional RetireOnDrop adapter helper. Rocsteady
holds it across the complete exchange and decoder, and explicitly completes
valid replies or healthy device rejections. Between-exchange workflow cancellation,
generation state and protocol validation remain here.

## Audit and ownership

etherbird `BoundedFifoQueue` and `Pool::execute_with_timeout` supply bounded
admission, an exclusive lease and the overall call deadline;
`Lifecycle::setup` runs before resource publication under `Config::setup_timeout`.
Pool timeout/cancellation signals the tracked job to drop its operation future;
destruction occurs when that job next runs. `Pool::stop` cancels
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
acquisition through write, read and decode. Setup additionally has etherbird's
setup deadline. Dropping queued calls sends cancellation to etherbird; they do not
run after admission. No remote completion claim follows from a timeout.
Cancellation pruning reclaims queue space asynchronously. Rocsteady requires
positive queue capacity and call/hook deadlines, although upstream permits zero
capacity (reject all) and zero call deadlines (expire without starting work).

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
request etherbird recovery except for the healthy/local classes above.

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
opcode-255 rejection to terminal Rejected. etherbird publishes `ResourceState::Failed`
and performs bounded teardown. `authentication_failure()` reads the framework cause;
Client maps authentication lifecycle errors to the existing public
Operation(Authentication) result. Stop takes precedence over the latch.
After stop the framework cause may be absent.
Changing credentials/strategy requires awaited stop and a new immutable Client.
Stop is local disconnect, not an invented wire logout. Explicit on-wire logout,
enhanced procedures and applicability metadata remain outstanding EXT-1 tasks.

## External implementation and evidence

Run `cargo run --example managed_extensions`. The consumer example implements
custom setup, a typed operation and a two-exchange workflow using only public APIs
and the published etherbird async-trait re-export. Its independent in-memory peer
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

EXT-2 profiles/contextual codecs and EXT-3 schema-backed reads are implemented below.
No arbitrary framing seam, unsupported authentication algorithm or disputed firmware
codec is introduced.
Contributions should provide source sections, model/firmware applicability,
independent positive/negative fixtures, simulator/integration coverage and review
before promotion to built-in support; user extensions do not close conformance rows.

## Explicit profiles and contextual codecs

Set `ClientConfig.profile` to an `Arc<dyn ProtocolProfile>` before constructing a
client. Clones and replacement generations keep this immutable selection. The default
is `DefaultProfile`, with the existing ordinary-unicast clock/configuration codecs.
`Client::profile()` and `ManagedContext::profile()` expose metadata and capabilities.
ProfileMetadata records name, applicability, assumptions and evidence; these are
implementer assertions, not a certification or automatically discovered device identity.
Protected/inconclusive discovery requires explicit caller selection and, when needed,
`with_setup` using the existing managed authentication boundary.

Capability distinguishes Verified, Provisional, UserSupplied and Unavailable.
DefaultProfile labels tested built-in codec forms Verified; this does not certify
every firmware variant of that opcode. Capability describes outgoing requests;
device-originated SRBX/errors retain their framework support independently.
Other legal raw/custom requests are
UserSupplied, preserving the existing escape hatch. Unavailable services fail locally
before ordinary Client admission. A missing required setup capability is terminal
instead of repeatedly reconnecting. Provisional/custom profiles are never auto-selected.
The built-in setup requires clock capability and optional legacy-login capability.
Custom setup owns its capability requirements; configure heartbeats only when its
profile supports opcode 7. Changed profiles require awaited stop and a new client.

`decode_clock` and `decode_configuration` return DecodeError and run under the
exchange guard. Clock setup and heartbeats use the same selected decoder as reads.
`ClockCodec` and `ConfigurationCodec` are named public ReplyCodec presets; a custom
profile can call or replace them. `DecodedOperation<Q, C>` supplies the original typed
request Q to `ReplyCodec<Q>::decode` alongside raw reply bytes. Use it through
`Client::operation` or `context.operation`; preserve unknown fields/raw bytes in
your output when interpretation is unavailable. No alternative is trial-parsed.

PayloadLimits can tighten request/reply budgets per opcode; limits above 255 never
relax the framework's ordinary payload ceiling. Opcode-180 plans additionally retain
the existing 240-byte response budget. Normal matching replies exceeding a profile
limit retire the stream. Valid device errors and SRBX signals retain their framework
formats. The only address policy seam is `accepts_source`: reply destination must
still match the host, source must be a legal unicast address, opcode/correlation,
CRC policy and exact frame lengths stay enforced. Routing/broadcast framing is not
enabled by source overrides. Neither transport ownership nor cancellation can be
overridden. Profiles should be deterministic and synchronous methods bounded.

No discovery-based profile cache exists; no cache is carried across replacement.
Extensions must bind remote handles to generations and revalidate identity-dependent
state in setup. Source/firmware evidence is required before contributing device presets.
The example's synthetic profile and identical-length test decoder certify framework
behavior only, not another ROC format.

## Schema providers and bounded reads

Implement `SchemaProvider::lookup(context, tlp)` for explicit SchemaContext family/
firmware identity. Return None only for missing entries; return an error for unsupported
applicability. ParameterSchema supplies DataType (and therefore width), Access,
optional logical bounds and optional numeric bounds. Missing metadata is never guessed.
ReadOnly/ReadWrite are readable; WriteOnly and unresolved Conditional fail locally.
Logical bounds are enforced; numeric bounds are write metadata, not a read-value filter.
The built-in Roc800Catalog covers 130 parameters, including clock, port and module fields; schema-aware
writes and complete PT conformance remain unavailable. PT-101/103 have all read
widths plus optional CatalogMetadata enum/bit/unit annotations. Unknown values
remain raw; source write caveats do not alter read planning or establish write access. See [CATALOG.md](CATALOG.md).

`SchemaChain` searches providers in order: place caller overrides first, catalogs
after them. First Some wins; an error stops lookup, so unsupported identity cannot
silently fall back. Raw `read_parameters` with caller-supplied types remains available.
Providers are shared Send + Sync objects; provide an immutable metadata snapshot during
planning. Supply independently established identity rather than deriving firmware from
configuration length. The framework does not discover or refresh that caller assertion.

`ReadPlan::build` resolves and validates the entire ordered TLP list before admission.
It preserves duplicates and packs opcode-180 batches within both request and expected
response limits. A single value that cannot fit is rejected locally. `Client::read_schema`
builds and executes a plan; `read_plan` rechecks all batches against its selected profile
in case the plan was built for another client. One Workflow holds exclusive ownership
of one generation for the entire attempt, including all batch decodes. No batch resumes
after reconnection; competing clones/heartbeats cannot interleave.

On success the result is ordered Vec<Value>. SchemaReadError preserves the original
etherbird error and ReadProgress with fully decoded values and completed_batches.
A failed batch contributes no values; progress is the confirmed snapshot at failure,
including overall call timeout. Malformed replies retire the stream; valid device
rejections remain healthy. Caller cancellation has no returned progress. To resume,
explicitly choose new work after reestablishing identity and considering consistency:
even exclusive ownership does not freeze the controller's values across batches.
Overall call expiry signals asynchronous job cancellation in etherbird; an immediately
submitted call can still observe Disconnected while retirement reaches the supervisor.
Observe replacement/readiness before an explicit subsequent attempt. This is separate
from automatic operation replay, which is never used.

Run `cargo run --locked --example schema_profiles` for public external profile/schema
implementations with mixed widths and bounds over an independent loopback TCP peer.
`tests/profiles_schema.rs` covers identity, precedence, local failures, both budgets,
partial progress, codec replacement, malformed replies, setup/heartbeat selection,
healthy rejection and a committed fixture effect followed by lost acknowledgement.
All wire framing and values in that peer are independent of production codecs.
No physical-device or serial interoperability evidence is supplied by these tests.

## Discovery-backed schemas

ProtocolProfile now also exposes contextual `decode_point_types` and `decode_io_points`,
explicit `io_layout`, `validate_io_layout` and an `io_location` policy. PointTypeCodec
and IoPointCodec are public presets implementing ReplyCodec; read_point_types and
read_io_points on Client, Session and ManagedContext use selected profile decoders.
The public request Operation implementations use the source-based presets directly,
allowing explicit per-operation selection like the existing ClockCodec.

`Roc800Profile` selects a named 160/240/224-entry layout. FirmwareVersion is an
independent caller assertion; configuration revision/length do not establish it.
`IoDiscovery` holds exclusive ownership while reading configuration and both selectors
twice, then comparing final configuration. An observed change returns DiscoveryChanged
and keeps a healthy stream. This does not freeze device state. Profiles retain a narrow
source-based mapping default, with contradictory diagram/compatibility choices recorded
in [the discovery evidence](CATALOG.md#discovery-contracts).

IoSnapshot keeps raw selector arrays, physical positions, per-type logical numbers,
configuration, explicit firmware and generation. `by_logical(...).tlp(...)` maps
types 101-109 to the physical TLP component. A bare logical ordinal is not a wire
location. Absent/reserved positions and unknown point types are not guessed into schemas.

`Roc800Catalog::from_discovery` checks the ROC800 identity and installed I/O locations.
`SchemaProvider::generation` optionally declares a generation requirement. ReadPlan
captures it before admission, then rejects a different generation before wire use.
SchemaChain conservatively combines all declared requirements and rejects conflicts.
Static providers remain unbound by default. Old snapshot/catalog objects can remain in
caller memory, but schema reads through the managed plan cannot reuse them after
replacement. Arbitrary copied TLPs remain caller-owned, as with other copied handles.

Run `cargo run --locked --example discovery_catalog` for both discovery services and
catalog-backed reads through an independent TCP peer. Tests in `tests/discovery.rs`
and `tests/discovery_simulator.rs` distinguish source-derived byte oracles from the
shared-codec simulator. Individual malformed replies retire the stream; valid device
errors, unsupported local metadata and observed discovery changes retain it.

## Bounded alarm/event reads

The shared PreparedOperation path exposes read_alarms/read_events on Client,
Session and ManagedContext, with contextual profile codecs and decoder retirement.
Range reads use one exclusive Workflow, fixed record/profile budgets and one overall
deadline. They return only decoded-page progress on failure and never resume or
replay implicitly. Consumed checkpoints use raw record anchors, including NaNs;
resume checks before and after paging. LogChanged is a healthy continuity error.
Matching bytes do not prove gap-free delivery or controller identity. See
[log contracts, source variants and evidence](LOGS.md).

Explicit acknowledge_alarms accepts only an alarm checkpoint, verifies its raw
anchor and sends the consumed index inside the same exclusive context. Client
failure retains NotAttempted/Rejected/Unconfirmed/Confirmed outcome evidence;
Session/ManagedContext use the same core and an Error-typed failure. A caller-driven
SrbxPoller reuses existing pool stopped watches and serialized opcode-118 reads,
with no extra task/lifecycle latch. It reports bounded notifications and lag without
automatic ingestion/acknowledgement. Caller identity/complete consumption remain
required; an empty acknowledgement does not prove SRBX cleared. See LOGS.md.
