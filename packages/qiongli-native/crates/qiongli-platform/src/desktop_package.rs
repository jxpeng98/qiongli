use std::collections::BTreeSet;
use std::fmt::{self, Display, Formatter};
use std::io::Cursor;

use qiongli_content::LogicalMode;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    Architecture, ArtifactIdentityV1, CapabilityProfile, InstallerKind, OperatingSystem, ProductId,
    VerifiedNativeArtifact,
};

pub const DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const DESKTOP_PACKAGE_MANIFEST_FILE: &str = ".qiongli-desktop-package.json";

const MAX_ARCHIVE_BYTES: usize = 272 * 1024 * 1024;
const MAX_BINARY_BYTES: usize = 128 * 1024 * 1024;
const MAX_LAUNCHER_BYTES: usize = 16 * 1024 * 1024;
const MAX_UPDATE_HELPER_BYTES: usize = 16 * 1024 * 1024;
const MAX_ICON_BYTES: usize = 2 * 1024 * 1024;
const MAX_LICENSE_BYTES: usize = 256 * 1024;
const MAX_MANIFEST_BYTES: usize = 256 * 1024;
const MAX_ENTRY_COUNT: usize = 8;
const CONTENT_ROOT_DOMAIN: &[u8] = b"qiongli-desktop-package-content-root-v1\0";
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const ZIP_VERSION: u16 = 20;
const ZIP_VERSION_MADE_BY_UNIX: u16 = (3 << 8) | ZIP_VERSION;
const ZIP_UTF8_FLAG: u16 = 0x0800;
const ZIP_STORED_METHOD: u16 = 0;
const ZIP_DOS_TIME: u16 = 0;
const ZIP_DOS_DATE: u16 = 0x0021;
const ZIP_LOCAL_HEADER: u32 = 0x0403_4b50;
const ZIP_CENTRAL_HEADER: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const ZIP_REGULAR_ATTRIBUTES: u32 = 0o100644_u32 << 16;
const ZIP_EXECUTABLE_ATTRIBUTES: u32 = 0o100755_u32 << 16;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopPackageRecordType {
    QiongliDesktopPackage,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopPackageStatus {
    AssembledUnpublished,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopPackageKind {
    MacosApplicationZip,
    WindowsPortableZip,
    LinuxAppDirZip,
}

impl DesktopPackageKind {
    #[must_use]
    pub const fn for_operating_system(os: OperatingSystem) -> Self {
        match os {
            OperatingSystem::Macos => Self::MacosApplicationZip,
            OperatingSystem::Windows => Self::WindowsPortableZip,
            OperatingSystem::Linux => Self::LinuxAppDirZip,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesktopApplicationMetadataV1 {
    pub product_name: String,
    pub window_title: String,
    pub application_identifier: String,
    pub product_version: String,
    pub license: String,
}

impl DesktopApplicationMetadataV1 {
    #[must_use]
    pub fn new(
        product_name: impl Into<String>,
        window_title: impl Into<String>,
        application_identifier: impl Into<String>,
        product_version: impl Into<String>,
        license: impl Into<String>,
    ) -> Self {
        Self {
            product_name: product_name.into(),
            window_title: window_title.into(),
            application_identifier: application_identifier.into(),
            product_version: product_version.into(),
            license: license.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesktopPackageEntryV1 {
    pub path: String,
    pub mode: LogicalMode,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DesktopPackageManifestV1 {
    pub schema_version: u32,
    pub record_type: DesktopPackageRecordType,
    pub status: DesktopPackageStatus,
    pub package_kind: DesktopPackageKind,
    pub artifact: ArtifactIdentityV1,
    pub source_artifact: ArtifactIdentityV1,
    pub product_source_commit: String,
    pub source_artifact_manifest_sha256: String,
    pub resource_pack_sha256: String,
    pub canonical_binary_sha256: String,
    pub launcher_sha256: String,
    pub update_helper_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_control_sha256: Option<String>,
    pub application: DesktopApplicationMetadataV1,
    pub package_root: String,
    pub manifest_path: String,
    pub entry_content_root_sha256: String,
    pub entries: Vec<DesktopPackageEntryV1>,
}

pub struct DesktopPackageInput<'a> {
    source_artifact: &'a VerifiedNativeArtifact,
    binaries: DesktopPackageBinaries<'a>,
    icon_png: &'a [u8],
    license_bytes: &'a [u8],
    product_source_commit: &'a str,
    application: DesktopApplicationMetadataV1,
    product_control: Option<&'a [u8]>,
}

#[derive(Clone, Copy)]
pub struct DesktopPackageBinaries<'a> {
    canonical: &'a [u8],
    launcher: &'a [u8],
    update_helper: &'a [u8],
}

impl<'a> DesktopPackageBinaries<'a> {
    #[must_use]
    pub const fn new(canonical: &'a [u8], launcher: &'a [u8], update_helper: &'a [u8]) -> Self {
        Self {
            canonical,
            launcher,
            update_helper,
        }
    }
}

impl<'a> DesktopPackageInput<'a> {
    #[must_use]
    pub const fn new(
        source_artifact: &'a VerifiedNativeArtifact,
        binaries: DesktopPackageBinaries<'a>,
        icon_png: &'a [u8],
        license_bytes: &'a [u8],
        product_source_commit: &'a str,
        application: DesktopApplicationMetadataV1,
    ) -> Self {
        Self {
            source_artifact,
            binaries,
            icon_png,
            license_bytes,
            product_source_commit,
            application,
            product_control: None,
        }
    }

    #[must_use]
    pub const fn with_product_control(mut self, product_control: &'a [u8]) -> Self {
        self.product_control = Some(product_control);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedDesktopPackage {
    manifest: DesktopPackageManifestV1,
    manifest_bytes: Vec<u8>,
    archive_bytes: Vec<u8>,
    file_name: String,
    archive_sha256: String,
}

impl VerifiedDesktopPackage {
    #[must_use]
    pub const fn manifest(&self) -> &DesktopPackageManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest_bytes
    }

    #[must_use]
    pub fn archive_bytes(&self) -> &[u8] {
        &self.archive_bytes
    }

    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    #[must_use]
    pub fn archive_sha256(&self) -> &str {
        &self.archive_sha256
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopPackageError {
    InvalidSourceArtifact,
    InvalidProductSource,
    InvalidApplicationMetadata,
    CanonicalBinaryInvalid,
    LauncherInvalid,
    IconInvalid,
    LicenseInvalid,
    ManifestInvalid,
    ArchiveInvalid,
    ArchiveTooLarge,
    ArchiveDrift,
}

impl DesktopPackageError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InvalidSourceArtifact => "desktop-package-source-artifact-invalid",
            Self::InvalidProductSource => "desktop-package-product-source-invalid",
            Self::InvalidApplicationMetadata => "desktop-package-application-metadata-invalid",
            Self::CanonicalBinaryInvalid => "desktop-package-canonical-binary-invalid",
            Self::LauncherInvalid => "desktop-package-launcher-invalid",
            Self::IconInvalid => "desktop-package-icon-invalid",
            Self::LicenseInvalid => "desktop-package-license-invalid",
            Self::ManifestInvalid => "desktop-package-manifest-invalid",
            Self::ArchiveInvalid => "desktop-package-archive-invalid",
            Self::ArchiveTooLarge => "desktop-package-archive-too-large",
            Self::ArchiveDrift => "desktop-package-archive-drift",
        }
    }
}

impl Display for DesktopPackageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.reason_code())
    }
}

impl std::error::Error for DesktopPackageError {}

pub fn compose_desktop_package(
    input: DesktopPackageInput<'_>,
) -> Result<VerifiedDesktopPackage, DesktopPackageError> {
    validate_input(&input)?;
    let source_manifest = input.source_artifact.manifest();
    let source_identity = source_manifest.artifact.clone();
    let mut desktop_identity = source_identity.clone();
    desktop_identity.installer_kind = InstallerKind::NativeInstaller;
    let kind = DesktopPackageKind::for_operating_system(desktop_identity.os);
    let package_root = package_root(desktop_identity.os).to_string();
    let manifest_path = manifest_path(desktop_identity.os);
    let payload = build_payload_entries(DesktopPayloadInput {
        artifact: &desktop_identity,
        binaries: input.binaries,
        icon_png: input.icon_png,
        license_bytes: input.license_bytes,
        application: &input.application,
        product_control: input.product_control,
    })?;
    let entries = payload
        .iter()
        .map(PayloadEntry::manifest_entry)
        .collect::<Vec<_>>();
    let manifest = DesktopPackageManifestV1 {
        schema_version: DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION,
        record_type: DesktopPackageRecordType::QiongliDesktopPackage,
        status: DesktopPackageStatus::AssembledUnpublished,
        package_kind: kind,
        artifact: desktop_identity,
        source_artifact: source_identity,
        product_source_commit: input.product_source_commit.to_string(),
        source_artifact_manifest_sha256: input.source_artifact.manifest_sha256().to_string(),
        resource_pack_sha256: source_manifest.content.pack_sha256.clone(),
        canonical_binary_sha256: source_manifest.binary_sha256.clone(),
        launcher_sha256: sha256_hex(input.binaries.launcher),
        update_helper_sha256: sha256_hex(input.binaries.update_helper),
        product_control_sha256: input.product_control.map(sha256_hex),
        application: input.application,
        package_root,
        manifest_path: manifest_path.clone(),
        entry_content_root_sha256: entry_content_root(&entries),
        entries,
    };
    validate_manifest(
        &manifest,
        input.source_artifact,
        input.product_source_commit,
    )?;
    let manifest_bytes = canonical_json(&manifest)?;
    let mut archive_entries = payload;
    archive_entries.push(PayloadEntry {
        path: manifest_path,
        mode: LogicalMode::Regular,
        bytes: manifest_bytes,
    });
    archive_entries.sort_by(|left, right| left.path.cmp(&right.path));
    let archive_bytes = build_zip(&archive_entries)?;
    verify_desktop_package(
        input.source_artifact,
        input.product_source_commit,
        &archive_bytes,
    )
}

pub fn verify_desktop_package(
    source_artifact: &VerifiedNativeArtifact,
    product_source_commit: &str,
    archive_bytes: &[u8],
) -> Result<VerifiedDesktopPackage, DesktopPackageError> {
    let parsed = parse_zip(archive_bytes)?;
    let expected_manifest_path = manifest_path(source_artifact.manifest().artifact.os);
    let manifest_entry = parsed
        .iter()
        .find(|entry| entry.path == expected_manifest_path)
        .ok_or(DesktopPackageError::ManifestInvalid)?;
    if manifest_entry.mode != LogicalMode::Regular
        || manifest_entry.bytes.len() > MAX_MANIFEST_BYTES
    {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    let manifest: DesktopPackageManifestV1 = serde_json::from_slice(manifest_entry.bytes)
        .map_err(|_| DesktopPackageError::ManifestInvalid)?;
    let manifest_bytes = canonical_json(&manifest)?;
    if manifest_bytes != manifest_entry.bytes {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    validate_manifest(&manifest, source_artifact, product_source_commit)?;
    if parsed.len() != manifest.entries.len() + 1 {
        return Err(DesktopPackageError::ArchiveDrift);
    }
    for expected in &manifest.entries {
        let actual = parsed
            .iter()
            .find(|entry| entry.path == expected.path)
            .ok_or(DesktopPackageError::ArchiveDrift)?;
        if actual.mode != expected.mode
            || actual.bytes.len() as u64 != expected.size_bytes
            || sha256_hex(actual.bytes) != expected.sha256
        {
            return Err(DesktopPackageError::ArchiveDrift);
        }
    }
    let expected_paths = expected_payload_paths(
        manifest.artifact.os,
        manifest.product_control_sha256.is_some(),
    );
    if manifest
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>()
        != expected_paths
    {
        return Err(DesktopPackageError::ArchiveDrift);
    }

    let file_name = desktop_package_file_name(&manifest.artifact)?;
    Ok(VerifiedDesktopPackage {
        manifest,
        manifest_bytes,
        archive_bytes: archive_bytes.to_vec(),
        file_name,
        archive_sha256: sha256_hex(archive_bytes),
    })
}

pub(crate) fn verify_macos_update_desktop_manifest(
    update: &crate::VerifiedNativeUpdateManifest,
    manifest_bytes: &[u8],
) -> Result<DesktopPackageManifestV1, crate::NativeUpdateEvidenceError> {
    let update_manifest = update.manifest();
    if manifest_bytes.len() as u64 != update_manifest.desktop_manifest_size_bytes
        || manifest_bytes.len() > MAX_MANIFEST_BYTES
    {
        return Err(crate::NativeUpdateEvidenceError::DesktopManifestInvalid);
    }
    if sha256_hex(manifest_bytes) != update_manifest.desktop_manifest_sha256 {
        return Err(crate::NativeUpdateEvidenceError::DesktopManifestDigestMismatch);
    }
    let manifest = serde_json::from_slice::<DesktopPackageManifestV1>(manifest_bytes)
        .map_err(|_| crate::NativeUpdateEvidenceError::DesktopManifestInvalid)?;
    if canonical_json(&manifest)
        .map_err(|_| crate::NativeUpdateEvidenceError::DesktopManifestInvalid)?
        != manifest_bytes
    {
        return Err(crate::NativeUpdateEvidenceError::DesktopManifestInvalid);
    }
    validate_manifest_document(&manifest)
        .map_err(|_| crate::NativeUpdateEvidenceError::DesktopManifestInvalid)?;
    let mut expected_source = update_manifest.artifact.clone();
    expected_source.installer_kind = InstallerKind::PortableArchive;
    if manifest.artifact != update_manifest.artifact
        || manifest.source_artifact != expected_source
        || manifest.product_source_commit != update_manifest.source_commit
        || manifest.resource_pack_sha256 != update_manifest.resource_pack_sha256
    {
        return Err(crate::NativeUpdateEvidenceError::DesktopManifestInvalid);
    }
    Ok(manifest)
}

pub fn parse_desktop_package_manifest(
    manifest_bytes: &[u8],
) -> Result<DesktopPackageManifestV1, DesktopPackageError> {
    if manifest_bytes.is_empty() || manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    let manifest = serde_json::from_slice::<DesktopPackageManifestV1>(manifest_bytes)
        .map_err(|_| DesktopPackageError::ManifestInvalid)?;
    if canonical_json(&manifest)? != manifest_bytes {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    validate_manifest_document(&manifest)?;
    Ok(manifest)
}

pub fn attach_product_control_to_desktop_manifest(
    manifest_bytes: &[u8],
    product_control_bytes: &[u8],
) -> Result<Vec<u8>, DesktopPackageError> {
    if product_control_bytes.is_empty() || product_control_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    let mut manifest = parse_desktop_package_manifest(manifest_bytes)?;
    let digest = sha256_hex(product_control_bytes);
    if let Some(existing) = manifest.product_control_sha256.as_deref() {
        if existing != digest {
            return Err(DesktopPackageError::ManifestInvalid);
        }
        return Ok(manifest_bytes.to_vec());
    }
    manifest.entries.push(DesktopPackageEntryV1 {
        path: product_control_path(manifest.artifact.os),
        mode: LogicalMode::Regular,
        size_bytes: product_control_bytes.len() as u64,
        sha256: digest.clone(),
    });
    manifest
        .entries
        .sort_by(|left, right| left.path.cmp(&right.path));
    manifest.product_control_sha256 = Some(digest);
    manifest.entry_content_root_sha256 = entry_content_root(&manifest.entries);
    validate_manifest_document(&manifest)?;
    canonical_json(&manifest)
}

pub fn desktop_package_file_name(
    artifact: &ArtifactIdentityV1,
) -> Result<String, DesktopPackageError> {
    validate_desktop_identity(artifact)?;
    let suffix = match artifact.os {
        OperatingSystem::Macos => "app.zip",
        OperatingSystem::Windows => "zip",
        OperatingSystem::Linux => "appdir.zip",
    };
    Ok(format!(
        "qiongli-desktop-{}-{}-{}.{}",
        artifact.version,
        os_label(artifact.os),
        architecture_label(artifact.arch),
        suffix
    ))
}

#[derive(Clone)]
struct PayloadEntry {
    path: String,
    mode: LogicalMode,
    bytes: Vec<u8>,
}

struct DesktopPayloadInput<'a> {
    artifact: &'a ArtifactIdentityV1,
    binaries: DesktopPackageBinaries<'a>,
    icon_png: &'a [u8],
    license_bytes: &'a [u8],
    application: &'a DesktopApplicationMetadataV1,
    product_control: Option<&'a [u8]>,
}

impl PayloadEntry {
    fn manifest_entry(&self) -> DesktopPackageEntryV1 {
        DesktopPackageEntryV1 {
            path: self.path.clone(),
            mode: self.mode,
            size_bytes: self.bytes.len() as u64,
            sha256: sha256_hex(&self.bytes),
        }
    }
}

fn validate_input(input: &DesktopPackageInput<'_>) -> Result<(), DesktopPackageError> {
    let manifest = input.source_artifact.manifest();
    validate_source_identity(&manifest.artifact)?;
    if input.source_artifact.manifest_sha256().len() != 64
        || !is_lower_hex(input.source_artifact.manifest_sha256(), 64)
        || input.binaries.canonical.is_empty()
        || input.binaries.canonical.len() > MAX_BINARY_BYTES
        || !binary_magic_matches(manifest.artifact.os, input.binaries.canonical)
        || input.binaries.canonical.len() as u64 != manifest.entries[0].size_bytes
        || sha256_hex(input.binaries.canonical) != manifest.binary_sha256
    {
        return Err(DesktopPackageError::CanonicalBinaryInvalid);
    }
    if input.binaries.launcher.is_empty()
        || input.binaries.launcher.len() > MAX_LAUNCHER_BYTES
        || !binary_magic_matches(manifest.artifact.os, input.binaries.launcher)
    {
        return Err(DesktopPackageError::LauncherInvalid);
    }
    if input.binaries.update_helper.is_empty()
        || input.binaries.update_helper.len() > MAX_UPDATE_HELPER_BYTES
        || !binary_magic_matches(manifest.artifact.os, input.binaries.update_helper)
    {
        return Err(DesktopPackageError::LauncherInvalid);
    }
    validate_png(input.icon_png)?;
    if input.license_bytes.is_empty()
        || input.license_bytes.len() > MAX_LICENSE_BYTES
        || !input.license_bytes.starts_with(b"MIT License")
        || !input
            .license_bytes
            .windows(b"Permission is hereby granted".len())
            .any(|window| window == b"Permission is hereby granted")
    {
        return Err(DesktopPackageError::LicenseInvalid);
    }
    if !valid_source_commit(input.product_source_commit) {
        return Err(DesktopPackageError::InvalidProductSource);
    }
    validate_application_metadata(&input.application, &manifest.artifact.version)
}

fn validate_manifest(
    manifest: &DesktopPackageManifestV1,
    source_artifact: &VerifiedNativeArtifact,
    product_source_commit: &str,
) -> Result<(), DesktopPackageError> {
    validate_manifest_document(manifest)?;
    let source_manifest = source_artifact.manifest();
    let mut expected_desktop_identity = source_manifest.artifact.clone();
    expected_desktop_identity.installer_kind = InstallerKind::NativeInstaller;
    if manifest.artifact != expected_desktop_identity
        || manifest.source_artifact != source_manifest.artifact
        || manifest.product_source_commit != product_source_commit
        || manifest.source_artifact_manifest_sha256 != source_artifact.manifest_sha256()
        || manifest.resource_pack_sha256 != source_manifest.content.pack_sha256
        || manifest.canonical_binary_sha256 != source_manifest.binary_sha256
    {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    Ok(())
}

fn validate_manifest_document(
    manifest: &DesktopPackageManifestV1,
) -> Result<(), DesktopPackageError> {
    validate_desktop_identity(&manifest.artifact)
        .map_err(|_| DesktopPackageError::ManifestInvalid)?;
    validate_source_identity(&manifest.source_artifact)
        .map_err(|_| DesktopPackageError::ManifestInvalid)?;
    if manifest.schema_version != DESKTOP_PACKAGE_MANIFEST_SCHEMA_VERSION
        || manifest.record_type != DesktopPackageRecordType::QiongliDesktopPackage
        || manifest.status != DesktopPackageStatus::AssembledUnpublished
        || manifest.package_kind != DesktopPackageKind::for_operating_system(manifest.artifact.os)
        || !valid_source_commit(&manifest.product_source_commit)
        || !is_lower_hex(&manifest.source_artifact_manifest_sha256, 64)
        || !is_lower_hex(&manifest.resource_pack_sha256, 64)
        || !is_lower_hex(&manifest.canonical_binary_sha256, 64)
        || !is_lower_hex(&manifest.launcher_sha256, 64)
        || !is_lower_hex(&manifest.update_helper_sha256, 64)
        || manifest
            .product_control_sha256
            .as_deref()
            .is_some_and(|digest| !is_lower_hex(digest, 64))
        || manifest.package_root != package_root(manifest.artifact.os)
        || manifest.manifest_path != manifest_path(manifest.artifact.os)
        || manifest.entries.is_empty()
        || manifest.entries.len() >= MAX_ENTRY_COUNT
    {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    validate_application_metadata(&manifest.application, &manifest.artifact.version)
        .map_err(|_| DesktopPackageError::ManifestInvalid)?;
    let mut prior = None;
    for entry in &manifest.entries {
        if !valid_archive_path(&entry.path)
            || entry.path == manifest.manifest_path
            || entry.size_bytes == 0
            || !is_lower_hex(&entry.sha256, 64)
            || prior.is_some_and(|value: &str| value >= entry.path.as_str())
        {
            return Err(DesktopPackageError::ManifestInvalid);
        }
        prior = Some(entry.path.as_str());
    }
    if entry_content_root(&manifest.entries) != manifest.entry_content_root_sha256 {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    if manifest
        .entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>()
        != expected_payload_paths(
            manifest.artifact.os,
            manifest.product_control_sha256.is_some(),
        )
    {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    let canonical = manifest
        .entries
        .iter()
        .find(|entry| entry.path == canonical_binary_path(manifest.artifact.os))
        .ok_or(DesktopPackageError::ManifestInvalid)?;
    let launcher = manifest
        .entries
        .iter()
        .find(|entry| entry.path == launcher_path(manifest.artifact.os))
        .ok_or(DesktopPackageError::ManifestInvalid)?;
    let update_helper = manifest
        .entries
        .iter()
        .find(|entry| entry.path == update_helper_path(manifest.artifact.os))
        .ok_or(DesktopPackageError::ManifestInvalid)?;
    if canonical.mode != LogicalMode::Executable
        || canonical.sha256 != manifest.canonical_binary_sha256
        || launcher.mode != LogicalMode::Executable
        || launcher.sha256 != manifest.launcher_sha256
        || update_helper.mode != LogicalMode::Executable
        || update_helper.sha256 != manifest.update_helper_sha256
    {
        return Err(DesktopPackageError::ManifestInvalid);
    }
    match manifest.product_control_sha256.as_deref() {
        Some(expected) => {
            let control = manifest
                .entries
                .iter()
                .find(|entry| entry.path == product_control_path(manifest.artifact.os))
                .ok_or(DesktopPackageError::ManifestInvalid)?;
            if control.mode != LogicalMode::Regular || control.sha256 != expected {
                return Err(DesktopPackageError::ManifestInvalid);
            }
        }
        None => {
            if manifest
                .entries
                .iter()
                .any(|entry| entry.path == product_control_path(manifest.artifact.os))
            {
                return Err(DesktopPackageError::ManifestInvalid);
            }
        }
    }
    Ok(())
}

fn validate_source_identity(artifact: &ArtifactIdentityV1) -> Result<(), DesktopPackageError> {
    artifact
        .validate()
        .map_err(|_| DesktopPackageError::InvalidSourceArtifact)?;
    if artifact.product != ProductId::Qiongli
        || artifact.profile != CapabilityProfile::Lite
        || artifact.installer_kind != InstallerKind::PortableArchive
    {
        return Err(DesktopPackageError::InvalidSourceArtifact);
    }
    Ok(())
}

fn validate_desktop_identity(artifact: &ArtifactIdentityV1) -> Result<(), DesktopPackageError> {
    artifact
        .validate()
        .map_err(|_| DesktopPackageError::InvalidSourceArtifact)?;
    if artifact.product != ProductId::Qiongli
        || artifact.profile != CapabilityProfile::Lite
        || artifact.installer_kind != InstallerKind::NativeInstaller
    {
        return Err(DesktopPackageError::InvalidSourceArtifact);
    }
    Ok(())
}

fn validate_application_metadata(
    metadata: &DesktopApplicationMetadataV1,
    expected_version: &str,
) -> Result<(), DesktopPackageError> {
    if metadata.product_name != "Qiongli"
        || metadata.window_title != "Qiongli 2"
        || metadata.application_identifier != "io.github.jxpeng98.qiongli"
        || metadata.product_version != expected_version
        || metadata.license != "MIT"
    {
        return Err(DesktopPackageError::InvalidApplicationMetadata);
    }
    Ok(())
}

fn build_payload_entries(
    input: DesktopPayloadInput<'_>,
) -> Result<Vec<PayloadEntry>, DesktopPackageError> {
    let mut entries = match input.artifact.os {
        OperatingSystem::Macos => vec![
            payload(
                "Qiongli.app/Contents/Info.plist",
                LogicalMode::Regular,
                macos_info_plist(input.artifact, input.application)?.into_bytes(),
            ),
            payload(
                launcher_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.launcher.to_vec(),
            ),
            payload(
                canonical_binary_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.canonical.to_vec(),
            ),
            payload(
                update_helper_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.update_helper.to_vec(),
            ),
            payload(
                "Qiongli.app/Contents/Resources/LICENSE",
                LogicalMode::Regular,
                input.license_bytes.to_vec(),
            ),
            payload(
                "Qiongli.app/Contents/Resources/Qiongli.icns",
                LogicalMode::Regular,
                icns_from_png(input.icon_png)?,
            ),
        ],
        OperatingSystem::Windows => vec![
            payload(
                "Qiongli/LICENSE",
                LogicalMode::Regular,
                input.license_bytes.to_vec(),
            ),
            payload(
                launcher_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.launcher.to_vec(),
            ),
            payload(
                "Qiongli/Qiongli.exe.manifest",
                LogicalMode::Regular,
                windows_application_manifest(input.application).into_bytes(),
            ),
            payload(
                canonical_binary_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.canonical.to_vec(),
            ),
            payload(
                update_helper_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.update_helper.to_vec(),
            ),
            payload(
                "Qiongli/qiongli.png",
                LogicalMode::Regular,
                input.icon_png.to_vec(),
            ),
        ],
        OperatingSystem::Linux => vec![
            payload(
                "Qiongli.AppDir/.DirIcon",
                LogicalMode::Regular,
                input.icon_png.to_vec(),
            ),
            payload(
                launcher_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.launcher.to_vec(),
            ),
            payload(
                "Qiongli.AppDir/LICENSE",
                LogicalMode::Regular,
                input.license_bytes.to_vec(),
            ),
            payload(
                canonical_binary_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.canonical.to_vec(),
            ),
            payload(
                update_helper_path(input.artifact.os),
                LogicalMode::Executable,
                input.binaries.update_helper.to_vec(),
            ),
            payload(
                "Qiongli.AppDir/qiongli.desktop",
                LogicalMode::Regular,
                linux_desktop_entry(input.application).into_bytes(),
            ),
            payload(
                "Qiongli.AppDir/qiongli.png",
                LogicalMode::Regular,
                input.icon_png.to_vec(),
            ),
        ],
    };
    if let Some(control) = input.product_control {
        if control.is_empty() || control.len() > MAX_MANIFEST_BYTES {
            return Err(DesktopPackageError::ManifestInvalid);
        }
        entries.push(payload(
            product_control_path(input.artifact.os),
            LogicalMode::Regular,
            control.to_vec(),
        ));
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn payload(path: impl Into<String>, mode: LogicalMode, bytes: Vec<u8>) -> PayloadEntry {
    PayloadEntry {
        path: path.into(),
        mode,
        bytes,
    }
}

fn macos_info_plist(
    artifact: &ArtifactIdentityV1,
    application: &DesktopApplicationMetadataV1,
) -> Result<String, DesktopPackageError> {
    let version = Version::parse(&artifact.version)
        .map_err(|_| DesktopPackageError::InvalidApplicationMetadata)?;
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"https://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>CFBundleDevelopmentRegion</key>\n  <string>en</string>\n  <key>CFBundleDisplayName</key>\n  <string>{}</string>\n  <key>CFBundleExecutable</key>\n  <string>Qiongli</string>\n  <key>CFBundleIconFile</key>\n  <string>Qiongli.icns</string>\n  <key>CFBundleIdentifier</key>\n  <string>{}</string>\n  <key>CFBundleInfoDictionaryVersion</key>\n  <string>6.0</string>\n  <key>CFBundleName</key>\n  <string>{}</string>\n  <key>CFBundlePackageType</key>\n  <string>APPL</string>\n  <key>CFBundleShortVersionString</key>\n  <string>{}.{}.{}</string>\n  <key>CFBundleVersion</key>\n  <string>{}.{}.{}</string>\n  <key>NSHighResolutionCapable</key>\n  <true/>\n  <key>QiongliProductVersion</key>\n  <string>{}</string>\n</dict>\n</plist>\n",
        application.product_name,
        application.application_identifier,
        application.product_name,
        version.major,
        version.minor,
        version.patch,
        version.major,
        version.minor,
        version.patch,
        application.product_version,
    ))
}

fn windows_application_manifest(application: &DesktopApplicationMetadataV1) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\">\n  <assemblyIdentity name=\"{}\" version=\"2.0.0.0\" processorArchitecture=\"*\" type=\"win32\"/>\n  <trustInfo xmlns=\"urn:schemas-microsoft-com:asm.v3\">\n    <security><requestedPrivileges><requestedExecutionLevel level=\"asInvoker\" uiAccess=\"false\"/></requestedPrivileges></security>\n  </trustInfo>\n  <application xmlns=\"urn:schemas-microsoft-com:asm.v3\"><windowsSettings><dpiAware xmlns=\"http://schemas.microsoft.com/SMI/2005/WindowsSettings\">true/pm</dpiAware></windowsSettings></application>\n</assembly>\n",
        application.application_identifier
    )
}

fn linux_desktop_entry(application: &DesktopApplicationMetadataV1) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={}\nComment=Native academic research manager\nExec=AppRun\nIcon=qiongli\nTerminal=false\nCategories=Education;Science;\nStartupWMClass={}\nX-AppImage-Version={}\n",
        application.product_name, application.application_identifier, application.product_version,
    )
}

fn icns_from_png(png: &[u8]) -> Result<Vec<u8>, DesktopPackageError> {
    validate_png(png)?;
    let total = png
        .len()
        .checked_add(16)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(DesktopPackageError::IconInvalid)?;
    let element = png
        .len()
        .checked_add(8)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(DesktopPackageError::IconInvalid)?;
    let mut bytes = Vec::with_capacity(total as usize);
    bytes.extend_from_slice(b"icns");
    bytes.extend_from_slice(&total.to_be_bytes());
    bytes.extend_from_slice(b"ic08");
    bytes.extend_from_slice(&element.to_be_bytes());
    bytes.extend_from_slice(png);
    Ok(bytes)
}

fn validate_png(png: &[u8]) -> Result<(), DesktopPackageError> {
    if png.len() < 33
        || png.len() > MAX_ICON_BYTES
        || png.get(..8) != Some(PNG_SIGNATURE)
        || png.get(12..16) != Some(b"IHDR")
        || png.get(16..20) != Some(&256_u32.to_be_bytes())
        || png.get(20..24) != Some(&256_u32.to_be_bytes())
        || png.get(24) != Some(&8)
        || png.get(25) != Some(&6)
    {
        return Err(DesktopPackageError::IconInvalid);
    }
    let decoder = png::Decoder::new(Cursor::new(png));
    let mut reader = decoder
        .read_info()
        .map_err(|_| DesktopPackageError::IconInvalid)?;
    let info = reader.info();
    if info.width != 256
        || info.height != 256
        || info.color_type != png::ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
    {
        return Err(DesktopPackageError::IconInvalid);
    }
    let output_size = reader
        .output_buffer_size()
        .ok_or(DesktopPackageError::IconInvalid)?;
    if output_size != 256 * 256 * 4 {
        return Err(DesktopPackageError::IconInvalid);
    }
    let mut decoded = vec![0; output_size];
    let frame = reader
        .next_frame(&mut decoded)
        .map_err(|_| DesktopPackageError::IconInvalid)?;
    if frame.buffer_size() != output_size
        || frame.width != 256
        || frame.height != 256
        || frame.color_type != png::ColorType::Rgba
        || frame.bit_depth != png::BitDepth::Eight
    {
        return Err(DesktopPackageError::IconInvalid);
    }
    Ok(())
}

fn package_root(os: OperatingSystem) -> &'static str {
    match os {
        OperatingSystem::Macos => "Qiongli.app",
        OperatingSystem::Windows => "Qiongli",
        OperatingSystem::Linux => "Qiongli.AppDir",
    }
}

fn manifest_path(os: OperatingSystem) -> String {
    match os {
        OperatingSystem::Macos => {
            format!("Qiongli.app/Contents/Resources/{DESKTOP_PACKAGE_MANIFEST_FILE}")
        }
        OperatingSystem::Windows => format!("Qiongli/{DESKTOP_PACKAGE_MANIFEST_FILE}"),
        OperatingSystem::Linux => format!("Qiongli.AppDir/{DESKTOP_PACKAGE_MANIFEST_FILE}"),
    }
}

fn launcher_path(os: OperatingSystem) -> &'static str {
    match os {
        OperatingSystem::Macos => "Qiongli.app/Contents/MacOS/Qiongli",
        OperatingSystem::Windows => "Qiongli/Qiongli.exe",
        OperatingSystem::Linux => "Qiongli.AppDir/AppRun",
    }
}

fn canonical_binary_path(os: OperatingSystem) -> &'static str {
    match os {
        OperatingSystem::Macos => "Qiongli.app/Contents/MacOS/qiongli-cli",
        OperatingSystem::Windows => "Qiongli/qiongli-cli.exe",
        OperatingSystem::Linux => "Qiongli.AppDir/qiongli-cli",
    }
}

fn update_helper_path(os: OperatingSystem) -> &'static str {
    match os {
        OperatingSystem::Macos => "Qiongli.app/Contents/MacOS/qiongli-update-helper",
        OperatingSystem::Windows => "Qiongli/qiongli-update-helper.exe",
        OperatingSystem::Linux => "Qiongli.AppDir/qiongli-update-helper",
    }
}

fn product_control_path(os: OperatingSystem) -> String {
    match os {
        OperatingSystem::Macos => format!(
            "Qiongli.app/Contents/Resources/{}",
            crate::PACKAGED_PRODUCT_CONTROL_FILE
        ),
        OperatingSystem::Windows => {
            format!("Qiongli/{}", crate::PACKAGED_PRODUCT_CONTROL_FILE)
        }
        OperatingSystem::Linux => {
            format!("Qiongli.AppDir/{}", crate::PACKAGED_PRODUCT_CONTROL_FILE)
        }
    }
}

fn expected_payload_paths(os: OperatingSystem, has_product_control: bool) -> BTreeSet<String> {
    let paths: &[&str] = match os {
        OperatingSystem::Macos => &[
            "Qiongli.app/Contents/Info.plist",
            "Qiongli.app/Contents/MacOS/Qiongli",
            "Qiongli.app/Contents/MacOS/qiongli-cli",
            "Qiongli.app/Contents/MacOS/qiongli-update-helper",
            "Qiongli.app/Contents/Resources/LICENSE",
            "Qiongli.app/Contents/Resources/Qiongli.icns",
        ],
        OperatingSystem::Windows => &[
            "Qiongli/LICENSE",
            "Qiongli/Qiongli.exe",
            "Qiongli/Qiongli.exe.manifest",
            "Qiongli/qiongli-cli.exe",
            "Qiongli/qiongli-update-helper.exe",
            "Qiongli/qiongli.png",
        ],
        OperatingSystem::Linux => &[
            "Qiongli.AppDir/.DirIcon",
            "Qiongli.AppDir/AppRun",
            "Qiongli.AppDir/LICENSE",
            "Qiongli.AppDir/qiongli-cli",
            "Qiongli.AppDir/qiongli-update-helper",
            "Qiongli.AppDir/qiongli.desktop",
            "Qiongli.AppDir/qiongli.png",
        ],
    };
    let mut expected = paths
        .iter()
        .map(|path| (*path).to_string())
        .collect::<BTreeSet<_>>();
    if has_product_control {
        expected.insert(product_control_path(os));
    }
    expected
}

fn binary_magic_matches(os: OperatingSystem, bytes: &[u8]) -> bool {
    match os {
        OperatingSystem::Macos => matches!(
            bytes.get(..4),
            Some([0xcf, 0xfa, 0xed, 0xfe])
                | Some([0xfe, 0xed, 0xfa, 0xcf])
                | Some([0xca, 0xfe, 0xba, 0xbe])
                | Some([0xbe, 0xba, 0xfe, 0xca])
        ),
        OperatingSystem::Windows => bytes.starts_with(b"MZ"),
        OperatingSystem::Linux => bytes.starts_with(b"\x7fELF"),
    }
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && is_lower_hex(value, value.len())
}

fn valid_archive_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 256
        && path.is_ascii()
        && !path.starts_with('/')
        && !path.ends_with('/')
        && !path.contains('\\')
        && path
            .split('/')
            .all(|component| !component.is_empty() && !matches!(component, "." | ".."))
}

pub(crate) fn entry_content_root(entries: &[DesktopPackageEntryV1]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CONTENT_ROOT_DOMAIN);
    for entry in entries {
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

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, DesktopPackageError> {
    serde_json_canonicalizer::to_vec(value).map_err(|_| DesktopPackageError::ManifestInvalid)
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

fn os_label(os: OperatingSystem) -> &'static str {
    match os {
        OperatingSystem::Macos => "macos",
        OperatingSystem::Windows => "windows",
        OperatingSystem::Linux => "linux",
    }
}

fn architecture_label(architecture: Architecture) -> &'static str {
    match architecture {
        Architecture::Aarch64 => "aarch64",
        Architecture::X86_64 => "x86-64",
    }
}

#[derive(Clone, Copy)]
struct ZipSourceEntry<'a> {
    path: &'a str,
    mode: LogicalMode,
    bytes: &'a [u8],
}

