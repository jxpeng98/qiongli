use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
#[cfg(target_os = "macos")]
use std::time::Duration;

use qiongli_content::LogicalMode;
use qiongli_platform::{DesktopPackageManifestV1, VerifiedNativeUpdateEvidence};
use sha2::{Digest, Sha256};

const EXTRACTING_DIRECTORY: &str = ".application.partial";
const STAGED_DIRECTORY: &str = "application";
const APPLICATION_ROOT: &str = "Qiongli.app";
const CODE_RESOURCES_PATH: &str = "Qiongli.app/Contents/_CodeSignature/CodeResources";
const MAX_ARCHIVE_ENTRIES: u16 = 32;
const MAX_CODE_RESOURCES_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SIGNED_BINARY_GROWTH_BYTES: u64 = 16 * 1024 * 1024;
#[cfg(target_os = "macos")]
const MAX_TOOL_OUTPUT_BYTES: usize = 64 * 1024;
#[cfg(target_os = "macos")]
const TOOL_TIMEOUT: Duration = Duration::from_secs(30);
const ZIP_EOCD_SIGNATURE: u32 = 0x0605_4b50;
const ZIP_CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
const ZIP_LOCAL_SIGNATURE: u32 = 0x0403_4b50;
const ZIP_EOCD_MIN_BYTES: usize = 22;
const ZIP_MAX_COMMENT_BYTES: usize = u16::MAX as usize;
const ZIP_ENCRYPTED_FLAG: u16 = 0x0001;
const ZIP_DATA_DESCRIPTOR_FLAG: u16 = 0x0008;
const ZIP_STORED_METHOD: u16 = 0;
const ZIP_DEFLATED_METHOD: u16 = 8;
const ZIP64_EXTRA_FIELD: u16 = 0x0001;
const UNIX_FILE_TYPE_MASK: u32 = 0o170000;
const UNIX_REGULAR_FILE: u32 = 0o100000;
const UNIX_DIRECTORY: u32 = 0o040000;

pub(crate) struct StagedMacosApplication {
    pub(crate) launcher_sha256: String,
    pub(crate) canonical_binary_sha256: String,
    pub(crate) update_helper_sha256: String,
}

pub(crate) fn stage_verified_macos_application(
    transaction_root: &Path,
    archive_path: &Path,
    desktop_manifest_bytes: &[u8],
    evidence: &VerifiedNativeUpdateEvidence,
    expected_team_id: &str,
) -> Result<StagedMacosApplication, &'static str> {
    let contract = MacosStageContract {
        desktop_manifest: evidence.desktop_manifest().clone(),
        desktop_manifest_bytes,
        signed_launcher_sha256: evidence.signed_launcher_sha256(),
        signed_canonical_binary_sha256: evidence.signed_canonical_binary_sha256(),
        signed_update_helper_sha256: evidence.signed_update_helper_sha256(),
        expected_team_id,
    };
    stage_with_tools(
        transaction_root,
        archive_path,
        &contract,
        &SystemMacosPlatformTools,
    )
}

pub(crate) fn discard_staged_macos_application(transaction_root: &Path) {
    for name in [EXTRACTING_DIRECTORY, STAGED_DIRECTORY] {
        let path = transaction_root.join(name);
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }
}

struct MacosStageContract<'a> {
    desktop_manifest: DesktopPackageManifestV1,
    desktop_manifest_bytes: &'a [u8],
    signed_launcher_sha256: &'a str,
    signed_canonical_binary_sha256: &'a str,
    signed_update_helper_sha256: &'a str,
    expected_team_id: &'a str,
}

trait MacosPlatformTools {
    fn extract(&self, archive_path: &Path, destination: &Path) -> Result<(), &'static str>;
    fn verify_trust(&self, application: &Path, expected_team_id: &str) -> Result<(), &'static str>;
}

struct SystemMacosPlatformTools;

impl MacosPlatformTools for SystemMacosPlatformTools {
    fn extract(&self, archive_path: &Path, destination: &Path) -> Result<(), &'static str> {
        extract_with_ditto(archive_path, destination)
    }

    fn verify_trust(&self, application: &Path, expected_team_id: &str) -> Result<(), &'static str> {
        verify_macos_trust(application, expected_team_id)
    }
}

fn stage_with_tools(
    transaction_root: &Path,
    archive_path: &Path,
    contract: &MacosStageContract<'_>,
    tools: &impl MacosPlatformTools,
) -> Result<StagedMacosApplication, &'static str> {
    validate_private_transaction_root(transaction_root)?;
    validate_zip_layout(archive_path, contract)?;
    let extracting_root = transaction_root.join(EXTRACTING_DIRECTORY);
    let staged_root = transaction_root.join(STAGED_DIRECTORY);
    ensure_absent(&extracting_root)?;
    ensure_absent(&staged_root)?;
    create_private_directory(&extracting_root)?;

    let result = (|| {
        tools.extract(archive_path, &extracting_root)?;
        let application = extracting_root.join(APPLICATION_ROOT);
        validate_extracted_application(&application, contract)?;
        tools.verify_trust(&application, contract.expected_team_id)?;
        rename_without_replacement(&extracting_root, &staged_root)?;
        Ok(StagedMacosApplication {
            launcher_sha256: contract.signed_launcher_sha256.to_string(),
            canonical_binary_sha256: contract.signed_canonical_binary_sha256.to_string(),
            update_helper_sha256: contract.signed_update_helper_sha256.to_string(),
        })
    })();

    if result.is_err() {
        discard_staged_macos_application(transaction_root);
    }
    result
}

