// Production-connected CLI tests for WO29 Slice A trust-locality
// (decision 0029 Option D: per-invocation, per-path operator attestation).
//
// These spawn the actual `hum` binary to verify the provability-aware
// admission gate, the `--trust-locality files.read=<path>` attestation, the
// refusal shapes, and the JSON run envelope. Adapter-only tests are
// insufficient per review.
//
// Provability-awareness: the suite never assumes the platform's storage
// classification. A no-trust probe (consent + matching grant, no
// attestation) runs first and the observed outcome decides the branch. On
// this Linux box the fixture lives on overlayfs, so the classifier yields
// Unproven and attested runs take the `external-trust` path; on
// proof-capable storage the same assertions pin `proved` (proof outranks
// trust).

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

// Byte-exact expected stdout of the probe on fixtures/wordfreq/sample.txt.
// The app entry completes with AppSuccess, so the CLI prints nothing beyond
// the program's own writes.
const EXPECTED_STDOUT: &[u8] = b"hum  lang\nhum\n";

fn hum_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hum"))
}

fn write_probe_program(name: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "hum_wo29_trust_locality_{name}_{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("trust_locality_probe.hum");
    std::fs::write(&path, PROBE_PROGRAM).expect("write probe program");
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

/// Consent + matching files.read grant, no attestation: the provability
/// probe. Exit 1 means the classifier left the storage unproven; exit 0
/// means proof admitted the read.
fn probe_no_trust(program: &Path, fixture: &str) -> std::process::Output {
    let args = vec![
        "--allow".to_owned(),
        "stdout.write".to_owned(),
        format!("--allow=files.read={fixture}"),
        "--args".to_owned(),
        fixture.to_owned(),
    ];
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
                            let code = u32::from_str_radix(&hex, 16).unwrap_or_else(|_| {
                                panic!("bad \\u escape in field {field}: {envelope}")
                            });
                            decoded.push(char::from_u32(code).unwrap_or_else(|| {
                                panic!("bad \\u escape in field {field}: {envelope}")
                            }));
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
            let end = end.unwrap_or_else(|| {
                panic!("field {field} string value must terminate in: {envelope}")
            });
            values.push(decoded);
            rest = &quoted[end + 1..];
        } else {
            // null or a non-string value: advance past this key occurrence.
            rest = value_start;
        }
    }
    values
}

#[test]
fn cli_trust_locality_probe_and_attested_admission() {
    // Provability-aware admission: the no-trust probe decides the branch.
    // - exit 1: storage is unproven (overlayfs on hosted Linux). Pins
    //   negative (b): consent + matching grant without attestation refuses
    //   with FileReadError.unavailable. The trust run must then admit with
    //   the external-trust label.
    // - exit 0: storage is proof-capable. The probe already admitted via
    //   proof; the trust run must keep the proved label (proof outranks
    //   trust).
    let (_probe_dir, program) = write_probe_program("admission");
    let fixture = fixture_path();

    let probe = probe_no_trust(&program, &fixture);
    let probe_stderr = String::from_utf8_lossy(&probe.stderr);
    assert!(
        !probe_stderr.contains("panicked"),
        "probe must fail closed without panicking: {probe_stderr}"
    );
    let admitted_class: &str = match probe.status.code() {
        Some(1) => {
            // Negative pin (b): matching --allow without trust on unprovable
            // storage refuses with FileReadError.unavailable.
            assert!(probe.stdout.is_empty(), "refused read must write no stdout");
            assert!(
                probe_stderr.contains("ProbeError.read")
                    && probe_stderr.contains("FileReadError.unavailable"),
                "unprovable no-trust probe must fail closed with ProbeError.read caused by FileReadError.unavailable: {probe_stderr}"
            );
            "external-trust"
        }
        Some(0) => {
            // Proof-capable storage: the probe admits with the proved label.
            assert_eq!(
                probe.stdout.as_slice(),
                EXPECTED_STDOUT,
                "proved probe stdout must be byte-exact"
            );
            assert!(
                probe_stderr.contains("proved"),
                "proved probe must carry the proved evidence label on stderr: {probe_stderr}"
            );
            "proved"
        }
        other => panic!("no-trust probe must exit 0 or 1, got {other:?}; stderr: {probe_stderr}"),
    };

    // Human trust run: attestation admits; byte-exact stdout; the evidence
    // label matches the observed admission class.
    let trusted = run_with_trust(&program, &fixture, false);
    let trusted_stderr = String::from_utf8_lossy(&trusted.stderr);
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
    match admitted_class {
        "external-trust" => assert!(
            trusted_stderr.contains("trusted-not-proven"),
            "attested run on unprovable storage must carry the trusted-not-proven label: {trusted_stderr}"
        ),
        _ => {
            assert!(
                trusted_stderr.contains("proved"),
                "attested run on proof-capable storage must carry the proved label: {trusted_stderr}"
            );
            assert!(
                !trusted_stderr.contains("trusted-not-proven"),
                "proof outranks trust: proved storage must not carry the trusted-not-proven label: {trusted_stderr}"
            );
        }
    }

    // JSON trust run: stdout carries ONLY the envelope; the envelope parses,
    // program_output_bytes equals the expected bytes exactly, and the
    // classified authority event carries the observed admission class.
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
        vec![admitted_class.to_owned()],
        "envelope must classify the read as {admitted_class}: {envelope}"
    );
    match admitted_class {
        "external-trust" => assert!(
            json_stderr.contains("trusted-not-proven"),
            "JSON mode must still emit the evidence line on stderr: {json_stderr}"
        ),
        _ => assert!(
            json_stderr.contains("proved") && !json_stderr.contains("trusted-not-proven"),
            "JSON mode on proof-capable storage must carry the proved label: {json_stderr}"
        ),
    }
}

