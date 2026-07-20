use std::fmt::{self, Debug, Formatter};
use std::path::{Path, PathBuf};

use qiongli_config::ConfigRoot;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::ProjectError;
use crate::migration::{
    ProjectMigrationCommitV1, VerifiedProjectMigration, apply_migration_files, migration_commit,
    preview_migration,
};
use crate::model::{
    ArticleProjectManifestV1, ArticleProjectSummaryV1, LibraryHealth, MAX_LIBRARY_PROJECTS,
    MAX_SEMANTIC_REVISION, ProjectHealth, ProjectId, ProjectKind, ProjectLifecycle,
    ProjectMutationEffect, ProjectMutationKind, ProjectMutationPreviewV1, ProjectNextAction,
    ProjectStage, RESEARCH_LIBRARY_SCHEMA_VERSION, RegisteredProjectV1, ResearchLibrarySnapshotV1,
};
use crate::portable::{
    PortableProjectCommitV1, PortableProjectOperation, VerifiedPortableProjectOperation,
    apply_files, preview_export, preview_import,
};
use crate::storage::{
    LibraryStore, create_project_root, empty_semantic_digest, missing_continuity,
    project_root_from_string, project_root_label, project_root_string, read_manifest,
    read_overview, semantic_digest, validate_create_project_root, validate_existing_project_root,
    write_manifest,
};

const PROJECT_MUTATION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectRegistrationOptions {
    pub project_id: Option<ProjectId>,
    pub display_name: Option<String>,
    pub project_kind: Option<ProjectKind>,
    pub stage: Option<ProjectStage>,
}

impl ProjectRegistrationOptions {
    #[must_use]
    pub const fn existing() -> Self {
        Self {
            project_id: None,
            display_name: None,
            project_kind: None,
            stage: None,
        }
    }

    #[must_use]
    pub fn new(display_name: impl Into<String>, project_kind: ProjectKind) -> Self {
        Self {
            project_id: None,
            display_name: Some(display_name.into()),
            project_kind: Some(project_kind),
            stage: Some(ProjectStage::Idea),
        }
    }

    #[must_use]
    pub fn with_project_id(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
        self
    }

    #[must_use]
    pub const fn with_stage(mut self, stage: ProjectStage) -> Self {
        self.stage = Some(stage);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedProjectMutation {
    expected_plan_digest: String,
    filesystem_write: bool,
}

impl ApprovedProjectMutation {
    #[must_use]
    pub fn new(expected_plan_digest: impl Into<String>, filesystem_write: bool) -> Self {
        Self {
            expected_plan_digest: expected_plan_digest.into(),
            filesystem_write,
        }
    }
}

#[derive(Clone)]
pub struct VerifiedProjectMutation {
    preview: ProjectMutationPreviewV1,
    root: PathBuf,
    root_reference_digest: String,
    observed_manifest_digest: Option<String>,
    next_manifest: Option<ArticleProjectManifestV1>,
}

impl VerifiedProjectMutation {
    #[must_use]
    pub const fn preview(&self) -> &ProjectMutationPreviewV1 {
        &self.preview
    }
}

impl Debug for VerifiedProjectMutation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedProjectMutation")
            .field("preview", &self.preview)
            .field("root", &"<registered-project-root>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMutationCommitV1 {
    pub schema_version: u32,
    pub operation: ProjectMutationKind,
    pub project_id: ProjectId,
    pub library_revision: u64,
    pub semantic_revision: u64,
    pub index_rebuild_required: bool,
}

#[derive(Clone)]
pub struct ProjectStateService {
    pub(crate) store: LibraryStore,
}

#[derive(Clone)]
pub struct RegisteredProjectRoot {
    path: PathBuf,
}

impl RegisteredProjectRoot {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Debug for RegisteredProjectRoot {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RegisteredProjectRoot")
            .field("path", &"<registered-project-root>")
            .finish()
    }
}

impl ProjectStateService {
    #[must_use]
    pub const fn new(config_root: ConfigRoot) -> Self {
        Self {
            store: LibraryStore::new(config_root),
        }
    }

    pub fn generate_project_id(&self) -> Result<ProjectId, ProjectError> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| ProjectError::RandomUnavailable)?;
        Ok(ProjectId::from_random_bytes(&bytes))
    }

    pub fn preview_migrate(
        &self,
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
        options: ProjectRegistrationOptions,
        now_unix: u64,
    ) -> Result<VerifiedProjectMigration, ProjectError> {
        validate_timestamp(now_unix)?;
        let source = source.as_ref();
        let destination = destination.as_ref();
        validate_existing_project_root(source)?;
        validate_create_project_root(destination)?;
        if read_manifest(source)?.is_some() {
            return Err(ProjectError::MigrationSourceInvalid);
        }
        let library = self.store.load()?;
        library.validate()?;
        let project_id = match options.project_id {
            Some(project_id) => project_id,
            None => self.generate_project_id()?,
        };
        let manifest = ArticleProjectManifestV1::new(
            project_id,
            options
                .display_name
                .unwrap_or_else(|| project_root_label(source)),
            options.project_kind.unwrap_or(ProjectKind::Article),
            options.stage.unwrap_or(ProjectStage::Idea),
            semantic_digest(source)?,
            now_unix,
        )?;
        validate_library_identity(
            &library.projects,
            destination,
            &manifest.project_id,
            ProjectMutationEffect::CreateProject,
        )?;
        preview_migration(
            source,
            destination,
            manifest,
            library.revision,
            missing_continuity(source)?,
        )
    }

