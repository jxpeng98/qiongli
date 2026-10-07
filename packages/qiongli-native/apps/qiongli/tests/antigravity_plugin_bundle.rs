#![allow(clippy::disallowed_methods)]
use qiongli_platform::{
    CodexPluginBundleKind, approve_antigravity_plugin_bundle_target,
    approve_codex_plugin_bundle_target, compose_local_antigravity_plugin_source,
    remove_local_antigravity_plugin_source, verify_codex_plugin_bundle,
    verify_local_antigravity_plugin_source,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
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
        #[cfg(windows)]
        qiongli_windows_security::create_owner_only_directory(&root).unwrap();
        #[cfg(not(windows))]
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

fn top_level_skill_entries(
    receipt: &qiongli_platform::CodexPluginBundleReceiptV1,
) -> BTreeSet<String> {
    receipt
        .entries
        .iter()
        .filter_map(|entry| {
            let parts: Vec<_> = entry.path.split('/').collect();
            (parts.len() == 3 && parts[0] == "skills" && parts[2] == "SKILL.md")
                .then(|| entry.path.clone())
        })
        .collect()
}

#[test]
fn antigravity_compacts_entries_but_preserves_shared_resources_and_variants() {
    use qiongli_content::WorkflowOverrides;
    use qiongli_platform::compose_local_codex_plugin_source_with_language;
    use std::collections::BTreeMap;
    let f = Fixture::new();
    // Projection/receipt evidence only; native execution is checked separately.
    fs::write(&f.binary, b"synthetic projection verification binary").unwrap();
    let content = qiongli::embedded_content().unwrap();
    let resource = content
        .pack()
        .resource_for_profile("marketplace-lite", "workflow/SKILL.md")
        .unwrap()
        .unwrap();
    let mut customized = resource.bytes().to_vec();
    customized.extend_from_slice(b"\nCustom AGY variant marker.\n");
    let overrides = WorkflowOverrides::new(
        content.pack(),
        BTreeMap::from([("workflow/SKILL.md".to_owned(), customized)]),
    )
    .unwrap()
    .unwrap();
    for language in ["en", "zh"] {
        let locale_root = f.root.join(language);
        fs::create_dir(&locale_root).unwrap();
        let agy_path = locale_root.join("qiongli-antigravity");
        let agy_target = approve_antigravity_plugin_bundle_target(&agy_path).unwrap();
        let agy = compose_local_antigravity_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &agy_target,
            Some(&overrides),
            None,
            Some(language),
        )
        .unwrap();
        let codex_path = locale_root.join("qiongli");
        let codex_target = approve_codex_plugin_bundle_target(&codex_path).unwrap();
        let codex = compose_local_codex_plugin_source_with_language(
            content.pack(),
            &f.binary,
            &f.digest(),
            &codex_target,
            Some(&overrides),
            None,
            false,
            Some(language),
        )
        .unwrap();
        assert_eq!(
            top_level_skill_entries(agy.receipt()),
            BTreeSet::from([
                "skills/qiongli-workflow/SKILL.md".to_owned(),
                "skills/no-qiongli/SKILL.md".to_owned()
            ])
        );
        assert_eq!(top_level_skill_entries(codex.receipt()).len(), 22);
        assert_eq!(
            agy.receipt().workflow_variant_sha256.as_deref(),
            Some(overrides.variant_sha256())
        );
        assert_eq!(agy.receipt().skill_language.as_deref(), Some(language));
        let skill = fs::read_to_string(agy_path.join("skills/qiongli-workflow/SKILL.md")).unwrap();
        assert!(skill.contains("name: qiongli\n"));
        assert!(skill.contains("Custom AGY variant marker."));
        assert!(skill.contains("## Antigravity workflow entry"));
        assert!(skill.contains("load `workflows/<name>.md`"));
        for entry in &codex.receipt().entries {
            if entry.path.starts_with("skills/qiongli-workflow/")
                && entry.path != "skills/qiongli-workflow/SKILL.md"
                || entry.path == "skills/no-qiongli/SKILL.md"
            {
                assert_eq!(
                    fs::read(agy_path.join(&entry.path)).unwrap(),
                    fs::read(codex_path.join(&entry.path)).unwrap(),
                    "{}",
                    entry.path
                );
            }
        }
        let mcp: serde_json::Value =
            serde_json::from_slice(&fs::read(agy_path.join("mcp_config.json")).unwrap()).unwrap();
        assert_eq!(mcp["mcpServers"].as_object().unwrap().len(), 1);
        assert_eq!(
            verify_local_antigravity_plugin_source(&agy_target).unwrap(),
            agy
        );
    }
}

