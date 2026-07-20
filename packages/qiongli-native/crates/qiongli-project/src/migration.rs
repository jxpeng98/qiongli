use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::ProjectError;
use crate::model::{
    ArticleProjectManifestV1, MissingContinuityArtifact, ProjectId, ProjectKind, ProjectStage,
};
use crate::portable::{
    PortableProjectEntryV1, canonical_digest, commit_staging, copy_inventory,
    create_staging_directory, ensure_private_subdirectories, migration_inventory, path_digest,
    write_private_file,
};
use crate::storage::{
    project_root_label, read_manifest, semantic_digest, validate_create_project_root,
    validate_existing_project_root, write_manifest,
};

pub const PROJECT_MIGRATION_SCHEMA_VERSION: u32 = 1;
pub const PROJECT_MIGRATION_DOCUMENT_KIND: &str = "qiongli-project-migration";
const MIGRATION_RECEIPT_RELATIVE_PATH: &str = ".qiongli/v2/project-migration.json";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMigrationPreviewV1 {
    pub schema_version: u32,
    pub plan_digest: String,
    pub project_id: ProjectId,
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
}

#[derive(Serialize)]
struct ProjectMigrationPlanSemantics<'a> {
    schema_version: u32,
    project_id: &'a ProjectId,
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
struct ProjectMigrationReceiptV1<'a> {
    schema_version: u32,
    document_kind: &'static str,
    project_id: &'a ProjectId,
    plan_digest: &'a str,
    copied_file_count: usize,
    copied_bytes: u64,
    excluded_entry_count: usize,
    source_retained: bool,
    accepted_at_unix: u64,
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
    if semantic_digest(source)? != manifest.semantic_digest {
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
        || semantic_digest(&plan.source)? != plan.manifest.semantic_digest
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
            document_kind: PROJECT_MIGRATION_DOCUMENT_KIND,
            project_id: &plan.manifest.project_id,
            plan_digest: &plan.preview.plan_digest,
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