    pub fn apply_migration(
        &self,
        plan: &VerifiedProjectMigration,
        approval: &ApprovedProjectMutation,
        now_unix: u64,
    ) -> Result<ProjectMigrationCommitV1, ProjectError> {
        validate_timestamp(now_unix)?;
        if !approval.filesystem_write {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.preview().plan_digest {
            return Err(ProjectError::PlanMismatch);
        }
        let library = self.store.load()?;
        library.validate()?;
        if library.revision != plan.preview().expected_library_revision {
            return Err(ProjectError::RevisionConflict);
        }
        validate_library_identity(
            &library.projects,
            plan.destination(),
            &plan.preview().project_id,
            ProjectMutationEffect::CreateProject,
        )?;
        apply_migration_files(plan, now_unix)?;

        let registration = match self.preview_register(
            plan.destination(),
            ProjectRegistrationOptions::existing(),
            now_unix,
        ) {
            Ok(registration)
                if registration.preview().expected_library_revision
                    == plan.preview().expected_library_revision =>
            {
                registration
            }
            Ok(_) => return Err(ProjectError::RecoveryRequired),
            Err(_) => return Err(ProjectError::RecoveryRequired),
        };
        let digest = registration.preview().plan_digest.clone();
        let commit = self
            .apply(
                &registration,
                &ApprovedProjectMutation::new(digest, true),
                now_unix,
            )
            .map_err(|_| ProjectError::RecoveryRequired)?;
        Ok(migration_commit(plan, commit.library_revision))
    }

    pub fn preview_export(
        &self,
        project_id: &ProjectId,
        destination: impl AsRef<Path>,
    ) -> Result<VerifiedPortableProjectOperation, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        let plan = preview_export(&root, destination.as_ref(), library.revision)?;
        if plan.package().project_id != *project_id {
            return Err(ProjectError::ProjectIdentityConflict);
        }
        Ok(plan)
    }

    pub fn preview_import(
        &self,
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> Result<VerifiedPortableProjectOperation, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let plan = preview_import(source.as_ref(), destination.as_ref(), library.revision)?;
        validate_library_identity(
            &library.projects,
            destination.as_ref(),
            &plan.package().project_id,
            ProjectMutationEffect::RegisterExistingManifest,
        )?;
        Ok(plan)
    }

