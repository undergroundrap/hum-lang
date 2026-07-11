use std::collections::{BTreeMap, BTreeSet};

use crate::corpus::{
    Corpus, CorpusVariant, ExpectedFact, Polarity, candidate_vocabulary_in,
    expected_allocation_domains, expected_capture_facts, expected_policy_facts,
    expected_resource_domains,
};
use crate::cost::{
    AnalysisCost, AnalysisTrace, DiagnosticCost, ImplementationCost, measure_diagnostic,
};
use crate::inventory::candidate_inventory;
use crate::normalize::{AlphaNormalizer, normalize_identity_path, push_field};

pub const RESULT_FIELDS: [&str; 21] = [
    "candidate_id",
    "case_id",
    "variant_id",
    "candidate_native_result",
    "candidate_native_evidence",
    "neutral_normalized_summary",
    "status",
    "required_source_annotations",
    "inferred_requirement_facts",
    "inferred_capture_facts",
    "allocation_facts",
    "resource_facts",
    "added_machinery",
    "primary_reason",
    "primary_blame_site",
    "related_blame_sites",
    "repair_direction",
    "implementation_cost",
    "analysis_cost",
    "diagnostic_cost",
    "missing_evidence",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateStatus {
    Accepted,
    Rejected,
    Unsupported,
    Incomplete,
}

impl CandidateStatus {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "accepted" => Some(Self::Accepted),
            "rejected" => Some(Self::Rejected),
            "unsupported" => Some(Self::Unsupported),
            "incomplete" => Some(Self::Incomplete),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Unsupported => "unsupported",
            Self::Incomplete => "incomplete",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StableFact {
    pub id: String,
    pub attributes: BTreeMap<String, String>,
    pub route: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateResult {
    pub candidate_id: String,
    pub case_id: String,
    pub variant_id: String,
    pub candidate_native_result: String,
    pub candidate_native_evidence: BTreeMap<String, String>,
    pub neutral_normalized_summary: Vec<String>,
    pub status: CandidateStatus,
    pub required_source_annotations: Vec<StableFact>,
    pub inferred_requirement_facts: Vec<StableFact>,
    pub inferred_capture_facts: Vec<StableFact>,
    pub allocation_facts: Vec<StableFact>,
    pub resource_facts: Vec<StableFact>,
    pub added_machinery: Vec<StableFact>,
    pub primary_reason: String,
    pub primary_blame_site: String,
    pub related_blame_sites: Vec<String>,
    pub repair_direction: String,
    pub implementation_cost: ImplementationCost,
    pub analysis_cost: AnalysisCost,
    pub diagnostic_cost: DiagnosticCost,
    pub missing_evidence: Vec<String>,
}

fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(';').map(str::to_owned).collect()
    }
}

fn parse_map(value: &str) -> Option<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    if value.is_empty() {
        return Some(map);
    }
    for entry in value.split(';') {
        let (key, value) = entry.split_once('=')?;
        if key.is_empty() || map.insert(key.to_owned(), value.to_owned()).is_some() {
            return None;
        }
    }
    Some(map)
}

fn parse_facts(value: &str) -> Option<Vec<StableFact>> {
    if value.is_empty() {
        return Some(Vec::new());
    }
    value
        .split(';')
        .map(|entry| {
            let mut parts = entry.split(',');
            let id = parts.next()?.to_owned();
            let mut attributes = BTreeMap::new();
            let mut route = Vec::new();
            for part in parts {
                let (key, value) = part.split_once('=')?;
                if key == "route" {
                    route = value.split('>').map(str::to_owned).collect();
                } else if attributes
                    .insert(key.to_owned(), value.to_owned())
                    .is_some()
                {
                    return None;
                }
            }
            if id.is_empty() || route.is_empty() {
                return None;
            }
            Some(StableFact {
                id,
                attributes,
                route,
            })
        })
        .collect()
}

fn parse_usizes(value: &str, count: usize) -> Option<Vec<usize>> {
    let values: Option<Vec<_>> = value.split(',').map(|part| part.parse().ok()).collect();
    values.filter(|values| values.len() == count)
}

