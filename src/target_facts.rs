use crate::version;

pub const TARGET_FACTS_SCHEMA: &str = "hum.target_facts.v0";
pub const TARGET_FACT_RECORD_SCHEMA: &str = "hum.target_fact_record.v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTargetDeclarationKind {
    TargetFactRecord,
    RequiredCapabilityFamily,
    DeniedCapabilityFamily,
}

impl SourceTargetDeclarationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TargetFactRecord => "target_fact_record",
            Self::RequiredCapabilityFamily => "required_capability_family",
            Self::DeniedCapabilityFamily => "denied_capability_family",
        }
    }
}

struct TargetFactField {
    name: &'static str,
    kind: &'static str,
    required: bool,
    description: &'static str,
}

struct CapabilityFamily {
    family: &'static str,
    status: &'static str,
    examples: &'static [&'static str],
    absence_policy: &'static str,
}

struct CapabilityAvailability {
    family: &'static str,
    availability: &'static str,
    note: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetCapabilityStatus {
    pub target_id: &'static str,
    pub availability: &'static str,
    pub note: &'static str,
}

impl TargetCapabilityStatus {
    pub fn is_unavailable(self) -> bool {
        is_unavailable_availability(self.availability)
    }
}

struct TargetFixture {
    id: &'static str,
    status: &'static str,
    triple: &'static str,
    os: &'static str,
    arch: &'static str,
    abi: &'static str,
    endian: &'static str,
    pointer_width_bits: u8,
    path_kind: &'static str,
    newline_policy: &'static str,
    filesystem: &'static str,
    process: &'static str,
    network: &'static str,
    clock: &'static str,
    random: &'static str,
    atomics: &'static str,
    simd: &'static str,
    capabilities: &'static [CapabilityAvailability],
    non_claims: &'static [&'static str],
}

const TARGET_FACT_FIELDS: &[TargetFactField] = &[
    TargetFactField {
        name: "triple",
        kind: "identity",
        required: true,
        description: "stable target identity used by tooling and artifact evidence",
    },
    TargetFactField {
        name: "os",
        kind: "identity",
        required: true,
        description: "operating-system or host-environment family",
    },
    TargetFactField {
        name: "arch",
        kind: "identity",
        required: true,
        description: "processor or virtual-machine architecture family",
    },
    TargetFactField {
        name: "abi",
        kind: "layout",
        required: true,
        description: "calling convention and binary-interface family",
    },
    TargetFactField {
        name: "endian",
        kind: "layout",
        required: true,
        description: "byte order for stored numeric representations",
    },
    TargetFactField {
        name: "pointer_width_bits",
        kind: "layout",
        required: true,
        description: "pointer width visible to layout-sensitive lowering",
    },
    TargetFactField {
        name: "path_kind",
        kind: "platform",
        required: true,
        description: "path semantics family, not a string formatting hint",
    },
    TargetFactField {
        name: "newline_policy",
        kind: "platform",
        required: true,
        description: "source and artifact newline normalization policy",
    },
    TargetFactField {
        name: "filesystem",
        kind: "authority",
        required: true,
        description: "file API availability before profile denial is applied",
    },
    TargetFactField {
        name: "process",
        kind: "authority",
        required: true,
        description: "spawn, environment, and exit behavior availability",
    },
    TargetFactField {
        name: "network",
        kind: "authority",
        required: true,
        description: "network API availability before profile denial is applied",
    },
    TargetFactField {
        name: "clock",
        kind: "authority",
        required: true,
        description: "monotonic and wall-clock availability class",
    },
    TargetFactField {
        name: "random",
        kind: "authority",
        required: true,
        description: "entropy and deterministic-seed availability class",
    },
    TargetFactField {
        name: "atomics",
        kind: "hardware",
        required: true,
        description: "atomic operation support relevant to lowering",
    },
    TargetFactField {
        name: "simd",
        kind: "hardware",
        required: true,
        description: "baseline vector feature class when known",
    },
];

