use crate::backend_input::{self, UnverifiedBackendInputV0};
use crate::{sha256, version};
use std::fmt::Write as _;
use std::ops::Range;

pub const IR_VERIFY_SCHEMA: &str = "hum.ir_verify.v0";

const NON_CLAIMS: [&str; 4] = [
    "not_backend_ready_v0",
    "not_executable_v0",
    "not_a_signature_v0",
    "no_durable_authority_v0",
];

const REQUIRED_PASSES: [&str; 14] = [
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

const PARAMETER_SEMANTIC_PREFIX: &str = "resolver-definition|scope=resolver-scope|parent=resolver-scope|parent=resolver-root|owner=semantic-file:0|kind=file|owner-kind=file|owner=resolver-definition|scope=resolver-scope|parent=resolver-root|owner=semantic-file:0|kind=file|owner-kind=file|kind=task|origin=resolver-item:file-0:path-0|shape=task|params=Borrow:named:Int,Borrow:named:Int|result=named:Int|effects=0|body=return|0|binary(Add;identifier;identifier)|kind=callable|owner-kind=task|kind=parameter|origin=";

pub(crate) struct VerifiedBackendInput<'artifact> {
    artifact: &'artifact [u8],
    payload_range: Range<usize>,
}

impl<'artifact> VerifiedBackendInput<'artifact> {
    fn from_verified_parts(artifact: &'artifact [u8], payload_range: Range<usize>) -> Self {
        Self {
            artifact,
            payload_range,
        }
    }

    pub(crate) fn artifact(&self) -> &'artifact [u8] {
        self.artifact
    }

    pub(crate) fn payload(&self) -> &'artifact [u8] {
        &self.artifact[self.payload_range.clone()]
    }

    pub(crate) fn projection_count(&self) -> usize {
        usize::from(!self.payload_range.is_empty())
    }
}

pub(crate) struct IrVerifyReport {
    status: &'static str,
    artifact_id: Option<String>,
    payload_bytes: usize,
    rejections: Vec<Rejection>,
}

struct Rejection {
    code: &'static str,
    byte_offset: Option<usize>,
    logical_path: &'static str,
    reason: &'static str,
}

impl IrVerifyReport {
    pub(crate) fn accepted(&self) -> bool {
        self.rejections.is_empty()
    }
}

pub(crate) fn with_verified_backend_input<R>(
    artifact: &[u8],
    consume: impl for<'artifact> FnOnce(VerifiedBackendInput<'artifact>) -> R,
) -> (IrVerifyReport, Option<R>) {
    match verify(artifact) {
        Ok(verified) => {
            let report = IrVerifyReport {
                status: "accepted_canonical_backend_input_v0",
                artifact_id: Some(verified.artifact_id.clone()),
                payload_bytes: verified.payload_range.len(),
                rejections: Vec::new(),
            };
            let access =
                VerifiedBackendInput::from_verified_parts(artifact, verified.payload_range);
            (report, Some(consume(access)))
        }
        Err(report) => (report, None),
    }
}

pub(crate) fn ir_verify_text(report: &IrVerifyReport) -> String {
    let mut out = String::new();
    out.push_str("Hum IR verify\n");
    let _ = writeln!(out, "schema: {IR_VERIFY_SCHEMA}");
    out.push_str("tool: ir-verify\n");
    let _ = writeln!(out, "version: {}", version::HUM_VERSION);
    let _ = writeln!(out, "status: {}", report.status);
    let _ = writeln!(
        out,
        "artifact_schema: {}",
        backend_input::BACKEND_INPUT_SCHEMA
    );
    let _ = writeln!(
        out,
        "artifact_id: {}",
        report.artifact_id.as_deref().unwrap_or("null")
    );
    push_human_summary(&mut out, report);
    out.push_str("rejections:\n");
    for rejection in &report.rejections {
        let _ = writeln!(
            out,
            "  {} byte_offset={} logical_path={} reason={}",
            rejection.code,
            rejection
                .byte_offset
                .map_or_else(|| "null".to_string(), |offset| offset.to_string()),
            rejection.logical_path,
            rejection.reason
        );
    }
    out.push_str("non_claims_v0:\n");
    for claim in NON_CLAIMS {
        let _ = writeln!(out, "  {claim}");
    }
    out
}

pub(crate) fn ir_verify_json(report: &IrVerifyReport) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    json_string_field(&mut out, 2, "schema", IR_VERIFY_SCHEMA, true);
    json_string_field(&mut out, 2, "tool", "ir-verify", true);
    json_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    json_string_field(&mut out, 2, "status", report.status, true);
    json_string_field(
        &mut out,
        2,
        "artifact_schema",
        backend_input::BACKEND_INPUT_SCHEMA,
        true,
    );
    out.push_str("  \"artifact_id\": ");
    match &report.artifact_id {
        Some(id) => push_json_string(&mut out, id),
        None => out.push_str("null"),
    }
    out.push_str(",\n  \"summary\": {\n");
    push_json_summary(&mut out, report);
    out.push_str("  },\n  \"rejections\": [");
    if report.rejections.is_empty() {
        out.push_str("],\n");
    } else {
        out.push('\n');
        for (index, rejection) in report.rejections.iter().enumerate() {
            out.push_str("    {\"code\": ");
            push_json_string(&mut out, rejection.code);
            out.push_str(", \"byte_offset\": ");
            match rejection.byte_offset {
                Some(offset) => {
                    let _ = write!(out, "{offset}");
                }
                None => out.push_str("null"),
            }
            out.push_str(", \"logical_path\": ");
            push_json_string(&mut out, rejection.logical_path);
            out.push_str(", \"reason\": ");
            push_json_string(&mut out, rejection.reason);
            out.push('}');
            if index + 1 != report.rejections.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ],\n");
    }
    out.push_str("  \"non_claims_v0\": [");
    for (index, claim) in NON_CLAIMS.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_json_string(&mut out, claim);
    }
    out.push_str("]\n}\n");
    out
}

struct VerifiedParts {
    artifact_id: String,
    payload_range: Range<usize>,
}

struct DecodedBackendInputV0 {
    model: UnverifiedBackendInputV0,
    scalar_semantics_match: bool,
    ordered_linkage_matches: bool,
    required_passes_match: bool,
}

macro_rules! closed_object {
    ($node:expr; $($key:ident),+ $(,)?) => {{
        let members = object($node)?;
        exact_keys(members, &[$(stringify!($key)),+])?;
        members
    }};
}

macro_rules! closed_single_object {
    ($node:expr; $($key:ident),+ $(,)?) => {
        closed_object!(single(array($node)?)?; $($key),+)
    };
}

macro_rules! decoded_value {
    (text $members:ident $key:ident) => {
        string_field($members, stringify!($key))?.to_string()
    };
    (strings $members:ident $key:ident) => {
        string_array(field($members, stringify!($key))?)?
    };
    (number $members:ident $key:ident) => {
        usize_field($members, stringify!($key))?
    };
    (boolean $members:ident $key:ident) => {
        bool_field($members, stringify!($key))?
    };
    (span $members:ident $key:ident) => {
        source_span(field($members, stringify!($key))?)?
    };
}

macro_rules! decoded_record {
    ($constructor:ident; $($kind:ident $members:ident $key:ident),+ $(,)?) => {
        backend_input::$constructor($(decoded_value!($kind $members $key)),+)
    };
}

type Failure = (&'static str, &'static str, &'static str);
const BOM: Failure = ("invalid_framing_v0", "$", "UTF-8 BOM forbidden");
const FRAMING: Failure = ("invalid_framing_v0", "$", "single final LF required");
const UTF8: Failure = ("invalid_utf8_v0", "$", "invalid UTF-8");
const BAD_SPANS: Failure = ("malformed_json_v0", "$", "inconsistent JSON spans");
const DUPLICATE: Failure = ("duplicate_key_v0", "$", "duplicate object key");
const ENVELOPE_OBJECT: Failure = ("invalid_envelope_v0", "$", "envelope must be an object");
const ENVELOPE_KEYS: Failure = ("invalid_envelope_v0", "$", "envelope keys/order differ");
const MISSING_SCHEMA: Failure = ("invalid_envelope_v0", "$.schema", "schema missing/invalid");
const UNSUPPORTED_SCHEMA: Failure = ("unsupported_schema_v0", "$.schema", "unsupported schema");
const NO_ID: Failure = (
    "invalid_artifact_id_v0",
    "$.artifact_id",
    "artifact ID missing",
);
const MISSING_PAYLOAD: Failure = ("invalid_envelope_v0", "$.payload", "payload missing");
const BAD_ID: Failure = (
    "invalid_artifact_id_v0",
    "$.artifact_id",
    "artifact ID spelling invalid",
);
const NONCANONICAL: Failure = ("noncanonical_bytes_v0", "$", "noncanonical re-emission");
const NO_DIGEST: Failure = (
    "digest_unavailable_v0",
    "$.artifact_id",
    "digest unavailable",
);
const BAD_DIGEST: Failure = (
    "artifact_id_mismatch_v0",
    "$.artifact_id",
    "payload digest mismatch",
);

struct RejectionContext<'a> {
    artifact_id: Option<&'a str>,
    payload_bytes: usize,
}

