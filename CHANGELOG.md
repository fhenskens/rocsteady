# Changelog

## Unreleased

- Share checked payload consumption between arbitrary and contiguous parameter
  reads, preserving operation-specific errors and independently tested literals.
- Generate shell/PowerShell package consumers from one manifest template; handle
  both Cargo package-ID forms and inherit the extracted crate's lockfile.

- Support the current stable Rust toolchain; remove the fixed Rust 1.88 minimum
  and its CI jobs while retaining Linux/macOS/Windows coverage.

- Share built-in request preparation, reply budgets and decoding across Client,
  Session and managed workflows. Encoding validates before admission; decoding
  remains inside the exchange retirement guard.
- Adopt published etherbird 0.4.2 and reuse its RetireOnDrop helper. Preserve
  healthy device rejections and no replay; no sibling path dependency is required.
- Share allocation-free parameter budgets across codecs, profiles and planning;
  separate catalog facts from lookup policy and move unicast validation to protocol.

- Extend the ROC800 catalog from 30 to 50 parameters with all PT-136 clock read
  widths, singleton addressing and explicit 1.20/3.40 introduction gates.
  Preserve TIME as raw values and leave the disputed DST end-week range unspecified.
- Add independent clock literals/TCP reads, firmware-boundary and retirement tests,
  simulator coverage and clock reads in the example and packaged consumer.
- Remove a setup test's peer-accept scheduling race by counting connector attempts
  directly when proving that missing setup capabilities do not retry.

- Implement opcode-100 command-11 classification with exact echoes, counts 0-245,
  preserved unknown codes and bounded generation-exclusive range paging.
- Implement both opcode-50 selectors and explicit 160/240/224-entry layouts through
  replaceable profile codecs/mapping policies; detect observed configuration/selector
  changes, preserving physical locations separately from logical ordinals.
- Add 30 source-audited ROC800 system/DI/AI/diagnostic catalog entries, explicit
  firmware/installed-point validation, and generation-bound schema providers/plans.
- Add independent specification-derived TCP fixtures for every initial catalog entry,
  simulator support, a discovery/catalog example and a packaged consumer check.
- Add DiscoveryChanged error classification for healthy but inconsistent discovery;
  retain explicit DEC-3 and hardware limitations in the catalog manifest/work plans.

- Add immutable ProtocolProfile selection, capability/applicability metadata,
  source correlation and tighter payload budgets; clock/configuration decoding
  uses the selected profile during setup, reads and heartbeats.
- Add ReplyCodec/DecodedOperation with original request context, using the managed
  retirement guard; predefined ClockCodec and ConfigurationCodec are replaceable.
- Add SchemaProvider/SchemaChain, explicit identity and access/location metadata,
  bounded ReadPlan and generation-exclusive schema reads with confirmed failure progress.
- Add an external profile/schema TCP example and independent fixtures proving
  malformed-response retirement, healthy rejection and no replay after a committed effect.
- Adding ClientConfig.profile changes exhaustive struct literals; use ..Default::default()
  or supply an explicit profile. These experimental API changes require release review.

- Adopt published etherbird 0.4.1: terminal lifecycle authentication failures,
  healthy operation error retention, checked configuration and shutdown cache release.
- Remove local authentication latch/gate and nested healthy-error results while
  preserving authentication error compatibility, positive deadlines, no replay,
  cancellation retirement and explicit Session teardown.
- Use upstream BoundedFifoQueue and single-attempt readiness/call timeouts;
  remove the local FIFO and timeout/shutdown race wrapper.

## 0.1.0-alpha.1 — 2026-10-09

- Configuration discovery, clock/parameter reads, legacy login, frame/CRC validation
  and preserved device errors over TCP and optional serial streams.
- etherbird 0.3.1 supervision with bounded admission, replacement setup, exclusive
  ownership, call/wire deadlines, cancellation retirement and awaited shutdown.
- Public SetupStrategy, Operation and Workflow with scoped ManagedContext.
- Typed DecodeError, application ErrorKind, terminal authentication failure,
  exclusively owned generation state and checked GenerationValue handles.
- Independent fixtures, simulator fault injection, compile-checked onboarding,
  credential-provider/recovery and serial examples, MSRV and package checks.

Experimental read-only built-ins. No physical controller validation, enhanced
authentication, profiles/schema providers or complete ROC Plus service coverage.
No operation is automatically replayed. Hardware interoperability remains unverified.
