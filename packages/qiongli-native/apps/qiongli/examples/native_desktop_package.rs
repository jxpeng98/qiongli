use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Read as _, Write as _};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_platform::{
    DesktopApplicationMetadataV1, DesktopPackageBinaries, DesktopPackageInput,
    GrantVerificationContext, InstallerKind, PackagedProductControlV1, ReleaseChannel,
    approve_native_artifact_target, compose_desktop_package, compose_native_artifact,
    current_target_native_artifact_identity, native_artifact_id, verify_desktop_package,
};
use serde::Serialize;

const LICENSE_BYTES: &[u8] = include_bytes!("../../../../../LICENSE");
const MAX_CANONICAL_BYTES: u64 = 128 * 1024 * 1024;
const MAX_LAUNCHER_BYTES: u64 = 16 * 1024 * 1024;
const MAX_UPDATE_HELPER_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PRODUCT_CONTROL_BYTES: u64 = 256 * 1024;
const DESKTOP_MANIFEST_FILE: &str = "qiongli-desktop-package.manifest.json";
const DESKTOP_RECEIPT_FILE: &str = "qiongli-desktop-package.receipt.json";

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let arguments = Arguments::parse(env::args_os().skip(1))?;
    if qiongli::embedded_source_commit() != Some(arguments.source_commit.as_str()) {
        return Err("desktop-package-source-commit-unbound");
    }
    create_private_directory(&arguments.output)?;
    let staging = create_child_directory(&arguments.output, ".staging")?;
    let result = assemble(&arguments, &staging);
    let cleanup = fs::remove_dir_all(&staging);
    if result.is_ok() && cleanup.is_err() {
        return Err("desktop-package-staging-cleanup-failed");
    }
    let assembled = result?;
    assert_exact_output(
        &arguments.output,
        [
            assembled.package_file.as_str(),
            DESKTOP_MANIFEST_FILE,
            DESKTOP_RECEIPT_FILE,
        ],
    )?;
    println!("{}", assembled.public_receipt);
    Ok(())
}

