use std::collections::{BTreeMap, BTreeSet};

pub const CORPUS_TEXT: &str = include_str!("../corpus.txt");
pub const RELATIONSHIP_ORIGINS_TEXT: &str = include_str!("../relationship_origins.txt");
pub const CORPUS_DOCUMENT: &str =
    include_str!("../../../docs/bakeoff/EFFECT_POLYMORPHISM_CORPUS.md");

pub const FIELD_NAMES: [&str; 22] = [
    "case_id",
    "variant_id",
    "polarity",
    "frequency",
    "rationale",
    "behavior",
    "values",
    "callable_inputs",
    "callable_result",
    "call_sites",
    "callable_disposition",
    "latent_operations",
    "captured_facts",
    "ownership_transfer",
    "registration_lifetime",
    "expected_observation",
    "primary_reason",
    "required_sites",
    "all_sites",
    "source_requirements",
    "operator_consent",
    "operation_exercise",
];

const CASE_IDS: [&str; 12] = [
    "effect.pure_map",
    "effect.effectful_map",
    "effect.filter_retain",
    "effect.fold",
    "effect.retry",
    "effect.with_timeout",
    "effect.parallel_map",
    "effect.callback_registry",
    "effect.event_handler_factory",
    "effect.memoizing_wrapper",
    "effect.logging_middleware",
    "effect.linear_capture",
];

pub const FROZEN_VARIANT_IDS: [(&str, &str); 29] = [
    ("effect.pure_map", "positive"),
    ("effect.pure_map", "misuse_callback_shape"),
    ("effect.effectful_map", "positive"),
    ("effect.effectful_map", "misuse_erased_requirement"),
    ("effect.filter_retain", "positive_two_list_odd_filter"),
    ("effect.filter_retain", "positive_retain_delete"),
    ("effect.filter_retain", "misuse_same_list_mutation"),
    ("effect.filter_retain", "misuse_stale_retained_view"),
    ("effect.fold", "positive"),
    ("effect.fold", "misuse_erased_step"),
    ("effect.retry", "positive"),
    ("effect.retry", "misuse_erased_remaining"),
    ("effect.with_timeout", "positive_type_only"),
    ("effect.with_timeout", "misuse_claims_runtime"),
    ("effect.parallel_map", "positive_type_only"),
    ("effect.parallel_map", "misuse_erased_callback"),
    ("effect.callback_registry", "positive"),
    ("effect.callback_registry", "misuse_outlives_state"),
    ("effect.event_handler_factory", "positive"),
    ("effect.event_handler_factory", "misuse_laundered_authority"),
    ("effect.memoizing_wrapper", "positive"),
    ("effect.memoizing_wrapper", "misuse_hidden_cache"),
    ("effect.logging_middleware", "positive"),
    ("effect.logging_middleware", "misuse_erased_wrapped"),
    ("effect.linear_capture", "positive_move"),
    ("effect.linear_capture", "misuse_move_without_transfer"),
    ("effect.linear_capture", "misuse_escape"),
    ("effect.linear_capture", "misuse_double_use"),
    ("effect.linear_capture", "misuse_outlives"),
];

pub const FROZEN_CORPUS_FINGERPRINT: u64 = 9_927_983_142_673_634_928;
const SITE_KINDS: [&str; 8] = [
    "definition",
    "callable",
    "call",
    "capture",
    "resource",
    "registration",
    "return",
    "use",
];

const CANDIDATE_VOCABULARY: [&str; 10] = [
    "koka",
    "flix",
    "effekt",
    "scala",
    "row variable",
    "boolean formula",
    "capture set",
    "handler mechanism",
    "tail variable",
    "subcapture",
];

