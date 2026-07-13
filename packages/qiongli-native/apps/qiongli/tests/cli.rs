#![allow(clippy::disallowed_methods)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_RUNTIME_COPY_ID: AtomicU64 = AtomicU64::new(0);

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

#[test]
fn copied_binary_starts_without_the_source_checkout_or_runtime_path() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test clock must follow the Unix epoch")
        .as_nanos();
    let copy_id = NEXT_RUNTIME_COPY_ID.fetch_add(1, Ordering::Relaxed);
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-embedded-runtime-{}-{nonce}-{copy_id}",
        std::process::id(),
    ));
    fs::create_dir(&runtime_root).expect("isolated runtime root must be created");
    let source = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let copied = runtime_root.join(
        source
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source, &copied).expect("native executable must copy outside the checkout");

    let output = Command::new(&copied)
        .arg("--version")
        .current_dir(&runtime_root)
        .env("PATH", "")
        .output()
        .expect("copied native executable must start without external runtimes");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("qiongli {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());

    fs::remove_dir_all(runtime_root).expect("isolated runtime root must be removed");
}
