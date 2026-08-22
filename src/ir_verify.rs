use crate::ast::Program;
use crate::backend_input;
use crate::diagnostic::Diagnostic;
use crate::{sha256, version};
use std::fmt::Write as _;
use std::marker::PhantomData;
use std::ops::Range;

pub const IR_VERIFY_SCHEMA: &str = "hum.ir_verify.v0";
const CANONICAL_ARTIFACT_ID: &str =
    "sha256:a37707c23cc20a1720e45de901624e3101183a77ec1b5eb4ed55095b5097b82f";

const NON_CLAIMS: [&str; 4] = [
    "not_backend_ready_v0",
    "not_executable_v0",
    "not_a_signature_v0",
    "no_durable_authority_v0",
];

#[allow(unexpected_cfgs)]
mod retired_issue_assembled_compile_proof {
    #[cfg(hum_compile_fail_canonical_minimal_add_backend_facts_escape)]
    fn retired_issue_assembled_must_not_compile() {
        crate::backend_input::issue_assembled();
    }
}

#[derive(Clone)]
struct VerifiedProjection {
    artifact_id: String,
    payload_digest: String,
    compiler_version: String,
    semantic_contract: String,
    target_context: String,
    source_revision: String,
    source_path: String,
    function_id: String,
    linkage_kind: String,
    linkage_symbol: String,
    parameter_value_ids: [String; 2],
    parameter_definition_ids: [String; 2],
    parameter_definition_source_identities: [String; 2],
    parameter_types: [String; 2],
    parameter_spans: [(usize, usize); 2],
    operation_id: String,
    operation_source_identity: String,
    result_value_id: String,
    result_type: String,
    operation_span: (usize, usize),
    overflow_value_type: String,
    overflow_operation: String,
    overflow_behavior: String,
    profile: String,
    required_passes: Vec<String>,
}

impl VerifiedProjection {
    fn logical_identity(&self) -> LogicalIdentity {
        LogicalIdentity {
            source_revision: self.source_revision.clone(),
            source_path: self.source_path.clone(),
            required_passes: self
                .required_passes
                .iter()
                .enumerate()
                .map(|(ordinal, name)| format!("{name}@file:0:ordinal:{ordinal}"))
                .collect(),
            function_id: self.function_id.clone(),
            linkage_kind: self.linkage_kind.clone(),
            linkage_symbol: self.linkage_symbol.clone(),
            parameter_value_ids: self.parameter_value_ids.clone(),
            parameter_definition_ids: self.parameter_definition_ids.clone(),
            parameter_definition_source_identities: self
                .parameter_definition_source_identities
                .clone(),
            parameter_types: self.parameter_types.clone(),
            parameter_spans: self.parameter_spans,
            operation_id: self.operation_id.clone(),
            operation_source_identity: self.operation_source_identity.clone(),
            result_value_id: self.result_value_id.clone(),
            result_type: self.result_type.clone(),
            operation_span: self.operation_span,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct LogicalIdentity {
    source_revision: String,
    source_path: String,
    required_passes: Vec<String>,
    function_id: String,
    linkage_kind: String,
    linkage_symbol: String,
    parameter_value_ids: [String; 2],
    parameter_definition_ids: [String; 2],
    parameter_definition_source_identities: [String; 2],
    parameter_types: [String; 2],
    parameter_spans: [(usize, usize); 2],
    operation_id: String,
    operation_source_identity: String,
    result_value_id: String,
    result_type: String,
    operation_span: (usize, usize),
}

pub(crate) struct LiveIdentityRequest<'expected> {
    expected: Option<&'expected LogicalIdentity>,
    observed: Option<LogicalIdentity>,
    program_identity: usize,
}

impl<'expected> LiveIdentityRequest<'expected> {
    fn new(expected: &'expected LogicalIdentity, program: &Program) -> Self {
        Self {
            expected: Some(expected),
            observed: None,
            program_identity: std::ptr::from_ref(program).addr(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn observe(
        &mut self,
        source_revision: &str,
        source_path: &str,
        required_passes: Vec<String>,
        function_id: &str,
        linkage_kind: &str,
        linkage_symbol: &str,
        parameter_value_ids: [String; 2],
        parameter_definition_ids: [String; 2],
        parameter_definition_source_identities: [String; 2],
        parameter_types: [&str; 2],
        parameter_spans: [(usize, usize); 2],
        operation_id: String,
        operation_source_identity: String,
        result_value_id: String,
        result_type: &str,
        operation_span: (usize, usize),
    ) {
        assert!(
            self.observed.is_none(),
            "live identity observed more than once"
        );
        let mut observed = LogicalIdentity {
            source_revision: source_revision.to_string(),
            source_path: source_path.to_string(),
            required_passes,
            function_id: function_id.to_string(),
            linkage_kind: linkage_kind.to_string(),
            linkage_symbol: linkage_symbol.to_string(),
            parameter_value_ids,
            parameter_definition_ids,
            parameter_definition_source_identities,
            parameter_types: parameter_types.map(str::to_string),
            parameter_spans,
            operation_id,
            operation_source_identity,
            result_value_id,
            result_type: result_type.to_string(),
            operation_span,
        };
        mix_authenticated_live_identity_for_test(&mut observed, self.program_identity);
        self.observed = Some(observed);
    }

    fn matches(&self) -> bool {
        self.expected
            .zip(self.observed.as_ref())
            .is_some_and(|(expected, observed)| {
                identities_match_for_test(expected, observed, self.program_identity)
            })
    }
}

pub(crate) struct VerifiedBackendInput<'artifact> {
    projection: VerifiedProjection,
    _artifact: PhantomData<&'artifact [u8]>,
}

impl VerifiedBackendInput<'_> {
    pub(crate) fn schema(&self) -> &'static str {
        backend_input::BACKEND_INPUT_SCHEMA
    }
    pub(crate) fn artifact_id(&self) -> &str {
        &self.projection.artifact_id
    }
    pub(crate) fn compiler_version(&self) -> &str {
        &self.projection.compiler_version
    }
    pub(crate) fn semantic_contract(&self) -> &str {
        &self.projection.semantic_contract
    }
    pub(crate) fn target_context(&self) -> &str {
        &self.projection.target_context
    }
    pub(crate) fn source_revision(&self) -> &str {
        &self.projection.source_revision
    }
    pub(crate) fn source_path(&self) -> &str {
        &self.projection.source_path
    }
    pub(crate) fn function_id(&self) -> &str {
        &self.projection.function_id
    }
    pub(crate) fn linkage(&self) -> (&str, &str) {
        (
            &self.projection.linkage_kind,
            &self.projection.linkage_symbol,
        )
    }
    pub(crate) fn parameter_value_ids(&self) -> [&str; 2] {
        self.projection
            .parameter_value_ids
            .each_ref()
            .map(String::as_str)
    }
    pub(crate) fn parameter_definition_ids(&self) -> [&str; 2] {
        self.projection
            .parameter_definition_ids
            .each_ref()
            .map(String::as_str)
    }
    pub(crate) fn parameter_types(&self) -> [&str; 2] {
        self.projection
            .parameter_types
            .each_ref()
            .map(String::as_str)
    }
    pub(crate) fn parameter_spans(&self) -> [(usize, usize); 2] {
        self.projection.parameter_spans
    }
    pub(crate) fn operation_id(&self) -> &str {
        &self.projection.operation_id
    }
    pub(crate) fn result(&self) -> (&str, &str) {
        (
            &self.projection.result_value_id,
            &self.projection.result_type,
        )
    }
    pub(crate) fn operation_span(&self) -> (usize, usize) {
        self.projection.operation_span
    }
    pub(crate) fn overflow_edge(&self) -> (&str, &str, &str) {
        (
            &self.projection.overflow_value_type,
            &self.projection.overflow_operation,
            &self.projection.overflow_behavior,
        )
    }
    pub(crate) fn profile(&self) -> &str {
        &self.projection.profile
    }
    pub(crate) fn required_passes(&self) -> impl ExactSizeIterator<Item = &str> {
        self.projection.required_passes.iter().map(String::as_str)
    }
}

pub(crate) struct IrVerifyReport {
    status: &'static str,
    artifact_id: Option<String>,
    payload_digest: Option<String>,
    facts: ReportFacts,
    rejected_check: Option<&'static str>,
    findings: Vec<Finding>,
}

#[derive(Clone, Default)]
struct ReportFacts {
    semantic_contract: Option<String>,
    compiler_version: Option<String>,
    target_context: Option<String>,
    source_revision: Option<String>,
    task_count: Option<usize>,
    function_count: Option<usize>,
    operation_count: Option<usize>,
    ordered_pass_count: Option<usize>,
}

impl ReportFacts {
    fn accepted(projection: &VerifiedProjection) -> Self {
        Self {
            semantic_contract: Some(projection.semantic_contract.clone()),
            compiler_version: Some(projection.compiler_version.clone()),
            target_context: Some(projection.target_context.clone()),
            source_revision: Some(projection.source_revision.clone()),
            task_count: Some(1),
            function_count: Some(1),
            operation_count: Some(1),
            ordered_pass_count: Some(projection.required_passes.len()),
        }
    }
}

struct Finding {
    code: &'static str,
    byte_offset: Option<usize>,
    logical_path: &'static str,
    reason: &'static str,
}

impl IrVerifyReport {
    pub(crate) fn accepted(&self) -> bool {
        self.findings.is_empty()
    }
}

#[derive(Debug)]
struct Failure {
    row: &'static str,
    code: &'static str,
    path: &'static str,
    reason: &'static str,
    offset: Option<usize>,
}

impl Failure {
    fn at(
        row: &'static str,
        code: &'static str,
        path: &'static str,
        reason: &'static str,
        offset: usize,
    ) -> Self {
        Self {
            row,
            code,
            path,
            reason,
            offset: Some(offset),
        }
    }
    fn without_offset(
        row: &'static str,
        code: &'static str,
        path: &'static str,
        reason: &'static str,
    ) -> Self {
        Self {
            row,
            code,
            path,
            reason,
            offset: None,
        }
    }
}

fn rejected(
    failure: Failure,
    artifact_id: Option<String>,
    payload_digest: Option<String>,
    facts: ReportFacts,
) -> IrVerifyReport {
    IrVerifyReport {
        status: "rejected_backend_input_v0",
        artifact_id,
        payload_digest,
        facts,
        rejected_check: Some(failure.row),
        findings: vec![Finding {
            code: failure.code,
            byte_offset: failure.offset,
            logical_path: failure.path,
            reason: failure.reason,
        }],
    }
}

fn accepted(projection: &VerifiedProjection) -> IrVerifyReport {
    IrVerifyReport {
        status: "accepted_canonical_backend_input_v0",
        artifact_id: Some(projection.artifact_id.clone()),
        payload_digest: Some(projection.payload_digest.clone()),
        facts: ReportFacts::accepted(projection),
        rejected_check: None,
        findings: Vec::new(),
    }
}

pub(crate) fn verify_backend_input(artifact: &[u8]) -> IrVerifyReport {
    match verify_bytes(artifact) {
        Ok(projection) => accepted(&projection),
        Err(error) => {
            let (failure, artifact_id, digest, facts) = *error;
            rejected(failure, artifact_id, digest, facts)
        }
    }
}

pub(crate) fn with_verified_backend_input<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    artifact: &[u8],
    consume: impl for<'artifact> FnOnce(VerifiedBackendInput<'artifact>) -> R,
) -> (IrVerifyReport, Option<R>) {
    let projection = match verify_bytes(artifact) {
        Ok(projection) => projection,
        Err(error) => {
            let (failure, artifact_id, digest, facts) = *error;
            return (rejected(failure, artifact_id, digest, facts), None);
        }
    };
    let expected_identity = projection.logical_identity();
    let mut request = LiveIdentityRequest::new(&expected_identity, program);
    if !backend_input::bind_canonical_minimal_add_live_identity(&mut request, program, diagnostics)
    {
        return (
            rejected(
                Failure::without_offset(
                    "A-R09",
                    "live_cross_binding_mismatch_v0",
                    "$.payload",
                    "live canonical backend facts unavailable",
                ),
                Some(projection.artifact_id.clone()),
                Some(projection.payload_digest.clone()),
                ReportFacts::accepted(&projection),
            ),
            None,
        );
    }
    if check_enabled("A-R09") && !request.matches() {
        return (
            rejected(
                Failure::without_offset(
                    "A-R09",
                    "live_cross_binding_mismatch_v0",
                    "$.payload",
                    "decoded logical identities disagree with live typed facts",
                ),
                Some(projection.artifact_id.clone()),
                Some(projection.payload_digest.clone()),
                ReportFacts::accepted(&projection),
            ),
            None,
        );
    }
    fn issue<'artifact, R>(
        _program: &'artifact Program,
        _artifact: &'artifact [u8],
        projection: VerifiedProjection,
        consume: impl FnOnce(VerifiedBackendInput<'artifact>) -> R,
    ) -> R {
        consume(VerifiedBackendInput {
            projection,
            _artifact: PhantomData,
        })
    }
    let report = accepted(&projection);
    let value = issue(program, artifact, projection, consume);
    (report, Some(value))
}