    pub fn apply_portable(
        &self,
        plan: &VerifiedPortableProjectOperation,
        approval: &ApprovedProjectMutation,
        now_unix: u64,
    ) -> Result<PortableProjectCommitV1, ProjectError> {
        validate_timestamp(now_unix)?;
        if !approval.filesystem_write {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.preview().plan_digest {
            return Err(ProjectError::PlanMismatch);
        }
        let library = self.store.load()?;
        library.validate()?;
        if library.revision != plan.preview().expected_library_revision {
            return Err(ProjectError::RevisionConflict);
        }
        match plan.preview().operation {
            PortableProjectOperation::Export => {
                let entry = library
                    .projects
                    .iter()
                    .find(|entry| entry.project_id == plan.preview().project_id)
                    .ok_or(ProjectError::ProjectNotRegistered)?;
                if project_root_from_string(&entry.root_path)? != plan.source() {
                    return Err(ProjectError::RevisionConflict);
                }
                apply_files(plan)?;
                Ok(portable_commit(plan, None))
            }
            PortableProjectOperation::Import => {
                validate_library_identity(
                    &library.projects,
                    plan.destination(),
                    &plan.preview().project_id,
                    ProjectMutationEffect::RegisterExistingManifest,
                )?;
                apply_files(plan)?;
                let registration = self
                    .preview_register(
                        plan.destination(),
                        ProjectRegistrationOptions::existing(),
                        now_unix,
                    )
                    .map_err(|_| ProjectError::RecoveryRequired)?;
                let digest = registration.preview().plan_digest.clone();
                let commit = self
                    .apply(
                        &registration,
                        &ApprovedProjectMutation::new(digest, true),
                        now_unix,
                    )
                    .map_err(|_| ProjectError::RecoveryRequired)?;
                Ok(portable_commit(plan, Some(commit.library_revision)))
            }
        }
    }

    pub fn snapshot(&self) -> Result<ResearchLibrarySnapshotV1, ProjectError> {
        let document = self.store.load()?;
        document.validate()?;
        let mut projects = document
            .projects
            .iter()
            .map(inspect_registered_project)
            .collect::<Vec<_>>();
        projects.sort_by(|left, right| {
            lifecycle_rank(left.lifecycle)
                .cmp(&lifecycle_rank(right.lifecycle))
                .then_with(|| {
                    right
                        .academically_updated_at_unix
                        .cmp(&left.academically_updated_at_unix)
                })
                .then_with(|| left.project_id.cmp(&right.project_id))
        });
        Ok(ResearchLibrarySnapshotV1 {
            schema_version: RESEARCH_LIBRARY_SCHEMA_VERSION,
            revision: document.revision,
            health: if projects.is_empty() {
                LibraryHealth::Empty
            } else {
                LibraryHealth::Ready
            },
            projects,
        })
    }

    pub fn resolve_project_root(
        &self,
        project_id: &ProjectId,
    ) -> Result<RegisteredProjectRoot, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let path = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&path)?;
        let (manifest, _) = read_manifest(&path)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id {
            return Err(ProjectError::ProjectIdentityConflict);
        }
        Ok(RegisteredProjectRoot { path })
    }

    pub fn preview_register(
        &self,
        root: impl AsRef<Path>,
        options: ProjectRegistrationOptions,
        now_unix: u64,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        validate_timestamp(now_unix)?;
        let root = root.as_ref();
        validate_existing_project_root(root)?;
        let library = self.store.load()?;
        library.validate()?;
        let observed = read_manifest(root)?;
        let semantic_digest = semantic_digest(root)?;
        let (manifest, observed_manifest_digest, effect, manifest_action) = match observed {
            Some((manifest, digest)) => {
                validate_registration_options_against_manifest(&options, &manifest)?;
                let effect = if library.projects.iter().any(|entry| {
                    entry.project_id == manifest.project_id
                        && entry.root_path == root.to_str().unwrap_or_default()
                }) {
                    ProjectMutationEffect::NoChange
                } else {
                    ProjectMutationEffect::RegisterExistingManifest
                };
                (manifest, Some(digest), effect, "preserve-existing")
            }
            None => {
                let project_id = match options.project_id {
                    Some(project_id) => project_id,
                    None => self.generate_project_id()?,
                };
                let display_name = options
                    .display_name
                    .unwrap_or_else(|| project_root_label(root));
                let project_kind = options.project_kind.unwrap_or(ProjectKind::Article);
                let stage = options.stage.unwrap_or(ProjectStage::Idea);
                let manifest = ArticleProjectManifestV1::new(
                    project_id,
                    display_name,
                    project_kind,
                    stage,
                    semantic_digest,
                    now_unix,
                )?;
                (
                    manifest,
                    None,
                    ProjectMutationEffect::CreateManifestAndRegister,
                    "create-portable-manifest",
                )
            }
        };
        validate_library_identity(&library.projects, root, &manifest.project_id, effect)?;
        build_plan(
            root,
            &manifest,
            BuildPlanOptions {
                observed_manifest_digest,
                operation: ProjectMutationKind::Register,
                effect,
                expected_library_revision: library.revision,
                manifest_action,
                missing_continuity_artifacts: missing_continuity(root)?,
            },
        )
    }

    pub fn preview_create(
        &self,
        root: impl AsRef<Path>,
        options: ProjectRegistrationOptions,
        now_unix: u64,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        validate_timestamp(now_unix)?;
        let root = root.as_ref();
        validate_create_project_root(root)?;
        let library = self.store.load()?;
        library.validate()?;
        let project_id = match options.project_id {
            Some(project_id) => project_id,
            None => self.generate_project_id()?,
        };
        let display_name = options
            .display_name
            .unwrap_or_else(|| project_root_label(root));
        let manifest = ArticleProjectManifestV1::new(
            project_id,
            display_name,
            options.project_kind.unwrap_or(ProjectKind::Article),
            options.stage.unwrap_or(ProjectStage::Idea),
            empty_semantic_digest(),
            now_unix,
        )?;
        validate_library_identity(
            &library.projects,
            root,
            &manifest.project_id,
            ProjectMutationEffect::CreateProject,
        )?;
        build_plan(
            root,
            &manifest,
            BuildPlanOptions {
                observed_manifest_digest: None,
                operation: ProjectMutationKind::Create,
                effect: ProjectMutationEffect::CreateProject,
                expected_library_revision: library.revision,
                manifest_action: "create-project-and-portable-manifest",
                missing_continuity_artifacts: vec![
                    crate::MissingContinuityArtifact::ResearchState,
                    crate::MissingContinuityArtifact::DecisionLog,
                    crate::MissingContinuityArtifact::StageHandoff,
                    crate::MissingContinuityArtifact::LiteratureMap,
                    crate::MissingContinuityArtifact::ClaimEvidenceLedger,
                    crate::MissingContinuityArtifact::ManuscriptClaimMap,
                ],
            },
        )
    }

    pub fn preview_archive(
        &self,
        project_id: &ProjectId,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        self.preview_lifecycle(
            project_id,
            ProjectLifecycle::Archived,
            ProjectMutationKind::Archive,
        )
    }

    pub fn preview_repair_manifest(
        &self,
        project_id: &ProjectId,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        validate_existing_project_root(&root)?;
        if read_manifest(&root)?.is_some() {
            return Err(ProjectError::ProjectManifestConflict);
        }
        let manifest = manifest_from_entry(entry)?;
        build_plan(
            &root,
            &manifest,
            BuildPlanOptions {
                observed_manifest_digest: None,
                operation: ProjectMutationKind::RepairManifest,
                effect: ProjectMutationEffect::RebuildPortableManifest,
                expected_library_revision: library.revision,
                manifest_action: "rebuild-portable-manifest-from-private-index",
                missing_continuity_artifacts: missing_continuity(&root)?,
            },
        )
    }

    pub fn preview_restore(
        &self,
        project_id: &ProjectId,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        self.preview_lifecycle(
            project_id,
            ProjectLifecycle::Active,
            ProjectMutationKind::Restore,
        )
    }

    pub fn preview_refresh(
        &self,
        project_id: &ProjectId,
        now_unix: u64,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        validate_timestamp(now_unix)?;
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        let (mut manifest, observed_digest) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id {
            return Err(ProjectError::ProjectIdentityConflict);
        }
        let next_digest = semantic_digest(&root)?;
        let effect = if next_digest == manifest.semantic_digest {
            ProjectMutationEffect::NoChange
        } else {
            manifest.semantic_revision = manifest
                .semantic_revision
                .checked_add(1)
                .filter(|revision| *revision <= MAX_SEMANTIC_REVISION)
                .ok_or(ProjectError::RevisionConflict)?;
            manifest.semantic_digest = next_digest;
            manifest.academically_updated_at_unix = now_unix;
            ProjectMutationEffect::UpdateSemanticRevision
        };
        build_plan(
            &root,
            &manifest,
            BuildPlanOptions {
                observed_manifest_digest: Some(observed_digest),
                operation: ProjectMutationKind::Refresh,
                effect,
                expected_library_revision: library.revision,
                manifest_action: if effect == ProjectMutationEffect::NoChange {
                    "preserve-current-revision"
                } else {
                    "advance-semantic-revision"
                },
                missing_continuity_artifacts: missing_continuity(&root)?,
            },
        )
    }

    pub fn preview_unregister(
        &self,
        project_id: &ProjectId,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        let (manifest, observed_digest, missing) = match read_manifest(&root) {
            Ok(Some((manifest, digest))) => {
                if manifest.project_id != *project_id {
                    return Err(ProjectError::ProjectIdentityConflict);
                }
                (manifest, Some(digest), missing_continuity(&root)?)
            }
            Ok(None) | Err(ProjectError::ProjectRootMissing) => {
                (manifest_from_entry(entry)?, None, Vec::new())
            }
            Err(error) => return Err(error),
        };
        build_plan(
            &root,
            &manifest,
            BuildPlanOptions {
                observed_manifest_digest: observed_digest,
                operation: ProjectMutationKind::Unregister,
                effect: ProjectMutationEffect::RemoveLibraryEntry,
                expected_library_revision: library.revision,
                manifest_action: "preserve-project-remove-index-entry",
                missing_continuity_artifacts: missing,
            },
        )
    }

    pub fn apply(
        &self,
        plan: &VerifiedProjectMutation,
        approval: &ApprovedProjectMutation,
        now_unix: u64,
    ) -> Result<ProjectMutationCommitV1, ProjectError> {
        validate_timestamp(now_unix)?;
        if !approval.filesystem_write {
            return Err(ProjectError::ApprovalRequired);
        }
        if approval.expected_plan_digest != plan.preview.plan_digest {
            return Err(ProjectError::PlanMismatch);
        }
        validate_plan(plan)?;
        if plan.preview.effect == ProjectMutationEffect::NoChange {
            return Ok(ProjectMutationCommitV1 {
                schema_version: PROJECT_MUTATION_SCHEMA_VERSION,
                operation: plan.preview.operation,
                project_id: plan.preview.project_id.clone(),
                library_revision: plan.preview.expected_library_revision,
                semantic_revision: plan.preview.expected_project_revision.unwrap_or(1),
                index_rebuild_required: false,
            });
        }
        let mut mutation = self.store.begin(plan.preview.expected_library_revision)?;
        revalidate_library_plan(&mutation.document.projects, plan)?;

        let mut manifest_written = false;
        match plan.preview.operation {
            ProjectMutationKind::Create => {
                validate_create_project_root(&plan.root)?;
                create_project_root(&plan.root)?;
                let manifest = plan
                    .next_manifest
                    .as_ref()
                    .ok_or(ProjectError::PlanMismatch)?;
                write_manifest(&plan.root, manifest, None)?;
                manifest_written = true;
                insert_registration(
                    &mut mutation.document.projects,
                    &plan.root,
                    manifest,
                    now_unix,
                )?;
            }
            ProjectMutationKind::Register => {
                validate_existing_project_root(&plan.root)?;
                revalidate_manifest_observation(plan)?;
                let manifest = plan
                    .next_manifest
                    .as_ref()
                    .ok_or(ProjectError::PlanMismatch)?;
                if plan.preview.effect == ProjectMutationEffect::CreateManifestAndRegister {
                    write_manifest(&plan.root, manifest, None)?;
                    manifest_written = true;
                }
                insert_registration(
                    &mut mutation.document.projects,
                    &plan.root,
                    manifest,
                    now_unix,
                )?;
            }
            ProjectMutationKind::Archive
            | ProjectMutationKind::Restore
            | ProjectMutationKind::Refresh
            | ProjectMutationKind::RepairManifest => {
                revalidate_manifest_observation(plan)?;
                let manifest = plan
                    .next_manifest
                    .as_ref()
                    .ok_or(ProjectError::PlanMismatch)?;
                write_manifest(
                    &plan.root,
                    manifest,
                    plan.observed_manifest_digest.as_deref(),
                )?;
                manifest_written = true;
                update_registration(&mut mutation.document.projects, &plan.root, manifest)?;
            }
            ProjectMutationKind::Unregister => {
                revalidate_unregister_observation(plan)?;
                mutation
                    .document
                    .projects
                    .retain(|entry| entry.project_id != plan.preview.project_id);
            }
        }

        let library_revision = mutation.commit().map_err(|error| {
            if manifest_written {
                ProjectError::RecoveryRequired
            } else {
                error
            }
        })?;
        Ok(ProjectMutationCommitV1 {
            schema_version: PROJECT_MUTATION_SCHEMA_VERSION,
            operation: plan.preview.operation,
            project_id: plan.preview.project_id.clone(),
            library_revision,
            semantic_revision: plan
                .next_manifest
                .as_ref()
                .map_or(1, |manifest| manifest.semantic_revision),
            index_rebuild_required: false,
        })
    }

    fn preview_lifecycle(
        &self,
        project_id: &ProjectId,
        lifecycle: ProjectLifecycle,
        operation: ProjectMutationKind,
    ) -> Result<VerifiedProjectMutation, ProjectError> {
        let library = self.store.load()?;
        library.validate()?;
        let entry = library
            .projects
            .iter()
            .find(|entry| &entry.project_id == project_id)
            .ok_or(ProjectError::ProjectNotRegistered)?;
        let root = project_root_from_string(&entry.root_path)?;
        let (mut manifest, observed_digest) =
            read_manifest(&root)?.ok_or(ProjectError::ProjectManifestMissing)?;
        if manifest.project_id != *project_id {
            return Err(ProjectError::ProjectIdentityConflict);
        }
        let effect = if manifest.lifecycle == lifecycle {
            ProjectMutationEffect::NoChange
        } else {
            manifest.lifecycle = lifecycle;
            ProjectMutationEffect::UpdateLifecycle
        };
        build_plan(
            &root,
            &manifest,
            BuildPlanOptions {
                observed_manifest_digest: Some(observed_digest),
                operation,
                effect,
                expected_library_revision: library.revision,
                manifest_action: if effect == ProjectMutationEffect::NoChange {
                    "preserve-current-lifecycle"
                } else {
                    "update-portable-lifecycle"
                },
                missing_continuity_artifacts: missing_continuity(&root)?,
            },
        )
    }
}

