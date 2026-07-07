#![forbid(unsafe_code)]

mod ast;
mod capabilities;
mod check;
mod diagnostic;
mod diagnostic_catalog;
mod diagnostics;
mod explain;
mod graph;
mod json;
mod node_id;
mod parser;
mod syntax;
mod test_skeletons;
mod version;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use ast::Program;
use diagnostic::{Diagnostic, Severity};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(ExitCode::SUCCESS);
    }
    if args.len() == 1 && matches!(args[0].as_str(), "--version" | "-V") {
        print!("{}", version::version_text());
        return Ok(ExitCode::SUCCESS);
    }

    let options = parse_cli(args)?;
    if options.command == "syntax" {
        match options.syntax_format {
            SyntaxFormat::Json => print!("{}", syntax::syntax_json()),
            SyntaxFormat::TextMate => print!("{}", syntax::textmate_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "capabilities" {
        match options.capabilities_format {
            CapabilitiesFormat::Human => print!("{}", capabilities::capabilities_text()),
            CapabilitiesFormat::Json => print!("{}", capabilities::capabilities_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "version" {
        match options.version_format {
            VersionFormat::Human => print!("{}", version::version_text()),
            VersionFormat::Json => print!("{}", version::version_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "explain" {
        let code = options
            .explain_code
            .as_deref()
            .expect("explain command should have a diagnostic code");
        match options.explain_format {
            ExplainFormat::Human => print!("{}", explain::explain_text(code)?),
            ExplainFormat::Json => print!("{}", explain::explain_json(code)?),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "diagnostics" {
        match options.diagnostics_format {
            DiagnosticsFormat::Human => print!("{}", diagnostics::diagnostics_text()),
            DiagnosticsFormat::Json => print!("{}", diagnostics::diagnostics_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }

    let loaded = load_program(&options.inputs)?;
    let program = loaded.program;
    let diagnostics = loaded.diagnostics;
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);

    match options.command.as_str() {
        "check" => {
            match options.check_format {
                CheckFormat::Human => {
                    print_diagnostics(&diagnostics);
                    println!(
                        "checked {} file(s): {} error(s), {} warning(s)",
                        program.files.len(),
                        diagnostics
                            .iter()
                            .filter(|diagnostic| diagnostic.severity == Severity::Error)
                            .count(),
                        diagnostics
                            .iter()
                            .filter(|diagnostic| diagnostic.severity == Severity::Warning)
                            .count()
                    );
                }
                CheckFormat::Json => print!("{}", diagnostics::check_json(&program, &diagnostics)),
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "graph" => {
            println!("{}", json::program_to_json(&program, &diagnostics));
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "test-skeletons" => {
            print_diagnostics(&diagnostics);
            if !has_errors {
                let skeletons = test_skeletons::program_to_test_skeletons(&program);
                if skeletons.is_empty() {
                    eprintln!("no unlinked test obligations");
                } else {
                    print!("{skeletons}");
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        other => Err(format!(
            "unknown command `{other}`; expected `check`, `graph`, `test-skeletons`, `syntax`, `version`, `explain`, `diagnostics`, or `capabilities`"
        )),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyntaxFormat {
    Json,
    TextMate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VersionFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExplainFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticsFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapabilitiesFormat {
    Human,
    Json,
}

#[derive(Debug)]
struct CliOptions {
    command: String,
    inputs: Vec<PathBuf>,
    show_timings: bool,
    check_format: CheckFormat,
    syntax_format: SyntaxFormat,
    version_format: VersionFormat,
    explain_format: ExplainFormat,
    diagnostics_format: DiagnosticsFormat,
    capabilities_format: CapabilitiesFormat,
    explain_code: Option<String>,
}

struct LoadedProgram {
    program: Program,
    diagnostics: Vec<Diagnostic>,
    timings: Vec<FileTiming>,
    total: Duration,
}

struct FileTiming {
    path: String,
    read: Duration,
    parse: Duration,
    check: Duration,
}

fn parse_cli(args: Vec<String>) -> Result<CliOptions, String> {
    let command = args[0].clone();
    if !matches!(
        command.as_str(),
        "check"
            | "graph"
            | "test-skeletons"
            | "syntax"
            | "version"
            | "explain"
            | "diagnostics"
            | "capabilities"
    ) {
        return Err(format!(
            "unknown command `{command}`; expected `check`, `graph`, `test-skeletons`, `syntax`, `version`, `explain`, `diagnostics`, or `capabilities`"
        ));
    }

    let mut show_timings = false;
    let mut raw_inputs = Vec::new();
    let mut check_format = CheckFormat::Human;
    let mut syntax_format = SyntaxFormat::Json;
    let mut version_format = VersionFormat::Human;
    let mut explain_format = ExplainFormat::Human;
    let mut diagnostics_format = DiagnosticsFormat::Human;
    let mut capabilities_format = CapabilitiesFormat::Human;
    let mut args = args.into_iter().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--timings" => show_timings = true,
            "--format"
                if matches!(
                    command.as_str(),
                    "check" | "syntax" | "version" | "explain" | "diagnostics" | "capabilities"
                ) =>
            {
                let Some(value) = args.next() else {
                    return Err(format!("`{command} --format` requires a format value"));
                };
                match command.as_str() {
                    "check" => check_format = parse_check_format(&value)?,
                    "syntax" => syntax_format = parse_syntax_format(&value)?,
                    "version" => version_format = parse_version_format(&value)?,
                    "explain" => explain_format = parse_explain_format(&value)?,
                    "diagnostics" => diagnostics_format = parse_diagnostics_format(&value)?,
                    "capabilities" => capabilities_format = parse_capabilities_format(&value)?,
                    _ => unreachable!(),
                }
            }
            flag if matches!(
                command.as_str(),
                "check" | "syntax" | "version" | "explain" | "diagnostics" | "capabilities"
            ) && flag.starts_with("--format=") =>
            {
                let value = flag.trim_start_matches("--format=");
                match command.as_str() {
                    "check" => check_format = parse_check_format(value)?,
                    "syntax" => syntax_format = parse_syntax_format(value)?,
                    "version" => version_format = parse_version_format(value)?,
                    "explain" => explain_format = parse_explain_format(value)?,
                    "diagnostics" => diagnostics_format = parse_diagnostics_format(value)?,
                    "capabilities" => capabilities_format = parse_capabilities_format(value)?,
                    _ => unreachable!(),
                }
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag `{flag}`")),
            _ => raw_inputs.push(arg),
        }
    }

    if command == "syntax" {
        if show_timings {
            return Err("`syntax` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`syntax` does not accept input files".to_string());
        }
        return Ok(CliOptions {
            command,
            inputs: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            explain_code: None,
        });
    }

    if command == "version" {
        if show_timings {
            return Err("`version` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`version` does not accept input files".to_string());
        }
        return Ok(CliOptions {
            command,
            inputs: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            explain_code: None,
        });
    }

    if command == "explain" {
        if show_timings {
            return Err("`explain` does not support `--timings`".to_string());
        }
        if raw_inputs.len() != 1 {
            return Err("`explain` requires exactly one diagnostic code".to_string());
        }
        return Ok(CliOptions {
            command,
            inputs: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            explain_code: raw_inputs.first().cloned(),
        });
    }

    if command == "diagnostics" {
        if show_timings {
            return Err("`diagnostics` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`diagnostics` does not accept input files".to_string());
        }
        return Ok(CliOptions {
            command,
            inputs: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            explain_code: None,
        });
    }

    if command == "capabilities" {
        if show_timings {
            return Err("`capabilities` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`capabilities` does not accept input files".to_string());
        }
        return Ok(CliOptions {
            command,
            inputs: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            explain_code: None,
        });
    }

    Ok(CliOptions {
        command,
        inputs: collect_inputs(&raw_inputs)?,
        show_timings,
        check_format,
        syntax_format,
        version_format,
        explain_format,
        diagnostics_format,
        capabilities_format,
        explain_code: None,
    })
}

fn parse_check_format(value: &str) -> Result<CheckFormat, String> {
    match value {
        "human" => Ok(CheckFormat::Human),
        "json" => Ok(CheckFormat::Json),
        other => Err(format!(
            "unknown check format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_syntax_format(value: &str) -> Result<SyntaxFormat, String> {
    match value {
        "json" => Ok(SyntaxFormat::Json),
        "textmate" => Ok(SyntaxFormat::TextMate),
        other => Err(format!(
            "unknown syntax format `{other}`; expected `json` or `textmate`"
        )),
    }
}

fn parse_version_format(value: &str) -> Result<VersionFormat, String> {
    match value {
        "human" => Ok(VersionFormat::Human),
        "json" => Ok(VersionFormat::Json),
        other => Err(format!(
            "unknown version format `{other}`; expected `human` or `json`"
        )),
    }
}
fn parse_explain_format(value: &str) -> Result<ExplainFormat, String> {
    match value {
        "human" => Ok(ExplainFormat::Human),
        "json" => Ok(ExplainFormat::Json),
        other => Err(format!(
            "unknown explain format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_diagnostics_format(value: &str) -> Result<DiagnosticsFormat, String> {
    match value {
        "human" => Ok(DiagnosticsFormat::Human),
        "json" => Ok(DiagnosticsFormat::Json),
        other => Err(format!(
            "unknown diagnostics format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_capabilities_format(value: &str) -> Result<CapabilitiesFormat, String> {
    match value {
        "human" => Ok(CapabilitiesFormat::Human),
        "json" => Ok(CapabilitiesFormat::Json),
        other => Err(format!(
            "unknown capabilities format `{other}`; expected `human` or `json`"
        )),
    }
}

fn load_program(paths: &[PathBuf]) -> Result<LoadedProgram, String> {
    let total_start = Instant::now();
    let mut program = Program::default();
    let mut diagnostics = Vec::new();
    let mut timings = Vec::new();

    for path in paths {
        let read_start = Instant::now();
        let source = fs::read_to_string(path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        let read = read_start.elapsed();

        let parse_start = Instant::now();
        let parsed = parser::parse_source(path.display().to_string(), &source);
        let parse = parse_start.elapsed();

        let check_start = Instant::now();
        let file_diagnostics = check::check_file(&parsed.file);
        let check = check_start.elapsed();

        diagnostics.extend(parsed.diagnostics);
        diagnostics.extend(file_diagnostics);
        timings.push(FileTiming {
            path: path.display().to_string(),
            read,
            parse,
            check,
        });
        program.files.push(parsed.file);
    }

    Ok(LoadedProgram {
        program,
        diagnostics,
        timings,
        total: total_start.elapsed(),
    })
}

fn collect_inputs(raw_inputs: &[String]) -> Result<Vec<PathBuf>, String> {
    if raw_inputs.is_empty() {
        return Err("pass a .hum file or directory".to_string());
    }

    let mut inputs = Vec::new();
    for raw_input in raw_inputs {
        let path = PathBuf::from(raw_input);
        if path.is_dir() {
            collect_hum_files(&path, &mut inputs)?;
        } else if path.is_file() {
            inputs.push(path);
        } else {
            return Err(format!("input `{raw_input}` does not exist"));
        }
    }

    inputs.sort();
    inputs.dedup();
    Ok(inputs)
}

fn collect_hum_files(path: &Path, inputs: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("failed to read directory `{}`: {error}", path.display()))?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read directory entry in `{}`: {error}",
                path.display()
            )
        })?;
        let child = entry.path();
        if child.is_dir() {
            collect_hum_files(&child, inputs)?;
        } else if child
            .extension()
            .is_some_and(|extension| extension == "hum")
        {
            inputs.push(child);
        }
    }
    Ok(())
}

fn print_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("{}", diagnostic.render());
    }
}

fn print_timings(timings: &[FileTiming], total: Duration) {
    eprintln!("timings:");
    for timing in timings {
        eprintln!(
            "  {} read={} parse={} check={}",
            timing.path,
            format_duration(timing.read),
            format_duration(timing.parse),
            format_duration(timing.check)
        );
    }
    eprintln!("  total={}", format_duration(total));
}

fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{micros}us")
    } else {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    }
}

fn print_help() {
    println!("Hum compiler front-end");
    println!();
    println!("Usage:");
    println!("  hum check [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum graph [--timings] <file-or-dir>...");
    println!("  hum test-skeletons [--timings] <file-or-dir>...");
    println!("  hum syntax [--format json|textmate]");
    println!("  hum version [--format human|json]");
    println!("  hum explain <H####> [--format human|json]");
    println!("  hum diagnostics [--format human|json]");
    println!("  hum capabilities [--format human|json]");
    println!();
    println!("Commands:");
    println!("  check           Parse Hum files and run milestone-0 intent checks");
    println!("  graph           Emit hum.semantic_graph.v0 JSON for agents and tools");
    println!("  test-skeletons  Print Hum test skeletons for unlinked obligations");
    println!("  syntax          Emit syntax JSON or generated TextMate grammar");
    println!("  version         Print toolchain identity and schema versions");
    println!("  explain         Explain a stable diagnostic code");
    println!("  diagnostics     List stable diagnostic codes");
    println!("  capabilities    List machine-readable tool and editor surfaces");
    println!();
    println!("Options:");
    println!("  --timings   Print read/parse/check timings per input file");
    println!("  --version   Print toolchain identity");
    println!("  --format    Choose command output format where supported");
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        CapabilitiesFormat, CheckFormat, DiagnosticsFormat, ExplainFormat, SyntaxFormat,
        VersionFormat, load_program, parse_cli,
    };

    #[test]
    fn parses_check_json_format() {
        let options = parse_cli(vec![
            "check".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("check json command");
        assert_eq!(options.command, "check");
        assert_eq!(options.check_format, CheckFormat::Json);
    }

    #[test]
    fn rejects_unknown_check_format() {
        let error = parse_cli(vec![
            "check".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("check should reject unknown formats");
        assert_eq!(
            error,
            "unknown check format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_syntax_command_without_inputs() {
        let options = parse_cli(vec!["syntax".to_string()]).expect("syntax command");
        assert_eq!(options.command, "syntax");
        assert!(options.inputs.is_empty());
        assert!(!options.show_timings);
        assert_eq!(options.syntax_format, SyntaxFormat::Json);
    }

    #[test]
    fn parses_syntax_textmate_format() {
        let options = parse_cli(vec![
            "syntax".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect("syntax textmate command");
        assert_eq!(options.command, "syntax");
        assert_eq!(options.syntax_format, SyntaxFormat::TextMate);
    }

    #[test]
    fn rejects_unknown_syntax_format() {
        let error = parse_cli(vec![
            "syntax".to_string(),
            "--format".to_string(),
            "yaml".to_string(),
        ])
        .expect_err("syntax should reject unknown formats");
        assert_eq!(
            error,
            "unknown syntax format `yaml`; expected `json` or `textmate`"
        );
    }

    #[test]
    fn parses_version_command_without_inputs() {
        let options = parse_cli(vec!["version".to_string()]).expect("version command");
        assert_eq!(options.command, "version");
        assert!(options.inputs.is_empty());
        assert!(!options.show_timings);
        assert_eq!(options.version_format, VersionFormat::Human);
    }

    #[test]
    fn parses_version_json_format() {
        let options = parse_cli(vec![
            "version".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("version json command");
        assert_eq!(options.command, "version");
        assert_eq!(options.version_format, VersionFormat::Json);
    }

    #[test]
    fn rejects_unknown_version_format() {
        let error = parse_cli(vec![
            "version".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("version should reject unknown formats");
        assert_eq!(
            error,
            "unknown version format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_explain_command_with_code() {
        let options =
            parse_cli(vec!["explain".to_string(), "H0201".to_string()]).expect("explain command");
        assert_eq!(options.command, "explain");
        assert_eq!(options.explain_code.as_deref(), Some("H0201"));
        assert_eq!(options.explain_format, ExplainFormat::Human);
    }

    #[test]
    fn parses_explain_json_format() {
        let options = parse_cli(vec![
            "explain".to_string(),
            "H0201".to_string(),
            "--format=json".to_string(),
        ])
        .expect("explain json command");
        assert_eq!(options.command, "explain");
        assert_eq!(options.explain_format, ExplainFormat::Json);
    }

    #[test]
    fn rejects_unknown_explain_format() {
        let error = parse_cli(vec![
            "explain".to_string(),
            "H0201".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("explain should reject unknown formats");
        assert_eq!(
            error,
            "unknown explain format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn rejects_explain_without_exactly_one_code() {
        let missing = parse_cli(vec!["explain".to_string()]).expect_err("missing code");
        assert_eq!(missing, "`explain` requires exactly one diagnostic code");

        let extra = parse_cli(vec![
            "explain".to_string(),
            "H0201".to_string(),
            "H0501".to_string(),
        ])
        .expect_err("extra code");
        assert_eq!(extra, "`explain` requires exactly one diagnostic code");
    }

    #[test]
    fn parses_diagnostics_command_without_inputs() {
        let options = parse_cli(vec!["diagnostics".to_string()]).expect("diagnostics command");
        assert_eq!(options.command, "diagnostics");
        assert!(options.inputs.is_empty());
        assert!(!options.show_timings);
        assert_eq!(options.diagnostics_format, DiagnosticsFormat::Human);
    }

    #[test]
    fn parses_diagnostics_json_format() {
        let options = parse_cli(vec![
            "diagnostics".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("diagnostics json command");
        assert_eq!(options.command, "diagnostics");
        assert_eq!(options.diagnostics_format, DiagnosticsFormat::Json);
    }

    #[test]
    fn rejects_unknown_diagnostics_format() {
        let error = parse_cli(vec![
            "diagnostics".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("diagnostics should reject unknown formats");
        assert_eq!(
            error,
            "unknown diagnostics format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn rejects_diagnostics_command_inputs() {
        let error = parse_cli(vec!["diagnostics".to_string(), "examples".to_string()])
            .expect_err("diagnostics should reject inputs");
        assert_eq!(error, "`diagnostics` does not accept input files");
    }

    #[test]
    fn parses_capabilities_command_without_inputs() {
        let options = parse_cli(vec!["capabilities".to_string()]).expect("capabilities command");
        assert_eq!(options.command, "capabilities");
        assert!(options.inputs.is_empty());
        assert!(!options.show_timings);
        assert_eq!(options.capabilities_format, CapabilitiesFormat::Human);
    }

    #[test]
    fn parses_capabilities_json_format() {
        let options = parse_cli(vec![
            "capabilities".to_string(),
            "--format=json".to_string(),
        ])
        .expect("capabilities json command");
        assert_eq!(options.command, "capabilities");
        assert_eq!(options.capabilities_format, CapabilitiesFormat::Json);
    }

    #[test]
    fn rejects_unknown_capabilities_format() {
        let error = parse_cli(vec![
            "capabilities".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("capabilities should reject unknown formats");
        assert_eq!(
            error,
            "unknown capabilities format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn rejects_capabilities_command_inputs() {
        let error = parse_cli(vec!["capabilities".to_string(), "examples".to_string()])
            .expect_err("capabilities should reject inputs");
        assert_eq!(error, "`capabilities` does not accept input files");
    }

    #[test]
    fn rejects_version_command_inputs() {
        let error = parse_cli(vec!["version".to_string(), "examples".to_string()])
            .expect_err("version should reject inputs");
        assert_eq!(error, "`version` does not accept input files");
    }

    #[test]
    fn rejects_syntax_command_inputs() {
        let error = parse_cli(vec!["syntax".to_string(), "examples".to_string()])
            .expect_err("syntax should reject inputs");
        assert_eq!(error, "`syntax` does not accept input files");
    }

    #[test]
    fn reference_surface_fixture_stays_clean() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("reference_surface.hum");
        let loaded = load_program(&[fixture]).expect("load reference surface fixture");
        assert!(
            loaded.diagnostics.is_empty(),
            "reference surface diagnostics: {:#?}",
            loaded.diagnostics
        );
        assert_eq!(
            crate::test_skeletons::program_to_test_skeletons(&loaded.program),
            "",
            "reference surface should not have unlinked test obligations"
        );
    }
}