type VerifyError = (Failure, Option<String>, Option<String>, ReportFacts);
type VerifyResult = Result<VerifiedProjection, Box<VerifyError>>;

fn verify_bytes(artifact: &[u8]) -> VerifyResult {
    let unbound = |failure| Box::new((failure, None, None, ReportFacts::default()));
    if artifact.is_empty() {
        return Err(unbound(Failure::without_offset(
            "A-R01",
            "invalid_framing_v0",
            "$",
            "artifact is empty",
        )));
    }
    if artifact.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(unbound(Failure::at(
            "A-R01",
            "invalid_framing_v0",
            "$",
            "UTF-8 BOM forbidden",
            0,
        )));
    }
    if artifact.contains(&b'\r') || !artifact.ends_with(b"\n") || artifact.ends_with(b"\n\n") {
        return Err(unbound(Failure::without_offset(
            "A-R01",
            "invalid_framing_v0",
            "$",
            "exactly one final LF and no CR bytes required",
        )));
    }
    let body = std::str::from_utf8(&artifact[..artifact.len() - 1]).map_err(|error| {
        unbound(Failure::at(
            "A-R01",
            "invalid_utf8_v0",
            "$",
            "invalid UTF-8",
            error.valid_up_to(),
        ))
    })?;
    let root = Parser::new(body).parse().map_err(|error| {
        unbound(Failure::at(
            "A-R01",
            error.code,
            "$",
            error.reason,
            error.offset,
        ))
    })?;
    if let Some(offset) = first_duplicate_key_offset(&root) {
        return Err(unbound(Failure::at(
            "A-R01",
            "duplicate_key_v0",
            "$",
            "duplicate object key",
            offset,
        )));
    }
    let envelope = exact_object(&root, &["schema", "artifact_id", "payload"], "A-R01", "$")
        .map_err(&unbound)?;
    if text_field(envelope, "schema", "A-R01").map_err(unbound)?
        != backend_input::BACKEND_INPUT_SCHEMA
    {
        return Err(unbound(Failure::at(
            "A-R04",
            "identity_mismatch_v0",
            "$.schema",
            "backend-input schema differs",
            root.span.start,
        )));
    }
    let declared_id = text_field(envelope, "artifact_id", "A-R03")
        .map_err(unbound)?
        .to_string();
    if !valid_sha256_id(&declared_id) {
        return Err(Box::new((
            Failure::without_offset(
                "A-R03",
                "declared_artifact_id_mismatch_v0",
                "$.artifact_id",
                "declared artifact ID must be lowercase sha256",
            ),
            Some(declared_id),
            None,
            ReportFacts::default(),
        )));
    }
    let payload = field(envelope, "payload", "A-R01").map_err(|failure| {
        Box::new((
            failure,
            Some(declared_id.clone()),
            None,
            ReportFacts::default(),
        ))
    })?;
    let digest = sha256::digest(&artifact[payload.span.clone()]).ok_or_else(|| {
        Box::new((
            Failure::without_offset(
                "A-R02",
                "payload_digest_unavailable_v0",
                "$.payload",
                "payload digest unavailable",
            ),
            Some(declared_id.clone()),
            None,
            ReportFacts::default(),
        ))
    })?;
    let payload_digest = format!("sha256:{}", sha256::lowercase_hex(&digest));
    let mut canonical = String::with_capacity(body.len() + 1);
    emit_json(&root, &mut canonical);
    canonical.push('\n');
    if check_enabled("A-R01") && canonical.as_bytes() != artifact {
        return Err(Box::new((
            Failure::without_offset(
                "A-R01",
                "noncanonical_bytes_v0",
                "$",
                "parse and canonical re-encoding differ",
            ),
            Some(declared_id),
            Some(payload_digest),
            ReportFacts::default(),
        )));
    }
    let mut facts = ReportFacts::default();
    let decoded = decode_projection(
        payload,
        declared_id.clone(),
        payload_digest.clone(),
        &mut facts,
    );
    if payload_digest != declared_id {
        if check_enabled("A-R02") && declared_id == CANONICAL_ARTIFACT_ID {
            return Err(Box::new((
                Failure::without_offset(
                    "A-R02",
                    "payload_digest_mismatch_v0",
                    "$.payload",
                    "changed authenticated payload bytes retain a foreign digest",
                ),
                Some(declared_id),
                Some(payload_digest),
                ReportFacts::default(),
            )));
        }
        return match decoded {
            Ok(projection) if !check_enabled("A-R03") => Ok(projection),
            Ok(_) => Err(Box::new((
                Failure::without_offset(
                    "A-R03",
                    "declared_artifact_id_mismatch_v0",
                    "$.artifact_id",
                    "declared artifact ID disagrees with unchanged valid payload bytes",
                ),
                Some(declared_id),
                Some(payload_digest),
                ReportFacts::default(),
            ))),
            Err(failure) => Err(Box::new((
                failure,
                Some(declared_id),
                Some(payload_digest),
                ReportFacts::default(),
            ))),
        };
    }
    decoded.map_err(|failure| Box::new((failure, Some(declared_id), Some(payload_digest), facts)))
}

