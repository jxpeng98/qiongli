use std::fs;
#[cfg(windows)]
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use qiongli_config::{
    ConfigError, ConfigState, GlobalSettings, GlobalSettingsStore, resolve_config_root,
};
#[cfg(any(unix, windows))]
use qiongli_content::ProfileId;
use serde_json::json;

const SECRET_REF: &str = "qsr1_0123456789abcdef0123456789abcdef";
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    compatibility_root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("config crate must live below the native workspace");
        let compatibility_root = native_root
            .join("target/qiongli-config-tests")
            .join(format!(
                "{name}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
        let _ = fs::remove_dir_all(&compatibility_root);
        Self { compatibility_root }
    }

    fn store(&self) -> GlobalSettingsStore {
        let root = resolve_config_root(Some(self.compatibility_root.as_os_str()), test_home())
            .expect("fixture root must resolve");
        GlobalSettingsStore::new(root)
    }

    fn state_root(&self) -> PathBuf {
        self.compatibility_root.join("v2")
    }

    fn settings_path(&self) -> PathBuf {
        self.state_root().join("settings.json")
    }

    fn write_document(&self, document: &str) {
        #[cfg(unix)]
        {
            fs::create_dir_all(self.state_root()).unwrap();
            set_mode(&self.state_root(), 0o700);
            fs::write(self.settings_path(), document).unwrap();
            set_mode(&self.settings_path(), 0o600);
        }
        #[cfg(windows)]
        {
            fs::create_dir_all(&self.compatibility_root).unwrap();
            if !self.state_root().exists() {
                drop(
                    qiongli_windows_security::create_owner_only_directory(&self.state_root())
                        .unwrap(),
                );
            }
            match fs::remove_file(self.settings_path()) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => panic!("failed to reset fixture document: {error:?}"),
            }
            let mut file =
                qiongli_windows_security::create_owner_only_new_file(&self.settings_path())
                    .unwrap();
            file.write_all(document.as_bytes()).unwrap();
            file.sync_all().unwrap();
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.compatibility_root);
    }
}

#[test]
fn missing_state_returns_revision_zero_without_writing() {
    let fixture = Fixture::new("missing");
    let loaded = fixture.store().load().unwrap();
    assert_eq!(loaded.revision, 0);
    assert_eq!(loaded.settings, GlobalSettings::default());
    assert!(!fixture.compatibility_root.exists());
}

#[test]
fn valid_state_loads_without_rewriting_bytes() {
    let fixture = Fixture::new("valid");
    let document = valid_document();
    fixture.write_document(&document);
    let loaded = fixture.store().load().unwrap();
    assert_eq!(loaded.revision, 1);
    assert_eq!(loaded.settings, GlobalSettings::default());
    assert_eq!(
        fs::read_to_string(fixture.settings_path()).unwrap(),
        document
    );
}

#[test]
fn malformed_future_and_oversized_state_remains_unchanged() {
    let fixture = Fixture::new("invalid");
    for (document, expected) in [
        (String::from("not-json"), ConfigError::InvalidDocument),
        (
            valid_document().replace("\"schema_version\": 1", "\"schema_version\": 2"),
            ConfigError::UnsupportedSchema { observed: Some(2) },
        ),
        (
            " ".repeat(qiongli_config::MAX_GLOBAL_SETTINGS_BYTES + 1),
            ConfigError::DocumentTooLarge,
        ),
    ] {
        fixture.write_document(&document);
        let before = fs::read(fixture.settings_path()).unwrap();
        assert_eq!(fixture.store().load(), Err(expected));
        assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
    }
}

#[test]
fn status_redacts_paths_emails_and_secret_refs() {
    let fixture = Fixture::new("private-user");
    let mut document: serde_json::Value = serde_json::from_str(&valid_document()).unwrap();
    document["providers"]["openalex"]["email"] = json!("researcher@example.org");
    document["providers"]["openalex"]["api_key_ref"] = json!(SECRET_REF);
    document["agent_backends"]["openai"]["enabled"] = json!(true);
    document["agent_backends"]["openai"]["api_key_ref"] = json!(SECRET_REF);
    let document = serde_json::to_string_pretty(&document).unwrap();
    fixture.write_document(&document);
    let status = fixture.store().status();
    let debug = format!("{status:?}");
    let json = serde_json::to_string(&status).unwrap();
    for canary in ["private-user", "researcher@example.org", SECRET_REF] {
        assert!(!debug.contains(canary));
        assert!(!json.contains(canary));
    }
    #[cfg(any(unix, windows))]
    assert_eq!(status.state, ConfigState::Ready);
    #[cfg(not(any(unix, windows)))]
    assert_eq!(status.state, ConfigState::WriteUnsupported);
    assert_eq!(status.revision, Some(1));
    assert!(
        status
            .providers
            .as_ref()
            .unwrap()
            .openalex
            .secret_ref_present
    );
    let openai = &status.agent_backends.as_ref().unwrap().openai;
    assert!(openai.enabled);
    assert_eq!(openai.readiness, qiongli_config::ProviderReadiness::Ready);
    assert!(openai.secret_ref_present);
}

