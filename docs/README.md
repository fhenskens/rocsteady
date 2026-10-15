# Documentation guide

Start here when planning or implementing rocsteady. The target is ROC Plus
D301180X012, October 2022, Chapters 1–7; delivery starts with an extensible client
and verified defaults. Unresolved device behavior remains explicit.
Ambiguous device behavior uses public extension contracts; sufficiently specified
alternatives will be provided as named selectable built-in extensions through those
same contracts, so callers can replace them with their own implementations.

## Resuming development

Start with the [current handoff](OUTSTANDING_WORK.md#current-handoff--2026-10-15)
and [recommended next slice](OUTSTANDING_WORK.md#recommended-next-slice).
Version 0.1.3 includes the 202-parameter read catalog, history metadata (OP-105/108)
and indexed/day-oriented history reads (OP-135–139), with explicit policies/codecs,
bounded paging and confirmed progress. Feature contracts and coverage state each
service's limits. The handoff separates implemented work, remaining baseline
acceptance and physical validation; the roadmap recommends OP-166 next.

## Reading order

1. [Coverage](COVERAGE.md): implemented behavior and current evidence.
2. [Roadmap](OUTSTANDING_WORK.md): milestones, dependencies and release gates.
3. [Extensibility plan](EXTENSIBILITY.md): delivered boundaries and remaining
   EXT acceptance requirements; implementation contracts live in MANAGED_EXTENSIONS.md.
4. [Current decision register](reference/DECISIONS.md): delivery dispositions,
   open questions and evidence required for the selected work.
5. [Service work packages](SPEC_WORK_PLAN.md) and
   [schema work packages](SCHEMA_WORK_PLAN.md): select OP/CORE/PT variants and
   their completion criteria, then read the actual manual tables.

## Document responsibilities

| Document | Authoritative within this repository for |
| --- | --- |
| [Usage](USAGE.md) | Credentials, reads, errors, serial configuration and runnable snippets |
| [Managed contracts](MANAGED_EXTENSIONS.md) | Public extension ownership, deadlines, cancellation and generation rules |
| [Coverage](COVERAGE.md) | Current implemented support and limitations |
| [Requirements](REQUIREMENTS.md) | Stable requirement IDs, executable test evidence and audit limits |
| [Roadmap](OUTSTANDING_WORK.md) | Delivery order, dependencies and product/release gates |
| [Extensibility](EXTENSIBILITY.md) | Delivered/planned extension boundaries and external-consumer acceptance |
| [Service plan](SPEC_WORK_PLAN.md) | Baseline opcode/core inventory and closure criteria |
| [Schema plan](SCHEMA_WORK_PLAN.md) | Point-type inventory and per-parameter closure criteria |
| [Catalog](CATALOG.md) | Selected source-audited parameter metadata, discovery policies, per-variant fixtures and explicit firmware/hardware limitations |
| [History information](HISTORY_INFORMATION.md) | OP-139 configured inventory, ordered selections, optional timestamps and bounded paging |
| [Day-oriented history](DAY_HISTORY.md) | OP-137/138 table defaults, caller interpretation, whole-day reads and explicit indexed fallback |
| [Multi-point history](MULTI_HISTORY.md) | OP-136 time-major rows, point/period budgets, paging and partial progress |
| [Single-point history](HISTORY.md) | OP-135 selectors, bounds/policies, paging and partial progress |
| [History metadata](HISTORY_METADATA.md) | OP-105/108 contracts, defaults, prebuilt alternatives and evidence |
| [Logs and SRBX](LOGS.md) | Typed records, bounded paging/checkpoints, explicit acknowledgement outcomes and idle polling |
| [Architecture](ARCHITECTURE.md) | Existing ownership, recovery and stream behavior |
| [Release evidence](RELEASE.md) | Publication, version policy, repeatable checks and validation provenance |
| [Dependency audit](DEPENDENCIES.md) | Locked dependency licenses and package contents |
| [Hardware](HARDWARE.md) | Channel prerequisites and physical-validation requirements |
| [Reference index](reference/README.md) | Normative source identity and research navigation |
| [Decision register](reference/DECISIONS.md) | Current DEC dispositions and closure evidence |

Work plans do not replace the vendor wire specification. The research records
explain evidence and its limitations; they do not establish hardware compatibility.

Keep the layers distinct: usage/contracts describe supported APIs, coverage and
catalog manifests describe implemented scope, work plans describe remaining
acceptance, and reference documents preserve source evidence and unresolved
questions. Historical research is retained for provenance; current dispositions
come from the decision register. Published release checks do not establish
physical-controller interoperability.

## Working on a slice

Select stable EXT/CORE/OP/PT identifiers and relevant decision rows. Record the
contracts and source sections, derive independent literal fixtures, implement the
API/codecs and simulator, then verify transport and lifecycle behavior. Update
coverage and the selected work-package statuses together. Record physical evidence
separately; user-supplied behavior does not close built-in conformance items.

When new evidence changes a conclusion, update the decision register and affected
plans. Add the source to the catalog and preserve dated research records as history.
New subquestion IDs append within their DEC group; do not renumber existing rows.
