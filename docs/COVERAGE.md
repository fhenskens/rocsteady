# Protocol coverage and implementation sequence

Specification baseline:
[Emerson D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
The linked document is authoritative; its text and tables are not redistributed.
The full implementation target is tracked in [the roadmap](OUTSTANDING_WORK.md),
[service work packages](SPEC_WORK_PLAN.md) and [schema work packages](SCHEMA_WORK_PLAN.md).
These inventories describe planned scope, not additional implemented support.
The [requirements index](REQUIREMENTS.md) links implemented wire requirements and
client policies to executable assertions, with explicit audit and evidence limits.
The [extensibility plan](EXTENSIBILITY.md) includes implemented managed setup,
typed operations, exclusive workflows, explicit profiles and schema-backed reads.
See [contracts and exact evidence](MANAGED_EXTENSIONS.md). Connector,
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
| 50 | Both I/O selectors; explicit 160/240/224-entry layouts; repeated consistency checks; physical/per-type logical positions kept distinct | Independent Table 2-12 TCP fixtures in tests/discovery.rs and simulator tests; DEC-3/hardware applicability remains unresolved |
| 100 | Command 11 point-type classification, echoed headers, counts 0-245, bounded range paging, unknown codes preserved | Independent Table 2-13 fixtures/TCP and tests/discovery_simulator.rs |
| 105 | Typed current/today/yesterday extrema and last-period value; raw occurrence times by default; selectable provisional calendar/prefixed-epoch or consumer codecs | Table 2-14 literals/TCP, simulator, consumer; [history contracts](HISTORY_METADATA.md); DEC-4.01 remains open |
| 108 | 1–20 ordered points/tags, segment/count/point echoes, raw common periodic index | Table 2-15 literal/TCP boundaries, changing indices, simulator and consumer |
| 118 / 119 | Typed alarm/event pages, all enumerated records/tags, bounded windows and observed anchor checks | Independent literals/TCP, simulator, package consumer and [log contracts](LOGS.md); durable gap-free delivery not claimed |
| 135 | All five single-point selectors, count-zero probes, bounded windows, partial progress and index-movement reporting; explicit replaceable minute policies | Table 2-18 literals/TCP, simulator, consumer; [history contracts](HISTORY.md); DEC-4.03/04 remain open |
| 167 | Typed contiguous parameter reads | Echo/range/width checks and TCP integration |
| 180 | Typed arbitrary TLP reads | Echo/count/width/response-budget checks and TCP integration |
| 224 / 225 | Stable SRBX notifications, opt-in bounded idle polling, explicit consumed-index acknowledgement and staged outcomes | Chapter 6 literals, independent TCP/lifecycle cases, simulator pending comparisons and [contracts](LOGS.md) |
| 255 | Preserve multiple and unknown error entries | Fixed error fixtures; healthy connection preserved |
| Parameter types | Binary, fixed ASCII, signed/unsigned integers, float/double, TLP, time | Literal byte-order/width tests |
| TCP | Fresh connection seam and simulator transport | Real loopback sockets |
| Serial | Configurable stream connector | Compilation only; physical validation outstanding |
| Supervision | Login/readiness restoration, queue bound, deadlines, heartbeat, cancellation/shutdown | etherbird-backed integration suite |
| Managed extensions | Immutable setup strategies, typed operations, exclusive workflows, terminal auth, owned generation state and checked handles | External fixtures, packaged consumer, README/usage doctests and [release evidence](RELEASE.md) |
| Protocol profiles | Explicit applicability/capabilities, replaceable clock/configuration codecs, contextual operations, tighter budgets and source correlation | Independent TCP cases in tests/profiles_schema.rs; OP-105 provisional occurrence presets are explicit; no hardware evidence |
| Schema providers/catalog | Caller identity, ordered overrides, access/location bounds, generation-bound discovery, bounded opcode-180 batches and confirmed progress; 202 PT-91/95/101/102/103/104/105/109/136/139 entries | Independent TCP fixture for every catalog entry, discovery_catalog example and [manifest](CATALOG.md); no complete PT conformance or typed writes |

Table 2-3 is used for the clock year representation instead of the older illustration
in Figure 1-2. Error offsets are retained as raw bytes; their meaning depends on the
operation. TLP parameter widths come from caller-provided schemas.

## Typed validation and configuration applicability

Typed replies are decoded under the Session exchange guard: malformed
configuration, clock, login acknowledgement, history metadata, log and parameter replies retire the
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
| I/O layout and user-defined point discovery | 50, 100 | Typed selectors and command 11 implemented; automatic firmware/layout applicability and DEC-3 hardware evidence outstanding |
| Set clock / parameter writes | 8, 166, 181 | Raw unicast exchange only; write codecs and partial-write semantics outstanding |
| Configurable tables | 10, 11 | Raw unicast exchange only; typed codecs outstanding |
| Enhanced login/session key and logout | Additional forms of 17 | Not implemented |
| Store and forward / broadcast | 24 / no-reply operations | Unsupported by request API |
| Other history samples/transactions | 136, 137, 138, 139, 206 | Raw unicast exchange only; typed sample paging/record models outstanding; OP-105/108 metadata and OP-135 single-point reads implemented above |
| Alarms/events | 118, 119 | Fixed records and bounded retrieval implemented; checkpoint continuity limits explicit; physical evidence outstanding |
| File transfers | 203 | Raw unicast exchange only; file lifecycle/chunking outstanding |
| Peer network tunnel | 205 | Raw unicast exchange only; typed tunneled exchange outstanding |
| SRBX workflow | 224 / 225 | Explicit consumed-checkpoint acknowledgement and bounded idle polling implemented; ingestion completeness/identity are caller contracts; durable delivery/physical evidence remain open |
| Parameter schema catalog | Device/firmware point dictionaries | 202-entry built-in slice, including all PT-136 read widths; remaining metadata/parameters/rows outstanding |
| Physical interoperability | Ethernet / serial controller | No hardware tests performed |

The Chapter 3 inventory contains 62 point-type tables: ten have partial built-in
coverage and 52 remain open. None is claimed fully conformant. The 202 delivered
parameters are a read-oriented subset; full closure includes remaining metadata,
write conditions, variants and applicability evidence. See the
[schema inventory](SCHEMA_WORK_PLAN.md) for the per-table status.

A raw request capability is not a complete service implementation. The current
simulator rejects operations outside its documented subset, and the crate does not
claim full ROC Plus conformance.

## Next implementation steps

See [OUTSTANDING_WORK.md](OUTSTANDING_WORK.md) for the detailed backlog and
completion criteria, including transport regressions, hardware evidence and releases.

1. Complete the remaining [extension acceptance tasks](EXTENSIBILITY.md):
   authentication applicability/logout and outstanding lifecycle cases. The first
   managed slice, API review and alpha onboarding are implemented; see
   [release evidence](RELEASE.md) for completed release checks and outstanding
   physical-validation limits.
2. Expand the [catalog](CATALOG.md), beyond the delivered PT-102/104/105 read fields; establish remaining firmware/layout
   applicability and supported discovery/setup policies without guessing identity.
3. Expand schema-backed parameter models and read-only history services; establish
   durable alarm-ingestion and controller-identity policies beyond the explicit SRBX APIs,
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
