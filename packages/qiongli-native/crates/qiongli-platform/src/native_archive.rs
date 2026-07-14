use std::fmt::{self, Debug, Display, Formatter};
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use qiongli_content::LoadedResourcePack;
use same_file::Handle;
use sha2::{Digest, Sha256};

use crate::native_artifact::{
    NativeArtifactPayload, commit_native_artifact_payload, read_native_artifact_payload,
    verify_native_artifact_payload,
};
use crate::{
    ArtifactIdentityV1, NativeArtifactError, NativeArtifactTarget, VerifiedNativeArtifact,
    native_artifact_binary_path, native_artifact_id,
};

pub const NATIVE_PORTABLE_ARCHIVE_EXTENSION: &str = "zip";

const MAX_ARCHIVE_BYTES: u64 = 129 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
const MAX_BINARY_BYTES: u64 = 128 * 1024 * 1024;
const ZIP_ENTRY_COUNT: u16 = 4;
const ZIP_VERSION: u16 = 20;
const ZIP_VERSION_MADE_BY_UNIX: u16 = (3 << 8) | ZIP_VERSION;
const ZIP_UTF8_FLAG: u16 = 0x0800;
const ZIP_STORED_METHOD: u16 = 0;
const ZIP_DOS_TIME: u16 = 0;
const ZIP_DOS_DATE: u16 = 0x0021;
const ZIP_LOCAL_HEADER: u32 = 0x0403_4b50;
const ZIP_CENTRAL_HEADER: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const ZIP_DIRECTORY_ATTRIBUTES: u32 = (0o040755_u32 << 16) | 0x10;
const ZIP_REGULAR_ATTRIBUTES: u32 = 0o100644_u32 << 16;
const ZIP_EXECUTABLE_ATTRIBUTES: u32 = 0o100755_u32 << 16;
const TARGET_LOCK_FILE: &str = ".qiongli.qiongli-native-portable-archive.lock";
static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct NativePortableArchiveTarget {
    path: PathBuf,
    artifact: ArtifactIdentityV1,
    artifact_id: String,
    file_name: String,
}

impl NativePortableArchiveTarget {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn artifact(&self) -> &ArtifactIdentityV1 {
        &self.artifact
    }

    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

impl Debug for NativePortableArchiveTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativePortableArchiveTarget")
            .field("path", &"<approved-native-portable-archive>")
            .field("artifact", &self.artifact)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedNativePortableArchive {
    artifact: ArtifactIdentityV1,
    file_name: String,
    size_bytes: u64,
    archive_sha256: String,
    manifest_sha256: String,
}

impl VerifiedNativePortableArchive {
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactIdentityV1 {
        &self.artifact
    }

    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    #[must_use]
    pub const fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    #[must_use]
    pub fn archive_sha256(&self) -> &str {
        &self.archive_sha256
    }

    #[must_use]
    pub fn manifest_sha256(&self) -> &str {
        &self.manifest_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePortableArchiveError {
    UnsupportedPlatform,
    InvalidIdentity,
    InvalidTarget,
    UnsafeTarget,
    TargetExists,
    TargetBusy,
    SourceArtifactInvalid,
    ArchiveMissing,
    ArchiveInvalid,
    ArchiveTooLarge,
    ArchiveDrift,
    DestinationExists,
    DestinationBusy,
    DestinationUnsafe,
    ExtractionFailed,
    PersistenceFailed(io::ErrorKind),
    CommitFailed(io::ErrorKind),
    CommittedPersistenceFailed(io::ErrorKind),
    CommittedVerificationFailed,
}

impl NativePortableArchiveError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "native-portable-archive-platform-unsupported",
            Self::InvalidIdentity => "native-portable-archive-identity-invalid",
            Self::InvalidTarget => "native-portable-archive-target-invalid",
            Self::UnsafeTarget => "native-portable-archive-target-unsafe",
            Self::TargetExists => "native-portable-archive-target-exists",
            Self::TargetBusy => "native-portable-archive-target-busy",
            Self::SourceArtifactInvalid => "native-portable-archive-source-invalid",
            Self::ArchiveMissing => "native-portable-archive-missing",
            Self::ArchiveInvalid => "native-portable-archive-invalid",
            Self::ArchiveTooLarge => "native-portable-archive-too-large",
            Self::ArchiveDrift => "native-portable-archive-drift",
            Self::DestinationExists => "native-portable-archive-destination-exists",
            Self::DestinationBusy => "native-portable-archive-destination-busy",
            Self::DestinationUnsafe => "native-portable-archive-destination-unsafe",
            Self::ExtractionFailed => "native-portable-archive-extraction-failed",
            Self::PersistenceFailed(_) => "native-portable-archive-persistence-failed",
            Self::CommitFailed(_) => "native-portable-archive-commit-failed",
            Self::CommittedPersistenceFailed(_) => {
                "native-portable-archive-committed-persistence-failed"
            }
            Self::CommittedVerificationFailed => {
                "native-portable-archive-committed-verification-failed"
            }
        }
    }
}

