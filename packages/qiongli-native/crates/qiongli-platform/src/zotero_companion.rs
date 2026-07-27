use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};

use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const ZOTERO_COMPANION_ARTIFACT_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE: &str = "qiongli-zotero-companion.manifest.json";
pub const ZOTERO_COMPANION_PACKAGED_XPI_FILE: &str = "qiongli-zotero-companion.xpi";
pub const ZOTERO_COMPANION_ID: &str = "qiongli-zotero-companion@qiongli.local";
pub const ZOTERO_COMPANION_DISPLAY_NAME: &str = "Qiongli Zotero Companion";
pub const ZOTERO_COMPANION_ENDPOINT_VERSION: &str = "2";
pub const ZOTERO_COMPANION_ZOTERO_MIN_VERSION: &str = "8.0";
pub const ZOTERO_COMPANION_ZOTERO_MAX_VERSION: &str = "9.0.*";
pub const ZOTERO_COMPANION_UPDATE_URL: &str = "https://github.com/jxpeng98/qiongli/releases/latest/download/qiongli-zotero-companion-updates.json";
pub const ZOTERO_COMPANION_SOURCE_PATHS: [&str; 4] = [
    "README.md",
    "bootstrap.js",
    "chrome/content/qiongli-bridge.js",
    "manifest.json",
];

