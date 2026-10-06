use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::{
    CompatibleProduct, ResourcePackBuildMetadata, WorkflowOverrides,
    approve_materialization_target, build_resource_pack, collect_canonical_sources,
    load_resource_pack, materialize_profile_with_overrides, verify_materialization,
};

const DIRECTORY_ROOTS: [&str; 12] = [
    ".claude-plugin",
    ".codex-plugin",
    "distribution",
    "mcp-contracts",
    "roles",
    "schemas",
    "skills",
    "standards",
    "subjects",
    "templates",
    "venue-profiles",
    "workflow",
];
static NEXT_TREE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn skill_language_localizes_metadata_and_preserves_bodies_and_receipts() {
    use qiongli_content::{materialize_profile_with_language, project_profile_with_language};
    let tree = TestTree::new();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../content/workflow");
    let mut paths = vec![
        "SKILL.md".to_owned(),
        "no-qiongli/SKILL.md".into(),
        "agents/openai.yaml".into(),
        "references/skill-descriptions.json".into(),
    ];
    for entry in fs::read_dir(source.join("workflows")).unwrap() {
        paths.push(format!(
            "workflows/{}",
            entry.unwrap().file_name().to_str().unwrap()
        ));
    }
    for path in paths {
        let target = tree.source.join("workflow").join(&path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(source.join(path), target).unwrap();
    }
    let built = tree.pack();
    let pack = load_resource_pack(built.core_bytes(), built.pack_sha256()).unwrap();
    let mut count = 0;
    for language in ["zh", "en"] {
        let resources = project_profile_with_language(&pack, "full", None, Some(language)).unwrap();
        for r in resources.iter().filter(|r| {
            r.path() == "workflow/SKILL.md"
                || r.path() == "workflow/no-qiongli/SKILL.md"
                || (r.path().starts_with("workflow/workflows/")
                    && !r.path().ends_with("/qiongli.md"))
        }) {
            let original = pack
                .resource_for_profile("full", r.path())
                .unwrap()
                .unwrap();
            let text = std::str::from_utf8(r.bytes()).unwrap();
            assert_eq!(
                text.split_once("\n---\n").unwrap().1,
                std::str::from_utf8(original.bytes())
                    .unwrap()
                    .split_once("\n---\n")
                    .unwrap()
                    .1
            );
            let description = text
                .lines()
                .find(|line| line.starts_with("description: "))
                .unwrap();
            assert_eq!(
                description
                    .chars()
                    .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)),
                language == "zh",
                "{}",
                r.path()
            );
            count += 1;
        }
        let ui = resources
            .iter()
            .find(|r| r.path() == "workflow/agents/openai.yaml")
            .unwrap();
        assert_eq!(
            std::str::from_utf8(ui.bytes()).unwrap().contains("问理"),
            language == "zh"
        );
        let target = approve_materialization_target(tree.root.join("localized")).unwrap();
        let receipt =
            materialize_profile_with_language(&pack, "full", &target, None, Some(language))
                .unwrap();
        assert_eq!(receipt.skill_language.as_deref(), Some(language));
        assert_eq!(verify_materialization(&target).unwrap(), receipt);
    }
    assert_eq!(count, 44);
    assert!(project_profile_with_language(&pack, "full", None, Some("fr")).is_err());
    let target = approve_materialization_target(tree.root.join("localized")).unwrap();
    fs::write(target.path().join("workflow/SKILL.md"), "changed").unwrap();
    assert!(verify_materialization(&target).is_err());
}

struct TestTree {
    root: PathBuf,
    source: PathBuf,
}

impl TestTree {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let id = NEXT_TREE_ID.fetch_add(1, Ordering::Relaxed);
        let requested_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qiongli-content-integration-tests")
            .join(format!(
                "qiongli-workflow-overrides-{}-{nonce}-{id}",
                std::process::id()
            ));
        fs::create_dir_all(&requested_root).expect("test root must be created");
        let root = fs::canonicalize(&requested_root).expect("test root must canonicalize");
        let source = root.join("source");
        fs::create_dir(&source).expect("test source must be created");
        for directory in DIRECTORY_ROOTS {
            fs::create_dir_all(source.join(directory)).expect("canonical directory must exist");
        }
        for (path, content) in [
            ("workflow/SKILL.md", "# Canonical workflow\n"),
            ("skills/method.md", "# Canonical method\n"),
            ("skills-core.md", "core\n"),
            ("skills-summary.md", "summary\n"),
            ("schemas/example.json", "{}\n"),
            ("distribution/plugins.yaml", "plugins: []\n"),
            ("mcp-contracts/tools.json", "{}\n"),
        ] {
            fs::write(source.join(path), content).expect("fixture must be written");
        }
        Self { root, source }
    }

    fn pack(&self) -> qiongli_content::BuiltResourcePack {
        let resources = collect_canonical_sources(&self.source).expect("fixture must collect");
        build_resource_pack(
            &ResourcePackBuildMetadata {
                pack_id: "qiongli-core".to_owned(),
                content_version: "2.0.0-alpha.3".to_owned(),
                source_commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
                compatible_product: CompatibleProduct {
                    minimum: "2.0.0-alpha.1".to_owned(),
                    maximum_exclusive: "3.0.0".to_owned(),
                },
            },
            &resources,
        )
        .expect("fixture pack must build")
    }
}

impl Drop for TestTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn customized_workflow_materialization_remains_exactly_receipt_managed() {
    let tree = TestTree::new();
    let built = tree.pack();
    let pack = load_resource_pack(built.core_bytes(), built.pack_sha256())
        .expect("fixture pack must load");
    let overrides = WorkflowOverrides::new(
        &pack,
        BTreeMap::from([(
            "workflow/SKILL.md".to_owned(),
            b"# Customized workflow\n".to_vec(),
        )]),
    )
    .expect("allowed Markdown override must validate")
    .expect("changed content must produce a variant");
    let target =
        approve_materialization_target(tree.root.join("installed")).expect("target must approve");

    let receipt =
        materialize_profile_with_overrides(&pack, "skill-only", &target, Some(&overrides))
            .expect("customized profile must materialize");

    assert_eq!(
        receipt.workflow_variant_sha256.as_deref(),
        Some(overrides.variant_sha256())
    );
    assert_eq!(
        fs::read(target.path().join("workflow/SKILL.md")).unwrap(),
        b"# Customized workflow\n"
    );
    assert_eq!(verify_materialization(&target).unwrap(), receipt);
}
