use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::{
    CompatibleProduct, LogicalMode, RESOURCE_PACK_FORMAT_VERSION, RESOURCE_PACK_HEADER_LEN,
    RESOURCE_PACK_MAGIC, ResourcePackBuildMetadata, ResourcePackLockError, ResourcePackLockV1,
    ResourcePackManifestV1, ResourcePackWriterError, build_resource_pack,
    collect_canonical_sources,
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
            "qiongli-content-writer-{}-{nonce}-{tree_id}",
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
        fs::write(root.join("skills/example.md"), b"alpha").expect("skill fixture must be written");
        Self { root }
    }

    fn rewrite_skill(&self, bytes: &[u8]) {
        fs::write(self.root.join("skills/example.md"), bytes)
            .expect("skill fixture must be rewritten");
    }

    fn collect(&self) -> Vec<qiongli_content::CollectedResource> {
        collect_canonical_sources(&self.root).expect("test content must collect")
    }
}

impl Drop for TestTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn metadata() -> ResourcePackBuildMetadata {
    ResourcePackBuildMetadata {
        pack_id: "qiongli-core".to_string(),
        content_version: "1.19.0-beta.1".to_string(),
        source_commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
        compatible_product: CompatibleProduct {
            minimum: "2.0.0-alpha.1".to_string(),
            maximum_exclusive: "3.0.0".to_string(),
        },
    }
}

#[test]
fn repeated_and_reordered_builds_are_byte_identical() {
    let tree = TestTree::new();
    let resources = tree.collect();
    let mut reversed = resources.clone();
    reversed.reverse();

    let first = build_resource_pack(&metadata(), &resources).expect("first pack must build");
    let second = build_resource_pack(&metadata(), &reversed).expect("second pack must build");

    assert_eq!(first.core_bytes(), second.core_bytes());
    assert_eq!(first.manifest_bytes(), second.manifest_bytes());
    assert_eq!(first.manifest(), second.manifest());
    assert_eq!(first.pack_sha256(), second.pack_sha256());
    assert_eq!(
        first.manifest().content_root_sha256,
        second.manifest().content_root_sha256
    );
}

#[test]
fn header_manifest_and_payload_use_the_frozen_v1_layout() {
    let tree = TestTree::new();
    let resources = tree.collect();
    let built = build_resource_pack(&metadata(), &resources).expect("pack must build");
    let core = built.core_bytes();

    assert_eq!(&core[..RESOURCE_PACK_MAGIC.len()], &RESOURCE_PACK_MAGIC);
    let version_start = RESOURCE_PACK_MAGIC.len();
    let version_end = version_start + size_of::<u32>();
    let version = u32::from_le_bytes(
        core[version_start..version_end]
            .try_into()
            .expect("version width"),
    );
    assert_eq!(version, RESOURCE_PACK_FORMAT_VERSION);

    let length_end = version_end + size_of::<u64>();
    let manifest_len = u64::from_le_bytes(
        core[version_end..length_end]
            .try_into()
            .expect("manifest length width"),
    );
    assert_eq!(length_end, RESOURCE_PACK_HEADER_LEN);
    assert_eq!(manifest_len, built.manifest_bytes().len() as u64);

    let manifest_end = RESOURCE_PACK_HEADER_LEN + built.manifest_bytes().len();
    assert_eq!(
        &core[RESOURCE_PACK_HEADER_LEN..manifest_end],
        built.manifest_bytes()
    );
    let expected_payload = resources
        .iter()
        .flat_map(|resource| resource.bytes().iter().copied())
        .collect::<Vec<_>>();
    assert_eq!(&core[manifest_end..], expected_payload);

    let manifest_text =
        std::str::from_utf8(built.manifest_bytes()).expect("manifest must be UTF-8");
    assert!(manifest_text.starts_with(
        r#"{"compatible_product":{"maximum_exclusive":"3.0.0","minimum":"2.0.0-alpha.1"},"compiler_contract_version":1,"content_root_sha256":"#
    ));
    assert!(!manifest_text.contains('\n'));
    assert!(!manifest_text.contains(tree.root.to_string_lossy().as_ref()));
    assert_eq!(
        ResourcePackManifestV1::from_json(manifest_text).expect("manifest must parse"),
        *built.manifest()
    );
}

