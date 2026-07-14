#![allow(clippy::disallowed_methods)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_config::{
    EmailAddress, GLOBAL_SETTINGS_FILE, GlobalSettings, GlobalSettingsStore, resolve_config_root,
};
use qiongli_content::{MATERIALIZATION_RECEIPT_FILE, ProfileId};
use serde_json::Value;

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    config_root: PathBuf,
    home: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-cli-tests");
        fs::create_dir_all(&test_base).expect("CLI test base must be created");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let root = test_base.join(format!(
            "{name}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("isolated CLI root must be created");
        set_private_directory_mode(&root);
        let home = root.join("home");
        fs::create_dir(&home).expect("isolated CLI home must be created");
        set_private_directory_mode(&home);
        let config_root = root.join("private-config-path-canary");
        Self {
            root,
            config_root,
            home,
        }
    }

    fn store(&self) -> GlobalSettingsStore {
        let root = resolve_config_root(Some(self.config_root.as_os_str()), &self.home)
            .expect("fixture config root must resolve");
        GlobalSettingsStore::new(root)
    }

    fn state_root(&self) -> PathBuf {
        self.config_root.join("v2")
    }

    fn settings_path(&self) -> PathBuf {
        self.state_root().join(GLOBAL_SETTINGS_FILE)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn set_private_directory_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture directory mode must be private");
}

#[cfg(not(unix))]
fn set_private_directory_mode(_path: &Path) {}

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

fn run_without_home_or_path(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .args(args)
        .env("PATH", "")
        .env_remove("QIONGLI_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH")
        .output()
        .expect("native qiongli binary should start without home or PATH")
}

fn fixture_command(executable: &Path, fixture: &Fixture) -> Command {
    let mut command = Command::new(executable);
    command
        .current_dir(&fixture.root)
        .env("QIONGLI_CONFIG_HOME", &fixture.config_root)
        .env("HOME", &fixture.home)
        .env("USERPROFILE", &fixture.home);
    command
}

fn run_configured(fixture: &Fixture, args: &[&str]) -> Output {
    fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), fixture)
        .args(args)
        .output()
        .expect("configured native qiongli binary should start")
}

fn run_configured_os(
    executable: &Path,
    fixture: &Fixture,
    args: &[OsString],
    without_path: bool,
) -> Output {
    let mut command = fixture_command(executable, fixture);
    command.args(args);
    if without_path {
        command.env("PATH", "");
    }
    command
        .output()
        .expect("configured native qiongli binary should start")
}

fn parse_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("command stdout must be one JSON object")
}

