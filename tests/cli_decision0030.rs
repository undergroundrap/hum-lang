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
fn cli_uncoded_full_type_rejection_via_production_binary() {
    // F2: Uncoded rejections must surface via the real CLI, not be silently dropped.
    let path = write_cli_fixture(
        "uncoded",
        "task add(a: Int, b: Int) -> UInt {\n  does:\n    return a + b\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1), "uncoded rejection must exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("rejection") || combined.contains("no registered diagnostic code"),
        "must report uncoded rejection: {combined}"
    );
    // JSON: exit 1, full_type_check in stages, even with zero coded diagnostics.
    let out_json = run_hum_check(&path, true);
    assert_eq!(out_json.status.code(), Some(1));
    let json_stdout = String::from_utf8_lossy(&out_json.stdout);
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
        stdout.contains("type_check"),
        "stages must include type_check: {stdout}"
    );
    assert!(
        !stdout.contains("full_type_check"),
        "precedence: full_type_check must not run after type_check errors: {stdout}"
    );
}