const CAPABILITY_FAMILIES: &[CapabilityFamily] = &[
    CapabilityFamily {
        family: "target.layout",
        status: "reserved",
        examples: &["endian", "alignment", "pointer_width_bits", "abi"],
        absence_policy: "diagnose when layout-sensitive code lacks required facts",
    },
    CapabilityFamily {
        family: "target.cpu",
        status: "reserved",
        examples: &["atomics", "simd", "accelerator_features"],
        absence_policy: "diagnose or require a fallback when code names a CPU feature",
    },
    CapabilityFamily {
        family: "target.memory",
        status: "reserved",
        examples: &["heap", "stack", "pages", "shared_memory"],
        absence_policy: "profile decides whether fallback, recomputation, or rejection is allowed",
    },
    CapabilityFamily {
        family: "target.path",
        status: "reserved",
        examples: &["separator", "roots", "case_sensitivity", "reserved_names"],
        absence_policy: "reject OS-path operations without a target path model",
    },
    CapabilityFamily {
        family: "os.filesystem",
        status: "reserved",
        examples: &["read", "write", "create", "delete", "watch"],
        absence_policy: "strict profiles deny hidden file authority",
    },
    CapabilityFamily {
        family: "os.clock",
        status: "reserved",
        examples: &["monotonic_time", "wall_time", "timers"],
        absence_policy: "deterministic profiles reject hidden wall-clock authority",
    },
    CapabilityFamily {
        family: "os.random",
        status: "reserved",
        examples: &["system_entropy", "deterministic_seed"],
        absence_policy: "reproducible profiles reject hidden entropy",
    },
    CapabilityFamily {
        family: "os.process",
        status: "reserved",
        examples: &["spawn", "exec", "environment", "exit_status"],
        absence_policy: "shell-like behavior requires explicit typed authority",
    },
    CapabilityFamily {
        family: "os.network",
        status: "reserved",
        examples: &["sockets", "dns", "tls", "ports", "endpoints"],
        absence_policy: "offline profiles deny network by default",
    },
    CapabilityFamily {
        family: "sandbox.host",
        status: "reserved",
        examples: &["wasi_imports", "browser_apis", "plugin_host_calls"],
        absence_policy: "sandbox imports must be declared and adapter-owned",
    },
    CapabilityFamily {
        family: "artifact.release",
        status: "reserved",
        examples: &["hashes", "signatures", "sbom", "provenance"],
        absence_policy: "artifact cannot claim auditable portability without evidence",
    },
];

const WINDOWS_CAPABILITIES: &[CapabilityAvailability] = &[
    CapabilityAvailability {
        family: "target.layout",
        availability: "known_fixture",
        note: "little-endian x86_64 with 64-bit pointers",
    },
    CapabilityAvailability {
        family: "target.path",
        availability: "known_fixture",
        note: "Windows paths require roots, drives, reserved names, and normalization policy",
    },
    CapabilityAvailability {
        family: "os.filesystem",
        availability: "available_profile_gated",
        note: "strict profiles still require declared roots",
    },
    CapabilityAvailability {
        family: "os.clock",
        availability: "monotonic_and_wall_available_profile_gated",
        note: "clock authority must stay visible to deterministic profiles",
    },
    CapabilityAvailability {
        family: "os.random",
        availability: "system_available_profile_gated",
        note: "system entropy is available but reproducibility profiles must gate it",
    },
    CapabilityAvailability {
        family: "os.process",
        availability: "available_profile_gated",
        note: "process spawning must remain explicit authority",
    },
    CapabilityAvailability {
        family: "os.network",
        availability: "available_profile_gated",
        note: "offline-tool profiles deny it",
    },
];

