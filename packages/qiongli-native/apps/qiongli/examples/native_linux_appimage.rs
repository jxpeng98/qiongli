use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read as _;
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::io::Write as _;

use qiongli_content::LogicalMode;
use qiongli_platform::{
    Architecture, CapabilityProfile, DesktopPackageKind, DesktopPackageManifestV1,
    DesktopPackageRecordType, DesktopPackageStatus, InstallerKind, OperatingSystem, ProductId,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

const DESKTOP_MANIFEST_FILE: &str = "qiongli-desktop-package.manifest.json";
const DESKTOP_RECEIPT_FILE: &str = "qiongli-desktop-package.receipt.json";
const APPIMAGE_RECEIPT_FILE: &str = "qiongli-linux-appimage.receipt.json";
const APPIMAGE_TOOL_RELEASE: &str =
    "https://github.com/AppImage/appimagetool/releases/tag/continuous";
const APPIMAGE_TOOL_X86_64_ASSET: &str = "appimagetool-x86_64.AppImage";
const APPIMAGE_TOOL_X86_64_SHA256: &str =
    "a6d71e2b6cd66f8e8d16c37ad164658985e0cf5fcaa950c90a482890cb9d13e0";
const APPIMAGE_TOOL_AARCH64_ASSET: &str = "appimagetool-aarch64.AppImage";
const APPIMAGE_TOOL_AARCH64_SHA256: &str =
    "1b00524ba8c6b678dc15ef88a5c25ec24def36cdfc7e3abb32ddcd068e8007fe";
const APPIMAGE_TYPE_2_MAGIC: &[u8; 3] = b"AI\x02";
const CONTENT_ROOT_DOMAIN: &[u8] = b"qiongli-desktop-package-content-root-v1\0";
const MAX_APPDIR_ARCHIVE_BYTES: u64 = 272 * 1024 * 1024;
const MAX_APPIMAGE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_TOOL_BYTES: u64 = 32 * 1024 * 1024;
const MAX_JSON_BYTES: u64 = 256 * 1024;
const MAX_ENTRY_BYTES: u64 = 128 * 1024 * 1024;

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    if env::consts::OS != "linux" {
        return Err("linux-appimage-platform-unsupported");
    }
    let arguments = Arguments::parse(env::args_os().skip(1))?;
    let manifest_bytes = read_bounded(&arguments.desktop_manifest, MAX_JSON_BYTES)?;
    let manifest: DesktopPackageManifestV1 = parse_canonical_json(&manifest_bytes)?;
    validate_manifest(&manifest, &arguments.source_commit)?;

    let receipt_bytes = read_bounded(&arguments.desktop_receipt, MAX_JSON_BYTES)?;
    let receipt: DesktopPackageReceiptV1 = parse_canonical_json(&receipt_bytes)?;
    let appdir_archive_bytes = read_bounded(&arguments.appdir_package, MAX_APPDIR_ARCHIVE_BYTES)?;
    validate_desktop_receipt(&receipt, &arguments, &manifest_bytes, &appdir_archive_bytes)?;

    let tool_bytes = read_bounded(&arguments.appimagetool, MAX_TOOL_BYTES)?;
    let (tool_asset, expected_tool_sha256) = tool_identity(manifest.artifact.arch);
    let tool_sha256 = sha256_hex(&tool_bytes);
    if tool_sha256 != expected_tool_sha256 {
        return Err("linux-appimage-tool-digest-invalid");
    }

    let appimage_bytes = read_bounded(&arguments.appimage, MAX_APPIMAGE_BYTES)?;
    if appimage_bytes.get(..4) != Some(b"\x7fELF")
        || appimage_bytes.get(8..11) != Some(APPIMAGE_TYPE_2_MAGIC)
        || !is_executable(&arguments.appimage)?
    {
        return Err("linux-appimage-container-invalid");
    }
    let expected_appimage_file = appimage_file_name(&manifest);
    if arguments
        .appimage
        .file_name()
        .and_then(|name| name.to_str())
        != Some(expected_appimage_file.as_str())
    {
        return Err("linux-appimage-file-name-invalid");
    }
    verify_extracted_appdir(&arguments.extracted_appdir, &manifest, &manifest_bytes)?;

    let output = arguments.package_root.join(APPIMAGE_RECEIPT_FILE);
    if output.exists() {
        return Err("linux-appimage-output-exists");
    }
    let appimage_sha256 = sha256_hex(&appimage_bytes);
    let final_receipt = LinuxAppImageReceiptV1 {
        schema_version: 1,
        record_type: "qiongli-linux-appimage",
        status: "assembled-unpublished",
        container_format: "appimage-type-2",
        product_source_commit: &arguments.source_commit,
        artifact: &manifest.artifact,
        source_appdir_package_file: &receipt.package_file,
        source_appdir_package_size_bytes: receipt.package_size_bytes,
        source_appdir_package_sha256: &receipt.package_sha256,
        desktop_package_manifest_file: DESKTOP_MANIFEST_FILE,
        desktop_package_manifest_sha256: sha256_hex(&manifest_bytes),
        appimagetool_release: APPIMAGE_TOOL_RELEASE,
        appimagetool_asset: tool_asset,
        appimagetool_sha256: &tool_sha256,
        appimage_file: &expected_appimage_file,
        appimage_size_bytes: appimage_bytes.len() as u64,
        appimage_sha256: &appimage_sha256,
        extracted_entry_content_root_sha256: &manifest.entry_content_root_sha256,
    };
    let output_bytes = canonical_json(&final_receipt)?;
    write_private_file(&output, &output_bytes)?;
    assert_exact_package_output(
        &arguments.package_root,
        &receipt.package_file,
        &expected_appimage_file,
    )?;
    let rendered =
        String::from_utf8(output_bytes).map_err(|_| "linux-appimage-output-encoding-invalid")?;
    println!("{rendered}");
    Ok(())
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DesktopPackageReceiptV1 {
    schema_version: u32,
    status: String,
    product_source_commit: String,
    package_file: String,
    package_size_bytes: u64,
    package_sha256: String,
    package_manifest_file: String,
    package_manifest_sha256: String,
}

#[derive(Serialize)]
struct LinuxAppImageReceiptV1<'a> {
    schema_version: u32,
    record_type: &'static str,
    status: &'static str,
    container_format: &'static str,
    product_source_commit: &'a str,
    artifact: &'a qiongli_platform::ArtifactIdentityV1,
    source_appdir_package_file: &'a str,
    source_appdir_package_size_bytes: u64,
    source_appdir_package_sha256: &'a str,
    desktop_package_manifest_file: &'static str,
    desktop_package_manifest_sha256: String,
    appimagetool_release: &'static str,
    appimagetool_asset: &'static str,
    appimagetool_sha256: &'a str,
    appimage_file: &'a str,
    appimage_size_bytes: u64,
    appimage_sha256: &'a str,
    extracted_entry_content_root_sha256: &'a str,
}

