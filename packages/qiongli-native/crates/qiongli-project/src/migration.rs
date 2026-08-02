use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ProjectError;
use crate::model::{
    ArticleProjectManifestV1, MAX_SEMANTIC_REVISION, MissingContinuityArtifact, ProjectId,
    ProjectKind, ProjectStage, valid_lower_hex,
};
use crate::portable::{
    PortableProjectEntryV1, canonical_digest, commit_staging, copy_inventory,
    create_staging_directory, ensure_private_subdirectories, migration_inventory, path_digest,
    write_private_file,
};
use crate::storage::{
    ProjectRegistrationJournalLock, project_root_label, read_manifest,
    read_private_project_metadata, semantic_digest, semantic_digest_for_project,
    validate_create_project_root, validate_existing_project_root, write_manifest,
    write_private_project_metadata_once_locked,
};

pub const PROJECT_MIGRATION_SCHEMA_VERSION: u32 = 2;
pub const PROJECT_MIGRATION_DOCUMENT_KIND: &str = "qiongli-project-migration";
const MIGRATION_RECEIPT_RELATIVE_PATH: &str = ".qiongli/v2/project-migration.json";
const MIGRATION_REGISTRATION_RELATIVE_PATH: &str = ".qiongli/v2/project-migration-registered.json";
const MIGRATION_REGISTRATION_DOCUMENT_KIND: &str = "qiongli-project-migration-registration";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub project_id: ProjectId,
    pub manifest_created_at_unix: u64,
    pub display_name: String,
    pub project_kind: ProjectKind,
    pub stage: ProjectStage,
    pub source_label: String,
    pub destination_label: String,
    pub copied_file_count: usize,
    pub copied_bytes: u64,
    pub excluded_entry_count: usize,
    pub expected_library_revision: u64,
    pub missing_continuity_artifacts: Vec<MissingContinuityArtifact>,
    pub source_retained: bool,
    pub approvals_required: Vec<String>,
}

#[derive(Clone)]
pub struct VerifiedProjectMigration {
    preview: ProjectMigrationPreviewV1,
    source: PathBuf,
    destination: PathBuf,
    source_reference_digest: String,
    destination_reference_digest: String,
    manifest: ArticleProjectManifestV1,
    inventory: Vec<PortableProjectEntryV1>,
}

pub(crate) struct CommittedProjectMigration {
    pub(crate) accepted_at_unix: u64,
    pub(crate) manifest: ArticleProjectManifestV1,
    pub(crate) manifest_digest: String,
}

impl VerifiedProjectMigration {
    #[must_use]
    pub const fn preview(&self) -> &ProjectMigrationPreviewV1 {
        &self.preview
    }

    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }
}

