use std::collections::{BTreeMap, BTreeSet};

use crate::corpus::{Corpus, CorpusVariant, capture_kind};
use crate::cost::{AnalysisTrace, measure_diagnostic};
use crate::inventory::candidate_inventory;
use crate::result_contract::{
    CandidateExecution, CandidateFactory, CandidateResult, CandidateRun, CandidateStatus,
    HarnessEvidence, ResultEvidence, StableFact,
};

pub const CANDIDATE_ID: &str = "boolean_formulas";
const MAX_CANONICAL_ATOMS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FormulaVariable(usize);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FormulaBuilder {
    names: Vec<String>,
}

impl FormulaBuilder {
    pub fn fresh(&mut self, name: impl Into<String>) -> FormulaVariable {
        let variable = FormulaVariable(self.names.len());
        self.names.push(name.into());
        variable
    }

    fn normalized_variables(&self) -> Vec<String> {
        self.names
            .iter()
            .enumerate()
            .map(|(index, name)| format!("{name}->$f{index}"))
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectFormula {
    False,
    True,
    Variable(FormulaVariable),
    Label(String),
    Not(Box<EffectFormula>),
    And(Vec<EffectFormula>),
    Or(Vec<EffectFormula>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalFormula {
    atoms: Vec<String>,
    truth: Vec<bool>,
}

impl CanonicalFormula {
    pub fn render(&self) -> String {
        let truth = self
            .truth
            .iter()
            .map(|value| if *value { '1' } else { '0' })
            .collect::<String>();
        format!("atoms=[{}];truth={truth}", self.atoms.join(","))
    }
}

impl EffectFormula {
    pub fn variable(variable: FormulaVariable) -> Self {
        Self::Variable(variable)
    }

    pub fn label(label: impl Into<String>) -> Self {
        Self::Label(label.into())
    }

    pub fn union(self, other: Self) -> Self {
        Self::Or(vec![self, other])
    }

    pub fn intersection(self, other: Self) -> Self {
        Self::And(vec![self, other])
    }

    pub fn complement(self) -> Self {
        Self::Not(Box::new(self))
    }

    pub fn difference(self, other: Self) -> Self {
        self.intersection(other.complement())
    }

    fn collect_atoms(
        &self,
        labels: &mut BTreeSet<String>,
        variables: &mut BTreeSet<FormulaVariable>,
    ) {
        match self {
            Self::False | Self::True => {}
            Self::Variable(variable) => {
                variables.insert(*variable);
            }
            Self::Label(label) => {
                labels.insert(label.clone());
            }
            Self::Not(inner) => inner.collect_atoms(labels, variables),
            Self::And(items) | Self::Or(items) => {
                for item in items {
                    item.collect_atoms(labels, variables);
                }
            }
        }
    }

    fn evaluate(
        &self,
        labels: &BTreeMap<String, bool>,
        variables: &BTreeMap<FormulaVariable, bool>,
    ) -> bool {
        match self {
            Self::False => false,
            Self::True => true,
            Self::Variable(variable) => variables[variable],
            Self::Label(label) => labels[label],
            Self::Not(inner) => !inner.evaluate(labels, variables),
            Self::And(items) => items.iter().all(|item| item.evaluate(labels, variables)),
            Self::Or(items) => items.iter().any(|item| item.evaluate(labels, variables)),
        }
    }

    pub fn canonical(&self) -> Result<CanonicalFormula, String> {
        let mut labels = BTreeSet::new();
        let mut variables = BTreeSet::new();
        self.collect_atoms(&mut labels, &mut variables);
        if labels.len() + variables.len() > MAX_CANONICAL_ATOMS {
            return Err(format!(
                "formula has {} atoms, maximum is {MAX_CANONICAL_ATOMS}",
                labels.len() + variables.len()
            ));
        }
        let labels = labels.into_iter().collect::<Vec<_>>();
        let variables = variables.into_iter().collect::<Vec<_>>();
        let atoms = labels
            .iter()
            .map(|label| format!("label:{label}"))
            .chain(
                variables
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("variable:$f{index}")),
            )
            .collect::<Vec<_>>();
        let mut truth = Vec::with_capacity(1usize << atoms.len());
        for assignment in 0..(1usize << atoms.len()) {
            let label_values = labels
                .iter()
                .enumerate()
                .map(|(index, label)| (label.clone(), assignment & (1 << index) != 0))
                .collect::<BTreeMap<_, _>>();
            let variable_values = variables
                .iter()
                .enumerate()
                .map(|(index, variable)| {
                    (*variable, assignment & (1 << (labels.len() + index)) != 0)
                })
                .collect::<BTreeMap<_, _>>();
            truth.push(self.evaluate(&label_values, &variable_values));
        }
        Ok(CanonicalFormula { atoms, truth })
    }

    pub fn equivalent(&self, other: &Self) -> Result<bool, String> {
        let mut labels = BTreeSet::new();
        let mut variables = BTreeSet::new();
        self.collect_atoms(&mut labels, &mut variables);
        other.collect_atoms(&mut labels, &mut variables);
        if labels.len() + variables.len() > MAX_CANONICAL_ATOMS {
            return Err(format!(
                "shared formula domain has {} atoms, maximum is {MAX_CANONICAL_ATOMS}",
                labels.len() + variables.len()
            ));
        }
        let labels = labels.into_iter().collect::<Vec<_>>();
        let variables = variables.into_iter().collect::<Vec<_>>();
        for assignment in 0..(1usize << (labels.len() + variables.len())) {
            let label_values = labels
                .iter()
                .enumerate()
                .map(|(index, label)| (label.clone(), assignment & (1 << index) != 0))
                .collect::<BTreeMap<_, _>>();
            let variable_values = variables
                .iter()
                .enumerate()
                .map(|(index, variable)| {
                    (*variable, assignment & (1 << (labels.len() + index)) != 0)
                })
                .collect::<BTreeMap<_, _>>();
            if self.evaluate(&label_values, &variable_values)
                != other.evaluate(&label_values, &variable_values)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FormulaSolver {
    constraints: Vec<(EffectFormula, EffectFormula)>,
    normalization_steps: usize,
}

impl FormulaSolver {
    pub fn constrain_equivalent(
        &mut self,
        left: EffectFormula,
        right: EffectFormula,
    ) -> Result<(), String> {
        self.normalization_steps += 2;
        if !left.equivalent(&right)? {
            return Err("effect formulas are not equivalent".to_owned());
        }
        self.constraints.push((left, right));
        Ok(())
    }

    fn constraint_count(&self) -> usize {
        self.constraints.len()
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
struct AssociatedEffectFacts {
    declared: BTreeSet<String>,
    inferred: BTreeSet<String>,
    callable: String,
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
    shape: ShapeFacts,
    iteration: Option<IterationFacts>,
    runtime_credit: Option<RuntimeCreditFacts>,
    registration: Option<RegistrationFacts>,
    authority: Option<AuthorityFacts>,
    allocation: AllocationFacts,
    ownership: Option<OwnershipFacts>,
    associated_effect: Option<AssociatedEffectFacts>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormulaFailure {
    reason: String,
    primary: String,
    related: Vec<String>,
    detail: String,
}

#[derive(Clone, Debug)]
struct CandidateAnalysis {
    inferred: EffectFormula,
    claimed: EffectFormula,
    builder: FormulaBuilder,
    solver: FormulaSolver,
    handlers: Vec<String>,
    checks: BTreeSet<&'static str>,
    failure: Option<FormulaFailure>,
    trace: AnalysisTrace,
}

impl CandidateAnalysis {
    fn snapshot(&self) -> String {
        let inferred = self
            .inferred
            .canonical()
            .map(|value| value.render())
            .unwrap_or_else(|error| format!("normalization_error:{error}"));
        let claimed = self
            .claimed
            .canonical()
            .map(|value| value.render())
            .unwrap_or_else(|error| format!("normalization_error:{error}"));
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
            "inferred={inferred};claimed={claimed};variables={};constraints={};handlers={};checks={};outcome={failure}",
            self.builder.normalized_variables().join(","),
            self.solver.constraint_count(),
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
        .filter(|domain| capture_kind(domain) == "authority")
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
    let associated_effect = matches!(
        variant.callable_disposition.as_str(),
        "stored_callable" | "returned_callable"
    )
    .then(|| {
        let requirements = requirements
            .iter()
            .map(|requirement| requirement.label.clone())
            .collect::<BTreeSet<_>>();
        AssociatedEffectFacts {
            declared: requirements.clone(),
            inferred: requirements,
            callable: format!("{}.callable", variant.case_id),
        }
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
        shape,
        iteration,
        runtime_credit,
        registration,
        authority,
        allocation,
        ownership,
        associated_effect,
    }
}

fn check_side_facts(
    program: &CandidateProgram,
    checks: &mut BTreeSet<&'static str>,
    trace: &mut AnalysisTrace,
) -> Option<FormulaFailure> {
    checks.insert("callable_shape_guard");
    trace.visit_node();
    trace.generate_constraint();
    if program.shape.expected_input != program.shape.actual_input
        || program.shape.expected_result != program.shape.actual_result
    {
        return Some(FormulaFailure {
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
            return Some(FormulaFailure {
                reason: "active_iteration_mutation".to_owned(),
                primary: "source.mutation".to_owned(),
                related: vec!["filter.call".to_owned()],
                detail: "source mutation overlaps active iteration".to_owned(),
            });
        }
        if iteration.deletion_happened && iteration.stale_view_used {
            return Some(FormulaFailure {
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
            return Some(FormulaFailure {
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
            return Some(FormulaFailure {
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
            return Some(FormulaFailure {
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
        return Some(FormulaFailure {
            reason: "hidden_allocation_or_resource".to_owned(),
            primary: "cache.allocation".to_owned(),
            related: vec!["memoize.call".to_owned()],
            detail: "required callable or cache allocation is not reported".to_owned(),
        });
    }
    if let Some(associated) = &program.associated_effect {
        checks.insert("associated_requirement_guard");
        trace.visit_node();
        trace.generate_fact();
        trace.generate_constraint();
        if associated.declared != associated.inferred {
            return Some(FormulaFailure {
                reason: "associated_requirement_mismatch".to_owned(),
                primary: associated.callable.clone(),
                related: program.call_sites.clone(),
                detail: "stored or returned callable requirement association differs".to_owned(),
            });
        }
    }
    if let Some(ownership) = &program.ownership {
        checks.insert("linear_resource_guard");
        trace.visit_node();
        trace.generate_constraint();
        if !ownership.transfer_recorded {
            return Some(FormulaFailure {
                reason: "ownership_transfer_missing".to_owned(),
                primary: "resource.capture".to_owned(),
                related: vec!["factory.call".to_owned()],
                detail: "captured resource has no recorded transfer".to_owned(),
            });
        }
        if ownership.uses > 1 {
            return Some(FormulaFailure {
                reason: "linear_resource_double_use".to_owned(),
                primary: "first.use".to_owned(),
                related: vec!["second.use".to_owned()],
                detail: "linear resource is consumed by two retained paths".to_owned(),
            });
        }
        if ownership.callable_end > ownership.resource_end {
            if ownership.stored {
                return Some(FormulaFailure {
                    reason: "captured_resource_outlives".to_owned(),
                    primary: "close.site".to_owned(),
                    related: vec!["late.use".to_owned(), "register.call".to_owned()],
                    detail: "stored callable remains live after resource close".to_owned(),
                });
            }
            return Some(FormulaFailure {
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
    let mut builder = FormulaBuilder::default();
    let ambient = EffectFormula::variable(builder.fresh("$ambient"));
    let mut inferred = ambient;
    let mut trace = AnalysisTrace::default();
    let mut checks = BTreeSet::from(["effect_formula_inference"]);
    trace.enter_live_item();
    trace.visit_node();
    trace.generate_fact();
    trace.normalization_step();
    for requirement in &program.requirements {
        inferred = inferred.union(EffectFormula::label(&requirement.label));
        trace.visit_node();
        trace.generate_fact();
        trace.generate_constraint();
        trace.normalization_step();
    }
    for _ in program.associated_effect.iter() {
        trace.visit_node();
        trace.generate_constraint();
        trace.normalization_step();
    }
    let mut handler_sites = Vec::new();
    let mut handler_failure = None;
    for handler in &program.handlers {
        handler_sites.push(format!("{}@{}", handler.label, handler.site));
        if !program
            .requirements
            .iter()
            .any(|requirement| requirement.label == handler.label)
        {
            handler_failure = Some(FormulaFailure {
                reason: "handled_requirement_absent".to_owned(),
                primary: handler.site.clone(),
                related: Vec::new(),
                detail: format!("handled requirement {} is absent", handler.label),
            });
            break;
        }
        inferred = inferred.difference(EffectFormula::label(&handler.label));
        trace.visit_node();
        trace.generate_constraint();
        trace.normalization_step();
    }
    let mut claimed = inferred.clone();
    if let Some(label) = &program.omitted_claim_label {
        claimed = claimed.difference(EffectFormula::label(label));
        trace.visit_node();
        trace.generate_constraint();
        trace.normalization_step();
    }
    let mut solver = FormulaSolver::default();
    let formula_failure = match handler_failure {
        Some(failure) => Some(failure),
        None => match solver.constrain_equivalent(inferred.clone(), claimed.clone()) {
            Ok(()) => None,
            Err(_) => {
                let missing = program
                    .omitted_claim_label
                    .as_ref()
                    .expect("only an omitted semantic requirement changes the claim");
                let requirement = program
                    .requirements
                    .iter()
                    .find(|requirement| &requirement.label == missing)
                    .expect("omitted claim label comes from a requirement");
                Some(FormulaFailure {
                    reason: if program.handlers.is_empty() {
                        "latent_requirement_erased"
                    } else {
                        "unhandled_requirement_erased"
                    }
                    .to_owned(),
                    primary: requirement.origin.clone(),
                    related: vec![requirement.boundary.clone()],
                    detail: format!("claimed requirement set excludes {missing}"),
                })
            }
        },
    };
    let side_failure = check_side_facts(program, &mut checks, &mut trace);
    trace.normalization_step();
    trace.leave_live_item();
    CandidateAnalysis {
        inferred,
        claimed,
        builder,
        solver,
        handlers: handler_sites,
        checks,
        failure: formula_failure.or(side_failure),
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
                    ("capture_kind", capture_kind(domain).to_owned()),
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
        .filter(|domain| capture_kind(domain) == "resource")
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
        "effect_formula_solver",
        "implemented",
        true,
        true,
    )];
    if program.associated_effect.is_some() {
        facts.push(machinery_fact(
            program,
            "checker",
            "associated_requirement_constraints",
            "implemented",
            true,
            true,
        ));
    }
    for check in &analysis.checks {
        if *check == "effect_formula_inference" {
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
    let inferred = analysis
        .inferred
        .canonical()
        .expect("corpus formula stays within the atom bound")
        .render();
    let claimed = analysis
        .claimed
        .canonical()
        .expect("corpus formula stays within the atom bound")
        .render();
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
            "effect requirement rejection {}: {} at {}; related {}; the inferred callable requirements preserve the affected operation but the claimed boundary does not; repair: {}",
            failure.reason,
            failure.detail,
            failure.primary,
            failure.related.join(","),
            repair
        )
    });
    let native_terms = if analysis.failure.is_some() {
        vec!["effect formula".to_owned()]
    } else {
        Vec::new()
    };
    let diagnostic_cost =
        measure_diagnostic(&rendered_diagnostic, &sites, &sites, &native_terms, &repair);
    let native_snapshot = analysis.snapshot();
    let native_evidence = BTreeMap::from([
        ("analysis".to_owned(), native_snapshot.clone()),
        (
            "algebra".to_owned(),
            "union,intersection,difference,complement,equivalence".to_owned(),
        ),
        ("claimed_formula".to_owned(), claimed.clone()),
        ("inferred_formula".to_owned(), inferred.clone()),
        (
            "normalization".to_owned(),
            "bounded_complete_truth_function".to_owned(),
        ),
    ]);
    let implementation_cost = candidate_inventory(CANDIDATE_ID)
        .expect("formula candidate has a harness-owned inventory")
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
pub struct FormulaCandidateFactory;

#[derive(Clone, Copy, Debug, Default)]
pub struct FormulaCandidateRun;

impl CandidateFactory for FormulaCandidateFactory {
    type Run = FormulaCandidateRun;

    fn fresh(&self) -> Self::Run {
        FormulaCandidateRun
    }
}

impl CandidateRun for FormulaCandidateRun {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrowthMeasurement {
    pub graph_nodes: usize,
    pub atoms: usize,
    pub truth_assignments: usize,
    pub generated_constraints: usize,
    pub normalization_steps: usize,
    pub canonical_bytes: usize,
}

pub fn measure_bounded_growth(graph_nodes: usize) -> GrowthMeasurement {
    assert!((1..=12).contains(&graph_nodes));
    let mut formula = EffectFormula::False;
    for index in 0..graph_nodes {
        let label = EffectFormula::label(format!("synthetic.effect.{index:02}"));
        formula = if index % 3 == 2 {
            formula.union(label.complement().complement())
        } else {
            formula.union(label)
        };
    }
    let canonical = formula
        .canonical()
        .expect("bounded synthetic graph stays under the atom cap");
    GrowthMeasurement {
        graph_nodes,
        atoms: canonical.atoms.len(),
        truth_assignments: canonical.truth.len(),
        generated_constraints: graph_nodes.saturating_sub(1),
        normalization_steps: graph_nodes * 2,
        canonical_bytes: canonical.render().len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{Polarity, checked_in};
    use crate::result_contract::{CandidateRun, run_fresh_twice, validate_run};

    fn program(corpus: &Corpus, case_id: &str, variant_id: &str) -> CandidateProgram {
        build_program(
            corpus
                .variants
                .iter()
                .find(|variant| variant.case_id == case_id && variant.variant_id == variant_id)
                .expect("test variant exists"),
        )
    }

    #[test]
    fn all_29_variants_satisfy_the_frozen_contract_repeatably() {
        let corpus = checked_in();
        let bytes = run_fresh_twice(&corpus, &FormulaCandidateFactory)
            .expect("candidate analysis must satisfy the frozen contract twice");
        assert!(!bytes.is_empty());
        let mut execution = FormulaCandidateRun.execute(&corpus);
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
    fn expected_answer_fields_cannot_change_native_analysis() {
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
    fn union_intersection_difference_complement_and_equivalence_are_executable() {
        let a = EffectFormula::label("a");
        let b = EffectFormula::label("b");
        let union_left = a.clone().union(b.clone());
        let union_right = b.clone().union(a.clone());
        assert!(union_left.equivalent(&union_right).unwrap());

        let de_morgan_left = a.clone().union(b.clone()).complement();
        let de_morgan_right = a.clone().complement().intersection(b.clone().complement());
        assert!(de_morgan_left.equivalent(&de_morgan_right).unwrap());

        let difference = union_left.difference(a.clone());
        let explicit = a.clone().union(b.clone()).intersection(a.complement());
        assert!(difference.equivalent(&explicit).unwrap());
        assert!(!difference.equivalent(&EffectFormula::True).unwrap());
    }

    #[test]
    fn equivalence_uses_one_shared_domain_and_removes_irrelevant_support() {
        let a = EffectFormula::label("a");
        let b = EffectFormula::label("b");

        let tautology = a.clone().union(a.clone().complement());
        assert!(tautology.equivalent(&EffectFormula::True).unwrap());

        let contradiction = a.clone().intersection(EffectFormula::False);
        assert!(contradiction.equivalent(&EffectFormula::False).unwrap());

        let absorption = a.clone().union(a.clone().intersection(b.clone()));
        assert!(absorption.equivalent(&a.clone()).unwrap());

        let non_equivalent = a.clone().union(b);
        assert!(!non_equivalent.equivalent(&a).unwrap());
    }

    #[test]
    fn equivalence_applies_the_atom_bound_to_the_shared_domain() {
        let left = (0..9).fold(EffectFormula::False, |formula, index| {
            formula.union(EffectFormula::label(format!("left.{index}")))
        });
        let right = (0..8).fold(EffectFormula::False, |formula, index| {
            formula.union(EffectFormula::label(format!("right.{index}")))
        });
        assert!(left.canonical().is_ok());
        assert!(right.canonical().is_ok());
        assert_eq!(
            left.equivalent(&right).unwrap_err(),
            "shared formula domain has 17 atoms, maximum is 16"
        );
    }

    #[test]
    fn formula_order_and_variable_names_normalize_stably() {
        fn normalized(first: &str, second: &str, reverse: bool) -> String {
            let mut builder = FormulaBuilder::default();
            let left = EffectFormula::variable(builder.fresh(first));
            let right = EffectFormula::variable(builder.fresh(second));
            let formula = if reverse {
                right.union(left)
            } else {
                left.union(right)
            };
            formula.canonical().unwrap().render()
        }
        assert_eq!(
            normalized("$left", "$right", false),
            normalized("$renamed_a", "$renamed_b", true)
        );
    }

    #[test]
    fn effect_exclusion_is_a_real_formula_constraint() {
        let blocked = EffectFormula::label("blocking.operation");
        let exclusion = EffectFormula::True.difference(blocked.clone());
        assert!(exclusion.equivalent(&blocked.clone().complement()).unwrap());
        assert!(!exclusion.equivalent(&EffectFormula::True).unwrap());

        let mut solver = FormulaSolver::default();
        solver
            .constrain_equivalent(exclusion.clone(), blocked.complement())
            .unwrap();
        assert_eq!(solver.constraint_count(), 1);
        assert!(
            solver
                .constrain_equivalent(exclusion, EffectFormula::True)
                .is_err()
        );
    }

    #[test]
    fn bounded_formula_growth_measurement_is_repeatable_and_explicit() {
        let first = measure_bounded_growth(12);
        let second = measure_bounded_growth(12);
        assert_eq!(first, second);
        assert_eq!(first.graph_nodes, 12);
        assert_eq!(first.atoms, 12);
        assert_eq!(first.truth_assignments, 4096);
        assert_eq!(first.generated_constraints, 11);
        assert!(first.normalization_steps > 0);
        assert!(first.canonical_bytes > first.truth_assignments);
    }

    #[test]
    fn every_semantic_guard_is_independently_mutable() {
        let corpus = checked_in();

        let mut shape = program(&corpus, "effect.pure_map", "positive");
        shape.shape.actual_result = "incompatible result".to_owned();
        assert_eq!(
            analyze_program(&shape).failure.unwrap().reason,
            "callable_shape_mismatch"
        );

        let mut erased = program(&corpus, "effect.effectful_map", "positive");
        erased.omitted_claim_label = Some("callback.requirement".to_owned());
        assert_eq!(
            analyze_program(&erased).failure.unwrap().reason,
            "latent_requirement_erased"
        );

        let mut mutation = program(
            &corpus,
            "effect.filter_retain",
            "positive_two_list_odd_filter",
        );
        mutation.iteration.as_mut().unwrap().active_source_mutation = true;
        assert_eq!(
            analyze_program(&mutation).failure.unwrap().reason,
            "active_iteration_mutation"
        );

        let mut stale = program(&corpus, "effect.filter_retain", "positive_retain_delete");
        stale.iteration.as_mut().unwrap().stale_view_used = true;
        assert_eq!(
            analyze_program(&stale).failure.unwrap().reason,
            "stale_retained_item_view"
        );

        let mut runtime = program(&corpus, "effect.with_timeout", "positive_type_only");
        runtime.runtime_credit.as_mut().unwrap().credited = true;
        assert_eq!(
            analyze_program(&runtime).failure.unwrap().reason,
            "unimplemented_machinery_credited"
        );

        let mut registration = program(&corpus, "effect.callback_registry", "positive");
        registration.registration.as_mut().unwrap().registration_end = 2;
        assert_eq!(
            analyze_program(&registration).failure.unwrap().reason,
            "registration_outlives_capture"
        );

        let mut authority = program(&corpus, "effect.event_handler_factory", "positive");
        authority.authority.as_mut().unwrap().captured.clear();
        assert_eq!(
            analyze_program(&authority).failure.unwrap().reason,
            "captured_authority_erased"
        );

        let mut allocation = program(&corpus, "effect.memoizing_wrapper", "positive");
        allocation.allocation.reported.remove("cache_storage");
        assert_eq!(
            analyze_program(&allocation).failure.unwrap().reason,
            "hidden_allocation_or_resource"
        );

        let mut transfer = program(&corpus, "effect.linear_capture", "positive_move");
        transfer.ownership.as_mut().unwrap().transfer_recorded = false;
        assert_eq!(
            analyze_program(&transfer).failure.unwrap().reason,
            "ownership_transfer_missing"
        );

        let mut double_use = program(&corpus, "effect.linear_capture", "positive_move");
        double_use.ownership.as_mut().unwrap().uses = 2;
        assert_eq!(
            analyze_program(&double_use).failure.unwrap().reason,
            "linear_resource_double_use"
        );

        let mut escape = program(&corpus, "effect.linear_capture", "positive_move");
        escape.ownership.as_mut().unwrap().resource_end = 1;
        assert_eq!(
            analyze_program(&escape).failure.unwrap().reason,
            "captured_resource_escapes"
        );

        let mut outlives = program(&corpus, "effect.linear_capture", "positive_move");
        outlives.ownership.as_mut().unwrap().resource_end = 1;
        outlives.ownership.as_mut().unwrap().stored = true;
        assert_eq!(
            analyze_program(&outlives).failure.unwrap().reason,
            "captured_resource_outlives"
        );
    }

    #[test]
    fn single_cause_diagnostic_uses_domain_language_not_solver_dump() {
        let corpus = checked_in();
        let execution = FormulaCandidateRun.execute(&corpus);
        let key = (
            "effect.retry".to_owned(),
            "misuse_erased_remaining".to_owned(),
        );
        let result = execution
            .results
            .iter()
            .find(|result| result.case_id == key.0 && result.variant_id == key.1)
            .unwrap();
        let evidence = &execution.evidence.results[&key];
        assert_eq!(result.diagnostic_cost.primary_diagnostic_count, 1);
        assert!(
            evidence
                .rendered_diagnostic
                .contains("the inferred callable requirements preserve the affected operation")
        );
        assert!(
            evidence
                .rendered_diagnostic
                .contains("at action.call; related retry.call")
        );
        assert!(!evidence.rendered_diagnostic.contains("truth="));
        assert!(!evidence.rendered_diagnostic.contains("And("));
    }

    #[test]
    fn stored_and_returned_callables_expose_allocation_and_associated_pressure() {
        let corpus = checked_in();
        for variant in corpus.variants.iter().filter(|variant| {
            matches!(
                variant.callable_disposition.as_str(),
                "stored_callable" | "returned_callable"
            )
        }) {
            let program = build_program(variant);
            assert!(program.allocation.required.contains("callable_environment"));
            assert!(program.associated_effect.is_some());
            let analysis = analyze_program(&program);
            let machinery = added_machinery(&program, &analysis);
            assert!(machinery.iter().any(|fact| {
                fact.id == "machinery.runtime.callable_environment"
                    && fact.attributes["state"] == "unimplemented"
                    && fact.attributes["creditable"] == "false"
            }));
            assert!(machinery.iter().any(|fact| {
                fact.id == "machinery.checker.associated_requirement_constraints"
                    && fact.attributes["state"] == "implemented"
                    && fact.attributes["exercised"] == "true"
            }));
        }
    }

    #[test]
    fn exact_authority_source_consent_and_exercise_remain_separate() {
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
    fn ordinary_positive_annotations_are_inferred_and_costs_are_nonzero() {
        let corpus = checked_in();
        let execution = FormulaCandidateRun.execute(&corpus);
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
            assert!(result.implementation_cost.nonblank_noncomment_lines > 0);
            assert!(result.analysis_cost.visited_corpus_nodes > 0);
        }
    }

    #[test]
    fn formula_atom_bound_fails_closed() {
        let formula = (0..=MAX_CANONICAL_ATOMS).fold(EffectFormula::False, |formula, index| {
            formula.union(EffectFormula::label(format!("effect.{index}")))
        });
        assert_eq!(
            formula.canonical().unwrap_err(),
            "formula has 17 atoms, maximum is 16"
        );
    }

    #[test]
    fn absent_handler_and_associated_requirement_mismatch_fail_closed() {
        let corpus = checked_in();
        let mut handler = program(&corpus, "effect.retry", "positive");
        handler.handlers[0].label = "absent.requirement".to_owned();
        let failure = analyze_program(&handler).failure.unwrap();
        assert_eq!(failure.reason, "handled_requirement_absent");
        assert_eq!(failure.primary, "retry.handle");

        let mut associated = program(&corpus, "effect.logging_middleware", "positive");
        associated
            .associated_effect
            .as_mut()
            .unwrap()
            .declared
            .clear();
        let failure = analyze_program(&associated).failure.unwrap();
        assert_eq!(failure.reason, "associated_requirement_mismatch");
        assert_eq!(failure.primary, "effect.logging_middleware.callable");
    }
}
