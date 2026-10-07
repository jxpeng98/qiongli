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
            .join("target/pi-installer-tests");
        fs::create_dir_all(&base).unwrap();
        let root = base.join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        #[cfg(windows)]
        qiongli_windows_security::create_owner_only_directory(&root).unwrap();
        #[cfg(not(windows))]
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
    fn agent(&self) -> PathBuf {
        self.home.join(".pi/agent")
    }
    fn json(&self, name: &str, value: &Value) {
        fs::create_dir_all(self.agent()).unwrap();
        fs::write(self.agent().join(name), serde_json::to_vec(value).unwrap()).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn pi_version_requires_first_published_native_mcp_release() {
    for version in ["0.99.0", "0.99.1\n", "1.0.0", "1.1.0"] {
        assert!(validate_version(version).is_ok());
    }
    for version in [
        "0.87.1",
        "0.98.9",
        "0.99.0-beta.1",
        "1.1.0-alpha",
        "pi 1.1.0",
        "unknown",
    ] {
        assert_eq!(validate_version(version), Err("pi-version-unsupported"));
    }
}

#[test]
fn pi_registration_allows_only_own_package_and_preserves_filters_and_models() {
    let home_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("synthetic-home");
    let home = home_path.as_path();
    let agent = home.join(".pi/agent");
    let source = home.join("qiongli-pi");
    let unrelated = serde_json::json!({"source":"npm:unrelated-package", "skills":[], "extensions":["extensions/allowed.js"]});
    let before = serde_json::json!({"defaultModel":"user-model-canary", "packages":[unrelated.clone()], "custom":{"keep":true}});
    let mut after = before.clone();
    after["packages"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!("../../qiongli-pi"));
    assert_eq!(
        verify_registration(&before, &after, &agent, home, &source),
        Ok(())
    );
    assert_eq!(registered(&after, &agent, home, &source), Ok(true));
    assert_eq!(before["packages"][0], unrelated);
    for changed in [
        serde_json::json!({"defaultModel":"changed", "packages":[unrelated.clone(),"../../qiongli-pi"], "custom":{"keep":true}}),
        serde_json::json!({"defaultModel":"user-model-canary", "packages":[{"source":"npm:unrelated-package","skills":["*"]},"../../qiongli-pi"], "custom":{"keep":true}}),
        serde_json::json!({"defaultModel":"user-model-canary", "packages":[unrelated.clone(),"../../qiongli-pi","npm:new-unrelated"], "custom":{"keep":true}}),
    ] {
        assert!(verify_registration(&before, &changed, &agent, home, &source).is_err());
    }
    for package in [
        serde_json::json!({"source":"../../qiongli-pi", "skills":[]}),
        serde_json::json!({"source":"../../qiongli-pi","extensions":[]}),
    ] {
        assert_eq!(
            registered(
                &serde_json::json!({"packages":[package]}),
                &agent,
                home,
                &source
            ),
            Err("pi-package-filtered")
        );
    }
    assert_eq!(
        registered(
            &serde_json::json!({"packages":["../../qiongli-pi","../../qiongli-pi"]}),
            &agent,
            home,
            &source
        ),
        Err("pi-package-conflict")
    );
    assert_eq!(
        registered(
            &serde_json::json!({"packages":["npm:qiongli"]}),
            &agent,
            home,
            &source
        ),
        Err("pi-package-conflict")
    );
    assert!(registered(&serde_json::json!({"packages":[{}]}), &agent, home, &source).is_err());
}

#[test]
fn pi_existing_relative_settings_reuse_custom_source_without_writes() {
    let f = Fixture::new();
    let source = f.root.join("qiongli-pi");
    let settings =
        serde_json::json!({"defaultModel":"user-model-canary","packages":["../../../qiongli-pi"]});
    f.json("settings.json", &settings);
    let bytes = fs::read(f.agent().join("settings.json")).unwrap();
    assert_eq!(select_source(&f.environment, None).unwrap(), source);
    assert_eq!(
        select_source(&f.environment, Some(&source)).unwrap(),
        source
    );
    assert_eq!(
        select_source(&f.environment, Some(&f.home.join("qiongli-pi"))),
        Err("pi-package-conflict")
    );
    assert_eq!(fs::read(f.agent().join("settings.json")).unwrap(), bytes);
    assert!(!source.exists());
    assert!(!f.root.join("config").exists());
}

#[test]
fn pi_plan_is_read_only_and_refuses_profile_overrides_and_disabled_mcp() {
    let f = Fixture::new();
    let content = crate::embedded_content().unwrap();
    let executable = f.root.join("fake-pi");
    fs::write(&executable, b"synthetic executable only; never run").unwrap();
    let source = f.home.join("qiongli-pi");
    f.json("settings.json", &serde_json::json!({"defaultModel":"keep-model", "packages":[{"source":"npm:unrelated","skills":[]}]}));
    let before = fs::read(f.agent().join("settings.json")).unwrap();
    let plan = prepare(&f.environment, &content, &executable, &source, "en").unwrap();
    assert_eq!(
        plan.commands,
        vec![vec![
            "install".to_owned(),
            source.to_str().unwrap().to_owned()
        ]]
    );
    assert_eq!(plan.source_receipt_sha256, None);
    assert_eq!(fs::read(f.agent().join("settings.json")).unwrap(), before);
    assert!(!source.exists());
    assert!(!f.root.join("config").exists());
    for server in ["qiongli", "qiongli-next", "qiongli_next"] {
        f.json(
            "mcp.json",
            &serde_json::json!({"mcpServers":{server:{"command":"keep"}}}),
        );
        assert_eq!(
            prepare(&f.environment, &content, &executable, &source, "en").unwrap_err(),
            "pi-mcp-conflict"
        );
    }
    f.json("mcp.json", &serde_json::json!({}));
    f.json(
        "settings.json",
        &serde_json::json!({"extensions":["-builtin:mcp"]}),
    );
    assert_eq!(
        prepare(&f.environment, &content, &executable, &source, "en").unwrap_err(),
        "pi-package-filtered"
    );
    f.json(
        "settings.json",
        &serde_json::json!({"packages":[{"source":"../../qiongli-pi","skills":[]}]}),
    );
    assert_eq!(
        prepare(&f.environment, &content, &executable, &source, "en").unwrap_err(),
        "pi-package-filtered"
    );
    assert_eq!(
        prepare(
            &f.environment,
            &content,
            &executable,
            &f.agent().join("qiongli-pi"),
            "en"
        )
        .unwrap_err(),
        "plugin-source-destination-reserved"
    );
}

#[cfg(unix)]
#[test]
fn pi_profile_refuses_links_oversize_and_writable_files() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    fs::create_dir_all(f.agent()).unwrap();
    let path = f.agent().join("settings.json");
    symlink(f.root.join("missing"), &path).unwrap();
    assert!(read_json(&path).is_err());
    fs::remove_file(&path).unwrap();
    fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap();
    assert_eq!(read_json(&path), Err("pi-profile-invalid"));
    fs::write(&path, b"{}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
    assert_eq!(read_json(&path), Err("pi-profile-invalid"));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::hard_link(&path, f.root.join("linked-profile")).unwrap();
    assert_eq!(read_json(&path), Err("pi-profile-invalid"));
}

#[cfg(unix)]
#[test]
fn pi_private_runner_preserves_literal_arguments_and_creates_private_files() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let executable = f.root.join("fake pi '$() executable");
    fs::write(&executable, "#!/bin/sh\n/bin/mkdir \"$1\"\nprintf '%s\\n' \"$2\" > \"$1/settings.json\"\nprintf '%s\\n' \"$2\"\n").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let directory = f.root.join("profile with spaces");
    let literal = "literal $(touch injected) 'quoted' \"double\"";
    let output = crate::desktop::bounded_private_host_os_command_with_timeout(
        &f.environment,
        &executable,
        &[directory.clone().into_os_string(), literal.into()],
        std::time::Duration::from_secs(5),
    )
    .unwrap();
    assert_eq!(output.trim_end(), literal);
    assert_eq!(
        fs::read_to_string(directory.join("settings.json")).unwrap(),
        format!("{literal}\n")
    );
    assert_eq!(
        fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(directory.join("settings.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(!f.home.join("injected").exists());
}