#[test]
fn status_is_read_only_for_missing_state() {
    let fixture = Fixture::new("status-missing");
    let status = fixture.store().status();
    #[cfg(any(unix, windows))]
    assert_eq!(status.state, ConfigState::Missing);
    #[cfg(not(any(unix, windows)))]
    assert_eq!(status.state, ConfigState::WriteUnsupported);
    assert_eq!(status.revision, Some(0));
    assert!(!fixture.compatibility_root.exists());
}

#[test]
#[cfg(unix)]
fn linked_or_insecure_managed_paths_fail_without_disclosing_paths() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let fixture = Fixture::new("linked");
    let outside = Fixture::new("outside");
    fs::create_dir_all(outside.state_root()).unwrap();
    fs::create_dir_all(&fixture.compatibility_root).unwrap();
    symlink(outside.state_root(), fixture.state_root()).unwrap();
    let error = fixture.store().load().unwrap_err();
    assert_eq!(error, ConfigError::UnsafeManagedPath);
    assert!(!format!("{error:?}").contains("linked"));

    fs::remove_file(fixture.state_root()).unwrap();
    fixture.write_document(&valid_document());
    fs::set_permissions(fixture.settings_path(), fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        fixture.store().load(),
        Err(ConfigError::InsecurePermissions)
    );
}

#[test]
#[cfg(windows)]
fn windows_reparse_and_insecure_managed_paths_fail_without_touching_targets() {
    use std::os::windows::fs::{symlink_dir, symlink_file};

    let fixture = Fixture::new("windows-reparse");
    let outside = Fixture::new("windows-target");
    fs::create_dir_all(outside.state_root()).unwrap();
    fs::write(outside.state_root().join("canary.txt"), b"unchanged\n").unwrap();
    fs::create_dir_all(&fixture.compatibility_root).unwrap();
    symlink_dir(outside.state_root(), fixture.state_root()).unwrap();

    let error = fixture.store().load().unwrap_err();
    assert_eq!(error, ConfigError::UnsafeManagedPath);
    assert!(!format!("{error:?}").contains("windows-reparse"));
    assert_eq!(
        fs::read(outside.state_root().join("canary.txt")).unwrap(),
        b"unchanged\n"
    );

    fs::remove_dir(fixture.state_root()).unwrap();
    drop(qiongli_windows_security::create_owner_only_directory(&fixture.state_root()).unwrap());
    let outside_settings = outside.state_root().join("outside-settings.json");
    fs::write(&outside_settings, valid_document()).unwrap();
    symlink_file(&outside_settings, fixture.settings_path()).unwrap();
    assert_eq!(fixture.store().load(), Err(ConfigError::UnsafeManagedPath));
    assert_eq!(
        fs::read_to_string(&outside_settings).unwrap(),
        valid_document()
    );

    fs::remove_file(fixture.settings_path()).unwrap();
    fs::write(fixture.settings_path(), valid_document()).unwrap();
    assert_eq!(
        fixture.store().load(),
        Err(ConfigError::InsecurePermissions)
    );
}

#[test]
#[cfg(windows)]
fn windows_hard_linked_settings_are_rejected() {
    let fixture = Fixture::new("windows-hard-link");
    fixture.write_document(&valid_document());
    fs::hard_link(
        fixture.settings_path(),
        fixture.state_root().join("settings-alias.json"),
    )
    .unwrap();

    assert_eq!(fixture.store().load(), Err(ConfigError::UnsafeManagedPath));
}

#[test]
#[cfg(any(unix, windows))]
fn supported_replace_commits_owner_only_monotonic_documents() {
    let fixture = Fixture::new("replace");
    let store = fixture.store();
    let first = store.replace(0, GlobalSettings::default()).unwrap();
    assert_eq!(first.revision, 1);
    assert!(!first.cleanup_required);
    #[cfg(unix)]
    {
        assert_eq!(mode(&fixture.state_root()), 0o700);
        assert_eq!(mode(&fixture.settings_path()), 0o600);
        assert_eq!(mode(&fixture.state_root().join(".settings.lock")), 0o600);
    }
    #[cfg(windows)]
    {
        qiongli_windows_security::open_owner_only_directory(&fixture.state_root()).unwrap();
        qiongli_windows_security::open_owner_only_file(&fixture.settings_path()).unwrap();
        qiongli_windows_security::open_owner_only_file(
            &fixture.state_root().join(".settings.lock"),
        )
        .unwrap();
    }

    let mut next = GlobalSettings {
        default_profile: ProfileId::Full,
        ..GlobalSettings::default()
    };
    next.providers.crossref.enabled = true;
    next.providers.crossref.email =
        Some(qiongli_config::EmailAddress::parse("researcher@example.org").unwrap());
    let second = store.replace(1, next.clone()).unwrap();
    assert_eq!(second.revision, 2);
    assert!(!second.cleanup_required);
    assert_eq!(store.load().unwrap().settings, next);
    assert_eq!(
        transaction_artifacts(&fixture.state_root()),
        Vec::<String>::new()
    );
}