fn validate_zip_layout(
    archive_path: &Path,
    contract: &MacosStageContract<'_>,
) -> Result<(), &'static str> {
    let mut archive = open_private_regular_file(archive_path)
        .map_err(|_| "native-update-archive-layout-invalid")?;
    let archive_size = archive
        .metadata()
        .map_err(|_| "native-update-archive-layout-invalid")?
        .len();
    if archive_size < ZIP_EOCD_MIN_BYTES as u64 {
        return Err("native-update-archive-layout-invalid");
    }
    let tail_size = archive_size.min(
        (ZIP_EOCD_MIN_BYTES
            .saturating_add(ZIP_MAX_COMMENT_BYTES)
            .saturating_add(4)) as u64,
    );
    archive
        .seek(SeekFrom::End(
            -i64::try_from(tail_size).map_err(|_| "native-update-archive-layout-invalid")?,
        ))
        .map_err(|_| "native-update-archive-layout-invalid")?;
    let mut tail =
        vec![0_u8; usize::try_from(tail_size).map_err(|_| "native-update-archive-layout-invalid")?];
    archive
        .read_exact(&mut tail)
        .map_err(|_| "native-update-archive-layout-invalid")?;
    let eocd_offset = tail
        .windows(4)
        .rposition(|bytes| bytes == ZIP_EOCD_SIGNATURE.to_le_bytes())
        .ok_or("native-update-archive-layout-invalid")?;
    let eocd = tail
        .get(eocd_offset..)
        .ok_or("native-update-archive-layout-invalid")?;
    if eocd.len() < ZIP_EOCD_MIN_BYTES {
        return Err("native-update-archive-layout-invalid");
    }
    let disk = read_u16(eocd, 4)?;
    let central_disk = read_u16(eocd, 6)?;
    let entries_on_disk = read_u16(eocd, 8)?;
    let entry_count = read_u16(eocd, 10)?;
    let central_size = u64::from(read_u32(eocd, 12)?);
    let central_offset = u64::from(read_u32(eocd, 16)?);
    let comment_size = usize::from(read_u16(eocd, 20)?);
    let eocd_absolute_offset = archive_size
        .saturating_sub(tail_size)
        .saturating_add(eocd_offset as u64);
    if disk != 0
        || central_disk != 0
        || entries_on_disk != entry_count
        || entry_count == 0
        || entry_count > MAX_ARCHIVE_ENTRIES
        || comment_size != eocd.len().saturating_sub(ZIP_EOCD_MIN_BYTES)
        || central_offset
            .checked_add(central_size)
            .is_none_or(|end| end != eocd_absolute_offset)
    {
        return Err("native-update-archive-layout-invalid");
    }

    let expected_files = expected_archive_files(&contract.desktop_manifest);
    let expected_directories = expected_archive_directories(&expected_files);
    let mut observed_files = BTreeSet::new();
    let mut observed_directories = BTreeSet::new();
    archive
        .seek(SeekFrom::Start(central_offset))
        .map_err(|_| "native-update-archive-layout-invalid")?;
    let mut consumed = 0_u64;
    for _ in 0..entry_count {
        let mut header = [0_u8; 46];
        archive
            .read_exact(&mut header)
            .map_err(|_| "native-update-archive-layout-invalid")?;
        consumed = consumed
            .checked_add(header.len() as u64)
            .ok_or("native-update-archive-layout-invalid")?;
        if read_u32(&header, 0)? != ZIP_CENTRAL_SIGNATURE {
            return Err("native-update-archive-layout-invalid");
        }
        let flags = read_u16(&header, 8)?;
        let method = read_u16(&header, 10)?;
        let crc32 = read_u32(&header, 16)?;
        let compressed_size = u64::from(read_u32(&header, 20)?);
        let uncompressed_size = u64::from(read_u32(&header, 24)?);
        let name_size = usize::from(read_u16(&header, 28)?);
        let extra_size = usize::from(read_u16(&header, 30)?);
        let comment_size = usize::from(read_u16(&header, 32)?);
        let disk_start = read_u16(&header, 34)?;
        let external_attributes = read_u32(&header, 38)?;
        let local_offset = u64::from(read_u32(&header, 42)?);
        if flags & ZIP_ENCRYPTED_FLAG != 0
            || !matches!(method, ZIP_STORED_METHOD | ZIP_DEFLATED_METHOD)
            || name_size == 0
            || disk_start != 0
        {
            return Err("native-update-archive-layout-invalid");
        }
        let variable_size = name_size
            .checked_add(extra_size)
            .and_then(|value| value.checked_add(comment_size))
            .ok_or("native-update-archive-layout-invalid")?;
        let mut variable = vec![0_u8; variable_size];
        archive
            .read_exact(&mut variable)
            .map_err(|_| "native-update-archive-layout-invalid")?;
        consumed = consumed
            .checked_add(variable_size as u64)
            .ok_or("native-update-archive-layout-invalid")?;
        let name_bytes = variable
            .get(..name_size)
            .ok_or("native-update-archive-layout-invalid")?;
        let extra = variable
            .get(name_size..name_size.saturating_add(extra_size))
            .ok_or("native-update-archive-layout-invalid")?;
        validate_zip_extra_fields(extra)?;
        let name =
            std::str::from_utf8(name_bytes).map_err(|_| "native-update-archive-layout-invalid")?;
        validate_archive_path(name)?;
        let local_contract = LocalHeaderContract {
            local_offset,
            central_offset,
            crc32,
            compressed_size,
            uncompressed_size,
            flags,
            method,
            name: name_bytes,
        };
        validate_local_header(&mut archive, &local_contract)?;

        let is_directory = name.ends_with('/');
        validate_archive_entry_kind(external_attributes, is_directory)?;
        if is_directory {
            if uncompressed_size != 0 {
                return Err("native-update-archive-layout-invalid");
            }
            let directory = name.trim_end_matches('/').to_string();
            if !expected_directories.contains(&directory) || !observed_directories.insert(directory)
            {
                return Err("native-update-archive-layout-invalid");
            }
        } else {
            validate_archive_entry_size(name, uncompressed_size, contract)?;
            if !expected_files.contains(name) || !observed_files.insert(name.to_string()) {
                return Err("native-update-archive-layout-invalid");
            }
        }
    }
    if consumed != central_size || observed_files != expected_files {
        return Err("native-update-archive-layout-invalid");
    }
    Ok(())
}