impl CandidateResult {
    pub fn from_top_level_fields(fields: Vec<(String, String)>) -> Result<Self, String> {
        let allowed: BTreeSet<_> = RESULT_FIELDS.into_iter().collect();
        let mut map = BTreeMap::new();
        for (name, value) in fields {
            if !allowed.contains(name.as_str()) {
                return Err(format!("unknown top-level candidate result field {name}"));
            }
            if map.insert(name.clone(), value).is_some() {
                return Err(format!("duplicate top-level candidate result field {name}"));
            }
        }
        let mut missing = Vec::new();
        for name in RESULT_FIELDS {
            if !map.contains_key(name) {
                missing.push(format!("missing_field:{name}"));
            }
        }
        let value = |name: &str| map.get(name).cloned().unwrap_or_default();
        let mut invalid = Vec::new();
        let status = CandidateStatus::parse(&value("status")).unwrap_or_else(|| {
            invalid.push("invalid_field:status".to_owned());
            CandidateStatus::Incomplete
        });
        let evidence = parse_map(&value("candidate_native_evidence")).unwrap_or_else(|| {
            invalid.push("invalid_field:candidate_native_evidence".to_owned());
            BTreeMap::new()
        });
        let mut facts = |name: &str| {
            parse_facts(&value(name)).unwrap_or_else(|| {
                invalid.push(format!("invalid_field:{name}"));
                Vec::new()
            })
        };
        let required_source_annotations = facts("required_source_annotations");
        let inferred_requirement_facts = facts("inferred_requirement_facts");
        let inferred_capture_facts = facts("inferred_capture_facts");
        let allocation_facts = facts("allocation_facts");
        let resource_facts = facts("resource_facts");
        let added_machinery = facts("added_machinery");
        let implementation = parse_usizes(&value("implementation_cost"), 2).unwrap_or_else(|| {
            invalid.push("invalid_field:implementation_cost".to_owned());
            vec![0; 2]
        });
        let analysis = parse_usizes(&value("analysis_cost"), 5).unwrap_or_else(|| {
            invalid.push("invalid_field:analysis_cost".to_owned());
            vec![0; 5]
        });
        let diagnostic = value("diagnostic_cost");
        let diagnostic_parts: Vec<_> = diagnostic.split(',').collect();
        let diagnostic_cost = if diagnostic_parts.len() == 6 {
            match (
                diagnostic_parts[0].parse(),
                diagnostic_parts[1].parse(),
                diagnostic_parts[2].parse(),
                diagnostic_parts[3].parse(),
                diagnostic_parts[4].parse(),
                diagnostic_parts[5].parse(),
            ) {
                (Ok(a), Ok(b), Ok(c), Ok(d), Ok(e), Ok(f)) => DiagnosticCost {
                    primary_diagnostic_count: a,
                    required_site_count: b,
                    covered_required_site_count: c,
                    rendered_utf8_bytes: d,
                    candidate_native_term_count: e,
                    has_model_neutral_repair: f,
                },
                _ => {
                    invalid.push("invalid_field:diagnostic_cost".to_owned());
                    DiagnosticCost::default()
                }
            }
        } else {
            invalid.push("invalid_field:diagnostic_cost".to_owned());
            DiagnosticCost::default()
        };
        let mut missing_evidence = split_list(&value("missing_evidence"));
        missing_evidence.extend(missing);
        missing_evidence.extend(invalid);
        missing_evidence.sort();
        missing_evidence.dedup();
        let final_status = if missing_evidence.is_empty() {
            status
        } else {
            CandidateStatus::Incomplete
        };
        Ok(Self {
            candidate_id: value("candidate_id"),
            case_id: value("case_id"),
            variant_id: value("variant_id"),
            candidate_native_result: value("candidate_native_result"),
            candidate_native_evidence: evidence,
            neutral_normalized_summary: split_list(&value("neutral_normalized_summary")),
            status: final_status,
            required_source_annotations,
            inferred_requirement_facts,
            inferred_capture_facts,
            allocation_facts,
            resource_facts,
            added_machinery,
            primary_reason: value("primary_reason"),
            primary_blame_site: value("primary_blame_site"),
            related_blame_sites: split_list(&value("related_blame_sites")),
            repair_direction: value("repair_direction"),
            implementation_cost: ImplementationCost {
                nonblank_noncomment_lines: implementation[0],
                dependency_count: implementation[1],
            },
            analysis_cost: AnalysisCost {
                visited_corpus_nodes: analysis[0],
                generated_facts: analysis[1],
                generated_constraints: analysis[2],
                normalization_steps: analysis[3],
                maximum_live_items: analysis[4],
            },
            diagnostic_cost,
            missing_evidence,
        })
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut output = String::new();
        let mut alpha = AlphaNormalizer::default();
        push_field(&mut output, "candidate_id", &self.candidate_id);
        push_field(&mut output, "case_id", &self.case_id);
        push_field(&mut output, "variant_id", &self.variant_id);
        push_field(
            &mut output,
            "candidate_native_result",
            &alpha.normalize(&self.candidate_native_result),
        );
        let native_evidence = self
            .candidate_native_evidence
            .iter()
            .map(|(key, value)| format!("{}={}", alpha.normalize(key), alpha.normalize(value)))
            .collect::<Vec<_>>()
            .join(";");
        push_field(&mut output, "candidate_native_evidence", &native_evidence);
        let mut summary = self.neutral_normalized_summary.clone();
        summary.sort();
        push_field(
            &mut output,
            "neutral_normalized_summary",
            &summary.join(";"),
        );
        push_field(&mut output, "status", self.status.as_str());
        for (name, facts) in [
            (
                "required_source_annotations",
                &self.required_source_annotations,
            ),
            (
                "inferred_requirement_facts",
                &self.inferred_requirement_facts,
            ),
            ("inferred_capture_facts", &self.inferred_capture_facts),
            ("allocation_facts", &self.allocation_facts),
            ("resource_facts", &self.resource_facts),
            ("added_machinery", &self.added_machinery),
        ] {
            let mut facts = facts.clone();
            facts.sort_by(|left, right| left.id.cmp(&right.id));
            let rendered = facts
                .iter()
                .map(|fact| {
                    let attrs = fact
                        .attributes
                        .iter()
                        .map(|(key, value)| {
                            let value = if key.ends_with("_identity_path") {
                                normalize_identity_path(value)
                            } else {
                                value.clone()
                            };
                            format!("{key}={value}")
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("{}[{}]@{}", fact.id, attrs, fact.route.join(">"))
                })
                .collect::<Vec<_>>()
                .join(";");
            push_field(&mut output, name, &rendered);
        }
        push_field(&mut output, "primary_reason", &self.primary_reason);
        push_field(&mut output, "primary_blame_site", &self.primary_blame_site);
        let mut related = self.related_blame_sites.clone();
        related.sort();
        push_field(&mut output, "related_blame_sites", &related.join(";"));
        push_field(&mut output, "repair_direction", &self.repair_direction);
        push_field(
            &mut output,
            "implementation_cost",
            &format!(
                "{},{}",
                self.implementation_cost.nonblank_noncomment_lines,
                self.implementation_cost.dependency_count
            ),
        );
        push_field(
            &mut output,
            "analysis_cost",
            &format!(
                "{},{},{},{},{}",
                self.analysis_cost.visited_corpus_nodes,
                self.analysis_cost.generated_facts,
                self.analysis_cost.generated_constraints,
                self.analysis_cost.normalization_steps,
                self.analysis_cost.maximum_live_items
            ),
        );
        push_field(
            &mut output,
            "diagnostic_cost",
            &format!(
                "{},{},{},{},{},{}",
                self.diagnostic_cost.primary_diagnostic_count,
                self.diagnostic_cost.required_site_count,
                self.diagnostic_cost.covered_required_site_count,
                self.diagnostic_cost.rendered_utf8_bytes,
                self.diagnostic_cost.candidate_native_term_count,
                self.diagnostic_cost.has_model_neutral_repair
            ),
        );
        let mut missing = self.missing_evidence.clone();
        missing.sort();
        push_field(&mut output, "missing_evidence", &missing.join(";"));
        output.into_bytes()
    }
}

fn variant<'a>(corpus: &'a Corpus, result: &CandidateResult) -> Option<&'a CorpusVariant> {
    corpus.variants.iter().find(|variant| {
        variant.case_id == result.case_id && variant.variant_id == result.variant_id
    })
}

