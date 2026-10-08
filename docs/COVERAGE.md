# Protocol coverage and implementation sequence

Specification baseline:
[Emerson D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
The linked document is authoritative; its text and tables are not redistributed.
The full implementation target is tracked in [the roadmap](OUTSTANDING_WORK.md),
[service work packages](SPEC_WORK_PLAN.md) and [schema work packages](SCHEMA_WORK_PLAN.md).
These inventories describe planned scope, not additional implemented support.
The [extensibility plan](EXTENSIBILITY.md) includes implemented managed setup,
typed operations and exclusive workflows. Profiles and schema providers remain
planned. See [contracts and exact evidence](MANAGED_EXTENSIONS.md). Connector,
caller-provided parameter types and raw single-unicast requests remain supported.
[Reference specifications](reference/README.md) records its source, revision and
the sections used by the implementation and fixtures.

## Implemented foundation

| Area / opcode | Coverage | Evidence |
| --- | --- | --- |
| Unicast frame codec | Addresses, length, checksum, exact framing | Published CRC/SRBX vectors and malformed-frame tests |
| CRC-16 | Zero seed, reflected polynomial, wire byte order | Chapter 4 vector plus standard check value |
| 6 | Original/extended configuration, point counts, raw security and identity fields; unknown data retained | Independent Table 2-2 payloads, fixed TCP fixture, malformed reply and recovery tests |
| 7 | Typed clock read and validation | Table 2-3 fixture and independent peer |
| 17 | Legacy operator ID/numeric password and optional access level | Fixed request tests, rejected/accepted simulator login, setup recovery |
| 167 | Typed contiguous parameter reads | Echo/range/width checks and TCP integration |
| 180 | Typed arbitrary TLP reads | Echo/count/width/response-budget checks and TCP integration |
| 224 | Route interleaved signals to stable receiver | Published vector and recovery integration |
| 255 | Preserve multiple and unknown error entries | Fixed error fixtures; healthy connection preserved |
| Parameter types | Binary, fixed ASCII, signed/unsigned integers, float/double, TLP, time | Literal byte-order/width tests |
| TCP | Fresh connection seam and simulator transport | Real loopback sockets |
| Serial | Configurable stream connector | Compilation only; physical validation outstanding |
| Supervision | Login/readiness restoration, queue bound, deadlines, heartbeat, cancellation/shutdown | Etherbird-backed integration suite |
| Managed extensions | Immutable setup strategies, typed operations, exclusive workflows, terminal auth, owned generation state and checked handles | External fixtures, packaged consumer, README/usage doctests and [release evidence](RELEASE.md) |

Table 2-3 is used for the clock year representation instead of the older illustration
in Figure 1-2. Error offsets are retained as raw bytes; their meaning depends on the
operation. TLP parameter widths come from caller-provided schemas.

## Typed validation and configuration applicability

Typed replies are decoded under the Session exchange guard: malformed
configuration, clock, login acknowledgement and parameter replies retire the
generation before another caller can use it. This applies to standalone Session
and managed Client; ordinary valid device rejections and local validation remain
healthy. Legacy login rejection during managed setup is the deliberate exception:
it latches terminal authentication failure and that setup generation is discarded.
Regression evidence: `standalone_session_retires_every_malformed_typed_reply`
and `standalone_typed_device_rejection_keeps_session_usable`.

Configuration layout interpretation requires an independently established
firmware major version: `logical_compatibility(None)` and versions below 2 return
None, while `logical_compatibility_raw()` always preserves the byte. Neither the
configuration revision nor response length establishes firmware applicability.
Evidence: `layout_interpretation_requires_independent_firmware_applicability`.
This does not resolve DEC-3 mappings or DEC-8 firmware discovery.

## Remaining typed service coverage

| Service | Opcodes / scope | Status |
| --- | --- | --- |
| I/O layout and user-defined point discovery | 50, 100 | Raw unicast exchange only; typed parsing/simulation outstanding |
| Set clock / parameter writes | 8, 166, 181 | Raw unicast exchange only; write codecs and partial-write semantics outstanding |
| Configurable tables | 10, 11 | Raw unicast exchange only; typed codecs outstanding |
| Enhanced login/session key and logout | Additional forms of 17 | Not implemented |
| Store and forward / broadcast | 24 / no-reply operations | Unsupported by request API |
| History | 105, 108, 135, 136, 137, 138, 139, 206 | Raw unicast exchange only; typed paging/record models outstanding |
| Alarms/events | 118, 119 | Raw unicast exchange only; typed log retrieval outstanding |
| File transfers | 203 | Raw unicast exchange only; file lifecycle/chunking outstanding |
| Peer network tunnel | 205 | Raw unicast exchange only; typed tunneled exchange outstanding |
| SRBX full workflow | 224 / 225 | Signal routing implemented; log-consumption and acknowledgement workflow outstanding |
| Parameter schema catalog | Device/firmware point dictionaries | Caller-provided types; built-in dictionary outstanding |
| Physical interoperability | Ethernet / serial controller | No hardware tests performed |

A raw request capability is not a complete service implementation. The current
simulator rejects operations outside its documented subset, and the crate does not
claim full ROC Plus conformance.

## Next implementation steps

See [OUTSTANDING_WORK.md](OUTSTANDING_WORK.md) for the detailed backlog and
completion criteria, including transport regressions, hardware evidence and releases.

1. Complete the remaining [extension acceptance tasks](EXTENSIBILITY.md):
   authentication applicability/logout and outstanding lifecycle cases. The first
   managed slice, API review and alpha onboarding are implemented; see
   [release evidence](RELEASE.md) for pending hosted CI and hardware gates.
2. Complete protocol profiles and schema providers, then use configuration
   discovery for supported device/setup policies and firmware-aware selection.
3. Expand schema-backed parameter models and read-only alarm/history services,
   each starting with independently specified wire fixtures.
4. Implement built-in enhanced authentication only after its procedure is established;
   known device procedures may be user-supplied through the authentication seam.
5. Add write/file/routing operations with explicit side-effect and session-lifetime
   semantics, and no implicit operation replay.
6. Validate physical Ethernet and serial communication as described in HARDWARE.md;
   hardware and CI validation can proceed alongside the software milestones.

Each service needs a documented request/response shape, positive and malformed
fixtures, simulator dispatch, TCP integration, and a recorded hardware-validation
status. Passing a frame round trip alone is insufficient.
