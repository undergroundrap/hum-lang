use std::fs;
use std::path::Path;

use crate::version;

pub const DOCTOR_SCHEMA: &str = "hum.doctor.v0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            CheckStatus::Pass => "pass",
            CheckStatus::Warn => "warn",
            CheckStatus::Fail => "fail",
        }
    }
}

struct DoctorCheck {
    id: &'static str,
    label: &'static str,
    status: CheckStatus,
    required: bool,
    message: String,
}

#[derive(Default)]
struct DoctorSummary {
    pass: usize,
    warn: usize,
    fail: usize,
}

impl DoctorSummary {
    fn status(&self) -> CheckStatus {
        if self.fail > 0 {
            CheckStatus::Fail
        } else if self.warn > 0 {
            CheckStatus::Warn
        } else {
            CheckStatus::Pass
        }
    }
}

pub fn doctor_text() -> String {
    doctor_text_at(Path::new("."))
}

pub fn doctor_json() -> String {
    doctor_json_at(Path::new("."))
}

fn doctor_text_at(root: &Path) -> String {
    let checks = doctor_checks(root);
    let summary = summarize(&checks);
    let mut out = String::new();
    out.push_str(&format!(
        "Hum doctor ({DOCTOR_SCHEMA})\ntool: hum {} {}\nworkspace: current directory\nsummary: {} ({} pass, {} warn, {} fail)\n",
        version::HUM_VERSION,
        version::HUM_STATUS,
        summary.status().as_str(),
        summary.pass,
        summary.warn,
        summary.fail
    ));
    out.push_str("checks:\n");
    for check in &checks {
        out.push_str(&format!(
            "  {} {}: {}\n",
            check.status.as_str(),
            check.id,
            check.message
        ));
    }
    out.push_str("next:\n");
    for step in NEXT_STEPS {
        out.push_str(&format!("  {step}\n"));
    }
    out
}

fn doctor_json_at(root: &Path) -> String {
    let checks = doctor_checks(root);
    let summary = summarize(&checks);
    let mut out = String::new();
    out.push_str("{\n");
    push_string_field(&mut out, 2, "schema", DOCTOR_SCHEMA, true);
    push_string_field(&mut out, 2, "tool", "hum", true);
    push_string_field(&mut out, 2, "version", version::HUM_VERSION, true);
    push_string_field(&mut out, 2, "status", version::HUM_STATUS, true);
    push_string_field(&mut out, 2, "milestone", version::HUM_MILESTONE, true);
    push_string_field(&mut out, 2, "workspace", "current_directory", true);
    push_summary(&mut out, 2, &summary, true);
    push_checks(&mut out, 2, &checks, true);
    push_next_steps(&mut out, 2, false);
    out.push_str("}\n");
    out
}

fn doctor_checks(root: &Path) -> Vec<DoctorCheck> {
    vec![
        check_all_exist(
            root,
            "repo_manifest",
            "Repo manifest",
            true,
            &["Cargo.toml"],
            "Cargo.toml exists",
            "Cargo.toml is missing; run doctor from the Hum repo root",
        ),
        check_version_file(root),
        check_all_exist(
            root,
            "license_notice",
            "License and notice",
            true,
            &["LICENSE", "NOTICE.md"],
            "LICENSE and NOTICE.md exist",
            "LICENSE or NOTICE.md is missing",
        ),
        check_all_exist(
            root,
            "text_hygiene_policy",
            "Text hygiene policy",
            true,
            &[
                ".editorconfig",
                ".gitattributes",
                "tools/check_text_hygiene.ps1",
            ],
            "UTF-8, line-ending, and text hygiene policy files exist",
            "text hygiene policy files are missing",
        ),
        check_all_exist(
            root,
            "public_readiness_policy",
            "Public readiness policy",
            true,
            &[".gitignore", "tools/check_public_readiness.ps1"],
            "public-readiness guardrails exist",
            "public-readiness guardrails are missing",
        ),
        check_all_exist(
            root,
            "preflight_script",
            "Preflight script",
            true,
            &["tools/check_all.ps1"],
            "tools/check_all.ps1 exists",
            "tools/check_all.ps1 is missing",
        ),
        check_hosted_ci(root),
        check_all_exist(
            root,
            "setup_docs",
            "Setup docs",
            true,
            &["docs/SETUP.md", "docs/TEXT_HYGIENE_WORKFLOW.md"],
            "setup and text-hygiene docs exist",
            "setup or text-hygiene docs are missing",
        ),
        check_all_exist(
            root,
            "editor_assets",
            "Editor assets",
            true,
            &[
                "docs/EDITOR_AND_INTEGRATION_STRATEGY.md",
                "docs/LSP_CAPABILITY_MATRIX.md",
                "editors/textmate/hum.tmLanguage.json",
            ],
            "editor strategy, LSP matrix, and TextMate grammar exist",
            "editor strategy, LSP matrix, or TextMate grammar is missing",
        ),
        check_all_exist(
            root,
            "reference_fixtures",
            "Reference fixtures",
            true,
            &["examples/reference_surface.hum", "fixtures/editor"],
            "reference source and editor recovery fixtures exist",
            "reference source or editor recovery fixtures are missing",
        ),
    ]
}