const LINUX_CAPABILITIES: &[CapabilityAvailability] = &[
    CapabilityAvailability {
        family: "target.layout",
        availability: "known_fixture",
        note: "little-endian x86_64 with 64-bit pointers",
    },
    CapabilityAvailability {
        family: "target.path",
        availability: "known_fixture",
        note: "POSIX-style path model requires explicit symlink and root policy",
    },
    CapabilityAvailability {
        family: "os.filesystem",
        availability: "available_profile_gated",
        note: "strict profiles still require declared roots",
    },
    CapabilityAvailability {
        family: "os.clock",
        availability: "monotonic_and_wall_available_profile_gated",
        note: "clock authority must stay visible to deterministic profiles",
    },
    CapabilityAvailability {
        family: "os.random",
        availability: "system_available_profile_gated",
        note: "system entropy is available but reproducibility profiles must gate it",
    },
    CapabilityAvailability {
        family: "os.process",
        availability: "available_profile_gated",
        note: "environment and executable lookup are explicit authority",
    },
    CapabilityAvailability {
        family: "os.network",
        availability: "available_profile_gated",
        note: "offline-tool profiles deny it",
    },
];

const WASI_CAPABILITIES: &[CapabilityAvailability] = &[
    CapabilityAvailability {
        family: "target.layout",
        availability: "known_fixture",
        note: "wasm32 linear-memory layout with 32-bit pointers",
    },
    CapabilityAvailability {
        family: "target.path",
        availability: "sandbox_adapter_owned",
        note: "paths are virtual host imports, not native OS paths",
    },
    CapabilityAvailability {
        family: "os.filesystem",
        availability: "import_required",
        note: "preopened directories define authority",
    },
    CapabilityAvailability {
        family: "os.clock",
        availability: "host_import_profile_gated",
        note: "clock depends on host imports and deterministic profile policy",
    },
    CapabilityAvailability {
        family: "os.random",
        availability: "host_import_profile_gated",
        note: "randomness depends on host imports and reproducibility policy",
    },
    CapabilityAvailability {
        family: "os.process",
        availability: "mostly_absent",
        note: "process behavior is host-defined and must be declared",
    },
    CapabilityAvailability {
        family: "os.network",
        availability: "absent_by_default",
        note: "network requires host-specific imports",
    },
];

const EMBEDDED_CAPABILITIES: &[CapabilityAvailability] = &[
    CapabilityAvailability {
        family: "target.layout",
        availability: "known_fixture",
        note: "little-endian ARM with 32-bit pointers",
    },
    CapabilityAvailability {
        family: "target.memory",
        availability: "profile_required",
        note: "heap, stack, and static regions must be explicit",
    },
    CapabilityAvailability {
        family: "target.path",
        availability: "absent_by_default",
        note: "no native filesystem path model",
    },
    CapabilityAvailability {
        family: "os.filesystem",
        availability: "absent_by_default",
        note: "no native filesystem authority",
    },
    CapabilityAvailability {
        family: "os.clock",
        availability: "device_specific",
        note: "time depends on explicit hardware timer authority",
    },
    CapabilityAvailability {
        family: "os.random",
        availability: "device_specific",
        note: "entropy depends on explicit hardware or seed authority",
    },
    CapabilityAvailability {
        family: "os.process",
        availability: "absent_by_default",
        note: "no general process model",
    },
    CapabilityAvailability {
        family: "os.network",
        availability: "device_specific",
        note: "network requires explicit device or bus authority",
    },
];