impl RejectionContext<'_> {
    fn reject(
        &self,
        (code, logical_path, reason): Failure,
        byte_offset: Option<usize>,
    ) -> IrVerifyReport {
        IrVerifyReport {
            status: "rejected_backend_input_v0",
            artifact_id: self.artifact_id.map(ToOwned::to_owned),
            payload_bytes: self.payload_bytes,
            rejections: vec![Rejection {
                code,
                byte_offset,
                logical_path,
                reason,
            }],
        }
    }
}

fn verify(artifact: &[u8]) -> Result<VerifiedParts, IrVerifyReport> {
    let unbound = RejectionContext {
        artifact_id: None,
        payload_bytes: 0,
    };
    if artifact.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(unbound.reject(BOM, Some(0)));
    }
    if !artifact.ends_with(b"\n") || artifact.ends_with(b"\n\n") || artifact.contains(&b'\r') {
        return Err(unbound.reject(FRAMING, None));
    }
    let body = std::str::from_utf8(&artifact[..artifact.len() - 1])
        .map_err(|error| unbound.reject(UTF8, Some(error.valid_up_to())))?;
    let root = Parser::new(body)
        .parse()
        .map_err(|error| unbound.reject((error.code, "$", error.reason), Some(error.offset)))?;
    if !raw_spans_are_well_formed(&root, body.len()) {
        return Err(unbound.reject(BAD_SPANS, None));
    }
    if let Some(offset) = first_duplicate_key_offset(&root) {
        return Err(unbound.reject(DUPLICATE, Some(offset)));
    }
    let envelope =
        object(&root).map_err(|_| unbound.reject(ENVELOPE_OBJECT, Some(root.span.start)))?;
    exact_keys(envelope, &["schema", "artifact_id", "payload"])
        .map_err(|_| unbound.reject(ENVELOPE_KEYS, Some(root.span.start)))?;
    let declared_id = string_field(envelope, "artifact_id")
        .ok()
        .map(ToOwned::to_owned);
    let envelope_context = RejectionContext {
        artifact_id: declared_id.as_deref(),
        payload_bytes: 0,
    };
    let schema = string_field(envelope, "schema")
        .map_err(|_| envelope_context.reject(MISSING_SCHEMA, Some(root.span.start)))?;
    if schema != backend_input::BACKEND_INPUT_SCHEMA {
        return Err(envelope_context.reject(UNSUPPORTED_SCHEMA, Some(root.span.start)));
    }
    let declared_id = string_field(envelope, "artifact_id")
        .map_err(|_| unbound.reject(NO_ID, None))?
        .to_string();
    let payload = field(envelope, "payload").map_err(|_| {
        RejectionContext {
            artifact_id: Some(&declared_id),
            payload_bytes: 0,
        }
        .reject(MISSING_PAYLOAD, None)
    })?;
    let payload_range = payload.span.clone();
    let bound = RejectionContext {
        artifact_id: Some(&declared_id),
        payload_bytes: payload_range.len(),
    };
    let parts = extract_model(payload).map_err(|(path, reason)| {
        bound.reject(
            ("semantic_model_mismatch_v0", path, reason),
            Some(payload.span.start),
        )
    })?;
    let reencoded = backend_input::reencode_unverified_backend_input_v0(&parts.model, &declared_id)
        .ok_or_else(|| bound.reject(BAD_ID, None))?;
    if reencoded.bytes() != artifact {
        return Err(bound.reject(NONCANONICAL, None));
    }
    let digest = sha256::digest(&artifact[payload_range.clone()])
        .ok_or_else(|| bound.reject(NO_DIGEST, None))?;
    let computed = format!("sha256:{}", sha256::lowercase_hex(&digest));
    if computed != declared_id {
        return Err(bound.reject(BAD_DIGEST, None));
    }
    validate_model(&parts).map_err(|(path, reason)| {
        bound.reject(
            ("semantic_model_mismatch_v0", path, reason),
            Some(payload.span.start),
        )
    })?;
    Ok(VerifiedParts {
        artifact_id: computed,
        payload_range,
    })
}

