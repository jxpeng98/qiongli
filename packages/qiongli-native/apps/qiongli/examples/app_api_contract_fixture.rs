use std::process::ExitCode;

fn main() -> ExitCode {
    match qiongli::app_api_contract_fixture_json() {
        Ok(fixture) => {
            print!("{fixture}");
            ExitCode::SUCCESS
        }
        Err(code) => {
            eprintln!("error: {code}");
            ExitCode::FAILURE
        }
    }
}