impl Display for NativePortableArchiveError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())?;
        match self {
            Self::PersistenceFailed(kind)
            | Self::CommitFailed(kind)
            | Self::CommittedPersistenceFailed(kind) => write!(formatter, " ({kind:?})"),
            _ => Ok(()),
        }
    }
}

impl std::error::Error for NativePortableArchiveError {}

pub fn native_portable_archive_file_name(
    artifact: &ArtifactIdentityV1,
) -> Result<String, NativePortableArchiveError> {
    let artifact_id = native_artifact_id(artifact).map_err(map_identity_error)?;
    Ok(format!("{artifact_id}.{NATIVE_PORTABLE_ARCHIVE_EXTENSION}"))
}

/// Approves a caller-selected archive path at a trusted CLI, UI, release, or test boundary.
///
/// Model-generated and MCP-provided paths must not be passed to this function.
pub fn approve_native_portable_archive_target(
    path: impl AsRef<Path>,
    artifact: &ArtifactIdentityV1,
) -> Result<NativePortableArchiveTarget, NativePortableArchiveError> {
    let artifact_id = native_artifact_id(artifact).map_err(map_identity_error)?;
    let file_name = native_portable_archive_file_name(artifact)?;
    let path = path.as_ref();
    validate_target_path(path, &file_name)?;
    validate_target_parent(path)?;
    if let Some(metadata) = path_metadata(path)?
        && (metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_file())
    {
        return Err(NativePortableArchiveError::UnsafeTarget);
    }
    Ok(NativePortableArchiveTarget {
        path: path.to_path_buf(),
        artifact: artifact.clone(),
        artifact_id,
        file_name,
    })
}

pub fn compose_native_portable_archive(
    pack: &LoadedResourcePack<'_>,
    source: &NativeArtifactTarget,
    target: &NativePortableArchiveTarget,
) -> Result<VerifiedNativePortableArchive, NativePortableArchiveError> {
    revalidate_target(target)?;
    if path_metadata(target.path())?.is_some() {
        return Err(NativePortableArchiveError::TargetExists);
    }
    if source.artifact() != target.artifact() || source.artifact_id() != target.artifact_id() {
        return Err(NativePortableArchiveError::InvalidIdentity);
    }
    let payload = read_native_artifact_payload(pack, source).map_err(map_source_error)?;
    if payload.verified.manifest().artifact != *target.artifact() {
        return Err(NativePortableArchiveError::SourceArtifactInvalid);
    }
    let archive_bytes = build_archive_bytes(target.artifact_id(), &payload)?;
    let expected = verified_archive(
        target.artifact(),
        &archive_bytes,
        payload.verified.manifest_sha256(),
    )?;

    let _lock = TargetLock::acquire(target)?;
    revalidate_target(target)?;
    if path_metadata(target.path())?.is_some() {
        return Err(NativePortableArchiveError::TargetExists);
    }
    let parent = target
        .path()
        .parent()
        .ok_or(NativePortableArchiveError::InvalidTarget)?;
    let (staging, staging_file) = create_staging_file(parent)?;
    let cleanup = FileCleanup::new(staging.clone());
    write_archive_file(&staging, staging_file, &archive_bytes)?;
    let staged_bytes = read_archive_file(&staging)?;
    let staged = parse_archive_bytes(pack, target.artifact(), &staged_bytes)?.verified;
    if staged != expected {
        return Err(NativePortableArchiveError::ArchiveDrift);
    }

    revalidate_target(target)?;
    if path_metadata(target.path())?.is_some() {
        return Err(NativePortableArchiveError::TargetExists);
    }
    rename_no_replace(&staging, target.path())?;
    cleanup.disarm();
    sync_directory(parent).map_err(|error| match error {
        NativePortableArchiveError::PersistenceFailed(kind) => {
            NativePortableArchiveError::CommittedPersistenceFailed(kind)
        }
        other => other,
    })?;

    verify_native_portable_archive(pack, target)
        .map_err(|_| NativePortableArchiveError::CommittedVerificationFailed)
}

pub fn verify_native_portable_archive(
    pack: &LoadedResourcePack<'_>,
    target: &NativePortableArchiveTarget,
) -> Result<VerifiedNativePortableArchive, NativePortableArchiveError> {
    revalidate_target(target)?;
    let bytes = read_archive_file(target.path())?;
    Ok(parse_archive_bytes(pack, target.artifact(), &bytes)?.verified)
}

pub fn extract_native_portable_archive(
    pack: &LoadedResourcePack<'_>,
    source: &NativePortableArchiveTarget,
    destination: &NativeArtifactTarget,
) -> Result<VerifiedNativeArtifact, NativePortableArchiveError> {
    revalidate_target(source)?;
    let bytes = read_archive_file(source.path())?;
    let parsed = parse_archive_bytes(pack, source.artifact(), &bytes)?;
    if destination.artifact() != parsed.verified.artifact()
        || destination.artifact_id() != source.artifact_id()
    {
        return Err(NativePortableArchiveError::DestinationUnsafe);
    }
    commit_native_artifact_payload(
        pack,
        parsed.verified.artifact(),
        parsed.manifest_bytes,
        parsed.binary_bytes,
        destination,
    )
    .map_err(map_extraction_error)
}