fn validate_archive_entry_size(
    path: &str,
    size: u64,
    contract: &MacosStageContract<'_>,
) -> Result<(), &'static str> {
    if path == CODE_RESOURCES_PATH {
        return if (1..=MAX_CODE_RESOURCES_BYTES).contains(&size) {
            Ok(())
        } else {
            Err("native-update-archive-layout-invalid")
        };
    }
    if path == contract.desktop_manifest.manifest_path {
        return if size == contract.desktop_manifest_bytes.len() as u64 {
            Ok(())
        } else {
            Err("native-update-archive-layout-invalid")
        };
    }
    let entry = contract
        .desktop_manifest
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .ok_or("native-update-archive-layout-invalid")?;
    if matches!(
        path,
        "Qiongli.app/Contents/MacOS/Qiongli"
            | "Qiongli.app/Contents/MacOS/qiongli-cli"
            | "Qiongli.app/Contents/MacOS/qiongli-update-helper"
    ) {
        if size == 0
            || size
                > entry
                    .size_bytes
                    .saturating_add(MAX_SIGNED_BINARY_GROWTH_BYTES)
        {
            return Err("native-update-archive-layout-invalid");
        }
    } else if size != entry.size_bytes {
        return Err("native-update-archive-layout-invalid");
    }
    Ok(())
}

struct LocalHeaderContract<'a> {
    local_offset: u64,
    central_offset: u64,
    crc32: u32,
    compressed_size: u64,
    uncompressed_size: u64,
    flags: u16,
    method: u16,
    name: &'a [u8],
}

fn validate_local_header(
    archive: &mut File,
    contract: &LocalHeaderContract<'_>,
) -> Result<(), &'static str> {
    let central_position = archive
        .stream_position()
        .map_err(|_| "native-update-archive-layout-invalid")?;
    archive
        .seek(SeekFrom::Start(contract.local_offset))
        .map_err(|_| "native-update-archive-layout-invalid")?;
    let mut header = [0_u8; 30];
    archive
        .read_exact(&mut header)
        .map_err(|_| "native-update-archive-layout-invalid")?;
    let name_size = usize::from(read_u16(&header, 26)?);
    let extra_size = usize::from(read_u16(&header, 28)?);
    let local_crc32 = read_u32(&header, 14)?;
    let local_compressed_size = u64::from(read_u32(&header, 18)?);
    let local_uncompressed_size = u64::from(read_u32(&header, 22)?);
    if read_u32(&header, 0)? != ZIP_LOCAL_SIGNATURE
        || read_u16(&header, 6)? != contract.flags
        || read_u16(&header, 8)? != contract.method
        || name_size != contract.name.len()
    {
        return Err("native-update-archive-layout-invalid");
    }
    let uses_data_descriptor = contract.flags & ZIP_DATA_DESCRIPTOR_FLAG != 0;
    if (!uses_data_descriptor
        && (local_crc32 != contract.crc32
            || local_compressed_size != contract.compressed_size
            || local_uncompressed_size != contract.uncompressed_size))
        || (uses_data_descriptor
            && !((local_crc32 == 0 && local_compressed_size == 0 && local_uncompressed_size == 0)
                || (local_crc32 == contract.crc32
                    && local_compressed_size == contract.compressed_size
                    && local_uncompressed_size == contract.uncompressed_size)))
    {
        return Err("native-update-archive-layout-invalid");
    }
    let data_end = contract
        .local_offset
        .checked_add(header.len() as u64)
        .and_then(|value| value.checked_add(name_size as u64))
        .and_then(|value| value.checked_add(extra_size as u64))
        .and_then(|value| value.checked_add(contract.compressed_size))
        .ok_or("native-update-archive-layout-invalid")?;
    if contract.local_offset >= contract.central_offset || data_end > contract.central_offset {
        return Err("native-update-archive-layout-invalid");
    }
    let mut name = vec![0_u8; name_size];
    archive
        .read_exact(&mut name)
        .map_err(|_| "native-update-archive-layout-invalid")?;
    if name != contract.name {
        return Err("native-update-archive-layout-invalid");
    }
    let mut extra = vec![0_u8; extra_size];
    archive
        .read_exact(&mut extra)
        .map_err(|_| "native-update-archive-layout-invalid")?;
    validate_zip_extra_fields(&extra)?;
    archive
        .seek(SeekFrom::Start(central_position))
        .map_err(|_| "native-update-archive-layout-invalid")?;
    Ok(())
}

fn validate_zip_extra_fields(mut bytes: &[u8]) -> Result<(), &'static str> {
    while !bytes.is_empty() {
        if bytes.len() < 4 {
            return Err("native-update-archive-layout-invalid");
        }
        let identifier = read_u16(bytes, 0)?;
        let size = usize::from(read_u16(bytes, 2)?);
        if identifier == ZIP64_EXTRA_FIELD || bytes.len() < 4_usize.saturating_add(size) {
            return Err("native-update-archive-layout-invalid");
        }
        bytes = &bytes[4 + size..];
    }
    Ok(())
}

fn validate_archive_entry_kind(
    external_attributes: u32,
    is_directory: bool,
) -> Result<(), &'static str> {
    let mode = external_attributes >> 16;
    let file_type = mode & UNIX_FILE_TYPE_MASK;
    if file_type == 0 {
        let dos_directory = external_attributes & 0x10 != 0;
        if dos_directory != is_directory {
            return Err("native-update-archive-layout-invalid");
        }
        return Ok(());
    }
    if (is_directory && file_type != UNIX_DIRECTORY)
        || (!is_directory && file_type != UNIX_REGULAR_FILE)
    {
        return Err("native-update-archive-entry-unsafe");
    }
    Ok(())
}

fn expected_archive_files(manifest: &DesktopPackageManifestV1) -> BTreeSet<String> {
    manifest
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .chain([
            manifest.manifest_path.clone(),
            CODE_RESOURCES_PATH.to_string(),
        ])
        .collect()
}

fn expected_archive_directories(files: &BTreeSet<String>) -> BTreeSet<String> {
    let mut directories = BTreeSet::new();
    for file in files {
        let mut current = PathBuf::new();
        for component in Path::new(file).components() {
            let Component::Normal(component) = component else {
                continue;
            };
            current.push(component);
            if current.to_string_lossy() != *file {
                directories.insert(current.to_string_lossy().into_owned());
            }
        }
    }
    directories
}

fn validate_archive_path(path: &str) -> Result<(), &'static str> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains('\0')
        || Path::new(trimmed).components().any(|component| {
            matches!(
                component,
                Component::RootDir
                    | Component::Prefix(_)
                    | Component::CurDir
                    | Component::ParentDir
            )
        })
    {
        return Err("native-update-archive-entry-unsafe");
    }
    Ok(())
}

