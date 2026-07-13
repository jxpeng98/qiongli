#![forbid(clippy::disallowed_methods)]

use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

const USAGE: &str = "Qiongli native platform\n\nUsage:\n  qiongli --version\n  qiongli --help\n\nOptions:\n  -h, --help  Print help\n  --version   Print the native product version\n";

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Help,
    Version,
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, &'static str> {
    let mut args = args.into_iter();
    let Some(raw_command) = args.next() else {
        return Err("a command or option is required");
    };

    if args.next().is_some() {
        return Err("unexpected extra argument");
    }

    let Some(command) = raw_command.to_str() else {
        return Err("the command is not valid UTF-8");
    };

    match command {
        "-h" | "--help" => Ok(Command::Help),
        "--version" => Ok(Command::Version),
        _ => Err("unknown command or option"),
    }
}

fn main() -> ExitCode {
    if let Err(error) = qiongli::embedded_content() {
        eprintln!("error: embedded content integrity verification failed: {error}");
        return ExitCode::FAILURE;
    }

    match parse_args(env::args_os().skip(1)) {
        Ok(Command::Help) => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("qiongli {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_args};
    use std::ffi::OsString;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_accepts_only_the_bootstrap_contract() {
        assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
    }

    #[test]
    fn parser_rejects_bare_unknown_and_extra_arguments() {
        assert!(parse_args(Vec::<OsString>::new()).is_err());
        assert!(parse_args(args(&["ui"])).is_err());
        assert!(parse_args(args(&["--version", "extra"])).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn parser_rejects_non_utf8_unix_arguments() {
        use std::os::unix::ffi::OsStringExt;

        let invalid = OsString::from_vec(vec![0xff, 0xfe]);
        assert!(parse_args([invalid]).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn parser_rejects_unpaired_windows_surrogates() {
        use std::os::windows::ffi::OsStringExt;

        let invalid = OsString::from_wide(&[0xd800]);
        assert!(parse_args([invalid]).is_err());
    }
}
