#![forbid(unsafe_code)]

mod ast;
mod check;
mod diagnostic;
mod graph;
mod json;
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
    if options.command == "version" {
        match options.version_format {
            VersionFormat::Human => print!("{}", version::version_text()),
            VersionFormat::Json => print!("{}", version::version_json()),
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
            "unknown command `{other}`; expected `check`, `graph`, `test-skeletons`, `syntax`, or `version`"
        )),
    }
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

#[derive(Debug)]
struct CliOptions {
    command: String,
    inputs: Vec<PathBuf>,
    show_timings: bool,
    syntax_format: SyntaxFormat,
    version_format: VersionFormat,
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
        "check" | "graph" | "test-skeletons" | "syntax" | "version"
    ) {
        return Err(format!(
            "unknown command `{command}`; expected `check`, `graph`, `test-skeletons`, `syntax`, or `version`"
        ));
    }

    let mut show_timings = false;
    let mut raw_inputs = Vec::new();
    let mut syntax_format = SyntaxFormat::Json;
    let mut version_format = VersionFormat::Human;
    let mut args = args.into_iter().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--timings" => show_timings = true,
            "--format" if command == "syntax" || command == "version" => {
                let Some(value) = args.next() else {
                    return Err(format!("`{command} --format` requires a format value"));
                };
                if command == "syntax" {
                    syntax_format = parse_syntax_format(&value)?;
                } else {
                    version_format = parse_version_format(&value)?;
                }
            }
            flag if (command == "syntax" || command == "version")
                && flag.starts_with("--format=") =>
            {
                let value = flag.trim_start_matches("--format=");
                if command == "syntax" {
                    syntax_format = parse_syntax_format(value)?;
                } else {
                    version_format = parse_version_format(value)?;
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
            syntax_format,
            version_format,
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
            syntax_format,
            version_format,
        });
    }

    Ok(CliOptions {
        command,
        inputs: collect_inputs(&raw_inputs)?,
        show_timings,
        syntax_format,
        version_format,
    })
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
    println!("  hum check [--timings] <file-or-dir>...");
    println!("  hum graph [--timings] <file-or-dir>...");
    println!("  hum test-skeletons [--timings] <file-or-dir>...");
    println!("  hum syntax [--format json|textmate]");
    println!("  hum version [--format human|json]");
    println!();
    println!("Commands:");
    println!("  check   Parse Hum files and run milestone-0 intent checks");
    println!("  graph           Emit hum.semantic_graph.v0 JSON for agents and tools");
    println!("  test-skeletons  Print Hum test skeletons for unlinked obligations");
    println!("  syntax          Emit syntax JSON or generated TextMate grammar");
    println!("  version         Print toolchain identity and schema versions");
    println!();
    println!("Options:");
    println!("  --timings   Print read/parse/check timings per input file");
    println!("  --version   Print toolchain identity");
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{SyntaxFormat, VersionFormat, load_program, parse_cli};

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