fn validate_extracted_application(
    application: &Path,
    contract: &MacosStageContract<'_>,
) -> Result<(), &'static str> {
    let expected_files = expected_archive_files(&contract.desktop_manifest);
    let expected_directories = expected_archive_directories(&expected_files);
    let mut observed_files = BTreeSet::new();
    let mut observed_directories = BTreeSet::new();
    inspect_tree(
        application,
        application
            .parent()
            .ok_or("native-update-application-layout-invalid")?,
        &mut observed_files,
        &mut observed_directories,
    )?;
    if observed_files != expected_files || observed_directories != expected_directories {
        return Err("native-update-application-layout-invalid");
    }

    let internal_manifest = read_regular_file(
        &application
            .parent()
            .ok_or("native-update-application-layout-invalid")?
            .join(&contract.desktop_manifest.manifest_path),
        contract.desktop_manifest_bytes.len() as u64,
    )?;
    if internal_manifest != contract.desktop_manifest_bytes {
        return Err("native-update-internal-manifest-mismatch");
    }
    for entry in &contract.desktop_manifest.entries {
        let expected_sha256 = if entry.path == "Qiongli.app/Contents/MacOS/Qiongli" {
            contract.signed_launcher_sha256
        } else if entry.path == "Qiongli.app/Contents/MacOS/qiongli-cli" {
            contract.signed_canonical_binary_sha256
        } else if entry.path == "Qiongli.app/Contents/MacOS/qiongli-update-helper" {
            contract.signed_update_helper_sha256
        } else {
            &entry.sha256
        };
        let path = application
            .parent()
            .ok_or("native-update-application-layout-invalid")?
            .join(&entry.path);
        verify_extracted_file(&path, entry.mode, entry.size_bytes, expected_sha256)?;
    }
    let code_resources = application
        .parent()
        .ok_or("native-update-application-layout-invalid")?
        .join(CODE_RESOURCES_PATH);
    let metadata = fs::symlink_metadata(code_resources)
        .map_err(|_| "native-update-code-signature-resources-missing")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > MAX_CODE_RESOURCES_BYTES
    {
        return Err("native-update-code-signature-resources-invalid");
    }
    Ok(())
}

fn inspect_tree(
    path: &Path,
    extraction_root: &Path,
    files: &mut BTreeSet<String>,
    directories: &mut BTreeSet<String>,
) -> Result<(), &'static str> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "native-update-application-layout-invalid")?;
    validate_extracted_metadata(&metadata)?;
    let relative = path
        .strip_prefix(extraction_root)
        .map_err(|_| "native-update-application-layout-invalid")?
        .to_string_lossy()
        .into_owned();
    if metadata.is_dir() {
        if !directories.insert(relative) {
            return Err("native-update-application-layout-invalid");
        }
        let mut children = fs::read_dir(path)
            .map_err(|_| "native-update-application-layout-invalid")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "native-update-application-layout-invalid")?;
        children.sort_by_key(std::fs::DirEntry::file_name);
        for child in children {
            inspect_tree(&child.path(), extraction_root, files, directories)?;
        }
    } else if metadata.is_file() {
        if !files.insert(relative) {
            return Err("native-update-application-layout-invalid");
        }
    } else {
        return Err("native-update-application-entry-unsafe");
    }
    Ok(())
}

#[cfg(unix)]
fn validate_extracted_metadata(metadata: &fs::Metadata) -> Result<(), &'static str> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if metadata.file_type().is_symlink()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o022 != 0
        || (metadata.is_file() && metadata.nlink() != 1)
    {
        return Err("native-update-application-entry-unsafe");
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_extracted_metadata(_metadata: &fs::Metadata) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

fn verify_extracted_file(
    path: &Path,
    logical_mode: LogicalMode,
    unsigned_size: u64,
    expected_sha256: &str,
) -> Result<(), &'static str> {
    let mut file =
        open_extracted_regular_file(path).map_err(|_| "native-update-application-entry-unsafe")?;
    let metadata = file
        .metadata()
        .map_err(|_| "native-update-application-entry-unsafe")?;
    validate_logical_mode(&metadata, logical_mode)?;
    let is_signed_binary = matches!(
        path.file_name().and_then(|value| value.to_str()),
        Some("Qiongli" | "qiongli-cli" | "qiongli-update-helper")
    );
    if (is_signed_binary
        && metadata.len() > unsigned_size.saturating_add(MAX_SIGNED_BINARY_GROWTH_BYTES))
        || (!is_signed_binary && metadata.len() != unsigned_size)
        || metadata.len() == 0
        || sha256_reader(&mut file)? != expected_sha256
    {
        return Err("native-update-application-digest-mismatch");
    }
    Ok(())
}

#[cfg(unix)]
fn validate_logical_mode(
    metadata: &fs::Metadata,
    logical_mode: LogicalMode,
) -> Result<(), &'static str> {
    use std::os::unix::fs::PermissionsExt;
    let executable = metadata.permissions().mode() & 0o111 != 0;
    if executable != (logical_mode == LogicalMode::Executable) {
        return Err("native-update-application-mode-mismatch");
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_logical_mode(
    _metadata: &fs::Metadata,
    _logical_mode: LogicalMode,
) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

fn read_regular_file(path: &Path, expected_size: u64) -> Result<Vec<u8>, &'static str> {
    let mut file = open_extracted_regular_file(path)
        .map_err(|_| "native-update-internal-manifest-mismatch")?;
    let metadata = file
        .metadata()
        .map_err(|_| "native-update-internal-manifest-mismatch")?;
    if metadata.len() != expected_size || expected_size == 0 {
        return Err("native-update-internal-manifest-mismatch");
    }
    let mut bytes = Vec::with_capacity(
        usize::try_from(expected_size).map_err(|_| "native-update-internal-manifest-mismatch")?,
    );
    file.read_to_end(&mut bytes)
        .map_err(|_| "native-update-internal-manifest-mismatch")?;
    if bytes.len() as u64 != expected_size {
        return Err("native-update-internal-manifest-mismatch");
    }
    Ok(bytes)
}

fn sha256_reader(file: &mut File) -> Result<String, &'static str> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| "native-update-application-digest-mismatch")?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(encode_lower_hex(&hasher.finalize()))
}

