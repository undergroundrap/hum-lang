use std::env;
use std::process::ExitCode;

unsafe extern "C" {
    fn hum_spike_checked_add(left: i64, right: i64, result: *mut i64) -> i32;
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let left = args
        .next()
        .and_then(|value| value.parse::<i64>().ok())
        .expect("first signed 64-bit integer argument");
    let right = args
        .next()
        .and_then(|value| value.parse::<i64>().ok())
        .expect("second signed 64-bit integer argument");
    assert!(args.next().is_none(), "exactly two arguments are required");

    let mut result = 0_i64;
    // SAFETY: the generated function has the declared C ABI and writes one i64
    // only when the supplied pointer is non-null; `result` is live and aligned.
    let status = unsafe { hum_spike_checked_add(left, right, &mut result) };
    match status {
        0 => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        1 => {
            eprintln!("runtime trap: integer overflow while evaluating `a + b`");
            ExitCode::from(2)
        }
        other => {
            eprintln!("unexpected generated-code status {other}");
            ExitCode::from(70)
        }
    }
}
