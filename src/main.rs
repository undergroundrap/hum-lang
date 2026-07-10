#![forbid(unsafe_code)]

mod app_entry;
mod ast;
mod backend_contract;
mod capabilities;
mod check;
mod core_body;
mod core_contract;
mod core_expr;
mod core_lower;
mod core_preview;
mod core_verify;
mod diagnostic;
mod diagnostic_catalog;
mod diagnostics;
mod doctor;
mod effect_check;
mod element_place;
mod evidence;
mod explain;
mod field_place;
mod full_type_check;
mod graph;
mod ir_contract;
mod ir_readiness;
mod json;
mod lsp;
mod math_obligations;
mod node_id;
mod ownership_check;
mod parser;
mod profile_check;
mod resolve;
mod resource_check;
mod resource_report;
mod return_dependency;
mod run;
mod runtime_profiles;
mod state_model;
mod syntax;
mod target_facts;
mod test_skeletons;
mod type_check;
mod type_env;
mod typed_failure;
mod version;
mod writable_field_alias;

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
    if options.command == "profiles" {
        match options.runtime_profiles_format {
            RuntimeProfilesFormat::Human => print!("{}", runtime_profiles::runtime_profiles_text()),
            RuntimeProfilesFormat::Json => print!("{}", runtime_profiles::runtime_profiles_json()),
        }
        return Ok(ExitCode::SUCCESS);
    }
    if options.command == "state-model" {
        match options.state_model_format {
            StateModelFormat::Human => print!("{}", state_model::state_model_text()),
            StateModelFormat::Json => print!("{}", state_model::state_model_json()),
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
    let mut diagnostics = loaded.diagnostics;
    if options.command == "run" {
        diagnostics.retain(|diagnostic| !app_entry::is_app_entry_diagnostic(diagnostic));
        if options.run_entry.is_none()
            && !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            diagnostics.extend(app_entry::diagnostics(&program));
        }
    }
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);
    let app_mode = options.command == "run"
        && options.run_entry.is_none()
        && !has_errors
        && app_entry::analyze(&program).entry.is_some();

    if app_mode {
        if resolve::resolve_has_errors(&program, &diagnostics) {
            eprint!("{}", resolve::resolve_text(&program, &diagnostics));
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            return Ok(ExitCode::from(1));
        }
        if type_check::type_check_has_errors(&program, &diagnostics) {
            eprint!("{}", type_check::type_check_text(&program, &diagnostics));
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            return Ok(ExitCode::from(1));
        }
        if full_type_check::full_type_check_has_errors(&program, &diagnostics) {
            eprint!(
                "{}",
                full_type_check::full_type_check_text(&program, &diagnostics)
            );
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            return Ok(ExitCode::from(1));
        }
    }

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
        "run" => {
            if has_errors {
                print_diagnostics(&diagnostics);
                if options.show_timings {
                    print_timings(&loaded.timings, loaded.total);
                }
                return Ok(ExitCode::from(1));
            }

            let report =
                run::run_program(&program, options.run_entry.as_deref(), &options.run_args);
            print_diagnostics(&report.diagnostics);
            let code = match report.outcome {
                run::RunOutcome::Success(output) => {
                    println!("{output}");
                    ExitCode::SUCCESS
                }
                run::RunOutcome::AppSuccess => ExitCode::SUCCESS,
                run::RunOutcome::Failure(output) => {
                    println!("{output}");
                    ExitCode::from(1)
                }
                run::RunOutcome::AppFailure(output) => {
                    eprintln!("{output}");
                    ExitCode::from(1)
                }
                run::RunOutcome::ContractViolation => ExitCode::from(1),
                run::RunOutcome::Trap(message) => {
                    eprintln!("runtime trap: {message}");
                    ExitCode::from(2)
                }
            };
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(code)
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
        "core-lower" => {
            if options.core_lower_format == CoreLowerFormat::Human {
                print_diagnostics(&diagnostics);
            }
            match options.core_lower_format {
                CoreLowerFormat::Human => {
                    print!("{}", core_lower::core_lower_text(&program, &diagnostics))
                }
                CoreLowerFormat::Json => {
                    print!("{}", core_lower::core_lower_json(&program, &diagnostics))
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
        "core-verify" => {
            if options.core_verify_format == CoreVerifyFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_core_verify_errors =
                core_verify::core_verify_has_errors(&program, &diagnostics);
            match options.core_verify_format {
                CoreVerifyFormat::Human => {
                    print!("{}", core_verify::core_verify_text(&program, &diagnostics))
                }
                CoreVerifyFormat::Json => {
                    print!("{}", core_verify::core_verify_json(&program, &diagnostics))
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_core_verify_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "resolve" => {
            if options.resolve_format == ResolveFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_resolver_errors = resolve::resolve_has_errors(&program, &diagnostics);
            match options.resolve_format {
                ResolveFormat::Human => {
                    print!("{}", resolve::resolve_text(&program, &diagnostics))
                }
                ResolveFormat::Json => {
                    print!("{}", resolve::resolve_json(&program, &diagnostics))
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_resolver_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "type-env" => {
            if options.type_env_format == TypeEnvFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_type_env_errors = type_env::type_env_has_errors(&program, &diagnostics);
            match options.type_env_format {
                TypeEnvFormat::Human => {
                    print!("{}", type_env::type_env_text(&program, &diagnostics))
                }
                TypeEnvFormat::Json => {
                    print!("{}", type_env::type_env_json(&program, &diagnostics))
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_type_env_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "type-check" => {
            if options.type_check_format == TypeCheckFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_type_check_errors = type_check::type_check_has_errors(&program, &diagnostics);
            match options.type_check_format {
                TypeCheckFormat::Human => {
                    print!("{}", type_check::type_check_text(&program, &diagnostics))
                }
                TypeCheckFormat::Json => {
                    print!("{}", type_check::type_check_json(&program, &diagnostics))
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_type_check_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "full-type-check" => {
            if options.type_check_format == TypeCheckFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_full_type_check_errors =
                full_type_check::full_type_check_has_errors(&program, &diagnostics);
            match options.type_check_format {
                TypeCheckFormat::Human => print!(
                    "{}",
                    full_type_check::full_type_check_text(&program, &diagnostics)
                ),
                TypeCheckFormat::Json => print!(
                    "{}",
                    full_type_check::full_type_check_json(&program, &diagnostics)
                ),
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_full_type_check_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "effect-check" => {
            if options.type_check_format == TypeCheckFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_effect_check_errors =
                effect_check::effect_check_has_errors(&program, &diagnostics);
            match options.type_check_format {
                TypeCheckFormat::Human => {
                    print!(
                        "{}",
                        effect_check::effect_check_text(&program, &diagnostics)
                    )
                }
                TypeCheckFormat::Json => {
                    print!(
                        "{}",
                        effect_check::effect_check_json(&program, &diagnostics)
                    )
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_effect_check_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "ownership-check" => {
            if options.type_check_format == TypeCheckFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_ownership_check_errors =
                ownership_check::ownership_check_has_errors(&program, &diagnostics);
            match options.type_check_format {
                TypeCheckFormat::Human => {
                    print!(
                        "{}",
                        ownership_check::ownership_check_text(&program, &diagnostics)
                    )
                }
                TypeCheckFormat::Json => {
                    print!(
                        "{}",
                        ownership_check::ownership_check_json(&program, &diagnostics)
                    )
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_ownership_check_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "resource-check" => {
            if options.type_check_format == TypeCheckFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_resource_check_errors =
                resource_check::resource_check_has_errors(&program, &diagnostics);
            match options.type_check_format {
                TypeCheckFormat::Human => {
                    print!(
                        "{}",
                        resource_check::resource_check_text(&program, &diagnostics)
                    )
                }
                TypeCheckFormat::Json => {
                    print!(
                        "{}",
                        resource_check::resource_check_json(&program, &diagnostics)
                    )
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_resource_check_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            })
        }
        "profile-check" => {
            if options.type_check_format == TypeCheckFormat::Human {
                print_diagnostics(&diagnostics);
            }
            let has_profile_check_errors =
                profile_check::profile_check_has_errors(&program, &diagnostics);
            match options.type_check_format {
                TypeCheckFormat::Human => {
                    print!(
                        "{}",
                        profile_check::profile_check_text(&program, &diagnostics)
                    )
                }
                TypeCheckFormat::Json => {
                    print!(
                        "{}",
                        profile_check::profile_check_json(&program, &diagnostics)
                    )
                }
            }
            if options.show_timings {
                print_timings(&loaded.timings, loaded.total);
            }
            Ok(if has_errors || has_profile_check_errors {
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
            "unknown command `{other}`; expected `check`, `run`, `graph`, `evidence`, `math-obligations`, `resource-report`, `core-preview`, `core-lower`, `core-verify`, `resolve`, `type-env`, `type-check`, `full-type-check`, `effect-check`, `ownership-check`, `resource-check`, `profile-check`, `ir-readiness`, `test-skeletons`, `syntax`, `version`, `explain`, `diagnostics`, `capabilities`, `core-contract`, `ir-contract`, `backend-contract`, `profiles`, `state-model`, `lsp`, `doctor`, or `target-facts`"
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
enum CoreLowerFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CoreVerifyFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolveFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeEnvFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeCheckFormat {
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
enum RuntimeProfilesFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateModelFormat {
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
    run_entry: Option<String>,
    run_args: Vec<String>,
    show_timings: bool,
    check_format: CheckFormat,
    syntax_format: SyntaxFormat,
    version_format: VersionFormat,
    explain_format: ExplainFormat,
    diagnostics_format: DiagnosticsFormat,
    capabilities_format: CapabilitiesFormat,
    core_contract_format: CoreContractFormat,
    core_preview_format: CorePreviewFormat,
    core_lower_format: CoreLowerFormat,
    core_verify_format: CoreVerifyFormat,
    resolve_format: ResolveFormat,
    type_env_format: TypeEnvFormat,
    type_check_format: TypeCheckFormat,
    ir_contract_format: IrContractFormat,
    backend_contract_format: BackendContractFormat,
    runtime_profiles_format: RuntimeProfilesFormat,
    state_model_format: StateModelFormat,
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
            | "run"
            | "graph"
            | "test-skeletons"
            | "evidence"
            | "math-obligations"
            | "resource-report"
            | "core-preview"
            | "core-lower"
            | "core-verify"
            | "resolve"
            | "type-env"
            | "type-check"
            | "full-type-check"
            | "effect-check"
            | "ownership-check"
            | "resource-check"
            | "profile-check"
            | "ir-readiness"
            | "syntax"
            | "version"
            | "explain"
            | "diagnostics"
            | "capabilities"
            | "core-contract"
            | "ir-contract"
            | "backend-contract"
            | "profiles"
            | "state-model"
            | "lsp"
            | "doctor"
            | "target-facts"
    ) {
        return Err(format!(
            "unknown command `{command}`; expected `check`, `run`, `graph`, `evidence`, `math-obligations`, `resource-report`, `core-preview`, `core-lower`, `core-verify`, `resolve`, `type-env`, `type-check`, `full-type-check`, `effect-check`, `ownership-check`, `resource-check`, `profile-check`, `ir-readiness`, `test-skeletons`, `syntax`, `version`, `explain`, `diagnostics`, `capabilities`, `core-contract`, `ir-contract`, `backend-contract`, `profiles`, `state-model`, `lsp`, `doctor`, or `target-facts`"
        ));
    }

    let mut show_timings = false;
    let mut raw_inputs = Vec::new();
    let mut run_entry = None;
    let mut run_args = Vec::new();
    let mut check_format = CheckFormat::Human;
    let mut syntax_format = SyntaxFormat::Json;
    let mut version_format = VersionFormat::Human;
    let mut explain_format = ExplainFormat::Human;
    let mut diagnostics_format = DiagnosticsFormat::Human;
    let mut capabilities_format = CapabilitiesFormat::Human;
    let mut core_contract_format = CoreContractFormat::Human;
    let mut core_preview_format = CorePreviewFormat::Human;
    let mut core_lower_format = CoreLowerFormat::Human;
    let mut core_verify_format = CoreVerifyFormat::Human;
    let mut resolve_format = ResolveFormat::Human;
    let mut type_env_format = TypeEnvFormat::Human;
    let mut type_check_format = TypeCheckFormat::Human;
    let mut ir_contract_format = IrContractFormat::Human;
    let mut backend_contract_format = BackendContractFormat::Human;
    let mut runtime_profiles_format = RuntimeProfilesFormat::Human;
    let mut state_model_format = StateModelFormat::Human;
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
            "--entry" if command == "run" => {
                let Some(value) = args.next() else {
                    return Err("`run --entry` requires a task name".to_string());
                };
                run_entry = Some(value);
            }
            flag if command == "run" && flag.starts_with("--entry=") => {
                let value = flag.trim_start_matches("--entry=");
                if value.is_empty() {
                    return Err("`run --entry` requires a task name".to_string());
                }
                run_entry = Some(value.to_string());
            }
            "--args" if command == "run" => {
                run_args.extend(args);
                break;
            }
            flag if command == "run" && flag.starts_with("--args=") => {
                let value = flag.trim_start_matches("--args=");
                if !value.is_empty() {
                    run_args.push(value.to_string());
                }
                run_args.extend(args);
                break;
            }
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
                        | "profiles"
                        | "state-model"
                        | "lsp"
                        | "doctor"
                        | "target-facts"
                        | "evidence"
                        | "math-obligations"
                        | "resource-report"
                        | "core-preview"
                        | "core-lower"
                        | "core-verify"
                        | "resolve"
                        | "type-env"
                        | "type-check"
                        | "full-type-check"
                        | "effect-check"
                        | "ownership-check"
                        | "resource-check"
                        | "profile-check"
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
                    "profiles" => runtime_profiles_format = parse_runtime_profiles_format(&value)?,
                    "state-model" => state_model_format = parse_state_model_format(&value)?,
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
                    "core-lower" => core_lower_format = parse_core_lower_format(&value)?,
                    "core-verify" => core_verify_format = parse_core_verify_format(&value)?,
                    "resolve" => resolve_format = parse_resolve_format(&value)?,
                    "type-env" => type_env_format = parse_type_env_format(&value)?,
                    "type-check" => type_check_format = parse_type_check_format(&value)?,
                    "full-type-check" => type_check_format = parse_full_type_check_format(&value)?,
                    "effect-check" => type_check_format = parse_effect_check_format(&value)?,
                    "ownership-check" => type_check_format = parse_ownership_check_format(&value)?,
                    "resource-check" => type_check_format = parse_resource_check_format(&value)?,
                    "profile-check" => type_check_format = parse_profile_check_format(&value)?,
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
                    | "profiles"
                    | "state-model"
                    | "lsp"
                    | "doctor"
                    | "target-facts"
                    | "evidence"
                    | "math-obligations"
                    | "resource-report"
                    | "core-preview"
                    | "core-lower"
                    | "core-verify"
                    | "resolve"
                    | "type-env"
                    | "type-check"
                    | "full-type-check"
                    | "effect-check"
                    | "ownership-check"
                    | "resource-check"
                    | "profile-check"
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
                    "profiles" => runtime_profiles_format = parse_runtime_profiles_format(value)?,
                    "state-model" => state_model_format = parse_state_model_format(value)?,
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
                    "core-lower" => core_lower_format = parse_core_lower_format(value)?,
                    "core-verify" => core_verify_format = parse_core_verify_format(value)?,
                    "resolve" => resolve_format = parse_resolve_format(value)?,
                    "type-env" => type_env_format = parse_type_env_format(value)?,
                    "type-check" => type_check_format = parse_type_check_format(value)?,
                    "full-type-check" => type_check_format = parse_full_type_check_format(value)?,
                    "effect-check" => type_check_format = parse_effect_check_format(value)?,
                    "ownership-check" => type_check_format = parse_ownership_check_format(value)?,
                    "resource-check" => type_check_format = parse_resource_check_format(value)?,
                    "profile-check" => type_check_format = parse_profile_check_format(value)?,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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

    if command == "profiles" {
        if show_timings {
            return Err("`profiles` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`profiles` does not accept input files".to_string());
        }
        return Ok(CliOptions {
            command,
            inputs: Vec::new(),
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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

    if command == "state-model" {
        if show_timings {
            return Err("`state-model` does not support `--timings`".to_string());
        }
        if !raw_inputs.is_empty() {
            return Err("`state-model` does not accept input files".to_string());
        }
        return Ok(CliOptions {
            command,
            inputs: Vec::new(),
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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
            run_entry: None,
            run_args: Vec::new(),
            show_timings,
            check_format,
            syntax_format,
            version_format,
            explain_format,
            diagnostics_format,
            capabilities_format,
            core_contract_format,
            core_preview_format,
            core_lower_format,
            core_verify_format,
            resolve_format,
            type_env_format,
            type_check_format,
            ir_contract_format,
            backend_contract_format,
            runtime_profiles_format,
            state_model_format,
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

    let inputs = collect_inputs(&raw_inputs)?;
    if command == "run" {
        let input_is_file = raw_inputs.len() == 1 && PathBuf::from(&raw_inputs[0]).is_file();
        if !input_is_file || inputs.len() != 1 {
            return Err("`run` requires exactly one .hum file".to_string());
        }
    }

    Ok(CliOptions {
        command,
        inputs,
        run_entry,
        run_args,
        show_timings,
        check_format,
        syntax_format,
        version_format,
        explain_format,
        diagnostics_format,
        capabilities_format,
        core_contract_format,
        core_preview_format,
        core_lower_format,
        core_verify_format,
        resolve_format,
        type_env_format,
        type_check_format,
        ir_contract_format,
        backend_contract_format,
        runtime_profiles_format,
        state_model_format,
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

fn parse_core_lower_format(value: &str) -> Result<CoreLowerFormat, String> {
    match value {
        "human" => Ok(CoreLowerFormat::Human),
        "json" => Ok(CoreLowerFormat::Json),
        other => Err(format!(
            "unknown core-lower format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_core_verify_format(value: &str) -> Result<CoreVerifyFormat, String> {
    match value {
        "human" => Ok(CoreVerifyFormat::Human),
        "json" => Ok(CoreVerifyFormat::Json),
        other => Err(format!(
            "unknown core-verify format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_resolve_format(value: &str) -> Result<ResolveFormat, String> {
    match value {
        "human" => Ok(ResolveFormat::Human),
        "json" => Ok(ResolveFormat::Json),
        other => Err(format!(
            "unknown resolve format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_type_env_format(value: &str) -> Result<TypeEnvFormat, String> {
    match value {
        "human" => Ok(TypeEnvFormat::Human),
        "json" => Ok(TypeEnvFormat::Json),
        other => Err(format!(
            "unknown type-env format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_type_check_format(value: &str) -> Result<TypeCheckFormat, String> {
    match value {
        "human" => Ok(TypeCheckFormat::Human),
        "json" => Ok(TypeCheckFormat::Json),
        other => Err(format!(
            "unknown type-check format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_full_type_check_format(value: &str) -> Result<TypeCheckFormat, String> {
    match value {
        "human" => Ok(TypeCheckFormat::Human),
        "json" => Ok(TypeCheckFormat::Json),
        other => Err(format!(
            "unknown full-type-check format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_effect_check_format(value: &str) -> Result<TypeCheckFormat, String> {
    match value {
        "human" => Ok(TypeCheckFormat::Human),
        "json" => Ok(TypeCheckFormat::Json),
        other => Err(format!(
            "unknown effect-check format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_ownership_check_format(value: &str) -> Result<TypeCheckFormat, String> {
    match value {
        "human" => Ok(TypeCheckFormat::Human),
        "json" => Ok(TypeCheckFormat::Json),
        other => Err(format!(
            "unknown ownership-check format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_resource_check_format(value: &str) -> Result<TypeCheckFormat, String> {
    match value {
        "human" => Ok(TypeCheckFormat::Human),
        "json" => Ok(TypeCheckFormat::Json),
        other => Err(format!(
            "unknown resource-check format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_profile_check_format(value: &str) -> Result<TypeCheckFormat, String> {
    match value {
        "human" => Ok(TypeCheckFormat::Human),
        "json" => Ok(TypeCheckFormat::Json),
        other => Err(format!(
            "unknown profile-check format `{other}`; expected `human` or `json`"
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

fn parse_runtime_profiles_format(value: &str) -> Result<RuntimeProfilesFormat, String> {
    match value {
        "human" => Ok(RuntimeProfilesFormat::Human),
        "json" => Ok(RuntimeProfilesFormat::Json),
        other => Err(format!(
            "unknown profiles format `{other}`; expected `human` or `json`"
        )),
    }
}

fn parse_state_model_format(value: &str) -> Result<StateModelFormat, String> {
    match value {
        "human" => Ok(StateModelFormat::Human),
        "json" => Ok(StateModelFormat::Json),
        other => Err(format!(
            "unknown state-model format `{other}`; expected `human` or `json`"
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
        let mut file_diagnostics = check::check_file(&parsed.file);
        if !parsed
            .diagnostics
            .iter()
            .chain(&file_diagnostics)
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            file_diagnostics.extend(app_entry::diagnostics_for_file(&parsed.file));
        }
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
    println!("  hum run [--timings] <file> [--entry <task>] [--args ...]");
    println!("  hum graph [--timings] <file-or-dir>...");
    println!("  hum evidence [--format human|json] [--timings] <file-or-dir>...");
    println!(
        "  hum math-obligations [--format human|json] [--out-dir <dir>] [--timings] <file-or-dir>..."
    );
    println!("  hum resource-report [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum core-preview [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum core-lower [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum core-verify [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum resolve [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum type-env [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum type-check [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum full-type-check [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum effect-check [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum ownership-check [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum resource-check [--format human|json] [--timings] <file-or-dir>...");
    println!("  hum profile-check [--format human|json] [--timings] <file-or-dir>...");
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
    println!("  hum profiles [--format human|json]");
    println!("  hum state-model [--format human|json]");
    println!("  hum lsp --capabilities [--format human|json]");
    println!("  hum doctor [--format human|json]");
    println!("  hum target-facts [--format human|json]");
    println!();
    println!("Commands:");
    println!("  check           Parse Hum files and run milestone-0 intent checks");
    println!("  run             Interpret one checked Hum file in the first executable subset");
    println!("  graph           Emit hum.semantic_graph.v0 JSON for agents and tools");
    println!("  evidence          Summarize security and trust evidence obligations");
    println!("  math-obligations  Export V0 math obligations for external validators");
    println!("  resource-report   Classify source-declared resource and optimization claims");
    println!("  core-preview      Emit Core Hum preview candidates without execution");
    println!("  core-lower        Emit an unverified Core Hum artifact without execution or IR");
    println!("  core-verify       Verify non-executing Core Hum artifact invariants");
    println!("  resolve           Emit checked scopes, definitions, references, and place links");
    println!("  type-env          Emit declared type environment facts without type checking");
    println!("  type-check        Validate declaration annotations without expression inference");
    println!("  full-type-check   Check recognized Core/body statement types without execution");
    println!("  effect-check      Check recognized Core/body effects without execution");
    println!("  ownership-check   Check recognized Core/body ownership facts without execution");
    println!("  resource-check    Check declared allocation/resource intent without execution");
    println!("  profile-check     Check runtime profile policy declarations without execution");
    println!(
        "  ir-readiness      Report source readiness after profile checking, before Hum IR lowering"
    );
    println!("  test-skeletons    Print Hum test skeletons for unlinked obligations");
    println!("  syntax          Emit syntax JSON or generated TextMate grammar");
    println!("  version         Print toolchain identity and schema versions");
    println!("  explain         Explain a stable diagnostic code");
    println!("  diagnostics     List stable diagnostic codes");
    println!("  capabilities    List machine-readable tool and editor surfaces");
    println!("  core-contract   Emit the Core Hum executable subset contract");
    println!("  ir-contract     Emit the Hum IR ownership and preservation contract");
    println!("  backend-contract  Emit the backend adapter contract and staged backend ladder");
    println!("  profiles          Emit the runtime profile policy catalog");
    println!("  state-model       Emit the state and mutation permission model");
    println!("  lsp             Preview LSP adapter capabilities");
    println!("  doctor          Check portable repo setup and guardrails");
    println!("  target-facts    Emit target fact fields and portability fixtures");
    println!();
    println!("Options:");
    println!("  --timings   Print read/parse/check timings per input file");
    println!("  --version   Print toolchain identity");
    println!("  --format    Choose command output format where supported");
    println!("  --out-dir   Write one math obligation JSON file per obligation");
    println!("  --entry     Select the task used by `hum run`");
    println!("  --args      Pass all remaining values to `hum run`");
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        BackendContractFormat, CapabilitiesFormat, CheckFormat, CoreContractFormat,
        CoreLowerFormat, CorePreviewFormat, CoreVerifyFormat, DiagnosticsFormat, DoctorFormat,
        EvidenceFormat, ExplainFormat, IrContractFormat, IrReadinessFormat, LspFormat,
        MathObligationsFormat, ResolveFormat, ResourceReportFormat, RuntimeProfilesFormat,
        StateModelFormat, SyntaxFormat, TargetFactsFormat, TypeCheckFormat, TypeEnvFormat,
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
    fn parses_run_entry_and_args() {
        let options = parse_cli(vec![
            "run".to_string(),
            "examples/core/add.hum".to_string(),
            "--entry".to_string(),
            "add".to_string(),
            "--args".to_string(),
            "2".to_string(),
            "3".to_string(),
        ])
        .expect("run command");

        assert_eq!(options.command, "run");
        assert_eq!(options.inputs, vec![PathBuf::from("examples/core/add.hum")]);
        assert_eq!(options.run_entry.as_deref(), Some("add"));
        assert_eq!(options.run_args, vec!["2".to_string(), "3".to_string()]);
    }

    #[test]
    fn rejects_run_directory_input() {
        let error = parse_cli(vec!["run".to_string(), "examples".to_string()])
            .expect_err("run should require one file");
        assert_eq!(error, "`run` requires exactly one .hum file");
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
    fn parses_profiles_json_format() {
        let options = parse_cli(vec!["profiles".to_string(), "--format=json".to_string()])
            .expect("profiles json command");
        assert_eq!(options.command, "profiles");
        assert_eq!(options.runtime_profiles_format, RuntimeProfilesFormat::Json);
    }

    #[test]
    fn rejects_profiles_inputs() {
        let error = parse_cli(vec!["profiles".to_string(), "examples".to_string()])
            .expect_err("profiles should reject inputs");
        assert_eq!(error, "`profiles` does not accept input files");
    }

    #[test]
    fn rejects_unknown_profiles_format() {
        let error = parse_cli(vec![
            "profiles".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("profiles should reject unknown formats");
        assert_eq!(
            error,
            "unknown profiles format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_state_model_json_format() {
        let options = parse_cli(vec!["state-model".to_string(), "--format=json".to_string()])
            .expect("state-model json command");
        assert_eq!(options.command, "state-model");
        assert_eq!(options.state_model_format, StateModelFormat::Json);
    }

    #[test]
    fn rejects_state_model_inputs() {
        let error = parse_cli(vec!["state-model".to_string(), "examples".to_string()])
            .expect_err("state-model should reject inputs");
        assert_eq!(error, "`state-model` does not accept input files");
    }

    #[test]
    fn rejects_unknown_state_model_format() {
        let error = parse_cli(vec![
            "state-model".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
        ])
        .expect_err("state-model should reject unknown formats");
        assert_eq!(
            error,
            "unknown state-model format `textmate`; expected `human` or `json`"
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
    fn parses_core_lower_json_format() {
        let options = parse_cli(vec![
            "core-lower".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("core-lower json command");
        assert_eq!(options.command, "core-lower");
        assert_eq!(options.core_lower_format, CoreLowerFormat::Json);
    }

    #[test]
    fn rejects_unknown_core_lower_format() {
        let error = parse_cli(vec![
            "core-lower".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("core-lower should reject unknown formats");
        assert_eq!(
            error,
            "unknown core-lower format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_core_verify_json_format() {
        let options = parse_cli(vec![
            "core-verify".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("core-verify json command");
        assert_eq!(options.command, "core-verify");
        assert_eq!(options.core_verify_format, CoreVerifyFormat::Json);
    }

    #[test]
    fn rejects_unknown_core_verify_format() {
        let error = parse_cli(vec![
            "core-verify".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("core-verify should reject unknown formats");
        assert_eq!(
            error,
            "unknown core-verify format `textmate`; expected `human` or `json`"
        );
    }
    #[test]
    fn parses_resolve_json_format() {
        let options = parse_cli(vec![
            "resolve".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("resolve json command");
        assert_eq!(options.command, "resolve");
        assert_eq!(options.resolve_format, ResolveFormat::Json);
    }

    #[test]
    fn rejects_unknown_resolve_format() {
        let error = parse_cli(vec![
            "resolve".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("resolve should reject unknown formats");
        assert_eq!(
            error,
            "unknown resolve format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_type_env_json_format() {
        let options = parse_cli(vec![
            "type-env".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("type-env json command");
        assert_eq!(options.command, "type-env");
        assert_eq!(options.type_env_format, TypeEnvFormat::Json);
    }

    #[test]
    fn rejects_unknown_type_env_format() {
        let error = parse_cli(vec![
            "type-env".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("type-env should reject unknown formats");
        assert_eq!(
            error,
            "unknown type-env format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_type_check_json_format() {
        let options = parse_cli(vec![
            "type-check".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "examples".to_string(),
        ])
        .expect("type-check json command");
        assert_eq!(options.command, "type-check");
        assert_eq!(options.type_check_format, TypeCheckFormat::Json);
    }

    #[test]
    fn rejects_unknown_type_check_format() {
        let error = parse_cli(vec![
            "type-check".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("type-check should reject unknown formats");
        assert_eq!(
            error,
            "unknown type-check format `textmate`; expected `human` or `json`"
        );
    }
    #[test]
    fn parses_full_type_check_json_format() {
        let options = parse_cli(vec![
            "full-type-check".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("full-type-check json command");
        assert_eq!(options.command, "full-type-check");
        assert_eq!(options.type_check_format, TypeCheckFormat::Json);
    }

    #[test]
    fn rejects_unknown_full_type_check_format() {
        let error = parse_cli(vec![
            "full-type-check".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("full-type-check should reject unknown formats");
        assert_eq!(
            error,
            "unknown full-type-check format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_effect_check_json_format() {
        let options = parse_cli(vec![
            "effect-check".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("effect-check json command");
        assert_eq!(options.command, "effect-check");
        assert_eq!(options.type_check_format, TypeCheckFormat::Json);
    }

    #[test]
    fn rejects_unknown_effect_check_format() {
        let error = parse_cli(vec![
            "effect-check".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("effect-check should reject unknown formats");
        assert_eq!(
            error,
            "unknown effect-check format `textmate`; expected `human` or `json`"
        );
    }
    #[test]
    fn parses_ownership_check_json_format() {
        let options = parse_cli(vec![
            "ownership-check".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("ownership-check json command");
        assert_eq!(options.command, "ownership-check");
        assert_eq!(options.type_check_format, TypeCheckFormat::Json);
    }

    #[test]
    fn rejects_unknown_ownership_check_format() {
        let error = parse_cli(vec![
            "ownership-check".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("ownership-check should reject unknown formats");
        assert_eq!(
            error,
            "unknown ownership-check format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_resource_check_json_format() {
        let options = parse_cli(vec![
            "resource-check".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("resource-check json command");
        assert_eq!(options.command, "resource-check");
        assert_eq!(options.type_check_format, TypeCheckFormat::Json);
    }

    #[test]
    fn rejects_unknown_resource_check_format() {
        let error = parse_cli(vec![
            "resource-check".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("resource-check should reject unknown formats");
        assert_eq!(
            error,
            "unknown resource-check format `textmate`; expected `human` or `json`"
        );
    }

    #[test]
    fn parses_profile_check_json_format() {
        let options = parse_cli(vec![
            "profile-check".to_string(),
            "--format=json".to_string(),
            "examples".to_string(),
        ])
        .expect("profile-check json command");
        assert_eq!(options.command, "profile-check");
        assert_eq!(options.type_check_format, TypeCheckFormat::Json);
    }

    #[test]
    fn rejects_unknown_profile_check_format() {
        let error = parse_cli(vec![
            "profile-check".to_string(),
            "--format".to_string(),
            "textmate".to_string(),
            "examples".to_string(),
        ])
        .expect_err("profile-check should reject unknown formats");
        assert_eq!(
            error,
            "unknown profile-check format `textmate`; expected `human` or `json`"
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
