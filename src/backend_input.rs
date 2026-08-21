use crate::ast::{
    CanonicalExpressionKind, Item, ParsedBinaryOperator, ParsedBodyStatement, Program,
};
use crate::diagnostic::Diagnostic;
use crate::profile_check;
use crate::sha256;
use crate::type_check;
use crate::version;
use std::fmt::Write as _;
use std::ops::Range;

pub const BACKEND_INPUT_SCHEMA: &str = "hum.backend_input.v0";

pub(crate) const SEMANTIC_CONTRACT: &str = "hum.canonical_minimal_add_backend_facts.v0";
pub(crate) const TARGET_CONTEXT: &str = "target_independent_checked_i64_v0";
pub(crate) const FEATURE_SET: &str = "canonical_minimal_add_checked_i64_v0";
pub(crate) const SOURCE_PATH: &str = "examples/core/minimal_add.hum";
pub(crate) const MODULE_NAME: &str = "examples.core.minimal_add";
pub(crate) const SOURCE_REVISION_SHA256: &str =
    "sha256:aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6";

pub(crate) const REQUIRED_PASSES: [&str; 14] = [
    "parse",
    "semantic_graph_build",
    "resolve",
    "body_grammar",
    "core_preview",
    "core_lowering",
    "core_verify",
    "type_check",
    "full_type_check",
    "effect_check",
    "ownership_alias_check",
    "allocation_resource_check",
    "contract_evidence_linking_checked_empty_for_exact_item",
    "profile_check",
];

pub(crate) struct CanonicalBackendInputArtifact {
    bytes: Vec<u8>,
    payload_range: Range<usize>,
    artifact_id: String,
}

impl CanonicalBackendInputArtifact {
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn payload(&self) -> &[u8] {
        &self.bytes[self.payload_range.clone()]
    }

    pub(crate) fn artifact_id(&self) -> &str {
        &self.artifact_id
    }
}