const TARGET_FIXTURES: &[TargetFixture] = &[
    TargetFixture {
        id: "windows-x86_64-msvc",
        status: "fixture",
        triple: "windows-x86_64-msvc",
        os: "windows",
        arch: "x86_64",
        abi: "msvc",
        endian: "little",
        pointer_width_bits: 64,
        path_kind: "windows",
        newline_policy: "preserve",
        filesystem: "available_profile_gated",
        process: "available_profile_gated",
        network: "available_profile_gated",
        clock: "monotonic_and_wall_available_profile_gated",
        random: "system_available_profile_gated",
        atomics: "u64_available",
        simd: "sse2_baseline",
        capabilities: WINDOWS_CAPABILITIES,
        non_claims: &[
            "not probed by this binary",
            "not a backend target selection",
            "not artifact evidence",
        ],
    },
    TargetFixture {
        id: "linux-x86_64-gnu",
        status: "fixture",
        triple: "linux-x86_64-gnu",
        os: "linux",
        arch: "x86_64",
        abi: "gnu",
        endian: "little",
        pointer_width_bits: 64,
        path_kind: "posix",
        newline_policy: "preserve",
        filesystem: "available_profile_gated",
        process: "available_profile_gated",
        network: "available_profile_gated",
        clock: "monotonic_and_wall_available_profile_gated",
        random: "system_available_profile_gated",
        atomics: "u64_available",
        simd: "sse2_baseline",
        capabilities: LINUX_CAPABILITIES,
        non_claims: &[
            "not probed by this binary",
            "not a backend target selection",
            "not artifact evidence",
        ],
    },
    TargetFixture {
        id: "wasm32-wasi-preview1",
        status: "fixture",
        triple: "wasm32-wasi-preview1",
        os: "wasi",
        arch: "wasm32",
        abi: "wasi-preview1",
        endian: "little",
        pointer_width_bits: 32,
        path_kind: "wasi_virtual",
        newline_policy: "preserve",
        filesystem: "import_required",
        process: "mostly_absent",
        network: "absent_by_default",
        clock: "host_import_profile_gated",
        random: "host_import_profile_gated",
        atomics: "profile_dependent",
        simd: "profile_dependent",
        capabilities: WASI_CAPABILITIES,
        non_claims: &[
            "not probed by this binary",
            "not a Wasm artifact claim",
            "not a host import grant",
        ],
    },
    TargetFixture {
        id: "thumbv7em-none-eabihf",
        status: "fixture",
        triple: "thumbv7em-none-eabihf",
        os: "none",
        arch: "thumbv7em",
        abi: "eabihf",
        endian: "little",
        pointer_width_bits: 32,
        path_kind: "absent",
        newline_policy: "preserve",
        filesystem: "absent_by_default",
        process: "absent_by_default",
        network: "device_specific",
        clock: "device_specific",
        random: "device_specific",
        atomics: "profile_dependent",
        simd: "profile_dependent",
        capabilities: EMBEDDED_CAPABILITIES,
        non_claims: &[
            "not probed by this binary",
            "not firmware support",
            "not safety-critical qualification",
        ],
    },
];

pub fn parse_source_target_declaration_line(
    text: &str,
) -> Option<(SourceTargetDeclarationKind, String)> {
    let (key, value) = text.split_once(':')?;
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    match key.trim() {
        "triple" | "record" | "target" => Some((
            SourceTargetDeclarationKind::TargetFactRecord,
            value.to_string(),
        )),
        "requires" | "requires capability" | "requires capability family" => Some((
            SourceTargetDeclarationKind::RequiredCapabilityFamily,
            value.to_string(),
        )),
        "denies" | "denies capability" | "denies capability family" => Some((
            SourceTargetDeclarationKind::DeniedCapabilityFamily,
            value.to_string(),
        )),
        _ => None,
    }
}

pub fn is_known_target_fact_record(value: &str) -> bool {
    TARGET_FIXTURES
        .iter()
        .any(|fixture| fixture.id == value || fixture.triple == value)
}

pub fn is_known_capability_family(value: &str) -> bool {
    CAPABILITY_FAMILIES
        .iter()
        .any(|family| family.family == value)
}

