# Changelog

## 0.1.3

- Add OP-139 configured-point inventory and selected minute/periodic/daily reads
  with optional timestamps, preserved point order/duplicates, independent budgets
  and explicit policy/codec/profile replacements.
- Add bounded selected-point paging, optional inventory membership/recheck, confirmed
  progress and cursor/configuration-change outcomes without automatic replay/resume.
  Include independent requirement/TCP/lifecycle tests, explicit simulator values,
  history_information example and packaged public consumer checks.

- Add OP-137 day/month index lookup and OP-138 whole-day values/point-255
  timestamps, preserving raw fields, invalid zero-count daily indices and IEEE bits.
  Named table codecs, per-call/profile replacements and public typed operations
  support explicit caller interpretation without guessing an archival year or extra
  OP-137 point field.
- Add opt-in OP-137/135 indexed day paging with explicit geometry/timestamp point,
  exclusive generation ownership, confirmed progress and final mapping recheck.
  Include independent requirements/TCP/lifecycle tests, simulator oversized-day
  behavior, day_history example and extracted-package consumer checks.

- Add OP-136 multi-point minute/periodic/daily reads with time-major timestamp/value
  rows, zero-period probes, exact point/period budgets and raw FLOAT/TIME preservation.
- Add bounded one-generation period paging, explicit short/zero replies, confirmed
  failure progress and index-change policies without implicit replay/resume. Reuse
  public geometry policies and expose profile/per-call reply codec replacement.
- Add independent requirement-linked literals/TCP/lifecycle tests, explicit simulator
  data, a runnable multi_history example and extracted-package consumer coverage.

- Reconcile documentation for schema read planning, idle SRBX polling, dependency
  adoption and decision-table formatting.

## 0.1.2 — 2026-10-15

- Add all 27 PT-102 discrete-output and 17 PT-104 analog-output read fields;
  the ROC800 catalog includes explicit firmware gates,
  source access/ranges, enum/alarm-bit/unit metadata and unresolved-source caveats.
- Add all 28 PT-105 pulse-input read fields with counts, rates, contract-day totals,
  mode/units/alarm metadata and an explicit nonzero-conversion constraint. The
  catalog now contains 202 parameters; values are reported without local conversion.
- Add independent literal/TCP requirements tests, all-layout simulator values,
  discovery example output reads and public extracted-package consumer checks.
- Add OP-105 min/max and OP-108 history-tag/index reads to Client, Session and
  workflows. Default to lossless raw occurrence bytes; provide named provisional
  calendar/prefixed-epoch codecs, per-call selection and public profile/codec overrides.
  Include independent requirements/TCP/lifecycle tests, simulator, example and
  extracted-package consumer coverage; no timestamp autodetection.
- Add OP-135 single-point history values and timestamps, count-zero probes,
  bounded one-generation paging, explicit progress/short reads and index-change
  outcomes. Default to table minute bounds; provide selectable provisional prose
  bounds, configured geometry and caller policy/codec replacements. Include
  independent requirements/TCP/lifecycle tests, simulator, example and consumer.
- Link implemented requirements to spec-referenced assertions; strengthen exact
  parameter budgets, echoed field validation, weekday and published frame tests.

Output writes, full point-table conformance and physical interoperability remain open.

## 0.1.1 — 2026-10-15

- Declare the docs.rs documentation URL in crate metadata and build hosted API
  documentation with all features, including serial and simulator APIs.

## 0.1.0 — 2026-10-15

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
