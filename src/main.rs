#![forbid(unsafe_code)]

mod ast;
mod backend_contract;
mod capabilities;
mod check;
mod core_body;
mod core_contract;
mod core_expr;
mod core_preview;
mod diagnostic;
mod diagnostic_catalog;
mod diagnostics;
mod doctor;
mod evidence;
mod explain;
mod graph;
mod ir_contract;
mod ir_readiness;
mod json;
mod lsp;
mod math_obligations;
mod node_id;
mod parser;
mod resource_report;
mod syntax;
mod target_facts;
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
    if options.command == "core-contract" {
        match options.core_contract_format {
            CoreContractFormat::Human => print!("{}", core_contract::core_contract_text()),
            CoreContractFormat::Json => print!("{}", core_contract::core_contract_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "ir-contract" {
        match options.ir_contract_format {
            IrContractFormat::Human => print!("{}", ir_contract::ir_contract_text()),
            IrContractFormat::Json => print!("{}", ir_contract::ir_contract_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "backend-contract" {
        match options.backend_contract_format {
            BackendContractFormat::Human => print!("{}", backend_contract::backend_contract_text()),
            BackendContractFormat::Json => print!("{}", backend_contract::backend_contract_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "lsp" {
        match options.lsp_format {
            LspFormat::Human => print!("{}", lsp::lsp_capabilities_text()),
            LspFormat::Json => print!("{}", lsp::lsp_capabilities_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "doctor" {
        match options.doctor_format {
            DoctorFormat::Human => print!("{}", doctor::doctor_text()),
            DoctorFormat::Json => print!("{}", doctor::doctor_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "target-facts" {
        match options.target_facts_format {
            TargetFactsFormat::Human => print!("{}", target_facts::target_facts_text()),
            TargetFactsFormat::Json => print!("{}", target_facts::target_facts_json()),
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
        "evidence" => {
            if options.evidence_format == EvidenceFormat::Human {
                print_diagnostics(&diagnostics);
            }
            match options.evidence_format {
                EvidenceFormat::Human => {
                    print!("{}", evidence::evidence_text(&program, &diagnostics))
                }
                EvidenceFormat::Json => {
                    print!("{}", evidence::evidence_json(&program, &diagnostics))
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
        "math-obligations" => {
            if options.math_obligations_format == MathObligationsFormat::Human {
                print_diagnostics(&diagnostics);
            }
            if let Some(out_dir) = &options.math_obligations_out_dir {
                if has_errors {
                    eprintln!(
                        "not writing math obligation files because source diagnostics include errors"
                    );
                } else {
                    let written = write_math_obligation_files(&program, out_dir)?;
                    eprintln!(
                        "wrote {written} math obligation file(s) to {}",
                        out_dir.display()
                    );
                }
            }
            match options.math_obligations_format {
                MathObligationsFormat::Human => print!(
                    "{}",
                    math_obligations::math_obligations_text(&program, &diagnostics)
                ),
                MathObligationsFormat::Json => print!(
                    "{}",
                    math_obligations::math_obligations_json(&program, &diagnostics)
                ),
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
        "resource-report" => {
            if options.resource_report_format == ResourceReportFormat::Human {
                print_diagnostics(&diagnostics);
            }
            match options.resource_report_format {
                ResourceReportFormat::Human => print!(
                    "{}",
                    resource_report::resource_report_text(&program, &diagnostics)
                ),
                ResourceReportFormat::Json => print!(
                    "{}",
                    resource_report::resource_report_json(&program, &diagnostics)
                ),
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
        "core-preview" => {
            if options.core_preview_format == CorePreviewFormat::Human {
                print_diagnostics(&diagnostics);
            }
            match options.core_preview_format {
                CorePreviewFormat::Human => {
                    print!(
                        "{}",
                        core_preview::core_preview_text(&program, &diagnostics)
                    )
                }
                CorePreviewFormat::Json => {
                    print!(
                        "{}",
                        core_preview::core_preview_json(&program, &diagnostics)
                    )
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
        "ir-readiness" => {
            if options.ir_readiness_format == IrReadinessFormat::Human {
                print_diagnostics(&diagnostics);
            }
            match options.ir_readiness_format {
                IrReadinessFormat::Human => print!(
                    "{}",
                    ir_readiness::ir_readiness_text(&program, &diagnostics)
                ),
                IrReadinessFormat::Json => print!(
                    "{}",
                    ir_readiness::ir_readiness_json(&program, &diagnostics)
                ),
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
            "unknown command `{other}`; expected `check`, `graph`, `evidence`, `math-obligations`, `resource-report`, `core-preview`, `ir-readiness`, `test-skeletons`, `syntax`, `version`, `explain`, `diagnostics`, `capabilities`, `core-contract`, `ir-contract`, `backend-contract`, `lsp`, `doctor`, or `target-facts`"
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CoreContractFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CorePreviewFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IrContractFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendContractFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LspFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetFactsFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathObligationsFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceReportFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IrReadinessFormat {
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
    core_contract_format: CoreContractFormat,
    core_preview_format: CorePreviewFormat,
    ir_contract_format: IrContractFormat,
    backend_contract_format: BackendContractFormat,
    lsp_format: LspFormat,
    doctor_format: DoctorFormat,
    target_facts_format: TargetFactsFormat,
    evidence_format: EvidenceFormat,
    math_obligations_format: MathObligationsFormat,
    resource_report_format: ResourceReportFormat,
    ir_readiness_format: IrReadinessFormat,
    math_obligations_out_dir: Option<PathBuf>,
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
            | "evidence"
            | "math-obligations"
            | "resource-report"
            | "core-preview"
            | "ir-readiness"
            | "syntax"
            | "version"
            | "explain"
            | "diagnostics"
            | "capabilities"
            | "core-contract"
            | "ir-contract"
            | "backend-contract"
            | "lsp"
            | "doctor"
            | "target-facts"
    ) {
        return Err(format!(
            "unknown command `{command}`; expected `check`, `graph`, `evidence`, `math-obligations`, `resource-report`, `core-preview`, `ir-readiness`, `test-skeletons`, `syntax`, `version`, `explain`, `diagnostics`, `capabilities`, `core-contract`, `ir-contract`, `backend-contract`, `lsp`, `doctor`, or `target-facts`"
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
    let mut core_contract_format = CoreContractFormat::Human;
    let mut core_preview_format = CorePreviewFormat::Human;
    let mut ir_contract_format = IrContractFormat::Human;
    let mut backend_contract_format = BackendContractFormat::Human;
    let mut lsp_format = LspFormat::Human;
    let mut doctor_format = DoctorFormat::Human;
    let mut target_facts_format = TargetFactsFormat::Human;
    let mut evidence_format = EvidenceFormat::Human;
    let mut math_obligations_format = MathObligationsFormat::Human;
    let mut resource_report_format = ResourceReportFormat::Human;
    let mut ir_readiness_format = IrReadinessFormat::Human;
    let mut math_obligations_out_dir = None;
    let mut lsp_show_capabilities = false;
    let mut args = args.into_iter().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--timings" => show_timings = true,
            "--capabilities" if command == "lsp" => lsp_show_capabilities = true,
            "--out-dir" if command == "math-obligations" => {
                let Some(value) = args.next() else {
                    return Err("`math-obligations --out-dir` requires a directory".to_string());
                };
                math_obligations_out_dir = Some(PathBuf::from(value));
            }
            flag if command == "math-obligations" && flag.starts_with("--out-dir=") => {
                let value = flag.trim_start_matches("--out-dir=");
                if value.is_empty() {
                    return Err("`math-obligations --out-dir` requires a directory".to_string());
                }
                math_obligations_out_dir = Some(PathBuf::from(value));
            }
            "--format"
                if matches!(
                    command.as_str(),
                    "check"
                        | "syntax"
                        | "version"
                        | "explain"
                        | "diagnostics"
                        | "capabilities"
                        | "core-contract"
                        | "ir-contract"
                        | "backend-contract"
                        | "lsp"
                        | "doctor"
                        | "target-facts"
                        | "evidence"
                        | "math-obligations"
                        | "resource-report"
                        | "core-preview"
                        | "ir-readiness"
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
                    "core-contract" => core_contract_format = parse_core_contract_format(&value)?,
                    "ir-contract" => ir_contract_format = parse_ir_contract_format(&value)?,
                    "backend-contract" => {
                        backend_contract_format = parse_backend_contract_format(&value)?
                    }
                    "lsp" => lsp_format = parse_lsp_format(&value)?,
                    "doctor" => doctor_format = parse_doctor_format(&value)?,
                    "target-facts" => target_facts_format = parse_target_facts_format(&value)?,
                    "evidence" => evidence_format = parse_evidence_format(&value)?,
                    "math-obligations" => {
                        math_obligations_format = parse_math_obligations_format(&value)?
                    }
                    "resource-report" => {
                        resource_report_format = parse_resource_report_format(&value)?
                    }
                    "core-preview" => core_preview_format = parse_core_preview_format(&value)?,
                    "ir-readiness" => ir_readiness_format = parse_ir_readiness_format(&value)?,
                    _ => unreachable!(),
                }
            }
            flag if matches!(
                command.as_str(),
                "check"
                    | "syntax"
                    | "version"
                    | "explain"
                    | "diagnostics"
                    | "capabilities"
                    | "core-contract"
                    | "ir-contract"
                    | "backend-contract"
                    | "lsp"
                    | "doctor"
                    | "target-facts"
                    | "evidence"
                    | "math-obligations"
                    | "resource-report"
                    | "core-preview"
                    | "ir-readiness"
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
                    "core-contract" => core_contract_format = parse_core_contract_format(value)?,
                    "ir-contract" => ir_contract_format = parse_ir_contract_format(value)?,
                    "backend-contract" => {
                        backend_contract_format = parse_backend_contract_format(value)?
                    }
                    "lsp" => lsp_format = parse_lsp_format(value)?,
                    "doctor" => doctor_format = parse_doctor_format(value)?,
                    "target-facts" => target_facts_format = parse_target_facts_format(value)?,
                    "evidence" => evidence_format = parse_evidence_format(value)?,
                    "math-obligations" => {
                        math_obligations_format = parse_math_obligations_format(value)?
                    }
                    "resource-report" => {
                        resource_report_format = parse_resource_report_format(value)?
                    }
                    "core-preview" => core_preview_format = parse_core_preview_format(value)?,
                    "ir-readiness" => ir_readiness_format = parse_ir_readiness_format(value)?,
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
            explain_code: None,
        });
    }

    if command == "core-contract" {
        if show_timings {
            return Err("`core-contract` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`core-contract` does not accept input files".to_string());
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
            explain_code: None,
        });
    }

    if command == "ir-contract" {
        if show_timings {
            return Err("`ir-contract` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`ir-contract` does not accept input files".to_string());
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
            explain_code: None,
        });
    }

    if command == "backend-contract" {
        if show_timings {
            return Err("`backend-contract` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`backend-contract` does not accept input files".to_string());
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
            explain_code: None,
        });
    }

    if command == "lsp" {
        if show_timings {
            return Err("`lsp` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`lsp` does not accept input files".to_string());
        }
        if !lsp_show_capabilities {
            return Err(
                "`lsp` server mode is not implemented yet; use `hum lsp --capabilities`"
                    .to_string(),
            );
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
            explain_code: None,
        });
    }

    if command == "doctor" {
        if show_timings {
            return Err("`doctor` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`doctor` does not accept input files".to_string());
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
            explain_code: None,
        });
    }

    if command == "target-facts" {
        if show_timings {
            return Err("`target-facts` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`target-facts` does not accept input files".to_string());
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
            core_contract_format,
            core_preview_format,
            ir_contract_format,
            backend_contract_format,
            lsp_format,
            doctor_format,
            target_facts_format,
            evidence_format,
            math_obligations_format,
            resource_report_format,
            ir_readiness_format,
            math_obligations_out_dir: math_obligations_out_dir.clone(),
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
        core_contract_format,
        core_preview_format,
        ir_contract_format,
        backend_contract_format,
        lsp_format,
        doctor_format,
        target_facts_format,
        evidence_format,
        math_obligations_format,
        resource_report_format,
        ir_readiness_format,
        math_obligations_out_dir,
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

fn parse_core_contract_format(value: &str) -> Result<CoreContractFormat, String> {
    match value {
        "human" => Ok(CoreContractFormat::Human),
        "json" => Ok(CoreContractFormat::Json),
        other => Err(format!(
            "unknown core-contract format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_core_preview_format(value: &str) -> Result<CorePreviewFormat, String> {
    match value {
        "human" => Ok(CorePreviewFormat::Human),
        "json" => Ok(CorePreviewFormat::Json),
        other => Err(format!(
            "unknown core-preview format `{other}`; expected `human` or `json`"
        )),
    }
}
fn parse_ir_contract_format(value: &str) -> Result<IrContractFormat, String> {
    match value {
        "human" => Ok(IrContractFormat::Human),
        "json" => Ok(IrContractFormat::Json),
        other => Err(format!(
            "unknown ir-contract format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_backend_contract_format(value: &str) -> Result<BackendContractFormat, String> {
    match value {
        "human" => Ok(BackendContractFormat::Human),
        "json" => Ok(BackendContractFormat::Json),
        other => Err(format!(
            "unknown backend-contract format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_lsp_format(value: &str) -> Result<LspFormat, String> {
    match value {
        "human" => Ok(LspFormat::Human),
        "json" => Ok(LspFormat::Json),
        other => Err(format!(
            "unknown lsp format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_doctor_format(value: &str) -> Result<DoctorFormat, String> {
    match value {
        "human" => Ok(DoctorFormat::Human),
        "json" => Ok(DoctorFormat::Json),
        other => Err(format!(
            "unknown doctor format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_target_facts_format(value: &str) -> Result<TargetFactsFormat, String> {
    match value {
        "human" => Ok(TargetFactsFormat::Human),
        "json" => Ok(TargetFactsFormat::Json),
        other => Err(format!(
            "unknown target-facts format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_evidence_format(value: &str) -> Result<EvidenceFormat, String> {
    match value {
        "human" => Ok(EvidenceFormat::Human),
        "json" => Ok(EvidenceFormat::Json),
        other => Err(format!(
            "unknown evidence format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_math_obligations_format(value: &str) -> Result<MathObligationsFormat, String> {
    match value {
        "human" => Ok(MathObligationsFormat::Human),
        "json" => Ok(MathObligationsFormat::Json),
        other => Err(format!(
            "unknown math-obligations format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_resource_report_format(value: &str) -> Result<ResourceReportFormat, String> {
    match value {
        "human" => Ok(ResourceReportFormat::Human),
        "json" => Ok(ResourceReportFormat::Json),
        other => Err(format!(
            "unknown resource-report format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_ir_readiness_format(value: &str) -> Result<IrReadinessFormat, String> {
    match value {
        "human" => Ok(IrReadinessFormat::Human),
        "json" => Ok(IrReadinessFormat::Json),
        other => Err(format!(
            "unknown ir-readiness format `{other}`; expected `human` or `json`"
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

fn write_math_obligation_files(program: &Program, out_dir: &Path) -> Result<usize, String> {
    fs::create_dir_all(out_dir).map_err(|error| {
        format!(
            "failed to create math obligations directory `{}`: {error}",
            out_dir.display()
        )
    })?;

    let files = math_obligations::obligation_files(program);
    for file in &files {
        let path = out_dir.join(&file.file_name);
        fs::write(&path, file.json.as_bytes()).map_err(|error| {
            format!(
                "failed to write math obligation `{}`: {error}",
                path.display()
            )
        })?;
    }

    Ok(files.len())
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
    println!("  hum evidence [--format human|json] [--timings] <file-or-dir>...");
    println!(
        "  hum math-obligations [--format human|json] [--out-dir <dir>] [--timings] <file-or-dir>..."
    );
    println!("  hum resource-report [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum core-preview [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum ir-readiness [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum test-skeletons [--timings] <file-or-dir>...");
    println!("  hum syntax [--format json|textmate]");
    println!("  hum version [--format human|json]");
    println!("  hum explain <H####> [--format human|json]");
    println!("  hum diagnostics [--format human|json]");
    println!("  hum capabilities [--format human|json]");
    println!("  hum core-contract [--format human|json]");
    println!("  hum ir-contract [--format human|json]");
    println!("  hum backend-contract [--format human|json]");
    println!("  hum lsp --capabilities [--format human|json]");
    println!("  hum doctor [--format human|json]");
    println!("  hum target-facts [--format human|json]");
    println!();
    println!("Commands:");
    println!("  check           Parse Hum files and run milestone-0 intent checks");
    println!("  graph           Emit hum.semantic_graph.v0 JSON for agents and tools");
    println!("  evidence          Summarize security and trust evidence obligations");
    println!("  math-obligations  Export V0 math obligations for external validators");
    println!("  resource-report   Classify source-declared resource and optimization claims");
    println!("  core-preview      Emit Core Hum preview candidates without execution");
    println!("  ir-readiness      Report source readiness for future Core Hum and Hum IR lowering");
    println!("  test-skeletons    Print Hum test skeletons for unlinked obligations");
    println!("  syntax          Emit syntax JSON or generated TextMate grammar");
    println!("  version         Print toolchain identity and schema versions");
    println!("  explain         Explain a stable diagnostic code");
    println!("  diagnostics     List stable diagnostic codes");
    println!("  capabilities    List machine-readable tool and editor surfaces");
    println!("  core-contract   Emit the Core Hum executable subset contract");
    println!("  ir-contract     Emit the Hum IR ownership and preservation contract");
    println!("  backend-contract  Emit the backend adapter contract and staged backend ladder");
    println!("  lsp             Preview LSP adapter capabilities");
    println!("  doctor          Check portable repo setup and guardrails");
    println!("  target-facts    Emit target fact fields and portability fixtures");
    println!();
    println!("Options:");
    println!("  --timings   Print read/parse/check timings per input file");
    println!("  --version   Print toolchain identity");
    println!("  --format    Choose command output format where supported");
    println!("  --out-dir   Write one math obligation JSON file per obligation");
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        BackendContractFormat, CapabilitiesFormat, CheckFormat, CoreContractFormat,
        CorePreviewFormat, DiagnosticsFormat, DoctorFormat, EvidenceFormat, ExplainFormat,
        IrContractFormat, IrReadinessFormat, LspFormat, MathObligationsFormat,
        ResourceReportFormat, SyntaxFormat, TargetFactsFormat, VersionFormat, load_program,
        parse_cli,
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
    fn parses_core_contract_json_format() {
        let options = parse_cli(vec![
            "core-contract".to_string(),
            "--format=json".to_string(),
        ])
        .expect("core-contract json command");
        assert_eq!(options.command, "core-contract");
        assert_eq!(options.core_contract_format, CoreContractFormat::Json);
    }

    #[test]
    fn rejects_core_contract_inputs() {
        let error = parse_cli(vec!["core-contract".to_string(), "examples".to_string()])
            .expect_err("core-contract should reject inputs");
        assert_eq!(error, "`core-contract` does not accept input files");
    }

    #[test]
    fn rejects_unknown_core_contract_format() {
        let error = parse_cli(vec![
            "core-contract".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("core-contract should reject unknown formats");
        assert_eq!(
            error,
            "unknown core-contract format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_ir_contract_json_format() {
        let options = parse_cli(vec!["ir-contract".to_string(), "--format=json".to_string()])
            .expect("ir-contract json command");
        assert_eq!(options.command, "ir-contract");
        assert_eq!(options.ir_contract_format, IrContractFormat::Json);
    }

    #[test]
    fn rejects_ir_contract_inputs() {
        let error = parse_cli(vec!["ir-contract".to_string(), "examples".to_string()])
            .expect_err("ir-contract should reject inputs");
        assert_eq!(error, "`ir-contract` does not accept input files");
    }

    #[test]
    fn rejects_unknown_ir_contract_format() {
        let error = parse_cli(vec![
            "ir-contract".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("ir-contract should reject unknown formats");
        assert_eq!(
            error,
            "unknown ir-contract format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_backend_contract_json_format() {
        let options = parse_cli(vec![
            "backend-contract".to_string(),
            "--format=json".to_string(),
        ])
        .expect("backend-contract json command");
        assert_eq!(options.command, "backend-contract");
        assert_eq!(options.backend_contract_format, BackendContractFormat::Json);
    }

    #[test]
    fn rejects_backend_contract_inputs() {
        let error = parse_cli(vec!["backend-contract".to_string(), "examples".to_string()])
            .expect_err("backend-contract should reject inputs");
        assert_eq!(error, "`backend-contract` does not accept input files");
    }

    #[test]
    fn rejects_unknown_backend_contract_format() {
        let error = parse_cli(vec![
            "backend-contract".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("backend-contract should reject unknown formats");
        assert_eq!(
            error,
            "unknown backend-contract format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_lsp_capabilities_command() {
        let options = parse_cli(vec!["lsp".to_string(), "--capabilities".to_string()])
            .expect("lsp capabilities command");
        assert_eq!(options.command, "lsp");
        assert_eq!(options.lsp_format, LspFormat::Human);
    }

    #[test]
    fn parses_lsp_capabilities_json_format() {
        let options = parse_cli(vec![
            "lsp".to_string(),
            "--capabilities".to_string(),
            "--format=json".to_string(),
        ])
        .expect("lsp capabilities json command");
        assert_eq!(options.command, "lsp");
        assert_eq!(options.lsp_format, LspFormat::Json);
    }

    #[test]
    fn rejects_lsp_without_capabilities_flag() {
        let error = parse_cli(vec!["lsp".to_string()]).expect_err("lsp should require mode");
        assert_eq!(
            error,
            "`lsp` server mode is not implemented yet; use `hum lsp --capabilities`"
        );
    }

    #[test]
    fn rejects_unknown_lsp_format() {
        let error = parse_cli(vec![
            "lsp".to_string(),
            "--capabilities".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("lsp should reject unknown formats");
        assert_eq!(
            error,
            "unknown lsp format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn rejects_lsp_command_inputs() {
        let error = parse_cli(vec![
            "lsp".to_string(),
            "--capabilities".to_string(),
            "examples".to_string(),
        ])
        .expect_err("lsp should reject inputs");
        assert_eq!(error, "`lsp` does not accept input files");
    }

    #[test]
    fn parses_evidence_json_format() {
        let options = parse_cli(vec![
            "evidence".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("evidence json command");
        assert_eq!(options.command, "evidence");
        assert_eq!(options.evidence_format, EvidenceFormat::Json);
    }

    #[test]
    fn rejects_unknown_evidence_format() {
        let error = parse_cli(vec![
            "evidence".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("evidence should reject unknown formats");
        assert_eq!(
            error,
            "unknown evidence format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_math_obligations_json_format_and_out_dir() {
        let options = parse_cli(vec![
            "math-obligations".to_string(),
            "--format=json".to_string(),
            "--out-dir".to_string(),
            "target/math-obligations".to_string(),
            "examples".to_string(),
        ])
        .expect("math-obligations json command");
        assert_eq!(options.command, "math-obligations");
        assert_eq!(options.math_obligations_format, MathObligationsFormat::Json);
        assert_eq!(
            options.math_obligations_out_dir.as_deref(),
            Some(std::path::Path::new("target/math-obligations"))
        );
    }

    #[test]
    fn rejects_unknown_math_obligations_format() {
        let error = parse_cli(vec![
            "math-obligations".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("math-obligations should reject unknown formats");
        assert_eq!(
            error,
            "unknown math-obligations format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_resource_report_json_format() {
        let options = parse_cli(vec![
            "resource-report".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("resource-report json command");
        assert_eq!(options.command, "resource-report");
        assert_eq!(options.resource_report_format, ResourceReportFormat::Json);
    }

    #[test]
    fn parses_core_preview_json_format() {
        let options = parse_cli(vec![
            "core-preview".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("core-preview json command");
        assert_eq!(options.command, "core-preview");
        assert_eq!(options.core_preview_format, CorePreviewFormat::Json);
    }

    #[test]
    fn rejects_unknown_core_preview_format() {
        let error = parse_cli(vec![
            "core-preview".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("core-preview should reject unknown formats");
        assert_eq!(
            error,
            "unknown core-preview format `textmate`; expected `human` or `json`"
        );
    }
    #[test]
    fn parses_ir_readiness_json_format() {
        let options = parse_cli(vec![
            "ir-readiness".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("ir-readiness json command");
        assert_eq!(options.command, "ir-readiness");
        assert_eq!(options.ir_readiness_format, IrReadinessFormat::Json);
    }

    #[test]
    fn rejects_unknown_resource_report_format() {
        let error = parse_cli(vec![
            "resource-report".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("resource-report should reject unknown formats");
        assert_eq!(
            error,
            "unknown resource-report format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn rejects_unknown_ir_readiness_format() {
        let error = parse_cli(vec![
            "ir-readiness".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("ir-readiness should reject unknown formats");
        assert_eq!(
            error,
            "unknown ir-readiness format `textmate`; expected `human` or `json`"
        );
    }
    #[test]
    fn parses_doctor_command_without_inputs() {
        let options = parse_cli(vec!["doctor".to_string()]).expect("doctor command");
        assert_eq!(options.command, "doctor");
        assert!(options.inputs.is_empty());
        assert!(!options.show_timings);
        assert_eq!(options.doctor_format, DoctorFormat::Human);
    }

    #[test]
    fn parses_doctor_json_format() {
        let options = parse_cli(vec!["doctor".to_string(), "--format=json".to_string()])
            .expect("doctor json command");
        assert_eq!(options.command, "doctor");
        assert_eq!(options.doctor_format, DoctorFormat::Json);
    }

    #[test]
    fn rejects_unknown_doctor_format() {
        let error = parse_cli(vec![
            "doctor".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("doctor should reject unknown formats");
        assert_eq!(
            error,
            "unknown doctor format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn rejects_doctor_command_inputs() {
        let error = parse_cli(vec!["doctor".to_string(), "examples".to_string()])
            .expect_err("doctor should reject inputs");
        assert_eq!(error, "`doctor` does not accept input files");
    }

    #[test]
    fn parses_target_facts_json_format() {
        let options = parse_cli(vec![
            "target-facts".to_string(),
            "--format=json".to_string(),
        ])
        .expect("target-facts json command");
        assert_eq!(options.command, "target-facts");
        assert_eq!(options.target_facts_format, TargetFactsFormat::Json);
    }

    #[test]
    fn rejects_unknown_target_facts_format() {
        let error = parse_cli(vec![
            "target-facts".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("target-facts should reject unknown formats");
        assert_eq!(
            error,
            "unknown target-facts format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn rejects_target_facts_command_inputs() {
        let error = parse_cli(vec!["target-facts".to_string(), "examples".to_string()])
            .expect_err("target-facts should reject inputs");
        assert_eq!(error, "`target-facts` does not accept input files");
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