struct Arguments {
    package_root: PathBuf,
    appdir_package: PathBuf,
    desktop_manifest: PathBuf,
    desktop_receipt: PathBuf,
    appimagetool: PathBuf,
    appimage: PathBuf,
    extracted_appdir: PathBuf,
    source_commit: String,
}

impl Arguments {
    fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let arguments = arguments.into_iter().collect::<Vec<_>>();
        let mut package_root = None;
        let mut appdir_package = None;
        let mut desktop_manifest = None;
        let mut desktop_receipt = None;
        let mut appimagetool = None;
        let mut appimage = None;
        let mut extracted_appdir = None;
        let mut source_commit = None;
        let mut index = 0;
        while index < arguments.len() {
            let option = arguments[index]
                .to_str()
                .ok_or("linux-appimage-usage-invalid")?;
            let value = arguments
                .get(index + 1)
                .ok_or("linux-appimage-usage-invalid")?;
            match option {
                "--package-root" if package_root.is_none() => {
                    package_root = Some(PathBuf::from(value));
                }
                "--appdir-package" if appdir_package.is_none() => {
                    appdir_package = Some(PathBuf::from(value));
                }
                "--desktop-manifest" if desktop_manifest.is_none() => {
                    desktop_manifest = Some(PathBuf::from(value));
                }
                "--desktop-receipt" if desktop_receipt.is_none() => {
                    desktop_receipt = Some(PathBuf::from(value));
                }
                "--appimagetool" if appimagetool.is_none() => {
                    appimagetool = Some(PathBuf::from(value));
                }
                "--appimage" if appimage.is_none() => {
                    appimage = Some(PathBuf::from(value));
                }
                "--extracted-appdir" if extracted_appdir.is_none() => {
                    extracted_appdir = Some(PathBuf::from(value));
                }
                "--source-commit" if source_commit.is_none() => {
                    source_commit = value.to_str().map(ToOwned::to_owned);
                }
                _ => return Err("linux-appimage-usage-invalid"),
            }
            index += 2;
        }
        let parsed = Self {
            package_root: package_root.ok_or("linux-appimage-usage-invalid")?,
            appdir_package: appdir_package.ok_or("linux-appimage-usage-invalid")?,
            desktop_manifest: desktop_manifest.ok_or("linux-appimage-usage-invalid")?,
            desktop_receipt: desktop_receipt.ok_or("linux-appimage-usage-invalid")?,
            appimagetool: appimagetool.ok_or("linux-appimage-usage-invalid")?,
            appimage: appimage.ok_or("linux-appimage-usage-invalid")?,
            extracted_appdir: extracted_appdir.ok_or("linux-appimage-usage-invalid")?,
            source_commit: source_commit.ok_or("linux-appimage-usage-invalid")?,
        };
        parsed.validate()?;
        Ok(parsed)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if !valid_absolute_path(&self.package_root)
            || !valid_absolute_path(&self.appdir_package)
            || !valid_absolute_path(&self.desktop_manifest)
            || !valid_absolute_path(&self.desktop_receipt)
            || !valid_absolute_path(&self.appimagetool)
            || !valid_absolute_path(&self.appimage)
            || !valid_absolute_path(&self.extracted_appdir)
            || !valid_source_commit(&self.source_commit)
            || !is_real_directory(&self.package_root)
            || !is_real_directory(&self.extracted_appdir)
            || self.appdir_package.parent() != Some(self.package_root.as_path())
            || self.desktop_manifest.parent() != Some(self.package_root.as_path())
            || self.desktop_receipt.parent() != Some(self.package_root.as_path())
            || self.appimage.parent() != Some(self.package_root.as_path())
            || self
                .desktop_manifest
                .file_name()
                .and_then(|name| name.to_str())
                != Some(DESKTOP_MANIFEST_FILE)
            || self
                .desktop_receipt
                .file_name()
                .and_then(|name| name.to_str())
                != Some(DESKTOP_RECEIPT_FILE)
        {
            return Err("linux-appimage-usage-invalid");
        }
        Ok(())
    }
}