#[test]
fn cli_trust_locality_misuse_pins() {
    // Negative pins around the attestation. (a) and (d) are unconditional:
    // the consent gate runs before classification. (c) and (e) are
    // provability-aware: on unprovable storage they refuse; on proof-capable
    // storage proof admits (proof outranks trust) and the env var stays
    // inert.
    let (probe_dir, program) = write_probe_program("misuse");
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

    // Branch for (c) and (e).
    let probe = probe_no_trust(&program, &fixture);
    let unprovable = match probe.status.code() {
        Some(1) => true,
        Some(0) => false,
        other => panic!(
            "no-trust probe must exit 0 or 1, got {other:?}; stderr: {}",
            String::from_utf8_lossy(&probe.stderr)
        ),
    };

    // (c) --trust-locality path != --allow path: the attestation does not
    // match the grant. A nonexistent sibling of the probe dir is a valid
    // native path that is not the granted fixture.
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
    if unprovable {
        assert_eq!(
            mismatch.status.code(),
            Some(1),
            "(c) mismatched attestation must exit 1 on unprovable storage"
        );
        assert!(
            mismatch_stderr.contains("FileReadError.unavailable"),
            "(c) mismatched attestation must refuse with FileReadError.unavailable: {mismatch_stderr}"
        );
        assert!(
            !mismatch_stderr.contains("trusted-not-proven"),
            "(c) mismatched attestation must not emit a trust label: {mismatch_stderr}"
        );
    } else {
        assert_eq!(
            mismatch.status.code(),
            Some(0),
            "(c) proof admits despite the mismatched attestation"
        );
        assert_eq!(
            mismatch.stdout.as_slice(),
            EXPECTED_STDOUT,
            "(c) proved stdout must be byte-exact"
        );
        assert!(
            mismatch_stderr.contains("proved") && !mismatch_stderr.contains("trusted-not-proven"),
            "(c) proof outranks trust: {mismatch_stderr}"
        );
    }
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
    if unprovable {
        assert_eq!(
            env_run.status.code(),
            Some(1),
            "(e) env-only attestation must exit 1 on unprovable storage"
        );
        assert!(
            env_stderr.contains("FileReadError.unavailable"),
            "(e) env must not attest: refusal must be FileReadError.unavailable: {env_stderr}"
        );
        assert!(
            !env_stderr.contains("trusted-not-proven"),
            "(e) env must not attest: no trust label may be emitted: {env_stderr}"
        );
    } else {
        assert_eq!(
            env_run.status.code(),
            Some(0),
            "(e) proof admits; the env var stays inert"
        );
        assert!(
            env_stderr.contains("proved") && !env_stderr.contains("trusted-not-proven"),
            "(e) env var must not change the proved label: {env_stderr}"
        );
    }
}
