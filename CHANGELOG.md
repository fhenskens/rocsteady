# Changelog

## 0.1.0 — Unreleased

- Add explicit protocol profiles, contextual reply codecs and replaceable
  configuration/clock/log decoding, with capability and payload-budget validation.
- Add schema providers/chains and bounded generation-exclusive read plans with
  confirmed partial progress on failure.
- Add opcode-100 point-type classification and both opcode-50 I/O selectors,
  explicit 160/240/224-entry layouts, observed consistency checks and physical
  location mapping. Discovery does not infer firmware or authentication policy.
- Add 130 source-audited ROC800 catalog parameters across PT-91/95/101/103/109/136/139,
  including all PT-101/103/136 read widths, firmware/location gates and optional
  enum, bit and fixed-unit metadata. Preserve raw unknown values and conflicting
  source constraints; complete point-table conformance and typed writes remain open.
- Add typed opcode-118/119 alarm/event pages and all enumerated records/tags,
  preserving raw records, overlapping old/new storage and incomplete large values.
  Add bounded ring windows and caller-consumed checkpoints with observed anchor
  checks; matching anchors do not establish durable gap-free continuity.
- Add opt-in caller-driven idle SRBX polling with bounded notifications, lag
  reporting, shutdown-aware intervals and normal call/exchange deadlines.
- Add explicit opcode-225 acknowledgement of consumed alarm checkpoints, raw-anchor
  verification and NotAttempted/Rejected/Unconfirmed/Confirmed outcomes. Close the
  send gate on cancellation/deadline and never replay. Confirmed receipt does not
  prove pending status cleared when newer alarms exist.
- Adopt published etherbird 0.4.2 and its shared lifecycle helpers, bounded FIFO,
  terminal authentication handling and call deadlines. Consolidate operation
  preparation/decoding and parameter budgets within rocsteady; remove the sibling
  path dependency and duplicated lifecycle wrappers.
- Support current stable Rust with Linux/macOS/Windows CI; remove the fixed Rust
  1.88 requirement. Share the shell/PowerShell packaged-consumer manifest template.
- Add independent wire/TCP and simulator fixtures, discovery and log/SRBX examples,
  public-API package-consumer coverage and updated scope/release documentation.

Migration: `ClientConfig.profile` changes exhaustive struct literals. Supply a
profile or use `..Default::default()`. Releases now use ordinary `0.x.y` versions;
all releases below 1.0 remain experimental. Breaking API changes use a new minor
version; compatible additions and fixes use patch releases. See
[RELEASE.md](docs/RELEASE.md).
These features are not part of published 0.1.0-alpha.1.

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
