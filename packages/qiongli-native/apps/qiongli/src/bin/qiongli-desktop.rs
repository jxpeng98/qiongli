#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::env;
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
    let current_executable = env::current_exe()?;
    let mut command = Command::new(canonical_executable_path(&current_executable)?);
    command.arg("ui");
    configure_desktop_child(&mut command);
    command.status().map(|status| status.success())
}

fn canonical_executable_path(current_executable: &Path) -> io::Result<PathBuf> {
    let directory = current_executable.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "desktop launcher has no parent directory",
        )
    })?;
    Ok(directory.join(canonical_executable_name()))
}

#[cfg(target_os = "windows")]
const fn canonical_executable_name() -> &'static str {
    "qiongli.exe"
}

#[cfg(not(target_os = "windows"))]
const fn canonical_executable_name() -> &'static str {
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
            Path::new("/application/bin").join(canonical_executable_name())
        );
    }
}
