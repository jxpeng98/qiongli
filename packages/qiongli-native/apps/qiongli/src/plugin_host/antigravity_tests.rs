#![allow(clippy::disallowed_methods)]
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};
struct Fixture {
    root: PathBuf,
    home: PathBuf,
    environment: CommandEnvironment,
}
impl Fixture {
    fn new() -> Self {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/antigravity-installer-tests");
        fs::create_dir_all(&base).unwrap();
        let root = base.join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let environment = CommandEnvironment::with_paths(
            Some(root.join("config").into_os_string()),
            Some(home.clone()),
            None,
        );
        Self {
            root,
            home,
            environment,
        }
    }
    fn prepare(&self, source: &Path) -> Result<Plan, &'static str> {
        let executable = std::env::current_exe().unwrap();
        prepare(
            &self.environment,
            &crate::embedded_content().unwrap(),
            &executable,
            source,
            "en",
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn antigravity_plan_is_read_only_and_records_official_commands() {
    let f = Fixture::new();
    let source = f.home.join("qiongli-antigravity");
    let plan = f.prepare(&source).unwrap();
    assert!(!source.exists());
    assert!(!f.home.join(".gemini").exists());
    assert_eq!(
        plan.commands,
        vec![
            vec!["plugin", "install", source.to_str().unwrap()],
            vec!["plugin", "enable", crate::plugin_source::plugin_name()]
        ]
    );
    assert!(plan.source_receipt_sha256.is_none());
    assert!(plan.cache_receipt_sha256.is_none());
    let mut out = Vec::new();
    preview(&mut out, "antigravity-plugin-files", &plan).unwrap();
    assert!(String::from_utf8(out).unwrap().contains("filesystem-write"));
    assert!(!source.exists());
}

#[test]
fn antigravity_plan_refuses_reserved_unmanaged_and_mcp_conflicts() {
    let f = Fixture::new();
    assert_eq!(
        f.prepare(&f.home.join(".gemini/qiongli")).err(),
        Some("plugin-source-destination-reserved")
    );
    assert_eq!(
        f.prepare(&f.home.join("arbitrary")).err(),
        Some("plugin-source-destination-invalid")
    );
    let source = f.home.join("qiongli-antigravity");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("canary"), b"keep").unwrap();
    assert!(f.prepare(&source).is_err());
    assert_eq!(fs::read(source.join("canary")).unwrap(), b"keep");
    fs::remove_dir_all(&source).unwrap();
    let config = f.home.join(".gemini/config");
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("mcp_config.json"),
        br#"{"mcpServers":{"qiongli":{"command":"other"}}}"#,
    )
    .unwrap();
    assert_eq!(f.prepare(&source).err(), Some("antigravity-mcp-conflict"));
    assert!(!source.exists());
}

#[test]
fn antigravity_profile_tracks_disabled_state_and_requires_native_import() {
    let f = Fixture::new();
    let config = f.home.join(".gemini/config");
    fs::create_dir_all(&config).unwrap();
    let settings = config.join("config.json");
    fs::write(
        &settings,
        br#"{"plugins":{"qiongli":{"enabled":false}},"model":"preserve"}"#,
    )
    .unwrap();
    assert!(!enabled(&settings, "qiongli").unwrap());
    let imports = config.join("import_manifest.json");
    fs::write(
        &imports,
        br#"{"imports":[{"name":"qiongli","source":"claude-code"}]}"#,
    )
    .unwrap();
    assert!(!registered(&imports, "qiongli").unwrap());
    fs::write(
        &imports,
        br#"{"imports":[{"name":"qiongli","source":"antigravity"}]}"#,
    )
    .unwrap();
    assert!(registered(&imports, "qiongli").unwrap());
    assert!(
        String::from_utf8(fs::read(&settings).unwrap())
            .unwrap()
            .contains("preserve")
    );
}

#[cfg(unix)]
#[test]
fn antigravity_profile_refuses_symlink_and_oversized_json() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let file = f.home.join("profile.json");
    let target = f.root.join("canary");
    fs::write(&target, b"keep").unwrap();
    symlink(&target, &file).unwrap();
    assert_eq!(
        read_optional(&file).err(),
        Some("antigravity-profile-invalid")
    );
    assert_eq!(fs::read(&target).unwrap(), b"keep");
    fs::remove_file(&file).unwrap();
    fs::write(&file, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert_eq!(
        read_optional(&file).err(),
        Some("antigravity-profile-invalid")
    );
}

#[test]
fn antigravity_profile_refuses_nonobject_and_invalid_enabled_flags() {
    let f = Fixture::new();
    let config = f.home.join(".gemini/config");
    fs::create_dir_all(&config).unwrap();
    let settings = config.join("config.json");
    for invalid in [
        br#"[]"#.as_slice(),
        br#"{"plugins":{"qiongli":{"enabled":"false"}}}"#.as_slice(),
    ] {
        fs::write(&settings, invalid).unwrap();
        assert_eq!(
            enabled(&settings, "qiongli").err(),
            Some("antigravity-profile-invalid")
        );
        let source = f.home.join("qiongli-antigravity");
        assert_eq!(
            f.prepare(&source).err(),
            Some("antigravity-profile-invalid")
        );
        assert!(!source.exists());
    }
    fs::write(&settings, b"{}").unwrap();
    assert!(enabled(&settings, "qiongli").unwrap());
}