fn portable_commit(
    plan: &VerifiedPortableProjectOperation,
    library_revision: Option<u64>,
) -> PortableProjectCommitV1 {
    PortableProjectCommitV1 {
        schema_version: crate::PORTABLE_PROJECT_SCHEMA_VERSION,
        operation: plan.preview().operation,
        project_id: plan.preview().project_id.clone(),
        library_revision,
        files_copied: plan.preview().file_count,
        total_bytes: plan.preview().total_bytes,
        destination_label: plan.preview().destination_label.clone(),
    }
}

struct BuildPlanOptions<'a> {
    observed_manifest_digest: Option<String>,
    operation: ProjectMutationKind,
    effect: ProjectMutationEffect,
    expected_library_revision: u64,
    manifest_action: &'a str,
    missing_continuity_artifacts: Vec<crate::MissingContinuityArtifact>,
}

fn build_plan(
    root: &Path,
    manifest: &ArticleProjectManifestV1,
    options: BuildPlanOptions<'_>,
) -> Result<VerifiedProjectMutation, ProjectError> {
    let BuildPlanOptions {
        observed_manifest_digest,
        operation,
        effect,
        expected_library_revision,
        manifest_action,
        missing_continuity_artifacts,
    } = options;
    let root_string = project_root_string(root)?;
    let root_reference_digest = sha256(root_string.as_bytes());
    let semantics = ProjectPlanSemantics {
        schema_version: PROJECT_MUTATION_SCHEMA_VERSION,
        operation,
        effect,
        project_id: manifest.project_id.clone(),
        display_name: manifest.display_name.clone(),
        project_kind: manifest.project_kind,
        stage: manifest.stage,
        lifecycle: manifest.lifecycle,
        semantic_revision: manifest.semantic_revision,
        semantic_digest: manifest.semantic_digest.clone(),
        expected_library_revision,
        root_reference_digest: root_reference_digest.clone(),
        observed_manifest_digest: observed_manifest_digest.clone(),
        manifest_action: manifest_action.to_string(),
        missing_continuity_artifacts: missing_continuity_artifacts.clone(),
    };
    let plan_digest = canonical_digest(&semantics)?;
    let preview = ProjectMutationPreviewV1 {
        schema_version: PROJECT_MUTATION_SCHEMA_VERSION,
        plan_digest,
        operation,
        effect,
        project_id: manifest.project_id.clone(),
        display_name: manifest.display_name.clone(),
        project_kind: manifest.project_kind,
        stage: manifest.stage,
        expected_library_revision,
        expected_project_revision: observed_manifest_digest
            .as_ref()
            .map(|_| manifest.semantic_revision),
        root_label: project_root_label(root),
        manifest_action: manifest_action.to_string(),
        missing_continuity_artifacts,
        approvals_required: if effect == ProjectMutationEffect::NoChange {
            Vec::new()
        } else {
            vec!["filesystem-write".to_string()]
        },
    };
    Ok(VerifiedProjectMutation {
        preview,
        root: root.to_path_buf(),
        root_reference_digest,
        observed_manifest_digest,
        next_manifest: (operation != ProjectMutationKind::Unregister).then(|| manifest.clone()),
    })
}

