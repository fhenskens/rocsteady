# Implemented requirements and executable evidence

Baseline: [Emerson D301180X012, October 2022](https://www.documentation.emersonprocess.com/groups/public/documents/instruction_manuals/d301180x012.pdf).
Audit recorded 2026-10-15. This index covers the declared implemented subset in
[COVERAGE.md](COVERAGE.md); unimplemented OP/PT variants remain in the work plans.
It is not a declaration of exhaustive clause coverage or physical conformance.

`REQ-` identifies a wire/table requirement or its defensive validation.
`POL-` identifies a rocsteady contract or an explicit interpretation; those are not
additional vendor requirements. IDs appear beside executable assertions in the
named tests. Existing tests exercise several requirements; an ID does not mean
that a test proves every statement in the referenced section. Rust assertions,
independent literal fixtures and TCP peers encode the expected behavior; this
index provides navigation rather than substituting for those assertions.

The new boundary tests were checked directly against §1.2, Table 2-3,
Table 2-24, §2.22/Table 2-25 and Chapter 6 in the baseline PDF. Other entries
index the existing source-cited fixtures and catalog audit evidence. They are
not a claim that every source table was freshly re-audited in this pass.
OP-136–139 additions were checked against Tables 2-19–2-22 and the
corresponding §§2.16–2.19; source ambiguity and physical evidence remain separate.

## Requirement index

| Stable ID | Source | Requirement asserted | Executable test |
| --- | --- | --- | --- |
| REQ-CRC-VECTOR | Ch.4 | Published CRC and independent check value | [crc_matches_manual_chapter_four_vector](../tests/protocol.rs) |
| REQ-SRBX-WIRE | Ch.6 | Published signal and acknowledgement wire bytes | [published_srbx_and_ack_vectors_decode_and_encode_exactly](../tests/protocol.rs) |
| REQ-FRAME-LENGTH | §1.2 | Reject truncation, trailing bytes and corrupt frames | [frame_length_and_crc_failures_are_rejected](../tests/protocol.rs) |
| REQ-007-FIELDS | Table 2-3 | Eight clock bytes, UINT16 year and field validation | [clock_uses_uint16_year_not_obsolete_leap_year_byte](../tests/protocol.rs) |
| REQ-017-LEGACY | Table 2-7 | Three operator bytes, little-endian password, optional access | [legacy_login_packs_password_little_endian_and_optional_access](../tests/protocol.rs) |
| REQ-180-REQUEST | Table 2-25 | Count followed by ordered TLPs; reject invalid budgets | [opcode_180_request_has_count_and_ordered_tlps](../tests/protocol.rs) |
| REQ-180-VALUES | Table 2-25 | Decode mixed widths; reject truncation and trailing bytes | [opcode_180_response_echoes_tlp_before_each_value](../tests/protocol.rs) |
| REQ-167-REQUEST | Table 2-24 | Contiguous request fields and range validation | [opcode_167_validates_count_range_and_echoed_block_header](../tests/protocol.rs) |
| REQ-TYPES | §3.4 table types; Ch.5 | Independent literals for every supported data type and byte order | [all_supported_data_types_decode_independent_literals](../tests/protocol.rs) |
| REQ-255-PAIRS | §2.29 | Preserve multiple error/offset pairs and unknown codes; reject malformed lists | [error_response_preserves_all_codes_and_offsets_including_unknown_codes](../tests/protocol.rs) |
| REQ-006-ORIGINAL | Table 2-2 | Original/pre-1.20 lengths, fields and point counts | [original_and_pre_120_discovery_have_distinct_available_counts](../tests/configuration.rs) |
| REQ-006-EXTENDED | Table 2-2 | Extended counts, truncation and unknown-field preservation | [extended_counts_and_unknown_fields_are_preserved](../tests/configuration.rs) |
| POL-006-FIRMWARE | DEC-8; configuration contract | Do not infer layout firmware from response length/revision | [layout_interpretation_requires_independent_firmware_applicability](../tests/configuration.rs) |
| REQ-050-SELECTORS | Table 2-12; DEC-3 | Both selectors, explicit lengths, absent/unknown positions | [table_2_12_explicit_selector_lengths_preserve_absent_unknown_and_diagnostic_bytes](../tests/discovery.rs) |
| REQ-100-CLASSIFICATION | Table 2-13 | Command/start/count echoes, classifications and range checks | [table_2_13_literals_validate_command_start_count_and_classifications](../tests/discovery.rs) |
| REQ-100-PAGING | Table 2-13; client contract | Independent TCP pages across all 256 types and zero count | [independent_tcp_pages_cover_256_types_and_zero_count_without_wrapping](../tests/discovery.rs) |
| POL-050-MAPPING | §3.2; DEC-3 | Explicit layout maps logical ordinals to physical TLP locations | [all_layouts_map_logicals_to_physical_tlps_and_preserve_unknown_points](../tests/discovery.rs) |
| REQ-CATALOG-VALUES | Chapter 3; CATALOG.md per-parameter manifest | Every delivered entry has independent literal/TCP width, value, access and introduction assertions | [every_catalog_row_reads_independent_literal_values_over_tcp](../tests/discovery.rs) |
| REQ-CATALOG-INPUTS | Tables 3-13/3-15 | Account for every DI/AI read width, range and display metadata | [input_tables_account_for_every_read_width_range_and_display_meaning](../tests/discovery.rs) |
| REQ-CATALOG-BITS | Tables 3-13/3-15 | Known alarm bits retain unknown/unused bits | [input_alarm_bit_meanings_preserve_all_unused_bits](../tests/discovery.rs) |
| REQ-CATALOG-CLOCK | Table 3-48 | Clock field widths, location and firmware gates | [clock_catalog_firmware_ranges_and_singleton_location_follow_table_3_48](../tests/discovery.rs) |
| POL-CATALOG-GENERATION | Schema/discovery contract | Invalid locations fail locally; replacement invalidates snapshot-bound providers/plans | [invalid_locations_fail_locally_and_replacement_invalidates_catalog_and_plans](../tests/discovery.rs) |
| REQ-118-RECORDS | Table 2-16 | Alarm variants, flags, reasons and float widths | [table_2_16_alarm_variants_flags_reason_and_float_widths_are_literal](../tests/logs.rs) |
| REQ-119-RECORDS | Table 2-17 | Eight event kinds, unknown kinds and spare bytes | [table_2_17_all_eight_event_kinds_and_unknown_kind_preserve_spare_bytes](../tests/logs.rs) |
| REQ-119-TAGS | §2.14 | Event-specific old/new tagged values and overlapping fields | [event_specific_tags_decode_old_new_and_overlaps_without_inventing_large_values](../tests/logs.rs) |
| REQ-LOG-HEADERS | §§2.13–2.14 | Page headers, record limits, malformed lengths and checkpoint validation | [page_headers_record_limits_and_checkpoints_are_strict](../tests/logs.rs) |
| POL-LOG-CONTINUITY | Log contract | Changed/overwritten anchors report gaps | [overwritten_or_changed_anchor_reports_gap_without_retiring_healthy_connection](../tests/logs.rs) |
| POL-LOG-PROGRESS | Log contract | Interrupted paging exposes confirmed progress; resume is explicit | [interrupted_second_page_returns_confirmed_progress_and_explicit_resume_after_reconnect](../tests/logs.rs) |
| REQ-225-CONSUMED | Ch.6; §2.28 | Acknowledgement uses consumed index and published request bytes | [chapter_6_consumed_index_seven_is_acknowledged_with_published_request_bytes](../tests/srbx.rs) |
| REQ-225-PENDING | Ch.6 | New alarm before acknowledgement leaves pending status | [new_alarm_before_ack_leaves_pending_and_does_not_jump_to_unconsumed_index](../tests/srbx.rs) |
| POL-225-ANCHOR | SRBX contract | Never acknowledge an unverified consumed anchor | [changed_or_missing_consumed_anchor_never_sends_ack_and_keeps_connection](../tests/srbx.rs) |
| POL-224-IDLE | SRBX contract | Opt-in polling receives idle signals and detects index growth | [idle_polling_receives_repeated_signals_and_missed_signal_still_reports_index_growth](../tests/srbx.rs) |
| POL-SESSION-MALFORMED | Managed ownership contract | Malformed typed replies retire the session | [standalone_session_retires_every_malformed_typed_reply](../tests/integration.rs) |
| POL-SESSION-REJECTION | Managed ownership contract | Valid device rejections preserve the session | [standalone_typed_device_rejection_keeps_session_usable](../tests/integration.rs) |
| POL-RECOVERY-SETUP | Managed setup contract | Replacement restores login before next call | [failures_return_once_then_new_session_logs_in_before_next_call](../tests/integration.rs) |
| POL-CANCELLATION | Managed ownership contract | Late replies cannot contaminate subsequent calls | [timeout_or_cancellation_discards_late_response_and_does_not_contaminate_next_call](../tests/integration.rs) |
| POL-NO-REPLAY | Managed operation contract | Failed calls return once; concurrent calls serialize | [failed_request_is_not_replayed_and_concurrent_calls_are_serialized](../tests/integration.rs) |
| POL-SCHEMA-BUDGET | Schema read contract | Plans obey request, response and framework budgets | [planning_obeys_each_budget_and_framework_ceiling](../tests/profiles_schema.rs) |
| POL-PROFILE-CODEC | Profile contract | Public codec replacement and failure retirement | [contextual_codec_is_replaceable_and_decode_failure_retires](../tests/profiles_schema.rs) |
| POL-SCHEMA-PROGRESS | Schema read contract | Failed second batch preserves confirmed progress without replay | [failed_second_batch_reports_confirmed_progress_without_resume](../tests/profiles_schema.rs) |
| REQ-FRAME-CRC | §1.2; Ch.6 | Published offsets and CRC byte order; compatibility mode retains length validation | [req_frame_crc_published_ack_has_exact_addresses_payload_and_checksum_order](../tests/protocol.rs) |
| REQ-007-WEEKDAY | Table 2-3 | Accept every weekday 1–7; reject every other byte | [req_007_weekday_accepts_all_seven_values_and_rejects_other_bytes](../tests/protocol.rs) |
| REQ-167-BUDGET | Table 2-24 | Accept 230 value bytes; reject 231; parameter 255 does not wrap | [req_167_budget_accepts_230_value_bytes_and_rejects_231](../tests/protocol.rs) |
| REQ-167-ECHO | Table 2-24 | Reject corruption of each of four echoed fields | [req_167_echo_rejects_each_wrong_header_field](../tests/protocol.rs) |
| REQ-180-BUDGET | §2.22; Table 2-25 | Accept 240 total payload bytes; reject 241, counting count/TLP overhead | [req_180_budget_counts_header_and_tlp_in_240_bytes](../tests/protocol.rs) |
| REQ-180-ECHO | Table 2-25 | Reject count and each TLP component mismatch, including reordered results | [req_180_echo_rejects_count_and_each_tlp_component_in_each_position](../tests/protocol.rs) |
| REQ-OUTPUT-TABLES | Tables 3-14/3-16, §§3.4.13/3.4.15; ownership contract for POL | All 44 fields: independent width/access/version/range/annotation inventory | [output_tables_account_for_every_width_access_version_range_and_annotation](../tests/discovery.rs) |
| REQ-OUTPUT-ALARMS | Tables 3-14/3-16, §§3.4.13/3.4.15; ownership contract for POL | All alarm bit combinations and unused-bit masks | [output_alarm_masks_preserve_every_unknown_bit](../tests/discovery.rs) |
| REQ-OUTPUT-READS | Tables 3-14/3-16, §§3.4.13/3.4.15; ownership contract for POL | Independent TCP reads, all layouts and invalid locations | [output_all_fields_read_over_tcp_for_every_layout](../tests/discovery.rs) |
| REQ-OUTPUT-FIRMWARE | Tables 3-14/3-16, §§3.4.13/3.4.15; ownership contract for POL | 1.00/1.09/1.10/3.00/3.01 applicability boundaries | [output_firmware_boundaries_fail_locally_before_any_batch](../tests/discovery.rs) |
| POL-OUTPUT-RETIREMENT | Tables 3-14/3-16, §§3.4.13/3.4.15; ownership contract for POL | Healthy rejections, malformed/disconnected reads and stale-plan rejection | [output_rejections_malformed_replies_and_generation_replacement](../tests/discovery.rs) |
| REQ-OUTPUT-RAW | Tables 3-14/3-16, §§3.4.13/3.4.15; ownership contract for POL | Unknown codes, unused bits and IEEE NaN/infinity payload preservation | [output_unknown_values_and_float_bits_survive_reads](../tests/discovery.rs) |
| REQ-OUTPUT-SIMULATOR | Tables 3-14/3-16 | Explicit values for every output field, all layouts | [simulator_reads_every_output_field_in_each_layout](../tests/discovery_simulator.rs) |
| REQ-OUTPUT-PACKAGE | Tables 3-14/3-16 | Output reads and metadata through extracted package | [package consumer](../scripts/package-consumer.rs) |
| REQ-PI-TABLE | §3.4.16, Table 3-17, pp.3-72–74; ownership contract for POL | All 28 widths/access/version/range/unit/source facts; nonzero constraint explicit | [pulse_table_accounts_for_every_read_field_and_constraint](../tests/pulse_catalog.rs) |
| REQ-PI-ENUMS | §3.4.16, Table 3-17, pp.3-72–74; ownership contract for POL | Every documented enum code and every unknown byte | [pulse_enumerations_preserve_unknown_codes](../tests/pulse_catalog.rs) |
| REQ-PI-ALARMS | §3.4.16, Table 3-17, pp.3-72–74; ownership contract for POL | All alarm masks and raw byte combinations | [pulse_alarm_bits_match_source_and_retain_unused_bit](../tests/pulse_catalog.rs) |
| REQ-PI-READS | §3.4.16, Table 3-17, pp.3-72–74; ownership contract for POL | Independent TCP literals, firmware/location boundaries, all applicable layouts | [pulse_reads_every_field_at_firmware_and_location_boundaries](../tests/pulse_catalog.rs) |
| POL-PI-RETIREMENT | §3.4.16, Table 3-17, pp.3-72–74; ownership contract for POL | Malformed replies, healthy rejections, no replay and stale-plan rejection | [pulse_malformed_replies_rejections_and_replacements_have_explicit_outcomes](../tests/pulse_catalog.rs) |
| REQ-PI-RAW | §3.4.16, Table 3-17, pp.3-72–74; ownership contract for POL | Raw counts/codes, zero conversion, NaN/infinity/signed-zero bits and totals preserved | [pulse_raw_values_preserve_unknown_codes_out_of_range_counts_and_float_bits](../tests/pulse_catalog.rs) |
| REQ-PI-SIMULATOR | Table 3-17 | Explicit values for all 28 pulse fields and every layout | [simulator_reads_every_pulse_field_in_each_layout](../tests/discovery_simulator.rs) |
| REQ-PI-PACKAGE | Table 3-17 | Counts, rates, totals and metadata through the packaged public API | [package consumer](../scripts/package-consumer.rs) |
| REQ-105-REQUEST / REQ-108-REQUEST | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Baseline bounds, max count, ordered duplicates and explicit applicability overrides | [requests_validate_boundaries_and_explicit_applicability_overrides](../tests/history_metadata.rs) |
| REQ-105-FIELDS | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Every fixed field, occurrence raw bytes, unknown archival methods and IEEE values | [min_max_literals_cover_every_field_and_unknown_archival_method](../tests/history_metadata.rs) |
| REQ-105-VALIDATE | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | All truncations, trailing bytes and both echoes | [min_max_rejects_every_truncation_trailing_data_and_wrong_echo](../tests/history_metadata.rs) |
| POL-105-OCCURRENCES | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Raw default and explicit conflicting interpretations with lossless bytes and bounds | [raw_calendar_and_prefixed_epoch_choices_are_explicit_and_lossless](../tests/history_metadata.rs) |
| REQ-108-REPLY | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | 224-byte maximum reply, echoes/order, raw tags and all truncations | [history_tags_validate_maximum_count_echoes_order_and_padding](../tests/history_metadata.rs) |
| REQ-105-TCP / REQ-108-TCP / POL-105-EXTENSION | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Independent TCP requests/replies, defaults, alternatives, consumer output types and changing indices | [independent_tcp_defaults_and_replaceable_prebuilt_and_custom_codecs](../tests/history_metadata.rs) |
| POL-HISTORY-BUDGET | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Both service budgets/capabilities checked before connect/admission | [impossible_request_reply_budgets_and_capabilities_fail_before_connect](../tests/history_metadata.rs) |
| POL-HISTORY-RETIREMENT | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Malformed typed replies retire; device rejections preserve; no replay | [malformed_metadata_and_valid_device_rejections_have_distinct_lifecycle_outcomes](../tests/history_metadata.rs) |
| POL-HISTORY-CANCEL | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Active cancellation/timeouts retire and recover without replay | [interrupted_metadata_requests_return_once_and_recover_for_explicit_calls](../tests/history_metadata.rs) |
| POL-HISTORY-OWNERSHIP | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Shared codec semantics and exclusive workflow ownership | [standalone_and_workflow_interfaces_use_the_same_typed_contract](../tests/history_metadata.rs) |
| POL-105-DECODE-GUARD | Tables 2-14/2-15, pp.2-22–24; DEC-4.01 and managed contracts for POL | Consumer decoder failure retires under the exchange guard | [custom_occurrence_decoder_failure_retires_before_the_next_call](../tests/history_metadata.rs) |
| REQ-HISTORY-SIMULATOR | Tables 2-14/2-15 | Explicit metadata model, missing points and invalid requests | [simulator tests](../tests/history_metadata_simulator.rs) |
| REQ-HISTORY-PACKAGE | Tables 2-14/2-15; extension contracts | Both services, prebuilt and consumer codec choices through extracted package | [consumer](../scripts/package-consumer.rs) |
| REQ-135-REQUEST / POL-135-BOUNDS | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Request fields, page/window bounds, table/prose/configured/consumer geometry | [request_bounds_policies_and_wire_fields_are_explicit](../tests/history.rs) |
| REQ-135-SELECTORS | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | All five selectors, count 0/60, FLOAT vs TIME and exact IEEE bits | [every_selector_decodes_literals_with_index_only_and_maximum_page_shapes](../tests/history.rs) |
| REQ-135-VALIDATE | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | All truncations, echoes, returned count/width and selected index bounds | [malformed_count_echo_width_and_current_index_are_rejected](../tests/history.rs) |
| REQ-135-TCP | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Independent TCP requests/replies, 245-byte pages, probes and explicit prose policy | [independent_tcp_supports_all_selectors_probes_and_named_alternative](../tests/history.rs) |
| POL-135-PAGING | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Tighter budgets, known ring split, one generation and terminal unknown-capacity index | [paging_splits_profile_limits_and_known_retention_boundary](../tests/history.rs) |
| POL-135-BUDGET | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Capability/budgets fail before connection; range splitting fits smaller replies | [invalid_capability_and_budgets_fail_before_any_request](../tests/history.rs) |
| POL-135-SHORT | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Short/empty valid pages retain explicit remaining window and stop | [short_pages_preserve_remaining_window_and_do_not_request_more](../tests/history.rs) |
| POL-135-INDEX-CHANGE | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Strict/observe choices, confirmed changed page, flag and healthy connection | [observed_index_change_is_explicit_and_preserves_healthy_connection](../tests/history.rs) |
| POL-135-PROGRESS / POL-135-NO-REPLAY | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Confirmed prefix after malformed/device/transport/timeout failure; explicit new window | [interrupted_second_page_returns_progress_then_allows_explicit_new_window](../tests/history.rs) |
| POL-135-CALL-DEADLINE | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Whole-call timeout/cancellation retire pending work without replay | [call_deadline_and_cancellation_retire_pending_history_pages](../tests/history.rs) |
| POL-135-EXTENSION | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Consumer policy/codec/profile overrides with guarded page shape | [public_custom_policy_codec_and_profile_are_replaceable_with_guarded_shape](../tests/history.rs) |
| POL-135-SESSION | §2.15/Table 2-18, pp.2-31–32; DEC-4.03/04 and managed contracts for POL | Standalone and exclusive workflow methods share the same semantics | [standalone_and_workflow_history_methods_share_the_contract](../tests/history.rs) |
| REQ-135-SIMULATOR | Table 2-18 | All selectors, raw words, probes, retention boundaries and missing/invalid requests | [simulator tests](../tests/history_simulator.rs) |
| REQ-135-PACKAGE | Table 2-18; extension contracts | Default values/times, configured paging and external codec through extracted package | [consumer](../scripts/package-consumer.rs) |
| POL-135-STOP | Managed shutdown contract | Awaited shutdown returns confirmed history prefix and cancels the pending page without replacement | [awaited_shutdown_returns_confirmed_history_prefix](../tests/history.rs) |
| REQ-136-REQUEST / POL-136-BOUNDS | §2.16/Table 2-19, pp.2-32–33; policy contract for POL | Seven-byte request, selectors 0–2, point and index bounds, all point/period products and explicit geometry | [request_fields_selectors_products_and_applicability](../tests/multi_history.rs) |
| REQ-136-ORDER / REQ-136-RAW | Table 2-19 | Time-major TIME then ascending point FLOATs; raw NaN/infinity/signed-zero bits | [literal_time_major_rows_preserve_timestamp_and_float_bits](../tests/multi_history.rs) |
| REQ-136-VALIDATE | Table 2-19 | Both echoes, every truncation, trailing bytes, whole-period count/width and current-index bounds | [malformed_echoes_counts_truncations_and_trailing_bytes_are_rejected](../tests/multi_history.rs) |
| REQ-136-TCP / REQ-136-ZERO | Table 2-19; zero-point formula interpretation remains physically unverified | Independent TCP requests/replies, all selectors, max 60 elements/246 bytes, zero-period probes and timestamp-only rows | [literal_peer_covers_all_selectors_and_maximum_payload](../tests/multi_history.rs) |
| POL-136-PAGING / POL-136-BUDGET | Multi-point contract | Whole-period profile budgets, local failures, known-ring boundaries and unwrapped UINT16 space | [bounded_ranges_split_periods_and_known_ring_boundaries](../tests/multi_history.rs), [impossible_budgets_fail_locally_and_exact_budgets_pass](../tests/multi_history.rs) |
| POL-136-SHORT | Table 2-19 zero-element invalid-request form; paging contract | Short/empty positive replies stop with confirmed incomplete progress; no inferred absence | [short_or_zero_element_replies_stop_with_explicit_incomplete_progress](../tests/multi_history.rs) |
| POL-136-INDEX-CHANGE | Multi-point continuity contract | Stop/observe movement, retain changed page and healthy connection | [changed_index_preserves_the_observed_page_and_healthy_connection](../tests/multi_history.rs) |
| POL-136-PROGRESS / POL-136-NO-REPLAY | Managed history contract | Malformed, rejected, disconnected/stalled page preserves prefix; explicit fresh window only | [interrupted_page_returns_confirmed_prefix_and_explicit_recovery_only](../tests/multi_history.rs) |
| POL-136-CANCELLATION | Managed history contract | Overall deadline, cancellation and awaited stop retire pending work without replay | [deadline_cancellation_and_stop_preserve_no_replay_contract](../tests/multi_history.rs) |
| POL-136-EXTENSION | Managed profile/codec contract | External codecs, profile shape validation and shared Session/workflow methods | [public_codecs_profiles_sessions_and_workflows_obey_the_contract](../tests/multi_history.rs) |
| POL-136-LATE / POL-136-POLICY | Managed stream/geometry contracts | Delayed old reply cannot contaminate replacement; custom geometry checked for every point | [late_reply_cannot_contaminate_the_replacement](../tests/multi_history.rs), [custom_geometry_is_checked_for_every_selected_point](../tests/multi_history.rs) |
| REQ-136-SIMULATOR | Table 2-19 | Explicit words for each selector, point ordering, missing/short/zero replies and ring boundaries | [simulator_multi_point_periods_preserve_order_bits_and_explicit_geometry](../tests/multi_history_simulator.rs) |
| REQ-136-PACKAGE | Table 2-19; public API contract | Default rows, consumer codec and configured bounded range in extracted package | [package consumer](../scripts/package-consumer.rs) |
| REQ-137-REQUEST / REQ-138-REQUEST / POL-DAY-CALENDAR | §§2.17–2.18/Tables 2-20/21, pp.2-34–35; defensive calendar policy | Exact request fields, table-based OP-137 without point, special OP-138 target, yearless calendar and explicit segment limits | [request_layouts_and_yearless_calendar_boundaries](../tests/day_history.rs) |
| REQ-137-FIELDS / REQ-137-DAILY | Table 2-20 | Four UINT16 fields; daily validity follows count, preserving invalid raw index | [indices_keep_raw_invalid_daily_index_and_every_uint16_field](../tests/day_history.rs) |
| REQ-137-VALIDATE / REQ-138-VALIDATE | Tables 2-20/21 | Every truncation, trailing bytes, all echoes, both count widths and frame-field bounds | [codecs_reject_truncation_wrong_echoes_counts_and_oversized_frames](../tests/day_history.rs) |
| REQ-138-VALUES / REQ-138-TIMESTAMPS | Table 2-21 | All periodic before daily words; raw TIME/FLOAT/NaN/infinity/signed-zero bits | [float_and_time_words_are_lossless_and_never_coerced](../tests/day_history.rs) |
| REQ-DAY-TCP / POL-DAY-BUDGET | Tables 2-20/21; profile budget contract | Independent requests/replies, empty/multiple daily entries, exact limits and actual oversized reply retirement | [literal_peer_reads_indices_values_timestamps_and_empty_days](../tests/day_history.rs), [exact_profile_limits_and_impossible_budgets_are_enforced](../tests/day_history.rs) |
| POL-DAY-LIFECYCLE | Managed ownership contract | Both opcodes preserve missing-day rejection, retire malformed/disconnected/timed-out/late replies without replay | [missing_day_rejections_malformed_replies_and_disconnects_have_distinct_outcomes](../tests/day_history.rs) |
| POL-DAY-INDEXED / POL-DAY-TIMESTAMP-POINT | Explicit indexed day contract | Coherent columns, whole-generation paging/wrap and mapping recheck; explicit OP-135 timestamp point, no sentinel inference | [indexed_alternative_pages_both_columns_and_rechecks_mapping](../tests/day_history.rs), [indexed_timestamps_require_explicit_point_and_use_selectors_three_and_four](../tests/day_history.rs) |
| POL-DAY-ZERO / POL-DAY-UNWRAPPED | Table 2-20 validity; indexed geometry contract | Skip empty columns/invalid indices, detect changed mapping, never guess unknown ring wrap | [empty_columns_skip_invalid_indices_and_mapping_changes_are_reported](../tests/day_history.rs), [unknown_geometry_stays_unwrapped_and_invalid_daily_index_is_not_a_mapping](../tests/day_history.rs) |
| POL-DAY-PROGRESS / POL-DAY-NO-REPLAY / POL-DAY-PLAN | Managed indexed day contract | Confirmed prefix/positions after errors; no replay; plan both columns before samples; stop on short replies | [indexed_failure_retains_prefix_without_replaying_or_resuming](../tests/day_history.rs), [indexed_invalid_daily_geometry_is_local_and_short_pages_stop](../tests/day_history.rs) |
| POL-DAY-CANCELLATION / POL-DAY-INDEX-MOVEMENT | Managed indexed day contract | Deadline/abort/stop, per-column stop/observe policy and failed postcheck preserve progress | [indexed_deadline_cancellation_and_stop_retire_pending_work](../tests/day_history.rs), [indexed_cursor_movement_is_explicit_and_postcheck_failures_keep_all_samples](../tests/day_history.rs) |
| POL-DAY-EXTENSION | EXT-2/4 contracts; DEC-4.05/06 remain open | Default public codecs, caller-supplied year label, profile structural checks and external Session/workflow use | [external_codec_and_profile_validation_share_session_and_workflow_boundaries](../tests/day_history.rs), [package consumer](../scripts/package-consumer.rs) |
| REQ-DAY-SIMULATOR | Tables 2-20/21; indexed day contract | Explicit raw words, missing/oversized days, zero daily count and opt-in indexed value/timestamp paging | [simulator_reads_values_timestamps_missing_days_and_oversized_indexed_alternative](../tests/day_history_simulator.rs) |
| REQ-DAY-PACKAGE | Tables 2-20/21; EXT-2/4 contracts | Default reads, caller codec, explicit request Operation and indexed workflows through extracted public API | [package consumer](../scripts/package-consumer.rs) |
| REQ-139-REQUEST / POL-139-SELECTION | §2.19/Table 2-22 p.2-36; nonempty selection is library policy | Exact commands/fields, every point-period product and timestamp budget, selector/point/index applicability | [request_fields_products_selectors_and_explicit_geometry](../tests/history_information.rs) |
| REQ-139-INVENTORY | Table 2-22 | Empty/max inventory, command/segment/count echoes, raw order/duplicates and all truncations | [inventory_count_echo_and_wire_budget_preserve_all_points](../tests/history_information.rs) |
| REQ-139-ORDER / REQ-139-RAW | Table 2-22 | Time-major optional TIME plus requested-order FLOATs, duplicated points and IEEE special bits | [literal_rows_preserve_float_bits_and_optional_timestamp_word](../tests/history_information.rs) |
| REQ-139-VALIDATE / REQ-139-TCP | Table 2-22 | All truncations/header/count/flag/width failures, known indices, every selector/flag, probes and exact 247-byte replies over TCP | [malformed_reply_headers_counts_flags_and_current_indices_are_rejected](../tests/history_information.rs), [independent_tcp_checks_inventory_and_every_selection_shape](../tests/history_information.rs) |
| POL-139-PAGING / POL-139-BUDGET | Selected-history contract | Whole-period request/reply budgets, local unavailable/impossible failures and known/unknown ring boundaries | [ranges_split_periods_and_wrap_only_known_geometry](../tests/history_information.rs), [impossible_budgets_and_unavailable_capability_fail_locally](../tests/history_information.rs) |
| POL-139-INVENTORY / POL-139-INVENTORY-FAILURE | Opt-in inventory observation contract | Missing selected point, changed final list and failed postcheck preserve confirmed progress without replay | [inventory_checks_report_changed_or_removed_points_without_replay](../tests/history_information.rs), [failed_final_inventory_retains_all_confirmed_samples](../tests/history_information.rs) |
| POL-139-SHORT / POL-139-CONTINUITY | Selected-history contract | Short replies stop, observed movement is strict/explicit-observe, healthy channel preserved | [short_replies_do_not_guess_missing_periods_or_continue_inventory_checks](../tests/history_information.rs), [index_movement_stops_or_continues_only_by_explicit_policy](../tests/history_information.rs) |
| POL-139-PROGRESS / POL-139-NO-REPLAY / POL-139-LATE | Managed history contract | Confirmed prefixes, no replay across recovery and delayed old replies cannot contaminate replacement | [failures_return_only_confirmed_pages_and_require_explicit_new_window](../tests/history_information.rs), [delayed_selected_reply_cannot_contaminate_a_new_generation](../tests/history_information.rs) |
| POL-139-CANCELLATION | Managed history contract | Call deadline, caller abort and awaited stop retire pending work without replay | [deadline_cancellation_and_stop_never_replay_pending_pages](../tests/history_information.rs) |
| POL-139-EXTENSION / POL-139-GEOMETRY / POL-139-INVENTORY-SESSION | Public codec/profile/geometry contracts | Consumer codecs, mandatory profile invariants, coherent per-point geometry and standalone malformed inventory retirement | [consumer_codecs_profile_invariants_sessions_and_workflows_are_supported](../tests/history_information.rs), [per_point_geometry_must_be_coherent](../tests/history_information.rs), [malformed_standalone_inventory_retires_before_reuse](../tests/history_information.rs) |
| REQ-139-SIMULATOR | Table 2-22; selected-history contract | Explicit raw values/TIME for all selectors/flags, sparse/duplicate order, zero/max inventory, short and invalid requests | [simulator_inventory_and_selection_use_explicit_values_and_order](../tests/history_information_simulator.rs) |
| REQ-139-PACKAGE | Table 2-22; public extension contracts | Inventory, explicit request Operation, consumer codec, timestamp flags and inventory-checked paging through extracted public API | [package consumer](../scripts/package-consumer.rs) |


## Qualifications and remaining evidence

- **Framing:** raw byte-field capacity is tested separately from service budgets.
  A 255-byte frame-codec test does not establish that a controller accepts that
  size. DEC-1 total-message limits and Ethernet CRC behavior remain open.
- **Clock:** Table 2-3 takes precedence over the obsolete Figure 1-2 year example.
  Calendar validation beyond the explicit weekday range is a defensive client
  policy; reported weekday/calendar consistency is not inferred.
- **Login:** only the legacy form is implemented. Tests do not establish automatic
  security applicability or enhanced authentication (DEC-2/8).
- **Discovery:** selected 160/240/224 layouts are tested interpretations, not a
  resolution of conflicting diagrams or firmware applicability (DEC-3). Repeated
  discovery detects observed changes, not an atomic device snapshot.
- **Catalog:** the 202-entry per-parameter source/literal/TCP manifest is
  [CATALOG.md](CATALOG.md). Defaults, update ownership, conditional writes,
  omitted/reserved parameters and unimplemented table variants remain open.
  No entire PT row is closed by the all-entry test.
- **Logs/SRBX:** tested bounded windows and observed anchors cannot prove durable,
  gap-free ingestion, controller identity or absence of a full unseen ring wrap.
- **Transport:** independent loopback TCP and simulator tests are software evidence.
  Actual SerialConnector exchanges and physical Ethernet/controller/firmware
  interoperability have no evidence; see [HARDWARE.md](HARDWARE.md).
- **Full audit:** fine-grained clause extraction for every delivered configuration
  offset, catalog metadata field, event tag and service-local error meaning still
  needs review against the actual tables. Do not count this index's rows as a
  percentage of the vendor specification. Unknown error offsets remain raw.

## Maintaining the tests

For each feature or changed interpretation, derive expected bytes and boundaries
from the cited source before implementation. Add a stable requirement ID beside
its behavioral assertions and update this index. Include valid cases, exact limits,
malformed/truncated replies and applicable independent TCP/lifecycle cases. Use
literal expected values rather than the production encoder as the only oracle.
Keep disputed interpretations linked to [DECISIONS.md](reference/DECISIONS.md)
and physical evidence separate. Do not close an OP/PT merely because a test passes.

Run the evidence suite with:

```sh
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
```

Named requirements can be selected with Cargo's filter, for example
`cargo test --locked --test protocol req_167_`.