pub fn candidate_vocabulary_in(text: &str) -> Option<&'static str> {
    let lower = text.to_ascii_lowercase();
    CANDIDATE_VOCABULARY
        .into_iter()
        .find(|forbidden| lower.contains(forbidden))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Polarity {
    Positive,
    Misuse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusSite {
    pub id: String,
    pub kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusVariant {
    pub case_id: String,
    pub variant_id: String,
    pub polarity: Polarity,
    pub frequency: String,
    pub rationale: String,
    pub behavior: String,
    pub values: Vec<String>,
    pub callable_inputs: String,
    pub callable_result: String,
    pub call_sites: Vec<String>,
    pub callable_disposition: String,
    pub latent_operations: Vec<String>,
    pub captured_facts: Vec<String>,
    pub ownership_transfer: String,
    pub registration_lifetime: String,
    pub expected_observation: String,
    pub primary_reason: String,
    pub required_sites: Vec<String>,
    pub all_sites: Vec<CorpusSite>,
    pub source_requirements: Vec<String>,
    pub operator_consent: Vec<String>,
    pub operation_exercise: Vec<String>,
    pub source_requirement_origins: BTreeMap<String, String>,
    pub operator_consent_origins: BTreeMap<String, String>,
    pub operation_exercise_origins: BTreeMap<String, String>,
    pub capture_origins: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Corpus {
    pub variants: Vec<CorpusVariant>,
}

type OriginMap = BTreeMap<String, String>;
type OriginRows = BTreeMap<(String, String), [OriginMap; 4]>;

pub fn corpus_fingerprint(corpus: &Corpus) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in format!("{:?}", corpus.variants).bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn expected_allocation_domains(variant: &CorpusVariant) -> Vec<&'static str> {
    match variant.case_id.as_str() {
        "effect.callback_registry" => vec!["callable_environment", "registry_storage"],
        "effect.event_handler_factory" | "effect.logging_middleware" | "effect.linear_capture" => {
            vec!["callable_environment"]
        }
        "effect.memoizing_wrapper" => vec!["callable_environment", "cache_storage"],
        _ => vec!["none_explicit"],
    }
}

pub fn expected_resource_domains(variant: &CorpusVariant) -> Vec<String> {
    let mut domains: Vec<_> = variant
        .captured_facts
        .iter()
        .filter(|domain| capture_kind(domain) == "resource")
        .cloned()
        .collect();
    if domains.is_empty() {
        domains.push("none_explicit".to_owned());
    }
    domains.sort();
    domains.dedup();
    domains
}

pub fn capture_kind(domain: &str) -> &'static str {
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExpectedFact {
    pub id: String,
    pub domain: String,
    pub origin: String,
    pub disposition: String,
    pub callable: String,
    pub route: Vec<String>,
    pub relationship_kind: String,
}

fn affected_callable(variant: &CorpusVariant) -> String {
    format!("{}.callable", variant.case_id)
}

fn origin_map(spec: &str, line: usize, field: &str) -> Result<OriginMap, String> {
    let mut result = BTreeMap::new();
    for entry in spec.split(';') {
        let (domain, origin) = entry
            .split_once('@')
            .ok_or_else(|| format!("origin line {line} has malformed {field} entry {entry}"))?;
        if domain.is_empty()
            || origin.is_empty()
            || result
                .insert(domain.to_owned(), origin.to_owned())
                .is_some()
        {
            return Err(format!(
                "origin line {line} has invalid or duplicate {field} domain {domain}"
            ));
        }
    }
    Ok(result)
}

fn frozen_origin_rows() -> Result<OriginRows, Vec<String>> {
    let mut errors = Vec::new();
    let mut rows = BTreeMap::new();
    let mut identities = Vec::new();
    for (index, raw) in RELATIONSHIP_ORIGINS_TEXT.lines().enumerate() {
        let line = index + 1;
        let raw = raw.trim_end_matches('\r');
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = raw.split('|').collect();
        if fields.len() != 6 {
            errors.push(format!(
                "origin line {line} has {} fields, expected 6",
                fields.len()
            ));
            continue;
        }
        let key = (fields[0].to_owned(), fields[1].to_owned());
        identities.push(key.clone());
        let parsed = [
            origin_map(fields[2], line, "source_requirement_origins"),
            origin_map(fields[3], line, "operator_consent_origins"),
            origin_map(fields[4], line, "operation_exercise_origins"),
            origin_map(fields[5], line, "capture_origins"),
        ];
        if let Some(error) = parsed.iter().find_map(|value| value.as_ref().err()) {
            errors.push(error.clone());
            continue;
        }
        let maps = parsed.map(Result::unwrap);
        if rows.insert(key.clone(), maps).is_some() {
            errors.push(format!("duplicate origin row {}:{}", key.0, key.1));
        }
    }
    let frozen: Vec<_> = FROZEN_VARIANT_IDS
        .iter()
        .map(|(case, variant)| ((*case).to_owned(), (*variant).to_owned()))
        .collect();
    if identities != frozen {
        errors
            .push("relationship-origin identity/order differs from frozen 29-row exam".to_owned());
    }
    if errors.is_empty() {
        Ok(rows)
    } else {
        Err(errors)
    }
}

pub fn expected_policy_facts(variant: &CorpusVariant) -> Vec<ExpectedFact> {
    let mut facts = Vec::new();
    for (kind, domains, disposition, origins) in [
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
            facts.push(ExpectedFact {
                id: format!("{kind}.{domain}"),
                domain: domain.clone(),
                origin: origins
                    .get(domain)
                    .expect("validated corpus has every policy origin")
                    .clone(),
                disposition: disposition.to_owned(),
                callable: affected_callable(variant),
                route: variant.call_sites.clone(),
                relationship_kind: kind.to_owned(),
            });
        }
    }
    facts
}

pub fn expected_capture_facts(variant: &CorpusVariant) -> Vec<ExpectedFact> {
    variant
        .captured_facts
        .iter()
        .map(|domain| ExpectedFact {
            id: format!("capture.{domain}"),
            domain: domain.clone(),
            origin: variant
                .capture_origins
                .get(domain)
                .cloned()
                .expect("validated corpus has every capture origin"),
            disposition: if domain == "none_explicit" {
                "none_explicit"
            } else {
                "retained"
            }
            .to_owned(),
            callable: affected_callable(variant),
            route: variant.call_sites.clone(),
            relationship_kind: capture_kind(domain).to_owned(),
        })
        .collect()
}

fn list(value: &str) -> Vec<String> {
    value.split(';').map(str::to_owned).collect()
}

fn sites(value: &str, line: usize) -> Result<Vec<CorpusSite>, String> {
    value
        .split(';')
        .map(|entry| {
            let (id, kind) = entry
                .split_once(':')
                .ok_or_else(|| format!("corpus line {line} has malformed site {entry}"))?;
            if id.is_empty() || kind.is_empty() {
                return Err(format!("corpus line {line} has empty site identity"));
            }
            Ok(CorpusSite {
                id: id.to_owned(),
                kind: kind.to_owned(),
            })
        })
        .collect()
}

pub fn parse(text: &str) -> Result<Corpus, Vec<String>> {
    let mut errors = Vec::new();
    let mut variants = Vec::new();
    let origin_rows = frozen_origin_rows()?;
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let raw = raw.trim_end_matches('\r');
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = raw.split('|').collect();
        if fields.len() != FIELD_NAMES.len() {
            errors.push(format!(
                "corpus line {line} has {} fields, expected {}",
                fields.len(),
                FIELD_NAMES.len()
            ));
            continue;
        }
        if let Some((field, _)) = FIELD_NAMES
            .iter()
            .zip(fields.iter())
            .find(|(_, value)| value.is_empty())
        {
            errors.push(format!("corpus line {line} has empty field {field}"));
            continue;
        }
        let polarity = match fields[2] {
            "positive" => Polarity::Positive,
            "misuse" => Polarity::Misuse,
            other => {
                errors.push(format!("corpus line {line} has invalid polarity {other}"));
                continue;
            }
        };
        let parsed_sites = match sites(fields[18], line) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        let Some(origin_maps) = origin_rows.get(&(fields[0].to_owned(), fields[1].to_owned()))
        else {
            errors.push(format!(
                "corpus line {line} has no frozen relationship-origin record"
            ));
            continue;
        };
        variants.push(CorpusVariant {
            case_id: fields[0].to_owned(),
            variant_id: fields[1].to_owned(),
            polarity,
            frequency: fields[3].to_owned(),
            rationale: fields[4].to_owned(),
            behavior: fields[5].to_owned(),
            values: list(fields[6]),
            callable_inputs: fields[7].to_owned(),
            callable_result: fields[8].to_owned(),
            call_sites: list(fields[9]),
            callable_disposition: fields[10].to_owned(),
            latent_operations: list(fields[11]),
            captured_facts: list(fields[12]),
            ownership_transfer: fields[13].to_owned(),
            registration_lifetime: fields[14].to_owned(),
            expected_observation: fields[15].to_owned(),
            primary_reason: fields[16].to_owned(),
            required_sites: list(fields[17]),
            all_sites: parsed_sites,
            source_requirements: list(fields[19]),
            operator_consent: list(fields[20]),
            operation_exercise: list(fields[21]),
            source_requirement_origins: origin_maps[0].clone(),
            operator_consent_origins: origin_maps[1].clone(),
            operation_exercise_origins: origin_maps[2].clone(),
            capture_origins: origin_maps[3].clone(),
        });
    }
    if errors.is_empty() {
        let corpus = Corpus { variants };
        match validate(&corpus, text) {
            Ok(()) => Ok(corpus),
            Err(mut validation) => {
                errors.append(&mut validation);
                Err(errors)
            }
        }
    } else {
        Err(errors)
    }
}