struct ZipCentralRecord {
    path: String,
    crc32: u32,
    size: u32,
    attributes: u32,
    local_offset: u32,
}

fn build_zip(entries: &[PayloadEntry]) -> Result<Vec<u8>, DesktopPackageError> {
    if entries.is_empty() || entries.len() > MAX_ENTRY_COUNT {
        return Err(DesktopPackageError::ArchiveInvalid);
    }
    let sources = entries
        .iter()
        .map(|entry| ZipSourceEntry {
            path: &entry.path,
            mode: entry.mode,
            bytes: &entry.bytes,
        })
        .collect::<Vec<_>>();
    let mut prior = None;
    let capacity = sources.iter().try_fold(2_048_usize, |total, entry| {
        if !valid_archive_path(entry.path)
            || entry.bytes.is_empty()
            || prior.is_some_and(|value: &str| value >= entry.path)
        {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        prior = Some(entry.path);
        total
            .checked_add(entry.bytes.len())
            .and_then(|value| value.checked_add(entry.path.len() * 2 + 80))
            .ok_or(DesktopPackageError::ArchiveTooLarge)
    })?;
    if capacity > MAX_ARCHIVE_BYTES {
        return Err(DesktopPackageError::ArchiveTooLarge);
    }
    let mut bytes = Vec::with_capacity(capacity);
    let mut central = Vec::with_capacity(sources.len());
    for entry in &sources {
        let name_length =
            u16::try_from(entry.path.len()).map_err(|_| DesktopPackageError::ArchiveInvalid)?;
        let size =
            u32::try_from(entry.bytes.len()).map_err(|_| DesktopPackageError::ArchiveTooLarge)?;
        let local_offset =
            u32::try_from(bytes.len()).map_err(|_| DesktopPackageError::ArchiveTooLarge)?;
        let crc32 = crc32fast::hash(entry.bytes);
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
        bytes.extend_from_slice(entry.path.as_bytes());
        bytes.extend_from_slice(entry.bytes);
        central.push(ZipCentralRecord {
            path: entry.path.to_string(),
            crc32,
            size,
            attributes: mode_attributes(entry.mode),
            local_offset,
        });
    }
    let central_offset =
        u32::try_from(bytes.len()).map_err(|_| DesktopPackageError::ArchiveTooLarge)?;
    for record in &central {
        let name_length =
            u16::try_from(record.path.len()).map_err(|_| DesktopPackageError::ArchiveInvalid)?;
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
        push_u32(&mut bytes, record.attributes);
        push_u32(&mut bytes, record.local_offset);
        bytes.extend_from_slice(record.path.as_bytes());
    }
    let central_size = u32::try_from(bytes.len())
        .ok()
        .and_then(|end| end.checked_sub(central_offset))
        .ok_or(DesktopPackageError::ArchiveTooLarge)?;
    let entry_count =
        u16::try_from(central.len()).map_err(|_| DesktopPackageError::ArchiveTooLarge)?;
    push_u32(&mut bytes, ZIP_END_OF_CENTRAL_DIRECTORY);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, entry_count);
    push_u16(&mut bytes, entry_count);
    push_u32(&mut bytes, central_size);
    push_u32(&mut bytes, central_offset);
    push_u16(&mut bytes, 0);
    if bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(DesktopPackageError::ArchiveTooLarge);
    }
    Ok(bytes)
}

