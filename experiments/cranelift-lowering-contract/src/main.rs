use hum_cranelift_lowering_contract::{RepoPaths, attempt_real_lowering};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("contract probe failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "probe".to_string());
    if command != "probe" {
        return Err(format!("unknown command `{command}`; expected `probe`"));
    }
    let paths = RepoPaths::discover()?;
    let report = attempt_real_lowering(&paths)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report.to_json())
            .map_err(|error| format!("report serialization failed: {error}"))?
    );
    Ok(())
}
