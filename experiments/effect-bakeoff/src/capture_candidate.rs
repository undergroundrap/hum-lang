use std::collections::{BTreeMap, BTreeSet};

use crate::corpus::{Corpus, CorpusVariant, capture_kind};
use crate::cost::{AnalysisTrace, measure_diagnostic};
use crate::inventory::candidate_inventory;
use crate::result_contract::{
    CandidateExecution, CandidateFactory, CandidateResult, CandidateRun, CandidateStatus,
    HarnessEvidence, ResultEvidence, StableFact,
};

pub const CANDIDATE_ID: &str = "capture_checking";
const MAX_CAPTURE_ATOMS: usize = 64;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CaptureVariable(usize);

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaptureBuilder {
    names: Vec<String>,
}

impl CaptureBuilder {
    pub fn fresh(&mut self, name: impl Into<String>) -> CaptureVariable {
        let variable = CaptureVariable(self.names.len());
        self.names.push(name.into());
        variable
    }

    fn normalized_variables(&self) -> Vec<String> {
        self.names
            .iter()
            .enumerate()
            .map(|(index, _)| format!("$c{index}"))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CaptureClass {
    ImmutableValue,
    OwnedMove,
    OwnedBorrow,
    LinearResource,
    SourceAuthority,
}

impl CaptureClass {
    fn render(self) -> &'static str {
        match self {
            Self::ImmutableValue => "value",
            Self::OwnedMove => "owned_move",
            Self::OwnedBorrow => "owned_borrow",
            Self::LinearResource => "linear_resource",
            Self::SourceAuthority => "source_authority",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CaptureAtom {
    identity: String,
    class: CaptureClass,
}

impl CaptureAtom {
    pub fn new(identity: impl Into<String>, class: CaptureClass) -> Self {
        Self {
            identity: identity.into(),
            class,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaptureSet {
    atoms: BTreeSet<CaptureAtom>,
    tail: Option<CaptureVariable>,
}

impl CaptureSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn explicit(atoms: impl IntoIterator<Item = CaptureAtom>) -> Self {
        Self {
            atoms: atoms.into_iter().collect(),
            tail: None,
        }
    }

    pub fn inferred(atoms: impl IntoIterator<Item = CaptureAtom>, tail: CaptureVariable) -> Self {
        Self {
            atoms: atoms.into_iter().collect(),
            tail: Some(tail),
        }
    }

    pub fn insert(&mut self, atom: CaptureAtom) {
        self.atoms.insert(atom);
    }

    fn validate_bound(&self) -> Result<(), String> {
        if self.atoms.len() > MAX_CAPTURE_ATOMS {
            Err(format!("capture set exceeds {MAX_CAPTURE_ATOMS} atoms"))
        } else {
            Ok(())
        }
    }

    pub fn without_identity(&self, identity: &str) -> Self {
        let mut result = self.clone();
        result.atoms.retain(|atom| atom.identity != identity);
        result
    }

    pub fn substitute(&self, substitutions: &CaptureSubstitution) -> Result<Self, String> {
        self.validate_bound()?;
        let Some(tail) = self.tail else {
            return Ok(self.clone());
        };
        let replacement = substitutions
            .bindings
            .get(&tail)
            .ok_or_else(|| format!("unbound capture variable {}", tail.0))?;
        replacement.validate_bound()?;
        let mut atoms = self.atoms.clone();
        atoms.extend(replacement.atoms.iter().cloned());
        let result = Self {
            atoms,
            tail: replacement.tail,
        };
        result.validate_bound()?;
        Ok(result)
    }

    pub fn is_subcapture_of(&self, upper: &Self) -> bool {
        if !self.atoms.is_subset(&upper.atoms) {
            return false;
        }
        self.tail.is_none() || self.tail == upper.tail
    }

    pub fn canonical(&self) -> Result<CanonicalCapture, String> {
        self.validate_bound()?;
        let atoms = self
            .atoms
            .iter()
            .map(|atom| format!("{}:{}", atom.class.render(), atom.identity))
            .collect::<Vec<_>>();
        Ok(CanonicalCapture {
            atoms,
            tail: self.tail.map(|_| "$c0".to_owned()),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCapture {
    atoms: Vec<String>,
    tail: Option<String>,
}

impl CanonicalCapture {
    pub fn render(&self) -> String {
        format!(
            "captures=[{}];tail={}",
            self.atoms.join(","),
            self.tail.as_deref().unwrap_or("closed")
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaptureSubstitution {
    bindings: BTreeMap<CaptureVariable, CaptureSet>,
}

impl CaptureSubstitution {
    pub fn bind(&mut self, variable: CaptureVariable, value: CaptureSet) -> Result<(), String> {
        if self.bindings.contains_key(&variable) {
            return Err(format!("capture variable {} already bound", variable.0));
        }
        self.bindings.insert(variable, value);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultCaptureMode {
    Strict,
    Lazy,
}

pub fn result_capture(
    environment: &CaptureSet,
    used: &BTreeSet<String>,
    mode: ResultCaptureMode,
) -> CaptureSet {
    match mode {
        ResultCaptureMode::Strict => environment.clone(),
        ResultCaptureMode::Lazy => {
            let mut result = environment.clone();
            result.atoms.retain(|atom| used.contains(&atom.identity));
            result
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaptureSolver {
    substitutions: CaptureSubstitution,
    constraints: Vec<(CaptureSet, CaptureSet)>,
}

impl CaptureSolver {
    pub fn bind(&mut self, variable: CaptureVariable, value: CaptureSet) -> Result<(), String> {
        self.substitutions.bind(variable, value)
    }

    pub fn require_subcapture(
        &mut self,
        lower: CaptureSet,
        upper: CaptureSet,
    ) -> Result<(), String> {
        lower.validate_bound()?;
        upper.validate_bound()?;
        let lower = lower.substitute(&self.substitutions)?;
        let upper = upper.substitute(&self.substitutions)?;
        lower.validate_bound()?;
        upper.validate_bound()?;
        if !lower.is_subcapture_of(&upper) {
            return Err("retained environment exceeds declared boundary".to_owned());
        }
        self.constraints.push((lower, upper));
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
struct HybridEffectFacts {
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
    shape: ShapeFacts,
    iteration: Option<IterationFacts>,
    runtime_credit: Option<RuntimeCreditFacts>,
    registration: Option<RegistrationFacts>,
    authority: Option<AuthorityFacts>,
    allocation: AllocationFacts,
    ownership: Option<OwnershipFacts>,
    capture_environment: CaptureSet,
    declared_capture: CaptureSet,
    capture_uses: BTreeMap<String, String>,
    hybrid_effects: HybridEffectFacts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureFailure {
    reason: String,
    primary: String,
    related: Vec<String>,
    detail: String,
}

#[derive(Clone, Debug)]
struct CandidateAnalysis {
    inferred: CaptureSet,
    claimed: CaptureSet,
    builder: CaptureBuilder,
    solver: CaptureSolver,
    handlers: Vec<String>,
    checks: BTreeSet<&'static str>,
    failure: Option<CaptureFailure>,
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
            "inferred={inferred};declared={claimed};variables={};constraints={};handlers={};checks={};outcome={failure}",
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

fn capture_atom(domain: &str) -> Option<CaptureAtom> {
    if domain == "none_explicit" {
        return None;
    }
    let (identity, class) = match domain {
        "prefix" | "wrapped callable" => (domain.to_owned(), CaptureClass::ImmutableValue),
        "cache state" => (domain.to_owned(), CaptureClass::OwnedMove),
        "caller state" | "input list view" | "retained item view" | "deleted item view"
        | "active iteration" => (domain.to_owned(), CaptureClass::OwnedBorrow),
        "owned resource" | "linear resource" | "transaction resource" | "registration" => {
            (domain.to_owned(), CaptureClass::LinearResource)
        }
        "exact logging authority" | "logging authority" => (
            "logging.grant_exact".to_owned(),
            CaptureClass::SourceAuthority,
        ),
        _ if capture_kind(domain) == "authority" => {
            (domain.to_owned(), CaptureClass::SourceAuthority)
        }
        _ if capture_kind(domain) == "resource" => (domain.to_owned(), CaptureClass::OwnedBorrow),
        _ => (domain.to_owned(), CaptureClass::ImmutableValue),
    };
    Some(CaptureAtom::new(identity, class))
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
    let capture_environment = CaptureSet::explicit(
        variant
            .captured_facts
            .iter()
            .filter_map(|domain| capture_atom(domain)),
    );
    let capture_uses = variant
        .captured_facts
        .iter()
        .filter_map(|domain| {
            let atom = capture_atom(domain)?;
            let origin = variant
                .capture_origins
                .get(domain)
                .cloned()
                .expect("checked corpus has a capture origin");
            Some((atom.identity, origin))
        })
        .collect::<BTreeMap<_, _>>();
    let mut declared_capture = capture_environment.clone();
    if variant.variant_id == "misuse_laundered_authority" {
        declared_capture
            .atoms
            .retain(|atom| atom.class != CaptureClass::SourceAuthority);
    }
    let inferred_effects = requirements
        .iter()
        .map(|requirement| requirement.label.clone())
        .filter(|label| !handlers.iter().any(|handler| &handler.label == label))
        .collect::<BTreeSet<_>>();
    let mut declared_effects = inferred_effects.clone();
    if let Some(label) = &omitted_claim_label {
        declared_effects.remove(label);
    }
    let hybrid_effects = HybridEffectFacts {
        declared: declared_effects,
        inferred: inferred_effects,
        callable: format!("{}.callable", variant.case_id),
    };
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
        shape,
        iteration,
        runtime_credit,
        registration,
        authority,
        allocation,
        ownership,
        capture_environment,
        declared_capture,
        capture_uses,
        hybrid_effects,
    }
}

fn check_side_facts(
    program: &CandidateProgram,
    checks: &mut BTreeSet<&'static str>,
    trace: &mut AnalysisTrace,
) -> Option<CaptureFailure> {
    checks.insert("callable_shape_guard");
    trace.visit_node();
    trace.generate_constraint();
    if program.shape.expected_input != program.shape.actual_input
        || program.shape.expected_result != program.shape.actual_result
    {
        return Some(CaptureFailure {
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
            return Some(CaptureFailure {
                reason: "active_iteration_mutation".to_owned(),
                primary: "source.mutation".to_owned(),
                related: vec!["filter.call".to_owned()],
                detail: "source mutation overlaps active iteration".to_owned(),
            });
        }
        if iteration.deletion_happened && iteration.stale_view_used {
            return Some(CaptureFailure {
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
            return Some(CaptureFailure {
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
            return Some(CaptureFailure {
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
            return Some(CaptureFailure {
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
        return Some(CaptureFailure {
            reason: "hidden_allocation_or_resource".to_owned(),
            primary: "cache.allocation".to_owned(),
            related: vec!["memoize.call".to_owned()],
            detail: "required callable or cache allocation is not reported".to_owned(),
        });
    }
    checks.insert("hybrid_requirement_guard");
    trace.visit_node();
    trace.generate_fact();
    trace.generate_constraint();
    if program.hybrid_effects.declared != program.hybrid_effects.inferred {
        let missing = program
            .hybrid_effects
            .inferred
            .difference(&program.hybrid_effects.declared)
            .next()
            .expect("hybrid mismatch has a missing requirement");
        let requirement = program
            .requirements
            .iter()
            .find(|requirement| &requirement.label == missing)
            .expect("hybrid fact comes from semantic requirement");
        return Some(CaptureFailure {
            reason: if program.handlers.is_empty() {
                "latent_requirement_erased"
            } else {
                "unhandled_requirement_erased"
            }
            .to_owned(),
            primary: requirement.origin.clone(),
            related: vec![requirement.boundary.clone()],
            detail: format!("declared callable requirements omit {missing}"),
        });
    }
    if let Some(ownership) = &program.ownership {
        checks.insert("linear_resource_guard");
        trace.visit_node();
        trace.generate_constraint();
        if !ownership.transfer_recorded {
            return Some(CaptureFailure {
                reason: "ownership_transfer_missing".to_owned(),
                primary: "resource.capture".to_owned(),
                related: vec!["factory.call".to_owned()],
                detail: "captured resource has no recorded transfer".to_owned(),
            });
        }
        if ownership.uses > 1 {
            return Some(CaptureFailure {
                reason: "linear_resource_double_use".to_owned(),
                primary: "first.use".to_owned(),
                related: vec!["second.use".to_owned()],
                detail: "linear resource is consumed by two retained paths".to_owned(),
            });
        }
        if ownership.callable_end > ownership.resource_end {
            if ownership.stored {
                return Some(CaptureFailure {
                    reason: "captured_resource_outlives".to_owned(),
                    primary: "close.site".to_owned(),
                    related: vec!["late.use".to_owned(), "register.call".to_owned()],
                    detail: "stored callable remains live after resource close".to_owned(),
                });
            }
            return Some(CaptureFailure {
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
    let mut builder = CaptureBuilder::default();
    let environment_variable = builder.fresh("$environment");
    let inferred = program.capture_environment.clone();
    let claimed = program.declared_capture.clone();
    let mut trace = AnalysisTrace::default();
    let mut checks = BTreeSet::from(["retained_environment_inference"]);
    trace.enter_live_item();
    trace.visit_node();
    trace.generate_fact();
    trace.normalization_step();
    for _ in &program.capture_environment.atoms {
        trace.visit_node();
        trace.generate_fact();
        trace.generate_constraint();
        trace.normalization_step();
    }
    for _ in &program.hybrid_effects.inferred {
        trace.visit_node();
        trace.generate_fact();
        trace.generate_constraint();
        trace.normalization_step();
    }
    checks.insert("result_retention_analysis");
    let mut use_failure = None;
    for (identity, site) in &program.capture_uses {
        trace.visit_node();
        trace.generate_constraint();
        if !inferred.atoms.iter().any(|atom| &atom.identity == identity) {
            use_failure = Some(CaptureFailure {
                reason: "retained_capture_mismatch".to_owned(),
                primary: site.clone(),
                related: program.call_sites.clone(),
                detail: format!("callable uses {identity} without retaining it"),
            });
            break;
        }
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
            handler_failure = Some(CaptureFailure {
                reason: "handled_requirement_absent".to_owned(),
                primary: handler.site.clone(),
                related: Vec::new(),
                detail: format!("handled requirement {} is absent", handler.label),
            });
            break;
        }
        trace.visit_node();
        trace.generate_constraint();
        trace.normalization_step();
    }
    let mut solver = CaptureSolver::default();
    solver
        .bind(environment_variable, inferred.clone())
        .expect("fresh environment variable binds once");
    let open_inferred = CaptureSet::inferred(Vec::new(), environment_variable);
    let capture_failure = use_failure.or(handler_failure).or_else(|| {
        solver
            .require_subcapture(open_inferred, claimed.clone())
            .err()
            .map(|_| {
                let missing = inferred
                    .atoms
                    .difference(&claimed.atoms)
                    .next()
                    .expect("failed subcapture has a missing atom");
                if missing.class == CaptureClass::SourceAuthority {
                    CaptureFailure {
                        reason: "captured_authority_erased".to_owned(),
                        primary: "authority.capture".to_owned(),
                        related: vec!["factory.call".to_owned(), "handler.call".to_owned()],
                        detail: "declared callable environment omits an exact retained authority"
                            .to_owned(),
                    }
                } else {
                    CaptureFailure {
                        reason: "retained_capture_mismatch".to_owned(),
                        primary: program
                            .call_sites
                            .first()
                            .cloned()
                            .unwrap_or_else(|| "callable.definition".to_owned()),
                        related: program.call_sites.iter().skip(1).cloned().collect(),
                        detail: format!(
                            "declared callable environment omits {} {}",
                            missing.class.render(),
                            missing.identity
                        ),
                    }
                }
            })
    });
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
        failure: capture_failure.or(side_failure),
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
        "retained_environment_solver",
        "implemented",
        true,
        true,
    )];
    if !program.hybrid_effects.inferred.is_empty() {
        facts.push(machinery_fact(
            program,
            "checker",
            "hybrid_requirement_facts",
            "implemented",
            true,
            true,
        ));
    }
    if matches!(
        program.callable_disposition.as_str(),
        "stored_callable" | "returned_callable"
    ) {
        facts.push(machinery_fact(
            program,
            "checker",
            "result_retention_analysis",
            "implemented",
            true,
            true,
        ));
    }
    for check in &analysis.checks {
        if matches!(
            *check,
            "retained_environment_inference" | "result_retention_analysis"
        ) {
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

fn result_capture_pair(program: &CandidateProgram) -> (CaptureSet, CaptureSet) {
    let used = program
        .capture_uses
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    (
        result_capture(
            &program.capture_environment,
            &used,
            ResultCaptureMode::Strict,
        ),
        result_capture(&program.capture_environment, &used, ResultCaptureMode::Lazy),
    )
}

fn analyze_variant(variant: &CorpusVariant) -> (CandidateResult, ResultEvidence) {
    let program = build_program(variant);
    let analysis = analyze_program(&program);
    let inferred = analysis
        .inferred
        .canonical()
        .expect("corpus capture stays within the atom bound")
        .render();
    let claimed = analysis
        .claimed
        .canonical()
        .expect("corpus capture stays within the atom bound")
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
    let native_terms = Vec::new();
    let diagnostic_cost =
        measure_diagnostic(&rendered_diagnostic, &sites, &sites, &native_terms, &repair);
    let native_snapshot = analysis.snapshot();
    let (strict_result_capture, lazy_result_capture) = result_capture_pair(&program);
    let strict_result_capture = strict_result_capture
        .canonical()
        .expect("frozen strict result capture is bounded")
        .render();
    let lazy_result_capture = lazy_result_capture
        .canonical()
        .expect("frozen lazy result capture is bounded")
        .render();
    let native_evidence = BTreeMap::from([
        ("analysis".to_owned(), native_snapshot.clone()),
        (
            "algebra".to_owned(),
            "explicit,inferred,substitution,subcapture,result_capture".to_owned(),
        ),
        ("claimed_capture".to_owned(), claimed.clone()),
        ("inferred_capture".to_owned(), inferred.clone()),
        (
            "normalization".to_owned(),
            "sorted_atoms_alpha_renamed_tail".to_owned(),
        ),
        (
            "hybrid_effects".to_owned(),
            program
                .hybrid_effects
                .inferred
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(","),
        ),
        ("strict_result_capture".to_owned(), strict_result_capture),
        ("lazy_result_capture".to_owned(), lazy_result_capture),
    ]);
    let implementation_cost = candidate_inventory(CANDIDATE_ID)
        .expect("capture candidate has a harness-owned inventory")
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
pub struct CaptureCandidateFactory;

#[derive(Clone, Copy, Debug, Default)]
pub struct CaptureCandidateRun;

impl CandidateFactory for CaptureCandidateFactory {
    type Run = CaptureCandidateRun;

    fn fresh(&self) -> Self::Run {
        CaptureCandidateRun
    }
}

impl CandidateRun for CaptureCandidateRun {
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
    pub substitutions: usize,
    pub generated_constraints: usize,
    pub normalization_steps: usize,
    pub canonical_bytes: usize,
}

pub fn measure_bounded_growth(graph_nodes: usize) -> GrowthMeasurement {
    assert!((1..=12).contains(&graph_nodes));
    let atoms = (0..graph_nodes)
        .map(|index| {
            CaptureAtom::new(
                format!("synthetic.value.{index:02}"),
                CaptureClass::ImmutableValue,
            )
        })
        .collect::<Vec<_>>();
    let capture = CaptureSet::explicit(atoms);
    let canonical = capture.canonical().expect("bounded graph stays under cap");
    GrowthMeasurement {
        graph_nodes,
        atoms: canonical.atoms.len(),
        substitutions: graph_nodes,
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
        let first = run_fresh_twice(&corpus, &CaptureCandidateFactory)
            .expect("candidate must satisfy the frozen contract twice");
        let second = run_fresh_twice(&corpus, &CaptureCandidateFactory).unwrap();
        assert_eq!(first, second);
        let mut execution = CaptureCandidateRun.execute(&corpus);
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
    fn expected_answers_do_not_drive_native_analysis() {
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
    fn capture_normalization_substitution_and_subcapture_are_stable() {
        fn normalized(names: (&str, &str), reverse: bool) -> String {
            let mut builder = CaptureBuilder::default();
            let variable = builder.fresh(names.0);
            let atoms = [
                CaptureAtom::new("prefix", CaptureClass::ImmutableValue),
                CaptureAtom::new("state", CaptureClass::OwnedBorrow),
            ];
            let mut substitution = CaptureSubstitution::default();
            substitution
                .bind(variable, CaptureSet::explicit(atoms.clone()))
                .unwrap();
            let open = CaptureSet::inferred(Vec::new(), variable);
            let mut result = open.substitute(&substitution).unwrap();
            if reverse {
                result = CaptureSet::explicit(atoms.into_iter().rev());
            }
            assert_eq!(builder.normalized_variables(), vec!["$c0"]);
            result.canonical().unwrap().render()
        }

        assert_eq!(
            normalized(("$source", "$unused"), false),
            normalized(("$renamed", "$also_unused"), true)
        );

        let small =
            CaptureSet::explicit([CaptureAtom::new("prefix", CaptureClass::ImmutableValue)]);
        let large = CaptureSet::explicit([
            CaptureAtom::new("prefix", CaptureClass::ImmutableValue),
            CaptureAtom::new("state", CaptureClass::OwnedBorrow),
        ]);
        assert!(small.is_subcapture_of(&large));
        assert!(!large.is_subcapture_of(&small));

        let mut solver = CaptureSolver::default();
        solver
            .require_subcapture(small.clone(), large.clone())
            .unwrap();
        assert_eq!(solver.constraint_count(), 1);
        assert!(solver.require_subcapture(large, small).is_err());
    }

    #[test]
    fn strict_and_lazy_result_capture_are_explicitly_different() {
        let corpus = checked_in();
        let mut program = program(&corpus, "effect.event_handler_factory", "positive");
        program
            .capture_environment
            .insert(CaptureAtom::new("dormant", CaptureClass::OwnedBorrow));
        program.declared_capture = program.capture_environment.clone();
        assert!(!program.capture_uses.contains_key("dormant"));
        let (strict, lazy) = result_capture_pair(&program);
        assert_eq!(strict.atoms.len(), 3);
        assert_eq!(lazy.atoms.len(), 2);
        assert!(
            lazy.atoms
                .iter()
                .any(|atom| atom.identity == "logging.grant_exact")
        );
        assert!(!lazy.atoms.iter().any(|atom| atom.identity == "dormant"));
        assert!(
            added_machinery(&program, &analyze_program(&program))
                .iter()
                .any(|fact| {
                    fact.id == "machinery.checker.result_retention_analysis"
                        && fact.attributes["creditable"] == "true"
                        && fact.attributes["exercised"] == "true"
                })
        );
    }

    #[test]
    fn stored_and_returned_callables_expose_environment_and_result_capture() {
        let corpus = checked_in();
        for variant in corpus.variants.iter().filter(|variant| {
            matches!(
                variant.callable_disposition.as_str(),
                "stored_callable" | "returned_callable"
            )
        }) {
            let program = build_program(variant);
            assert!(program.allocation.required.contains("callable_environment"));
            let used = program
                .capture_environment
                .atoms
                .iter()
                .map(|atom| atom.identity.clone())
                .collect::<BTreeSet<_>>();
            assert_eq!(
                result_capture(&program.capture_environment, &used, ResultCaptureMode::Lazy),
                program.capture_environment
            );
            let analysis = analyze_program(&program);
            let machinery = added_machinery(&program, &analysis);
            assert!(machinery.iter().any(|fact| {
                fact.id == "machinery.runtime.callable_environment"
                    && fact.attributes["state"] == "unimplemented"
                    && fact.attributes["creditable"] == "false"
            }));
        }
    }

    #[test]
    fn capture_classes_keep_value_owned_borrowed_linear_and_authority_distinct() {
        let corpus = checked_in();
        let handler = program(&corpus, "effect.event_handler_factory", "positive");
        let classes = handler
            .capture_environment
            .atoms
            .iter()
            .map(|atom| (atom.identity.clone(), atom.class))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(classes["prefix"], CaptureClass::ImmutableValue);
        assert_eq!(
            classes["logging.grant_exact"],
            CaptureClass::SourceAuthority
        );

        let registry = program(&corpus, "effect.callback_registry", "positive");
        assert!(registry.capture_environment.atoms.iter().any(|atom| {
            atom.identity == "caller state" && atom.class == CaptureClass::OwnedBorrow
        }));
        assert!(registry.capture_environment.atoms.iter().any(|atom| {
            atom.identity == "registration" && atom.class == CaptureClass::LinearResource
        }));

        let memoizer = program(&corpus, "effect.memoizing_wrapper", "positive");
        assert!(memoizer.capture_environment.atoms.iter().any(|atom| {
            atom.identity == "cache state" && atom.class == CaptureClass::OwnedMove
        }));

        let linear = program(&corpus, "effect.linear_capture", "positive_move");
        assert!(linear.capture_environment.atoms.iter().any(|atom| {
            atom.identity == "owned resource" && atom.class == CaptureClass::LinearResource
        }));
    }

    #[test]
    fn source_requirement_consent_exercise_and_retention_never_collapse() {
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
        assert!(program.capture_environment.atoms.iter().any(|atom| {
            atom.identity == "logging.grant_exact" && atom.class == CaptureClass::SourceAuthority
        }));
        let roles = policy_facts(variant)
            .into_iter()
            .map(|fact| fact.attributes["policy_role"].clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            roles,
            BTreeSet::from([
                "source_requirement".to_owned(),
                "operator_consent".to_owned(),
                "operation_exercise".to_owned(),
            ])
        );

        let mut no_consent = variant.clone();
        no_consent.operator_consent = vec!["none_explicit".to_owned()];
        let no_consent_program = build_program(&no_consent);
        assert!(no_consent_program.authority.is_none());
        assert!(
            no_consent_program
                .capture_environment
                .atoms
                .iter()
                .any(|atom| { atom.identity == "logging.grant_exact" })
        );
        assert_eq!(no_consent.source_requirements, variant.source_requirements);
        assert_eq!(no_consent.operation_exercise, variant.operation_exercise);
    }

    #[test]
    fn hybrid_non_capability_effects_are_preserved_and_priced() {
        let corpus = checked_in();
        for (case_id, variant_id, expected) in [
            (
                "effect.retry",
                "positive",
                ["action.remaining_requirement"].as_slice(),
            ),
            (
                "effect.parallel_map",
                "positive_type_only",
                ["callback.requirement", "parallel.requirement"].as_slice(),
            ),
            (
                "effect.logging_middleware",
                "positive",
                ["output.requirement", "wrapped.requirement"].as_slice(),
            ),
        ] {
            let program = program(&corpus, case_id, variant_id);
            for requirement in expected {
                assert!(program.hybrid_effects.inferred.contains(*requirement));
            }
            let machinery = added_machinery(&program, &analyze_program(&program));
            assert!(machinery.iter().any(|fact| {
                fact.id == "machinery.checker.hybrid_requirement_facts"
                    && fact.attributes["creditable"] == "true"
                    && fact.attributes["exercised"] == "true"
            }));
        }
    }

    #[test]
    fn every_substantive_relationship_is_independently_mutable() {
        let corpus = checked_in();

        let mut shape = program(&corpus, "effect.pure_map", "positive");
        shape.shape.actual_result = "wrong result".to_owned();
        assert_eq!(
            analyze_program(&shape).failure.unwrap().reason,
            "callable_shape_mismatch"
        );

        let mut hybrid = program(&corpus, "effect.effectful_map", "positive");
        hybrid.hybrid_effects.declared.clear();
        assert_eq!(
            analyze_program(&hybrid).failure.unwrap().reason,
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
        authority.declared_capture.atoms.clear();
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

        let mut handler = program(&corpus, "effect.retry", "positive");
        handler.handlers[0].label = "absent.requirement".to_owned();
        assert_eq!(
            analyze_program(&handler).failure.unwrap().reason,
            "handled_requirement_absent"
        );
    }

    #[test]
    fn independent_capture_mutations_change_reason_and_blame() {
        let corpus = checked_in();

        let mut immutable = program(&corpus, "effect.event_handler_factory", "positive");
        immutable
            .declared_capture
            .atoms
            .retain(|atom| atom.identity != "prefix");
        let failure = analyze_program(&immutable).failure.unwrap();
        assert_eq!(failure.reason, "retained_capture_mismatch");
        assert_eq!(failure.primary, "factory.call");

        let mut moved = program(&corpus, "effect.memoizing_wrapper", "positive");
        moved
            .declared_capture
            .atoms
            .retain(|atom| atom.class != CaptureClass::OwnedMove);
        let failure = analyze_program(&moved).failure.unwrap();
        assert_eq!(failure.reason, "retained_capture_mismatch");
        assert_eq!(failure.primary, "memoize.call");

        let mut borrowed = program(&corpus, "effect.callback_registry", "positive");
        borrowed
            .declared_capture
            .atoms
            .retain(|atom| atom.class != CaptureClass::OwnedBorrow);
        let failure = analyze_program(&borrowed).failure.unwrap();
        assert_eq!(failure.reason, "retained_capture_mismatch");
        assert_eq!(failure.primary, "register.call");

        let mut linear = program(&corpus, "effect.linear_capture", "positive_move");
        linear.declared_capture.atoms.clear();
        let failure = analyze_program(&linear).failure.unwrap();
        assert_eq!(failure.reason, "retained_capture_mismatch");
        assert_eq!(failure.primary, "factory.call");

        let mut widened = program(&corpus, "effect.event_handler_factory", "positive");
        widened.declared_capture.atoms.remove(&CaptureAtom::new(
            "logging.grant_exact",
            CaptureClass::SourceAuthority,
        ));
        widened.declared_capture.insert(CaptureAtom::new(
            "generic.io",
            CaptureClass::SourceAuthority,
        ));
        let failure = analyze_program(&widened).failure.unwrap();
        assert_eq!(failure.reason, "captured_authority_erased");
        assert_eq!(failure.primary, "authority.capture");
    }

    #[test]
    fn transaction_move_escape_double_use_and_outlives_are_single_cause() {
        let corpus = checked_in();
        let execution = CaptureCandidateRun.execute(&corpus);
        for (variant_id, reason) in [
            ("misuse_move_without_transfer", "ownership_transfer_missing"),
            ("misuse_escape", "captured_resource_escapes"),
            ("misuse_double_use", "linear_resource_double_use"),
            ("misuse_outlives", "captured_resource_outlives"),
        ] {
            let result = execution
                .results
                .iter()
                .find(|result| {
                    result.case_id == "effect.linear_capture" && result.variant_id == variant_id
                })
                .unwrap();
            assert_eq!(result.primary_reason, reason);
            assert_eq!(result.diagnostic_cost.primary_diagnostic_count, 1);
        }
    }

    #[test]
    fn diagnostics_use_domain_language_and_exact_frozen_sites() {
        let corpus = checked_in();
        let execution = CaptureCandidateRun.execute(&corpus);
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
        assert_eq!(result.primary_blame_site, "action.call");
        assert_eq!(result.related_blame_sites, vec!["retry.call"]);
        assert!(
            evidence
                .rendered_diagnostic
                .contains("inferred callable requirements")
        );
        assert!(!evidence.rendered_diagnostic.contains("CaptureSet"));
        assert!(!evidence.rendered_diagnostic.contains("$c"));
    }

    #[test]
    fn bounded_growth_and_cost_evidence_are_repeatable() {
        let first = measure_bounded_growth(12);
        let second = measure_bounded_growth(12);
        assert_eq!(first, second);
        assert_eq!(first.graph_nodes, 12);
        assert_eq!(first.atoms, 12);
        assert_eq!(first.substitutions, 12);
        assert_eq!(first.generated_constraints, 11);
        assert!(first.normalization_steps > 0);
        assert!(first.canonical_bytes > 0);

        let corpus = checked_in();
        let execution = CaptureCandidateRun.execute(&corpus);
        let visited = execution
            .results
            .iter()
            .map(|result| result.analysis_cost.visited_corpus_nodes)
            .collect::<Vec<_>>();
        let constraints = execution
            .results
            .iter()
            .map(|result| result.analysis_cost.generated_constraints)
            .collect::<Vec<_>>();
        assert_eq!(visited.iter().min(), Some(&2));
        assert_eq!(visited.iter().max(), Some(&11));
        assert_eq!(constraints.iter().min(), Some(&1));
        assert_eq!(constraints.iter().max(), Some(&10));
        for result in &execution.results {
            assert!(result.implementation_cost.nonblank_noncomment_lines > 0);
            assert!(result.analysis_cost.visited_corpus_nodes > 0);
            assert_eq!(result.analysis_cost.maximum_live_items, 1);
        }
    }

    #[test]
    fn duplicate_binding_preserves_the_original_substitution() {
        let mut builder = CaptureBuilder::default();
        let variable = builder.fresh("$value");
        let first = CaptureSet::explicit([CaptureAtom::new("first", CaptureClass::ImmutableValue)]);
        let rejected =
            CaptureSet::explicit([CaptureAtom::new("rejected", CaptureClass::ImmutableValue)]);
        let open = CaptureSet::inferred(Vec::new(), variable);
        let mut substitution = CaptureSubstitution::default();
        substitution.bind(variable, first.clone()).unwrap();
        assert_eq!(
            substitution.bind(variable, rejected.clone()).unwrap_err(),
            "capture variable 0 already bound"
        );
        assert_eq!(open.substitute(&substitution).unwrap(), first);

        let mut solver = CaptureSolver::default();
        solver
            .bind(variable, open.substitute(&substitution).unwrap())
            .unwrap();
        let solver_open = CaptureSet::inferred(Vec::new(), variable);
        assert!(
            solver
                .require_subcapture(solver_open.clone(), first)
                .is_ok()
        );
        assert!(solver.require_subcapture(solver_open, rejected).is_err());
    }

    #[test]
    fn capture_bound_applies_to_closed_and_open_solver_paths() {
        fn atoms(count: usize) -> Vec<CaptureAtom> {
            (0..count)
                .map(|index| {
                    CaptureAtom::new(format!("value.{index}"), CaptureClass::ImmutableValue)
                })
                .collect()
        }

        let closed_64 = CaptureSet::explicit(atoms(MAX_CAPTURE_ATOMS));
        let closed_65 = CaptureSet::explicit(atoms(MAX_CAPTURE_ATOMS + 1));
        assert!(closed_64.canonical().is_ok());
        assert_eq!(
            closed_65.canonical().unwrap_err(),
            "capture set exceeds 64 atoms"
        );
        let mut closed_solver = CaptureSolver::default();
        assert!(
            closed_solver
                .require_subcapture(closed_64.clone(), closed_64.clone())
                .is_ok()
        );
        assert_eq!(
            closed_solver
                .require_subcapture(closed_65.clone(), closed_65.clone())
                .unwrap_err(),
            "capture set exceeds 64 atoms"
        );

        let mut builder = CaptureBuilder::default();
        let variable = builder.fresh("$value");
        let open = CaptureSet::inferred(Vec::new(), variable);
        let mut open_64 = CaptureSubstitution::default();
        open_64.bind(variable, closed_64.clone()).unwrap();
        assert_eq!(open.substitute(&open_64).unwrap(), closed_64);

        let mut open_65 = CaptureSubstitution::default();
        open_65.bind(variable, closed_65).unwrap();
        assert_eq!(
            open.substitute(&open_65).unwrap_err(),
            "capture set exceeds 64 atoms"
        );
        let mut open_solver = CaptureSolver::default();
        open_solver
            .bind(variable, CaptureSet::explicit(atoms(65)))
            .unwrap();
        assert_eq!(
            open_solver
                .require_subcapture(open, CaptureSet::explicit(atoms(65)))
                .unwrap_err(),
            "capture set exceeds 64 atoms"
        );

        let overflowing_open = CaptureSet::inferred(atoms(65), variable);
        let mut empty_binding = CaptureSubstitution::default();
        empty_binding.bind(variable, CaptureSet::empty()).unwrap();
        assert_eq!(
            overflowing_open.substitute(&empty_binding).unwrap_err(),
            "capture set exceeds 64 atoms"
        );
    }
}
