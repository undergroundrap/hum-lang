use hum_cranelift_feasibility::{
    RepoPaths, assess_hum_add, build_jit_checked_add, collect_hum_evidence,
    emit_checked_add_object, link_and_run_native, operator_from_core,
    require_capability_probe_operator, run_hum_add,
};
use std::process::ExitCode;
use std::time::Instant;

const CASES: &[(i64, i64)] = &[(2, 3), (-7, 11), (0, 0), (1_000_000, 24)];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("proof failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let command = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "proof".to_string());
    if command != "proof" {
        return Err(format!("unknown command `{command}`; expected `proof`"));
    }
    let total = Instant::now();
    let paths = RepoPaths::discover()?;
    println!("cranelift_codegen_version={}", cranelift_codegen::VERSION);
    println!("cranelift_jit_version={}", cranelift_jit::VERSION);
    println!("cranelift_object_version={}", cranelift_object::VERSION);
    let hum = collect_hum_evidence(&paths, &paths.minimal_add)?;
    require_capability_probe_operator(&hum.core_lower, "add")?;
    let blocker = assess_hum_add(&hum).expect_err("assessment is intentionally fail-closed");

    println!("hum_source={}", paths.minimal_add.display());
    println!("hum_check=success");
    println!(
        "real_core_operator={}",
        operator_from_core(&hum).unwrap_or("missing")
    );
    println!("hum_to_cranelift=NO_GO");
    println!("hum_lowering_blocker={}", blocker.code);
    println!("hum_ir_ready={}", blocker.current_ir_ready);
    println!("hum_core_type_status={}", blocker.current_type_status);
    println!("hum_missing_passes={}", blocker.missing_passes.join(","));
    println!("hum_missing_facts={}", blocker.missing_facts.join(" | "));

    let jit = build_jit_checked_add(
        &CASES
            .iter()
            .copied()
            .chain([(i64::MAX, 1), (i64::MIN, -1)])
            .collect::<Vec<_>>(),
    )?;
    println!("cranelift_jit=GO");
    println!("jit_elapsed_us={}", jit.elapsed.as_micros());
    for case in &jit.cases {
        println!(
            "jit_case={},{};status={};value={}",
            case.left,
            case.right,
            case.status,
            case.value
                .map_or_else(|| "overflow".to_string(), |value| value.to_string())
        );
    }

    let object = emit_checked_add_object(&paths)?;
    println!("cranelift_object=GO");
    println!("object_path={}", object.object_path.display());
    println!("clif_path={}", object.clif_path.display());
    println!("object_bytes={}", object.object_bytes);
    println!("object_elapsed_us={}", object.elapsed.as_micros());

    let native = link_and_run_native(&paths, &object, CASES)?;
    println!("standalone_native_executable=GO");
    println!("native_executable={}", native.executable_path.display());
    println!("link_elapsed_ms={}", native.compile.elapsed.as_millis());
    for (left, right, generated) in &native.ordinary {
        let interpreted = run_hum_add(&paths, *left, *right)?;
        if generated.status != 0
            || interpreted.status != 0
            || generated.stdout != interpreted.stdout
        {
            return Err(format!(
                "interpreter/native mismatch for {left},{right}: Hum={interpreted:?}; native={generated:?}"
            ));
        }
        println!(
            "differential_case={left},{right};result={};hum_us={};native_us={}",
            generated.stdout,
            interpreted.elapsed.as_micros(),
            generated.elapsed.as_micros()
        );
    }
    let interpreted_overflow = run_hum_add(&paths, i64::MAX, 1)?;
    if native.overflow.status != 2
        || interpreted_overflow.status != 2
        || native.overflow.stderr != interpreted_overflow.stderr
    {
        return Err(format!(
            "overflow mismatch: Hum={interpreted_overflow:?}; native={:?}",
            native.overflow
        ));
    }
    println!("checked_overflow=GO;exit=2;diagnostic_match=true");
    println!("semantic_agreement_for_backend_kernel=GO");
    println!("semantic_agreement_for_hum_lowering=NOT_PROVEN");
    println!("total_elapsed_ms={}", total.elapsed().as_millis());
    Ok(())
}