fn validate_manifest(
    manifest: &DesktopPackageManifestV1,
    source_commit: &str,
) -> Result<(), &'static str> {
    let artifact = &manifest.artifact;
    let source = &manifest.source_artifact;
    if manifest.schema_version != 1
        || manifest.record_type != DesktopPackageRecordType::QiongliDesktopPackage
        || manifest.status != DesktopPackageStatus::AssembledUnpublished
        || manifest.package_kind != DesktopPackageKind::LinuxAppDirZip
        || artifact.validate().is_err()
        || source.validate().is_err()
        || artifact.product != ProductId::Qiongli
        || artifact.profile != CapabilityProfile::Lite
        || artifact.installer_kind != InstallerKind::NativeInstaller
        || artifact.os != OperatingSystem::Linux
        || source.product != artifact.product
        || source.version != artifact.version
        || source.channel != artifact.channel
        || source.profile != artifact.profile
        || source.os != artifact.os
        || source.arch != artifact.arch
        || source.installer_kind != InstallerKind::PortableArchive
        || manifest.product_source_commit != source_commit
        || manifest.package_root != "Qiongli.AppDir"
        || manifest.manifest_path != "Qiongli.AppDir/.qiongli-desktop-package.json"
        || manifest.application.product_name != "Qiongli"
        || manifest.application.window_title != "Qiongli 2"
        || manifest.application.application_identifier != "io.github.jxpeng98.qiongli"
        || manifest.application.product_version != artifact.version
        || manifest.application.license != "MIT"
        || manifest.entries.len() != 7
        || !is_lower_hex(&manifest.source_artifact_manifest_sha256, 64)
        || !is_lower_hex(&manifest.resource_pack_sha256, 64)
        || !is_lower_hex(&manifest.canonical_binary_sha256, 64)
        || !is_lower_hex(&manifest.launcher_sha256, 64)
        || !is_lower_hex(&manifest.update_helper_sha256, 64)
        || !is_lower_hex(&manifest.entry_content_root_sha256, 64)
    {
        return Err("linux-appimage-manifest-invalid");
    }
    let expected_paths = [
        "Qiongli.AppDir/.DirIcon",
        "Qiongli.AppDir/AppRun",
        "Qiongli.AppDir/LICENSE",
        "Qiongli.AppDir/qiongli-cli",
        "Qiongli.AppDir/qiongli-update-helper",
        "Qiongli.AppDir/qiongli.desktop",
        "Qiongli.AppDir/qiongli.png",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect::<BTreeSet<_>>();
    let actual_paths = manifest
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();
    let app_run = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "Qiongli.AppDir/AppRun")
        .ok_or("linux-appimage-manifest-invalid")?;
    let canonical = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "Qiongli.AppDir/qiongli-cli")
        .ok_or("linux-appimage-manifest-invalid")?;
    let update_helper = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "Qiongli.AppDir/qiongli-update-helper")
        .ok_or("linux-appimage-manifest-invalid")?;
    let dir_icon = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "Qiongli.AppDir/.DirIcon")
        .ok_or("linux-appimage-manifest-invalid")?;
    let icon = manifest
        .entries
        .iter()
        .find(|entry| entry.path == "Qiongli.AppDir/qiongli.png")
        .ok_or("linux-appimage-manifest-invalid")?;
    if actual_paths != expected_paths
        || manifest
            .entries
            .windows(2)
            .any(|entries| entries[0].path >= entries[1].path)
        || manifest.entries.iter().any(|entry| {
            entry.size_bytes == 0
                || entry.size_bytes > MAX_ENTRY_BYTES
                || !is_lower_hex(&entry.sha256, 64)
        })
        || app_run.mode != LogicalMode::Executable
        || app_run.sha256 != manifest.launcher_sha256
        || canonical.mode != LogicalMode::Executable
        || canonical.sha256 != manifest.canonical_binary_sha256
        || update_helper.mode != LogicalMode::Executable
        || update_helper.sha256 != manifest.update_helper_sha256
        || manifest
            .entries
            .iter()
            .filter(|entry| {
                entry.path != app_run.path
                    && entry.path != canonical.path
                    && entry.path != update_helper.path
            })
            .any(|entry| entry.mode != LogicalMode::Regular)
        || dir_icon.sha256 != icon.sha256
        || entry_content_root(manifest) != manifest.entry_content_root_sha256
    {
        return Err("linux-appimage-manifest-invalid");
    }
    Ok(())
}

