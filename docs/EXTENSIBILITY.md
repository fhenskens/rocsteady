# Extensible client work plan

Status: managed EXT-1/4/5 slice plus explicit EXT-2 profiles/codecs and EXT-3
schema-backed reads implemented. Remaining device presets, write planning and
authentication acceptance tasks stay open. Updated 2026-10-15.
This is the early delivery milestone in [the roadmap](OUTSTANDING_WORK.md).
The published etherbird crate remains responsible for lifecycle supervision,
admission, exclusive leases, recovery and awaited shutdown.

## Product objective and scope

Deliver a useful ROC Plus client with verified built-ins and supported extension
points for device-specific or unresolved behavior. Full built-in specification
coverage remains a long-term objective. Unresolved public specification details
do not block the extension platform or unrelated well-defined services.

Users may provide their own knowledge without forking the crate. Installing an
extension is explicit; rocsteady does not certify its wire assumptions or device
compatibility. Missing required behavior returns an actionable capability error.
A raw request alone does not constitute a usable typed extension framework.

## EXT-1: authentication and readiness strategies

- [x] Define an async strategy boundary for login, protocol readiness, and restoration on every new connection generation.
- [x] Run strategies inside bounded lifecycle setup before publishing readiness; retain built-in legacy login/clock probing as defaults.
- [ ] Let a strategy handle protected discovery and custom readiness probes, with explicit device/firmware applicability.
- [x] Define generation-local session state, including keys/handles, and prevent it from leaking into a replacement.
- [x] Support application credential providers; keep secrets out of Debug, tracing and fixture output.
- [x] Define rejected/unsupported authentication errors separately from transient transport failures and apply the intended recovery policy.
- [ ] Define explicit logout and changed-credentials behavior without silently undoing caller intent through relogin.
- [x] Expose only managed protocol exchanges to setup strategies; no unrestricted shared socket.
- [x] Provide a compile-checked external strategy example demonstrating setup and restoration using a deterministic fixture procedure.
- [x] Clearly label the example procedure as a fixture, not an invented enhanced-security algorithm.

Acceptance: an external consumer implements authentication without editing the
crate; replacement reruns setup, unsuccessful/cancelled setup is never ready,
and state/secrets stay within the specified lifetime.

## EXT-2: protocol profiles and disputed reply codecs

- [x] Implement predefined selectable alternatives through the same public
  extension contracts available to consumers, without privileged codec paths.
- [x] Permit callers to replace a predefined alternative with their own implementation;
  document applicability, assumptions and independent evidence for each preset.
- [x] Define coherent profiles with identity, applicable family/firmware, capabilities and assumption/validation metadata.
- [x] Keep supported defaults simple; require explicit selection of provisional/custom profiles.
- [x] Allow selected typed request/reply codecs to receive raw payloads and original request context.
- [x] Provide a narrow source-correlation policy seam for ordinary unicast; keep mandatory destination/opcode/framing/size validation and stream-retirement ownership in the framework. Routed framing remains unsupported.
- [ ] Audit whether any supported alternative actually requires a framing seam; do not infer that reply-payload extensibility permits arbitrary transport framing.
- [x] Keep config-layout status, legacy layout flag and family-specific clock formats distinct.
- [x] Define profile selection/override when discovery is absent, protected or inconclusive.
- [x] Reject unsupported capabilities clearly; return unknown raw data where a type can honestly retain it.
- [x] Preserve raw OP-105 occurrence-time/unknown fields when meaning is uncertain; named provisional codecs and caller replacements use the same public interfaces.
- [ ] Invalidate profile-derived caches when verified replacement identity/capabilities change.
- [x] Test wrong echoes/opcodes/addresses, malformed typed replies and profiles with identical-length alternatives.
- [x] Do not autodetect a format solely because it parses; never trial alternative writes/deletes/logins.

Evidence: [profiles_schema tests](../tests/profiles_schema.rs),
[profile/schema consumer example](../examples/schema_profiles.rs), and
[contracts](MANAGED_EXTENSIONS.md). DefaultProfile, ClockCodec and ConfigurationCodec
are named presets for existing verified codec forms. OP-105 now also provides
RawOccurrenceCodec by default and named provisional CalendarOccurrenceCodec and
PrefixedEpochOccurrenceCodec through MinMaxCodec<C>; see [scope/evidence](HISTORY_METADATA.md).
OP-135 adds TableHistoryPolicy defaults, explicit provisional ProseMinuteHistoryPolicy,
configured geometry and arbitrary consumer HistoryIndexPolicy/ReplyCodec replacements
with public per-call selection; see [single-point contracts](HISTORY.md). The subsequent discovery slice adds Roc800Profile with explicit I/O
layouts, contextual opcode-50/100 presets and generation-bound Roc800Catalog snapshots;
see [CATALOG.md](CATALOG.md). Automatic profile caches remain absent. Device-specific
interoperability, occurrence-time interpretation and routed framing still need
their own evidence; these remain open despite the replaceable provisional codecs.

