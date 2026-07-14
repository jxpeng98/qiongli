use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::{
    CompatibleProduct, LogicalMode, MATERIALIZATION_RECEIPT_FILE, MATERIALIZATION_RECEIPT_VERSION,
    MaterializationAuthorization, MaterializationError, ProfileId, ResourcePackBuildMetadata,
    approve_materialization_target, build_resource_pack, collect_canonical_sources,
    load_resource_pack, materialize_profile, temporary_materialization_target,
    verify_materialization,
};

const DIRECTORY_ROOTS: [&str; 11] = [
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
        let tree_id = NEXT_TREE_ID.fetch_add(1, Ordering::Relaxed);
        let test_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qiongli-content-integration-tests");
        fs::create_dir_all(&test_base).expect("materializer test base must be created");
        let requested_root = test_base.join(format!(
            "qiongli-content-materializer-{}-{nonce}-{tree_id}",
            std::process::id(),
        ));
        fs::create_dir(&requested_root).expect("test root must be created");
        let root = fs::canonicalize(&requested_root).expect("test root must canonicalize");
        let source = root.join("source");
        fs::create_dir(&source).expect("test content root must be created");
        for directory in DIRECTORY_ROOTS {
            fs::create_dir_all(source.join(directory))
                .expect("canonical test directory must be created");
        }
        for (path, bytes) in [
            ("skills-core.md", b"core".as_slice()),
            ("skills-summary.md", b"summary".as_slice()),
            ("skills/example.md", b"alpha".as_slice()),
            ("skills/old-only.md", b"old".as_slice()),
            ("schemas/example.json", b"{}".as_slice()),
            ("distribution/plugins.yaml", b"plugins: []\n".as_slice()),
            ("mcp-contracts/tools.json", b"{}".as_slice()),
            ("workflow/run.sh", b"#!/bin/sh\n".as_slice()),
        ] {
            fs::write(source.join(path), bytes).expect("materializer fixture must be written");
        }
        Self { root, source }
    }

    fn target(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn build(&self, content_version: &str) -> qiongli_content::BuiltResourcePack {
        let mut resources =
            collect_canonical_sources(&self.source).expect("test content must collect");
        resources
            .iter_mut()
            .find(|resource| resource.path == "workflow/run.sh")
            .expect("workflow fixture must collect")
            .mode = LogicalMode::Executable;
        build_resource_pack(
            &ResourcePackBuildMetadata {
                pack_id: "qiongli-core".to_string(),
                content_version: content_version.to_string(),
                source_commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
                compatible_product: CompatibleProduct {
                    minimum: "2.0.0-alpha.1".to_string(),
                    maximum_exclusive: "3.0.0".to_string(),
                },
            },
            &resources,
        )
        .expect("test pack must build")
    }
}

impl Drop for TestTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn temporary_targets_are_unique_and_rooted_in_the_canonical_os_temp_directory() {
    let first = temporary_materialization_target().expect("temporary target must resolve");
    let second = temporary_materialization_target().expect("temporary target must resolve");
    let canonical_temp =
        fs::canonicalize(std::env::temp_dir()).expect("temporary directory must canonicalize");

    assert!(first.path().starts_with(&canonical_temp));
    assert!(second.path().starts_with(&canonical_temp));
    assert_ne!(first.path(), second.path());
    assert_eq!(
        first.authorization(),
        MaterializationAuthorization::Temporary
    );
    assert!(!first.path().exists());
    assert!(!second.path().exists());
    assert_private_temp_parent(first.path().parent().unwrap());
    assert_private_temp_parent(second.path().parent().unwrap());
    fs::remove_dir(first.path().parent().unwrap()).expect("first private temp parent must clean");
    fs::remove_dir(second.path().parent().unwrap()).expect("second private temp parent must clean");
}

#[test]
fn caller_selected_targets_require_explicit_absolute_normalized_approval() {
    let tree = TestTree::new();
    let approved = approve_materialization_target(tree.target("install"))
        .expect("absolute target must be explicitly approved");

    assert_eq!(approved.path(), tree.target("install"));
    assert_eq!(
        approved.authorization(),
        MaterializationAuthorization::ExplicitlyApproved
    );
    assert!(matches!(
        approve_materialization_target(Path::new("relative/install")),
        Err(MaterializationError::InvalidTarget { .. })
    ));
    assert!(matches!(
        approve_materialization_target(absolute_traversal_target(&tree.root)),
        Err(MaterializationError::InvalidTarget { .. })
    ));
}

#[cfg(windows)]
fn absolute_traversal_target(_root: &Path) -> PathBuf {
    PathBuf::from(r"C:\qiongli-child\..\qiongli-install")
}

#[cfg(not(windows))]
fn absolute_traversal_target(root: &Path) -> PathBuf {
    root.join("child/../install")
}

