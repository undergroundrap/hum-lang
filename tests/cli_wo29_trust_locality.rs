// Production-connected CLI tests for WO29 Slice A trust-locality
// (decision 0029 Option D: per-invocation, per-path operator attestation,
// grant-first amendment per decision 0029 section 14 / PR #58).
//
// These spawn the actual `hum` binary to verify the grant-first admission
// gate, the `--trust-locality files.read=<path>` attestation, the refusal
// shapes, and the JSON run envelope. Adapter-only tests are insufficient
// per review.
//
// Grant-first: no classifier in this WO version emits `proved`, so the
// consent-without-attestation probe ALWAYS refuses fail-closed on every
// platform and every storage: exit 1, empty stdout, the typed
// `<App>Error.read` caused-by `FileReadError.unavailable` chain, no panic,
// no proved mention. The JSON envelope preserves the gate reason verbatim
// in the files.read exercise event's `result` (the platform-selected
// constant from `NativePath::locality_gate_reason`) and the classifier's
// exact `Unproven` reason in `classifier_reason`; the attested run must
// carry that same `classifier_reason` verbatim (run-to-run preservation,
// never hardcoded: the classifier reason is storage/host-specific).
//
// Honest coverage gaps (BDFL test-plan correction, 2026-09-27): the
// B2/C1/C2 adversarial identity cases (walked/opened and proof-evidence
// contradictions) have no deterministic CLI seam -- production emits
// `walked_opened_identity_mismatch_before_read_v0` and
// `proof_evidence_identity_mismatch_before_read_v0` only on real races
// between the component walk and the handle open, which the synchronous
// CLI surface cannot induce. They are NOT faked here. D5's
// `build_run_json_envelope` is infallible by construction (returns `String`
// directly, no `Result`); the hand-rolled construction has no failure path,
// so no CLI input reaches an envelope-construction failure -- and the
// `json_envelope_*` tests below pin the output shapes it must preserve
// (empty output, trailing newline, partial output on failure, authoritative
// exit code) through the real `hum run --format json` command.

use std::path::{Path, PathBuf};
use std::process::Command;

// Minimal probe program: reads the file named by its CLI arg and emits each
// non-empty line followed by a newline. For fixtures/wordfreq/sample.txt
// ("hum  lang\n\nhum\n" -- two spaces between the first two words, one empty
// line) the byte-exact stdout is "hum  lang\nhum\n" (14 bytes). The program
// text is written inline to a temp dir (no new repo fixture file), following the write_cli_fixture pattern in
// cli_decision0030.rs. The shape mirrors the proven
// examples/probes/exact_file_read.hum probe: app entry with `starts with:`,
// `try files_read_text(input) or fail`, and the typed-parameter hand-off for
// the for-each loop variable.
const PROBE_PROGRAM: &str = r#"type ProbeError {
  code: Text
}

app trust_locality_probe {
  why:
    minimal trust-locality probe: read the granted file and emit each
    non-empty line followed by a newline

  uses:
    files.read
    stdout.write

  starts with:
    run_probe

  task run_probe(input: Path) -> Result Unit, ProbeError {
    why:
      read the file named by the CLI arg and emit its non-empty lines

    uses:
      files.read
      stdout.write

    fails when:
      the file cannot be read or a line cannot be written

    allocates:
      one bounded file buffer

    does:
      let text = try files_read_text(input) or fail ProbeError.read
      let lines = text_split(text, "\n")
      for each line in lines {
        if line != "" {
          let wrote = try probe_write_line(line) or fail ProbeError.output
        }
      }
      let done = try stdout_write("") or fail ProbeError.output
      return done
  }

  task probe_write_line(line: Text) -> Result Unit, ProbeError {
    why:
      write one line followed by a newline; the typed parameter carries
      the Text proof the for-each loop variable cannot supply

    uses:
      stdout.write

    fails when:
      the line or its newline cannot be written

    does:
      let wrote_text = try stdout_write(line) or fail ProbeError.output
      let wrote_newline = try stdout_write("\n") or fail ProbeError.output
      return wrote_newline
  }
}
"#;

// Quiet program: no file reads at all, only stdout. Used by the E2
// envelope-only test to prove the new file-read evidence line is absent on
// an otherwise successful run (asserting the specific line's absence, not
// globally empty stderr).
const QUIET_PROGRAM: &str = r#"type QuietError {
  code: Text
}

app quiet_probe {
  why:
    minimal quiet probe: emit one line without reading any file

  uses:
    stdout.write

  starts with:
    run_quiet

  task run_quiet(input: Path) -> Result Unit, QuietError {
    why:
      emit one line; the CLI arg is accepted and ignored

    uses:
      stdout.write

    fails when:
      the line cannot be written

    does:
      let wrote = try stdout_write("quiet") or fail QuietError.output
      return wrote
  }
}
"#;