fn check_version_file(root: &Path) -> DoctorCheck {
    let path = root.join("VERSION");
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let actual = contents.trim();
            if actual == version::HUM_VERSION {
                DoctorCheck {
                    id: "version_file",
                    label: "Version file",
                    status: CheckStatus::Pass,
                    required: true,
                    message: format!(
                        "VERSION matches Cargo package version {}",
                        version::HUM_VERSION
                    ),
                }
            } else {
                DoctorCheck {
                    id: "version_file",
                    label: "Version file",
                    status: CheckStatus::Fail,
                    required: true,
                    message: format!(
                        "VERSION is {actual}, but Cargo package version is {}",
                        version::HUM_VERSION
                    ),
                }
            }
        }
        Err(_) => DoctorCheck {
            id: "version_file",
            label: "Version file",
            status: CheckStatus::Fail,
            required: true,
            message: "VERSION is missing or unreadable".to_string(),
        },
    }
}

fn check_all_exist(
    root: &Path,
    id: &'static str,
    label: &'static str,
    required: bool,
    paths: &[&str],
    pass_message: &'static str,
    missing_message: &'static str,
) -> DoctorCheck {
    let missing = paths
        .iter()
        .filter(|path| !root.join(path).exists())
        .copied()
        .collect::<Vec<_>>();

    if missing.is_empty() {
        DoctorCheck {
            id,
            label,
            status: CheckStatus::Pass,
            required,
            message: pass_message.to_string(),
        }
    } else {
        let status = if required {
            CheckStatus::Fail
        } else {
            CheckStatus::Warn
        };
        DoctorCheck {
            id,
            label,
            status,
            required,
            message: format!("{missing_message}: {}", missing.join(", ")),
        }
    }
}

fn check_hosted_ci(root: &Path) -> DoctorCheck {
    let path = ".github/workflows/ci.yml";
    match fs::read_to_string(root.join(path)) {
        Ok(contents) if contents.contains("tools/check_all.ps1") => DoctorCheck {
            id: "hosted_ci",
            label: "Hosted CI",
            status: CheckStatus::Pass,
            required: false,
            message: "hosted CI calls the repo preflight".to_string(),
        },
        Ok(_) => DoctorCheck {
            id: "hosted_ci",
            label: "Hosted CI",
            status: CheckStatus::Warn,
            required: false,
            message: "hosted CI workflow does not call tools/check_all.ps1".to_string(),
        },
        Err(_) => DoctorCheck {
            id: "hosted_ci",
            label: "Hosted CI",
            status: CheckStatus::Warn,
            required: false,
            message: format!("hosted CI workflow is missing: {path}"),
        },
    }
}
fn summarize(checks: &[DoctorCheck]) -> DoctorSummary {
    let mut summary = DoctorSummary::default();
    for check in checks {
        match check.status {
            CheckStatus::Pass => summary.pass += 1,
            CheckStatus::Warn => summary.warn += 1,
            CheckStatus::Fail => summary.fail += 1,
        }
    }
    summary
}

const NEXT_STEPS: &[&str] = &[
    "Run tools/check_all.ps1 before commits and release-style handoffs.",
    "Keep editor-local state and absolute machine paths out of the repo.",
    "Use hum capabilities --format json before enabling optional adapters.",
];

fn push_summary(out: &mut String, indent: usize, summary: &DoctorSummary, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"summary\": {\n");
    push_string_field(out, indent + 2, "status", summary.status().as_str(), true);
    push_usize_field(out, indent + 2, "pass", summary.pass, true);
    push_usize_field(out, indent + 2, "warn", summary.warn, true);
    push_usize_field(out, indent + 2, "fail", summary.fail, false);
    push_indent(out, indent);
    out.push('}');
    push_comma_newline(out, comma);
}

fn push_checks(out: &mut String, indent: usize, checks: &[DoctorCheck], comma: bool) {
    push_indent(out, indent);
    out.push_str("\"checks\": [\n");
    for (index, check) in checks.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        out.push_str("{\n");
        push_string_field(out, indent + 4, "id", check.id, true);
        push_string_field(out, indent + 4, "label", check.label, true);
        push_string_field(out, indent + 4, "status", check.status.as_str(), true);
        push_bool_field(out, indent + 4, "required", check.required, true);
        push_string_field(out, indent + 4, "message", &check.message, false);
        push_indent(out, indent + 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, indent);
    out.push(']');
    push_comma_newline(out, comma);
}

fn push_next_steps(out: &mut String, indent: usize, comma: bool) {
    push_indent(out, indent);
    out.push_str("\"next_steps\": [\n");
    for (index, step) in NEXT_STEPS.iter().enumerate() {
        if index > 0 {
            out.push_str(",\n");
        }
        push_indent(out, indent + 2);
        push_json_string(out, step);
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

fn push_usize_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(&format!(": {value}"));
    push_comma_newline(out, comma);
}

fn push_bool_field(out: &mut String, indent: usize, key: &str, value: bool, comma: bool) {
    push_indent(out, indent);
    push_json_string(out, key);
    out.push_str(if value { ": true" } else { ": false" });
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
    use std::path::Path;

    use super::{doctor_json_at, doctor_text_at};

    #[test]
    fn text_doctor_report_summarizes_repo_health() {
        let text = doctor_text_at(Path::new(env!("CARGO_MANIFEST_DIR")));

        assert!(text.contains("Hum doctor (hum.doctor.v0)"));
        assert!(text.contains("summary: pass"));
        assert!(text.contains("text_hygiene_policy"));
        assert!(text.contains("public_readiness_policy"));
    }

    #[test]
    fn json_doctor_report_lists_portable_setup_checks() {
        let json = doctor_json_at(Path::new(env!("CARGO_MANIFEST_DIR")));

        assert!(json.contains("\"schema\": \"hum.doctor.v0\""));
        assert!(json.contains("\"workspace\": \"current_directory\""));
        assert!(json.contains("\"status\": \"pass\""));
        assert!(json.contains("\"id\": \"preflight_script\""));
        assert!(json.contains("\"id\": \"editor_assets\""));
    }
}
