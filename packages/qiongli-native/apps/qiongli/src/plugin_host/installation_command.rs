//! Visible progress and bounded diagnostics for already-approved Host commands.
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::cli_content::line;
use crate::command::CommandEnvironment;
use crate::install_output::InstallWriter;

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
    if text.contains("profile \"desktop\" is managed exclusively by the Electron application") {
        return Some((
            "desktop-profile-manager-required",
            "Use DeepSeek Desktop's Plugin manager or its Desktop-installed dsh command for this profile. The npm-installed dsh cannot manage the reserved desktop profile; the version number alone does not identify the correct launcher.",
        ));
    }
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
    writer: &mut impl InstallWriter,
) -> Result<(), &'static str> {
    run_observed(
        environment,
        executable,
        args,
        timeout,
        private_profile,
        writer,
        false,
    )
    .map(|_| ())
}

/// DSH retains actual launch context for verbose output and automatic failure diagnostics.
pub(crate) fn run_detailed(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
    timeout: Duration,
    writer: &mut impl InstallWriter,
) -> Result<crate::desktop::HostCommandOutput, &'static str> {
    run_observed(environment, executable, args, timeout, false, writer, true)
}

fn describe_launch(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
    timeout: Duration,
    writer: &mut impl Write,
) -> Result<(), &'static str> {
    let command = crate::desktop::official_host_command(
        environment,
        executable,
        &args.iter().map(OsString::from).collect::<Vec<_>>(),
    )
    .map_err(|error| error.reason_code())?;
    let path = command
        .get_envs()
        .find_map(|(key, value)| (key == "PATH").then_some(value).flatten());
    let paths = path
        .map(|path| std::env::split_paths(path).collect::<Vec<_>>())
        .unwrap_or_default();
    crate::cli_content::show_json(writer, &serde_json::json!({
        "working_directory": command.get_current_dir(),
        "environment": "isolated; only the listed keys are passed; shell proxy/auth variables are not inherited",
        "environment_keys": command.get_envs().map(|(key, _)| key.to_string_lossy().into_owned()).collect::<Vec<_>>(),
        "search_path": paths,
        "stdin": "closed", "stdout_stderr": "captured separately; 512 KiB limit per stream",
        "timeout_seconds": timeout.as_secs(),
    }).to_string())
}

fn reported_milestones(stdout: &[u8], stderr: &[u8]) -> Vec<&'static str> {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);
    let lines = stdout.lines().chain(stderr.lines()).collect::<Vec<_>>();
    let mut observed = Vec::new();
    if lines
        .iter()
        .any(|line| line.starts_with("Progress: resolved "))
    {
        observed.push("pnpm reported dependency-resolution progress");
    }
    let package = format!("Downloading qiongli@{}:", env!("CARGO_PKG_VERSION"));
    if lines.iter().any(|line| line.starts_with(&package)) {
        observed.push("pnpm reported a Qiongli package download");
    }
    if lines
        .iter()
        .any(|line| line.starts_with("Done in ") && line.contains(" using pnpm v"))
    {
        observed.push("pnpm reported command completion; Qiongli verification still required");
    }
    observed
}

