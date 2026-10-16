# Prepared protocol-definition questions

Draft only; no inquiry has been sent. Baseline: ROC800 ROC Plus D301180X012,
October 2022. Please identify exact device family, firmware, document revision
and whether an answer is normative, an erratum or implementation-specific.
Public documentation/reference vectors are preferred; do not include production
credentials, private keys or operational controller data.

## Priority 1: enhanced OP-17 authentication — DEC-2.01/02/04

- What public/developer supplement defines how to unwrap the 24-byte session-key
  response and construct the 30-byte operator-ID / 40-byte password fields?
- Specify the algorithm, key source/derivation, mode, nonce/IV, encoding, padding,
  credential normalization and exact order of operations. Supply a non-secret vector
  from known dummy inputs through complete request/reply bytes.
- Is the key bound to the TCP connection, communication port, login attempt or
  controller? What invalidates it, and what is its validity interval on ROC800
  specifically? Related-family five-second wording is not sufficient.
- Which identity/firmware/security-enable/configuration reads are available before
  authentication? Identify error responses and the effect of logout or key expiry.

## Priority 2: OP-203 files — DEC-5.01–03

- Confirm delete request command 5 versus the October 2022 request row's 6, with
  an erratum or applicable implementation definition. Directory is separately 6.
- For directory request/reply forms (including response 64), specify count widths,
  complete layouts, filename terminators, continuation input and EOF/no-progress rules.
- Define handle lifetime across logout/disconnect/reconnect, block budgets and whether
  cancellation/uncertain writes permit any supported explicit reconciliation/resume.

## Priority 3: OP-206 transactions — DEC-5.05–07

- Confirm command-2 offset placement at frame offset 10 and whether every stored
  field includes a TLP before its type tag. Table labels and repetition text conflict.
- Does Message Data Size include the continuation byte, and does command-1 Payload
  Size count any per-field TLPs? Explain tag 15's `T_STRING40` spelling/width.
- Can fields span response pages? Specify offset/continuation behavior for an empty,
  replaced or vanished record, including same-number reuse after rollover.

## Other documentary clarifications

- OP-205 response commissioned-index width versus the printed following offsets.
- ROC800 Ethernet SRBX applicability: reconcile the protocol framework with the
  ROCLINK February 2024 manual's exclusion of SRBX over TCP (p.3-8).
- Identify any public errata collection or supported developer specification channel
  covering firmware variants and the contradictory parameter rows in DEC-10.