fn validate_desktop_receipt(
    receipt: &DesktopPackageReceiptV1,
    arguments: &Arguments,
    manifest_bytes: &[u8],
    appdir_archive_bytes: &[u8],
) -> Result<(), &'static str> {
    if receipt.schema_version != 1
        || receipt.status != "assembled-unpublished"
        || receipt.product_source_commit != arguments.source_commit
        || receipt.package_file
            != arguments
                .appdir_package
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("linux-appimage-source-receipt-invalid")?
        || receipt.package_size_bytes != appdir_archive_bytes.len() as u64
        || receipt.package_sha256 != sha256_hex(appdir_archive_bytes)
        || receipt.package_manifest_file != DESKTOP_MANIFEST_FILE
        || receipt.package_manifest_sha256 != sha256_hex(manifest_bytes)
        || !is_lower_hex(&receipt.package_sha256, 64)
        || !is_lower_hex(&receipt.package_manifest_sha256, 64)
    {
        return Err("linux-appimage-source-receipt-invalid");
    }
    Ok(())
}

fn verify_extracted_appdir(
    root: &Path,
    manifest: &DesktopPackageManifestV1,
    manifest_bytes: &[u8],
) -> Result<(), &'static str> {
    let mut actual_names = fs::read_dir(root)
        .map_err(|_| "linux-appimage-extracted-layout-invalid")?
        .map(|entry| {
            let entry = entry.map_err(|_| "linux-appimage-extracted-layout-invalid")?;
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|_| "linux-appimage-extracted-layout-invalid")?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("linux-appimage-extracted-layout-invalid");
            }
            entry
                .file_name()
                .into_string()
                .map_err(|_| "linux-appimage-extracted-layout-invalid")
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let manifest_name = Path::new(&manifest.manifest_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("linux-appimage-extracted-layout-invalid")?;
    let expected_manifest = actual_names
        .take(manifest_name)
        .ok_or("linux-appimage-extracted-layout-invalid")?;
    if expected_manifest != manifest_name || actual_names.len() != manifest.entries.len() {
        return Err("linux-appimage-extracted-layout-invalid");
    }
    let extracted_manifest = read_bounded(&root.join(manifest_name), MAX_JSON_BYTES)?;
    if extracted_manifest != manifest_bytes {
        return Err("linux-appimage-extracted-manifest-drift");
    }
    for expected in &manifest.entries {
        let relative = expected
            .path
            .strip_prefix("Qiongli.AppDir/")
            .ok_or("linux-appimage-extracted-layout-invalid")?;
        if !actual_names.remove(relative) {
            return Err("linux-appimage-extracted-layout-invalid");
        }
        let path = root.join(relative);
        let bytes = read_bounded(&path, MAX_ENTRY_BYTES)?;
        if bytes.len() as u64 != expected.size_bytes
            || sha256_hex(&bytes) != expected.sha256
            || logical_mode(&path)? != expected.mode
        {
            return Err("linux-appimage-extracted-entry-drift");
        }
    }
    if !actual_names.is_empty() {
        return Err("linux-appimage-extracted-layout-invalid");
    }
    Ok(())
}

