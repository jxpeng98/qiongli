use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match qiongli::run_native_update_helper(env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason_code) => {
            eprintln!("error: {reason_code}");
            ExitCode::FAILURE
        }
    }
}
