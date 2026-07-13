use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_content::{
    CollectorError, CollectorLimits, LogicalMode, ResourceKind, collect_canonical_sources,
    collect_canonical_sources_with_limits,
};

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
            "qiongli-content-collector-{}-{nonce}-{tree_id}",
            std::process::id(),
        ));
        fs::create_dir(&root).expect("test content root must be created");
        for directory in DIRECTORY_ROOTS {
            fs::create_dir_all(root.join(directory))
                .expect("canonical test directory must be created");
        }
        fs::write(root.join("skills-core.md"), b"a").expect("skills-core fixture must be written");
        fs::write(root.join("skills-summary.md"), b"b")
            .expect("skills-summary fixture must be written");
        Self { root }
    }

    fn write(&self, relative_path: &str, bytes: &[u8]) -> PathBuf {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent must be created");
        }
        fs::write(&path, bytes).expect("fixture must be written");
        path
    }
}

impl Drop for TestTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn repository_content_collects_into_sorted_typed_resources() {
    let content_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../content");
    let resources =
        collect_canonical_sources(content_root).expect("repository content must collect");

    assert!(!resources.is_empty());
    assert_eq!(
        resources
            .iter()
            .map(|resource| resource.resource_kind)
            .collect::<BTreeSet<_>>()
            .len(),
        11
    );
    assert!(
        resources
            .windows(2)
            .all(|pair| pair[0].path.as_bytes() < pair[1].path.as_bytes())
    );
    assert!(resources.iter().all(|resource| {
        resource.mode == LogicalMode::Regular
            && resource.size_bytes() == resource.bytes().len() as u64
            && !resource.path.starts_with('/')
            && !resource.path.contains('\\')
    }));

    let contract = resources
        .iter()
        .find(|resource| resource.path == "mcp-contracts/v2/registry.json")
        .expect("Contract v2 registry must be collected");
    assert_eq!(contract.resource_kind, ResourceKind::McpContract);

    let skill = resources
        .iter()
        .find(|resource| resource.path == "skills/A_framing/question-refiner.md")
        .expect("canonical skill must be collected");
    assert_eq!(skill.resource_kind, ResourceKind::Skill);
}

#[test]
fn missing_allowlisted_root_fails_closed() {
    let tree = TestTree::new();
    fs::remove_dir(tree.root.join("workflow")).expect("workflow fixture must be removed");

    assert!(matches!(
        collect_canonical_sources(&tree.root),
        Err(CollectorError::MissingSource { .. })
    ));
}

#[test]
fn symbolic_link_with_traversal_target_is_rejected() {
    let tree = TestTree::new();
    let link = tree.root.join("skills/linked.md");
    if let Err(error) = create_file_symlink(Path::new("../skills-core.md"), &link) {
        if matches!(
            error.kind(),
            io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
        ) {
            return;
        }
        panic!("symlink fixture could not be created: {error}");
    }

    assert!(matches!(
        collect_canonical_sources(&tree.root),
        Err(CollectorError::LinkNotAllowed { .. })
    ));
}

#[test]
fn hard_link_alias_is_rejected() {
    let tree = TestTree::new();
    let original = tree.write("skills/original.md", b"same inode");
    let alias = tree.root.join("skills/alias.md");
    if let Err(error) = fs::hard_link(&original, &alias) {
        if matches!(
            error.kind(),
            io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
        ) {
            return;
        }
        panic!("hard-link fixture could not be created: {error}");
    }

    assert!(matches!(
        collect_canonical_sources(&tree.root),
        Err(CollectorError::HardLinkNotAllowed { .. } | CollectorError::HardLinkAlias { .. })
    ));
}

#[test]
fn individual_file_size_limit_is_enforced_before_collection() {
    let tree = TestTree::new();
    tree.write("skills/oversized.md", b"four");
    let limits = CollectorLimits {
        max_entry_bytes: 3,
        ..CollectorLimits::default()
    };

    assert!(matches!(
        collect_canonical_sources_with_limits(&tree.root, limits),
        Err(CollectorError::FileTooLarge { .. })
    ));
}

#[test]
fn cumulative_size_limit_is_enforced() {
    let tree = TestTree::new();
    tree.write("skills/two.md", b"ab");
    let limits = CollectorLimits {
        max_total_bytes: 3,
        ..CollectorLimits::default()
    };

    assert!(matches!(
        collect_canonical_sources_with_limits(&tree.root, limits),
        Err(CollectorError::TotalSizeExceeded { .. })
    ));
}

#[test]
fn entry_count_limit_is_enforced() {
    let tree = TestTree::new();
    tree.write("skills/third.md", b"c");
    let limits = CollectorLimits {
        max_entries: 2,
        ..CollectorLimits::default()
    };

    assert!(matches!(
        collect_canonical_sources_with_limits(&tree.root, limits),
        Err(CollectorError::EntryLimitExceeded { limit: 2 })
    ));
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