Acceptance: an external consumer supplies a disputed decoder/profile explicitly.
The framework continues to reject unsafe/malformed exchanges, retire interrupted
streams and preserve valid device-rejection semantics.

## EXT-3: schema providers

- [x] Define lookup by point type/parameter and applicable device/firmware, including location rules.
- [x] Support width/type, access and optional constraints without guessing missing metadata.
- [x] Define deterministic precedence for caller overrides and built-in catalogs; preserve caller-supplied raw parameter access.
- [ ] Integrate provider lookup with schema-aware write planning and explicit
  partial/uncertain outcomes (M4); bounded read planning is complete below.
- [x] Integrate provider lookup with bounded read batch planning and confirmed failure progress; write planning awaits M4 codecs/outcome semantics.
- [x] Separate local validation from device rejection and uncertain remote outcomes.
- [x] Add an external-provider example for a vendor/user-defined point type, with mixed widths and bounds.
- [x] Keep comprehensive catalog manifests in SCHEMA_WORK_PLAN.md; a provider seam does not close those rows.

Evidence: [schema implementation](../src/schema.rs),
[independent mixed-width/budget/progress tests](../tests/profiles_schema.rs), and
[external provider example](../examples/schema_profiles.rs). SchemaContext is explicit
caller identity, not inferred discovery. Access and logical bounds are enforced for
reads; optional numeric bounds are metadata reserved for future writes. Missing schema,
unsupported identity, conditional access and invalid widths fail before admission.

Acceptance: a user adds a point type and correct typed access without changing
rocsteady, and oversized/unsupported requests fail before channel use.

## EXT-4: typed custom operations and exclusive workflows

- [x] Expose single-exchange typed operations with local request validation and raw reply decoding in the managed operation boundary.
- [x] Expose a scoped workflow context that retains exclusive channel ownership across multiple exchanges.
- [x] Define the workflow's overall deadline, per-exchange wire deadlines and readiness admission behavior.
- [x] Specify how heartbeat/unsolicited dispatch interacts with workflow ownership; prevent responses from being stolen by competing readers.
- [x] Define error classification for local validation, healthy device rejection, malformed typed data and transport failure.
- [x] Tie remote handles and partial progress to connection generation; fail an interrupted workflow explicitly.
- [x] Do not transparently resume/replay a workflow across replacement; a caller may start an explicit new attempt when its semantics permit it.
- [x] Keep stop/cancellation managed and awaited; a dropped incomplete exchange retires the stream even for extensions.
- [x] Prevent extensions from bypassing queue bounds, receiving an untracked shared stream or retaining an exclusive context after its scope ends.
- [x] Provide a compile-checked custom operation and multi-exchange fixture workflow in an external consumer crate/example.
- [x] Test competing clones, late replies, decoder failure and stop with in-flight work.
- [x] Test disconnect after a simulated side effect with an independent committed-effect counter: committed_side_effect_with_lost_ack_is_never_replayed in tests/profiles_schema.rs models a fixture-only effect and lost acknowledgement over TCP, then checks recovery and explicit caller attempts.

Acceptance: typed operations and workflows can be implemented outside the crate
while preserving managed admission, ownership and cancellation guarantees. Remote
completion remains an operation-specific contract; no implicit side-effect replay.

## EXT-5: contracts, examples and extension lifecycle

- [x] Write API/design contracts before fixing public trait signatures: ownership, async lifetimes, Send/Sync, error types, supported dependency versions and versioning policy.
- [x] Keep extension registration/configuration immutable or define atomic changes and generation invalidation explicitly.
- [x] Distinguish built-in verified, built-in provisional, user-supplied and unavailable capabilities in docs/diagnostics.
- [x] Publish guides for all four extension seams, including what knowledge/evidence an implementer must supply.
- [x] Test external implementation ergonomics with a consumer using the published etherbird dependency; examples must not depend on crate-private access.
- [x] Maintain framework contract tests across dependency upgrades; existing extension tests and new profile/schema tests run against published etherbird 0.4.2. The user confirmed hosted stable Windows/Linux/macOS CI passed for 0.1.0; see RELEASE.md.
- [x] Establish a contribution path: source/evidence, independent fixtures, applicable model/firmware, review, simulator/integration tests, then promotion to built-in support.
- [x] Keep extension compatibility claims separate from the crate's built-in conformance matrix.

