use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let environment = qiongli::CommandEnvironment::from_process();
    let output = match qiongli::embedded_content() {
        Ok(content) => qiongli::run_cli(env::args_os().skip(1), &environment, &content),
        Err(_) => qiongli::failed_embedded_content_output(),
    };
    if !output.stdout().is_empty() {
        print!("{}", output.stdout());
    }
    if !output.stderr().is_empty() {
        eprint!("{}", output.stderr());
    }
    ExitCode::from(output.exit_code())
}