fn validate_plan(plan: &VerifiedProjectMutation) -> Result<(), ProjectError> {
    if project_root_string(&plan.root).map(|value| sha256(value.as_bytes()))
        != Ok(plan.root_reference_digest.clone())
    {
        return Err(ProjectError::PlanMismatch);
    }
    if plan.preview.operation == ProjectMutationKind::Unregister {
        return (plan.preview.effect == ProjectMutationEffect::RemoveLibraryEntry
            && plan.next_manifest.is_none()
            && plan
                .observed_manifest_digest
                .as_ref()
                .is_none_or(|digest| digest.len() == 64))
        .then_some(())
        .ok_or(ProjectError::PlanMismatch);
    }
    let manifest = plan
        .next_manifest
        .as_ref()
        .ok_or(ProjectError::PlanMismatch)?;
    if manifest.project_id != plan.preview.project_id
        || manifest.display_name != plan.preview.display_name
        || manifest.project_kind != plan.preview.project_kind
        || manifest.stage != plan.preview.stage
    {
        return Err(ProjectError::PlanMismatch);
    }
    Ok(())
}

fn revalidate_manifest_observation(plan: &VerifiedProjectMutation) -> Result<(), ProjectError> {
    let observed = read_manifest(&plan.root)?;
    match (observed, plan.observed_manifest_digest.as_deref()) {
        (None, None) => Ok(()),
        (Some((manifest, digest)), Some(expected))
            if digest == expected && manifest.project_id == plan.preview.project_id =>
        {
            Ok(())
        }
        _ => Err(ProjectError::RevisionConflict),
    }
}

fn revalidate_unregister_observation(plan: &VerifiedProjectMutation) -> Result<(), ProjectError> {
    match (
        read_manifest(&plan.root),
        plan.observed_manifest_digest.as_deref(),
    ) {
        (Err(ProjectError::ProjectRootMissing), None) | (Ok(None), None) => Ok(()),
        (Ok(Some((manifest, digest))), Some(expected))
            if digest == expected && manifest.project_id == plan.preview.project_id =>
        {
            Ok(())
        }
        _ => Err(ProjectError::RevisionConflict),
    }
}

fn revalidate_library_plan(
    projects: &[RegisteredProjectV1],
    plan: &VerifiedProjectMutation,
) -> Result<(), ProjectError> {
    match plan.preview.operation {
        ProjectMutationKind::Create | ProjectMutationKind::Register => validate_library_identity(
            projects,
            &plan.root,
            &plan.preview.project_id,
            plan.preview.effect,
        ),
        _ => {
            let root = project_root_string(&plan.root)?;
            projects
                .iter()
                .any(|entry| entry.project_id == plan.preview.project_id && entry.root_path == root)
                .then_some(())
                .ok_or(ProjectError::RevisionConflict)
        }
    }
}

fn validate_library_identity(
    projects: &[RegisteredProjectV1],
    root: &Path,
    project_id: &ProjectId,
    effect: ProjectMutationEffect,
) -> Result<(), ProjectError> {
    let root = project_root_string(root)?;
    for entry in projects {
        if &entry.project_id == project_id && entry.root_path != root {
            return Err(ProjectError::ProjectIdentityConflict);
        }
        if entry.root_path == root && &entry.project_id != project_id {
            return Err(ProjectError::ProjectIdentityConflict);
        }
        if &entry.project_id == project_id
            && entry.root_path == root
            && effect != ProjectMutationEffect::NoChange
        {
            return Err(ProjectError::ProjectAlreadyRegistered);
        }
    }
    if projects.len() >= MAX_LIBRARY_PROJECTS && effect != ProjectMutationEffect::NoChange {
        return Err(ProjectError::LibraryFull);
    }
    Ok(())
}

fn insert_registration(
    projects: &mut Vec<RegisteredProjectV1>,
    root: &Path,
    manifest: &ArticleProjectManifestV1,
    now_unix: u64,
) -> Result<(), ProjectError> {
    if projects.len() >= MAX_LIBRARY_PROJECTS {
        return Err(ProjectError::LibraryFull);
    }
    projects.push(RegisteredProjectV1 {
        project_id: manifest.project_id.clone(),
        display_name: manifest.display_name.clone(),
        project_kind: manifest.project_kind,
        stage: manifest.stage,
        lifecycle: manifest.lifecycle,
        semantic_revision: manifest.semantic_revision,
        semantic_digest: manifest.semantic_digest.clone(),
        root_path: project_root_string(root)?,
        registered_at_unix: now_unix,
        last_opened_at_unix: None,
        academically_updated_at_unix: manifest.academically_updated_at_unix,
    });
    Ok(())
}

