# Changelog

## Unreleased

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

- Adopt published Etherbird 0.4.1: terminal lifecycle authentication failures,
  healthy operation error retention, checked configuration and shutdown cache release.
- Remove local authentication latch/gate and nested healthy-error results while
  preserving authentication error compatibility, positive deadlines, no replay,
  cancellation retirement and explicit Session teardown.
- Use upstream BoundedFifoQueue and single-attempt readiness/call timeouts;
  remove the local FIFO and timeout/shutdown race wrapper.

## 0.1.0-alpha.1 — 2026-10-09

- Configuration discovery, clock/parameter reads, legacy login, frame/CRC validation
  and preserved device errors over TCP and optional serial streams.
- Etherbird 0.3.1 supervision with bounded admission, replacement setup, exclusive
  ownership, call/wire deadlines, cancellation retirement and awaited shutdown.
- Public SetupStrategy, Operation and Workflow with scoped ManagedContext.
- Typed DecodeError, application ErrorKind, terminal authentication failure,
  exclusively owned generation state and checked GenerationValue handles.
- Independent fixtures, simulator fault injection, compile-checked onboarding,
  credential-provider/recovery and serial examples, MSRV and package checks.

Experimental read-only built-ins. No physical controller validation, enhanced
authentication, profiles/schema providers or complete ROC Plus service coverage.
No operation is automatically replayed. Hardware interoperability remains unverified.