fn public_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
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
fn root_and_nested_help_use_stdout_and_return_success() {
    for args in [
        ["--help"].as_slice(),
        ["-h"].as_slice(),
        ["content", "--help"].as_slice(),
        ["config", "--help"].as_slice(),
        ["install", "--help"].as_slice(),
    ] {
        let output = run(args);
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn content_list_is_versioned_deterministic_and_runtime_independent() {
    let first = run_without_path(&["content", "list"]);
    let second = run_without_path(&["content", "list"]);
    assert!(first.status.success());
    assert!(second.status.success());
    assert!(first.stderr.is_empty());
    assert_eq!(first.stdout, second.stdout);

    let value = parse_json(&first);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "content-list");
    assert_eq!(value["pack_id"], "qiongli-core");
    assert_eq!(value["pack_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(value["content_root_sha256"].as_str().unwrap().len(), 64);
    let profiles = value["profiles"]
        .as_array()
        .expect("content profiles must be an array");
    assert_eq!(profiles.len(), 3);
    assert_eq!(profiles[0]["id"], "skill-only");
    assert_eq!(profiles[1]["id"], "marketplace-lite");
    assert_eq!(profiles[2]["id"], "full");
}

#[test]
fn explicit_content_materialization_uses_the_embedded_pack_without_leaking_the_target() {
    let fixture = Fixture::new("materialize-private-canary");
    let target = fixture.root.join("materialized-skill-only");
    let args = [
        OsString::from("content"),
        OsString::from("materialize"),
        OsString::from("--target"),
        target.clone().into_os_string(),
        OsString::from("--profile"),
        OsString::from("skill-only"),
    ];
    let output = run_configured_os(
        Path::new(env!("CARGO_BIN_EXE_qiongli")),
        &fixture,
        &args,
        true,
    );
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty());
    let value = parse_json(&output);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "content-materialize");
    assert_eq!(value["profile"], "skill-only");
    assert_eq!(value["authorization"], "explicitly-approved");
    assert!(value["entry_count"].as_u64().unwrap() > 0);
    assert!(!public_output(&output).contains(&target.to_string_lossy().into_owned()));
    assert!(target.join(MATERIALIZATION_RECEIPT_FILE).is_file());

    let receipt: Value = serde_json::from_slice(
        &fs::read(target.join(MATERIALIZATION_RECEIPT_FILE))
            .expect("materialization receipt must be readable"),
    )
    .expect("materialization receipt must be JSON");
    assert_eq!(receipt["profile"], "skill-only");
    assert_eq!(receipt["pack_sha256"], value["pack_sha256"]);
}

#[test]
fn failed_materialization_is_redacted_and_preserves_the_existing_target() {
    let fixture = Fixture::new("materialize-failure-private-canary");
    let unmanaged = fixture.root.join("unmanaged-target-private-canary");
    fs::create_dir(&unmanaged).expect("unmanaged target must be created");
    let existing = unmanaged.join("existing-private-canary.txt");
    fs::write(&existing, b"preserve-me").expect("existing target file must be written");
    let before = fs::read(&existing).unwrap();
    let args = [
        OsString::from("content"),
        OsString::from("materialize"),
        OsString::from("--profile"),
        OsString::from("full"),
        OsString::from("--target"),
        unmanaged.clone().into_os_string(),
    ];
    let output = run_configured_os(
        Path::new(env!("CARGO_BIN_EXE_qiongli")),
        &fixture,
        &args,
        false,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"error: unmanaged-materialization-target\n");
    assert!(!public_output(&output).contains(&unmanaged.to_string_lossy().into_owned()));
    assert_eq!(fs::read(&existing).unwrap(), before);
    assert!(!unmanaged.join(MATERIALIZATION_RECEIPT_FILE).exists());

    let relative_canary = "relative-target-private-canary";
    let output = run_configured(
        &fixture,
        &[
            "content",
            "materialize",
            "--profile",
            "full",
            "--target",
            relative_canary,
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"error: invalid-materialization-target\n");
    assert!(!public_output(&output).contains(relative_canary));
    assert!(!fixture.root.join(relative_canary).exists());
}

#[test]
fn config_show_and_set_are_redacted_revision_safe_and_owner_only() {
    let fixture = Fixture::new("config-lifecycle-private-canary");
    let missing = run_configured(&fixture, &["config", "show"]);
    assert!(missing.status.success(), "{}", public_output(&missing));
    assert!(missing.stderr.is_empty());
    let missing_json = parse_json(&missing);
    assert_eq!(missing_json["schema_version"], 1);
    assert_eq!(missing_json["command"], "config-show");
    assert_eq!(missing_json["config"]["root_source"], "override");
    assert_eq!(
        missing_json["config"]["symbolic_state_root"],
        "<configured-root>/v2"
    );
    assert_eq!(missing_json["config"]["state"], "missing");
    assert_eq!(missing_json["config"]["revision"], 0);
    assert_eq!(
        missing_json["config"]["default_profile"],
        "marketplace-lite"
    );
    assert!(!public_output(&missing).contains(&fixture.root.to_string_lossy().into_owned()));

    let set = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--default-profile",
            "full",
            "--expected-revision",
            "0",
        ],
    );
    assert!(set.status.success(), "{}", public_output(&set));
    let set_json = parse_json(&set);
    assert_eq!(set_json["schema_version"], 1);
    assert_eq!(set_json["command"], "config-set");
    assert_eq!(set_json["revision"], 1);
    assert_eq!(set_json["default_profile"], "full");
    assert_eq!(set_json["cleanup_required"], false);
    assert_private_config_permissions(&fixture);

    let ready = run_configured(&fixture, &["config", "show"]);
    assert!(ready.status.success());
    let ready_json = parse_json(&ready);
    assert_eq!(ready_json["config"]["state"], "ready");
    assert_eq!(ready_json["config"]["revision"], 1);
    assert_eq!(ready_json["config"]["default_profile"], "full");

    let before = fs::read(fixture.settings_path()).unwrap();
    let stale = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--expected-revision",
            "0",
            "--default-profile",
            "skill-only",
        ],
    );
    assert_eq!(stale.status.code(), Some(1));
    assert!(stale.stdout.is_empty());
    assert_eq!(stale.stderr, b"error: revision-conflict\n");
    assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
}

