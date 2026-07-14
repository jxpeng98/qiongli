use std::env;
use std::io::{self, BufReader};
use std::process::ExitCode;

fn main() -> ExitCode {
    let environment = qiongli::CommandEnvironment::from_process();
    let content = match qiongli::embedded_content() {
        Ok(content) => content,
        Err(_) => return render_output(qiongli::failed_embedded_content_output()),
    };
    match qiongli::prepare_action(env::args_os().skip(1), &environment, &content) {
        qiongli::ProductAction::Output(output) => render_output(output),
        qiongli::ProductAction::ServeLiteMcpStdio => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut reader = BufReader::new(stdin.lock());
            let mut writer = stdout.lock();
            match qiongli::serve_lite_mcp(&mut reader, &mut writer, &environment, &content) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("error: {}", error.reason_code());
                    ExitCode::FAILURE
                }
            }
        }
        qiongli::ProductAction::LaunchDesktop => {
            if qiongli::run_desktop(environment, content).is_ok() {
                ExitCode::SUCCESS
            } else {
                eprintln!("error: desktop-ui-start-failed");
                ExitCode::FAILURE
            }
        }
    }
}

fn render_output(output: qiongli::CliOutput) -> ExitCode {
    if !output.stdout().is_empty() {
        print!("{}", output.stdout());
    }
    if !output.stderr().is_empty() {
        eprint!("{}", output.stderr());
    }
    ExitCode::from(output.exit_code())
}