#[cfg(unix)]
fn open_private_regular_file(path: &Path) -> Result<File, &'static str> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let descriptor = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(|_| "native-update-staged-file-unavailable")?;
    let file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|_| "native-update-staged-file-unavailable")?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.nlink() != 1
    {
        return Err("native-update-staged-file-unsafe");
    }
    Ok(file)
}

#[cfg(not(unix))]
fn open_private_regular_file(_path: &Path) -> Result<File, &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(unix)]
fn open_extracted_regular_file(path: &Path) -> Result<File, &'static str> {
    let descriptor = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(|_| "native-update-application-entry-unsafe")?;
    let file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|_| "native-update-application-entry-unsafe")?;
    validate_extracted_metadata(&metadata)?;
    if !metadata.is_file() {
        return Err("native-update-application-entry-unsafe");
    }
    Ok(file)
}

#[cfg(not(unix))]
fn open_extracted_regular_file(_path: &Path) -> Result<File, &'static str> {
    Err("native-update-target-unsupported")
}

fn ensure_absent(path: &Path) -> Result<(), &'static str> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err("native-update-application-staging-conflict"),
    }
}

#[cfg(unix)]
fn validate_private_transaction_root(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let metadata = fs::symlink_metadata(path).map_err(|_| "native-update-staging-unavailable")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err("native-update-staging-unsafe");
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_transaction_root(_path: &Path) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), &'static str> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| "native-update-application-staging-failed")
}

#[cfg(not(unix))]
fn create_private_directory(_path: &Path) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(unix)]
fn rename_without_replacement(source: &Path, destination: &Path) -> Result<(), &'static str> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        source,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|_| "native-update-application-staging-failed")
}

#[cfg(not(unix))]
fn rename_without_replacement(_source: &Path, _destination: &Path) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(target_os = "macos")]
fn extract_with_ditto(archive_path: &Path, destination: &Path) -> Result<(), &'static str> {
    let arguments = [
        std::ffi::OsStr::new("-x"),
        std::ffi::OsStr::new("-k"),
        std::ffi::OsStr::new("--norsrc"),
        std::ffi::OsStr::new("--noextattr"),
        std::ffi::OsStr::new("--noqtn"),
        std::ffi::OsStr::new("--noacl"),
        archive_path.as_os_str(),
        destination.as_os_str(),
    ];
    let output = run_bounded_tool(Path::new("/usr/bin/ditto"), &arguments)?;
    if output.status_success {
        Ok(())
    } else {
        Err("native-update-archive-extraction-failed")
    }
}

#[cfg(not(target_os = "macos"))]
fn extract_with_ditto(_archive_path: &Path, _destination: &Path) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(target_os = "macos")]
fn verify_macos_trust(application: &Path, expected_team_id: &str) -> Result<(), &'static str> {
    let verify = run_bounded_tool(
        Path::new("/usr/bin/codesign"),
        &[
            std::ffi::OsStr::new("--verify"),
            std::ffi::OsStr::new("--strict"),
            std::ffi::OsStr::new("--verbose=2"),
            application.as_os_str(),
        ],
    )?;
    if !verify.status_success {
        return Err("native-update-codesign-verification-failed");
    }
    let details = run_bounded_tool(
        Path::new("/usr/bin/codesign"),
        &[
            std::ffi::OsStr::new("--display"),
            std::ffi::OsStr::new("--verbose=4"),
            application.as_os_str(),
        ],
    )?;
    if !details.status_success {
        return Err("native-update-codesign-identity-unavailable");
    }
    let combined = details.combined_output();
    let team_claim = format!("TeamIdentifier={expected_team_id}");
    if !combined
        .windows(team_claim.len())
        .any(|window| window == team_claim.as_bytes())
        || !combined
            .windows(b"Authority=Developer ID Application:".len())
            .any(|window| window == b"Authority=Developer ID Application:")
    {
        return Err("native-update-codesign-identity-mismatch");
    }
    let requirement = run_bounded_tool(
        Path::new("/usr/bin/codesign"),
        &[
            std::ffi::OsStr::new("--display"),
            std::ffi::OsStr::new("--requirements"),
            std::ffi::OsStr::new(":-"),
            application.as_os_str(),
        ],
    )?;
    if !requirement.status_success
        || !requirement
            .combined_output()
            .windows(b"identifier \"io.github.jxpeng98.qiongli\"".len())
            .any(|window| window == b"identifier \"io.github.jxpeng98.qiongli\"")
    {
        return Err("native-update-designated-requirement-mismatch");
    }
    let stapler = run_bounded_tool(
        Path::new("/usr/bin/stapler"),
        &[
            std::ffi::OsStr::new("validate"),
            std::ffi::OsStr::new("-q"),
            application.as_os_str(),
        ],
    )?;
    if !stapler.status_success {
        return Err("native-update-staple-validation-failed");
    }
    let gatekeeper = run_bounded_tool(
        Path::new("/usr/sbin/spctl"),
        &[
            std::ffi::OsStr::new("--assess"),
            std::ffi::OsStr::new("--type"),
            std::ffi::OsStr::new("execute"),
            std::ffi::OsStr::new("--verbose=4"),
            application.as_os_str(),
        ],
    )?;
    if !gatekeeper.status_success {
        return Err("native-update-gatekeeper-assessment-failed");
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn verify_macos_trust(_application: &Path, _expected_team_id: &str) -> Result<(), &'static str> {
    Err("native-update-target-unsupported")
}

