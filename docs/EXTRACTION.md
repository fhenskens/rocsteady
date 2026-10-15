# Shared-library boundaries

Reviewed 2026-10-15. This records reusable patterns and their intended ownership;
it is not a commitment to implement additional crates or change polling policy.

## Etherbird adapter helpers

Rocsteady adopts etherbird 0.4.3's `Generation`/`GenerationValue<T>` and
`DisconnectOnDrop<G>`. They are optional-to-use main-library helpers available
with default features and pooling, without additional dependencies.

Each Session creates a private opaque generation token. Rocsteady's public
`GenerationValue<T>` wraps the upstream binding while retaining numeric generation
reporting, its existing error shape and a live connection check. Numeric counters
alone do not establish resource identity. Tokens do not prove controller identity,
persistence or remote-handle validity after an application copies a value.

A managed setup/workflow passes its acquired ownership guard to
`DisconnectOnDrop`. Interruption publishes connection loss before ownership is
released, even between exchanges. Individual exchanges still use `RetireOnDrop`
through response decoding. Rocsteady decides which outcomes permit safe release;
etherbird does not infer acknowledgement, partial effects or replay permission.

Implementation: [Session](../src/session.rs), [managed context and handles](../src/extensions.rs),
[client workflows](../src/client.rs) and [explicit alarm acknowledgement](../src/srbx.rs).
Composition evidence: [extension tests](../tests/extensions.rs), especially
`generation_bound_handles_reject_other_clients_without_io_or_recovery`,
`stale_handle_and_missing_state_fail_locally_without_reconnecting` and
`competing_clone_cannot_enter_between_steps_and_cancellation_retires`.

## Further candidates

| Pattern | Existing evidence | Possible destination and boundary |
| --- | --- | --- |
| Cancellation gate synchronized with progress | [Write execution](../src/writes.rs), [queued cancellation and lost-ack tests](../tests/writes.rs) | Potential etherbird adapter helper if another adapter needs the same permission-to-start boundary. Preserve synchronized gate closure and conservative uncertainty. Acknowledgement/device-error meanings stay in the adapter. This is distinct from resource retirement. |
| Ordered batches with multiple budgets | [Read planner](../src/schema.rs), [write planner](../src/writes.rs), [read tests](../tests/profiles_schema.rs), [write budget tests](../tests/arbitrary_writes.rs) | A protocol utility crate after a second consumer establishes common requirements. Generalize item cost, request/response budgets, count limits and indivisible items; keep ROC opcodes, TLPs and widths here. Preserve input order and duplicates. |
| Confirmed partial progress on failure | [History paging](../src/history.rs), [multi-point paging](../src/multi_history.rs), [schema reads](../src/schema.rs), [writes](../src/writes.rs) and their requirement-linked tests | Potential companion utility or shared design convention. Results need confirmed work, failed/uncertain work and unattempted remainder. A generic container must not imply atomicity, durable cursors, safe retry or automatic continuation. Keep service-specific progress semantics. |
| Deterministic transport fault injection | [Independent managed peer](../examples/support/managed_fixture.rs), [discovery peer](../examples/support/discovery_fixture.rs), [write tests](../tests/arbitrary_writes.rs), [monitoring tests](../tests/monitoring.rs) | A development/test utility crate. Candidate controls include disconnect after effect, lost/delayed replies and changed discovery after reconnect. Protocol literals and expected effects must remain independent of production encoders. Shared fault scheduling must not turn the fixture into a second shared codec. |

Do not extract these solely to remove similar-looking code. First identify a
second independent consumer and write down the shared contract, cancellation and
ownership guarantees, error/progress semantics and dependency direction. Test the
abstraction through both adapters before moving a public API. No new library is
selected or implemented for these candidates.

ROC schemas/catalogs, firmware and sensor applicability, history geometry/codecs,
controller checkpoints, alarm acknowledgement and caller-owned polling stay in
rocsteady or its application. They depend on device or application semantics.
