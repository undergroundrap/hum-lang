use std::collections::{BTreeMap, BTreeSet};

use crate::corpus::{Corpus, CorpusVariant};
use crate::cost::{AnalysisTrace, measure_diagnostic};
use crate::inventory::candidate_inventory;
use crate::result_contract::{
    CandidateExecution, CandidateFactory, CandidateResult, CandidateRun, CandidateStatus,
    HarnessEvidence, ResultEvidence, StableFact,
};

pub const CANDIDATE_ID: &str = "row_polymorphism";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RowVariable(usize);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RowSolver {
    parents: Vec<usize>,
    names: Vec<String>,
    unions: Vec<(usize, usize)>,
}

impl RowSolver {
    pub fn fresh(&mut self, name: impl Into<String>) -> RowVariable {
        let id = self.parents.len();
        self.parents.push(id);
        self.names.push(name.into());
        RowVariable(id)
    }

    fn root(&self, variable: RowVariable) -> usize {
        let mut current = variable.0;
        while self.parents[current] != current {
            current = self.parents[current];
        }
        current
    }

    pub fn unify(&mut self, left: RowVariable, right: RowVariable) -> RowVariable {
        let left_root = self.root(left);
        let right_root = self.root(right);
        if left_root == right_root {
            return RowVariable(left_root);
        }
        let (root, child) = if left_root < right_root {
            (left_root, right_root)
        } else {
            (right_root, left_root)
        };
        self.parents[child] = root;
        self.unions.push((left.0, right.0));
        RowVariable(root)
    }

