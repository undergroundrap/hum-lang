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
    // Positive: multiword Result success type (Result List Text, SplitError)
    // with valid two-arg text_split call. Must exit 0, have zero errors,
    // and reach full_type_check stage. Absence of H0643 alone is insufficient.
    // Regression for Ubuntu job 108467610692.
    let path = write_cli_fixture(
        "result_multiword_positive",
        "type SplitError {\n  code: Text\n}\n\ntask split_args(text: Text, sep: Text) -> Result List Text, SplitError {\n  fails when:\n    The separator is empty.\n  does:\n    let pieces = try text_split(text, sep) or fail SplitError.split\n    return pieces\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(
        out.status.code(),
        Some(0),
        "multiword Result success must exit 0"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("error["),
        "multiword Result success must have zero errors: {combined}"
    );
    // JSON stages must include full_type_check, proving the repaired
    // projection was exercised.
    let out_json = run_hum_check(&path, true);
    let json_stdout = String::from_utf8_lossy(&out_json.stdout);
    assert!(
        json_stdout.contains("full_type_check"),
        "JSON stages must include full_type_check: {json_stdout}"
    );
}

#[test]
fn cli_result_multiword_success_type_negative() {
    // Negative: genuine mismatch must reach the repaired full-type projection
    // and produce exact H0643 with the complete expected type "List Text".
    // Uses a call expression (not a direct variable) to bypass the earlier
    // H0606 type-check stage and reach full-type-check.
    let path = write_cli_fixture(
        "result_multiword_negative",
        "type SplitError {\n  code: Text\n}\n\ntask get_text() -> Text {\n  does:\n    return \"hello\"\n}\n\ntask split_args(text: Text) -> Result List Text, SplitError {\n  does:\n    return get_text()\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(1), "genuine mismatch must exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("H0643"),
        "negative must produce exact H0643 (not H0606): {combined}"
    );
    assert!(
        combined.contains("List Text"),
        "H0643 must name the complete expected type List Text: {combined}"
    );
}