fn extract_model(
    payload: &JsonNode,
) -> Result<DecodedBackendInputV0, (&'static str, &'static str)> {
    let p = closed_object!(payload;
        compiler, source_revision, module, functions, types, definitions, effects, resources,
        failure_edges, unsupported
    );
    let compiler = closed_object!(field(p, "compiler")?;
        version, ir_schema, semantic_contract, feature_set, target_context
    );
    let source = closed_object!(field(p, "source_revision")?;
        id, sha256, file_ordinal, normalized_path
    );
    let module = closed_object!(field(p, "module")?; id, name, files);
    let function = closed_single_object!(field(p, "functions")?;
        id, source_item_id, display_name, item_kind, linkage, source_span, abi, blocks,
        expressions, required_passes
    );
    let linkage = closed_object!(field(function, "linkage")?; kind, symbol);
    let function_source_span =
        closed_object!(field(function, "source_span")?; source_id, line, column);
    let abi = closed_object!(field(function, "abi")?;
        calling_convention, parameters, parameter_types, result, result_type, integer_width,
        trap_convention
    );
    let parameters = array(field(abi, "parameters")?)?;
    let parameter_types = array(field(abi, "parameter_types")?)?;
    let block = closed_single_object!(field(function, "blocks")?; id, operations);
    let operation = closed_single_object!(field(block, "operations")?;
        id, section_id, kind, statement_id, expression_id, result_value_id, source_span
    );
    let operation_source_span =
        closed_object!(field(operation, "source_span")?; source_id, line, column);
    let expression = closed_single_object!(field(function, "expressions")?;
        id, kind, operator, children, result_value_id, checked_type_id, effect_id,
        resource_id, failure_edge_id, unsupported, source_provenance
    );
    let provenance = closed_object!(field(expression, "source_provenance")?;
        source_id, statement_id, line, column
    );
    let children = array(field(expression, "children")?)?;
    if children.len() != 2 || parameters.len() != 2 || parameter_types.len() != 2 {
        return Err((
            "$.payload.functions[0]",
            "exactly two ordered parameters, parameter types, and children are required",
        ));
    }
    let definitions = array(field(p, "definitions")?)?;
    if definitions.len() != 2 {
        return Err((
            "$.payload.definitions",
            "exactly two definitions are required",
        ));
    }
    let mut decoded_parameters = Vec::new();
    for index in 0..2 {
        let child = closed_object!(&children[index]; ordinal, node_id, value_id, definition_id);
        let definition = closed_object!(&definitions[index];
            id, semantic_id, kind, ordinal, value_id, type_id, source_span
        );
        let definition_span =
            closed_object!(field(definition, "source_span")?; source_id, line, column);
        let abi_value = string(&parameters[index])?.to_string();
        let abi_type = string(&parameter_types[index])?.to_string();
        let child_ordinal = usize_field(child, "ordinal")?;
        let node_id = string_field(child, "node_id")?.to_string();
        let child_value = string_field(child, "value_id")?.to_string();
        let child_definition = string_field(child, "definition_id")?.to_string();
        let definition_id = string_field(definition, "id")?.to_string();
        let semantic_id = string_field(definition, "semantic_id")?.to_string();
        let definition_kind = string_field(definition, "kind")?.to_string();
        let definition_ordinal = usize_field(definition, "ordinal")?;
        let definition_value = string_field(definition, "value_id")?.to_string();
        let definition_type = string_field(definition, "type_id")?.to_string();
        let source_id = string_field(definition_span, "source_id")?.to_string();
        let span = source_span(field(definition, "source_span")?)?;
        let position = format!("parameter-position:{index}|shape=parameter-position:{index}");
        let expected_node = format!(
            "parser-body:resolver-item:file-0:path-0:statement-0:expression-0:binary-{}",
            if index == 0 { "left" } else { "right" }
        );
        let expected_definition = format!(
            "def_{}_parameter_{}",
            index + 1,
            if index == 0 { "a" } else { "b" }
        );
        let expected_semantic = format!("{PARAMETER_SEMANTIC_PREFIX}{position}");
        let linkage_matches = abi_type == "type:int64"
            && child_ordinal == index
            && node_id == expected_node
            && child_definition == expected_definition
            && definition_id == expected_definition
            && definition_kind == "parameter"
            && definition_ordinal == index
            && definition_type == "type:int64"
            && source_id == "source:0"
            && span == (3, if index == 0 { 10 } else { 18 })
            && semantic_id == expected_semantic
            && abi_value == child_value
            && abi_value == definition_value
            && abi_value == format!("core-value:param:{semantic_id}");
        decoded_parameters.push((
            backend_input::backend_parameter_v0(
                abi_value,
                abi_type,
                child_ordinal,
                node_id,
                child_value,
                child_definition,
                definition_id,
                semantic_id,
                definition_kind,
                definition_ordinal,
                definition_value,
                definition_type,
                source_id,
                span,
            ),
            linkage_matches,
        ));
    }
    let required = array(field(function, "required_passes")?)?;
    let required_passes = required
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let row = closed_object!(row; name, status, selected, ordinal);
            let name = string_field(row, "name")?.to_string();
            let status = string_field(row, "status")?.to_string();
            let selected = usize_field(row, "selected")?;
            let ordinal = usize_field(row, "ordinal")?;
            let matches = REQUIRED_PASSES.get(ordinal).copied() == Some(name.as_str())
                && status == "passed"
                && selected == 1
                && ordinal == index;
            Ok((
                backend_input::backend_pass_v0(name, status, selected, ordinal),
                matches,
            ))
        })
        .collect::<Result<Vec<_>, (&'static str, &'static str)>>()?;
    let checked_type = closed_single_object!(field(p, "types")?;
        id, source_type_id, name, kind, signed, bits
    );
    let effect = closed_single_object!(field(p, "effects")?; id, effects, external_authority);
    let resource = closed_single_object!(field(p, "resources")?;
        id, allocation_declaration, allocations, moves, borrows, aliases, ownership_transfers,
        contract_predicates, evidence_obligations, profile
    );
    let failure_edge = closed_single_object!(field(p, "failure_edges")?;
        id, value_type, operation, behavior
    );
    let scalar_semantics_match = string_field(compiler, "version")? == version::HUM_VERSION
        && string_field(compiler, "ir_schema")? == "hum.ir_contract.v0"
        && string_field(compiler, "semantic_contract")?
            == "hum.canonical_minimal_add_backend_facts.v0"
        && string_array(field(compiler, "feature_set")?)?
            == ["canonical_minimal_add_checked_i64_v0"]
        && string_field(compiler, "target_context")? == "target_independent_checked_i64_v0"
        && string_field(source, "id")? == "source:0"
        && string_field(source, "sha256")?
            == "sha256:aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6"
        && usize_field(source, "file_ordinal")? == 0
        && string_field(source, "normalized_path")? == "examples/core/minimal_add.hum"
        && string_field(module, "id")? == "module:examples.core.minimal_add"
        && string_field(module, "name")? == "examples.core.minimal_add"
        && string_array(field(module, "files")?)? == ["source:0"]
        && string_field(function, "id")? == "function:0"
        && string_field(function, "source_item_id")? == "resolver-item:file-0:path-0"
        && string_field(function, "display_name")? == "add"
        && string_field(function, "item_kind")? == "task"
        && string_field(linkage, "kind")? == "internal"
        && string_field(linkage, "symbol")? == "hum_fn_0"
        && string_field(function_source_span, "source_id")? == "source:0"
        && source_span(field(function, "source_span")?)? == (3, 1)
        && string_field(abi, "calling_convention")? == "hum_internal_v0"
        && string_field(abi, "result")?
            == "core-value:parser-body:resolver-item:file-0:path-0:statement-0:expression-0"
        && string_field(abi, "result_type")? == "type:int64"
        && usize_field(abi, "integer_width")? == 64
        && string_field(abi, "trap_convention")? == "hum_checked_trap_v0"
        && string_field(block, "id")? == "block:function:0:0"
        && string_field(operation, "id")? == "operation:function:0:block:0:0"
        && string_field(operation, "section_id")? == "section:function:0:does:0"
        && string_field(operation, "kind")? == "return"
        && string_field(operation, "statement_id")?
            == "parser-body:resolver-item:file-0:path-0:statement-0"
        && string_field(operation, "expression_id")?
            == "parser-body:resolver-item:file-0:path-0:statement-0:expression-0"
        && string_field(operation, "result_value_id")? == string_field(abi, "result")?
        && string_field(operation_source_span, "source_id")? == "source:0"
        && source_span(field(operation, "source_span")?)? == (8, 5)
        && string_field(expression, "id")? == string_field(operation, "expression_id")?
        && string_field(expression, "kind")? == "binary"
        && string_field(expression, "operator")? == "checked_add"
        && string_field(expression, "result_value_id")? == string_field(abi, "result")?
        && string_field(expression, "checked_type_id")? == "type:int64"
        && string_field(expression, "effect_id")? == "effect:function:0:0"
        && string_field(expression, "resource_id")? == "resource:function:0:0"
        && string_field(expression, "failure_edge_id")? == "failure-edge:function:0:0"
        && string_array(field(expression, "unsupported")?)?.is_empty()
        && string_field(provenance, "source_id")? == "source:0"
        && string_field(provenance, "statement_id")? == string_field(operation, "statement_id")?
        && (
            usize_field(provenance, "line")?,
            usize_field(provenance, "column")?,
        ) == (8, 12)
        && string_field(checked_type, "id")? == "type:int64"
        && string_field(checked_type, "source_type_id")? == "hum-type:builtin:Int"
        && string_field(checked_type, "name")? == "Int"
        && string_field(checked_type, "kind")? == "integer"
        && bool_field(checked_type, "signed")?
        && usize_field(checked_type, "bits")? == 64
        && string_field(effect, "id")? == string_field(expression, "effect_id")?
        && string_array(field(effect, "effects")?)?.is_empty()
        && string_array(field(effect, "external_authority")?)?.is_empty()
        && string_field(resource, "id")? == string_field(expression, "resource_id")?
        && string_field(resource, "allocation_declaration")? == "nothing"
        && string_array(field(resource, "allocations")?)?.is_empty()
        && string_array(field(resource, "moves")?)?.is_empty()
        && string_array(field(resource, "borrows")?)?.is_empty()
        && string_array(field(resource, "aliases")?)?.is_empty()
        && string_array(field(resource, "ownership_transfers")?)?.is_empty()
        && string_array(field(resource, "contract_predicates")?)?.is_empty()
        && string_array(field(resource, "evidence_obligations")?)?.is_empty()
        && string_field(resource, "profile")? == "normal"
        && string_field(failure_edge, "id")? == string_field(expression, "failure_edge_id")?
        && string_field(failure_edge, "value_type")? == "signed_64"
        && string_field(failure_edge, "operation")? == "checked_add"
        && string_field(failure_edge, "behavior")? == "runtime_trap_on_overflow"
        && string_array(field(p, "unsupported")?)?.is_empty();
    let ordered_linkage_matches = decoded_parameters.iter().all(|(_, matches)| *matches);
    let required_passes_match = required_passes.len() == REQUIRED_PASSES.len()
        && required_passes.iter().all(|(_, matches)| *matches);
    let parameters = decoded_parameters
        .into_iter()
        .map(|(model, _)| model)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| {
            (
                "$.payload.functions[0].abi.parameters",
                "two parameters required",
            )
        })?;
    let passes = required_passes
        .into_iter()
        .map(|(model, _)| model)
        .collect();
    Ok(DecodedBackendInputV0 {
        model: backend_input::unverified_backend_input_v0(
            decoded_record!(backend_compiler_v0;
                text compiler version, text compiler ir_schema, text compiler semantic_contract,
                strings compiler feature_set, text compiler target_context
            ),
            decoded_record!(backend_source_revision_v0;
                text source id, text source sha256, number source file_ordinal,
                text source normalized_path
            ),
            decoded_record!(backend_module_v0;
                text module id, text module name, strings module files
            ),
            decoded_record!(backend_function_v0;
                text function id, text function source_item_id, text function display_name,
                text function item_kind, text linkage kind, text linkage symbol,
                text function_source_span source_id, span function source_span
            ),
            decoded_record!(backend_abi_v0;
                text abi calling_convention, text abi result, text abi result_type,
                number abi integer_width, text abi trap_convention
            ),
            decoded_record!(backend_operation_v0;
                text block id, text operation id, text operation section_id, text operation kind,
                text operation statement_id, text operation expression_id,
                text operation result_value_id, text operation_source_span source_id,
                span operation source_span
            ),
            decoded_record!(backend_expression_v0;
                text expression id, text expression kind, text expression operator,
                text expression result_value_id, text expression checked_type_id,
                text expression effect_id, text expression resource_id,
                text expression failure_edge_id, strings expression unsupported,
                text provenance source_id, text provenance statement_id, span expression source_provenance
            ),
            parameters,
            decoded_record!(backend_type_v0;
                text checked_type id, text checked_type source_type_id, text checked_type name,
                text checked_type kind, boolean checked_type signed, number checked_type bits
            ),
            decoded_record!(backend_effect_v0;
                text effect id, strings effect effects, strings effect external_authority
            ),
            decoded_record!(backend_resource_v0;
                text resource id, text resource allocation_declaration, strings resource allocations,
                strings resource moves, strings resource borrows, strings resource aliases,
                strings resource ownership_transfers, strings resource contract_predicates,
                strings resource evidence_obligations, text resource profile
            ),
            decoded_record!(backend_failure_edge_v0;
                text failure_edge id, text failure_edge value_type, text failure_edge operation,
                text failure_edge behavior
            ),
            passes,
            string_array(field(p, "unsupported")?)?,
        ),
        scalar_semantics_match,
        ordered_linkage_matches,
        required_passes_match,
    })
}