#[cfg(unix)]
fn assert_private_config_permissions(fixture: &Fixture) {
    use std::os::unix::fs::PermissionsExt;

    let state_mode = fs::metadata(fixture.state_root())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    let settings_mode = fs::metadata(fixture.settings_path())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(state_mode, 0o700);
    assert_eq!(settings_mode, 0o600);
}

#[cfg(not(unix))]
fn assert_private_config_permissions(_fixture: &Fixture) {}

#[test]
fn config_set_preserves_provider_fields_and_show_hides_public_identifiers() {
    let fixture = Fixture::new("provider-preservation-private-canary");
    let mut settings = GlobalSettings::default();
    settings.providers.crossref.enabled = true;
    settings.providers.crossref.email = Some(
        EmailAddress::parse("provider-email-private-canary@example.org")
            .expect("provider email fixture must be valid"),
    );
    let expected_providers = settings.providers.clone();
    fixture
        .store()
        .replace(0, settings)
        .expect("initial provider settings must persist");

    let output = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--expected-revision",
            "1",
            "--default-profile",
            "skill-only",
        ],
    );
    assert!(output.status.success(), "{}", public_output(&output));
    let loaded = fixture.store().load().unwrap();
    assert_eq!(loaded.revision, 2);
    assert_eq!(loaded.settings.default_profile, ProfileId::SkillOnly);
    assert_eq!(loaded.settings.providers, expected_providers);

    let shown = run_configured(&fixture, &["config", "show"]);
    assert!(shown.status.success());
    let shown_text = public_output(&shown);
    assert!(!shown_text.contains("provider-email-private-canary@example.org"));
    assert!(!shown_text.contains(&fixture.root.to_string_lossy().into_owned()));
    let shown_json = parse_json(&shown);
    assert_eq!(
        shown_json["config"]["providers"]["crossref"]["readiness"],
        "ready"
    );
}

#[test]
fn status_and_doctor_are_read_only_redacted_and_explicit_about_limitations() {
    let fixture = Fixture::new("diagnostics-private-canary");
    let status = run_configured(&fixture, &["status"]);
    assert!(status.status.success(), "{}", public_output(&status));
    assert!(status.stderr.is_empty());
    let status_json = parse_json(&status);
    assert_eq!(status_json["schema_version"], 1);
    assert_eq!(status_json["command"], "status");
    assert_eq!(status_json["content"]["state"], "ready");
    assert_eq!(status_json["config"]["state"], "missing");
    assert!(!fixture.config_root.exists());

    let doctor = run_configured(&fixture, &["doctor"]);
    assert!(doctor.status.success(), "{}", public_output(&doctor));
    assert!(doctor.stderr.is_empty());
    let doctor_json = parse_json(&doctor);
    assert_eq!(doctor_json["schema_version"], 1);
    assert_eq!(doctor_json["command"], "doctor");
    assert_eq!(doctor_json["overall"], "ready");
    let checks = doctor_json["checks"].as_array().unwrap();
    let config = checks
        .iter()
        .find(|check| check["id"] == "global-config")
        .unwrap();
    assert_eq!(config["state"], "missing");
    assert_eq!(config["blocking"], false);
    let secure_store = checks
        .iter()
        .find(|check| check["id"] == "secure-store")
        .unwrap();
    assert_eq!(secure_store["state"], "unavailable");
    assert_eq!(secure_store["blocking"], false);
    assert!(!fixture.config_root.exists());
    assert!(!public_output(&doctor).contains(&fixture.root.to_string_lossy().into_owned()));
}