Acceptance: readers have runnable examples and documented missing-capability
errors. They can add their own implementations without needing us to guess the
unresolved wire behavior.

## Handling ambiguity during delivery

The delivery policy is **extensible ambiguous behavior with predefined extensions**.
Every DEC subquestion must have an identified customization boundary or an explicit
framework/scope blocker. Where alternatives are sufficiently specified, provide
named built-in implementations of that same public boundary as selectable options.
Users can replace them when their device needs another interpretation; they should
not need to fork the crate or use private APIs. Unknown procedures remain available
for caller implementation without inventing a built-in answer.

Use EXT-1 for authentication/readiness, EXT-2 for profiles and contextual codecs,
EXT-3 for schema metadata and EXT-4 for operations/workflows. Built-in strategies,
profiles, providers and workflows must satisfy the same contracts as caller ones.
This policy applies to all ambiguous behavior, not only reply decoding.

A preset is verified or explicitly provisional, with source references, applicable
device/firmware, assumptions and independent positive/negative fixtures. Offer
multiple named presets when each alternative is sufficiently defined to implement
and test. Use sensible supported defaults where evidence permits. Raw lossless decoding
can be the default when disputed semantic interpretations remain explicit options;
ordinary callers need not build a custom profile. Interpretation selection is
explicit; an interpreted default requires applicable evidence. Never
select by successful parsing alone or trial alternative side-effecting requests.
No option is required where the missing information prevents an honest codec.

Customization cannot disable mandatory frame validation, bounded admission,
exclusive ownership, generation lifetimes, cancellation retirement or no implicit
replay. Address/size policies may vary only within defined validated contracts.
Partial commits stay uncertain when evidence is missing. Undocumented supplemental
services still need an explicit scope decision. These framework requirements are
not device options; record the blocker before exposing a customization boundary.

For each DEC subquestion record a delivery disposition alongside its research:
verified built-in; explicit provisional codec; supported user extension; or deferred
because the framework boundary is insufficient. The disposition is not a claim
that the underlying ambiguity is resolved.

Examples: enhanced credential wrapping can remain user-supplied through EXT-1;
ambiguous typed payloads through EXT-2/4; vendor schemas through EXT-3. Partial
write success is not “resolved” by a profile: preserve unknown outcomes until the
operation has applicable evidence. Shared-reader/routing behavior still requires a
framework contract; unrestricted plugin code is not a substitute for ownership.

## First implementation slice

Delivered: the managed exchange boundary supports custom setup, typed operations
and exclusive workflows. Contracts live in [MANAGED_EXTENSIONS.md](MANAGED_EXTENSIONS.md);
examples and regressions are in [managed_extensions](../examples/managed_extensions.rs)
and [tests/extensions.rs](../tests/extensions.rs). Later profile/schema and discovery
fixtures exercise those seams through public APIs, including preset replacement.

Verified contracts include exclusive ownership, bounded admission/deadlines,
healthy rejection versus malformed-response retirement, cancellation/stop,
generation-local state, immutable registration and no implicit replay. The
[changelog](../CHANGELOG.md) records delivery; [release evidence](RELEASE.md)
records local and user-confirmed hosted checks separately.

EXT-4/5 are delivered. Remaining EXT-1/2/3 requirements stay in their checklists
above: protected discovery/authentication applicability, logout/credential changes,
framing/preset evidence, occurrence-time semantics, identity/cache policy and
schema-aware write planning. A delivered extension seam does not establish a
complete device procedure or built-in conformance.

## Delivery order

Continue with the [recommended next slice](OUTSTANDING_WORK.md#recommended-next-slice)
and the open EXT checklist items, using current [managed contracts](MANAGED_EXTENSIONS.md).
Promote independently evidenced device behavior into replaceable named presets;
keep unsupported and provisional behavior explicit. Speculative enhanced login,
file/tunnel formats and a universal firmware dictionary are not prerequisites
for unrelated, sufficiently specified services.
