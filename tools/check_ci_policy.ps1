param(
  [string] $Mode = 'Library',
  [string] $Base = '',
  [string] $Head = '',
  [string] $Integration = '',
  [string] $RepositoryRoot = ''
)

# This is routing, not attestation. Its accepted-base revision is used by CI.
# Review and owner-authorized integration protect changes to its callers too.
function Assert-HumCiSha([string] $Value) {
  if ($Value -cnotmatch '\A[0-9a-f]{40}\z') { throw 'ci_policy: invalid revision identity' }
}

function Get-HumCiOwnership {
  # Literal accepted ownership, never populated from the candidate checkout.
  # All unlisted paths (including new lookalikes and policy/schema files) are Full.
  $Owners = New-Object 'Collections.Generic.Dictionary[string,int]' ([StringComparer]::Ordinal)
  foreach ($Path in @(
    'examples/control_flow.hum'
    'examples/core/add.hum'
    'examples/core/count_completed.hum'
    'examples/core/divide.hum'
    'examples/core/minimal_add.hum'
    'examples/probes/bounded_stdout.hum'
    'examples/probes/capability_root.hum'
    'examples/probes/causal_failures.hum'
    'examples/probes/element_views.hum'
    'examples/probes/exact_file_read.hum'
    'examples/probes/fallible_app_entry.hum'
    'examples/probes/field_places.hum'
    'examples/probes/field_views.hum'
    'examples/probes/first_word.hum'
    'examples/probes/integrated_local_app.hum'
    'examples/probes/list_builder.hum'
    'examples/probes/opaque_native_path.hum'
    'examples/probes/passed_callable_row.hum'
    'examples/probes/passed_pure_callable.hum'
    'examples/probes/pure_app_entry.hum'
    'examples/probes/runner_replay_clock.hum'
    'examples/probes/task_list_flow.hum'
    'examples/probes/transaction_once.hum'
    'examples/probes/word_count.hum'
    'examples/probes/writable_field_aliases.hum'
    'examples/reference_surface.hum'
    'examples/session_server.hum'
    'examples/task_list.hum'
    'examples/task_list_tests.hum'
    'fixtures/app_entry/session_aa_missing_replay_caller_fail.hum'
    'fixtures/app_entry/session_aa_missing_replay_source_fail.hum'
    'fixtures/app_entry/session_aa_recursion_missing_caller_precedence_fail.hum'
    'fixtures/app_entry/session_aa_recursion_missing_source_precedence_fail.hum'
    'fixtures/app_entry/session_aa_replay_recursion_fail.hum'
    'fixtures/app_entry/session_aa_reserved_replay_name_fail.hum'
    'fixtures/app_entry/session_ab_direct_entry_path_fail.hum'
    'fixtures/app_entry/session_ab_multiple_path_parameters_fail.hum'
    'fixtures/app_entry/session_ad_missing_file_source_fail.hum'
    'fixtures/app_entry/session_ad_reserved_file_read_name_fail.hum'
    'fixtures/app_entry/session_x_direct_child_shadows_external_pass.hum'
    'fixtures/app_entry/session_x_duplicate_direct_child_fail.hum'
    'fixtures/app_entry/session_x_duplicate_start_fail.hum'
    'fixtures/app_entry/session_x_empty_start_fail.hum'
    'fixtures/app_entry/session_x_external_helper_fail.hum'
    'fixtures/app_entry/session_x_external_same_name_fail.hum'
    'fixtures/app_entry/session_x_full_type_binding_mismatch_fail.hum'
    'fixtures/app_entry/session_x_invalid_result_fail.hum'
    'fixtures/app_entry/session_x_invalid_start_name_fail.hum'
    'fixtures/app_entry/session_x_missing_start_fail.hum'
    'fixtures/app_entry/session_x_multiple_apps_fail.hum'
    'fixtures/app_entry/session_x_multiple_start_lines_fail.hum'
    'fixtures/app_entry/session_x_nested_call_shadows_external_pass.hum'
    'fixtures/app_entry/session_x_nested_non_child_fail.hum'
    'fixtures/app_entry/session_x_undeclared_result_root_fail.hum'
    'fixtures/app_entry/session_x_unit_return_mismatch_fail.hum'
    'fixtures/app_entry/session_x_unknown_start_fail.hum'
    'fixtures/app_entry/session_y_app_capability_mismatch_fail.hum'
    'fixtures/app_entry/session_y_entry_capability_bypass_fail.hum'
    'fixtures/app_entry/session_y_entry_transitive_process_fail.hum'
    'fixtures/app_entry/session_y_entry_transitive_wildcard_fail.hum'
    'fixtures/app_entry/session_y_missing_caller_capability_fail.hum'
    'fixtures/app_entry/session_y_policy_id_occurrences_pass.hum'
    'fixtures/app_entry/session_y_unknown_capability_fail.hum'
    'fixtures/app_entry/session_z_legacy_output_without_app_fail.hum'
    'fixtures/app_entry/session_z_missing_output_caller_fail.hum'
    'fixtures/app_entry/session_z_missing_stdout_source_fail.hum'
    'fixtures/app_entry/session_z_output_recursion_fail.hum'
    'fixtures/app_entry/session_z_recursion_missing_caller_precedence_fail.hum'
    'fixtures/app_entry/session_z_recursion_missing_source_precedence_fail.hum'
    'fixtures/app_entry/session_z_reserved_stdout_name_fail.hum'
    'fixtures/callable/session_al_anonymous_callable_fail.hum'
    'fixtures/callable/session_al_argument_hws_fail.hum'
    'fixtures/callable/session_al_chained_application_fail.hum'
    'fixtures/callable/session_al_compound_transport_fail.hum'
    'fixtures/callable/session_al_cross_file_fail/caller.hum'
    'fixtures/callable/session_al_cross_file_fail/receiver.hum'
    'fixtures/callable/session_al_extra_indirect_close_fail.hum'
    'fixtures/callable/session_al_fallible_task_fail.hum'
    'fixtures/callable/session_al_lexical_identity_pass.hum'
    'fixtures/callable/session_al_mismatched_delimiter_fail.hum'
    'fixtures/callable/session_al_missing_indirect_close_fail.hum'
    'fixtures/callable/session_al_missing_outer_close_fail.hum'
    'fixtures/callable/session_al_mixed_body_type_fail.hum'
    'fixtures/callable/session_al_multiple_application_fail.hum'
    'fixtures/callable/session_al_multiple_target_params_fail.hum'
    'fixtures/callable/session_al_nested_application_fail.hum'
    'fixtures/callable/session_al_nested_callable_escape_fail.hum'
    'fixtures/callable/session_al_non_task_value_fail.hum'
    'fixtures/callable/session_al_parameter_hws_fail.hum'
    'fixtures/callable/session_al_permission_argument_fail.hum'
    'fixtures/callable/session_al_permission_type_fail.hum'
    'fixtures/callable/session_al_recursive_relationship_fail.hum'
    'fixtures/callable/session_al_resource_nonparticipant_fail.hum'
    'fixtures/callable/session_al_returned_callable_fail.hum'
    'fixtures/callable/session_al_save_transport_fail.hum'
    'fixtures/callable/session_al_selected_invalid_receiver_fail.hum'
    'fixtures/callable/session_al_set_transport_fail.hum'
    'fixtures/callable/session_al_shadowed_invalid_receiver_pass.hum'
    'fixtures/callable/session_al_stored_callable_fail.hum'
    'fixtures/callable/session_al_trailing_indirect_prose_fail.hum'
    'fixtures/callable/session_al_two_indirect_arguments_fail.hum'
    'fixtures/callable/session_al_unknown_ordinary_type_fail.hum'
    'fixtures/callable/session_al_unproven_row_fail.hum'
    'fixtures/callable/session_al_unrelated_unknown_type_pass.hum'
    'fixtures/callable/session_al_unresolved_value_fail.hum'
    'fixtures/callable/session_al_wrong_input_fail.hum'
    'fixtures/callable/session_al_wrong_result_fail.hum'
    'fixtures/callable/session_al_zero_application_fail.hum'
    'fixtures/callable/session_al_zero_indirect_arguments_fail.hum'
    'fixtures/callable/session_al_zero_target_params_fail.hum'
    'fixtures/callable/session_am_mixed_pure_effectful_applications_fail.hum'
    'fixtures/callable/session_am_multiple_direct_applications_fail.hum'
    'fixtures/diagnostics/session_ao_adjacent_distinct_causes_fail.hum'
    'fixtures/diagnostics/session_ao_callable_prior_blocker_fail.hum'
    'fixtures/diagnostics/session_ao_same_code_distinct_occurrences_fail.hum'
    'fixtures/diagnostics/session_ao_typed_failure_prior_blocker_fail.hum'
    'fixtures/diagnostics/session_ap_authority_ownership_precedence_fail.hum'
    'fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum'
    'fixtures/diagnostics/session_ap_ownership_resource_profile_chain_fail.hum'
    'fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum'
    'fixtures/diagnostics/session_ap_path_predicate_precedence_fail.hum'
    'fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum'
    'fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum'
    'fixtures/diagnostics/session_aq_app_scope_reanalysis_fail.hum'
    'fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum'
    'fixtures/diagnostics/session_aq_same_code_distinct_occurrences_fail.hum'
    'fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum'
    'fixtures/diagnostics/session_aq_static_runtime_shared_ownership_fail.hum'
    'fixtures/editor/incomplete_task_header.hum'
    'fixtures/editor/malformed_nested_item.hum'
    'fixtures/editor/mid_edit_missing_does.hum'
    'fixtures/editor/missing_close_brace.hum'
    'fixtures/editor/orphan_body_line.hum'
    'fixtures/effect_check/session_w_avoids_failure_fail.hum'
    'fixtures/effect_check/session_w_missing_fails_when_fail.hum'
    'fixtures/effect_check/session_w_placeholder_fails_when_direct_fail.hum'
    'fixtures/effect_check/session_w_placeholder_fails_when_propagation_fail.hum'
    'fixtures/effect_check/session_w_placeholder_fails_when_wrap_fail.hum'
    'fixtures/effect_check/simple_pass.hum'
    'fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum'
    'fixtures/foundation/pre_ar_comparison_conjunction_pass.hum'
    'fixtures/foundation/pre_ar_condition_chained_comparison_fail.hum'
    'fixtures/foundation/pre_ar_nested_chained_comparison_fail.hum'
    'fixtures/foundation/pre_ar_predicate_chained_comparison_fail.hum'
    'fixtures/foundation/pre_ar_real_unclosed_block_fail.hum'
    'fixtures/foundation/pre_ar_return_chained_comparison_fail.hum'
    'fixtures/foundation/pre_ar_text_braces_pass.hum'
    'fixtures/full_type_check/session_aa_implicit_replay_fail.hum'
    'fixtures/full_type_check/session_aa_invalid_replay_call_fail.hum'
    'fixtures/full_type_check/session_ab_path_contract_comparison_fail.hum'
    'fixtures/full_type_check/session_ab_path_storage_fail.hum'
    'fixtures/full_type_check/session_ab_test_path_construction_fail.hum'
    'fixtures/full_type_check/session_ab_text_literal_path_fail.hum'
    'fixtures/full_type_check/session_ad_file_read_wrong_type_fail.hum'
    'fixtures/full_type_check/session_ad_implicit_file_read_fail.hum'
    'fixtures/full_type_check/session_af_path_trailing_prose_fail.hum'
    'fixtures/full_type_check/session_af_predicate_v2_boundary_fail.hum'
    'fixtures/full_type_check/session_af_predicate_v2_calls_and_types_fail.hum'
    'fixtures/full_type_check/session_af_predicate_v2_places_fail.hum'
    'fixtures/full_type_check/session_af_predicate_v2_text_uint_fail.hum'
    'fixtures/full_type_check/session_w_direct_wrong_root_fail.hum'
    'fixtures/full_type_check/session_w_implicit_fallible_call_fail.hum'
    'fixtures/full_type_check/session_w_incompatible_unwrapped_fail.hum'
    'fixtures/full_type_check/session_w_nested_implicit_calls_fail.hum'
    'fixtures/full_type_check/session_w_precedence_fail.hum'
    'fixtures/full_type_check/session_w_try_infallible_fail.hum'
    'fixtures/full_type_check/session_w_try_prefix_fallible_fail.hum'
    'fixtures/full_type_check/session_w_trying_infallible_pass.hum'
    'fixtures/full_type_check/session_w_unsupported_try_core_fail.hum'
    'fixtures/full_type_check/session_w_unsupported_try_shape_fail.hum'
    'fixtures/full_type_check/session_w_wrong_wrapper_root_fail.hum'
    'fixtures/full_type_check/session_z_implicit_stdout_fail.hum'
    'fixtures/full_type_check/session_z_stdout_wrong_type_fail.hum'
    'fixtures/full_type_check/simple_pass.hum'
    'fixtures/ownership_check/session_j_borrow_pass.hum'
    'fixtures/ownership_check/session_j_borrow_write_fail.hum'
    'fixtures/ownership_check/session_j_change_pass.hum'
    'fixtures/ownership_check/session_j_consume_pass.hum'
    'fixtures/ownership_check/session_j_double_consume_fail.hum'
    'fixtures/ownership_check/session_j_use_after_move_fail.hum'
    'fixtures/ownership_check/session_k_branch_consume_fail.hum'
    'fixtures/ownership_check/session_k_double_consume_fail.hum'
    'fixtures/ownership_check/session_k_missing_consume_fail.hum'
    'fixtures/ownership_check/session_l_return_parameter_view_pass.hum'
    'fixtures/ownership_check/session_l_return_view_internal_fail.hum'
    'fixtures/ownership_check/session_l_return_view_local_fail.hum'
    'fixtures/ownership_check/session_n_return_view_local_slice_fail.hum'
    'fixtures/ownership_check/session_n_return_view_lost_provenance_fail.hum'
    'fixtures/ownership_check/session_o_field_write_borrow_fail.hum'
    'fixtures/ownership_check/session_p_add_after_finish_fail.hum'
    'fixtures/ownership_check/session_p_append_during_iteration_fail.hum'
    'fixtures/ownership_check/session_r_stale_item_field_view_fail.hum'
    'fixtures/ownership_check/session_r_stale_point_field_view_fail.hum'
    'fixtures/ownership_check/session_s_append_iteration_view_overlap_fail.hum'
    'fixtures/ownership_check/session_s_stale_element_view_fail.hum'
    'fixtures/ownership_check/session_v_alias_control_flow_fail.hum'
    'fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum'
    'fixtures/ownership_check/session_v_alias_escape_fail.hum'
    'fixtures/ownership_check/session_v_alias_name_collision_fail.hum'
    'fixtures/ownership_check/session_v_alias_permission_wrapper_fail.hum'
    'fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum'
    'fixtures/ownership_check/session_v_alias_to_alias_fail.hum'
    'fixtures/ownership_check/session_v_borrowed_owner_alias_fail.hum'
    'fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum'
    'fixtures/ownership_check/session_v_nested_alias_place_fail.hum'
    'fixtures/ownership_check/session_v_overlap_read_fail.hum'
    'fixtures/ownership_check/session_v_program8_overlap_write_fail.hum'
    'fixtures/ownership_check/session_v_second_alias_fail.hum'
    'fixtures/ownership_check/simple_pass.hum'
    'fixtures/profile_check/simple_pass.hum'
    'fixtures/programs/hello_world/unsupported_helper_call_fail.hum'
    'fixtures/programs/hello_world/unsupported_nonliteral_output_fail.hum'
    'fixtures/programs/hello_world/unsupported_two_writes_fail.hum'
    'fixtures/programs/integer_sign/duplicate_app_fail.hum'
    'fixtures/programs/integer_sign/duplicate_module_fail.hum'
    'fixtures/programs/integer_sign/helper_before_start_fail.hum'
    'fixtures/programs/integer_sign/illegal_pre_app_store_fail.hum'
    'fixtures/programs/integer_sign/illegal_pre_app_task_fail.hum'
    'fixtures/programs/integer_sign/illegal_pre_app_test_fail.hum'
    'fixtures/programs/integer_sign/late_module_fail.hum'
    'fixtures/programs/integer_sign/layout_valid_pass.hum'
    'fixtures/programs/integer_sign/missing_app_fail.hum'
    'fixtures/programs/integer_sign/missing_module_fail.hum'
    'fixtures/programs/integer_sign/module_path_identity_fail.hum'
    'fixtures/programs/integer_sign/semantic_after_app_fail.hum'
    'fixtures/programs/integer_sign/start_not_first_fail.hum'
    'fixtures/programs/integer_sign/type_after_app_fail.hum'
    'fixtures/programs/integer_sign/unsupported_shape_fail.hum'
    'fixtures/resource_check/simple_pass.hum'
    'fixtures/run/session_af_predicate_v2_inequality_fail.hum'
    'fixtures/run/session_af_predicate_v2_inequality_pass.hum'
    'fixtures/run/session_af_predicate_v2_lone_bang_fail.hum'
    'fixtures/run/session_af_predicate_v2_malformed_neighbor_fail.hum'
    'fixtures/run/session_af_predicate_v2_mixed_full_type_fail.hum'
    'fixtures/run/session_af_predicate_v2_prose_warning.hum'
    'fixtures/run/session_af_predicate_v2_quoted_prose_warning.hum'
    'fixtures/run/session_af_predicate_v2_reachable_callee_fail.hum'
    'fixtures/run/session_af_predicate_v2_valid_neighbor_fail.hum'
    'fixtures/run/session_af_predicate_v2_wrong_content_fail.hum'
    'fixtures/run/session_af_predicate_v2_wrong_count_fail.hum'
    'fixtures/run/session_af_predicate_v2_wrong_text_fail.hum'
    'fixtures/run/session_o_complete_item_field_place.hum'
    'fixtures/run/session_t_old_in_needs_prose.hum'
    'fixtures/run/session_t_wrong_swap_contract.hum'
    'fixtures/run/wrong_add_contract.hum'
    'fixtures/target_facts/session_z_reserved_stdio_requirement_fail.hum'
    'src/app_entry.rs'
    'src/ast.rs'
    'src/backend_contract.rs'
    'src/backend_cranelift.rs'
    'src/backend_input.rs'
    'src/callable.rs'
    'src/capabilities.rs'
    'src/capability_root.rs'
    'src/check.rs'
    'src/core_body.rs'
    'src/core_contract.rs'
    'src/core_expr.rs'
    'src/core_lower.rs'
    'src/core_preview.rs'
    'src/core_verify.rs'
    'src/diagnostic.rs'
    'src/diagnostic_catalog.rs'
    'src/diagnostics.rs'
    'src/doctor.rs'
    'src/effect_check.rs'
    'src/element_place.rs'
    'src/evidence.rs'
    'src/explain.rs'
    'src/field_place.rs'
    'src/file_read.rs'
    'src/full_type_check.rs'
    'src/graph.rs'
    'src/ir_contract.rs'
    'src/ir_readiness.rs'
    'src/ir_verify.rs'
    'src/json.rs'
    'src/lsp.rs'
    'src/main.rs'
    'src/math_obligations.rs'
    'src/native_path.rs'
    'src/native_program.rs'
    'src/node_id.rs'
    'src/operator_grant.rs'
    'src/ownership_check.rs'
    'src/parser.rs'
    'src/path_boundary.rs'
    'src/predicate.rs'
    'src/profile_check.rs'
    'src/resolve.rs'
    'src/resource_check.rs'
    'src/resource_report.rs'
    'src/return_dependency.rs'
    'src/run.rs'
    'src/runtime_profiles.rs'
    'src/sha256.rs'
    'src/state_model.rs'
    'src/syntax.rs'
    'src/target_facts.rs'
    'src/test_skeletons.rs'
    'src/type_check.rs'
    'src/type_env.rs'
    'src/typed_failure.rs'
    'src/version.rs'
    'src/writable_field_alias.rs'
  )) { $Owners.Add($Path, 2) }
  $Owners['src/run.rs'] = 1
  $Owners['examples/probes/word_count.hum'] = 0
  $Owners.Add('README.md', 0)
  $Owners.Add('docs/LANGUAGE_REFERENCE.md', 0)
  return ,$Owners
}