fn validate_model(parts: &DecodedBackendInputV0) -> Result<(), (&'static str, &'static str)> {
    if !parts.scalar_semantics_match {
        return Err((
            "$.payload",
            "minimal-add identity or source lineage differs",
        ));
    }
    if !parts.ordered_linkage_matches {
        return Err((
            "$.payload.definitions",
            "ordered child, definition, value, and parameter linkage differs",
        ));
    }
    if !parts.required_passes_match {
        return Err((
            "$.payload.functions[0].required_passes",
            "required pass set differs",
        ));
    }
    Ok(())
}

struct JsonNode {
    value: JsonValue,
    span: Range<usize>,
}

enum JsonValue {
    Object(Vec<JsonMember>),
    Array(Vec<JsonNode>),
    String(String),
    Number(String),
    Bool(bool),
    Null,
}

struct JsonMember {
    key: String,
    key_span: Range<usize>,
    value: JsonNode,
    member_span: Range<usize>,
}

#[derive(Debug)]
struct ParseError {
    code: &'static str,
    offset: usize,
    reason: &'static str,
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            pos: 0,
        }
    }
    fn parse(mut self) -> Result<JsonNode, ParseError> {
        let value = self.value()?;
        self.ws();
        if self.pos != self.bytes.len() {
            return Err(self.error("trailing_json_bytes_v0", "trailing JSON bytes"));
        }
        Ok(value)
    }
    fn value(&mut self) -> Result<JsonNode, ParseError> {
        self.ws();
        let start = self.pos;
        let value = match self.peek() {
            Some(b'{') => self.object()?,
            Some(b'[') => self.array()?,
            Some(b'"') => JsonValue::String(self.string()?),
            Some(b't') => self.literal(b"true", JsonValue::Bool(true))?,
            Some(b'f') => self.literal(b"false", JsonValue::Bool(false))?,
            Some(b'n') => self.literal(b"null", JsonValue::Null)?,
            Some(b'-' | b'0'..=b'9') => JsonValue::Number(self.number()?),
            _ => return Err(self.error("malformed_json_v0", "expected JSON value")),
        };
        Ok(JsonNode {
            value,
            span: start..self.pos,
        })
    }
    fn object(&mut self) -> Result<JsonValue, ParseError> {
        self.take(b'{')?;
        self.ws();
        let mut members = Vec::new();
        if self.consume(b'}') {
            return Ok(JsonValue::Object(members));
        }
        loop {
            self.ws();
            let member_start = self.pos;
            if self.peek() != Some(b'"') {
                return Err(self.error("malformed_json_v0", "expected object key"));
            }
            let key_start = self.pos;
            let key = self.string()?;
            let key_span = key_start..self.pos;
            self.ws();
            self.take(b':')?;
            let value = self.value()?;
            let member_span = member_start..value.span.end;
            members.push(JsonMember {
                key,
                key_span,
                value,
                member_span,
            });
            self.ws();
            if self.consume(b'}') {
                break;
            }
            self.take(b',')?;
        }
        Ok(JsonValue::Object(members))
    }
    fn array(&mut self) -> Result<JsonValue, ParseError> {
        self.take(b'[')?;
        self.ws();
        let mut values = Vec::new();
        if self.consume(b']') {
            return Ok(JsonValue::Array(values));
        }
        loop {
            values.push(self.value()?);
            self.ws();
            if self.consume(b']') {
                break;
            }
            self.take(b',')?;
        }
        Ok(JsonValue::Array(values))
    }
    fn string(&mut self) -> Result<String, ParseError> {
        self.take(b'"')?;
        let mut out = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.error("malformed_json_v0", "unterminated string"));
            };
            self.pos += 1;
            match byte {
                b'"' => break,
                b'\\' => {
                    let escape = self
                        .peek()
                        .ok_or_else(|| self.error("invalid_escape_v0", "incomplete escape"))?;
                    self.pos += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let scalar = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&scalar) {
                                if !self.consume(b'\\') || !self.consume(b'u') {
                                    return Err(self.error(
                                        "invalid_escape_v0",
                                        "high surrogate must be followed by a low surrogate",
                                    ));
                                }
                                let low = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err(
                                        self.error("invalid_escape_v0", "invalid low surrogate")
                                    );
                                }
                                0x1_0000 + ((scalar - 0xd800) << 10) + (low - 0xdc00)
                            } else if (0xdc00..=0xdfff).contains(&scalar) {
                                return Err(
                                    self.error("invalid_escape_v0", "unpaired low surrogate")
                                );
                            } else {
                                scalar
                            };
                            out.push(char::from_u32(scalar).ok_or_else(|| {
                                self.error("invalid_escape_v0", "invalid Unicode scalar")
                            })?);
                        }
                        _ => return Err(self.error("invalid_escape_v0", "invalid string escape")),
                    }
                }
                0..=31 => return Err(self.error("invalid_control_v0", "unescaped control byte")),
                _ if byte.is_ascii() => out.push(byte as char),
                _ => {
                    self.pos -= 1;
                    let rest = std::str::from_utf8(&self.bytes[self.pos..])
                        .map_err(|_| self.error("invalid_utf8_v0", "invalid UTF-8"))?;
                    let ch = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.error("invalid_utf8_v0", "invalid UTF-8"))?;
                    self.pos += ch.len_utf8();
                    out.push(ch);
                }
            }
        }
        Ok(out)
    }
    fn hex4(&mut self) -> Result<u32, ParseError> {
        if self.pos + 4 > self.bytes.len() {
            return Err(self.error("invalid_escape_v0", "short Unicode escape"));
        }
        let text = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
            .map_err(|_| self.error("invalid_escape_v0", "invalid Unicode escape"))?;
        self.pos += 4;
        u32::from_str_radix(text, 16)
            .map_err(|_| self.error("invalid_escape_v0", "invalid Unicode escape"))
    }
    fn number(&mut self) -> Result<String, ParseError> {
        let start = self.pos;
        if self.consume(b'-') && self.peek() == Some(b'0') {
            return Err(self.error("invalid_number_v0", "negative zero forbidden"));
        }
        if self.consume(b'0') {
            if matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("invalid_number_v0", "leading zero forbidden"));
            }
        } else {
            let before = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            if before == self.pos {
                return Err(self.error("invalid_number_v0", "invalid integer"));
            }
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(self.error(
                "invalid_number_v0",
                "only nonnegative canonical integers are allowed",
            ));
        }
        Ok(std::str::from_utf8(&self.bytes[start..self.pos])
            .unwrap_or_default()
            .to_string())
    }
    fn literal(&mut self, expected: &[u8], value: JsonValue) -> Result<JsonValue, ParseError> {
        for byte in expected {
            self.take(*byte)?;
        }
        Ok(value)
    }
    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn take(&mut self, byte: u8) -> Result<(), ParseError> {
        if self.consume(byte) {
            Ok(())
        } else {
            Err(self.error("malformed_json_v0", "unexpected JSON token"))
        }
    }
    fn error(&self, code: &'static str, reason: &'static str) -> ParseError {
        ParseError {
            code,
            offset: self.pos,
            reason,
        }
    }
}