#[test]
#[cfg(any(unix, windows))]
fn stale_revision_never_changes_live_bytes() {
    let fixture = Fixture::new("conflict");
    let store = fixture.store();
    store.replace(0, GlobalSettings::default()).unwrap();
    let before = fs::read(fixture.settings_path()).unwrap();
    assert_eq!(
        store.replace(0, GlobalSettings::default()),
        Err(ConfigError::RevisionConflict { observed: 1 })
    );
    assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
}

#[test]
#[cfg(any(unix, windows))]
fn concurrent_replacements_have_one_winner_and_one_conflict() {
    use std::sync::{Arc, Barrier};

    let fixture = Fixture::new("concurrent");
    fixture
        .store()
        .replace(0, GlobalSettings::default())
        .unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let handles = [ProfileId::SkillOnly, ProfileId::Full].map(|profile| {
        let store = fixture.store();
        let barrier = Arc::clone(&barrier);
        std::thread::spawn(move || {
            let settings = GlobalSettings {
                default_profile: profile,
                ..GlobalSettings::default()
            };
            barrier.wait();
            store.replace(1, settings)
        })
    });
    barrier.wait();
    let results = handles.map(|handle| handle.join().unwrap());
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(ConfigError::RevisionConflict { observed: 2 })))
            .count(),
        1
    );
    assert_eq!(fixture.store().load().unwrap().revision, 2);
}

#[test]
#[cfg(unix)]
fn insecure_existing_lock_fails_before_live_state_changes() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::new("lock-mode");
    let store = fixture.store();
    store.replace(0, GlobalSettings::default()).unwrap();
    let before = fs::read(fixture.settings_path()).unwrap();
    fs::set_permissions(
        fixture.state_root().join(".settings.lock"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    assert_eq!(
        store.replace(1, GlobalSettings::default()),
        Err(ConfigError::InsecurePermissions)
    );
    assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
}

#[test]
#[cfg(windows)]
fn windows_insecure_existing_lock_fails_before_live_state_changes() {
    let fixture = Fixture::new("windows-lock-dacl");
    let store = fixture.store();
    store.replace(0, GlobalSettings::default()).unwrap();
    let before = fs::read(fixture.settings_path()).unwrap();
    let lock_path = fixture.state_root().join(".settings.lock");
    fs::remove_file(&lock_path).unwrap();
    fs::write(&lock_path, b"broad-dacl\n").unwrap();

    assert_eq!(
        store.replace(1, GlobalSettings::default()),
        Err(ConfigError::InsecurePermissions)
    );
    assert_eq!(fs::read(fixture.settings_path()).unwrap(), before);
}

fn valid_document() -> String {
    String::from(
        "{\n  \"document_kind\": \"qiongli-global-settings\",\n  \"schema_version\": 1,\n  \"revision\": 1,\n  \"default_profile\": \"marketplace-lite\",\n  \"providers\": {\n    \"openalex\": {\n      \"enabled\": false,\n      \"email\": null,\n      \"api_key_ref\": null\n    },\n    \"semantic_scholar\": {\n      \"enabled\": false,\n      \"api_key_ref\": null\n    },\n    \"crossref\": {\n      \"enabled\": false,\n      \"email\": null\n    },\n    \"pubmed\": {\n      \"enabled\": false,\n      \"api_key_ref\": null\n    },\n    \"arxiv\": {\n      \"enabled\": true\n    }\n  }\n}\n",
    )
}

#[cfg(unix)]
fn test_home() -> &'static Path {
    Path::new("/home/qiongli-test")
}

#[cfg(windows)]
fn test_home() -> &'static Path {
    Path::new(r"C:\Users\qiongli-test")
}

#[cfg(not(any(unix, windows)))]
fn test_home() -> &'static Path {
    Path::new("/home/qiongli-test")
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;

    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[cfg(any(unix, windows))]
fn transaction_artifacts(state_root: &Path) -> Vec<String> {
    let mut artifacts = fs::read_dir(state_root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.contains("qiongli-stage") || name.contains("qiongli-recovery"))
        .collect::<Vec<_>>();
    artifacts.sort();
    artifacts
}
