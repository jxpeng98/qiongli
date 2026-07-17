#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitCode;

#[allow(
    dead_code,
    reason = "the thin launcher imports only the startup subset of the shared desktop metadata contract"
)]
#[path = "../desktop_contract.rs"]
mod desktop_contract;

fn main() -> ExitCode {
    match launch_canonical_desktop() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            show_startup_error(
                "Qiongli closed before its desktop window completed startup.",
                desktop_contract::DESKTOP_STARTUP_ERROR_CODE,
            );
            ExitCode::FAILURE
        }
        Err(_) => {
            show_startup_error(
                "Qiongli could not find or start its bundled native runtime. Reinstall the application and try again.",
                desktop_contract::DESKTOP_RUNTIME_ERROR_CODE,
            );
            ExitCode::FAILURE
        }
    }
}

#[allow(
    clippy::disallowed_methods,
    reason = "the desktop launcher starts only its sibling canonical Qiongli binary, never an external language runtime"
)]
fn launch_canonical_desktop() -> io::Result<bool> {
    let mode = parse_launch_mode(env::args_os().skip(1))?;
    let current_executable = env::current_exe()?;
    let mut command = Command::new(canonical_executable_path(&current_executable)?);
    command.arg("ui");
    if mode == DesktopLaunchMode::StartupCheck {
        command.arg("--startup-check");
    }
    configure_desktop_child(&mut command);
    command.status().map(|status| status.success())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DesktopLaunchMode {
    Window,
    StartupCheck,
}

fn parse_launch_mode(
    arguments: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> io::Result<DesktopLaunchMode> {
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<_>>();
    match arguments.as_slice() {
        [] => Ok(DesktopLaunchMode::Window),
        [argument] if argument == "--startup-check" => Ok(DesktopLaunchMode::StartupCheck),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "desktop launcher accepts only the internal startup check",
        )),
    }
}

fn canonical_executable_path(current_executable: &Path) -> io::Result<PathBuf> {
    let directory = current_executable.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "desktop launcher has no parent directory",
        )
    })?;
    let packaged = directory.join(packaged_canonical_executable_name());
    if packaged.is_file() {
        return Ok(packaged);
    }
    Ok(directory.join(development_canonical_executable_name()))
}

#[cfg(target_os = "windows")]
const fn packaged_canonical_executable_name() -> &'static str {
    "qiongli-cli.exe"
}

#[cfg(not(target_os = "windows"))]
const fn packaged_canonical_executable_name() -> &'static str {
    "qiongli-cli"
}

#[cfg(target_os = "windows")]
const fn development_canonical_executable_name() -> &'static str {
    "qiongli.exe"
}

#[cfg(not(target_os = "windows"))]
const fn development_canonical_executable_name() -> &'static str {
    "qiongli"
}

#[cfg(target_os = "windows")]
fn configure_desktop_child(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn configure_desktop_child(_command: &mut Command) {}

fn show_startup_error(description: &str, reason_code: &str) {
    let _result = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(format!(
            "{} could not start",
            desktop_contract::DESKTOP_PRODUCT_NAME
        ))
        .set_description(format!("{description}\n\nError code: {reason_code}"))
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_resolves_only_the_sibling_canonical_executable() {
        let launcher = Path::new("/application/bin/qiongli-desktop");
        assert_eq!(
            canonical_executable_path(launcher).expect("launcher path must resolve"),
            Path::new("/application/bin").join(development_canonical_executable_name())
        );
        assert_ne!(
            packaged_canonical_executable_name(),
            development_canonical_executable_name()
        );
    }

    #[test]
    fn launcher_accepts_only_window_or_internal_startup_check() {
        assert_eq!(
            parse_launch_mode(std::iter::empty::<&OsStr>()).unwrap(),
            DesktopLaunchMode::Window
        );
        assert_eq!(
            parse_launch_mode([OsStr::new("--startup-check")]).unwrap(),
            DesktopLaunchMode::StartupCheck
        );
        assert!(parse_launch_mode([OsStr::new("--help")]).is_err());
        assert!(
            parse_launch_mode([OsStr::new("--startup-check"), OsStr::new("unexpected")]).is_err()
        );
    }
}