struct ParsedArchive<'a> {
    verified: VerifiedNativePortableArchive,
    manifest_bytes: &'a [u8],
    binary_bytes: &'a [u8],
}

#[derive(Clone, Copy)]
struct ZipEntry<'a> {
    name: &'a str,
    data: &'a [u8],
    external_attributes: u32,
    max_size: u64,
}

struct CentralRecord {
    name: String,
    crc32: u32,
    size: u32,
    external_attributes: u32,
    local_offset: u32,
}

fn build_archive_bytes(
    artifact_id: &str,
    payload: &NativeArtifactPayload,
) -> Result<Vec<u8>, NativePortableArchiveError> {
    let binary_path = &payload.verified.manifest().binary_path;
    build_zip_bytes(
        artifact_id,
        binary_path,
        &payload.manifest_bytes,
        &payload.binary_bytes,
    )
}

fn build_zip_bytes(
    artifact_id: &str,
    binary_path: &str,
    manifest_bytes: &[u8],
    binary_bytes: &[u8],
) -> Result<Vec<u8>, NativePortableArchiveError> {
    if manifest_bytes.len() as u64 > MAX_MANIFEST_BYTES
        || binary_bytes.is_empty()
        || binary_bytes.len() as u64 > MAX_BINARY_BYTES
    {
        return Err(NativePortableArchiveError::ArchiveTooLarge);
    }
    let root = format!("{artifact_id}/");
    let manifest = format!("{artifact_id}/{}", crate::NATIVE_ARTIFACT_MANIFEST_FILE);
    let bin = format!("{artifact_id}/bin/");
    let binary = format!("{artifact_id}/{binary_path}");
    let entries = [
        ZipEntry {
            name: &root,
            data: &[],
            external_attributes: ZIP_DIRECTORY_ATTRIBUTES,
            max_size: 0,
        },
        ZipEntry {
            name: &manifest,
            data: manifest_bytes,
            external_attributes: ZIP_REGULAR_ATTRIBUTES,
            max_size: MAX_MANIFEST_BYTES,
        },
        ZipEntry {
            name: &bin,
            data: &[],
            external_attributes: ZIP_DIRECTORY_ATTRIBUTES,
            max_size: 0,
        },
        ZipEntry {
            name: &binary,
            data: binary_bytes,
            external_attributes: ZIP_EXECUTABLE_ATTRIBUTES,
            max_size: MAX_BINARY_BYTES,
        },
    ];
    let capacity = manifest_bytes
        .len()
        .checked_add(binary_bytes.len())
        .and_then(|value| value.checked_add(2_048))
        .ok_or(NativePortableArchiveError::ArchiveTooLarge)?;
    let mut bytes = Vec::with_capacity(capacity);
    let mut central = Vec::with_capacity(entries.len());

    for entry in entries {
        if entry.data.len() as u64 > entry.max_size {
            return Err(NativePortableArchiveError::ArchiveTooLarge);
        }
        let name_length = u16::try_from(entry.name.len())
            .map_err(|_| NativePortableArchiveError::ArchiveInvalid)?;
        let size = u32::try_from(entry.data.len())
            .map_err(|_| NativePortableArchiveError::ArchiveTooLarge)?;
        let local_offset =
            u32::try_from(bytes.len()).map_err(|_| NativePortableArchiveError::ArchiveTooLarge)?;
        let crc32 = crc32fast::hash(entry.data);
        push_u32(&mut bytes, ZIP_LOCAL_HEADER);
        push_u16(&mut bytes, ZIP_VERSION);
        push_u16(&mut bytes, ZIP_UTF8_FLAG);
        push_u16(&mut bytes, ZIP_STORED_METHOD);
        push_u16(&mut bytes, ZIP_DOS_TIME);
        push_u16(&mut bytes, ZIP_DOS_DATE);
        push_u32(&mut bytes, crc32);
        push_u32(&mut bytes, size);
        push_u32(&mut bytes, size);
        push_u16(&mut bytes, name_length);
        push_u16(&mut bytes, 0);
        bytes.extend_from_slice(entry.name.as_bytes());
        bytes.extend_from_slice(entry.data);
        central.push(CentralRecord {
            name: entry.name.to_string(),
            crc32,
            size,
            external_attributes: entry.external_attributes,
            local_offset,
        });
    }

    let central_offset =
        u32::try_from(bytes.len()).map_err(|_| NativePortableArchiveError::ArchiveTooLarge)?;
    for record in &central {
        let name_length = u16::try_from(record.name.len())
            .map_err(|_| NativePortableArchiveError::ArchiveInvalid)?;
        push_u32(&mut bytes, ZIP_CENTRAL_HEADER);
        push_u16(&mut bytes, ZIP_VERSION_MADE_BY_UNIX);
        push_u16(&mut bytes, ZIP_VERSION);
        push_u16(&mut bytes, ZIP_UTF8_FLAG);
        push_u16(&mut bytes, ZIP_STORED_METHOD);
        push_u16(&mut bytes, ZIP_DOS_TIME);
        push_u16(&mut bytes, ZIP_DOS_DATE);
        push_u32(&mut bytes, record.crc32);
        push_u32(&mut bytes, record.size);
        push_u32(&mut bytes, record.size);
        push_u16(&mut bytes, name_length);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u32(&mut bytes, record.external_attributes);
        push_u32(&mut bytes, record.local_offset);
        bytes.extend_from_slice(record.name.as_bytes());
    }
    let central_size = u32::try_from(bytes.len())
        .ok()
        .and_then(|end| end.checked_sub(central_offset))
        .ok_or(NativePortableArchiveError::ArchiveTooLarge)?;
    push_u32(&mut bytes, ZIP_END_OF_CENTRAL_DIRECTORY);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, ZIP_ENTRY_COUNT);
    push_u16(&mut bytes, ZIP_ENTRY_COUNT);
    push_u32(&mut bytes, central_size);
    push_u32(&mut bytes, central_offset);
    push_u16(&mut bytes, 0);
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(NativePortableArchiveError::ArchiveTooLarge);
    }
    Ok(bytes)
}