pub fn checked_in() -> Corpus {
    parse(CORPUS_TEXT).expect("checked-in effect corpus must be valid")
}

pub fn validate(corpus: &Corpus, source: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut keys = BTreeSet::new();
    let mut polarities: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for variant in &corpus.variants {
        let key = (variant.case_id.as_str(), variant.variant_id.as_str());
        if !keys.insert(key) {
            errors.push(format!("duplicate corpus variant {}:{}", key.0, key.1));
        }
        if !CASE_IDS.contains(&variant.case_id.as_str()) {
            errors.push(format!("unknown corpus case {}", variant.case_id));
        }
        polarities
            .entry(&variant.case_id)
            .or_default()
            .insert(match variant.polarity {
                Polarity::Positive => "positive",
                Polarity::Misuse => "misuse",
            });
        let available: BTreeSet<_> = variant.all_sites.iter().map(|site| &site.id).collect();
        for required in &variant.required_sites {
            if !available.contains(required) {
                errors.push(format!(
                    "{}:{} requires absent blame site {required}",
                    variant.case_id, variant.variant_id
                ));
            }
        }
        for site in &variant.all_sites {
            if !SITE_KINDS.contains(&site.kind.as_str()) {
                errors.push(format!(
                    "{}:{} has invalid site kind {}",
                    variant.case_id, variant.variant_id, site.kind
                ));
            }
        }
        for (relationship, domains, origins) in [
            (
                "source_requirement",
                &variant.source_requirements,
                &variant.source_requirement_origins,
            ),
            (
                "operator_consent",
                &variant.operator_consent,
                &variant.operator_consent_origins,
            ),
            (
                "operation_exercise",
                &variant.operation_exercise,
                &variant.operation_exercise_origins,
            ),
            ("capture", &variant.captured_facts, &variant.capture_origins),
        ] {
            let expected_domains: BTreeSet<_> = domains.iter().collect();
            let origin_domains: BTreeSet<_> = origins.keys().collect();
            if expected_domains != origin_domains {
                errors.push(format!(
                    "{}:{} has incomplete {relationship} domain-to-origin joins",
                    variant.case_id, variant.variant_id
                ));
            }
            for (domain, origin) in origins {
                if !available.contains(origin) {
                    errors.push(format!(
                        "{}:{} maps {relationship} domain {domain} to absent site {origin}",
                        variant.case_id, variant.variant_id
                    ));
                }
            }
        }
        if variant.polarity == Polarity::Misuse && variant.primary_reason == "none_expected" {
            errors.push(format!(
                "{}:{} misuse has no fundamental reason",
                variant.case_id, variant.variant_id
            ));
        }
    }
    for case_id in CASE_IDS {
        match polarities.get(case_id) {
            Some(kinds) if kinds.contains("positive") && kinds.contains("misuse") => {}
            _ => errors.push(format!("{case_id} lacks a positive or misuse half")),
        }
    }
    let actual_ids: Vec<_> = corpus
        .variants
        .iter()
        .map(|v| (v.case_id.as_str(), v.variant_id.as_str()))
        .collect();
    if actual_ids != FROZEN_VARIANT_IDS {
        errors.push("corpus variant identity/order differs from frozen 29-row exam".to_owned());
    }
    let fingerprint = corpus_fingerprint(corpus);
    if fingerprint != FROZEN_CORPUS_FINGERPRINT {
        errors.push(format!(
            "corpus relationship fingerprint differs from frozen exam: {fingerprint}"
        ));
    }
    if let Some(forbidden) = candidate_vocabulary_in(source) {
        errors.push(format!(
            "neutral corpus contains candidate vocabulary {forbidden}"
        ));
    }
    if let Some(forbidden) = candidate_vocabulary_in(RELATIONSHIP_ORIGINS_TEXT) {
        errors.push(format!(
            "neutral relationship-origin corpus contains candidate vocabulary {forbidden}"
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate_document(corpus: &Corpus) -> Result<(), Vec<String>> {
    let mut missing = Vec::new();
    for variant in &corpus.variants {
        for identity in [
            &variant.case_id,
            &variant.variant_id,
            &variant.primary_reason,
        ] {
            if !CORPUS_DOCUMENT.contains(identity) {
                missing.push(format!("corpus document omits {identity}"));
            }
        }
        for site in &variant.required_sites {
            if !CORPUS_DOCUMENT.contains(site) {
                missing.push(format!("corpus document omits required site {site}"));
            }
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}