#[test]
fn content_or_declared_mode_changes_entry_root_and_pack_digests() {
    let tree = TestTree::new();
    let original_resources = tree.collect();
    let original =
        build_resource_pack(&metadata(), &original_resources).expect("original pack must build");

    tree.rewrite_skill(b"bravo");
    let changed_resources = tree.collect();
    let changed =
        build_resource_pack(&metadata(), &changed_resources).expect("changed pack must build");

    let original_entry = original
        .manifest()
        .entries
        .iter()
        .find(|entry| entry.path == "skills/example.md")
        .expect("original skill entry");
    let changed_entry = changed
        .manifest()
        .entries
        .iter()
        .find(|entry| entry.path == "skills/example.md")
        .expect("changed skill entry");
    assert_eq!(original_entry.size_bytes, changed_entry.size_bytes);
    assert_eq!(original_entry.payload_offset, changed_entry.payload_offset);
    assert_ne!(original_entry.sha256, changed_entry.sha256);
    assert_ne!(
        original.manifest().content_root_sha256,
        changed.manifest().content_root_sha256
    );
    assert_ne!(original.pack_sha256(), changed.pack_sha256());

    let mut executable_resources = original_resources;
    executable_resources
        .iter_mut()
        .find(|resource| resource.path == "skills/example.md")
        .expect("mutable skill resource")
        .mode = LogicalMode::Executable;
    let executable = build_resource_pack(&metadata(), &executable_resources)
        .expect("mode-adjusted pack must build");
    assert_ne!(
        original.manifest().content_root_sha256,
        executable.manifest().content_root_sha256
    );
    assert_ne!(original.pack_sha256(), executable.pack_sha256());
}

#[test]
fn release_metadata_changes_pack_identity_but_not_content_identity() {
    let tree = TestTree::new();
    let resources = tree.collect();
    let first = build_resource_pack(&metadata(), &resources).expect("first pack must build");
    let mut changed_metadata = metadata();
    changed_metadata.source_commit = "fedcba9876543210fedcba9876543210fedcba98".to_string();
    let second =
        build_resource_pack(&changed_metadata, &resources).expect("second pack must build");

    assert_eq!(
        first.manifest().content_root_sha256,
        second.manifest().content_root_sha256
    );
    assert_ne!(first.manifest_bytes(), second.manifest_bytes());
    assert_ne!(first.pack_sha256(), second.pack_sha256());
}

#[test]
fn invalid_input_fails_before_emitting_a_pack() {
    assert!(matches!(
        build_resource_pack(&metadata(), &[]),
        Err(ResourcePackWriterError::EmptyResources)
    ));

    let tree = TestTree::new();
    let mut resources = tree.collect();
    resources.push(resources[0].clone());
    assert!(matches!(
        build_resource_pack(&metadata(), &resources),
        Err(ResourcePackWriterError::DuplicatePath { .. })
    ));

    let mut invalid_metadata = metadata();
    invalid_metadata.source_commit = "not-a-commit".to_string();
    assert!(matches!(
        build_resource_pack(&invalid_metadata, &tree.collect()),
        Err(ResourcePackWriterError::InvalidManifest(_))
    ));
}

#[test]
fn repository_content_rebuilds_with_identical_bytes_and_hashes() {
    let content_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../content");
    let resources =
        collect_canonical_sources(content_root).expect("repository content must collect");

    let first = build_resource_pack(&metadata(), &resources).expect("first pack must build");
    let second = build_resource_pack(&metadata(), &resources).expect("second pack must build");

    assert_eq!(first.core_bytes(), second.core_bytes());
    assert_eq!(first.pack_sha256(), second.pack_sha256());
    assert_eq!(
        first.manifest().content_root_sha256,
        second.manifest().content_root_sha256
    );
}

#[test]
fn resource_pack_lock_round_trips_and_rejects_source_drift() {
    let tree = TestTree::new();
    let original =
        build_resource_pack(&metadata(), &tree.collect()).expect("original pack must build");
    let lock = ResourcePackLockV1::from_built(&original);
    let canonical_lock = lock
        .to_canonical_json()
        .expect("resource-pack lock must canonicalize");
    let parsed = ResourcePackLockV1::from_json(
        std::str::from_utf8(&canonical_lock).expect("lock must be UTF-8"),
    )
    .expect("canonical resource-pack lock must parse");

    assert_eq!(parsed, lock);
    parsed
        .verify(&original)
        .expect("matching built pack must satisfy its lock");
    assert_eq!(parsed.entry_count, original.manifest().entries.len() as u64);
    assert_eq!(parsed.pack_sha256, original.pack_sha256());

    tree.rewrite_skill(b"bravo");
    let drifted =
        build_resource_pack(&metadata(), &tree.collect()).expect("drifted pack must build");
    assert!(matches!(
        parsed.verify(&drifted),
        Err(ResourcePackLockError::ContentRootMismatch { .. })
    ));
}