#[cfg(unix)]
#[test]
fn explicitly_approved_targets_reject_group_or_world_writable_parents() {
    use std::os::unix::fs::PermissionsExt;

    let tree = TestTree::new();
    let insecure_parent = tree.target("shared");
    fs::create_dir(&insecure_parent).expect("shared parent must be created");
    fs::set_permissions(&insecure_parent, fs::Permissions::from_mode(0o777))
        .expect("shared parent mode must be set");

    assert!(matches!(
        approve_materialization_target(insecure_parent.join("install")),
        Err(MaterializationError::InsecureTargetParent { .. })
    ));
}

#[test]
fn materializes_only_the_selected_profile_with_a_canonical_managed_receipt() {
    let tree = TestTree::new();
    let built = tree.build("1.19.0-beta.1");
    let loaded =
        load_resource_pack(built.core_bytes(), built.pack_sha256()).expect("test pack must load");
    let target = approve_materialization_target(tree.target("install"))
        .expect("test target must be approved");

    let receipt = materialize_profile(&loaded, "skill-only", &target)
        .expect("verified profile must materialize");

    assert_eq!(
        verify_materialization(&target).expect("managed tree must verify read-only"),
        receipt
    );

    assert_eq!(receipt.receipt_version, MATERIALIZATION_RECEIPT_VERSION);
    assert_eq!(receipt.pack_id, "qiongli-core");
    assert_eq!(receipt.content_version, "1.19.0-beta.1");
    assert_eq!(receipt.profile, ProfileId::SkillOnly);
    assert_eq!(receipt.pack_sha256, built.pack_sha256());
    assert_eq!(
        receipt.content_root_sha256,
        built.manifest().content_root_sha256
    );
    assert_eq!(
        receipt.authorization,
        MaterializationAuthorization::ExplicitlyApproved
    );
    assert_eq!(
        fs::read(target.path().join("skills/example.md")).unwrap(),
        b"alpha"
    );
    assert_eq!(
        fs::read(target.path().join("workflow/run.sh")).unwrap(),
        b"#!/bin/sh\n"
    );
    assert!(!target.path().join("schemas/example.json").exists());
    assert!(!target.path().join("distribution/plugins.yaml").exists());

    let receipt_bytes = fs::read(target.path().join(MATERIALIZATION_RECEIPT_FILE))
        .expect("managed receipt must be written");
    assert!(!receipt_bytes.ends_with(b"\n"));
    assert_eq!(
        receipt_bytes,
        serde_json_canonicalizer::to_vec(&receipt).expect("receipt must canonicalize")
    );
    assert_eq!(
        receipt
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "skills-core.md",
            "skills-summary.md",
            "skills/example.md",
            "skills/old-only.md",
            "workflow/run.sh",
        ]
    );

    assert_logical_modes(target.path());
}

#[test]
fn refuses_unmanaged_and_drifted_targets_without_overwriting_prior_bytes() {
    let tree = TestTree::new();
    let built = tree.build("1.19.0-beta.1");
    let loaded =
        load_resource_pack(built.core_bytes(), built.pack_sha256()).expect("test pack must load");

    let unmanaged_path = tree.target("unmanaged");
    fs::create_dir(&unmanaged_path).expect("unmanaged target must be created");
    fs::write(unmanaged_path.join("keep.txt"), b"keep").expect("unmanaged fixture must be written");
    let unmanaged = approve_materialization_target(&unmanaged_path)
        .expect("unmanaged target path may still be approved");
    assert!(matches!(
        materialize_profile(&loaded, "full", &unmanaged),
        Err(MaterializationError::UnmanagedTarget { .. })
    ));
    assert_eq!(fs::read(unmanaged_path.join("keep.txt")).unwrap(), b"keep");

    let managed = approve_materialization_target(tree.target("managed"))
        .expect("managed target must be approved");
    materialize_profile(&loaded, "full", &managed).expect("initial write must succeed");
    fs::write(managed.path().join("skills/example.md"), b"tampered")
        .expect("managed fixture must drift");
    assert!(matches!(
        materialize_profile(&loaded, "full", &managed),
        Err(MaterializationError::ManagedTargetDrift { .. })
    ));
    assert_eq!(
        fs::read(managed.path().join("skills/example.md")).unwrap(),
        b"tampered"
    );
}

#[test]
fn profile_preflight_failure_leaves_the_target_absent() {
    let tree = TestTree::new();
    let built = tree.build("1.19.0-beta.1");
    let loaded =
        load_resource_pack(built.core_bytes(), built.pack_sha256()).expect("test pack must load");
    let target = approve_materialization_target(tree.target("install"))
        .expect("test target must be approved");

    assert!(matches!(
        materialize_profile(&loaded, "unknown", &target),
        Err(MaterializationError::Profile(_))
    ));
    assert!(!target.path().exists());
    assert_eq!(
        fs::read_dir(&tree.root)
            .expect("test root must list")
            .count(),
        1
    );
}

