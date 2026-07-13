use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::{
    CompatibleProduct, RESOURCE_PACK_HEADER_LEN, RESOURCE_PACK_MAGIC, ResourceKind,
    ResourcePackBuildMetadata, ResourcePackLimits, ResourcePackLoaderError, build_resource_pack,
    collect_canonical_sources, load_resource_pack, load_resource_pack_with_limits,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

const DIRECTORY_ROOTS: [&str; 10] = [
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
            "qiongli-content-loader-{}-{nonce}-{tree_id}",
            std::process::id(),
        ));
        fs::create_dir(&root).expect("test content root must be created");
        for directory in DIRECTORY_ROOTS {
            fs::create_dir_all(root.join(directory))
                .expect("canonical test directory must be created");
        }
        for (path, bytes) in [
            ("skills-core.md", b"core".as_slice()),
            ("skills-summary.md", b"summary".as_slice()),
            ("skills/example.md", b"alpha".as_slice()),
            ("schemas/example.json", b"{}".as_slice()),
            ("distribution/plugins.yaml", b"plugins: []\n".as_slice()),
            ("mcp-contracts/tools.json", b"{}".as_slice()),
        ] {
            fs::write(root.join(path), bytes).expect("loader fixture must be written");
        }
        Self { root }
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

fn built_pack() -> qiongli_content::BuiltResourcePack {
    let tree = TestTree::new();
    let resources = collect_canonical_sources(&tree.root).expect("test content must collect");
    build_resource_pack(&metadata(), &resources).expect("test pack must build")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn payload_start(core: &[u8]) -> usize {
    let manifest_len = u64::from_le_bytes(
        core[12..RESOURCE_PACK_HEADER_LEN]
            .try_into()
            .expect("manifest length width"),
    );
    RESOURCE_PACK_HEADER_LEN + usize::try_from(manifest_len).expect("fixture manifest length")
}

fn replace_manifest(core: &[u8], manifest_bytes: &[u8]) -> Vec<u8> {
    let payload = &core[payload_start(core)..];
    let mut rebuilt =
        Vec::with_capacity(RESOURCE_PACK_HEADER_LEN + manifest_bytes.len() + payload.len());
    rebuilt.extend_from_slice(&core[..12]);
    rebuilt.extend_from_slice(
        &u64::try_from(manifest_bytes.len())
            .expect("fixture manifest length")
            .to_le_bytes(),
    );
    rebuilt.extend_from_slice(manifest_bytes);
    rebuilt.extend_from_slice(payload);
    rebuilt
}

#[test]
fn loads_lists_and_reads_profile_resources_without_materializing() {
    let built = built_pack();
    let loaded = load_resource_pack(built.core_bytes(), built.pack_sha256())
        .expect("deterministic pack must load");

    assert_eq!(loaded.core_bytes(), built.core_bytes());
    assert_eq!(loaded.manifest_bytes(), built.manifest_bytes());
    assert_eq!(loaded.manifest(), built.manifest());
    assert_eq!(loaded.pack_sha256(), built.pack_sha256());

    let lite = loaded
        .resources_for_profile("lite")
        .expect("lite alias must resolve");
    assert_eq!(lite.len(), loaded.manifest().entries.len());
    let skill = lite
        .iter()
        .find(|resource| resource.entry().path == "skills/example.md")
        .expect("skill resource must be listed");
    assert_eq!(skill.bytes(), b"alpha");

    let skill_only = loaded
        .resources_for_profile("skill-only")
        .expect("skill-only profile must resolve");
    assert_eq!(skill_only.len(), 3);
    assert!(skill_only.iter().all(|resource| {
        !matches!(
            resource.entry().resource_kind,
            ResourceKind::TargetMetadata | ResourceKind::McpContract | ResourceKind::Schema
        )
    }));
    assert!(matches!(
        loaded.resources_for_profile("unknown"),
        Err(ResourcePackLoaderError::InvalidProfile(_))
    ));
}

#[test]
fn expected_whole_pack_digest_is_required_and_verified_first() {
    let built = built_pack();
    assert!(matches!(
        load_resource_pack(built.core_bytes(), "not-a-digest"),
        Err(ResourcePackLoaderError::InvalidExpectedPackSha256)
    ));

    let mut corrupt = built.core_bytes().to_vec();
    *corrupt.last_mut().expect("fixture payload byte") ^= 0x01;
    assert!(matches!(
        load_resource_pack(&corrupt, built.pack_sha256()),
        Err(ResourcePackLoaderError::PackDigestMismatch)
    ));
}

#[test]
fn header_and_manifest_corruption_fail_closed() {
    let built = built_pack();

    let truncated = &built.core_bytes()[..RESOURCE_PACK_HEADER_LEN - 1];
    assert!(matches!(
        load_resource_pack(truncated, &sha256_hex(truncated)),
        Err(ResourcePackLoaderError::TruncatedHeader { .. })
    ));

    let mut invalid_magic = built.core_bytes().to_vec();
    invalid_magic[0] ^= 0x01;
    assert!(matches!(
        load_resource_pack(&invalid_magic, &sha256_hex(&invalid_magic)),
        Err(ResourcePackLoaderError::InvalidMagic)
    ));

    let mut unsupported_version = built.core_bytes().to_vec();
    unsupported_version[RESOURCE_PACK_MAGIC.len()..12].copy_from_slice(&2_u32.to_le_bytes());
    assert!(matches!(
        load_resource_pack(&unsupported_version, &sha256_hex(&unsupported_version),),
        Err(ResourcePackLoaderError::UnsupportedFormatVersion { found: 2 })
    ));

    let mut truncated_manifest = built.core_bytes().to_vec();
    let impossible_len = u64::try_from(truncated_manifest.len()).expect("fixture length");
    truncated_manifest[12..RESOURCE_PACK_HEADER_LEN].copy_from_slice(&impossible_len.to_le_bytes());
    assert!(matches!(
        load_resource_pack(&truncated_manifest, &sha256_hex(&truncated_manifest)),
        Err(ResourcePackLoaderError::TruncatedManifest { .. })
    ));
}

#[test]
fn noncanonical_manifest_and_identity_mutations_fail_closed() {
    let built = built_pack();

    let pretty_manifest = serde_json::to_vec_pretty(built.manifest()).expect("pretty manifest");
    let noncanonical = replace_manifest(built.core_bytes(), &pretty_manifest);
    assert!(matches!(
        load_resource_pack(&noncanonical, &sha256_hex(&noncanonical)),
        Err(ResourcePackLoaderError::NonCanonicalManifest)
    ));

    let mut value: Value =
        serde_json::from_slice(built.manifest_bytes()).expect("fixture manifest JSON");
    value["content_root_sha256"] = Value::String("0".repeat(64));
    let changed_manifest =
        serde_json_canonicalizer::to_vec(&value).expect("changed manifest must canonicalize");
    let changed_root = replace_manifest(built.core_bytes(), &changed_manifest);
    assert!(matches!(
        load_resource_pack(&changed_root, &sha256_hex(&changed_root)),
        Err(ResourcePackLoaderError::ContentRootMismatch)
    ));

    let mut changed_payload = built.core_bytes().to_vec();
    *changed_payload.last_mut().expect("fixture payload byte") ^= 0x01;
    assert!(matches!(
        load_resource_pack(&changed_payload, &sha256_hex(&changed_payload)),
        Err(ResourcePackLoaderError::EntryDigestMismatch { .. })
    ));

    let mut trailing_payload = built.core_bytes().to_vec();
    trailing_payload.push(0);
    assert!(matches!(
        load_resource_pack(&trailing_payload, &sha256_hex(&trailing_payload)),
        Err(ResourcePackLoaderError::PayloadLengthMismatch { .. })
    ));
}

#[test]
fn nonportable_and_colliding_manifest_paths_fail_closed() {
    let built = built_pack();

    let mut value: Value =
        serde_json::from_slice(built.manifest_bytes()).expect("fixture manifest JSON");
    value["entries"][0]["path"] = Value::String("CON".to_string());
    let invalid_path_manifest =
        serde_json_canonicalizer::to_vec(&value).expect("invalid-path manifest must canonicalize");
    let invalid_path = replace_manifest(built.core_bytes(), &invalid_path_manifest);
    assert!(matches!(
        load_resource_pack(&invalid_path, &sha256_hex(&invalid_path)),
        Err(ResourcePackLoaderError::InvalidEntryPath { .. })
    ));

    let mut value: Value =
        serde_json::from_slice(built.manifest_bytes()).expect("fixture manifest JSON");
    value["entries"][0]["path"] = Value::String("distribution/A.md".to_string());
    value["entries"][1]["path"] = Value::String("distribution/a.md".to_string());
    value["entries"][1]["resource_kind"] = Value::String("target-metadata".to_string());
    let collision_manifest =
        serde_json_canonicalizer::to_vec(&value).expect("collision manifest must canonicalize");
    let collision = replace_manifest(built.core_bytes(), &collision_manifest);
    assert!(matches!(
        load_resource_pack(&collision, &sha256_hex(&collision)),
        Err(ResourcePackLoaderError::PortablePathCollision { .. })
    ));
}

#[test]
fn paths_cannot_escape_the_canonical_roots_or_spoof_profile_kinds() {
    let built = built_pack();

    let mut value: Value =
        serde_json::from_slice(built.manifest_bytes()).expect("fixture manifest JSON");
    let entries = value["entries"]
        .as_array_mut()
        .expect("fixture entries array");
    entries
        .last_mut()
        .expect("fixture entry")
        .as_object_mut()
        .expect("fixture entry object")
        .insert(
            "path".to_string(),
            Value::String("zz-outside/example.md".to_string()),
        );
    let outside_manifest =
        serde_json_canonicalizer::to_vec(&value).expect("outside manifest must canonicalize");
    let outside = replace_manifest(built.core_bytes(), &outside_manifest);
    assert!(matches!(
        load_resource_pack(&outside, &sha256_hex(&outside)),
        Err(ResourcePackLoaderError::EntryOutsideCanonicalSources { .. })
    ));

    let mut value: Value =
        serde_json::from_slice(built.manifest_bytes()).expect("fixture manifest JSON");
    let entries = value["entries"]
        .as_array_mut()
        .expect("fixture entries array");
    let distribution = entries
        .iter_mut()
        .find(|entry| {
            entry["path"]
                .as_str()
                .is_some_and(|path| path.starts_with("distribution/"))
        })
        .expect("distribution fixture entry");
    distribution["resource_kind"] = Value::String("skill".to_string());
    let spoofed_manifest =
        serde_json_canonicalizer::to_vec(&value).expect("spoofed manifest must canonicalize");
    let spoofed = replace_manifest(built.core_bytes(), &spoofed_manifest);
    assert!(matches!(
        load_resource_pack(&spoofed, &sha256_hex(&spoofed)),
        Err(ResourcePackLoaderError::ResourceKindMismatch { .. })
    ));
}

#[test]
fn configured_limits_are_enforced_before_content_is_exposed() {
    let built = built_pack();
    let default_limits = ResourcePackLimits::default();

    let mut limits = default_limits;
    limits.max_pack_bytes = u64::try_from(built.core_bytes().len() - 1).expect("fixture length");
    assert!(matches!(
        load_resource_pack_with_limits(built.core_bytes(), built.pack_sha256(), limits),
        Err(ResourcePackLoaderError::PackTooLarge { .. })
    ));

    let mut limits = default_limits;
    limits.max_manifest_bytes =
        u64::try_from(built.manifest_bytes().len() - 1).expect("fixture manifest length");
    assert!(matches!(
        load_resource_pack_with_limits(built.core_bytes(), built.pack_sha256(), limits),
        Err(ResourcePackLoaderError::ManifestTooLarge { .. })
    ));

    let mut limits = default_limits;
    limits.max_entries = built.manifest().entries.len() - 1;
    assert!(matches!(
        load_resource_pack_with_limits(built.core_bytes(), built.pack_sha256(), limits),
        Err(ResourcePackLoaderError::EntryLimitExceeded { .. })
    ));

    let mut limits = default_limits;
    limits.max_entry_bytes = 1;
    assert!(matches!(
        load_resource_pack_with_limits(built.core_bytes(), built.pack_sha256(), limits),
        Err(ResourcePackLoaderError::EntryTooLarge { .. })
    ));

    let mut limits = default_limits;
    limits.max_payload_bytes = 1;
    assert!(matches!(
        load_resource_pack_with_limits(built.core_bytes(), built.pack_sha256(), limits),
        Err(ResourcePackLoaderError::PayloadTooLarge { .. })
    ));

    let mut limits = default_limits;
    limits.max_path_depth = 1;
    assert!(matches!(
        load_resource_pack_with_limits(built.core_bytes(), built.pack_sha256(), limits),
        Err(ResourcePackLoaderError::PathDepthExceeded { .. })
    ));
}

#[test]
fn invalid_zero_limits_fail_before_parsing() {
    let built = built_pack();
    let limits = ResourcePackLimits {
        max_entries: 0,
        ..ResourcePackLimits::default()
    };

    assert!(matches!(
        load_resource_pack_with_limits(built.core_bytes(), built.pack_sha256(), limits),
        Err(ResourcePackLoaderError::InvalidLimits("max_entries"))
    ));
}
