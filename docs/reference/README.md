# Reference specifications

## Research records

- [2026-10-16 development journal](SESSION_2026_10_16.md): historical feature,
  coverage-audit and consolidation timings/checkpoints for the 0.2.0 release.
- [Current decision register](DECISIONS.md): start here for current conclusions,
  delivery dispositions, affected work and closure evidence for the tracked subquestions.
  The records below preserve research history and source provenance.
- [Systematic search record](SEARCH_RECORD.md): exact executed queries, batch
  results, per-ambiguity assessments, failed retrievals and remaining public leads.
- [Online source catalog](SOURCES.md): publishers, revisions, relevant locations,
  research date and source limitations.
- [Specification ambiguity findings](AMBIGUITIES.md): DEC-1 through DEC-10,
  confidence, implementation implications and remaining evidence requirements.
- [2026-10-16 definition follow-up](FOLLOW_UP_2026_10_16.md): new ROCLINK,
  CygNet, ABB and parser-history inspections, hardware scope and concrete next leads.
- [Prepared definition questions](DEFINITION_REQUEST.md): unsent, focused questions
  for future authorized vendor/implementation-owner inquiries.
- [Second research pass](SECOND_PASS.md): older-manual comparisons, pinned parser
  inspection, synthetic-fixture provenance and revised assessments.

Research reviewed on 2026-10-08 supplements the baseline below; it does not change
the normative revision or establish physical interoperability. Related-family
manuals and third-party parser choices require explicit applicability checks.

## ROC Plus protocol baseline

**Title:** ROC Plus Protocol Specifications Manual  
**Publisher:** Emerson  
**Document number:** D301180X012  
**Revision:** October 2022  
**Source checked:** 2026-10-08

[Open the original specification PDF on Emerson's documentation site](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).

This is the specification used for the initial rocsteady implementation. The
reference is the publisher's PDF; the repository does not contain a rehosted copy.
The vendor document retains its own copyright and is not covered by rocsteady's
Apache-2.0 license.

### Implementation references

For the implementation reading order, see [the documentation guide](../README.md).
The complete source inventory is tracked in [the service work plan](../SPEC_WORK_PLAN.md)
and [the 62 point-type work packages](../SCHEMA_WORK_PLAN.md). Specification
contradictions and stale index-only services remain explicit decision items;
the implementation must not guess undocumented wire forms.

| Area | Manual reference |
| --- | --- |
| Frame format, addressing and offsets | Sections 1.2–1.4 |
| Supported operation inventory | Section 2.1 |
| Clock read | Section 2.3, Table 2-3 |
| System configuration discovery | Section 2.2, Table 2-2; offsets 6 through 220 |
| Login and security forms | Section 2.7, Table 2-7 |
| Configured/selected history | Section 2.19, Table 2-22, printed p.2-36; reviewed 2026-10-15 |
| Day-oriented history | Sections 2.17–2.18, Tables 2-20–2-21, printed pp.2-34–35; reviewed 2026-10-15 |
| Multi-point history | Section 2.16, Table 2-19, printed pp.2-32–33; reviewed 2026-10-15 |
| Contiguous parameter read | Section 2.21, Table 2-24 |
| Arbitrary parameter reads | Section 2.22, Table 2-25 |
| SRBX and errors | Sections 2.27–2.29 |
| Multi-variable sensor reads | Section 3.4.19, Table 3-20, printed pp.3-82–89; reviewed 2026-10-15 |
| RTD / thermocouple read fields | Sections 3.4.17–18, Tables 3-18/19, printed pp.3-75–81; reviewed 2026-10-15 |
| TLP addressing and data types | Sections 3.1–3.3 |
| CRC and published example | Chapter 4 |
| Numeric byte representation | Chapter 5 |
| SRBX acknowledgement sequence | Chapter 6 |
| Routed messages | Chapter 7 |

Use the manual's printed page/section numbers when citing fixtures; PDF viewer
page numbers also include front matter. The clock codec follows Table 2-3 rather
than the older illustration in Figure 1-2. The remaining implementation questions
are tracked in [OUTSTANDING_WORK.md](../OUTSTANDING_WORK.md).

Before changing the baseline revision, review codec/schema differences and update
the corresponding fixtures and [coverage record](../COVERAGE.md). A link that
continues to work is not proof that the publisher has left the file unchanged.

## Channel and hardware references

- [ROC800-Series product data sheet, D301155X012](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301155x012.pdf):
  supported physical communication interfaces.
- [ROCLINK 800 configuration manual, D301250X012](https://www.emerson.com/is/content/emerson/en/final-control/documents/d301250x012.pdf):
  Ethernet port and connection configuration.
- [ROC800 instruction manual, LOI wiring section](https://www3.emersonprocess.com/remote/mobile/flipbook/a6175_ROC800-series_remote_operations_controller_instruction_manual/files/assets/basic-html/page-127.html):
  controller-specific serial connector context.

The hardware references supplement the protocol manual. Confirm their revision
and applicability to the actual controller/module before hardware validation;
see [HARDWARE.md](../HARDWARE.md).