// Byte-exact expected stdout of the probe on fixtures/wordfreq/sample.txt.
// The app entry completes with AppSuccess, so the CLI prints nothing beyond
// the program's own writes.
const EXPECTED_STDOUT: &[u8] = b"hum  lang\nhum\n";

// D5 output-shape probes: small programs exercising the JSON run envelope's
// `program_output_bytes` / `exit_code` contract through the real
// `hum run --format json` command. The envelope constructor itself is
// infallible (returns `String` directly, no `Result`); these pin the shapes
// it must preserve on the wire.
const EMPTY_OUTPUT_PROGRAM: &str = r#"type EmptyError {
  code: Text
}

app empty_output_probe {
  why:
    succeed without writing any bytes; the envelope must carry an empty
    program_output_bytes array

  uses:
    stdout.write

  starts with:
    run_empty

  task run_empty(input: Path) -> Result Unit, EmptyError {
    why:
      the CLI arg is accepted and ignored; nothing is written

    uses:
      stdout.write

    fails when:
      never; the empty path always succeeds

    does:
      return
  }
}
"#;

const TRAILING_NEWLINE_PROGRAM: &str = r#"type NewlineError {
  code: Text
}

app trailing_newline_probe {
  why:
    write bytes ending in a newline; the envelope must preserve the
    trailing newline byte exactly

  uses:
    stdout.write

  starts with:
    run_newline

  task run_newline(input: Path) -> Result Unit, NewlineError {
    why:
      emit one line with its terminator; the CLI arg is ignored

    uses:
      stdout.write

    fails when:
      the write cannot complete

    does:
      let wrote = try stdout_write("abc\n") or fail NewlineError.output
      return wrote
  }
}
"#;

const PARTIAL_FAILURE_PROGRAM: &str = r#"type PartialError {
  code: Text
}

app partial_failure_probe {
  why:
    write partial bytes then fail outright; the envelope must carry the
    partial program_output_bytes with the authoritative nonzero exit code

  uses:
    stdout.write

  starts with:
    run_partial

  task run_partial(input: Path) -> Result Unit, PartialError {
    why:
      emit bytes, then take the unconditional failure path

    uses:
      stdout.write

    fails when:
      always, after the partial write

    does:
      let wrote = try stdout_write("partial-bytes") or fail PartialError.output
      if true {
        fail PartialError.boom
      }
      return wrote
  }
}
"#;

const EMPTY_FAILURE_PROGRAM: &str = r#"type EmptyFailureError {
  code: Text
}

app empty_failure_probe {
  why:
    fail without writing anything; the envelope exit_code must equal the
    process exit code

  starts with:
    run_empty_failure

  task run_empty_failure(input: Path) -> Result Unit, EmptyFailureError {
    why:
      take the failure path immediately

    fails when:
      the failure probe is exercised

    does:
      if true {
        fail EmptyFailureError.boom
      }
      return
  }
}
"#;

/// The gate reason recorded in the files.read exercise event's `result` on
/// the no-grant refusal. Platform-selected by
/// `NativePath::locality_gate_reason` (deterministic per platform arm, never
/// the retired pre-demotion reason). This names the gate, not the
/// classifier; the classifier's own reason travels in `classifier_reason`
/// and is compared run-to-run, never hardcoded.
#[cfg(unix)]
const GATE_REASON: &str = "p1_locality_unproven_on_this_platform_v0";
#[cfg(not(unix))]
const GATE_REASON: &str = "windows_locality_unproven_grant_first_v0";

fn hum_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hum"))
}

fn write_program(name: &str, text: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "hum_wo29_trust_locality_{name}_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(format!("{name}.hum"));
    std::fs::write(&path, text).expect("write probe program");
    (dir, path)
}

fn fixture_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/wordfreq/sample.txt")
        .to_str()
        .expect("fixture path is UTF-8")
        .to_owned()
}

fn run_hum_run(args: &[String], program: &Path, env: Option<(&str, &str)>) -> std::process::Output {
    let mut cmd = Command::new(hum_binary());
    cmd.arg("run");
    cmd.arg(program);
    for arg in args {
        cmd.arg(arg);
    }
    if let Some((key, value)) = env {
        cmd.env(key, value);
    }
    cmd.output().expect("run hum")
}

/// Consent + matching files.read grant, no attestation: grant-first, this
/// ALWAYS refuses (exit 1) because no classifier emits `proved` and no
/// attestation is present.
fn probe_no_trust(program: &Path, fixture: &str, json: bool) -> std::process::Output {
    let mut args = vec![
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        format!("--allow=files.read={fixture}"),
    ];
    if json {
        args.push("--format".to_owned());
        args.push("json".to_owned());
    }
    args.push("--args".to_owned());
    args.push(fixture.to_owned());
    run_hum_run(&args, program, None)
}

