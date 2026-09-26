// Production-connected CLI tests for Decision 0030 Option B.
// These spawn the actual `hum` binary to verify: human output, JSON diagnostics,
// exit status, and stages together. Adapter-only tests are insufficient per review.

use std::path::{Path, PathBuf};
use std::process::Command;

fn hum_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hum"))
}

fn write_cli_fixture(name: &str, content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "hum_decision0030_cli_{}_{}",
        name,
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(format!("{name}.hum"));
    std::fs::write(&path, content).expect("write fixture");
    path
}

fn run_hum_check(path: &Path, json: bool) -> std::process::Output {
    let mut cmd = Command::new(hum_binary());
    cmd.arg("check");
    if json {
        cmd.arg("--format=json");
    }
    cmd.arg(path);
    cmd.output().expect("run hum check")
}

fn run_hum_check_multi(paths: &[&Path], json: bool) -> std::process::Output {
    let mut cmd = Command::new(hum_binary());
    cmd.arg("check");
    if json {
        cmd.arg("--format=json");
    }
    for path in paths {
        cmd.arg(path);
    }
    cmd.output().expect("run hum check")
}

#[test]
fn cli_h0606_return_mismatch_via_production_binary() {
    // F1: H0606 must be reported via the real CLI, not dropped by the adapter.
    let path = write_cli_fixture(
        "h0606",
        "task bad_return(title: Text) -> UInt {\n  does:\n    return title\n}\n",
    );
    // Human output.
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1), "H0606 must exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("H0606"),
        "human output must contain H0606: {stderr}"
    );
    assert!(
        stderr.contains("help:"),
        "human output must preserve help text: {stderr}"
    );
    // JSON output.
    let out_json = run_hum_check(&path, true);
    assert_eq!(out_json.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out_json.stdout);
    assert!(
        stdout.contains("H0606"),
        "JSON must contain H0606: {stdout}"
    );
    assert!(
        stdout.contains("type_check"),
        "JSON stages must include type_check: {stdout}"
    );
    // full_type_check must NOT run after type_check errors (precedence).
    assert!(
        !stdout.contains("full_type_check"),
        "JSON must stop at type_check on error: {stdout}"
    );
}

#[test]
fn cli_h0643_full_type_rejection_via_production_binary() {
    // F2 (BDFL-authorized H0643): The full-type statement mismatch must surface
    // as a registered diagnostic through the ordinary pipeline. Human and JSON
    // must expose the same rejection: code, source location, explanation/help,
    // accurate error count, exit 1.
    let path = write_cli_fixture(
        "h0643",
        "task add(a: Int, b: Int) -> UInt {\n  does:\n    return a + b\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1), "H0643 rejection must exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    // Human output must name H0643 with source location and help.
    assert!(
        combined.contains("H0643"),
        "human output must contain H0643: {combined}"
    );
    assert!(
        combined.contains("3:5"),
        "human output must contain source location 3:5: {combined}"
    );
    assert!(
        combined.contains("has type Int") && combined.contains("requires UInt"),
        "human output must explain the mismatch: {combined}"
    );
    assert!(
        combined.contains("1 error(s)"),
        "human summary must report 1 error: {combined}"
    );
    // JSON: must contain the actual H0643 diagnostic with code, span, message,
    // and accurate error count — not just exit 1 and stages.
    let out_json = run_hum_check(&path, true);
    assert_eq!(out_json.status.code(), Some(1));
    let json_stdout = String::from_utf8_lossy(&out_json.stdout);
    assert!(
        json_stdout.contains("\"code\": \"H0643\""),
        "JSON must contain H0643 code: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"title\": \"statement expression type mismatch\""),
        "JSON must contain H0643 title: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"severity\": \"error\""),
        "JSON must mark H0643 as error: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"line\": 3"),
        "JSON must contain line 3: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"errors\": 1"),
        "JSON summary must report 1 error: {json_stdout}"
    );
    assert!(
        json_stdout.contains("full_type_check"),
        "JSON stages must include full_type_check: {json_stdout}"
    );
}