#[test]
fn replaces_a_valid_managed_tree_without_staging_or_backup_residue() {
    let tree = TestTree::new();
    let target = approve_materialization_target(tree.target("install"))
        .expect("test target must be approved");
    let first = tree.build("1.19.0-beta.1");
    let first_loaded = load_resource_pack(first.core_bytes(), first.pack_sha256())
        .expect("first test pack must load");
    materialize_profile(&first_loaded, "full", &target).expect("first write must succeed");

    fs::write(tree.source.join("skills/example.md"), b"bravo").expect("source fixture must change");
    fs::remove_file(tree.source.join("skills/old-only.md"))
        .expect("old-only source must be removed");
    let second = tree.build("1.19.0-beta.2");
    let second_loaded = load_resource_pack(second.core_bytes(), second.pack_sha256())
        .expect("second test pack must load");

    let receipt = materialize_profile(&second_loaded, "full", &target)
        .expect("managed replacement must succeed");

    assert_eq!(receipt.content_version, "1.19.0-beta.2");
    assert_eq!(
        fs::read(target.path().join("skills/example.md")).unwrap(),
        b"bravo"
    );
    assert!(!target.path().join("skills/old-only.md").exists());
    let mut siblings = fs::read_dir(&tree.root)
        .expect("test root must list")
        .map(|entry| entry.expect("test sibling must read").file_name())
        .collect::<Vec<_>>();
    siblings.sort();
    assert_eq!(siblings, vec!["install", "source"]);
}

#[test]
fn rejects_linked_ancestors_and_targets_without_writing_through_them() {
    let tree = TestTree::new();
    let outside = tree.target("outside");
    fs::create_dir(&outside).expect("outside fixture must be created");
    fs::write(outside.join("sentinel"), b"unchanged").expect("sentinel must be written");

    let linked_parent = tree.target("linked-parent");
    if let Err(error) = create_directory_symlink(&outside, &linked_parent) {
        if matches!(
            error.kind(),
            io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
        ) {
            return;
        }
        panic!("directory symlink fixture could not be created: {error}");
    }

    assert!(matches!(
        approve_materialization_target(linked_parent.join("install")),
        Err(MaterializationError::LinkNotAllowed { .. })
    ));

    let linked_target = tree.target("linked-target");
    create_directory_symlink(&outside, &linked_target)
        .expect("target symlink fixture must be created");
    assert!(matches!(
        approve_materialization_target(&linked_target),
        Err(MaterializationError::LinkNotAllowed { .. })
    ));
    assert_eq!(fs::read(outside.join("sentinel")).unwrap(), b"unchanged");
    assert!(!outside.join("install").exists());

    let built = tree.build("1.19.0-beta.1");
    let loaded =
        load_resource_pack(built.core_bytes(), built.pack_sha256()).expect("test pack must load");
    let controlled_parent = tree.target("controlled");
    fs::create_dir(&controlled_parent).expect("controlled parent must be created");
    let swapped = approve_materialization_target(controlled_parent.join("install"))
        .expect("controlled target must approve");
    fs::rename(&controlled_parent, tree.target("controlled-original"))
        .expect("controlled parent must move");
    create_directory_symlink(&outside, &controlled_parent)
        .expect("controlled parent replacement must be linked");
    assert!(matches!(
        materialize_profile(&loaded, "full", &swapped),
        Err(MaterializationError::LinkNotAllowed { .. })
    ));
    assert!(!outside.join("install").exists());

    let managed = approve_materialization_target(tree.target("managed"))
        .expect("managed target must be approved");
    materialize_profile(&loaded, "full", &managed).expect("initial write must succeed");
    let managed_skill = managed.path().join("skills/example.md");
    fs::remove_file(&managed_skill).expect("managed skill must be removed");
    create_file_symlink(&outside.join("sentinel"), &managed_skill)
        .expect("managed link fixture must be created");
    assert!(matches!(
        materialize_profile(&loaded, "full", &managed),
        Err(MaterializationError::LinkNotAllowed { .. })
    ));
    assert_eq!(fs::read(outside.join("sentinel")).unwrap(), b"unchanged");
}

#[cfg(unix)]
fn assert_logical_modes(target: &Path) {
    use std::os::unix::fs::PermissionsExt;

    assert_eq!(
        fs::metadata(target.join("skills/example.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o644
    );
    assert_eq!(
        fs::metadata(target.join("workflow/run.sh"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
    assert_eq!(
        fs::metadata(target.join(MATERIALIZATION_RECEIPT_FILE))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o644
    );
}

#[cfg(not(unix))]
fn assert_logical_modes(_target: &Path) {}

#[cfg(unix)]
fn assert_private_temp_parent(parent: &Path) {
    use std::os::unix::fs::PermissionsExt;

    assert_eq!(
        fs::metadata(parent).unwrap().permissions().mode() & 0o777,
        0o700
    );
}

#[cfg(not(unix))]
fn assert_private_temp_parent(parent: &Path) {
    assert!(parent.is_dir());
}

#[cfg(unix)]
fn create_directory_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

#[cfg(not(any(unix, windows)))]
fn create_directory_symlink(_target: &Path, _link: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "symlinks are not supported on this target",
    ))
}

#[cfg(unix)]
fn create_file_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_file_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(not(any(unix, windows)))]
fn create_file_symlink(_target: &Path, _link: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "symlinks are not supported on this target",
    ))
}