const MAX_XPI_BYTES: usize = 2 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const MAX_SOURCE_ENTRY_BYTES: usize = 1024 * 1024;
const MAX_SOURCE_ENTRY_COUNT: usize = ZOTERO_COMPANION_SOURCE_PATHS.len();
const CONTENT_ROOT_DOMAIN: &[u8] = b"qiongli-zotero-companion-content-root-v1\0";
const ZIP_VERSION: u16 = 20;
const ZIP_VERSION_MADE_BY_UNIX: u16 = (3 << 8) | ZIP_VERSION;
const ZIP_FLAGS: u16 = 0;
const ZIP_STORED_METHOD: u16 = 0;
const ZIP_DOS_TIME: u16 = 0;
const ZIP_DOS_DATE: u16 = 0x0021;
const ZIP_LOCAL_HEADER: u32 = 0x0403_4b50;
const ZIP_CENTRAL_HEADER: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const ZIP_REGULAR_ATTRIBUTES: u32 = 0o100644_u32 << 16;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ZoteroCompanionArtifactRecordType {
    QiongliZoteroCompanionArtifact,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ZoteroCompanionArtifactStatus {
    AssembledUnpublished,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZoteroCompanionArtifactEntryV1 {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZoteroCompanionArtifactManifestV1 {
    pub schema_version: u32,
    pub record_type: ZoteroCompanionArtifactRecordType,
    pub status: ZoteroCompanionArtifactStatus,
    pub companion_id: String,
    pub display_name: String,
    pub companion_version: String,
    pub zotero_min_version: String,
    pub zotero_max_version: String,
    pub endpoint_version: String,
    pub artifact_file: String,
    pub artifact_size_bytes: u64,
    pub artifact_sha256: String,
    pub entry_content_root_sha256: String,
    pub entries: Vec<ZoteroCompanionArtifactEntryV1>,
}

#[derive(Clone, Copy)]
pub struct ZoteroCompanionSourceEntry<'a> {
    pub path: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedZoteroCompanionArtifact {
    manifest: ZoteroCompanionArtifactManifestV1,
    manifest_bytes: Vec<u8>,
    xpi_bytes: Vec<u8>,
}

impl VerifiedZoteroCompanionArtifact {
    #[must_use]
    pub const fn manifest(&self) -> &ZoteroCompanionArtifactManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest_bytes
    }

    #[must_use]
    pub fn xpi_bytes(&self) -> &[u8] {
        &self.xpi_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoteroCompanionArtifactError {
    SourceInvalid,
    SourceIdentityMismatch,
    ManifestInvalid,
    ArchiveInvalid,
    ArchiveTooLarge,
    ArtifactDrift,
}

impl ZoteroCompanionArtifactError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::SourceInvalid => "zotero-companion-source-invalid",
            Self::SourceIdentityMismatch => "zotero-companion-source-identity-mismatch",
            Self::ManifestInvalid => "zotero-companion-artifact-manifest-invalid",
            Self::ArchiveInvalid => "zotero-companion-xpi-invalid",
            Self::ArchiveTooLarge => "zotero-companion-xpi-too-large",
            Self::ArtifactDrift => "zotero-companion-artifact-drift",
        }
    }
}

impl Display for ZoteroCompanionArtifactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for ZoteroCompanionArtifactError {}

pub fn compose_zotero_companion_artifact(
    entries: &[ZoteroCompanionSourceEntry<'_>],
) -> Result<VerifiedZoteroCompanionArtifact, ZoteroCompanionArtifactError> {
    let sources = validate_source_entries(entries)?;
    let identity = source_identity(&sources)?;
    let xpi_bytes = build_zip(&sources)?;
    let manifest_entries = sources
        .iter()
        .map(|entry| ZoteroCompanionArtifactEntryV1 {
            path: entry.path.to_owned(),
            size_bytes: entry.bytes.len() as u64,
            sha256: sha256_hex(entry.bytes),
        })
        .collect::<Vec<_>>();
    let manifest = ZoteroCompanionArtifactManifestV1 {
        schema_version: ZOTERO_COMPANION_ARTIFACT_MANIFEST_SCHEMA_VERSION,
        record_type: ZoteroCompanionArtifactRecordType::QiongliZoteroCompanionArtifact,
        status: ZoteroCompanionArtifactStatus::AssembledUnpublished,
        companion_id: ZOTERO_COMPANION_ID.to_owned(),
        display_name: ZOTERO_COMPANION_DISPLAY_NAME.to_owned(),
        companion_version: identity.version.clone(),
        zotero_min_version: ZOTERO_COMPANION_ZOTERO_MIN_VERSION.to_owned(),
        zotero_max_version: ZOTERO_COMPANION_ZOTERO_MAX_VERSION.to_owned(),
        endpoint_version: ZOTERO_COMPANION_ENDPOINT_VERSION.to_owned(),
        artifact_file: artifact_file_name(&identity.version),
        artifact_size_bytes: xpi_bytes.len() as u64,
        artifact_sha256: sha256_hex(&xpi_bytes),
        entry_content_root_sha256: entry_content_root(&manifest_entries),
        entries: manifest_entries,
    };
    validate_manifest(&manifest)?;
    let manifest_bytes = canonical_json(&manifest)?;
    verify_zotero_companion_artifact(&manifest_bytes, &xpi_bytes)
}

pub fn verify_zotero_companion_artifact(
    manifest_bytes: &[u8],
    xpi_bytes: &[u8],
) -> Result<VerifiedZoteroCompanionArtifact, ZoteroCompanionArtifactError> {
    if manifest_bytes.is_empty() || manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(ZoteroCompanionArtifactError::ManifestInvalid);
    }
    let manifest = serde_json::from_slice::<ZoteroCompanionArtifactManifestV1>(manifest_bytes)
        .map_err(|_| ZoteroCompanionArtifactError::ManifestInvalid)?;
    if canonical_json(&manifest)? != manifest_bytes {
        return Err(ZoteroCompanionArtifactError::ManifestInvalid);
    }
    validate_manifest(&manifest)?;
    if xpi_bytes.is_empty()
        || xpi_bytes.len() > MAX_XPI_BYTES
        || xpi_bytes.len() as u64 != manifest.artifact_size_bytes
        || sha256_hex(xpi_bytes) != manifest.artifact_sha256
    {
        return Err(ZoteroCompanionArtifactError::ArtifactDrift);
    }
    let parsed = parse_zip(xpi_bytes)?;
    if parsed.len() != manifest.entries.len() {
        return Err(ZoteroCompanionArtifactError::ArtifactDrift);
    }
    for (actual, expected) in parsed.iter().zip(&manifest.entries) {
        if actual.path != expected.path
            || actual.bytes.len() as u64 != expected.size_bytes
            || sha256_hex(actual.bytes) != expected.sha256
        {
            return Err(ZoteroCompanionArtifactError::ArtifactDrift);
        }
    }
    let sources = parsed
        .iter()
        .map(|entry| ZoteroCompanionSourceEntry {
            path: &entry.path,
            bytes: entry.bytes,
        })
        .collect::<Vec<_>>();
    let sources = validate_source_entries(&sources)?;
    let identity = source_identity(&sources)?;
    if identity.version != manifest.companion_version {
        return Err(ZoteroCompanionArtifactError::SourceIdentityMismatch);
    }
    Ok(VerifiedZoteroCompanionArtifact {
        manifest,
        manifest_bytes: manifest_bytes.to_vec(),
        xpi_bytes: xpi_bytes.to_vec(),
    })
}

fn validate_manifest(
    manifest: &ZoteroCompanionArtifactManifestV1,
) -> Result<(), ZoteroCompanionArtifactError> {
    if manifest.schema_version != ZOTERO_COMPANION_ARTIFACT_MANIFEST_SCHEMA_VERSION
        || manifest.record_type != ZoteroCompanionArtifactRecordType::QiongliZoteroCompanionArtifact
        || manifest.status != ZoteroCompanionArtifactStatus::AssembledUnpublished
        || manifest.companion_id != ZOTERO_COMPANION_ID
        || manifest.display_name != ZOTERO_COMPANION_DISPLAY_NAME
        || Version::parse(&manifest.companion_version).is_err()
        || manifest.zotero_min_version != ZOTERO_COMPANION_ZOTERO_MIN_VERSION
        || manifest.zotero_max_version != ZOTERO_COMPANION_ZOTERO_MAX_VERSION
        || manifest.endpoint_version != ZOTERO_COMPANION_ENDPOINT_VERSION
        || manifest.artifact_file != artifact_file_name(&manifest.companion_version)
        || manifest.artifact_size_bytes == 0
        || manifest.artifact_size_bytes > MAX_XPI_BYTES as u64
        || !is_lower_hex(&manifest.artifact_sha256, 64)
        || manifest.entries.len() != MAX_SOURCE_ENTRY_COUNT
        || manifest
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<BTreeSet<_>>()
            != ZOTERO_COMPANION_SOURCE_PATHS.into_iter().collect()
    {
        return Err(ZoteroCompanionArtifactError::ManifestInvalid);
    }
    let mut prior = None;
    for entry in &manifest.entries {
        if !valid_archive_path(&entry.path)
            || entry.size_bytes == 0
            || entry.size_bytes > MAX_SOURCE_ENTRY_BYTES as u64
            || !is_lower_hex(&entry.sha256, 64)
            || prior.is_some_and(|value: &str| value >= entry.path.as_str())
        {
            return Err(ZoteroCompanionArtifactError::ManifestInvalid);
        }
        prior = Some(entry.path.as_str());
    }
    if entry_content_root(&manifest.entries) != manifest.entry_content_root_sha256 {
        return Err(ZoteroCompanionArtifactError::ManifestInvalid);
    }
    Ok(())
}

struct SourceIdentity {
    version: String,
}

fn source_identity(
    entries: &[ZoteroCompanionSourceEntry<'_>],
) -> Result<SourceIdentity, ZoteroCompanionArtifactError> {
    let manifest_bytes = entries
        .iter()
        .find(|entry| entry.path == "manifest.json")
        .map(|entry| entry.bytes)
        .ok_or(ZoteroCompanionArtifactError::SourceInvalid)?;
    let manifest = serde_json::from_slice::<Value>(manifest_bytes)
        .map_err(|_| ZoteroCompanionArtifactError::SourceInvalid)?;
    let zotero = manifest
        .get("applications")
        .and_then(Value::as_object)
        .and_then(|applications| applications.get("zotero"))
        .and_then(Value::as_object)
        .ok_or(ZoteroCompanionArtifactError::SourceIdentityMismatch)?;
    let version = manifest
        .get("version")
        .and_then(Value::as_str)
        .filter(|value| Version::parse(value).is_ok())
        .ok_or(ZoteroCompanionArtifactError::SourceIdentityMismatch)?;
    if manifest.get("manifest_version").and_then(Value::as_u64) != Some(2)
        || manifest.get("name").and_then(Value::as_str) != Some(ZOTERO_COMPANION_DISPLAY_NAME)
        || zotero.get("id").and_then(Value::as_str) != Some(ZOTERO_COMPANION_ID)
        || zotero.get("update_url").and_then(Value::as_str) != Some(ZOTERO_COMPANION_UPDATE_URL)
        || zotero.get("strict_min_version").and_then(Value::as_str)
            != Some(ZOTERO_COMPANION_ZOTERO_MIN_VERSION)
        || zotero.get("strict_max_version").and_then(Value::as_str)
            != Some(ZOTERO_COMPANION_ZOTERO_MAX_VERSION)
    {
        return Err(ZoteroCompanionArtifactError::SourceIdentityMismatch);
    }
    let version_declaration = format!("version: \"{version}\"");
    let endpoint_declaration = format!("endpoint_version: \"{ZOTERO_COMPANION_ENDPOINT_VERSION}\"");
    for path in ["bootstrap.js", "chrome/content/qiongli-bridge.js"] {
        let source = entries
            .iter()
            .find(|entry| entry.path == path)
            .and_then(|entry| std::str::from_utf8(entry.bytes).ok())
            .ok_or(ZoteroCompanionArtifactError::SourceInvalid)?;
        if !source.contains(&version_declaration) || !source.contains(&endpoint_declaration) {
            return Err(ZoteroCompanionArtifactError::SourceIdentityMismatch);
        }
    }
    Ok(SourceIdentity {
        version: version.to_owned(),
    })
}

fn validate_source_entries<'a>(
    entries: &'a [ZoteroCompanionSourceEntry<'a>],
) -> Result<Vec<ZoteroCompanionSourceEntry<'a>>, ZoteroCompanionArtifactError> {
    if entries.len() != MAX_SOURCE_ENTRY_COUNT {
        return Err(ZoteroCompanionArtifactError::SourceInvalid);
    }
    let mut sources = entries.to_vec();
    sources.sort_by(|left, right| left.path.cmp(right.path));
    if sources
        .iter()
        .map(|entry| entry.path)
        .collect::<BTreeSet<_>>()
        != ZOTERO_COMPANION_SOURCE_PATHS.into_iter().collect()
        || sources.iter().any(|entry| {
            !valid_archive_path(entry.path)
                || entry.bytes.is_empty()
                || entry.bytes.len() > MAX_SOURCE_ENTRY_BYTES
        })
    {
        return Err(ZoteroCompanionArtifactError::SourceInvalid);
    }
    Ok(sources)
}