#[test]
fn cli_result_scalar_unchanged() {
    // Scalar Results unchanged: Result UInt, WorkError must exit 0 with
    // zero errors. The projection fix must not affect scalar handling.
    let path = write_cli_fixture(
        "result_scalar",
        "type WorkError {\n  code: Text\n}\n\ntask get_value() -> Result UInt, WorkError {\n  does:\n    return 42\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(out.status.code(), Some(0), "scalar Result must exit 0");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("error["),
        "scalar Result must have zero errors: {combined}"
    );
}

#[test]
fn cli_result_typed_failure_positive_path() {
    // End-to-end typed-failure positive path: the same valid text_split
    // fixture as the multiword positive must exit 0 with zero errors.
    // This proves the positive inline typed-failure path still works; it
    // does NOT prove expected_error_value_type selected the correct error
    // root — that is covered by the focused unit test
    // result_projection_selects_error_root_not_success_token in
    // src/full_type_check.rs, which exercises the production owner directly.
    let path = write_cli_fixture(
        "result_error_root",
        "type SplitError {\n  code: Text\n}\n\ntask split_args(text: Text, sep: Text) -> Result List Text, SplitError {\n  fails when:\n    The separator is empty.\n  does:\n    let pieces = try text_split(text, sep) or fail SplitError.split\n    return pieces\n}\n",
    );
    let out = run_hum_check(&path, false);
    assert_eq!(
        out.status.code(),
        Some(0),
        "typed-failure positive path must exit 0"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("error["),
        "typed-failure positive path must have zero errors: {combined}"
    );
}

fn run_hum(args: &[&str], path: &Path) -> std::process::Output {
    let mut cmd = Command::new(hum_binary());
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(path);
    cmd.output().expect("run hum")
}

// WO30 set-target follow-up (BDFL-authorized 2026-09-27): `set` to an
// exactly-`UInt` place with a statically known negative literal RHS is
// exactly one H0642 per statement, through the production binary.
#[test]
fn cli_h0642_set_negative_literal_via_production_binary() {
    let path = write_cli_fixture(
        "h0642_set",
        "task t() -> UInt {\n  does:\n    change count: UInt = 1\n    set count = -5\n    set count = (-5)\n    return count\n}\n",
    );
    // Human output: exact diagnostics, locations, help, stage summary.
    let out = run_hum(&["check"], &path);
    assert_eq!(out.status.code(), Some(1), "set H0642 must exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("error[H0642]"),
        "human output must contain H0642: {stderr}"
    );
    assert!(
        stderr.matches("error[H0642]").count() == 2,
        "human output must contain exactly 2 H0642: {stderr}"
    );
    assert!(
        stderr.contains("4:5"),
        "human output must locate the plain set at 4:5: {stderr}"
    );
    assert!(
        stderr.contains("5:5"),
        "human output must locate the grouped set at 5:5: {stderr}"
    );
    assert!(
        stderr.contains("Assign a non-negative integer literal"),
        "human output must carry the set-target help text: {stderr}"
    );
    assert!(
        !stderr.contains("H0641"),
        "no H0641 for set targets: {stderr}"
    );
    assert!(
        !stderr.contains("H0643"),
        "no H0643 duplicating the H0642: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("2 error(s)"),
        "human summary must report 2 errors: {combined}"
    );
    // JSON output: code, title, severity, span, stage.
    let out_json = run_hum(&["check", "--format=json"], &path);
    assert_eq!(out_json.status.code(), Some(1));
    let json_stdout = String::from_utf8_lossy(&out_json.stdout);
    assert!(
        json_stdout.contains("\"code\": \"H0642\""),
        "JSON must contain H0642 code: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"title\": \"negative integer literal in UInt position\""),
        "JSON must contain H0642 title: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"severity\": \"error\""),
        "JSON must mark H0642 as error: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"line\": 4"),
        "JSON must contain line 4: {json_stdout}"
    );
    assert!(
        json_stdout.contains("\"line\": 5"),
        "JSON must contain line 5: {json_stdout}"
    );
    assert!(
        json_stdout.matches("\"code\": \"H0642\"").count() == 2,
        "JSON must contain exactly 2 H0642: {json_stdout}"
    );
    assert!(
        json_stdout.contains("full_type_check"),
        "JSON stages must include full_type_check: {json_stdout}"
    );
}

#[test]
fn cli_h0642_set_indexed_and_field_places_via_production_binary() {
    // Indexed (`List UInt` element) and field (record `UInt` field) places
    // are H0642 through the production binary.
    let path = write_cli_fixture(
        "h0642_set_places",
        "type Counter {\n  count: UInt\n}\n\ntask t() -> UInt {\n  does:\n    change xs: List UInt = [1, 2]\n    change counter: Counter = {count: 1}\n    set xs[0] = -5\n    set counter.count = (-5)\n    return xs[0]\n}\n",
    );
    let out = run_hum(&["check"], &path);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        stderr.matches("error[H0642]").count(),
        2,
        "indexed and field places must each be H0642: {stderr}"
    );
    assert!(
        stderr.contains("9:5"),
        "human output must locate the indexed set at 9:5: {stderr}"
    );
    assert!(
        stderr.contains("10:5"),
        "human output must locate the field set at 10:5: {stderr}"
    );
    assert!(
        !stderr.contains("H0643"),
        "no H0643 duplicating the H0642: {stderr}"
    );
}