/// The attested invocation: `--trust-locality files.read=<path>` precedes
/// `--args` (after `--args` it would be a program arg).
fn run_with_trust(program: &Path, fixture: &str, json: bool) -> std::process::Output {
    let mut args = vec![
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        format!("--allow=files.read={fixture}"),
    ];
    if json {
        args.push("--format".to_owned());
        args.push("json".to_owned());
    }
    args.push("--trust-locality".to_owned());
    args.push(format!("files.read={fixture}"));
    args.push("--args".to_owned());
    args.push(fixture.to_owned());
    run_hum_run(&args, program, None)
}

/// Decode a JSON string body (the text after the opening quote), returning
/// the decoded value and the byte index of the closing quote within
/// `quoted`.
fn decode_json_string_body(quoted: &str, context: &str) -> (String, usize) {
    let mut decoded = String::new();
    let mut chars = quoted.char_indices();
    let mut end = None;
    while let Some((idx, c)) = chars.next() {
        match c {
            '"' => {
                end = Some(idx);
                break;
            }
            '\\' => match chars.next() {
                Some((_, 'n')) => decoded.push('\n'),
                Some((_, 't')) => decoded.push('\t'),
                Some((_, '"')) => decoded.push('"'),
                Some((_, '\\')) => decoded.push('\\'),
                Some((_, 'u')) => {
                    let hex: String = chars.by_ref().take(4).map(|(_, h)| h).collect();
                    let code = u32::from_str_radix(&hex, 16)
                        .unwrap_or_else(|_| panic!("bad \\u escape in {context}"));
                    decoded.push(
                        char::from_u32(code)
                            .unwrap_or_else(|| panic!("bad \\u escape in {context}")),
                    );
                }
                Some((_, e)) => {
                    decoded.push('\\');
                    decoded.push(e);
                }
                None => break,
            },
            _ => decoded.push(c),
        }
    }
    let end = end.unwrap_or_else(|| panic!("string value must terminate in {context}"));
    (decoded, end)
}

/// Extract the `program_output_bytes` number array from the hand-rolled run
/// envelope. No serde: the Slice A contract forbids new dependencies, so the
/// test parses the array directly (whitespace-insensitive, byte-exact
/// values).
fn envelope_program_output_bytes(envelope: &str) -> Vec<u8> {
    let key = "\"program_output_bytes\"";
    let pos = envelope
        .find(key)
        .unwrap_or_else(|| panic!("envelope must contain program_output_bytes: {envelope}"));
    let after_key = &envelope[pos + key.len()..];
    let colon = after_key
        .find(':')
        .expect("program_output_bytes must be followed by ':'");
    let array = after_key[colon + 1..].trim_start();
    assert!(
        array.starts_with('['),
        "program_output_bytes must be a JSON array: {envelope}"
    );
    let end = array
        .find(']')
        .expect("program_output_bytes array must close");
    array[1..end]
        .split(',')
        .filter(|entry| !entry.trim().is_empty())
        .map(|entry| {
            entry.trim().parse::<u8>().unwrap_or_else(|_| {
                panic!("program_output_bytes entries must be u8, got {entry:?}: {envelope}")
            })
        })
        .collect()
}

/// The envelope prefix before `program_output_bytes`: carries the top-level
/// `outcome` and `exit_code` without interference from nested authority
/// events (which may repeat field names with their own values).
fn envelope_head(envelope: &str) -> &str {
    let pos = envelope
        .find("\"program_output_bytes\"")
        .unwrap_or_else(|| panic!("envelope must contain program_output_bytes: {envelope}"));
    &envelope[..pos]
}

/// The top-level `outcome` string of the run envelope (e.g. `app_success`,
/// `app_failure`), read from the envelope head only.
fn envelope_head_outcome(envelope: &str) -> String {
    let head = envelope_head(envelope);
    let key = "\"outcome\":\"";
    let pos = head
        .find(key)
        .unwrap_or_else(|| panic!("envelope head must contain outcome: {envelope}"));
    let (decoded, _) = decode_json_string_body(&head[pos + key.len()..], "outcome");
    decoded
}

/// The top-level `exit_code` number of the run envelope, read from the
/// envelope head only.
fn envelope_head_exit_code(envelope: &str) -> u64 {
    let head = envelope_head(envelope);
    let key = "\"exit_code\":";
    let pos = head
        .find(key)
        .unwrap_or_else(|| panic!("envelope head must contain exit_code: {envelope}"));
    let rest = head[pos + key.len()..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end]
        .parse()
        .unwrap_or_else(|_| panic!("exit_code must be numeric: {envelope}"))
}

/// Collect every non-null string value of a named JSON field in the
/// envelope, tolerating the hand-rolled emitter's whitespace. Only the
/// classified files.read exercise carries a non-null
/// locality_classification, so the result is exactly one entry for this
/// probe.
fn envelope_string_field_values(envelope: &str, field: &str) -> Vec<String> {
    let key = format!("\"{field}\"");
    let mut values = Vec::new();
    let mut rest = envelope;
    while let Some(pos) = rest.find(&key) {
        let after_key = &rest[pos + key.len()..];
        let colon = after_key
            .find(':')
            .unwrap_or_else(|| panic!("field {field} must be followed by ':' in: {envelope}"));
        let value_start = after_key[colon + 1..].trim_start();
        if let Some(quoted) = value_start.strip_prefix('"') {
            let (decoded, end) = decode_json_string_body(quoted, field);
            values.push(decoded);
            rest = &quoted[end + 1..];
        } else {
            // null or a non-string value: advance past this key occurrence.
            rest = value_start;
        }
    }
    values
}

