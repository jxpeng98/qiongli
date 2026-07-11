#![allow(clippy::disallowed_methods)]

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .args(args)
        .output()
        .expect("native qiongli binary should start")
}

fn run_without_path(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .args(args)
        .env("PATH", "")
        .output()
        .expect("native qiongli binary should start without PATH")
}

#[test]
fn version_uses_the_workspace_package_version() {
    let output = run(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("qiongli {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn help_uses_stdout_and_returns_success() {
    for args in [["--help"].as_slice(), ["-h"].as_slice()] {
        let output = run(args);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn bootstrap_commands_do_not_require_an_external_runtime_path() {
    for args in [["--version"].as_slice(), ["--help"].as_slice()] {
        let output = run_without_path(args);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn invalid_invocations_fail_as_usage_errors() {
    let cases: &[(&[&str], Option<&str>)] = &[
        (&[], None),
        (&["ui"], None),
        (
            &["--version", "trailing-credential-canary-must-not-echo"],
            Some("trailing-credential-canary-must-not-echo"),
        ),
        (
            &["credential-canary-must-not-echo"],
            Some("credential-canary-must-not-echo"),
        ),
    ];

    for &(args, canary) in cases {
        let output = run(args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("error:"));
        assert!(stderr.contains("Usage:"));
        if let Some(canary) = canary {
            assert!(!stderr.contains(canary));
        }
    }
}
