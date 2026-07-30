//! Private Work Order 11 fixed validation corpus.

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    use crate::CommandObservation;
    use crate::validation_session::{
        ObservationRequest, SessionMetrics, SourceSnapshot, ValidationPolicy, ValidationSession,
    };

    const REFERENCE_SOURCE: &str = "examples/reference_surface.hum";
    const BLOCKER_SOURCE: &str = "fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum";
    const BOUNDED_SOURCE: &str = "examples/probes/bounded_stdout.hum";
    const PRIVATE_FIELDS: [&str; 6] = [
        "occurrence_id",
        "cause_key",
        "semantic_owner",
        "owning_stage",
        "semantic_origin",
        "relationship_route",
    ];

    #[derive(Clone, Copy)]
    enum OutputExpectation {
        Json {
            exit: u8,
            schema: &'static str,
            required: &'static [&'static str],
        },
        CheckHuman,
        HumanReport {
            exit: u8,
            required: &'static [&'static str],
        },
        AllowedRuntime,
        DeniedRuntime,
        ExactRepeat {
            original: &'static str,
        },
    }

    struct ObservationCase {
        request: ObservationRequest,
        expectation: OutputExpectation,
        migrated_from_fast: bool,
    }

    #[derive(Default)]
    struct ConclusionLedger {
        passed: BTreeSet<&'static str>,
    }

    impl ConclusionLedger {
        fn pass(&mut self, id: &'static str) {
            assert!(self.passed.insert(id), "duplicate conclusion id: {id}");
        }
    }

    #[derive(Debug, Clone, Default)]
    struct WorkloadPhases {
        corpus_construction: Duration,
        logical_observations: Duration,
        product_process_corpus: Duration,
        parity_and_comparison: Duration,
        mutation_and_cleanup_isolation: Duration,
        orchestration_overhead: Duration,
    }

    #[derive(Debug)]
    struct WorkloadResult {
        policy: ValidationPolicy,
        wall: Duration,
        metrics: SessionMetrics,
        phases: WorkloadPhases,
        conclusions: BTreeSet<&'static str>,
        expected_conclusions: usize,
        product_invocations: u64,
        migrated_product_invocations: u64,
        sentinel_count: u64,
        stdout_parity: u64,
        stderr_parity: u64,
        exit_parity: u64,
        human_parity: u64,
        json_parity: u64,
        mutation_controls: u64,
        traceability_rows: usize,
        product_processes: Vec<ProductProcessRecord>,
        start_offset: Duration,
        end_offset: Duration,
    }

    #[derive(Debug, Clone)]
    struct ProductProcessRecord {
        policy: ValidationPolicy,
        attribution: &'static str,
        conclusion_id: &'static str,
        arguments: Vec<String>,
        exit_code: u8,
    }

    #[derive(Debug, Clone)]
    struct EquivalenceTuple {
        executable: String,
        toolchain: String,
        repository_commit: String,
        dirty_manifest_sha256: String,
        scoped_tree: String,
        complete_tree: String,
        working_directory: String,
        package: &'static str,
        manifest: String,
        target: &'static str,
        target_directory: String,
        features: &'static str,
        default_features: &'static str,
        profile: &'static str,
        environment: String,
        evidence_tier: &'static str,
        test_filter: &'static str,
        ignored_state: &'static str,
        harness: &'static str,
        source_fixture_identities_and_order: String,
        platform: String,
        external_adapters_and_authority: &'static str,
    }

    #[derive(Debug, Clone)]
    struct TraceabilityRow {
        conclusion_id: &'static str,
        old_label: &'static str,
        old_location: &'static str,
        old_command: String,
        tuple: EquivalenceTuple,
        ordered_inputs: Vec<&'static str>,
        positive_assertions: &'static str,
        negative_assertions: &'static str,
        output_channel_exit_relationship: &'static str,
        replacement_location: &'static str,
        retained_producer: &'static str,
        retained_transcript: &'static str,
        cache_policy: &'static str,
        sentinel: &'static str,
        equivalence: &'static str,
    }

    #[test]
    fn work_order_11_unit_1_fixed_slice_reference_optimized_pair_is_load_bearing() {
        let test_origin = Instant::now();
        let test_started_unix_us = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_micros();
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(
            std::env::current_dir().expect("current directory"),
            repo,
            "the fixed corpus requires the repository root as its exact working directory"
        );
        let product_exe = required_external_product(&repo);
        let metrics_path = required_external_metrics_path(&repo);
        let process_ledger_path = required_external_process_ledger_path(&repo);
        assert_reference_predicate_cache_sabotage();
        let traceability = traceability_ledger();

        let reference = run_workload(
            ValidationPolicy::Reference,
            &repo,
            &product_exe,
            &traceability,
            test_origin,
        );
        let optimized = run_workload(
            ValidationPolicy::Optimized,
            &repo,
            &product_exe,
            &traceability,
            test_origin,
        );

        assert_eq!(
            reference.conclusions, optimized.conclusions,
            "Reference and Optimized must produce the same ordered conclusion identities"
        );
        assert_eq!(
            reference.expected_conclusions,
            optimized.expected_conclusions
        );
        assert_eq!(
            reference.conclusions.len(),
            reference.expected_conclusions,
            "Reference conclusion ledger is incomplete"
        );
        assert_eq!(
            optimized.conclusions.len(),
            optimized.expected_conclusions,
            "Optimized conclusion ledger is incomplete"
        );
        assert_eq!(reference.metrics.distinct_source_identities, 3);
        assert_eq!(optimized.metrics.distinct_source_identities, 3);
        assert_eq!(optimized.metrics.source_reads, 3);
        assert_eq!(optimized.metrics.parses, 3);
        assert_eq!(optimized.metrics.initial_checks, 3);
        assert!(reference.metrics.source_reads > optimized.metrics.source_reads);
        assert!(reference.metrics.parses > optimized.metrics.parses);
        assert!(reference.metrics.initial_checks > optimized.metrics.initial_checks);
        assert!(
            reference.metrics.substantive_static_computations
                > optimized.metrics.substantive_static_computations
        );
        assert!(optimized.metrics.cache_hits >= 1);
        assert!(optimized.metrics.cache_reuses >= 1);
        assert_eq!(reference.metrics.cache_hits, 0);
        assert_eq!(reference.metrics.cache_reuses, 0);
        assert_eq!(optimized.metrics.runtime_requests, 2);
        assert_eq!(optimized.metrics.runtime_executions, 2);
        assert_eq!(optimized.metrics.runtime_result_cache_hits, 0);
        assert!(reference.product_invocations > optimized.product_invocations);
        assert!(optimized.product_invocations <= if cfg!(windows) { 7 } else { 6 });
        assert!(optimized.wall < reference.wall);
        let ratio = optimized.wall.as_secs_f64() / reference.wall.as_secs_f64();
        assert!(
            ratio < 0.95,
            "Unit 1 fixed-slice acceleration was not material: ratio={ratio:.6}"
        );

        let metrics = render_metrics_record(&reference, &optimized, ratio, test_started_unix_us);
        fs::write(&metrics_path, metrics.as_bytes())
            .unwrap_or_else(|error| panic!("failed to write external Unit 1 metrics: {error}"));
        write_slice_product_process_ledger(
            &process_ledger_path,
            &traceability,
            &reference,
            &optimized,
        );
    }

    fn assert_reference_predicate_cache_sabotage() {
        let mut session = ValidationSession::from_paths(
            ValidationPolicy::Reference,
            vec![PathBuf::from(REFERENCE_SOURCE)],
        )
        .expect("predicate cache sabotage session");
        let request = ObservationRequest::static_observation(
            "reference_predicate_cache_isolation_sabotage",
            ["resolve", "--format", "json", REFERENCE_SOURCE],
        );
        let observation = session
            .assert_reference_predicate_cache_isolation(&request)
            .expect("Reference must displace a deliberately stale predicate analysis");
        assert_eq!(observation.exit_code, 0);
        assert!(observation.stderr.is_empty());
        let stdout = std::str::from_utf8(&observation.stdout).expect("resolve JSON UTF-8");
        assert!(stdout.contains("\"schema\": \"hum.resolve.v0\""));
        assert_eq!(session.metrics().static_requests, 1);
        assert_eq!(session.metrics().substantive_static_computations, 1);
        assert_eq!(session.metrics().cache_hits, 0);
    }

    fn run_workload(
        policy: ValidationPolicy,
        repo: &Path,
        product_exe: &Path,
        traceability: &[TraceabilityRow],
        test_origin: Instant,
    ) -> WorkloadResult {
        let start_offset = test_origin.elapsed();
        let wall_start = Instant::now();
        let mut phases = WorkloadPhases::default();
        let construction_start = Instant::now();
        let mut reference =
            ValidationSession::from_paths(policy, vec![PathBuf::from(REFERENCE_SOURCE)])
                .expect("reference session");
        let mut blocker =
            ValidationSession::from_paths(policy, vec![PathBuf::from(BLOCKER_SOURCE)])
                .expect("blocker session");
        let mut bounded =
            ValidationSession::from_paths(policy, vec![PathBuf::from(BOUNDED_SOURCE)])
                .expect("bounded-output session");
        phases.corpus_construction = construction_start.elapsed();

        assert_eq!(reference.policy(), policy);
        assert_eq!(blocker.policy(), policy);
        assert_eq!(bounded.policy(), policy);
        let cases = all_cases();
        let expected_ids = expected_conclusions(&cases);
        let mut ledger = ConclusionLedger::default();
        let mut observations = BTreeMap::new();
        let observation_start = Instant::now();
        for case in &cases {
            let session = match fixed_source_argument(&case.request.arguments) {
                REFERENCE_SOURCE => &mut reference,
                BLOCKER_SOURCE => &mut blocker,
                BOUNDED_SOURCE => &mut bounded,
                other => panic!(
                    "observation {} escaped the fixed corpus: {other}",
                    case.request.conclusion_id
                ),
            };
            let observation = session
                .observe(&case.request)
                .unwrap_or_else(|error| panic!("{} failed: {error}", case.request.conclusion_id));
            assert_observation(case, &observation, &observations);
            ledger.pass(case.request.conclusion_id);
            observations.insert(case.request.conclusion_id, observation);
        }
        phases.logical_observations = observation_start.elapsed();

        let mut product_invocations = 0;
        let mut migrated_product_invocations = 0;
        let mut product_processes = Vec::new();
        let mut product_outputs = BTreeMap::new();
        let product_start = Instant::now();
        if policy == ValidationPolicy::Reference {
            for case in cases.iter().filter(|case| case.migrated_from_fast) {
                let product = run_product(repo, product_exe, &case.request.arguments);
                product_invocations += 1;
                migrated_product_invocations += 1;
                product_processes.push(ProductProcessRecord {
                    policy,
                    attribution: "migrated-reference-legacy-command",
                    conclusion_id: case.request.conclusion_id,
                    arguments: case.request.arguments.clone(),
                    exit_code: product.exit_code,
                });
                assert_exact_parity(
                    case.request.conclusion_id,
                    observations
                        .get(case.request.conclusion_id)
                        .expect("in-process observation"),
                    &product,
                );
                product_outputs.insert(case.request.conclusion_id, product);
            }
        }
        phases.product_process_corpus = product_start.elapsed();

        let parity_start = Instant::now();
        let sentinel_ids = [
            "reference_check_human",
            "reference_ir_readiness_json",
            "blocker_check_human",
            "blocker_ir_readiness_json",
            "bounded_run_allowed",
            "bounded_run_default_denied",
        ];
        let mut sentinel_count = 0;
        let mut stdout_parity = 0;
        let mut stderr_parity = 0;
        let mut exit_parity = 0;
        let mut human_parity = 0;
        let mut json_parity = 0;
        for id in sentinel_ids {
            let case = cases
                .iter()
                .find(|case| case.request.conclusion_id == id)
                .expect("sentinel case");
            let product = if let Some(existing) = product_outputs.get(id) {
                existing.clone()
            } else {
                product_invocations += 1;
                let product = run_product(repo, product_exe, &case.request.arguments);
                product_processes.push(ProductProcessRecord {
                    policy,
                    attribution: "retained-product-parity-sentinel",
                    conclusion_id: sentinel_conclusion_id(id),
                    arguments: case.request.arguments.clone(),
                    exit_code: product.exit_code,
                });
                product
            };
            assert_exact_parity(
                id,
                observations.get(id).expect("sentinel observation"),
                &product,
            );
            sentinel_count += 1;
            stdout_parity += 1;
            stderr_parity += 1;
            exit_parity += 1;
            if matches!(
                case.expectation,
                OutputExpectation::Json { .. } | OutputExpectation::ExactRepeat { .. }
            ) {
                json_parity += 1;
            } else {
                human_parity += 1;
            }
            ledger.pass(sentinel_conclusion_id(id));
        }
        if cfg!(windows) {
            let slash_arguments = vec![
                "run".to_string(),
                BOUNDED_SOURCE.replace('/', "\\"),
                "--allow".to_string(),
                "stdout.write".to_string(),
                "--args".to_string(),
                "hello".to_string(),
            ];
            let product = run_product(repo, product_exe, &slash_arguments);
            product_invocations += 1;
            product_processes.push(ProductProcessRecord {
                policy,
                attribution: "retained-windows-separator-sentinel",
                conclusion_id: "sentinel_bounded_windows_separator_identity",
                arguments: slash_arguments.clone(),
                exit_code: product.exit_code,
            });
            assert_exact_parity(
                "bounded_windows_separator_identity",
                observations
                    .get("bounded_run_allowed")
                    .expect("bounded allowed observation"),
                &product,
            );
            sentinel_count += 1;
            stdout_parity += 1;
            stderr_parity += 1;
            exit_parity += 1;
            human_parity += 1;
            ledger.pass("sentinel_bounded_windows_separator_identity");
        }
        phases.parity_and_comparison = parity_start.elapsed();

        let isolation_start = Instant::now();
        let mutation_controls = assert_isolation_controls(policy, &mut ledger);
        phases.mutation_and_cleanup_isolation = isolation_start.elapsed();

        reference
            .metrics()
            .assert_equations()
            .expect("reference source-session equations");
        blocker
            .metrics()
            .assert_equations()
            .expect("blocker source-session equations");
        bounded
            .metrics()
            .assert_equations()
            .expect("bounded source-session equations");
        let metrics = aggregate_metrics([
            reference.into_metrics(),
            blocker.into_metrics(),
            bounded.into_metrics(),
        ]);
        metrics
            .assert_equations()
            .expect("aggregate fixed-corpus equations");
        ledger.pass("session_count_equations");
        ledger.pass("runtime_requests_equal_executions");
        ledger.pass("runtime_result_cache_hits_zero");
        if policy == ValidationPolicy::Optimized {
            assert_eq!(metrics.source_reads, 3);
            assert_eq!(metrics.parses, 3);
            assert_eq!(metrics.initial_checks, 3);
            assert!(metrics.cache_hits >= 1);
        }
        ledger.pass("one_load_per_optimized_source_identity");
        ledger.pass("eligible_repetition_cache_hit");
        ledger.pass("traceability_ledger_complete");

        let expected_conclusions = expected_ids.len();
        assert_eq!(
            ledger.passed,
            expected_ids,
            "{} conclusion ledger mismatch",
            policy.name()
        );
        let wall = wall_start.elapsed();
        let end_offset = test_origin.elapsed();
        let attributed = phases
            .corpus_construction
            .saturating_add(phases.logical_observations)
            .saturating_add(phases.product_process_corpus)
            .saturating_add(phases.parity_and_comparison)
            .saturating_add(phases.mutation_and_cleanup_isolation);
        phases.orchestration_overhead = wall.saturating_sub(attributed);
        assert_phase_reconciliation(wall, &phases);

        WorkloadResult {
            policy,
            wall,
            metrics,
            phases,
            conclusions: ledger.passed,
            expected_conclusions,
            product_invocations,
            migrated_product_invocations,
            sentinel_count,
            stdout_parity,
            stderr_parity,
            exit_parity,
            human_parity,
            json_parity,
            mutation_controls,
            traceability_rows: traceability.len(),
            product_processes,
            start_offset,
            end_offset,
        }
    }

    fn fixed_source_argument(arguments: &[String]) -> &'static str {
        let mut matches = arguments.iter().filter_map(|argument| {
            let normalized = argument.replace('\\', "/");
            match normalized.as_str() {
                REFERENCE_SOURCE => Some(REFERENCE_SOURCE),
                BLOCKER_SOURCE => Some(BLOCKER_SOURCE),
                BOUNDED_SOURCE => Some(BOUNDED_SOURCE),
                _ => None,
            }
        });
        let source = matches
            .next()
            .expect("observation has one fixed-corpus source input");
        assert!(
            matches.next().is_none(),
            "observation has more than one fixed-corpus source input"
        );
        source
    }

    fn all_cases() -> Vec<ObservationCase> {
        let mut cases = Vec::new();
        cases.push(case(
            "reference_check_human",
            ["check", REFERENCE_SOURCE],
            OutputExpectation::CheckHuman,
            false,
        ));
        cases.push(json_case(
            "reference_check_json",
            "check",
            REFERENCE_SOURCE,
            0,
            "hum.check.v0",
            &["\"errors\": 0", "\"warnings\": 0"],
            false,
        ));
        for (id, command, schema, exit, required) in [
            (
                "reference_resolve_json",
                "resolve",
                "hum.resolve.v0",
                0,
                &["\"status\": \"checked_resolver_v0\""][..],
            ),
            (
                "reference_type_env_json",
                "type-env",
                "hum.type_env.v0",
                0,
                &["\"status\": \"type_environment_v0\""][..],
            ),
            (
                "reference_type_check_json",
                "type-check",
                "hum.type_check.v0",
                0,
                &["\"type_errors\": 0"][..],
            ),
            (
                "reference_core_preview_json",
                "core-preview",
                "hum.core_preview.v0",
                0,
                &["\"execution_ready\": 0"][..],
            ),
            (
                "reference_core_lower_json",
                "core-lower",
                "hum.core_lower.v0",
                0,
                &["\"ir_ready\": 0"][..],
            ),
            (
                "reference_core_verify_json",
                "core-verify",
                "hum.core_verify.v0",
                0,
                &["\"failed_checks\": 0"][..],
            ),
            (
                "reference_full_type_check_json",
                "full-type-check",
                "hum.full_type_check.v0",
                1,
                &["\"blocking_issues\""][..],
            ),
            (
                "reference_effect_check_json",
                "effect-check",
                "hum.effect_check.v0",
                1,
                &["\"blocking_issues\""][..],
            ),
            (
                "reference_ownership_check_json",
                "ownership-check",
                "hum.ownership_check.v0",
                1,
                &["\"blocking_issues\""][..],
            ),
            (
                "reference_resource_check_json",
                "resource-check",
                "hum.resource_check.v0",
                1,
                &["\"blocking_issues\""][..],
            ),
            (
                "reference_profile_check_json",
                "profile-check",
                "hum.profile_check.v0",
                1,
                &["\"blocking_issues\""][..],
            ),
            (
                "reference_ir_readiness_json",
                "ir-readiness",
                "hum.ir_readiness.v0",
                0,
                &["\"ready_for_ir\": 0"][..],
            ),
        ] {
            cases.push(json_case(
                id,
                command,
                REFERENCE_SOURCE,
                exit,
                schema,
                required,
                false,
            ));
        }

        cases.push(case(
            "blocker_check_human",
            ["check", BLOCKER_SOURCE],
            OutputExpectation::CheckHuman,
            false,
        ));
        cases.push(json_case(
            "blocker_check_json",
            "check",
            BLOCKER_SOURCE,
            0,
            "hum.check.v0",
            &["\"errors\": 0"],
            false,
        ));
        cases.push(json_case(
            "blocker_resolve_json",
            "resolve",
            BLOCKER_SOURCE,
            0,
            "hum.resolve.v0",
            &["\"resolver_errors\": 0"],
            false,
        ));
        for (command, schema) in [
            ("type-check", "hum.type_check.v0"),
            ("full-type-check", "hum.full_type_check.v0"),
            ("effect-check", "hum.effect_check.v0"),
            ("ownership-check", "hum.ownership_check.v0"),
            ("resource-check", "hum.resource_check.v0"),
            ("profile-check", "hum.profile_check.v0"),
        ] {
            let human_id = blocker_id(command, "human");
            cases.push(case(
                human_id,
                [command, BLOCKER_SOURCE],
                OutputExpectation::HumanReport {
                    exit: 1,
                    required: if command == "profile-check" {
                        &["blocked_by_resource_check_errors"]
                    } else {
                        &["H0605"]
                    },
                },
                true,
            ));
            let json_id = blocker_id(command, "json");
            cases.push(json_case(
                json_id,
                command,
                BLOCKER_SOURCE,
                1,
                schema,
                if command == "profile-check" {
                    &["blocked_by_resource_check_errors"]
                } else {
                    &["H0605"]
                },
                true,
            ));
        }
        cases.push(json_case(
            "blocker_ir_readiness_json",
            "ir-readiness",
            BLOCKER_SOURCE,
            0,
            "hum.ir_readiness.v0",
            &["blocked_by_type_errors", "\"ready_for_ir\": 0"],
            false,
        ));
        cases.push(case(
            "blocker_graph_json",
            ["graph", BLOCKER_SOURCE],
            OutputExpectation::Json {
                exit: 1,
                schema: "hum.semantic_graph.v0",
                required: &["\"code\": \"H0605\"", "\"pipeline_diagnostics\""],
            },
            true,
        ));

        cases.push(json_case(
            "bounded_check_json",
            "check",
            BOUNDED_SOURCE,
            0,
            "hum.check.v0",
            &["\"errors\": 0"],
            false,
        ));
        for (id, command, schema) in [
            ("bounded_resolve_json", "resolve", "hum.resolve.v0"),
            (
                "bounded_full_type_check_json",
                "full-type-check",
                "hum.full_type_check.v0",
            ),
            (
                "bounded_effect_check_json",
                "effect-check",
                "hum.effect_check.v0",
            ),
            (
                "bounded_ownership_check_json",
                "ownership-check",
                "hum.ownership_check.v0",
            ),
            (
                "bounded_resource_check_json",
                "resource-check",
                "hum.resource_check.v0",
            ),
            (
                "bounded_core_preview_json",
                "core-preview",
                "hum.core_preview.v0",
            ),
            ("bounded_core_lower_json", "core-lower", "hum.core_lower.v0"),
            (
                "bounded_core_verify_json",
                "core-verify",
                "hum.core_verify.v0",
            ),
        ] {
            cases.push(json_case(id, command, BOUNDED_SOURCE, 0, schema, &[], true));
        }
        cases.push(case(
            "bounded_graph_json",
            ["graph", BOUNDED_SOURCE],
            OutputExpectation::Json {
                exit: 0,
                schema: "hum.semantic_graph.v0",
                required: &["\"text\": \"stdout.write\""],
            },
            true,
        ));
        cases.push(case(
            "bounded_effect_check_repeat",
            ["effect-check", "--format", "json", BOUNDED_SOURCE],
            OutputExpectation::ExactRepeat {
                original: "bounded_effect_check_json",
            },
            false,
        ));
        cases.push(case(
            "bounded_run_allowed",
            [
                "run",
                BOUNDED_SOURCE,
                "--allow",
                "stdout.write",
                "--args",
                "hello",
            ],
            OutputExpectation::AllowedRuntime,
            true,
        ));
        cases.push(case(
            "bounded_run_default_denied",
            ["run", BOUNDED_SOURCE, "--args", "blocked"],
            OutputExpectation::DeniedRuntime,
            true,
        ));
        cases
    }

    fn case<const N: usize>(
        id: &'static str,
        arguments: [&'static str; N],
        expectation: OutputExpectation,
        migrated_from_fast: bool,
    ) -> ObservationCase {
        let fresh = matches!(
            expectation,
            OutputExpectation::AllowedRuntime | OutputExpectation::DeniedRuntime
        );
        let request = if fresh {
            ObservationRequest::fresh_observation(id, arguments)
        } else {
            ObservationRequest::static_observation(id, arguments)
        };
        ObservationCase {
            request,
            expectation,
            migrated_from_fast,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn json_case(
        id: &'static str,
        command: &'static str,
        source: &'static str,
        exit: u8,
        schema: &'static str,
        required: &'static [&'static str],
        migrated_from_fast: bool,
    ) -> ObservationCase {
        case(
            id,
            [command, "--format", "json", source],
            OutputExpectation::Json {
                exit,
                schema,
                required,
            },
            migrated_from_fast,
        )
    }

    fn blocker_id(command: &str, format: &str) -> &'static str {
        match (command, format) {
            ("type-check", "human") => "blocker_type_check_human",
            ("type-check", "json") => "blocker_type_check_json",
            ("full-type-check", "human") => "blocker_full_type_check_human",
            ("full-type-check", "json") => "blocker_full_type_check_json",
            ("effect-check", "human") => "blocker_effect_check_human",
            ("effect-check", "json") => "blocker_effect_check_json",
            ("ownership-check", "human") => "blocker_ownership_check_human",
            ("ownership-check", "json") => "blocker_ownership_check_json",
            ("resource-check", "human") => "blocker_resource_check_human",
            ("resource-check", "json") => "blocker_resource_check_json",
            ("profile-check", "human") => "blocker_profile_check_human",
            ("profile-check", "json") => "blocker_profile_check_json",
            _ => panic!("unmapped blocker observation: {command}/{format}"),
        }
    }

    fn assert_observation(
        case: &ObservationCase,
        observation: &CommandObservation,
        prior: &BTreeMap<&'static str, CommandObservation>,
    ) {
        match case.expectation {
            OutputExpectation::Json {
                exit,
                schema,
                required,
            } => {
                assert_eq!(
                    observation.exit_code,
                    exit,
                    "{} stdout={} stderr={}",
                    case.request.conclusion_id,
                    String::from_utf8_lossy(&observation.stdout),
                    String::from_utf8_lossy(&observation.stderr)
                );
                assert!(
                    observation.stderr.is_empty(),
                    "{} wrote unexpected stderr: {}",
                    case.request.conclusion_id,
                    String::from_utf8_lossy(&observation.stderr)
                );
                assert_json(&observation.stdout);
                let text = std::str::from_utf8(&observation.stdout).expect("JSON is UTF-8");
                assert!(
                    text.contains(&format!("\"schema\": \"{schema}\"")),
                    "{} lost schema {schema}",
                    case.request.conclusion_id
                );
                for needle in required {
                    assert!(
                        text.contains(needle),
                        "{} lost required assertion {needle:?}",
                        case.request.conclusion_id
                    );
                }
                if case.request.conclusion_id.starts_with("blocker_") {
                    assert_private_fields_absent(case.request.conclusion_id, text);
                }
            }
            OutputExpectation::CheckHuman => {
                assert_eq!(observation.exit_code, 0, "{}", case.request.conclusion_id);
                assert_eq!(
                    observation.stdout,
                    b"checked 1 file(s): 0 error(s), 0 warning(s)\n"
                );
                assert!(observation.stderr.is_empty());
            }
            OutputExpectation::HumanReport { exit, required } => {
                assert_eq!(
                    observation.exit_code, exit,
                    "{}",
                    case.request.conclusion_id
                );
                assert!(observation.stderr.is_empty());
                let text = std::str::from_utf8(&observation.stdout).expect("human output is UTF-8");
                assert!(text.ends_with('\n'));
                for needle in required {
                    assert!(
                        text.contains(needle),
                        "{} lost required assertion {needle:?}",
                        case.request.conclusion_id
                    );
                }
                if case.request.conclusion_id.starts_with("blocker_") {
                    assert_private_fields_absent(case.request.conclusion_id, text);
                }
            }
            OutputExpectation::AllowedRuntime => {
                assert_eq!(observation.exit_code, 0);
                assert_eq!(observation.stdout, b"hello");
                assert!(observation.stderr.is_empty());
            }
            OutputExpectation::DeniedRuntime => {
                assert_eq!(observation.exit_code, 1);
                assert!(observation.stdout.is_empty());
                let stderr =
                    std::str::from_utf8(&observation.stderr).expect("runtime stderr is UTF-8");
                for needle in [
                    "failure: AppError.output",
                    "caused by: OutputError.denied",
                    "while calling `stdout_write`",
                ] {
                    assert!(stderr.contains(needle), "missing {needle:?} from denial");
                }
                assert!(!stderr.contains("runtime trap"));
                assert!(stderr.ends_with('\n'));
            }
            OutputExpectation::ExactRepeat { original } => {
                assert_eq!(
                    observation,
                    prior.get(original).expect("original repeated observation"),
                    "{} did not return exact cached bytes",
                    case.request.conclusion_id
                );
            }
        }
    }

    fn assert_private_fields_absent(id: &str, text: &str) {
        for field in PRIVATE_FIELDS {
            assert!(!text.contains(field), "{id} leaked private field {field}");
        }
    }

    fn run_product(repo: &Path, product_exe: &Path, arguments: &[String]) -> CommandObservation {
        let output = Command::new(product_exe)
            .current_dir(repo)
            .args(arguments)
            .output()
            .unwrap_or_else(|error| panic!("failed to execute {}: {error}", product_exe.display()));
        let exit_code = output
            .status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .expect("Hum product process must return an exact u8 exit code");
        CommandObservation {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code,
        }
    }

    fn assert_exact_parity(
        id: &str,
        in_process: &CommandObservation,
        product: &CommandObservation,
    ) {
        assert_eq!(
            in_process.stdout, product.stdout,
            "{id} stdout differed by one or more bytes"
        );
        assert_eq!(
            in_process.stderr, product.stderr,
            "{id} stderr differed by one or more bytes"
        );
        assert_eq!(
            in_process.exit_code, product.exit_code,
            "{id} exit code differed"
        );
    }

    fn assert_isolation_controls(policy: ValidationPolicy, ledger: &mut ConclusionLedger) -> u64 {
        let original = ValidationSession::from_paths(
            ValidationPolicy::Optimized,
            vec![PathBuf::from(REFERENCE_SOURCE)],
        )
        .expect("isolation source");
        let snapshots = original.snapshots();
        let alpha = mutate_snapshot(&snapshots[0], b"show every", b"show Every");
        let beta = mutate_snapshot(&snapshots[0], b"show every", b"show  every");
        let alpha_result_first = observe_mutation(policy, alpha.clone());
        let beta_result_second = observe_mutation(policy, beta.clone());
        let beta_result_first = observe_mutation(policy, beta);
        let alpha_result_second = observe_mutation(policy, alpha);
        assert_eq!(alpha_result_first.0, alpha_result_second.0);
        assert_eq!(beta_result_first.0, beta_result_second.0);
        assert_ne!(alpha_result_first.1, original.identity().clone());
        assert_ne!(beta_result_first.1, original.identity().clone());
        assert_eq!(alpha_result_first.2.cache_hits, 0);
        assert_eq!(beta_result_first.2.cache_hits, 0);
        ledger.pass("changed_source_byte_cannot_hit_old_identity");
        ledger.pass("two_mutation_controls_are_order_independent");

        let forward = ValidationSession::from_snapshots(
            ValidationPolicy::Optimized,
            vec![
                original.snapshots()[0].clone(),
                SourceSnapshot {
                    path: PathBuf::from(BOUNDED_SOURCE),
                    bytes: fs::read(BOUNDED_SOURCE).expect("bounded source bytes"),
                },
            ],
        )
        .expect("forward identity");
        let reverse = ValidationSession::from_snapshots(
            ValidationPolicy::Optimized,
            vec![
                SourceSnapshot {
                    path: PathBuf::from(BOUNDED_SOURCE),
                    bytes: fs::read(BOUNDED_SOURCE).expect("bounded source bytes"),
                },
                original.snapshots()[0].clone(),
            ],
        )
        .expect("reverse identity");
        assert_ne!(forward.identity(), reverse.identity());
        assert_ne!(
            forward.identity().ordered_paths(),
            reverse.identity().ordered_paths()
        );
        ledger.pass("reversed_ordered_inputs_have_distinct_identity");

        let request = ObservationRequest::static_observation(
            "failure_cleanup_probe",
            ["effect-check", "--format", "json", BOUNDED_SOURCE],
        );
        let failed_attempt: Result<(), &'static str> = {
            let mut session = ValidationSession::from_paths(
                ValidationPolicy::Optimized,
                vec![PathBuf::from(BOUNDED_SOURCE)],
            )
            .expect("failure cleanup first session");
            let _ = session.observe(&request).expect("failure cleanup observe");
            Err("intentional bounded failure after cache construction")
        };
        assert!(failed_attempt.is_err());
        let mut clean = ValidationSession::from_paths(
            ValidationPolicy::Optimized,
            vec![PathBuf::from(BOUNDED_SOURCE)],
        )
        .expect("failure cleanup next session");
        let clean_observation = clean.observe(&request).expect("clean observation");
        assert_eq!(clean.metrics().cache_hits, 0);
        assert_eq!(clean.metrics().cache_misses, 1);
        assert_json(&clean_observation.stdout);
        ledger.pass("failure_cleanup_cannot_contaminate_next_case");
        4
    }

    fn observe_mutation(
        policy: ValidationPolicy,
        snapshot: SourceSnapshot,
    ) -> (
        CommandObservation,
        crate::validation_session::SessionIdentity,
        SessionMetrics,
    ) {
        let mut session =
            ValidationSession::from_snapshots(policy, vec![snapshot]).expect("mutation session");
        let request = ObservationRequest::static_observation(
            "source_identity_mutation_control",
            ["check", "--format", "json", REFERENCE_SOURCE],
        );
        let result = session.observe(&request).expect("mutation observation");
        let identity = session.identity().clone();
        let metrics = session.into_metrics();
        (result, identity, metrics)
    }

    fn mutate_snapshot(
        source: &SourceSnapshot,
        needle: &[u8],
        replacement: &[u8],
    ) -> SourceSnapshot {
        let offset = source
            .bytes
            .windows(needle.len())
            .position(|window| window == needle)
            .expect("mutation needle");
        let mut bytes =
            Vec::with_capacity(source.bytes.len() + replacement.len().saturating_sub(needle.len()));
        bytes.extend_from_slice(&source.bytes[..offset]);
        bytes.extend_from_slice(replacement);
        bytes.extend_from_slice(&source.bytes[offset + needle.len()..]);
        SourceSnapshot {
            path: source.path.clone(),
            bytes,
        }
    }

    fn aggregate_metrics<const N: usize>(items: [SessionMetrics; N]) -> SessionMetrics {
        let mut total = SessionMetrics::default();
        for item in items {
            total.distinct_source_identities += item.distinct_source_identities;
            total.source_reads += item.source_reads;
            total.parses += item.parses;
            total.initial_checks += item.initial_checks;
            total.static_requests += item.static_requests;
            total.cacheable_requests += item.cacheable_requests;
            total.non_cacheable_requests += item.non_cacheable_requests;
            total.substantive_static_computations += item.substantive_static_computations;
            total.cache_lookups += item.cache_lookups;
            total.cache_hits += item.cache_hits;
            total.cache_misses += item.cache_misses;
            total.cache_entries_constructed += item.cache_entries_constructed;
            total.cache_reuses += item.cache_reuses;
            total.runtime_requests += item.runtime_requests;
            total.runtime_executions += item.runtime_executions;
            total.runtime_result_cache_hits += item.runtime_result_cache_hits;
            total.phases.source_read += item.phases.source_read;
            total.phases.parse += item.phases.parse;
            total.phases.initial_check += item.phases.initial_check;
            total.phases.substantive_static_analysis += item.phases.substantive_static_analysis;
            total.phases.cache_lookup += item.phases.cache_lookup;
            total.phases.cache_hit += item.phases.cache_hit;
            total.phases.cache_miss += item.phases.cache_miss;
            total.phases.cache_entry_construction += item.phases.cache_entry_construction;
            total.phases.cache_reuse += item.phases.cache_reuse;
            total.phases.fresh_runtime_execution += item.phases.fresh_runtime_execution;
        }
        total
    }

    fn expected_conclusions(cases: &[ObservationCase]) -> BTreeSet<&'static str> {
        let mut expected = cases
            .iter()
            .map(|case| case.request.conclusion_id)
            .collect::<BTreeSet<_>>();
        for id in [
            "sentinel_reference_check_human",
            "sentinel_reference_ir_readiness_json",
            "sentinel_blocker_check_human",
            "sentinel_blocker_ir_readiness_json",
            "sentinel_bounded_run_allowed",
            "sentinel_bounded_run_default_denied",
            "changed_source_byte_cannot_hit_old_identity",
            "two_mutation_controls_are_order_independent",
            "reversed_ordered_inputs_have_distinct_identity",
            "failure_cleanup_cannot_contaminate_next_case",
            "session_count_equations",
            "runtime_requests_equal_executions",
            "runtime_result_cache_hits_zero",
            "one_load_per_optimized_source_identity",
            "eligible_repetition_cache_hit",
            "traceability_ledger_complete",
        ] {
            assert!(expected.insert(id), "duplicate expected conclusion: {id}");
        }
        if cfg!(windows) {
            assert!(expected.insert("sentinel_bounded_windows_separator_identity"));
        }
        expected
    }

    fn sentinel_conclusion_id(id: &str) -> &'static str {
        match id {
            "reference_check_human" => "sentinel_reference_check_human",
            "reference_ir_readiness_json" => "sentinel_reference_ir_readiness_json",
            "blocker_check_human" => "sentinel_blocker_check_human",
            "blocker_ir_readiness_json" => "sentinel_blocker_ir_readiness_json",
            "bounded_run_allowed" => "sentinel_bounded_run_allowed",
            "bounded_run_default_denied" => "sentinel_bounded_run_default_denied",
            _ => panic!("unmapped sentinel conclusion: {id}"),
        }
    }

    fn required_evidence_binding(name: &str) -> String {
        std::env::var(name)
            .unwrap_or_else(|_| panic!("{name} must bind the manifest-qualified Fast producer"))
    }

    fn migrated_command_tuple(
        ordered_inputs: &[&str],
        external_adapters_and_authority: &'static str,
    ) -> EquivalenceTuple {
        let repository_commit = required_evidence_binding("HUM_UNIT1_BASE");
        let dirty_manifest_sha256 = required_evidence_binding("HUM_UNIT1_MANIFEST_SHA256");
        let scoped_tree = required_evidence_binding("HUM_UNIT1_SCOPED_TREE");
        let complete_tree = required_evidence_binding("HUM_UNIT1_COMPLETE_TREE");
        let toolchain = required_evidence_binding("HUM_UNIT1_TOOLCHAIN_IDENTITY");
        let environment = required_evidence_binding("HUM_UNIT1_ENVIRONMENT_IDENTITY");
        let target_directory = required_evidence_binding("HUM_UNIT1_TARGET_DIRECTORY");
        let actor = required_evidence_binding("HUM_UNIT1_ACTOR");
        let executable = required_evidence_binding("HUM_UNIT1_PRODUCT_EXE");
        let working_directory = std::env::current_dir()
            .expect("traceability working directory")
            .display()
            .to_string();
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("Cargo.toml")
            .display()
            .to_string();
        assert_eq!(
            repository_commit,
            "15d502ecd95b563b44db9c3c7c3a5b5034fbe61f"
        );
        assert_eq!(dirty_manifest_sha256.len(), 64);
        assert_eq!(scoped_tree.len(), 40);
        assert_eq!(complete_tree.len(), 40);
        assert!(matches!(actor.as_str(), "implementer" | "reviewer"));
        let source_fixture_identities_and_order = format!(
            "base={repository_commit};manifest={dirty_manifest_sha256};scoped_tree={scoped_tree};complete_tree={complete_tree};ordered_inputs={}",
            ordered_inputs.join(",")
        );
        EquivalenceTuple {
            executable,
            toolchain,
            repository_commit: repository_commit.clone(),
            dirty_manifest_sha256,
            scoped_tree: scoped_tree.clone(),
            complete_tree,
            working_directory,
            package: "hum-lang",
            manifest,
            target: "default-package-debug-hum-binary",
            target_directory,
            features: "default",
            default_features: "enabled",
            profile: "dev",
            environment: format!("{environment};actor={actor}"),
            evidence_tier: "fast",
            test_filter: "n/a",
            ignored_state: "n/a",
            harness: "direct native product process",
            source_fixture_identities_and_order,
            platform: format!(
                "os={};arch={}",
                std::env::consts::OS,
                std::env::consts::ARCH
            ),
            external_adapters_and_authority,
        }
    }

    fn traceability_ledger() -> Vec<TraceabilityRow> {
        let mut rows = Vec::new();
        for (command, format, id) in [
            ("type-check", "human", "blocker_type_check_human"),
            ("type-check", "json", "blocker_type_check_json"),
            ("full-type-check", "human", "blocker_full_type_check_human"),
            ("full-type-check", "json", "blocker_full_type_check_json"),
            ("effect-check", "human", "blocker_effect_check_human"),
            ("effect-check", "json", "blocker_effect_check_json"),
            ("ownership-check", "human", "blocker_ownership_check_human"),
            ("ownership-check", "json", "blocker_ownership_check_json"),
            ("resource-check", "human", "blocker_resource_check_human"),
            ("resource-check", "json", "blocker_resource_check_json"),
            ("profile-check", "human", "blocker_profile_check_human"),
            ("profile-check", "json", "blocker_profile_check_json"),
        ] {
            let arguments = if format == "json" {
                format!("{command} --format json {BLOCKER_SOURCE}")
            } else {
                format!("{command} {BLOCKER_SOURCE}")
            };
            rows.push(TraceabilityRow {
                conclusion_id: id,
                old_label: "Session AP H0605 blocker chain <stage> <format>",
                old_location: "tools/check_all.ps1@15d502e:1231-1244",
                old_command: format!("target/debug/hum {arguments}"),
                tuple: migrated_command_tuple(&[BLOCKER_SOURCE], "no external adapters or runtime authority"),
                ordered_inputs: vec![BLOCKER_SOURCE],
                positive_assertions: "exit=1; JSON parses when selected; H0605/blocker projection remains stage-owned",
                negative_assertions: "no occurrence_id/cause_key/semantic_owner/owning_stage/semantic_origin/relationship_route",
                output_channel_exit_relationship: "stdout carries the asserted human or JSON report; stderr is empty; exit=1",
                replacement_location: "src/validation_corpus.rs::work_order_11_unit_1_fixed_slice_reference_optimized_pair_is_load_bearing::<conclusion_id>",
                retained_producer: "shared in-process loaded-program observation plus Reference product parity",
                retained_transcript: "slice-metrics.json and slice-product-process-ledger.tsv",
                cache_policy: "eligible exact immutable static observation",
                sentinel: "failing check and IR sentinels bind source/process channels",
                equivalence: "same executable semantics, cwd, source bytes, ordered input, command, format, exit assertion, blocker assertion, and required absences",
            });
        }
        rows.push(TraceabilityRow {
            conclusion_id: "blocker_graph_json",
            old_label: "Session AP authoritative graph projection fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum",
            old_location: "tools/check_all.ps1@15d502e:1259-1262",
            old_command: format!("target/debug/hum graph {BLOCKER_SOURCE}"),
            tuple: migrated_command_tuple(&[BLOCKER_SOURCE], "no external adapters or runtime authority"),
            ordered_inputs: vec![BLOCKER_SOURCE],
            positive_assertions: "exit=1; complete JSON parses; H0605 pipeline diagnostic remains present",
            negative_assertions: "no occurrence_id or cause_key; stderr empty",
            output_channel_exit_relationship: "stdout carries graph JSON; stderr is empty; exit=1",
            replacement_location: "src/validation_corpus.rs::work_order_11_unit_1_fixed_slice_reference_optimized_pair_is_load_bearing::blocker_graph_json",
            retained_producer: "shared in-process loaded-program observation plus Reference product parity",
            retained_transcript: "slice-metrics.json and slice-product-process-ledger.tsv",
            cache_policy: "eligible exact immutable static observation",
            sentinel: "failing check and IR sentinels bind source/process channels",
            equivalence: "same graph command, cwd, source identity, JSON validity, exit, diagnostic presence, and stronger complete private-field absence",
        });

        for (id, command) in [
            ("bounded_resolve_json", "resolve --format json"),
            (
                "bounded_full_type_check_json",
                "full-type-check --format json",
            ),
            ("bounded_effect_check_json", "effect-check --format json"),
            (
                "bounded_ownership_check_json",
                "ownership-check --format json",
            ),
            (
                "bounded_resource_check_json",
                "resource-check --format json",
            ),
            ("bounded_core_preview_json", "core-preview --format json"),
            ("bounded_core_lower_json", "core-lower --format json"),
            ("bounded_core_verify_json", "core-verify --format json"),
        ] {
            rows.push(TraceabilityRow {
                conclusion_id: id,
                old_label: "Session Z <command> positive",
                old_location: "tools/check_all.ps1@15d502e:2436-2439",
                old_command: format!("target/debug/hum {command} {BOUNDED_SOURCE}"),
                tuple: migrated_command_tuple(&[BOUNDED_SOURCE], "no external adapters or runtime authority"),
                ordered_inputs: vec![BOUNDED_SOURCE],
                positive_assertions: "exit=0; complete JSON parses; exact command schema remains present",
                negative_assertions: "stderr empty; no private diagnostic identity fields",
                output_channel_exit_relationship: "stdout carries JSON; stderr is empty; exit=0",
                replacement_location: "src/validation_corpus.rs::work_order_11_unit_1_fixed_slice_reference_optimized_pair_is_load_bearing::<conclusion_id>",
                retained_producer: "shared in-process loaded-program observation plus Reference product parity",
                retained_transcript: "slice-metrics.json and slice-product-process-ledger.tsv",
                cache_policy: "eligible exact immutable static observation",
                sentinel: "allowed/default-denied runtime sentinels bind the same loaded source",
                equivalence: "same command, cwd, source bytes, format, success, JSON validity, and stronger channel/private-field assertions",
            });
        }
        rows.push(TraceabilityRow {
            conclusion_id: "bounded_graph_json",
            old_label: "Session Z graph positive",
            old_location: "tools/check_all.ps1@15d502e:2440-2441",
            old_command: format!("target/debug/hum graph {BOUNDED_SOURCE}"),
            tuple: migrated_command_tuple(&[BOUNDED_SOURCE], "no external adapters or runtime authority"),
            ordered_inputs: vec![BOUNDED_SOURCE],
            positive_assertions: "exit=0; complete JSON parses; semantic graph schema and stdout.write capability remain",
            negative_assertions: "stderr empty; no private diagnostic identity fields",
            output_channel_exit_relationship: "stdout carries graph JSON; stderr is empty; exit=0",
            replacement_location: "src/validation_corpus.rs::work_order_11_unit_1_fixed_slice_reference_optimized_pair_is_load_bearing::bounded_graph_json",
            retained_producer: "shared in-process loaded-program observation plus Reference product parity",
            retained_transcript: "slice-metrics.json and slice-product-process-ledger.tsv",
            cache_policy: "eligible exact immutable static observation",
            sentinel: "allowed/default-denied runtime sentinels bind the same loaded source",
            equivalence: "same graph command and fixture with stronger exit, channel, schema, capability, and absence assertions",
        });
        for (id, label, command, cache_policy, sentinel) in [
            (
                "bounded_run_allowed",
                "run Session Z exact allow",
                format!("target/debug/hum run {BOUNDED_SOURCE} --allow stdout.write --args hello"),
                "mandatory freshness; runtime state and adapters never cached",
                "exact allowed-run product sentinel",
            ),
            (
                "bounded_run_default_denied",
                "run Session Z default deny",
                format!("target/debug/hum run {BOUNDED_SOURCE} --args blocked"),
                "mandatory freshness; runtime state and authority decision never cached",
                "exact default-denied product sentinel",
            ),
        ] {
            rows.push(TraceabilityRow {
                conclusion_id: id,
                old_label: label,
                old_location: "tools/check_all.ps1@15d502e:2442-2457",
                old_command: command,
                tuple: migrated_command_tuple(&[BOUNDED_SOURCE], "fresh output, replay, file-locality, file-read, grant, and authority adapters"),
                ordered_inputs: vec![BOUNDED_SOURCE],
                positive_assertions: "exact stdout bytes, exact stderr bytes, exact exit; typed causal denial fields when denied",
                negative_assertions: "no extra newline/output channel; no runtime trap; zero runtime-result cache hits",
                output_channel_exit_relationship: "allowed: stdout=hello, stderr empty, exit=0; denied: stdout empty, typed causal stderr, exit=1",
                replacement_location: "src/validation_corpus.rs::work_order_11_unit_1_fixed_slice_reference_optimized_pair_is_load_bearing::<conclusion_id>",
                retained_producer: "fresh shared runtime observation plus exact real-product sentinel",
                retained_transcript: "slice-metrics.json and slice-product-process-ledger.tsv",
                cache_policy,
                sentinel,
                equivalence: "same product behavior and stronger raw byte/channel/exit comparison through one shared renderer",
            });
        }
        if cfg!(windows) {
            rows.push(TraceabilityRow {
                conclusion_id: "sentinel_bounded_windows_separator_identity",
                old_label: "run Session Z Windows separator identity",
                old_location: "tools/check_all.ps1@15d502e:2444-2448",
                old_command: r"target/debug/hum run examples\probes\bounded_stdout.hum --allow stdout.write --args hello".to_string(),
                tuple: migrated_command_tuple(&[r"examples\probes\bounded_stdout.hum"], "fresh Windows path, output, replay, file-locality, file-read, grant, and authority adapters"),
                ordered_inputs: vec![r"examples\probes\bounded_stdout.hum"],
                positive_assertions: "slash and backslash product spellings have exact equal stdout, stderr, and exit",
                negative_assertions: "no path-spelling policy split; no extra channel bytes",
                output_channel_exit_relationship: "stdout=hello; stderr empty; exit=0 for both path spellings",
                replacement_location: "src/validation_corpus.rs::work_order_11_unit_1_fixed_slice_reference_optimized_pair_is_load_bearing::bounded_windows_separator_identity",
                retained_producer: "fresh Windows exact real-product sentinel",
                retained_transcript: "slice-metrics.json and slice-product-process-ledger.tsv",
                cache_policy: "mandatory fresh product process",
                sentinel: "Windows source-identity product sentinel",
                equivalence: "same Windows-only path spelling, command, grant, argument, and exact three-part comparison",
            });
        }
        assert_traceability_rows(&rows);
        rows
    }

    fn assert_traceability_rows(rows: &[TraceabilityRow]) {
        let mut ids = BTreeSet::new();
        let mut tuple_bindings = BTreeSet::new();
        for row in rows {
            assert!(
                ids.insert(row.conclusion_id),
                "duplicate traceability conclusion: {}",
                row.conclusion_id
            );
            assert!(!row.old_label.is_empty());
            assert!(row.old_location.contains("@15d502e:"));
            assert!(row.old_command.starts_with("target/debug/hum "));
            let tuple = &row.tuple;
            assert!(Path::new(&tuple.executable).is_absolute());
            assert!(Path::new(&tuple.executable).is_file());
            assert!(!tuple.toolchain.is_empty());
            assert_eq!(
                tuple.repository_commit,
                "15d502ecd95b563b44db9c3c7c3a5b5034fbe61f"
            );
            assert!(
                tuple.dirty_manifest_sha256.len() == 64
                    && tuple
                        .dirty_manifest_sha256
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
            for tree in [&tuple.scoped_tree, &tuple.complete_tree] {
                assert!(
                    tree.len() == 40
                        && tree
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                );
            }
            assert_eq!(
                Path::new(&tuple.working_directory),
                std::env::current_dir()
                    .expect("traceability current directory")
                    .as_path()
            );
            assert_eq!(tuple.package, "hum-lang");
            assert_eq!(
                Path::new(&tuple.manifest),
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("Cargo.toml")
                    .as_path()
            );
            assert!(Path::new(&tuple.manifest).is_file());
            assert_eq!(tuple.target, "default-package-debug-hum-binary");
            assert!(Path::new(&tuple.target_directory).is_absolute());
            assert_eq!(tuple.features, "default");
            assert_eq!(tuple.default_features, "enabled");
            assert_eq!(tuple.profile, "dev");
            assert!(tuple.environment.starts_with("sha256:"));
            assert!(
                tuple.environment.contains("actor=implementer")
                    || tuple.environment.contains("actor=reviewer")
            );
            assert_eq!(tuple.evidence_tier, "fast");
            assert_eq!(tuple.test_filter, "n/a");
            assert_eq!(tuple.ignored_state, "n/a");
            assert_eq!(tuple.harness, "direct native product process");
            assert!(tuple.platform.contains("os="));
            assert!(tuple.platform.contains(";arch="));
            assert!(!tuple.external_adapters_and_authority.is_empty());
            assert_eq!(row.ordered_inputs.len(), 1);
            let ordered_inputs = row.ordered_inputs.join(",");
            assert_eq!(
                tuple
                    .source_fixture_identities_and_order
                    .split("ordered_inputs=")
                    .nth(1),
                Some(ordered_inputs.as_str())
            );
            assert!(!row.positive_assertions.is_empty());
            assert!(!row.negative_assertions.is_empty());
            assert!(!row.output_channel_exit_relationship.is_empty());
            assert!(
                row.replacement_location
                    .starts_with("src/validation_corpus.rs::")
            );
            assert!(!row.retained_producer.is_empty());
            assert_eq!(
                row.retained_transcript,
                "slice-metrics.json and slice-product-process-ledger.tsv"
            );
            assert!(!row.cache_policy.is_empty());
            assert!(!row.sentinel.is_empty());
            assert!(row.equivalence.contains("same"));
            tuple_bindings.insert(format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                tuple.executable,
                tuple.toolchain,
                tuple.repository_commit,
                tuple.dirty_manifest_sha256,
                tuple.scoped_tree,
                tuple.complete_tree,
                tuple.working_directory,
                tuple.package,
                tuple.manifest,
                tuple.target,
                tuple.target_directory,
                tuple.features,
                tuple.default_features,
                tuple.profile,
                tuple.environment,
                tuple.evidence_tier,
                tuple.harness,
                tuple.platform,
            ));
        }
        assert_eq!(
            tuple_bindings.len(),
            1,
            "migrated command equivalence tuples conflict"
        );
        let mut expected_ids = all_cases()
            .into_iter()
            .filter(|case| case.migrated_from_fast)
            .map(|case| case.request.conclusion_id)
            .collect::<BTreeSet<_>>();
        if cfg!(windows) {
            assert!(expected_ids.insert("sentinel_bounded_windows_separator_identity"));
        }
        assert_eq!(
            ids, expected_ids,
            "traceability must map every migrated legacy command exactly once and no others"
        );
    }

    fn assert_phase_reconciliation(wall: Duration, phases: &WorkloadPhases) {
        let children = phases
            .corpus_construction
            .saturating_add(phases.logical_observations)
            .saturating_add(phases.product_process_corpus)
            .saturating_add(phases.parity_and_comparison)
            .saturating_add(phases.mutation_and_cleanup_isolation)
            .saturating_add(phases.orchestration_overhead);
        let error = wall.abs_diff(children);
        let tolerance = Duration::from_millis(10).max(wall.div_f64(10_000.0));
        assert!(
            error <= tolerance,
            "fixed-slice phase reconciliation error {:?} exceeded {:?}",
            error,
            tolerance
        );
    }

    fn render_metrics_record(
        reference: &WorkloadResult,
        optimized: &WorkloadResult,
        ratio: f64,
        test_started_unix_us: u128,
    ) -> String {
        assert_eq!(reference.policy, ValidationPolicy::Reference);
        assert_eq!(optimized.policy, ValidationPolicy::Optimized);
        let delta = reference.wall.saturating_sub(optimized.wall).as_micros();
        let mut remaining = delta;
        let product = measured_attribution(
            reference
                .phases
                .product_process_corpus
                .saturating_sub(optimized.phases.product_process_corpus)
                .as_micros(),
            &mut remaining,
        );
        let front_end = measured_attribution(
            reference
                .metrics
                .phases
                .source_read
                .saturating_add(reference.metrics.phases.parse)
                .saturating_add(reference.metrics.phases.initial_check)
                .saturating_sub(
                    optimized
                        .metrics
                        .phases
                        .source_read
                        .saturating_add(optimized.metrics.phases.parse)
                        .saturating_add(optimized.metrics.phases.initial_check),
                )
                .as_micros(),
            &mut remaining,
        );
        let static_reuse = measured_attribution(
            reference
                .metrics
                .phases
                .substantive_static_analysis
                .saturating_sub(optimized.metrics.phases.substantive_static_analysis)
                .as_micros(),
            &mut remaining,
        );
        let orchestration = remaining;
        let conclusions = optimized
            .conclusions
            .iter()
            .map(|id| format!("\"{id}\""))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            concat!(
                "{{",
                "\"record\":\"hum-workorder11-unit1-metrics-v1\",",
                "\"platform\":\"{}\",",
                "\"ratio\":{:.6},",
                "\"timeline\":{{",
                "\"test_started_unix_us\":{},",
                "\"reference_start_us\":{},\"reference_end_us\":{},",
                "\"optimized_start_us\":{},\"optimized_end_us\":{}",
                "}},",
                "\"reference\":{},",
                "\"optimized\":{},",
                "\"delta_attribution_us\":{{",
                "\"equivalent_product_process_deduplication\":{},",
                "\"source_read_parse_initial_check\":{},",
                "\"substantive_static_reuse\":{},",
                "\"assertion_manifest_orchestration\":{}",
                "}},",
                "\"conclusions\":{{",
                "\"expected\":{},\"reference_produced\":{},\"optimized_produced\":{},",
                "\"passed\":{},\"failed\":0,\"ids\":[{}]",
                "}}",
                "}}\n"
            ),
            std::env::consts::OS,
            ratio,
            test_started_unix_us,
            reference.start_offset.as_micros(),
            reference.end_offset.as_micros(),
            optimized.start_offset.as_micros(),
            optimized.end_offset.as_micros(),
            render_workload_metrics(reference),
            render_workload_metrics(optimized),
            product,
            front_end,
            static_reuse,
            orchestration,
            optimized.expected_conclusions,
            reference.conclusions.len(),
            optimized.conclusions.len(),
            optimized.conclusions.len(),
            conclusions,
        )
    }

    fn render_workload_metrics(result: &WorkloadResult) -> String {
        let metrics = &result.metrics;
        format!(
            concat!(
                "{{",
                "\"policy\":\"{}\",",
                "\"wall_us\":{},",
                "\"product_invocations\":{},",
                "\"migrated_product_invocations\":{},",
                "\"source_identities\":{},\"source_reads\":{},\"parses\":{},\"initial_checks\":{},",
                "\"static_requests\":{},\"cacheable_requests\":{},\"non_cacheable_requests\":{},",
                "\"substantive_computations\":{},\"cache_lookups\":{},\"cache_hits\":{},",
                "\"cache_misses\":{},\"cache_entries_constructed\":{},\"cache_reuses\":{},",
                "\"runtime_requests\":{},\"runtime_executions\":{},\"runtime_result_cache_hits\":{},",
                "\"sentinels\":{},\"stdout_parity\":{},\"stderr_parity\":{},\"exit_parity\":{},",
                "\"human_parity\":{},\"json_parity\":{},\"mutation_controls\":{},",
                "\"traceability_rows\":{},",
                "\"phases\":{{",
                "\"readiness_setup_us\":0,",
                "\"manifest_binding_us\":0,",
                "\"build_prebuild_us\":0,",
                "\"root_listing_us\":0,",
                "\"root_execution_us\":0,",
                "\"package_manifest_tests_us\":0,",
                "\"selector_verification_us\":0,",
                "\"document_readiness_us\":0,",
                "\"remaining_product_process_corpus_us\":{},",
                "\"reusable_corpus_construction_us\":{},",
                "\"source_reads_us\":{},\"parse_us\":{},\"initial_check_us\":{},",
                "\"substantive_static_analysis_us\":{},",
                "\"cache_lookup_us\":{},\"cache_hit_us\":{},\"cache_miss_us\":{},",
                "\"cache_entry_construction_us\":{},\"cache_reuse_us\":{},",
                "\"fresh_runtime_execution_us\":{},",
                "\"parity_corruption_isolation_us\":{},",
                "\"transcript_finalization_us\":0,\"cleanup_us\":0,",
                "\"orchestration_overhead_us\":{}",
                "}}",
                "}}"
            ),
            result.policy.name(),
            result.wall.as_micros(),
            result.product_invocations,
            result.migrated_product_invocations,
            metrics.distinct_source_identities,
            metrics.source_reads,
            metrics.parses,
            metrics.initial_checks,
            metrics.static_requests,
            metrics.cacheable_requests,
            metrics.non_cacheable_requests,
            metrics.substantive_static_computations,
            metrics.cache_lookups,
            metrics.cache_hits,
            metrics.cache_misses,
            metrics.cache_entries_constructed,
            metrics.cache_reuses,
            metrics.runtime_requests,
            metrics.runtime_executions,
            metrics.runtime_result_cache_hits,
            result.sentinel_count,
            result.stdout_parity,
            result.stderr_parity,
            result.exit_parity,
            result.human_parity,
            result.json_parity,
            result.mutation_controls,
            result.traceability_rows,
            result.phases.product_process_corpus.as_micros(),
            result.phases.corpus_construction.as_micros(),
            metrics.phases.source_read.as_micros(),
            metrics.phases.parse.as_micros(),
            metrics.phases.initial_check.as_micros(),
            metrics.phases.substantive_static_analysis.as_micros(),
            metrics.phases.cache_lookup.as_micros(),
            metrics.phases.cache_hit.as_micros(),
            metrics.phases.cache_miss.as_micros(),
            metrics.phases.cache_entry_construction.as_micros(),
            metrics.phases.cache_reuse.as_micros(),
            metrics.phases.fresh_runtime_execution.as_micros(),
            result
                .phases
                .parity_and_comparison
                .saturating_add(result.phases.mutation_and_cleanup_isolation)
                .as_micros(),
            result.phases.orchestration_overhead.as_micros(),
        )
    }

    fn measured_attribution(measured: u128, remaining: &mut u128) -> u128 {
        let attributed = measured.min(*remaining);
        *remaining -= attributed;
        attributed
    }

    fn write_slice_product_process_ledger(
        path: &Path,
        traceability: &[TraceabilityRow],
        reference: &WorkloadResult,
        optimized: &WorkloadResult,
    ) {
        assert_eq!(
            reference.product_processes.len() as u64,
            reference.product_invocations
        );
        assert_eq!(
            optimized.product_processes.len() as u64,
            optimized.product_invocations
        );
        let migrated_ids = reference
            .product_processes
            .iter()
            .filter(|record| record.attribution == "migrated-reference-legacy-command")
            .map(|record| record.conclusion_id)
            .collect::<BTreeSet<_>>();
        let traceability_ids = traceability
            .iter()
            .filter(|row| row.conclusion_id != "sentinel_bounded_windows_separator_identity")
            .map(|row| row.conclusion_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            migrated_ids, traceability_ids,
            "migrated Reference product processes must reconcile exactly with traceability"
        );
        assert_eq!(
            migrated_ids.len() as u64,
            reference.migrated_product_invocations
        );

        let mut lines = vec!["hum-workorder11-unit1-slice-product-processes-v1".to_string()];
        let records = reference
            .product_processes
            .iter()
            .chain(&optimized.product_processes)
            .collect::<Vec<_>>();
        for (index, record) in records.iter().enumerate() {
            let stage = record
                .arguments
                .first()
                .expect("product process command stage");
            let source = fixed_source_argument(&record.arguments);
            let command = record.arguments.join(" ");
            for field in [
                record.policy.name(),
                record.attribution,
                record.conclusion_id,
                stage,
                source,
                command.as_str(),
            ] {
                assert!(
                    !field
                        .chars()
                        .any(|character| matches!(character, '\t' | '\r' | '\n')),
                    "slice product-process ledger field contains a control separator"
                );
            }
            lines.push(format!(
                "process\t{:04}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                index + 1,
                record.policy.name(),
                record.attribution,
                record.conclusion_id,
                stage,
                source,
                command,
                record.exit_code,
            ));
        }
        lines.push(format!(
            "summary\t{}\t{}\t{}\t{}",
            records.len(),
            reference.product_invocations,
            optimized.product_invocations,
            reference.migrated_product_invocations,
        ));
        fs::write(path, format!("{}\n", lines.join("\n")).as_bytes()).unwrap_or_else(|error| {
            panic!("failed to write slice product-process ledger: {error}")
        });
    }

    fn required_external_product(repo: &Path) -> PathBuf {
        let path = PathBuf::from(
            std::env::var_os("HUM_UNIT1_PRODUCT_EXE")
                .expect("HUM_UNIT1_PRODUCT_EXE must bind the prebuilt candidate product"),
        );
        assert!(path.is_absolute());
        assert!(
            path.is_file(),
            "candidate product is missing: {}",
            path.display()
        );
        assert!(
            path.starts_with(repo.join("target")) || !path.starts_with(repo),
            "candidate product must be the explicit Fast build or an external equivalent"
        );
        path
    }

    fn required_external_metrics_path(repo: &Path) -> PathBuf {
        required_external_evidence_path(repo, "HUM_UNIT1_METRICS_PATH")
    }

    fn required_external_process_ledger_path(repo: &Path) -> PathBuf {
        required_external_evidence_path(repo, "HUM_UNIT1_SLICE_PROCESS_LEDGER_PATH")
    }

    fn required_external_evidence_path(repo: &Path, variable: &str) -> PathBuf {
        let path = PathBuf::from(
            std::env::var_os(variable)
                .unwrap_or_else(|| panic!("{variable} must name an external evidence artifact")),
        );
        assert!(path.is_absolute());
        assert!(
            !path.starts_with(repo),
            "Unit 1 evidence must remain outside the repository"
        );
        let parent = path.parent().expect("evidence path parent");
        assert!(parent.is_dir(), "evidence parent must already exist");
        assert!(!path.exists(), "evidence artifact must be newly absent");
        path
    }

    fn assert_json(bytes: &[u8]) {
        let text = std::str::from_utf8(bytes).expect("JSON output must be UTF-8");
        let mut parser = JsonParser {
            bytes: text.as_bytes(),
            cursor: 0,
        };
        parser.value().expect("valid JSON value");
        parser.whitespace();
        assert_eq!(
            parser.cursor,
            parser.bytes.len(),
            "JSON output had trailing non-whitespace bytes"
        );
    }

    struct JsonParser<'a> {
        bytes: &'a [u8],
        cursor: usize,
    }

    impl JsonParser<'_> {
        fn value(&mut self) -> Result<(), &'static str> {
            self.whitespace();
            match self.peek() {
                Some(b'{') => self.object(),
                Some(b'[') => self.array(),
                Some(b'"') => self.string(),
                Some(b't') => self.literal(b"true"),
                Some(b'f') => self.literal(b"false"),
                Some(b'n') => self.literal(b"null"),
                Some(b'-' | b'0'..=b'9') => self.number(),
                _ => Err("expected JSON value"),
            }
        }

        fn object(&mut self) -> Result<(), &'static str> {
            self.take(b'{')?;
            self.whitespace();
            if self.consume(b'}') {
                return Ok(());
            }
            loop {
                self.string()?;
                self.whitespace();
                self.take(b':')?;
                self.value()?;
                self.whitespace();
                if self.consume(b'}') {
                    return Ok(());
                }
                self.take(b',')?;
                self.whitespace();
            }
        }

        fn array(&mut self) -> Result<(), &'static str> {
            self.take(b'[')?;
            self.whitespace();
            if self.consume(b']') {
                return Ok(());
            }
            loop {
                self.value()?;
                self.whitespace();
                if self.consume(b']') {
                    return Ok(());
                }
                self.take(b',')?;
            }
        }

        fn string(&mut self) -> Result<(), &'static str> {
            self.take(b'"')?;
            loop {
                match self.next().ok_or("unterminated JSON string")? {
                    b'"' => return Ok(()),
                    b'\\' => match self.next().ok_or("unterminated JSON escape")? {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                        b'u' => {
                            for _ in 0..4 {
                                if !self.next().is_some_and(|byte| byte.is_ascii_hexdigit()) {
                                    return Err("invalid JSON unicode escape");
                                }
                            }
                        }
                        _ => return Err("invalid JSON escape"),
                    },
                    0x00..=0x1f => return Err("unescaped JSON control byte"),
                    _ => {}
                }
            }
        }

        fn number(&mut self) -> Result<(), &'static str> {
            self.consume(b'-');
            if self.consume(b'0') {
                if self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                    return Err("JSON number has a leading zero");
                }
            } else {
                self.digits()?;
            }
            if self.consume(b'.') {
                self.digits()?;
            }
            if self.peek().is_some_and(|byte| matches!(byte, b'e' | b'E')) {
                self.cursor += 1;
                if self.peek().is_some_and(|byte| matches!(byte, b'+' | b'-')) {
                    self.cursor += 1;
                }
                self.digits()?;
            }
            Ok(())
        }

        fn digits(&mut self) -> Result<(), &'static str> {
            let start = self.cursor;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.cursor += 1;
            }
            if self.cursor == start {
                return Err("JSON number requires digits");
            }
            Ok(())
        }

        fn literal(&mut self, literal: &[u8]) -> Result<(), &'static str> {
            if self.bytes.get(self.cursor..self.cursor + literal.len()) == Some(literal) {
                self.cursor += literal.len();
                Ok(())
            } else {
                Err("invalid JSON literal")
            }
        }

        fn whitespace(&mut self) {
            while self
                .peek()
                .is_some_and(|byte| matches!(byte, b' ' | b'\n' | b'\r' | b'\t'))
            {
                self.cursor += 1;
            }
        }

        fn take(&mut self, expected: u8) -> Result<(), &'static str> {
            if self.consume(expected) {
                Ok(())
            } else {
                Err("unexpected JSON token")
            }
        }

        fn consume(&mut self, expected: u8) -> bool {
            if self.peek() == Some(expected) {
                self.cursor += 1;
                true
            } else {
                false
            }
        }

        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.cursor).copied()
        }

        fn next(&mut self) -> Option<u8> {
            let byte = self.peek()?;
            self.cursor += 1;
            Some(byte)
        }
    }
}