#[test]
fn cli_h0640_arity_via_production_binary() {
    let path = write_cli_fixture(
        "h0640",
        "task callee(a: UInt, b: UInt) -> UInt {\n  does:\n    return a\n}\ntask caller() -> UInt {\n  does:\n    return callee(1)\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("H0640"), "must contain H0640: {stderr}");
}

#[test]
fn cli_h0641_arg_type_via_production_binary() {
    let path = write_cli_fixture(
        "h0641",
        "task callee(a: UInt) -> UInt {\n  does:\n    return a\n}\ntask caller() -> UInt {\n  does:\n    return callee(\"hello\")\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("H0641"), "must contain H0641: {stderr}");
}

#[test]
fn cli_h0642_negative_uint_via_production_binary() {
    let path = write_cli_fixture(
        "h0642",
        "task taker(x: UInt) -> UInt {\n  does:\n    return x\n}\ntask caller() -> UInt {\n  does:\n    return taker(-5)\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("H0642"), "must contain H0642: {stderr}");
}

#[test]
fn cli_honest_success_via_production_binary() {
    // A clean file must exit 0 with all stages in JSON.
    let path = write_cli_fixture(
        "clean",
        "task main() -> UInt {\n  does:\n    return 42\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(
        out.status.code(),
        Some(0),
        "clean file must exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out_json = run_hum_check(&path, true);
    assert_eq!(out_json.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out_json.stdout);
    for stage in [
        "parse",
        "source_check",
        "app_entry",
        "type_check",
        "full_type_check",
    ] {
        assert!(
            stdout.contains(stage),
            "JSON stages must include {stage}: {stdout}"
        );
    }
}

#[test]
fn cli_combined_errors_precedence_via_production_binary() {
    // Multiple errors: type_check errors block full_type_check (precedence).
    // H0606 retains its existing meaning (type_check-stage trivial return-type
    // diagnostic); H0643 (full-type-proved mismatch) must not appear when
    // type_check already rejected.
    let path = write_cli_fixture(
        "combined",
        "task bad(title: Text) -> UInt {\n  does:\n    return title\n}\ntask add(a: Int, b: Int) -> UInt {\n  does:\n    return a + b\n}\n",
    );
    let out_json = run_hum_check(&path, true);
    assert_eq!(out_json.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out_json.stdout);
    // H0606 (type_check) must appear; full_type_check must NOT run (precedence).
    assert!(stdout.contains("H0606"), "must contain H0606: {stdout}");
    assert!(
        !stdout.contains("H0643"),
        "precedence: H0643 must not appear when type_check errors block full_type_check: {stdout}"
    );
    assert!(
        stdout.contains("type_check"),
        "stages must include type_check: {stdout}"
    );
    assert!(
        !stdout.contains("full_type_check"),
        "precedence: full_type_check must not run after type_check errors: {stdout}"
    );
}

#[test]
fn cli_d3_parse_error_stages_via_production_binary() {
    // D3 intersection semantics: a parse error means `app_entry` (and later
    // stages) did not run for all files. Stages must be exactly
    // ["parse", "source_check"] — honest about what ran.
    let path = write_cli_fixture(
        "parse_error",
        "task broken( -> UInt {\n  does:\n    return 42\n}\n",
    );
    let out_json = run_hum_check(&path, true);
    assert_eq!(out_json.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out_json.stdout);
    // Parse the stages array from JSON.
    assert!(
        stdout.contains("\"stages\": [\"parse\", \"source_check\"]"),
        "parse error stages must be exactly [parse, source_check]: {stdout}"
    );
    assert!(
        !stdout.contains("app_entry"),
        "app_entry must not appear for parse error: {stdout}"
    );
}

#[test]
fn cli_d3_source_error_stages_via_production_binary() {
    // D3 intersection semantics: a source_check error (e.g., reserved builtin
    // name) means `app_entry` did not run. Stages must be ["parse", "source_check"].
    let path = write_cli_fixture(
        "source_error",
        "task stdout_write() -> UInt {\n  does:\n    return 42\n}\n",
    );
    let out_json = run_hum_check(&path, true);
    assert_eq!(out_json.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out_json.stdout);
    assert!(
        stdout.contains("\"stages\": [\"parse\", \"source_check\"]"),
        "source error stages must be exactly [parse, source_check]: {stdout}"
    );
    assert!(
        !stdout.contains("app_entry"),
        "app_entry must not appear for source error: {stdout}"
    );
}

#[test]
fn cli_d3_mixed_input_stages_via_production_binary() {
    // D3 intersection semantics: with mixed clean/error inputs, a stage appears
    // only if it ran for EVERY file. The clean file would run app_entry, but
    // the error file skips it — so stages must be ["parse", "source_check"].
    let clean = write_cli_fixture(
        "mixed_clean",
        "task main() -> UInt {\n  does:\n    return 42\n}\n",
    );
    let error = write_cli_fixture(
        "mixed_error",
        "task broken( -> UInt {\n  does:\n    return 42\n}\n",
    );
    let out_json = run_hum_check_multi(&[&clean, &error], true);
    assert_eq!(out_json.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out_json.stdout);
    assert!(
        stdout.contains("\"stages\": [\"parse\", \"source_check\"]"),
        "mixed input stages must be exactly [parse, source_check] (intersection): {stdout}"
    );
    assert!(
        !stdout.contains("app_entry"),
        "app_entry must not appear for mixed inputs (did not run for all files): {stdout}"
    );
}
#[test]
fn cli_result_multiword_success_type_positive() {
    let path = write_cli_fixture(
        "result_multiword_positive",
        "type SplitError {\n  code: Text\n}\n\ntask split_args(text: Text) -> Result List Text, SplitError {\n  does:\n    let pieces = text_split(text)\n    return pieces\n}\n",
    );
    let out = run_hum_check(&path, false);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("H0643"),
        "positive must not have H0643: {combined}"
    );
}

#[test]
fn cli_result_multiword_success_type_negative() {
    // Negative: genuine mismatch with multiword Result success type must
    // still be rejected. The diagnostic (H0606 at type-check or H0643 at
    // full-type-check) must name the complete expected type "List Text",
    // proving the projection preserves multiword success types.
    let path = write_cli_fixture(
        "result_multiword_negative",
        "type SplitError {\n  code: Text\n}\n\ntask split_args(text: Text) -> Result List Text, SplitError {\n  does:\n    return text\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1));
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("H0606") || combined.contains("H0643"),
        "negative must be rejected with H0606 or H0643: {combined}"
    );
    assert!(
        combined.contains("List Text"),
        "diagnostic must name the complete expected type List Text: {combined}"
    );
}

#[test]
fn cli_result_scalar_unchanged() {
    let path = write_cli_fixture(
        "result_scalar",
        "type WorkError {\n  code: Text\n}\n\ntask get_value() -> Result UInt, WorkError {\n  does:\n    return 42\n}\n",
    );
    let out = run_hum_check(&path, false);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("error["),
        "scalar must have no errors: {combined}"
    );
}