#[test]
fn doctor_returns_blocking_json_for_invalid_config_without_exposing_document_bytes() {
    let fixture = Fixture::new("doctor-invalid-private-canary");
    let initialized = run_configured(
        &fixture,
        &[
            "config",
            "set",
            "--expected-revision",
            "0",
            "--default-profile",
            "full",
        ],
    );
    assert!(initialized.status.success());
    fs::write(fixture.settings_path(), b"invalid-document-private-canary")
        .expect("managed config must be made invalid");

    let doctor = run_configured(&fixture, &["doctor"]);
    assert_eq!(doctor.status.code(), Some(1));
    assert!(doctor.stderr.is_empty());
    let doctor_json = parse_json(&doctor);
    assert_eq!(doctor_json["overall"], "attention");
    let config = doctor_json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "global-config")
        .unwrap();
    assert_eq!(config["state"], "invalid");
    assert_eq!(config["blocking"], true);
    let output = public_output(&doctor);
    assert!(!output.contains("invalid-document-private-canary"));
    assert!(!output.contains(&fixture.root.to_string_lossy().into_owned()));
}

#[test]
fn supported_commands_do_not_require_an_external_runtime_path() {
    for args in [
        ["--version"].as_slice(),
        ["--help"].as_slice(),
        ["content", "list"].as_slice(),
        ["install", "status"].as_slice(),
    ] {
        let output = run_without_home_or_path(args);
        assert!(output.status.success(), "{}", public_output(&output));
        assert!(output.stderr.is_empty());
    }

    let fixture = Fixture::new("empty-path-private-canary");
    for args in [["status"].as_slice(), ["doctor"].as_slice()] {
        let mut command = fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), &fixture);
        let output = command
            .args(args)
            .env("PATH", "")
            .output()
            .expect("native data command should start without PATH");
        assert!(output.status.success(), "{}", public_output(&output));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn install_status_is_read_only_and_truthful_for_source_builds() {
    let output = run_without_home_or_path(&["install", "status"]);
    assert!(output.status.success(), "{}", public_output(&output));
    assert!(output.stderr.is_empty());
    let value = parse_json(&output);
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["command"], "install-status");
    assert_eq!(value["contracts"]["artifact_identity"], 1);
    assert_eq!(value["contracts"]["launch_grant"], 1);
    assert_eq!(value["contracts"]["install_plan"], 1);
    assert_eq!(value["contracts"]["install_receipt"], 1);
    assert!(value["current_target"]["os"].is_string());
    assert!(value["current_target"]["arch"].is_string());
    assert_eq!(value["transaction_engine"], "grant-and-approval-gated");
    assert_eq!(value["launch_grant"], "unavailable");
    assert_eq!(value["preview"], "unavailable");
    assert_eq!(value["apply"], "unavailable");
    assert_eq!(value["targets"][0]["family"], "codex-local");
    assert_eq!(value["targets"][1]["family"], "claude-code-local");
    assert_eq!(value["targets"][0]["state"], "contract-only");
    assert_eq!(value["targets"][1]["state"], "contract-only");
}