struct ParsedZipEntry<'a> {
    path: String,
    mode: LogicalMode,
    bytes: &'a [u8],
    crc32: u32,
    size: u32,
    attributes: u32,
    local_offset: u32,
}

fn parse_zip(bytes: &[u8]) -> Result<Vec<ParsedZipEntry<'_>>, DesktopPackageError> {
    if bytes.len() < 22 || bytes.len() > MAX_ARCHIVE_BYTES {
        return Err(DesktopPackageError::ArchiveInvalid);
    }
    let eocd_start = bytes
        .len()
        .checked_sub(22)
        .ok_or(DesktopPackageError::ArchiveInvalid)?;
    let mut eocd = SliceReader::new(&bytes[eocd_start..]);
    if eocd.read_u32()? != ZIP_END_OF_CENTRAL_DIRECTORY
        || eocd.read_u16()? != 0
        || eocd.read_u16()? != 0
    {
        return Err(DesktopPackageError::ArchiveInvalid);
    }
    let disk_entries = usize::from(eocd.read_u16()?);
    let total_entries = usize::from(eocd.read_u16()?);
    let central_size =
        usize::try_from(eocd.read_u32()?).map_err(|_| DesktopPackageError::ArchiveInvalid)?;
    let central_offset =
        usize::try_from(eocd.read_u32()?).map_err(|_| DesktopPackageError::ArchiveInvalid)?;
    if eocd.read_u16()? != 0
        || !eocd.is_finished()
        || disk_entries != total_entries
        || total_entries == 0
        || total_entries > MAX_ENTRY_COUNT
        || central_offset.checked_add(central_size) != Some(eocd_start)
    {
        return Err(DesktopPackageError::ArchiveInvalid);
    }

    let mut local_reader = SliceReader::new(&bytes[..central_offset]);
    let mut entries = Vec::with_capacity(total_entries);
    let mut prior_path = None;
    for _ in 0..total_entries {
        let local_offset = u32::try_from(local_reader.position())
            .map_err(|_| DesktopPackageError::ArchiveInvalid)?;
        if local_reader.read_u32()? != ZIP_LOCAL_HEADER
            || local_reader.read_u16()? != ZIP_VERSION
            || local_reader.read_u16()? != ZIP_UTF8_FLAG
            || local_reader.read_u16()? != ZIP_STORED_METHOD
            || local_reader.read_u16()? != ZIP_DOS_TIME
            || local_reader.read_u16()? != ZIP_DOS_DATE
        {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        let crc32 = local_reader.read_u32()?;
        let compressed_size = local_reader.read_u32()?;
        let size = local_reader.read_u32()?;
        let name_length = usize::from(local_reader.read_u16()?);
        if local_reader.read_u16()? != 0 || compressed_size != size {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        let path = std::str::from_utf8(local_reader.take(name_length)?)
            .map_err(|_| DesktopPackageError::ArchiveInvalid)?
            .to_string();
        if !valid_archive_path(&path)
            || prior_path
                .as_ref()
                .is_some_and(|prior: &String| prior >= &path)
        {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        prior_path = Some(path.clone());
        let data_length = usize::try_from(size).map_err(|_| DesktopPackageError::ArchiveInvalid)?;
        let data = local_reader.take(data_length)?;
        if data.is_empty() || crc32fast::hash(data) != crc32 {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        entries.push(ParsedZipEntry {
            path,
            mode: LogicalMode::Regular,
            bytes: data,
            crc32,
            size,
            attributes: 0,
            local_offset,
        });
    }
    if !local_reader.is_finished() {
        return Err(DesktopPackageError::ArchiveInvalid);
    }

    let central_end = central_offset
        .checked_add(central_size)
        .ok_or(DesktopPackageError::ArchiveInvalid)?;
    let mut central_reader = SliceReader::new(&bytes[central_offset..central_end]);
    for entry in &mut entries {
        if central_reader.read_u32()? != ZIP_CENTRAL_HEADER
            || central_reader.read_u16()? != ZIP_VERSION_MADE_BY_UNIX
            || central_reader.read_u16()? != ZIP_VERSION
            || central_reader.read_u16()? != ZIP_UTF8_FLAG
            || central_reader.read_u16()? != ZIP_STORED_METHOD
            || central_reader.read_u16()? != ZIP_DOS_TIME
            || central_reader.read_u16()? != ZIP_DOS_DATE
            || central_reader.read_u32()? != entry.crc32
            || central_reader.read_u32()? != entry.size
            || central_reader.read_u32()? != entry.size
        {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        let name_length = usize::from(central_reader.read_u16()?);
        if central_reader.read_u16()? != 0
            || central_reader.read_u16()? != 0
            || central_reader.read_u16()? != 0
            || central_reader.read_u16()? != 0
        {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        let attributes = central_reader.read_u32()?;
        if central_reader.read_u32()? != entry.local_offset
            || central_reader.take(name_length)? != entry.path.as_bytes()
        {
            return Err(DesktopPackageError::ArchiveInvalid);
        }
        entry.mode = attributes_mode(attributes)?;
        entry.attributes = attributes;
    }
    if !central_reader.is_finished() {
        return Err(DesktopPackageError::ArchiveInvalid);
    }
    Ok(entries)
}

fn mode_attributes(mode: LogicalMode) -> u32 {
    match mode {
        LogicalMode::Regular => ZIP_REGULAR_ATTRIBUTES,
        LogicalMode::Executable => ZIP_EXECUTABLE_ATTRIBUTES,
    }
}

fn attributes_mode(attributes: u32) -> Result<LogicalMode, DesktopPackageError> {
    match attributes {
        ZIP_REGULAR_ATTRIBUTES => Ok(LogicalMode::Regular),
        ZIP_EXECUTABLE_ATTRIBUTES => Ok(LogicalMode::Executable),
        _ => Err(DesktopPackageError::ArchiveInvalid),
    }
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

    fn take(&mut self, length: usize) -> Result<&'a [u8], DesktopPackageError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(DesktopPackageError::ArchiveInvalid)?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(DesktopPackageError::ArchiveInvalid)?;
        self.position = end;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, DesktopPackageError> {
        let value: [u8; 2] = self
            .take(2)?
            .try_into()
            .map_err(|_| DesktopPackageError::ArchiveInvalid)?;
        Ok(u16::from_le_bytes(value))
    }

    fn read_u32(&mut self) -> Result<u32, DesktopPackageError> {
        let value: [u8; 4] = self
            .take(4)?
            .try_into()
            .map_err(|_| DesktopPackageError::ArchiveInvalid)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(os: OperatingSystem) -> ArtifactIdentityV1 {
        ArtifactIdentityV1 {
            product: ProductId::Qiongli,
            version: "2.0.0-alpha.1".to_string(),
            channel: crate::ReleaseChannel::Alpha,
            profile: CapabilityProfile::Lite,
            os,
            arch: Architecture::X86_64,
            installer_kind: InstallerKind::NativeInstaller,
        }
    }

    fn application() -> DesktopApplicationMetadataV1 {
        DesktopApplicationMetadataV1::new(
            "Qiongli",
            "Qiongli 2",
            "io.github.jxpeng98.qiongli",
            "2.0.0-alpha.1",
            "MIT",
        )
    }

    fn png_stub() -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, 256, 256);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&vec![0; 256 * 256 * 4]).unwrap();
        drop(writer);
        bytes
    }

    #[test]
    fn all_platform_layouts_are_exact_and_collision_free() {
        for os in [
            OperatingSystem::Macos,
            OperatingSystem::Windows,
            OperatingSystem::Linux,
        ] {
            let canonical = match os {
                OperatingSystem::Macos => b"\xcf\xfa\xed\xfecanonical".as_slice(),
                OperatingSystem::Windows => b"MZcanonical".as_slice(),
                OperatingSystem::Linux => b"\x7fELFcanonical".as_slice(),
            };
            let launcher = match os {
                OperatingSystem::Macos => b"\xcf\xfa\xed\xfelauncher".as_slice(),
                OperatingSystem::Windows => b"MZlauncher".as_slice(),
                OperatingSystem::Linux => b"\x7fELFlauncher".as_slice(),
            };
            let update_helper = match os {
                OperatingSystem::Macos => b"\xcf\xfa\xed\xfeupdate-helper".as_slice(),
                OperatingSystem::Windows => b"MZupdate-helper".as_slice(),
                OperatingSystem::Linux => b"\x7fELFupdate-helper".as_slice(),
            };
            let artifact = artifact(os);
            let icon = png_stub();
            let application = application();
            let entries = build_payload_entries(DesktopPayloadInput {
                artifact: &artifact,
                binaries: DesktopPackageBinaries::new(canonical, launcher, update_helper),
                icon_png: &icon,
                license_bytes: b"MIT License\nPermission is hereby granted",
                application: &application,
                product_control: None,
            })
            .unwrap();
            assert_eq!(
                entries
                    .iter()
                    .map(|entry| entry.path.clone())
                    .collect::<BTreeSet<_>>(),
                expected_payload_paths(os, false)
            );
            assert_ne!(launcher_path(os), canonical_binary_path(os));
            assert_ne!(update_helper_path(os), canonical_binary_path(os));
            assert_ne!(update_helper_path(os), launcher_path(os));
        }
    }

    #[test]
    fn generic_zip_is_deterministic_and_rejects_tampering() {
        let entries = vec![
            payload(
                "Qiongli/AppRun",
                LogicalMode::Executable,
                b"launcher".to_vec(),
            ),
            payload("Qiongli/LICENSE", LogicalMode::Regular, b"license".to_vec()),
        ];
        let first = build_zip(&entries).unwrap();
        let second = build_zip(&entries).unwrap();
        assert_eq!(first, second);
        let parsed = parse_zip(&first).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].mode, LogicalMode::Executable);
        assert_eq!(parsed[1].mode, LogicalMode::Regular);

        let mut tampered = first;
        let offset = tampered
            .windows(b"launcher".len())
            .position(|window| window == b"launcher")
            .unwrap();
        tampered[offset] ^= 1;
        assert!(matches!(
            parse_zip(&tampered),
            Err(DesktopPackageError::ArchiveInvalid)
        ));
    }

    #[test]
    fn icon_validation_decodes_the_complete_png() {
        let valid = png_stub();
        validate_png(&valid).unwrap();
        let mut corrupted = valid;
        let midpoint = corrupted.len() / 2;
        corrupted[midpoint] ^= 1;
        assert_eq!(
            validate_png(&corrupted),
            Err(DesktopPackageError::IconInvalid)
        );
    }

    #[test]
    fn desktop_file_names_are_target_specific() {
        for (os, suffix) in [
            (OperatingSystem::Macos, "macos-x86-64.app.zip"),
            (OperatingSystem::Windows, "windows-x86-64.zip"),
            (OperatingSystem::Linux, "linux-x86-64.appdir.zip"),
        ] {
            assert!(
                desktop_package_file_name(&artifact(os))
                    .unwrap()
                    .ends_with(suffix)
            );
        }
    }

    #[test]
    fn errors_are_fixed_and_never_render_paths() {
        for error in [
            DesktopPackageError::InvalidSourceArtifact,
            DesktopPackageError::InvalidProductSource,
            DesktopPackageError::InvalidApplicationMetadata,
            DesktopPackageError::CanonicalBinaryInvalid,
            DesktopPackageError::LauncherInvalid,
            DesktopPackageError::IconInvalid,
            DesktopPackageError::LicenseInvalid,
            DesktopPackageError::ManifestInvalid,
            DesktopPackageError::ArchiveInvalid,
            DesktopPackageError::ArchiveTooLarge,
            DesktopPackageError::ArchiveDrift,
        ] {
            let rendered = error.to_string();
            assert!(rendered.starts_with("desktop-package-"));
            assert!(!rendered.contains('/'));
            assert!(!rendered.contains('\\'));
        }
    }
}
