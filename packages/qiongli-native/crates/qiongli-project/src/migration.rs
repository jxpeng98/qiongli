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

pub const PROJECT_MIGRATION_SCHEMA_VERSION: u32 = 1;
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
    let Some(receipt_bytes) =
        read_private_project_metadata(&plan.destination, MIGRATION_RECEIPT_RELATIVE_PATH)?
    else {
        return Ok(None);
    };
    let receipt_value = crate::json::parse_unique_json(&receipt_bytes)
        .map_err(|_| ProjectError::RecoveryRequired)?;
    let receipt: ProjectMigrationReceiptV1 =
        serde_json::from_value(receipt_value).map_err(|_| ProjectError::RecoveryRequired)?;
    if receipt.schema_version != PROJECT_MIGRATION_SCHEMA_VERSION
        || receipt.document_kind != PROJECT_MIGRATION_DOCUMENT_KIND
        || receipt.project_id != plan.preview.project_id
        || receipt.plan_digest != plan.preview.plan_digest
        || receipt.copied_file_count != plan.preview.copied_file_count
        || receipt.copied_bytes != plan.preview.copied_bytes
        || receipt.excluded_entry_count != plan.preview.excluded_entry_count
        || !receipt.source_retained
        || receipt.accepted_at_unix > MAX_SEMANTIC_REVISION
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
    let Some(bytes) =
        read_private_project_metadata(&plan.destination, MIGRATION_REGISTRATION_RELATIVE_PATH)?
    else {
        return Ok(false);
    };
    let value =
        crate::json::parse_unique_json(&bytes).map_err(|_| ProjectError::RecoveryRequired)?;
    let registration: ProjectMigrationRegistrationV1 =
        serde_json::from_value(value).map_err(|_| ProjectError::RecoveryRequired)?;
    if registration.schema_version != PROJECT_MIGRATION_SCHEMA_VERSION
        || registration.document_kind != MIGRATION_REGISTRATION_DOCUMENT_KIND
        || registration.project_id != plan.preview.project_id
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
    let registration = ProjectMigrationRegistrationV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        document_kind: MIGRATION_REGISTRATION_DOCUMENT_KIND.to_string(),
        project_id: plan.preview.project_id.clone(),
        plan_digest: plan.preview.plan_digest.clone(),
    };
    let bytes = serde_json_canonicalizer::to_vec(&registration)
        .map_err(|_| ProjectError::RecoveryRequired)?;
    write_private_project_metadata_once_locked(
        lock,
        &plan.destination,
        MIGRATION_REGISTRATION_RELATIVE_PATH,
        &bytes,
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
    let receipt_value = crate::json::parse_unique_json(&receipt_bytes)
        .map_err(|_| ProjectError::RecoveryRequired)?;
    let receipt: ProjectMigrationReceiptV1 =
        serde_json::from_value(receipt_value).map_err(|_| ProjectError::RecoveryRequired)?;
    if receipt.schema_version != PROJECT_MIGRATION_SCHEMA_VERSION
        || receipt.document_kind != PROJECT_MIGRATION_DOCUMENT_KIND
        || &receipt.project_id != project_id
        || !valid_lower_hex(&receipt.plan_digest, 64)
        || receipt.copied_file_count == 0
        || !receipt.source_retained
        || receipt.accepted_at_unix > MAX_SEMANTIC_REVISION
    {
        return Err(ProjectError::RecoveryRequired);
    }
    let registration = ProjectMigrationRegistrationV1 {
        schema_version: PROJECT_MIGRATION_SCHEMA_VERSION,
        document_kind: MIGRATION_REGISTRATION_DOCUMENT_KIND.to_string(),
        project_id: receipt.project_id,
        plan_digest: receipt.plan_digest,
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
