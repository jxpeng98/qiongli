//! Visible progress and bounded diagnostics for already-approved Host commands.
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::cli_content::line;
use crate::command::CommandEnvironment;

pub(crate) fn display_command(executable: &str, args: &[String]) -> String {
    if std::iter::once(executable)
        .chain(args.iter().map(String::as_str))
        .any(|part| part.chars().any(char::is_control))
    {
        return "Command contains control characters; use the reviewed argument list.".into();
    }
    let quote = |part: &str| {
        if !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_=./:@".contains(&b))
        {
            return part.to_owned();
        }
        #[cfg(windows)]
        let escaped = part.replace('\'', "''");
        #[cfg(not(windows))]
        let escaped = part.replace('\'', "'\\''");
        format!("'{escaped}'")
    };
    let command = std::iter::once(executable)
        .chain(args.iter().map(String::as_str))
        .map(quote)
        .collect::<Vec<_>>()
        .join(" ");
    #[cfg(windows)]
    {
        format!("& {command}")
    }
    #[cfg(not(windows))]
    {
        command
    }
}

// Only known codes/categories leave the bounded capture. Package managers may
// echo credentials, URLs, config or terminal escapes in their raw output.
fn diagnostic(output: &[u8]) -> Option<(&'static str, &'static str)> {
    let text = String::from_utf8_lossy(output);
    let tokens = text
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .collect::<Vec<_>>();
    for (code, hint) in [
        (
            "ETARGET",
            "A requested package version is unavailable in the selected registry. Check Qiongli and dependency versions against that registry before retrying.",
        ),
        (
            "ERR_PNPM_NO_MATCHING_VERSION",
            "A requested package version is unavailable in the selected registry. Check Qiongli and dependency versions against that registry before retrying.",
        ),
        (
            "E404",
            "The registry did not find a requested package. Check the exact package/version and registry; a dependency may also be missing.",
        ),
        (
            "ERR_PNPM_FETCH_404",
            "The registry did not find a requested package. Check the exact package/version and registry; a dependency may also be missing.",
        ),
        (
            "EACCES",
            "The manager could not access a required path. Check the selected profile and package-cache permissions; do not delete the profile.",
        ),
        (
            "EPERM",
            "The manager could not modify a required path. Check permissions and whether another process is using it.",
        ),
        (
            "E401",
            "The registry requires valid authentication. Review the package manager's registry configuration.",
        ),
        (
            "E403",
            "The registry refused access. Review its access policy and the package manager's configuration.",
        ),
        (
            "ENOTFOUND",
            "The manager could not resolve a host. Check DNS and registry connectivity.",
        ),
        (
            "EAI_AGAIN",
            "The manager reported a temporary DNS failure. Check connectivity before retrying.",
        ),
        (
            "ECONNRESET",
            "The network connection was reset. Check connectivity and the package manager's proxy configuration.",
        ),
        (
            "ECONNREFUSED",
            "The network connection was refused. Check registry/proxy availability.",
        ),
        (
            "ETIMEDOUT",
            "A network request timed out. Check registry connectivity and proxy settings.",
        ),
        (
            "CERT_HAS_EXPIRED",
            "TLS certificate validation failed. Check system time and the trusted certificate configuration.",
        ),
        (
            "UNABLE_TO_VERIFY_LEAF_SIGNATURE",
            "TLS certificate validation failed. Review the package manager's certificate configuration.",
        ),
        (
            "EBADPLATFORM",
            "A package does not support this OS or architecture. Check the package's supported targets.",
        ),
    ] {
        if tokens.contains(&code) {
            return Some((code, hint));
        }
    }
    let lower = text.to_ascii_lowercase();
    if lower.contains("unknown option") || lower.contains("unknown command") {
        Some((
            "unsupported-command",
            "The manager rejected an option or command. Check its version and help against the displayed command.",
        ))
    } else if lower.contains("command not found") || lower.contains("executable not found") {
        Some((
            "missing-executable",
            "A required executable is unavailable to the manager. Check its runtime/package-manager installation and PATH.",
        ))
    } else {
        None
    }
}