function Get-HumCiProfile([string[]] $Paths) {
  $Owners = Get-HumCiOwnership
  $Rank = 0
  $Seen = New-Object 'System.Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
  if ($Paths.Count -eq 0) { return 'full' }
  foreach ($Path in $Paths) {
    if ([string]::IsNullOrEmpty($Path) -or $Path -cmatch '[\x00-\x20\x7f\\:]' -or
        $Path.StartsWith('/') -or @($Path.Split('/') | Where-Object { $_ -in @('', '.', '..') }).Count -ne 0 -or
        -not $Seen.Add($Path)) { throw 'ci_policy: malformed or ambiguous inventory' }
    $Current = if ($Owners.ContainsKey($Path)) { $Owners[$Path] } else { 3 }
    $Rank = [Math]::Max($Rank, $Current)
  }
  @('language', 'runtime', 'compiler', 'full')[$Rank]
}

function Read-HumCiGit([string] $Root, [string[]] $Arguments) {
  # Production callers use read-only queries; fixture tests also create owned
  # Git objects. No shell interpretation of revision/path operands occurs.
  $Git = (Get-Command git -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
  $Info = New-Object Diagnostics.ProcessStartInfo
  $Info.FileName = $Git
  $Info.WorkingDirectory = $Root
  $Info.UseShellExecute = $false
  $Info.CreateNoWindow = $true
  $Info.RedirectStandardOutput = $true
  $Info.RedirectStandardError = $true
  # All arguments are fixed tokens or authenticated hexadecimal revisions.
  $Info.Arguments = '--no-optional-locks ' + ($Arguments -join ' ')
  $Info.StandardOutputEncoding = New-Object Text.UTF8Encoding($false, $true)
  $Process = New-Object Diagnostics.Process
  $Process.StartInfo = $Info
  try {
    if (-not $Process.Start()) { throw 'ci_policy: Git did not start' }
    $Out = $Process.StandardOutput.ReadToEndAsync()
    $Err = $Process.StandardError.ReadToEndAsync()
    if (-not $Process.WaitForExit(30000)) { $Process.Kill(); $Process.WaitForExit(); throw 'ci_policy: Git query deadline' }
    $Text = $Out.GetAwaiter().GetResult()
    $Warnings = $Err.GetAwaiter().GetResult()
    if ($Warnings.Length -ne 0) { [Console]::Error.Write($Warnings) }
    if ($Process.ExitCode -ne 0) { throw 'ci_policy: Git query failed' }
    $Text
  } finally { $Process.Dispose() }
}

function ConvertFrom-HumCiRawChanges([string] $Raw) {
  if ($Raw.Length -eq 0) { return }
  if (-not $Raw.EndsWith([string][char]0, [StringComparison]::Ordinal)) { throw 'ci_policy: incomplete raw inventory' }
  $Fields = $Raw.Substring(0, $Raw.Length - 1).Split([char]0)
  if ($Fields.Count % 2 -ne 0) { throw 'ci_policy: incomplete change record' }
  $Rows = New-Object 'Collections.Generic.List[object]'
  $Seen = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
  for ($Index = 0; $Index -lt $Fields.Count; $Index += 2) {
    $Match = [regex]::Match($Fields[$Index], '\A:([0-7]{6}) ([0-7]{6}) ([0-9a-f]{40}) ([0-9a-f]{40}) ([AMDT])\z')
    $Path = $Fields[$Index + 1]
    if (-not $Match.Success -or -not $Seen.Add($Path)) { throw 'ci_policy: malformed or duplicate raw record' }
    $null = Get-HumCiProfile @($Path) # Validate spelling before publishing any row.
    $OldMode = $Match.Groups[1].Value; $NewMode = $Match.Groups[2].Value
    $OldId = $Match.Groups[3].Value; $NewId = $Match.Groups[4].Value; $Status = $Match.Groups[5].Value
    $OldAbsent = $OldMode -ceq '000000'; $NewAbsent = $NewMode -ceq '000000'
    if ($OldAbsent -ne ($OldId -ceq ('0' * 40)) -or $NewAbsent -ne ($NewId -ceq ('0' * 40)) -or
        ($Status -ceq 'A' -and (-not $OldAbsent -or $NewAbsent)) -or
        ($Status -ceq 'D' -and ($OldAbsent -or -not $NewAbsent)) -or
        ($Status -cin @('M','T') -and ($OldAbsent -or $NewAbsent)) -or
        ($OldMode -ceq $NewMode -and $OldId -ceq $NewId)) { throw 'ci_policy: inconsistent raw record' }
    $Rows.Add([pscustomobject]@{Path=$Path;OldMode=$OldMode;NewMode=$NewMode;OldId=$OldId;NewId=$NewId;Status=$Status})
  }
  $Rows.ToArray()
}

function Get-HumCiChangeProfile([object[]] $Changes) {
  # Additions/deletions (including both rename sides) require renewed ownership.
  # Only ordinary, same-mode, registered file modifications are normal.
  if ($Changes.Count -eq 0) { return 'full' }
  foreach ($Row in $Changes) {
    if ($Row.Status -cne 'M' -or $Row.OldMode -cne '100644' -or $Row.NewMode -cne '100644') { return 'full' }
  }
  Get-HumCiProfile @($Changes.Path)
}

function Read-HumCiChanges([string] $Root, [string] $Before, [string] $After) {
  $Raw = Read-HumCiGit $Root @('diff', '--no-ext-diff', '--no-textconv', '--no-renames', '--raw', '--no-abbrev', '-z', $Before, $After, '--')
  @(ConvertFrom-HumCiRawChanges $Raw)
}

function Get-HumCiSelection([string] $Root, [string] $Base, [string] $Head, [string] $Integration) {
  foreach ($Sha in @($Base, $Head, $Integration)) { Assert-HumCiSha $Sha }
  $Actual = (Read-HumCiGit $Root @('rev-parse', 'HEAD')).TrimEnd([char[]]@(10,13))
  if ($Actual -cne $Integration) { throw 'ci_policy: checkout differs from integration revision' }
  $Parents = (Read-HumCiGit $Root @('rev-list', '--parents', '-n', '1', $Integration)).TrimEnd([char[]]@(10,13)).Split(' ')
  if ($Parents.Count -ne 3 -or $Parents[0] -cne $Integration -or $Parents[1] -cne $Base -or $Parents[2] -cne $Head) {
    throw 'ci_policy: integration parents differ from the requested base/head'
  }
  $Tree = (Read-HumCiGit $Root @('rev-parse', "$Integration^{tree}")).TrimEnd([char[]]@(10,13))
  Assert-HumCiSha $Tree
  $Profile = 'full'; $Paths = @(); $Reason = 'ambiguous_inventory'
  try {
    if ((Read-HumCiGit $Root @('rev-parse','--is-shallow-repository')).Trim() -cne 'false') { throw 'ci_policy: shallow history' }
    $Changes = @(Read-HumCiChanges $Root $Base $Integration)
    $Profile = Get-HumCiChangeProfile $Changes; $Paths = @($Changes.Path); $Reason = 'classified_changes'
  } catch { Write-Warning 'ci_policy: incomplete change evidence selects Full' }
  [pscustomobject]@{ Profile=$Profile; Base=$Base; Head=$Head; Integration=$Integration; Tree=$Tree; Paths=$Paths; Reason=$Reason }
}

function Get-HumCiPushSelection([string] $Root, [string] $Before, [string] $Head) {
  Assert-HumCiSha $Head
  $Actual = (Read-HumCiGit $Root @('rev-parse','HEAD')).TrimEnd([char[]]@(10,13))
  if ($Actual -cne $Head) { throw 'ci_policy: push checkout differs from event head' }
  $Tree = (Read-HumCiGit $Root @('rev-parse',"$Head^{tree}")).TrimEnd([char[]]@(10,13)); Assert-HumCiSha $Tree
  $Result = [pscustomobject]@{Profile='full';Base=$Before;Head=$Head;Integration=$Head;Tree=$Tree;Paths=@();Reason='ambiguous_history'}
  try {
    Assert-HumCiSha $Before
    if ($Before -ceq ('0'*40) -or $Before -ceq $Head -or
        (Read-HumCiGit $Root @('rev-parse','--is-shallow-repository')).Trim() -cne 'false') { throw 'ci_policy: incomplete push history' }
    $null = Read-HumCiGit $Root @('merge-base','--is-ancestor',$Before,$Head)
    $Text = Read-HumCiGit $Root @('rev-list','--reverse','--max-count=513',"$Before..$Head")
    $Commits = @($Text.TrimEnd([char[]]@(10,13)) -split '\r?\n')
    if ($Commits.Count -eq 0 -or $Commits.Count -gt 512) { throw 'ci_policy: incomplete bounded push inventory' }
    $Rank = 0; $Names = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::Ordinal)
    foreach ($Commit in $Commits) {
      Assert-HumCiSha $Commit
      $Parents = (Read-HumCiGit $Root @('rev-list','--parents','-n','1',$Commit)).TrimEnd([char[]]@(10,13)).Split(' ')
      if ($Parents.Count -lt 2 -or $Parents[0] -cne $Commit) { throw 'ci_policy: unresolved push parent' }
      foreach ($Parent in $Parents[1..($Parents.Count-1)]) {
        Assert-HumCiSha $Parent
        $Changes = @(Read-HumCiChanges $Root $Parent $Commit)
        # An empty merge side is harmless; an entirely empty push remains Full.
        if ($Changes.Count -eq 0) { continue }
        $Profile = Get-HumCiChangeProfile $Changes
        $Rank = [Math]::Max($Rank, [array]::IndexOf(@('language','runtime','compiler','full'),$Profile))
        foreach ($Change in $Changes) { $null = $Names.Add($Change.Path) }
      }
    }
    $Result.Paths = @($Names | Sort-Object)
    if ($Names.Count -gt 0) { $Result.Profile = @('language','runtime','compiler','full')[$Rank] }
    $Result.Reason = 'complete_push_range'
  } catch { Write-Warning 'ci_policy: unresolved push range selects Full' }
  $Result
}