#[test]
fn invalid_invocations_and_environment_fail_without_echoing_private_values() {
    let cases: &[(&[&str], Option<&str>)] = &[
        (&[], None),
        (&["ui"], None),
        (&["content"], None),
        (&["content", "help"], None),
        (
            &["content", "list", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (&["content", "materialize", "--profile", "full"], None),
        (&["config"], None),
        (&["config", "-h"], None),
        (&["install"], None),
        (
            &["install", "status", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (
            &["config", "show", "extra-private-canary"],
            Some("extra-private-canary"),
        ),
        (
            &[
                "config",
                "set",
                "--expected-revision",
                "revision-private-canary",
            ],
            Some("revision-private-canary"),
        ),
        (
            &["--version", "trailing-private-canary"],
            Some("trailing-private-canary"),
        ),
        (&["command-private-canary"], Some("command-private-canary")),
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

    let fixture = Fixture::new("environment-error-private-canary");
    let output = fixture_command(Path::new(env!("CARGO_BIN_EXE_qiongli")), &fixture)
        .arg("status")
        .env("QIONGLI_CONFIG_HOME", "relative-environment-private-canary")
        .output()
        .expect("native status command should start");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"error: invalid-config-home\n");
    assert!(!public_output(&output).contains("relative-environment-private-canary"));

    let output = Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .arg("status")
        .env_remove("QIONGLI_CONFIG_HOME")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH")
        .output()
        .expect("native status command should start without a home");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"error: home-unavailable\n");
}

#[test]
fn copied_binary_lists_and_materializes_embedded_content_without_source_lookup() {
    let fixture = Fixture::new("copied-runtime-private-canary");
    let source = PathBuf::from(env!("CARGO_BIN_EXE_qiongli"));
    let runtime_root = std::env::temp_dir().join(format!(
        "qiongli-copied-runtime-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos(),
        NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&runtime_root).expect("outside-checkout runtime root must be created");
    set_private_directory_mode(&runtime_root);
    let copied = runtime_root.join(
        source
            .file_name()
            .expect("native executable must have a file name"),
    );
    fs::copy(&source, &copied).expect("native executable must copy outside the checkout");

    let list = fixture_command(&copied, &fixture)
        .current_dir(&runtime_root)
        .args(["content", "list"])
        .env("PATH", "")
        .output()
        .expect("copied executable must list embedded content outside the checkout");
    assert!(list.status.success(), "{}", public_output(&list));
    assert_eq!(parse_json(&list)["command"], "content-list");

    let install_status = fixture_command(&copied, &fixture)
        .current_dir(&runtime_root)
        .args(["install", "status"])
        .env("PATH", "")
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .env_remove("HOMEDRIVE")
        .env_remove("HOMEPATH")
        .output()
        .expect("copied executable must report install status without a runtime");
    assert!(
        install_status.status.success(),
        "{}",
        public_output(&install_status)
    );
    let install_value = parse_json(&install_status);
    assert_eq!(install_value["command"], "install-status");
    assert_eq!(
        install_value["transaction_engine"],
        "grant-and-approval-gated"
    );
    assert_eq!(install_value["launch_grant"], "unavailable");
    assert_eq!(install_value["apply"], "unavailable");

    let target = fixture.root.join("copied-binary-materialized");
    let materialize = fixture_command(&copied, &fixture)
        .current_dir(&runtime_root)
        .args([
            OsString::from("content"),
            OsString::from("materialize"),
            OsString::from("--profile"),
            OsString::from("lite"),
            OsString::from("--target"),
            target.clone().into_os_string(),
        ])
        .env("PATH", "")
        .output()
        .expect("copied executable must materialize embedded content outside the checkout");
    assert!(
        materialize.status.success(),
        "{}",
        public_output(&materialize)
    );
    assert!(materialize.stderr.is_empty());
    let value = parse_json(&materialize);
    assert_eq!(value["profile"], "marketplace-lite");
    assert!(target.join(MATERIALIZATION_RECEIPT_FILE).is_file());
    assert!(!public_output(&materialize).contains(&target.to_string_lossy().into_owned()));
    fs::remove_dir_all(runtime_root).expect("outside-checkout runtime root must be removed");
}