fn decode_projection(
    payload: &JsonNode,
    artifact_id: String,
    payload_digest: String,
    facts: &mut ReportFacts,
) -> Result<VerifiedProjection, Failure> {
    let p = exact_object(
        payload,
        &[
            "compiler",
            "source_revision",
            "module",
            "functions",
            "types",
            "definitions",
            "effects",
            "resources",
            "failure_edges",
            "unsupported",
        ],
        "A-R06",
        "$.payload",
    )?;
    let compiler = exact_object(
        field(p, "compiler", "A-R04")?,
        &[
            "version",
            "ir_schema",
            "semantic_contract",
            "feature_set",
            "target_context",
        ],
        "A-R04",
        "$.payload.compiler",
    )?;
    let source = exact_object(
        field(p, "source_revision", "A-R04")?,
        &["id", "sha256", "file_ordinal", "normalized_path"],
        "A-R04",
        "$.payload.source_revision",
    )?;
    let module = exact_object(
        field(p, "module", "A-R04")?,
        &["id", "name", "files"],
        "A-R04",
        "$.payload.module",
    )?;
    let functions = array(field(p, "functions", "A-R06")?, "A-R06")?;
    let function = exact_object(
        one(functions, "A-R06", "$.payload.functions")?,
        &[
            "id",
            "source_item_id",
            "display_name",
            "item_kind",
            "linkage",
            "source_span",
            "abi",
            "blocks",
            "expressions",
            "required_passes",
        ],
        "A-R06",
        "$.payload.functions[0]",
    )?;
    let linkage = exact_object(
        field(function, "linkage", "A-R06")?,
        &["kind", "symbol"],
        "A-R06",
        "$.payload.functions[0].linkage",
    )?;
    let function_span = span(field(function, "source_span", "A-R06")?, "A-R06")?;
    let abi = exact_object(
        field(function, "abi", "A-R06")?,
        &[
            "calling_convention",
            "parameters",
            "parameter_types",
            "result",
            "result_type",
            "integer_width",
            "trap_convention",
        ],
        "A-R06",
        "$.payload.functions[0].abi",
    )?;
    let parameters = string_array(field(abi, "parameters", "A-R06")?, "A-R06")?;
    let parameter_types = string_array(field(abi, "parameter_types", "A-R06")?, "A-R06")?;
    let blocks = array(field(function, "blocks", "A-R06")?, "A-R06")?;
    let block = exact_object(
        one(blocks, "A-R06", "$.payload.functions[0].blocks")?,
        &["id", "operations"],
        "A-R06",
        "$.payload.functions[0].blocks[0]",
    )?;
    let operations = array(field(block, "operations", "A-R06")?, "A-R06")?;
    let operation = exact_object(
        one(
            operations,
            "A-R06",
            "$.payload.functions[0].blocks[0].operations",
        )?,
        &[
            "id",
            "section_id",
            "kind",
            "statement_id",
            "expression_id",
            "result_value_id",
            "source_span",
        ],
        "A-R06",
        "$.payload.functions[0].blocks[0].operations[0]",
    )?;
    let operation_span = span(field(operation, "source_span", "A-R06")?, "A-R06")?;
    let expressions = array(field(function, "expressions", "A-R06")?, "A-R06")?;
    let expression = exact_object(
        one(expressions, "A-R06", "$.payload.functions[0].expressions")?,
        &[
            "id",
            "kind",
            "operator",
            "children",
            "result_value_id",
            "checked_type_id",
            "effect_id",
            "resource_id",
            "failure_edge_id",
            "unsupported",
            "source_provenance",
        ],
        "A-R06",
        "$.payload.functions[0].expressions[0]",
    )?;
    let provenance = exact_object(
        field(expression, "source_provenance", "A-R06")?,
        &["source_id", "statement_id", "line", "column"],
        "A-R06",
        "$.payload.functions[0].expressions[0].source_provenance",
    )?;
    let children = array(field(expression, "children", "A-R06")?, "A-R06")?;
    let definitions = array(field(p, "definitions", "A-R06")?, "A-R06")?;
    if parameters.len() != 2
        || parameter_types.len() != 2
        || children.len() != 2
        || definitions.len() != 2
    {
        return Err(Failure::without_offset(
            "A-R06",
            "structure_mismatch_v0",
            "$.payload.functions[0]",
            "exactly two ordered parameters, children, and definitions required",
        ));
    }
    let mut value_ids = Vec::new();
    let mut definition_ids = Vec::new();
    let mut definition_source_identities = Vec::new();
    let mut definition_spans = Vec::new();
    for index in 0..2 {
        let child = exact_object(
            &children[index],
            &["ordinal", "node_id", "value_id", "definition_id"],
            "A-R06",
            "$.payload.functions[0].expressions[0].children[]",
        )?;
        let definition = exact_object(
            &definitions[index],
            &[
                "id",
                "semantic_id",
                "kind",
                "ordinal",
                "value_id",
                "type_id",
                "source_span",
            ],
            "A-R06",
            "$.payload.definitions[]",
        )?;
        let value = text_field(definition, "value_id", "A-R06")?;
        let definition_id = text_field(definition, "id", "A-R06")?;
        let expected_node = format!(
            "parser-body:resolver-item:file-0:path-0:statement-0:expression-0:binary-{}",
            if index == 0 { "left" } else { "right" }
        );
        let expected_definition = format!(
            "def_{}_parameter_{}",
            index + 1,
            if index == 0 { "a" } else { "b" }
        );
        require(
            "A-R06",
            usize_field(child, "ordinal", "A-R06")? == index
                && text_field(child, "node_id", "A-R06")? == expected_node
                && text_field(child, "value_id", "A-R06")? == value
                && text_field(child, "definition_id", "A-R06")? == definition_id
                && definition_id == expected_definition
                && text_field(definition, "kind", "A-R06")? == "parameter"
                && usize_field(definition, "ordinal", "A-R06")? == index
                && text_field(definition, "type_id", "A-R06")? == "type:int64"
                && parameters[index] == value
                && parameter_types[index] == "type:int64",
            "structure_mismatch_v0",
            "$.payload.definitions",
            "ordered parameter, child, value, and definition linkage differs",
        )?;
        value_ids.push(value.to_string());
        definition_ids.push(definition_id.to_string());
        definition_source_identities
            .push(text_field(definition, "semantic_id", "A-R06")?.to_string());
        definition_spans.push(span(field(definition, "source_span", "A-R06")?, "A-R06")?);
    }
    let passes = array(field(function, "required_passes", "A-R05")?, "A-R05")?;
    let mut pass_names = Vec::new();
    let pass_valid = passes.len() == backend_input::REQUIRED_PASSES.len()
        && passes.iter().enumerate().all(|(ordinal, row)| {
            let Ok(row) = exact_object(
                row,
                &["name", "status", "selected", "ordinal"],
                "A-R05",
                "$.payload.functions[0].required_passes[]",
            ) else {
                return false;
            };
            let Ok(name) = text_field(row, "name", "A-R05") else {
                return false;
            };
            pass_names.push(name.to_string());
            name == backend_input::REQUIRED_PASSES[ordinal]
                && text_field(row, "status", "A-R05").ok() == Some("passed")
                && usize_field(row, "selected", "A-R05").ok() == Some(1)
                && usize_field(row, "ordinal", "A-R05").ok() == Some(ordinal)
        });
    require(
        "A-R05",
        pass_valid,
        "pass_binding_mismatch_v0",
        "$.payload.functions[0].required_passes",
        "required pass order, result, selection, or ordinal differs",
    )?;
    facts.ordered_pass_count = Some(pass_names.len());
    let types = array(field(p, "types", "A-R06")?, "A-R06")?;
    let checked_type = exact_object(
        one(types, "A-R06", "$.payload.types")?,
        &["id", "source_type_id", "name", "kind", "signed", "bits"],
        "A-R06",
        "$.payload.types[0]",
    )?;
    let effects = array(field(p, "effects", "A-R07")?, "A-R07")?;
    let effect = exact_object(
        one(effects, "A-R07", "$.payload.effects")?,
        &["id", "effects", "external_authority"],
        "A-R07",
        "$.payload.effects[0]",
    )?;
    let resources = array(field(p, "resources", "A-R07")?, "A-R07")?;
    let resource = exact_object(
        one(resources, "A-R07", "$.payload.resources")?,
        &[
            "id",
            "allocation_declaration",
            "allocations",
            "moves",
            "borrows",
            "aliases",
            "ownership_transfers",
            "contract_predicates",
            "evidence_obligations",
            "profile",
        ],
        "A-R07",
        "$.payload.resources[0]",
    )?;
    let failure_edges = array(field(p, "failure_edges", "A-R08")?, "A-R08")?;
    let edge = exact_object(
        one(failure_edges, "A-R08", "$.payload.failure_edges")?,
        &["id", "value_type", "operation", "behavior"],
        "A-R08",
        "$.payload.failure_edges[0]",
    )?;
    require(
        "A-R04",
        text_field(compiler, "version", "A-R04")? == version::HUM_VERSION
            && text_field(compiler, "ir_schema", "A-R04")?
                == crate::ir_contract::IR_CONTRACT_SCHEMA
            && text_field(compiler, "semantic_contract", "A-R04")?
                == backend_input::SEMANTIC_CONTRACT
            && string_array(field(compiler, "feature_set", "A-R04")?, "A-R04")?
                == [backend_input::FEATURE_SET]
            && text_field(compiler, "target_context", "A-R04")? == backend_input::TARGET_CONTEXT
            && text_field(source, "id", "A-R04")? == "source:0"
            && text_field(source, "sha256", "A-R04")? == backend_input::SOURCE_REVISION_SHA256
            && usize_field(source, "file_ordinal", "A-R04")? == 0
            && text_field(source, "normalized_path", "A-R04")? == backend_input::SOURCE_PATH
            && text_field(module, "id", "A-R04")? == "module:examples.core.minimal_add"
            && text_field(module, "name", "A-R04")? == backend_input::MODULE_NAME
            && string_array(field(module, "files", "A-R04")?, "A-R04")? == ["source:0"],
        "identity_mismatch_v0",
        "$.payload.compiler",
        "compiler, semantic, target, feature, or source identity differs",
    )?;
    facts.semantic_contract = Some(text_field(compiler, "semantic_contract", "A-R04")?.to_string());
    facts.compiler_version = Some(text_field(compiler, "version", "A-R04")?.to_string());
    facts.target_context = Some(text_field(compiler, "target_context", "A-R04")?.to_string());
    facts.source_revision = Some(text_field(source, "sha256", "A-R04")?.to_string());
    let result_value_id = text_field(abi, "result", "A-R06")?.to_string();
    let operation_id = text_field(operation, "id", "A-R06")?.to_string();
    require(
        "A-R06",
        text_field(function, "id", "A-R06")? == "function:0"
            && text_field(function, "source_item_id", "A-R06")? == "resolver-item:file-0:path-0"
            && text_field(function, "display_name", "A-R06")? == "add"
            && text_field(function, "item_kind", "A-R06")? == "task"
            && text_field(linkage, "kind", "A-R06")? == "internal"
            && text_field(linkage, "symbol", "A-R06")? == "hum_fn_0"
            && function_span == (3, 1)
            && text_field(abi, "calling_convention", "A-R06")? == "hum_internal_v0"
            && text_field(abi, "result_type", "A-R06")? == "type:int64"
            && usize_field(abi, "integer_width", "A-R06")? == 64
            && text_field(abi, "trap_convention", "A-R06")? == "hum_checked_trap_v0"
            && text_field(block, "id", "A-R06")? == "block:function:0:0"
            && operation_id == "operation:function:0:block:0:0"
            && text_field(operation, "section_id", "A-R06")? == "section:function:0:does:0"
            && text_field(operation, "kind", "A-R06")? == "return"
            && text_field(operation, "expression_id", "A-R06")?
                == text_field(expression, "id", "A-R06")?
            && text_field(operation, "result_value_id", "A-R06")? == result_value_id
            && operation_span == (8, 5)
            && text_field(expression, "kind", "A-R06")? == "binary"
            && text_field(expression, "operator", "A-R06")? == "checked_add"
            && text_field(expression, "result_value_id", "A-R06")? == result_value_id
            && text_field(expression, "checked_type_id", "A-R06")? == "type:int64"
            && text_field(provenance, "source_id", "A-R06")? == "source:0"
            && usize_field(provenance, "line", "A-R06")? == 8
            && usize_field(provenance, "column", "A-R06")? == 12
            && text_field(checked_type, "id", "A-R06")? == "type:int64"
            && text_field(checked_type, "source_type_id", "A-R06")? == "hum-type:builtin:Int"
            && text_field(checked_type, "name", "A-R06")? == "Int"
            && text_field(checked_type, "kind", "A-R06")? == "integer"
            && bool_field(checked_type, "signed", "A-R06")?
            && usize_field(checked_type, "bits", "A-R06")? == 64
            && value_ids[0] != value_ids[1]
            && definition_ids[0] != definition_ids[1]
            && result_value_id != value_ids[0]
            && result_value_id != value_ids[1],
        "structure_mismatch_v0",
        "$.payload.functions[0]",
        "task, function, operation, result, type, or span structure differs",
    )?;
    facts.task_count = Some(1);
    facts.function_count = Some(functions.len());
    facts.operation_count = Some(operations.len());
    let empty_fields = [
        (expression, "unsupported"),
        (effect, "effects"),
        (effect, "external_authority"),
        (resource, "allocations"),
        (resource, "moves"),
        (resource, "borrows"),
        (resource, "aliases"),
        (resource, "ownership_transfers"),
        (resource, "contract_predicates"),
        (resource, "evidence_obligations"),
        (p, "unsupported"),
    ];
    let all_empty = empty_fields.iter().all(|(members, key)| {
        field(members, key, "A-R07")
            .and_then(|node| array(node, "A-R07"))
            .is_ok_and(<[_]>::is_empty)
    });
    require(
        "A-R07",
        all_empty
            && text_field(resource, "allocation_declaration", "A-R07")? == "nothing"
            && text_field(resource, "profile", "A-R07")? == "normal",
        "profile_or_checked_empty_mismatch_v0",
        "$.payload.resources[0]",
        "profile or checked-empty set differs",
    )?;
    require(
        "A-R08",
        text_field(edge, "id", "A-R08")? == "failure-edge:function:0:0"
            && text_field(edge, "value_type", "A-R08")? == "signed_64"
            && text_field(edge, "operation", "A-R08")? == "checked_add"
            && text_field(edge, "behavior", "A-R08")? == "runtime_trap_on_overflow"
            && text_field(expression, "failure_edge_id", "A-R08")?
                == text_field(edge, "id", "A-R08")?,
        "overflow_edge_mismatch_v0",
        "$.payload.failure_edges[0]",
        "checked-add overflow/failure-edge facts differ",
    )?;
    Ok(VerifiedProjection {
        artifact_id,
        payload_digest,
        compiler_version: text_field(compiler, "version", "A-R04")?.to_string(),
        semantic_contract: text_field(compiler, "semantic_contract", "A-R04")?.to_string(),
        target_context: text_field(compiler, "target_context", "A-R04")?.to_string(),
        source_revision: text_field(source, "sha256", "A-R04")?.to_string(),
        source_path: text_field(source, "normalized_path", "A-R04")?.to_string(),
        function_id: text_field(function, "id", "A-R06")?.to_string(),
        linkage_kind: text_field(linkage, "kind", "A-R06")?.to_string(),
        linkage_symbol: text_field(linkage, "symbol", "A-R06")?.to_string(),
        parameter_value_ids: value_ids.try_into().expect("two parameter values"),
        parameter_definition_ids: definition_ids.try_into().expect("two definitions"),
        parameter_definition_source_identities: definition_source_identities
            .try_into()
            .expect("two definition source identities"),
        parameter_types: parameter_types.try_into().expect("two parameter types"),
        parameter_spans: definition_spans.try_into().expect("two parameter spans"),
        operation_id,
        operation_source_identity: text_field(expression, "id", "A-R06")?.to_string(),
        result_value_id,
        result_type: text_field(abi, "result_type", "A-R06")?.to_string(),
        operation_span,
        overflow_value_type: text_field(edge, "value_type", "A-R08")?.to_string(),
        overflow_operation: text_field(edge, "operation", "A-R08")?.to_string(),
        overflow_behavior: text_field(edge, "behavior", "A-R08")?.to_string(),
        profile: text_field(resource, "profile", "A-R07")?.to_string(),
        required_passes: pass_names,
    })
}