fn object(node: &JsonNode) -> Result<&[JsonMember], (&'static str, &'static str)> {
    match &node.value {
        JsonValue::Object(value) => Ok(value),
        _ => Err(("$", "object required")),
    }
}
fn array(node: &JsonNode) -> Result<&[JsonNode], (&'static str, &'static str)> {
    match &node.value {
        JsonValue::Array(value) => Ok(value),
        _ => Err(("$", "array required")),
    }
}
fn string(node: &JsonNode) -> Result<&str, (&'static str, &'static str)> {
    match &node.value {
        JsonValue::String(value) => Ok(value),
        _ => Err(("$", "string required")),
    }
}
fn string_array(node: &JsonNode) -> Result<Vec<String>, (&'static str, &'static str)> {
    array(node)?
        .iter()
        .map(|value| string(value).map(ToOwned::to_owned))
        .collect()
}
fn exact_keys(
    members: &[JsonMember],
    expected: &[&str],
) -> Result<(), (&'static str, &'static str)> {
    if members.len() == expected.len()
        && members
            .iter()
            .zip(expected)
            .all(|(member, expected)| member.key == *expected)
    {
        Ok(())
    } else {
        Err(("$.payload", "closed object keys or order differ"))
    }
}
fn field<'a>(
    members: &'a [JsonMember],
    key: &str,
) -> Result<&'a JsonNode, (&'static str, &'static str)> {
    members
        .iter()
        .find(|member| member.key == key)
        .map(|member| &member.value)
        .ok_or(("$", "required key missing"))
}
fn string_field<'a>(
    members: &'a [JsonMember],
    key: &str,
) -> Result<&'a str, (&'static str, &'static str)> {
    string(field(members, key)?)
}
fn usize_field(members: &[JsonMember], key: &str) -> Result<usize, (&'static str, &'static str)> {
    match &field(members, key)?.value {
        JsonValue::Number(value) => value.parse().map_err(|_| ("$", "integer required")),
        _ => Err(("$", "integer required")),
    }
}
fn bool_field(members: &[JsonMember], key: &str) -> Result<bool, (&'static str, &'static str)> {
    match &field(members, key)?.value {
        JsonValue::Bool(value) => Ok(*value),
        _ => Err(("$", "boolean required")),
    }
}
fn single(values: &[JsonNode]) -> Result<&JsonNode, (&'static str, &'static str)> {
    if let [value] = values {
        Ok(value)
    } else {
        Err(("$", "exactly one row required"))
    }
}
fn source_span(node: &JsonNode) -> Result<(usize, usize), (&'static str, &'static str)> {
    let span = object(node)?;
    Ok((usize_field(span, "line")?, usize_field(span, "column")?))
}

fn raw_spans_are_well_formed(node: &JsonNode, bound: usize) -> bool {
    if node.span.start > node.span.end || node.span.end > bound {
        return false;
    }
    match &node.value {
        JsonValue::Object(members) => members.iter().all(|member| {
            member.key_span.start >= node.span.start
                && member.key_span.start < member.key_span.end
                && member.key_span.end <= member.value.span.start
                && member.member_span.start == member.key_span.start
                && member.member_span.end == member.value.span.end
                && member.member_span.end <= node.span.end
                && raw_spans_are_well_formed(&member.value, bound)
        }),
        JsonValue::Array(values) => values
            .iter()
            .all(|value| raw_spans_are_well_formed(value, bound)),
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Bool(_) | JsonValue::Null => true,
    }
}

fn first_duplicate_key_offset(node: &JsonNode) -> Option<usize> {
    match &node.value {
        JsonValue::Object(members) => {
            for (index, member) in members.iter().enumerate() {
                if members[..index]
                    .iter()
                    .any(|previous| previous.key == member.key)
                {
                    return Some(member.key_span.start);
                }
                if let Some(offset) = first_duplicate_key_offset(&member.value) {
                    return Some(offset);
                }
            }
            None
        }
        JsonValue::Array(values) => values.iter().find_map(first_duplicate_key_offset),
        JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Bool(_) | JsonValue::Null => None,
    }
}