#[cfg(target_os = "macos")]
struct BoundedToolOutput {
    status_success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[cfg(target_os = "macos")]
impl BoundedToolOutput {
    fn combined_output(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(self.stdout.len().saturating_add(self.stderr.len()));
        output.extend_from_slice(&self.stdout);
        output.extend_from_slice(&self.stderr);
        output
    }
}

#[cfg(target_os = "macos")]
#[allow(
    clippy::disallowed_methods,
    reason = "R3O invokes only fixed-path macOS trust tools, never a language runtime or shell"
)]
fn run_bounded_tool(
    executable: &Path,
    arguments: &[&std::ffi::OsStr],
) -> Result<BoundedToolOutput, &'static str> {
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Instant;

    let mut child = Command::new(executable)
        .args(arguments)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "native-update-platform-tool-unavailable")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("native-update-platform-tool-unavailable")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("native-update-platform-tool-unavailable")?;
    let stdout_reader = thread::spawn(move || read_bounded_output(stdout));
    let stderr_reader = thread::spawn(move || read_bounded_output(stderr));
    let deadline = Instant::now() + TOOL_TIMEOUT;
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|_| "native-update-platform-tool-failed")?
        {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("native-update-platform-tool-timeout");
        }
        thread::sleep(Duration::from_millis(10));
    };
    let (stdout, stdout_overflow) = stdout_reader
        .join()
        .map_err(|_| "native-update-platform-tool-failed")??;
    let (stderr, stderr_overflow) = stderr_reader
        .join()
        .map_err(|_| "native-update-platform-tool-failed")??;
    if stdout_overflow || stderr_overflow {
        return Err("native-update-platform-tool-output-too-large");
    }
    Ok(BoundedToolOutput {
        status_success: status.success(),
        stdout,
        stderr,
    })
}

