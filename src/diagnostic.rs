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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PriorBlockerRef {
    pub(crate) occurrence_id: DiagnosticOccurrenceId,
    pub(crate) cause_key: crate::diagnostic_catalog::DiagnosticCauseKey,
    pub(crate) code: DiagnosticCode,
    pub(crate) semantic_origin: String,
    pub(crate) relationship_route: Vec<String>,
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
}

#[derive(Default)]
pub(crate) struct DiagnosticOccurrenceCollector {
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
    pub(crate) fn typed_failure_identity(
        task_identity: &str,
        statement_index: usize,
        form: &str,
        call_span: &Span,
        callee_span: Option<&Span>,
    ) -> DiagnosticOccurrenceIdentity {
        let semantic_origin = crate::node_id::span(
            "typed_failure_statement",
            call_span,
            &format!("{task_identity}:statement-{statement_index}:{form}"),
        );
        let mut relationship_route = vec![format!(
            "typed_failure_relationship:caller={task_identity}:statement={statement_index}:form={form}"
        )];
        if let Some(callee_span) = callee_span {
            relationship_route.push(format!(
                "semantic_site={}",
                crate::node_id::span("diagnostic-semantic-site", callee_span, "definition")
            ));
        }
        relationship_route.push(format!(
            "semantic_site={}",
            crate::node_id::span("diagnostic-semantic-site", call_span, "expression")
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
            diagnostic_seal: diagnostic.clone(),
            diagnostic,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), DiagnosticInvariantError> {
        let Some(cause) = crate::diagnostic_catalog::diagnostic_cause(
            self.code,
            self.cause_reason()
                .ok_or(DiagnosticInvariantError::UnknownCause)?,
        ) else {
            return Err(DiagnosticInvariantError::UnknownCause);
        };
        if self.cause_key != cause.key {
            return Err(DiagnosticInvariantError::UnknownCause);
        }
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
        }
    }

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
    #[cfg(test)]
    pub(crate) fn owning_stage(&self) -> &'static str {
        self.owning_stage
    }
    pub(crate) fn semantic_origin(&self) -> &str {
        &self.identity.semantic_origin
    }
    pub(crate) fn relationship_route(&self) -> &[String] {
        &self.identity.relationship_route
    }
    pub(crate) fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
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
        {
            return Err(DiagnosticInvariantError::PriorBlockerMismatch);
        }
        Ok(())
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
#[cfg(test)]
mod tests {
    use super::{
        Diagnostic, DiagnosticCode, DiagnosticInvariantError, DiagnosticOccurrence,
        DiagnosticOccurrenceCollector, Span,
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
            &span,
            Some(&Span::new("fixture.hum", 3, 1)),
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
            Err(DiagnosticInvariantError::UnknownCause)
        );

        let mut wrong_code = baseline.clone();
        wrong_code.code = DiagnosticCode::INCOMPATIBLE_FAILURE_PROPAGATION;
        assert_eq!(
            wrong_code.validate(),
            Err(DiagnosticInvariantError::UnknownCause)
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
}
