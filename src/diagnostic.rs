use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticCode {
    key: crate::diagnostic_catalog::DiagnosticCodeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelatedSpan {
    pub label: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Span>,
    pub related_spans: Vec<RelatedSpan>,
    pub help: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DiagnosticOccurrenceId(String);

impl DiagnosticOccurrenceId {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticOccurrence {
    id: DiagnosticOccurrenceId,
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    pub(crate) code: DiagnosticCode,
    semantic_owner: &'static str,
    owning_stage: &'static str,
    identity: DiagnosticOccurrenceIdentity,
    resolver_call_occurrence: Option<crate::resolve::ResolverCallOccurrenceId>,
    diagnostic: Diagnostic,
    diagnostic_seal: Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticOccurrenceIdentity {
    origin_kind: &'static str,
    route_kind: &'static str,
    semantic_origin: String,
    relationship_route: Vec<String>,
}

impl DiagnosticOccurrenceIdentity {
    pub(crate) fn extend_relationship_route(&mut self, parts: Vec<String>) {
        self.relationship_route.extend(parts);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PriorBlockerRef {
    pub(crate) occurrence_id: DiagnosticOccurrenceId,
    pub(crate) cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    pub(crate) code: DiagnosticCode,
    pub(crate) semantic_origin: String,
    pub(crate) relationship_route: Vec<String>,
    pub(crate) resolver_call_occurrence: Option<crate::resolve::ResolverCallOccurrenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticPrecedenceApplication {
    pub(crate) rule_id: &'static str,
    pub(crate) relationship: &'static str,
    pub(crate) applying_owner: &'static str,
    pub(crate) applying_stage: &'static str,
    pub(crate) dominant: PriorBlockerRef,
    pub(crate) suppressed: PriorBlockerRef,
    pub(crate) semantic_nodes: [String; 2],
    pub(crate) relationship_route: Vec<String>,
    pub(crate) canonical_sites: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticPrecedenceRelationship {
    application: DiagnosticPrecedenceApplication,
    dominant_occurrence: DiagnosticOccurrence,
    suppressed_occurrence: DiagnosticOccurrence,
    producer_owner: &'static str,
    producer_stage: &'static str,
    producer_relationship_id: String,
    producer_seal: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticPrecedenceConsumption {
    pub(crate) relationship_id: String,
    pub(crate) rule_id: &'static str,
    pub(crate) applying_owner: &'static str,
    pub(crate) applying_stage: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticProjection {
    producer_stage: &'static str,
    upstream_authority_id: String,
    prior_blockers: Vec<PriorBlockerRef>,
    projection_seal: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagnosticInvariantError {
    UnknownCause,
    CauseCodeMismatch,
    OwnerMismatch,
    OwningStageMismatch,
    SemanticOriginMismatch,
    RelationshipRouteMismatch,
    OccurrenceIdMismatch,
    DiagnosticProjectionMismatch,
    DuplicateOccurrence,
    MissingPriorOccurrence,
    PriorBlockerMismatch,
    DuplicatePriorBlocker,
    PriorBlockerOrderMismatch,
    UnknownPrecedenceRule,
    PrecedenceApplicationMismatch,
    PrecedenceRelationshipMismatch,
    PrecedenceConsumptionCountMismatch,
    ProjectionProvenanceMismatch,
}

#[derive(Default)]
pub(crate) struct DiagnosticOccurrenceCollector {
    occurrences: BTreeMap<DiagnosticOccurrenceId, DiagnosticOccurrence>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DiagnosticOccurrenceSet {
    occurrences: BTreeMap<DiagnosticOccurrenceId, DiagnosticOccurrence>,
}

impl DiagnosticCode {
    pub(crate) const fn from_key(key: crate::diagnostic_catalog::DiagnosticCodeKey) -> Self {
        Self { key }
    }

    pub(crate) const fn key(self) -> crate::diagnostic_catalog::DiagnosticCodeKey {
        self.key
    }

    pub fn as_str(self) -> &'static str {
        crate::diagnostic_catalog::allocation(self.key).spelling
    }

    pub fn title(self) -> &'static str {
        crate::diagnostic_catalog::allocation(self.key).title
    }
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

impl Span {
    pub fn new(file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column,
        }
    }
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            span,
            related_spans: Vec::new(),
            help: None,
        }
    }

    pub fn warning(code: DiagnosticCode, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            message: message.into(),
            span,
            related_spans: Vec::new(),
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_related_span(mut self, label: impl Into<String>, span: Span) -> Self {
        self.related_spans.push(RelatedSpan {
            label: label.into(),
            span,
        });
        self
    }

    pub fn render(&self) -> String {
        let mut rendered = String::new();
        match &self.span {
            Some(span) => {
                rendered.push_str(&format!(
                    "{}:{}:{}: {}[{}]: {}",
                    span.file,
                    span.line,
                    span.column,
                    self.severity.as_str(),
                    self.code.as_str(),
                    self.message
                ));
            }
            None => {
                rendered.push_str(&format!(
                    "{}[{}]: {}",
                    self.severity.as_str(),
                    self.code.as_str(),
                    self.message
                ));
            }
        }
        if let Some(help) = &self.help {
            rendered.push_str("\n  help: ");
            rendered.push_str(help);
        }
        for related in &self.related_spans {
            rendered.push_str(&format!(
                "\n  related: {} at {}:{}:{}",
                related.label, related.span.file, related.span.line, related.span.column
            ));
        }
        rendered
    }
}

impl DiagnosticOccurrence {
    pub(crate) fn registered_cause(
        cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
        diagnostic: Diagnostic,
        semantic_origin: impl Into<String>,
        relationship_route: Vec<String>,
    ) -> Result<Self, DiagnosticInvariantError> {
        let cause = crate::diagnostic_catalog::diagnostic_cause_for_key(cause_key)
            .ok_or(DiagnosticInvariantError::UnknownCause)?;
        let identity = Self::semantic_relationship_identity(
            cause.origin_kind,
            cause.route_kind,
            semantic_origin,
            relationship_route,
        );
        Self::registered(cause, identity, diagnostic)
    }

    pub(crate) fn semantic_relationship_identity(
        origin_kind: &'static str,
        route_kind: &'static str,
        semantic_origin: impl Into<String>,
        relationship_route: Vec<String>,
    ) -> DiagnosticOccurrenceIdentity {
        DiagnosticOccurrenceIdentity {
            origin_kind,
            route_kind,
            semantic_origin: semantic_origin.into(),
            relationship_route,
        }
    }

    pub(crate) fn typed_failure_identity(
        task_identity: &str,
        statement_index: usize,
        form: &str,
        callee_task_identity: Option<&str>,
        callee_definition_id: Option<&str>,
    ) -> DiagnosticOccurrenceIdentity {
        let semantic_origin =
            format!("typed-failure:{task_identity}:statement-{statement_index}:{form}");
        let mut relationship_route = vec![format!(
            "typed_failure_relationship:caller={task_identity}:statement={statement_index}:form={form}"
        )];
        relationship_route.push(format!("typed_failure_task={task_identity}"));
        if let Some(callee_task_identity) = callee_task_identity {
            relationship_route.push(format!("callee_task_identity={callee_task_identity}"));
        }
        if let Some(callee_definition_id) = callee_definition_id {
            relationship_route.push(format!("semantic_site=definition={callee_definition_id}"));
        }
        relationship_route.push(format!(
            "typed_failure_statement={task_identity}:statement-{statement_index}"
        ));
        DiagnosticOccurrenceIdentity {
            origin_kind: "typed_failure_statement",
            route_kind: "typed_failure_relationship",
            semantic_origin,
            relationship_route,
        }
    }

    pub(crate) fn callable_identity(
        detail_reason: &str,
        primary_span: &Span,
        relationship_sites: &[Span],
        semantic_links: Vec<String>,
    ) -> DiagnosticOccurrenceIdentity {
        let semantic_origin =
            crate::node_id::span("callable_relationship", primary_span, detail_reason);
        let mut relationship_route = vec![format!(
            "callable_definition_application_route:primary={}",
            crate::node_id::span("callable-diagnostic-site", primary_span, detail_reason)
        )];
        for site in relationship_sites {
            for role in ["definition", "expression", "unknown_type", "opaque_path"] {
                relationship_route.push(format!(
                    "semantic_site={}",
                    crate::node_id::span("diagnostic-semantic-site", site, role)
                ));
            }
        }
        relationship_route.extend(
            semantic_links
                .iter()
                .map(|link| format!("semantic_site={link}")),
        );
        relationship_route.extend(semantic_links);
        DiagnosticOccurrenceIdentity {
            origin_kind: "callable_relationship",
            route_kind: "callable_definition_application_route",
            semantic_origin,
            relationship_route,
        }
    }

    pub(crate) fn registered(
        cause: &crate::diagnostic_catalog::DiagnosticCauseSpec,
        identity: DiagnosticOccurrenceIdentity,
        diagnostic: Diagnostic,
    ) -> Result<Self, DiagnosticInvariantError> {
        if diagnostic.code != cause.code {
            return Err(DiagnosticInvariantError::CauseCodeMismatch);
        }
        if identity.origin_kind != cause.origin_kind {
            return Err(DiagnosticInvariantError::SemanticOriginMismatch);
        }
        if identity.route_kind != cause.route_kind || identity.relationship_route.is_empty() {
            return Err(DiagnosticInvariantError::RelationshipRouteMismatch);
        }
        let id = occurrence_id(cause.key, &identity);
        Ok(Self {
            id,
            cause_key: cause.key,
            code: cause.code,
            semantic_owner: cause.semantic_owner,
            owning_stage: cause.owning_stage,
            identity,
            resolver_call_occurrence: None,
            diagnostic_seal: diagnostic.clone(),
            diagnostic,
        })
    }

    pub(crate) fn producer_diagnostic(
        cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
        diagnostic: Diagnostic,
        semantic_origin: impl Into<String>,
        relationship_route: Vec<String>,
    ) -> Result<(Diagnostic, Self), DiagnosticInvariantError> {
        Self::registered_cause(cause_key, diagnostic, semantic_origin, relationship_route)
            .map(|occurrence| (occurrence.diagnostic.clone(), occurrence))
    }

    pub(crate) fn validate(&self) -> Result<(), DiagnosticInvariantError> {
        let Some(cause) = crate::diagnostic_catalog::DIAGNOSTIC_CAUSES
            .iter()
            .find(|cause| cause.key == self.cause_key)
        else {
            return Err(DiagnosticInvariantError::UnknownCause);
        };
        if self.diagnostic.code != self.code || cause.code != self.code {
            return Err(DiagnosticInvariantError::CauseCodeMismatch);
        }
        if self.semantic_owner != cause.semantic_owner {
            return Err(DiagnosticInvariantError::OwnerMismatch);
        }
        if self.owning_stage != cause.owning_stage {
            return Err(DiagnosticInvariantError::OwningStageMismatch);
        }
        if self.identity.origin_kind != cause.origin_kind {
            return Err(DiagnosticInvariantError::SemanticOriginMismatch);
        }
        if self.identity.route_kind != cause.route_kind
            || self.identity.relationship_route.is_empty()
        {
            return Err(DiagnosticInvariantError::RelationshipRouteMismatch);
        }
        if self.id != occurrence_id(self.cause_key, &self.identity) {
            return Err(DiagnosticInvariantError::OccurrenceIdMismatch);
        }
        let resolver_route = self
            .identity
            .relationship_route
            .iter()
            .filter_map(|part| part.strip_prefix("resolver_call_occurrence="))
            .collect::<Vec<_>>();
        match &self.resolver_call_occurrence {
            Some(resolver_call) => {
                let stable_key = resolver_call.stable_key();
                if resolver_route.len() != 1 || resolver_route[0] != stable_key {
                    return Err(DiagnosticInvariantError::RelationshipRouteMismatch);
                }
            }
            None if !resolver_route.is_empty() => {
                return Err(DiagnosticInvariantError::RelationshipRouteMismatch);
            }
            None => {}
        }
        if self.diagnostic != self.diagnostic_seal {
            return Err(DiagnosticInvariantError::DiagnosticProjectionMismatch);
        }
        Ok(())
    }

    pub(crate) fn prior_blocker(&self) -> PriorBlockerRef {
        PriorBlockerRef {
            occurrence_id: self.id.clone(),
            cause_key: self.cause_key,
            code: self.code,
            semantic_origin: self.identity.semantic_origin.clone(),
            relationship_route: self.identity.relationship_route.clone(),
            resolver_call_occurrence: self.resolver_call_occurrence.clone(),
        }
    }

    #[cfg(test)]
    fn cause_reason(&self) -> Option<&'static str> {
        crate::diagnostic_catalog::DIAGNOSTIC_CAUSES
            .iter()
            .find(|cause| cause.key == self.cause_key)
            .map(|cause| cause.reason)
    }

    #[cfg(test)]
    pub(crate) fn id(&self) -> &DiagnosticOccurrenceId {
        &self.id
    }
    pub(crate) fn cause_key(&self) -> crate::diagnostic_catalog::DiagnosticCauseKey {
        self.cause_key
    }
    #[cfg(test)]
    pub(crate) fn semantic_owner(&self) -> &'static str {
        self.semantic_owner
    }
    pub(crate) fn owning_stage(&self) -> &'static str {
        self.owning_stage
    }
    pub(crate) fn semantic_origin(&self) -> &str {
        &self.identity.semantic_origin
    }
    pub(crate) fn relationship_route(&self) -> &[String] {
        &self.identity.relationship_route
    }
    pub(crate) fn resolver_call_occurrence(
        &self,
    ) -> Option<&crate::resolve::ResolverCallOccurrenceId> {
        self.resolver_call_occurrence.as_ref()
    }
    pub(crate) fn with_resolver_call(
        mut self,
        call: &crate::resolve::ResolveCallOccurrenceSummary,
    ) -> Result<Self, DiagnosticInvariantError> {
        if self.resolver_call_occurrence.is_some() {
            return Err(DiagnosticInvariantError::RelationshipRouteMismatch);
        }
        self.resolver_call_occurrence = Some(call.resolver_occurrence_id().clone());
        self.validate()?;
        Ok(self)
    }
    pub(crate) fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }

    fn blame_sites(&self) -> Vec<String> {
        let mut sites = Vec::new();
        if let Some(span) = &self.diagnostic.span {
            sites.push(format!(
                "primary={}:{}:{}",
                span.file.replace('\\', "/"),
                span.line,
                span.column
            ));
        }
        sites.extend(self.diagnostic.related_spans.iter().map(|related| {
            format!(
                "related:{}={}:{}:{}",
                related.label,
                related.span.file.replace('\\', "/"),
                related.span.line,
                related.span.column
            )
        }));
        sites
    }

    #[cfg(test)]
    pub(crate) fn diagnostic_mut_for_test(&mut self) -> &mut Diagnostic {
        &mut self.diagnostic
    }
}

impl PriorBlockerRef {
    pub(crate) fn validate_against(
        &self,
        occurrence: &DiagnosticOccurrence,
    ) -> Result<(), DiagnosticInvariantError> {
        if self.occurrence_id != occurrence.id
            || self.cause_key != occurrence.cause_key
            || self.code != occurrence.code
            || self.semantic_origin != occurrence.semantic_origin()
            || self.relationship_route != occurrence.relationship_route()
            || self.resolver_call_occurrence != occurrence.resolver_call_occurrence
        {
            return Err(DiagnosticInvariantError::PriorBlockerMismatch);
        }
        Ok(())
    }
}

impl DiagnosticPrecedenceRelationship {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn producer_owned(
        rule_id: &'static str,
        producer_owner: &'static str,
        producer_stage: &'static str,
        producer_relationship_id: impl Into<String>,
        dominant: &DiagnosticOccurrence,
        suppressed: &DiagnosticOccurrence,
        semantic_nodes: [String; 2],
        relationship_route: Vec<String>,
    ) -> Result<Self, DiagnosticInvariantError> {
        dominant.validate()?;
        suppressed.validate()?;
        let rule = crate::diagnostic_catalog::DIAGNOSTIC_PRECEDENCE
            .iter()
            .find(|rule| rule.id == rule_id)
            .copied()
            .ok_or(DiagnosticInvariantError::UnknownPrecedenceRule)?;
        if !rule.dominant_causes.contains(&dominant.cause_key)
            || !rule.suppressed_causes.contains(&suppressed.cause_key)
        {
            return Err(DiagnosticInvariantError::UnknownPrecedenceRule);
        }
        let producer_relationship_id = producer_relationship_id.into();
        let mut canonical_sites = dominant.blame_sites();
        canonical_sites.extend(suppressed.blame_sites());
        if semantic_nodes
            != [
                dominant.semantic_origin().to_string(),
                suppressed.semantic_origin().to_string(),
            ]
            || relationship_route.is_empty()
            || canonical_sites.is_empty()
            || relationship_route.first() != Some(&producer_relationship_id)
        {
            return Err(DiagnosticInvariantError::PrecedenceRelationshipMismatch);
        }
        if producer_owner.is_empty()
            || producer_stage.is_empty()
            || producer_relationship_id.is_empty()
        {
            return Err(DiagnosticInvariantError::PrecedenceRelationshipMismatch);
        }
        let application = DiagnosticPrecedenceApplication {
            rule_id: rule.id,
            relationship: rule.relationship,
            applying_owner: rule.applying_owner,
            applying_stage: rule.applying_stage,
            dominant: dominant.prior_blocker(),
            suppressed: suppressed.prior_blocker(),
            semantic_nodes,
            relationship_route,
            canonical_sites,
        };
        let producer_seal = precedence_relationship_seal(
            producer_owner,
            producer_stage,
            &producer_relationship_id,
            &application,
        );
        Ok(Self {
            application,
            dominant_occurrence: dominant.clone(),
            suppressed_occurrence: suppressed.clone(),
            producer_owner,
            producer_stage,
            producer_relationship_id,
            producer_seal,
        })
    }

    fn validate_producer_seal(&self) -> Result<(), DiagnosticInvariantError> {
        self.dominant_occurrence.validate()?;
        self.suppressed_occurrence.validate()?;
        self.application
            .dominant
            .validate_against(&self.dominant_occurrence)?;
        self.application
            .suppressed
            .validate_against(&self.suppressed_occurrence)?;
        let rebuilt = Self::producer_owned(
            self.application.rule_id,
            self.producer_owner,
            self.producer_stage,
            self.producer_relationship_id.clone(),
            &self.dominant_occurrence,
            &self.suppressed_occurrence,
            self.application.semantic_nodes.clone(),
            self.application.relationship_route.clone(),
        )?;
        if &rebuilt != self {
            return Err(DiagnosticInvariantError::PrecedenceRelationshipMismatch);
        }
        if self.producer_seal
            != precedence_relationship_seal(
                self.producer_owner,
                self.producer_stage,
                &self.producer_relationship_id,
                &self.application,
            )
        {
            return Err(DiagnosticInvariantError::PrecedenceRelationshipMismatch);
        }
        Ok(())
    }

    pub(crate) fn application(&self) -> &DiagnosticPrecedenceApplication {
        &self.application
    }
}

impl DiagnosticProjection {
    pub(crate) fn from_upstream(
        producer_stage: &'static str,
        upstream: &DiagnosticOccurrenceSet,
    ) -> Result<Self, DiagnosticInvariantError> {
        upstream.validate()?;
        if producer_stage.is_empty() {
            return Err(DiagnosticInvariantError::ProjectionProvenanceMismatch);
        }
        let prior_blockers = upstream.prior_blockers();
        let upstream_authority_id = occurrence_authority_id(upstream);
        let projection_seal =
            projection_seal(producer_stage, &upstream_authority_id, &prior_blockers);
        Ok(Self {
            producer_stage,
            upstream_authority_id,
            prior_blockers,
            projection_seal,
        })
    }

    pub(crate) fn validate_against(
        &self,
        expected_stage: &str,
        upstream: &DiagnosticOccurrenceSet,
    ) -> Result<(), DiagnosticInvariantError> {
        upstream.validate()?;
        if self.producer_stage != expected_stage
            || self.upstream_authority_id != occurrence_authority_id(upstream)
            || self.projection_seal
                != projection_seal(
                    self.producer_stage,
                    &self.upstream_authority_id,
                    &self.prior_blockers,
                )
        {
            return Err(DiagnosticInvariantError::ProjectionProvenanceMismatch);
        }
        upstream.validate_prior_blockers(&self.prior_blockers)
    }

    #[cfg(test)]
    pub(crate) fn prior_blockers_mut_for_test(&mut self) -> &mut Vec<PriorBlockerRef> {
        &mut self.prior_blockers
    }
}

impl DiagnosticOccurrenceCollector {
    pub(crate) fn insert(
        &mut self,
        occurrence: DiagnosticOccurrence,
    ) -> Result<(), DiagnosticInvariantError> {
        occurrence.validate()?;
        if self.occurrences.contains_key(&occurrence.id) {
            return Err(DiagnosticInvariantError::DuplicateOccurrence);
        }
        self.occurrences.insert(occurrence.id.clone(), occurrence);
        Ok(())
    }

    pub(crate) fn validate_prior(
        &self,
        prior: &PriorBlockerRef,
    ) -> Result<(), DiagnosticInvariantError> {
        let occurrence = self
            .occurrences
            .get(&prior.occurrence_id)
            .ok_or(DiagnosticInvariantError::MissingPriorOccurrence)?;
        prior.validate_against(occurrence)
    }
}

impl DiagnosticOccurrenceSet {
    pub(crate) fn insert_owned(
        &mut self,
        occurrence: DiagnosticOccurrence,
    ) -> Result<(), DiagnosticInvariantError> {
        occurrence.validate()?;
        if self.occurrences.contains_key(&occurrence.id) {
            return Err(DiagnosticInvariantError::DuplicateOccurrence);
        }
        self.occurrences.insert(occurrence.id.clone(), occurrence);
        Ok(())
    }

    pub(crate) fn inherited(&self) -> Self {
        self.clone()
    }

    pub(crate) fn extend_owned(&mut self, other: &Self) -> Result<(), DiagnosticInvariantError> {
        for occurrence in other.occurrences.values() {
            self.insert_owned(occurrence.clone())?;
        }
        Ok(())
    }

    pub(crate) fn extend_owned_stage(
        &mut self,
        other: &Self,
        owning_stage: &str,
    ) -> Result<(), DiagnosticInvariantError> {
        for occurrence in other
            .occurrences
            .values()
            .filter(|occurrence| occurrence.owning_stage == owning_stage)
        {
            self.insert_owned(occurrence.clone())?;
        }
        Ok(())
    }

    pub(crate) fn validate_projection_from(
        authoritative: &Self,
        projected: &Self,
    ) -> Result<(), DiagnosticInvariantError> {
        authoritative.validate()?;
        projected.validate()?;
        authoritative.validate_prior_blockers(&projected.prior_blockers())
    }

    pub(crate) fn consume_precedence_relationship(
        &self,
        relationship: &DiagnosticPrecedenceRelationship,
    ) -> Result<DiagnosticPrecedenceConsumption, DiagnosticInvariantError> {
        self.validate()?;
        relationship.validate_producer_seal()?;
        self.validate_precedence_relationship(relationship)?;
        let application = relationship.application();
        Ok(DiagnosticPrecedenceConsumption {
            relationship_id: relationship.producer_relationship_id.clone(),
            rule_id: application.rule_id,
            applying_owner: application.applying_owner,
            applying_stage: application.applying_stage,
        })
    }

    fn validate_precedence_relationship(
        &self,
        relationship: &DiagnosticPrecedenceRelationship,
    ) -> Result<(), DiagnosticInvariantError> {
        relationship.validate_producer_seal()?;
        let application = relationship.application();
        let dominant = &relationship.dominant_occurrence;
        let suppressed = &relationship.suppressed_occurrence;
        application.dominant.validate_against(dominant)?;
        application.suppressed.validate_against(suppressed)?;
        let rule = crate::diagnostic_catalog::exact_precedence_spec(
            dominant.cause_key,
            suppressed.cause_key,
            application.relationship,
            application.applying_owner,
            application.applying_stage,
        )
        .filter(|rule| rule.id == application.rule_id)
        .ok_or(DiagnosticInvariantError::UnknownPrecedenceRule)?;
        if application.semantic_nodes
            != [
                dominant.semantic_origin().to_string(),
                suppressed.semantic_origin().to_string(),
            ]
            || application.rule_id != rule.id
            || application.relationship != rule.relationship
            || application.applying_owner != rule.applying_owner
            || application.applying_stage != rule.applying_stage
            || application.relationship_route.is_empty()
            || application.canonical_sites.is_empty()
        {
            return Err(DiagnosticInvariantError::PrecedenceApplicationMismatch);
        }
        let authoritative_dominant = self
            .occurrences
            .get(&application.dominant.occurrence_id)
            .ok_or(DiagnosticInvariantError::MissingPriorOccurrence)?;
        application
            .dominant
            .validate_against(authoritative_dominant)?;
        if let Some(authoritative_suppressed) =
            self.occurrences.get(&application.suppressed.occurrence_id)
        {
            application
                .suppressed
                .validate_against(authoritative_suppressed)?;
        }
        Ok(())
    }

    pub(crate) fn consume_precedence_relationships(
        &self,
        applying_stage: &str,
        relationships: &[DiagnosticPrecedenceRelationship],
        expected_count: usize,
    ) -> Result<Vec<DiagnosticPrecedenceConsumption>, DiagnosticInvariantError> {
        let mut seen = BTreeMap::new();
        let mut consumptions = Vec::new();
        for relationship in relationships {
            if relationship.application.applying_stage != applying_stage
                || seen
                    .insert(relationship.producer_relationship_id.clone(), ())
                    .is_some()
            {
                return Err(DiagnosticInvariantError::PrecedenceRelationshipMismatch);
            }
            consumptions.push(self.consume_precedence_relationship(relationship)?);
        }
        if consumptions.len() != expected_count {
            return Err(DiagnosticInvariantError::PrecedenceConsumptionCountMismatch);
        }
        Ok(consumptions)
    }

    pub(crate) fn prior_blockers(&self) -> Vec<PriorBlockerRef> {
        self.occurrences
            .values()
            .map(DiagnosticOccurrence::prior_blocker)
            .collect()
    }

    pub(crate) fn validate_prior_blockers(
        &self,
        priors: &[PriorBlockerRef],
    ) -> Result<(), DiagnosticInvariantError> {
        let expected = self.prior_blockers();
        if priors.len() != expected.len() {
            return Err(DiagnosticInvariantError::PriorBlockerMismatch);
        }
        let mut seen = BTreeMap::new();
        for prior in priors {
            if seen.insert(prior.occurrence_id.clone(), ()).is_some() {
                return Err(DiagnosticInvariantError::DuplicatePriorBlocker);
            }
            let occurrence = self
                .occurrences
                .get(&prior.occurrence_id)
                .ok_or(DiagnosticInvariantError::MissingPriorOccurrence)?;
            prior.validate_against(occurrence)?;
        }
        if priors != expected {
            return Err(DiagnosticInvariantError::PriorBlockerOrderMismatch);
        }
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<(), DiagnosticInvariantError> {
        for occurrence in self.occurrences.values() {
            occurrence.validate()?;
        }
        Ok(())
    }

    pub(crate) fn occurrences(&self) -> impl Iterator<Item = &DiagnosticOccurrence> {
        self.occurrences.values()
    }

    #[cfg(test)]
    pub(crate) fn remove_first_for_test(&mut self) {
        if let Some(id) = self.occurrences.keys().next().cloned() {
            self.occurrences.remove(&id);
        }
    }

    #[cfg(test)]
    pub(crate) fn corrupt_first_diagnostic_for_test(&mut self) {
        if let Some(occurrence) = self.occurrences.values_mut().next() {
            occurrence.diagnostic.message.push_str(" corrupt");
        }
    }
}

fn occurrence_id(
    cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    identity: &DiagnosticOccurrenceIdentity,
) -> DiagnosticOccurrenceId {
    DiagnosticOccurrenceId(format!(
        "diagnostic-occurrence:{}:{}:{}",
        cause_key.ordinal(),
        identity.semantic_origin,
        identity.relationship_route.join(">")
    ))
}

fn occurrence_authority_id(set: &DiagnosticOccurrenceSet) -> String {
    set.occurrences
        .keys()
        .map(DiagnosticOccurrenceId::as_str)
        .collect::<Vec<_>>()
        .join("|")
}

fn projection_seal(
    producer_stage: &str,
    upstream_authority_id: &str,
    prior_blockers: &[PriorBlockerRef],
) -> String {
    format!(
        "projection:{producer_stage}:{upstream_authority_id}:{}",
        prior_blockers
            .iter()
            .map(|prior| prior.occurrence_id.as_str())
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn precedence_relationship_seal(
    producer_owner: &str,
    producer_stage: &str,
    producer_relationship_id: &str,
    application: &DiagnosticPrecedenceApplication,
) -> String {
    format!(
        "precedence-producer:{producer_owner}:{producer_stage}:{producer_relationship_id}:{}:{}:{}:{}:{}:{}:{}",
        application.rule_id,
        application.dominant.occurrence_id.as_str(),
        application.suppressed.occurrence_id.as_str(),
        application.semantic_nodes.join("|"),
        application.relationship_route.join("|"),
        application.canonical_sites.join("|"),
        application.relationship,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        Diagnostic, DiagnosticCode, DiagnosticInvariantError, DiagnosticOccurrence,
        DiagnosticOccurrenceCollector, DiagnosticOccurrenceSet, DiagnosticPrecedenceRelationship,
        Span,
    };

    #[test]
    fn render_includes_stable_code() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UNDECLARED_SAVE_TARGET,
            "declared mutation is missing",
            Some(Span::new("bad.hum", 7, 3)),
        )
        .with_help("Add the target under `changes:`.");

        let rendered = diagnostic.render();
        assert!(rendered.contains("bad.hum:7:3: error[H0201]"));
        assert!(rendered.contains("help: Add the target under `changes:`."));
    }

    fn occurrence(statement_index: usize) -> DiagnosticOccurrence {
        let cause = crate::diagnostic_catalog::diagnostic_cause(
            DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY,
            "fallible_call_requires_try_v0",
        )
        .expect("registered test cause");
        let span = Span::new("fixture.hum", 7 + statement_index, 3);
        let identity = DiagnosticOccurrence::typed_failure_identity(
            "typed-failure-task:test",
            statement_index,
            "implicit_fallible_call",
            Some("resolver-item:file-0:path-0"),
            Some("def_0_task_test"),
        );
        DiagnosticOccurrence::registered(
            cause,
            identity,
            Diagnostic::error(
                DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY,
                "stable public message",
                Some(span),
            )
            .with_help("stable public help")
            .with_related_span("callee", Span::new("fixture.hum", 3, 1)),
        )
        .expect("valid occurrence")
    }

    fn exact_reason_occurrence(
        code: DiagnosticCode,
        reason: &'static str,
        semantic_origin: &str,
        route: &str,
    ) -> DiagnosticOccurrence {
        let cause = crate::diagnostic_catalog::diagnostic_cause_for_reason(reason)
            .expect("registered test cause");
        DiagnosticOccurrence::registered_cause(
            cause.key,
            Diagnostic::error(
                code,
                "stable public diagnostic",
                Some(Span::new("fixture.hum", 11, 7)),
            ),
            semantic_origin,
            vec![
                route.to_string(),
                format!("semantic_site={semantic_origin}"),
            ],
        )
        .expect("registered exact cause")
    }

    fn program_and_static_diagnostics(
        path: &str,
        source: &str,
        include_app_boundaries: bool,
    ) -> (crate::ast::Program, Vec<Diagnostic>) {
        let parsed = crate::parser::parse_source(path, source);
        let mut diagnostics = parsed.diagnostics;
        diagnostics.extend(crate::check::check_file(&parsed.file));
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        if include_app_boundaries {
            diagnostics.extend(crate::app_entry::diagnostics(&program));
            diagnostics.extend(crate::path_boundary::diagnostics(&program));
            diagnostics.extend(crate::capability_root::diagnostics(&program));
        }
        (program, diagnostics)
    }

    fn program_diagnostics_and_authority(
        path: &str,
        source: &str,
        include_app_boundaries: bool,
    ) -> (
        crate::ast::Program,
        Vec<Diagnostic>,
        DiagnosticOccurrenceSet,
    ) {
        let parsed = crate::parser::parse_source(path, source);
        let checked = crate::check::check_file_with_occurrences(&parsed.file);
        let mut occurrences = parsed.diagnostic_occurrences.clone();
        occurrences
            .extend_owned(&checked.diagnostic_occurrences)
            .expect("source occurrences");
        let mut diagnostics = parsed.diagnostics;
        diagnostics.extend(checked.diagnostics);
        let program = crate::ast::Program {
            files: vec![parsed.file],
        };
        if include_app_boundaries {
            diagnostics.extend(crate::app_entry::diagnostics(&program));
            occurrences
                .extend_owned(&crate::app_entry::diagnostic_occurrence_set(&program))
                .expect("app occurrences");
            diagnostics.extend(crate::path_boundary::diagnostics(&program));
            occurrences
                .extend_owned(&crate::path_boundary::diagnostic_occurrence_set(&program))
                .expect("Path occurrences");
            diagnostics.extend(crate::capability_root::diagnostics(&program));
            occurrences
                .extend_owned(&crate::capability_root::diagnostic_occurrence_set(&program))
                .expect("capability occurrences");
        }
        (program, diagnostics, occurrences)
    }

    #[test]
    fn occurrence_identity_rejects_every_registered_field_mutation() {
        let baseline = occurrence(0);

        let mut wrong_cause = baseline.clone();
        wrong_cause.cause_key = crate::diagnostic_catalog::diagnostic_cause(
            DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION,
            "unwrapped_failure_roots_must_match_v0",
        )
        .expect("second cause")
        .key;
        assert_eq!(
            wrong_cause.validate(),
            Err(DiagnosticInvariantError::CauseCodeMismatch)
        );

        let mut wrong_code = baseline.clone();
        wrong_code.code = DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION;
        assert_eq!(
            wrong_code.validate(),
            Err(DiagnosticInvariantError::CauseCodeMismatch)
        );

        let mut wrong_owner = baseline.clone();
        wrong_owner.semantic_owner = "different_owner";
        assert_eq!(
            wrong_owner.validate(),
            Err(DiagnosticInvariantError::OwnerMismatch)
        );

        let mut wrong_stage = baseline.clone();
        wrong_stage.owning_stage = "effect_check";
        assert_eq!(
            wrong_stage.validate(),
            Err(DiagnosticInvariantError::OwningStageMismatch)
        );

        let mut wrong_origin = baseline.clone();
        wrong_origin.identity.origin_kind = "display_name_only";
        assert_eq!(
            wrong_origin.validate(),
            Err(DiagnosticInvariantError::SemanticOriginMismatch)
        );

        let mut wrong_route = baseline.clone();
        wrong_route.identity.route_kind = "generic_route";
        assert_eq!(
            wrong_route.validate(),
            Err(DiagnosticInvariantError::RelationshipRouteMismatch)
        );

        let mut wrong_id = baseline.clone();
        wrong_id.id.0.push_str("-corrupt");
        assert_eq!(
            wrong_id.validate(),
            Err(DiagnosticInvariantError::OccurrenceIdMismatch)
        );
    }

    #[test]
    fn collector_rejects_duplicate_emission_and_exact_prior_substitution() {
        let baseline = occurrence(0);
        let mut collector = DiagnosticOccurrenceCollector::default();
        collector.insert(baseline.clone()).expect("first owner");
        assert_eq!(
            collector.insert(baseline.clone()),
            Err(DiagnosticInvariantError::DuplicateOccurrence)
        );

        let prior = baseline.prior_blocker();
        collector.validate_prior(&prior).expect("exact prior");
        let mut substituted = prior.clone();
        substituted.code = DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION;
        assert_eq!(
            collector.validate_prior(&substituted),
            Err(DiagnosticInvariantError::PriorBlockerMismatch)
        );

        let other = occurrence(1);
        collector
            .insert(other)
            .expect("same code and cause with a distinct semantic origin");
    }

    #[test]
    fn rendered_fields_cannot_create_occurrence_identity() {
        let baseline = occurrence(0);
        for mutation in 0..4 {
            let mut rendered_only = baseline.clone();
            match mutation {
                0 => rendered_only.diagnostic.message = "different message".to_string(),
                1 => rendered_only.diagnostic.help = Some("different help".to_string()),
                2 => rendered_only.diagnostic.span = Some(Span::new("fixture.hum", 99, 1)),
                _ => rendered_only.diagnostic.related_spans.clear(),
            }
            assert_eq!(
                rendered_only.validate(),
                Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
            );
            let mut collector = DiagnosticOccurrenceCollector::default();
            collector.insert(baseline.clone()).expect("baseline");
            assert_eq!(
                collector.insert(rendered_only),
                Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
            );
        }
    }

    #[test]
    fn code_only_and_public_identical_alternative_causes_fail_closed() {
        assert_eq!(
            DiagnosticOccurrence::registered_cause(
                crate::diagnostic_catalog::DiagnosticCauseKey::producer_owned(u16::MAX),
                Diagnostic::error(
                    DiagnosticCode::FALLIBLE_CALL_REQUIRES_TRY,
                    "public code without producer cause",
                    Some(Span::new("fixture.hum", 7, 3)),
                ),
                "code-only-attempt",
                vec!["code-only-attempt".to_string()],
            ),
            Err(DiagnosticInvariantError::UnknownCause)
        );

        let pair = crate::diagnostic_catalog::DIAGNOSTIC_CAUSES
            .iter()
            .enumerate()
            .find_map(|(index, first)| {
                crate::diagnostic_catalog::DIAGNOSTIC_CAUSES[index + 1..]
                    .iter()
                    .find(|second| {
                        first.code == second.code
                            && first.origin_kind == second.origin_kind
                            && first.route_kind == second.route_kind
                    })
                    .map(|second| (first, second))
            })
            .expect("registry has a same-code alternative producer cause");
        let diagnostic = Diagnostic::error(
            pair.0.code,
            "public-identical diagnostic",
            Some(Span::new("fixture.hum", 9, 5)),
        );
        let identity = DiagnosticOccurrence::semantic_relationship_identity(
            pair.0.origin_kind,
            pair.0.route_kind,
            "producer-owned-node",
            vec!["producer-owned-route".to_string()],
        );
        let authoritative =
            DiagnosticOccurrence::registered(pair.0, identity.clone(), diagnostic.clone())
                .expect("first producer cause");
        let substituted = DiagnosticOccurrence::registered(pair.1, identity, diagnostic)
            .expect("same public bytes but different producer cause");
        assert_ne!(authoritative.id(), substituted.id());
        let mut authority = DiagnosticOccurrenceSet::default();
        authority.insert_owned(authoritative).expect("authority");
        let mut projection = DiagnosticOccurrenceSet::default();
        projection.insert_owned(substituted).expect("projection");
        assert_eq!(
            DiagnosticOccurrenceSet::validate_projection_from(&authority, &projection),
            Err(DiagnosticInvariantError::MissingPriorOccurrence)
        );
    }

    #[test]
    fn canonical_identity_rejects_valid_prefix_and_route_substitutions() {
        let baseline = occurrence(0);
        let other = occurrence(1);

        let mut origin_substitution = baseline.clone();
        origin_substitution.identity.semantic_origin = other.identity.semantic_origin.clone();
        assert_eq!(
            origin_substitution.validate(),
            Err(DiagnosticInvariantError::OccurrenceIdMismatch)
        );

        let mut route_substitution = baseline.clone();
        route_substitution.identity.relationship_route = other.identity.relationship_route.clone();
        assert_eq!(
            route_substitution.validate(),
            Err(DiagnosticInvariantError::OccurrenceIdMismatch)
        );

        let mut added_route = baseline.clone();
        added_route
            .identity
            .relationship_route
            .push("typed_failure_relationship:valid-looking-added-route".to_string());
        assert_eq!(
            added_route.validate(),
            Err(DiagnosticInvariantError::OccurrenceIdMismatch)
        );

        let mut removed_route = baseline.clone();
        removed_route.identity.relationship_route.pop();
        assert_eq!(
            removed_route.validate(),
            Err(DiagnosticInvariantError::OccurrenceIdMismatch)
        );

        let mut cross_occurrence = baseline.clone();
        cross_occurrence.identity = other.identity;
        assert_eq!(
            cross_occurrence.validate(),
            Err(DiagnosticInvariantError::OccurrenceIdMismatch)
        );
    }

    #[test]
    fn occurrence_set_rejects_every_prior_blocker_projection_corruption() {
        let first = occurrence(0);
        let second = occurrence(1);
        let mut set = DiagnosticOccurrenceSet::default();
        set.insert_owned(first.clone()).expect("first occurrence");
        set.insert_owned(second.clone()).expect("second occurrence");
        set.validate().expect("canonical occurrence set");
        assert_eq!(set.occurrences().count(), 2);

        let canonical = set.prior_blockers();
        set.validate_prior_blockers(&canonical)
            .expect("canonical prior blockers");

        let mut missing = canonical.clone();
        missing.pop();
        assert_eq!(
            set.validate_prior_blockers(&missing),
            Err(DiagnosticInvariantError::PriorBlockerMismatch)
        );

        let mut duplicate = canonical.clone();
        duplicate[1] = duplicate[0].clone();
        assert_eq!(
            set.validate_prior_blockers(&duplicate),
            Err(DiagnosticInvariantError::DuplicatePriorBlocker)
        );

        let mut reordered = canonical.clone();
        reordered.swap(0, 1);
        assert_eq!(
            set.validate_prior_blockers(&reordered),
            Err(DiagnosticInvariantError::PriorBlockerOrderMismatch)
        );

        let mut substituted = canonical.clone();
        substituted[0].code = DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION;
        assert_eq!(
            set.validate_prior_blockers(&substituted),
            Err(DiagnosticInvariantError::PriorBlockerMismatch)
        );

        let mut extra = canonical.clone();
        extra.push(first.prior_blocker());
        assert_eq!(
            set.validate_prior_blockers(&extra),
            Err(DiagnosticInvariantError::PriorBlockerMismatch)
        );

        let mut missing_projection = set.clone();
        missing_projection.remove_first_for_test();
        assert_eq!(
            DiagnosticOccurrenceSet::validate_projection_from(&set, &missing_projection),
            Err(DiagnosticInvariantError::PriorBlockerMismatch)
        );

        let mut internally_corrupt = set.clone();
        internally_corrupt.corrupt_first_diagnostic_for_test();
        assert_eq!(
            DiagnosticOccurrenceSet::validate_projection_from(&set, &internally_corrupt),
            Err(DiagnosticInvariantError::DiagnosticProjectionMismatch)
        );
    }

    #[test]
    fn every_static_stage_projection_rejects_independent_corruption() {
        let source = include_str!(
            "../fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum"
        );
        let (program, diagnostics, source_occurrences) = program_diagnostics_and_authority(
            "fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum",
            source,
            false,
        );
        let authority = crate::type_check::diagnostic_occurrence_set_from_source(
            &program,
            &diagnostics,
            &source_occurrences,
        );
        assert_eq!(authority.occurrences().count(), 2);
        let projections = vec![
            (
                "core_lower",
                crate::core_lower::diagnostic_projection_from_preview(&authority)
                    .expect("Core lower projection"),
            ),
            (
                "effect_check",
                crate::effect_check::diagnostic_projection_from_full_type(&authority)
                    .expect("effect projection"),
            ),
            (
                "ownership_check",
                crate::ownership_check::diagnostic_projection_from_effect(&authority)
                    .expect("ownership projection"),
            ),
            (
                "resource_check",
                crate::resource_check::diagnostic_projection_from_ownership(&authority)
                    .expect("resource projection"),
            ),
            (
                "profile_check",
                crate::profile_check::diagnostic_projection_from_resource(&authority)
                    .expect("profile projection"),
            ),
            (
                "graph",
                super::DiagnosticProjection::from_upstream("graph", &authority)
                    .expect("graph projection"),
            ),
        ];

        for (stage, canonical) in projections {
            canonical
                .validate_against(stage, &authority)
                .expect("canonical stage projection");

            let mut missing = canonical.clone();
            missing.prior_blockers_mut_for_test().pop();
            assert!(
                missing.validate_against(stage, &authority).is_err(),
                "{stage} missing"
            );

            let mut duplicate = canonical.clone();
            let first = duplicate.prior_blockers_mut_for_test()[0].clone();
            duplicate.prior_blockers_mut_for_test().push(first.clone());
            assert!(
                duplicate.validate_against(stage, &authority).is_err(),
                "{stage} duplicate"
            );

            let mut reordered = canonical.clone();
            reordered.prior_blockers_mut_for_test().swap(0, 1);
            assert!(
                reordered.validate_against(stage, &authority).is_err(),
                "{stage} reordered"
            );

            let mut substituted = canonical.clone();
            substituted.prior_blockers_mut_for_test()[0]
                .semantic_origin
                .push_str(":substituted");
            assert!(
                substituted.validate_against(stage, &authority).is_err(),
                "{stage} substituted"
            );

            let mut cross_occurrence = canonical.clone();
            let second = cross_occurrence.prior_blockers_mut_for_test()[1].clone();
            cross_occurrence.prior_blockers_mut_for_test()[0] = second;
            assert!(
                cross_occurrence
                    .validate_against(stage, &authority)
                    .is_err(),
                "{stage} cross occurrence"
            );

            let mut extra = canonical;
            extra.prior_blockers_mut_for_test().push(first);
            assert!(
                extra.validate_against(stage, &authority).is_err(),
                "{stage} extra"
            );
        }
    }

    #[test]
    fn exact_precedence_rejects_every_relationship_and_occurrence_substitution() {
        let dominant = exact_reason_occurrence(
            DiagnosticCode::PATH_SOURCE_CONSTRUCTION,
            "path_source_construction_v0",
            "path-node:input",
            "path_boundary_route:input",
        );
        let suppressed = exact_reason_occurrence(
            DiagnosticCode::INVALID_EXECUTABLE_PREDICATE,
            "malformed_executable_predicate_v2",
            "predicate-node:input",
            "predicate_place_route:input",
        );
        let same_code_alternative = exact_reason_occurrence(
            DiagnosticCode::INVALID_EXECUTABLE_PREDICATE,
            "arbitrary_helper_call_not_allowed_v2",
            "predicate-node:helper",
            "predicate_place_route:helper",
        );
        let mut set = DiagnosticOccurrenceSet::default();
        set.insert_owned(dominant.clone())
            .expect("dominant authority");
        let canonical = DiagnosticPrecedenceRelationship::producer_owned(
            "path_over_predicate_v0",
            "predicate",
            "core_preview",
            "path-predicate-relationship:input",
            &dominant,
            &suppressed,
            [
                dominant.semantic_origin().to_string(),
                suppressed.semantic_origin().to_string(),
            ],
            vec!["path-predicate-relationship:input".to_string()],
        )
        .expect("producer-owned Path/predicate relationship");
        let consumptions =
            crate::core_preview::consume_path_precedence(&set, std::slice::from_ref(&canonical), 1)
                .expect("named production stage consumes exactly one relationship");
        assert_eq!(consumptions.len(), 1);
        assert_eq!(consumptions[0].rule_id, "path_over_predicate_v0");
        assert_eq!(consumptions[0].applying_owner, "predicate");
        assert_eq!(consumptions[0].applying_stage, "core_preview");
        assert_eq!(
            consumptions[0].relationship_id,
            "path-predicate-relationship:input"
        );

        for field in 0..11 {
            let mut corrupt = canonical.clone();
            match field {
                0 => corrupt.application.rule_id = "different_rule_v0",
                1 => corrupt.application.relationship = "different_relationship_v0",
                2 => corrupt.application.applying_owner = "different_owner",
                3 => corrupt.application.applying_stage = "different_stage",
                4 => corrupt.application.semantic_nodes[0] = "compatible-looking-node".to_string(),
                5 => {
                    corrupt.application.relationship_route[0] =
                        "compatible-looking-route".to_string()
                }
                6 => corrupt.application.canonical_sites[0] = "compatible-looking-site".to_string(),
                7 => {
                    corrupt.suppressed_occurrence = same_code_alternative.clone();
                    corrupt.application.suppressed = same_code_alternative.prior_blocker();
                }
                8 => corrupt.producer_owner = "different_producer",
                9 => corrupt.producer_stage = "different_producer_stage",
                _ => corrupt.producer_relationship_id = "different-relationship-id".to_string(),
            }
            assert!(
                crate::core_preview::consume_path_precedence(
                    &set,
                    std::slice::from_ref(&corrupt),
                    1,
                )
                .is_err(),
                "precedence mutation {field}"
            );
        }

        let mut compatible_span_public_diagnostic = canonical;
        compatible_span_public_diagnostic
            .application
            .suppressed
            .semantic_origin = same_code_alternative.semantic_origin().to_string();
        assert_eq!(
            crate::core_preview::consume_path_precedence(
                &set,
                std::slice::from_ref(&compatible_span_public_diagnostic),
                1,
            ),
            Err(DiagnosticInvariantError::PriorBlockerMismatch)
        );
    }

    #[test]
    fn registered_ap_rules_dispatch_only_to_their_named_consumers() {
        for rule_id in [
            "parser_over_resolver_v0",
            "resolver_over_type_v0",
            "authority_over_ownership_v0",
            "path_over_predicate_v0",
            "effect_failure_over_ownership_v0",
        ] {
            let rule = crate::diagnostic_catalog::DIAGNOSTIC_PRECEDENCE
                .iter()
                .find(|candidate| candidate.id == rule_id)
                .copied()
                .expect("AP rule");
            let dominant_cause = crate::diagnostic_catalog::DIAGNOSTIC_CAUSES
                .iter()
                .find(|cause| cause.key == rule.dominant_causes[0])
                .expect("dominant cause");
            let suppressed_cause = crate::diagnostic_catalog::DIAGNOSTIC_CAUSES
                .iter()
                .find(|cause| cause.key == rule.suppressed_causes[0])
                .expect("suppressed cause");
            let dominant = DiagnosticOccurrence::registered(
                dominant_cause,
                DiagnosticOccurrence::semantic_relationship_identity(
                    dominant_cause.origin_kind,
                    dominant_cause.route_kind,
                    format!("{rule_id}:dominant-node"),
                    vec![format!("{rule_id}:dominant-route")],
                ),
                Diagnostic::error(
                    dominant_cause.code,
                    "producer-owned dominant projection",
                    Some(Span::new("precedence.hum", 3, 1)),
                ),
            )
            .expect("dominant occurrence");
            let suppressed = DiagnosticOccurrence::registered(
                suppressed_cause,
                DiagnosticOccurrence::semantic_relationship_identity(
                    suppressed_cause.origin_kind,
                    suppressed_cause.route_kind,
                    format!("{rule_id}:suppressed-node"),
                    vec![format!("{rule_id}:suppressed-route")],
                ),
                Diagnostic::error(
                    suppressed_cause.code,
                    "producer-owned suppressed projection",
                    Some(Span::new("precedence.hum", 3, 1)),
                ),
            )
            .expect("suppressed occurrence");
            let relationship = DiagnosticPrecedenceRelationship::producer_owned(
                rule.id,
                rule.applying_owner,
                rule.applying_stage,
                format!("producer-relationship:{rule_id}"),
                &dominant,
                &suppressed,
                [
                    dominant.semantic_origin().to_string(),
                    suppressed.semantic_origin().to_string(),
                ],
                vec![format!("producer-relationship:{rule_id}")],
            )
            .expect("producer relationship");
            let mut set = DiagnosticOccurrenceSet::default();
            set.insert_owned(dominant).expect("dominant authority");
            let result = match rule.applying_stage {
                "resolve" => crate::resolve::consume_parser_precedence(
                    &set,
                    std::slice::from_ref(&relationship),
                    1,
                ),
                "type_check" => crate::type_check::consume_resolver_precedence(
                    &set,
                    std::slice::from_ref(&relationship),
                    1,
                ),
                "core_preview" => crate::core_preview::consume_path_precedence(
                    &set,
                    std::slice::from_ref(&relationship),
                    1,
                ),
                "ownership_check" => crate::ownership_check::consume_ownership_precedence(
                    &set,
                    std::slice::from_ref(&relationship),
                    1,
                ),
                _ => unreachable!("unexpected AP applying stage"),
            }
            .expect("named stage consumption");
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].rule_id, rule_id);
        }
    }

    #[test]
    fn static_prior_blocker_chain_preserves_exact_type_occurrence() {
        let source =
            include_str!("../fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum");
        let (program, diagnostics, source_occurrences) = program_diagnostics_and_authority(
            "fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum",
            source,
            false,
        );
        let type_set = crate::type_check::diagnostic_occurrence_set_from_source(
            &program,
            &diagnostics,
            &source_occurrences,
        );
        assert_eq!(type_set.occurrences().count(), 1);
        assert_eq!(
            type_set.occurrences().next().map(|fact| fact.code),
            Some(DiagnosticCode::UNKNOWN_TYPE_NAME)
        );
        for downstream in [
            crate::full_type_check::diagnostic_occurrence_set_from_source(
                &program,
                &diagnostics,
                &source_occurrences,
            )
            .expect("full type transport"),
            crate::effect_check::diagnostic_occurrence_set_from_source(
                &program,
                &diagnostics,
                &source_occurrences,
            )
            .expect("effect transport"),
            crate::ownership_check::diagnostic_occurrence_set_from_source(
                &program,
                &diagnostics,
                &source_occurrences,
            )
            .expect("ownership transport"),
            crate::resource_check::diagnostic_occurrence_set_from_source(
                &program,
                &diagnostics,
                &source_occurrences,
            )
            .expect("resource transport"),
            crate::profile_check::diagnostic_occurrence_set_from_source(
                &program,
                &diagnostics,
                &source_occurrences,
            )
            .expect("profile transport"),
        ] {
            assert_eq!(downstream, type_set);
            downstream
                .validate_prior_blockers(&type_set.prior_blockers())
                .expect("exact prior blocker chain");
        }
        crate::profile_check::validate_prior_blocker_projection(&program, &diagnostics)
            .expect("resource/profile projection");
    }

    #[test]
    fn same_line_type_causes_keep_distinct_semantic_origins() {
        let source = include_str!(
            "../fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum"
        );
        let (program, diagnostics, source_occurrences) = program_diagnostics_and_authority(
            "fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum",
            source,
            false,
        );
        let occurrences = crate::type_check::diagnostic_occurrence_set_from_source(
            &program,
            &diagnostics,
            &source_occurrences,
        );
        let matching = occurrences
            .occurrences()
            .filter(|occurrence| occurrence.code == DiagnosticCode::UNKNOWN_TYPE_NAME)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 2);
        assert_ne!(matching[0].id(), matching[1].id());
        assert_ne!(matching[0].semantic_origin(), matching[1].semantic_origin());
        assert_eq!(
            matching
                .iter()
                .map(|occurrence| occurrence.diagnostic().span.as_ref().unwrap().line)
                .collect::<Vec<_>>(),
            vec![3, 3]
        );
    }

    #[test]
    fn parser_and_resolver_causes_remain_distinct_through_type_environment() {
        let source =
            include_str!("../fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum");
        let (program, diagnostics, source_occurrences) = program_diagnostics_and_authority(
            "fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum",
            source,
            false,
        );
        let resolved = crate::resolve::diagnostic_occurrence_set_from_source(
            &program,
            &diagnostics,
            &source_occurrences,
        );
        assert_eq!(
            resolved
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::UNEXPECTED_TOP_LEVEL_LINE)
                .count(),
            1
        );
        assert_eq!(
            resolved
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::UNRESOLVED_NAME)
                .count(),
            1
        );
        assert_eq!(
            crate::type_env::type_env_report_from_source(
                &program,
                &diagnostics,
                &source_occurrences,
            )
            .diagnostic_occurrences,
            resolved
        );
    }

    #[test]
    fn path_and_authority_precedence_preserve_only_fundamental_static_causes() {
        let path_source =
            include_str!("../fixtures/diagnostics/session_ap_path_predicate_precedence_fail.hum");
        let (path_program, path_diagnostics, path_source_occurrences) =
            program_diagnostics_and_authority(
                "fixtures/diagnostics/session_ap_path_predicate_precedence_fail.hum",
                path_source,
                true,
            );
        let path_set = crate::full_type_check::diagnostic_occurrence_set_from_source(
            &path_program,
            &path_diagnostics,
            &path_source_occurrences,
        )
        .expect("Path source transport");
        let path_preview_set = crate::core_preview::diagnostic_occurrence_set_from_source(
            &path_program,
            &path_diagnostics,
            &path_source_occurrences,
        );
        let path_relationships = crate::core_preview::path_precedence_relationships(
            &path_preview_set,
            &crate::predicate::analyze_program(&path_program),
        );
        assert_eq!(path_relationships.len(), 1);
        assert_eq!(
            crate::core_preview::consume_path_precedence(
                &path_preview_set,
                &path_relationships,
                1,
            )
            .expect("genuine Path/predicate relationship")
            .len(),
            1
        );
        assert_eq!(
            path_set
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::PATH_SOURCE_CONSTRUCTION)
                .count(),
            1
        );
        assert_eq!(
            path_set
                .occurrences()
                .filter(|occurrence| {
                    occurrence.code == DiagnosticCode::INVALID_EXECUTABLE_PREDICATE
                })
                .count(),
            0
        );

        let authority_source = include_str!(
            "../fixtures/diagnostics/session_ap_authority_ownership_precedence_fail.hum"
        );
        let (authority_program, authority_diagnostics, authority_source_occurrences) =
            program_diagnostics_and_authority(
                "fixtures/diagnostics/session_ap_authority_ownership_precedence_fail.hum",
                authority_source,
                true,
            );
        let authority_set = crate::ownership_check::diagnostic_occurrence_set_from_source(
            &authority_program,
            &authority_diagnostics,
            &authority_source_occurrences,
        )
        .expect("authority source transport");
        let authority_upstream = crate::effect_check::diagnostic_occurrence_set_from_source(
            &authority_program,
            &authority_diagnostics,
            &authority_source_occurrences,
        )
        .expect("authority upstream");
        let authority_relationships = crate::ownership_check::ownership_precedence_relationships(
            &authority_program,
            &authority_upstream,
        );
        assert_eq!(authority_relationships.len(), 1);
        assert!(
            authority_relationships[0]
                .application
                .relationship_route
                .iter()
                .any(|part| part.starts_with("resolver_call_occurrence="))
        );
        assert_eq!(
            crate::ownership_check::consume_ownership_precedence(
                &authority_upstream,
                &authority_relationships,
                1,
            )
            .expect("genuine authority/ownership relationship")
            .len(),
            1
        );
        assert_eq!(
            authority_set
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::MISSING_CALLER_CAPABILITY)
                .count(),
            1
        );
        assert_eq!(
            authority_set
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::USE_AFTER_MOVE)
                .count(),
            0
        );
    }

    #[test]
    fn effect_and_ownership_blockers_keep_one_owner_through_later_gates() {
        let effect_source =
            include_str!("../fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum");
        let (effect_program, effect_diagnostics) = program_and_static_diagnostics(
            "fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum",
            effect_source,
            false,
        );
        let effect_set =
            crate::effect_check::diagnostic_occurrence_set(&effect_program, &effect_diagnostics);
        assert_eq!(
            effect_set
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::MISSING_FAILURE_DECLARATION)
                .count(),
            1
        );
        assert_eq!(
            effect_set
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::USE_AFTER_MOVE)
                .count(),
            0
        );
        let effect_relationships = crate::ownership_check::ownership_precedence_relationships(
            &effect_program,
            &effect_set,
        );
        assert_eq!(effect_relationships.len(), 1);
        assert!(
            effect_relationships[0]
                .application
                .relationship_route
                .iter()
                .any(|part| part.starts_with("resolver_call_occurrence="))
        );
        assert_eq!(
            crate::ownership_check::consume_ownership_precedence(
                &effect_set,
                &effect_relationships,
                1,
            )
            .expect("genuine effect/ownership relationship")
            .len(),
            1
        );

        let (_, _, effect_source_occurrences) = program_diagnostics_and_authority(
            "fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum",
            effect_source,
            false,
        );
        let effect_from_source = crate::effect_check::diagnostic_occurrence_set_from_source(
            &effect_program,
            &effect_diagnostics,
            &effect_source_occurrences,
        )
        .expect("effect occurrence transport");
        let ownership_from_source = crate::ownership_check::diagnostic_occurrence_set_from_source(
            &effect_program,
            &effect_diagnostics,
            &effect_source_occurrences,
        )
        .expect("ownership occurrence transport");
        assert_eq!(
            effect_from_source
                .occurrences()
                .filter(|occurrence| {
                    occurrence.code == DiagnosticCode::MISSING_FAILURE_DECLARATION
                })
                .count(),
            1
        );
        assert_eq!(
            ownership_from_source
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::USE_AFTER_MOVE)
                .count(),
            0
        );
        crate::profile_check::diagnostic_occurrence_set_from_source(
            &effect_program,
            &effect_diagnostics,
            &effect_source_occurrences,
        )
        .expect("profile occurrence transport");

        let ownership_source = include_str!(
            "../fixtures/diagnostics/session_ap_ownership_resource_profile_chain_fail.hum"
        );
        let (ownership_program, ownership_diagnostics) = program_and_static_diagnostics(
            "fixtures/diagnostics/session_ap_ownership_resource_profile_chain_fail.hum",
            ownership_source,
            false,
        );
        let ownership_set = crate::ownership_check::diagnostic_occurrence_set(
            &ownership_program,
            &ownership_diagnostics,
        );
        assert_eq!(
            ownership_set
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::USE_AFTER_MOVE)
                .count(),
            1
        );
        assert_eq!(
            ownership_set
                .occurrences()
                .find(|occurrence| occurrence.code == DiagnosticCode::USE_AFTER_MOVE)
                .and_then(|occurrence| occurrence.cause_reason()),
            Some("value_used_after_move_v0")
        );
        assert_eq!(
            crate::resource_check::diagnostic_occurrence_set(
                &ownership_program,
                &ownership_diagnostics,
            ),
            ownership_set
        );
        assert_eq!(
            crate::profile_check::diagnostic_occurrence_set(
                &ownership_program,
                &ownership_diagnostics,
            ),
            ownership_set
        );
    }

    #[test]
    fn ownership_precedence_requires_exact_resolver_call_occurrences() {
        let authority_source = r#"app tests_ap_same_statement_authority {
  uses:
    stdout.write

  starts with:
    caller

  task finish(consume value: Text) -> UInt {
    does:
      return 1
  }

  task authority_helper(value: Text) -> UInt {
    uses:
      stdout.write

    allocates:
      nothing

    does:
      return 1
  }

  task caller -> Unit {
    allocates:
      callee-defined allocation behavior

    does:
      let value: Text = "owned"
      let first: UInt = finish(consume value)
      let combined: UInt = authority_helper("safe") + finish(consume value)
      return
  }
}
"#;
        let (authority_program, authority_diagnostics, authority_source_occurrences) =
            program_diagnostics_and_authority(
                "same-statement-authority.hum",
                authority_source,
                true,
            );
        let authority = crate::effect_check::diagnostic_occurrence_set_from_source(
            &authority_program,
            &authority_diagnostics,
            &authority_source_occurrences,
        )
        .expect("authority transport");
        assert_eq!(
            authority
                .occurrences()
                .filter(|occurrence| occurrence.code == DiagnosticCode::MISSING_CALLER_CAPABILITY)
                .count(),
            1
        );
        let authority_occurrence = authority
            .occurrences()
            .find(|occurrence| occurrence.code == DiagnosticCode::MISSING_CALLER_CAPABILITY)
            .expect("one authority occurrence");
        assert_eq!(
            authority_occurrence
                .relationship_route()
                .iter()
                .filter(|part| part.starts_with("resolver_call_occurrence="))
                .count(),
            1
        );
        for required in ["resolver_call_owner=", "resolver_call_target="] {
            assert!(
                authority_occurrence
                    .relationship_route()
                    .iter()
                    .any(|part| part.starts_with(required)),
                "missing exact authority route part {required}"
            );
        }
        assert!(
            crate::ownership_check::ownership_precedence_relationships(
                &authority_program,
                &authority,
            )
            .is_empty(),
            "same-statement sibling calls must not manufacture authority precedence"
        );

        let effect_source =
            include_str!("../fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum");
        let (effect_program, effect_diagnostics, effect_source_occurrences) =
            program_diagnostics_and_authority(
                "session_ap_effect_ownership_precedence_fail.hum",
                effect_source,
                false,
            );
        let effect_authority = crate::effect_check::diagnostic_occurrence_set_from_source(
            &effect_program,
            &effect_diagnostics,
            &effect_source_occurrences,
        )
        .expect("effect production transport");
        let effect_occurrence = effect_authority
            .occurrences()
            .find(|occurrence| occurrence.code == DiagnosticCode::MISSING_FAILURE_DECLARATION)
            .expect("production H0907 occurrence");
        assert!(effect_occurrence.resolver_call_occurrence().is_some());

        let effect_relationships = crate::ownership_check::ownership_precedence_relationships(
            &effect_program,
            &effect_authority,
        );
        assert_eq!(
            effect_relationships.len(),
            1,
            "the production shared-call fixture must create one effect/ownership relationship"
        );
        assert_eq!(
            effect_relationships[0]
                .application
                .dominant
                .resolver_call_occurrence,
            effect_relationships[0]
                .application
                .suppressed
                .resolver_call_occurrence,
            "precedence must be sealed by the exact shared resolver occurrence"
        );
    }
}