/// Split the envelope's `authority_events` array into per-event object
/// slices. The hand-rolled emitter writes compact JSON; the scanner tracks
/// brace depth while skipping string literals (with escape handling), so
/// nested objects and arrays inside events are handled.
fn authority_event_slices(envelope: &str) -> Vec<&str> {
    let key = "\"authority_events\"";
    let pos = envelope
        .find(key)
        .unwrap_or_else(|| panic!("envelope must contain authority_events: {envelope}"));
    let array_open = pos
        + envelope[pos..]
            .find('[')
            .expect("authority_events must be a JSON array");
    let bytes = envelope.as_bytes();
    let mut events = Vec::new();
    let mut i = array_open + 1;
    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut event_start: Option<usize> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else if b == b'"' {
            in_string = true;
        } else if b == b'{' {
            if depth == 0 {
                event_start = Some(i);
            }
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0
                && let Some(start) = event_start.take()
            {
                events.push(&envelope[start..=i]);
            }
        } else if b == b']' && depth == 0 {
            break;
        }
        i += 1;
    }
    events
}

/// Extract every string-array value of a named JSON field in the envelope
/// (one entry per non-null occurrence), decoding each element.
fn envelope_string_array_field_values(envelope: &str, field: &str) -> Vec<Vec<String>> {
    let key = format!("\"{field}\"");
    let mut all = Vec::new();
    let mut rest = envelope;
    while let Some(pos) = rest.find(&key) {
        let after_key = &rest[pos + key.len()..];
        let colon = after_key
            .find(':')
            .unwrap_or_else(|| panic!("field {field} must be followed by ':'"));
        let value_start = after_key[colon + 1..].trim_start();
        if let Some(array) = value_start.strip_prefix('[') {
            let mut elements = Vec::new();
            let mut elem_rest = array;
            loop {
                let elem_trimmed = elem_rest.trim_start();
                if let Some(after_bracket) = elem_trimmed.strip_prefix(']') {
                    elem_rest = after_bracket;
                    break;
                }
                let quoted = elem_trimmed.strip_prefix('"').unwrap_or_else(|| {
                    panic!("{field} array elements must be strings: {envelope}")
                });
                let (decoded, end) = decode_json_string_body(quoted, field);
                elements.push(decoded);
                elem_rest = quoted[end + 1..].trim_start();
                if let Some(after_comma) = elem_rest.strip_prefix(',') {
                    elem_rest = after_comma;
                }
            }
            all.push(elements);
            rest = elem_rest;
        } else {
            // null or a non-array value: advance past this key occurrence.
            rest = value_start;
        }
    }
    all
}

/// The files.read `operation_exercise` event slice. Panics when the envelope
/// carries none (every file-reading run emits exactly one).
fn files_read_exercise_event(envelope: &str) -> &str {
    authority_event_slices(envelope)
        .into_iter()
        .find(|event| {
            event.contains("\"capability_id\":\"files.read\"")
                && event.contains("\"event_kind\":\"operation_exercise\"")
        })
        .unwrap_or_else(|| {
            panic!("envelope must contain a files.read operation_exercise event: {envelope}")
        })
}

/// Read a string field from one event slice; panics when the field is absent
/// or null.
fn event_string_field(event: &str, field: &str) -> String {
    let key = format!("\"{field}\":\"");
    let pos = event
        .find(&key)
        .unwrap_or_else(|| panic!("event must carry string field {field}: {event}"));
    let (decoded, _) = decode_json_string_body(&event[pos + key.len()..], field);
    decoded
}