fn require_exact_fact_attributes(
    missing: &mut Vec<String>,
    field: &str,
    facts: &[StableFact],
    required: &[&str],
) {
    let expected: BTreeSet<_> = required.iter().copied().collect();
    for fact in facts {
        for key in required {
            if !fact.attributes.contains_key(*key) {
                missing.push(format!("missing_fact_attribute:{field}:{}:{key}", fact.id));
            }
        }
        for key in fact.attributes.keys() {
            if !expected.contains(key.as_str()) {
                missing.push(format!(
                    "unexpected_fact_attribute:{field}:{}:{key}",
                    fact.id
                ));
            }
        }
    }
}

pub fn validate_result(corpus: &Corpus, result: &mut CandidateResult) {
    let mut missing = result.missing_evidence.clone();
    let required_strings = [
        ("candidate_id", &result.candidate_id),
        ("case_id", &result.case_id),
        ("variant_id", &result.variant_id),
        ("candidate_native_result", &result.candidate_native_result),
        ("primary_reason", &result.primary_reason),
        ("primary_blame_site", &result.primary_blame_site),
        ("repair_direction", &result.repair_direction),
    ];
    for (name, value) in required_strings {
        if value.trim().is_empty() {
            missing.push(format!("empty_field:{name}"));
        }
    }
    for (name, is_empty) in [
        (
            "candidate_native_evidence",
            result.candidate_native_evidence.is_empty(),
        ),
        (
            "neutral_normalized_summary",
            result.neutral_normalized_summary.is_empty(),
        ),
        (
            "required_source_annotations",
            result.required_source_annotations.is_empty(),
        ),
        (
            "inferred_requirement_facts",
            result.inferred_requirement_facts.is_empty(),
        ),
        (
            "inferred_capture_facts",
            result.inferred_capture_facts.is_empty(),
        ),
        ("allocation_facts", result.allocation_facts.is_empty()),
        ("resource_facts", result.resource_facts.is_empty()),
        ("added_machinery", result.added_machinery.is_empty()),
        ("related_blame_sites", result.related_blame_sites.is_empty()),
    ] {
        if is_empty {
            missing.push(format!("empty_field:{name}"));
        }
    }
    require_exact_fact_attributes(
        &mut missing,
        "required_source_annotations",
        &result.required_source_annotations,
        &["site", "purpose", "mode"],
    );
    require_exact_fact_attributes(
        &mut missing,
        "inferred_requirement_facts",
        &result.inferred_requirement_facts,
        &["origin", "disposition", "callable", "domain", "policy_role"],
    );
    require_exact_fact_attributes(
        &mut missing,
        "inferred_capture_facts",
        &result.inferred_capture_facts,
        &[
            "origin",
            "disposition",
            "callable",
            "domain",
            "capture_kind",
        ],
    );
    for (name, facts) in [
        ("allocation_facts", &result.allocation_facts),
        ("resource_facts", &result.resource_facts),
    ] {
        require_exact_fact_attributes(
            &mut missing,
            name,
            facts,
            &["kind", "trigger", "lifetime", "owner", "evidence", "domain"],
        );
    }
    require_exact_fact_attributes(
        &mut missing,
        "added_machinery",
        &result.added_machinery,
        &["layer", "state", "creditable", "exercised", "restructuring"],
    );
    let allowed_layers = ["language", "checker", "runtime", "ownership", "tooling"];
    let allowed_states = ["implemented", "unimplemented", "not_applicable"];
    let allowed_booleans = ["true", "false"];
    let mut machinery_ids = BTreeSet::new();
    for machinery in &result.added_machinery {
        if !machinery_ids.insert(machinery.id.as_str()) {
            missing.push("duplicate_machinery_id".to_owned());
        }
        let layer = machinery.attributes.get("layer").map(String::as_str);
        let stable_implemented_id = layer.is_some_and(|layer| {
            machinery
                .id
                .strip_prefix(&format!("machinery.{layer}."))
                .is_some_and(|suffix| {
                    !suffix.is_empty()
                        && suffix.bytes().all(|byte| {
                            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                        })
                        && !["generic", "placeholder", "todo", "unknown"].contains(&suffix)
                })
        });
        let stable_explicit_none_id = machinery.id == "machinery.explicit_none";
        if !stable_explicit_none_id && !stable_implemented_id {
            missing.push(format!("invalid_machinery_id:{}", machinery.id));
        }
        if machinery
            .attributes
            .get("layer")
            .is_some_and(|layer| !allowed_layers.contains(&layer.as_str()))
        {
            missing.push(format!("invalid_machinery_layer:{}", machinery.id));
        }
        if machinery
            .attributes
            .get("state")
            .is_none_or(|v| !allowed_states.contains(&v.as_str()))
        {
            missing.push(format!("invalid_machinery_state:{}", machinery.id));
        }
        for key in ["creditable", "exercised", "restructuring"] {
            if machinery
                .attributes
                .get(key)
                .is_none_or(|v| !allowed_booleans.contains(&v.as_str()))
            {
                missing.push(format!("invalid_machinery_boolean:{}:{key}", machinery.id));
            }
        }
        if stable_explicit_none_id
            && (machinery
                .attributes
                .get("creditable")
                .is_none_or(|v| v != "false")
                || machinery
                    .attributes
                    .get("state")
                    .is_none_or(|v| v != "not_applicable")
                || machinery
                    .attributes
                    .get("exercised")
                    .is_none_or(|v| v != "false")
                || machinery
                    .attributes
                    .get("restructuring")
                    .is_none_or(|v| v != "false"))
        {
            missing.push(format!("invalid_explicit_none_machinery:{}", machinery.id));
        }
        if !stable_explicit_none_id
            && machinery
                .attributes
                .get("state")
                .is_some_and(|state| state == "not_applicable")
        {
            missing.push(format!("invalid_not_applicable_machinery:{}", machinery.id));
        }
        if machinery
            .attributes
            .get("state")
            .is_some_and(|state| state != "implemented")
            && machinery
                .attributes
                .get("creditable")
                .is_some_and(|creditable| creditable == "true")
        {
            missing.push(format!("unimplemented_machinery_credited:{}", machinery.id));
        }
        if machinery
            .attributes
            .get("exercised")
            .is_some_and(|v| v != "true")
            && machinery
                .attributes
                .get("creditable")
                .is_some_and(|v| v == "true")
        {
            missing.push(format!("unexercised_machinery_credited:{}", machinery.id));
        }
        if machinery
            .attributes
            .get("restructuring")
            .is_some_and(|v| v == "true")
            && machinery
                .attributes
                .get("creditable")
                .is_some_and(|v| v == "true")
        {
            missing.push(format!("restructuring_credited:{}", machinery.id));
        }
    }
    for (field, facts) in [
        (
            "required_source_annotations",
            &result.required_source_annotations,
        ),
        (
            "inferred_requirement_facts",
            &result.inferred_requirement_facts,
        ),
        ("inferred_capture_facts", &result.inferred_capture_facts),
        ("allocation_facts", &result.allocation_facts),
        ("resource_facts", &result.resource_facts),
        ("added_machinery", &result.added_machinery),
    ] {
        for fact in facts {
            for text in std::iter::once(fact.id.as_str())
                .chain(fact.attributes.keys().map(String::as_str))
                .chain(fact.attributes.values().map(String::as_str))
                .chain(fact.route.iter().map(String::as_str))
            {
                if let Some(word) = candidate_vocabulary_in(text) {
                    missing.push(format!(
                        "candidate_vocabulary_in_structured_fact:{field}:{}:{word}",
                        fact.id
                    ));
                }
            }
        }
    }
    let allowed_summary_terms = [
        "callable_input",
        "callable_result",
        "propagated_requirement",
        "handled_requirement",
        "stored_callable",
        "returned_callable",
        "capture",
        "escape",
        "exact_authority_requirement",
        "ownership_transfer",
        "resource_lifetime",
    ];
    for summary in &result.neutral_normalized_summary {
        let term = summary
            .split_once(':')
            .map_or(summary.as_str(), |pair| pair.0);
        if !allowed_summary_terms.contains(&term) {
            missing.push(format!("invalid_neutral_summary_term:{term}"));
        }
        if let Some(word) = candidate_vocabulary_in(summary) {
            missing.push(format!("candidate_vocabulary_in_neutral_summary:{word}"));
        }
    }
    for (field, text) in [
        ("primary_reason", result.primary_reason.as_str()),
        ("primary_blame_site", result.primary_blame_site.as_str()),
        ("repair_direction", result.repair_direction.as_str()),
    ] {
        if let Some(word) = candidate_vocabulary_in(text) {
            missing.push(format!("candidate_vocabulary_in_{field}:{word}"));
        }
    }
    if result.implementation_cost.nonblank_noncomment_lines == 0 {
        missing.push("absent_cost:implementation_cost".to_owned());
    }
    if result.analysis_cost.visited_corpus_nodes == 0
        || result.analysis_cost.normalization_steps == 0
        || result.analysis_cost.maximum_live_items == 0
    {
        missing.push("absent_cost:analysis_cost".to_owned());
    }
    let Some(expected) = variant(corpus, result) else {
        missing.push("unknown_case_or_variant".to_owned());
        result.missing_evidence = missing;
        result.status = CandidateStatus::Incomplete;
        return;
    };
    let disposition = match expected.callable_disposition.as_str() {
        "stored_callable" => "stored_callable:required".to_owned(),
        "returned_callable" => "returned_callable:required".to_owned(),
        other => format!("callable_input:disposition={other}"),
    };
    let expected_summaries = [
        format!("callable_input:shape={}", expected.callable_inputs),
        format!("callable_result:shape={}", expected.callable_result),
        format!(
            "callable_result:observation={}",
            expected.expected_observation
        ),
        format!("ownership_transfer:{}", expected.ownership_transfer),
        format!("resource_lifetime:{}", expected.registration_lifetime),
        disposition,
    ];
    let actual_summaries: BTreeSet<_> = result.neutral_normalized_summary.iter().collect();
    let frozen_summaries: BTreeSet<_> = expected_summaries.iter().collect();
    if result.neutral_normalized_summary.len() != expected_summaries.len()
        || actual_summaries != frozen_summaries
    {
        missing.push("structured_relationship_mismatch:summary".to_owned());
    }
    let allowed_annotation_purposes = [
        "callable_relationship",
        "requirement_visibility",
        "capture_visibility",
        "ownership_transfer",
        "allocation_visibility",
        "resource_lifetime",
    ];
    let allowed_annotation_modes = ["required", "inferred"];
    let available_sites: BTreeSet<_> = expected
        .all_sites
        .iter()
        .map(|site| site.id.as_str())
        .collect();
    let mut annotation_ids = BTreeSet::new();
    for annotation in &result.required_source_annotations {
        let site = annotation.attributes.get("site").map(String::as_str);
        let purpose = annotation.attributes.get("purpose").map(String::as_str);
        let mode = annotation.attributes.get("mode").map(String::as_str);
        if !annotation_ids.insert(annotation.id.as_str())
            || site.is_none_or(|site| !available_sites.contains(site))
            || purpose.is_none_or(|purpose| !allowed_annotation_purposes.contains(&purpose))
            || mode.is_none_or(|mode| !allowed_annotation_modes.contains(&mode))
            || annotation.id
                != format!(
                    "annotation.{}.{}.{}",
                    site.unwrap_or(""),
                    purpose.unwrap_or(""),
                    mode.unwrap_or("")
                )
            || annotation.route != expected.call_sites
        {
            missing.push("structured_relationship_mismatch:source_annotation".to_owned());
        }
    }
    let structured = |facts: &[StableFact], kind_key: &str| -> BTreeSet<ExpectedFact> {
        facts
            .iter()
            .filter_map(|f| {
                Some(ExpectedFact {
                    id: f.id.clone(),
                    domain: f.attributes.get("domain")?.clone(),
                    origin: f.attributes.get("origin")?.clone(),
                    disposition: f.attributes.get("disposition")?.clone(),
                    callable: f.attributes.get("callable")?.clone(),
                    route: f.route.clone(),
                    relationship_kind: f.attributes.get(kind_key)?.clone(),
                })
            })
            .collect()
    };
    let expected_policy = expected_policy_facts(expected);
    if result.inferred_requirement_facts.len() != expected_policy.len()
        || structured(&result.inferred_requirement_facts, "policy_role")
            != expected_policy.into_iter().collect()
    {
        missing.push("structured_relationship_mismatch:policy".to_owned());
    }
    let expected_capture = expected_capture_facts(expected);
    if result.inferred_capture_facts.len() != expected_capture.len()
        || structured(&result.inferred_capture_facts, "capture_kind")
            != expected_capture.into_iter().collect()
    {
        missing.push("structured_relationship_mismatch:capture".to_owned());
    }
    let fact_domains = |facts: &[StableFact]| -> BTreeSet<String> {
        facts
            .iter()
            .filter_map(|f| f.attributes.get("domain").cloned())
            .collect()
    };
    if fact_domains(&result.inferred_capture_facts)
        != expected.captured_facts.iter().cloned().collect()
    {
        missing.push("relationship_domain_mismatch:captures".to_owned());
    }
    let expected_allocations: BTreeSet<_> = expected_allocation_domains(expected)
        .into_iter()
        .map(str::to_owned)
        .collect();
    if result.allocation_facts.len() != expected_allocations.len()
        || fact_domains(&result.allocation_facts) != expected_allocations
    {
        missing.push("relationship_domain_mismatch:allocation".to_owned());
    }
    let expected_resources: BTreeSet<_> = expected_resource_domains(expected).into_iter().collect();
    if result.resource_facts.len() != expected_resources.len()
        || fact_domains(&result.resource_facts) != expected_resources
    {
        missing.push("relationship_domain_mismatch:resource".to_owned());
    }
    for (name, facts) in [
        ("allocation", &result.allocation_facts),
        ("resource", &result.resource_facts),
    ] {
        let mut ids = BTreeSet::new();
        for fact in facts {
            let domain = fact.attributes.get("domain");
            let expected_evidence = if domain.is_some_and(|d| d == "none_explicit") {
                "explicit"
            } else if name == "allocation" {
                "prototype_assumed"
            } else {
                "explicit"
            };
            if !ids.insert(fact.id.as_str())
                || fact.id != format!("{name}.{}", domain.map_or("", String::as_str))
                || fact.attributes.get("kind") != domain
                || fact.attributes.get("evidence").map(String::as_str) != Some(expected_evidence)
                || fact.attributes.get("trigger") != expected.call_sites.first()
                || fact.attributes.get("lifetime") != Some(&expected.registration_lifetime)
                || fact.attributes.get("owner") != Some(&format!("{}.callable", expected.case_id))
                || fact.route != expected.call_sites
            {
                missing.push(format!("structured_relationship_mismatch:{name}"));
            }
        }
    }
    for machinery in &result.added_machinery {
        let expected_route = if machinery.id == "machinery.explicit_none" {
            vec!["harness".to_owned(), "explicit_none".to_owned()]
        } else {
            expected.call_sites.clone()
        };
        if machinery.route != expected_route {
            missing.push("structured_relationship_mismatch:machinery_route".to_owned());
        }
    }
    if result.status == CandidateStatus::Rejected {
        if result.primary_reason != expected.primary_reason {
            missing.push("wrong_primary_reason".to_owned());
        }
        let mut covered = BTreeSet::new();
        covered.insert(result.primary_blame_site.as_str());
        covered.extend(result.related_blame_sites.iter().map(String::as_str));
        if expected
            .required_sites
            .iter()
            .any(|site| !covered.contains(site.as_str()))
        {
            missing.push("incomplete_required_blame_sites".to_owned());
        }
        if result.diagnostic_cost.primary_diagnostic_count != 1
            || result.diagnostic_cost.covered_required_site_count
                != result.diagnostic_cost.required_site_count
            || !result.diagnostic_cost.has_model_neutral_repair
        {
            missing.push("invalid_diagnostic_cost".to_owned());
        }
    }
    if expected.polarity == Polarity::Misuse && result.status == CandidateStatus::Accepted {
        missing.push("misuse_accepted".to_owned());
    }
    missing.sort();
    missing.dedup();
    result.missing_evidence = missing;
    if !result.missing_evidence.is_empty() {
        result.status = CandidateStatus::Incomplete;
    }
}

