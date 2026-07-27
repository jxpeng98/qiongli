use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{
    VerifiedZoteroCompanionArtifact, ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE,
    ZOTERO_COMPANION_ENDPOINT_VERSION, ZOTERO_COMPANION_PACKAGED_XPI_FILE,
    ZOTERO_COMPANION_ZOTERO_MAX_VERSION, ZOTERO_COMPANION_ZOTERO_MIN_VERSION,
    verify_zotero_companion_artifact,
};

pub const ZOTERO_COMPANION_STAGE_RECEIPT_SCHEMA_VERSION: u32 = 1;
pub const ZOTERO_COMPANION_STAGE_RECEIPT_FILE: &str = "qiongli-zotero-companion.stage.receipt.json";

const STAGE_ROOT_DIRECTORY: &str = "zotero";
const STAGE_COMPANION_DIRECTORY: &str = "companion";
const MAX_RECEIPT_BYTES: u64 = 64 * 1024;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_XPI_BYTES: u64 = 2 * 1024 * 1024;
const PLAN_DOMAIN: &[u8] = b"qiongli-zotero-companion-stage-plan-v1\0";
static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ZoteroCompanionStageRecordType {
    QiongliZoteroCompanionStage,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ZoteroCompanionStageStatus {
    PreparedForZoteroHandoff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoteroCompanionStageEffect {
    Stage,
    AlreadyCurrent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZoteroCompanionStageReceiptV1 {
    pub schema_version: u32,
    pub record_type: ZoteroCompanionStageRecordType,
    pub status: ZoteroCompanionStageStatus,
    pub companion_version: String,
    pub endpoint_version: String,
    pub zotero_min_version: String,
    pub zotero_max_version: String,
    pub artifact_manifest_file: String,
    pub artifact_manifest_size_bytes: u64,
    pub artifact_manifest_sha256: String,
    pub xpi_file: String,
    pub xpi_size_bytes: u64,
    pub xpi_sha256: String,
}

#[derive(Clone)]
pub struct ZoteroCompanionStagePlan {
    state_root: PathBuf,
    stage_root: PathBuf,
    artifact: VerifiedZoteroCompanionArtifact,
    receipt: ZoteroCompanionStageReceiptV1,
    receipt_bytes: Vec<u8>,
    plan_digest_sha256: String,
    effect: ZoteroCompanionStageEffect,
}

impl ZoteroCompanionStagePlan {
    #[must_use]
    pub const fn effect(&self) -> ZoteroCompanionStageEffect {
        self.effect
    }

    #[must_use]
    pub fn plan_digest_sha256(&self) -> &str {
        &self.plan_digest_sha256
    }

    #[must_use]
    pub fn companion_version(&self) -> &str {
        &self.receipt.companion_version
    }

    #[must_use]
    pub fn artifact_sha256(&self) -> &str {
        &self.receipt.xpi_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedZoteroCompanionStage {
    receipt: ZoteroCompanionStageReceiptV1,
    root: PathBuf,
}

impl VerifiedZoteroCompanionStage {
    #[must_use]
    pub const fn receipt(&self) -> &ZoteroCompanionStageReceiptV1 {
        &self.receipt
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn xpi_path(&self) -> PathBuf {
        self.root.join(ZOTERO_COMPANION_PACKAGED_XPI_FILE)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoteroCompanionStageError {
    InvalidStateRoot,
    InvalidArtifact,
    StageDrift,
    ApprovalRequired,
    PlanChanged,
    PersistenceFailed,
}

impl ZoteroCompanionStageError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::InvalidStateRoot => "zotero-companion-stage-root-invalid",
            Self::InvalidArtifact => "zotero-companion-stage-artifact-invalid",
            Self::StageDrift => "zotero-companion-stage-drift",
            Self::ApprovalRequired => "zotero-companion-stage-approval-required",
            Self::PlanChanged => "zotero-companion-stage-plan-changed",
            Self::PersistenceFailed => "zotero-companion-stage-persistence-failed",
        }
    }
}

pub fn preview_zotero_companion_stage(
    state_root: &Path,
    artifact: &VerifiedZoteroCompanionArtifact,
) -> Result<ZoteroCompanionStagePlan, ZoteroCompanionStageError> {
    validate_state_root(state_root)?;
    let manifest = artifact.manifest();
    let stage_root = stage_root(state_root, artifact)?;
    let receipt = ZoteroCompanionStageReceiptV1 {
        schema_version: ZOTERO_COMPANION_STAGE_RECEIPT_SCHEMA_VERSION,
        record_type: ZoteroCompanionStageRecordType::QiongliZoteroCompanionStage,
        status: ZoteroCompanionStageStatus::PreparedForZoteroHandoff,
        companion_version: manifest.companion_version.clone(),
        endpoint_version: manifest.endpoint_version.clone(),
        zotero_min_version: manifest.zotero_min_version.clone(),
        zotero_max_version: manifest.zotero_max_version.clone(),
        artifact_manifest_file: ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE.to_owned(),
        artifact_manifest_size_bytes: artifact.manifest_bytes().len() as u64,
        artifact_manifest_sha256: sha256_hex(artifact.manifest_bytes()),
        xpi_file: ZOTERO_COMPANION_PACKAGED_XPI_FILE.to_owned(),
        xpi_size_bytes: artifact.xpi_bytes().len() as u64,
        xpi_sha256: sha256_hex(artifact.xpi_bytes()),
    };
    validate_receipt(&receipt)?;
    let receipt_bytes =
        canonical_json(&receipt).map_err(|_| ZoteroCompanionStageError::InvalidArtifact)?;
    let effect = if path_exists(&stage_root)? {
        verify_stage_root(&stage_root, artifact, &receipt)?;
        ZoteroCompanionStageEffect::AlreadyCurrent
    } else {
        ZoteroCompanionStageEffect::Stage
    };
    Ok(ZoteroCompanionStagePlan {
        state_root: state_root.to_path_buf(),
        stage_root,
        artifact: artifact.clone(),
        plan_digest_sha256: plan_digest(&receipt_bytes),
        receipt,
        receipt_bytes,
        effect,
    })
}

pub fn apply_zotero_companion_stage(
    plan: &ZoteroCompanionStagePlan,
    expected_plan_digest_sha256: &str,
    approve_filesystem_write: bool,
) -> Result<VerifiedZoteroCompanionStage, ZoteroCompanionStageError> {
    if expected_plan_digest_sha256 != plan.plan_digest_sha256 {
        return Err(ZoteroCompanionStageError::PlanChanged);
    }
    if plan.effect == ZoteroCompanionStageEffect::AlreadyCurrent {
        return verify_zotero_companion_stage(&plan.state_root, &plan.artifact)?
            .ok_or(ZoteroCompanionStageError::StageDrift);
    }
    if !approve_filesystem_write {
        return Err(ZoteroCompanionStageError::ApprovalRequired);
    }

    let parent = plan
        .stage_root
        .parent()
        .ok_or(ZoteroCompanionStageError::InvalidStateRoot)?;
    ensure_directory_chain(&plan.state_root, parent)?;
    if path_exists(&plan.stage_root)? {
        return verify_zotero_companion_stage(&plan.state_root, &plan.artifact)?
            .ok_or(ZoteroCompanionStageError::StageDrift);
    }

    let temporary = parent.join(format!(
        ".qiongli-zotero-companion-stage-{}-{}",
        std::process::id(),
        NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed)
    ));
    create_private_directory(&temporary)?;
    let result = (|| {
        write_new_file(
            &temporary.join(ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE),
            plan.artifact.manifest_bytes(),
        )?;
        write_new_file(
            &temporary.join(ZOTERO_COMPANION_PACKAGED_XPI_FILE),
            plan.artifact.xpi_bytes(),
        )?;
        write_new_file(
            &temporary.join(ZOTERO_COMPANION_STAGE_RECEIPT_FILE),
            &plan.receipt_bytes,
        )?;
        sync_directory(&temporary)?;
        fs::rename(&temporary, &plan.stage_root)
            .map_err(|_| ZoteroCompanionStageError::PersistenceFailed)?;
        sync_directory(parent)?;
        verify_stage_root(&plan.stage_root, &plan.artifact, &plan.receipt)
    })();
    if result.is_err() && temporary.exists() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result?;
    Ok(VerifiedZoteroCompanionStage {
        receipt: plan.receipt.clone(),
        root: plan.stage_root.clone(),
    })
}

pub fn verify_zotero_companion_stage(
    state_root: &Path,
    artifact: &VerifiedZoteroCompanionArtifact,
) -> Result<Option<VerifiedZoteroCompanionStage>, ZoteroCompanionStageError> {
    validate_state_root(state_root)?;
    let root = stage_root(state_root, artifact)?;
    if !path_exists(&root)? {
        return Ok(None);
    }
    let receipt_bytes = read_regular_bounded(
        &root.join(ZOTERO_COMPANION_STAGE_RECEIPT_FILE),
        MAX_RECEIPT_BYTES,
    )?;
    let receipt = serde_json::from_slice::<ZoteroCompanionStageReceiptV1>(&receipt_bytes)
        .map_err(|_| ZoteroCompanionStageError::StageDrift)?;
    if canonical_json(&receipt).map_err(|_| ZoteroCompanionStageError::StageDrift)? != receipt_bytes
    {
        return Err(ZoteroCompanionStageError::StageDrift);
    }
    verify_stage_root(&root, artifact, &receipt)?;
    Ok(Some(VerifiedZoteroCompanionStage { receipt, root }))
}

fn verify_stage_root(
    root: &Path,
    artifact: &VerifiedZoteroCompanionArtifact,
    receipt: &ZoteroCompanionStageReceiptV1,
) -> Result<(), ZoteroCompanionStageError> {
    let metadata = fs::symlink_metadata(root).map_err(|_| ZoteroCompanionStageError::StageDrift)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ZoteroCompanionStageError::StageDrift);
    }
    let expected = [
        ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE,
        ZOTERO_COMPANION_PACKAGED_XPI_FILE,
        ZOTERO_COMPANION_STAGE_RECEIPT_FILE,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    let actual = fs::read_dir(root)
        .map_err(|_| ZoteroCompanionStageError::StageDrift)?
        .map(|entry| {
            let entry = entry.map_err(|_| ZoteroCompanionStageError::StageDrift)?;
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|_| ZoteroCompanionStageError::StageDrift)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(ZoteroCompanionStageError::StageDrift);
            }
            entry
                .file_name()
                .into_string()
                .map_err(|_| ZoteroCompanionStageError::StageDrift)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if actual != expected {
        return Err(ZoteroCompanionStageError::StageDrift);
    }

    let manifest_bytes = read_regular_bounded(
        &root.join(ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE),
        MAX_MANIFEST_BYTES,
    )?;
    let xpi_bytes = read_regular_bounded(
        &root.join(ZOTERO_COMPANION_PACKAGED_XPI_FILE),
        MAX_XPI_BYTES,
    )?;
    let receipt_bytes = read_regular_bounded(
        &root.join(ZOTERO_COMPANION_STAGE_RECEIPT_FILE),
        MAX_RECEIPT_BYTES,
    )?;
    let observed_receipt = serde_json::from_slice::<ZoteroCompanionStageReceiptV1>(&receipt_bytes)
        .map_err(|_| ZoteroCompanionStageError::StageDrift)?;
    if canonical_json(&observed_receipt).map_err(|_| ZoteroCompanionStageError::StageDrift)?
        != receipt_bytes
        || &observed_receipt != receipt
        || verify_zotero_companion_artifact(&manifest_bytes, &xpi_bytes)
            .map_err(|_| ZoteroCompanionStageError::StageDrift)?
            != *artifact
        || receipt.artifact_manifest_size_bytes != manifest_bytes.len() as u64
        || receipt.artifact_manifest_sha256 != sha256_hex(&manifest_bytes)
        || receipt.xpi_size_bytes != xpi_bytes.len() as u64
        || receipt.xpi_sha256 != sha256_hex(&xpi_bytes)
    {
        return Err(ZoteroCompanionStageError::StageDrift);
    }
    Ok(())
}

fn validate_receipt(
    receipt: &ZoteroCompanionStageReceiptV1,
) -> Result<(), ZoteroCompanionStageError> {
    if receipt.schema_version != ZOTERO_COMPANION_STAGE_RECEIPT_SCHEMA_VERSION
        || receipt.record_type != ZoteroCompanionStageRecordType::QiongliZoteroCompanionStage
        || receipt.status != ZoteroCompanionStageStatus::PreparedForZoteroHandoff
        || receipt.endpoint_version != ZOTERO_COMPANION_ENDPOINT_VERSION
        || receipt.zotero_min_version != ZOTERO_COMPANION_ZOTERO_MIN_VERSION
        || receipt.zotero_max_version != ZOTERO_COMPANION_ZOTERO_MAX_VERSION
        || receipt.artifact_manifest_file != ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE
        || receipt.xpi_file != ZOTERO_COMPANION_PACKAGED_XPI_FILE
        || receipt.companion_version.is_empty()
        || receipt.companion_version.len() > 80
        || receipt.artifact_manifest_size_bytes == 0
        || receipt.artifact_manifest_size_bytes > MAX_MANIFEST_BYTES
        || receipt.xpi_size_bytes == 0
        || receipt.xpi_size_bytes > MAX_XPI_BYTES
        || !is_lower_sha256(&receipt.artifact_manifest_sha256)
        || !is_lower_sha256(&receipt.xpi_sha256)
    {
        return Err(ZoteroCompanionStageError::InvalidArtifact);
    }
    Ok(())
}

fn stage_root(
    state_root: &Path,
    artifact: &VerifiedZoteroCompanionArtifact,
) -> Result<PathBuf, ZoteroCompanionStageError> {
    let manifest = artifact.manifest();
    if manifest.artifact_sha256.len() < 16 {
        return Err(ZoteroCompanionStageError::InvalidArtifact);
    }
    Ok(state_root
        .join(STAGE_ROOT_DIRECTORY)
        .join(STAGE_COMPANION_DIRECTORY)
        .join(format!(
            "{}-{}",
            manifest.companion_version,
            &manifest.artifact_sha256[..16]
        )))
}

fn validate_state_root(path: &Path) -> Result<(), ZoteroCompanionStageError> {
    if !path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::CurDir | Component::ParentDir | Component::RootDir
            ) && !matches!(component, Component::RootDir)
        })
    {
        return Err(ZoteroCompanionStageError::InvalidStateRoot);
    }
    if path.exists() {
        let metadata =
            fs::symlink_metadata(path).map_err(|_| ZoteroCompanionStageError::InvalidStateRoot)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(ZoteroCompanionStageError::InvalidStateRoot);
        }
    }
    Ok(())
}