#[test]
fn cli_trust_locality_no_trust_refusal_and_attested_admission() {
    // Grant-first admission. The no-trust probe ALWAYS refuses: no classifier
    // emits `proved`, so consent + matching grant without attestation fails
    // closed with FileReadError.unavailable on every platform and storage.
    // The attested run then admits with the external-trust label.
    let (_probe_dir, program) = write_program("trust_locality_probe", PROBE_PROGRAM);
    let fixture = fixture_path();

    // Human no-trust probe: exit 1, no stdout, typed refusal, no panic, no
    // proved mention.
    let probe = probe_no_trust(&program, &fixture, false);
    let probe_stderr = String::from_utf8_lossy(&probe.stderr);
    assert_eq!(
        probe.status.code(),
        Some(1),
        "no-trust probe must exit 1 (grant-first refusal): {probe_stderr}"
    );
    assert!(probe.stdout.is_empty(), "refused read must write no stdout");
    assert!(
        probe_stderr.contains("ProbeError.read")
            && probe_stderr.contains("FileReadError.unavailable"),
        "no-trust probe must fail closed with ProbeError.read caused by FileReadError.unavailable: {probe_stderr}"
    );
    assert!(
        !probe_stderr.contains("panicked"),
        "probe must fail closed without panicking: {probe_stderr}"
    );
    assert!(
        !probe_stderr.contains("proved"),
        "no-trust refusal must never mention proved: {probe_stderr}"
    );

    // JSON no-trust probe (BDFL A2/A5 + B3): the files.read exercise event
    // preserves the platform gate reason verbatim in `result`, records the
    // classifier's exact Unproven reason in `classifier_reason`, admits
    // nothing (null classification), opens no file and reads no payload.
    let probe_json = probe_no_trust(&program, &fixture, true);
    let probe_json_stderr = String::from_utf8_lossy(&probe_json.stderr);
    assert_eq!(
        probe_json.status.code(),
        Some(1),
        "JSON no-trust probe must exit 1: {probe_json_stderr}"
    );
    let refusal_envelope = String::from_utf8_lossy(&probe_json.stdout);
    assert!(
        refusal_envelope.trim_start().starts_with('{'),
        "JSON mode stdout must carry only the envelope: {refusal_envelope}"
    );
    let refusal_event = files_read_exercise_event(&refusal_envelope);
    assert_eq!(
        event_string_field(refusal_event, "result"),
        GATE_REASON,
        "refusal event result must preserve the platform gate reason verbatim"
    );
    let refusal_classifier_reason = event_string_field(refusal_event, "classifier_reason");
    assert!(
        !refusal_classifier_reason.is_empty(),
        "refusal event must record the classifier Unproven reason verbatim in classifier_reason"
    );
    assert!(
        refusal_event.contains("\"locality_classification\":null"),
        "refusal must admit nothing: locality_classification must be null: {refusal_event}"
    );
    assert!(
        refusal_event.contains("\"adapter_called\":false"),
        "refusal must open no file: adapter_called must be false: {refusal_event}"
    );
    assert!(
        refusal_event.contains("\"byte_count\":0"),
        "refusal must read no payload: byte_count must be 0: {refusal_event}"
    );

    // Human trust run: attestation admits with the external-trust label on
    // coherent storage. On storage where the classifier's selected
    // mountinfo device contradicts stat (p1_mountinfo_stat_contradiction_v0)
    // -- e.g. namespaced sandbox mounts -- the live backing bind rejects
    // fail-closed: attestation covers missing observations, never
    // contradictions.
    let trusted = run_with_trust(&program, &fixture, false);
    let trusted_stderr = String::from_utf8_lossy(&trusted.stderr);
    if trusted_stderr.contains("p1_mountinfo_stat_contradiction_v0") {
        assert_eq!(
            trusted.status.code(),
            Some(1),
            "contradictory backing evidence must fail closed: {trusted_stderr}"
        );
        assert!(
            trusted.stdout.is_empty(),
            "contradictory backing evidence must read no payload"
        );
        assert!(
            trusted_stderr.contains("FileReadError.contradictory_backing_evidence"),
            "contradiction must surface the typed rejection: {trusted_stderr}"
        );
        assert!(
            !trusted_stderr.contains("panicked"),
            "contradiction must fail closed without panicking: {trusted_stderr}"
        );
        // The JSON run takes the same rejection path; the envelope still
        // carries the contradiction reason and the verbatim classifier
        // reason for forensics.
        let trusted_json = run_with_trust(&program, &fixture, true);
        let json_stderr = String::from_utf8_lossy(&trusted_json.stderr);
        assert_eq!(
            trusted_json.status.code(),
            Some(1),
            "JSON contradiction run must exit 1: {json_stderr}"
        );
        let envelope = String::from_utf8_lossy(&trusted_json.stdout);
        assert!(
            envelope.trim_start().starts_with('{'),
            "JSON mode stdout must carry only the envelope: {envelope}"
        );
        let trust_event = files_read_exercise_event(&envelope);
        assert_eq!(
            event_string_field(trust_event, "result"),
            "observed_backing_identity_mismatch_before_read_v0",
            "rejection event must record the contradiction reason"
        );
        assert_eq!(
            event_string_field(trust_event, "classifier_reason"),
            refusal_classifier_reason,
            "rejection must preserve the classifier reason verbatim"
        );
        return;
    }
    assert_eq!(
        trusted.status.code(),
        Some(0),
        "attested run must exit 0: {trusted_stderr}"
    );
    assert_eq!(
        trusted.stdout.as_slice(),
        EXPECTED_STDOUT,
        "attested run stdout must be byte-exact"
    );
    assert!(
        !trusted_stderr.contains("panicked"),
        "attested run must not panic: {trusted_stderr}"
    );
    assert!(
        trusted_stderr.contains("trusted-not-proven"),
        "attested run must carry the trusted-not-proven evidence label: {trusted_stderr}"
    );

    // JSON trust run: stdout carries ONLY the envelope; the envelope parses,
    // program_output_bytes equals the expected bytes exactly, the classified
    // authority event carries external-trust, and its classifier_reason is
    // the refusal run's reason preserved verbatim (never hardcoded: the
    // classifier reason is storage/host-specific).
    let trusted_json = run_with_trust(&program, &fixture, true);
    let json_stderr = String::from_utf8_lossy(&trusted_json.stderr);
    assert_eq!(
        trusted_json.status.code(),
        Some(0),
        "JSON attested run must exit 0: {json_stderr}"
    );
    let envelope = String::from_utf8_lossy(&trusted_json.stdout);
    assert!(
        envelope.trim_start().starts_with('{'),
        "JSON mode stdout must carry only the envelope: {envelope}"
    );
    assert_eq!(
        envelope_program_output_bytes(&envelope),
        EXPECTED_STDOUT,
        "envelope program_output_bytes must equal the expected bytes exactly"
    );
    assert_eq!(
        envelope_string_field_values(&envelope, "locality_classification"),
        vec!["external-trust".to_owned()],
        "envelope must classify the read as external-trust: {envelope}"
    );
    let trust_event = files_read_exercise_event(&envelope);
    assert_eq!(
        event_string_field(trust_event, "classifier_reason"),
        refusal_classifier_reason,
        "trust path must preserve the classifier reason verbatim from the refusal run"
    );
    assert!(
        json_stderr.contains("trusted-not-proven"),
        "JSON mode must still emit the evidence line on stderr: {json_stderr}"
    );

    // Trust evidence bundle: the attestation fact, the matching-grant fact,
    // the classifier's Unproven reason marked honestly as unproven, the
    // bound file identity -- and the P1-P4 observation/enforcement
    // outcomes, never proof claims.
    let evidence_arrays = envelope_string_array_field_values(&envelope, "locality_evidence");
    assert_eq!(
        evidence_arrays.len(),
        1,
        "exactly one event must carry locality_evidence: {envelope}"
    );
    let evidence_lines = &evidence_arrays[0];
    assert!(
        evidence_lines
            .iter()
            .any(|line| line == &format!("attestation: operator attested files.read={fixture}")),
        "evidence must carry the attestation fact: {evidence_lines:?}"
    );
    assert!(
        evidence_lines
            .iter()
            .any(|line| line == &format!("allow: matching files.read={fixture}")),
        "evidence must carry the matching-grant fact: {evidence_lines:?}"
    );
    assert!(
        evidence_lines
            .iter()
            .any(|line| line
                == &format!("classifier: {refusal_classifier_reason} (proof unavailable)")),
        "evidence must honestly record the classifier's Unproven reason: {evidence_lines:?}"
    );
    for prefix in ["P1:", "P2:", "P3:", "P4:"] {
        assert!(
            evidence_lines.iter().any(|line| line.starts_with(prefix)),
            "external-trust evidence must carry a {prefix} observation/enforcement outcome: {evidence_lines:?}"
        );
    }
    for line in evidence_lines {
        assert!(
            !line.contains("proved"),
            "no proof claims on the trust path: {line}"
        );
    }

    // Bound file identity: the opened file's platform-correct identity, per
    // FileObjectIdentity::render. Values are host-dependent: on unix they
    // are verified exactly against the fixture's stat (dev, ino); on Windows
    // only the volume_serial/file_index shape is asserted.
    let identities = envelope_string_field_values(&envelope, "bound_file_identity");
    assert_eq!(
        identities.len(),
        1,
        "exactly one event must carry bound_file_identity: {envelope}"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::metadata(&fixture).expect("fixture metadata");
        assert_eq!(
            identities[0],
            format!("dev={} ino={}", metadata.dev(), metadata.ino()),
            "bound_file_identity must be the opened file's (dev, ino)"
        );
    }
    #[cfg(windows)]
    {
        let identity = &identities[0];
        let parts: Vec<&str> = identity.split(' ').collect();
        assert_eq!(
            parts.len(),
            2,
            "bound_file_identity must be 'volume_serial=<n> file_id=<32 hex>', got {identity:?}"
        );
        let serial = parts[0].strip_prefix("volume_serial=").unwrap_or_else(|| {
            panic!("bound_file_identity must start with volume_serial=, got {identity:?}")
        });
        let file_id = parts[1]
            .strip_prefix("file_id=")
            .unwrap_or_else(|| panic!("bound_file_identity must carry file_id=, got {identity:?}"));
        assert!(
            !serial.is_empty() && serial.chars().all(|c| c.is_ascii_digit()),
            "volume_serial must be numeric, got {identity:?}"
        );
        assert!(
            file_id.len() == 32
                && file_id
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "file_id must be 32 lowercase hex digits, got {identity:?}"
        );
    }
    #[cfg(not(any(unix, windows)))]
    {
        assert!(
            !identities[0].is_empty(),
            "bound_file_identity must be recorded: {:?}",
            identities[0]
        );
    }
}

