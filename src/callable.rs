use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::ast::{
    Item, Param, ParsedBodyStatement, ParsedBodyStatementKind, ParsedCall, ParsedCallCloseStatus,
    ParsedCallTrailingStatus, ParsedExpression, ParsedExpressionKind, Program, Task, TypeSyntax,
    TypeSyntaxKind,
};
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticOccurrence, DiagnosticOccurrenceCollector, Severity, Span,
};
use crate::node_id;
use crate::resolve::{
    self, ResolveDefinitionSummary, ResolveReferenceSummary, ResolveScopeSummary,
};

pub(crate) const CALLABLE_FACT_MODEL: &str = "canonical_callable_semantic_spine_am_v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallableDefinitionFact {
    pub(crate) id: String,
    pub(crate) resolver_definition_id: String,
    pub(crate) lexical_scope_id: String,
    pub(crate) source_span: Span,
    pub(crate) input_definition_ids: Vec<String>,
    pub(crate) input_types: Vec<String>,
    pub(crate) result_type: String,
    pub(crate) failure_root: Option<String>,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallableTypeFact {
    pub(crate) id: String,
    pub(crate) input_types: Vec<String>,
    pub(crate) result_type: String,
    pub(crate) failure_root: Option<String>,
    pub(crate) latent_row_id: String,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LatentEffectRowFact {
    pub(crate) id: String,
    pub(crate) labels: Vec<String>,
    pub(crate) normalized_labels: Vec<String>,
    pub(crate) tail_id: Option<String>,
    pub(crate) tail_alias: Option<String>,
    pub(crate) normalized_tail: Option<String>,
    pub(crate) status: &'static str,
    pub(crate) origin: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EffectLabelOccurrenceFact {
    pub(crate) id: String,
    pub(crate) label: &'static str,
    pub(crate) alias_id: String,
    pub(crate) resolver_reference_id: String,
    pub(crate) source_span: Span,
    pub(crate) owner_definition_id: String,
    pub(crate) target_definition_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowSubstitutionFact {
    pub(crate) id: String,
    pub(crate) application_id: String,
    pub(crate) input_row_id: String,
    pub(crate) tail_id: String,
    pub(crate) argument_row_id: String,
    pub(crate) output_row_id: String,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallableValueFact {
    pub(crate) id: String,
    pub(crate) source_span: Span,
    pub(crate) resolver_reference_id: String,
    pub(crate) referring_scope_id: String,
    pub(crate) resolved_task_definition_id: String,
    pub(crate) expected_callable_type_id: String,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OrdinaryArgumentFact {
    UIntLiteral(u64),
    Definition {
        definition_id: String,
        binding_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallableApplicationFact {
    pub(crate) id: String,
    pub(crate) caller_definition_id: String,
    pub(crate) caller_span: Span,
    pub(crate) receiver_definition_id: String,
    pub(crate) receiver_span: Span,
    pub(crate) callable_parameter_definition_id: String,
    pub(crate) callable_parameter_name: String,
    pub(crate) ordinary_parameter_definition_id: String,
    pub(crate) ordinary_parameter_name: String,
    pub(crate) callable_value_id: String,
    pub(crate) target_definition_id: String,
    pub(crate) direct_call_span: Span,
    pub(crate) indirect_call_span: Span,
    direct_statement_span: Span,
    indirect_statement_span: Span,
    pub(crate) ordinary_argument: OrdinaryArgumentFact,
    pub(crate) result_type: String,
    pub(crate) failure_root: Option<String>,
    pub(crate) input_row_id: String,
    pub(crate) output_row_id: String,
    pub(crate) status: &'static str,
    pub(crate) reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallableDiagnosticFact {
    pub(crate) id: String,
    pub(crate) reason: &'static str,
    pub(crate) detail_reason: &'static str,
    fallback_diagnostic: Option<Diagnostic>,
    pub(crate) occurrence: Option<DiagnosticOccurrence>,
    relationship_sites: Vec<Span>,
}

impl CallableDiagnosticFact {
    fn diagnostic(&self) -> &Diagnostic {
        self.occurrence.as_ref().map_or_else(
            || {
                self.fallback_diagnostic
                    .as_ref()
                    .expect("fallback diagnostic")
            },
            |occurrence| occurrence.diagnostic(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallableCoreNodeFact {
    kind: String,
    id: String,
    relationship_id: String,
    result_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallableGraphEdgeFact {
    id: String,
    kind: String,
    from: String,
    to: String,
    owner_definition_id: String,
    application_id: String,
    source_span: Span,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CallableAnalysis {
    pub(crate) definitions: Vec<CallableDefinitionFact>,
    pub(crate) types: Vec<CallableTypeFact>,
    pub(crate) rows: Vec<LatentEffectRowFact>,
    pub(crate) label_occurrences: Vec<EffectLabelOccurrenceFact>,
    pub(crate) substitutions: Vec<RowSubstitutionFact>,
    pub(crate) values: Vec<CallableValueFact>,
    pub(crate) applications: Vec<CallableApplicationFact>,
    pub(crate) diagnostics: Vec<CallableDiagnosticFact>,
    definition_names: BTreeMap<String, String>,
    definition_spans: BTreeMap<String, Span>,
    resolver_definitions: Vec<ResolveDefinitionSummary>,
    resolver_scopes: Vec<ResolveScopeSummary>,
    resolver_references: Vec<ResolveReferenceSummary>,
    canonical_definitions: Vec<CallableDefinitionFact>,
    canonical_types: Vec<CallableTypeFact>,
    canonical_rows: Vec<LatentEffectRowFact>,
    canonical_label_occurrences: Vec<EffectLabelOccurrenceFact>,
    canonical_substitutions: Vec<RowSubstitutionFact>,
    canonical_values: Vec<CallableValueFact>,
    canonical_applications: Vec<CallableApplicationFact>,
    canonical_diagnostics: Vec<CallableDiagnosticFact>,
    typed_failure_prior: Vec<DiagnosticOccurrence>,
}

#[derive(Clone)]
struct TaskEntry<'a> {
    task: &'a Task,
    file: &'a str,
    definition_id: String,
    callable_scope_id: String,
}

#[derive(Clone)]
struct ReceiverInfo<'a> {
    entry: TaskEntry<'a>,
    callable_param: &'a Param,
    callable_param_definition_id: String,
    ordinary_param: &'a Param,
    ordinary_param_definition_id: String,
    callable_type_id: String,
    row_id: String,
    indirect_span: Option<Span>,
    indirect_statement_span: Option<Span>,
    valid: bool,
}

#[derive(Clone, PartialEq, Eq)]
struct AnalysisKey {
    program: Program,
    definitions: Vec<ResolveDefinitionSummary>,
    scopes: Vec<ResolveScopeSummary>,
    references: Vec<ResolveReferenceSummary>,
}

thread_local! {
    static ANALYSIS_CACHE: RefCell<Option<(AnalysisKey, Arc<CallableAnalysis>)>> = const { RefCell::new(None) };
}

pub(crate) fn analyze_program(program: &Program) -> Arc<CallableAnalysis> {
    let definitions = resolve::resolve_definition_summaries(program, &[]);
    let scopes = resolve::resolve_scope_summaries(program, &[]);
    let references = resolve::resolve_reference_summaries(program, &[]);
    let key = analysis_key(program, &definitions, &scopes, &references);
    ANALYSIS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some((cached_key, analysis)) = cache.as_ref()
            && cached_key == &key
        {
            return Arc::clone(analysis);
        }
        let analysis = Arc::new(CallableAnalysis::build(
            program,
            definitions,
            scopes,
            references,
        ));
        *cache = Some((key, Arc::clone(&analysis)));
        analysis
    })
}

pub(crate) fn diagnostics(program: &Program, prior: &[Diagnostic]) -> Vec<Diagnostic> {
    let analysis = analyze_program(program);
    let unknown_type_priors = crate::type_check::unknown_type_diagnostics(program, &[]);
    let path_priors = crate::path_boundary::diagnostics(program);
    analysis
        .diagnostics
        .iter()
        .filter(|fact| !analysis.prior_owns(fact, prior, &unknown_type_priors, &path_priors))
        .map(|fact| fact.diagnostic().clone())
        .collect()
}

pub(crate) fn diagnostic_occurrences(program: &Program) -> Vec<DiagnosticOccurrence> {
    analyze_program(program)
        .diagnostics
        .iter()
        .filter_map(|fact| fact.occurrence.clone())
        .collect()
}

pub(crate) fn validate_diagnostic_occurrences(
    program: &Program,
) -> Result<(), crate::diagnostic::DiagnosticInvariantError> {
    let analysis = analyze_program(program);
    if analysis.diagnostics.iter().any(|fact| {
        matches!(
            fact.diagnostic().code,
            DiagnosticCode::INVALID_CALLABLE_FORM | DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH
        ) && fact.occurrence.is_none()
    }) {
        return Err(crate::diagnostic::DiagnosticInvariantError::UnknownCause);
    }
    let mut collector = DiagnosticOccurrenceCollector::default();
    for occurrence in analysis
        .diagnostics
        .iter()
        .filter_map(|fact| fact.occurrence.clone())
    {
        collector.insert(occurrence)?;
    }
    Ok(())
}

pub(crate) fn stage_blockers(program: &Program, stage: &str) -> usize {
    let analysis = analyze_program(program);
    let diagnostic_blockers = analysis.diagnostics_for_stage(stage).len();
    let verifier_blockers = if stage == "core_verify" {
        analysis.verify().len()
    } else {
        0
    };
    diagnostic_blockers + verifier_blockers
}

impl CallableAnalysis {
    fn build(
        program: &Program,
        definitions: Vec<ResolveDefinitionSummary>,
        scopes: Vec<ResolveScopeSummary>,
        references: Vec<ResolveReferenceSummary>,
    ) -> Self {
        let tasks = task_entries(program, &definitions, &scopes);
        let known_types = known_type_names(program);
        let mut analysis = Self {
            resolver_definitions: definitions.clone(),
            resolver_scopes: scopes.clone(),
            resolver_references: references.clone(),
            typed_failure_prior: crate::typed_failure::analyze_program(program)
                .occurrences()
                .into_iter()
                .collect(),
            ..Self::default()
        };
        for definition in &definitions {
            analysis
                .definition_names
                .insert(definition.id.clone(), definition.name.clone());
            analysis
                .definition_spans
                .insert(definition.id.clone(), definition.source_span.clone());
        }

        let mut receivers = Vec::new();
        for entry in &tasks {
            if entry.task.params.iter().any(|param| {
                matches!(
                    param.type_syntax.kind,
                    TypeSyntaxKind::Callable(_) | TypeSyntaxKind::CallableCandidate { .. }
                )
            }) {
                receivers.push(analysis.analyze_receiver(entry.clone(), &definitions, &references));
            }
        }

        let receiver_by_definition = receivers
            .iter()
            .map(|receiver| (receiver.entry.definition_id.clone(), receiver.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut direct_relationship_spans: BTreeMap<String, Span> = BTreeMap::new();
        for caller in &tasks {
            for statement in &caller.task.body_syntax {
                visit_statement_calls(statement, &mut |statement_span, expression, call| {
                    let Some(callee) = identifier(&call.callee) else {
                        return;
                    };
                    let Some(reference) =
                        reference_at(&references, &callee.span, "callable_callee_ref")
                    else {
                        return;
                    };
                    let Some(receiver_id) = reference.resolved_definition_id.as_deref() else {
                        return;
                    };
                    let Some(receiver) = receiver_by_definition.get(receiver_id) else {
                        return;
                    };
                    if let Some(first_span) = direct_relationship_spans.get(receiver_id) {
                        analysis.push_invalid(
                            &expression.span,
                            "Session AM accepts exactly one direct callable relationship and application",
                            "multiple_direct_callable_applications_unsupported_v0",
                            vec![first_span.clone(), receiver.entry.task.span.clone()],
                        );
                        return;
                    }
                    direct_relationship_spans
                        .insert(receiver_id.to_string(), portable_span(&expression.span));
                    analysis.analyze_direct_application(
                        caller,
                        statement_span,
                        expression,
                        call,
                        receiver,
                        &tasks,
                        &definitions,
                        &references,
                        &known_types,
                    );
                });
            }
        }

        analysis.definitions.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.types.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.rows.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.label_occurrences.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.substitutions.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.values.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.applications.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.diagnostics.sort_by(|a, b| a.id.cmp(&b.id));
        analysis.canonical_definitions = analysis.definitions.clone();
        analysis.canonical_types = analysis.types.clone();
        analysis.canonical_rows = analysis.rows.clone();
        analysis.canonical_label_occurrences = analysis.label_occurrences.clone();
        analysis.canonical_substitutions = analysis.substitutions.clone();
        analysis.canonical_values = analysis.values.clone();
        analysis.canonical_applications = analysis.applications.clone();
        analysis.canonical_diagnostics = analysis.diagnostics.clone();
        analysis
    }

    fn analyze_receiver<'a>(
        &mut self,
        entry: TaskEntry<'a>,
        definitions: &[ResolveDefinitionSummary],
        references: &[ResolveReferenceSummary],
    ) -> ReceiverInfo<'a> {
        let callable_params = entry
            .task
            .params
            .iter()
            .filter(|param| {
                matches!(
                    param.type_syntax.kind,
                    TypeSyntaxKind::Callable(_) | TypeSyntaxKind::CallableCandidate { .. }
                )
            })
            .collect::<Vec<_>>();
        let callable_param = callable_params
            .first()
            .copied()
            .unwrap_or(&entry.task.params[0]);
        let ordinary_param = entry
            .task
            .params
            .iter()
            .find(|param| !std::ptr::eq(*param, callable_param))
            .unwrap_or(callable_param);
        let callable_param_definition_id = definition_for_span(
            definitions,
            &callable_param.span,
            "parameter",
            Some(&entry.callable_scope_id),
        )
        .map_or_else(
            || missing_id("callable-parameter", &callable_param.span),
            |definition| definition.id.clone(),
        );
        let ordinary_param_definition_id = definition_for_span(
            definitions,
            &ordinary_param.span,
            "parameter",
            Some(&entry.callable_scope_id),
        )
        .map_or_else(
            || missing_id("ordinary-parameter", &ordinary_param.span),
            |definition| definition.id.clone(),
        );
        let callable_type_id = semantic_id("callable-type", &callable_param.type_syntax.span);
        let row_id = semantic_id("latent-row", &callable_param.type_syntax.span);
        let mut valid = true;

        if callable_params.len() != 1
            || entry.task.params.len() != 2
            || !std::ptr::eq(
                entry.task.params.first().unwrap_or(callable_param),
                callable_param,
            )
        {
            valid = false;
            self.push_invalid(
                &entry.task.span,
                "the AL receiver must declare exactly one callable parameter followed by one UInt parameter",
                "receiver_parameter_shape_outside_al_v0",
                vec![callable_param.span.clone()],
            );
        }
        if callable_param.permission_explicit || ordinary_param.permission_explicit {
            valid = false;
            self.push_invalid(
                &callable_param.span,
                "Session AL does not accept permission-bearing callable or ordinary parameters",
                "permission_bearing_callable_parameter_v0",
                vec![entry.task.span.clone()],
            );
        }
        if !callable_param.type_hws_valid
            || !ordinary_param.type_hws_valid
            || !ordinary_param.separator_hws_valid
        {
            valid = false;
            self.push_invalid(
                &callable_param.span,
                "callable parameters require horizontal whitespace after `:` and after the separating comma",
                "callable_parameter_horizontal_whitespace_v0",
                vec![ordinary_param.span.clone()],
            );
        }
        match &callable_param.type_syntax.kind {
            TypeSyntaxKind::Callable(callable) if exact_uint_callable(callable) => {}
            TypeSyntaxKind::CallableCandidate { reason } => {
                valid = false;
                self.push_invalid(
                    &callable_param.type_syntax.span,
                    "callable type must use the exact `task(UInt) -> UInt` spelling",
                    reason,
                    vec![entry.task.span.clone()],
                );
            }
            _ => {
                valid = false;
                self.push_invalid(
                    &callable_param.type_syntax.span,
                    "callable type must contain exactly one UInt input and one UInt result",
                    "callable_type_shape_outside_al_v0",
                    vec![entry.task.span.clone()],
                );
            }
        }
        if !is_named(&ordinary_param.type_syntax, "UInt")
            || entry
                .task
                .result_syntax
                .as_ref()
                .is_none_or(|result| !is_named(result, "UInt"))
        {
            valid = false;
            self.push_mismatch(
                &entry.task.span,
                "receiver input and result must both be UInt",
                "receiver_ordinary_signature_mismatch_v0",
                vec![ordinary_param.type_syntax.span.clone()],
            );
        }

        let mut indirect_calls = Vec::new();
        let mut retained_use = None;
        for statement in &entry.task.body_syntax {
            match &statement.kind {
                ParsedBodyStatementKind::Return(expression) => {
                    find_unsupported_callable_value_use(
                        expression,
                        &callable_param_definition_id,
                        references,
                        false,
                        &mut retained_use,
                    );
                    visit_expression_calls(expression, &mut |candidate, call| {
                        if call_callee_resolves_to(call, &callable_param_definition_id, references)
                        {
                            indirect_calls.push((candidate, call, true, &statement.span));
                        }
                    });
                }
                ParsedBodyStatementKind::Binding { value, .. } => {
                    if let Some(expression) = value {
                        find_unsupported_callable_value_use(
                            expression,
                            &callable_param_definition_id,
                            references,
                            false,
                            &mut retained_use,
                        );
                        visit_expression_calls(expression, &mut |candidate, call| {
                            if call_callee_resolves_to(
                                call,
                                &callable_param_definition_id,
                                references,
                            ) {
                                indirect_calls.push((candidate, call, false, &statement.span));
                            }
                        });
                    }
                }
                ParsedBodyStatementKind::Other { expressions } => {
                    for expression in expressions {
                        find_unsupported_callable_value_use(
                            expression,
                            &callable_param_definition_id,
                            references,
                            false,
                            &mut retained_use,
                        );
                        visit_expression_calls(expression, &mut |candidate, call| {
                            if call_callee_resolves_to(
                                call,
                                &callable_param_definition_id,
                                references,
                            ) {
                                indirect_calls.push((candidate, call, false, &statement.span));
                            }
                        });
                    }
                }
            }
        }
        if let Some(span) = retained_use {
            valid = false;
            self.push_invalid(
                &span,
                "callable parameters cannot be stored or returned in Session AL",
                "callable_storage_or_return_unsupported_v0",
                vec![callable_param.span.clone()],
            );
        }
        if indirect_calls.is_empty() {
            valid = false;
            self.push_invalid(
                &callable_param.span,
                "the callable parameter must be applied exactly once",
                "required_exactly_one_callable_application_v0",
                vec![entry.task.span.clone()],
            );
        } else if indirect_calls.len() > 1 {
            valid = false;
            self.push_invalid(
                &indirect_calls[1].0.span,
                "the callable parameter may be applied exactly once in Session AL",
                "multiple_callable_applications_unsupported_v0",
                vec![indirect_calls[0].0.span.clone()],
            );
        }

        let indirect_span = indirect_calls
            .first()
            .map(|(expression, call, is_return, _)| {
                if indirect_calls.len() > 1 {
                    return expression.span.clone();
                }
                if !*is_return
                    || call.close_status != ParsedCallCloseStatus::Closed
                    || call.trailing_status != ParsedCallTrailingStatus::Complete
                {
                    valid = false;
                    self.push_invalid(
                        &expression.span,
                        "indirect application must be the complete return expression",
                        "indirect_application_shape_outside_al_v0",
                        vec![callable_param.span.clone()],
                    );
                } else if call.arguments.len() != 1 {
                    valid = false;
                    self.push_mismatch(
                        &expression.span,
                        "indirect application expects exactly one UInt argument",
                        "indirect_argument_count_mismatch_v0",
                        vec![callable_param.type_syntax.span.clone()],
                    );
                } else if !expression_resolves_to(
                    &call.arguments[0],
                    &ordinary_param_definition_id,
                    references,
                ) {
                    valid = false;
                    self.push_invalid(
                        &call.arguments[0].span,
                        "indirect application argument must be the receiver's UInt parameter",
                        "indirect_argument_outside_al_v0",
                        vec![ordinary_param.span.clone()],
                    );
                }
                expression.span.clone()
            });
        let indirect_statement_span = indirect_calls
            .first()
            .map(|(_, _, _, span)| portable_span(span));

        self.definitions.push(CallableDefinitionFact {
            id: semantic_id("callable-definition", &entry.task.span),
            resolver_definition_id: entry.definition_id.clone(),
            lexical_scope_id: entry.callable_scope_id.clone(),
            source_span: portable_span(&entry.task.span),
            input_definition_ids: vec![
                callable_param_definition_id.clone(),
                ordinary_param_definition_id.clone(),
            ],
            input_types: vec!["task(UInt) -> UInt".to_string(), "UInt".to_string()],
            result_type: "UInt".to_string(),
            failure_root: None,
            status: if valid {
                "recognized_al_receiver_v0"
            } else {
                "blocked_callable_receiver_v0"
            },
        });
        self.types.push(CallableTypeFact {
            id: callable_type_id.clone(),
            input_types: vec!["UInt".to_string()],
            result_type: "UInt".to_string(),
            failure_root: None,
            latent_row_id: row_id.clone(),
            status: if valid {
                "recognized_closed_empty_callable_type_v0"
            } else {
                "blocked_callable_type_v0"
            },
        });
        self.rows.push(LatentEffectRowFact {
            id: row_id.clone(),
            labels: Vec::new(),
            normalized_labels: Vec::new(),
            tail_id: None,
            tail_alias: None,
            normalized_tail: None,
            status: if valid {
                "closed_empty_v0"
            } else {
                "blocked_v0"
            },
            origin: "inferred_from_checked_callable_body_v0",
        });

        ReceiverInfo {
            entry,
            callable_param,
            callable_param_definition_id,
            ordinary_param,
            ordinary_param_definition_id,
            callable_type_id,
            row_id,
            indirect_span,
            indirect_statement_span,
            valid,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_direct_application(
        &mut self,
        caller: &TaskEntry<'_>,
        direct_statement_span: &Span,
        expression: &ParsedExpression,
        call: &ParsedCall,
        receiver: &ReceiverInfo<'_>,
        tasks: &[TaskEntry<'_>],
        definitions: &[ResolveDefinitionSummary],
        references: &[ResolveReferenceSummary],
        known_types: &BTreeSet<String>,
    ) {
        if !receiver.valid {
            return;
        }
        if caller.definition_id == receiver.entry.definition_id {
            self.push_invalid(
                &expression.span,
                "recursive callable relationships are outside Session AL",
                "recursive_callable_relationship_unsupported_v0",
                vec![receiver.entry.task.span.clone()],
            );
            return;
        }
        if node_id::source_path_identity(caller.file)
            != node_id::source_path_identity(receiver.entry.file)
        {
            self.push_invalid(
                &expression.span,
                "cross-file callable relationships are outside Session AL",
                "cross_file_callable_value_unsupported_v0",
                vec![caller.task.span.clone(), receiver.entry.task.span.clone()],
            );
            return;
        }
        if call.close_status != ParsedCallCloseStatus::Closed
            || call.trailing_status != ParsedCallTrailingStatus::Complete
            || call.arguments.len() != 2
        {
            self.push_invalid(
                &expression.span,
                "the receiving task call must pass one task value and one UInt value",
                "receiver_call_shape_outside_al_v0",
                vec![receiver.entry.task.span.clone()],
            );
            return;
        }
        if !call.argument_separators_hws_valid {
            self.push_invalid(
                &expression.span,
                "the direct receiver call requires horizontal whitespace after its comma",
                "direct_application_horizontal_whitespace_v0",
                vec![receiver.entry.task.span.clone()],
            );
            return;
        }

        let task_value = &call.arguments[0];
        let Some(identifier) = identifier(task_value) else {
            self.push_invalid(
                &task_value.span,
                "the callable argument must be one visible same-file task identifier",
                "task_value_shape_outside_al_v0",
                vec![receiver.callable_param.span.clone()],
            );
            return;
        };
        let Some(reference) = reference_at(references, &identifier.span, "callable_argument_ref")
        else {
            self.push_unresolved(
                &identifier.span,
                &identifier.name,
                vec![receiver.entry.task.span.clone()],
            );
            return;
        };
        let Some(target_definition_id) = reference.resolved_definition_id.as_deref() else {
            self.push_unresolved(
                &identifier.span,
                &identifier.name,
                vec![receiver.entry.task.span.clone()],
            );
            return;
        };
        let Some(target_definition) = definitions
            .iter()
            .find(|definition| definition.id == target_definition_id)
        else {
            self.push_unresolved(
                &identifier.span,
                &identifier.name,
                vec![receiver.entry.task.span.clone()],
            );
            return;
        };
        if target_definition.definition_kind != "task" {
            self.push_invalid(
                &identifier.span,
                "the callable argument resolves to a non-task value",
                "task_value_resolved_to_non_task_v0",
                vec![target_definition.source_span.clone()],
            );
            return;
        }
        let Some(target) = tasks
            .iter()
            .find(|entry| entry.definition_id == target_definition_id)
        else {
            self.push_invalid(
                &identifier.span,
                "cross-file callable values are outside Session AL",
                "cross_file_callable_value_unsupported_v0",
                vec![target_definition.source_span.clone()],
            );
            return;
        };
        if node_id::source_path_identity(target.file) != node_id::source_path_identity(caller.file)
        {
            self.push_invalid(
                &identifier.span,
                "cross-file callable values are outside Session AL",
                "cross_file_callable_value_unsupported_v0",
                vec![target.task.span.clone()],
            );
            return;
        }
        if target.definition_id == receiver.entry.definition_id
            || target.definition_id == caller.definition_id
        {
            self.push_invalid(
                &identifier.span,
                "recursive callable relationships are outside Session AL",
                "recursive_callable_relationship_unsupported_v0",
                vec![
                    caller.task.span.clone(),
                    receiver.entry.task.span.clone(),
                    target.task.span.clone(),
                ],
            );
            return;
        }
        if task_has_unknown_ordinary_type(target.task, known_types) {
            return;
        }
        let (target_inputs, target_result, target_failure) = task_signature(target.task);
        if target_inputs != ["UInt"] || target_result != "UInt" || target_failure.is_some() {
            let actual = format!(
                "inputs=({}) result={} failure_root={}",
                target_inputs.join(","),
                target_result,
                target_failure.unwrap_or("none")
            );
            self.push_mismatch(
                &identifier.span,
                &format!("expected task(UInt) -> UInt with failure_root=none; actual {actual}"),
                if target_failure.is_some() {
                    "callable_failure_root_mismatch_v0"
                } else if target_inputs != ["UInt"] {
                    "callable_input_type_mismatch_v0"
                } else {
                    "callable_result_type_mismatch_v0"
                },
                vec![
                    target.task.span.clone(),
                    receiver.callable_param.type_syntax.span.clone(),
                ],
            );
            return;
        }
        let change_origins =
            task_change_origins(target, &self.resolver_references, &self.resolver_scopes);
        if !task_has_supported_am_latent_row(target.task, &change_origins) {
            self.push_mismatch(
                &identifier.span,
                "the passed task has a latent effect outside the bounded empty/change row slice",
                "callable_latent_row_outside_bounded_am_slice_v0",
                vec![
                    target.task.span.clone(),
                    receiver.callable_param.type_syntax.span.clone(),
                ],
            );
            return;
        }

        let Some(ordinary_argument) =
            ordinary_argument_fact(&call.arguments[1], definitions, references, tasks)
        else {
            self.push_mismatch(
                &call.arguments[1].span,
                "the receiver's ordinary argument must have exact type UInt",
                "receiver_argument_type_mismatch_v0",
                vec![receiver.ordinary_param.span.clone()],
            );
            return;
        };
        let value_id = semantic_id("callable-value", &identifier.span);
        self.values.push(CallableValueFact {
            id: value_id.clone(),
            source_span: portable_span(&identifier.span),
            resolver_reference_id: reference.id.clone(),
            referring_scope_id: reference.scope_id.clone(),
            resolved_task_definition_id: target_definition_id.to_string(),
            expected_callable_type_id: receiver.callable_type_id.clone(),
            status: "resolved_compatible_task_value_v0",
        });
        let Some(indirect_span) = receiver.indirect_span.clone() else {
            return;
        };
        let application_id = semantic_id("callable-application", &expression.span);
        let mut output_row_id = receiver.row_id.clone();
        let mut application_status = "accepted_al_indirect_application_v0";
        let mut application_reason = "canonical_callable_relationship_checked_v0";
        if !change_origins.is_empty() {
            let tail_id = semantic_id("row-tail", &receiver.callable_param.type_syntax.span);
            if let Some(row) = self.rows.iter_mut().find(|row| row.id == receiver.row_id) {
                row.tail_id = Some(tail_id.clone());
                row.tail_alias = Some("row".to_string());
                row.normalized_tail = Some("row0".to_string());
                row.status = "open_single_tail_v0";
            }
            if let Some(callable_type) = self
                .types
                .iter_mut()
                .find(|fact| fact.id == receiver.callable_type_id)
            {
                callable_type.status = "recognized_open_callable_type_v0";
            }
            let mut occurrence_ids = Vec::new();
            let mut normalized_labels = Vec::new();
            for origin in change_origins {
                let id = semantic_id("effect-label-change", &origin.source_span);
                let alias_id = semantic_id("effect-alias-change", &origin.source_span);
                occurrence_ids.push(id.clone());
                normalized_labels.push("change".to_string());
                if !self
                    .label_occurrences
                    .iter()
                    .any(|occurrence| occurrence.id == id)
                {
                    self.label_occurrences.push(EffectLabelOccurrenceFact {
                        id,
                        label: "change",
                        alias_id,
                        resolver_reference_id: origin.id.clone(),
                        source_span: portable_span(&origin.source_span),
                        owner_definition_id: target.definition_id.clone(),
                        target_definition_id: origin
                            .resolved_definition_id
                            .clone()
                            .unwrap_or_else(|| missing_id("mutation-target", &origin.source_span)),
                    });
                }
            }
            occurrence_ids.sort();
            normalized_labels.sort();
            output_row_id = semantic_id("latent-row-output", &expression.span);
            let argument_row_id = semantic_id("latent-row-argument", &identifier.span);
            self.rows.push(LatentEffectRowFact {
                id: argument_row_id.clone(),
                labels: occurrence_ids.clone(),
                normalized_labels: normalized_labels.clone(),
                tail_id: None,
                tail_alias: None,
                normalized_tail: None,
                status: "closed_nonempty_change_v0",
                origin: "inferred_from_checked_callable_body_v0",
            });
            self.rows.push(LatentEffectRowFact {
                id: output_row_id.clone(),
                labels: occurrence_ids,
                normalized_labels,
                tail_id: None,
                tail_alias: None,
                normalized_tail: None,
                status: "propagated_nonempty_change_v0",
                origin: "substituted_at_callable_application_v0",
            });
            self.substitutions.push(RowSubstitutionFact {
                id: semantic_id("row-substitution", &expression.span),
                application_id: application_id.clone(),
                input_row_id: receiver.row_id.clone(),
                tail_id,
                argument_row_id,
                output_row_id: output_row_id.clone(),
                status: "accepted_single_structural_substitution_v0",
            });
            application_status = "accepted_am_indirect_application_v0";
            application_reason = "complete_latent_row_propagated_v0";
        }
        self.applications.push(CallableApplicationFact {
            id: application_id,
            caller_definition_id: caller.definition_id.clone(),
            caller_span: portable_span(&caller.task.span),
            receiver_definition_id: receiver.entry.definition_id.clone(),
            receiver_span: portable_span(&receiver.entry.task.span),
            callable_parameter_definition_id: receiver.callable_param_definition_id.clone(),
            callable_parameter_name: receiver.callable_param.name.clone(),
            ordinary_parameter_definition_id: receiver.ordinary_param_definition_id.clone(),
            ordinary_parameter_name: receiver.ordinary_param.name.clone(),
            callable_value_id: value_id,
            target_definition_id: target_definition_id.to_string(),
            direct_call_span: portable_span(&expression.span),
            indirect_call_span: portable_span(&indirect_span),
            direct_statement_span: portable_span(direct_statement_span),
            indirect_statement_span: receiver
                .indirect_statement_span
                .clone()
                .unwrap_or_else(|| portable_span(&indirect_span)),
            ordinary_argument,
            result_type: "UInt".to_string(),
            failure_root: None,
            input_row_id: receiver.row_id.clone(),
            output_row_id,
            status: application_status,
            reason: application_reason,
        });
    }

    fn push_invalid(
        &mut self,
        span: &Span,
        message: &str,
        detail_reason: &'static str,
        related: Vec<Span>,
    ) {
        self.push_diagnostic(DiagnosticCode::INVALID_CALLABLE_FORM, span, message, detail_reason, related,
            "Use exactly one same-file task(UInt) -> UInt value, pass it to one receiver, and invoke its callable parameter exactly once as the complete return expression.");
    }

    fn push_mismatch(
        &mut self,
        span: &Span,
        message: &str,
        detail_reason: &'static str,
        related: Vec<Span>,
    ) {
        self.push_diagnostic(DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH, span, message, detail_reason, related,
            "Make the task value, callable parameter, ordinary argument, result, and failure_root=none match exactly.");
    }

    fn push_unresolved(&mut self, span: &Span, name: &str, related: Vec<Span>) {
        self.push_diagnostic(
            DiagnosticCode::UNRESOLVED_NAME,
            span,
            &format!("name `{name}` is not visible in this callable relationship"),
            "callable_task_value_unresolved_v0",
            related,
            "Declare or name one visible same-file task before passing it as the callable value.",
        );
    }

    fn push_diagnostic(
        &mut self,
        code: DiagnosticCode,
        span: &Span,
        message: &str,
        detail_reason: &'static str,
        related: Vec<Span>,
        help: &'static str,
    ) {
        let id = semantic_id(
            &format!("callable-diagnostic-{}-{detail_reason}", code.as_str()),
            span,
        );
        let mut diagnostic =
            Diagnostic::error(code, message, Some(portable_span(span))).with_help(help);
        let mut relationship_sites = vec![portable_span(span)];
        for (index, related_span) in related.into_iter().enumerate() {
            relationship_sites.push(portable_span(&related_span));
            diagnostic = diagnostic.with_related_span(
                format!("callable relationship site {}", index + 1),
                portable_span(&related_span),
            );
        }
        let occurrence =
            crate::diagnostic_catalog::diagnostic_cause(code, detail_reason).map(|cause| {
                let mut semantic_links = Vec::new();
                for site in &relationship_sites {
                    for definition in &self.resolver_definitions {
                        if same_span(&definition.source_span, site) {
                            semantic_links.push(format!("definition={}", definition.id));
                        }
                    }
                    for reference in &self.resolver_references {
                        if same_span(&reference.source_span, site) {
                            semantic_links.push(format!("reference={}", reference.id));
                        }
                    }
                }
                let identity = DiagnosticOccurrence::callable_identity(
                    detail_reason,
                    &portable_span(span),
                    &relationship_sites,
                    semantic_links,
                );
                DiagnosticOccurrence::registered(cause, identity, diagnostic.clone())
                    .expect("callable occurrence must satisfy its registered cause")
            });
        self.diagnostics.push(CallableDiagnosticFact {
            id,
            reason: match code.as_str() {
                "H1401" => "invalid_or_unsupported_callable_form_v0",
                "H1402" => "callable_signature_mismatch_v0",
                _ => detail_reason,
            },
            detail_reason,
            fallback_diagnostic: occurrence.is_none().then_some(diagnostic),
            occurrence,
            relationship_sites,
        });
    }

    fn diagnostics_for_stage(&self, stage: &str) -> Vec<CallableDiagnosticFact> {
        let signature_ready = !matches!(stage, "resolve" | "type_env" | "syntax");
        self.diagnostics
            .iter()
            .filter(|fact| {
                fact.diagnostic().code != DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH
                    || signature_ready
            })
            .filter(|fact| !self.prior_owns(fact, &[], &[], &[]))
            .cloned()
            .collect()
    }

    fn prior_owns(
        &self,
        fact: &CallableDiagnosticFact,
        prior: &[Diagnostic],
        canonical_unknown_types: &[Diagnostic],
        canonical_paths: &[Diagnostic],
    ) -> bool {
        let Some(candidate) = fact.occurrence.as_ref() else {
            return false;
        };
        let typed_failure_owns = self.typed_failure_prior.iter().any(|existing| {
            existing.diagnostic().severity == Severity::Error
                && crate::diagnostic_catalog::precedence_spec(
                    existing.cause_key(),
                    candidate.cause_key(),
                )
                .is_some_and(|rule| {
                    rule.applying_owner == "callable_analysis"
                        && rule.relationship == "shared_callable_relationship_site_v0"
                        && existing.relationship_route().iter().any(|existing_site| {
                            existing_site.starts_with("semantic_site=")
                                && candidate
                                    .relationship_route()
                                    .iter()
                                    .any(|candidate_site| candidate_site == existing_site)
                        })
                })
        });
        typed_failure_owns
            || prior.iter().any(|existing| {
                let Some((dominant_cause, semantic_site)) =
                    canonical_prior_cause(existing, canonical_unknown_types, canonical_paths)
                else {
                    return false;
                };
                existing.severity == Severity::Error
                    && crate::diagnostic_catalog::precedence_spec(
                        dominant_cause.key,
                        candidate.cause_key(),
                    )
                    .is_some_and(|rule| {
                        rule.applying_owner == "callable_analysis"
                            && rule.relationship == "shared_callable_relationship_site_v0"
                            && candidate.relationship_route().contains(&semantic_site)
                    })
            })
    }

    pub(crate) fn direct_application(
        &self,
        task: &Task,
        span: &Span,
    ) -> Option<&CallableApplicationFact> {
        let caller_id = self.definition_id_for_task(task)?;
        self.applications.iter().find(|fact| {
            fact.caller_definition_id == caller_id && same_span(&fact.direct_statement_span, span)
        })
    }

    pub(crate) fn indirect_application(
        &self,
        task: &Task,
        span: &Span,
    ) -> Option<&CallableApplicationFact> {
        let receiver_id = self.definition_id_for_task(task)?;
        self.applications.iter().find(|fact| {
            fact.receiver_definition_id == receiver_id
                && same_span(&fact.indirect_statement_span, span)
        })
    }

    pub(crate) fn indirect_application_with_id(
        &self,
        task: &Task,
        span: &Span,
        id: &str,
    ) -> Option<&CallableApplicationFact> {
        let receiver_id = self.definition_id_for_task(task)?;
        self.applications.iter().find(|fact| {
            fact.id == id
                && fact.receiver_definition_id == receiver_id
                && same_span(&fact.indirect_statement_span, span)
        })
    }

    pub(crate) fn definition_id_for_task(&self, task: &Task) -> Option<&str> {
        self.definitions
            .iter()
            .find(|fact| same_span(&fact.source_span, &task.span))
            .map(|fact| fact.resolver_definition_id.as_str())
            .or_else(|| {
                self.definition_spans
                    .iter()
                    .find(|(_id, span)| same_span(span, &task.span))
                    .map(|(id, _span)| id.as_str())
            })
    }

    pub(crate) fn definition_id_for_span(&self, span: &Span) -> Option<&str> {
        self.definition_spans
            .iter()
            .find(|(_id, definition_span)| same_span(definition_span, span))
            .map(|(id, _span)| id.as_str())
    }

    pub(crate) fn callable_argument_target_definition_ids(&self, task: &Task) -> Vec<&str> {
        let Some(scope) = self.resolver_scopes.iter().find(|scope| {
            scope.scope_kind == "callable"
                && scope.owner_kind == "task"
                && scope
                    .source_span
                    .as_ref()
                    .is_some_and(|span| same_span(span, &task.span))
        }) else {
            return Vec::new();
        };
        self.resolver_references
            .iter()
            .filter(|reference| {
                reference.scope_id == scope.id
                    && reference.reference_kind == "callable_argument_ref"
            })
            .filter_map(|reference| reference.resolved_definition_id.as_deref())
            .filter(|definition_id| {
                self.resolver_definitions.iter().any(|definition| {
                    definition.id == *definition_id && definition.definition_kind == "task"
                })
            })
            .collect()
    }

    pub(crate) fn callable_callee_target_definition_ids(
        &self,
        task: &Task,
        statement_span: &Span,
    ) -> Vec<&str> {
        let Some(scope) = self.resolver_scopes.iter().find(|scope| {
            scope.scope_kind == "callable"
                && scope.owner_kind == "task"
                && scope
                    .source_span
                    .as_ref()
                    .is_some_and(|span| same_span(span, &task.span))
        }) else {
            return Vec::new();
        };
        self.resolver_references
            .iter()
            .filter(|reference| {
                reference.scope_id == scope.id
                    && reference.reference_kind == "callable_callee_ref"
                    && same_line(&reference.source_span, statement_span)
            })
            .filter_map(|reference| reference.resolved_definition_id.as_deref())
            .filter(|definition_id| {
                self.resolver_definitions.iter().any(|definition| {
                    definition.id == *definition_id && definition.definition_kind == "task"
                })
            })
            .collect()
    }

    pub(crate) fn task_participates(&self, task: &Task) -> bool {
        let Some(definition_id) = self.definition_id_for_task(task) else {
            return false;
        };
        self.applications.iter().any(|application| {
            application.caller_definition_id == definition_id
                || application.receiver_definition_id == definition_id
                || application.target_definition_id == definition_id
        })
    }

    pub(crate) fn bridge_status(&self) -> &'static str {
        if self.substitutions.is_empty() {
            "not_applicable_to_al_ordinary_value_v0"
        } else {
            "not_applicable_to_am_ordinary_value_v0"
        }
    }

    pub(crate) fn is_nonretained_closed_empty_task_definition(&self, task: &Task) -> bool {
        task.params.len() == 1
            && is_named(&task.params[0].type_syntax, "UInt")
            && task
                .result_syntax
                .as_ref()
                .is_some_and(|result| is_named(result, "UInt"))
            && task_has_closed_empty_latent_row(task)
    }

    pub(crate) fn text(&self, stage: &str) -> String {
        let diagnostics = self.diagnostics_for_stage(stage);
        let bridge_status = self.bridge_status();
        let mut out = format!(
            "callable_facts: model={} stage={} definitions={} types={} rows={} label_occurrences={} substitutions={} values={} applications={} diagnostics={} status={}\n",
            CALLABLE_FACT_MODEL,
            stage,
            self.definitions.len(),
            self.types.len(),
            self.rows.len(),
            self.label_occurrences.len(),
            self.substitutions.len(),
            self.values.len(),
            self.applications.len(),
            diagnostics.len(),
            if !diagnostics.is_empty() {
                "blocked_v0"
            } else if self.types.is_empty() {
                "not_applicable_v0"
            } else {
                if self.substitutions.is_empty() {
                    "accepted_al_v0"
                } else {
                    "accepted_am_v0"
                }
            }
        );
        for fact in &self.definitions {
            out.push_str(&format!("  definition id={} resolver_definition={} scope={} inputs=[{}] result={} failure_root={} status={} span={}:{}:{}\n",
                fact.id, fact.resolver_definition_id, fact.lexical_scope_id, fact.input_types.join(","), fact.result_type,
                fact.failure_root.as_deref().unwrap_or("none"), fact.status, fact.source_span.file, fact.source_span.line, fact.source_span.column));
        }
        for fact in &self.types {
            out.push_str(&format!(
                "  type id={} inputs=[{}] result={} failure_root={} row={} status={}\n",
                fact.id,
                fact.input_types.join(","),
                fact.result_type,
                fact.failure_root.as_deref().unwrap_or("none"),
                fact.latent_row_id,
                fact.status
            ));
        }
        for row in &self.rows {
            out.push_str(&format!(
                "  row id={} labels=[{}] normalized_labels=[{}] tail={} tail_alias={} normalized_tail={} status={} origin={}\n",
                row.id,
                row.labels.join(","),
                row.normalized_labels.join(","),
                row.tail_id.as_deref().unwrap_or("none"),
                row.tail_alias.as_deref().unwrap_or("none"),
                row.normalized_tail.as_deref().unwrap_or("none"),
                row.status,
                row.origin
            ));
        }
        for fact in &self.label_occurrences {
            out.push_str(&format!(
                "  label_occurrence id={} label={} alias={} resolver_reference={} owner={} target={} span={}:{}:{}\n",
                fact.id,
                fact.label,
                fact.alias_id,
                fact.resolver_reference_id,
                fact.owner_definition_id,
                fact.target_definition_id,
                fact.source_span.file,
                fact.source_span.line,
                fact.source_span.column
            ));
        }
        for fact in &self.substitutions {
            out.push_str(&format!(
                "  row_substitution id={} application={} input_row={} tail={} argument_row={} output_row={} status={}\n",
                fact.id, fact.application_id, fact.input_row_id, fact.tail_id,
                fact.argument_row_id, fact.output_row_id, fact.status
            ));
        }
        for fact in &self.values {
            out.push_str(&format!(
                "  value id={} reference={} referring_scope={} target={} expected_type={} status={} span={}:{}:{}\n",
                fact.id,
                fact.resolver_reference_id,
                fact.referring_scope_id,
                fact.resolved_task_definition_id,
                fact.expected_callable_type_id,
                fact.status,
                fact.source_span.file,
                fact.source_span.line,
                fact.source_span.column
            ));
        }
        for fact in &self.applications {
            out.push_str(&format!("  application id={} caller={} receiver={} target={} callable_parameter={} ordinary_parameter={} input_row={} output_row={} result={} failure_root={} status={} reason={} direct_span={}:{}:{} indirect_span={}:{}:{}\n",
                fact.id, fact.caller_definition_id, fact.receiver_definition_id, fact.target_definition_id,
                fact.callable_parameter_definition_id, fact.ordinary_parameter_definition_id, fact.input_row_id, fact.output_row_id,
                fact.result_type, fact.failure_root.as_deref().unwrap_or("none"), fact.status, fact.reason,
                fact.direct_call_span.file, fact.direct_call_span.line, fact.direct_call_span.column,
                fact.indirect_call_span.file, fact.indirect_call_span.line, fact.indirect_call_span.column));
        }
        for fact in &diagnostics {
            let span = fact
                .diagnostic()
                .span
                .as_ref()
                .expect("callable diagnostic span");
            out.push_str(&format!(
                "  diagnostic id={} code={} reason={} detail_reason={} message={} help={} primary_span={}:{}:{} related={}\n",
                fact.id,
                fact.diagnostic().code.as_str(),
                fact.reason,
                fact.detail_reason,
                fact.diagnostic().message,
                fact.diagnostic().help.as_deref().unwrap_or(""),
                span.file,
                span.line,
                span.column,
                fact.diagnostic().related_spans.len()
            ));
            for related in &fact.diagnostic().related_spans {
                out.push_str(&format!(
                    "    related label={} span={}:{}:{}\n",
                    related.label, related.span.file, related.span.line, related.span.column
                ));
            }
        }
        if !self.types.is_empty() {
            out.push_str(&format!("  bridge ownership={bridge_status} resource={bridge_status} callable_environment_allocations=0 retained_values=0 retained_resources=0 retained_authority=0\n"));
        }
        if stage.starts_with("core_") {
            for fact in &self.types {
                out.push_str(&format!(
                    "  core_node kind=callable_type id={} row={} result={} failure_root={}\n",
                    fact.id,
                    fact.latent_row_id,
                    fact.result_type,
                    fact.failure_root.as_deref().unwrap_or("none")
                ));
            }
            for fact in &self.values {
                out.push_str(&format!(
                    "  core_node kind=callable_value id={} target={} reference={}\n",
                    fact.id, fact.resolved_task_definition_id, fact.resolver_reference_id
                ));
            }
            for fact in &self.applications {
                out.push_str(&format!(
                    "  core_node kind=callable_application id={} value={} input_row={} output_row={} result={}\n",
                    fact.id, fact.callable_value_id, fact.input_row_id, fact.output_row_id, fact.result_type
                ));
            }
        }
        out.push_str("  nonclaims: no_capture no_environment no_general_allocation_proof no_ownership_proof no_authority_proof no_handler no_global_principal_inference\n");
        out
    }

    pub(crate) fn json(&self, stage: &str) -> String {
        let diagnostics = self.diagnostics_for_stage(stage);
        let bridge_status = self.bridge_status();
        let mut out = String::new();
        out.push_str("{\n");
        json_string_field(&mut out, 2, "model", CALLABLE_FACT_MODEL, true);
        json_string_field(&mut out, 2, "stage", stage, true);
        json_string_field(
            &mut out,
            2,
            "status",
            if !diagnostics.is_empty() {
                "blocked_v0"
            } else if self.types.is_empty() {
                "not_applicable_v0"
            } else {
                if self.substitutions.is_empty() {
                    "accepted_al_v0"
                } else {
                    "accepted_am_v0"
                }
            },
            true,
        );
        json_usize_field(&mut out, 2, "definitions", self.definitions.len(), true);
        json_usize_field(&mut out, 2, "types", self.types.len(), true);
        json_usize_field(&mut out, 2, "rows", self.rows.len(), true);
        json_usize_field(
            &mut out,
            2,
            "label_occurrences",
            self.label_occurrences.len(),
            true,
        );
        json_usize_field(&mut out, 2, "substitutions", self.substitutions.len(), true);
        json_usize_field(&mut out, 2, "values", self.values.len(), true);
        json_usize_field(&mut out, 2, "applications", self.applications.len(), true);
        json_usize_field(&mut out, 2, "diagnostics", diagnostics.len(), true);
        json_string_field(
            &mut out,
            2,
            "ownership_status",
            if self.types.is_empty() {
                "not_applicable_v0"
            } else {
                bridge_status
            },
            true,
        );
        json_string_field(
            &mut out,
            2,
            "resource_status",
            if self.types.is_empty() {
                "not_applicable_v0"
            } else {
                bridge_status
            },
            true,
        );
        json_usize_field(&mut out, 2, "callable_environment_allocations", 0, true);
        json_usize_field(&mut out, 2, "retained_values", 0, true);
        json_usize_field(&mut out, 2, "retained_resources", 0, true);
        json_usize_field(&mut out, 2, "retained_authority", 0, true);
        push_json_definitions(&mut out, &self.definitions, true);
        push_json_types(&mut out, &self.types, true);
        push_json_rows(&mut out, &self.rows, true);
        push_json_label_occurrences(&mut out, &self.label_occurrences, true);
        push_json_substitutions(&mut out, &self.substitutions, true);
        push_json_values(&mut out, &self.values, true);
        push_json_applications(&mut out, &self.applications, true);
        push_json_diagnostics(&mut out, &diagnostics, true);
        if stage.starts_with("core_") {
            push_json_core_nodes(&mut out, self, false);
        } else if stage == "graph" {
            push_json_graph_edges(&mut out, self, false);
        } else {
            json_indent(&mut out, 2);
            out.push_str("\"stage_relationships\": []\n");
        }
        out.push_str("}\n");
        out
    }

    pub(crate) fn verify(&self) -> Vec<&'static str> {
        let mut failures = Vec::new();
        let definition_ids = self
            .definitions
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        let type_ids = self
            .types
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        let row_ids = self
            .rows
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        let label_ids = self
            .label_occurrences
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        let label_alias_ids = self
            .label_occurrences
            .iter()
            .map(|fact| fact.alias_id.as_str())
            .collect::<BTreeSet<_>>();
        let substitution_ids = self
            .substitutions
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        let value_ids = self
            .values
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        let application_ids = self
            .applications
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        let diagnostic_ids = self
            .diagnostics
            .iter()
            .map(|fact| fact.id.as_str())
            .collect::<BTreeSet<_>>();
        if definition_ids.len() != self.definitions.len()
            || type_ids.len() != self.types.len()
            || row_ids.len() != self.rows.len()
            || label_ids.len() != self.label_occurrences.len()
            || label_alias_ids.len() != self.label_occurrences.len()
            || substitution_ids.len() != self.substitutions.len()
            || value_ids.len() != self.values.len()
            || application_ids.len() != self.applications.len()
            || diagnostic_ids.len() != self.diagnostics.len()
        {
            failures.push("callable_fact_identity_not_unique_v0");
        }
        let application_receivers = self
            .applications
            .iter()
            .map(|fact| fact.receiver_definition_id.as_str())
            .collect::<BTreeSet<_>>();
        let substituted_receivers = self
            .substitutions
            .iter()
            .filter_map(|substitution| {
                self.applications
                    .iter()
                    .find(|application| application.id == substitution.application_id)
                    .map(|application| application.receiver_definition_id.as_str())
            })
            .collect::<BTreeSet<_>>();
        if application_receivers.len() != self.applications.len()
            || substituted_receivers.len() != self.substitutions.len()
        {
            failures.push("callable_multiple_relationships_outside_am_v0");
        }
        if self.definitions != self.canonical_definitions
            || self.types != self.canonical_types
            || self.rows != self.canonical_rows
            || self.label_occurrences != self.canonical_label_occurrences
            || self.substitutions != self.canonical_substitutions
            || self.values != self.canonical_values
            || self.applications != self.canonical_applications
            || self.diagnostics != self.canonical_diagnostics
        {
            failures.push("callable_fact_source_relationship_corrupt_v0");
        }
        for fact in &self.definitions {
            if fact.id.is_empty()
                || fact.resolver_definition_id.is_empty()
                || fact.lexical_scope_id.is_empty()
                || fact.input_definition_ids.len() != 2
                || fact.input_definition_ids.iter().any(String::is_empty)
                || fact.input_types != ["task(UInt) -> UInt", "UInt"]
                || fact.result_type != "UInt"
                || fact.failure_root.is_some()
                || !matches!(
                    fact.status,
                    "recognized_al_receiver_v0" | "blocked_callable_receiver_v0"
                )
            {
                failures.push("callable_definition_signature_corrupt_v0");
            }
            if fact.id != semantic_id("callable-definition", &fact.source_span) {
                failures.push("callable_definition_identity_corrupt_v0");
            }
            let exact_definition = self.resolver_definitions.iter().find(|definition| {
                definition.id == fact.resolver_definition_id
                    && definition.definition_kind == "task"
                    && same_span(&definition.source_span, &fact.source_span)
            });
            let exact_scope = self.resolver_scopes.iter().find(|scope| {
                scope.id == fact.lexical_scope_id
                    && scope.scope_kind == "callable"
                    && scope.owner_kind == "task"
                    && scope
                        .source_span
                        .as_ref()
                        .is_some_and(|span| same_span(span, &fact.source_span))
            });
            let expected_inputs = exact_scope.map(|scope| {
                let mut inputs = self
                    .resolver_definitions
                    .iter()
                    .filter(|definition| {
                        definition.definition_kind == "parameter" && definition.scope_id == scope.id
                    })
                    .collect::<Vec<_>>();
                inputs.sort_by_key(|definition| {
                    (definition.source_span.line, definition.source_span.column)
                });
                inputs
                    .into_iter()
                    .map(|definition| definition.id.clone())
                    .collect::<Vec<_>>()
            });
            if exact_definition.is_none()
                || exact_scope.is_none()
                || expected_inputs.as_ref() != Some(&fact.input_definition_ids)
            {
                failures.push("callable_definition_resolver_relationship_corrupt_v0");
            }
        }
        for row in &self.rows {
            let valid = match row.status {
                "closed_empty_v0" | "blocked_v0" => {
                    row.labels.is_empty()
                        && row.normalized_labels.is_empty()
                        && row.tail_id.is_none()
                        && row.tail_alias.is_none()
                        && row.normalized_tail.is_none()
                }
                "open_single_tail_v0" => {
                    row.labels.is_empty()
                        && row.normalized_labels.is_empty()
                        && row.tail_id.is_some()
                        && row.tail_alias.as_deref() == Some("row")
                        && row.normalized_tail.as_deref() == Some("row0")
                }
                "closed_nonempty_change_v0" | "propagated_nonempty_change_v0" => {
                    !row.labels.is_empty()
                        && row.labels.iter().all(|id| label_ids.contains(id.as_str()))
                        && row.normalized_labels.len() == row.labels.len()
                        && row.normalized_labels.iter().all(|label| label == "change")
                        && row.tail_id.is_none()
                        && row.tail_alias.is_none()
                        && row.normalized_tail.is_none()
                }
                _ => false,
            };
            if !valid
                || !matches!(
                    row.origin,
                    "inferred_from_checked_callable_body_v0"
                        | "substituted_at_callable_application_v0"
                )
            {
                failures.push("callable_latent_row_relationship_corrupt_v0");
            }
        }
        for fact in &self.label_occurrences {
            let exact_reference = self.resolver_references.iter().find(|reference| {
                reference.id == fact.resolver_reference_id
                    && reference.reference_kind == "mutation_target"
                    && same_span(&reference.source_span, &fact.source_span)
                    && reference.resolved_definition_id.as_deref()
                        == Some(fact.target_definition_id.as_str())
            });
            let owner_scope = self.resolver_scopes.iter().find(|scope| {
                scope.scope_kind == "callable"
                    && scope.owner_kind == "task"
                    && self.resolver_definitions.iter().any(|definition| {
                        definition.id == fact.owner_definition_id
                            && scope
                                .source_span
                                .as_ref()
                                .is_some_and(|span| same_span(span, &definition.source_span))
                    })
            });
            if fact.id != semantic_id("effect-label-change", &fact.source_span)
                || fact.label != "change"
                || fact.alias_id != semantic_id("effect-alias-change", &fact.source_span)
                || exact_reference.is_none()
                || owner_scope.is_none()
                || !exact_reference.is_some_and(|reference| {
                    owner_scope.is_some_and(|scope| {
                        scope_is_within(&reference.scope_id, &scope.id, &self.resolver_scopes)
                    })
                })
            {
                failures.push("callable_effect_label_occurrence_corrupt_v0");
            }
        }
        for fact in &self.types {
            if !row_ids.contains(fact.latent_row_id.as_str()) {
                failures.push("callable_type_missing_latent_row_v0");
            }
            if fact.id.is_empty()
                || fact.input_types != ["UInt"]
                || fact.result_type != "UInt"
                || fact.failure_root.is_some()
                || !matches!(
                    fact.status,
                    "recognized_closed_empty_callable_type_v0"
                        | "recognized_open_callable_type_v0"
                        | "blocked_callable_type_v0"
                )
            {
                failures.push("callable_type_signature_corrupt_v0");
            }
            let expected_row_status = match fact.status {
                "recognized_closed_empty_callable_type_v0" => "closed_empty_v0",
                "recognized_open_callable_type_v0" => "open_single_tail_v0",
                _ => "blocked_v0",
            };
            if self
                .rows
                .iter()
                .find(|row| row.id == fact.latent_row_id)
                .is_some_and(|row| row.status != expected_row_status)
            {
                failures.push("callable_type_row_status_mismatch_v0");
            }
        }
        for fact in &self.values {
            if !type_ids.contains(fact.expected_callable_type_id.as_str()) {
                failures.push("callable_value_missing_type_v0");
            }
            if !self
                .definition_names
                .contains_key(&fact.resolved_task_definition_id)
            {
                failures.push("callable_value_missing_target_definition_v0");
            }
            if fact.id.is_empty()
                || fact.resolver_reference_id.is_empty()
                || fact.referring_scope_id.is_empty()
                || fact.resolved_task_definition_id.is_empty()
                || fact.expected_callable_type_id.is_empty()
                || fact.status != "resolved_compatible_task_value_v0"
            {
                failures.push("callable_value_fact_corrupt_v0");
            }
            if fact.id != semantic_id("callable-value", &fact.source_span) {
                failures.push("callable_value_identity_corrupt_v0");
            }
            let exact_reference = self.resolver_references.iter().find(|reference| {
                reference.id == fact.resolver_reference_id
                    && reference.reference_kind == "callable_argument_ref"
                    && reference.scope_id == fact.referring_scope_id
                    && same_span(&reference.source_span, &fact.source_span)
                    && reference.resolved_definition_id.as_deref()
                        == Some(fact.resolved_task_definition_id.as_str())
            });
            if exact_reference.is_none() {
                failures.push("callable_value_resolver_relationship_corrupt_v0");
            }
        }
        for fact in &self.applications {
            if !value_ids.contains(fact.callable_value_id.as_str()) {
                failures.push("callable_application_missing_value_v0");
            }
            if !row_ids.contains(fact.input_row_id.as_str())
                || !row_ids.contains(fact.output_row_id.as_str())
            {
                failures.push("callable_application_missing_row_v0");
            }
            let is_al = fact.status == "accepted_al_indirect_application_v0";
            let is_am = fact.status == "accepted_am_indirect_application_v0";
            if (is_al && fact.input_row_id != fact.output_row_id)
                || (is_am && fact.input_row_id == fact.output_row_id)
            {
                failures.push("callable_application_row_identity_mismatch_v0");
            }
            if fact.result_type != "UInt" || fact.failure_root.is_some() {
                failures.push("callable_application_signature_corrupt_v0");
            }
            if fact.id.is_empty()
                || fact.callable_parameter_name.is_empty()
                || fact.ordinary_parameter_name.is_empty()
                || !(is_al || is_am)
                || (is_al && fact.reason != "canonical_callable_relationship_checked_v0")
                || (is_am && fact.reason != "complete_latent_row_propagated_v0")
            {
                failures.push("callable_application_fact_corrupt_v0");
            }
            if fact.id != semantic_id("callable-application", &fact.direct_call_span) {
                failures.push("callable_application_identity_corrupt_v0");
            }
            if [
                &fact.caller_definition_id,
                &fact.receiver_definition_id,
                &fact.target_definition_id,
                &fact.callable_parameter_definition_id,
                &fact.ordinary_parameter_definition_id,
            ]
            .iter()
            .any(|definition_id| !self.definition_names.contains_key(*definition_id))
            {
                failures.push("callable_application_missing_definition_v0");
            }
            if self
                .values
                .iter()
                .find(|value| value.id == fact.callable_value_id)
                .is_some_and(|value| value.resolved_task_definition_id != fact.target_definition_id)
            {
                failures.push("callable_application_target_identity_mismatch_v0");
            }
            let exact_caller = self.resolver_definitions.iter().any(|definition| {
                definition.id == fact.caller_definition_id
                    && definition.definition_kind == "task"
                    && same_span(&definition.source_span, &fact.caller_span)
            });
            let exact_receiver = self.resolver_definitions.iter().any(|definition| {
                definition.id == fact.receiver_definition_id
                    && definition.definition_kind == "task"
                    && same_span(&definition.source_span, &fact.receiver_span)
            });
            let exact_value = self.values.iter().any(|value| {
                value.id == fact.callable_value_id
                    && value.resolved_task_definition_id == fact.target_definition_id
                    && value.referring_scope_id
                        == self
                            .resolver_references
                            .iter()
                            .find(|reference| reference.id == value.resolver_reference_id)
                            .map_or("", |reference| reference.scope_id.as_str())
            });
            let receiver_scope = self.resolver_scopes.iter().find(|scope| {
                scope.scope_kind == "callable"
                    && scope.owner_kind == "task"
                    && scope
                        .source_span
                        .as_ref()
                        .is_some_and(|span| same_span(span, &fact.receiver_span))
            });
            let parameters_exact = receiver_scope.is_some_and(|scope| {
                self.resolver_definitions.iter().any(|definition| {
                    definition.id == fact.callable_parameter_definition_id
                        && definition.definition_kind == "parameter"
                        && definition.scope_id == scope.id
                }) && self.resolver_definitions.iter().any(|definition| {
                    definition.id == fact.ordinary_parameter_definition_id
                        && definition.definition_kind == "parameter"
                        && definition.scope_id == scope.id
                })
            });
            let direct_reference_exact = self.resolver_references.iter().any(|reference| {
                reference.reference_kind == "callable_callee_ref"
                    && same_span(&reference.source_span, &fact.direct_call_span)
                    && reference.resolved_definition_id.as_deref()
                        == Some(fact.receiver_definition_id.as_str())
            });
            let indirect_reference_exact = self.resolver_references.iter().any(|reference| {
                reference.reference_kind == "callable_callee_ref"
                    && same_span(&reference.source_span, &fact.indirect_call_span)
                    && reference.resolved_definition_id.as_deref()
                        == Some(fact.callable_parameter_definition_id.as_str())
            });
            if !exact_caller
                || !exact_receiver
                || !exact_value
                || !parameters_exact
                || !direct_reference_exact
                || !indirect_reference_exact
            {
                failures.push("callable_application_resolver_relationship_corrupt_v0");
            }
        }
        for fact in &self.substitutions {
            let input = self.rows.iter().find(|row| row.id == fact.input_row_id);
            let argument = self.rows.iter().find(|row| row.id == fact.argument_row_id);
            let output = self.rows.iter().find(|row| row.id == fact.output_row_id);
            let application = self
                .applications
                .iter()
                .find(|app| app.id == fact.application_id);
            let mut expected_occurrence_ids = application.map_or_else(Vec::new, |application| {
                self.label_occurrences
                    .iter()
                    .filter(|label| label.owner_definition_id == application.target_definition_id)
                    .map(|label| label.id.clone())
                    .collect::<Vec<_>>()
            });
            expected_occurrence_ids.sort();
            if fact.id.is_empty()
                || application.is_some_and(|application| {
                    fact.id != semantic_id("row-substitution", &application.direct_call_span)
                        || fact.input_row_id != application.input_row_id
                        || fact.output_row_id != application.output_row_id
                })
                || fact.status != "accepted_single_structural_substitution_v0"
                || input.and_then(|row| row.tail_id.as_deref()) != Some(fact.tail_id.as_str())
                || argument.is_none()
                || output.is_none()
                || application.is_none()
                || argument.map(|row| &row.labels) != output.map(|row| &row.labels)
                || argument.map(|row| &row.normalized_labels)
                    != output.map(|row| &row.normalized_labels)
                || argument.map(|row| &row.labels) != Some(&expected_occurrence_ids)
                || output.map(|row| &row.labels) != Some(&expected_occurrence_ids)
            {
                failures.push("callable_row_substitution_relationship_corrupt_v0");
            }
        }
        for fact in &self.diagnostics {
            let Some(span) = fact.diagnostic().span.as_ref() else {
                failures.push("callable_diagnostic_identity_corrupt_v0");
                continue;
            };
            let expected = semantic_id(
                &format!(
                    "callable-diagnostic-{}-{}",
                    fact.diagnostic().code.as_str(),
                    fact.detail_reason
                ),
                span,
            );
            if fact.id != expected {
                failures.push("callable_diagnostic_identity_corrupt_v0");
            }
            if matches!(
                fact.diagnostic().code,
                DiagnosticCode::INVALID_CALLABLE_FORM | DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH
            ) && fact
                .occurrence
                .as_ref()
                .is_none_or(|occurrence| occurrence.validate().is_err())
            {
                failures.push("callable_diagnostic_occurrence_corrupt_v0");
            }
        }
        let mut occurrence_collector = DiagnosticOccurrenceCollector::default();
        for occurrence in self
            .diagnostics
            .iter()
            .filter_map(|fact| fact.occurrence.clone())
        {
            if occurrence_collector.insert(occurrence).is_err() {
                failures.push("callable_diagnostic_occurrence_corrupt_v0");
            }
        }
        failures.sort_unstable();
        failures.dedup();
        failures
    }
}

pub(crate) fn append_text(out: &mut String, program: &Program, stage: &str) {
    out.push_str(&analyze_program(program).text(stage));
}

pub(crate) fn inject_json(mut out: String, program: &Program, stage: &str) -> String {
    let field = format!(
        "  \"callable_facts\": {}",
        indent_json(&analyze_program(program).json(stage), 2).trim_start()
    );
    if let Some(close) = out.rfind("\n}") {
        let prefix = &out[..close];
        let comma = if prefix.trim_end().ends_with(',') {
            ""
        } else {
            ","
        };
        out.insert_str(close, &format!("{comma}\n{field}"));
    }
    out
}

fn canonical_prior_cause(
    diagnostic: &Diagnostic,
    canonical_unknown_types: &[Diagnostic],
    canonical_paths: &[Diagnostic],
) -> Option<(
    &'static crate::diagnostic_catalog::DiagnosticCauseSpec,
    String,
)> {
    let span = diagnostic.span.as_ref()?;
    let (reason, role) = match diagnostic.code {
        DiagnosticCode::ITEM_HEADER_MISSING_OPEN_BRACE => (
            "callable_precedence_item_header_missing_open_brace_v0",
            "expression",
        ),
        DiagnosticCode::UNEXPECTED_SIGNATURE_TEXT => (
            "callable_precedence_unexpected_signature_text_v0",
            "expression",
        ),
        DiagnosticCode::CALLABLE_SIGNATURE_MISSING_CLOSE_PAREN => {
            ("callable_precedence_missing_close_paren_v0", "expression")
        }
        DiagnosticCode::PARAMETER_MISSING_TYPE => (
            "callable_precedence_parameter_missing_type_v0",
            "expression",
        ),
        DiagnosticCode::INVALID_IDENTIFIER => {
            ("callable_precedence_invalid_identifier_v0", "expression")
        }
        DiagnosticCode::DUPLICATE_NAME_IN_SCOPE => {
            ("callable_precedence_duplicate_name_v0", "definition")
        }
        DiagnosticCode::UNKNOWN_TYPE_NAME
            if canonical_unknown_types
                .iter()
                .any(|canonical| canonical == diagnostic) =>
        {
            (
                "callable_precedence_unknown_type_relationship_v0",
                "unknown_type",
            )
        }
        DiagnosticCode::PATH_SOURCE_CONSTRUCTION
            if canonical_paths
                .iter()
                .any(|canonical| canonical == diagnostic) =>
        {
            (
                "callable_precedence_opaque_path_relationship_v0",
                "opaque_path",
            )
        }
        _ => return None,
    };
    let cause = crate::diagnostic_catalog::diagnostic_cause(diagnostic.code, reason)?;
    let semantic_site = format!(
        "semantic_site={}",
        node_id::span("diagnostic-semantic-site", span, role)
    );
    Some((cause, semantic_site))
}

fn task_entries<'a>(
    program: &'a Program,
    definitions: &[ResolveDefinitionSummary],
    scopes: &[ResolveScopeSummary],
) -> Vec<TaskEntry<'a>> {
    fn collect<'a>(items: &'a [Item], file: &'a str, out: &mut Vec<(&'a Task, &'a str)>) {
        for item in items {
            match item {
                Item::App(app) => collect(&app.items, file, out),
                Item::Task(task) => out.push((task, file)),
                _ => {}
            }
        }
    }
    let mut raw = Vec::new();
    for file in &program.files {
        collect(&file.items, &file.path, &mut raw);
    }
    raw.into_iter()
        .filter_map(|(task, file)| {
            let definition = definition_for_span(definitions, &task.span, "task", None)?;
            let scope = scopes.iter().find(|scope| {
                scope.scope_kind == "callable"
                    && scope.owner_kind == "task"
                    && scope
                        .source_span
                        .as_ref()
                        .is_some_and(|span| same_span(span, &task.span))
            })?;
            Some(TaskEntry {
                task,
                file,
                definition_id: definition.id.clone(),
                callable_scope_id: scope.id.clone(),
            })
        })
        .collect()
}

fn visit_statement_calls<'a>(
    statement: &'a ParsedBodyStatement,
    visitor: &mut impl FnMut(&'a Span, &'a ParsedExpression, &'a ParsedCall),
) {
    match &statement.kind {
        ParsedBodyStatementKind::Return(expression) => {
            visit_expression_calls_with_statement(&statement.span, expression, visitor)
        }
        ParsedBodyStatementKind::Binding {
            value: Some(expression),
            ..
        } => visit_expression_calls_with_statement(&statement.span, expression, visitor),
        ParsedBodyStatementKind::Other { expressions } => {
            for expression in expressions {
                visit_expression_calls_with_statement(&statement.span, expression, visitor);
            }
        }
        _ => {}
    }
}

fn visit_expression_calls_with_statement<'a>(
    statement_span: &'a Span,
    expression: &'a ParsedExpression,
    visitor: &mut impl FnMut(&'a Span, &'a ParsedExpression, &'a ParsedCall),
) {
    match &expression.kind {
        ParsedExpressionKind::Call(call) => {
            visitor(statement_span, expression, call);
            visit_expression_calls_with_statement(statement_span, &call.callee, visitor);
            for argument in &call.arguments {
                visit_expression_calls_with_statement(statement_span, argument, visitor);
            }
        }
        ParsedExpressionKind::Permission { value, .. } => {
            visit_expression_calls_with_statement(statement_span, value, visitor)
        }
        ParsedExpressionKind::Compound { operands } => {
            for operand in operands {
                visit_expression_calls_with_statement(statement_span, operand, visitor);
            }
        }
        _ => {}
    }
}

fn visit_expression_calls<'a>(
    expression: &'a ParsedExpression,
    visitor: &mut impl FnMut(&'a ParsedExpression, &'a ParsedCall),
) {
    match &expression.kind {
        ParsedExpressionKind::Call(call) => {
            visitor(expression, call);
            visit_expression_calls(&call.callee, visitor);
            for argument in &call.arguments {
                visit_expression_calls(argument, visitor);
            }
        }
        ParsedExpressionKind::Permission { value, .. } => visit_expression_calls(value, visitor),
        ParsedExpressionKind::Compound { operands } => {
            for operand in operands {
                visit_expression_calls(operand, visitor);
            }
        }
        _ => {}
    }
}

fn find_unsupported_callable_value_use(
    expression: &ParsedExpression,
    definition_id: &str,
    references: &[ResolveReferenceSummary],
    callee_position: bool,
    found: &mut Option<Span>,
) {
    if found.is_some() {
        return;
    }
    match &expression.kind {
        ParsedExpressionKind::Identifier(_) => {
            if !callee_position && expression_resolves_to(expression, definition_id, references) {
                *found = Some(expression.span.clone());
            }
        }
        ParsedExpressionKind::Call(call) => {
            find_unsupported_callable_value_use(
                &call.callee,
                definition_id,
                references,
                true,
                found,
            );
            for argument in &call.arguments {
                find_unsupported_callable_value_use(
                    argument,
                    definition_id,
                    references,
                    false,
                    found,
                );
            }
        }
        ParsedExpressionKind::Permission { value, .. } => {
            find_unsupported_callable_value_use(value, definition_id, references, false, found);
        }
        ParsedExpressionKind::Compound { operands } => {
            for operand in operands {
                find_unsupported_callable_value_use(
                    operand,
                    definition_id,
                    references,
                    false,
                    found,
                );
            }
        }
        _ => {}
    }
}

fn identifier(expression: &ParsedExpression) -> Option<&crate::ast::ParsedIdentifier> {
    match &expression.kind {
        ParsedExpressionKind::Identifier(identifier) => Some(identifier),
        _ => None,
    }
}

fn reference_at<'a>(
    references: &'a [ResolveReferenceSummary],
    span: &Span,
    kind: &str,
) -> Option<&'a ResolveReferenceSummary> {
    references.iter().find(|reference| {
        reference.reference_kind == kind && same_span(&reference.source_span, span)
    })
}

fn call_callee_resolves_to(
    call: &ParsedCall,
    definition_id: &str,
    references: &[ResolveReferenceSummary],
) -> bool {
    identifier(&call.callee)
        .and_then(|identifier| reference_at(references, &identifier.span, "callable_callee_ref"))
        .and_then(|reference| reference.resolved_definition_id.as_deref())
        == Some(definition_id)
}

fn expression_resolves_to(
    expression: &ParsedExpression,
    definition_id: &str,
    references: &[ResolveReferenceSummary],
) -> bool {
    identifier(expression)
        .and_then(|identifier| {
            reference_at(references, &identifier.span, "callable_value_ref")
                .or_else(|| reference_at(references, &identifier.span, "callable_argument_ref"))
        })
        .and_then(|reference| reference.resolved_definition_id.as_deref())
        == Some(definition_id)
}

fn ordinary_argument_fact(
    expression: &ParsedExpression,
    definitions: &[ResolveDefinitionSummary],
    references: &[ResolveReferenceSummary],
    tasks: &[TaskEntry<'_>],
) -> Option<OrdinaryArgumentFact> {
    match &expression.kind {
        ParsedExpressionKind::UIntLiteral(value) => Some(OrdinaryArgumentFact::UIntLiteral(*value)),
        ParsedExpressionKind::Identifier(identifier) => {
            let reference = reference_at(references, &identifier.span, "callable_argument_ref")?;
            let definition_id = reference.resolved_definition_id.as_ref()?;
            let definition = definitions
                .iter()
                .find(|definition| &definition.id == definition_id)?;
            if definition.definition_kind == "parameter" {
                let is_uint = tasks.iter().any(|entry| {
                    entry.task.params.iter().any(|param| {
                        same_span(&param.span, &definition.source_span)
                            && is_named(&param.type_syntax, "UInt")
                    })
                });
                is_uint.then(|| OrdinaryArgumentFact::Definition {
                    definition_id: definition_id.clone(),
                    binding_name: definition.name.clone(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn task_signature(task: &Task) -> (Vec<&str>, &str, Option<&str>) {
    let inputs = task
        .params
        .iter()
        .map(|param| named_type(&param.type_syntax).unwrap_or("unsupported"))
        .collect::<Vec<_>>();
    match task.result_syntax.as_ref().map(|syntax| &syntax.kind) {
        Some(TypeSyntaxKind::Named { name }) => (inputs, name.as_str(), None),
        Some(TypeSyntaxKind::Result {
            value,
            failure_root,
        }) => (
            inputs,
            named_type(value).unwrap_or("unsupported"),
            Some(failure_root.as_str()),
        ),
        _ => (inputs, "Unit", None),
    }
}

fn known_type_names(program: &Program) -> BTreeSet<String> {
    let mut names = [
        "Unit",
        "Bool",
        "Int",
        "UInt",
        "Float",
        "Text",
        "Bytes",
        "Path",
        "OutputError",
        "ReplayClockError",
        "FileReadError",
        "Result",
        "Option",
        "Maybe",
        "list",
        "List",
        "Vec",
        "Slice",
        "Span",
        "Map",
        "Set",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    fn collect(items: &[Item], names: &mut BTreeSet<String>) {
        for item in items {
            match item {
                Item::App(app) => collect(&app.items, names),
                Item::Type(item) => {
                    names.insert(item.name.clone());
                }
                _ => {}
            }
        }
    }
    for file in &program.files {
        collect(&file.items, &mut names);
    }
    names
}

fn task_has_unknown_ordinary_type(task: &Task, known_types: &BTreeSet<String>) -> bool {
    task.params
        .iter()
        .any(|param| named_type(&param.type_syntax).is_some_and(|name| !known_types.contains(name)))
        || task
            .result_syntax
            .as_ref()
            .and_then(named_type)
            .is_some_and(|name| !known_types.contains(name))
}

fn task_has_closed_empty_latent_row(task: &Task) -> bool {
    if !task.effect_syntax.is_empty() {
        return false;
    }
    !task
        .body_syntax
        .iter()
        .any(|statement| match &statement.kind {
            ParsedBodyStatementKind::Return(expression) => expression_contains_call(expression),
            ParsedBodyStatementKind::Binding { value, .. } => {
                value.as_ref().is_some_and(expression_contains_call)
            }
            ParsedBodyStatementKind::Other { expressions } => {
                expressions.iter().any(expression_contains_call)
            }
        })
}

fn task_has_supported_am_latent_row(
    task: &Task,
    change_origins: &[ResolveReferenceSummary],
) -> bool {
    if !task.effect_syntax.is_empty() {
        return false;
    }
    if change_origins.is_empty() {
        return task_has_closed_empty_latent_row(task);
    }
    !task
        .body_syntax
        .iter()
        .any(|statement| match &statement.kind {
            ParsedBodyStatementKind::Return(expression) => expression_contains_call(expression),
            ParsedBodyStatementKind::Binding { value, .. } => {
                value.as_ref().is_some_and(expression_contains_call)
            }
            ParsedBodyStatementKind::Other { expressions } => {
                expressions.iter().any(expression_contains_call)
            }
        })
}

fn task_change_origins(
    task: &TaskEntry<'_>,
    references: &[ResolveReferenceSummary],
    scopes: &[ResolveScopeSummary],
) -> Vec<ResolveReferenceSummary> {
    let mut origins = references
        .iter()
        .filter(|reference| {
            reference.reference_kind == "mutation_target"
                && scope_is_within(&reference.scope_id, &task.callable_scope_id, scopes)
        })
        .cloned()
        .collect::<Vec<_>>();
    origins.sort_by_key(|reference| {
        (
            node_id::source_path_identity(&reference.source_span.file),
            reference.source_span.line,
            reference.source_span.column,
            reference.id.clone(),
        )
    });
    origins
}

fn scope_is_within(scope_id: &str, ancestor_id: &str, scopes: &[ResolveScopeSummary]) -> bool {
    let mut current = Some(scope_id);
    let mut visited = BTreeSet::new();
    while let Some(id) = current {
        if id == ancestor_id {
            return true;
        }
        if !visited.insert(id) {
            return false;
        }
        current = scopes
            .iter()
            .find(|scope| scope.id == id)
            .and_then(|scope| scope.parent_scope_id.as_deref());
    }
    false
}

fn expression_contains_call(expression: &ParsedExpression) -> bool {
    match &expression.kind {
        ParsedExpressionKind::Call(_) => true,
        ParsedExpressionKind::Permission { value, .. } => expression_contains_call(value),
        ParsedExpressionKind::Compound { operands } => {
            operands.iter().any(expression_contains_call)
        }
        ParsedExpressionKind::Identifier(_)
        | ParsedExpressionKind::UIntLiteral(_)
        | ParsedExpressionKind::Unsupported { .. }
        | ParsedExpressionKind::Other => false,
    }
}

fn exact_uint_callable(callable: &crate::ast::CallableTypeSyntax) -> bool {
    callable.inputs.len() == 1
        && is_named(&callable.inputs[0], "UInt")
        && is_named(&callable.result, "UInt")
}

fn is_named(syntax: &TypeSyntax, expected: &str) -> bool {
    named_type(syntax) == Some(expected)
}
fn named_type(syntax: &TypeSyntax) -> Option<&str> {
    match &syntax.kind {
        TypeSyntaxKind::Named { name } => Some(name),
        _ => None,
    }
}

fn definition_for_span<'a>(
    definitions: &'a [ResolveDefinitionSummary],
    span: &Span,
    kind: &str,
    scope: Option<&str>,
) -> Option<&'a ResolveDefinitionSummary> {
    definitions.iter().find(|definition| {
        definition.definition_kind == kind
            && same_span(&definition.source_span, span)
            && scope.is_none_or(|scope| definition.scope_id == scope)
    })
}

fn semantic_id(kind: &str, span: &Span) -> String {
    node_id::span(kind, &portable_span(span), kind)
}
fn missing_id(kind: &str, span: &Span) -> String {
    semantic_id(&format!("missing-{kind}"), span)
}
fn portable_span(span: &Span) -> Span {
    Span::new(
        node_id::source_path_identity(&span.file),
        span.line,
        span.column,
    )
}
fn same_span(left: &Span, right: &Span) -> bool {
    node_id::source_path_identity(&left.file) == node_id::source_path_identity(&right.file)
        && left.line == right.line
        && left.column == right.column
}
fn same_line(left: &Span, right: &Span) -> bool {
    node_id::source_path_identity(&left.file) == node_id::source_path_identity(&right.file)
        && left.line == right.line
}

fn analysis_key(
    program: &Program,
    definitions: &[ResolveDefinitionSummary],
    scopes: &[ResolveScopeSummary],
    references: &[ResolveReferenceSummary],
) -> AnalysisKey {
    AnalysisKey {
        program: program.clone(),
        definitions: definitions.to_vec(),
        scopes: scopes.to_vec(),
        references: references.to_vec(),
    }
}

fn indent_json(text: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
fn json_indent(out: &mut String, spaces: usize) {
    out.push_str(&" ".repeat(spaces));
}
fn json_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    json_indent(out, indent);
    out.push_str(&format!(
        "\"{}\": \"{}\"{}\n",
        json_escape(key),
        json_escape(value),
        if comma { "," } else { "" }
    ));
}
fn json_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    json_indent(out, indent);
    out.push_str(&format!(
        "\"{}\": {}{}\n",
        json_escape(key),
        value,
        if comma { "," } else { "" }
    ));
}

fn push_json_definitions(out: &mut String, facts: &[CallableDefinitionFact], comma: bool) {
    push_json_fact_array(out, "definition_facts", facts, comma, |out, fact| {
        json_string_field(out, 6, "id", &fact.id, true);
        json_string_field(
            out,
            6,
            "resolver_definition_id",
            &fact.resolver_definition_id,
            true,
        );
        json_string_field(out, 6, "lexical_scope_id", &fact.lexical_scope_id, true);
        json_string_field(
            out,
            6,
            "input_definition_ids",
            &fact.input_definition_ids.join(","),
            true,
        );
        json_string_field(out, 6, "input_types", &fact.input_types.join(","), true);
        json_string_field(out, 6, "result_type", &fact.result_type, true);
        json_string_field(
            out,
            6,
            "failure_root",
            fact.failure_root.as_deref().unwrap_or("none"),
            true,
        );
        json_string_field(out, 6, "source_span", &span_text(&fact.source_span), true);
        json_string_field(out, 6, "status", fact.status, false);
    });
}

fn push_json_types(out: &mut String, facts: &[CallableTypeFact], comma: bool) {
    push_json_fact_array(out, "type_facts", facts, comma, |out, fact| {
        json_string_field(out, 6, "id", &fact.id, true);
        json_string_field(out, 6, "input_types", &fact.input_types.join(","), true);
        json_string_field(out, 6, "result_type", &fact.result_type, true);
        json_string_field(
            out,
            6,
            "failure_root",
            fact.failure_root.as_deref().unwrap_or("none"),
            true,
        );
        json_string_field(out, 6, "latent_row_id", &fact.latent_row_id, true);
        json_string_field(out, 6, "status", fact.status, false);
    });
}

fn push_json_rows(out: &mut String, facts: &[LatentEffectRowFact], comma: bool) {
    push_json_fact_array(out, "latent_row_facts", facts, comma, |out, fact| {
        json_string_field(out, 6, "id", &fact.id, true);
        json_string_field(out, 6, "labels", &fact.labels.join(","), true);
        json_string_field(
            out,
            6,
            "normalized_labels",
            &fact.normalized_labels.join(","),
            true,
        );
        json_string_field(
            out,
            6,
            "tail_id",
            fact.tail_id.as_deref().unwrap_or("none"),
            true,
        );
        json_string_field(
            out,
            6,
            "tail_alias",
            fact.tail_alias.as_deref().unwrap_or("none"),
            true,
        );
        json_string_field(
            out,
            6,
            "normalized_tail",
            fact.normalized_tail.as_deref().unwrap_or("none"),
            true,
        );
        json_string_field(out, 6, "origin", fact.origin, true);
        json_string_field(out, 6, "status", fact.status, false);
    });
}

fn push_json_label_occurrences(out: &mut String, facts: &[EffectLabelOccurrenceFact], comma: bool) {
    push_json_fact_array(
        out,
        "effect_label_occurrence_facts",
        facts,
        comma,
        |out, fact| {
            json_string_field(out, 6, "id", &fact.id, true);
            json_string_field(out, 6, "label", fact.label, true);
            json_string_field(out, 6, "alias_id", &fact.alias_id, true);
            json_string_field(
                out,
                6,
                "resolver_reference_id",
                &fact.resolver_reference_id,
                true,
            );
            json_string_field(
                out,
                6,
                "owner_definition_id",
                &fact.owner_definition_id,
                true,
            );
            json_string_field(
                out,
                6,
                "target_definition_id",
                &fact.target_definition_id,
                true,
            );
            json_string_field(out, 6, "source_span", &span_text(&fact.source_span), false);
        },
    );
}

fn push_json_substitutions(out: &mut String, facts: &[RowSubstitutionFact], comma: bool) {
    push_json_fact_array(out, "row_substitution_facts", facts, comma, |out, fact| {
        json_string_field(out, 6, "id", &fact.id, true);
        json_string_field(out, 6, "application_id", &fact.application_id, true);
        json_string_field(out, 6, "input_row_id", &fact.input_row_id, true);
        json_string_field(out, 6, "tail_id", &fact.tail_id, true);
        json_string_field(out, 6, "argument_row_id", &fact.argument_row_id, true);
        json_string_field(out, 6, "output_row_id", &fact.output_row_id, true);
        json_string_field(out, 6, "status", fact.status, false);
    });
}

fn push_json_values(out: &mut String, facts: &[CallableValueFact], comma: bool) {
    push_json_fact_array(out, "value_facts", facts, comma, |out, fact| {
        json_string_field(out, 6, "id", &fact.id, true);
        json_string_field(
            out,
            6,
            "resolver_reference_id",
            &fact.resolver_reference_id,
            true,
        );
        json_string_field(out, 6, "referring_scope_id", &fact.referring_scope_id, true);
        json_string_field(
            out,
            6,
            "resolved_task_definition_id",
            &fact.resolved_task_definition_id,
            true,
        );
        json_string_field(
            out,
            6,
            "expected_callable_type_id",
            &fact.expected_callable_type_id,
            true,
        );
        json_string_field(out, 6, "source_span", &span_text(&fact.source_span), true);
        json_string_field(out, 6, "status", fact.status, false);
    });
}

fn push_json_fact_array<T>(
    out: &mut String,
    key: &str,
    facts: &[T],
    comma: bool,
    mut fields: impl FnMut(&mut String, &T),
) {
    json_indent(out, 2);
    out.push_str(&format!("\"{}\": [\n", json_escape(key)));
    for (index, fact) in facts.iter().enumerate() {
        json_indent(out, 4);
        out.push_str("{\n");
        fields(out, fact);
        json_indent(out, 4);
        out.push_str(if index + 1 == facts.len() {
            "}\n"
        } else {
            "},\n"
        });
    }
    json_indent(out, 2);
    out.push_str(if comma { "],\n" } else { "]\n" });
}

fn span_text(span: &Span) -> String {
    format!("{}:{}:{}", span.file, span.line, span.column)
}

fn push_json_applications(out: &mut String, facts: &[CallableApplicationFact], comma: bool) {
    json_indent(out, 2);
    out.push_str("\"application_facts\": [\n");
    for (index, fact) in facts.iter().enumerate() {
        json_indent(out, 4);
        out.push_str("{\n");
        json_string_field(out, 6, "id", &fact.id, true);
        json_string_field(
            out,
            6,
            "caller_definition_id",
            &fact.caller_definition_id,
            true,
        );
        json_string_field(
            out,
            6,
            "receiver_definition_id",
            &fact.receiver_definition_id,
            true,
        );
        json_string_field(
            out,
            6,
            "target_definition_id",
            &fact.target_definition_id,
            true,
        );
        json_string_field(
            out,
            6,
            "callable_parameter_definition_id",
            &fact.callable_parameter_definition_id,
            true,
        );
        json_string_field(
            out,
            6,
            "ordinary_parameter_definition_id",
            &fact.ordinary_parameter_definition_id,
            true,
        );
        json_string_field(out, 6, "callable_value_id", &fact.callable_value_id, true);
        json_string_field(
            out,
            6,
            "callable_parameter_name",
            &fact.callable_parameter_name,
            true,
        );
        json_string_field(
            out,
            6,
            "ordinary_parameter_name",
            &fact.ordinary_parameter_name,
            true,
        );
        json_string_field(out, 6, "caller_span", &span_text(&fact.caller_span), true);
        json_string_field(
            out,
            6,
            "receiver_span",
            &span_text(&fact.receiver_span),
            true,
        );
        json_string_field(
            out,
            6,
            "direct_call_span",
            &span_text(&fact.direct_call_span),
            true,
        );
        json_string_field(
            out,
            6,
            "indirect_call_span",
            &span_text(&fact.indirect_call_span),
            true,
        );
        let ordinary_argument = match &fact.ordinary_argument {
            OrdinaryArgumentFact::UIntLiteral(value) => format!("uint:{value}"),
            OrdinaryArgumentFact::Definition {
                definition_id,
                binding_name,
            } => {
                format!("definition:{definition_id}:{binding_name}")
            }
        };
        json_string_field(out, 6, "ordinary_argument", &ordinary_argument, true);
        json_string_field(out, 6, "input_row_id", &fact.input_row_id, true);
        json_string_field(out, 6, "output_row_id", &fact.output_row_id, true);
        json_string_field(out, 6, "result_type", &fact.result_type, true);
        json_string_field(
            out,
            6,
            "failure_root",
            fact.failure_root.as_deref().unwrap_or("none"),
            true,
        );
        json_string_field(out, 6, "status", fact.status, true);
        json_string_field(out, 6, "reason", fact.reason, false);
        json_indent(out, 4);
        out.push_str(if index + 1 == facts.len() {
            "}\n"
        } else {
            "},\n"
        });
    }
    json_indent(out, 2);
    out.push_str(if comma { "],\n" } else { "]\n" });
}

fn push_json_core_nodes(out: &mut String, analysis: &CallableAnalysis, comma: bool) {
    json_indent(out, 2);
    out.push_str("\"core_nodes\": [\n");
    let nodes = core_node_facts(analysis);
    for (index, node) in nodes.iter().enumerate() {
        json_indent(out, 4);
        out.push_str("{\n");
        json_string_field(out, 6, "kind", &node.kind, true);
        json_string_field(out, 6, "id", &node.id, true);
        json_string_field(out, 6, "relationship_id", &node.relationship_id, true);
        json_string_field(out, 6, "result_type", &node.result_type, false);
        json_indent(out, 4);
        out.push_str(if index + 1 == nodes.len() {
            "}\n"
        } else {
            "},\n"
        });
    }
    json_indent(out, 2);
    out.push_str(if comma { "],\n" } else { "]\n" });
}

fn push_json_graph_edges(out: &mut String, analysis: &CallableAnalysis, comma: bool) {
    json_indent(out, 2);
    out.push_str("\"graph_edges\": [\n");
    let edges = graph_edge_facts(analysis);
    for (index, edge) in edges.iter().enumerate() {
        json_indent(out, 4);
        out.push_str("{\n");
        json_string_field(out, 6, "id", &edge.id, true);
        json_string_field(out, 6, "kind", &edge.kind, true);
        json_string_field(out, 6, "from", &edge.from, true);
        json_string_field(out, 6, "to", &edge.to, true);
        json_string_field(
            out,
            6,
            "owner_definition_id",
            &edge.owner_definition_id,
            true,
        );
        json_string_field(out, 6, "application_id", &edge.application_id, true);
        json_string_field(out, 6, "span", &span_text(&edge.source_span), false);
        json_indent(out, 4);
        out.push_str(if index + 1 == edges.len() {
            "}\n"
        } else {
            "},\n"
        });
    }
    json_indent(out, 2);
    out.push_str(if comma { "],\n" } else { "]\n" });
}

fn core_node_facts(analysis: &CallableAnalysis) -> Vec<CallableCoreNodeFact> {
    let mut nodes = Vec::new();
    nodes.extend(analysis.types.iter().map(|fact| CallableCoreNodeFact {
        kind: "callable_type".to_string(),
        id: fact.id.clone(),
        relationship_id: fact.latent_row_id.clone(),
        result_type: fact.result_type.clone(),
    }));
    nodes.extend(analysis.values.iter().map(|fact| CallableCoreNodeFact {
        kind: "callable_value".to_string(),
        id: fact.id.clone(),
        relationship_id: fact.resolved_task_definition_id.clone(),
        result_type: "task(UInt) -> UInt".to_string(),
    }));
    nodes.extend(
        analysis
            .applications
            .iter()
            .map(|fact| CallableCoreNodeFact {
                kind: "callable_application".to_string(),
                id: fact.id.clone(),
                relationship_id: fact.callable_value_id.clone(),
                result_type: fact.result_type.clone(),
            }),
    );
    nodes.extend(analysis.rows.iter().map(|fact| CallableCoreNodeFact {
        kind: "latent_effect_row".to_string(),
        id: fact.id.clone(),
        relationship_id: if fact.labels.is_empty() {
            fact.tail_id.clone().unwrap_or_else(|| "closed".to_string())
        } else {
            fact.labels.join(",")
        },
        result_type: fact.status.to_string(),
    }));
    nodes.extend(
        analysis
            .label_occurrences
            .iter()
            .map(|fact| CallableCoreNodeFact {
                kind: "effect_label_occurrence".to_string(),
                id: fact.id.clone(),
                relationship_id: fact.alias_id.clone(),
                result_type: fact.target_definition_id.clone(),
            }),
    );
    nodes.extend(
        analysis
            .substitutions
            .iter()
            .map(|fact| CallableCoreNodeFact {
                kind: "row_substitution".to_string(),
                id: fact.id.clone(),
                relationship_id: format!(
                    "{}>{}>{}",
                    fact.input_row_id, fact.argument_row_id, fact.output_row_id
                ),
                result_type: fact.status.to_string(),
            }),
    );
    nodes
}

#[cfg(test)]
fn verify_core_node_facts(
    analysis: &CallableAnalysis,
    facts: &[CallableCoreNodeFact],
) -> Vec<&'static str> {
    let expected = core_node_facts(analysis);
    let ids = facts
        .iter()
        .map(|fact| fact.id.as_str())
        .collect::<BTreeSet<_>>();
    if facts != expected || ids.len() != facts.len() {
        vec!["callable_core_projection_relationship_corrupt_v0"]
    } else {
        Vec::new()
    }
}

fn push_graph_edge(
    edges: &mut Vec<CallableGraphEdgeFact>,
    kind: &str,
    from: &str,
    to: &str,
    owner_definition_id: &str,
    application_id: &str,
    source_span: &Span,
) {
    edges.push(CallableGraphEdgeFact {
        id: semantic_id(
            &format!("callable-graph-{kind}-{application_id}-{from}-{to}"),
            source_span,
        ),
        kind: kind.to_string(),
        from: from.to_string(),
        to: to.to_string(),
        owner_definition_id: owner_definition_id.to_string(),
        application_id: application_id.to_string(),
        source_span: portable_span(source_span),
    });
}

fn graph_edge_facts(analysis: &CallableAnalysis) -> Vec<CallableGraphEdgeFact> {
    let mut edges = Vec::new();
    for application in &analysis.applications {
        for (kind, from, to, span) in [
            (
                "definition",
                application.target_definition_id.as_str(),
                application.callable_value_id.as_str(),
                &application.direct_call_span,
            ),
            (
                "value_use",
                application.callable_value_id.as_str(),
                application.id.as_str(),
                &application.direct_call_span,
            ),
            (
                "passed_as_argument",
                application.callable_value_id.as_str(),
                application.callable_parameter_definition_id.as_str(),
                &application.direct_call_span,
            ),
            (
                "parameter_bind",
                application.callable_parameter_definition_id.as_str(),
                application.target_definition_id.as_str(),
                &application.indirect_call_span,
            ),
            (
                "application",
                application.callable_parameter_definition_id.as_str(),
                application.id.as_str(),
                &application.indirect_call_span,
            ),
        ] {
            push_graph_edge(
                &mut edges,
                kind,
                from,
                to,
                &application.receiver_definition_id,
                &application.id,
                span,
            );
        }
    }
    for substitution in &analysis.substitutions {
        let Some(application) = analysis
            .applications
            .iter()
            .find(|application| application.id == substitution.application_id)
        else {
            continue;
        };
        for occurrence in analysis
            .label_occurrences
            .iter()
            .filter(|occurrence| occurrence.owner_definition_id == application.target_definition_id)
        {
            push_graph_edge(
                &mut edges,
                "effect_label_occurrence",
                &occurrence.owner_definition_id,
                &occurrence.id,
                &application.receiver_definition_id,
                &application.id,
                &occurrence.source_span,
            );
            push_graph_edge(
                &mut edges,
                "effect_alias",
                &occurrence.alias_id,
                &occurrence.id,
                &application.receiver_definition_id,
                &application.id,
                &occurrence.source_span,
            );
        }
        for (kind, from, to) in [
            (
                "row_variable",
                substitution.input_row_id.as_str(),
                substitution.tail_id.as_str(),
            ),
            (
                "row_substitution",
                substitution.input_row_id.as_str(),
                substitution.output_row_id.as_str(),
            ),
            (
                "row_argument",
                substitution.argument_row_id.as_str(),
                substitution.tail_id.as_str(),
            ),
            (
                "row_propagation",
                substitution.argument_row_id.as_str(),
                substitution.output_row_id.as_str(),
            ),
        ] {
            push_graph_edge(
                &mut edges,
                kind,
                from,
                to,
                &application.receiver_definition_id,
                &application.id,
                &application.direct_call_span,
            );
        }
    }
    edges
}

#[cfg(test)]
fn verify_graph_edge_facts(
    analysis: &CallableAnalysis,
    facts: &[CallableGraphEdgeFact],
) -> Vec<&'static str> {
    let expected = graph_edge_facts(analysis);
    let ids = facts
        .iter()
        .map(|fact| fact.id.as_str())
        .collect::<BTreeSet<_>>();
    if facts != expected
        || ids.len() != facts.len()
        || facts.iter().any(|fact| {
            fact.id.is_empty()
                || fact.kind.is_empty()
                || fact.from.is_empty()
                || fact.to.is_empty()
                || fact.owner_definition_id.is_empty()
                || fact.application_id.is_empty()
                || fact.source_span.file.is_empty()
        })
    {
        vec!["callable_graph_projection_relationship_corrupt_v0"]
    } else {
        Vec::new()
    }
}

fn push_json_diagnostics(out: &mut String, facts: &[CallableDiagnosticFact], comma: bool) {
    json_indent(out, 2);
    out.push_str("\"diagnostic_facts\": [\n");
    for (index, fact) in facts.iter().enumerate() {
        let span = fact
            .diagnostic()
            .span
            .as_ref()
            .expect("callable diagnostic span");
        json_indent(out, 4);
        out.push_str("{\n");
        json_string_field(out, 6, "id", &fact.id, true);
        json_string_field(out, 6, "code", fact.diagnostic().code.as_str(), true);
        json_string_field(out, 6, "reason", fact.reason, true);
        json_string_field(out, 6, "detail_reason", fact.detail_reason, true);
        json_string_field(out, 6, "message", &fact.diagnostic().message, true);
        json_string_field(
            out,
            6,
            "help",
            fact.diagnostic().help.as_deref().unwrap_or(""),
            true,
        );
        json_string_field(
            out,
            6,
            "primary_span",
            &format!("{}:{}:{}", span.file, span.line, span.column),
            true,
        );
        json_indent(out, 6);
        out.push_str("\"related_spans\": [\n");
        for (related_index, related) in fact.diagnostic().related_spans.iter().enumerate() {
            json_indent(out, 8);
            out.push_str("{\n");
            json_string_field(out, 10, "label", &related.label, true);
            json_string_field(
                out,
                10,
                "span",
                &format!(
                    "{}:{}:{}",
                    related.span.file, related.span.line, related.span.column
                ),
                false,
            );
            json_indent(out, 8);
            out.push_str(
                if related_index + 1 == fact.diagnostic().related_spans.len() {
                    "}\n"
                } else {
                    "},\n"
                },
            );
        }
        json_indent(out, 6);
        out.push_str("]\n");
        json_indent(out, 4);
        out.push_str(if index + 1 == facts.len() {
            "}\n"
        } else {
            "},\n"
        });
    }
    json_indent(out, 2);
    out.push_str(if comma { "],\n" } else { "]\n" });
}

#[cfg(test)]
mod tests {
    use super::{
        CALLABLE_FACT_MODEL, OrdinaryArgumentFact, analyze_program, core_node_facts,
        diagnostic_occurrences, graph_edge_facts, validate_diagnostic_occurrences,
        verify_core_node_facts, verify_graph_edge_facts,
    };
    use crate::ast::{Item, ParsedExpressionKind};
    use crate::diagnostic::{Diagnostic, DiagnosticCode};
    use crate::parser::parse_source;
    use std::sync::Arc;

    const SOURCE: &str = r#"module tests.callable

task increment(value: UInt) -> UInt {
  does:
    return value + 1
}

task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {
  does:
    return transform(value)
}

task run -> UInt {
  does:
    return apply_once(increment, 41)
}
"#;

    const AM_SOURCE: &str = r#"module tests.callable_row

task bump(value: UInt) -> UInt {
  does:
    change current: UInt = value
    set current = current + 1
    set current = current + 1
    return current
}

task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {
  does:
    return transform(value)
}

task run -> UInt {
  does:
    return apply_once(bump, 40)
}
"#;

    #[test]
    fn ao_callable_occurrences_keep_relationship_identity_and_precedence() {
        for (file, source, code) in [
            (
                "stored.hum",
                include_str!("../fixtures/callable/session_al_stored_callable_fail.hum"),
                DiagnosticCode::INVALID_CALLABLE_FORM,
            ),
            (
                "wrong_input.hum",
                include_str!("../fixtures/callable/session_al_wrong_input_fail.hum"),
                DiagnosticCode::CALLABLE_SIGNATURE_MISMATCH,
            ),
        ] {
            let parsed = parse_source(file, source);
            let program = crate::ast::Program {
                files: vec![parsed.file],
            };
            assert_eq!(validate_diagnostic_occurrences(&program), Ok(()));
            let first = diagnostic_occurrences(&program)
                .into_iter()
                .find(|occurrence| occurrence.code == code)
                .expect("registered callable occurrence");
            let second = diagnostic_occurrences(&program)
                .into_iter()
                .find(|occurrence| occurrence.code == code)
                .expect("repeat registered callable occurrence");
            assert_eq!(first.id(), second.id());
            assert_eq!(first.semantic_owner(), "callable_analysis");
            assert_eq!(first.owning_stage(), "shared_preflight");
            assert!(
                first
                    .relationship_route()
                    .iter()
                    .any(|site| site.starts_with("callable_definition_application_route"))
            );
            assert!(
                first
                    .relationship_route()
                    .iter()
                    .any(|site| site.starts_with("definition=") || site.starts_with("reference="))
            );

            let analysis = analyze_program(&program);
            let fact = analysis
                .diagnostics
                .iter()
                .find(|fact| fact.diagnostic().code == code)
                .expect("callable diagnostic fact");
            assert!(fact.fallback_diagnostic.is_none());
            for mutation in 0..5 {
                let mut drift = fact.clone();
                let occurrence = drift.occurrence.as_mut().expect("H140 occurrence carrier");
                let diagnostic = occurrence.diagnostic_mut_for_test();
                match mutation {
                    0 => diagnostic.code = DiagnosticCode::UNRESOLVED_NAME,
                    1 => diagnostic.message.push_str(" drift"),
                    2 => diagnostic.help = Some("drift".to_string()),
                    3 => diagnostic.span = Some(crate::diagnostic::Span::new("other.hum", 1, 1)),
                    _ => diagnostic.related_spans.clear(),
                }
                assert!(
                    occurrence.validate().is_err(),
                    "rendered projection mutation {mutation}"
                );
            }
            let dominant = if code == DiagnosticCode::INVALID_CALLABLE_FORM {
                DiagnosticCode::PATH_SOURCE_CONSTRUCTION
            } else {
                DiagnosticCode::UNKNOWN_TYPE_NAME
            };
            let prior = Diagnostic::error(
                dominant,
                "prior owner",
                Some(fact.relationship_sites[0].clone()),
            );
            let (unknown, paths) = if code == DiagnosticCode::INVALID_CALLABLE_FORM {
                (Vec::new(), vec![prior.clone()])
            } else {
                (vec![prior.clone()], Vec::new())
            };
            assert!(analysis.prior_owns(fact, std::slice::from_ref(&prior), &unknown, &paths));
            let generic = Diagnostic::error(
                DiagnosticCode::UNRESOLVED_NAME,
                "generic replacement",
                Some(fact.relationship_sites[0].clone()),
            );
            assert!(!analysis.prior_owns(fact, &[generic], &unknown, &paths));

            let impostor = Diagnostic::error(
                dominant,
                "different node with a compatible display span",
                Some(fact.relationship_sites[0].clone()),
            );
            assert!(!analysis.prior_owns(fact, &[impostor], &unknown, &paths));

            let mut other_node = prior.clone();
            other_node.span.as_mut().expect("prior span").column += 1;
            let (other_unknown, other_paths) = if code == DiagnosticCode::INVALID_CALLABLE_FORM {
                (Vec::new(), vec![other_node.clone()])
            } else {
                (vec![other_node.clone()], Vec::new())
            };
            assert!(!analysis.prior_owns(fact, &[other_node], &other_unknown, &other_paths));
        }
    }

    #[test]
    fn ao_independent_same_line_causes_keep_distinct_occurrences() {
        let source = r#"module tests.ao.adjacent

task apply_once(change transform: task(UInt) -> UInt,value: Text) -> UInt {
  does:
    return transform(value)
}
"#;
        let parsed = parse_source("session_ao_adjacent.hum", source);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let occurrences = diagnostic_occurrences(&program);
        assert!(occurrences.len() >= 2, "{occurrences:#?}");
        for occurrence in &occurrences {
            occurrence.validate().expect("closed occurrence");
        }
        let same_line = occurrences.iter().any(|left| {
            occurrences.iter().any(|right| {
                left.id() != right.id()
                    && left.diagnostic().span.as_ref().map(|span| span.line)
                        == right.diagnostic().span.as_ref().map(|span| span.line)
            })
        });
        assert!(same_line, "{occurrences:#?}");
    }

    #[test]
    fn am_rows_preserve_duplicate_occurrences_and_fail_closed_on_corruption() {
        let parsed = parse_source("callable_row.hum", AM_SOURCE);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let analysis = analyze_program(&program);
        assert!(analysis.verify().is_empty(), "{:?}", analysis.verify());
        assert_eq!(analysis.label_occurrences.len(), 2);
        assert_ne!(
            analysis.label_occurrences[0].id,
            analysis.label_occurrences[1].id
        );
        assert_ne!(
            analysis.label_occurrences[0].alias_id,
            analysis.label_occurrences[1].alias_id
        );
        assert_eq!(analysis.label_occurrences[0].label, "change");
        assert_eq!(analysis.label_occurrences[1].label, "change");
        assert_eq!(analysis.substitutions.len(), 1);
        let substitution = &analysis.substitutions[0];
        let argument = analysis
            .rows
            .iter()
            .find(|row| row.id == substitution.argument_row_id)
            .unwrap();
        let output = analysis
            .rows
            .iter()
            .find(|row| row.id == substitution.output_row_id)
            .unwrap();
        assert_eq!(argument.labels.len(), 2);
        assert_eq!(argument.labels, output.labels);

        let mut corrupt = (*analysis).clone();
        corrupt
            .rows
            .iter_mut()
            .find(|row| row.id == substitution.output_row_id)
            .unwrap()
            .labels
            .pop();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_fact_source_relationship_corrupt_v0")
        );
        assert!(
            corrupt
                .verify()
                .contains(&"callable_row_substitution_relationship_corrupt_v0")
        );

        let mut corrupt = (*analysis).clone();
        corrupt.substitutions[0].tail_id = "foreign-tail".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_row_substitution_relationship_corrupt_v0")
        );

        let mut corrupt = (*analysis).clone();
        corrupt
            .rows
            .iter_mut()
            .find(|row| row.id == substitution.argument_row_id)
            .unwrap()
            .labels[0] = "foreign".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_latent_row_relationship_corrupt_v0")
        );

        for field in ["normalized_labels", "tail_alias", "normalized_tail"] {
            let mut corrupt = (*analysis).clone();
            let row = corrupt
                .rows
                .iter_mut()
                .find(|row| row.id == substitution.input_row_id)
                .unwrap();
            match field {
                "normalized_labels" => row.normalized_labels.push("failure".to_string()),
                "tail_alias" => row.tail_alias = Some("other".to_string()),
                "normalized_tail" => row.normalized_tail = Some("row1".to_string()),
                _ => unreachable!(),
            }
            assert!(
                corrupt
                    .verify()
                    .contains(&"callable_latent_row_relationship_corrupt_v0"),
                "{field}: {:?}",
                corrupt.verify()
            );
        }

        let mut corrupt = (*analysis).clone();
        corrupt
            .rows
            .iter_mut()
            .find(|row| row.id == substitution.output_row_id)
            .unwrap()
            .labels
            .clear();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_latent_row_relationship_corrupt_v0")
        );

        let mut corrupt = (*analysis).clone();
        corrupt
            .rows
            .iter_mut()
            .find(|row| row.id == substitution.input_row_id)
            .unwrap()
            .tail_id = None;
        assert!(
            corrupt
                .verify()
                .contains(&"callable_latent_row_relationship_corrupt_v0")
        );

        let mut corrupt = (*analysis).clone();
        corrupt
            .rows
            .retain(|row| row.id != substitution.argument_row_id);
        assert!(
            corrupt
                .verify()
                .contains(&"callable_application_missing_row_v0")
                || corrupt
                    .verify()
                    .contains(&"callable_row_substitution_relationship_corrupt_v0")
        );

        let mut corrupt = (*analysis).clone();
        corrupt.label_occurrences[0].alias_id = "poisoned-alias".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_effect_label_occurrence_corrupt_v0")
        );

        let mut corrupt = (*analysis).clone();
        corrupt.label_occurrences[1].alias_id = corrupt.label_occurrences[0].alias_id.clone();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_fact_identity_not_unique_v0")
        );

        for field in ["id", "label", "reference", "span", "owner", "target"] {
            let mut corrupt = (*analysis).clone();
            match field {
                "id" => corrupt.label_occurrences[0].id = "wrong-id".to_string(),
                "label" => corrupt.label_occurrences[0].label = "failure",
                "reference" => {
                    corrupt.label_occurrences[0].resolver_reference_id = "wrong-ref".to_string()
                }
                "span" => corrupt.label_occurrences[0].source_span.column += 1,
                "owner" => {
                    corrupt.label_occurrences[0].owner_definition_id = "wrong-owner".to_string()
                }
                "target" => {
                    corrupt.label_occurrences[0].target_definition_id = "wrong-target".to_string()
                }
                _ => unreachable!(),
            }
            assert!(
                corrupt
                    .verify()
                    .contains(&"callable_effect_label_occurrence_corrupt_v0"),
                "{field}: {:?}",
                corrupt.verify()
            );
        }
        for field in [
            "id",
            "application",
            "input",
            "tail",
            "argument",
            "output",
            "status",
        ] {
            let mut corrupt = (*analysis).clone();
            match field {
                "id" => corrupt.substitutions[0].id = "wrong-id".to_string(),
                "application" => {
                    corrupt.substitutions[0].application_id = "wrong-application".to_string()
                }
                "input" => corrupt.substitutions[0].input_row_id = "wrong-input".to_string(),
                "tail" => corrupt.substitutions[0].tail_id = "wrong-tail".to_string(),
                "argument" => {
                    corrupt.substitutions[0].argument_row_id = "wrong-argument".to_string()
                }
                "output" => corrupt.substitutions[0].output_row_id = "wrong-output".to_string(),
                "status" => corrupt.substitutions[0].status = "wrong-status",
                _ => unreachable!(),
            }
            assert!(
                corrupt
                    .verify()
                    .contains(&"callable_row_substitution_relationship_corrupt_v0"),
                "{field}: {:?}",
                corrupt.verify()
            );
        }
    }

    #[test]
    fn am_256_label_stress_is_deterministic_and_bounded() {
        let mut source = String::from(
            "module tests.stress\n\ntask bump(value: UInt) -> UInt {\n  does:\n    change current: UInt = value\n",
        );
        for _ in 0..256 {
            source.push_str("    set current = current + 1\n");
        }
        source.push_str("    return current\n}\n\ntask apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {\n  does:\n    return transform(value)\n}\n\ntask run -> UInt {\n  does:\n    return apply_once(bump, 0)\n}\n");
        let parsed = parse_source("stress.hum", &source);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let first = analyze_program(&program);
        let second = analyze_program(&program);
        assert_eq!(first.label_occurrences.len(), 256);
        assert_eq!(first.json("core_verify"), second.json("core_verify"));
        assert!(first.verify().is_empty(), "{:?}", first.verify());
    }

    #[test]
    fn am_application_core_and_graph_fields_fail_closed_independently() {
        let parsed = parse_source("callable_row.hum", AM_SOURCE);
        let analysis = analyze_program(&crate::ast::Program {
            files: vec![parsed.file],
        });
        assert!(analysis.verify().is_empty(), "{:?}", analysis.verify());

        for field in [
            "id",
            "caller_definition_id",
            "caller_span",
            "receiver_definition_id",
            "receiver_span",
            "callable_parameter_definition_id",
            "callable_parameter_name",
            "ordinary_parameter_definition_id",
            "ordinary_parameter_name",
            "callable_value_id",
            "target_definition_id",
            "direct_call_span",
            "indirect_call_span",
            "direct_statement_span",
            "indirect_statement_span",
            "ordinary_argument",
            "result_type",
            "failure_root",
            "input_row_id",
            "output_row_id",
            "status",
            "reason",
        ] {
            let mut corrupt = (*analysis).clone();
            let application = &mut corrupt.applications[0];
            match field {
                "id" => application.id = "wrong".to_string(),
                "caller_definition_id" => application.caller_definition_id = "wrong".to_string(),
                "caller_span" => application.caller_span.column += 1,
                "receiver_definition_id" => {
                    application.receiver_definition_id = "wrong".to_string()
                }
                "receiver_span" => application.receiver_span.column += 1,
                "callable_parameter_definition_id" => {
                    application.callable_parameter_definition_id = "wrong".to_string()
                }
                "callable_parameter_name" => {
                    application.callable_parameter_name = "wrong".to_string()
                }
                "ordinary_parameter_definition_id" => {
                    application.ordinary_parameter_definition_id = "wrong".to_string()
                }
                "ordinary_parameter_name" => {
                    application.ordinary_parameter_name = "wrong".to_string()
                }
                "callable_value_id" => application.callable_value_id = "wrong".to_string(),
                "target_definition_id" => application.target_definition_id = "wrong".to_string(),
                "direct_call_span" => application.direct_call_span.column += 1,
                "indirect_call_span" => application.indirect_call_span.column += 1,
                "direct_statement_span" => application.direct_statement_span.column += 1,
                "indirect_statement_span" => application.indirect_statement_span.column += 1,
                "ordinary_argument" => {
                    application.ordinary_argument = OrdinaryArgumentFact::UIntLiteral(999)
                }
                "result_type" => application.result_type = "Text".to_string(),
                "failure_root" => application.failure_root = Some("Wrong".to_string()),
                "input_row_id" => application.input_row_id = "wrong".to_string(),
                "output_row_id" => application.output_row_id = "wrong".to_string(),
                "status" => application.status = "wrong",
                "reason" => application.reason = "wrong",
                _ => unreachable!(),
            }
            assert!(
                corrupt
                    .verify()
                    .contains(&"callable_fact_source_relationship_corrupt_v0"),
                "application {field}: {:?}",
                corrupt.verify()
            );
        }

        let core_nodes = core_node_facts(&analysis);
        assert!(verify_core_node_facts(&analysis, &core_nodes).is_empty());
        for index in 0..core_nodes.len() {
            for field in ["kind", "id", "relationship_id", "result_type"] {
                let mut corrupt = core_nodes.clone();
                match field {
                    "kind" => corrupt[index].kind = "wrong".to_string(),
                    "id" => corrupt[index].id = "wrong".to_string(),
                    "relationship_id" => corrupt[index].relationship_id = "wrong".to_string(),
                    "result_type" => corrupt[index].result_type = "wrong".to_string(),
                    _ => unreachable!(),
                }
                assert_eq!(
                    verify_core_node_facts(&analysis, &corrupt),
                    ["callable_core_projection_relationship_corrupt_v0"],
                    "core node {index} {field}"
                );
            }
        }

        let graph_edges = graph_edge_facts(&analysis);
        assert!(verify_graph_edge_facts(&analysis, &graph_edges).is_empty());
        for index in 0..graph_edges.len() {
            for field in [
                "id",
                "kind",
                "from",
                "to",
                "owner_definition_id",
                "application_id",
                "source_span",
            ] {
                let mut corrupt = graph_edges.clone();
                match field {
                    "id" => corrupt[index].id = "wrong".to_string(),
                    "kind" => corrupt[index].kind = "wrong".to_string(),
                    "from" => corrupt[index].from = "wrong".to_string(),
                    "to" => corrupt[index].to = "wrong".to_string(),
                    "owner_definition_id" => {
                        corrupt[index].owner_definition_id = "wrong".to_string()
                    }
                    "application_id" => corrupt[index].application_id = "wrong".to_string(),
                    "source_span" => corrupt[index].source_span.column += 1,
                    _ => unreachable!(),
                }
                assert_eq!(
                    verify_graph_edge_facts(&analysis, &corrupt),
                    ["callable_graph_projection_relationship_corrupt_v0"],
                    "graph edge {index} {field}"
                );
            }
        }
    }

    #[test]
    fn am_order_permutation_normalizes_identically() {
        let first_source = AM_SOURCE.replace(
            "change current: UInt = value\n    set current = current + 1\n    set current = current + 1\n    return current",
            "change first: UInt = value\n    change second: UInt = value\n    set first = first + 1\n    set second = second + 1\n    return first",
        );
        let second_source = format!(
            "\n{}",
            first_source.replace(
                "set first = first + 1\n    set second = second + 1",
                "set second = second + 1\n    set first = first + 1",
            )
        );
        let first_parsed = parse_source("order.hum", &first_source);
        let second_parsed = parse_source("order.hum", &second_source);
        let first = analyze_program(&crate::ast::Program {
            files: vec![first_parsed.file],
        });
        let second = analyze_program(&crate::ast::Program {
            files: vec![second_parsed.file],
        });
        let first_sub = &first.substitutions[0];
        let second_sub = &second.substitutions[0];
        let first_row = first
            .rows
            .iter()
            .find(|row| row.id == first_sub.output_row_id)
            .unwrap();
        let second_row = second
            .rows
            .iter()
            .find(|row| row.id == second_sub.output_row_id)
            .unwrap();
        assert_eq!(first_row.normalized_labels, ["change", "change"]);
        assert_eq!(first_row.normalized_labels, second_row.normalized_labels);
        assert_ne!(first_row.labels, second_row.labels);
        assert_ne!(first_sub.tail_id, second_sub.tail_id);
        let first_input = first
            .rows
            .iter()
            .find(|row| row.id == first_sub.input_row_id)
            .unwrap();
        let second_input = second
            .rows
            .iter()
            .find(|row| row.id == second_sub.input_row_id)
            .unwrap();
        assert_eq!(first_input.tail_alias, second_input.tail_alias);
        assert_eq!(first_input.normalized_tail, second_input.normalized_tail);
    }

    #[test]
    fn am_same_spelled_tails_remain_distinct_per_receiver() {
        let source = r#"module tests.distinct_tails

task bump_left(value: UInt) -> UInt {
  does:
    change left: UInt = value
    set left = left + 1
    return left
}

task bump_right(value: UInt) -> UInt {
  does:
    change right: UInt = value
    set right = right + 1
    return right
}

task apply_left(transform: task(UInt) -> UInt, value: UInt) -> UInt {
  does:
    return transform(value)
}

task apply_right(transform: task(UInt) -> UInt, value: UInt) -> UInt {
  does:
    return transform(value)
}

task run_left -> UInt {
  does:
    return apply_left(bump_left, 40)
}

task run_right -> UInt {
  does:
    return apply_right(bump_right, 41)
}
"#;
        let parsed = parse_source("distinct_tails.hum", source);
        let analysis = analyze_program(&crate::ast::Program {
            files: vec![parsed.file],
        });
        assert!(analysis.verify().is_empty(), "{:?}", analysis.verify());
        assert_eq!(analysis.substitutions.len(), 2);
        let first_tail = &analysis.substitutions[0].tail_id;
        let second_tail = &analysis.substitutions[1].tail_id;
        assert_ne!(first_tail, second_tail);
        for substitution in &analysis.substitutions {
            let input = analysis
                .rows
                .iter()
                .find(|row| row.id == substitution.input_row_id)
                .unwrap();
            assert_eq!(input.tail_id.as_ref(), Some(&substitution.tail_id));
            assert_eq!(input.tail_alias.as_deref(), Some("row"));
            assert_eq!(input.normalized_tail.as_deref(), Some("row0"));
        }

        let first_application = analysis
            .applications
            .iter()
            .find(|application| application.id == analysis.substitutions[0].application_id)
            .unwrap();
        let other_occurrence_id = analysis
            .label_occurrences
            .iter()
            .find(|occurrence| {
                occurrence.owner_definition_id != first_application.target_definition_id
            })
            .unwrap()
            .id
            .clone();
        let mut corrupt = (*analysis).clone();
        let substitution = corrupt.substitutions[0].clone();
        for row_id in [substitution.argument_row_id, substitution.output_row_id] {
            corrupt
                .rows
                .iter_mut()
                .find(|row| row.id == row_id)
                .unwrap()
                .labels = vec![other_occurrence_id.clone()];
        }
        assert!(
            corrupt
                .verify()
                .contains(&"callable_row_substitution_relationship_corrupt_v0"),
            "{:?}",
            corrupt.verify()
        );
    }

    #[test]
    fn parser_nodes_drive_one_memoized_callable_analysis() {
        let parsed = parse_source("callable.hum", SOURCE);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let first = analyze_program(&program);
        let second = analyze_program(&program);
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.applications.len(), 1);
        assert!(first.diagnostics.is_empty(), "{:?}", first.diagnostics);
        assert!(first.json("test").contains(CALLABLE_FACT_MODEL));
        let Item::Task(receiver) = &program.files[0].items[1] else {
            panic!("receiver task")
        };
        let ParsedExpressionKind::Call(_) = &match &receiver.body_syntax[0].kind {
            crate::ast::ParsedBodyStatementKind::Return(expression) => expression,
            _ => panic!("return"),
        }
        .kind
        else {
            panic!("parsed call")
        };
    }

    #[test]
    fn corrupt_core_relationships_fail_closed() {
        let parsed = parse_source("callable.hum", SOURCE);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let analysis = analyze_program(&program);
        assert!(analysis.verify().is_empty(), "{:?}", analysis.verify());
        let mut corrupt = (*analysis).clone();
        corrupt.applications[0].output_row_id = "foreign-row".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_application_row_identity_mismatch_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.applications[0].callable_value_id = "poisoned-expected-value".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_application_missing_value_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.applications[0].target_definition_id = "poisoned-target".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_application_target_identity_mismatch_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.definitions[0].input_definition_ids.swap(0, 1);
        assert!(
            corrupt
                .verify()
                .contains(&"callable_definition_resolver_relationship_corrupt_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.values[0].resolved_task_definition_id =
            corrupt.applications[0].receiver_definition_id.clone();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_value_resolver_relationship_corrupt_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.applications[0].callable_parameter_definition_id = corrupt.applications[0]
            .ordinary_parameter_definition_id
            .clone();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_application_resolver_relationship_corrupt_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.applications[0].id = "coherent-looking-application-id".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_application_identity_corrupt_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.applications[0].ordinary_argument = OrdinaryArgumentFact::UIntLiteral(40);
        assert!(
            corrupt
                .verify()
                .contains(&"callable_fact_source_relationship_corrupt_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.applications[0].direct_call_span.column += 1;
        assert!(
            corrupt
                .verify()
                .contains(&"callable_application_resolver_relationship_corrupt_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.definitions.push(corrupt.definitions[0].clone());
        corrupt.applications.push(corrupt.applications[0].clone());
        assert!(
            corrupt
                .verify()
                .contains(&"callable_fact_identity_not_unique_v0")
        );
        for mutate in [
            "label",
            "tail",
            "row_status",
            "row_origin",
            "type_input",
            "type_result",
            "type_failure",
            "definition_inputs",
            "definition_result",
            "definition_failure",
        ] {
            let mut corrupt = (*analysis).clone();
            let expected = match mutate {
                "label" => {
                    corrupt.rows[0].labels.push("io".to_string());
                    "callable_latent_row_relationship_corrupt_v0"
                }
                "tail" => {
                    corrupt.rows[0].tail_id = Some("foreign-tail".to_string());
                    "callable_latent_row_relationship_corrupt_v0"
                }
                "row_status" => {
                    corrupt.rows[0].status = "open_v0";
                    "callable_latent_row_relationship_corrupt_v0"
                }
                "row_origin" => {
                    corrupt.rows[0].origin = "missing_origin_v0";
                    "callable_latent_row_relationship_corrupt_v0"
                }
                "type_input" => {
                    corrupt.types[0].input_types = vec!["Text".to_string()];
                    "callable_type_signature_corrupt_v0"
                }
                "type_result" => {
                    corrupt.types[0].result_type = "Text".to_string();
                    "callable_type_signature_corrupt_v0"
                }
                "type_failure" => {
                    corrupt.types[0].failure_root = Some("Wrong".to_string());
                    "callable_type_signature_corrupt_v0"
                }
                "definition_inputs" => {
                    corrupt.definitions[0].input_definition_ids.clear();
                    "callable_definition_signature_corrupt_v0"
                }
                "definition_result" => {
                    corrupt.definitions[0].result_type = "Text".to_string();
                    "callable_definition_signature_corrupt_v0"
                }
                "definition_failure" => {
                    corrupt.definitions[0].failure_root = Some("Wrong".to_string());
                    "callable_definition_signature_corrupt_v0"
                }
                _ => unreachable!(),
            };
            assert!(
                corrupt.verify().contains(&expected),
                "{mutate}: {:?}",
                corrupt.verify()
            );
        }
    }

    #[test]
    fn diagnostic_id_duplicates_and_rewrites_fail_closed() {
        let parsed = parse_source(
            "invalid_callable.hum",
            "task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {\n  does:\n    return transform\n}\n",
        );
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let analysis = analyze_program(&program);
        assert!(!analysis.diagnostics.is_empty());
        let mut corrupt = (*analysis).clone();
        corrupt.diagnostics[0].id = "rewritten-diagnostic-id".to_string();
        assert!(
            corrupt
                .verify()
                .contains(&"callable_diagnostic_identity_corrupt_v0")
        );
        corrupt = (*analysis).clone();
        corrupt.diagnostics.push(corrupt.diagnostics[0].clone());
        assert!(
            corrupt
                .verify()
                .contains(&"callable_fact_identity_not_unique_v0")
        );
    }

    #[test]
    fn memoization_identity_includes_semantic_ast_and_resolver_inputs() {
        let parsed = parse_source("callable.hum", SOURCE);
        let mut program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let first = analyze_program(&program);
        let Item::Task(receiver) = &mut program.files[0].items[1] else {
            panic!("receiver task")
        };
        let crate::ast::ParsedBodyStatementKind::Return(expression) =
            &mut receiver.body_syntax[0].kind
        else {
            panic!("return")
        };
        expression.kind = ParsedExpressionKind::UIntLiteral(0);
        let second = analyze_program(&program);
        assert!(!Arc::ptr_eq(&first, &second));
        assert!(!second.diagnostics.is_empty());
    }

    #[test]
    fn legacy_section_text_cannot_rescue_a_blocked_parser_fact() {
        let parsed = parse_source(
            "legacy_sabotage.hum",
            "task apply_once(transform: task(UInt) -> UInt, value: UInt) -> UInt {\n  does:\n    return transform(value\n}\n",
        );
        let mut program = crate::ast::Program {
            files: vec![parsed.file],
        };
        let Item::Task(task) = &mut program.files[0].items[0] else {
            panic!("task")
        };
        task.sections
            .iter_mut()
            .find(|section| section.name == "does")
            .expect("does")
            .lines[0]
            .text = "return transform(value)".to_string();
        let analysis = analyze_program(&program);
        assert_eq!(analysis.diagnostics.len(), 1);
        assert_eq!(
            analysis.diagnostics[0].diagnostic().code,
            crate::diagnostic::DiagnosticCode::INVALID_CALLABLE_FORM
        );
        assert_eq!(
            analysis.diagnostics[0].detail_reason,
            "indirect_application_shape_outside_al_v0"
        );
    }
}