pub fn target_required_capability_status(
    target_record: &str,
    family: &str,
) -> Option<TargetCapabilityStatus> {
    if !is_known_capability_family(family) {
        return None;
    }

    let fixture = find_target_fixture(target_record)?;
    let capability = fixture
        .capabilities
        .iter()
        .find(|capability| capability.family == family);

    Some(match capability {
        Some(capability) => TargetCapabilityStatus {
            target_id: fixture.id,
            availability: capability.availability,
            note: capability.note,
        },
        None => TargetCapabilityStatus {
            target_id: fixture.id,
            availability: "absent_by_policy",
            note: "target fact record omits this capability family",
        },
    })
}

pub fn unavailable_required_capability_families(
    target_records: &[String],
    required_capability_families: &[String],
) -> Vec<String> {
    required_capability_families
        .iter()
        .filter(|family| {
            target_records.iter().any(|target_record| {
                target_required_capability_status(target_record, family)
                    .is_some_and(TargetCapabilityStatus::is_unavailable)
            })
        })
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn find_target_fixture(value: &str) -> Option<&'static TargetFixture> {
    TARGET_FIXTURES
        .iter()
        .find(|fixture| fixture.id == value || fixture.triple == value)
}

fn is_unavailable_availability(availability: &str) -> bool {
    matches!(
        availability,
        "absent_by_default" | "mostly_absent" | "absent_by_policy"
    )
}

const NON_GOALS_V0: &[&str] = &[
    "no host capability probing",
    "no backend target selection",
    "no artifact generation",
    "no runtime profile enforcement",
    "no package or foreign build-script execution",
    "no portability certification claim",
];

pub fn target_facts_text() -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Hum target facts ({TARGET_FACTS_SCHEMA})\ntool: hum {} {}\nmilestone: {}\nrecord_schema: {TARGET_FACT_RECORD_SCHEMA}\nmode: contract_only_no_host_probe\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        version::HUM_MILESTONE
    ));
    out.push_str("field_catalog:\n");
    for field in TARGET_FACT_FIELDS {
        out.push_str(&format!(
            "  {} [{} required={}]: {}\n",
            field.name, field.kind, field.required, field.description
        ));
    }
    out.push_str("capability_families:\n");
    for family in CAPABILITY_FAMILIES {
        out.push_str(&format!(
            "  {} [{}]: {}\n",
            family.family, family.status, family.absence_policy
        ));
    }
    out.push_str("fixture_records:\n");
    for fixture in TARGET_FIXTURES {
        out.push_str(&format!(
            "  {} [{}]: os={} arch={} abi={} path={} network={}\n",
            fixture.id,
            fixture.status,
            fixture.os,
            fixture.arch,
            fixture.abi,
            fixture.path_kind,
            fixture.network
        ));
    }
    out.push_str("non_goals_v0:\n");
    for non_goal in NON_GOALS_V0 {
        out.push_str(&format!("  {non_goal}\n"));
    }
    out
}

pub fn target_facts_json() -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", TARGET_FACTS_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(
        &mut out,
        2,
        "record_schema",
        TARGET_FACT_RECORD_SCHEMA,
        true,
    );
    push_string_field(&mut out, 2, "mode", "contract_only_no_host_probe", true);
    push_string_field(
        &mut out,
        2,
        "boundary_model",
        "docs/PORTABILITY_BOUNDARY_MODEL.md",
        true,
    );
    push_string_field(&mut out, 2, "default_policy", "unknown_fails_closed", true);
    push_field_catalog(&mut out, 2, true);
    push_capability_families(&mut out, 2, true);
    push_fixture_records(&mut out, 2, true);
    push_string_array(&mut out, 2, "non_goals_v0", NON_GOALS_V0, false);
    out.push_str("}\n");
    out
}