#[test]
fn cli_h0642_set_immutable_target_earlier_stage_wins() {
    // Earlier-stage precedence: `set` on an immutable (`let`) target is
    // rejected before full-type-check (H0202 at source_check), so no H0642
    // may appear and full_type_check must not run.
    let path = write_cli_fixture(
        "h0642_set_immutable",
        "task t() -> UInt {\n  does:\n    let x: UInt = 3\n    set x = -5\n    return x\n}\n",
    );
    let out = run_hum(&["check"], &path);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("H0202"),
        "immutable set target must keep its earlier-stage diagnostic: {stderr}"
    );
    assert!(
        !stderr.contains("H0642"),
        "H0642 must not fire for immutable targets: {stderr}"
    );
    let out_json = run_hum(&["check", "--format=json"], &path);
    let json_stdout = String::from_utf8_lossy(&out_json.stdout);
    assert!(
        !json_stdout.contains("full_type_check"),
        "precedence: full_type_check must not run after source_check errors: {json_stdout}"
    );
}

#[test]
fn cli_h0642_set_valid_assignment_exits_zero() {
    // Positive evidence: a non-negative literal assigned to a `UInt` place
    // is accepted — exit 0, zero errors.
    let path = write_cli_fixture(
        "h0642_set_valid",
        "task t() -> UInt {\n  does:\n    change count: UInt = 1\n    set count = 5\n    return count\n}\n",
    );
    let out = run_hum(&["check"], &path);
    assert_eq!(
        out.status.code(),
        Some(0),
        "valid set assignment must exit 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("error["),
        "valid set assignment must have zero errors: {combined}"
    );
}

#[test]
fn cli_h0642_set_run_preflight_rejects_before_effects() {
    // `hum run` preflight must reject the negative-literal set with H0642
    // before any side effect. The sentinel is printed with the supported
    // literal-output form (`let written = try stdout_write("...")`) BEFORE
    // the `set`, under explicit `--allow stdout.write` consent. The paired
    // control proves the execution path is reachable: it exits 0 and emits
    // the sentinel.
    let app_source = |set_rhs: &str| {
        format!(
            "module probe\n\napp probe {{\n  uses:\n    stdout.write\n\n  starts with:\n    run_tool\n\n  task run_tool -> Result Unit, OutputError {{\n    uses:\n      stdout.write\n\n    fails when:\n      output is denied\n\n    allocates:\n      one bounded text buffer\n\n    does:\n      let written = try stdout_write(\"H0642_SENTINEL_OK\")\n      change count: UInt = 42\n      set count = {set_rhs}\n      return written\n  }}\n}}\n"
        )
    };

    // Control: non-negative assignment. The path is reachable: exit 0 and
    // the sentinel is emitted.
    let control_path = write_cli_fixture("h0642_set_run_control", &app_source("7"));
    let control_out = run_hum(&["run", "--allow", "stdout.write"], &control_path);
    assert_eq!(
        control_out.status.code(),
        Some(0),
        "control must exit 0 (execution path reachable)"
    );
    let control_stdout = String::from_utf8_lossy(&control_out.stdout);
    assert!(
        control_stdout
            .lines()
            .any(|line| line.trim() == "H0642_SENTINEL_OK"),
        "control must emit the sentinel: {control_stdout}"
    );

    // Negative case: the H0642 preflight rejection must fire before any side
    // effect, so the sentinel is never emitted.
    let path = write_cli_fixture("h0642_set_run", &app_source("-5"));
    let out = run_hum(&["run", "--allow", "stdout.write"], &path);
    assert_eq!(out.status.code(), Some(1), "run preflight must exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("H0642"),
        "run preflight must reject with H0642: {combined}"
    );
    assert!(
        stderr.contains("unsupported_statements=0"),
        "no unsupported statement may supply the rejection: {combined}"
    );
    assert!(
        !stdout
            .lines()
            .any(|line| line.trim() == "H0642_SENTINEL_OK"),
        "run preflight must not execute the body (sentinel absent): {combined}"
    );
}

#[test]
fn cli_h0644_warns_only_for_unsupported_else_and_preserves_run_preflight() {
    let prefix = r#"type FileAppError {
  code: Text
}

app else_probe {
  uses:
    files.read

  starts with:
    run_tool

  task run_tool(input: Path) -> Result Unit, FileAppError {
    uses:
      files.read
    fails when:
      reading the input fails
    allocates:
      one bounded file buffer
    does:
      let input_text = try files_read_text(input) or fail FileAppError.read
      change value: UInt = 0
      if input_text == "enabled" {
        set value = 1
"#;
    let suffix = "        set value = 2\n      }\n      return\n  }\n}\n";
    let branch_line = prefix.lines().count() + 1;
    let original = write_cli_fixture("h0644_else", &format!("{prefix}      }} else {{\n{suffix}"));
    let rewrite = write_cli_fixture(
        "h0644_rewrite",
        &format!(
            "{prefix}      }}\n      # }} else {{\n      let marker: Text = \"}} else {{\"\n      if input_text != \"enabled\" {{\n{suffix}"
        ),
    );
    let unrelated = write_cli_fixture(
        "h0644_unrelated",
        &format!("{prefix}      }} otherwise {{\n{suffix}"),
    );

    let message = "this `else` form is unsupported by `hum run`";
    let plain = run_hum_check(&original, false);
    assert_eq!(plain.status.code(), Some(0));
    let plain_stderr = String::from_utf8_lossy(&plain.stderr);
    let mut warnings = plain_stderr.lines().filter(|line| line.contains("H0644"));
    let warning = warnings.next().expect("one warning on stderr");
    assert!(
        warning.contains(&format!(":{branch_line}:7: warning[H0644]: {message}")),
        "{warning}"
    );
    assert_eq!(warnings.next(), None);

    let json = run_hum_check(&original, true);
    assert_eq!(json.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&json.stdout);
    let code = "\"code\": \"H0644\"";
    assert_eq!(stdout.matches(code).count(), 1, "{stdout}");
    let (_, fields) = stdout.split_once(code).expect("warning object");
    let (fields, _) = fields.split_once("\n    }").expect("end of warning object");
    assert!(fields.contains("\"severity\": \"warning\""), "{fields}");
    assert!(fields.contains(message), "{fields}");
    assert!(
        fields.contains(&format!("\"line\": {branch_line}, \"column\": 7")),
        "{fields}"
    );
    assert!(stdout.contains("\"errors\": 0"), "{stdout}");

    for path in [&rewrite, &unrelated] {
        for json in [false, true] {
            let output = run_hum_check(path, json);
            assert_eq!(output.status.code(), Some(0), "{output:?}");
            assert!(!String::from_utf8_lossy(&output.stderr).contains("H0644"));
            assert!(!String::from_utf8_lossy(&output.stdout).contains("H0644"));
        }
    }

    let input = original.with_extension("txt");
    std::fs::write(&input, "enabled").expect("write exact file fixture");
    let grant = format!("files.read={}", input.display());
    let run = Command::new(hum_binary())
        .arg("run")
        .arg(&original)
        .arg(format!("--allow={grant}"))
        .args(["--trust-locality", &grant, "--args"])
        .arg(&input)
        .output()
        .expect("run hum");
    assert_eq!(run.status.code(), Some(1));
    assert!(run.stdout.is_empty(), "{run:?}");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(
        stderr.contains(&format!(
            ":{branch_line}:7 [blocked_unsupported_statement_v0] unknown_body_line"
        )),
        "{stderr}"
    );
    assert!(stderr.contains("not_in_core_body_grammar_v0"), "{stderr}");
    assert!(!stderr.contains("failure: FileAppError.read"), "{stderr}");
    assert!(!stderr.contains("trusted-not-proven"), "{stderr}");
    assert!(!stderr.contains("H0644"), "{stderr}");
}

const H0907_TYPE_STAGES: &str = r#""parse", "source_check", "app_entry", "path_boundary", "callable", "capability_root", "resolve", "type_check""#;
const H0907_FULL_STAGES: &str = r#""parse", "source_check", "app_entry", "path_boundary", "callable", "capability_root", "resolve", "type_check", "full_type_check""#;

const H0907_APP: &str = r#"module default_failure_declaration

type ProbeError {
  code: Text
}

app failure_probe {
  why:
    Exercise typed failure through the default entry.
  starts with:
    relay

  task source() -> Result Unit, ProbeError {
    why:
      Produce the named failure.
    fails when:
      This probe always reaches ProbeError.expected.
    does:
      fail ProbeError.expected
  }

  task relay() -> Result Unit, ProbeError {
    why:
      Propagate the source task's typed failure.
    does:
      let value = try source()
      return value
  }
}
"#;

fn h0907_declared_app() -> String {
    H0907_APP.replace(
        "    does:\n      let value = try source()",
        "    fails when:\n      The source task returns ProbeError.expected.\n    does:\n      let value = try source()",
    )
}

fn check_json_error_rows(report: &str) -> impl Iterator<Item = &str> {
    // The check renderer places top-level diagnostic objects on separate lines.
    let (_, diagnostics) = report
        .split_once("\n  \"diagnostics\": [")
        .expect("check JSON diagnostic array");
    diagnostics.split("\n    {").skip(1).filter_map(|row| {
        let (row, _) = row
            .split_once("\n    }")
            .expect("check JSON diagnostic row");
        row.lines()
            .any(|line| line.trim() == r#""severity": "error","#)
            .then_some(row)
    })
}

fn assert_h0907_check_pair(path: &Path, codes: &[&str], stages: &str) -> (String, String) {
    let plain = run_hum_check(path, false);
    let json = run_hum_check(path, true);
    let exit = Some(i32::from(!codes.is_empty()));
    let stderr = String::from_utf8(plain.stderr).expect("UTF-8 check diagnostics");
    let report = String::from_utf8(json.stdout).expect("UTF-8 check JSON");
    assert_eq!(plain.status.code(), exit, "{path:?}: {stderr}");
    assert_eq!(json.status.code(), exit, "{path:?}: {report}");
    assert!(json.stderr.is_empty(), "{path:?}: {:?}", json.stderr);

    let plain_codes = stderr
        .lines()
        .filter_map(|line| {
            let (_, diagnostic) = line.split_once(": error[")?;
            diagnostic.split_once("]: ").map(|(code, _)| code)
        })
        .collect::<Vec<_>>();
    assert_eq!(plain_codes, codes, "{path:?}: {stderr}");

    let warnings = stderr
        .lines()
        .filter(|line| line.contains(": warning["))
        .count();
    let errors = codes.len();
    assert_eq!(
        String::from_utf8(plain.stdout).unwrap(),
        format!("checked 1 file(s): {errors} error(s), {warnings} warning(s)\n"),
        "{path:?}"
    );
    assert!(report.contains(r#""schema": "hum.check.v0""#), "{report}");
    assert!(
        report
            .lines()
            .any(|line| line.trim() == format!("\"stages\": [{stages}],")),
        "{report}"
    );
    assert!(
        report.contains(&format!(
            r#""summary": {{"files": 1, "errors": {errors}, "warnings": {warnings}}}"#
        )),
        "{report}"
    );
    let rows = check_json_error_rows(&report).collect::<Vec<_>>();
    assert_eq!(rows.len(), errors, "{report}");
    for (row, code) in rows.into_iter().zip(codes) {
        assert!(
            row.lines()
                .any(|line| line.trim() == format!("\"code\": {code:?},")),
            "{row}"
        );
    }
    (stderr, report)
}

#[test]
fn cli_h0907_reports_each_reachable_failure_in_plain_and_json() {
    // The two-site case reaches relay from a distinct default entry.
    let twice = H0907_APP
        .replace("  starts with:\n    relay", "  starts with:\n    start")
        .replace(
            "      return value",
            "      let second = try source()\n      return second",
        );
    let twice = format!(
        r#"{}  task start() -> Result Unit, ProbeError {{
    fails when:
      The relay task returns ProbeError.expected.
    does:
      let result = try relay()
      return result
  }}
}}
"#,
        twice.strip_suffix("}\n").unwrap()
    );
    for (name, source, sites) in [
        ("h0907_one", H0907_APP, [(26, 23)].as_slice()),
        ("h0907_two", twice.as_str(), [(26, 23), (27, 24)].as_slice()),
    ] {
        let path = write_cli_fixture(name, source);
        let codes = vec!["H0907"; sites.len()];
        let (plain, json) = assert_h0907_check_pair(&path, &codes, H0907_FULL_STAGES);

        let run = run_hum(&["run"], &path);
        let stderr = String::from_utf8(run.stderr).expect("UTF-8 preflight diagnostics");
        assert_eq!(run.status.code(), Some(2), "{stderr}");
        assert!(run.stdout.is_empty(), "{stderr}");

        let run_lines = stderr.lines().collect::<Vec<_>>();
        assert_eq!(run_lines.len(), sites.len() * 2, "{stderr}");
        let check_lines = plain.lines().collect::<Vec<_>>();
        let check_blocks = check_lines
            .windows(2)
            .filter(|pair| pair[0].contains(": error[H0907]: "))
            .map(|pair| [pair[0], pair[1]])
            .collect::<Vec<_>>();
        let (preflight_blocks, remainder) = run_lines.as_chunks::<2>();
        assert!(remainder.is_empty(), "{stderr}");
        assert_eq!(check_blocks.as_slice(), preflight_blocks);

        let file = path.to_string_lossy().replace('\\', "/");
        for ((&(line, column), block), row) in sites
            .iter()
            .zip(preflight_blocks)
            .zip(check_json_error_rows(&json))
        {
            let location = format!("{file}:{line}:{column}: error[H0907]: ");
            let message = block[0].strip_prefix(&location).expect("exact H0907 span");
            let help = block[1].strip_prefix("  help: ").expect("H0907 help");
            assert!(
                message
                    .starts_with("typed failure requires a meaningful `fails when:` declaration"),
                "{message}"
            );
            assert!(
                help.ends_with("Add a concrete failure condition naming this path."),
                "{help}"
            );
            assert!(
                row.lines()
                    .any(|entry| entry.trim() == format!("\"message\": {message:?},")),
                "{row}"
            );
            assert!(
                row.lines().any(|entry| entry.trim()
                    == format!(
                        "\"span\": {{\"file\": {file:?}, \"line\": {line}, \"column\": {column}}},"
                    )),
                "{row}"
            );
            assert!(
                row.lines()
                    .any(|entry| entry.trim() == format!("\"help\": {help:?}")),
                "{row}"
            );
        }
    }
}

#[test]
fn cli_h0907_respects_declared_unreachable_and_missing_defaults() {
    let declared = write_cli_fixture("h0907_declared", &h0907_declared_app());
    assert_h0907_check_pair(&declared, &[], H0907_FULL_STAGES);
    let run = run_hum(&["run"], &declared);
    assert_eq!(run.status.code(), Some(1), "{run:?}");
    assert!(run.stdout.is_empty(), "{run:?}");
    assert!(
        String::from_utf8_lossy(&run.stderr).starts_with("failure: ProbeError.expected\n"),
        "{run:?}"
    );

    let unreachable = write_cli_fixture(
        "h0907_unreachable",
        &H0907_APP.replace(
            "  starts with:\n    relay",
            "  starts with:\n    start\n\n  task start() -> Unit {\n    does:\n      return\n  }",
        ),
    );
    assert_h0907_check_pair(&unreachable, &[], H0907_FULL_STAGES);
    let run = run_hum(&["run"], &unreachable);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    assert!(run.stdout.is_empty() && run.stderr.is_empty(), "{run:?}");
    let explicit = run_hum(&["run", "--entry", "relay"], &unreachable);
    assert_eq!(explicit.status.code(), Some(2), "{explicit:?}");
    assert_eq!(
        String::from_utf8_lossy(&explicit.stderr)
            .matches("error[H0907]")
            .count(),
        1
    );

    let no_default = write_cli_fixture(
        "h0907_no_default",
        r#"type ProbeError {
  code: Text
}

task first() -> UInt {
  does:
    return 1
}

task broken() -> Result UInt, ProbeError {
  does:
    fail ProbeError.expected
}
"#,
    );
    assert_h0907_check_pair(&no_default, &[], H0907_FULL_STAGES);
    let run = run_hum(&["run"], &no_default);
    assert_eq!(run.status.code(), Some(2), "{run:?}");
    assert!(run.stdout.is_empty(), "{run:?}");
    assert!(
        String::from_utf8_lossy(&run.stderr).contains("multiple tasks are available"),
        "{run:?}"
    );
    let explicit = run_hum(&["run", "--entry", "broken"], &no_default);
    assert_eq!(explicit.status.code(), Some(2), "{explicit:?}");
    assert_eq!(
        String::from_utf8_lossy(&explicit.stderr)
            .matches("error[H0907]")
            .count(),
        1
    );
}

#[test]
fn cli_h0907_preserves_the_bounded_literal_stdout_exemption() {
    let text = include_str!("../programs/hello_world.hum")
        .replace("\r\n", "\n")
        .replace(
            "    fails when:\n      bounded output is denied or the output adapter rejects the write\n\n",
            "",
        )
        .replace("\"Hello, world!\"", "\"Hello world!\"");
    assert!(!text.contains("fails when:"));
    assert!(text.contains("try stdout_write(\"Hello world!\")"));
    let path = write_cli_fixture("h0907_bounded_stdout", &text);

    assert_h0907_check_pair(&path, &[], H0907_FULL_STAGES);
    let run = run_hum(&["run", "--allow", "stdout.write"], &path);
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    assert_eq!(run.stdout, b"Hello world!");
    assert!(run.stderr.is_empty(), "{run:?}");
}

#[test]
fn cli_h0907_preserves_type_and_full_type_error_precedence() {
    for (name, body, code, stages) in [
        (
            "h0907_type_blocker",
            "      return \"not a UInt\"\n",
            "H0606",
            H0907_TYPE_STAGES,
        ),
        (
            "h0907_full_type_blocker",
            "      change count: UInt = 1\n      set count = -5\n      return count\n",
            "H0642",
            H0907_FULL_STAGES,
        ),
    ] {
        let app = H0907_APP.strip_suffix("}\n").unwrap();
        let text = format!("{app}\n  task earlier_error() -> UInt {{\n    does:\n{body}  }}\n}}\n");
        let path = write_cli_fixture(name, &text);
        assert_h0907_check_pair(&path, &[code], stages);
    }
}

#[test]
fn cli_h0907_keeps_other_effect_diagnostics_out_of_check() {
    let text = h0907_declared_app().replace(
        "    does:\n      let value = try source()",
        "    avoids:\n      failure\n    does:\n      let value = try source()",
    );
    let path = write_cli_fixture("h0907_effect_only", &text);
    assert_h0907_check_pair(&path, &[], H0907_FULL_STAGES);

    let effect = run_hum(&["effect-check", "--format=json"], &path);
    assert_eq!(effect.status.code(), Some(1), "{effect:?}");
    let json = String::from_utf8(effect.stdout).expect("UTF-8 effect JSON");
    assert!(json.contains("rejected_avoids_contradicted_v0"), "{json}");
    assert!(
        json.contains("avoids_failure_but_body_can_fail_v0"),
        "{json}"
    );
    assert!(!json.contains("\"diagnostic_code\": \"H0907\""), "{json}");
}