#[test]
fn antigravity_legacy_expanded_receipt_migrates_with_cas_and_drift_guards() {
    use qiongli_platform::compose_local_codex_plugin_source_with_language;
    let f = Fixture::new();
    // Projection/receipt evidence only; native execution is checked separately.
    fs::write(&f.binary, b"synthetic projection verification binary").unwrap();
    let content = qiongli::embedded_content().unwrap();
    let path = f.root.join("qiongli-antigravity");
    let target = approve_antigravity_plugin_bundle_target(&path).unwrap();
    let compact = compose_local_antigravity_plugin_source(
        content.pack(),
        &f.binary,
        &f.digest(),
        &target,
        None,
        None,
        Some("zh"),
    )
    .unwrap();
    let codex_path = f.root.join("qiongli");
    let codex_target = approve_codex_plugin_bundle_target(&codex_path).unwrap();
    let codex = compose_local_codex_plugin_source_with_language(
        content.pack(),
        &f.binary,
        &f.digest(),
        &codex_target,
        None,
        None,
        false,
        Some("zh"),
    )
    .unwrap();
    // Synthetic legacy shape: copy exact shared projector wrappers and rebind the
    // unchanged receipt schema with its public length-prefixed content-root format.
    let mut legacy = compact.receipt().clone();
    for entry in &codex.receipt().entries {
        if top_level_skill_entries(codex.receipt()).contains(&entry.path)
            && !top_level_skill_entries(compact.receipt()).contains(&entry.path)
        {
            fs::create_dir_all(path.join(&entry.path).parent().unwrap()).unwrap();
            fs::copy(codex_path.join(&entry.path), path.join(&entry.path)).unwrap();
            legacy.entries.push(entry.clone());
        }
    }
    legacy.entries.sort_by(|a, b| a.path.cmp(&b.path));
    let mut hash = Sha256::new();
    hash.update(b"qiongli-codex-plugin-bundle-content-root-v1\0");
    for entry in &legacy.entries {
        let mode = serde_json::to_value(entry.mode).unwrap();
        for field in [entry.path.as_bytes(), mode.as_str().unwrap().as_bytes()] {
            hash.update(u64::try_from(field.len()).unwrap().to_be_bytes());
            hash.update(field);
        }
        hash.update(entry.size_bytes.to_be_bytes());
        hash.update(u64::try_from(entry.sha256.len()).unwrap().to_be_bytes());
        hash.update(entry.sha256.as_bytes());
    }
    legacy.package_content_root_sha256 = format!("{:x}", hash.finalize());
    fs::write(
        path.join(".qiongli-antigravity-plugin-bundle.json"),
        serde_json_canonicalizer::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    let verified = verify_local_antigravity_plugin_source(&target).unwrap();
    assert_eq!(top_level_skill_entries(verified.receipt()).len(), 22);
    assert!(
        compose_local_antigravity_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            None,
            Some(compact.receipt_sha256()),
            None
        )
        .is_err()
    );
    let wrapper = path.join("skills/qiongli-paper-read/SKILL.md");
    let original = fs::read(&wrapper).unwrap();
    fs::write(&wrapper, b"legacy wrapper drift").unwrap();
    assert!(
        compose_local_antigravity_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            None,
            Some(verified.receipt_sha256()),
            None
        )
        .is_err()
    );
    assert!(remove_local_antigravity_plugin_source(&target, verified.receipt_sha256()).is_err());
    assert_eq!(fs::read(&wrapper).unwrap(), b"legacy wrapper drift");
    fs::write(&wrapper, original).unwrap();
    let migrated = compose_local_antigravity_plugin_source(
        content.pack(),
        &f.binary,
        &f.digest(),
        &target,
        None,
        Some(verified.receipt_sha256()),
        None,
    )
    .unwrap();
    assert_eq!(top_level_skill_entries(migrated.receipt()).len(), 2);
    assert_eq!(migrated.receipt().skill_language.as_deref(), Some("zh"));
    assert!(!wrapper.exists());
    remove_local_antigravity_plugin_source(&target, migrated.receipt_sha256()).unwrap();
}