fn parse_archive_bytes<'a>(
    pack: &LoadedResourcePack<'_>,
    artifact: &ArtifactIdentityV1,
    bytes: &'a [u8],
) -> Result<ParsedArchive<'a>, NativePortableArchiveError> {
    let payload = parse_zip_entries(artifact, bytes)?;
    let artifact_id = native_artifact_id(artifact).map_err(map_identity_error)?;
    let verified_artifact = verify_native_artifact_payload(
        pack,
        &artifact_id,
        payload.manifest_bytes,
        payload.binary_bytes,
    )
    .map_err(map_archive_payload_error)?;
    if verified_artifact.manifest().artifact != *artifact {
        return Err(NativePortableArchiveError::ArchiveDrift);
    }
    Ok(ParsedArchive {
        verified: verified_archive(artifact, bytes, verified_artifact.manifest_sha256())?,
        manifest_bytes: payload.manifest_bytes,
        binary_bytes: payload.binary_bytes,
    })
}

struct ParsedZipEntries<'a> {
    manifest_bytes: &'a [u8],
    binary_bytes: &'a [u8],
}

fn parse_zip_entries<'a>(
    artifact: &ArtifactIdentityV1,
    bytes: &'a [u8],
) -> Result<ParsedZipEntries<'a>, NativePortableArchiveError> {
    if bytes.is_empty() {
        return Err(NativePortableArchiveError::ArchiveInvalid);
    }
    if bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(NativePortableArchiveError::ArchiveTooLarge);
    }
    let artifact_id = native_artifact_id(artifact).map_err(map_identity_error)?;
    let binary_path = native_artifact_binary_path(artifact).map_err(map_identity_error)?;
    let root = format!("{artifact_id}/");
    let manifest = format!("{artifact_id}/{}", crate::NATIVE_ARTIFACT_MANIFEST_FILE);
    let bin = format!("{artifact_id}/bin/");
    let binary = format!("{artifact_id}/{binary_path}");
    let expected = [
        ZipEntry {
            name: &root,
            data: &[],
            external_attributes: ZIP_DIRECTORY_ATTRIBUTES,
            max_size: 0,
        },
        ZipEntry {
            name: &manifest,
            data: &[],
            external_attributes: ZIP_REGULAR_ATTRIBUTES,
            max_size: MAX_MANIFEST_BYTES,
        },
        ZipEntry {
            name: &bin,
            data: &[],
            external_attributes: ZIP_DIRECTORY_ATTRIBUTES,
            max_size: 0,
        },
        ZipEntry {
            name: &binary,
            data: &[],
            external_attributes: ZIP_EXECUTABLE_ATTRIBUTES,
            max_size: MAX_BINARY_BYTES,
        },
    ];
    let mut reader = SliceReader::new(bytes);
    let mut local_records = Vec::with_capacity(expected.len());
    for entry in expected {
        let offset = u32::try_from(reader.position())
            .map_err(|_| NativePortableArchiveError::ArchiveInvalid)?;
        if reader.read_u32()? != ZIP_LOCAL_HEADER
            || reader.read_u16()? != ZIP_VERSION
            || reader.read_u16()? != ZIP_UTF8_FLAG
            || reader.read_u16()? != ZIP_STORED_METHOD
            || reader.read_u16()? != ZIP_DOS_TIME
            || reader.read_u16()? != ZIP_DOS_DATE
        {
            return Err(NativePortableArchiveError::ArchiveInvalid);
        }
        let crc32 = reader.read_u32()?;
        let compressed_size = reader.read_u32()?;
        let uncompressed_size = reader.read_u32()?;
        let name_length = usize::from(reader.read_u16()?);
        if reader.read_u16()? != 0
            || compressed_size != uncompressed_size
            || u64::from(uncompressed_size) > entry.max_size
        {
            return Err(NativePortableArchiveError::ArchiveInvalid);
        }
        let name = reader.take(name_length)?;
        if name != entry.name.as_bytes() {
            return Err(NativePortableArchiveError::ArchiveInvalid);
        }
        let data = reader.take(
            usize::try_from(uncompressed_size)
                .map_err(|_| NativePortableArchiveError::ArchiveInvalid)?,
        )?;
        if crc32fast::hash(data) != crc32 {
            return Err(NativePortableArchiveError::ArchiveInvalid);
        }
        local_records.push(ParsedLocalRecord {
            name: entry.name.to_string(),
            data,
            crc32,
            size: uncompressed_size,
            external_attributes: entry.external_attributes,
            offset,
        });
    }

    let central_offset =
        u32::try_from(reader.position()).map_err(|_| NativePortableArchiveError::ArchiveInvalid)?;
    for local in &local_records {
        if reader.read_u32()? != ZIP_CENTRAL_HEADER
            || reader.read_u16()? != ZIP_VERSION_MADE_BY_UNIX
            || reader.read_u16()? != ZIP_VERSION
            || reader.read_u16()? != ZIP_UTF8_FLAG
            || reader.read_u16()? != ZIP_STORED_METHOD
            || reader.read_u16()? != ZIP_DOS_TIME
            || reader.read_u16()? != ZIP_DOS_DATE
            || reader.read_u32()? != local.crc32
            || reader.read_u32()? != local.size
            || reader.read_u32()? != local.size
        {
            return Err(NativePortableArchiveError::ArchiveInvalid);
        }
        let name_length = usize::from(reader.read_u16()?);
        if reader.read_u16()? != 0
            || reader.read_u16()? != 0
            || reader.read_u16()? != 0
            || reader.read_u16()? != 0
            || reader.read_u32()? != local.external_attributes
            || reader.read_u32()? != local.offset
            || reader.take(name_length)? != local.name.as_bytes()
        {
            return Err(NativePortableArchiveError::ArchiveInvalid);
        }
    }
    let central_end =
        u32::try_from(reader.position()).map_err(|_| NativePortableArchiveError::ArchiveInvalid)?;
    let central_size = central_end
        .checked_sub(central_offset)
        .ok_or(NativePortableArchiveError::ArchiveInvalid)?;
    if reader.read_u32()? != ZIP_END_OF_CENTRAL_DIRECTORY
        || reader.read_u16()? != 0
        || reader.read_u16()? != 0
        || reader.read_u16()? != ZIP_ENTRY_COUNT
        || reader.read_u16()? != ZIP_ENTRY_COUNT
        || reader.read_u32()? != central_size
        || reader.read_u32()? != central_offset
        || reader.read_u16()? != 0
        || !reader.is_finished()
    {
        return Err(NativePortableArchiveError::ArchiveInvalid);
    }

    Ok(ParsedZipEntries {
        manifest_bytes: local_records[1].data,
        binary_bytes: local_records[3].data,
    })
}

