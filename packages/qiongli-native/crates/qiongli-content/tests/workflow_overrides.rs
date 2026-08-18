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