fn ensure_directory_chain(
    state_root: &Path,
    target: &Path,
) -> Result<(), ZoteroCompanionStageError> {
    if !target.starts_with(state_root) {
        return Err(ZoteroCompanionStageError::InvalidStateRoot);
    }
    fs::create_dir_all(target).map_err(|_| ZoteroCompanionStageError::PersistenceFailed)?;
    let mut current = state_root.to_path_buf();
    for component in target
        .strip_prefix(state_root)
        .map_err(|_| ZoteroCompanionStageError::InvalidStateRoot)?
        .components()
    {
        current.push(component);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|_| ZoteroCompanionStageError::PersistenceFailed)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(ZoteroCompanionStageError::InvalidStateRoot);
        }
        set_private_directory_permissions(&current)?;
    }
    Ok(())
}

fn path_exists(path: &Path) -> Result<bool, ZoteroCompanionStageError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(ZoteroCompanionStageError::PersistenceFailed),
    }
}

fn read_regular_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, ZoteroCompanionStageError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ZoteroCompanionStageError::StageDrift)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err(ZoteroCompanionStageError::StageDrift);
    }
    fs::read(path).map_err(|_| ZoteroCompanionStageError::StageDrift)
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), ZoteroCompanionStageError> {
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt as _;

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(path)
        .map_err(|_| ZoteroCompanionStageError::PersistenceFailed)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| ZoteroCompanionStageError::PersistenceFailed)
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> Result<(), ZoteroCompanionStageError> {
    use std::os::unix::fs::DirBuilderExt as _;

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|_| ZoteroCompanionStageError::PersistenceFailed)
}