fn push_human_summary(out: &mut String, report: &IrVerifyReport) {
    let counts = summary_counts(report);
    for (name, value) in counts {
        let _ = writeln!(out, "{name}: {value}");
    }
}
fn push_json_summary(out: &mut String, report: &IrVerifyReport) {
    let counts = summary_counts(report);
    for (index, (name, value)) in counts.iter().enumerate() {
        let _ = writeln!(
            out,
            "    \"{name}\": {value}{}",
            if index + 1 == counts.len() { "" } else { "," }
        );
    }
}
fn summary_counts(report: &IrVerifyReport) -> Vec<(&'static str, usize)> {
    let accepted = usize::from(report.accepted());
    vec![
        ("payload_bytes", report.payload_bytes),
        ("source_count", accepted),
        ("module_count", accepted),
        ("function_count", accepted),
        ("block_count", accepted),
        ("operation_count", accepted),
        ("expression_count", accepted),
        ("type_count", accepted),
        ("definition_count", accepted * 2),
        ("effect_count", accepted),
        ("resource_count", accepted),
        ("failure_edge_count", accepted),
        ("required_pass_count", accepted * 14),
        ("unsupported_count", 0),
    ]
}
fn json_string_field(out: &mut String, indent: usize, name: &str, value: &str, comma: bool) {
    let _ = write!(out, "{}\"{name}\": ", " ".repeat(indent));
    push_json_string(out, value);
    out.push_str(if comma { ",\n" } else { "\n" });
}
fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch <= '\u{1f}' => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replace_once(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let offsets = bytes
            .windows(from.len())
            .enumerate()
            .filter_map(|(offset, window)| (window == from).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(offsets.len(), 1, "mutation needle must occur exactly once");
        let offset = offsets[0];
        let mut changed = Vec::with_capacity(bytes.len() - from.len() + to.len());
        changed.extend_from_slice(&bytes[..offset]);
        changed.extend_from_slice(to);
        changed.extend_from_slice(&bytes[offset + from.len()..]);
        changed
    }

    fn install_matching_payload_digest(bytes: &mut [u8]) {
        let text = std::str::from_utf8(&bytes[..bytes.len() - 1]).expect("UTF-8 mutation");
        let root = Parser::new(text).parse().expect("decodable mutation");
        let envelope = object(&root).expect("object mutation");
        let payload = field(envelope, "payload").expect("payload mutation");
        let digest = sha256::digest(&bytes[payload.span.clone()]).expect("payload digest");
        let hex = sha256::lowercase_hex(&digest);
        let marker = b"\"artifact_id\":\"sha256:";
        let offset = bytes
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("artifact ID marker")
            + marker.len();
        bytes[offset..offset + 64].copy_from_slice(hex.as_bytes());
    }

    fn assert_rejected(bytes: &[u8], expected_code: &str) {
        let mut called = 0;
        let (report, access) = with_verified_backend_input(bytes, |_| called += 1);
        assert!(!report.accepted());
        assert!(access.is_none());
        assert_eq!(called, 0);
        assert_eq!(report.rejections.len(), 1);
        assert_eq!(
            report.rejections[0].code,
            expected_code,
            "{}",
            ir_verify_json(&report)
        );
    }

    fn semantic_mutation(golden: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let mut changed = replace_once(golden, from, to);
        install_matching_payload_digest(&mut changed);
        changed
    }

    fn with_json_escapes(chunks: &[&[u8]]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (index, chunk) in chunks.iter().enumerate() {
            if index != 0 {
                bytes.push(92);
            }
            bytes.extend_from_slice(chunk);
        }
        bytes
    }

    fn swap_ranges(bytes: &[u8], left: Range<usize>, right: Range<usize>) -> Vec<u8> {
        assert!(left.end <= right.start, "swapped rows must not overlap");
        let mut changed = Vec::with_capacity(bytes.len());
        changed.extend_from_slice(&bytes[..left.start]);
        changed.extend_from_slice(&bytes[right.clone()]);
        changed.extend_from_slice(&bytes[left.end..right.start]);
        changed.extend_from_slice(&bytes[left]);
        changed.extend_from_slice(&bytes[right.end..]);
        changed
    }

    fn replace_range(bytes: &[u8], range: Range<usize>, replacement: &[u8]) -> Vec<u8> {
        let mut changed = Vec::with_capacity(bytes.len() - range.len() + replacement.len());
        changed.extend_from_slice(&bytes[..range.start]);
        changed.extend_from_slice(replacement);
        changed.extend_from_slice(&bytes[range.end..]);
        changed
    }

    fn collect_leaf_mutations(node: &JsonNode, artifact: &[u8], mutations: &mut Vec<Vec<u8>>) {
        match &node.value {
            JsonValue::Object(members) => {
                for member in members {
                    collect_leaf_mutations(&member.value, artifact, mutations);
                }
            }
            JsonValue::Array(values) => {
                for value in values {
                    collect_leaf_mutations(value, artifact, mutations);
                }
            }
            JsonValue::String(_) => {
                let raw = &artifact[node.span.clone()];
                let replacement = if raw.len() == 2 || raw.get(1) == Some(&b'\\') {
                    b"\"x\"".to_vec()
                } else {
                    let mut changed = raw.to_vec();
                    changed[1] = if changed[1] == b'x' { b'y' } else { b'x' };
                    changed
                };
                mutations.push(replace_range(artifact, node.span.clone(), &replacement));
            }
            JsonValue::Number(_) => {
                let mut replacement = artifact[node.span.clone()].to_vec();
                replacement[0] = if replacement[0] == b'9' { b'8' } else { b'9' };
                mutations.push(replace_range(artifact, node.span.clone(), &replacement));
            }
            JsonValue::Bool(value) => mutations.push(replace_range(
                artifact,
                node.span.clone(),
                if *value { b"false" } else { b"true" },
            )),
            JsonValue::Null => {
                mutations.push(replace_range(artifact, node.span.clone(), b"\"not-null\""))
            }
        }
    }

    #[test]
    fn canonical_backend_input_corruption_matrix_fails_closed() {
        let fixture = "fixtures/backend_input/minimal_add.backend_input.v0.json";
        let human = crate::parse_cli(vec!["ir-verify".to_string(), fixture.to_string()])
            .expect("default human ir-verify command");
        assert_eq!(human.command, "ir-verify");
        assert_eq!(human.inputs.len(), 1);
        let json = crate::parse_cli(vec![
            "ir-verify".to_string(),
            "--format".to_string(),
            "json".to_string(),
            fixture.to_string(),
        ])
        .expect("JSON ir-verify command");
        assert_eq!(json.command, "ir-verify");
        for rejected in [
            vec!["ir-verify".to_string()],
            vec![
                "ir-verify".to_string(),
                "--timings".to_string(),
                fixture.to_string(),
            ],
            vec![
                "ir-verify".to_string(),
                "--format".to_string(),
                "human".to_string(),
                fixture.to_string(),
            ],
            vec![
                "ir-verify".to_string(),
                "--format=json".to_string(),
                fixture.to_string(),
            ],
            vec![
                "ir-verify".to_string(),
                fixture.to_string(),
                "--format".to_string(),
                "json".to_string(),
            ],
            vec![
                "ir-verify".to_string(),
                fixture.to_string(),
                fixture.to_string(),
            ],
        ] {
            assert!(crate::parse_cli(rejected).is_err());
        }
        let golden = include_bytes!("../fixtures/backend_input/minimal_add.backend_input.v0.json");
        let mut callbacks = 0;
        let (report, result) = with_verified_backend_input(golden, |access| {
            callbacks += 1;
            assert_eq!(access.artifact(), golden);
            assert!(!access.payload().is_empty());
            assert_eq!(access.projection_count(), 1);
        });
        assert!(report.accepted());
        assert!(result.is_some());
        assert_eq!(callbacks, 1);
        assert_eq!(golden.len(), 8_715);
        assert_eq!(
            report.artifact_id.as_deref(),
            Some("sha256:a37707c23cc20a1720e45de901624e3101183a77ec1b5eb4ed55095b5097b82f")
        );
        assert_eq!(report.payload_bytes, 8_582);
        let human_report = ir_verify_text(&report);
        let json_report = ir_verify_json(&report);
        assert_eq!(human_report, ir_verify_text(&report));
        assert_eq!(json_report, ir_verify_json(&report));
        assert!(human_report.starts_with(
            "Hum IR verify\nschema: hum.ir_verify.v0\ntool: ir-verify\nversion: 0.0.1\nstatus: accepted_canonical_backend_input_v0\nartifact_schema: hum.backend_input.v0\n"
        ));
        assert!(json_report.starts_with(
            "{\n  \"schema\": \"hum.ir_verify.v0\",\n  \"tool\": \"ir-verify\",\n  \"version\": \"0.0.1\",\n  \"status\": \"accepted_canonical_backend_input_v0\",\n  \"artifact_schema\": \"hum.backend_input.v0\",\n"
        ));
        assert!(json_report.contains("\"payload_bytes\": 8582,\n    \"source_count\": 1,\n    \"module_count\": 1,\n    \"function_count\": 1,\n    \"block_count\": 1,\n    \"operation_count\": 1,\n    \"expression_count\": 1,\n    \"type_count\": 1,\n    \"definition_count\": 2,\n    \"effect_count\": 1,\n    \"resource_count\": 1,\n    \"failure_edge_count\": 1,\n    \"required_pass_count\": 14,\n    \"unsupported_count\": 0"));
        assert!(json_report.ends_with(
            "\"rejections\": [],\n  \"non_claims_v0\": [\"not_backend_ready_v0\", \"not_executable_v0\", \"not_a_signature_v0\", \"no_durable_authority_v0\"]\n}\n"
        ));
        for forbidden in [
            "VerifiedBackendInput",
            "UnverifiedBackendInputV0",
            "from_verified_parts",
            "payload_range",
            "projection_ranges",
        ] {
            assert!(!human_report.contains(forbidden));
            assert!(!json_report.contains(forbidden));
        }

        assert_rejected(&[], "invalid_framing_v0");
        assert_rejected(&golden[..golden.len() - 1], "invalid_framing_v0");
        let mut crlf = golden.to_vec();
        crlf.pop();
        crlf.extend_from_slice(b"\r\n");
        assert_rejected(&crlf, "invalid_framing_v0");
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(golden);
        assert_rejected(&bom, "invalid_framing_v0");
        let mut trailing = golden.to_vec();
        trailing.push(b'x');
        assert_rejected(&trailing, "invalid_framing_v0");
        let mut trailing_json = golden[..golden.len() - 1].to_vec();
        trailing_json.extend_from_slice(b"x\n");
        assert_rejected(&trailing_json, "trailing_json_bytes_v0");
        let mut invalid_utf8 = golden.to_vec();
        invalid_utf8[0] = 0xff;
        assert_rejected(&invalid_utf8, "invalid_utf8_v0");
        assert_rejected(b"{\"schema\":}\n", "malformed_json_v0");
        let mut invalid_escape = b"{\"schema\":\"bad".to_vec();
        invalid_escape.push(92);
        invalid_escape.extend_from_slice(b"q\"}\n");
        assert_rejected(&invalid_escape, "invalid_escape_v0");
        assert_rejected(b"{\"schema\":\"bad\x01\"}\n", "invalid_control_v0");
        assert_rejected(b"{\"schema\":-0}\n", "invalid_number_v0");
        assert_rejected(b"[]\n", "invalid_envelope_v0");

        let duplicate = replace_once(
            golden,
            b"{\"schema\":\"hum.backend_input.v0\",",
            b"{\"schema\":\"hum.backend_input.v0\",\"schema\":\"hum.backend_input.v0\",",
        );
        assert_rejected(&duplicate, "duplicate_key_v0");

        let envelope_missing = replace_once(golden, b"{\"schema\":\"hum.backend_input.v0\",", b"{");
        assert_rejected(&envelope_missing, "invalid_envelope_v0");
        let envelope_unknown = replace_once(
            golden,
            b"{\"schema\":\"hum.backend_input.v0\",",
            b"{\"schema\":\"hum.backend_input.v0\",\"unknown\":0,",
        );
        assert_rejected(&envelope_unknown, "invalid_envelope_v0");
        let envelope_reordered = {
            let text = std::str::from_utf8(&golden[..golden.len() - 1]).unwrap();
            let root = Parser::new(text).parse().unwrap();
            let members = object(&root).unwrap();
            swap_ranges(
                golden,
                members[0].member_span.clone(),
                members[1].member_span.clone(),
            )
        };
        assert_rejected(&envelope_reordered, "invalid_envelope_v0");

        let wrong_schema = replace_once(golden, b"hum.backend_input.v0", b"hum.backend_input.v1");
        assert_rejected(&wrong_schema, "unsupported_schema_v0");
        let uppercase_id = replace_once(golden, b"sha256:a377", b"sha256:A377");
        assert_rejected(&uppercase_id, "invalid_artifact_id_v0");
        let mut wrong_id = golden.to_vec();
        let start = std::str::from_utf8(&wrong_id)
            .unwrap()
            .find("sha256:")
            .unwrap()
            + 7;
        wrong_id[start..start + 64].fill(b'0');
        assert_rejected(&wrong_id, "artifact_id_mismatch_v0");

        for (from, to, code) in [
            (
                b"\"file_ordinal\":0".as_slice(),
                b"\"file_ordinal\":00".as_slice(),
                "invalid_number_v0",
            ),
            (
                b"\"file_ordinal\":0".as_slice(),
                b"\"file_ordinal\":+0".as_slice(),
                "malformed_json_v0",
            ),
            (
                b"\"file_ordinal\":0".as_slice(),
                b"\"file_ordinal\":-0".as_slice(),
                "invalid_number_v0",
            ),
            (
                b"\"file_ordinal\":0".as_slice(),
                b"\"file_ordinal\":0.0".as_slice(),
                "invalid_number_v0",
            ),
        ] {
            assert_rejected(&replace_once(golden, from, to), code);
        }

        let mut invalid_surrogate = b"\"display_name\":\"a".to_vec();
        invalid_surrogate.push(92);
        invalid_surrogate.extend_from_slice(b"ud800d\"");
        assert_rejected(
            &replace_once(golden, b"\"display_name\":\"add\"", &invalid_surrogate),
            "invalid_escape_v0",
        );

        for (from, to) in [
            (
                b"\"display_name\":\"add\"".as_slice(),
                with_json_escapes(&[b"\"display_name\":\"", b"u0061dd\""]),
            ),
            (
                b"examples/core/minimal_add.hum".as_slice(),
                with_json_escapes(&[b"examples", b"u002Fcore/minimal_add.hum"]),
            ),
            (
                b"\"display_name\":\"add\"".as_slice(),
                with_json_escapes(&[b"\"display_name\":\"a", b"ud83d", b"ude00d\""]),
            ),
        ] {
            let mut noncanonical = replace_once(golden, from, &to);
            install_matching_payload_digest(&mut noncanonical);
            assert_rejected(&noncanonical, "noncanonical_bytes_v0");
        }
        let null = semantic_mutation(golden, b"\"file_ordinal\":0", b"\"file_ordinal\":null");
        assert_rejected(&null, "semantic_model_mismatch_v0");

        for (from, to) in [
            (
                b"hum.ir_contract.v0".as_slice(),
                b"hum.ir_contract.v1".as_slice(),
            ),
            (
                b"hum.canonical_minimal_add_backend_facts.v0".as_slice(),
                b"hum.canonical_minimal_add_backend_facts.v1".as_slice(),
            ),
            (
                b"target_independent_checked_i64_v0".as_slice(),
                b"target_independent_checked_i64_v1".as_slice(),
            ),
            (
                b"examples/core/minimal_add.hum".as_slice(),
                b"examples/core/minimal_sub.hum".as_slice(),
            ),
            (
                b"module:examples.core.minimal_add".as_slice(),
                b"module:examples.core.minimal_sub".as_slice(),
            ),
            (
                b"\"id\":\"function:0\",\"source_item_id\"".as_slice(),
                b"\"id\":\"function:1\",\"source_item_id\"".as_slice(),
            ),
            (
                b"\"display_name\":\"add\"".as_slice(),
                b"\"display_name\":\"sub\"".as_slice(),
            ),
            (b"hum_internal_v0".as_slice(), b"hum_external_v0".as_slice()),
            (
                b"hum_checked_trap_v0".as_slice(),
                b"hum_checked_wrap_v0".as_slice(),
            ),
            (
                b"\"kind\":\"return\"".as_slice(),
                b"\"kind\":\"branch\"".as_slice(),
            ),
            (
                b"\"operator\":\"checked_add\"".as_slice(),
                b"\"operator\":\"checked_sub\"".as_slice(),
            ),
            (
                b"hum-type:builtin:Int".as_slice(),
                b"hum-type:builtin:UInt".as_slice(),
            ),
            (
                b"\"signed\":true".as_slice(),
                b"\"signed\":false".as_slice(),
            ),
            (
                b"\"effect_id\":\"effect:function:0:0\"".as_slice(),
                b"\"effect_id\":\"effect:function:0:1\"".as_slice(),
            ),
            (
                b"\"resource_id\":\"resource:function:0:0\"".as_slice(),
                b"\"resource_id\":\"resource:function:0:1\"".as_slice(),
            ),
            (
                b"\"failure_edge_id\":\"failure-edge:function:0:0\"".as_slice(),
                b"\"failure_edge_id\":\"failure-edge:function:0:1\"".as_slice(),
            ),
            (
                b"runtime_trap_on_overflow".as_slice(),
                b"runtime_wrap_on_overflow".as_slice(),
            ),
            (
                b"\"profile\":\"normal\"".as_slice(),
                b"\"profile\":\"strict\"".as_slice(),
            ),
            (
                b"\"name\":\"profile_check\"".as_slice(),
                b"\"name\":\"profile_skip_\"".as_slice(),
            ),
            (
                b"\"name\":\"profile_check\",\"status\":\"passed\"".as_slice(),
                b"\"name\":\"profile_check\",\"status\":\"failed\"".as_slice(),
            ),
        ] {
            let corrupted = semantic_mutation(golden, from, to);
            assert_rejected(&corrupted, "semantic_model_mismatch_v0");
        }

        let missing = semantic_mutation(golden, b"\"item_kind\":\"task\",", b"");
        assert_rejected(&missing, "semantic_model_mismatch_v0");
        let unknown = semantic_mutation(
            golden,
            b"\"item_kind\":\"task\",",
            b"\"item_kind\":\"task\",\"unknown\":\"closed\",",
        );
        assert_rejected(&unknown, "semantic_model_mismatch_v0");
        let reordered = semantic_mutation(
            golden,
            b"\"compiler\":{\"version\":\"0.0.1\",\"ir_schema\":\"hum.ir_contract.v0\"",
            b"\"compiler\":{\"ir_schema\":\"hum.ir_contract.v0\",\"version\":\"0.0.1\"",
        );
        assert_rejected(&reordered, "semantic_model_mismatch_v0");
        let (payload_start, first_member, second_member) = {
            let text = std::str::from_utf8(&golden[..golden.len() - 1]).unwrap();
            let root = Parser::new(text).parse().unwrap();
            let payload = field(object(&root).unwrap(), "payload").unwrap();
            let members = object(payload).unwrap();
            (
                payload.span.start,
                members[0].member_span.clone(),
                members[1].member_span.clone(),
            )
        };
        let mut payload_missing =
            replace_range(golden, first_member.start..second_member.start, b"");
        install_matching_payload_digest(&mut payload_missing);
        assert_rejected(&payload_missing, "semantic_model_mismatch_v0");
        let mut payload_unknown = replace_range(
            golden,
            payload_start + 1..payload_start + 1,
            b"\"unknown\":0,",
        );
        install_matching_payload_digest(&mut payload_unknown);
        assert_rejected(&payload_unknown, "semantic_model_mismatch_v0");
        let mut payload_reordered =
            swap_ranges(golden, first_member.clone(), second_member.clone());
        install_matching_payload_digest(&mut payload_reordered);
        assert_rejected(&payload_reordered, "semantic_model_mismatch_v0");

        let mut every_payload_scalar = Vec::new();
        {
            let text = std::str::from_utf8(&golden[..golden.len() - 1]).unwrap();
            let root = Parser::new(text).parse().unwrap();
            let payload = field(object(&root).unwrap(), "payload").unwrap();
            collect_leaf_mutations(payload, golden, &mut every_payload_scalar);
        }
        assert!(
            every_payload_scalar.len() >= 100,
            "the complete scalar matrix must remain substantial"
        );
        for mut corrupted in every_payload_scalar {
            install_matching_payload_digest(&mut corrupted);
            assert_rejected(&corrupted, "semantic_model_mismatch_v0");
        }
        for (from, to) in [
            (
                b"\"effects\":[]".as_slice(),
                b"\"effects\":[\"x\"]".as_slice(),
            ),
            (
                b"\"external_authority\":[]".as_slice(),
                b"\"external_authority\":[\"x\"]".as_slice(),
            ),
            (
                b"\"allocations\":[]".as_slice(),
                b"\"allocations\":[\"x\"]".as_slice(),
            ),
            (b"\"moves\":[]".as_slice(), b"\"moves\":[\"x\"]".as_slice()),
            (
                b"\"borrows\":[]".as_slice(),
                b"\"borrows\":[\"x\"]".as_slice(),
            ),
            (
                b"\"aliases\":[]".as_slice(),
                b"\"aliases\":[\"x\"]".as_slice(),
            ),
            (
                b"\"ownership_transfers\":[]".as_slice(),
                b"\"ownership_transfers\":[\"x\"]".as_slice(),
            ),
            (
                b"\"contract_predicates\":[]".as_slice(),
                b"\"contract_predicates\":[\"x\"]".as_slice(),
            ),
            (
                b"\"evidence_obligations\":[]".as_slice(),
                b"\"evidence_obligations\":[\"x\"]".as_slice(),
            ),
            (
                b"\"unsupported\":[],\"source_provenance\"".as_slice(),
                b"\"unsupported\":[\"x\"],\"source_provenance\"".as_slice(),
            ),
            (
                b"}],\"unsupported\":[]}}".as_slice(),
                b"}],\"unsupported\":[\"x\"]}}".as_slice(),
            ),
        ] {
            let corrupted = semantic_mutation(golden, from, to);
            assert_rejected(&corrupted, "semantic_model_mismatch_v0");
        }

        let profile_pass =
            b"{\"name\":\"profile_check\",\"status\":\"passed\",\"selected\":1,\"ordinal\":13}";
        let missing_pass = semantic_mutation(
            golden,
            b",{\"name\":\"profile_check\",\"status\":\"passed\",\"selected\":1,\"ordinal\":13}",
            b"",
        );
        assert_rejected(&missing_pass, "semantic_model_mismatch_v0");
        let duplicate_pass = semantic_mutation(
            golden,
            profile_pass,
            b"{\"name\":\"profile_check\",\"status\":\"passed\",\"selected\":1,\"ordinal\":13},{\"name\":\"profile_check\",\"status\":\"passed\",\"selected\":1,\"ordinal\":13}",
        );
        assert_rejected(&duplicate_pass, "semantic_model_mismatch_v0");
        for (from, to) in [
            (
                b"\"name\":\"profile_check\"".as_slice(),
                b"\"name\":\"foreign_check\"".as_slice(),
            ),
            (
                b"\"name\":\"profile_check\",\"status\":\"passed\"".as_slice(),
                b"\"name\":\"profile_check\",\"status\":\"skipped\"".as_slice(),
            ),
            (
                b"\"name\":\"profile_check\",\"status\":\"passed\",\"selected\":1".as_slice(),
                b"\"name\":\"profile_check\",\"status\":\"passed\",\"selected\":0".as_slice(),
            ),
            (
                b"\"name\":\"profile_check\",\"status\":\"passed\"".as_slice(),
                b"\"name\":\"profile_check\",\"status\":\"unimplemented\"".as_slice(),
            ),
        ] {
            let corrupted = semantic_mutation(golden, from, to);
            assert_rejected(&corrupted, "semantic_model_mismatch_v0");
        }

        let (child_definition_ranges, pass_ranges) = {
            let text = std::str::from_utf8(&golden[..golden.len() - 1]).unwrap();
            let root = Parser::new(text).parse().unwrap();
            let payload = object(field(object(&root).unwrap(), "payload").unwrap()).unwrap();
            let function =
                object(single(array(field(payload, "functions").unwrap()).unwrap()).unwrap())
                    .unwrap();
            let expression =
                object(single(array(field(function, "expressions").unwrap()).unwrap()).unwrap())
                    .unwrap();
            let children = array(field(expression, "children").unwrap()).unwrap();
            let passes = array(field(function, "required_passes").unwrap()).unwrap();
            (
                [
                    field(object(&children[0]).unwrap(), "definition_id")
                        .unwrap()
                        .span
                        .clone(),
                    field(object(&children[1]).unwrap(), "definition_id")
                        .unwrap()
                        .span
                        .clone(),
                ],
                [passes[12].span.clone(), passes[13].span.clone()],
            )
        };
        let mut reordered_pass =
            swap_ranges(golden, pass_ranges[0].clone(), pass_ranges[1].clone());
        install_matching_payload_digest(&mut reordered_pass);
        assert_rejected(&reordered_pass, "semantic_model_mismatch_v0");
        let mut cross_wired = swap_ranges(
            golden,
            child_definition_ranges[0].clone(),
            child_definition_ranges[1].clone(),
        );
        install_matching_payload_digest(&mut cross_wired);
        assert_rejected(&cross_wired, "semantic_model_mismatch_v0");

        let spaced = semantic_mutation(
            golden,
            b"\"target_context\":\"target_independent_checked_i64_v0\"},\"source_revision\"",
            b"\"target_context\":\"target_independent_checked_i64_v0\"}, \"source_revision\"",
        );
        assert_rejected(&spaced, "noncanonical_bytes_v0");
        let escaped_path = with_json_escapes(&[
            b"\"normalized_path\":\"examples",
            b"/core/minimal_add.hum\"",
        ]);
        let escaped_slash = semantic_mutation(
            golden,
            b"\"normalized_path\":\"examples/core/minimal_add.hum\"",
            &escaped_path,
        );
        assert_rejected(&escaped_slash, "noncanonical_bytes_v0");
    }

    #[test]
    fn verified_backend_input_is_byte_bound_and_compiler_sealed() {
        let golden = include_bytes!("../fixtures/backend_input/minimal_add.backend_input.v0.json");
        let (report, observed) = with_verified_backend_input(golden, |access| {
            (
                access.artifact().as_ptr(),
                access.payload().as_ptr(),
                access.projection_count(),
            )
        });
        assert!(report.accepted());
        let (artifact, payload, projections) = observed.unwrap();
        assert_eq!(artifact, golden.as_ptr());
        assert!(payload > artifact);
        assert_eq!(projections, 1);
        assert!(!std::any::type_name::<VerifiedBackendInput<'_>>().is_empty());

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let target = std::env::temp_dir().join(format!(
            "hum-wo20-verified-backend-input-{}-{nonce}",
            std::process::id()
        ));
        let cargo = std::env::var_os("CARGO")
            .or_else(|| option_env!("CARGO").map(std::ffi::OsString::from))
            .unwrap_or_else(|| std::ffi::OsString::from("cargo"));
        let output = std::process::Command::new(cargo)
            .args(["check", "--all-targets"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .env("CARGO_TARGET_DIR", &target)
            .env(
                "RUSTFLAGS",
                "--cfg hum_compile_fail_verified_backend_input_authority",
            )
            .output()
            .expect("spawn actual-type capability privacy proof");
        let _ = std::fs::remove_dir_all(&target);
        assert_eq!(output.status.code(), Some(101));
        let stderr = String::from_utf8(output.stderr).expect("compiler diagnostics are UTF-8");
        assert!(stderr.contains("associated function `from_verified_parts` is private"));
        assert!(stderr.contains("field `artifact` of struct `VerifiedBackendInput` is private"));
        assert!(stderr.contains("lifetime may not live long enough") || stderr.contains("E0521"));
        for forbidden in [
            "error[E0382]",
            "unexpected `cfg`",
            "unresolved import",
            "cannot find type",
            "cannot find module",
        ] {
            assert!(
                !stderr.contains(forbidden),
                "unrelated proof failure: {forbidden}"
            );
        }
    }
}