#[cfg(target_os = "macos")]
fn read_bounded_output(mut reader: impl Read) -> Result<(Vec<u8>, bool), &'static str> {
    let mut retained = Vec::new();
    let mut overflow = false;
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|_| "native-update-platform-tool-failed")?;
        if count == 0 {
            break;
        }
        let available = MAX_TOOL_OUTPUT_BYTES.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..count.min(available)]);
        overflow |= count > available;
    }
    Ok((retained, overflow))
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, &'static str> {
    let value = bytes
        .get(offset..offset.saturating_add(2))
        .and_then(|value| value.try_into().ok())
        .ok_or("native-update-archive-layout-invalid")?;
    Ok(u16::from_le_bytes(value))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, &'static str> {
    let value = bytes
        .get(offset..offset.saturating_add(4))
        .and_then(|value| value.try_into().ok())
        .ok_or("native-update-archive-layout-invalid")?;
    Ok(u32::from_le_bytes(value))
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(all(test, unix))]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Write;

    use qiongli_platform::{
        Architecture, ArtifactIdentityV1, CapabilityProfile,
        DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION, DesktopApplicationMetadataV1,
        DesktopPackageEntryV1, DesktopPackageKind, DesktopPackageRecordType, DesktopPackageStatus,
        DesktopZoteroCompanionBindingV1, InstallerKind, OperatingSystem, ProductId, ReleaseChannel,
    };

    use super::*;

    struct FixtureTools {
        files: BTreeMap<String, (LogicalMode, Vec<u8>)>,
        trust_error: Option<&'static str>,
    }

    impl MacosPlatformTools for FixtureTools {
        fn extract(&self, _archive_path: &Path, destination: &Path) -> Result<(), &'static str> {
            for (relative, (mode, bytes)) in &self.files {
                let path = destination.join(relative);
                fs::create_dir_all(
                    path.parent()
                        .ok_or("native-update-archive-extraction-failed")?,
                )
                .map_err(|_| "native-update-archive-extraction-failed")?;
                let mut file =
                    File::create(&path).map_err(|_| "native-update-archive-extraction-failed")?;
                file.write_all(bytes)
                    .map_err(|_| "native-update-archive-extraction-failed")?;
                set_mode(&path, *mode)?;
            }
            Ok(())
        }

        fn verify_trust(
            &self,
            _application: &Path,
            _expected_team_id: &str,
        ) -> Result<(), &'static str> {
            self.trust_error.map_or(Ok(()), Err)
        }
    }

    #[test]
    fn stages_only_the_exact_regular_application_tree() {
        let fixture = fixture("valid");
        let tools = FixtureTools {
            files: fixture.files.clone(),
            trust_error: None,
        };
        stage_with_tools(
            &fixture.transaction_root,
            &fixture.archive,
            &fixture.contract(),
            &tools,
        )
        .unwrap();
        assert!(
            fixture
                .transaction_root
                .join(STAGED_DIRECTORY)
                .join(APPLICATION_ROOT)
                .is_dir()
        );
        assert!(!fixture.transaction_root.join(EXTRACTING_DIRECTORY).exists());
        let _ = fs::remove_dir_all(fixture.test_root);
    }

    #[test]
    fn rejects_archive_links_unexpected_roots_and_local_name_drift() {
        for (name, mutation) in [
            ("symlink", ZipMutation::Symlink),
            ("unexpected", ZipMutation::Unexpected),
            ("local-drift", ZipMutation::LocalNameDrift),
        ] {
            let fixture = fixture_with_mutation(name, mutation);
            assert_eq!(
                validate_zip_layout(&fixture.archive, &fixture.contract()),
                Err(if mutation == ZipMutation::Symlink {
                    "native-update-archive-entry-unsafe"
                } else {
                    "native-update-archive-layout-invalid"
                })
            );
            let _ = fs::remove_dir_all(fixture.test_root);
        }
    }

    #[test]
    fn removes_partial_application_after_digest_or_platform_trust_failure() {
        for (name, mutate_digest, trust_error, expected) in [
            (
                "digest",
                true,
                None,
                "native-update-application-digest-mismatch",
            ),
            (
                "trust",
                false,
                Some("native-update-codesign-verification-failed"),
                "native-update-codesign-verification-failed",
            ),
        ] {
            let mut fixture = fixture(name);
            if mutate_digest {
                fixture
                    .files
                    .get_mut("Qiongli.app/Contents/Resources/LICENSE")
                    .unwrap()
                    .1
                    .push(b'x');
            }
            let tools = FixtureTools {
                files: fixture.files.clone(),
                trust_error,
            };
            assert_eq!(
                stage_with_tools(
                    &fixture.transaction_root,
                    &fixture.archive,
                    &fixture.contract(),
                    &tools
                )
                .map(|_| ()),
                Err(expected)
            );
            assert!(!fixture.transaction_root.join(EXTRACTING_DIRECTORY).exists());
            assert!(!fixture.transaction_root.join(STAGED_DIRECTORY).exists());
            let _ = fs::remove_dir_all(fixture.test_root);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    #[allow(
        clippy::disallowed_methods,
        reason = "the compatibility test exercises the fixed /usr/bin/ditto archive shape"
    )]
    fn accepts_the_fixed_ditto_archive_shape_used_for_signed_applications() {
        use std::process::Command;

        let fixture = fixture("ditto-shape");
        let source_root = fixture.test_root.join("source");
        fs::create_dir(&source_root).unwrap();
        let tools = FixtureTools {
            files: fixture.files.clone(),
            trust_error: None,
        };
        tools.extract(&fixture.archive, &source_root).unwrap();
        let ditto_archive = fixture.transaction_root.join("ditto-fixture.zip");
        let source_application = source_root.join(APPLICATION_ROOT);
        let status = Command::new("/usr/bin/ditto")
            .args([
                std::ffi::OsStr::new("-c"),
                std::ffi::OsStr::new("-k"),
                std::ffi::OsStr::new("--norsrc"),
                std::ffi::OsStr::new("--noextattr"),
                std::ffi::OsStr::new("--noqtn"),
                std::ffi::OsStr::new("--noacl"),
                std::ffi::OsStr::new("--keepParent"),
                source_application.as_os_str(),
                ditto_archive.as_os_str(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        set_private_file_mode(&ditto_archive).unwrap();
        assert_eq!(
            validate_zip_layout(&ditto_archive, &fixture.contract()),
            Ok(())
        );
        let _ = fs::remove_dir_all(fixture.test_root);
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    enum ZipMutation {
        None,
        Symlink,
        Unexpected,
        LocalNameDrift,
    }

    struct Fixture {
        test_root: PathBuf,
        transaction_root: PathBuf,
        archive: PathBuf,
        manifest: DesktopPackageManifestV1,
        manifest_bytes: Vec<u8>,
        files: BTreeMap<String, (LogicalMode, Vec<u8>)>,
        signed_launcher_sha256: String,
        signed_canonical_sha256: String,
        signed_update_helper_sha256: String,
    }

    impl Fixture {
        fn contract(&self) -> MacosStageContract<'_> {
            MacosStageContract {
                desktop_manifest: self.manifest.clone(),
                desktop_manifest_bytes: &self.manifest_bytes,
                signed_launcher_sha256: &self.signed_launcher_sha256,
                signed_canonical_binary_sha256: &self.signed_canonical_sha256,
                signed_update_helper_sha256: &self.signed_update_helper_sha256,
                expected_team_id: "ABC123DEFG",
            }
        }
    }

    fn fixture(name: &str) -> Fixture {
        fixture_with_mutation(name, ZipMutation::None)
    }

    fn fixture_with_mutation(name: &str, mutation: ZipMutation) -> Fixture {
        let test_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .join("target/qiongli-update-stage-tests")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&test_root);
        fs::create_dir_all(&test_root).unwrap();
        set_directory_mode(&test_root).unwrap();
        let transaction_root = test_root.join("update-fixture");
        fs::create_dir(&transaction_root).unwrap();
        set_directory_mode(&transaction_root).unwrap();

        let launcher = b"signed-launcher".to_vec();
        let canonical = b"signed-canonical".to_vec();
        let update_helper = b"signed-update-helper".to_vec();
        let license = b"MIT License".to_vec();
        let icon = b"icns-fixture".to_vec();
        let plist = b"plist-fixture".to_vec();
        let companion = crate::embedded_zotero_companion().unwrap();
        let zotero_companion =
            DesktopZoteroCompanionBindingV1::from_artifact(OperatingSystem::Macos, &companion);
        let mut entries = vec![
            entry(
                "Qiongli.app/Contents/Info.plist",
                LogicalMode::Regular,
                &plist,
            ),
            entry(
                "Qiongli.app/Contents/MacOS/Qiongli",
                LogicalMode::Executable,
                b"unsigned-launcher",
            ),
            entry(
                "Qiongli.app/Contents/MacOS/qiongli-cli",
                LogicalMode::Executable,
                b"unsigned-canonical",
            ),
            entry(
                "Qiongli.app/Contents/MacOS/qiongli-update-helper",
                LogicalMode::Executable,
                b"unsigned-update-helper",
            ),
            entry(
                "Qiongli.app/Contents/Resources/LICENSE",
                LogicalMode::Regular,
                &license,
            ),
            entry(
                "Qiongli.app/Contents/Resources/Qiongli.icns",
                LogicalMode::Regular,
                &icon,
            ),
            entry(
                &zotero_companion.xpi_path,
                LogicalMode::Regular,
                companion.xpi_bytes(),
            ),
            entry(
                &zotero_companion.artifact_manifest_path,
                LogicalMode::Regular,
                companion.manifest_bytes(),
            ),
        ];
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        let manifest = DesktopPackageManifestV1 {
            schema_version: DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION,
            record_type: DesktopPackageRecordType::QiongliDesktopPackage,
            status: DesktopPackageStatus::AssembledUnpublished,
            package_kind: DesktopPackageKind::MacosApplicationZip,
            artifact: identity(InstallerKind::NativeInstaller),
            source_artifact: identity(InstallerKind::PortableArchive),
            product_source_commit: "a".repeat(40),
            source_artifact_manifest_sha256: "1".repeat(64),
            resource_pack_sha256: "2".repeat(64),
            canonical_binary_sha256: sha256(b"unsigned-canonical"),
            launcher_sha256: sha256(b"unsigned-launcher"),
            update_helper_sha256: sha256(b"unsigned-update-helper"),
            product_control_sha256: None,
            zotero_companion: zotero_companion.clone(),
            application: DesktopApplicationMetadataV1::new(
                "Qiongli",
                "Qiongli 2",
                "io.github.jxpeng98.qiongli",
                "2.0.0-alpha.2",
                "MIT",
            ),
            package_root: APPLICATION_ROOT.to_string(),
            manifest_path: "Qiongli.app/Contents/Resources/.qiongli-desktop-package.json"
                .to_string(),
            entry_content_root_sha256: "3".repeat(64),
            entries,
        };
        let manifest_bytes = serde_json_canonicalizer::to_vec(&manifest).unwrap();
        let mut files = BTreeMap::from([
            (
                "Qiongli.app/Contents/Info.plist".to_string(),
                (LogicalMode::Regular, plist),
            ),
            (
                "Qiongli.app/Contents/MacOS/Qiongli".to_string(),
                (LogicalMode::Executable, launcher.clone()),
            ),
            (
                "Qiongli.app/Contents/MacOS/qiongli-cli".to_string(),
                (LogicalMode::Executable, canonical.clone()),
            ),
            (
                "Qiongli.app/Contents/MacOS/qiongli-update-helper".to_string(),
                (LogicalMode::Executable, update_helper.clone()),
            ),
            (
                "Qiongli.app/Contents/Resources/LICENSE".to_string(),
                (LogicalMode::Regular, license),
            ),
            (
                "Qiongli.app/Contents/Resources/Qiongli.icns".to_string(),
                (LogicalMode::Regular, icon),
            ),
            (
                zotero_companion.xpi_path.clone(),
                (LogicalMode::Regular, companion.xpi_bytes().to_vec()),
            ),
            (
                zotero_companion.artifact_manifest_path.clone(),
                (LogicalMode::Regular, companion.manifest_bytes().to_vec()),
            ),
            (
                manifest.manifest_path.clone(),
                (LogicalMode::Regular, manifest_bytes.clone()),
            ),
            (
                CODE_RESOURCES_PATH.to_string(),
                (LogicalMode::Regular, b"code-resources".to_vec()),
            ),
        ]);
        if mutation == ZipMutation::Unexpected {
            files.insert(
                "unexpected.txt".to_string(),
                (LogicalMode::Regular, b"unexpected".to_vec()),
            );
        }
        let archive = transaction_root.join("fixture.zip");
        write_fixture_zip(&archive, &files, mutation).unwrap();
        Fixture {
            test_root,
            transaction_root,
            archive,
            manifest,
            manifest_bytes,
            files,
            signed_launcher_sha256: sha256(&launcher),
            signed_canonical_sha256: sha256(&canonical),
            signed_update_helper_sha256: sha256(&update_helper),
        }
    }

    fn identity(installer_kind: InstallerKind) -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: "2.0.0-alpha.2".to_string(),
            channel: ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os: OperatingSystem::Macos,
            arch: Architecture::Aarch64,
            installer_kind,
        }
    }

    fn entry(path: &str, mode: LogicalMode, bytes: &[u8]) -> DesktopPackageEntryV1 {
        DesktopPackageEntryV1 {
            path: path.to_string(),
            mode,
            size_bytes: bytes.len() as u64,
            sha256: sha256(bytes),
        }
    }

    fn sha256(bytes: &[u8]) -> String {
        encode_lower_hex(&Sha256::digest(bytes))
    }

    fn write_fixture_zip(
        path: &Path,
        files: &BTreeMap<String, (LogicalMode, Vec<u8>)>,
        mutation: ZipMutation,
    ) -> Result<(), std::io::Error> {
        let expected_files = files.keys().cloned().collect::<BTreeSet<_>>();
        let directories = expected_archive_directories(&expected_files);
        let entries = directories
            .into_iter()
            .map(|path| (format!("{path}/"), LogicalMode::Regular, Vec::new(), true))
            .chain(
                files
                    .iter()
                    .map(|(path, (mode, bytes))| (path.clone(), *mode, bytes.clone(), false)),
            )
            .collect::<Vec<_>>();
        let mut bytes = Vec::new();
        let mut central = Vec::new();
        for (index, (name, mode, payload, directory)) in entries.iter().enumerate() {
            let local_offset = bytes.len() as u32;
            let local_name = if mutation == ZipMutation::LocalNameDrift && index == 0 {
                b"drift/".as_slice()
            } else {
                name.as_bytes()
            };
            push_u32(&mut bytes, ZIP_LOCAL_SIGNATURE);
            push_u16(&mut bytes, 20);
            push_u16(&mut bytes, 0);
            push_u16(&mut bytes, ZIP_STORED_METHOD);
            bytes.extend_from_slice(&[0_u8; 8]);
            push_u32(&mut bytes, payload.len() as u32);
            push_u32(&mut bytes, payload.len() as u32);
            push_u16(&mut bytes, local_name.len() as u16);
            push_u16(&mut bytes, 0);
            bytes.extend_from_slice(local_name);
            bytes.extend_from_slice(payload);

            push_u32(&mut central, ZIP_CENTRAL_SIGNATURE);
            push_u16(&mut central, (3 << 8) | 20);
            push_u16(&mut central, 20);
            push_u16(&mut central, 0);
            push_u16(&mut central, ZIP_STORED_METHOD);
            central.extend_from_slice(&[0_u8; 8]);
            push_u32(&mut central, payload.len() as u32);
            push_u32(&mut central, payload.len() as u32);
            push_u16(&mut central, name.len() as u16);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            push_u16(&mut central, 0);
            let unix_mode = if mutation == ZipMutation::Symlink && index == 0 {
                0o120777
            } else if *directory {
                0o040755
            } else if *mode == LogicalMode::Executable {
                0o100755
            } else {
                0o100644
            };
            push_u32(&mut central, unix_mode << 16);
            push_u32(&mut central, local_offset);
            central.extend_from_slice(name.as_bytes());
        }
        let central_offset = bytes.len() as u32;
        bytes.extend_from_slice(&central);
        push_u32(&mut bytes, ZIP_EOCD_SIGNATURE);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, entries.len() as u16);
        push_u16(&mut bytes, entries.len() as u16);
        push_u32(&mut bytes, central.len() as u32);
        push_u32(&mut bytes, central_offset);
        push_u16(&mut bytes, 0);
        fs::write(path, bytes)?;
        set_private_file_mode(path)
    }

    fn push_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    #[cfg(unix)]
    fn set_mode(path: &Path, mode: LogicalMode) -> Result<(), &'static str> {
        use std::os::unix::fs::PermissionsExt;
        let value = if mode == LogicalMode::Executable {
            0o700
        } else {
            0o600
        };
        fs::set_permissions(path, fs::Permissions::from_mode(value))
            .map_err(|_| "native-update-archive-extraction-failed")
    }

    #[cfg(unix)]
    fn set_directory_mode(path: &Path) -> Result<(), &'static str> {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|_| "native-update-archive-extraction-failed")
    }

    #[cfg(unix)]
    fn set_private_file_mode(path: &Path) -> Result<(), std::io::Error> {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
    }
}
