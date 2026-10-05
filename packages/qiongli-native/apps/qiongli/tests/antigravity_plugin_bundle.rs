#![allow(clippy::disallowed_methods)]
use qiongli_platform::{
    CodexPluginBundleKind, approve_antigravity_plugin_bundle_target,
    approve_codex_plugin_bundle_target, compose_local_antigravity_plugin_source,
    remove_local_antigravity_plugin_source, verify_codex_plugin_bundle,
    verify_local_antigravity_plugin_source,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    binary: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/antigravity-bundle-tests");
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
        let binary = root.join("source-qiongli");
        fs::copy(env!("CARGO_BIN_EXE_qiongli"), &binary).unwrap();
        Self { root, binary }
    }
    fn digest(&self) -> String {
        format!("{:x}", Sha256::digest(fs::read(&self.binary).unwrap()))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn antigravity_create_update_and_remove_are_receipt_bound() {
    let f = Fixture::new();
    let content = qiongli::embedded_content().unwrap();
    let path = f.root.join("qiongli-antigravity");
    let target = approve_antigravity_plugin_bundle_target(&path).unwrap();
    let first = compose_local_antigravity_plugin_source(
        content.pack(),
        &f.binary,
        &f.digest(),
        &target,
        None,
        None,
        Some("en"),
    )
    .unwrap();
    assert_eq!(
        first.receipt().package_kind,
        CodexPluginBundleKind::UserLocalAntigravityFullMcp
    );
    assert!(
        path.join(".qiongli-antigravity-plugin-bundle.json")
            .is_file()
    );
    assert!(!path.join(".codex-plugin").exists());
    assert!(!path.join(".mcp.json").exists());
    let mcp: serde_json::Value =
        serde_json::from_slice(&fs::read(path.join("mcp_config.json")).unwrap()).unwrap();
    let name = first.receipt().plugin_name();
    assert_eq!(
        mcp["mcpServers"][name]["command"],
        path.join(&first.receipt().binary_path).to_str().unwrap()
    );
    let skill = fs::read_to_string(path.join("skills/qiongli-workflow/SKILL.md")).unwrap();
    assert!(skill.contains("Antigravity Native Host Adapter"));
    assert!(!skill.contains("## Codex Native Host Adapter"));
    assert_eq!(
        verify_local_antigravity_plugin_source(&target).unwrap(),
        first
    );
    assert!(
        compose_local_antigravity_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            None,
            Some(&"0".repeat(64)),
            Some("en")
        )
        .is_err()
    );
    assert_eq!(
        verify_local_antigravity_plugin_source(&target).unwrap(),
        first
    );
    let updated = compose_local_antigravity_plugin_source(
        content.pack(),
        &f.binary,
        &f.digest(),
        &target,
        None,
        Some(first.receipt_sha256()),
        Some("en"),
    )
    .unwrap();
    assert!(remove_local_antigravity_plugin_source(&target, &"0".repeat(64)).is_err());
    remove_local_antigravity_plugin_source(&target, updated.receipt_sha256()).unwrap();
    assert!(!path.exists());
}

#[test]
fn antigravity_refuses_unmanaged_drift_foreign_receipt_and_bad_binary() {
    let f = Fixture::new();
    let content = qiongli::embedded_content().unwrap();
    let path = f.root.join("qiongli-antigravity");
    let target = approve_antigravity_plugin_bundle_target(&path).unwrap();
    assert!(
        compose_local_antigravity_plugin_source(
            content.pack(),
            &f.binary,
            &"0".repeat(64),
            &target,
            None,
            None,
            None
        )
        .is_err()
    );
    assert!(!path.exists());
    fs::create_dir(&path).unwrap();
    fs::write(path.join("keep"), b"unmanaged").unwrap();
    assert!(
        compose_local_antigravity_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            None,
            None,
            None
        )
        .is_err()
    );
    assert_eq!(fs::read(path.join("keep")).unwrap(), b"unmanaged");
    fs::remove_dir_all(&path).unwrap();
    let saved = compose_local_antigravity_plugin_source(
        content.pack(),
        &f.binary,
        &f.digest(),
        &target,
        None,
        None,
        None,
    )
    .unwrap();
    let codex = approve_codex_plugin_bundle_target(&path).unwrap();
    assert!(verify_codex_plugin_bundle(&codex).is_err());
    fs::write(
        path.join("skills/qiongli-workflow/SKILL.md"),
        b"deliberate synthetic drift",
    )
    .unwrap();
    assert!(verify_local_antigravity_plugin_source(&target).is_err());
    assert!(
        compose_local_antigravity_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            None,
            Some(saved.receipt_sha256()),
            None
        )
        .is_err()
    );
    assert_eq!(
        fs::read(path.join("skills/qiongli-workflow/SKILL.md")).unwrap(),
        b"deliberate synthetic drift"
    );
}

#[cfg(unix)]
#[test]
fn antigravity_refuses_symlink_target_and_source() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let content = qiongli::embedded_content().unwrap();
    let path = f.root.join("qiongli-antigravity");
    symlink(&f.root, &path).unwrap();
    assert!(approve_antigravity_plugin_bundle_target(&path).is_err());
    fs::remove_file(&path).unwrap();
    let target = approve_antigravity_plugin_bundle_target(&path).unwrap();
    let alias = f.root.join("binary-link");
    symlink(&f.binary, &alias).unwrap();
    assert!(
        compose_local_antigravity_plugin_source(
            content.pack(),
            &alias,
            &f.digest(),
            &target,
            None,
            None,
            None
        )
        .is_err()
    );
    assert!(!path.exists());
}

#[cfg(unix)]
#[test]
fn antigravity_private_cache_preserves_strict_source_and_drift_checks() {
    use qiongli_platform::verify_cached_antigravity_plugin_source;
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    // This test checks receipt/file policy, not native execution. Actual PTY uses the real binary.
    fs::write(&f.binary, b"#!/bin/sh\nexit 0\n").unwrap();
    let content = qiongli::embedded_content().unwrap();
    let path = f.root.join("qiongli-antigravity");
    let target = approve_antigravity_plugin_bundle_target(&path).unwrap();
    let saved = compose_local_antigravity_plugin_source(
        content.pack(),
        &f.binary,
        &f.digest(),
        &target,
        None,
        None,
        None,
    )
    .unwrap();
    fn private_dirs(path: &std::path::Path) {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        for entry in fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                private_dirs(&p);
            }
        }
    }
    private_dirs(&path);
    assert_eq!(
        verify_cached_antigravity_plugin_source(&target).unwrap(),
        saved
    );
    assert!(verify_local_antigravity_plugin_source(&target).is_err());
    for mode in [0o770, 0o707, 0o777] {
        fs::set_permissions(path.join("skills"), fs::Permissions::from_mode(mode)).unwrap();
        assert!(verify_cached_antigravity_plugin_source(&target).is_err());
    }
    fs::set_permissions(path.join("skills"), fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(path.join("bin/qiongli"), fs::Permissions::from_mode(0o700)).unwrap();
    assert!(verify_cached_antigravity_plugin_source(&target).is_err());
    fs::set_permissions(path.join("bin/qiongli"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        path.join("skills/qiongli-workflow/SKILL.md"),
        b"explicit cache drift",
    )
    .unwrap();
    assert!(verify_cached_antigravity_plugin_source(&target).is_err());
}
