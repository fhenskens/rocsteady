# Changelog

## Unreleased

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