fn artifact_file_name(version: &str) -> String {
    format!("qiongli-zotero-companion-{version}.xpi")
}

fn entry_content_root(entries: &[ZoteroCompanionArtifactEntryV1]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CONTENT_ROOT_DOMAIN);
    for entry in entries {
        hasher.update((entry.path.len() as u64).to_be_bytes());
        hasher.update(entry.path.as_bytes());
        hasher.update(entry.size_bytes.to_be_bytes());
        hasher.update(entry.sha256.as_bytes());
    }
    encode_hex(&hasher.finalize())
}

#[derive(Clone, Copy)]
struct ZipSourceEntry<'a> {
    path: &'a str,
    bytes: &'a [u8],
}

struct ZipCentralRecord {
    path: String,
    crc32: u32,
    size: u32,
    local_offset: u32,
}

fn build_zip(
    entries: &[ZoteroCompanionSourceEntry<'_>],
) -> Result<Vec<u8>, ZoteroCompanionArtifactError> {
    let sources = entries
        .iter()
        .map(|entry| ZipSourceEntry {
            path: entry.path,
            bytes: entry.bytes,
        })
        .collect::<Vec<_>>();
    let capacity = sources.iter().try_fold(2_048_usize, |total, entry| {
        total
            .checked_add(entry.bytes.len())
            .and_then(|value| value.checked_add(entry.path.len() * 2 + 80))
            .ok_or(ZoteroCompanionArtifactError::ArchiveTooLarge)
    })?;
    if capacity > MAX_XPI_BYTES {
        return Err(ZoteroCompanionArtifactError::ArchiveTooLarge);
    }
    let mut bytes = Vec::with_capacity(capacity);
    let mut central = Vec::with_capacity(sources.len());
    for entry in &sources {
        let name_length = u16::try_from(entry.path.len())
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
        let size = u32::try_from(entry.bytes.len())
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveTooLarge)?;
        let local_offset = u32::try_from(bytes.len())
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveTooLarge)?;
        let crc32 = crc32fast::hash(entry.bytes);
        push_u32(&mut bytes, ZIP_LOCAL_HEADER);
        push_u16(&mut bytes, ZIP_VERSION);
        push_u16(&mut bytes, ZIP_FLAGS);
        push_u16(&mut bytes, ZIP_STORED_METHOD);
        push_u16(&mut bytes, ZIP_DOS_TIME);
        push_u16(&mut bytes, ZIP_DOS_DATE);
        push_u32(&mut bytes, crc32);
        push_u32(&mut bytes, size);
        push_u32(&mut bytes, size);
        push_u16(&mut bytes, name_length);
        push_u16(&mut bytes, 0);
        bytes.extend_from_slice(entry.path.as_bytes());
        bytes.extend_from_slice(entry.bytes);
        central.push(ZipCentralRecord {
            path: entry.path.to_owned(),
            crc32,
            size,
            local_offset,
        });
    }
    let central_offset =
        u32::try_from(bytes.len()).map_err(|_| ZoteroCompanionArtifactError::ArchiveTooLarge)?;
    for record in &central {
        let name_length = u16::try_from(record.path.len())
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
        push_u32(&mut bytes, ZIP_CENTRAL_HEADER);
        push_u16(&mut bytes, ZIP_VERSION_MADE_BY_UNIX);
        push_u16(&mut bytes, ZIP_VERSION);
        push_u16(&mut bytes, ZIP_FLAGS);
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
        push_u32(&mut bytes, ZIP_REGULAR_ATTRIBUTES);
        push_u32(&mut bytes, record.local_offset);
        bytes.extend_from_slice(record.path.as_bytes());
    }
    let central_size = u32::try_from(bytes.len())
        .ok()
        .and_then(|end| end.checked_sub(central_offset))
        .ok_or(ZoteroCompanionArtifactError::ArchiveTooLarge)?;
    let entry_count =
        u16::try_from(central.len()).map_err(|_| ZoteroCompanionArtifactError::ArchiveTooLarge)?;
    push_u32(&mut bytes, ZIP_END_OF_CENTRAL_DIRECTORY);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, entry_count);
    push_u16(&mut bytes, entry_count);
    push_u32(&mut bytes, central_size);
    push_u32(&mut bytes, central_offset);
    push_u16(&mut bytes, 0);
    if bytes.len() > MAX_XPI_BYTES {
        return Err(ZoteroCompanionArtifactError::ArchiveTooLarge);
    }
    Ok(bytes)
}