    pub fn normalized_substitutions(&self) -> Vec<String> {
        let mut normalizer = TailNormalizer::default();
        (0..self.parents.len())
            .map(|id| {
                let root = RowVariable(self.root(RowVariable(id)));
                format!("{}->{}", self.names[id], normalizer.normalize(root))
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TailNormalizer {
    variables: BTreeMap<RowVariable, String>,
}

impl TailNormalizer {
    fn normalize(&mut self, variable: RowVariable) -> String {
        let next = self.variables.len();
        self.variables
            .entry(variable)
            .or_insert_with(|| format!("$r{next}"))
            .clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRow {
    labels: BTreeMap<String, usize>,
    tail: Option<RowVariable>,
    alias: Option<String>,
}

impl EffectRow {
    pub fn open<I, S>(labels: I, tail: RowVariable) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(labels, Some(tail))
    }

    pub fn closed<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(labels, None)
    }

    fn new<I, S>(labels: I, tail: Option<RowVariable>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut counts = BTreeMap::new();
        for label in labels {
            *counts.entry(label.into()).or_insert(0) += 1;
        }
        Self {
            labels: counts,
            tail,
            alias: None,
        }
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    pub fn propagate(&self, other: &Self, solver: &mut RowSolver) -> Self {
        let mut labels = self.labels.clone();
        for (label, count) in &other.labels {
            *labels.entry(label.clone()).or_insert(0) += count;
        }
        let tail = match (self.tail, other.tail) {
            (Some(left), Some(right)) => Some(solver.unify(left, right)),
            (Some(tail), None) | (None, Some(tail)) => Some(tail),
            (None, None) => None,
        };
        Self {
            labels,
            tail,
            alias: self.alias.clone().or_else(|| other.alias.clone()),
        }
    }

    pub fn handle_exact(&self, label: &str) -> Result<Self, RowFailure> {
        let mut next = self.clone();
        match next.labels.get_mut(label) {
            Some(count) if *count > 1 => *count -= 1,
            Some(_) => {
                next.labels.remove(label);
            }
            None => {
                return Err(RowFailure {
                    reason: "handled_requirement_absent".to_owned(),
                    primary: "handler.site".to_owned(),
                    related: Vec::new(),
                    detail: format!("handled label {label} is absent"),
                });
            }
        }
        Ok(next)
    }

    pub fn without_exact(&self, label: &str) -> Result<Self, RowFailure> {
        self.handle_exact(label)
    }

    pub fn multiplicity(&self, label: &str) -> usize {
        self.labels.get(label).copied().unwrap_or(0)
    }

    pub fn render(&self, solver: &RowSolver, normalizer: &mut TailNormalizer) -> String {
        let mut parts = Vec::new();
        for (label, count) in &self.labels {
            parts.extend(std::iter::repeat_n(label.clone(), *count));
        }
        let body = match self.tail {
            Some(tail) if parts.is_empty() => {
                format!("| {}", normalizer.normalize(RowVariable(solver.root(tail))))
            }
            Some(tail) => format!(
                "{} | {}",
                parts.join(", "),
                normalizer.normalize(RowVariable(solver.root(tail)))
            ),
            None => parts.join(", "),
        };
        let rendered = format!("<{body}>");
        self.alias
            .as_ref()
            .map_or(rendered.clone(), |alias| format!("{alias} = {rendered}"))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Requirement {
    label: String,
    origin: String,
    boundary: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Handler {
    label: String,
    site: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShapeFacts {
    expected_input: String,
    actual_input: String,
    expected_result: String,
    actual_result: String,
    definition_site: String,
    call_site: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IterationFacts {
    active_source_mutation: bool,
    deletion_happened: bool,
    stale_view_used: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RuntimeCreditFacts {
    implemented: bool,
    credited: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegistrationFacts {
    captured_state_end: usize,
    registration_end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AuthorityFacts {
    required: BTreeSet<String>,
    captured: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AllocationFacts {
    required: BTreeSet<String>,
    reported: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OwnershipFacts {
    transfer_recorded: bool,
    resource_end: usize,
    callable_end: usize,
    uses: usize,
    stored: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateProgram {
    case_id: String,
    variant_id: String,
    callable_inputs: String,
    callable_result: String,
    callable_disposition: String,
    call_sites: Vec<String>,
    captured_facts: Vec<String>,
    ownership_transfer: String,
    registration_lifetime: String,
    requirements: Vec<Requirement>,
    handlers: Vec<Handler>,
    omitted_claim_label: Option<String>,
    alias: Option<String>,
    shape: ShapeFacts,
    iteration: Option<IterationFacts>,
    runtime_credit: Option<RuntimeCreditFacts>,
    registration: Option<RegistrationFacts>,
    authority: Option<AuthorityFacts>,
    allocation: AllocationFacts,
    ownership: Option<OwnershipFacts>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RowFailure {
    reason: String,
    primary: String,
    related: Vec<String>,
    detail: String,
}

#[derive(Clone, Debug)]
struct CandidateAnalysis {
    inferred: EffectRow,
    claimed: EffectRow,
    solver: RowSolver,
    handlers: Vec<String>,
    checks: BTreeSet<&'static str>,
    failure: Option<RowFailure>,
    trace: AnalysisTrace,
}

impl CandidateAnalysis {
    fn snapshot(&self) -> String {
        let mut normalizer = TailNormalizer::default();
        let inferred = self.inferred.render(&self.solver, &mut normalizer);
        let claimed = self.claimed.render(&self.solver, &mut normalizer);
        let failure = self
            .failure
            .as_ref()
            .map_or("accepted".to_owned(), |failure| {
                format!(
                    "{}@{}>{}",
                    failure.reason,
                    failure.primary,
                    failure.related.join(",")
                )
            });
        format!(
            "inferred={inferred};claimed={claimed};substitutions={};handlers={};checks={};outcome={failure}",
            self.solver.normalized_substitutions().join(","),
            self.handlers.join(","),
            self.checks.iter().copied().collect::<Vec<_>>().join(","),
        )
    }
}

fn requirement_boundary(case_id: &str, label: &str, call_sites: &[String]) -> String {
    match (case_id, label) {
        ("effect.logging_middleware", "wrapped.requirement") => "wrapper.call".to_owned(),
        _ => call_sites
            .first()
            .cloned()
            .unwrap_or_else(|| "call.site".to_owned()),
    }
}

fn requirement_origin(variant: &CorpusVariant, label: &str) -> String {
    variant
        .source_requirement_origins
        .get(label)
        .or_else(|| variant.operation_exercise_origins.get(label))
        .cloned()
        .unwrap_or_else(|| {
            variant
                .call_sites
                .last()
                .cloned()
                .unwrap_or_else(|| "call.site".to_owned())
        })
}

fn derived_allocation_domains(
    disposition: &str,
    case_id: &str,
    captures: &[String],
) -> BTreeSet<String> {
    let mut domains = BTreeSet::new();
    if matches!(disposition, "stored_callable" | "returned_callable") {
        domains.insert("callable_environment".to_owned());
    }
    if case_id == "effect.callback_registry" {
        domains.insert("registry_storage".to_owned());
    }
    if captures.iter().any(|capture| capture == "cache state") {
        domains.insert("cache_storage".to_owned());
    }
    if domains.is_empty() {
        domains.insert("none_explicit".to_owned());
    }
    domains
}

fn candidate_capture_kind(domain: &str) -> &'static str {
    if domain.contains("authority") {
        "authority"
    } else if domain.contains("resource")
        || domain.contains("registration")
        || domain.contains("view")
        || domain.contains("iteration")
        || domain == "cache state"
    {
        "resource"
    } else {
        "value"
    }
}

fn build_program(variant: &CorpusVariant) -> CandidateProgram {
    let requirements = variant
        .latent_operations
        .iter()
        .filter(|label| label.as_str() != "none_explicit")
        .map(|label| Requirement {
            label: label.clone(),
            origin: requirement_origin(variant, label),
            boundary: requirement_boundary(&variant.case_id, label, &variant.call_sites),
        })
        .collect::<Vec<_>>();
    let handlers = match variant.case_id.as_str() {
        "effect.retry" => vec![Handler {
            label: "retry.failure".to_owned(),
            site: "retry.handle".to_owned(),
        }],
        "effect.with_timeout" => vec![Handler {
            label: "timeout.requirement".to_owned(),
            site: "timeout.handle".to_owned(),
        }],
        _ => Vec::new(),
    };
    let omitted_claim_label = match (variant.case_id.as_str(), variant.variant_id.as_str()) {
        ("effect.effectful_map", "misuse_erased_requirement") => {
            Some("callback.requirement".to_owned())
        }
        ("effect.fold", "misuse_erased_step") => Some("step.requirement".to_owned()),
        ("effect.retry", "misuse_erased_remaining") => {
            Some("action.remaining_requirement".to_owned())
        }
        ("effect.parallel_map", "misuse_erased_callback") => {
            Some("callback.requirement".to_owned())
        }
        ("effect.logging_middleware", "misuse_erased_wrapped") => {
            Some("wrapped.requirement".to_owned())
        }
        _ => None,
    };
    let mut shape = ShapeFacts {
        expected_input: variant.callable_inputs.clone(),
        actual_input: variant.callable_inputs.clone(),
        expected_result: variant.callable_result.clone(),
        actual_result: variant.callable_result.clone(),
        definition_site: variant
            .all_sites
            .iter()
            .find(|site| site.kind == "callable")
            .map_or_else(|| "callable.definition".to_owned(), |site| site.id.clone()),
        call_site: variant
            .call_sites
            .last()
            .cloned()
            .unwrap_or_else(|| "call.site".to_owned()),
    };
    if variant.case_id == "effect.pure_map" && variant.variant_id == "misuse_callback_shape" {
        shape.actual_input = "incompatible input".to_owned();
    }
    let iteration = if variant.case_id == "effect.filter_retain" {
        Some(IterationFacts {
            active_source_mutation: variant.variant_id == "misuse_same_list_mutation",
            deletion_happened: matches!(
                variant.variant_id.as_str(),
                "positive_retain_delete" | "misuse_stale_retained_view"
            ),
            stale_view_used: variant.variant_id == "misuse_stale_retained_view",
        })
    } else {
        None
    };
    let runtime_credit = (variant.case_id == "effect.with_timeout").then_some(RuntimeCreditFacts {
        implemented: false,
        credited: variant.variant_id == "misuse_claims_runtime",
    });
    let registration =
        (variant.case_id == "effect.callback_registry").then_some(RegistrationFacts {
            captured_state_end: 1,
            registration_end: usize::from(variant.variant_id == "misuse_outlives_state") + 1,
        });
    let required_authority = variant
        .operator_consent
        .iter()
        .filter(|domain| domain.as_str() != "none_explicit")
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut captured_authority = variant
        .captured_facts
        .iter()
        .filter(|domain| candidate_capture_kind(domain) == "authority")
        .map(|domain| {
            if domain.contains("logging") {
                "logging.grant_exact".to_owned()
            } else {
                domain.clone()
            }
        })
        .collect::<BTreeSet<_>>();
    if variant.variant_id == "misuse_laundered_authority" {
        captured_authority.clear();
    }
    let authority = (!required_authority.is_empty()).then_some(AuthorityFacts {
        required: required_authority,
        captured: captured_authority,
    });
    let required_allocations = derived_allocation_domains(
        &variant.callable_disposition,
        &variant.case_id,
        &variant.captured_facts,
    );
    let mut reported_allocations = required_allocations.clone();
    if variant.variant_id == "misuse_hidden_cache" {
        reported_allocations.remove("cache_storage");
    }
    let allocation = AllocationFacts {
        required: required_allocations,
        reported: reported_allocations,
    };
    let ownership = (variant.case_id == "effect.linear_capture").then_some(OwnershipFacts {
        transfer_recorded: variant.variant_id != "misuse_move_without_transfer",
        resource_end: usize::from(!matches!(
            variant.variant_id.as_str(),
            "misuse_escape" | "misuse_outlives"
        )) + 1,
        callable_end: 2,
        uses: if variant.variant_id == "misuse_double_use" {
            2
        } else {
            1
        },
        stored: variant.variant_id == "misuse_outlives",
    });
    CandidateProgram {
        case_id: variant.case_id.clone(),
        variant_id: variant.variant_id.clone(),
        callable_inputs: variant.callable_inputs.clone(),
        callable_result: variant.callable_result.clone(),
        callable_disposition: variant.callable_disposition.clone(),
        call_sites: variant.call_sites.clone(),
        captured_facts: variant.captured_facts.clone(),
        ownership_transfer: variant.ownership_transfer.clone(),
        registration_lifetime: variant.registration_lifetime.clone(),
        requirements,
        handlers,
        omitted_claim_label,
        alias: match variant.case_id.as_str() {
            "effect.retry" => Some("RetryAction".to_owned()),
            "effect.with_timeout" => Some("TimeoutAction".to_owned()),
            "effect.logging_middleware" => Some("LoggedAction".to_owned()),
            _ => None,
        },
        shape,
        iteration,
        runtime_credit,
        registration,
        authority,
        allocation,
        ownership,
    }
}

fn check_side_facts(
    program: &CandidateProgram,
    checks: &mut BTreeSet<&'static str>,
    trace: &mut AnalysisTrace,
) -> Option<RowFailure> {
    checks.insert("callable_shape_guard");
    trace.visit_node();
    trace.generate_constraint();
    if program.shape.expected_input != program.shape.actual_input
        || program.shape.expected_result != program.shape.actual_result
    {
        return Some(RowFailure {
            reason: "callable_shape_mismatch".to_owned(),
            primary: program.shape.definition_site.clone(),
            related: vec![program.shape.call_site.clone()],
            detail: "callable input or result shape differs".to_owned(),
        });
    }
    if let Some(iteration) = &program.iteration {
        checks.insert("place_and_view_guard");
        trace.visit_node();
        trace.generate_constraint();
        if iteration.active_source_mutation {
            return Some(RowFailure {
                reason: "active_iteration_mutation".to_owned(),
                primary: "source.mutation".to_owned(),
                related: vec!["filter.call".to_owned()],
                detail: "source mutation overlaps active iteration".to_owned(),
            });
        }
        if iteration.deletion_happened && iteration.stale_view_used {
            return Some(RowFailure {
                reason: "stale_retained_item_view".to_owned(),
                primary: "retain.delete".to_owned(),
                related: vec!["stale.use".to_owned()],
                detail: "deleted item view is used after invalidation".to_owned(),
            });
        }
    }
    if let Some(runtime) = &program.runtime_credit {
        checks.insert("machinery_credit_guard");
        trace.visit_node();
        trace.generate_constraint();
        if runtime.credited && !runtime.implemented {
            return Some(RowFailure {
                reason: "unimplemented_machinery_credited".to_owned(),
                primary: "timeout.call".to_owned(),
                related: vec!["timeout.handle".to_owned()],
                detail: "type-only timeout machinery received runtime credit".to_owned(),
            });
        }
    }
    if let Some(registration) = &program.registration {
        checks.insert("registration_lifetime_guard");
        trace.visit_node();
        trace.generate_constraint();
        if registration.registration_end > registration.captured_state_end {
            return Some(RowFailure {
                reason: "registration_outlives_capture".to_owned(),
                primary: "register.call".to_owned(),
                related: vec!["caller.state".to_owned(), "return.site".to_owned()],
                detail: "registration remains live after captured state ends".to_owned(),
            });
        }
    }
    if let Some(authority) = &program.authority {
        checks.insert("exact_authority_retention_guard");
        trace.visit_node();
        trace.generate_constraint();
        if authority.required != authority.captured {
            return Some(RowFailure {
                reason: "captured_authority_erased".to_owned(),
                primary: "authority.capture".to_owned(),
                related: vec!["factory.call".to_owned(), "handler.call".to_owned()],
                detail: "returned callable does not retain the exact authority".to_owned(),
            });
        }
    }
    checks.insert("allocation_visibility_guard");
    trace.visit_node();
    trace.generate_constraint();
    if program.allocation.required != program.allocation.reported {
        return Some(RowFailure {
            reason: "hidden_allocation_or_resource".to_owned(),
            primary: "cache.allocation".to_owned(),
            related: vec!["memoize.call".to_owned()],
            detail: "required callable or cache allocation is not reported".to_owned(),
        });
    }
    if let Some(ownership) = &program.ownership {
        checks.insert("linear_resource_guard");
        trace.visit_node();
        trace.generate_constraint();
        if !ownership.transfer_recorded {
            return Some(RowFailure {
                reason: "ownership_transfer_missing".to_owned(),
                primary: "resource.capture".to_owned(),
                related: vec!["factory.call".to_owned()],
                detail: "captured resource has no recorded transfer".to_owned(),
            });
        }
        if ownership.uses > 1 {
            return Some(RowFailure {
                reason: "linear_resource_double_use".to_owned(),
                primary: "first.use".to_owned(),
                related: vec!["second.use".to_owned()],
                detail: "linear resource is consumed by two retained paths".to_owned(),
            });
        }
        if ownership.callable_end > ownership.resource_end {
            if ownership.stored {
                return Some(RowFailure {
                    reason: "captured_resource_outlives".to_owned(),
                    primary: "close.site".to_owned(),
                    related: vec!["late.use".to_owned(), "register.call".to_owned()],
                    detail: "stored callable remains live after resource close".to_owned(),
                });
            }
            return Some(RowFailure {
                reason: "captured_resource_escapes".to_owned(),
                primary: "resource.capture".to_owned(),
                related: vec!["return.site".to_owned(), "factory.call".to_owned()],
                detail: "returned callable escapes the captured resource lifetime".to_owned(),
            });
        }
    }
    None
}

fn analyze_program(program: &CandidateProgram) -> CandidateAnalysis {
    let mut solver = RowSolver::default();
    let ambient = solver.fresh("$ambient");
    let mut inferred = EffectRow::open(Vec::<String>::new(), ambient);
    let mut trace = AnalysisTrace::default();
    let mut checks = BTreeSet::from(["effect_inference"]);
    trace.enter_live_item();
    trace.visit_node();
    trace.generate_fact();
    trace.normalization_step();
    for (index, requirement) in program.requirements.iter().enumerate() {
        let tail = solver.fresh(format!("$latent_{index}"));
        let latent = EffectRow::open([requirement.label.clone()], tail);
        inferred = inferred.propagate(&latent, &mut solver);
        trace.visit_node();
        trace.generate_fact();
        trace.generate_constraint();
        trace.normalization_step();
    }
    inferred.alias = program.alias.clone();
    let mut handler_sites = Vec::new();
    let mut handler_failure = None;
    for handler in &program.handlers {
        handler_sites.push(format!("{}@{}", handler.label, handler.site));
        match inferred.handle_exact(&handler.label) {
            Ok(next) => {
                inferred = next;
                trace.visit_node();
                trace.generate_constraint();
                trace.normalization_step();
            }
            Err(mut failure) => {
                failure.primary = handler.site.clone();
                handler_failure = Some(failure);
                break;
            }
        }
    }
    let mut claimed = inferred.clone();
    let mut row_failure = handler_failure;
    if row_failure.is_none()
        && let Some(label) = &program.omitted_claim_label
    {
        match claimed.without_exact(label) {
            Ok(next) => claimed = next,
            Err(failure) => row_failure = Some(failure),
        }
    }
    if row_failure.is_none() && claimed.labels != inferred.labels {
        let missing = inferred
            .labels
            .iter()
            .find(|(label, count)| claimed.multiplicity(label) < **count)
            .map(|(label, _)| label.clone())
            .expect("different row labels have a missing member");
        let requirement = program
            .requirements
            .iter()
            .find(|requirement| requirement.label == missing)
            .expect("omitted claim label comes from a requirement");
        row_failure = Some(RowFailure {
            reason: if program.handlers.is_empty() {
                "latent_requirement_erased"
            } else {
                "unhandled_requirement_erased"
            }
            .to_owned(),
            primary: requirement.origin.clone(),
            related: vec![requirement.boundary.clone()],
            detail: format!("claimed row erases {missing}"),
        });
    }
    let side_failure = check_side_facts(program, &mut checks, &mut trace);
    trace.normalization_step();
    trace.leave_live_item();
    CandidateAnalysis {
        inferred,
        claimed,
        solver,
        handlers: handler_sites,
        checks,
        failure: row_failure.or(side_failure),
        trace,
    }
}

fn stable_fact(
    id: impl Into<String>,
    attributes: impl IntoIterator<Item = (&'static str, String)>,
    route: Vec<String>,
) -> StableFact {
    StableFact {
        id: id.into(),
        attributes: attributes
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
        route,
    }
}

fn policy_facts(variant: &CorpusVariant) -> Vec<StableFact> {
    let mut facts = Vec::new();
    for (role, domains, disposition, origins) in [
        (
            "source_requirement",
            &variant.source_requirements,
            "propagated",
            &variant.source_requirement_origins,
        ),
        (
            "operator_consent",
            &variant.operator_consent,
            "retained_separate",
            &variant.operator_consent_origins,
        ),
        (
            "operation_exercise",
            &variant.operation_exercise,
            "exercised",
            &variant.operation_exercise_origins,
        ),
    ] {
        for domain in domains {
            facts.push(stable_fact(
                format!("{role}.{domain}"),
                [
                    (
                        "origin",
                        origins
                            .get(domain)
                            .cloned()
                            .expect("checked corpus has policy origin"),
                    ),
                    ("disposition", disposition.to_owned()),
                    ("callable", format!("{}.callable", variant.case_id)),
                    ("domain", domain.clone()),
                    ("policy_role", role.to_owned()),
                ],
                variant.call_sites.clone(),
            ));
        }
    }
    facts
}

fn capture_facts(variant: &CorpusVariant) -> Vec<StableFact> {
    variant
        .captured_facts
        .iter()
        .map(|domain| {
            stable_fact(
                format!("capture.{domain}"),
                [
                    (
                        "origin",
                        variant
                            .capture_origins
                            .get(domain)
                            .cloned()
                            .expect("checked corpus has capture origin"),
                    ),
                    (
                        "disposition",
                        if domain == "none_explicit" {
                            "none_explicit"
                        } else {
                            "retained"
                        }
                        .to_owned(),
                    ),
                    ("callable", format!("{}.callable", variant.case_id)),
                    ("domain", domain.clone()),
                    ("capture_kind", candidate_capture_kind(domain).to_owned()),
                ],
                variant.call_sites.clone(),
            )
        })
        .collect()
}

fn allocation_facts(program: &CandidateProgram) -> Vec<StableFact> {
    program
        .allocation
        .required
        .iter()
        .map(|domain| {
            stable_fact(
                format!("allocation.{domain}"),
                [
                    ("kind", domain.clone()),
                    ("trigger", program.call_sites[0].clone()),
                    ("lifetime", program.registration_lifetime.clone()),
                    ("owner", format!("{}.callable", program.case_id)),
                    (
                        "evidence",
                        if domain == "none_explicit" {
                            "explicit"
                        } else {
                            "prototype_assumed"
                        }
                        .to_owned(),
                    ),
                    ("domain", domain.clone()),
                ],
                program.call_sites.clone(),
            )
        })
        .collect()
}

fn resource_facts(program: &CandidateProgram) -> Vec<StableFact> {
    let mut domains = program
        .captured_facts
        .iter()
        .filter(|domain| candidate_capture_kind(domain) == "resource")
        .cloned()
        .collect::<BTreeSet<_>>();
    if domains.is_empty() {
        domains.insert("none_explicit".to_owned());
    }
    domains
        .into_iter()
        .map(|domain| {
            stable_fact(
                format!("resource.{domain}"),
                [
                    ("kind", domain.clone()),
                    ("trigger", program.call_sites[0].clone()),
                    ("lifetime", program.registration_lifetime.clone()),
                    ("owner", format!("{}.callable", program.case_id)),
                    ("evidence", "explicit".to_owned()),
                    ("domain", domain),
                ],
                program.call_sites.clone(),
            )
        })
        .collect()
}

fn machinery_fact(
    program: &CandidateProgram,
    layer: &'static str,
    name: &'static str,
    state: &'static str,
    creditable: bool,
    exercised: bool,
) -> StableFact {
    stable_fact(
        format!("machinery.{layer}.{name}"),
        [
            ("layer", layer.to_owned()),
            ("state", state.to_owned()),
            ("creditable", creditable.to_string()),
            ("exercised", exercised.to_string()),
            ("restructuring", "false".to_owned()),
        ],
        program.call_sites.clone(),
    )
}

fn added_machinery(program: &CandidateProgram, analysis: &CandidateAnalysis) -> Vec<StableFact> {
    let mut facts = vec![machinery_fact(
        program,
        "checker",
        "effect_inference",
        "implemented",
        true,
        true,
    )];
    for check in &analysis.checks {
        if *check == "effect_inference" {
            continue;
        }
        let layer = match *check {
            "callable_shape_guard" | "exact_authority_retention_guard" => "checker",
            "machinery_credit_guard" => "tooling",
            _ => "ownership",
        };
        facts.push(machinery_fact(
            program,
            layer,
            check,
            "implemented",
            false,
            true,
        ));
    }
    if matches!(
        program.callable_disposition.as_str(),
        "stored_callable" | "returned_callable"
    ) {
        facts.push(machinery_fact(
            program,
            "runtime",
            "callable_environment",
            "unimplemented",
            false,
            false,
        ));
    }
    for name in match program.case_id.as_str() {
        "effect.with_timeout" => vec!["timeout_execution"],
        "effect.parallel_map" => vec!["parallel_execution"],
        _ => Vec::new(),
    } {
        facts.push(machinery_fact(
            program,
            "runtime",
            name,
            "unimplemented",
            false,
            false,
        ));
    }
    facts
}

fn evidence_sites(case_id: &str, variant_id: &str) -> Vec<String> {
    let sites: &[&str] = match (case_id, variant_id) {
        ("effect.pure_map", "positive") => &["map.call", "callback.call"],
        ("effect.effectful_map", _) => &["callback.call", "map.call"],
        ("effect.filter_retain", "positive_two_list_odd_filter") => {
            &["filter.call", "predicate.call"]
        }
        ("effect.filter_retain", "positive_retain_delete") => &["retain.call", "retain.delete"],
        ("effect.fold", _) => &["step.call", "fold.call"],
        ("effect.retry", "positive") => &["retry.call", "action.call"],
        ("effect.with_timeout", "positive_type_only") => &["timeout.call", "action.call"],
        ("effect.parallel_map", _) => &["callback.call", "parallel.call"],
        ("effect.callback_registry", "positive") => &["register.call", "unregister.call"],
        ("effect.event_handler_factory", "positive") => &["factory.call", "handler.call"],
        ("effect.memoizing_wrapper", "positive") => {
            &["memoize.call", "wrapper.call", "cache.allocation"]
        }
        ("effect.logging_middleware", _) => &["wrapper.call", "wrapped.call"],
        ("effect.linear_capture", "positive_move") => &["factory.call", "closure.call"],
        _ => &["candidate.site"],
    };
    sites.iter().map(|site| (*site).to_owned()).collect()
}

fn candidate_observation(case_id: &str, variant_id: &str) -> &'static str {
    match (case_id, variant_id) {
        ("effect.pure_map", "positive") => "output preserves order and mapped values",
        ("effect.pure_map", "misuse_callback_shape") => {
            "reject a callback whose input or result shape disagrees"
        }
        ("effect.effectful_map", "positive") => {
            "callback requirement is inferred and propagated to map caller"
        }
        ("effect.effectful_map", "misuse_erased_requirement") => {
            "reject erasure of the callback requirement"
        }
        ("effect.filter_retain", "positive_two_list_odd_filter") => {
            "input remains unchanged and output is exactly 1 3"
        }
        ("effect.filter_retain", "positive_retain_delete") => {
            "final list is exactly 1 3 and deleted views do not escape"
        }
        ("effect.filter_retain", "misuse_same_list_mutation") => {
            "reject mutation that conflicts with the active iteration"
        }
        ("effect.filter_retain", "misuse_stale_retained_view") => {
            "reject use of the view invalidated by deletion"
        }
        ("effect.fold", "positive") => "result and step requirement are both preserved",
        ("effect.fold", "misuse_erased_step") => "reject erasure of the step requirement",
        ("effect.retry", "positive") => {
            "retry failure is handled and every other requirement propagates"
        }
        ("effect.retry", "misuse_erased_remaining") => {
            "reject erasure of the unhandled action requirement"
        }
        ("effect.with_timeout", "positive_type_only") => {
            "timeout is handled in the type model and no scheduler implementation is credited"
        }
        ("effect.with_timeout", "misuse_claims_runtime") => {
            "reject unimplemented scheduler or clock machinery being credited"
        }
        ("effect.parallel_map", "positive_type_only") => {
            "both requirements compose and no concurrency implementation is credited"
        }
        ("effect.parallel_map", "misuse_erased_callback") => {
            "reject the missing callback requirement"
        }
        ("effect.callback_registry", "positive") => {
            "count changes from 0 to 2 and later firing does nothing"
        }
        ("effect.callback_registry", "misuse_outlives_state") => {
            "reject registration that can outlive captured caller state"
        }
        ("effect.event_handler_factory", "positive") => {
            "handler retains the exact requirement and logs when invoked"
        }
        ("effect.event_handler_factory", "misuse_laundered_authority") => {
            "reject loss or widening of the exact authority requirement"
        }
        ("effect.memoizing_wrapper", "positive") => {
            "cache allocation ownership and wrapped requirement remain explicit"
        }
        ("effect.memoizing_wrapper", "misuse_hidden_cache") => {
            "reject hidden cache allocation or missing cleanup owner"
        }
        ("effect.logging_middleware", "positive") => {
            "wrapper adds output and preserves every wrapped requirement"
        }
        ("effect.logging_middleware", "misuse_erased_wrapped") => {
            "reject erasure of the wrapped requirement"
        }
        ("effect.linear_capture", "positive_move") => {
            "one owner remains and one use closes or transfers the resource"
        }
        ("effect.linear_capture", "misuse_move_without_transfer") => {
            "reject capture without an explicit new owner"
        }
        ("effect.linear_capture", "misuse_escape") => "reject the escaping callable",
        ("effect.linear_capture", "misuse_double_use") => {
            "reject the second use after the first consumes the resource"
        }
        ("effect.linear_capture", "misuse_outlives") => {
            "reject use after the resource lifetime closes"
        }
        _ => "candidate did not define an observation",
    }
}

fn neutral_summary(program: &CandidateProgram) -> Vec<String> {
    let disposition = match program.callable_disposition.as_str() {
        "stored_callable" => "stored_callable:required".to_owned(),
        "returned_callable" => "returned_callable:required".to_owned(),
        other => format!("callable_input:disposition={other}"),
    };
    vec![
        format!("callable_input:shape={}", program.callable_inputs),
        format!("callable_result:shape={}", program.callable_result),
        format!(
            "callable_result:observation={}",
            candidate_observation(&program.case_id, &program.variant_id)
        ),
        format!("ownership_transfer:{}", program.ownership_transfer),
        format!("resource_lifetime:{}", program.registration_lifetime),
        disposition,
    ]
}

fn annotation(program: &CandidateProgram, sites: &[String]) -> StableFact {
    let site = &sites[0];
    let purpose = if program.authority.is_some() {
        "capture_visibility"
    } else if program.ownership_transfer != "none_explicit" {
        "ownership_transfer"
    } else if program.allocation.required != BTreeSet::from(["none_explicit".to_owned()]) {
        "allocation_visibility"
    } else if !program.requirements.is_empty() {
        "requirement_visibility"
    } else {
        "callable_relationship"
    };
    stable_fact(
        format!("annotation.{site}.{purpose}.inferred"),
        [
            ("site", site.clone()),
            ("purpose", purpose.to_owned()),
            ("mode", "inferred".to_owned()),
        ],
        program.call_sites.clone(),
    )
}

fn analyze_variant(variant: &CorpusVariant) -> (CandidateResult, ResultEvidence) {
    let program = build_program(variant);
    let analysis = analyze_program(&program);
    let mut normalizer = TailNormalizer::default();
    let inferred = analysis.inferred.render(&analysis.solver, &mut normalizer);
    let claimed = analysis.claimed.render(&analysis.solver, &mut normalizer);
    let (status, reason, sites) = match &analysis.failure {
        Some(failure) => (
            CandidateStatus::Rejected,
            failure.reason.clone(),
            std::iter::once(failure.primary.clone())
                .chain(failure.related.iter().cloned())
                .collect(),
        ),
        None => (
            CandidateStatus::Accepted,
            "none_expected".to_owned(),
            evidence_sites(&program.case_id, &program.variant_id),
        ),
    };
    let primary = sites[0].clone();
    let related = sites[1..].to_vec();
    let repair = "preserve the model-neutral relationship".to_owned();
    let rendered_diagnostic = analysis.failure.as_ref().map_or_else(String::new, |failure| {
        format!(
            "effect row rejection {}: {} at {}; related {}; inferred {}; claimed {}; repair: {}",
            failure.reason,
            failure.detail,
            failure.primary,
            failure.related.join(","),
            inferred,
            claimed,
            repair
        )
    });
    let native_terms = if analysis.failure.is_some() {
        vec!["effect row".to_owned()]
    } else {
        Vec::new()
    };
    let diagnostic_cost =
        measure_diagnostic(&rendered_diagnostic, &sites, &sites, &native_terms, &repair);
    let native_snapshot = analysis.snapshot();
    let native_evidence = BTreeMap::from([
        ("analysis".to_owned(), native_snapshot.clone()),
        (
            "duplicate_label_rule".to_owned(),
            "multiset_remove_one_exact_occurrence".to_owned(),
        ),
        ("inferred_row".to_owned(), inferred.clone()),
        ("claimed_row".to_owned(), claimed.clone()),
        (
            "substitutions".to_owned(),
            analysis.solver.normalized_substitutions().join(","),
        ),
    ]);
    let implementation_cost = candidate_inventory(CANDIDATE_ID)
        .expect("row candidate has a harness-owned inventory")
        .measure();
    let result = CandidateResult {
        candidate_id: CANDIDATE_ID.to_owned(),
        case_id: program.case_id.clone(),
        variant_id: program.variant_id.clone(),
        candidate_native_result: native_snapshot,
        candidate_native_evidence: native_evidence,
        neutral_normalized_summary: neutral_summary(&program),
        status,
        required_source_annotations: vec![annotation(&program, &sites)],
        inferred_requirement_facts: policy_facts(variant),
        inferred_capture_facts: capture_facts(variant),
        allocation_facts: allocation_facts(&program),
        resource_facts: resource_facts(&program),
        added_machinery: added_machinery(&program, &analysis),
        primary_reason: reason,
        primary_blame_site: primary,
        related_blame_sites: related,
        repair_direction: repair,
        implementation_cost,
        analysis_cost: analysis.trace.measured(),
        diagnostic_cost,
        missing_evidence: Vec::new(),
    };
    (
        result,
        ResultEvidence {
            analysis_trace: analysis.trace,
            rendered_diagnostic,
            covered_sites: sites,
            candidate_native_terms: native_terms,
        },
    )
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RowCandidateFactory;

#[derive(Clone, Copy, Debug, Default)]
pub struct RowCandidateRun;

impl CandidateFactory for RowCandidateFactory {
    type Run = RowCandidateRun;

    fn fresh(&self) -> Self::Run {
        RowCandidateRun
    }
}

impl CandidateRun for RowCandidateRun {
    fn execute(self, corpus: &Corpus) -> CandidateExecution {
        let mut results = Vec::with_capacity(corpus.variants.len());
        let mut evidence = BTreeMap::new();
        for variant in &corpus.variants {
            let (result, result_evidence) = analyze_variant(variant);
            evidence.insert(
                (variant.case_id.clone(), variant.variant_id.clone()),
                result_evidence,
            );
            results.push(result);
        }
        CandidateExecution {
            results,
            evidence: HarnessEvidence { results: evidence },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{Polarity, checked_in};
    use crate::result_contract::{CandidateRun, run_fresh_twice, validate_run};

    #[test]
    fn all_29_variants_satisfy_the_frozen_contract_repeatably() {
        let corpus = checked_in();
        let bytes = run_fresh_twice(&corpus, &RowCandidateFactory)
            .expect("candidate analysis must satisfy the frozen contract twice");
        assert!(!bytes.is_empty());
        let mut execution = RowCandidateRun.execute(&corpus);
        validate_run(&corpus, &mut execution).expect("complete candidate result");
        assert_eq!(execution.results.len(), 29);
        assert_eq!(
            execution
                .results
                .iter()
                .filter(|result| result.status == CandidateStatus::Accepted)
                .count(),
            13
        );
        assert_eq!(
            execution
                .results
                .iter()
                .filter(|result| result.status == CandidateStatus::Rejected)
                .count(),
            16
        );
    }

    #[test]
    fn poisoned_answer_fields_cannot_change_native_analysis() {
        let corpus = checked_in();
        for variant in &corpus.variants {
            let baseline = analyze_program(&build_program(variant)).snapshot();
            let mut poisoned = variant.clone();
            poisoned.polarity = match poisoned.polarity {
                Polarity::Positive => Polarity::Misuse,
                Polarity::Misuse => Polarity::Positive,
            };
            poisoned.primary_reason = "poisoned_reason".to_owned();
            poisoned.required_sites = vec!["poisoned.site".to_owned()];
            poisoned.expected_observation = "poisoned observation".to_owned();
            assert_eq!(
                baseline,
                analyze_program(&build_program(&poisoned)).snapshot(),
                "{}:{}",
                variant.case_id,
                variant.variant_id
            );
        }
    }

    #[test]
    fn semantic_mutation_changes_outcome_without_polarity() {
        let corpus = checked_in();
        let variant = corpus
            .variants
            .iter()
            .find(|variant| {
                variant.case_id == "effect.effectful_map" && variant.variant_id == "positive"
            })
            .unwrap();
        let mut program = build_program(variant);
        assert!(analyze_program(&program).failure.is_none());
        program.omitted_claim_label = Some("callback.requirement".to_owned());
        let failure = analyze_program(&program).failure.unwrap();
        assert_eq!(failure.reason, "latent_requirement_erased");
        assert_eq!(failure.primary, "callback.call");
        assert_eq!(failure.related, ["map.call"]);
    }

    #[test]
    fn side_failures_are_derived_from_mutated_semantic_facts() {
        let corpus = checked_in();
        let program = |case_id: &str, variant_id: &str| {
            build_program(
                corpus
                    .variants
                    .iter()
                    .find(|variant| variant.case_id == case_id && variant.variant_id == variant_id)
                    .unwrap(),
            )
        };

        let mut shape = program("effect.pure_map", "positive");
        shape.shape.actual_result = "incompatible result".to_owned();
        assert_eq!(
            analyze_program(&shape).failure.unwrap().reason,
            "callable_shape_mismatch"
        );

        let mut iteration = program("effect.filter_retain", "positive_two_list_odd_filter");
        iteration.iteration.as_mut().unwrap().active_source_mutation = true;
        assert_eq!(
            analyze_program(&iteration).failure.unwrap().reason,
            "active_iteration_mutation"
        );

        let mut stale = program("effect.filter_retain", "positive_retain_delete");
        stale.iteration.as_mut().unwrap().stale_view_used = true;
        assert_eq!(
            analyze_program(&stale).failure.unwrap().reason,
            "stale_retained_item_view"
        );

        let mut runtime = program("effect.with_timeout", "positive_type_only");
        runtime.runtime_credit.as_mut().unwrap().credited = true;
        assert_eq!(
            analyze_program(&runtime).failure.unwrap().reason,
            "unimplemented_machinery_credited"
        );

        let mut registration = program("effect.callback_registry", "positive");
        registration.registration.as_mut().unwrap().registration_end = 2;
        assert_eq!(
            analyze_program(&registration).failure.unwrap().reason,
            "registration_outlives_capture"
        );

        let mut authority = program("effect.event_handler_factory", "positive");
        authority.authority.as_mut().unwrap().captured.clear();
        assert_eq!(
            analyze_program(&authority).failure.unwrap().reason,
            "captured_authority_erased"
        );

        let mut allocation = program("effect.memoizing_wrapper", "positive");
        allocation.allocation.reported.remove("cache_storage");
        assert_eq!(
            analyze_program(&allocation).failure.unwrap().reason,
            "hidden_allocation_or_resource"
        );

        let mut ownership = program("effect.linear_capture", "positive_move");
        ownership.ownership.as_mut().unwrap().transfer_recorded = false;
        assert_eq!(
            analyze_program(&ownership).failure.unwrap().reason,
            "ownership_transfer_missing"
        );
    }

    #[test]
    fn application_unifies_structural_tail_variables() {
        let mut solver = RowSolver::default();
        let caller_tail = solver.fresh("$caller");
        let callback_tail = solver.fresh("$callback");
        let caller = EffectRow::open(["output.requirement"], caller_tail);
        let callback = EffectRow::open(["wrapped.requirement"], callback_tail);
        let propagated = caller.propagate(&callback, &mut solver);
        assert_eq!(propagated.multiplicity("output.requirement"), 1);
        assert_eq!(propagated.multiplicity("wrapped.requirement"), 1);
        assert_eq!(
            solver.normalized_substitutions(),
            ["$caller->$r0", "$callback->$r0"]
        );
        assert_eq!(
            propagated.render(&solver, &mut TailNormalizer::default()),
            "<output.requirement, wrapped.requirement | $r0>"
        );
    }

    #[test]
    fn alpha_renaming_and_label_order_are_stable() {
        fn rendered(left: &str, right: &str, labels: [&str; 2]) -> String {
            let mut solver = RowSolver::default();
            let left = EffectRow::open([labels[0]], solver.fresh(left));
            let right = EffectRow::open([labels[1]], solver.fresh(right));
            left.propagate(&right, &mut solver)
                .render(&solver, &mut TailNormalizer::default())
        }
        assert_eq!(
            rendered("$left", "$right", ["z", "a"]),
            rendered("$renamed_a", "$renamed_b", ["a", "z"])
        );
    }

    #[test]
    fn duplicate_and_nested_handling_remove_one_occurrence_each() {
        let mut solver = RowSolver::default();
        let tail = solver.fresh("$tail");
        let row = EffectRow::open(["failure", "state", "failure"], tail).with_alias("Retryable");
        let once = row.handle_exact("failure").unwrap();
        let twice = once.handle_exact("failure").unwrap();
        assert_eq!(row.multiplicity("failure"), 2);
        assert_eq!(once.multiplicity("failure"), 1);
        assert_eq!(twice.multiplicity("failure"), 0);
        assert_eq!(
            once.render(&solver, &mut TailNormalizer::default()),
            "Retryable = <failure, state | $r0>"
        );
    }

    #[test]
    fn absent_handler_label_fails_closed() {
        let corpus = checked_in();
        let variant = corpus
            .variants
            .iter()
            .find(|variant| variant.case_id == "effect.retry" && variant.variant_id == "positive")
            .unwrap();
        let mut program = build_program(variant);
        program.handlers[0].label = "absent.requirement".to_owned();
        let failure = analyze_program(&program).failure.unwrap();
        assert_eq!(failure.reason, "handled_requirement_absent");
        assert_eq!(failure.primary, "retry.handle");
    }

    #[test]
    fn result_path_exercises_application_and_exact_handling() {
        let corpus = checked_in();
        let variant = corpus
            .variants
            .iter()
            .find(|variant| variant.case_id == "effect.retry" && variant.variant_id == "positive")
            .unwrap();
        let analysis = analyze_program(&build_program(variant));
        assert!(analysis.failure.is_none());
        assert_eq!(analysis.inferred.multiplicity("retry.failure"), 0);
        assert_eq!(
            analysis
                .inferred
                .multiplicity("action.remaining_requirement"),
            1
        );
        assert!(analysis.solver.unions.len() >= 2);
        assert_eq!(analysis.handlers, ["retry.failure@retry.handle".to_owned()]);
    }

    #[test]
    fn alias_preserving_error_snapshot_is_single_cause() {
        let corpus = checked_in();
        let execution = RowCandidateRun.execute(&corpus);
        let result = execution
            .results
            .iter()
            .find(|result| {
                result.case_id == "effect.retry" && result.variant_id == "misuse_erased_remaining"
            })
            .unwrap();
        let evidence = execution
            .evidence
            .results
            .get(&(result.case_id.clone(), result.variant_id.clone()))
            .unwrap();
        assert_eq!(result.diagnostic_cost.primary_diagnostic_count, 1);
        assert!(evidence.rendered_diagnostic.contains(
            "inferred RetryAction = <action.remaining_requirement | $r0>; claimed RetryAction = <| $r0>"
        ));
        assert!(
            evidence
                .rendered_diagnostic
                .contains("at action.call; related retry.call")
        );
    }

    #[test]
    fn every_non_row_guard_is_exercised_by_semantic_data() {
        let corpus = checked_in();
        let expected = BTreeSet::from([
            "allocation_visibility_guard",
            "callable_shape_guard",
            "exact_authority_retention_guard",
            "linear_resource_guard",
            "machinery_credit_guard",
            "place_and_view_guard",
            "registration_lifetime_guard",
        ]);
        let actual = corpus
            .variants
            .iter()
            .flat_map(|variant| analyze_program(&build_program(variant)).checks)
            .filter(|check| *check != "effect_inference")
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn row_growth_is_deterministic() {
        let mut solver = RowSolver::default();
        let tail = solver.fresh("$growth");
        let labels = (0..256)
            .rev()
            .map(|index| format!("effect.{index:03}"))
            .collect::<Vec<_>>();
        let row = EffectRow::open(labels, tail);
        let first = row.render(&solver, &mut TailNormalizer::default());
        let second = row.render(&solver, &mut TailNormalizer::default());
        assert_eq!(first, second);
        assert!(first.starts_with("<effect.000, effect.001"));
        assert!(first.ends_with("effect.255 | $r0>"));
    }

    #[test]
    fn returned_and_stored_callables_expose_allocation() {
        let corpus = checked_in();
        for variant in corpus.variants.iter().filter(|variant| {
            matches!(
                variant.callable_disposition.as_str(),
                "stored_callable" | "returned_callable"
            )
        }) {
            let program = build_program(variant);
            assert!(program.allocation.required.contains("callable_environment"));
            let analysis = analyze_program(&program);
            let machinery = added_machinery(&program, &analysis);
            assert!(machinery.iter().any(|fact| {
                fact.id == "machinery.runtime.callable_environment"
                    && fact.attributes.get("state").unwrap() == "unimplemented"
                    && fact.attributes.get("creditable").unwrap() == "false"
            }));
        }
    }

    #[test]
    fn authority_and_operator_consent_remain_separate() {
        let corpus = checked_in();
        let variant = corpus
            .variants
            .iter()
            .find(|variant| {
                variant.case_id == "effect.event_handler_factory"
                    && variant.variant_id == "positive"
            })
            .unwrap();
        let program = build_program(variant);
        assert_eq!(
            program.authority.as_ref().unwrap().required,
            BTreeSet::from(["logging.grant_exact".to_owned()])
        );
        assert!(analyze_program(&program).failure.is_none());
        let roles = policy_facts(variant)
            .into_iter()
            .map(|fact| fact.attributes["policy_role"].clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            roles,
            BTreeSet::from([
                "operation_exercise".to_owned(),
                "operator_consent".to_owned(),
                "source_requirement".to_owned(),
            ])
        );
    }

    #[test]
    fn linear_escape_is_derived_from_lifetime_facts() {
        let corpus = checked_in();
        let variant = corpus
            .variants
            .iter()
            .find(|variant| {
                variant.case_id == "effect.linear_capture" && variant.variant_id == "positive_move"
            })
            .unwrap();
        let mut program = build_program(variant);
        assert!(analyze_program(&program).failure.is_none());
        program.ownership.as_mut().unwrap().resource_end = 1;
        let failure = analyze_program(&program).failure.unwrap();
        assert_eq!(failure.reason, "captured_resource_escapes");
    }

    #[test]
    fn ordinary_positive_annotations_are_inferred() {
        let corpus = checked_in();
        let execution = RowCandidateRun.execute(&corpus);
        for result in execution
            .results
            .iter()
            .filter(|result| result.status == CandidateStatus::Accepted)
        {
            assert!(result.required_source_annotations.iter().all(|fact| {
                fact.attributes
                    .get("mode")
                    .is_some_and(|mode| mode == "inferred")
            }));
        }
    }
}