fn verified_archive(
    artifact: &ArtifactIdentityV1,
    bytes: &[u8],
    manifest_sha256: &str,
) -> Result<VerifiedNativePortableArchive, NativePortableArchiveError> {
    let size_bytes =
        u64::try_from(bytes.len()).map_err(|_| NativePortableArchiveError::ArchiveTooLarge)?;
    Ok(VerifiedNativePortableArchive {
        artifact: artifact.clone(),
        file_name: native_portable_archive_file_name(artifact)?,
        size_bytes,
        archive_sha256: sha256_hex(bytes),
        manifest_sha256: manifest_sha256.to_string(),
    })
}

struct ParsedLocalRecord<'a> {
    name: String,
    data: &'a [u8],
    crc32: u32,
    size: u32,
    external_attributes: u32,
    offset: u32,
}

struct SliceReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> SliceReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    const fn position(&self) -> usize {
        self.position
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], NativePortableArchiveError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(NativePortableArchiveError::ArchiveInvalid)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(NativePortableArchiveError::ArchiveInvalid)?;
        self.position = end;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, NativePortableArchiveError> {
        let value: [u8; 2] = self
            .take(2)?
            .try_into()
            .map_err(|_| NativePortableArchiveError::ArchiveInvalid)?;
        Ok(u16::from_le_bytes(value))
    }

    fn read_u32(&mut self) -> Result<u32, NativePortableArchiveError> {
        let value: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| NativePortableArchiveError::ArchiveInvalid)?;
        Ok(u32::from_le_bytes(value))
    }

    const fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn validate_target_path(
    path: &Path,
    expected_file_name: &str,
) -> Result<(), NativePortableArchiveError> {
    if !path.is_absolute()
        || path.parent().is_none()
        || path.file_name().and_then(|name| name.to_str()) != Some(expected_file_name)
        || path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(NativePortableArchiveError::InvalidTarget);
    }
    Ok(())
}