fn validate_run_internal(
    corpus: &Corpus,
    results: &mut [CandidateResult],
) -> Result<Vec<u8>, Vec<String>> {
    let expected: BTreeSet<_> = corpus
        .variants
        .iter()
        .map(|variant| (variant.case_id.as_str(), variant.variant_id.as_str()))
        .collect();
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for result in results.iter_mut() {
        validate_result(corpus, result);
        if !seen.insert((result.case_id.as_str(), result.variant_id.as_str())) {
            errors.push(format!(
                "duplicate result {}:{}",
                result.case_id, result.variant_id
            ));
        }
        if result.status == CandidateStatus::Incomplete {
            errors.push(format!(
                "incomplete result {}:{}: {}",
                result.case_id,
                result.variant_id,
                result.missing_evidence.join(",")
            ));
        }
    }
    for key in expected.difference(&seen) {
        errors.push(format!("missing result {}:{}", key.0, key.1));
    }
    for key in seen.difference(&expected) {
        errors.push(format!("extra result {}:{}", key.0, key.1));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    results.sort_by(|left, right| {
        (&left.case_id, &left.variant_id).cmp(&(&right.case_id, &right.variant_id))
    });
    Ok(results
        .iter()
        .flat_map(CandidateResult::canonical_bytes)
        .collect())
}

#[derive(Clone, Debug)]
pub struct ResultEvidence {
    pub analysis_trace: AnalysisTrace,
    pub rendered_diagnostic: String,
    pub covered_sites: Vec<String>,
    pub candidate_native_terms: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct HarnessEvidence {
    pub results: BTreeMap<(String, String), ResultEvidence>,
}

pub struct CandidateExecution {
    pub results: Vec<CandidateResult>,
    pub evidence: HarnessEvidence,
}
pub trait CandidateRun {
    fn execute(self, corpus: &Corpus) -> CandidateExecution;
}
pub trait CandidateFactory {
    type Run: CandidateRun;
    fn fresh(&self) -> Self::Run;
}

pub fn validate_run(
    corpus: &Corpus,
    execution: &mut CandidateExecution,
) -> Result<Vec<u8>, Vec<String>> {
    let mut errors = Vec::new();
    let candidate_ids: BTreeSet<_> = execution
        .results
        .iter()
        .map(|r| r.candidate_id.as_str())
        .collect();
    let implementation = if candidate_ids.len() == 1 {
        candidate_inventory(candidate_ids.iter().next().unwrap())
            .map(|inventory| inventory.measure())
    } else {
        None
    };
    if implementation.is_none() {
        errors.push("candidate has no unique harness-owned checked-in inventory".to_owned());
    }
    for result in &mut execution.results {
        let key = (result.case_id.clone(), result.variant_id.clone());
        let Some(evidence) = execution.evidence.results.get(&key) else {
            result
                .missing_evidence
                .push("missing_harness_evidence".to_owned());
            continue;
        };
        if implementation
            .as_ref()
            .is_some_and(|measured| &result.implementation_cost != measured)
        {
            result
                .missing_evidence
                .push("implementation_cost_mismatch".to_owned());
        }
        if !evidence
            .analysis_trace
            .check_reported(&result.analysis_cost)
        {
            result
                .missing_evidence
                .push("analysis_cost_mismatch".to_owned());
        }
        let expected = variant(corpus, result)
            .map(|v| v.required_sites.clone())
            .unwrap_or_default();
        let diagnostic = measure_diagnostic(
            &evidence.rendered_diagnostic,
            &expected,
            &evidence.covered_sites,
            &evidence.candidate_native_terms,
            &result.repair_direction,
        );
        if result.diagnostic_cost != diagnostic {
            result
                .missing_evidence
                .push("diagnostic_cost_mismatch".to_owned());
        }
    }
    if execution.evidence.results.len() != execution.results.len() {
        errors.push("harness evidence/result coverage mismatch".to_owned());
    }
    match validate_run_internal(corpus, &mut execution.results) {
        Ok(bytes) if errors.is_empty() => Ok(bytes),
        Ok(_) => Err(errors),
        Err(mut inner) => {
            errors.append(&mut inner);
            Err(errors)
        }
    }
}

pub fn run_fresh_twice<F: CandidateFactory>(
    corpus: &Corpus,
    factory: &F,
) -> Result<Vec<u8>, Vec<String>> {
    let mut first = factory.fresh().execute(corpus);
    let mut second = factory.fresh().execute(corpus);
    let first_bytes = validate_run(corpus, &mut first)?;
    let second_bytes = validate_run(corpus, &mut second)?;
    if first_bytes == second_bytes {
        Ok(first_bytes)
    } else {
        Err(vec![
            "fresh candidate runs are not byte-identical".to_owned(),
        ])
    }
}