impl Debug for VerifiedProjectMigration {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedProjectMigration")
            .field("preview", &self.preview)
            .field("source", &"<legacy-project-root>")
            .field("destination", &"<migrated-project-root>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationCommitV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub library_revision: u64,
    pub copied_file_count: usize,
    pub copied_bytes: u64,
    pub excluded_entry_count: usize,
    pub destination_label: String,
    pub source_retained: bool,
    pub migration_receipt: String,
    pub index_rebuild_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationRecoveryPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub project_id: ProjectId,
    pub display_name: String,
    pub source_label: String,
    pub destination_label: String,
    pub copied_file_count: usize,
    pub copied_bytes: u64,
    pub excluded_entry_count: usize,
    pub source_retained: bool,
    pub approvals_required: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMigrationArtifactCategory {
    ResearchState,
    Decisions,
    Evidence,
    Captures,
    SemanticLinks,
    Continuity,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMigrationArtifactState {
    Matched,
    NotPresent,
    MissingAtDestination,
    DestinationOnly,
    Changed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationArtifactReconciliationV1 {
    pub category: ProjectMigrationArtifactCategory,
    pub relative_path: String,
    pub state: ProjectMigrationArtifactState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMigrationReconciliationStatus {
    Matched,
    MatchedWithGaps,
    Drifted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationReconciliationV1 {
    pub status: ProjectMigrationReconciliationStatus,
    pub matched_artifact_count: usize,
    pub drifted_artifact_count: usize,
    pub continuity_gap_count: usize,
    pub artifacts: Vec<ProjectMigrationArtifactReconciliationV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMigrationRegistrationState {
    Registered,
    Unregistered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMigrationMarkerState {
    Ready,
    Missing,
    Conflicting,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationRollbackPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub project_id: ProjectId,
    pub destination_label: String,
    pub expected_library_revision: u64,
    pub registration_state: ProjectMigrationRegistrationState,
    pub marker_state: ProjectMigrationMarkerState,
    pub reconciliation: ProjectMigrationReconciliationV1,
    pub source_retained: bool,
    pub destination_removal: String,
    pub can_rollback: bool,
    pub blocked_reason: Option<String>,
    pub approvals_required: Vec<String>,
}

#[derive(Clone)]
pub struct VerifiedProjectMigrationRollback {
    preview: ProjectMigrationRollbackPreviewV1,
    source: PathBuf,
    destination: PathBuf,
    source_reference_digest: String,
    destination_reference_digest: String,
    receipt: ProjectMigrationReceiptV1,
    source_inventory: Vec<PortableProjectEntryV1>,
    destination_inventory: Vec<PortableProjectEntryV1>,
    manifest_digest: Option<String>,
}

impl VerifiedProjectMigrationRollback {
    #[must_use]
    pub const fn preview(&self) -> &ProjectMigrationRollbackPreviewV1 {
        &self.preview
    }

    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }
}

impl Debug for VerifiedProjectMigrationRollback {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedProjectMigrationRollback")
            .field("preview", &self.preview)
            .field("source", &"<legacy-project-root>")
            .field("destination", &"<migration-owned-root>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationRollbackCommitV1 {
    pub schema_version: u32,
    pub project_id: ProjectId,
    pub library_revision: u64,
    pub destination_label: String,
    pub removed_artifact_count: usize,
    pub source_retained: bool,
    pub destination_removed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectMigrationDoctorStatus {
    Ready,
    Attention,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationDoctorV1 {
    pub project_id: ProjectId,
    pub status: ProjectMigrationDoctorStatus,
    pub receipt_state: String,
    pub registration_marker_state: ProjectMigrationMarkerState,
    pub derived_index_state: String,
    pub next_actions: Vec<String>,
}

#[derive(Clone)]
pub struct VerifiedProjectMigrationRecovery {
    preview: ProjectMigrationRecoveryPreviewV1,
    source: PathBuf,
    destination: PathBuf,
    source_reference_digest: String,
    destination_reference_digest: String,
    manifest: ArticleProjectManifestV1,
    manifest_digest: String,
    inventory: Vec<PortableProjectEntryV1>,
    receipt: ProjectMigrationReceiptV1,
}

impl VerifiedProjectMigrationRecovery {
    #[must_use]
    pub const fn preview(&self) -> &ProjectMigrationRecoveryPreviewV1 {
        &self.preview
    }

    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }

    pub(crate) const fn manifest(&self) -> &ArticleProjectManifestV1 {
        &self.manifest
    }

    pub(crate) fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }

    pub(crate) const fn accepted_at_unix(&self) -> u64 {
        self.receipt.accepted_at_unix
    }

    pub(crate) const fn expected_library_revision(&self) -> u64 {
        self.receipt.expected_library_revision
    }
}

impl Debug for VerifiedProjectMigrationRecovery {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedProjectMigrationRecovery")
            .field("preview", &self.preview)
            .field("source", &"<legacy-project-root>")
            .field("destination", &"<migrated-project-root>")
            .finish()
    }
}

#[derive(Serialize)]
struct ProjectMigrationPlanSemantics<'a> {
    schema_version: u32,
    project_id: &'a ProjectId,
    manifest_digest: String,
    display_name: &'a str,
    project_kind: ProjectKind,
    stage: ProjectStage,
    semantic_digest: &'a str,
    inventory_digest: String,
    excluded_entry_count: usize,
    source_reference_digest: &'a str,
    destination_reference_digest: &'a str,
    expected_library_revision: u64,
}

#[derive(Serialize)]
struct ProjectMigrationRollbackSemantics<'a> {
    schema_version: u32,
    project_id: &'a ProjectId,
    migration_plan_digest: &'a str,
    source_reference_digest: &'a str,
    destination_reference_digest: &'a str,
    source_inventory_digest: String,
    destination_inventory_digest: String,
    manifest_digest: &'a Option<String>,
    expected_library_revision: u64,
    registration_state: ProjectMigrationRegistrationState,
    marker_state: ProjectMigrationMarkerState,
    reconciliation: &'a ProjectMigrationReconciliationV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ProjectMigrationReceiptV1 {
    schema_version: u32,
    document_kind: String,
    project_id: ProjectId,
    plan_digest: String,
    copied_file_count: usize,
    copied_bytes: u64,
    excluded_entry_count: usize,
    source_inventory_sha256: String,
    manifest_sha256: String,
    source_reference_sha256: String,
    destination_reference_sha256: String,
    expected_library_revision: u64,
    source_retained: bool,
    accepted_at_unix: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ProjectMigrationRegistrationV1 {
    schema_version: u32,
    document_kind: String,
    project_id: ProjectId,
    plan_digest: String,
}

pub(crate) fn preview_migration(
    source: &Path,
    destination: &Path,
    manifest: ArticleProjectManifestV1,
    expected_library_revision: u64,
    missing_continuity_artifacts: Vec<MissingContinuityArtifact>,
) -> Result<VerifiedProjectMigration, ProjectError> {
    validate_existing_project_root(source)?;
    validate_create_project_root(destination)?;
    reject_nested_destination(source, destination)?;
    if read_manifest(source)?.is_some() {
        return Err(ProjectError::MigrationSourceInvalid);
    }
    if semantic_digest_for_project(source, &manifest.project_id)? != manifest.semantic_digest {
        return Err(ProjectError::RevisionConflict);
    }
    let (inventory, excluded_entry_count) = migration_inventory(source)?;
    if inventory.is_empty() {
        return Err(ProjectError::MigrationSourceInvalid);
    }
    let copied_bytes = inventory.iter().map(|entry| entry.size_bytes).sum();
    let source_reference_digest = path_digest(source)?;
    let destination_reference_digest = path_digest(destination)?;
    let semantics = ProjectMigrationPlanSemantics {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        project_id: &manifest.project_id,
        manifest_digest: canonical_digest(&manifest)?,
        display_name: &manifest.display_name,
        project_kind: manifest.project_kind,
        stage: manifest.stage,
        semantic_digest: &manifest.semantic_digest,
        inventory_digest: canonical_digest(&inventory)?,
        excluded_entry_count,
        source_reference_digest: &source_reference_digest,
        destination_reference_digest: &destination_reference_digest,
        expected_library_revision,
    };
    let preview = ProjectMigrationPreviewV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        plan_digest: canonical_digest(&semantics)?,
        project_id: manifest.project_id.clone(),
        manifest_created_at_unix: manifest.created_at_unix,
        display_name: manifest.display_name.clone(),
        project_kind: manifest.project_kind,
        stage: manifest.stage,
        source_label: project_root_label(source),
        destination_label: project_root_label(destination),
        copied_file_count: inventory.len(),
        copied_bytes,
        excluded_entry_count,
        expected_library_revision,
        missing_continuity_artifacts,
        source_retained: true,
        approvals_required: vec!["filesystem-write".to_string()],
    };
    Ok(VerifiedProjectMigration {
        preview,
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
        source_reference_digest,
        destination_reference_digest,
        manifest,
        inventory,
    })
}

pub(crate) fn apply_migration_files(
    plan: &VerifiedProjectMigration,
    now_unix: u64,
) -> Result<(), ProjectError> {
    validate_plan_paths(plan)?;
    validate_existing_project_root(&plan.source)?;
    validate_create_project_root(&plan.destination)?;
    if read_manifest(&plan.source)?.is_some() {
        return Err(ProjectError::RevisionConflict);
    }
    let (inventory, excluded_entry_count) = migration_inventory(&plan.source)?;
    if inventory != plan.inventory
        || excluded_entry_count != plan.preview.excluded_entry_count
        || semantic_digest_for_project(&plan.source, &plan.manifest.project_id)?
            != plan.manifest.semantic_digest
    {
        return Err(ProjectError::RevisionConflict);
    }

    let staging = create_staging_directory(&plan.destination)?;
    let result = (|| {
        copy_inventory(&plan.source, &staging, &plan.inventory)?;
        write_manifest(&staging, &plan.manifest, None)?;
        if semantic_digest(&staging)? != plan.manifest.semantic_digest {
            return Err(ProjectError::RevisionConflict);
        }
        let receipt = ProjectMigrationReceiptV1 {
            schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
            document_kind: PROJECT_MIGRATION_DOCUMENT_KIND.to_string(),
            project_id: plan.manifest.project_id.clone(),
            plan_digest: plan.preview.plan_digest.clone(),
            copied_file_count: plan.preview.copied_file_count,
            copied_bytes: plan.preview.copied_bytes,
            excluded_entry_count: plan.preview.excluded_entry_count,
            source_inventory_sha256: canonical_digest(&plan.inventory)?,
            manifest_sha256: canonical_digest(&plan.manifest)?,
            source_reference_sha256: plan.source_reference_digest.clone(),
            destination_reference_sha256: plan.destination_reference_digest.clone(),
            expected_library_revision: plan.preview.expected_library_revision,
            source_retained: true,
            accepted_at_unix: now_unix,
        };
        let receipt_bytes = serde_json_canonicalizer::to_vec(&receipt)
            .map_err(|_| ProjectError::MigrationSourceInvalid)?;
        let receipt_path = staging.join(MIGRATION_RECEIPT_RELATIVE_PATH);
        ensure_private_subdirectories(
            &staging,
            receipt_path
                .parent()
                .ok_or(ProjectError::MigrationSourceInvalid)?,
        )?;
        write_private_file(&receipt_path, &receipt_bytes)?;
        commit_staging(&staging, &plan.destination)
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

pub(crate) fn ensure_migration_files(
    plan: &VerifiedProjectMigration,
    now_unix: u64,
) -> Result<CommittedProjectMigration, ProjectError> {
    validate_plan_paths(plan)?;
    if let Some(committed) = committed_migration(plan)? {
        return Ok(committed);
    }

    let apply_result = apply_migration_files(plan, now_unix);
    match committed_migration(plan)? {
        Some(committed) => Ok(committed),
        None => match apply_result {
            Ok(()) => Err(ProjectError::RecoveryRequired),
            Err(error) => Err(error),
        },
    }
}

pub(crate) fn committed_migration(
    plan: &VerifiedProjectMigration,
) -> Result<Option<CommittedProjectMigration>, ProjectError> {
    match validate_existing_project_root(&plan.destination) {
        Ok(()) => {}
        Err(ProjectError::ProjectRootMissing) => return Ok(None),
        Err(error) => return Err(error),
    }

    let receipt = migration_receipt(plan)?.ok_or(ProjectError::RecoveryRequired)?;

    let (manifest, manifest_digest) = read_manifest(&plan.destination)?
        .filter(|(manifest, _)| manifest == &plan.manifest)
        .ok_or(ProjectError::RecoveryRequired)?;
    if semantic_digest(&plan.destination)? != manifest.semantic_digest {
        return Err(ProjectError::RecoveryRequired);
    }

    let (mut inventory, excluded) = migration_inventory(&plan.destination)?;
    if excluded != 1 {
        return Err(ProjectError::RecoveryRequired);
    }
    let manifest_position = inventory
        .iter()
        .position(|entry| entry.relative_path == "context/project_manifest.json")
        .ok_or(ProjectError::RecoveryRequired)?;
    let manifest_entry = inventory.remove(manifest_position);
    let manifest_bytes = serde_json_canonicalizer::to_vec(&plan.manifest)
        .map_err(|_| ProjectError::RecoveryRequired)?;
    if manifest_entry.size_bytes != manifest_bytes.len() as u64
        || manifest_entry.sha256 != canonical_digest(&plan.manifest)?
        || inventory != plan.inventory
    {
        return Err(ProjectError::RecoveryRequired);
    }

    Ok(Some(CommittedProjectMigration {
        accepted_at_unix: receipt,
        manifest,
        manifest_digest,
    }))
}

pub(crate) fn migration_receipt(
    plan: &VerifiedProjectMigration,
) -> Result<Option<u64>, ProjectError> {
    match validate_existing_project_root(&plan.destination) {
        Ok(()) => {}
        Err(ProjectError::ProjectRootMissing) => return Ok(None),
        Err(error) => return Err(error),
    }
    let Some(receipt) = read_migration_receipt(&plan.destination)? else {
        return Ok(None);
    };
    if receipt.project_id != plan.preview.project_id
        || receipt.plan_digest != plan.preview.plan_digest
        || receipt.copied_file_count != plan.preview.copied_file_count
        || receipt.copied_bytes != plan.preview.copied_bytes
        || receipt.excluded_entry_count != plan.preview.excluded_entry_count
        || receipt.source_inventory_sha256 != canonical_digest(&plan.inventory)?
        || receipt.manifest_sha256 != canonical_digest(&plan.manifest)?
        || receipt.source_reference_sha256 != plan.source_reference_digest
        || receipt.destination_reference_sha256 != plan.destination_reference_digest
        || receipt.expected_library_revision != plan.preview.expected_library_revision
    {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(Some(receipt.accepted_at_unix))
}

pub(crate) fn migration_registration_completed(
    plan: &VerifiedProjectMigration,
) -> Result<bool, ProjectError> {
    match validate_existing_project_root(&plan.destination) {
        Ok(()) => {}
        Err(ProjectError::ProjectRootMissing) => return Ok(false),
        Err(error) => return Err(error),
    }
    let Some(registration) = read_migration_registration(&plan.destination)? else {
        return Ok(false);
    };
    if registration.project_id != plan.preview.project_id
        || registration.plan_digest != plan.preview.plan_digest
    {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(true)
}

pub(crate) fn complete_migration_registration_locked(
    plan: &VerifiedProjectMigration,
    library_revision: u64,
    lock: &ProjectRegistrationJournalLock,
) -> Result<(), ProjectError> {
    migration_receipt(plan)?.ok_or(ProjectError::RecoveryRequired)?;
    if migration_registration_completed(plan)? {
        return Ok(());
    }
    if library_revision <= plan.preview.expected_library_revision
        || library_revision > MAX_SEMANTIC_REVISION
    {
        return Err(ProjectError::RecoveryRequired);
    }
    write_migration_registration_locked(
        &plan.destination,
        &plan.preview.project_id,
        &plan.preview.plan_digest,
        lock,
    )
}

pub(crate) fn finalize_migration_registration_before_unregister(
    root: &Path,
    project_id: &ProjectId,
    lock: &ProjectRegistrationJournalLock,
) -> Result<(), ProjectError> {
    let Some(receipt_bytes) = read_private_project_metadata(root, MIGRATION_RECEIPT_RELATIVE_PATH)?
    else {
        return Ok(());
    };
    let receipt = decode_migration_receipt(&receipt_bytes)?;
    if &receipt.project_id != project_id {
        return Err(ProjectError::RecoveryRequired);
    }
    write_migration_registration_locked(root, &receipt.project_id, &receipt.plan_digest, lock)
}

pub(crate) fn preview_migration_recovery(
    source: &Path,
    destination: &Path,
) -> Result<VerifiedProjectMigrationRecovery, ProjectError> {
    validate_existing_project_root(source)?;
    validate_existing_project_root(destination)?;
    reject_nested_destination(source, destination)?;
    if read_manifest(source)?.is_some() {
        return Err(ProjectError::MigrationSourceInvalid);
    }
    let receipt = read_migration_receipt(destination)?.ok_or(ProjectError::RecoveryRequired)?;
    let (manifest, manifest_digest) =
        read_manifest(destination)?.ok_or(ProjectError::RecoveryRequired)?;
    if receipt.project_id != manifest.project_id
        || receipt.manifest_sha256 != manifest_digest
        || semantic_digest(destination)? != manifest.semantic_digest
        || semantic_digest_for_project(source, &manifest.project_id)? != manifest.semantic_digest
    {
        return Err(ProjectError::RecoveryRequired);
    }
    let source_reference_digest = path_digest(source)?;
    let destination_reference_digest = path_digest(destination)?;
    if receipt.source_reference_sha256 != source_reference_digest
        || receipt.destination_reference_sha256 != destination_reference_digest
    {
        return Err(ProjectError::PlanMismatch);
    }
    let (inventory, excluded_entry_count) = migration_inventory(source)?;
    if inventory.is_empty()
        || canonical_digest(&inventory)? != receipt.source_inventory_sha256
        || inventory.len() != receipt.copied_file_count
        || inventory.iter().map(|entry| entry.size_bytes).sum::<u64>() != receipt.copied_bytes
        || excluded_entry_count != receipt.excluded_entry_count
    {
        return Err(ProjectError::RevisionConflict);
    }
    validate_recovered_destination(destination, &manifest, &inventory)?;
    if let Some(registration) = read_migration_registration(destination)?
        && (registration.project_id != receipt.project_id
            || registration.plan_digest != receipt.plan_digest)
    {
        return Err(ProjectError::RecoveryRequired);
    }
    let preview = ProjectMigrationRecoveryPreviewV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        plan_digest: receipt.plan_digest.clone(),
        project_id: receipt.project_id.clone(),
        display_name: manifest.display_name.clone(),
        source_label: project_root_label(source),
        destination_label: project_root_label(destination),
        copied_file_count: receipt.copied_file_count,
        copied_bytes: receipt.copied_bytes,
        excluded_entry_count: receipt.excluded_entry_count,
        source_retained: true,
        approvals_required: vec!["filesystem-write".to_string()],
    };
    Ok(VerifiedProjectMigrationRecovery {
        preview,
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
        source_reference_digest,
        destination_reference_digest,
        manifest,
        manifest_digest,
        inventory,
        receipt,
    })
}

pub(crate) fn validate_migration_recovery(
    plan: &VerifiedProjectMigrationRecovery,
) -> Result<(), ProjectError> {
    let current = preview_migration_recovery(&plan.source, &plan.destination)?;
    if current.preview != plan.preview
        || current.source_reference_digest != plan.source_reference_digest
        || current.destination_reference_digest != plan.destination_reference_digest
        || current.manifest != plan.manifest
        || current.manifest_digest != plan.manifest_digest
        || current.inventory != plan.inventory
        || current.receipt != plan.receipt
    {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(())
}

pub(crate) fn complete_migration_recovery_registration_locked(
    plan: &VerifiedProjectMigrationRecovery,
    library_revision: u64,
    lock: &ProjectRegistrationJournalLock,
) -> Result<(), ProjectError> {
    validate_migration_recovery(plan)?;
    if library_revision <= plan.receipt.expected_library_revision
        || library_revision > MAX_SEMANTIC_REVISION
    {
        return Err(ProjectError::RecoveryRequired);
    }
    write_migration_registration_locked(
        &plan.destination,
        &plan.preview.project_id,
        &plan.preview.plan_digest,
        lock,
    )
}

pub(crate) fn migration_recovery_commit(
    plan: &VerifiedProjectMigrationRecovery,
    library_revision: u64,
) -> ProjectMigrationCommitV1 {
    ProjectMigrationCommitV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        project_id: plan.preview.project_id.clone(),
        library_revision,
        copied_file_count: plan.preview.copied_file_count,
        copied_bytes: plan.preview.copied_bytes,
        excluded_entry_count: plan.preview.excluded_entry_count,
        destination_label: plan.preview.destination_label.clone(),
        source_retained: true,
        migration_receipt: MIGRATION_RECEIPT_RELATIVE_PATH.to_string(),
        index_rebuild_required: true,
    }
}

pub(crate) fn migration_commit(
    plan: &VerifiedProjectMigration,
    library_revision: u64,
) -> ProjectMigrationCommitV1 {
    ProjectMigrationCommitV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        project_id: plan.preview.project_id.clone(),
        library_revision,
        copied_file_count: plan.preview.copied_file_count,
        copied_bytes: plan.preview.copied_bytes,
        excluded_entry_count: plan.preview.excluded_entry_count,
        destination_label: plan.preview.destination_label.clone(),
        source_retained: true,
        migration_receipt: MIGRATION_RECEIPT_RELATIVE_PATH.to_string(),
        index_rebuild_required: true,
    }
}

pub(crate) fn preview_migration_rollback(
    source: &Path,
    destination: &Path,
    expected_library_revision: u64,
    registration_state: ProjectMigrationRegistrationState,
) -> Result<VerifiedProjectMigrationRollback, ProjectError> {
    validate_existing_project_root(source)?;
    validate_existing_project_root(destination)?;
    reject_nested_destination(source, destination)?;
    if read_manifest(source)?.is_some() {
        return Err(ProjectError::MigrationSourceInvalid);
    }
    let receipt = read_migration_receipt(destination)?.ok_or(ProjectError::RecoveryRequired)?;
    let source_reference_digest = path_digest(source)?;
    let destination_reference_digest = path_digest(destination)?;
    if receipt.source_reference_sha256 != source_reference_digest
        || receipt.destination_reference_sha256 != destination_reference_digest
    {
        return Err(ProjectError::PlanMismatch);
    }

    let (source_inventory, source_excluded) = migration_inventory(source)?;
    let (mut destination_inventory, destination_excluded) = migration_inventory(destination)?;
    let manifest_position = destination_inventory
        .iter()
        .position(|entry| entry.relative_path == "context/project_manifest.json");
    let manifest_digest =
        manifest_position.map(|position| destination_inventory.remove(position).sha256);
    let source_inventory_matches_receipt = !source_inventory.is_empty()
        && canonical_digest(&source_inventory)? == receipt.source_inventory_sha256
        && source_inventory.len() == receipt.copied_file_count
        && source_inventory
            .iter()
            .map(|entry| entry.size_bytes)
            .sum::<u64>()
            == receipt.copied_bytes
        && source_excluded == receipt.excluded_entry_count;
    let manifest_matches_receipt = manifest_digest.as_deref() == Some(&receipt.manifest_sha256);
    let destination_matches_source = destination_inventory == source_inventory;
    let marker_state = match read_migration_registration(destination) {
        Ok(Some(marker))
            if marker.project_id == receipt.project_id
                && marker.plan_digest == receipt.plan_digest =>
        {
            ProjectMigrationMarkerState::Ready
        }
        Ok(None) => ProjectMigrationMarkerState::Missing,
        Ok(Some(_)) | Err(_) => ProjectMigrationMarkerState::Conflicting,
    };
    let reconciliation = migration_reconciliation(
        &source_inventory,
        &destination_inventory,
        source_inventory_matches_receipt,
        manifest_matches_receipt,
        destination_excluded,
    );
    let blocked_reason = if !source_inventory_matches_receipt {
        Some("project-migration-rollback-source-drift")
    } else if !manifest_matches_receipt
        || !destination_matches_source
        || destination_excluded != 1
        || reconciliation.drifted_artifact_count > 0
    {
        Some("project-migration-rollback-destination-drift")
    } else if marker_state == ProjectMigrationMarkerState::Conflicting {
        Some("project-migration-rollback-marker-conflict")
    } else {
        None
    };
    let can_rollback = blocked_reason.is_none();
    let semantics = ProjectMigrationRollbackSemantics {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        project_id: &receipt.project_id,
        migration_plan_digest: &receipt.plan_digest,
        source_reference_digest: &source_reference_digest,
        destination_reference_digest: &destination_reference_digest,
        source_inventory_digest: canonical_digest(&source_inventory)?,
        destination_inventory_digest: canonical_digest(&destination_inventory)?,
        manifest_digest: &manifest_digest,
        expected_library_revision,
        registration_state,
        marker_state,
        reconciliation: &reconciliation,
    };
    let preview = ProjectMigrationRollbackPreviewV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        plan_digest: canonical_digest(&semantics)?,
        project_id: receipt.project_id.clone(),
        destination_label: project_root_label(destination),
        expected_library_revision,
        registration_state,
        marker_state,
        reconciliation,
        source_retained: true,
        destination_removal: "exact-migration-owned-root".to_string(),
        can_rollback,
        blocked_reason: blocked_reason.map(str::to_string),
        approvals_required: if can_rollback {
            vec!["filesystem-write".to_string()]
        } else {
            Vec::new()
        },
    };
    Ok(VerifiedProjectMigrationRollback {
        preview,
        source: source.to_path_buf(),
        destination: destination.to_path_buf(),
        source_reference_digest,
        destination_reference_digest,
        receipt,
        source_inventory,
        destination_inventory,
        manifest_digest,
    })
}

pub(crate) fn validate_migration_rollback(
    plan: &VerifiedProjectMigrationRollback,
    expected_library_revision: u64,
    registration_state: ProjectMigrationRegistrationState,
) -> Result<(), ProjectError> {
    let current = preview_migration_rollback(
        &plan.source,
        &plan.destination,
        expected_library_revision,
        registration_state,
    )?;
    if current.preview != plan.preview
        || current.source_reference_digest != plan.source_reference_digest
        || current.destination_reference_digest != plan.destination_reference_digest
        || current.receipt != plan.receipt
        || current.source_inventory != plan.source_inventory
        || current.destination_inventory != plan.destination_inventory
        || current.manifest_digest != plan.manifest_digest
    {
        return Err(ProjectError::RevisionConflict);
    }
    if !plan.preview.can_rollback {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(())
}

pub(crate) fn remove_migration_destination(
    plan: &VerifiedProjectMigrationRollback,
) -> Result<(), ProjectError> {
    let current = preview_migration_rollback(
        &plan.source,
        &plan.destination,
        plan.preview.expected_library_revision,
        plan.preview.registration_state,
    )?;
    if current.source_reference_digest != plan.source_reference_digest
        || current.destination_reference_digest != plan.destination_reference_digest
        || current.receipt != plan.receipt
        || current.source_inventory != plan.source_inventory
        || current.destination_inventory != plan.destination_inventory
        || current.manifest_digest != plan.manifest_digest
        || !current.preview.can_rollback
    {
        return Err(ProjectError::RevisionConflict);
    }
    fs::remove_dir_all(&plan.destination)
        .map_err(|error| ProjectError::PersistenceFailed(error.kind()))?;
    if plan.destination.exists() {
        return Err(ProjectError::RecoveryRequired);
    }
    validate_existing_project_root(&plan.source)?;
    Ok(())
}

pub(crate) fn migration_rollback_commit(
    plan: &VerifiedProjectMigrationRollback,
    library_revision: u64,
) -> ProjectMigrationRollbackCommitV1 {
    ProjectMigrationRollbackCommitV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        project_id: plan.preview.project_id.clone(),
        library_revision,
        destination_label: plan.preview.destination_label.clone(),
        removed_artifact_count: plan.destination_inventory.len().saturating_add(
            if plan.preview.marker_state == ProjectMigrationMarkerState::Ready {
                3
            } else {
                2
            },
        ),
        source_retained: true,
        destination_removed: true,
    }
}

pub(crate) fn inspect_migration_doctor(
    root: &Path,
    project_id: &ProjectId,
) -> Result<Option<ProjectMigrationDoctorV1>, ProjectError> {
    let Some(receipt_bytes) = read_private_project_metadata(root, MIGRATION_RECEIPT_RELATIVE_PATH)?
    else {
        return Ok(None);
    };
    let receipt = match decode_migration_receipt(&receipt_bytes) {
        Ok(receipt) if &receipt.project_id == project_id => receipt,
        Ok(_) | Err(_) => {
            return Ok(Some(ProjectMigrationDoctorV1 {
                project_id: project_id.clone(),
                status: ProjectMigrationDoctorStatus::Attention,
                receipt_state: "conflicting".to_string(),
                registration_marker_state: ProjectMigrationMarkerState::Conflicting,
                derived_index_state: "rebuild-on-open".to_string(),
                next_actions: vec![
                    "select-source-and-destination-for-reconciliation".to_string(),
                    "export-or-resolve-before-rollback".to_string(),
                ],
            }));
        }
    };
    let marker_state = match read_migration_registration(root) {
        Ok(Some(marker))
            if marker.project_id == receipt.project_id
                && marker.plan_digest == receipt.plan_digest =>
        {
            ProjectMigrationMarkerState::Ready
        }
        Ok(None) => ProjectMigrationMarkerState::Missing,
        Ok(Some(_)) | Err(_) => ProjectMigrationMarkerState::Conflicting,
    };
    let mut next_actions = vec!["run-project-graph-doctor".to_string()];
    if marker_state == ProjectMigrationMarkerState::Missing {
        next_actions.insert(0, "resume-migration-registration".to_string());
    } else if marker_state == ProjectMigrationMarkerState::Conflicting {
        next_actions.insert(
            0,
            "select-source-and-destination-for-reconciliation".to_string(),
        );
    }
    Ok(Some(ProjectMigrationDoctorV1 {
        project_id: project_id.clone(),
        status: if marker_state == ProjectMigrationMarkerState::Ready {
            ProjectMigrationDoctorStatus::Ready
        } else {
            ProjectMigrationDoctorStatus::Attention
        },
        receipt_state: "ready".to_string(),
        registration_marker_state: marker_state,
        derived_index_state: "rebuild-on-open".to_string(),
        next_actions,
    }))
}

fn migration_reconciliation(
    source: &[PortableProjectEntryV1],
    destination: &[PortableProjectEntryV1],
    source_inventory_matches_receipt: bool,
    manifest_matches_receipt: bool,
    destination_excluded: usize,
) -> ProjectMigrationReconciliationV1 {
    let source_by_path = source
        .iter()
        .map(|entry| (entry.relative_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let destination_by_path = destination
        .iter()
        .map(|entry| (entry.relative_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let paths = source_by_path
        .keys()
        .chain(destination_by_path.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut artifacts = paths
        .into_iter()
        .map(|relative_path| {
            let state = match (
                source_by_path.get(relative_path),
                destination_by_path.get(relative_path),
            ) {
                (Some(source), Some(destination)) if source == destination => {
                    ProjectMigrationArtifactState::Matched
                }
                (Some(_), Some(_)) => ProjectMigrationArtifactState::Changed,
                (Some(_), None) => ProjectMigrationArtifactState::MissingAtDestination,
                (None, Some(_)) => ProjectMigrationArtifactState::DestinationOnly,
                (None, None) => ProjectMigrationArtifactState::NotPresent,
            };
            ProjectMigrationArtifactReconciliationV1 {
                category: migration_artifact_category(relative_path),
                relative_path: relative_path.to_string(),
                state,
            }
        })
        .collect::<Vec<_>>();

    let required_continuity = [
        "context/research_state.md",
        "context/decision_log.md",
        "context/stage_handoff.md",
        "literature/literature_map.md",
        "evidence/claim-evidence-ledger.csv",
        "manuscript/claims_evidence_map.md",
    ];
    let mut continuity_gap_count = 0usize;
    for relative_path in required_continuity {
        if !source_by_path.contains_key(relative_path)
            && !destination_by_path.contains_key(relative_path)
        {
            continuity_gap_count = continuity_gap_count.saturating_add(1);
            artifacts.push(ProjectMigrationArtifactReconciliationV1 {
                category: migration_artifact_category(relative_path),
                relative_path: relative_path.to_string(),
                state: ProjectMigrationArtifactState::NotPresent,
            });
        }
    }
    if !artifacts
        .iter()
        .any(|item| item.category == ProjectMigrationArtifactCategory::Captures)
    {
        artifacts.push(ProjectMigrationArtifactReconciliationV1 {
            category: ProjectMigrationArtifactCategory::Captures,
            relative_path: "context/captures/".to_string(),
            state: ProjectMigrationArtifactState::NotPresent,
        });
    }
    if !source_by_path.contains_key("graph/semantic_links.jsonl")
        && !destination_by_path.contains_key("graph/semantic_links.jsonl")
    {
        artifacts.push(ProjectMigrationArtifactReconciliationV1 {
            category: ProjectMigrationArtifactCategory::SemanticLinks,
            relative_path: "graph/semantic_links.jsonl".to_string(),
            state: ProjectMigrationArtifactState::NotPresent,
        });
    }
    if !source_inventory_matches_receipt {
        artifacts.push(ProjectMigrationArtifactReconciliationV1 {
            category: ProjectMigrationArtifactCategory::Other,
            relative_path: "migration-source-inventory".to_string(),
            state: ProjectMigrationArtifactState::Changed,
        });
    }
    if !manifest_matches_receipt {
        artifacts.push(ProjectMigrationArtifactReconciliationV1 {
            category: ProjectMigrationArtifactCategory::Continuity,
            relative_path: "context/project_manifest.json".to_string(),
            state: ProjectMigrationArtifactState::Changed,
        });
    }
    if destination_excluded != 1 {
        artifacts.push(ProjectMigrationArtifactReconciliationV1 {
            category: ProjectMigrationArtifactCategory::Other,
            relative_path: "migration-private-entries".to_string(),
            state: ProjectMigrationArtifactState::DestinationOnly,
        });
    }
    artifacts.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    let matched_artifact_count = artifacts
        .iter()
        .filter(|item| item.state == ProjectMigrationArtifactState::Matched)
        .count();
    let drifted_artifact_count = artifacts
        .iter()
        .filter(|item| {
            matches!(
                item.state,
                ProjectMigrationArtifactState::Changed
                    | ProjectMigrationArtifactState::MissingAtDestination
                    | ProjectMigrationArtifactState::DestinationOnly
            )
        })
        .count();
    ProjectMigrationReconciliationV1 {
        status: if drifted_artifact_count > 0 {
            ProjectMigrationReconciliationStatus::Drifted
        } else if continuity_gap_count > 0 {
            ProjectMigrationReconciliationStatus::MatchedWithGaps
        } else {
            ProjectMigrationReconciliationStatus::Matched
        },
        matched_artifact_count,
        drifted_artifact_count,
        continuity_gap_count,
        artifacts,
    }
}

fn migration_artifact_category(relative_path: &str) -> ProjectMigrationArtifactCategory {
    match relative_path {
        "context/research_state.md" => ProjectMigrationArtifactCategory::ResearchState,
        "context/decision_log.md" => ProjectMigrationArtifactCategory::Decisions,
        "evidence/claim-evidence-ledger.csv" => ProjectMigrationArtifactCategory::Evidence,
        "graph/semantic_links.jsonl" => ProjectMigrationArtifactCategory::SemanticLinks,
        "context/stage_handoff.md"
        | "literature/literature_map.md"
        | "manuscript/claims_evidence_map.md" => ProjectMigrationArtifactCategory::Continuity,
        value
            if value.starts_with("context/captures/")
                || value.starts_with("context/consolidations/") =>
        {
            ProjectMigrationArtifactCategory::Captures
        }
        _ => ProjectMigrationArtifactCategory::Other,
    }
}

fn validate_plan_paths(plan: &VerifiedProjectMigration) -> Result<(), ProjectError> {
    if path_digest(&plan.source)? != plan.source_reference_digest
        || path_digest(&plan.destination)? != plan.destination_reference_digest
    {
        return Err(ProjectError::PlanMismatch);
    }
    Ok(())
}

fn reject_nested_destination(source: &Path, destination: &Path) -> Result<(), ProjectError> {
    if destination.starts_with(source) || source.starts_with(destination) {
        return Err(ProjectError::InvalidProjectRoot);
    }
    Ok(())
}

fn read_migration_receipt(root: &Path) -> Result<Option<ProjectMigrationReceiptV1>, ProjectError> {
    read_private_project_metadata(root, MIGRATION_RECEIPT_RELATIVE_PATH)?
        .map(|bytes| decode_migration_receipt(&bytes))
        .transpose()
}

fn decode_migration_receipt(bytes: &[u8]) -> Result<ProjectMigrationReceiptV1, ProjectError> {
    let value =
        crate::json::parse_unique_json(bytes).map_err(|_| ProjectError::RecoveryRequired)?;
    let receipt: ProjectMigrationReceiptV1 =
        serde_json::from_value(value).map_err(|_| ProjectError::RecoveryRequired)?;
    if receipt.schema_version != PROJECT_MIGRATION_SCHEMA_VERSION
        || receipt.document_kind != PROJECT_MIGRATION_DOCUMENT_KIND
        || !valid_lower_hex(&receipt.plan_digest, 64)
        || receipt.copied_file_count == 0
        || !valid_lower_hex(&receipt.source_inventory_sha256, 64)
        || !valid_lower_hex(&receipt.manifest_sha256, 64)
        || !valid_lower_hex(&receipt.source_reference_sha256, 64)
        || !valid_lower_hex(&receipt.destination_reference_sha256, 64)
        || receipt.expected_library_revision > MAX_SEMANTIC_REVISION
        || !receipt.source_retained
        || receipt.accepted_at_unix > MAX_SEMANTIC_REVISION
    {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(receipt)
}

fn read_migration_registration(
    root: &Path,
) -> Result<Option<ProjectMigrationRegistrationV1>, ProjectError> {
    let Some(bytes) = read_private_project_metadata(root, MIGRATION_REGISTRATION_RELATIVE_PATH)?
    else {
        return Ok(None);
    };
    let value =
        crate::json::parse_unique_json(&bytes).map_err(|_| ProjectError::RecoveryRequired)?;
    let registration: ProjectMigrationRegistrationV1 =
        serde_json::from_value(value).map_err(|_| ProjectError::RecoveryRequired)?;
    if registration.schema_version != PROJECT_MIGRATION_SCHEMA_VERSION
        || registration.document_kind != MIGRATION_REGISTRATION_DOCUMENT_KIND
        || !valid_lower_hex(&registration.plan_digest, 64)
    {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(Some(registration))
}

fn write_migration_registration_locked(
    root: &Path,
    project_id: &ProjectId,
    plan_digest: &str,
    lock: &ProjectRegistrationJournalLock,
) -> Result<(), ProjectError> {
    let registration = ProjectMigrationRegistrationV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        document_kind: MIGRATION_REGISTRATION_DOCUMENT_KIND.to_string(),
        project_id: project_id.clone(),
        plan_digest: plan_digest.to_string(),
    };
    let bytes = serde_json_canonicalizer::to_vec(&registration)
        .map_err(|_| ProjectError::RecoveryRequired)?;
    write_private_project_metadata_once_locked(
        lock,
        root,
        MIGRATION_REGISTRATION_RELATIVE_PATH,
        &bytes,
    )
}

fn validate_recovered_destination(
    destination: &Path,
    manifest: &ArticleProjectManifestV1,
    source_inventory: &[PortableProjectEntryV1],
) -> Result<(), ProjectError> {
    let (mut inventory, excluded) = migration_inventory(destination)?;
    if excluded != 1 {
        return Err(ProjectError::RecoveryRequired);
    }
    let manifest_position = inventory
        .iter()
        .position(|entry| entry.relative_path == "context/project_manifest.json")
        .ok_or(ProjectError::RecoveryRequired)?;
    let manifest_entry = inventory.remove(manifest_position);
    let manifest_bytes =
        serde_json_canonicalizer::to_vec(manifest).map_err(|_| ProjectError::RecoveryRequired)?;
    if manifest_entry.size_bytes != manifest_bytes.len() as u64
        || manifest_entry.sha256 != canonical_digest(manifest)?
        || inventory != source_inventory
    {
        return Err(ProjectError::RecoveryRequired);
    }
    Ok(())
}