fn push_field_catalog(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"field_catalog\": [\n");
    for (index, field) in TARGET_FACT_FIELDS.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "name", field.name, true);
        push_string_field(out, indent + 4, "kind", field.kind, true);
        push_bool_field(out, indent + 4, "required", field.required, true);
        push_string_field(out, indent + 4, "description", field.description, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_capability_families(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"capability_families\": [\n");
    for (index, family) in CAPABILITY_FAMILIES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "family", family.family, true);
        push_string_field(out, indent + 4, "status", family.status, true);
        push_string_array(out, indent + 4, "examples", family.examples, true);
        push_string_field(
            out,
            indent + 4,
            "absence_policy",
            family.absence_policy,
            false,
        );
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_fixture_records(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"fixture_records\": [\n");
    for (index, fixture) in TARGET_FIXTURES.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_target_record(out, indent + 2, fixture);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_target_record(out: &mut String, indent: usize, fixture: &TargetFixture) {
    push_indent(out, indent);
    out.push_str("{\n");
    push_string_field(out, indent + 2, "schema", TARGET_FACT_RECORD_SCHEMA, true);
    push_string_field(out, indent + 2, "id", fixture.id, true);
    push_string_field(out, indent + 2, "status", fixture.status, true);
    push_string_field(
        out,
        indent + 2,
        "absence_policy",
        "unknown_or_absent_capabilities_fail_closed",
        true,
    );
    push_facts(out, indent + 2, fixture, true);
    push_capability_availability(out, indent + 2, fixture.capabilities, true);
    push_string_array(out, indent + 2, "non_claims", fixture.non_claims, false);
    push_indent(out, indent);
    out.push('}');
}

fn push_facts(out: &mut String, indent: usize, fixture: &TargetFixture, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"facts\": {\n");
    push_string_field(out, indent + 2, "triple", fixture.triple, true);
    push_string_field(out, indent + 2, "os", fixture.os, true);
    push_string_field(out, indent + 2, "arch", fixture.arch, true);
    push_string_field(out, indent + 2, "abi", fixture.abi, true);
    push_string_field(out, indent + 2, "endian", fixture.endian, true);
    push_u8_field(
        out,
        indent + 2,
        "pointer_width_bits",
        fixture.pointer_width_bits,
        true,
    );
    push_string_field(out, indent + 2, "path_kind", fixture.path_kind, true);
    push_string_field(
        out,
        indent + 2,
        "newline_policy",
        fixture.newline_policy,
        true,
    );
    push_string_field(out, indent + 2, "filesystem", fixture.filesystem, true);
    push_string_field(out, indent + 2, "process", fixture.process, true);
    push_string_field(out, indent + 2, "network", fixture.network, true);
    push_string_field(out, indent + 2, "clock", fixture.clock, true);
    push_string_field(out, indent + 2, "random", fixture.random, true);
    push_string_field(out, indent + 2, "atomics", fixture.atomics, true);
    push_string_field(out, indent + 2, "simd", fixture.simd, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_capability_availability(
    out: &mut String,
    indent: usize,
    capabilities: &[CapabilityAvailability],
    comma: bool,
) {
    push_indent(out, indent);
    out.push_str("\"capabilities\": [\n");
    for (index, capability) in capabilities.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "family", capability.family, true);
        push_string_field(
            out,
            indent + 4,
            "availability",
            capability.availability,
            true,
        );
        push_string_field(out, indent + 4, "note", capability.note, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": [\n");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        push_json_string(out, value);
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_string_field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(": ");
    push_json_string(out, value);
    push_comma_newline(out, comma);
}

fn push_bool_field(out: &mut String, indent: usize, key: &str, value: bool, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(if value { ": true" } else { ": false" });
    push_comma_newline(out, comma);
}