#[cfg(not(unix))]
fn create_private_directory(path: &Path) -> Result<(), ZoteroCompanionStageError> {
    fs::create_dir(path).map_err(|_| ZoteroCompanionStageError::PersistenceFailed)
}

#[cfg(unix)]
fn set_private_directory_permissions(path: &Path) -> Result<(), ZoteroCompanionStageError> {
    use std::os::unix::fs::PermissionsExt as _;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| ZoteroCompanionStageError::PersistenceFailed)
}

#[cfg(not(unix))]
const fn set_private_directory_permissions(_path: &Path) -> Result<(), ZoteroCompanionStageError> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), ZoteroCompanionStageError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| ZoteroCompanionStageError::PersistenceFailed)
}

#[cfg(not(unix))]
const fn sync_directory(_path: &Path) -> Result<(), ZoteroCompanionStageError> {
    Ok(())
}

fn plan_digest(receipt_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(PLAN_DOMAIN);
    hasher.update((receipt_bytes.len() as u64).to_be_bytes());
    hasher.update(receipt_bytes);
    encode_hex(&hasher.finalize())
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    serde_json_canonicalizer::to_vec(value)
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

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::{
        ZOTERO_COMPANION_DISPLAY_NAME, ZOTERO_COMPANION_ID, ZOTERO_COMPANION_SOURCE_PATHS,
        ZOTERO_COMPANION_UPDATE_URL, ZoteroCompanionSourceEntry, compose_zotero_companion_artifact,
    };

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    fn fixture_artifact() -> VerifiedZoteroCompanionArtifact {
        let readme = b"# Companion\n";
        let bootstrap = b"const response = { version: \"0.3.0\", endpoint_version: \"2\" };\n";
        let bridge = b"const response = { version: \"0.3.0\", endpoint_version: \"2\" };\n";
        let manifest = format!(
            "{{\"manifest_version\":2,\"name\":\"{ZOTERO_COMPANION_DISPLAY_NAME}\",\"version\":\"0.3.0\",\"applications\":{{\"zotero\":{{\"id\":\"{ZOTERO_COMPANION_ID}\",\"update_url\":\"{ZOTERO_COMPANION_UPDATE_URL}\",\"strict_min_version\":\"{ZOTERO_COMPANION_ZOTERO_MIN_VERSION}\",\"strict_max_version\":\"{ZOTERO_COMPANION_ZOTERO_MAX_VERSION}\"}}}}}}"
        );
        let bytes = [
            readme.as_slice(),
            bootstrap.as_slice(),
            bridge.as_slice(),
            manifest.as_bytes(),
        ];
        let entries = ZOTERO_COMPANION_SOURCE_PATHS
            .into_iter()
            .zip(bytes)
            .map(|(path, bytes)| ZoteroCompanionSourceEntry { path, bytes })
            .collect::<Vec<_>>();
        compose_zotero_companion_artifact(&entries).unwrap()
    }

    fn fixture_root(name: &str) -> PathBuf {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qiongli-zotero-companion-stage-tests")
            .join(format!(
                "{name}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&root).unwrap();
        root.canonicalize().unwrap()
    }

    #[test]
    fn stage_requires_approval_and_is_idempotent() {
        let root = fixture_root("stage");
        let artifact = fixture_artifact();
        let plan = preview_zotero_companion_stage(&root, &artifact).unwrap();
        assert_eq!(plan.effect(), ZoteroCompanionStageEffect::Stage);
        assert_eq!(
            apply_zotero_companion_stage(&plan, plan.plan_digest_sha256(), false).err(),
            Some(ZoteroCompanionStageError::ApprovalRequired)
        );
        let staged = apply_zotero_companion_stage(&plan, plan.plan_digest_sha256(), true).unwrap();
        assert_eq!(
            staged.receipt().xpi_sha256,
            artifact.manifest().artifact_sha256
        );
        let next = preview_zotero_companion_stage(&root, &artifact).unwrap();
        assert_eq!(next.effect(), ZoteroCompanionStageEffect::AlreadyCurrent);
        assert_eq!(
            apply_zotero_companion_stage(&next, next.plan_digest_sha256(), false)
                .unwrap()
                .xpi_path(),
            staged.xpi_path()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stage_rejects_receipt_or_artifact_drift() {
        let root = fixture_root("drift");
        let artifact = fixture_artifact();
        let plan = preview_zotero_companion_stage(&root, &artifact).unwrap();
        let staged = apply_zotero_companion_stage(&plan, plan.plan_digest_sha256(), true).unwrap();
        fs::write(staged.xpi_path(), b"changed").unwrap();
        assert_eq!(
            verify_zotero_companion_stage(&root, &artifact).err(),
            Some(ZoteroCompanionStageError::StageDrift)
        );
        fs::remove_dir_all(root).unwrap();
    }
}