fn require(
    row: &'static str,
    condition: bool,
    code: &'static str,
    path: &'static str,
    reason: &'static str,
) -> Result<(), Failure> {
    if condition || !check_enabled(row) {
        Ok(())
    } else {
        Err(Failure::without_offset(row, code, path, reason))
    }
}

fn valid_sha256_id(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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
}

#[derive(Debug)]
struct ParseError {
    code: &'static str,
    offset: usize,
    reason: &'static str,
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            position: 0,
        }
    }
    fn parse(mut self) -> Result<JsonNode, ParseError> {
        let value = self.value()?;
        self.whitespace();
        if self.position != self.bytes.len() {
            return Err(self.error("trailing_json_bytes_v0", "trailing JSON bytes"));
        }
        Ok(value)
    }
    fn value(&mut self) -> Result<JsonNode, ParseError> {
        self.whitespace();
        let start = self.position;
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
            span: start..self.position,
        })
    }
    fn object(&mut self) -> Result<JsonValue, ParseError> {
        self.take(b'{')?;
        self.whitespace();
        let mut members = Vec::new();
        if self.consume(b'}') {
            return Ok(JsonValue::Object(members));
        }
        loop {
            self.whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.error("malformed_json_v0", "expected object key"));
            }
            let key_start = self.position;
            let key = self.string()?;
            let key_span = key_start..self.position;
            self.whitespace();
            self.take(b':')?;
            let value = self.value()?;
            members.push(JsonMember {
                key,
                key_span,
                value,
            });
            self.whitespace();
            if self.consume(b'}') {
                break;
            }
            self.take(b',')?;
        }
        Ok(JsonValue::Object(members))
    }
    fn array(&mut self) -> Result<JsonValue, ParseError> {
        self.take(b'[')?;
        self.whitespace();
        let mut values = Vec::new();
        if self.consume(b']') {
            return Ok(JsonValue::Array(values));
        }
        loop {
            values.push(self.value()?);
            self.whitespace();
            if self.consume(b']') {
                break;
            }
            self.take(b',')?;
        }
        Ok(JsonValue::Array(values))
    }
    fn string(&mut self) -> Result<String, ParseError> {
        self.take(b'"')?;
        let mut value = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(self.error("malformed_json_v0", "unterminated string"));
            };
            self.position += 1;
            match byte {
                b'"' => break,
                b'\\' => {
                    let escape = self
                        .peek()
                        .ok_or_else(|| self.error("invalid_escape_v0", "incomplete escape"))?;
                    self.position += 1;
                    match escape {
                        b'"' => value.push('"'),
                        b'\\' => value.push('\\'),
                        b'/' => value.push('/'),
                        b'b' => value.push('\u{8}'),
                        b'f' => value.push('\u{c}'),
                        b'n' => value.push('\n'),
                        b'r' => value.push('\r'),
                        b't' => value.push('\t'),
                        b'u' => {
                            let high = self.hex4()?;
                            let scalar = if (0xd800..=0xdbff).contains(&high) {
                                if !self.consume(b'\\') || !self.consume(b'u') {
                                    return Err(self.error(
                                        "invalid_escape_v0",
                                        "high surrogate requires low surrogate",
                                    ));
                                }
                                let low = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err(
                                        self.error("invalid_escape_v0", "invalid low surrogate")
                                    );
                                }
                                0x1_0000 + ((high - 0xd800) << 10) + (low - 0xdc00)
                            } else if (0xdc00..=0xdfff).contains(&high) {
                                return Err(
                                    self.error("invalid_escape_v0", "unpaired low surrogate")
                                );
                            } else {
                                high
                            };
                            value.push(char::from_u32(scalar).ok_or_else(|| {
                                self.error("invalid_escape_v0", "invalid Unicode scalar")
                            })?);
                        }
                        _ => return Err(self.error("invalid_escape_v0", "invalid string escape")),
                    }
                }
                0..=31 => {
                    return Err(self.error("invalid_control_v0", "unescaped control byte"));
                }
                _ if byte.is_ascii() => value.push(char::from(byte)),
                _ => {
                    self.position -= 1;
                    let rest = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| self.error("invalid_utf8_v0", "invalid UTF-8"))?;
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| self.error("invalid_utf8_v0", "invalid UTF-8"))?;
                    self.position += character.len_utf8();
                    value.push(character);
                }
            }
        }
        Ok(value)
    }
    fn hex4(&mut self) -> Result<u32, ParseError> {
        if self.position + 4 > self.bytes.len() {
            return Err(self.error("invalid_escape_v0", "short Unicode escape"));
        }
        let text = std::str::from_utf8(&self.bytes[self.position..self.position + 4])
            .map_err(|_| self.error("invalid_escape_v0", "invalid Unicode escape"))?;
        self.position += 4;
        u32::from_str_radix(text, 16)
            .map_err(|_| self.error("invalid_escape_v0", "invalid Unicode escape"))
    }
    fn number(&mut self) -> Result<String, ParseError> {
        let start = self.position;
        if self.consume(b'-') {
            return Err(self.error("invalid_number_v0", "negative number forbidden"));
        }
        if self.consume(b'0') {
            if matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("invalid_number_v0", "leading zero forbidden"));
            }
        } else {
            let before = self.position;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
            if before == self.position {
                return Err(self.error("invalid_number_v0", "canonical integer required"));
            }
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(self.error("invalid_number_v0", "integer required"));
        }
        Ok(std::str::from_utf8(&self.bytes[start..self.position])
            .expect("ASCII number")
            .to_string())
    }
    fn literal(&mut self, bytes: &[u8], value: JsonValue) -> Result<JsonValue, ParseError> {
        for byte in bytes {
            self.take(*byte)?;
        }
        Ok(value)
    }
    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }
    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.position += 1;
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
            offset: self.position,
            reason,
        }
    }
}

