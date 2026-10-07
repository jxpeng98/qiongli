#![allow(clippy::disallowed_methods)]
use qiongli_content::WorkflowOverrides;
use qiongli_platform::{
    approve_codex_plugin_bundle_target, approve_pi_plugin_bundle_target,
    compose_local_codex_plugin_source_with_language, compose_local_pi_plugin_source,
    remove_local_pi_plugin_source, verify_local_pi_plugin_source,
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
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
            .join("target/pi-bundle-tests");
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
        let binary = root.join("synthetic-qiongli");
        // Projection/receipt checks only; the isolated official SDK check uses the real native binary.
        fs::write(&binary, b"synthetic Pi projection verification binary").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        }
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
fn entries(receipt: &qiongli_platform::CodexPluginBundleReceiptV1) -> BTreeSet<String> {
    receipt
        .entries
        .iter()
        .filter(|e| {
            e.path.starts_with("skills/")
                && e.path.split('/').count() == 3
                && e.path.ends_with("/SKILL.md")
        })
        .map(|e| e.path.clone())
        .collect()
}

#[test]
fn pi_compact_projection_retains_resources_locales_variants_and_shared_codex_wrappers() {
    let f = Fixture::new();
    let content = qiongli::embedded_content().unwrap();
    let workflow = content
        .pack()
        .resource_for_profile("marketplace-lite", "workflow/SKILL.md")
        .unwrap()
        .unwrap();
    let mut customized = workflow.bytes().to_vec();
    customized.extend_from_slice(b"\nSynthetic variant canary.\n");
    let overrides = WorkflowOverrides::new(
        content.pack(),
        BTreeMap::from([("workflow/SKILL.md".to_owned(), customized)]),
    )
    .unwrap()
    .unwrap();
    for language in ["en", "zh"] {
        let parent = f.root.join(language);
        fs::create_dir(&parent).unwrap();
        let path = parent.join("qiongli-pi");
        let target = approve_pi_plugin_bundle_target(&path).unwrap();
        let saved = compose_local_pi_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            Some(&overrides),
            None,
            Some(language),
        )
        .unwrap();
        assert_eq!(
            saved.receipt().package_kind,
            qiongli_platform::CodexPluginBundleKind::UserLocalPiFullMcp
        );
        assert_eq!(
            entries(saved.receipt()),
            BTreeSet::from([
                "skills/qiongli-workflow/SKILL.md".to_owned(),
                "skills/no-qiongli/SKILL.md".to_owned()
            ])
        );
        assert_eq!(saved.receipt().skill_language.as_deref(), Some(language));
        assert_eq!(
            saved.receipt().workflow_variant_sha256.as_deref(),
            Some(overrides.variant_sha256())
        );
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("package.json")).unwrap()).unwrap();
        assert_eq!(
            manifest["pi"]["skills"],
            serde_json::json!(["./skills/qiongli-workflow", "./skills/no-qiongli"])
        );
        assert_eq!(
            manifest["pi"]["extensions"],
            serde_json::json!(["./extensions/qiongli-mcp.js"])
        );
        assert!(manifest.get("dependencies").is_none() && manifest.get("scripts").is_none());
        assert!(!path.join(".codex-plugin").exists() && !path.join(".mcp.json").exists());
        let extension = fs::read_to_string(path.join("extensions/qiongli-mcp.js")).unwrap();
        assert!(extension.contains("pi.registerMcpServer("));
        assert!(
            extension.contains(
                &serde_json::to_string(&path.join(&saved.receipt().binary_path)).unwrap()
            )
        );
        assert!(
            extension.contains("\"exposure\":\"deferred\"")
                && extension.contains("\"full\"")
                && extension.contains("\"stdio\"")
        );
        assert!(!extension.contains("import ") && !extension.contains("npm"));
        let skill = fs::read_to_string(path.join("skills/qiongli-workflow/SKILL.md")).unwrap();
        assert!(
            skill.contains("name: qiongli\n")
                && skill.contains("## Pi workflow entry")
                && skill.contains("tool_search")
        );
        let codex_path = parent.join("qiongli");
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
        assert_eq!(entries(codex.receipt()).len(), 22);
        for entry in &codex.receipt().entries {
            if (entry.path.starts_with("skills/qiongli-workflow/")
                && entry.path != "skills/qiongli-workflow/SKILL.md")
                || entry.path == "skills/no-qiongli/SKILL.md"
            {
                assert_eq!(
                    fs::read(path.join(&entry.path)).unwrap(),
                    fs::read(codex_path.join(&entry.path)).unwrap(),
                    "{}",
                    entry.path
                );
            }
        }
        assert_eq!(verify_local_pi_plugin_source(&target).unwrap(), saved);
        let updated = compose_local_pi_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            Some(&overrides),
            Some(saved.receipt_sha256()),
            None,
        )
        .unwrap();
        assert_eq!(updated.receipt().skill_language.as_deref(), Some(language));
        remove_local_pi_plugin_source(&target, updated.receipt_sha256()).unwrap();
        assert!(!path.exists());
    }
}

#[test]
fn pi_receipts_refuse_bad_binary_unmanaged_stale_cas_foreign_host_and_drift() {
    let f = Fixture::new();
    let content = qiongli::embedded_content().unwrap();
    let path = f.root.join("qiongli-pi");
    let target = approve_pi_plugin_bundle_target(&path).unwrap();
    assert!(
        compose_local_pi_plugin_source(
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
    fs::write(path.join("keep"), b"unmanaged canary").unwrap();
    assert!(
        compose_local_pi_plugin_source(
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
    assert_eq!(fs::read(path.join("keep")).unwrap(), b"unmanaged canary");
    fs::remove_dir_all(&path).unwrap();
    let saved = compose_local_pi_plugin_source(
        content.pack(),
        &f.binary,
        &f.digest(),
        &target,
        None,
        None,
        None,
    )
    .unwrap();
    assert!(
        qiongli_platform::verify_codex_plugin_bundle(
            &approve_codex_plugin_bundle_target(&path).unwrap()
        )
        .is_err()
    );
    assert!(
        compose_local_pi_plugin_source(
            content.pack(),
            &f.binary,
            &f.digest(),
            &target,
            None,
            Some(&"0".repeat(64)),
            None
        )
        .is_err()
    );
    assert!(remove_local_pi_plugin_source(&target, &"0".repeat(64)).is_err());
    assert_eq!(verify_local_pi_plugin_source(&target).unwrap(), saved);
    let extension = path.join("extensions/qiongli-mcp.js");
    fs::write(&extension, b"deliberate extension drift").unwrap();
    assert!(verify_local_pi_plugin_source(&target).is_err());
    assert!(
        compose_local_pi_plugin_source(
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
    assert!(remove_local_pi_plugin_source(&target, saved.receipt_sha256()).is_err());
    assert_eq!(fs::read(&extension).unwrap(), b"deliberate extension drift");
}

#[cfg(unix)]
#[test]
fn pi_refuses_symlink_destination_and_binary() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let content = qiongli::embedded_content().unwrap();
    let path = f.root.join("qiongli-pi");
    symlink(&f.root, &path).unwrap();
    assert!(approve_pi_plugin_bundle_target(&path).is_err());
    fs::remove_file(&path).unwrap();
    let target = approve_pi_plugin_bundle_target(&path).unwrap();
    let alias = f.root.join("binary-link");
    symlink(&f.binary, &alias).unwrap();
    assert!(
        compose_local_pi_plugin_source(
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