pub(crate) fn run(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
    timeout: Duration,
    private_profile: bool,
    writer: &mut impl Write,
) -> Result<(), &'static str> {
    let command = display_command(&executable.to_string_lossy(), args);
    line(writer, &format!("  Running: {command}\n"))?;
    writer.flush().map_err(|_| "installation-output-failed")?;
    let start = Instant::now();
    let mut output_failed = false;
    let result = crate::desktop::host_installation_command_with_progress(
        environment,
        executable,
        &args.iter().map(OsString::from).collect::<Vec<_>>(),
        timeout,
        private_profile,
        |elapsed| {
            let result = line(
                writer,
                &format!(
                    "  Still running: {}s elapsed (limit {}s).\n",
                    elapsed.as_secs(),
                    timeout.as_secs()
                ),
            )
            .and_then(|()| writer.flush().map_err(|_| "installation-output-failed"));
            output_failed = result.is_err();
            !output_failed
        },
    );
    if output_failed {
        return Err("installation-output-failed");
    }
    let output = match result {
        Ok(output) => output,
        Err(error) => {
            line(
                writer,
                &format!(
                    "  Command failed after {}s: {}.\n  Inspect the manager state and retained files before retrying: {command}\n",
                    start.elapsed().as_secs(),
                    error.reason_code()
                ),
            )?;
            return Err(error.reason_code());
        }
    };
    if output.status.success() {
        return line(
            writer,
            &format!(
                "  Command finished in {}s; verifying installation next.\n",
                start.elapsed().as_secs()
            ),
        );
    }
    let status = output.status.code().map_or_else(
        || "terminated by signal".to_owned(),
        |code| code.to_string(),
    );
    line(writer, &format!("  Official manager exit: {status}.\n"))?;
    if let Some((code, hint)) = diagnostic(&output.stderr).or_else(|| diagnostic(&output.stdout)) {
        line(writer, &format!("  Reported category: {code}. {hint}\n"))?;
    } else {
        line(
            writer,
            "  No recognized error category. This exit alone does not identify the cause.\n",
        )?;
    }
    line(
        writer,
        &format!(
            "  Raw manager output is not echoed because it may contain credentials or configuration.\n  Inspect retained files, then run this command directly for full diagnostics: {command}\n"
        ),
    )?;
    Err("host-command-nonzero-exit")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_recognizes_codes_without_returning_manager_text() {
        let output = b"\x1b[2J ERR_PNPM_NO_MATCHING_VERSION https://user:secret@example.test?token=secret\nAuthorization: Bearer secret";
        let (code, hint) = diagnostic(output).unwrap();
        assert_eq!(code, "ERR_PNPM_NO_MATCHING_VERSION");
        assert!(
            !hint.contains("secret") && !hint.contains("example.test") && !hint.contains('\x1b')
        );
        assert!(diagnostic(b"private-config-and-unrecognized-error").is_none());
        assert!(diagnostic(b"prefixETARGETsuffix").is_none());
        assert_eq!(
            diagnostic(b"unknown option --registry").unwrap().0,
            "unsupported-command"
        );
    }

    #[test]
    fn retry_commands_quote_paths_and_reject_terminal_controls() {
        let display = display_command(
            "qiongli",
            &["--destination".into(), "a b'$(touch bad)".into()],
        );
        #[cfg(not(windows))]
        assert_eq!(display, "qiongli --destination 'a b'\\''$(touch bad)'");
        #[cfg(windows)]
        assert_eq!(display, "& qiongli --destination 'a b''$(touch bad)'");
        assert!(!display_command("qiongli", &["\x1b[2J\nunsafe".into()]).contains('\x1b'));
    }
}