fn validate_target_parent(path: &Path) -> Result<(), NativePortableArchiveError> {
    let parent = path
        .parent()
        .ok_or(NativePortableArchiveError::InvalidTarget)?;
    let mut current = PathBuf::new();
    for component in parent.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => current.push(component.as_os_str()),
            Component::Normal(_) => {
                current.push(component.as_os_str());
                let metadata = fs::symlink_metadata(&current)
                    .map_err(|_| NativePortableArchiveError::UnsafeTarget)?;
                if metadata.file_type().is_symlink()
                    || is_reparse_point(&metadata)
                    || !metadata.is_dir()
                {
                    return Err(NativePortableArchiveError::UnsafeTarget);
                }
                validate_parent_mode(&metadata)?;
            }
            Component::CurDir | Component::ParentDir => {
                return Err(NativePortableArchiveError::InvalidTarget);
            }
        }
    }
    validate_windows_parent(parent)
}

#[cfg(unix)]
fn validate_parent_mode(metadata: &Metadata) -> Result<(), NativePortableArchiveError> {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(NativePortableArchiveError::UnsafeTarget);
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_parent_mode(_metadata: &Metadata) -> Result<(), NativePortableArchiveError> {
    Ok(())
}

#[cfg(windows)]
fn validate_windows_parent(path: &Path) -> Result<(), NativePortableArchiveError> {
    qiongli_windows_security::open_owner_only_directory(path)
        .map(|_| ())
        .map_err(|_| NativePortableArchiveError::UnsafeTarget)
}

#[cfg(not(windows))]
fn validate_windows_parent(_path: &Path) -> Result<(), NativePortableArchiveError> {
    Ok(())
}

fn revalidate_target(
    target: &NativePortableArchiveTarget,
) -> Result<(), NativePortableArchiveError> {
    let artifact_id = native_artifact_id(target.artifact()).map_err(map_identity_error)?;
    let file_name = native_portable_archive_file_name(target.artifact())?;
    if artifact_id != target.artifact_id || file_name != target.file_name {
        return Err(NativePortableArchiveError::InvalidIdentity);
    }
    validate_target_path(target.path(), &file_name)?;
    validate_target_parent(target.path())?;
    if let Some(metadata) = path_metadata(target.path())?
        && (metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_file())
    {
        return Err(NativePortableArchiveError::UnsafeTarget);
    }
    Ok(())
}

fn read_archive_file(path: &Path) -> Result<Vec<u8>, NativePortableArchiveError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            NativePortableArchiveError::ArchiveMissing
        } else {
            NativePortableArchiveError::PersistenceFailed(error.kind())
        }
    })?;
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) || !metadata.is_file() {
        return Err(NativePortableArchiveError::ArchiveDrift);
    }
    if metadata.len() > MAX_ARCHIVE_BYTES {
        return Err(NativePortableArchiveError::ArchiveTooLarge);
    }
    if metadata.len() == 0 {
        return Err(NativePortableArchiveError::ArchiveInvalid);
    }
    verify_archive_file_security(path, &metadata)?;
    let mut file = File::open(path)
        .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()))?;
    let mut bytes = Vec::with_capacity(
        usize::try_from(metadata.len()).map_err(|_| NativePortableArchiveError::ArchiveTooLarge)?,
    );
    Read::by_ref(&mut file)
        .take(MAX_ARCHIVE_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()))?;
    if bytes.len() as u64 != metadata.len() || bytes.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(NativePortableArchiveError::ArchiveDrift);
    }
    Ok(bytes)
}

#[cfg(unix)]
fn verify_archive_file_security(
    _path: &Path,
    metadata: &Metadata,
) -> Result<(), NativePortableArchiveError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    if metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.permissions().mode() & 0o777 != 0o644
        || metadata.nlink() != 1
    {
        return Err(NativePortableArchiveError::ArchiveDrift);
    }
    Ok(())
}

#[cfg(windows)]
fn verify_archive_file_security(
    path: &Path,
    _metadata: &Metadata,
) -> Result<(), NativePortableArchiveError> {
    let file = qiongli_windows_security::open_owner_only_file(path)
        .map_err(|_| NativePortableArchiveError::ArchiveDrift)?;
    let facts = qiongli_windows_security::handle_facts(&file)
        .map_err(|_| NativePortableArchiveError::ArchiveDrift)?;
    if facts.number_of_links != 1 {
        return Err(NativePortableArchiveError::ArchiveDrift);
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn verify_archive_file_security(
    _path: &Path,
    _metadata: &Metadata,
) -> Result<(), NativePortableArchiveError> {
    Err(NativePortableArchiveError::UnsupportedPlatform)
}

fn write_archive_file(
    path: &Path,
    mut file: File,
    bytes: &[u8],
) -> Result<(), NativePortableArchiveError> {
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()))?;
    drop(file);
    set_regular_mode(path)?;
    sync_file_mode(path)
}

#[cfg(unix)]
fn create_private_new_file(path: &Path) -> Result<File, NativePortableArchiveError> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn create_private_new_file(path: &Path) -> Result<File, NativePortableArchiveError> {
    qiongli_windows_security::create_owner_only_new_file(path).map_err(|error| {
        NativePortableArchiveError::PersistenceFailed(
            error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
        )
    })
}

#[cfg(not(any(unix, windows)))]
fn create_private_new_file(_path: &Path) -> Result<File, NativePortableArchiveError> {
    Err(NativePortableArchiveError::UnsupportedPlatform)
}