#[test]
fn cli_trust_locality_misuse_pins() {
    // Negative pins around the attestation, all unconditional under
    // grant-first: (a) and (d) fail at the consent gate; (c) and (e) fail at
    // the locality gate because the attestation never matches.
    let (probe_dir, program) = write_program("trust_locality_misuse", PROBE_PROGRAM);
    let fixture = fixture_path();

    // (a) No --allow files.read on the OS-readable fixture: denied.
    let denied_args = vec![
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        "--args".to_owned(),
        fixture.clone(),
    ];
    let denied = run_hum_run(&denied_args, &program, None);
    let denied_stderr = String::from_utf8_lossy(&denied.stderr);
    assert_eq!(denied.status.code(), Some(1), "ungranted read must exit 1");
    assert!(
        denied_stderr.contains("ProbeError.read") && denied_stderr.contains("FileReadError.denied"),
        "(a) ungranted read must fail closed with ProbeError.read caused by FileReadError.denied: {denied_stderr}"
    );
    assert!(
        !denied_stderr.contains("panicked"),
        "(a) must not panic: {denied_stderr}"
    );

    // (d) --trust-locality without --allow files.read: denied. Attestation
    // without consent attests nothing.
    let trust_no_allow_args = vec![
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        "--trust-locality".to_owned(),
        format!("files.read={fixture}"),
        "--args".to_owned(),
        fixture.clone(),
    ];
    let trust_denied = run_hum_run(&trust_no_allow_args, &program, None);
    let trust_denied_stderr = String::from_utf8_lossy(&trust_denied.stderr);
    assert_eq!(
        trust_denied.status.code(),
        Some(1),
        "trust without grant must exit 1"
    );
    assert!(
        trust_denied_stderr.contains("FileReadError.denied"),
        "(d) trust without the files.read grant must fail closed with FileReadError.denied: {trust_denied_stderr}"
    );
    assert!(
        !trust_denied_stderr.contains("panicked"),
        "(d) must not panic: {trust_denied_stderr}"
    );

    // (c) --trust-locality path != --allow path: the attestation does not
    // match the grant, so the locality gate refuses with unavailable. A
    // nonexistent sibling of the probe dir is a valid native path that is
    // not the granted fixture.
    let other_path = probe_dir
        .join("not-the-fixture.txt")
        .to_str()
        .expect("temp path is UTF-8")
        .to_owned();
    let mismatch_args = vec![
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        format!("--allow=files.read={fixture}"),
        "--trust-locality".to_owned(),
        format!("files.read={other_path}"),
        "--args".to_owned(),
        fixture.clone(),
    ];
    let mismatch = run_hum_run(&mismatch_args, &program, None);
    let mismatch_stderr = String::from_utf8_lossy(&mismatch.stderr);
    assert_eq!(
        mismatch.status.code(),
        Some(1),
        "(c) mismatched attestation must exit 1"
    );
    assert!(
        mismatch_stderr.contains("FileReadError.unavailable"),
        "(c) mismatched attestation must refuse with FileReadError.unavailable: {mismatch_stderr}"
    );
    assert!(
        !mismatch_stderr.contains("trusted-not-proven"),
        "(c) mismatched attestation must not emit a trust label: {mismatch_stderr}"
    );

    // (e) HUM_TRUST_LOCALITY env var with no flag: env must not attest.
    let env_args = vec![
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        format!("--allow=files.read={fixture}"),
        "--args".to_owned(),
        fixture.clone(),
    ];
    let env_attestation = format!("files.read={fixture}");
    let env_run = run_hum_run(
        &env_args,
        &program,
        Some(("HUM_TRUST_LOCALITY", env_attestation.as_str())),
    );
    let env_stderr = String::from_utf8_lossy(&env_run.stderr);
    assert_eq!(
        env_run.status.code(),
        Some(1),
        "(e) env-only attestation must exit 1"
    );
    assert!(
        env_stderr.contains("FileReadError.unavailable"),
        "(e) env must not attest: refusal must be FileReadError.unavailable: {env_stderr}"
    );
    assert!(
        !env_stderr.contains("trusted-not-proven"),
        "(e) env must not attest: no trust label may be emitted: {env_stderr}"
    );
}