struct ParsedZipEntry<'a> {
    path: String,
    bytes: &'a [u8],
    crc32: u32,
    size: u32,
    local_offset: u32,
}

fn parse_zip(bytes: &[u8]) -> Result<Vec<ParsedZipEntry<'_>>, ZoteroCompanionArtifactError> {
    if bytes.len() < 22 || bytes.len() > MAX_XPI_BYTES {
        return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
    }
    let eocd_start = bytes
        .len()
        .checked_sub(22)
        .ok_or(ZoteroCompanionArtifactError::ArchiveInvalid)?;
    let mut eocd = SliceReader::new(&bytes[eocd_start..]);
    if eocd.read_u32()? != ZIP_END_OF_CENTRAL_DIRECTORY
        || eocd.read_u16()? != 0
        || eocd.read_u16()? != 0
    {
        return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
    }
    let disk_entries = usize::from(eocd.read_u16()?);
    let total_entries = usize::from(eocd.read_u16()?);
    let central_size = usize::try_from(eocd.read_u32()?)
        .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
    let central_offset = usize::try_from(eocd.read_u32()?)
        .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
    if eocd.read_u16()? != 0
        || !eocd.is_finished()
        || disk_entries != total_entries
        || total_entries != MAX_SOURCE_ENTRY_COUNT
        || central_offset.checked_add(central_size) != Some(eocd_start)
    {
        return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
    }
    let mut local = SliceReader::new(&bytes[..central_offset]);
    let mut entries = Vec::with_capacity(total_entries);
    let mut prior = None;
    for _ in 0..total_entries {
        let local_offset = u32::try_from(local.position())
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
        if local.read_u32()? != ZIP_LOCAL_HEADER
            || local.read_u16()? != ZIP_VERSION
            || local.read_u16()? != ZIP_FLAGS
            || local.read_u16()? != ZIP_STORED_METHOD
            || local.read_u16()? != ZIP_DOS_TIME
            || local.read_u16()? != ZIP_DOS_DATE
        {
            return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
        }
        let crc32 = local.read_u32()?;
        let compressed_size = local.read_u32()?;
        let size = local.read_u32()?;
        let name_length = usize::from(local.read_u16()?);
        if local.read_u16()? != 0 || compressed_size != size {
            return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
        }
        let path = std::str::from_utf8(local.take(name_length)?)
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?
            .to_owned();
        if !valid_archive_path(&path) || prior.as_ref().is_some_and(|prior: &String| prior >= &path)
        {
            return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
        }
        prior = Some(path.clone());
        let data_length =
            usize::try_from(size).map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
        let data = local.take(data_length)?;
        if data.is_empty() || crc32fast::hash(data) != crc32 {
            return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
        }
        entries.push(ParsedZipEntry {
            path,
            bytes: data,
            crc32,
            size,
            local_offset,
        });
    }
    if !local.is_finished() {
        return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
    }
    let central_end = central_offset
        .checked_add(central_size)
        .ok_or(ZoteroCompanionArtifactError::ArchiveInvalid)?;
    let mut central = SliceReader::new(&bytes[central_offset..central_end]);
    for entry in &entries {
        if central.read_u32()? != ZIP_CENTRAL_HEADER
            || central.read_u16()? != ZIP_VERSION_MADE_BY_UNIX
            || central.read_u16()? != ZIP_VERSION
            || central.read_u16()? != ZIP_FLAGS
            || central.read_u16()? != ZIP_STORED_METHOD
            || central.read_u16()? != ZIP_DOS_TIME
            || central.read_u16()? != ZIP_DOS_DATE
            || central.read_u32()? != entry.crc32
            || central.read_u32()? != entry.size
            || central.read_u32()? != entry.size
        {
            return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
        }
        let name_length = usize::from(central.read_u16()?);
        if central.read_u16()? != 0
            || central.read_u16()? != 0
            || central.read_u16()? != 0
            || central.read_u16()? != 0
            || central.read_u32()? != ZIP_REGULAR_ATTRIBUTES
            || central.read_u32()? != entry.local_offset
            || central.take(name_length)? != entry.path.as_bytes()
        {
            return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
        }
    }
    if !central.is_finished() {
        return Err(ZoteroCompanionArtifactError::ArchiveInvalid);
    }
    Ok(entries)
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

    fn take(&mut self, length: usize) -> Result<&'a [u8], ZoteroCompanionArtifactError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(ZoteroCompanionArtifactError::ArchiveInvalid)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(ZoteroCompanionArtifactError::ArchiveInvalid)?;
        self.position = end;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, ZoteroCompanionArtifactError> {
        let value: [u8; 2] = self
            .take(2)?
            .try_into()
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
        Ok(u16::from_le_bytes(value))
    }

    fn read_u32(&mut self) -> Result<u32, ZoteroCompanionArtifactError> {
        let value: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| ZoteroCompanionArtifactError::ArchiveInvalid)?;
        Ok(u32::from_le_bytes(value))
    }

    const fn is_finished(&self) -> bool {
        self.position == self.bytes.len()
    }
}