fn exact_object<'a>(
    node: &'a JsonNode,
    expected: &[&str],
    row: &'static str,
    path: &'static str,
) -> Result<&'a [JsonMember], Failure> {
    let JsonValue::Object(members) = &node.value else {
        return Err(Failure::at(
            row,
            "closed_shape_mismatch_v0",
            path,
            "object required",
            node.span.start,
        ));
    };
    if members.len() != expected.len()
        || !members
            .iter()
            .zip(expected)
            .all(|(member, expected)| member.key == *expected)
    {
        return Err(Failure::at(
            row,
            "closed_shape_mismatch_v0",
            path,
            "object keys or order differ",
            node.span.start,
        ));
    }
    Ok(members)
}

fn field<'a>(
    members: &'a [JsonMember],
    key: &str,
    row: &'static str,
) -> Result<&'a JsonNode, Failure> {
    members
        .iter()
        .find(|member| member.key == key)
        .map(|member| &member.value)
        .ok_or_else(|| Failure::without_offset(row, "missing_key_v0", "$", "required key missing"))
}

fn text_field<'a>(
    members: &'a [JsonMember],
    key: &str,
    row: &'static str,
) -> Result<&'a str, Failure> {
    let node = field(members, key, row)?;
    match &node.value {
        JsonValue::String(value) => Ok(value),
        _ => Err(Failure::at(
            row,
            "type_mismatch_v0",
            "$",
            "string required",
            node.span.start,
        )),
    }
}

fn array<'a>(node: &'a JsonNode, row: &'static str) -> Result<&'a [JsonNode], Failure> {
    match &node.value {
        JsonValue::Array(values) => Ok(values),
        _ => Err(Failure::at(
            row,
            "type_mismatch_v0",
            "$",
            "array required",
            node.span.start,
        )),
    }
}

fn one<'a>(
    values: &'a [JsonNode],
    row: &'static str,
    path: &'static str,
) -> Result<&'a JsonNode, Failure> {
    if let [value] = values {
        Ok(value)
    } else {
        Err(Failure::without_offset(
            row,
            "count_mismatch_v0",
            path,
            "exactly one row required",
        ))
    }
}

fn usize_field(members: &[JsonMember], key: &str, row: &'static str) -> Result<usize, Failure> {
    let node = field(members, key, row)?;
    match &node.value {
        JsonValue::Number(value) => value.parse().map_err(|_| {
            Failure::at(
                row,
                "integer_overflow_v0",
                "$",
                "bounded integer required",
                node.span.start,
            )
        }),
        _ => Err(Failure::at(
            row,
            "type_mismatch_v0",
            "$",
            "integer required",
            node.span.start,
        )),
    }
}

fn bool_field(members: &[JsonMember], key: &str, row: &'static str) -> Result<bool, Failure> {
    let node = field(members, key, row)?;
    match node.value {
        JsonValue::Bool(value) => Ok(value),
        _ => Err(Failure::at(
            row,
            "type_mismatch_v0",
            "$",
            "boolean required",
            node.span.start,
        )),
    }
}

fn string_array(node: &JsonNode, row: &'static str) -> Result<Vec<String>, Failure> {
    array(node, row)?
        .iter()
        .map(|node| match &node.value {
            JsonValue::String(value) => Ok(value.clone()),
            _ => Err(Failure::at(
                row,
                "type_mismatch_v0",
                "$",
                "string array required",
                node.span.start,
            )),
        })
        .collect()
}

fn span(node: &JsonNode, row: &'static str) -> Result<(usize, usize), Failure> {
    let members = exact_object(node, &["source_id", "line", "column"], row, "$.source_span")?;
    require(
        row,
        text_field(members, "source_id", row)? == "source:0",
        "span_mismatch_v0",
        "$.source_span",
        "source identity differs",
    )?;
    Ok((
        usize_field(members, "line", row)?,
        usize_field(members, "column", row)?,
    ))
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
        _ => None,
    }
}

fn emit_json(node: &JsonNode, out: &mut String) {
    match &node.value {
        JsonValue::Object(members) => {
            out.push('{');
            for (index, member) in members.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                push_json_string(out, &member.key);
                out.push(':');
                emit_json(&member.value, out);
            }
            out.push('}');
        }
        JsonValue::Array(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                emit_json(value, out);
            }
            out.push(']');
        }
        JsonValue::String(value) => push_json_string(out, value),
        JsonValue::Number(value) => out.push_str(value),
        JsonValue::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        JsonValue::Null => out.push_str("null"),
    }
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if character <= '\u{1f}' => {
                let _ = write!(out, "\\u{:04x}", u32::from(character));
            }
            character => out.push(character),
        }
    }
    out.push('"');
}

pub(crate) fn ir_verify_text(report: &IrVerifyReport) -> String {
    let mut out = String::new();
    out.push_str("Hum IR verify\n");
    let _ = writeln!(out, "schema: {IR_VERIFY_SCHEMA}");
    let _ = writeln!(out, "tool_version: {}", version::HUM_VERSION);
    let _ = writeln!(out, "status: {}", report.status);
    let _ = writeln!(
        out,
        "artifact_id: {}",
        report.artifact_id.as_deref().unwrap_or("null")
    );
    let _ = writeln!(
        out,
        "computed_payload_digest: {}",
        report.payload_digest.as_deref().unwrap_or("null")
    );
    let _ = writeln!(
        out,
        "semantic_contract: {}",
        report.facts.semantic_contract.as_deref().unwrap_or("null")
    );
    let _ = writeln!(
        out,
        "compiler_version: {}",
        report.facts.compiler_version.as_deref().unwrap_or("null")
    );
    let _ = writeln!(
        out,
        "target_context: {}",
        report.facts.target_context.as_deref().unwrap_or("null")
    );
    let _ = writeln!(
        out,
        "source_revision: {}",
        report.facts.source_revision.as_deref().unwrap_or("null")
    );
    for (name, value) in [
        ("task_count", report.facts.task_count),
        ("function_count", report.facts.function_count),
        ("operation_count", report.facts.operation_count),
        ("ordered_pass_count", report.facts.ordered_pass_count),
    ] {
        let _ = writeln!(
            out,
            "{name}: {}",
            value.map_or_else(|| "null".to_string(), |count| count.to_string())
        );
    }
    let _ = writeln!(
        out,
        "rejected_check: {}",
        report.rejected_check.unwrap_or("none")
    );
    out.push_str("findings:\n");
    for finding in &report.findings {
        let _ = writeln!(
            out,
            "  {} byte_offset={} logical_path={} reason={}",
            finding.code,
            finding
                .byte_offset
                .map_or_else(|| "null".to_string(), |value| value.to_string()),
            finding.logical_path,
            finding.reason
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
    out.push('{');
    for (index, (key, value)) in [
        ("schema", IR_VERIFY_SCHEMA),
        ("tool_version", version::HUM_VERSION),
        ("status", report.status),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            out.push(',');
        }
        push_json_string(&mut out, key);
        out.push(':');
        push_json_string(&mut out, value);
    }
    out.push_str(",\"artifact_id\":");
    push_optional_json_string(&mut out, report.artifact_id.as_deref());
    out.push_str(",\"computed_payload_digest\":");
    push_optional_json_string(&mut out, report.payload_digest.as_deref());
    for (key, value) in [
        (
            "semantic_contract",
            report.facts.semantic_contract.as_deref(),
        ),
        ("compiler_version", report.facts.compiler_version.as_deref()),
        ("target_context", report.facts.target_context.as_deref()),
        ("source_revision", report.facts.source_revision.as_deref()),
    ] {
        out.push(',');
        push_json_string(&mut out, key);
        out.push(':');
        push_optional_json_string(&mut out, value);
    }
    for (key, value) in [
        ("task_count", report.facts.task_count),
        ("function_count", report.facts.function_count),
        ("operation_count", report.facts.operation_count),
        ("ordered_pass_count", report.facts.ordered_pass_count),
    ] {
        out.push(',');
        push_json_string(&mut out, key);
        out.push(':');
        if let Some(value) = value {
            let _ = write!(out, "{value}");
        } else {
            out.push_str("null");
        }
    }
    out.push_str(",\"rejected_check\":");
    push_optional_json_string(&mut out, report.rejected_check);
    out.push_str(",\"findings\":[");
    for (index, finding) in report.findings.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str("{\"code\":");
        push_json_string(&mut out, finding.code);
        out.push_str(",\"byte_offset\":");
        if let Some(offset) = finding.byte_offset {
            let _ = write!(out, "{offset}");
        } else {
            out.push_str("null");
        }
        out.push_str(",\"logical_path\":");
        push_json_string(&mut out, finding.logical_path);
        out.push_str(",\"reason\":");
        push_json_string(&mut out, finding.reason);
        out.push('}');
    }
    out.push_str("],\"non_claims_v0\":[");
    for (index, claim) in NON_CLAIMS.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        push_json_string(&mut out, claim);
    }
    out.push_str("]}\n");
    out
}