fn entry_content_root(manifest: &DesktopPackageManifestV1) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CONTENT_ROOT_DOMAIN);
    for entry in &manifest.entries {
        hasher.update((entry.path.len() as u64).to_be_bytes());
        hasher.update(entry.path.as_bytes());
        hasher.update([match entry.mode {
            LogicalMode::Regular => 0,
            LogicalMode::Executable => 1,
        }]);
        hasher.update(entry.size_bytes.to_be_bytes());
        hasher.update(entry.sha256.as_bytes());
    }
    encode_hex(&hasher.finalize())
}

fn tool_identity(architecture: Architecture) -> (&'static str, &'static str) {
    match architecture {
        Architecture::X86_64 => (APPIMAGE_TOOL_X86_64_ASSET, APPIMAGE_TOOL_X86_64_SHA256),
        Architecture::Aarch64 => (APPIMAGE_TOOL_AARCH64_ASSET, APPIMAGE_TOOL_AARCH64_SHA256),
    }
}

fn appimage_file_name(manifest: &DesktopPackageManifestV1) -> String {
    let architecture = match manifest.artifact.arch {
        Architecture::X86_64 => "x86_64",
        Architecture::Aarch64 => "aarch64",
    };
    format!(
        "Qiongli-{}-{architecture}.AppImage",
        manifest.artifact.version
    )
}

fn parse_canonical_json<T>(bytes: &[u8]) -> Result<T, &'static str>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let value = serde_json::from_slice(bytes).map_err(|_| "linux-appimage-json-invalid")?;
    if canonical_json(&value)? != bytes {
        return Err("linux-appimage-json-noncanonical");
    }
    Ok(value)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, &'static str> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| "linux-appimage-json-invalid")
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "linux-appimage-input-invalid")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > limit
    {
        return Err("linux-appimage-input-invalid");
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len()).map_err(|_| "linux-appimage-input-invalid")?,
    );
    File::open(path)
        .map_err(|_| "linux-appimage-input-invalid")?
        .take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "linux-appimage-input-invalid")?;
    if bytes.len() as u64 != metadata.len() {
        return Err("linux-appimage-input-invalid");
    }
    Ok(bytes)
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
        .map_err(|_| "linux-appimage-output-write-failed")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "linux-appimage-output-write-failed")
}

#[cfg(not(unix))]
fn write_private_file(_path: &Path, _bytes: &[u8]) -> Result<(), &'static str> {
    Err("linux-appimage-platform-unsupported")
}

fn assert_exact_package_output(
    root: &Path,
    appdir_package_file: &str,
    appimage_file: &str,
) -> Result<(), &'static str> {
    let actual = fs::read_dir(root)
        .map_err(|_| "linux-appimage-output-invalid")?
        .map(|entry| {
            entry
                .map_err(|_| "linux-appimage-output-invalid")?
                .file_name()
                .into_string()
                .map_err(|_| "linux-appimage-output-invalid")
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected = [
        appdir_package_file,
        DESKTOP_MANIFEST_FILE,
        DESKTOP_RECEIPT_FILE,
        appimage_file,
        APPIMAGE_RECEIPT_FILE,
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err("linux-appimage-output-invalid");
    }
    Ok(())
}

fn valid_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_real_directory(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|metadata| !metadata.file_type().is_symlink() && metadata.is_dir())
}

#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool, &'static str> {
    use std::os::unix::fs::PermissionsExt;

    fs::symlink_metadata(path)
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .map_err(|_| "linux-appimage-input-invalid")
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> Result<bool, &'static str> {
    Ok(false)
}

#[cfg(unix)]
fn logical_mode(path: &Path) -> Result<LogicalMode, &'static str> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::symlink_metadata(path).map_err(|_| "linux-appimage-input-invalid")?;
    if metadata.permissions().mode() & 0o111 == 0 {
        Ok(LogicalMode::Regular)
    } else {
        Ok(LogicalMode::Executable)
    }
}

#[cfg(not(unix))]
fn logical_mode(_path: &Path) -> Result<LogicalMode, &'static str> {
    Err("linux-appimage-platform-unsupported")
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