#[test]
fn cli_trust_locality_quiet_program_emits_no_file_read_evidence() {
    // E2: an otherwise quiet successful program carries no file-read
    // evidence line. The assertion targets the new file-read evidence
    // specifically (no files.read authority event, no `files.read ...:`
    // stderr evidence line) -- not globally empty stderr.
    let (_quiet_dir, program) = write_program("trust_locality_quiet", QUIET_PROGRAM);
    let fixture = fixture_path();
    let args = vec![
        "--format".to_owned(),
        "json".to_owned(),
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        "--args".to_owned(),
        fixture,
    ];
    let run = run_hum_run(&args, &program, None);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(0),
        "quiet run must exit 0: {stderr}"
    );
    let envelope = String::from_utf8_lossy(&run.stdout);
    assert!(
        envelope.trim_start().starts_with('{'),
        "JSON mode stdout must carry only the envelope: {envelope}"
    );
    for event in authority_event_slices(&envelope) {
        assert!(
            !event.contains("\"capability_id\":\"files.read\""),
            "quiet run must emit no files.read authority event: {event}"
        );
    }
    for line in stderr.lines() {
        assert!(
            !line.starts_with("files.read "),
            "quiet run must emit no file-read evidence line on stderr: {line}"
        );
    }
}

#[test]
fn json_envelope_empty_output_shape() {
    // D5 output control 1: a successful program that writes nothing yields
    // an envelope with an empty program_output_bytes array, exit_code 0,
    // and stdout carrying only the envelope line.
    let (_dir, program) = write_program("d5_empty_output", EMPTY_OUTPUT_PROGRAM);
    let fixture = fixture_path();
    let args = vec![
        "--format".to_owned(),
        "json".to_owned(),
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        "--args".to_owned(),
        fixture,
    ];
    let run = run_hum_run(&args, &program, None);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(0),
        "empty-output run must exit 0: {stderr}"
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    // println! emits the envelope plus exactly one trailing newline: stdout
    // carries nothing else.
    assert_eq!(
        stdout.lines().count(),
        1,
        "JSON mode stdout must be exactly the envelope line: {stdout:?}"
    );
    let envelope = stdout.trim_end_matches('\n');
    assert_eq!(
        stdout,
        format!("{envelope}\n"),
        "stdout must be the envelope plus one trailing newline"
    );
    assert_eq!(
        envelope_head_outcome(envelope),
        "app_success",
        "empty-output run must report app_success"
    );
    assert_eq!(
        envelope_head_exit_code(envelope),
        0,
        "empty-output run must report exit_code 0"
    );
    assert_eq!(
        envelope_program_output_bytes(envelope),
        Vec::<u8>::new(),
        "empty output must produce an empty program_output_bytes array"
    );
}