fn push_optional_json_string(out: &mut String, value: Option<&str>) {
    if let Some(value) = value {
        push_json_string(out, value);
    } else {
        out.push_str("null");
    }
}

#[cfg(test)]
#[derive(Clone)]
struct AuthenticatedLiveContext {
    identity: LogicalIdentity,
    program_identity: usize,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MixedIdentityComponent {
    Pass,
    Definition,
    Operation,
    Result,
}

#[cfg(test)]
impl MixedIdentityComponent {
    fn named(name: &str) -> Self {
        match name {
            "pass" => Self::Pass,
            "definition" => Self::Definition,
            "operation" => Self::Operation,
            "result" => Self::Result,
            other => panic!("unknown authenticated identity component: {other}"),
        }
    }
}

#[cfg(test)]
struct AuthenticatedMix {
    component: MixedIdentityComponent,
    primary: AuthenticatedLiveContext,
    foreign: AuthenticatedLiveContext,
}

#[cfg(test)]
enum AuthenticatedMixState {
    Idle,
    Armed(Box<AuthenticatedMix>),
    Consumed,
}

#[cfg(test)]
thread_local! {
    static SKIPPED_CHECK: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
    static IDENTITY_COMPARISON: std::cell::Cell<Option<(&'static str, usize)>> = const { std::cell::Cell::new(None) };
    static AUTHENTICATED_MIX: std::cell::RefCell<AuthenticatedMixState> = const { std::cell::RefCell::new(AuthenticatedMixState::Idle) };
}

#[cfg(test)]
impl LiveIdentityRequest<'static> {
    fn capture_for_test(program: &Program) -> Self {
        Self {
            expected: None,
            observed: None,
            program_identity: std::ptr::from_ref(program).addr(),
        }
    }

    fn into_authenticated_context_for_test(self) -> AuthenticatedLiveContext {
        AuthenticatedLiveContext {
            identity: self
                .observed
                .expect("authenticated live context must contain one observation"),
            program_identity: self.program_identity,
        }
    }
}

#[cfg(test)]
fn arm_authenticated_mix_for_test(
    component: &str,
    primary: AuthenticatedLiveContext,
    foreign: AuthenticatedLiveContext,
) {
    assert_ne!(primary.program_identity, foreign.program_identity);
    let component = MixedIdentityComponent::named(component);
    let selected_differs = match component {
        MixedIdentityComponent::Pass => {
            primary.identity.required_passes[13] != foreign.identity.required_passes[13]
        }
        MixedIdentityComponent::Definition => {
            primary.identity.parameter_definition_source_identities[0]
                != foreign.identity.parameter_definition_source_identities[0]
        }
        MixedIdentityComponent::Operation => {
            primary.identity.operation_source_identity != foreign.identity.operation_source_identity
        }
        MixedIdentityComponent::Result => {
            primary.identity.result_value_id != foreign.identity.result_value_id
        }
    };
    assert!(
        selected_differs,
        "foreign authenticated component must differ"
    );
    AUTHENTICATED_MIX.with(|state| {
        let mut state = state.borrow_mut();
        assert!(matches!(*state, AuthenticatedMixState::Idle));
        *state = AuthenticatedMixState::Armed(Box::new(AuthenticatedMix {
            component,
            primary,
            foreign,
        }));
    });
}

#[cfg(test)]
fn mix_authenticated_live_identity_for_test(
    observed: &mut LogicalIdentity,
    program_identity: usize,
) {
    AUTHENTICATED_MIX.with(|state| {
        let prior = std::mem::replace(&mut *state.borrow_mut(), AuthenticatedMixState::Consumed);
        let AuthenticatedMixState::Armed(mix) = prior else {
            if matches!(prior, AuthenticatedMixState::Idle) {
                *state.borrow_mut() = AuthenticatedMixState::Idle;
                return;
            }
            panic!("authenticated identity mix used more than once");
        };
        assert_eq!(program_identity, mix.primary.program_identity);
        assert!(*observed == mix.primary.identity);
        let before = observed.clone();
        match mix.component {
            MixedIdentityComponent::Pass => {
                observed.required_passes[13] = mix.foreign.identity.required_passes[13].clone();
            }
            MixedIdentityComponent::Definition => {
                observed.parameter_definition_source_identities[0] =
                    mix.foreign.identity.parameter_definition_source_identities[0].clone();
            }
            MixedIdentityComponent::Operation => {
                observed.operation_source_identity =
                    mix.foreign.identity.operation_source_identity.clone();
            }
            MixedIdentityComponent::Result => {
                observed.result_value_id = mix.foreign.identity.result_value_id.clone();
            }
        }
        assert_eq!(identity_component_difference_count(&before, observed), 1);
    });
}

#[cfg(not(test))]
fn mix_authenticated_live_identity_for_test(
    _observed: &mut LogicalIdentity,
    _program_identity: usize,
) {
}

#[cfg(test)]
fn identity_component_difference_count(left: &LogicalIdentity, right: &LogicalIdentity) -> usize {
    usize::from(left.required_passes != right.required_passes)
        + usize::from(
            left.parameter_definition_source_identities
                != right.parameter_definition_source_identities,
        )
        + usize::from(left.operation_source_identity != right.operation_source_identity)
        + usize::from(left.result_value_id != right.result_value_id)
}

#[cfg(test)]
fn assert_authenticated_mix_consumed_for_test() {
    AUTHENTICATED_MIX.with(|state| {
        let prior = std::mem::replace(&mut *state.borrow_mut(), AuthenticatedMixState::Idle);
        assert!(matches!(prior, AuthenticatedMixState::Consumed));
    });
}

#[cfg(test)]
fn clear_authenticated_mix_for_test() {
    AUTHENTICATED_MIX.with(|state| *state.borrow_mut() = AuthenticatedMixState::Idle);
}

#[cfg(test)]
fn check_enabled(row: &'static str) -> bool {
    SKIPPED_CHECK.with(|skipped| skipped.get() != Some(row))
}

#[cfg(not(test))]
fn check_enabled(_row: &'static str) -> bool {
    true
}

#[cfg(test)]
fn identities_match_for_test(
    expected: &LogicalIdentity,
    observed: &LogicalIdentity,
    program_identity: usize,
) -> bool {
    match IDENTITY_COMPARISON.with(std::cell::Cell::take) {
        Some(("pointer", expected_program)) => expected_program == program_identity,
        Some(("ignore_pass", _)) => {
            expected.source_revision == observed.source_revision
                && expected.source_path == observed.source_path
                && expected.function_id == observed.function_id
                && expected.linkage_kind == observed.linkage_kind
                && expected.linkage_symbol == observed.linkage_symbol
                && expected.parameter_value_ids == observed.parameter_value_ids
                && expected.parameter_definition_ids == observed.parameter_definition_ids
                && expected.parameter_definition_source_identities
                    == observed.parameter_definition_source_identities
                && expected.parameter_types == observed.parameter_types
                && expected.parameter_spans == observed.parameter_spans
                && expected.operation_id == observed.operation_id
                && expected.operation_source_identity == observed.operation_source_identity
                && expected.result_value_id == observed.result_value_id
                && expected.result_type == observed.result_type
                && expected.operation_span == observed.operation_span
        }
        None => expected == observed,
        Some((other, _)) => panic!("unknown identity comparison: {other}"),
    }
}

#[cfg(not(test))]
fn identities_match_for_test(
    expected: &LogicalIdentity,
    observed: &LogicalIdentity,
    _program_identity: usize,
) -> bool {
    expected == observed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> (Program, Vec<Diagnostic>) {
        let parsed = crate::parser::parse_source(
            backend_input::SOURCE_PATH.to_string(),
            include_str!("../examples/core/minimal_add.hum"),
        );
        let checked = crate::check::check_parse_output(&parsed);
        assert!(parsed.diagnostics.is_empty());
        assert!(checked.diagnostics.is_empty());
        (
            Program {
                files: vec![parsed.file],
            },
            checked.diagnostics,
        )
    }

    fn foreign_subject() -> (Program, Vec<Diagnostic>) {
        let empty = crate::parser::parse_source_at_index("empty.hum".to_string(), "", 0);
        let foreign = crate::parser::parse_source_at_index(
            backend_input::SOURCE_PATH.to_string(),
            include_str!("../examples/core/minimal_add.hum"),
            1,
        );
        let empty_checked = crate::check::check_parse_output(&empty);
        let foreign_checked = crate::check::check_parse_output(&foreign);
        assert!(empty.diagnostics.is_empty());
        assert!(foreign.diagnostics.is_empty());
        assert!(empty_checked.diagnostics.is_empty());
        assert!(foreign_checked.diagnostics.is_empty());
        (
            Program {
                files: vec![empty.file, foreign.file],
            },
            Vec::new(),
        )
    }

    fn authenticated_context(
        program: &Program,
        diagnostics: &[Diagnostic],
        semantic_file_index: usize,
    ) -> AuthenticatedLiveContext {
        let mut request = LiveIdentityRequest::capture_for_test(program);
        assert!(
            backend_input::bind_canonical_minimal_add_live_identity_for_test(
                &mut request,
                program,
                diagnostics,
                semantic_file_index,
            )
        );
        request.into_authenticated_context_for_test()
    }

    fn golden() -> Vec<u8> {
        let (program, diagnostics) = subject();
        backend_input::canonical_minimal_add_artifact(&program, &diagnostics)
            .expect("canonical artifact")
            .bytes()
            .to_vec()
    }

    fn replace_once(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let locations = bytes
            .windows(from.len())
            .enumerate()
            .filter_map(|(offset, candidate)| (candidate == from).then_some(offset))
            .collect::<Vec<_>>();
        assert_eq!(locations.len(), 1, "mutation needle must be unique");
        let offset = locations[0];
        let mut changed = Vec::with_capacity(bytes.len() - from.len() + to.len());
        changed.extend_from_slice(&bytes[..offset]);
        changed.extend_from_slice(to);
        changed.extend_from_slice(&bytes[offset + from.len()..]);
        changed
    }

    fn install_digest(bytes: &mut [u8]) {
        let root = Parser::new(std::str::from_utf8(&bytes[..bytes.len() - 1]).unwrap())
            .parse()
            .unwrap();
        let envelope =
            exact_object(&root, &["schema", "artifact_id", "payload"], "test", "$").unwrap();
        let payload = field(envelope, "payload", "test").unwrap();
        let digest = sha256::digest(&bytes[payload.span.clone()]).unwrap();
        let digest = sha256::lowercase_hex(&digest);
        let marker = b"\"artifact_id\":\"sha256:";
        let offset = bytes
            .windows(marker.len())
            .position(|value| value == marker)
            .unwrap()
            + marker.len();
        bytes[offset..offset + 64].copy_from_slice(digest.as_bytes());
    }

    fn semantic_mutation(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let mut changed = replace_once(bytes, from, to);
        install_digest(&mut changed);
        changed
    }

    fn assert_rejected(bytes: &[u8], row: &str, code: &str) {
        let (program, diagnostics) = subject();
        let mut callbacks = 0;
        let mut backend_calls = 0;
        let (report, result) = with_verified_backend_input(&program, &diagnostics, bytes, |_| {
            callbacks += 1;
            backend_calls += 1;
        });
        assert!(result.is_none());
        assert_eq!(callbacks, 0);
        assert_eq!(backend_calls, 0);
        assert_eq!(report.rejected_check, Some(row));
        assert_eq!(report.findings[0].code, code, "{}", ir_verify_json(&report));
    }

    fn live_counts(
        program: &Program,
        diagnostics: &[Diagnostic],
        bytes: &[u8],
    ) -> (IrVerifyReport, usize, usize) {
        let mut callbacks = 0;
        let mut backend_calls = 0;
        let (report, _) = with_verified_backend_input(program, diagnostics, bytes, |_| {
            callbacks += 1;
            backend_calls += 1;
        });
        (report, callbacks, backend_calls)
    }

    fn live(
        program: &Program,
        diagnostics: &[Diagnostic],
        bytes: &[u8],
        expected: Option<(&str, &str)>,
        calls: usize,
    ) {
        let (report, callbacks, backend_calls) = live_counts(program, diagnostics, bytes);
        if let Some((row, code)) = expected {
            assert_eq!(report.rejected_check, Some(row));
            assert_eq!(report.findings[0].code, code);
        } else {
            assert!(report.accepted());
        }
        assert_eq!((callbacks, backend_calls), (calls, calls));
    }

    fn skip_once(row: &'static str) {
        SKIPPED_CHECK.with(|value| assert_eq!(value.replace(Some(row)), None));
    }

    #[test]
    fn canonical_minimal_add_artifact_corruption_matrix_is_complete() {
        let golden = golden();
        assert_eq!(golden.len(), 8_715);
        assert_eq!(
            sha256::lowercase_hex(&sha256::digest(&golden).unwrap()),
            "9a2affc59962e0d83a33633edce6f318d78a406a9d2e5ad2edc5b8e34cf7c293"
        );
        let (program, diagnostics) = subject();
        let primary_context = authenticated_context(&program, &diagnostics, 0);
        let (foreign_program, foreign_diagnostics) = foreign_subject();
        let foreign_context = authenticated_context(&foreign_program, &foreign_diagnostics, 1);
        assert_ne!(
            primary_context.program_identity,
            foreign_context.program_identity
        );
        assert_ne!(
            primary_context.identity.required_passes[13],
            foreign_context.identity.required_passes[13]
        );
        assert_ne!(
            primary_context
                .identity
                .parameter_definition_source_identities[0],
            foreign_context
                .identity
                .parameter_definition_source_identities[0]
        );
        assert_ne!(
            primary_context.identity.operation_source_identity,
            foreign_context.identity.operation_source_identity
        );
        assert_ne!(
            primary_context.identity.result_value_id,
            foreign_context.identity.result_value_id
        );
        let a_r09 = Some(("A-R09", "live_cross_binding_mismatch_v0"));
        let mut callbacks = 0;
        let (report, value) =
            with_verified_backend_input(&program, &diagnostics, &golden, |capability| {
                callbacks += 1;
                assert_eq!(capability.schema(), backend_input::BACKEND_INPUT_SCHEMA);
                assert_eq!(capability.required_passes().count(), 14);
                assert_eq!(capability.profile(), "normal");
                capability.artifact_id().to_string()
            });
        assert!(report.accepted());
        assert_eq!(callbacks, 1);
        assert_eq!(value.as_deref(), report.artifact_id.as_deref());

        for (kind, index) in backend_input::REPRESENTATIVE_AUTHENTICATION_CORRUPTIONS {
            backend_input::set_corruption_for_test(kind, index);
            live(&program, &diagnostics, &golden, a_r09, 0);
            live(&program, &diagnostics, &golden, None, 1);
        }

        let early = verify_backend_input(&[]);
        assert!(early.facts.semantic_contract.is_none());
        assert_eq!(early.facts.task_count, None);
        assert!(ir_verify_text(&early).contains("semantic_contract: null"));
        assert!(ir_verify_text(&early).contains("task_count: null"));
        assert!(ir_verify_json(&early).contains("\"semantic_contract\":null"));
        assert!(ir_verify_json(&early).contains("\"ordered_pass_count\":null"));
        assert_rejected(&[], "A-R01", "invalid_framing_v0");
        assert_rejected(&golden[..golden.len() - 1], "A-R01", "invalid_framing_v0");
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(&golden);
        assert_rejected(&bom, "A-R01", "invalid_framing_v0");
        let mut crlf = golden.clone();
        crlf.pop();
        crlf.extend_from_slice(b"\r\n");
        assert_rejected(&crlf, "A-R01", "invalid_framing_v0");
        let mut invalid_utf8 = golden.clone();
        invalid_utf8[0] = 0xff;
        assert_rejected(&invalid_utf8, "A-R01", "invalid_utf8_v0");
        let mut extra_lf = golden.clone();
        extra_lf.push(b'\n');
        assert_rejected(&extra_lf, "A-R01", "invalid_framing_v0");
        let unknown_key = replace_once(&golden, b"\"schema\":", b"\"unknown\":");
        assert_rejected(&unknown_key, "A-R01", "closed_shape_mismatch_v0");
        let duplicate_key = replace_once(
            &golden,
            b"\"schema\":\"hum.backend_input.v0\",",
            b"\"schema\":\"hum.backend_input.v0\",\"schema\":\"hum.backend_input.v0\",",
        );
        assert_rejected(&duplicate_key, "A-R01", "duplicate_key_v0");
        let leading_zero = replace_once(&golden, b"\"file_ordinal\":0", b"\"file_ordinal\":00");
        assert_rejected(&leading_zero, "A-R01", "invalid_number_v0");
        let spaced =
            semantic_mutation(&golden, b"},\"source_revision\"", b"}, \"source_revision\"");
        assert_rejected(&spaced, "A-R01", "noncanonical_bytes_v0");
        let mut payload_changed = replace_once(
            &golden,
            b"\"profile\":\"normal\"",
            b"\"profile\":\"strict\"",
        );
        assert_rejected(&payload_changed, "A-R02", "payload_digest_mismatch_v0");
        let digest_rejected = verify_backend_input(&payload_changed);
        assert_eq!(digest_rejected.facts.task_count, None);
        assert_eq!(digest_rejected.facts.ordered_pass_count, None);
        skip_once("A-R02");
        live(
            &program,
            &diagnostics,
            &payload_changed,
            Some(("A-R07", "profile_or_checked_empty_mismatch_v0")),
            0,
        );
        SKIPPED_CHECK.with(|value| value.set(None));
        let layered_bytes = payload_changed.clone();
        payload_changed = golden.clone();
        let id_marker = b"\"artifact_id\":\"sha256:";
        let id = payload_changed
            .windows(id_marker.len())
            .position(|candidate| candidate == id_marker)
            .unwrap()
            + id_marker.len();
        payload_changed[id] = if payload_changed[id] == b'a' {
            b'b'
        } else {
            b'a'
        };
        assert_rejected(
            &payload_changed,
            "A-R03",
            "declared_artifact_id_mismatch_v0",
        );
        let late = verify_backend_input(&semantic_mutation(
            &golden,
            b"\"profile\":\"normal\"",
            b"\"profile\":\"strict\"",
        ));
        assert_eq!(late.rejected_check, Some("A-R07"));
        assert_eq!(late.facts.task_count, Some(1));
        assert_eq!(late.facts.function_count, Some(1));
        assert_eq!(late.facts.operation_count, Some(1));
        assert_eq!(late.facts.ordered_pass_count, Some(14));
        assert!(ir_verify_text(&late).contains("task_count: 1\nfunction_count: 1"));
        assert!(ir_verify_json(&late).contains("\"ordered_pass_count\":14"));
        let identity_changed = semantic_mutation(
            &golden,
            b"\"target_context\":\"target_independent_checked_i64_v0\"",
            b"\"target_context\":\"target_independent_checked_i64_v1\"",
        );
        let pass_changed = semantic_mutation(
            &golden,
            b"\"name\":\"profile_check\",\"status\":\"passed\"",
            b"\"name\":\"profile_check\",\"status\":\"failed\"",
        );
        let structure_changed =
            semantic_mutation(&golden, b"\"integer_width\":64", b"\"integer_width\":32");
        let profile_changed = semantic_mutation(
            &golden,
            b"\"profile\":\"normal\"",
            b"\"profile\":\"strict\"",
        );
        let edge_changed = semantic_mutation(
            &golden,
            b"runtime_trap_on_overflow",
            b"runtime_wrap_on_overflow",
        );
        let owned_cases = [
            ("A-R01", spaced.clone(), "noncanonical_bytes_v0"),
            ("A-R02", layered_bytes, "payload_digest_mismatch_v0"),
            (
                "A-R03",
                payload_changed.clone(),
                "declared_artifact_id_mismatch_v0",
            ),
            ("A-R04", identity_changed, "identity_mismatch_v0"),
            ("A-R05", pass_changed, "pass_binding_mismatch_v0"),
            ("A-R06", structure_changed, "structure_mismatch_v0"),
            (
                "A-R07",
                profile_changed,
                "profile_or_checked_empty_mismatch_v0",
            ),
            ("A-R08", edge_changed, "overflow_edge_mismatch_v0"),
        ];
        for (row, bytes, code) in owned_cases.iter().filter(|(row, _, _)| *row != "A-R02") {
            assert_rejected(bytes, row, code);
            skip_once(row);
            live(&program, &diagnostics, bytes, None, 1);
            SKIPPED_CHECK.with(|value| value.set(None));
        }
        for (disabled, _, _) in owned_cases.iter().filter(|(row, _, _)| *row != "A-R02") {
            for (row, bytes, code) in owned_cases.iter().filter(|(row, _, _)| row != disabled) {
                skip_once(disabled);
                live(&program, &diagnostics, bytes, Some((row, code)), 0);
                SKIPPED_CHECK.with(|value| value.set(None));
            }
            skip_once(disabled);
            arm_authenticated_mix_for_test(
                "pass",
                primary_context.clone(),
                foreign_context.clone(),
            );
            live(&program, &diagnostics, &golden, a_r09, 0);
            assert_authenticated_mix_consumed_for_test();
            SKIPPED_CHECK.with(|value| value.set(None));
        }
        for kind in ["pass", "definition", "operation", "result"] {
            arm_authenticated_mix_for_test(kind, primary_context.clone(), foreign_context.clone());
            live(&program, &diagnostics, &golden, a_r09, 0);
            assert_authenticated_mix_consumed_for_test();
            arm_authenticated_mix_for_test(kind, primary_context.clone(), foreign_context.clone());
            skip_once("A-R09");
            live(&program, &diagnostics, &golden, None, 1);
            assert_authenticated_mix_consumed_for_test();
            SKIPPED_CHECK.with(|value| value.set(None));
            arm_authenticated_mix_for_test(kind, primary_context.clone(), foreign_context.clone());
            live(&program, &diagnostics, &golden, a_r09, 0);
            assert_authenticated_mix_consumed_for_test();
        }
        let (reparsed, reparsed_d) = subject();
        assert_ne!(
            std::ptr::from_ref(&program).addr(),
            std::ptr::from_ref(&reparsed).addr()
        );
        live(&reparsed, &reparsed_d, &golden, None, 1);

        let program_address = std::ptr::from_ref(&program).addr();
        IDENTITY_COMPARISON
            .with(|mode| assert_eq!(mode.replace(Some(("pointer", program_address))), None));
        live(&reparsed, &reparsed_d, &golden, a_r09, 0);

        let reparsed_context = authenticated_context(&reparsed, &reparsed_d, 0);
        arm_authenticated_mix_for_test("pass", reparsed_context.clone(), foreign_context.clone());
        IDENTITY_COMPARISON.with(|mode| assert_eq!(mode.replace(Some(("ignore_pass", 0))), None));
        live(&reparsed, &reparsed_d, &golden, None, 1);
        assert_authenticated_mix_consumed_for_test();
        for kind in ["definition", "operation", "result"] {
            arm_authenticated_mix_for_test(kind, reparsed_context.clone(), foreign_context.clone());
            IDENTITY_COMPARISON
                .with(|mode| assert_eq!(mode.replace(Some(("ignore_pass", 0))), None));
            live(&reparsed, &reparsed_d, &golden, a_r09, 0);
            assert_authenticated_mix_consumed_for_test();
        }

        assert!(std::panic::catch_unwind(assert_authenticated_mix_consumed_for_test).is_err());
        clear_authenticated_mix_for_test();
        assert!(
            std::panic::catch_unwind(|| {
                arm_authenticated_mix_for_test(
                    "unknown",
                    primary_context.clone(),
                    foreign_context.clone(),
                );
            })
            .is_err()
        );
        clear_authenticated_mix_for_test();
        arm_authenticated_mix_for_test("pass", primary_context.clone(), foreign_context.clone());
        assert!(
            std::panic::catch_unwind(|| {
                arm_authenticated_mix_for_test(
                    "pass",
                    primary_context.clone(),
                    foreign_context.clone(),
                );
            })
            .is_err()
        );
        clear_authenticated_mix_for_test();
        arm_authenticated_mix_for_test("pass", primary_context, foreign_context);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                live(&reparsed, &reparsed_d, &golden, a_r09, 0);
            }))
            .is_err()
        );
        clear_authenticated_mix_for_test();

        for (row, bytes, code) in &owned_cases {
            skip_once("A-R09");
            live(&program, &diagnostics, bytes, Some((row, code)), 0);
            SKIPPED_CHECK.with(|value| value.set(None));
        }
    }

    #[test]
    fn verified_backend_input_is_sealed_typed_and_lifetime_bound() {
        fn assert_no_marker<T>() {
            let name = std::any::type_name::<T>();
            assert!(!name.is_empty());
        }
        assert_no_marker::<VerifiedBackendInput<'_>>();
        let (program, diagnostics) = subject();
        let bytes = golden();
        let (report, observed) =
            with_verified_backend_input(&program, &diagnostics, &bytes, |capability| {
                (
                    capability.compiler_version().to_string(),
                    capability.semantic_contract().to_string(),
                    capability.target_context().to_string(),
                    capability.source_revision().to_string(),
                    capability.source_path().to_string(),
                    capability.function_id().to_string(),
                    capability.linkage().0.to_string(),
                    capability.parameter_value_ids()[0].to_string(),
                    capability.parameter_definition_ids()[0].to_string(),
                    capability.parameter_types()[0].to_string(),
                    capability.parameter_spans()[0],
                    capability.result().0.to_string(),
                    capability.operation_span(),
                    capability.overflow_edge().1.to_string(),
                )
            });
        assert!(report.accepted());
        let observed = observed.unwrap();
        assert_eq!(observed.0, version::HUM_VERSION);
        assert_eq!(observed.1, backend_input::SEMANTIC_CONTRACT);
        assert_eq!(observed.2, backend_input::TARGET_CONTEXT);
        assert_eq!(observed.3, backend_input::SOURCE_REVISION_SHA256);
        assert_eq!(observed.4, backend_input::SOURCE_PATH);
        assert_eq!(observed.5, "function:0");
        assert_eq!(observed.6, "internal");
        assert_eq!(observed.10, (3, 10));
        assert_eq!(observed.12, (8, 5));
        assert_eq!(observed.13, "checked_add");
    }
}