fn assemble(arguments: &Arguments, staging: &Path) -> Result<AssembledOutput, &'static str> {
    let canonical_source = staging.join(format!("canonical{}", env::consts::EXE_SUFFIX));
    stage_binary(&arguments.canonical, &canonical_source, MAX_CANONICAL_BYTES)?;
    let canonical_bytes = read_bounded(&canonical_source, MAX_CANONICAL_BYTES)
        .map_err(|_| "desktop-package-canonical-read-failed")?;
    let launcher_source = staging.join(format!("launcher{}", env::consts::EXE_SUFFIX));
    stage_binary(&arguments.launcher, &launcher_source, MAX_LAUNCHER_BYTES)?;
    let launcher_bytes = read_bounded(&launcher_source, MAX_LAUNCHER_BYTES)
        .map_err(|_| "desktop-package-launcher-read-failed")?;
    let helper_source = staging.join(format!("update-helper{}", env::consts::EXE_SUFFIX));
    stage_binary(
        &arguments.update_helper,
        &helper_source,
        MAX_UPDATE_HELPER_BYTES,
    )?;
    let helper_bytes = read_bounded(&helper_source, MAX_UPDATE_HELPER_BYTES)
        .map_err(|_| "desktop-package-update-helper-read-failed")?;
    let content = qiongli::embedded_content().map_err(|_| "desktop-package-content-invalid")?;
    let artifact =
        current_target_native_artifact_identity(env!("CARGO_PKG_VERSION"), ReleaseChannel::Alpha)
            .map_err(|_| "desktop-package-target-unsupported")?;
    let artifact_id =
        native_artifact_id(&artifact).map_err(|_| "desktop-package-artifact-invalid")?;
    let artifact_parent = create_child_directory(staging, "artifact")?;
    let artifact_target =
        approve_native_artifact_target(artifact_parent.join(&artifact_id), &artifact)
            .map_err(|_| "desktop-package-artifact-target-invalid")?;
    let source_artifact = compose_native_artifact(
        content.pack(),
        &artifact,
        &canonical_source,
        &artifact_target,
    )
    .map_err(|error| error.reason_code())?;
    let icon_png = qiongli::desktop_application_icon_png()
        .map_err(|_| "desktop-package-icon-encoding-failed")?;
    let metadata = qiongli::desktop_application_metadata();
    let zotero_companion =
        qiongli::embedded_zotero_companion().map_err(|error| error.reason_code())?;
    let application = DesktopApplicationMetadataV1::new(
        metadata.product_name(),
        metadata.window_title(),
        metadata.application_identifier(),
        metadata.version(),
        metadata.license(),
    );
    let product_control = arguments
        .product_control
        .as_ref()
        .map(|path| read_bounded(path, MAX_PRODUCT_CONTROL_BYTES))
        .transpose()
        .map_err(|_| "desktop-package-product-control-read-failed")?;
    if let Some(control) = product_control.as_deref() {
        validate_product_control(
            control,
            &source_artifact.manifest().artifact,
            &source_artifact.manifest().binary_sha256,
            content.pack().pack_sha256(),
            &arguments.source_commit,
        )?;
    }
    let mut package_input = DesktopPackageInput::new(
        &source_artifact,
        DesktopPackageBinaries::new(&canonical_bytes, &launcher_bytes, &helper_bytes),
        &icon_png,
        LICENSE_BYTES,
        &arguments.source_commit,
        application,
        &zotero_companion,
    );
    if let Some(control) = product_control.as_deref() {
        package_input = package_input.with_product_control(control);
    }
    let package = compose_desktop_package(package_input).map_err(|error| error.reason_code())?;
    verify_desktop_package(
        &source_artifact,
        &arguments.source_commit,
        package.archive_bytes(),
    )
    .map_err(|_| "desktop-package-verification-failed")?;

    let archive_path = arguments.output.join(package.file_name());
    write_private_file(&archive_path, package.archive_bytes())?;
    write_private_file(
        &arguments.output.join(DESKTOP_MANIFEST_FILE),
        package.manifest_bytes(),
    )?;
    let receipt = DesktopPackageReceiptV2 {
        schema_version: 2,
        status: "assembled-unpublished",
        product_source_commit: &arguments.source_commit,
        package_file: package.file_name(),
        package_size_bytes: package.archive_bytes().len() as u64,
        package_sha256: package.archive_sha256(),
        package_manifest_file: DESKTOP_MANIFEST_FILE,
        package_manifest_sha256: sha256_hex(package.manifest_bytes()),
        zotero_companion: DesktopPackageZoteroCompanionReceiptV1 {
            companion_version: &package.manifest().zotero_companion.companion_version,
            endpoint_version: &package.manifest().zotero_companion.endpoint_version,
            xpi_sha256: &package.manifest().zotero_companion.xpi_sha256,
            artifact_manifest_sha256: &package.manifest().zotero_companion.artifact_manifest_sha256,
        },
    };
    let receipt_bytes = canonical_json(&receipt)?;
    write_private_file(&arguments.output.join(DESKTOP_RECEIPT_FILE), &receipt_bytes)?;
    let public_receipt =
        String::from_utf8(receipt_bytes).map_err(|_| "desktop-package-receipt-encoding-failed")?;
    Ok(AssembledOutput {
        package_file: package.file_name().to_owned(),
        public_receipt,
    })
}

struct AssembledOutput {
    package_file: String,
    public_receipt: String,
}

#[derive(Serialize)]
struct DesktopPackageReceiptV2<'a> {
    schema_version: u32,
    status: &'static str,
    product_source_commit: &'a str,
    package_file: &'a str,
    package_size_bytes: u64,
    package_sha256: &'a str,
    package_manifest_file: &'static str,
    package_manifest_sha256: String,
    zotero_companion: DesktopPackageZoteroCompanionReceiptV1<'a>,
}

#[derive(Serialize)]
struct DesktopPackageZoteroCompanionReceiptV1<'a> {
    companion_version: &'a str,
    endpoint_version: &'a str,
    xpi_sha256: &'a str,
    artifact_manifest_sha256: &'a str,
}

struct Arguments {
    canonical: PathBuf,
    launcher: PathBuf,
    update_helper: PathBuf,
    output: PathBuf,
    source_commit: String,
    product_control: Option<PathBuf>,
}