struct CanonicalMinimalAddBackendFacts<'report, 'source> {
    program: &'source Program,
    item: &'source Item,
    statement: &'source ParsedBodyStatement,
    profile: profile_check::VerifiedMinimalAddProfile<'report>,
    task_signature: crate::ast::AuthenticatedCanonicalTaskSignature,
    compiler_version: &'static str,
    semantic_contract: &'static str,
    target_context: &'static str,
    required_passes: Vec<(&'static str, bool, usize, usize)>,
    ir_verify_state: &'static str,
    source_slots: [usize; 2],
    checked_empty_sets: Vec<&'static str>,
    failure_edges: Vec<[&'static str; 3]>,
}

impl CanonicalMinimalAddBackendFacts<'_, '_> {
    fn is_complete_with_final_profile_lineage(
        &self,
        final_profile_lineage: &profile_check::VerifiedMinimalAddProfile<'_>,
    ) -> bool {
        let Item::Task(task) = self.item else {
            return false;
        };
        let identity = self.profile.backend_identity();
        let [Some(left), Some(right)] = [identity.operand(0), identity.operand(1)] else {
            return false;
        };
        let semantic_facts_complete = self.compiler_version == version::HUM_VERSION
            && self.semantic_contract == SEMANTIC_CONTRACT
            && self.target_context == TARGET_CONTEXT
            && self.required_passes.len() == REQUIRED_PASSES.len()
            && self.required_passes.iter().zip(REQUIRED_PASSES).all(
                |((actual, passed, selected, context), expected)| {
                    *actual == expected
                        && *passed
                        && *selected == 1
                        && *context == identity.program_identity
                },
            )
            && self.ir_verify_state == "not_implemented"
            && self
                .profile
                .core_prerequisite_names()
                .eq(REQUIRED_PASSES[..7].iter().copied())
            && self.profile.profile_id() == "normal"
            && self.profile.resource().allocation_declaration() == Some("nothing")
            && identity.program_identity == std::ptr::from_ref(self.program).addr()
            && !identity.owner.file.source_revision.is_empty()
            && identity.owner.file.semantic_file_index < self.program.files.len()
            && !identity.owner.file.normalized_path.is_empty()
            && identity.source_module.is_none_or(|value| !value.is_empty())
            && !identity.owner.item_path.is_empty()
            && !identity.source_identities[0].is_empty()
            && identity.owner.item_kind == "task"
            && self.task_signature.matches_lowered_candidate(
                identity.owner.item_kind,
                &task.name,
                &task.span,
                &task.params,
                task.result.as_deref(),
            )
            && task.params.len() == 2
            && task.result.as_deref() == Some("Int")
            && crate::predicate::PredicateAnalysis::build(self.program)
                .facts_for_task(task)
                .next()
                .is_none()
            && crate::graph::evidence_obligations(task).is_empty()
            && identity
                .owner
                .section_slots
                .get(self.source_slots[0])
                .map(AsRef::as_ref)
                == Some("does")
            && identity
                .owner
                .section_slots
                .iter()
                .filter(|name| name.as_ref() == "does")
                .count()
                == 1
            && task
                .body_syntax
                .get(self.source_slots[1])
                .is_some_and(|row| std::ptr::eq(row, self.statement))
            && task
                .body_syntax
                .iter()
                .filter(|row| std::ptr::eq(*row, self.statement))
                .count()
                == 1
            && !identity.source_identities[1].is_empty()
            && !identity.source_identities[2].is_empty()
            && matches!(
                identity.root.kind,
                CanonicalExpressionKind::Binary {
                    operator: ParsedBinaryOperator::Add,
                    ..
                }
            )
            && identity.root.node_id.as_str() != left.1
            && identity.root.node_id.as_str() != right.1
            && left.2 != right.2
            && left.3 != right.3
            && [left.4, right.4] == ["Int", "Int"]
            && [left.5, right.5].iter().all(|span| !span.file.is_empty())
            && identity.checked_type
                == (
                    type_check::CANONICAL_MINIMAL_ADD_TYPE_ID,
                    type_check::CANONICAL_MINIMAL_ADD_TYPE_TEXT,
                )
            && identity.declared_result_compatible == Some(true)
            && self.checked_empty_sets
                == [
                    "effects",
                    "ownership_transfers",
                    "allocations",
                    "contract_predicates",
                    "evidence_obligations",
                    "unsupported_or_weakened",
                    "external_authority",
                ]
            && self.failure_edges == [["signed_64", "checked_add", "runtime_trap_on_overflow"]];
        if !semantic_facts_complete {
            return false;
        }
        let final_lineage_matches = final_profile_lineage.backend_identity().program_identity
            == std::ptr::from_ref(self.program).addr();
        observe_final_profile_lineage_comparison_for_test(final_lineage_matches);
        final_lineage_matches
    }

    #[cfg(test)]
    fn snapshot(&self) -> Vec<String> {
        let identity = self.profile.backend_identity();
        let mut snapshot = self
            .required_passes
            .iter()
            .map(|(name, _, _, _)| (*name).to_string())
            .collect::<Vec<_>>();
        snapshot.extend(
            self.checked_empty_sets
                .iter()
                .map(|state| format!("checked_empty:{state}")),
        );
        snapshot.extend([
            self.semantic_contract.to_string(),
            self.profile.profile_id().to_string(),
            identity.root.node_id.as_str().to_string(),
            identity.source_identities[2].clone(),
            self.failure_edges[0][1].to_string(),
        ]);
        snapshot
    }
}

fn assemble<'report, 'source>(
    program: &'source Program,
    item: &'source Item,
    statement: &'source ParsedBodyStatement,
    profile: profile_check::VerifiedMinimalAddProfile<'report>,
) -> Option<CanonicalMinimalAddBackendFacts<'report, 'source>> {
    let Item::Task(task) = item else {
        return None;
    };
    let identity = profile.backend_identity();
    let does_section_slot = identity
        .owner
        .section_slots
        .iter()
        .position(|name| name.as_ref() == "does")?;
    let operation_slot = task
        .body_syntax
        .iter()
        .position(|candidate| std::ptr::eq(candidate, statement))?;
    let task_signature = program.authenticate_canonical_task_signature(task).ok()?;
    #[allow(unused_mut)]
    let mut facts = CanonicalMinimalAddBackendFacts {
        program,
        item,
        statement,
        profile,
        task_signature,
        compiler_version: version::HUM_VERSION,
        semantic_contract: SEMANTIC_CONTRACT,
        target_context: TARGET_CONTEXT,
        required_passes: REQUIRED_PASSES
            .map(|name| (name, true, 1, std::ptr::from_ref(program).addr()))
            .to_vec(),
        ir_verify_state: "not_implemented",
        source_slots: [does_section_slot, operation_slot],
        checked_empty_sets: vec![
            "effects",
            "ownership_transfers",
            "allocations",
            "contract_predicates",
            "evidence_obligations",
            "unsupported_or_weakened",
            "external_authority",
        ],
        failure_edges: vec![["signed_64", "checked_add", "runtime_trap_on_overflow"]],
    };
    #[cfg(test)]
    corrupt_backend_facts_for_test(&mut facts);
    Some(facts)
}

fn assembled_is_authenticated(
    facts: &CanonicalMinimalAddBackendFacts<'_, '_>,
    final_profile_lineage: &profile_check::VerifiedMinimalAddProfile<'_>,
) -> bool {
    facts.is_complete_with_final_profile_lineage(final_profile_lineage)
}

fn canonical_minimal_add_artifact_for(
    program: &Program,
    diagnostics: &[Diagnostic],
    item: &Item,
    statement: &ParsedBodyStatement,
) -> Option<CanonicalBackendInputArtifact> {
    profile_check::with_profile_for_ir_readiness(program, diagnostics, |profile_access| {
        let profile = profile_access.canonical_minimal_add_for(item, statement)?;
        let facts = assemble(program, item, statement, profile)?;
        assembled_is_authenticated(&facts, &facts.profile)
            .then(|| encode_minimal_add_artifact(&facts))?
    })
}

pub(crate) fn canonical_minimal_add_artifact(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> Option<CanonicalBackendInputArtifact> {
    let [file] = program.files.as_slice() else {
        return None;
    };
    let [item] = file.items.as_slice() else {
        return None;
    };
    let Item::Task(task) = item else {
        return None;
    };
    let [statement] = task.body_syntax.as_slice() else {
        return None;
    };
    canonical_minimal_add_artifact_for(program, diagnostics, item, statement)
}

pub(crate) fn bind_canonical_minimal_add_live_identity(
    request: &mut crate::ir_verify::LiveIdentityRequest<'_>,
    program: &Program,
    diagnostics: &[Diagnostic],
) -> bool {
    let [file] = program.files.as_slice() else {
        return false;
    };
    let [item] = file.items.as_slice() else {
        return false;
    };
    let Item::Task(task) = item else {
        return false;
    };
    let [statement] = task.body_syntax.as_slice() else {
        return false;
    };
    bind_canonical_minimal_add_live_identity_for(request, program, diagnostics, item, statement)
}

fn bind_canonical_minimal_add_live_identity_for(
    request: &mut crate::ir_verify::LiveIdentityRequest<'_>,
    program: &Program,
    diagnostics: &[Diagnostic],
    item: &Item,
    statement: &ParsedBodyStatement,
) -> bool {
    profile_check::with_profile_for_ir_readiness(program, diagnostics, |profile_access| {
        let profile = profile_access.canonical_minimal_add_for(item, statement)?;
        let facts = assemble(program, item, statement, profile)?;
        assembled_is_authenticated(&facts, &facts.profile)
            .then(|| observe_live_identity(request, &facts))
    })
    .is_some()
}

fn observe_live_identity(
    request: &mut crate::ir_verify::LiveIdentityRequest<'_>,
    facts: &CanonicalMinimalAddBackendFacts<'_, '_>,
) {
    let identity = facts.profile.backend_identity();
    let [Some(left), Some(right)] = [identity.operand(0), identity.operand(1)] else {
        return;
    };
    let passes = facts
        .required_passes
        .iter()
        .enumerate()
        .map(|(ordinal, (name, _, _, _))| {
            format!(
                "{name}@file:{}:ordinal:{ordinal}",
                identity.owner.file.semantic_file_index
            )
        })
        .collect::<Vec<_>>();
    request.observe(
        SOURCE_REVISION_SHA256,
        SOURCE_PATH,
        passes,
        "function:0",
        "internal",
        "hum_fn_0",
        [left.0.to_string(), right.0.to_string()],
        [left.2.to_string(), right.2.to_string()],
        [left.3.to_string(), right.3.to_string()],
        ["type:int64", "type:int64"],
        [(left.5.line, left.5.column), (right.5.line, right.5.column)],
        "operation:function:0:block:0:0".to_string(),
        identity.root.node_id.as_str().to_string(),
        identity.source_identities[2].clone(),
        "type:int64",
        (facts.statement.span.line, facts.statement.span.column),
    );
}

#[cfg(test)]
pub(crate) fn bind_canonical_minimal_add_live_identity_for_test(
    request: &mut crate::ir_verify::LiveIdentityRequest<'_>,
    program: &Program,
    diagnostics: &[Diagnostic],
    semantic_file_index: usize,
) -> bool {
    let Some(file) = program.files.get(semantic_file_index) else {
        return false;
    };
    let [item] = file.items.as_slice() else {
        return false;
    };
    let Item::Task(task) = item else {
        return false;
    };
    let [statement] = task.body_syntax.as_slice() else {
        return false;
    };
    bind_canonical_minimal_add_live_identity_for(request, program, diagnostics, item, statement)
}

fn encode_minimal_add_artifact(
    facts: &CanonicalMinimalAddBackendFacts<'_, '_>,
) -> Option<CanonicalBackendInputArtifact> {
    let Item::Task(task) = facts.item else {
        return None;
    };
    let identity = facts.profile.backend_identity();
    let [Some(left), Some(right)] = [identity.operand(0), identity.operand(1)] else {
        return None;
    };
    let file = facts
        .program
        .files
        .get(identity.owner.file.semantic_file_index)?;
    let [item_ordinal] = identity.owner.item_path.as_ref() else {
        return None;
    };
    let Item::Task(bound_task) = file.items.get(*item_ordinal)? else {
        return None;
    };
    let source_revision = identity.owner.file.source_revision.as_ref();
    let source_digest = sha256::digest(source_revision)?;
    let source_digest = sha256::lowercase_hex(&source_digest);
    let module = identity.source_module?;
    let target_context = artifact_target_context_for_test(facts.target_context);

    (identity.owner.file.semantic_file_index == 0
        && identity.owner.file.normalized_path.as_ref() == SOURCE_PATH
        && file.path.replace('\\', "/") == SOURCE_PATH
        && module == MODULE_NAME
        && file.module.as_deref() == Some(MODULE_NAME)
        && *item_ordinal == 0
        && std::ptr::eq(bound_task, task)
        && task.name == "add"
        && task.params.len() == 2
        && task.params[0].name == "a"
        && task.params[1].name == "b"
        && task.params.iter().all(|parameter| parameter.ty == "Int")
        && task.result.as_deref() == Some("Int")
        && task.span.line == 3
        && task.span.column == 1
        && task.params[0].span.line == 3
        && task.params[0].span.column == 10
        && task.params[1].span.line == 3
        && task.params[1].span.column == 18
        && facts.statement.span.line == 8
        && facts.statement.span.column == 5
        && identity.root.range.start.line == 8
        && identity.root.range.start.column == 12
        && facts.source_slots == [1, 0]
        && facts.required_passes.len() == 14
        && format!("sha256:{source_digest}") == SOURCE_REVISION_SHA256)
        .then_some(())?;

    let payload = encode_payload(facts, target_context, &source_digest, left, right);
    let payload_digest = sha256::digest(payload.as_bytes())?;
    let artifact_id = format!("sha256:{}", sha256::lowercase_hex(&payload_digest));
    let prefix = format!(
        "{{\"schema\":\"{BACKEND_INPUT_SCHEMA}\",\"artifact_id\":\"{artifact_id}\",\"payload\":"
    );
    let payload_start = prefix.len();
    let payload_end = payload_start.checked_add(payload.len())?;
    let mut bytes = Vec::with_capacity(payload_end.checked_add(2)?);
    bytes.extend_from_slice(prefix.as_bytes());
    bytes.extend_from_slice(payload.as_bytes());
    bytes.extend_from_slice(b"}\n");
    Some(CanonicalBackendInputArtifact {
        bytes,
        payload_range: payload_start..payload_end,
        artifact_id,
    })
}

#[allow(clippy::too_many_arguments)]
fn encode_payload(
    facts: &CanonicalMinimalAddBackendFacts<'_, '_>,
    target_context: &str,
    source_digest: &str,
    left: (&str, &str, &str, &str, &str, &crate::diagnostic::Span),
    right: (&str, &str, &str, &str, &str, &crate::diagnostic::Span),
) -> String {
    let identity = facts.profile.backend_identity();
    let Item::Task(task) = facts.item else {
        unreachable!("validated minimal-add facts own a task")
    };
    let result_value_id = &identity.source_identities[2];
    let statement_id = &identity.source_identities[1];
    let root_id = identity.root.node_id.as_str();
    let mut out = String::with_capacity(4096);
    out.push_str("{\"compiler\":{\"version\":");
    push_json_string(&mut out, facts.compiler_version);
    out.push_str(",\"ir_schema\":\"hum.ir_contract.v0\",\"semantic_contract\":");
    push_json_string(&mut out, facts.semantic_contract);
    out.push_str(",\"feature_set\":[");
    push_json_string(&mut out, FEATURE_SET);
    out.push_str("],\"target_context\":");
    push_json_string(&mut out, target_context);
    out.push_str("},\"source_revision\":{\"id\":\"source:0\",\"sha256\":\"sha256:");
    out.push_str(source_digest);
    out.push_str("\",\"file_ordinal\":0,\"normalized_path\":");
    push_json_string(&mut out, SOURCE_PATH);
    out.push_str("},\"module\":{\"id\":\"module:examples.core.minimal_add\",\"name\":");
    push_json_string(&mut out, MODULE_NAME);
    out.push_str(
        ",\"files\":[\"source:0\"]},\"functions\":[{\"id\":\"function:0\",\"source_item_id\":",
    );
    push_json_string(&mut out, &identity.source_identities[0]);
    out.push_str(",\"display_name\":");
    push_json_string(&mut out, &task.name);
    out.push_str(",\"item_kind\":\"task\",\"linkage\":{\"kind\":\"internal\",\"symbol\":\"hum_fn_0\"},\"source_span\":{\"source_id\":\"source:0\",\"line\":");
    let _ = write!(out, "{},\"column\":{}", task.span.line, task.span.column);
    out.push_str("},\"abi\":{\"calling_convention\":\"hum_internal_v0\",\"parameters\":[");
    push_json_string(&mut out, left.0);
    out.push(',');
    push_json_string(&mut out, right.0);
    out.push_str("],\"parameter_types\":[\"type:int64\",\"type:int64\"],\"result\":");
    push_json_string(&mut out, result_value_id);
    out.push_str(",\"result_type\":\"type:int64\",\"integer_width\":64,\"trap_convention\":\"hum_checked_trap_v0\"},\"blocks\":[{\"id\":\"block:function:0:0\",\"operations\":[{\"id\":\"operation:function:0:block:0:0\",\"section_id\":\"section:function:0:does:0\",\"kind\":\"return\",\"statement_id\":");
    push_json_string(&mut out, statement_id);
    out.push_str(",\"expression_id\":");
    push_json_string(&mut out, root_id);
    out.push_str(",\"result_value_id\":");
    push_json_string(&mut out, result_value_id);
    out.push_str(",\"source_span\":{\"source_id\":\"source:0\",\"line\":");
    let _ = write!(
        out,
        "{},\"column\":{}",
        facts.statement.span.line, facts.statement.span.column
    );
    out.push_str("}}]}],\"expressions\":[{\"id\":");
    push_json_string(&mut out, root_id);
    out.push_str(",\"kind\":\"binary\",\"operator\":\"checked_add\",\"children\":[{\"ordinal\":0,\"node_id\":");
    push_json_string(&mut out, left.1);
    out.push_str(",\"value_id\":");
    push_json_string(&mut out, left.0);
    out.push_str(",\"definition_id\":");
    push_json_string(&mut out, left.2);
    out.push_str("},{\"ordinal\":1,\"node_id\":");
    push_json_string(&mut out, right.1);
    out.push_str(",\"value_id\":");
    push_json_string(&mut out, right.0);
    out.push_str(",\"definition_id\":");
    push_json_string(&mut out, right.2);
    out.push_str("}],\"result_value_id\":");
    push_json_string(&mut out, result_value_id);
    out.push_str(",\"checked_type_id\":\"type:int64\",\"effect_id\":\"effect:function:0:0\",\"resource_id\":\"resource:function:0:0\",\"failure_edge_id\":\"failure-edge:function:0:0\",\"unsupported\":[],\"source_provenance\":{\"source_id\":\"source:0\",\"statement_id\":");
    push_json_string(&mut out, statement_id);
    out.push_str(",\"line\":");
    let _ = write!(
        out,
        "{},\"column\":{}",
        identity.root.range.start.line, identity.root.range.start.column
    );
    out.push_str("}}],\"required_passes\":[");
    for (ordinal, (name, _, selected, _)) in facts.required_passes.iter().enumerate() {
        if ordinal > 0 {
            out.push(',');
        }
        out.push_str("{\"name\":");
        push_json_string(&mut out, name);
        out.push_str(",\"status\":\"passed\",\"selected\":");
        let _ = write!(out, "{selected},\"ordinal\":{ordinal}}}");
    }
    out.push_str("]}],\"types\":[{\"id\":\"type:int64\",\"source_type_id\":");
    push_json_string(&mut out, identity.checked_type.0);
    out.push_str(",\"name\":\"Int\",\"kind\":\"integer\",\"signed\":true,\"bits\":64}],\"definitions\":[{\"id\":");
    push_json_string(&mut out, left.2);
    out.push_str(",\"semantic_id\":");
    push_json_string(&mut out, left.3);
    out.push_str(",\"kind\":\"parameter\",\"ordinal\":0,\"value_id\":");
    push_json_string(&mut out, left.0);
    out.push_str(
        ",\"type_id\":\"type:int64\",\"source_span\":{\"source_id\":\"source:0\",\"line\":",
    );
    let _ = write!(out, "{},\"column\":{}", left.5.line, left.5.column);
    out.push_str("}},{\"id\":");
    push_json_string(&mut out, right.2);
    out.push_str(",\"semantic_id\":");
    push_json_string(&mut out, right.3);
    out.push_str(",\"kind\":\"parameter\",\"ordinal\":1,\"value_id\":");
    push_json_string(&mut out, right.0);
    out.push_str(
        ",\"type_id\":\"type:int64\",\"source_span\":{\"source_id\":\"source:0\",\"line\":",
    );
    let _ = write!(out, "{},\"column\":{}", right.5.line, right.5.column);
    out.push_str("}}],\"effects\":[{\"id\":\"effect:function:0:0\",\"effects\":[],\"external_authority\":[]}],\"resources\":[{\"id\":\"resource:function:0:0\",\"allocation_declaration\":\"nothing\",\"allocations\":[],\"moves\":[],\"borrows\":[],\"aliases\":[],\"ownership_transfers\":[],\"contract_predicates\":[],\"evidence_obligations\":[],\"profile\":\"normal\"}],\"failure_edges\":[{\"id\":\"failure-edge:function:0:0\",\"value_type\":\"signed_64\",\"operation\":\"checked_add\",\"behavior\":\"runtime_trap_on_overflow\"}],\"unsupported\":[]}");
    out
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{000c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            ch if ch <= '\u{001f}' => {
                let _ = write!(out, "\\u{:04x}", u32::from(ch));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

#[cfg(test)]
pub(crate) fn issue_with_final_profile_lineage_for_test(
    program: &Program,
    item: &Item,
    statement: &ParsedBodyStatement,
    honest_profile: profile_check::VerifiedMinimalAddProfile<'_>,
    foreign_profile_lineage: &profile_check::VerifiedMinimalAddProfile<'_>,
) -> (Option<Vec<String>>, usize) {
    FINAL_PROFILE_LINEAGE_OBSERVATION.with(|count| assert_eq!(count.replace(Some(0)), None));
    let result = assemble(program, item, statement, honest_profile).and_then(|facts| {
        assembled_is_authenticated(&facts, foreign_profile_lineage).then(|| facts.snapshot())
    });
    let comparisons = FINAL_PROFILE_LINEAGE_OBSERVATION.with(|count| {
        count
            .take()
            .expect("final profile-lineage observation must be armed")
    });
    (result, comparisons)
}

#[cfg(test)]
thread_local! {
    static FINAL_PROFILE_LINEAGE_OBSERVATION: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    static FACTS_CORRUPTION: std::cell::Cell<Option<(&'static str, usize)>> = const { std::cell::Cell::new(None) };
    static ARTIFACT_TARGET_CONTEXT_CORRUPTION: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn observe_final_profile_lineage_comparison_for_test(_accepted: bool) {
    FINAL_PROFILE_LINEAGE_OBSERVATION.with(|count| {
        if let Some(current) = count.get() {
            count.set(Some(
                current.checked_add(1).expect("lineage comparison count"),
            ));
        }
    });
}

#[cfg(not(test))]
fn observe_final_profile_lineage_comparison_for_test(_accepted: bool) {}

#[cfg(test)]
pub(crate) fn set_corruption_for_test(kind: &'static str, index: usize) {
    FACTS_CORRUPTION.with(|active| assert_eq!(active.replace(Some((kind, index))), None));
}

#[cfg(test)]
fn corrupt_backend_facts_for_test(facts: &mut CanonicalMinimalAddBackendFacts<'_, '_>) {
    let Some((kind, index)) = FACTS_CORRUPTION.with(std::cell::Cell::take) else {
        return;
    };
    let passes = &mut facts.required_passes;
    let edges = &mut facts.failure_edges;
    match kind {
        "pass_missing" => {
            if index < passes.len() {
                passes.remove(index);
            }
        }
        "pass_failed" => {
            let _ = passes.get_mut(index).map(|row| row.1 = false);
        }
        "pass_zero" | "pass_skipped" => {
            let _ = passes.get_mut(index).map(|row| row.2 = 0);
        }
        "pass_foreign" => {
            let _ = passes.get_mut(index).map(|row| row.3 ^= 1);
        }
        "pass_duplicate" => {
            if let (Some(row), Some(next)) = (passes.get(index).copied(), index.checked_add(1)) {
                passes.insert(next, row);
            }
        }
        "pass_reordered" => {
            if let Some(next) = index.checked_add(1).filter(|next| *next < passes.len()) {
                passes.swap(index, next);
            } else {
                passes.reverse();
            }
        }
        "empty_omitted" | "empty_missing" => {
            if index < facts.checked_empty_sets.len() {
                facts.checked_empty_sets.remove(index);
            }
        }
        "empty_not_checked" => set_empty_for_test(facts, index, "state_not_checked"),
        "empty_unsupported" => set_empty_for_test(facts, index, "state_unsupported_and_blocked"),
        "empty_substituted" | "empty_corrupted" => {
            set_empty_for_test(facts, index, "foreign_checked_empty_state")
        }
        "empty_duplicate" => {
            if let (Some(state), Some(next)) = (
                facts.checked_empty_sets.get(index).copied(),
                index.checked_add(1),
            ) {
                facts.checked_empty_sets.insert(next, state);
            }
        }
        "empty_reordered" => {
            let next = index
                .checked_add(1)
                .filter(|next| *next < facts.checked_empty_sets.len())
                .or_else(|| index.checked_sub(1));
            if let Some(next) = next {
                facts.checked_empty_sets.swap(index, next);
            }
        }
        "edge_omitted" => edges.clear(),
        "edge_duplicate" => {
            let _ = edges.first().copied().map(|edge| edges.push(edge));
        }
        "edge_reordered" => {
            let _ = edges.first_mut().map(|edge| edge.swap(0, 1));
        }
        "edge_wrong_type" => set_edge_for_test(edges, 0, "unsigned_64"),
        "edge_wraparound" => set_edge_for_test(edges, 1, "wrapping_add"),
        "edge_wrong_width" => set_edge_for_test(edges, 0, "signed_32"),
        "edge_foreign_trap" => set_edge_for_test(edges, 2, "foreign_trap"),
        _ => passes.clear(),
    }
}

#[cfg(test)]
fn set_empty_for_test(
    facts: &mut CanonicalMinimalAddBackendFacts<'_, '_>,
    index: usize,
    value: &'static str,
) {
    let _ = facts
        .checked_empty_sets
        .get_mut(index)
        .map(|slot| *slot = value);
}

#[cfg(test)]
fn set_edge_for_test(edges: &mut [[&'static str; 3]], index: usize, value: &'static str) {
    if let Some(field) = edges.first_mut().and_then(|edge| edge.get_mut(index)) {
        *field = value;
    }
}

#[cfg(test)]
fn artifact_target_context_for_test(honest: &'static str) -> &'static str {
    ARTIFACT_TARGET_CONTEXT_CORRUPTION
        .with(std::cell::Cell::take)
        .unwrap_or(honest)
}

#[cfg(not(test))]
fn artifact_target_context_for_test(honest: &'static str) -> &'static str {
    honest
}

#[cfg(test)]
pub(crate) fn set_artifact_target_context_corruption_for_test(value: &'static str) {
    ARTIFACT_TARGET_CONTEXT_CORRUPTION.with(|active| assert_eq!(active.replace(Some(value)), None));
}

#[cfg(test)]
pub(crate) fn minimal_add_artifact_for_test() -> CanonicalBackendInputArtifact {
    let source = include_str!("../examples/core/minimal_add.hum");
    let parsed = crate::parser::parse_source(SOURCE_PATH.to_string(), source);
    let checked = crate::check::check_parse_output(&parsed);
    assert!(parsed.diagnostics.is_empty());
    assert!(checked.diagnostics.is_empty());
    let program = Program {
        files: vec![parsed.file],
    };
    canonical_minimal_add_artifact(&program, &[]).expect("canonical fixture must issue")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_add_backend_input_bytes_are_canonical_and_deterministic() {
        let options = crate::parse_cli(vec!["backend-input".to_string(), SOURCE_PATH.to_string()])
            .expect("backend-input accepts exactly one source file");
        assert_eq!(options.command, "backend-input");
        assert_eq!(options.inputs.len(), 1);
        for rejected in [
            vec!["backend-input".to_string()],
            vec![
                "backend-input".to_string(),
                SOURCE_PATH.to_string(),
                SOURCE_PATH.to_string(),
            ],
            vec!["backend-input".to_string(), "examples/core".to_string()],
            vec![
                "backend-input".to_string(),
                "--format".to_string(),
                "json".to_string(),
                SOURCE_PATH.to_string(),
            ],
            vec![
                "backend-input".to_string(),
                "--timings".to_string(),
                SOURCE_PATH.to_string(),
            ],
        ] {
            assert!(crate::parse_cli(rejected).is_err());
        }

        let first = minimal_add_artifact_for_test();
        let second = minimal_add_artifact_for_test();
        assert_eq!(first.bytes(), second.bytes());
        assert_eq!(first.artifact_id(), second.artifact_id());
        assert_eq!(
            first.bytes(),
            include_bytes!("../fixtures/backend_input/minimal_add.backend_input.v0.json")
        );
        assert!(
            first
                .bytes()
                .starts_with(b"{\"schema\":\"hum.backend_input.v0\",\"artifact_id\":\"sha256:")
        );
        assert!(first.bytes().ends_with(b"}\n"));
        assert!(!first.bytes().contains(&b'\r'));
        assert_eq!(
            first.bytes().iter().filter(|byte| **byte == b'\n').count(),
            1
        );
        let payload_digest =
            sha256::lowercase_hex(&sha256::digest(first.payload()).expect("bounded payload"));
        assert_eq!(first.artifact_id(), format!("sha256:{payload_digest}"));
        let payload = std::str::from_utf8(first.payload()).expect("canonical UTF-8");
        assert!(payload.starts_with("{\"compiler\":"));
        assert!(payload.ends_with("\"unsupported\":[]}"));
        assert!(
            payload
                .contains("\"source_span\":{\"source_id\":\"source:0\",\"line\":3,\"column\":1}")
        );
        assert!(payload.contains("\"line\":8,\"column\":5"));
        assert!(payload.contains("\"line\":8,\"column\":12"));
        assert_eq!(payload.matches("\"status\":\"passed\"").count(), 14);
        assert!(!payload.contains("ir_verify"));
        assert!(!payload.contains("verified"));

        set_artifact_target_context_corruption_for_test("foreign_target_context_v0");
        let corrupted = minimal_add_artifact_for_test();
        assert_ne!(corrupted.bytes(), first.bytes());
        assert_ne!(corrupted.artifact_id(), first.artifact_id());
        assert!(
            std::str::from_utf8(corrupted.payload())
                .expect("UTF-8 corruption")
                .contains("foreign_target_context_v0")
        );

        let parsed = crate::parser::parse_source(
            SOURCE_PATH.to_string(),
            include_str!("../examples/core/minimal_add.hum"),
        );
        let checked = crate::check::check_parse_output(&parsed);
        let program = Program {
            files: vec![parsed.file],
        };
        let text = crate::ir_readiness::ir_readiness_text(&program, &checked.diagnostics);
        let json = crate::ir_readiness::ir_readiness_json(&program, &checked.diagnostics);
        let expected = [
            "canonical_minimal_add_backend_facts_v0",
            "source_and_operation_identity_bound_v0",
            "ordered_resolver_bindings_bound_v0",
            "verified_checked_type_bound_v0",
            "effect_checked_empty_v0",
            "ownership_checked_empty_v0",
            "resource_checked_empty_v0",
            "normal_profile_checked_v0",
            "checked_i64_overflow_trap_bound_v0",
            "canonical_backend_input_bytes_v0",
            "sha256_payload_identity_verified_v0",
            "ir_verify_passed_v0",
            "verified_backend_input_capability_lent_v0",
        ];
        let text_suffix = expected.join(", ");
        let text_line = text
            .lines()
            .find(|line| line.contains(expected[0]))
            .expect("canonical backend-input facts line");
        let text_start = text_line
            .find(expected[0])
            .expect("canonical backend-input facts suffix");
        assert_eq!(&text_line[text_start..], text_suffix);
        let json_suffix = expected
            .iter()
            .map(|fact| format!("\"{fact}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let json_line = json
            .lines()
            .find(|line| line.contains(expected[0]))
            .expect("JSON canonical backend-input facts line");
        let json_start = json_line
            .find(&format!("\"{}\"", expected[0]))
            .expect("JSON canonical backend-input facts suffix");
        assert_eq!(&json_line[json_start..], format!("{json_suffix}],"));
        assert!(text.contains("[ready_for_ir_with_verified_backend_input_v0]"));
        assert!(json.contains("\"status\": \"ready_for_ir_with_verified_backend_input_v0\""));
        assert!(text.contains("blocking_reasons: \n"));
        assert!(json.contains("\"blocking_reasons\": []"));
        assert!(text.contains("ir_ready=1"));
        assert!(json.contains("\"ir_ready\": 1"));
        assert!(text.contains("backend_ready=0"));
    }
}