function Assert-HumIntegrationHealth([object[]] $Runs, [datetimeoffset] $Now, [string] $Repository) {
  # Latest scheduled or dispatched run owns health: never search backward past a failure.
  # Dispatch always runs Full (only pull_request classifies), so it mints the same evidence.
  $Candidates = @($Runs | Where-Object { $_.event -cin @('schedule','workflow_dispatch') -and $_.path -ceq '.github/workflows/validation.yml' } | Sort-Object { [datetimeoffset]$_.created_at } -Descending)
  if ($Candidates.Count -eq 0) { throw 'ci_health: scheduled or dispatched integration result is missing; Full bootstrap/recovery is required' }
  $Run = $Candidates[0]
  if ($Run.repository.full_name -cne $Repository -or $Run.head_branch -cne 'main' -or
      $Run.status -cne 'completed' -or $Run.conclusion -cne 'success' -or
      [long]$Run.id -le 0 -or [int]$Run.run_attempt -le 0) { throw 'ci_health: latest integration run is not an authenticated successful main result' }
  Assert-HumCiSha $Run.head_sha
  $Age = $Now - [datetimeoffset]$Run.created_at
  if ($Age.TotalHours -lt 0 -or $Age.TotalHours -gt 30) { throw 'ci_health: integration result is stale or future-dated' }
  $Run
}

