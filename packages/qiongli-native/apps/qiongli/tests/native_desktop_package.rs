use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_platform::{
    DesktopApplicationMetadataV1, DesktopPackageBinaries, DesktopPackageError, DesktopPackageInput,
    DesktopPackageKind, DesktopPackageStatus, ReleaseChannel, approve_native_artifact_target,
    compose_desktop_package, compose_native_artifact, current_target_native_artifact_identity,
    native_artifact_id, verify_desktop_package,
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
const LICENSE_BYTES: &[u8] = include_bytes!("../../../../../LICENSE");

struct Fixture {
    root: PathBuf,
    canonical: PathBuf,
    launcher: PathBuf,
    update_helper: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-desktop-package-tests");
        fs::create_dir_all(&test_base).expect("desktop package test base must exist");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let root = test_base.join(format!(
            "package-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        create_private_directory(&root);
        let canonical = root.join(format!("canonical{}", std::env::consts::EXE_SUFFIX));
        let launcher = root.join(format!("launcher{}", std::env::consts::EXE_SUFFIX));
        let update_helper = root.join(format!("update-helper{}", std::env::consts::EXE_SUFFIX));
        fs::write(&canonical, test_binary(b"canonical"))
            .expect("canonical fixture binary must write");
        fs::write(&launcher, test_binary(b"launcher")).expect("launcher fixture binary must write");
        fs::write(&update_helper, test_binary(b"update-helper"))
            .expect("update helper fixture binary must write");
        set_executable_mode(&canonical);
        set_executable_mode(&launcher);
        set_executable_mode(&update_helper);
        Self {
            root,
            canonical,
            launcher,
            update_helper,
        }
    }

    fn artifact_target(&self, artifact_id: &str) -> PathBuf {
        let parent = self.root.join("artifact");
        create_private_directory(&parent);
        parent.join(artifact_id)
    }
}

fn test_binary(label: &[u8]) -> Vec<u8> {
    let mut bytes = match std::env::consts::OS {
        "macos" => b"\xcf\xfa\xed\xfe".to_vec(),
        "windows" => b"MZ".to_vec(),
        "linux" => b"\x7fELF".to_vec(),
        _ => panic!("desktop package test requires a supported target"),
    };
    bytes.extend_from_slice(label);
    bytes
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .expect("private fixture directory must be created");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture directory must remain private");
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    qiongli_windows_security::create_owner_only_directory(path)
        .expect("owner-only Windows fixture directory must be created");
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(path: &Path) {
    fs::create_dir(path).expect("private fixture directory must be created");
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture binary must be executable");
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) {}

#[test]
fn desktop_package_is_deterministic_bound_and_tamper_evident() {
    let fixture = Fixture::new();
    let content = qiongli::embedded_content().expect("embedded content must load");
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), ReleaseChannel::Alpha)
            .expect("current artifact identity must resolve");
    let artifact_id = native_artifact_id(&artifact).expect("artifact ID must render");
    let target = approve_native_artifact_target(fixture.artifact_target(&artifact_id), &artifact)
        .expect("source artifact target must approve");
    let source = compose_native_artifact(content.pack(), &artifact, &fixture.canonical, &target)
        .expect("source artifact must compose");
    let canonical = fs::read(&fixture.canonical).expect("canonical bytes must read");
    let launcher = fs::read(&fixture.launcher).expect("launcher bytes must read");
    let update_helper = fs::read(&fixture.update_helper).expect("update helper bytes must read");
    let icon = qiongli::desktop_application_icon_png().expect("packaged icon must encode");
    let metadata = qiongli::desktop_application_metadata();
    let application = DesktopApplicationMetadataV1::new(
        metadata.product_name(),
        metadata.window_title(),
        metadata.application_identifier(),
        metadata.version(),
        metadata.license(),
    );
    let source_commit = "a".repeat(40);
    let zotero_companion =
        qiongli::embedded_zotero_companion().expect("embedded Companion must verify");

    let first = compose_desktop_package(DesktopPackageInput::new(
        &source,
        DesktopPackageBinaries::new(&canonical, &launcher, &update_helper),
        &icon,
        LICENSE_BYTES,
        &source_commit,
        application.clone(),
        &zotero_companion,
    ))
    .expect("first desktop package must compose");
    let second = compose_desktop_package(DesktopPackageInput::new(
        &source,
        DesktopPackageBinaries::new(&canonical, &launcher, &update_helper),
        &icon,
        LICENSE_BYTES,
        &source_commit,
        application,
        &zotero_companion,
    ))
    .expect("second desktop package must compose");
    assert_eq!(first, second);
    assert_eq!(
        first,
        verify_desktop_package(&source, &source_commit, first.archive_bytes()).unwrap()
    );
    assert_eq!(
        first.manifest().status,
        DesktopPackageStatus::AssembledUnpublished
    );
    assert_eq!(
        first.manifest().package_kind,
        DesktopPackageKind::for_operating_system(artifact.os)
    );
    assert_eq!(first.manifest().source_artifact, artifact);
    assert_eq!(
        first.manifest().source_artifact_manifest_sha256,
        source.manifest_sha256()
    );
    assert_eq!(
        first.manifest().resource_pack_sha256,
        content.pack().pack_sha256()
    );
    assert_eq!(
        first.manifest().canonical_binary_sha256,
        source.manifest().binary_sha256
    );
    assert_eq!(first.manifest().update_helper_sha256.len(), 64);
    assert_eq!(first.manifest().zotero_companion.companion_version, "0.3.0");
    assert_eq!(first.manifest().zotero_companion.endpoint_version, "2");
    assert_eq!(first.manifest().product_source_commit, source_commit);
    assert!(first.file_name().starts_with("Qiongli-2.0.0-alpha.2-"));
    assert_eq!(first.archive_sha256().len(), 64);

    let mut tampered = first.archive_bytes().to_vec();
    let midpoint = tampered.len() / 2;
    tampered[midpoint] ^= 1;
    assert!(verify_desktop_package(&source, &source_commit, &tampered).is_err());
    assert_eq!(
        verify_desktop_package(&source, &"b".repeat(40), first.archive_bytes()),
        Err(DesktopPackageError::ManifestInvalid)
    );
}