fn push_u8_field(out: &mut String, indent: usize, key: &str, value: u8, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(&format!(": {value}"));
    push_comma_newline(out, comma);
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
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn push_comma_newline(out: &mut String, comma: bool) {
    if comma {
        out.push(',');
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::{
        SourceTargetDeclarationKind, is_known_capability_family, is_known_target_fact_record,
        parse_source_target_declaration_line, target_facts_json, target_facts_text,
        target_required_capability_status, unavailable_required_capability_families,
    };

    #[test]
    fn text_target_facts_lists_portability_contract() {
        let text = target_facts_text();

        assert!(text.contains("Hum target facts (hum.target_facts.v0)"));
        assert!(text.contains("record_schema: hum.target_fact_record.v0"));
        assert!(text.contains("mode: contract_only_no_host_probe"));
        assert!(text.contains("target.layout"));
        assert!(text.contains("windows-x86_64-msvc"));
        assert!(text.contains("wasm32-wasi-preview1"));
        assert!(text.contains("no host capability probing"));
    }

    #[test]
    fn parses_and_validates_source_target_declaration_lines() {
        assert_eq!(
            parse_source_target_declaration_line("triple: wasm32-wasi-preview1"),
            Some((
                SourceTargetDeclarationKind::TargetFactRecord,
                "wasm32-wasi-preview1".to_string()
            ))
        );
        assert_eq!(
            parse_source_target_declaration_line("requires: os.filesystem"),
            Some((
                SourceTargetDeclarationKind::RequiredCapabilityFamily,
                "os.filesystem".to_string()
            ))
        );
        assert_eq!(
            parse_source_target_declaration_line("denies: os.network"),
            Some((
                SourceTargetDeclarationKind::DeniedCapabilityFamily,
                "os.network".to_string()
            ))
        );
        assert_eq!(
            parse_source_target_declaration_line("maybe: os.network"),
            None
        );

        assert!(is_known_target_fact_record("wasm32-wasi-preview1"));
        assert!(!is_known_target_fact_record("mars32-secret"));
        assert!(is_known_capability_family("os.network"));
        assert!(!is_known_capability_family("os.telepathy"));
    }

    #[test]
    fn classifies_required_capability_availability_for_fixture_targets() {
        let network = target_required_capability_status("wasm32-wasi-preview1", "os.network")
            .expect("known WASI fixture");
        assert_eq!(network.availability, "absent_by_default");
        assert!(network.is_unavailable());

        let clock = target_required_capability_status("wasm32-wasi-preview1", "os.clock")
            .expect("known WASI fixture");
        assert_eq!(clock.availability, "host_import_profile_gated");
        assert!(!clock.is_unavailable());

        let cpu = target_required_capability_status("wasm32-wasi-preview1", "target.cpu")
            .expect("known WASI fixture");
        assert_eq!(cpu.availability, "absent_by_policy");
        assert!(cpu.is_unavailable());
        assert!(
            target_required_capability_status("wasm32-wasi-preview1", "os.telepathy").is_none()
        );

        assert_eq!(
            unavailable_required_capability_families(
                &["wasm32-wasi-preview1".to_string()],
                &["os.clock".to_string(), "os.network".to_string()]
            ),
            vec!["os.network".to_string()]
        );
    }

    #[test]
    fn json_target_facts_is_machine_readable() {
        let json = target_facts_json();

        assert!(json.contains("\"schema\": \"hum.target_facts.v0\""));
        assert!(json.contains("\"record_schema\": \"hum.target_fact_record.v0\""));
        assert!(json.contains("\"mode\": \"contract_only_no_host_probe\""));
        assert!(json.contains("\"default_policy\": \"unknown_fails_closed\""));
        assert!(json.contains("\"name\": \"pointer_width_bits\""));
        assert!(json.contains("\"family\": \"os.network\""));
        assert!(json.contains("\"id\": \"windows-x86_64-msvc\""));
        assert!(json.contains("\"id\": \"linux-x86_64-gnu\""));
        assert!(json.contains("\"id\": \"wasm32-wasi-preview1\""));
        assert!(json.contains("\"id\": \"thumbv7em-none-eabihf\""));
        assert!(
            json.contains("\"absence_policy\": \"unknown_or_absent_capabilities_fail_closed\"")
        );
        assert!(json.contains("\"not artifact evidence\""));
    }
}