#[cfg(unix)]
fn set_regular_mode(path: &Path) -> Result<(), NativePortableArchiveError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o644))
        .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn set_regular_mode(_path: &Path) -> Result<(), NativePortableArchiveError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn set_regular_mode(_path: &Path) -> Result<(), NativePortableArchiveError> {
    Err(NativePortableArchiveError::UnsupportedPlatform)
}

#[cfg(unix)]
fn sync_file_mode(path: &Path) -> Result<(), NativePortableArchiveError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()))
}

#[cfg(not(unix))]
fn sync_file_mode(_path: &Path) -> Result<(), NativePortableArchiveError> {
    Ok(())
}

fn create_staging_file(parent: &Path) -> Result<(PathBuf, File), NativePortableArchiveError> {
    for _ in 0..128 {
        let path = parent.join(format!(
            ".qiongli.native-portable-archive-stage-{}-{}.zip",
            std::process::id(),
            transaction_id()
        ));
        match create_private_new_file(&path) {
            Ok(file) => return Ok((path, file)),
            Err(NativePortableArchiveError::PersistenceFailed(io::ErrorKind::AlreadyExists)) => {}
            Err(error) => return Err(error),
        }
    }
    Err(NativePortableArchiveError::PersistenceFailed(
        io::ErrorKind::AlreadyExists,
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn rename_no_replace(source: &Path, destination: &Path) -> Result<(), NativePortableArchiveError> {
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        source,
        rustix::fs::CWD,
        destination,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|error| {
        let error = io::Error::from(error);
        NativePortableArchiveError::CommitFailed(error.kind())
    })
}

#[cfg(windows)]
fn rename_no_replace(source: &Path, destination: &Path) -> Result<(), NativePortableArchiveError> {
    qiongli_windows_security::move_file_write_through(source, destination, false).map_err(|error| {
        NativePortableArchiveError::CommitFailed(
            error.io_kind().unwrap_or(io::ErrorKind::PermissionDenied),
        )
    })
}

#[cfg(all(not(windows), not(any(target_os = "linux", target_os = "macos"))))]
fn rename_no_replace(
    _source: &Path,
    _destination: &Path,
) -> Result<(), NativePortableArchiveError> {
    Err(NativePortableArchiveError::UnsupportedPlatform)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), NativePortableArchiveError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()))
}

#[cfg(windows)]
fn sync_directory(_path: &Path) -> Result<(), NativePortableArchiveError> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn sync_directory(_path: &Path) -> Result<(), NativePortableArchiveError> {
    Err(NativePortableArchiveError::UnsupportedPlatform)
}

fn path_metadata(path: &Path) -> Result<Option<Metadata>, NativePortableArchiveError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(NativePortableArchiveError::PersistenceFailed(error.kind())),
    }
}

#[cfg(windows)]
fn is_reparse_point(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &Metadata) -> bool {
    false
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

const fn map_identity_error(error: NativeArtifactError) -> NativePortableArchiveError {
    match error {
        NativeArtifactError::UnsupportedPlatform => NativePortableArchiveError::UnsupportedPlatform,
        _ => NativePortableArchiveError::InvalidIdentity,
    }
}

const fn map_source_error(error: NativeArtifactError) -> NativePortableArchiveError {
    match error {
        NativeArtifactError::UnsupportedPlatform => NativePortableArchiveError::UnsupportedPlatform,
        NativeArtifactError::InvalidIdentity => NativePortableArchiveError::InvalidIdentity,
        _ => NativePortableArchiveError::SourceArtifactInvalid,
    }
}

const fn map_archive_payload_error(error: NativeArtifactError) -> NativePortableArchiveError {
    match error {
        NativeArtifactError::UnsupportedPlatform => NativePortableArchiveError::UnsupportedPlatform,
        NativeArtifactError::InvalidIdentity => NativePortableArchiveError::InvalidIdentity,
        NativeArtifactError::ManifestInvalid => NativePortableArchiveError::ArchiveInvalid,
        _ => NativePortableArchiveError::ArchiveDrift,
    }
}

const fn map_extraction_error(error: NativeArtifactError) -> NativePortableArchiveError {
    match error {
        NativeArtifactError::UnsupportedPlatform => NativePortableArchiveError::UnsupportedPlatform,
        NativeArtifactError::TargetExists => NativePortableArchiveError::DestinationExists,
        NativeArtifactError::TargetBusy => NativePortableArchiveError::DestinationBusy,
        NativeArtifactError::InvalidTarget | NativeArtifactError::UnsafeTarget => {
            NativePortableArchiveError::DestinationUnsafe
        }
        _ => NativePortableArchiveError::ExtractionFailed,
    }
}

fn transaction_id() -> u64 {
    NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed)
}

struct TargetLock {
    path: PathBuf,
    identity: Option<Handle>,
}