#[test]
fn json_envelope_preserves_trailing_newline() {
    // D5 output control 2: the trailing newline byte is preserved exactly
    // in program_output_bytes -- neither stripped nor added.
    let (_dir, program) = write_program("d5_trailing_newline", TRAILING_NEWLINE_PROGRAM);
    let fixture = fixture_path();
    let args = vec![
        "--format".to_owned(),
        "json".to_owned(),
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        "--args".to_owned(),
        fixture,
    ];
    let run = run_hum_run(&args, &program, None);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(0),
        "trailing-newline run must exit 0: {stderr}"
    );
    let envelope = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        envelope_program_output_bytes(envelope.trim_end()),
        b"abc\n",
        "envelope must preserve the trailing newline byte exactly"
    );
}

#[test]
fn json_envelope_partial_output_on_ordinary_failure() {
    // D5 output control 3: bytes written before an ordinary failure are
    // preserved in program_output_bytes, with the authoritative nonzero
    // exit code -- the envelope is still emitted on failure.
    let (_dir, program) = write_program("d5_partial_failure", PARTIAL_FAILURE_PROGRAM);
    let fixture = fixture_path();
    let args = vec![
        "--format".to_owned(),
        "json".to_owned(),
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        "--args".to_owned(),
        fixture,
    ];
    let run = run_hum_run(&args, &program, None);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(1),
        "partial-failure run must exit 1: {stderr}"
    );
    let envelope = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        envelope_head_outcome(envelope.trim_end()),
        "app_failure",
        "partial-failure run must report app_failure"
    );
    assert_eq!(
        envelope_head_exit_code(envelope.trim_end()),
        1,
        "partial-failure run must report exit_code 1"
    );
    assert_eq!(
        envelope_program_output_bytes(envelope.trim_end()),
        b"partial-bytes",
        "envelope must preserve the bytes written before the failure"
    );
}

#[test]
fn json_envelope_exit_code_is_authoritative() {
    // D5 output control 4: the envelope's exit_code equals the process exit
    // code the CLI reports -- the envelope is authoritative, never a second
    // rendering. The failing program writes nothing, so the bytes stay
    // empty while the code stays nonzero.
    let (_dir, program) = write_program("d5_empty_failure", EMPTY_FAILURE_PROGRAM);
    let fixture = fixture_path();
    let args = vec![
        "--format".to_owned(),
        "json".to_owned(),
        "--args".to_owned(),
        fixture,
    ];
    let run = run_hum_run(&args, &program, None);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(1),
        "empty-failure run must exit 1: {stderr}"
    );
    let envelope = String::from_utf8_lossy(&run.stdout);
    let envelope = envelope.trim_end();
    assert_eq!(
        envelope_head_exit_code(envelope),
        u64::from(run.status.code().expect("exit code present") as u32),
        "envelope exit_code must equal the process exit code"
    );
    assert_eq!(
        envelope_program_output_bytes(envelope),
        Vec::<u8>::new(),
        "failed run with no writes must carry empty program_output_bytes"
    );
}