fn valid_archive_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 128
        && path.is_ascii()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|component| !component.is_empty() && !matches!(component, "." | ".."))
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ZoteroCompanionArtifactError> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ZoteroCompanionArtifactError::ManifestInvalid)
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    value
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(version: &str, endpoint: &str) -> Vec<(String, Vec<u8>)> {
        vec![
            ("README.md".to_owned(), b"# Companion\n".to_vec()),
            (
                "bootstrap.js".to_owned(),
                format!(
                    "const response = {{ version: \"{version}\", endpoint_version: \"{endpoint}\" }};"
                )
                .into_bytes(),
            ),
            (
                "chrome/content/qiongli-bridge.js".to_owned(),
                format!(
                    "const response = {{ version: \"{version}\", endpoint_version: \"{endpoint}\" }};"
                )
                .into_bytes(),
            ),
            (
                "manifest.json".to_owned(),
                format!(
                    "{{\"manifest_version\":2,\"name\":\"{ZOTERO_COMPANION_DISPLAY_NAME}\",\"version\":\"{version}\",\"applications\":{{\"zotero\":{{\"id\":\"{ZOTERO_COMPANION_ID}\",\"update_url\":\"{ZOTERO_COMPANION_UPDATE_URL}\",\"strict_min_version\":\"{ZOTERO_COMPANION_ZOTERO_MIN_VERSION}\",\"strict_max_version\":\"{ZOTERO_COMPANION_ZOTERO_MAX_VERSION}\"}}}}}}"
                )
                .into_bytes(),
            ),
        ]
    }

    fn compose(
        source: &[(String, Vec<u8>)],
    ) -> Result<VerifiedZoteroCompanionArtifact, ZoteroCompanionArtifactError> {
        let entries = source
            .iter()
            .map(|(path, bytes)| ZoteroCompanionSourceEntry { path, bytes })
            .collect::<Vec<_>>();
        compose_zotero_companion_artifact(&entries)
    }

    #[test]
    fn artifact_is_deterministic_strict_and_tamper_evident() {
        let source = source("0.3.0", "2");
        let first = compose(&source).unwrap();
        let second = compose(&source).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.manifest().companion_version, "0.3.0");
        assert_eq!(first.manifest().endpoint_version, "2");
        assert_eq!(
            first,
            verify_zotero_companion_artifact(first.manifest_bytes(), first.xpi_bytes()).unwrap()
        );

        let mut changed_xpi = first.xpi_bytes().to_vec();
        let midpoint = changed_xpi.len() / 2;
        changed_xpi[midpoint] ^= 1;
        assert_eq!(
            verify_zotero_companion_artifact(first.manifest_bytes(), &changed_xpi),
            Err(ZoteroCompanionArtifactError::ArtifactDrift)
        );
    }

    #[test]
    fn artifact_rejects_endpoint_and_version_drift() {
        assert_eq!(
            compose(&source("0.3.0", "1")),
            Err(ZoteroCompanionArtifactError::SourceIdentityMismatch)
        );
        let mut source = source("0.3.0", "2");
        let bridge = source
            .iter_mut()
            .find(|(path, _)| path == "chrome/content/qiongli-bridge.js")
            .unwrap();
        bridge.1 = b"const response = { version: \"0.2.1\", endpoint_version: \"1\" };".to_vec();
        assert_eq!(
            compose(&source),
            Err(ZoteroCompanionArtifactError::SourceIdentityMismatch)
        );
    }
}
