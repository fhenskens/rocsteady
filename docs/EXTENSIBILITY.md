# Extensible client work plan

Status: first managed EXT-1/4/5 slice implemented; EXT-2/3 and remaining acceptance tasks are open.
This is the early delivery milestone in [the roadmap](OUTSTANDING_WORK.md).
The published Etherbird crate remains responsible for lifecycle supervision,
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

- [ ] Implement predefined selectable alternatives through the same public
  extension contracts available to consumers, without privileged codec paths.
- [ ] Permit callers to replace a predefined alternative with their own implementation;
  document applicability, assumptions and independent evidence for each preset.
- [ ] Define coherent profiles with identity, applicable family/firmware, capabilities and assumption/validation metadata.
- [ ] Keep supported defaults simple; require explicit selection of provisional/custom profiles.
- [ ] Allow selected typed request/reply codecs to receive raw payloads and original request context.
- [ ] Provide a narrow address/correlation policy seam for known direct-connect/routed behavior; keep mandatory framing/size validation and stream-retirement ownership in the framework.
- [ ] Audit whether any supported alternative actually requires a framing seam; do not infer that reply-payload extensibility permits arbitrary transport framing.
- [ ] Keep config-layout status, legacy layout flag and family-specific clock formats distinct.
- [ ] Define profile selection/override when discovery is absent, protected or inconclusive.
- [ ] Reject unsupported capabilities clearly; return unknown raw data where a type can honestly retain it.
- [ ] Preserve raw occurrence-time/unknown fields when an extension cannot establish their meaning.
- [ ] Invalidate profile-derived caches when verified replacement identity/capabilities change.
- [ ] Test wrong echoes/opcodes/addresses, malformed typed replies and profiles with identical-length alternatives.
- [ ] Do not autodetect a format solely because it parses; never trial alternative writes/deletes/logins.

Acceptance: an external consumer supplies a disputed decoder/profile explicitly.
The framework continues to reject unsafe/malformed exchanges, retire interrupted
streams and preserve valid device-rejection semantics.

## EXT-3: schema providers

- [ ] Define lookup by point type/parameter and applicable device/firmware, including location rules.
- [ ] Support width/type, access and optional constraints without guessing missing metadata.
- [ ] Define deterministic precedence for caller overrides and built-in catalogs; preserve caller-supplied raw parameter access.
- [ ] Integrate provider lookup with bounded read/write batch planning.
- [ ] Separate local validation from device rejection and uncertain remote outcomes.
- [ ] Add an external-provider example for a vendor/user-defined point type, with mixed widths and bounds.
- [ ] Keep comprehensive catalog manifests in SCHEMA_WORK_PLAN.md; a provider seam does not close those rows.

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
- [ ] Test disconnect after a simulated side effect with an independent committed-effect counter; the current extension fixture observes a request but does not model a write effect.

Acceptance: typed operations and workflows can be implemented outside the crate
while preserving managed admission, ownership and cancellation guarantees. Remote
completion remains an operation-specific contract; no implicit side-effect replay.

## EXT-5: contracts, examples and extension lifecycle

- [x] Write API/design contracts before fixing public trait signatures: ownership, async lifetimes, Send/Sync, error types, supported dependency versions and versioning policy.
- [x] Keep extension registration/configuration immutable or define atomic changes and generation invalidation explicitly.
- [ ] Distinguish built-in verified, built-in provisional, user-supplied and unavailable capabilities in docs/diagnostics.
- [ ] Publish guides for all four extension seams, including what knowledge/evidence an implementer must supply.
- [x] Test external implementation ergonomics with a consumer using the published Etherbird dependency; examples must not depend on crate-private access.
- [ ] Maintain framework contract tests across dependency upgrades.
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
and test. Selection is explicit; a default requires applicable evidence. Never
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

Contracts, dependency audit, evidence and remaining limitations are recorded in
[Managed extension contracts](MANAGED_EXTENSIONS.md). The runnable external example
is [managed_extensions](../examples/managed_extensions.rs); regression cases are
in [extensions](../tests/extensions.rs). This does not close full EXT acceptance.

Scope: EXT-5 contracts and the managed exchange boundary needed by EXT-1/4.
Demonstrate custom setup, a typed operation and a two-exchange fixture workflow
from an external consumer. EXT-2/3 follow once ownership is demonstrated.
No disputed device codec is required for this slice.

Before fixing public signatures, record these decisions here or in a linked design
document and check them against the existing architecture:

| Contract | Required decision and validation |
| --- | --- |
| Ownership and lifetimes | One managed exclusive context per setup/workflow; exchanges borrow it; no stream/context escapes. Demonstrate async lifetime and Send/Sync ergonomics externally. |
| Admission and deadlines | Retain bounded Etherbird admission and lifecycle setup deadlines; specify overall workflow and per-exchange deadlines. Test queued and active timeouts separately. |
| Error classification | Separate local validation, healthy device rejection, malformed typed decoding and transport failure; specify stream retirement for each. |
| Cancellation and stop | Interrupted exchanges retire their generation; dropped work releases ownership; stop cancels/drains work and is awaited. Test late replies. |
| Generation state | Setup reruns on replacement; keys/handles remain generation-local. Workflows cannot silently resume or replay side effects across replacement. |
| Reader interaction | Heartbeats and unsolicited dispatch share ownership rules; competing readers cannot consume workflow responses. Idle reception is a separate deliverable. |
| Registration and compatibility | Choose immutable registration or explicit invalidation; define dependency/API compatibility and secret-safe diagnostics. |

Implementation sequence:

1. Audit Client/Session/Connector and published Etherbird interfaces against these
   contracts; record any missing framework capability before exposing it.
2. Implement the minimum scoped managed exchange context and typed operation,
   retaining existing APIs and legacy login/clock readiness defaults.
3. Use that boundary for a custom setup strategy and two-exchange workflow,
   keeping the workflow in one generation under exclusive ownership.
4. Add external-consumer fixture examples and independent integration cases for
   rejection, malformed decoding, competing clones, deadlines, cancellation,
   recovery and stop. Test disconnect after a simulated side effect without replay.
5. Update coverage, EXT-1/4/5 subitems and relevant decision dispositions with exact
   evidence links; keep unfinished requirements and unresolved DEC rows open.

Alpha review adds typed DecodeError, exclusively owned state destroyed on disconnect,
checked GenerationValue handles, required-state errors and terminal authentication
policy. Credential-provider/recovery, README/usage doctests and extracted-package
consumer checks are recorded in [release evidence](RELEASE.md). Explicit device
logout remains unsupported; changed credentials require stopped-client replacement.

Slice exit: an external consumer compiles and runs all three examples without
crate-private access, and ownership/recovery checks pass. This closes only
evidenced subitems; full EXT-1/4/5 acceptance and M0-E require their remaining
checklist items. Applicability metadata and device logout remain explicit design tasks.

## Delivery order

1. Write EXT contracts and map known DEC questions to seams and remaining framework requirements.
2. Implement EXT-1 and EXT-4 using existing Session/Connector paths and Etherbird ownership.
3. Implement EXT-2 and EXT-3, then exercise all seams from outside the crate.
   Implement sufficiently specified alternatives as predefined extensions using
   these interfaces; demonstrate replacing one preset from the external consumer.
4. Ship supported defaults and extension guides once contract checks pass; continue typed services and built-in catalogs.
5. Promote independently evidenced community solutions as verified built-ins.

Do not attempt complete enhanced authentication, speculative file/tunnel formats
or a universal firmware dictionary merely to demonstrate the extension API.