fn manifest_from_entry(
    entry: &RegisteredProjectV1,
) -> Result<ArticleProjectManifestV1, ProjectError> {
    let manifest = ArticleProjectManifestV1 {
        schema_version: crate::ARTICLE_PROJECT_SCHEMA_VERSION,
        document_kind: crate::ARTICLE_PROJECT_DOCUMENT_KIND.to_string(),
        project_id: entry.project_id.clone(),
        display_name: entry.display_name.clone(),
        project_kind: entry.project_kind,
        stage: entry.stage,
        lifecycle: entry.lifecycle,
        semantic_revision: entry.semantic_revision,
        semantic_digest: entry.semantic_digest.clone(),
        created_at_unix: entry
            .registered_at_unix
            .min(entry.academically_updated_at_unix),
        academically_updated_at_unix: entry.academically_updated_at_unix,
    };
    manifest.validate()?;
    Ok(manifest)
}

fn update_registration(
    projects: &mut [RegisteredProjectV1],
    root: &Path,
    manifest: &ArticleProjectManifestV1,
) -> Result<(), ProjectError> {
    let root = project_root_string(root)?;
    let entry = projects
        .iter_mut()
        .find(|entry| entry.project_id == manifest.project_id && entry.root_path == root)
        .ok_or(ProjectError::ProjectNotRegistered)?;
    entry.display_name.clone_from(&manifest.display_name);
    entry.project_kind = manifest.project_kind;
    entry.stage = manifest.stage;
    entry.lifecycle = manifest.lifecycle;
    entry.semantic_revision = manifest.semantic_revision;
    entry.semantic_digest.clone_from(&manifest.semantic_digest);
    entry.academically_updated_at_unix = manifest.academically_updated_at_unix;
    Ok(())
}

fn inspect_registered_project(entry: &RegisteredProjectV1) -> ArticleProjectSummaryV1 {
    let fallback = |health, next_action| ArticleProjectSummaryV1 {
        project_id: entry.project_id.clone(),
        display_name: entry.display_name.clone(),
        project_kind: entry.project_kind,
        stage: entry.stage,
        lifecycle: entry.lifecycle,
        semantic_revision: entry.semantic_revision,
        registered_at_unix: entry.registered_at_unix,
        last_opened_at_unix: entry.last_opened_at_unix,
        academically_updated_at_unix: entry.academically_updated_at_unix,
        health,
        next_action,
        root_label: "Registered project".to_string(),
        overview: crate::ProjectOverviewV1::empty(),
    };
    let Ok(root) = project_root_from_string(&entry.root_path) else {
        return fallback(
            ProjectHealth::InspectionBlocked,
            ProjectNextAction::InspectPermissions,
        );
    };
    let root_label = project_root_label(&root);
    match read_manifest(&root) {
        Err(ProjectError::ProjectRootMissing) => {
            fallback(ProjectHealth::MissingRoot, ProjectNextAction::Relocate)
        }
        Err(_) => fallback(
            ProjectHealth::InspectionBlocked,
            ProjectNextAction::InspectPermissions,
        ),
        Ok(None) => fallback(
            ProjectHealth::MissingManifest,
            ProjectNextAction::RepairManifest,
        ),
        Ok(Some((manifest, _))) if manifest.project_id != entry.project_id => fallback(
            ProjectHealth::ManifestConflict,
            ProjectNextAction::InspectPermissions,
        ),
        Ok(Some((manifest, _))) => {
            let drifted = manifest.semantic_revision != entry.semantic_revision
                || manifest.semantic_digest != entry.semantic_digest
                || manifest.lifecycle != entry.lifecycle
                || manifest.stage != entry.stage
                || semantic_digest(&root).is_ok_and(|digest| digest != manifest.semantic_digest);
            ArticleProjectSummaryV1 {
                project_id: manifest.project_id,
                display_name: manifest.display_name,
                project_kind: manifest.project_kind,
                stage: manifest.stage,
                lifecycle: manifest.lifecycle,
                semantic_revision: manifest.semantic_revision,
                registered_at_unix: entry.registered_at_unix,
                last_opened_at_unix: entry.last_opened_at_unix,
                academically_updated_at_unix: manifest.academically_updated_at_unix,
                health: if drifted {
                    ProjectHealth::RevisionDrift
                } else {
                    ProjectHealth::Ready
                },
                next_action: if drifted {
                    ProjectNextAction::Refresh
                } else if manifest.lifecycle == ProjectLifecycle::Archived {
                    ProjectNextAction::Restore
                } else {
                    ProjectNextAction::Open
                },
                root_label,
                overview: read_overview(&root)
                    .unwrap_or_else(|_| crate::ProjectOverviewV1::empty()),
            }
        }
    }
}

fn validate_registration_options_against_manifest(
    options: &ProjectRegistrationOptions,
    manifest: &ArticleProjectManifestV1,
) -> Result<(), ProjectError> {
    if options
        .project_id
        .as_ref()
        .is_some_and(|project_id| project_id != &manifest.project_id)
        || options
            .display_name
            .as_ref()
            .is_some_and(|display_name| display_name != &manifest.display_name)
        || options
            .project_kind
            .is_some_and(|project_kind| project_kind != manifest.project_kind)
        || options.stage.is_some_and(|stage| stage != manifest.stage)
    {
        return Err(ProjectError::ProjectManifestConflict);
    }
    Ok(())
}

fn validate_timestamp(now_unix: u64) -> Result<(), ProjectError> {
    if now_unix > MAX_SEMANTIC_REVISION {
        Err(ProjectError::InvalidProjectDocument)
    } else {
        Ok(())
    }
}

fn lifecycle_rank(lifecycle: ProjectLifecycle) -> u8 {
    match lifecycle {
        ProjectLifecycle::Active => 0,
        ProjectLifecycle::Archived => 1,
    }
}