impl Arguments {
    fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let args = args.into_iter().collect::<Vec<_>>();
        let mut canonical = None;
        let mut launcher = None;
        let mut update_helper = None;
        let mut output = None;
        let mut source_commit = None;
        let mut product_control = None;
        let mut index = 0;
        while index < args.len() {
            let option = args[index]
                .to_str()
                .ok_or("desktop-package-usage-invalid")?;
            let value = args.get(index + 1).ok_or("desktop-package-usage-invalid")?;
            match option {
                "--canonical" if canonical.is_none() => canonical = Some(PathBuf::from(value)),
                "--launcher" if launcher.is_none() => launcher = Some(PathBuf::from(value)),
                "--update-helper" if update_helper.is_none() => {
                    update_helper = Some(PathBuf::from(value))
                }
                "--output" if output.is_none() => output = Some(PathBuf::from(value)),
                "--source-commit" if source_commit.is_none() => {
                    source_commit = value.to_str().map(ToOwned::to_owned)
                }
                "--product-control" if product_control.is_none() => {
                    product_control = Some(PathBuf::from(value))
                }
                _ => return Err("desktop-package-usage-invalid"),
            }
            index += 2;
        }
        let canonical = canonical.ok_or("desktop-package-usage-invalid")?;
        let launcher = launcher.ok_or("desktop-package-usage-invalid")?;
        let update_helper = update_helper.ok_or("desktop-package-usage-invalid")?;
        let output = output.ok_or("desktop-package-usage-invalid")?;
        let source_commit = source_commit.ok_or("desktop-package-usage-invalid")?;
        if !valid_input_path(&canonical)
            || !valid_input_path(&launcher)
            || !valid_input_path(&update_helper)
            || !valid_output_path(&output)
            || !valid_source_commit(&source_commit)
            || product_control
                .as_ref()
                .is_some_and(|path| !valid_input_path(path))
        {
            return Err("desktop-package-usage-invalid");
        }
        Ok(Self {
            canonical,
            launcher,
            update_helper,
            output,
            source_commit,
            product_control,
        })
    }
}

fn validate_product_control(
    bytes: &[u8],
    portable_artifact: &qiongli_platform::ArtifactIdentityV1,
    binary_sha256: &str,
    pack_sha256: &str,
    source_commit: &str,
) -> Result<(), &'static str> {
    let control = PackagedProductControlV1::from_json(bytes)
        .map_err(|_| "desktop-package-product-control-invalid")?;
    let authority = qiongli::embedded_release_authority()
        .map_err(|_| "desktop-package-product-control-authority-invalid")?
        .ok_or("desktop-package-product-control-authority-missing")?;
    let mut desktop_artifact = portable_artifact.clone();
    desktop_artifact.installer_kind = InstallerKind::NativeInstaller;
    let mut plugin_artifact = portable_artifact.clone();
    plugin_artifact.installer_kind = InstallerKind::PluginBundle;
    if control.artifact != desktop_artifact
        || control.product_source_commit != source_commit
        || control.canonical_binary_sha256 != binary_sha256
        || control.resource_pack_sha256 != pack_sha256
    {
        return Err("desktop-package-product-control-identity-mismatch");
    }
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "desktop-package-clock-invalid")?
        .as_secs();
    for plugin in &control.client_plugins {
        let context = GrantVerificationContext {
            now_unix,
            minimum_generation: authority.minimum_launch_grant_generation(),
            expected_artifact: &plugin_artifact,
            binary_sha256,
            resource_pack_sha256: pack_sha256,
            requested_mode: plugin.target.required_grant_mode(),
            requested_scope: plugin.target.integration_scope(),
        };
        plugin
            .signed_launch_grant
            .verify(authority.launch_grant_keys(), &context)
            .map_err(|_| "desktop-package-product-control-grant-invalid")?;
    }
    Ok(())
}

fn valid_input_path(path: &Path) -> bool {
    valid_absolute_path(path) && path.is_file()
}

fn valid_output_path(path: &Path) -> bool {
    valid_absolute_path(path)
        && !path.exists()
        && path.parent().is_some_and(Path::is_dir)
        && outside_checkout(path)
}