function Assert-HumIntegrationJobs([object[]] $Jobs, [object] $Run) {
  foreach ($Platform in @('windows','ubuntu')) {
    $Name = '(?:\A| / )preflight \(' + $Platform + '-latest\)\z'
    $Matches = @($Jobs | Where-Object { $_.name -cmatch $Name })
    if ($Matches.Count -ne 1) { throw "ci_health: missing or duplicate $Platform job" }
    $Job = $Matches[0]
    if ($Job.run_id -ne $Run.id -or $Job.head_sha -cne $Run.head_sha -or $Job.status -cne 'completed' -or $Job.conclusion -cne 'success') { throw "ci_health: invalid $Platform job binding or outcome" }
    $Steps = @('Checkout','Verify integration identity','Run Hum preflight','Close full evidence transport','Confirm selected work completion')
    if ($Platform -ceq 'ubuntu') { $Steps += 'Run exhaustive canonical-seal evidence' }
    foreach ($Name in $Steps) {
      $Rows = @($Job.steps | Where-Object { $_.name -ceq $Name })
      if ($Rows.Count -ne 1 -or $Rows[0].status -cne 'completed' -or $Rows[0].conclusion -cne 'success') { throw "ci_health: required $Platform step did not succeed: $Name" }
    }
  }
}

if ($MyInvocation.InvocationName -ne '.') {
  $ErrorActionPreference = 'Stop'
  switch -CaseSensitive ($Mode) {
    'Select' {
      $Result = Get-HumCiSelection $RepositoryRoot $Base $Head $Integration
      foreach ($Name in @('Profile','Base','Head','Integration','Tree')) {
        $Line = "$($Name.ToLowerInvariant())=$($Result.$Name)"
        Write-Output $Line
        if ($env:GITHUB_OUTPUT) { $Line | Out-File -LiteralPath $env:GITHUB_OUTPUT -Encoding utf8 -Append }
      }
    }
    'Library' { throw 'ci_policy: dot-source for library use' }
    default { throw 'ci_policy: unknown operation' }
  }
}