fn run_observed(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
    timeout: Duration,
    private_profile: bool,
    writer: &mut impl InstallWriter,
    detailed: bool,
) -> Result<crate::desktop::HostCommandOutput, &'static str> {
    let command = display_command(&executable.to_string_lossy(), args);
    let verbose = writer.verbose();
    if verbose {
        line(writer, &format!("  Running: {command}\n"))?;
        if detailed {
            describe_launch(environment, executable, args, timeout, writer)?;
        }
    } else if args.first().is_none_or(|arg| arg != "--version") {
        line(
            writer,
            &format!(
                "  Running official manager (limit {}s)…\n",
                timeout.as_secs()
            ),
        )?;
    }
    if !detailed {
        writer.progress("Manager")?;
    }
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
            let result = writer.waiting(elapsed, timeout);
            output_failed = result.is_err();
            !output_failed
        },
    );
    if output_failed {
        return Err("installation-output-failed");
    }
    writer.finish_progress()?;
    let output = match result {
        Ok(output) => output,
        Err(error) => {
            if detailed && !verbose {
                describe_launch(environment, executable, args, timeout, writer)?;
            }
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
    if detailed && !verbose && !output.status.success() {
        describe_launch(environment, executable, args, timeout, writer)?;
    }
    if detailed && (verbose || !output.status.success()) {
        line(
            writer,
            &format!(
                "  Captured output: stdout {} bytes; stderr {} bytes.\n",
                output.stdout.len(),
                output.stderr.len()
            ),
        )?;
        if args.first().is_some_and(|arg| arg == "plugin") {
            let milestones = reported_milestones(&output.stdout, &output.stderr);
            if milestones.is_empty() {
                line(
                    writer,
                    "  No recognized pnpm progress markers; no internal pnpm stage can be inferred.\n",
                )?;
            }
            for milestone in milestones {
                line(writer, &format!("  Manager observation: {milestone}.\n"))?;
            }
        }
    }
    if output.status.success() {
        if verbose || !detailed {
            line(
                writer,
                &format!(
                    "  Command finished in {}s with exit 0.\n",
                    start.elapsed().as_secs()
                ),
            )?;
        }
        return Ok(output);
    }
    let status = output.status.code().map_or_else(
        || "terminated by signal".to_owned(),
        |code| code.to_string(),
    );
    line(
        writer,
        &format!(
            "  Official manager exit: {status} ({}s elapsed).\n",
            start.elapsed().as_secs()
        ),
    )?;
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

pub(super) fn run_deepseek_terminal(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
    writer: &mut impl InstallWriter,
) -> Result<(), &'static str> {
    writer.finish_progress()?;
    if writer.verbose() {
        line(
            writer,
            &format!(
                "  Running: {}\n",
                display_command(&executable.to_string_lossy(), args)
            ),
        )?;
    }
    writer.flush().map_err(|_| "installation-output-failed")?;
    let status = crate::desktop::deepseek_terminal_command(environment, executable, args)
        .map_err(|error| error.reason_code())?;
    if status.success() {
        return Ok(());
    }
    let code = status.code().map_or_else(
        || "terminated by signal".to_owned(),
        |code| code.to_string(),
    );
    line(
        writer,
        &format!(
            "Official manager exit: {code}\nRetry after reviewing the DSH error above: {}\n",
            display_command(&executable.to_string_lossy(), args)
        ),
    )?;
    if status.code().is_none() || status.code() == Some(130) {
        return Err("installation-input-failed");
    }
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
            diagnostic(
                b"error: profile \"desktop\" is managed exclusively by the Electron application"
            )
            .unwrap()
            .0,
            "desktop-profile-manager-required"
        );
        assert!(diagnostic(b"profile desktop: unknown failure").is_none());
        assert_eq!(
            diagnostic(b"unknown option --registry").unwrap().0,
            "unsupported-command"
        );
    }

    #[test]
    fn manager_milestones_never_echo_raw_output_or_attest_installation() {
        let stdout = format!(
            "Progress: resolved 2, token=private\nDownloading qiongli@{}: secret\nDone in 5s using pnpm v11.7.0\n",
            env!("CARGO_PKG_VERSION")
        );
        let observations =
            reported_milestones(stdout.as_bytes(), b"Authorization: Bearer private\x1b[2J");
        assert_eq!(observations.len(), 3);
        assert!(observations.iter().all(|text| !text.contains("private")
            && !text.contains("secret")
            && !text.contains('\x1b')));
        assert!(
            observations
                .last()
                .unwrap()
                .contains("verification still required")
        );
        assert!(reported_milestones(b"unrecognized progress", b"private error").is_empty());
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
