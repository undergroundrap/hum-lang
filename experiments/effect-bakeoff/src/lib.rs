#![forbid(unsafe_code)]

pub mod corpus;
pub mod cost;
pub mod eligibility;
pub mod inventory;
pub mod normalize;
pub mod result_contract;
pub mod row_candidate;

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BTreeMap;

    use crate::corpus::{
        CORPUS_TEXT, Polarity, checked_in, expected_allocation_domains, expected_capture_facts,
        expected_policy_facts, expected_resource_domains, parse, validate, validate_document,
    };
    use crate::cost::{
        AnalysisCost, AnalysisTrace, DiagnosticCost, ImplementationCost, measure_diagnostic,
        measure_implementation,
    };
    use crate::eligibility::PURE_SECOND_CLASS_COMPUTATION_OBSERVATIONS;
    use crate::inventory::candidate_inventory;
    use crate::normalize::{AlphaNormalizer, normalize_identity_path};
    use crate::result_contract::{
        CandidateExecution, CandidateFactory, CandidateResult, CandidateRun, CandidateStatus,
        HarnessEvidence, RESULT_FIELDS, ResultEvidence, StableFact, run_fresh_twice,
        validate_result, validate_run,
    };

    fn fact(id: &str, attributes: &[(&str, &str)], route: &[&str]) -> StableFact {
        StableFact {
            id: id.to_owned(),
            attributes: attributes
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
            route: route.iter().map(|value| (*value).to_owned()).collect(),
        }
    }

    fn mock_result(variant: &crate::corpus::CorpusVariant) -> CandidateResult {
        let rejected = variant.polarity == Polarity::Misuse;
        let mut summary = vec![
            format!("callable_input:shape={}", variant.callable_inputs),
            format!("callable_result:shape={}", variant.callable_result),
            format!(
                "callable_result:observation={}",
                variant.expected_observation
            ),
            format!("ownership_transfer:{}", variant.ownership_transfer),
            format!("resource_lifetime:{}", variant.registration_lifetime),
        ];
        summary.push(match variant.callable_disposition.as_str() {
            "stored_callable" => "stored_callable:required".to_owned(),
            "returned_callable" => "returned_callable:required".to_owned(),
            other => format!("callable_input:disposition={other}"),
        });
        let requirement_facts = expected_policy_facts(variant)
            .into_iter()
            .map(|e| StableFact {
                id: e.id,
                route: e.route,
                attributes: BTreeMap::from([
                    ("origin".to_owned(), e.origin),
                    ("disposition".to_owned(), e.disposition),
                    ("callable".to_owned(), e.callable),
                    ("domain".to_owned(), e.domain),
                    ("policy_role".to_owned(), e.relationship_kind),
                ]),
            })
            .collect();
        let capture_facts = expected_capture_facts(variant)
            .into_iter()
            .map(|e| StableFact {
                id: e.id,
                route: e.route,
                attributes: BTreeMap::from([
                    ("origin".to_owned(), e.origin),
                    ("disposition".to_owned(), e.disposition),
                    ("callable".to_owned(), e.callable),
                    ("domain".to_owned(), e.domain),
                    ("capture_kind".to_owned(), e.relationship_kind),
                ]),
            })
            .collect();
        let allocation_facts = expected_allocation_domains(variant)
            .into_iter()
            .map(|domain| {
                fact(
                    &format!("allocation.{domain}"),
                    &[
                        ("kind", domain),
                        ("trigger", &variant.call_sites[0]),
                        ("lifetime", &variant.registration_lifetime),
                        ("owner", &format!("{}.callable", variant.case_id)),
                        (
                            "evidence",
                            if domain == "none_explicit" {
                                "explicit"
                            } else {
                                "prototype_assumed"
                            },
                        ),
                        ("domain", domain),
                    ],
                    &variant
                        .call_sites
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let resource_domains = expected_resource_domains(variant);
        let resource_facts = resource_domains
            .iter()
            .map(|domain| {
                fact(
                    &format!("resource.{domain}"),
                    &[
                        ("kind", domain),
                        ("trigger", &variant.call_sites[0]),
                        ("lifetime", &variant.registration_lifetime),
                        ("owner", &format!("{}.callable", variant.case_id)),
                        ("evidence", "explicit"),
                        ("domain", domain),
                    ],
                    &variant
                        .call_sites
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let annotation_site = &variant.required_sites[0];
        let annotation_purpose = if variant.source_requirements == ["none_explicit"] {
            "callable_relationship"
        } else {
            "requirement_visibility"
        };
        CandidateResult {
            candidate_id: "mock_candidate".to_owned(),
            case_id: variant.case_id.clone(),
            variant_id: variant.variant_id.clone(),
            candidate_native_result: "$native_requirement becomes $native_result".to_owned(),
            candidate_native_evidence: BTreeMap::from([
                ("native.$first".to_owned(), "present".to_owned()),
                ("native.$second".to_owned(), "present".to_owned()),
            ]),
            neutral_normalized_summary: summary,
            status: if rejected {
                CandidateStatus::Rejected
            } else {
                CandidateStatus::Accepted
            },
            required_source_annotations: vec![fact(
                &format!("annotation.{annotation_site}.{annotation_purpose}.inferred"),
                &[
                    ("site", annotation_site),
                    ("purpose", annotation_purpose),
                    ("mode", "inferred"),
                ],
                &variant
                    .call_sites
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )],
            inferred_requirement_facts: requirement_facts,
            inferred_capture_facts: capture_facts,
            allocation_facts,
            resource_facts,
            added_machinery: vec![fact(
                "machinery.explicit_none",
                &[
                    ("layer", "tooling"),
                    ("state", "not_applicable"),
                    ("creditable", "false"),
                    ("exercised", "false"),
                    ("restructuring", "false"),
                ],
                &["harness", "explicit_none"],
            )],
            primary_reason: variant.primary_reason.clone(),
            primary_blame_site: variant.required_sites[0].clone(),
            related_blame_sites: variant.required_sites[1..].to_vec(),
            repair_direction: "preserve the model-neutral relationship".to_owned(),
            implementation_cost: ImplementationCost {
                nonblank_noncomment_lines: 10,
                dependency_count: 0,
            },
            analysis_cost: AnalysisCost {
                visited_corpus_nodes: 4,
                generated_facts: 3,
                generated_constraints: 2,
                normalization_steps: 2,
                maximum_live_items: 2,
            },
            diagnostic_cost: DiagnosticCost {
                primary_diagnostic_count: usize::from(rejected),
                required_site_count: variant.required_sites.len(),
                covered_required_site_count: variant.required_sites.len(),
                rendered_utf8_bytes: usize::from(rejected) * 42,
                candidate_native_term_count: 0,
                has_model_neutral_repair: rejected,
            },
            missing_evidence: Vec::new(),
        }
    }

    fn complete_field_map() -> Vec<(String, String)> {
        vec![
            ("candidate_id", "mock"),
            ("case_id", "effect.pure_map"),
            ("variant_id", "positive"),
            ("candidate_native_result", "$e"),
            ("candidate_native_evidence", "term=$e"),
            ("neutral_normalized_summary", "callable_input:x"),
            ("status", "accepted"),
            (
                "required_source_annotations",
                "a,site=s,purpose=p,mode=inferred,route=d>c",
            ),
            (
                "inferred_requirement_facts",
                "r,origin=s,disposition=propagated,callable=f,route=d>c",
            ),
            (
                "inferred_capture_facts",
                "c,origin=s,disposition=retained,callable=f,route=d>c",
            ),
            (
                "allocation_facts",
                "a,kind=none,trigger=s,lifetime=call,owner=o,evidence=explicit,route=d>c",
            ),
            (
                "resource_facts",
                "r,kind=none,trigger=s,lifetime=call,owner=o,evidence=explicit,route=d>c",
            ),
            (
                "added_machinery",
                "m,layer=tooling,state=implemented,creditable=true,route=d>c",
            ),
            ("primary_reason", "none_expected"),
            ("primary_blame_site", "map.call"),
            ("related_blame_sites", "callback.call"),
            ("repair_direction", "preserve relationship"),
            ("implementation_cost", "1,0"),
            ("analysis_cost", "1,1,1,1,1"),
            ("diagnostic_cost", "0,2,2,0,0,false"),
            ("missing_evidence", ""),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect()
    }

    fn execution(mut results: Vec<CandidateResult>) -> CandidateExecution {
        let implementation = candidate_inventory("mock_candidate").unwrap().measure();
        let mut evidence = BTreeMap::new();
        for result in &mut results {
            result.implementation_cost = implementation.clone();
            let rejected = result.status == CandidateStatus::Rejected;
            let rendered = if rejected {
                "x".repeat(42)
            } else {
                String::new()
            };
            result.diagnostic_cost = measure_diagnostic(
                &rendered,
                &result
                    .related_blame_sites
                    .iter()
                    .cloned()
                    .chain(std::iter::once(result.primary_blame_site.clone()))
                    .collect::<Vec<_>>(),
                &result
                    .related_blame_sites
                    .iter()
                    .cloned()
                    .chain(std::iter::once(result.primary_blame_site.clone()))
                    .collect::<Vec<_>>(),
                &[],
                &result.repair_direction,
            );
            let mut trace = AnalysisTrace::default();
            for _ in 0..result.analysis_cost.visited_corpus_nodes {
                trace.visit_node();
            }
            for _ in 0..result.analysis_cost.generated_facts {
                trace.generate_fact();
            }
            for _ in 0..result.analysis_cost.generated_constraints {
                trace.generate_constraint();
            }
            for _ in 0..result.analysis_cost.normalization_steps {
                trace.normalization_step();
            }
            for _ in 0..result.analysis_cost.maximum_live_items {
                trace.enter_live_item();
            }
            for _ in 0..result.analysis_cost.maximum_live_items {
                trace.leave_live_item();
            }
            evidence.insert(
                (result.case_id.clone(), result.variant_id.clone()),
                ResultEvidence {
                    analysis_trace: trace,
                    rendered_diagnostic: rendered,
                    covered_sites: result
                        .related_blame_sites
                        .iter()
                        .cloned()
                        .chain(std::iter::once(result.primary_blame_site.clone()))
                        .collect(),
                    candidate_native_terms: Vec::new(),
                },
            );
        }
        CandidateExecution {
            results,
            evidence: HarnessEvidence { results: evidence },
        }
    }

    struct MockFactory;
    struct MockRun;
    impl CandidateFactory for MockFactory {
        type Run = MockRun;
        fn fresh(&self) -> MockRun {
            MockRun
        }
    }
    impl CandidateRun for MockRun {
        fn execute(self, corpus: &crate::corpus::Corpus) -> CandidateExecution {
            execution(corpus.variants.iter().map(mock_result).collect())
        }
    }

    struct LeakyFactory(Cell<usize>);
    struct LeakyRun(usize);
    impl CandidateFactory for LeakyFactory {
        type Run = LeakyRun;
        fn fresh(&self) -> LeakyRun {
            let n = self.0.get();
            self.0.set(n + 1);
            LeakyRun(n)
        }
    }
    impl CandidateRun for LeakyRun {
        fn execute(self, corpus: &crate::corpus::Corpus) -> CandidateExecution {
            let mut run = execution(corpus.variants.iter().map(mock_result).collect());
            run.results[0].candidate_native_result = format!("fresh-state-{}", self.0);
            run
        }
    }

    #[test]
    fn checked_in_corpus_is_complete_neutral_and_documented() {
        let corpus = checked_in();
        assert_eq!(
            corpus
                .variants
                .iter()
                .map(|v| &v.case_id)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            12
        );
        validate_document(&corpus).expect("maintained corpus document must cover data identities");
    }

    #[test]
    fn malformed_corpus_fails_closed() {
        let duplicate = format!(
            "{CORPUS_TEXT}\n{}",
            CORPUS_TEXT
                .lines()
                .find(|line| !line.starts_with('#'))
                .unwrap()
        );
        assert!(
            parse(&duplicate)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("duplicate"))
        );
        let missing_half = CORPUS_TEXT
            .lines()
            .filter(|line| !line.starts_with("effect.pure_map|misuse"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            parse(&missing_half)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("positive or misuse"))
        );
        let candidate_vocabulary =
            CORPUS_TEXT.replace("baseline higher-order", "row variable higher-order");
        assert!(
            parse(&candidate_vocabulary)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("candidate vocabulary"))
        );
        let missing_field = CORPUS_TEXT.replacen("|none_explicit", "", 1);
        assert!(
            parse(&missing_field)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("fields"))
        );
        let absent_blame = CORPUS_TEXT.replacen(
            "callback.definition;callback.call|map.definition",
            "absent.site|map.definition",
            1,
        );
        assert!(
            parse(&absent_blame)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("absent blame site"))
        );
    }

    #[test]
    fn every_frozen_relationship_domain_rejects_mutation() {
        let base = checked_in();
        macro_rules! reject_mutation {
            ($body:expr) => {{
                let mut corpus = base.clone();
                $body(&mut corpus.variants[0]);
                assert!(validate(&corpus, CORPUS_TEXT).is_err());
            }};
        }
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v.variant_id.push_str(".renamed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v.frequency.push_str(".changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v.rationale.push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v.behavior.push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v.values.push("extra".to_owned()));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .callable_inputs
            .push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .callable_result
            .push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .call_sites
            .push("extra.site".to_owned()));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .callable_disposition
            .push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .latent_operations
            .push("extra".to_owned()));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .captured_facts
            .push("extra".to_owned()));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .ownership_transfer
            .push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .registration_lifetime
            .push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .expected_observation
            .push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .primary_reason
            .push_str(" changed"));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .required_sites
            .push("extra.site".to_owned()));
        reject_mutation!(
            |v: &mut crate::corpus::CorpusVariant| v.all_sites[0].kind = "invalid".to_owned()
        );
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .source_requirements
            .push("extra".to_owned()));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .operator_consent
            .push("extra".to_owned()));
        reject_mutation!(|v: &mut crate::corpus::CorpusVariant| v
            .operation_exercise
            .push("extra".to_owned()));
    }

    #[test]
    fn every_omitted_result_field_forces_incomplete_and_extra_fields_reject() {
        for omitted in RESULT_FIELDS {
            let fields = complete_field_map()
                .into_iter()
                .filter(|(name, _)| name != omitted)
                .collect();
            let result = CandidateResult::from_top_level_fields(fields).unwrap();
            assert_eq!(result.status, CandidateStatus::Incomplete, "{omitted}");
            assert!(
                result
                    .missing_evidence
                    .iter()
                    .any(|item| item == &format!("missing_field:{omitted}"))
            );
        }
        let mut extra = complete_field_map();
        extra.push(("candidate_only_score".to_owned(), "free".to_owned()));
        assert!(
            CandidateResult::from_top_level_fields(extra)
                .unwrap_err()
                .contains("unknown")
        );
    }

    #[test]
    fn incomplete_result_and_cost_gates_fail_closed() {
        let corpus = checked_in();
        let mut result = mock_result(&corpus.variants[0]);
        result.allocation_facts.clear();
        result.implementation_cost = ImplementationCost::default();
        result
            .neutral_normalized_summary
            .push("candidate_restructure:free".to_owned());
        validate_result(&corpus, &mut result);
        assert_eq!(result.status, CandidateStatus::Incomplete);
        assert!(
            result
                .missing_evidence
                .contains(&"empty_field:allocation_facts".to_owned())
        );
        assert!(
            result
                .missing_evidence
                .contains(&"absent_cost:implementation_cost".to_owned())
        );
        assert!(
            result
                .missing_evidence
                .contains(&"invalid_neutral_summary_term:candidate_restructure".to_owned())
        );
    }

    #[test]
    fn fresh_runs_canonicalize_to_identical_bytes() {
        let corpus = checked_in();
        let bytes =
            run_fresh_twice(&corpus, &MockFactory).expect("fresh runs must normalize identically");
        assert!(!bytes.is_empty());
        let leaky = LeakyFactory(Cell::new(0));
        assert!(
            run_fresh_twice(&corpus, &leaky)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("byte-identical"))
        );
        assert_eq!(leaky.0.get(), 2);
    }

    #[test]
    fn run_rejects_missing_duplicate_and_incomplete_results() {
        let corpus = checked_in();
        let mut missing = execution(corpus.variants.iter().skip(1).map(mock_result).collect());
        assert!(
            validate_run(&corpus, &mut missing)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("missing result"))
        );
        let mut duplicate_results: Vec<_> = corpus.variants.iter().map(mock_result).collect();
        duplicate_results.push(mock_result(&corpus.variants[0]));
        let mut duplicate = execution(duplicate_results);
        assert!(
            validate_run(&corpus, &mut duplicate)
                .unwrap_err()
                .iter()
                .any(|error| error.contains("duplicate result"))
        );
    }

    #[test]
    fn semantic_laundering_and_every_cost_dimension_fail_closed() {
        let corpus = checked_in();
        let mut relationship = execution(corpus.variants.iter().map(mock_result).collect());
        let target = relationship
            .results
            .iter_mut()
            .find(|r| r.case_id == "effect.memoizing_wrapper" && r.variant_id == "positive")
            .unwrap();
        target.neutral_normalized_summary.clear();
        target.inferred_requirement_facts =
            mock_result(&corpus.variants[0]).inferred_requirement_facts;
        target.inferred_capture_facts = mock_result(&corpus.variants[0]).inferred_capture_facts;
        target.allocation_facts = mock_result(&corpus.variants[0]).allocation_facts;
        target.resource_facts = mock_result(&corpus.variants[0]).resource_facts;
        assert!(
            validate_run(&corpus, &mut relationship)
                .unwrap_err()
                .iter()
                .any(|e| e.contains("relationship") || e.contains("empty_field"))
        );

        for mutation in 0..13 {
            let mut run = execution(corpus.variants.iter().map(mock_result).collect());
            let result = run
                .results
                .iter_mut()
                .find(|r| r.case_id == "effect.event_handler_factory" && r.variant_id == "positive")
                .unwrap();
            match mutation {
                0 => {
                    result.neutral_normalized_summary.pop();
                }
                1 => {
                    result.inferred_requirement_facts[0]
                        .attributes
                        .insert("origin".into(), "wrong".into());
                }
                2 => {
                    result.inferred_requirement_facts[0]
                        .attributes
                        .insert("disposition".into(), "wrong".into());
                }
                3 => {
                    result.inferred_requirement_facts[0]
                        .attributes
                        .insert("callable".into(), "wrong".into());
                }
                4 => result.inferred_requirement_facts[0]
                    .route
                    .push("wrong".into()),
                5 => {
                    result.inferred_capture_facts[0]
                        .attributes
                        .insert("capture_kind".into(), "resource".into());
                }
                6 => {
                    result.allocation_facts[0]
                        .attributes
                        .insert("domain".into(), "wrong".into());
                }
                7 => {
                    result.resource_facts[0]
                        .attributes
                        .insert("domain".into(), "wrong".into());
                }
                8 => {
                    result.added_machinery[0]
                        .attributes
                        .insert("creditable".into(), "true".into());
                }
                9 => {
                    result.added_machinery[0]
                        .attributes
                        .insert("state".into(), "planned".into());
                    result.added_machinery[0]
                        .attributes
                        .insert("creditable".into(), "true".into());
                }
                10 => {
                    result.added_machinery[0]
                        .attributes
                        .insert("exercised".into(), "false".into());
                    result.added_machinery[0]
                        .attributes
                        .insert("creditable".into(), "true".into());
                }
                11 => {
                    result.added_machinery[0]
                        .attributes
                        .insert("restructuring".into(), "true".into());
                    result.added_machinery[0]
                        .attributes
                        .insert("creditable".into(), "true".into());
                }
                12 => {
                    let mut extra = result.inferred_capture_facts[0].clone();
                    extra.id = "contradictory.extra".into();
                    result.inferred_capture_facts.push(extra);
                }
                _ => unreachable!(),
            }
            assert!(
                validate_run(&corpus, &mut run).is_err(),
                "relationship mutation {mutation}"
            );
        }

        for dimension in 0..13 {
            let mut run = execution(corpus.variants.iter().map(mock_result).collect());
            let result = &mut run.results[0];
            match dimension {
                0 => result.implementation_cost.nonblank_noncomment_lines += 1,
                1 => result.implementation_cost.dependency_count += 1,
                2 => result.analysis_cost.visited_corpus_nodes += 1,
                3 => result.analysis_cost.generated_facts += 1,
                4 => result.analysis_cost.generated_constraints += 1,
                5 => result.analysis_cost.normalization_steps += 1,
                6 => result.analysis_cost.maximum_live_items += 1,
                7 => result.diagnostic_cost.primary_diagnostic_count += 1,
                8 => result.diagnostic_cost.required_site_count += 1,
                9 => result.diagnostic_cost.covered_required_site_count += 1,
                10 => result.diagnostic_cost.rendered_utf8_bytes += 1,
                11 => result.diagnostic_cost.candidate_native_term_count += 1,
                12 => {
                    result.diagnostic_cost.has_model_neutral_repair =
                        !result.diagnostic_cost.has_model_neutral_repair
                }
                _ => unreachable!(),
            }
            assert!(
                validate_run(&corpus, &mut run).is_err(),
                "cost dimension {dimension}"
            );
        }
    }

    #[test]
    fn normalization_preserves_route_order_and_display_spelling() {
        let mut alpha = AlphaNormalizer::default();
        assert_eq!(
            alpha.normalize("$effect + $other + $effect"),
            "$v0 + $v1 + $v0"
        );
        assert_eq!(
            normalize_identity_path("fixtures\\case.hum"),
            "fixtures/case.hum"
        );
        let corpus = checked_in();
        let mut result = mock_result(&corpus.variants[0]);
        result.required_source_annotations[0].attributes.insert(
            "source_identity_path".to_owned(),
            "fixtures\\case.hum".to_owned(),
        );
        result.required_source_annotations[0].attributes.insert(
            "source_display_path".to_owned(),
            "fixtures\\case.hum".to_owned(),
        );
        let bytes = String::from_utf8(result.canonical_bytes()).unwrap();
        assert!(bytes.contains("map.call>callback.call"));
        assert!(bytes.contains("source_identity_path=fixtures/case.hum"));
        assert!(bytes.contains("source_display_path=fixtures\\case.hum"));
    }

    #[test]
    fn every_structured_result_field_has_an_independent_mutation() {
        let corpus = checked_in();
        macro_rules! reject {
            ($body:expr) => {{
                let mut run = execution(corpus.variants.iter().map(mock_result).collect());
                let r = run
                    .results
                    .iter_mut()
                    .find(|r| {
                        r.case_id == "effect.event_handler_factory" && r.variant_id == "positive"
                    })
                    .unwrap();
                $body(r);
                assert!(validate_run(&corpus, &mut run).is_err());
            }};
        }
        reject!(|r: &mut CandidateResult| r.required_source_annotations[0].id = "wrong".into());
        for key in ["site", "purpose", "mode"] {
            reject!(|r: &mut CandidateResult| {
                r.required_source_annotations[0]
                    .attributes
                    .insert(key.into(), "wrong".into());
            });
        }
        reject!(|r: &mut CandidateResult| r.required_source_annotations[0]
            .route
            .push("wrong".into()));
        reject!(|r: &mut CandidateResult| r.inferred_requirement_facts[0].id = "wrong".into());
        for key in ["origin", "disposition", "callable", "domain", "policy_role"] {
            reject!(|r: &mut CandidateResult| {
                r.inferred_requirement_facts[0]
                    .attributes
                    .insert(key.into(), "wrong".into());
            });
        }
        reject!(|r: &mut CandidateResult| r.inferred_requirement_facts[0]
            .route
            .push("wrong".into()));
        reject!(|r: &mut CandidateResult| r.inferred_capture_facts[0].id = "wrong".into());
        for key in [
            "origin",
            "disposition",
            "callable",
            "domain",
            "capture_kind",
        ] {
            reject!(|r: &mut CandidateResult| {
                r.inferred_capture_facts[0]
                    .attributes
                    .insert(key.into(), "wrong".into());
            });
        }
        reject!(|r: &mut CandidateResult| r.inferred_capture_facts[0].route.push("wrong".into()));
        for target in ["allocation", "resource"] {
            for key in ["kind", "trigger", "lifetime", "owner", "evidence", "domain"] {
                reject!(|r: &mut CandidateResult| {
                    let fact = if target == "allocation" {
                        &mut r.allocation_facts[0]
                    } else {
                        &mut r.resource_facts[0]
                    };
                    fact.attributes.insert(key.into(), "wrong".into());
                });
            }
        }
        reject!(|r: &mut CandidateResult| r.allocation_facts[0].id = "wrong".into());
        reject!(|r: &mut CandidateResult| r.allocation_facts[0].route.push("wrong".into()));
        reject!(|r: &mut CandidateResult| r.resource_facts[0].id = "wrong".into());
        reject!(|r: &mut CandidateResult| r.resource_facts[0].route.push("wrong".into()));
        reject!(|r: &mut CandidateResult| {
            let duplicate = r.allocation_facts[0].clone();
            r.allocation_facts.push(duplicate);
        });
        reject!(|r: &mut CandidateResult| {
            let duplicate = r.resource_facts[0].clone();
            r.resource_facts.push(duplicate);
        });
        reject!(|r: &mut CandidateResult| r.added_machinery[0].id = "wrong".into());
        for (key, value) in [
            ("layer", "wrong"),
            ("state", "unknown"),
            ("creditable", "yes"),
            ("exercised", "maybe"),
            ("restructuring", "unknown"),
        ] {
            reject!(|r: &mut CandidateResult| {
                r.added_machinery[0]
                    .attributes
                    .insert(key.into(), value.into());
            });
        }
        reject!(|r: &mut CandidateResult| r.added_machinery[0].route.push("wrong".into()));
        for field in 0..6 {
            reject!(|r: &mut CandidateResult| {
                let facts = match field {
                    0 => &mut r.required_source_annotations,
                    1 => &mut r.inferred_requirement_facts,
                    2 => &mut r.inferred_capture_facts,
                    3 => &mut r.allocation_facts,
                    4 => &mut r.resource_facts,
                    5 => &mut r.added_machinery,
                    _ => unreachable!(),
                };
                facts[0]
                    .attributes
                    .insert("candidate_score".into(), "100".into());
            });
        }
    }

    #[test]
    fn corpus_owned_domain_origins_reject_valid_site_substitution() {
        let mut source_origin = checked_in();
        let variant = source_origin
            .variants
            .iter_mut()
            .find(|variant| {
                variant.case_id == "effect.callback_registry" && variant.variant_id == "positive"
            })
            .unwrap();
        variant
            .source_requirement_origins
            .insert("callback.requirement".into(), "fire.call".into());
        assert!(validate(&source_origin, CORPUS_TEXT).is_err());

        let mut capture_origin = checked_in();
        let variant = capture_origin
            .variants
            .iter_mut()
            .find(|variant| {
                variant.case_id == "effect.callback_registry" && variant.variant_id == "positive"
            })
            .unwrap();
        variant
            .capture_origins
            .insert("registration".into(), "caller.state".into());
        assert!(validate(&capture_origin, CORPUS_TEXT).is_err());
    }

    #[test]
    fn cost_measurements_are_deterministic_and_trace_checked() {
        let implementation =
            measure_implementation("// comment\nfn a() {}\n\nfn b() {}\n", "[dependencies]\n");
        assert_eq!(implementation.nonblank_noncomment_lines, 2);
        assert_eq!(implementation.dependency_count, 0);
        let comments = measure_implementation(
            "#![forbid(unsafe_code)]\n/* block\ncomment */\nfn kept() {} // tail\n",
            "",
        );
        assert_eq!(comments.nonblank_noncomment_lines, 2);
        let literal_adversaries = measure_implementation(
            concat!(
                "#[test]\n",
                "fn literals() {\n",
                "    let string = \"/* not a comment */ // neither\";\n",
                "    let raw = r###\"/* still text */ // still text\"###;\n",
                "    let bytes = b\"/* bytes */ // bytes\";\n",
                "    let byte_raw = br#\"// raw bytes /* */\"#;\n",
                "    let slash = '/';\n",
                "    // one line comment\n",
                "    /* one block comment */\n",
                "}\n",
            ),
            "",
        );
        assert_eq!(literal_adversaries.nonblank_noncomment_lines, 8);
        let all_sections = "[dependencies]\na=\"1\"\n[dev-dependencies]\nb=\"1\"\n[build-dependencies]\nc=\"1\"\n[target.'cfg(windows)'.dependencies]\nd=\"1\"\n";
        assert_eq!(
            measure_implementation("fn a() {}", all_sections).dependency_count,
            4
        );
        let mut trace = AnalysisTrace::default();
        trace.enter_live_item();
        trace.visit_node();
        trace.generate_fact();
        trace.generate_constraint();
        trace.normalization_step();
        trace.leave_live_item();
        assert!(trace.check_reported(&trace.measured()));
        let mut mismatched = trace.measured();
        mismatched.generated_facts += 1;
        assert!(!trace.check_reported(&mismatched));
        let diagnostic = measure_diagnostic(
            "one explanation",
            &["a".to_owned(), "b".to_owned()],
            &["b".to_owned(), "a".to_owned()],
            &["native".to_owned()],
            "preserve the relationship",
        );
        assert_eq!(diagnostic.primary_diagnostic_count, 1);
        assert_eq!(diagnostic.covered_required_site_count, 2);
    }

    #[test]
    fn pure_second_class_eligibility_is_transparent_and_non_selecting() {
        assert_eq!(PURE_SECOND_CLASS_COMPUTATION_OBSERVATIONS.len(), 5);
        assert!(
            PURE_SECOND_CLASS_COMPUTATION_OBSERVATIONS
                .iter()
                .any(|item| item.relationship == "stored_callable")
        );
        assert!(
            PURE_SECOND_CLASS_COMPUTATION_OBSERVATIONS
                .iter()
                .filter(|item| item.relationship == "returned_callable")
                .count()
                >= 3
        );
        assert!(
            PURE_SECOND_CLASS_COMPUTATION_OBSERVATIONS
                .iter()
                .any(|item| item.case_id == "effect.linear_capture")
        );
        assert!(
            PURE_SECOND_CLASS_COMPUTATION_OBSERVATIONS
                .iter()
                .all(|item| item.status == "ineligible_without_restructuring")
        );
    }

    #[test]
    fn experiment_has_no_dependencies_and_is_not_a_root_dependency() {
        let manifest = include_str!("../Cargo.toml");
        assert_eq!(
            measure_implementation("fn proof() {}", manifest).dependency_count,
            0
        );
        let root_manifest = include_str!("../../../Cargo.toml");
        assert!(!root_manifest.contains("hum-effect-bakeoff"));
    }
}
