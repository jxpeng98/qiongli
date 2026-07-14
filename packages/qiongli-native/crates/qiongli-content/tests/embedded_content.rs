use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::{
    CompatibleProduct, EmbeddedContent, ProfileId, ResourcePackBuildMetadata,
    ResourcePackLoaderError, build_resource_pack, collect_canonical_sources,
    temporary_materialization_target,
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
}

impl TestTree {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let tree_id = NEXT_TREE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "qiongli-embedded-content-{}-{nonce}-{tree_id}",
            std::process::id(),
        ));
        fs::create_dir(&root).expect("test content root must be created");
        for directory in DIRECTORY_ROOTS {
            fs::create_dir_all(root.join(directory))
                .expect("canonical test directory must be created");
        }
        fs::write(root.join("skills-core.md"), b"core")
            .expect("skills-core fixture must be written");
        fs::write(root.join("skills-summary.md"), b"summary")
            .expect("skills-summary fixture must be written");
        fs::write(root.join("skills/example.md"), b"academic workflow")
            .expect("skill fixture must be written");
        fs::write(root.join("mcp-contracts/registry.json"), b"{}")
            .expect("contract fixture must be written");
        Self { root }
    }

    fn build_static(&self) -> (&'static [u8], String) {
        let resources = collect_canonical_sources(&self.root).expect("test content must collect");
        let built = build_resource_pack(
            &ResourcePackBuildMetadata {
                pack_id: "qiongli-core".to_string(),
                content_version: "1.19.0-beta.1".to_string(),
                source_commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
                compatible_product: CompatibleProduct {
                    minimum: "2.0.0-alpha.1".to_string(),
                    maximum_exclusive: "3.0.0".to_string(),
                },
            },
            &resources,
        )
        .expect("test pack must build");
        let digest = built.pack_sha256().to_string();
        (Box::leak(built.into_core_bytes()), digest)
    }
}

impl Drop for TestTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn embedded_service_lists_reads_and_materializes_profiles() {
    let tree = TestTree::new();
    let (bytes, digest) = tree.build_static();
    let content = EmbeddedContent::load(bytes, &digest).expect("embedded pack must verify");

    assert_eq!(
        content
            .profiles()
            .iter()
            .map(|profile| profile.id)
            .collect::<Vec<_>>(),
        vec![
            ProfileId::SkillOnly,
            ProfileId::MarketplaceLite,
            ProfileId::Full,
        ]
    );
    assert_eq!(
        content
            .read_profile_resource("skill-only", "skills/example.md")
            .expect("known profile must resolve")
            .expect("selected skill must exist")
            .bytes(),
        b"academic workflow"
    );
    assert!(
        content
            .read_profile_resource("skill-only", "mcp-contracts/registry.json")
            .expect("known profile must resolve")
            .is_none()
    );
    assert!(matches!(
        content.read_profile_resource("unknown", "skills/example.md"),
        Err(ResourcePackLoaderError::InvalidProfile(_))
    ));

    let target = temporary_materialization_target().expect("temporary target must be approved");
    let container = target
        .path()
        .parent()
        .expect("temporary target must have a private container")
        .to_path_buf();
    let receipt = content
        .materialize_profile("skill-only", &target)
        .expect("embedded profile must materialize");
    assert_eq!(receipt.profile, ProfileId::SkillOnly);
    assert_eq!(
        fs::read(target.path().join("skills/example.md"))
            .expect("materialized skill must be readable"),
        b"academic workflow"
    );
    fs::remove_dir_all(container).expect("temporary materialization must be removed");
}