impl TargetLock {
    fn acquire(target: &NativePortableArchiveTarget) -> Result<Self, NativePortableArchiveError> {
        let parent = target
            .path()
            .parent()
            .ok_or(NativePortableArchiveError::InvalidTarget)?;
        let path = parent.join(TARGET_LOCK_FILE);
        let mut file = match create_private_new_file(&path) {
            Ok(file) => file,
            Err(NativePortableArchiveError::PersistenceFailed(io::ErrorKind::AlreadyExists)) => {
                return Err(NativePortableArchiveError::TargetBusy);
            }
            Err(error) => return Err(error),
        };
        let setup = writeln!(file, "{}", std::process::id())
            .and_then(|()| file.sync_all())
            .map_err(|error| NativePortableArchiveError::PersistenceFailed(error.kind()));
        drop(file);
        if let Err(error) = setup {
            let _ = fs::remove_file(&path);
            return Err(error);
        }
        let identity = Handle::from_path(&path).map_err(|error| {
            let _ = fs::remove_file(&path);
            NativePortableArchiveError::PersistenceFailed(error.kind())
        })?;
        Ok(Self {
            path,
            identity: Some(identity),
        })
    }
}

impl Drop for TargetLock {
    fn drop(&mut self) {
        let still_owned = self.identity.as_ref().is_some_and(|expected| {
            Handle::from_path(&self.path).is_ok_and(|current| &current == expected)
        });
        self.identity.take();
        if still_owned {
            let _ = fs::remove_file(&self.path);
        }
    }
}

struct FileCleanup {
    path: PathBuf,
    armed: bool,
}

impl FileCleanup {
    const fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for FileCleanup {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReleaseChannel, current_target_native_artifact_identity};

    #[test]
    fn file_name_is_concrete_and_deterministic() {
        let artifact =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .unwrap();
        let artifact_id = native_artifact_id(&artifact).unwrap();
        assert_eq!(
            native_portable_archive_file_name(&artifact).unwrap(),
            format!("{artifact_id}.zip")
        );
    }

    #[test]
    fn zip_writer_is_byte_deterministic() {
        let artifact =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .unwrap();
        let artifact_id = native_artifact_id(&artifact).unwrap();
        let binary_path = native_artifact_binary_path(&artifact).unwrap();
        let first = build_zip_bytes(&artifact_id, binary_path, b"{}", b"binary").unwrap();
        let second = build_zip_bytes(&artifact_id, binary_path, b"{}", b"binary").unwrap();
        assert_eq!(first, second);
        assert_eq!(&first[..4], &ZIP_LOCAL_HEADER.to_le_bytes());
        assert_eq!(
            &first[first.len() - 22..first.len() - 18],
            &ZIP_END_OF_CENTRAL_DIRECTORY.to_le_bytes()
        );
        assert!(parse_zip_entries(&artifact, &first).is_ok());
    }

    #[test]
    fn zip_parser_rejects_noncanonical_structure() {
        let artifact =
            current_target_native_artifact_identity("2.0.0-alpha.1", ReleaseChannel::Alpha)
                .unwrap();
        let artifact_id = native_artifact_id(&artifact).unwrap();
        let binary_path = native_artifact_binary_path(&artifact).unwrap();
        let canonical = build_zip_bytes(&artifact_id, binary_path, b"{}", b"binary").unwrap();

        let mut trailing = canonical.clone();
        trailing.push(0);
        assert!(matches!(
            parse_zip_entries(&artifact, &trailing),
            Err(NativePortableArchiveError::ArchiveInvalid)
        ));

        let mut bad_method = canonical.clone();
        bad_method[8] = 8;
        assert!(matches!(
            parse_zip_entries(&artifact, &bad_method),
            Err(NativePortableArchiveError::ArchiveInvalid)
        ));

        let mut bad_crc = canonical.clone();
        let binary_offset = bad_crc
            .windows(b"binary".len())
            .position(|window| window == b"binary")
            .unwrap();
        bad_crc[binary_offset] ^= 0x01;
        assert!(matches!(
            parse_zip_entries(&artifact, &bad_crc),
            Err(NativePortableArchiveError::ArchiveInvalid)
        ));

        let mut bad_attributes = canonical.clone();
        let eocd_start = bad_attributes.len() - 22;
        let central_offset = u32::from_le_bytes(
            bad_attributes[eocd_start + 16..eocd_start + 20]
                .try_into()
                .unwrap(),
        ) as usize;
        bad_attributes[central_offset + 38] ^= 0x01;
        assert!(matches!(
            parse_zip_entries(&artifact, &bad_attributes),
            Err(NativePortableArchiveError::ArchiveInvalid)
        ));

        let mut truncated = canonical;
        truncated.pop();
        assert!(matches!(
            parse_zip_entries(&artifact, &truncated),
            Err(NativePortableArchiveError::ArchiveInvalid)
        ));
    }

    #[test]
    fn errors_do_not_expose_paths() {
        let rendered =
            NativePortableArchiveError::PersistenceFailed(io::ErrorKind::PermissionDenied)
                .to_string();
        assert_eq!(
            rendered,
            "native-portable-archive-persistence-failed (PermissionDenied)"
        );
        assert!(!rendered.contains('/'));
    }
}