#[derive(Serialize)]
struct ProjectPlanSemantics {
    schema_version: u32,
    operation: ProjectMutationKind,
    effect: ProjectMutationEffect,
    project_id: ProjectId,
    display_name: String,
    project_kind: ProjectKind,
    stage: ProjectStage,
    lifecycle: ProjectLifecycle,
    semantic_revision: u64,
    semantic_digest: String,
    expected_library_revision: u64,
    root_reference_digest: String,
    observed_manifest_digest: Option<String>,
    manifest_action: String,
    missing_continuity_artifacts: Vec<crate::MissingContinuityArtifact>,
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ProjectError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProjectError::InvalidProjectDocument)?;
    Ok(sha256(&bytes))
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::resolve_config_root;

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    fn fixture() -> (PathBuf, ProjectStateService) {
        let root = std::env::temp_dir().join(format!(
            "qiongli-project-service-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let home = root.join("home");
        fs::create_dir(&home).unwrap();
        let config = resolve_config_root(None, &home).unwrap();
        (root, ProjectStateService::new(config))
    }

    #[test]
    fn create_register_archive_restore_and_reopen_three_projects() {
        let (fixture, service) = fixture();
        for index in 0..3 {
            let root = fixture.join(format!("paper-{index}"));
            let options =
                ProjectRegistrationOptions::new(format!("Paper {index}"), ProjectKind::Article);
            let plan = service.preview_create(&root, options, 10 + index).unwrap();
            let approval = ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true);
            service.apply(&plan, &approval, 10 + index).unwrap();
        }
        let snapshot = service.snapshot().unwrap();
        assert_eq!(snapshot.projects.len(), 3);
        assert!(
            snapshot
                .projects
                .iter()
                .all(|project| project.health == ProjectHealth::Ready)
        );

        let project_id = snapshot.projects[0].project_id.clone();
        let archive = service.preview_archive(&project_id).unwrap();
        service
            .apply(
                &archive,
                &ApprovedProjectMutation::new(archive.preview().plan_digest.clone(), true),
                20,
            )
            .unwrap();
        assert_eq!(
            service.snapshot().unwrap().projects[2].lifecycle,
            ProjectLifecycle::Archived
        );

        let restore = service.preview_restore(&project_id).unwrap();
        service
            .apply(
                &restore,
                &ApprovedProjectMutation::new(restore.preview().plan_digest.clone(), true),
                21,
            )
            .unwrap();
        assert!(
            service
                .snapshot()
                .unwrap()
                .projects
                .iter()
                .all(|project| project.lifecycle == ProjectLifecycle::Active)
        );
    }

    #[test]
    fn stale_library_revision_and_unapproved_writes_fail_without_mutation() {
        let (fixture, service) = fixture();
        let first_root = fixture.join("first");
        let first = service
            .preview_create(
                &first_root,
                ProjectRegistrationOptions::new("First", ProjectKind::Article),
                1,
            )
            .unwrap();
        assert_eq!(
            service.apply(
                &first,
                &ApprovedProjectMutation::new(first.preview().plan_digest.clone(), false),
                1,
            ),
            Err(ProjectError::ApprovalRequired)
        );
        assert!(!first_root.exists());

        let second_root = fixture.join("second");
        let stale = service
            .preview_create(
                &second_root,
                ProjectRegistrationOptions::new("Second", ProjectKind::Article),
                2,
            )
            .unwrap();
        service
            .apply(
                &first,
                &ApprovedProjectMutation::new(first.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        assert_eq!(
            service.apply(
                &stale,
                &ApprovedProjectMutation::new(stale.preview().plan_digest.clone(), true),
                2,
            ),
            Err(ProjectError::RevisionConflict)
        );
        assert!(!second_root.exists());
    }

    #[test]
    fn create_can_materialize_a_missing_research_container_under_a_workspace() {
        let (fixture, service) = fixture();
        let root = fixture.join("RESEARCH").join("nested-paper");
        let plan = service
            .preview_create(
                &root,
                ProjectRegistrationOptions::new("Nested paper", ProjectKind::Article),
                1,
            )
            .unwrap();

        service
            .apply(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();

        assert!(root.is_dir());
        assert!(root.join("context/project_manifest.json").is_file());
        assert_eq!(
            service.snapshot().unwrap().projects[0].root_label,
            "nested-paper"
        );
    }

    #[test]
    fn refresh_advances_only_when_semantic_artifacts_change() {
        let (fixture, service) = fixture();
        let root = fixture.join("paper");
        let plan = service
            .preview_create(
                &root,
                ProjectRegistrationOptions::new("Paper", ProjectKind::Review),
                1,
            )
            .unwrap();
        service
            .apply(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        let project_id = service.snapshot().unwrap().projects[0].project_id.clone();
        let unchanged = service.preview_refresh(&project_id, 2).unwrap();
        assert_eq!(unchanged.preview().effect, ProjectMutationEffect::NoChange);

        fs::write(
            root.join("context/research_state.md"),
            "RQ: Does X change Y?\n",
        )
        .unwrap();
        let changed = service.preview_refresh(&project_id, 3).unwrap();
        assert_eq!(
            changed.preview().effect,
            ProjectMutationEffect::UpdateSemanticRevision
        );
        service
            .apply(
                &changed,
                &ApprovedProjectMutation::new(changed.preview().plan_digest.clone(), true),
                3,
            )
            .unwrap();
        let project = &service.snapshot().unwrap().projects[0];
        assert_eq!(project.semantic_revision, 2);
        assert_eq!(
            project.overview.focal_question.as_deref(),
            Some("Does X change Y?")
        );
    }

    #[test]
    fn portable_export_import_excludes_private_runtime_material() {
        let (fixture, service) = fixture();
        let root = fixture.join("source-paper");
        let create = service
            .preview_create(
                &root,
                ProjectRegistrationOptions::new("Portable paper", ProjectKind::Article),
                1,
            )
            .unwrap();
        service
            .apply(
                &create,
                &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        fs::write(
            root.join("context/research_state.md"),
            "RQ: Can this move?\n",
        )
        .unwrap();
        fs::write(root.join("secret-token.txt"), "not portable").unwrap();
        fs::create_dir(root.join("sessions")).unwrap();
        fs::write(root.join("sessions/raw.json"), "{}").unwrap();
        let project_id = service.snapshot().unwrap().projects[0].project_id.clone();
        let refresh = service.preview_refresh(&project_id, 2).unwrap();
        service
            .apply(
                &refresh,
                &ApprovedProjectMutation::new(refresh.preview().plan_digest.clone(), true),
                2,
            )
            .unwrap();

        let package = fixture.join("portable-package");
        let export = service.preview_export(&project_id, &package).unwrap();
        assert_eq!(export.preview().excluded_entry_count, 2);
        service
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                3,
            )
            .unwrap();
        assert!(package.join("qiongli-portable-project.json").is_file());
        assert!(!package.join("project/secret-token.txt").exists());
        assert!(!package.join("project/sessions").exists());

        let other_home = fixture.join("other-home");
        fs::create_dir(&other_home).unwrap();
        let other_config = resolve_config_root(None, &other_home).unwrap();
        let other = ProjectStateService::new(other_config);
        let imported_root = fixture.join("imported-paper");
        let import = other.preview_import(&package, &imported_root).unwrap();
        let commit = other
            .apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                4,
            )
            .unwrap();
        assert_eq!(commit.library_revision, Some(1));
        assert_eq!(other.snapshot().unwrap().projects[0].project_id, project_id);
        assert_eq!(
            fs::read_to_string(imported_root.join("context/research_state.md")).unwrap(),
            "RQ: Can this move?\n"
        );
    }

    #[test]
    fn legacy_project_migration_copies_academic_files_and_retains_source() {
        let (fixture, service) = fixture();
        let source = fixture.join("legacy-paper");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("context")).unwrap();
        let research_state = b"RQ: Can legacy work move safely?\n";
        fs::write(source.join("context/research_state.md"), research_state).unwrap();
        fs::create_dir(source.join(".qiongli")).unwrap();
        fs::write(
            source.join(".qiongli/guidance_manifest.yaml"),
            b"active_subject: economics\n",
        )
        .unwrap();
        fs::write(source.join("secret-token.txt"), b"legacy-secret").unwrap();

        let destination = fixture.join("migrated-paper");
        let plan = service
            .preview_migrate(
                &source,
                &destination,
                ProjectRegistrationOptions::new("Migrated paper", ProjectKind::Article),
                10,
            )
            .unwrap();
        assert!(plan.preview().source_retained);
        assert_eq!(plan.preview().copied_file_count, 1);
        assert_eq!(plan.preview().excluded_entry_count, 2);
        let debug = format!("{plan:?}");
        assert!(!debug.contains(source.to_str().unwrap()));
        assert!(!debug.contains(destination.to_str().unwrap()));
        assert!(!debug.contains("legacy-secret"));

        assert_eq!(
            service.apply_migration(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), false),
                10,
            ),
            Err(ProjectError::ApprovalRequired)
        );
        assert!(!destination.exists());

        let commit = service
            .apply_migration(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                10,
            )
            .unwrap();
        assert_eq!(commit.library_revision, 1);
        assert!(commit.source_retained);
        assert_eq!(
            fs::read(source.join("context/research_state.md")).unwrap(),
            research_state
        );
        assert!(!source.join("context/project_manifest.json").exists());
        assert!(source.join(".qiongli/guidance_manifest.yaml").is_file());
        assert!(source.join("secret-token.txt").is_file());
        assert_eq!(
            fs::read(destination.join("context/research_state.md")).unwrap(),
            research_state
        );
        assert!(destination.join("context/project_manifest.json").is_file());
        assert!(
            destination
                .join(".qiongli/v2/project-migration.json")
                .is_file()
        );
        assert!(!destination.join(".qiongli/guidance_manifest.yaml").exists());
        assert!(!destination.join("secret-token.txt").exists());
        assert_eq!(
            service.snapshot().unwrap().projects[0].project_id,
            plan.preview().project_id
        );
    }

    #[test]
    fn legacy_project_migration_rejects_source_drift_before_copy() {
        let (fixture, service) = fixture();
        let source = fixture.join("legacy-drift");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("context")).unwrap();
        fs::write(source.join("context/research_state.md"), b"RQ: Before\n").unwrap();
        let destination = fixture.join("migrated-drift");
        let plan = service
            .preview_migrate(
                &source,
                &destination,
                ProjectRegistrationOptions::new("Drift", ProjectKind::Review),
                20,
            )
            .unwrap();
        fs::write(source.join("context/research_state.md"), b"RQ: After\n").unwrap();
        assert_eq!(
            service.apply_migration(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                20,
            ),
            Err(ProjectError::RevisionConflict)
        );
        assert!(!destination.exists());
        assert!(!source.join("context/project_manifest.json").exists());
    }

    #[test]
    fn doctor_repair_rebuilds_only_a_missing_portable_manifest() {
        let (fixture, service) = fixture();
        let root = fixture.join("paper");
        let create = service
            .preview_create(
                &root,
                ProjectRegistrationOptions::new("Repairable", ProjectKind::Review),
                1,
            )
            .unwrap();
        service
            .apply(
                &create,
                &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        let project_id = create.preview().project_id.clone();
        fs::remove_file(root.join("context/project_manifest.json")).unwrap();
        assert_eq!(
            service.snapshot().unwrap().projects[0].health,
            ProjectHealth::MissingManifest
        );

        let repair = service.preview_repair_manifest(&project_id).unwrap();
        assert_eq!(
            repair.preview().effect,
            ProjectMutationEffect::RebuildPortableManifest
        );
        service
            .apply(
                &repair,
                &ApprovedProjectMutation::new(repair.preview().plan_digest.clone(), true),
                2,
            )
            .unwrap();
        assert_eq!(
            service.snapshot().unwrap().projects[0].health,
            ProjectHealth::Ready
        );
        assert!(matches!(
            service.preview_repair_manifest(&project_id),
            Err(ProjectError::ProjectManifestConflict)
        ));
    }

    #[test]
    fn unregister_can_remove_an_unrecoverable_missing_root_without_deleting_artifacts() {
        let (fixture, service) = fixture();
        let root = fixture.join("paper");
        let create = service
            .preview_create(
                &root,
                ProjectRegistrationOptions::new("Missing", ProjectKind::Article),
                1,
            )
            .unwrap();
        service
            .apply(
                &create,
                &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                1,
            )
            .unwrap();
        let project_id = create.preview().project_id.clone();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(
            service.snapshot().unwrap().projects[0].health,
            ProjectHealth::MissingRoot
        );
        let unregister = service.preview_unregister(&project_id).unwrap();
        service
            .apply(
                &unregister,
                &ApprovedProjectMutation::new(unregister.preview().plan_digest.clone(), true),
                2,
            )
            .unwrap();
        assert!(service.snapshot().unwrap().projects.is_empty());
    }
}