fn valid_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn outside_checkout(output: &Path) -> bool {
    let Some(output_parent) = output.parent() else {
        return false;
    };
    let Some(checkout_root) = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4) else {
        return false;
    };
    let Ok(output_parent) = fs::canonicalize(output_parent) else {
        return false;
    };
    let Ok(checkout_root) = fs::canonicalize(checkout_root) else {
        return false;
    };
    !output_parent.starts_with(checkout_root)
}

fn stage_binary(source: &Path, destination: &Path, limit: u64) -> Result<(), &'static str> {
    let metadata = fs::symlink_metadata(source).map_err(|_| "desktop-package-binary-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > limit
    {
        return Err("desktop-package-binary-invalid");
    }
    let source = File::open(source).map_err(|_| "desktop-package-binary-invalid")?;
    let mut destination_file = create_private_executable(destination)?;
    let copied = std::io::copy(
        &mut source.take(limit.saturating_add(1)),
        &mut destination_file,
    )
    .map_err(|_| "desktop-package-binary-stage-failed")?;
    if copied != metadata.len() || copied > limit {
        return Err("desktop-package-binary-stage-failed");
    }
    destination_file
        .sync_all()
        .map_err(|_| "desktop-package-binary-stage-failed")?;
    drop(destination_file);
    set_executable_mode(destination)
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, ()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > limit
    {
        return Err(());
    }
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).map_err(|_| ())?);
    File::open(path)
        .map_err(|_| ())?
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() as u64 != metadata.len() {
        return Err(());
    }
    Ok(bytes)
}

fn create_child_directory(root: &Path, leaf: &str) -> Result<PathBuf, &'static str> {
    let path = root.join(leaf);
    create_private_directory(&path)?;
    Ok(path)
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| "desktop-package-directory-create-failed")
}

#[cfg(windows)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    qiongli_windows_security::create_owner_only_directory(path)
        .map(|_| ())
        .map_err(|_| "desktop-package-directory-create-failed")
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(_path: &Path) -> Result<(), &'static str> {
    Err("desktop-package-platform-unsupported")
}

#[cfg(unix)]
fn create_private_executable(path: &Path) -> Result<File, &'static str> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o700)
        .open(path)
        .map_err(|_| "desktop-package-binary-stage-failed")
}

#[cfg(windows)]
fn create_private_executable(path: &Path) -> Result<File, &'static str> {
    qiongli_windows_security::create_owner_only_new_file(path)
        .map_err(|_| "desktop-package-binary-stage-failed")
}

#[cfg(not(any(unix, windows)))]
fn create_private_executable(_path: &Path) -> Result<File, &'static str> {
    Err("desktop-package-platform-unsupported")
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| "desktop-package-binary-stage-failed")
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) -> Result<(), &'static str> {
    Ok(())
}

#[cfg(unix)]
fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| "desktop-package-output-write-failed")?;
    write_and_sync(&mut file, bytes)
}

#[cfg(windows)]
fn write_private_file(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let mut file = qiongli_windows_security::create_owner_only_new_file(path)
        .map_err(|_| "desktop-package-output-write-failed")?;
    write_and_sync(&mut file, bytes)
}

#[cfg(not(any(unix, windows)))]
fn write_private_file(_path: &Path, _bytes: &[u8]) -> Result<(), &'static str> {
    Err("desktop-package-platform-unsupported")
}

fn write_and_sync(file: &mut File, bytes: &[u8]) -> Result<(), &'static str> {
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "desktop-package-output-write-failed")
}

fn assert_exact_output<'a>(
    output: &Path,
    expected: impl IntoIterator<Item = &'a str>,
) -> Result<(), &'static str> {
    let mut actual = fs::read_dir(output)
        .map_err(|_| "desktop-package-output-invalid")?
        .map(|entry| {
            entry
                .map_err(|_| "desktop-package-output-invalid")?
                .file_name()
                .into_string()
                .map_err(|_| "desktop-package-output-invalid")
        })
        .collect::<Result<Vec<_>, _>>()?;
    actual.sort();
    let mut expected = expected
        .into_iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        return Err("desktop-package-output-invalid");
    }
    Ok(())
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| "desktop-package-receipt-serialization-failed")
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;

    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}
