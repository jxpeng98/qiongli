use std::fmt::{self, Debug, Formatter};
use std::path::{Path, PathBuf};

use qiongli_config::ConfigRoot;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::ProjectError;
use crate::migration::{
    ProjectMigrationCommitV1, VerifiedProjectMigration, committed_migration,
    complete_migration_registration_locked, ensure_migration_files,
    finalize_migration_registration_before_unregister, migration_commit,
    migration_registration_completed, preview_migration,
};
use crate::model::{
    ArticleProjectManifestV1, ArticleProjectSummaryV1, LibraryHealth, MAX_LIBRARY_PROJECTS,
    MAX_REGISTRATION_TOMBSTONES, MAX_SEMANTIC_REVISION, ProjectHealth, ProjectId, ProjectKind,
    ProjectLifecycle, ProjectMutationEffect, ProjectMutationKind, ProjectMutationPreviewV1,
    ProjectNextAction, ProjectRegistrationTombstoneIdentityKindV1, ProjectRegistrationTombstoneV1,
    ProjectStage, RESEARCH_LIBRARY_SCHEMA_VERSION, RegisteredProjectV1, ResearchLibraryDocumentV1,
    ResearchLibrarySnapshotV1,
};
use crate::portable::{
    PortableProjectCommitV1, PortableProjectOperation, VerifiedPortableProjectOperation,
    apply_export_files, committed_import, complete_portable_import_registration_locked,
    ensure_import_files, finalize_portable_import_registration_before_unregister,
    portable_import_registration_completed, preview_export, preview_import,
};
use crate::storage::{
    LibraryStore, create_project_root, empty_semantic_digest, lock_project_registration_journal,
    missing_continuity, project_root_from_string, project_root_label, project_root_string,
    read_manifest, read_overview, semantic_digest, validate_create_project_root,
    validate_existing_project_root, write_manifest,
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
        if registration_recovery_is_blocked(
            &library,
            plan.destination(),
            &plan.preview().project_id,
            plan.preview().expected_library_revision,
        )? {
            return Err(ProjectError::RecoveryRequired);
        }
        if registered_destination_revision(
            &library,
            plan.destination(),
            &plan.preview().project_id,
        )?
        .is_some()
        {
            let journal = lock_project_registration_journal(plan.destination())?;
            let current = self.store.load()?;
            current.validate()?;
            if registration_recovery_is_blocked(
                &current,
                plan.destination(),
                &plan.preview().project_id,
                plan.preview().expected_library_revision,
            )? {
                return Err(ProjectError::RecoveryRequired);
            }
            let library_revision = registered_destination_revision(
                &current,
                plan.destination(),
                &plan.preview().project_id,
            )?
            .ok_or(ProjectError::RecoveryRequired)?;
            committed_migration(plan)?.ok_or(ProjectError::RecoveryRequired)?;
            complete_migration_registration_locked(plan, library_revision, &journal)?;
            return Ok(migration_commit(plan, library_revision));
        }
        if migration_registration_completed(plan)? {
            return Err(ProjectError::RecoveryRequired);
        }
        if library.revision < plan.preview().expected_library_revision {
            return Err(ProjectError::RecoveryRequired);
        }
        let committed = if library.revision == plan.preview().expected_library_revision {
            validate_library_identity(
                &library.projects,
                plan.destination(),
                &plan.preview().project_id,
                ProjectMutationEffect::CreateProject,
            )?;
            ensure_migration_files(plan, now_unix)?
        } else {
            committed_migration(plan)?.ok_or(ProjectError::RevisionConflict)?
        };
        let journal = lock_project_registration_journal(plan.destination())?;
        let current = self.store.load()?;
        current.validate()?;
        if registration_recovery_is_blocked(
            &current,
            plan.destination(),
            &plan.preview().project_id,
            plan.preview().expected_library_revision,
        )? {
            return Err(ProjectError::RecoveryRequired);
        }
        if let Some(library_revision) = registered_destination_revision(
            &current,
            plan.destination(),
            &plan.preview().project_id,
        )? {
            committed_migration(plan)?.ok_or(ProjectError::RecoveryRequired)?;
            complete_migration_registration_locked(plan, library_revision, &journal)?;
            return Ok(migration_commit(plan, library_revision));
        }
        if migration_registration_completed(plan)? {
            return Err(ProjectError::RecoveryRequired);
        }
        let library_revision = self.finish_committed_registration(
            plan.destination(),
            &plan.preview().project_id,
            &committed.manifest,
            &committed.manifest_digest,
            committed.accepted_at_unix,
            plan.preview().expected_library_revision,
        )?;
        complete_migration_registration_locked(plan, library_revision, &journal)?;
        Ok(migration_commit(plan, library_revision))
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
        match plan.preview().operation {
            PortableProjectOperation::Export => {
                if library.revision != plan.preview().expected_library_revision {
                    return Err(ProjectError::RevisionConflict);
                }
                let entry = library
                    .projects
                    .iter()
                    .find(|entry| entry.project_id == plan.preview().project_id)
                    .ok_or(ProjectError::ProjectNotRegistered)?;
                if project_root_from_string(&entry.root_path)? != plan.source() {
                    return Err(ProjectError::RevisionConflict);
                }
                apply_export_files(plan)?;
                Ok(portable_commit(plan, None))
            }
            PortableProjectOperation::Import => {
                if registration_recovery_is_blocked(
                    &library,
                    plan.destination(),
                    &plan.preview().project_id,
                    plan.preview().expected_library_revision,
                )? {
                    return Err(ProjectError::RecoveryRequired);
                }
                if registered_destination_revision(
                    &library,
                    plan.destination(),
                    &plan.preview().project_id,
                )?
                .is_some()
                {
                    let journal = lock_project_registration_journal(plan.destination())?;
                    let current = self.store.load()?;
                    current.validate()?;
                    if registration_recovery_is_blocked(
                        &current,
                        plan.destination(),
                        &plan.preview().project_id,
                        plan.preview().expected_library_revision,
                    )? {
                        return Err(ProjectError::RecoveryRequired);
                    }
                    let library_revision = registered_destination_revision(
                        &current,
                        plan.destination(),
                        &plan.preview().project_id,
                    )?
                    .ok_or(ProjectError::RecoveryRequired)?;
                    committed_import(plan)?.ok_or(ProjectError::RecoveryRequired)?;
                    complete_portable_import_registration_locked(plan, library_revision, &journal)?;
                    return Ok(portable_commit(plan, Some(library_revision)));
                }
                if portable_import_registration_completed(plan)? {
                    return Err(ProjectError::RecoveryRequired);
                }
                if library.revision < plan.preview().expected_library_revision {
                    return Err(ProjectError::RecoveryRequired);
                }
                let committed = if library.revision == plan.preview().expected_library_revision {
                    validate_library_identity(
                        &library.projects,
                        plan.destination(),
                        &plan.preview().project_id,
                        ProjectMutationEffect::RegisterExistingManifest,
                    )?;
                    ensure_import_files(plan, now_unix)?
                } else {
                    committed_import(plan)?.ok_or(ProjectError::RevisionConflict)?
                };
                let journal = lock_project_registration_journal(plan.destination())?;
                let current = self.store.load()?;
                current.validate()?;
                if registration_recovery_is_blocked(
                    &current,
                    plan.destination(),
                    &plan.preview().project_id,
                    plan.preview().expected_library_revision,
                )? {
                    return Err(ProjectError::RecoveryRequired);
                }
                if let Some(library_revision) = registered_destination_revision(
                    &current,
                    plan.destination(),
                    &plan.preview().project_id,
                )? {
                    committed_import(plan)?.ok_or(ProjectError::RecoveryRequired)?;
                    complete_portable_import_registration_locked(plan, library_revision, &journal)?;
                    return Ok(portable_commit(plan, Some(library_revision)));
                }
                if portable_import_registration_completed(plan)? {
                    return Err(ProjectError::RecoveryRequired);
                }
                let library_revision = self.finish_committed_registration(
                    plan.destination(),
                    &plan.preview().project_id,
                    &committed.manifest,
                    &committed.manifest_digest,
                    committed.accepted_at_unix,
                    plan.preview().expected_library_revision,
                )?;
                complete_portable_import_registration_locked(plan, library_revision, &journal)?;
                Ok(portable_commit(plan, Some(library_revision)))
            }
        }
    }

    fn finish_committed_registration(
        &self,
        destination: &Path,
        project_id: &ProjectId,
        expected_manifest: &ArticleProjectManifestV1,
        expected_manifest_digest: &str,
        accepted_at_unix: u64,
        recovery_expected_library_revision: u64,
    ) -> Result<u64, ProjectError> {
        for _ in 0..3 {
            let library = self.store.load()?;
            library.validate()?;
            if registration_recovery_is_blocked(
                &library,
                destination,
                project_id,
                recovery_expected_library_revision,
            )? {
                return Err(ProjectError::RecoveryRequired);
            }
            if let Some(library_revision) =
                registered_destination_revision(&library, destination, project_id)?
            {
                validate_destination_manifest(
                    destination,
                    expected_manifest,
                    expected_manifest_digest,
                )?;
                return Ok(library_revision);
            }
            validate_library_identity(
                &library.projects,
                destination,
                project_id,
                ProjectMutationEffect::RegisterExistingManifest,
            )
            .map_err(|_| ProjectError::RecoveryRequired)?;

            let mut mutation = match self.store.begin(library.revision) {
                Ok(mutation) => mutation,
                Err(ProjectError::RevisionConflict) => continue,
                Err(error) => return Err(error),
            };
            if registration_recovery_is_blocked(
                &mutation.document,
                destination,
                project_id,
                recovery_expected_library_revision,
            )? {
                return Err(ProjectError::RecoveryRequired);
            }
            if let Some(library_revision) =
                registered_destination_revision(&mutation.document, destination, project_id)?
            {
                validate_destination_manifest(
                    destination,
                    expected_manifest,
                    expected_manifest_digest,
                )?;
                return Ok(library_revision);
            }
            validate_library_identity(
                &mutation.document.projects,
                destination,
                project_id,
                ProjectMutationEffect::RegisterExistingManifest,
            )
            .map_err(|_| ProjectError::RecoveryRequired)?;
            validate_destination_manifest(
                destination,
                expected_manifest,
                expected_manifest_digest,
            )?;
            clear_registration_tombstones(&mut mutation.document, destination, project_id)?;
            insert_registration(
                &mut mutation.document.projects,
                destination,
                expected_manifest,
                accepted_at_unix,
            )
            .map_err(|_| ProjectError::RecoveryRequired)?;
            return mutation
                .commit()
                .map_err(|_| ProjectError::RecoveryRequired);
        }

        let library = self
            .store
            .load()
            .map_err(|_| ProjectError::RecoveryRequired)?;
        library
            .validate()
            .map_err(|_| ProjectError::RecoveryRequired)?;
        if registration_recovery_is_blocked(
            &library,
            destination,
            project_id,
            recovery_expected_library_revision,
        )? {
            return Err(ProjectError::RecoveryRequired);
        }
        if let Some(library_revision) =
            registered_destination_revision(&library, destination, project_id)?
        {
            validate_destination_manifest(
                destination,
                expected_manifest,
                expected_manifest_digest,
            )?;
            return Ok(library_revision);
        }
        Err(ProjectError::RecoveryRequired)
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
        let registration_journal = if plan.preview.operation == ProjectMutationKind::Unregister {
            match lock_project_registration_journal(&plan.root) {
                Ok(lock) => Some(lock),
                Err(ProjectError::ProjectRootMissing) => None,
                Err(error) => return Err(error),
            }
        } else {
            None
        };
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
                if registration_recovery_is_blocked(
                    &mutation.document,
                    &plan.root,
                    &plan.preview.project_id,
                    plan.preview.expected_library_revision,
                )? {
                    return Err(ProjectError::RevisionConflict);
                }
                clear_registration_tombstones(
                    &mut mutation.document,
                    &plan.root,
                    &plan.preview.project_id,
                )?;
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
                if registration_recovery_is_blocked(
                    &mutation.document,
                    &plan.root,
                    &plan.preview.project_id,
                    plan.preview.expected_library_revision,
                )? {
                    return Err(ProjectError::RevisionConflict);
                }
                clear_registration_tombstones(
                    &mut mutation.document,
                    &plan.root,
                    &plan.preview.project_id,
                )?;
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
                if let Some(journal) = registration_journal.as_ref() {
                    finalize_migration_registration_before_unregister(
                        &plan.root,
                        &plan.preview.project_id,
                        journal,
                    )?;
                    finalize_portable_import_registration_before_unregister(
                        &plan.root,
                        &plan.preview.project_id,
                        journal,
                    )?;
                }
                record_registration_tombstones(
                    &mut mutation.document,
                    &plan.root,
                    &plan.preview.project_id,
                )?;
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
    projects.push(registration_entry(root, manifest, now_unix)?);
    Ok(())
}

fn registration_entry(
    root: &Path,
    manifest: &ArticleProjectManifestV1,
    registered_at_unix: u64,
) -> Result<RegisteredProjectV1, ProjectError> {
    Ok(RegisteredProjectV1 {
        project_id: manifest.project_id.clone(),
        display_name: manifest.display_name.clone(),
        project_kind: manifest.project_kind,
        stage: manifest.stage,
        lifecycle: manifest.lifecycle,
        semantic_revision: manifest.semantic_revision,
        semantic_digest: manifest.semantic_digest.clone(),
        root_path: project_root_string(root)?,
        registered_at_unix,
        last_opened_at_unix: None,
        academically_updated_at_unix: manifest.academically_updated_at_unix,
    })
}

fn registered_destination_revision(
    document: &ResearchLibraryDocumentV1,
    root: &Path,
    project_id: &ProjectId,
) -> Result<Option<u64>, ProjectError> {
    let root_path = project_root_string(root)?;
    let mut exact_match = false;
    for entry in &document.projects {
        let id_matches = &entry.project_id == project_id;
        let root_matches = entry.root_path == root_path;
        if id_matches || root_matches {
            if !id_matches || !root_matches || exact_match {
                return Err(ProjectError::ProjectIdentityConflict);
            }
            exact_match = true;
        }
    }
    Ok(exact_match.then_some(document.revision))
}

fn registration_recovery_is_blocked(
    document: &ResearchLibraryDocumentV1,
    root: &Path,
    project_id: &ProjectId,
    expected_library_revision: u64,
) -> Result<bool, ProjectError> {
    if expected_library_revision < document.registration_recovery_floor_revision {
        return Ok(true);
    }
    let root_reference_digest = root_reference_digest(root)?;
    Ok(document.registration_tombstones.iter().any(|tombstone| {
        expected_library_revision < tombstone.unregistered_at_library_revision
            && match tombstone.identity_kind {
                ProjectRegistrationTombstoneIdentityKindV1::ProjectId => {
                    tombstone.identity_value == project_id.as_str()
                }
                ProjectRegistrationTombstoneIdentityKindV1::RootReferenceDigest => {
                    tombstone.identity_value == root_reference_digest
                }
            }
    }))
}

fn record_registration_tombstones(
    document: &mut ResearchLibraryDocumentV1,
    root: &Path,
    project_id: &ProjectId,
) -> Result<(), ProjectError> {
    let unregistered_at_library_revision = document
        .revision
        .checked_add(1)
        .filter(|revision| *revision <= MAX_SEMANTIC_REVISION)
        .ok_or(ProjectError::RevisionConflict)?;
    upsert_registration_tombstone(
        document,
        ProjectRegistrationTombstoneIdentityKindV1::ProjectId,
        project_id.as_str().to_string(),
        unregistered_at_library_revision,
    );
    upsert_registration_tombstone(
        document,
        ProjectRegistrationTombstoneIdentityKindV1::RootReferenceDigest,
        root_reference_digest(root)?,
        unregistered_at_library_revision,
    );
    compact_registration_tombstones(document);
    Ok(())
}

fn upsert_registration_tombstone(
    document: &mut ResearchLibraryDocumentV1,
    identity_kind: ProjectRegistrationTombstoneIdentityKindV1,
    identity_value: String,
    unregistered_at_library_revision: u64,
) {
    if let Some(tombstone) = document
        .registration_tombstones
        .iter_mut()
        .find(|tombstone| {
            tombstone.identity_kind == identity_kind && tombstone.identity_value == identity_value
        })
    {
        tombstone.unregistered_at_library_revision = unregistered_at_library_revision;
        return;
    }
    document
        .registration_tombstones
        .push(ProjectRegistrationTombstoneV1 {
            identity_kind,
            identity_value,
            unregistered_at_library_revision,
        });
}

fn compact_registration_tombstones(document: &mut ResearchLibraryDocumentV1) {
    while document.registration_tombstones.len() > MAX_REGISTRATION_TOMBSTONES {
        let Some(oldest_revision) = document
            .registration_tombstones
            .iter()
            .map(|tombstone| tombstone.unregistered_at_library_revision)
            .min()
        else {
            break;
        };
        document.registration_recovery_floor_revision = document
            .registration_recovery_floor_revision
            .max(oldest_revision);
        let recovery_floor_revision = document.registration_recovery_floor_revision;
        document.registration_tombstones.retain(|tombstone| {
            tombstone.unregistered_at_library_revision > recovery_floor_revision
        });
    }
}

fn clear_registration_tombstones(
    document: &mut ResearchLibraryDocumentV1,
    root: &Path,
    project_id: &ProjectId,
) -> Result<(), ProjectError> {
    let root_reference_digest = root_reference_digest(root)?;
    document
        .registration_tombstones
        .retain(|tombstone| !match tombstone.identity_kind {
            ProjectRegistrationTombstoneIdentityKindV1::ProjectId => {
                tombstone.identity_value == project_id.as_str()
            }
            ProjectRegistrationTombstoneIdentityKindV1::RootReferenceDigest => {
                tombstone.identity_value == root_reference_digest
            }
        });
    Ok(())
}

fn root_reference_digest(root: &Path) -> Result<String, ProjectError> {
    Ok(sha256(project_root_string(root)?.as_bytes()))
}

fn validate_destination_manifest(
    root: &Path,
    expected_manifest: &ArticleProjectManifestV1,
    expected_manifest_digest: &str,
) -> Result<(), ProjectError> {
    read_manifest(root)?
        .filter(|(manifest, digest)| {
            manifest == expected_manifest && digest == expected_manifest_digest
        })
        .map(|_| ())
        .ok_or(ProjectError::RecoveryRequired)
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
    const COMMON_CREDENTIAL_FILES: [&str; 8] = [
        ".npmrc",
        ".pypirc",
        "id_rsa",
        "id_dsa",
        "id_ecdsa",
        "id_ed25519",
        "auth.json",
        ".netrc",
    ];

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

    fn write_common_credential_files(root: &Path) {
        for (index, file_name) in COMMON_CREDENTIAL_FILES.iter().enumerate() {
            fs::write(root.join(file_name), format!("private-value-{index}"))
                .expect("credential fixture must be writable");
        }
    }

    fn isolated_service(fixture: &Path, name: &str) -> ProjectStateService {
        let home = fixture.join(name);
        fs::create_dir(&home).unwrap();
        ProjectStateService::new(resolve_config_root(None, &home).unwrap())
    }

    fn assert_common_credential_files_absent(root: &Path) {
        for file_name in COMMON_CREDENTIAL_FILES {
            assert!(
                !root.join(file_name).exists(),
                "credential file {file_name} must be excluded"
            );
        }
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

    #[cfg(unix)]
    #[test]
    fn project_reads_reject_a_symlinked_intermediate_directory() {
        use std::os::unix::fs::symlink;

        let (fixture, service) = fixture();
        let root = fixture.join("symlinked-context-paper");
        let create = service
            .preview_create(
                &root,
                ProjectRegistrationOptions::new("Symlinked context", ProjectKind::Article),
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

        let external_context = fixture.join("external-context");
        fs::rename(root.join("context"), &external_context).unwrap();
        fs::write(
            external_context.join("research_state.md"),
            "RQ: This text is outside the registered project root.\n",
        )
        .unwrap();
        symlink(&external_context, root.join("context")).unwrap();

        assert!(matches!(
            crate::storage::read_manifest(&root),
            Err(ProjectError::UnsafeProjectRoot)
        ));
        assert!(matches!(
            crate::storage::read_overview(&root),
            Err(ProjectError::UnsafeProjectRoot)
        ));
        assert!(matches!(
            crate::storage::read_semantic_artifact(&root, "context/research_state.md"),
            Err(ProjectError::UnsafeProjectRoot)
        ));
        let snapshot = service.snapshot().unwrap();
        assert_eq!(
            snapshot.projects[0].health,
            ProjectHealth::InspectionBlocked
        );
        assert!(snapshot.projects[0].overview.focal_question.is_none());
    }

    #[test]
    fn portable_import_journal_recovers_and_replay_returns_current_revision() {
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
        write_common_credential_files(&root);
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
        assert_eq!(export.preview().excluded_entry_count, 10);
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
        assert_common_credential_files_absent(&package.join("project"));

        let no_receipt_home = fixture.join("no-receipt-home");
        fs::create_dir(&no_receipt_home).unwrap();
        let no_receipt_config = resolve_config_root(None, &no_receipt_home).unwrap();
        let no_receipt_service = ProjectStateService::new(no_receipt_config);
        let no_receipt_root = fixture.join("portable-no-receipt");
        let no_receipt_import = no_receipt_service
            .preview_import(&package, &no_receipt_root)
            .unwrap();
        let independent = no_receipt_service
            .preview_create(
                &no_receipt_root,
                ProjectRegistrationOptions::new("Independent portable ID", ProjectKind::Review)
                    .with_project_id(project_id.clone()),
                4,
            )
            .unwrap();
        no_receipt_service
            .apply(
                &independent,
                &ApprovedProjectMutation::new(independent.preview().plan_digest.clone(), true),
                4,
            )
            .unwrap();
        assert_eq!(
            no_receipt_service.apply_portable(
                &no_receipt_import,
                &ApprovedProjectMutation::new(
                    no_receipt_import.preview().plan_digest.clone(),
                    true,
                ),
                5,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        assert!(
            !no_receipt_root
                .join(".qiongli/v2/portable-import-registered.json")
                .exists()
        );

        let orphan_home = fixture.join("portable-orphan-home");
        fs::create_dir(&orphan_home).unwrap();
        let orphan_config = resolve_config_root(None, &orphan_home).unwrap();
        let orphan_service = ProjectStateService::new(orphan_config);
        let orphan_root = fixture.join("portable-orphan");
        let orphan_import = orphan_service
            .preview_import(&package, &orphan_root)
            .unwrap();
        ensure_import_files(&orphan_import, 6).unwrap();
        let unrelated_root = fixture.join("portable-orphan-unrelated");
        let unrelated = orphan_service
            .preview_create(
                &unrelated_root,
                ProjectRegistrationOptions::new("Unrelated portable paper", ProjectKind::Review),
                7,
            )
            .unwrap();
        orphan_service
            .apply(
                &unrelated,
                &ApprovedProjectMutation::new(unrelated.preview().plan_digest.clone(), true),
                7,
            )
            .unwrap();
        let recovered = orphan_service
            .apply_portable(
                &orphan_import,
                &ApprovedProjectMutation::new(orphan_import.preview().plan_digest.clone(), true),
                8,
            )
            .unwrap();
        assert_eq!(recovered.library_revision, Some(2));
        let recovered_entry = orphan_service
            .store
            .load()
            .unwrap()
            .projects
            .into_iter()
            .find(|entry| entry.project_id == project_id)
            .unwrap();
        assert_eq!(recovered_entry.registered_at_unix, 6);
        assert!(
            orphan_root
                .join(".qiongli/v2/portable-import-registered.json")
                .is_file()
        );

        let other_home = fixture.join("other-home");
        fs::create_dir(&other_home).unwrap();
        let other_config = resolve_config_root(None, &other_home).unwrap();
        let other = ProjectStateService::new(other_config);
        let imported_root = fixture.join("imported-paper");
        let import = other.preview_import(&package, &imported_root).unwrap();
        assert_eq!(ensure_import_files(&import, 4).unwrap().accepted_at_unix, 4);
        assert!(
            imported_root
                .join(".qiongli/v2/portable-import.json")
                .is_file()
        );
        assert!(other.snapshot().unwrap().projects.is_empty());
        let commit = other
            .apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                40,
            )
            .unwrap();
        assert_eq!(commit.library_revision, Some(1));
        assert!(
            imported_root
                .join(".qiongli/v2/portable-import-registered.json")
                .is_file()
        );
        assert_eq!(other.snapshot().unwrap().projects[0].project_id, project_id);
        assert_eq!(
            other.store.load().unwrap().projects[0].registered_at_unix,
            4
        );
        let later_root = fixture.join("later-paper");
        let later = other
            .preview_create(
                &later_root,
                ProjectRegistrationOptions::new("Later paper", ProjectKind::Review),
                41,
            )
            .unwrap();
        other
            .apply(
                &later,
                &ApprovedProjectMutation::new(later.preview().plan_digest.clone(), true),
                41,
            )
            .unwrap();
        let reconciled = other
            .apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                42,
            )
            .unwrap();
        assert_eq!(reconciled.library_revision, Some(2));
        assert_eq!(other.snapshot().unwrap().projects.len(), 2);
        assert_eq!(
            fs::read_to_string(imported_root.join("context/research_state.md")).unwrap(),
            "RQ: Can this move?\n"
        );

        let unregister = other.preview_unregister(&project_id).unwrap();
        other
            .apply(
                &unregister,
                &ApprovedProjectMutation::new(unregister.preview().plan_digest.clone(), true),
                43,
            )
            .unwrap();
        assert_eq!(
            other.apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                44,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        assert!(
            other
                .store
                .load()
                .unwrap()
                .projects
                .iter()
                .all(|entry| entry.project_id != project_id)
        );
    }

    #[test]
    fn unregister_backfills_a_missing_portable_import_completion_marker() {
        let (fixture, source_service) = fixture();
        let source_root = fixture.join("portable-crash-source");
        let create = source_service
            .preview_create(
                &source_root,
                ProjectRegistrationOptions::new("Portable crash window", ProjectKind::Article),
                50,
            )
            .unwrap();
        source_service
            .apply(
                &create,
                &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                50,
            )
            .unwrap();
        let project_id = create.preview().project_id.clone();
        let package = fixture.join("portable-crash-package");
        let export = source_service
            .preview_export(&project_id, &package)
            .unwrap();
        source_service
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                51,
            )
            .unwrap();

        let import_home = fixture.join("portable-crash-home");
        fs::create_dir(&import_home).unwrap();
        let import_config = resolve_config_root(None, &import_home).unwrap();
        let import_service = ProjectStateService::new(import_config);
        let imported_root = fixture.join("portable-crash-destination");
        let import = import_service
            .preview_import(&package, &imported_root)
            .unwrap();
        ensure_import_files(&import, 52).unwrap();
        let registration = import_service
            .preview_register(&imported_root, ProjectRegistrationOptions::existing(), 52)
            .unwrap();
        import_service
            .apply(
                &registration,
                &ApprovedProjectMutation::new(registration.preview().plan_digest.clone(), true),
                52,
            )
            .unwrap();
        let completion = imported_root.join(".qiongli/v2/portable-import-registered.json");
        assert!(!completion.exists());

        let unregister = import_service.preview_unregister(&project_id).unwrap();
        import_service
            .apply(
                &unregister,
                &ApprovedProjectMutation::new(unregister.preview().plan_digest.clone(), true),
                53,
            )
            .unwrap();
        assert!(completion.is_file());
        assert!(import_service.snapshot().unwrap().projects.is_empty());
        assert_eq!(
            import_service.apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                54,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        assert!(imported_root.is_dir());
    }

    #[test]
    fn legacy_migration_journal_recovers_and_replay_does_not_undo_unregister() {
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
        write_common_credential_files(&source);

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
        assert_eq!(plan.preview().excluded_entry_count, 10);
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

        assert_eq!(
            ensure_migration_files(&plan, 10).unwrap().accepted_at_unix,
            10
        );
        assert!(service.snapshot().unwrap().projects.is_empty());
        let commit = service
            .apply_migration(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                11,
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
        for file_name in COMMON_CREDENTIAL_FILES {
            assert!(source.join(file_name).is_file());
        }
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
        assert!(
            destination
                .join(".qiongli/v2/project-migration-registered.json")
                .is_file()
        );
        assert!(!destination.join(".qiongli/guidance_manifest.yaml").exists());
        assert!(!destination.join("secret-token.txt").exists());
        assert_common_credential_files_absent(&destination);
        assert_eq!(
            service.snapshot().unwrap().projects[0].project_id,
            plan.preview().project_id
        );
        assert_eq!(
            service.store.load().unwrap().projects[0].registered_at_unix,
            10
        );
        let later_root = fixture.join("later-migration-paper");
        let later = service
            .preview_create(
                &later_root,
                ProjectRegistrationOptions::new("Later migration paper", ProjectKind::Review),
                12,
            )
            .unwrap();
        service
            .apply(
                &later,
                &ApprovedProjectMutation::new(later.preview().plan_digest.clone(), true),
                12,
            )
            .unwrap();
        let reconciled = service
            .apply_migration(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                13,
            )
            .unwrap();
        assert_eq!(reconciled.library_revision, 2);

        let unregister = service
            .preview_unregister(&plan.preview().project_id)
            .unwrap();
        service
            .apply(
                &unregister,
                &ApprovedProjectMutation::new(unregister.preview().plan_digest.clone(), true),
                14,
            )
            .unwrap();
        assert_eq!(
            service.apply_migration(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                15,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        assert!(
            service
                .store
                .load()
                .unwrap()
                .projects
                .iter()
                .all(|entry| entry.project_id != plan.preview().project_id)
        );
    }

    #[test]
    fn unregister_backfills_a_missing_migration_completion_marker() {
        let (fixture, service) = fixture();
        let source = fixture.join("migration-crash-source");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("context")).unwrap();
        fs::write(
            source.join("context/research_state.md"),
            b"RQ: Can unregister close the migration crash window?\n",
        )
        .unwrap();
        let destination = fixture.join("migration-crash-destination");
        let migration = service
            .preview_migrate(
                &source,
                &destination,
                ProjectRegistrationOptions::new("Migration crash window", ProjectKind::Article),
                60,
            )
            .unwrap();
        ensure_migration_files(&migration, 60).unwrap();
        let registration = service
            .preview_register(&destination, ProjectRegistrationOptions::existing(), 60)
            .unwrap();
        service
            .apply(
                &registration,
                &ApprovedProjectMutation::new(registration.preview().plan_digest.clone(), true),
                60,
            )
            .unwrap();
        let completion = destination.join(".qiongli/v2/project-migration-registered.json");
        assert!(!completion.exists());

        let unregister = service
            .preview_unregister(&migration.preview().project_id)
            .unwrap();
        service
            .apply(
                &unregister,
                &ApprovedProjectMutation::new(unregister.preview().plan_digest.clone(), true),
                61,
            )
            .unwrap();
        assert!(completion.is_file());
        assert!(service.snapshot().unwrap().projects.is_empty());
        assert_eq!(
            service.apply_migration(
                &migration,
                &ApprovedProjectMutation::new(migration.preview().plan_digest.clone(), true),
                62,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        assert!(destination.is_dir());
    }

    #[test]
    fn committed_migration_recovers_after_an_unrelated_library_change() {
        let (fixture, service) = fixture();
        let source = fixture.join("legacy-concurrent");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("context")).unwrap();
        fs::write(
            source.join("context/research_state.md"),
            b"RQ: Can a committed migration be recovered?\n",
        )
        .unwrap();
        let destination = fixture.join("migrated-concurrent");
        let migration = service
            .preview_migrate(
                &source,
                &destination,
                ProjectRegistrationOptions::new("Concurrent migration", ProjectKind::Article),
                20,
            )
            .unwrap();
        ensure_migration_files(&migration, 20).unwrap();

        let unrelated_root = fixture.join("unrelated-paper");
        let unrelated = service
            .preview_create(
                &unrelated_root,
                ProjectRegistrationOptions::new("Unrelated paper", ProjectKind::Review),
                21,
            )
            .unwrap();
        service
            .apply(
                &unrelated,
                &ApprovedProjectMutation::new(unrelated.preview().plan_digest.clone(), true),
                21,
            )
            .unwrap();

        let recovered = service
            .apply_migration(
                &migration,
                &ApprovedProjectMutation::new(migration.preview().plan_digest.clone(), true),
                22,
            )
            .unwrap();
        assert_eq!(recovered.library_revision, 2);
        let document = service.store.load().unwrap();
        assert_eq!(document.revision, 2);
        assert_eq!(document.projects.len(), 2);
        assert!(
            document
                .projects
                .iter()
                .any(|entry| entry.project_id == unrelated.preview().project_id)
        );
        let migrated = document
            .projects
            .iter()
            .find(|entry| entry.project_id == migration.preview().project_id)
            .unwrap();
        assert_eq!(migrated.registered_at_unix, 20);
        assert!(destination.is_dir());
        assert!(
            destination
                .join(".qiongli/v2/project-migration.json")
                .is_file()
        );
        let journal = lock_project_registration_journal(&destination).unwrap();
        complete_migration_registration_locked(&migration, document.revision, &journal).unwrap();
        drop(journal);
        fs::write(
            destination.join(".qiongli/v2/project-migration-registered.json"),
            b"{}",
        )
        .unwrap();
        assert_eq!(
            service.apply_migration(
                &migration,
                &ApprovedProjectMutation::new(migration.preview().plan_digest.clone(), true),
                23,
            ),
            Err(ProjectError::RecoveryRequired)
        );
    }

    #[test]
    fn migration_replay_rejects_an_exact_registration_without_its_receipt() {
        let (fixture, service) = fixture();
        let source = fixture.join("legacy-no-receipt");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("context")).unwrap();
        fs::write(
            source.join("context/research_state.md"),
            b"RQ: Does this registration belong to the migration?\n",
        )
        .unwrap();
        let destination = fixture.join("independently-registered");
        let migration = service
            .preview_migrate(
                &source,
                &destination,
                ProjectRegistrationOptions::new("Migration preview", ProjectKind::Article),
                30,
            )
            .unwrap();

        let independent = service
            .preview_create(
                &destination,
                ProjectRegistrationOptions::new("Independent project", ProjectKind::Review)
                    .with_project_id(migration.preview().project_id.clone()),
                31,
            )
            .unwrap();
        service
            .apply(
                &independent,
                &ApprovedProjectMutation::new(independent.preview().plan_digest.clone(), true),
                31,
            )
            .unwrap();

        assert_eq!(
            service.apply_migration(
                &migration,
                &ApprovedProjectMutation::new(migration.preview().plan_digest.clone(), true),
                32,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        assert!(
            !destination
                .join(".qiongli/v2/project-migration-registered.json")
                .exists()
        );
    }

    #[test]
    fn migration_plan_digest_binds_manifest_timestamps() {
        let (fixture, service) = fixture();
        let source = fixture.join("legacy-timestamp-binding");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("context")).unwrap();
        fs::write(
            source.join("context/research_state.md"),
            b"RQ: Are migration timestamps part of approval?\n",
        )
        .unwrap();
        let destination = fixture.join("timestamp-bound-migration");
        let project_id = service.generate_project_id().unwrap();
        let options = ProjectRegistrationOptions::new("Timestamp bound", ProjectKind::Article)
            .with_project_id(project_id);
        let first = service
            .preview_migrate(&source, &destination, options.clone(), 40)
            .unwrap();
        let second = service
            .preview_migrate(&source, &destination, options, 41)
            .unwrap();
        assert_ne!(first.preview().plan_digest, second.preview().plan_digest);
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

    #[test]
    fn missing_root_unregister_blocks_an_old_portable_import_before_explicit_reregistration() {
        let (fixture, source_service) = fixture();
        let source_root = fixture.join("portable-missing-root-source");
        let create = source_service
            .preview_create(
                &source_root,
                ProjectRegistrationOptions::new("Portable missing root", ProjectKind::Article),
                100,
            )
            .unwrap();
        source_service
            .apply(
                &create,
                &ApprovedProjectMutation::new(create.preview().plan_digest.clone(), true),
                100,
            )
            .unwrap();
        let package = fixture.join("portable-missing-root-package");
        let export = source_service
            .preview_export(&create.preview().project_id, &package)
            .unwrap();
        source_service
            .apply_portable(
                &export,
                &ApprovedProjectMutation::new(export.preview().plan_digest.clone(), true),
                101,
            )
            .unwrap();

        let import_service = isolated_service(&fixture, "portable-missing-root-home");
        let destination = fixture.join("portable-missing-root-destination");
        let import = import_service
            .preview_import(&package, &destination)
            .unwrap();
        ensure_import_files(&import, 102).unwrap();
        let registration = import_service
            .preview_register(&destination, ProjectRegistrationOptions::existing(), 102)
            .unwrap();
        import_service
            .apply(
                &registration,
                &ApprovedProjectMutation::new(registration.preview().plan_digest.clone(), true),
                102,
            )
            .unwrap();

        let parked = fixture.join("portable-missing-root-parked");
        fs::rename(&destination, &parked).unwrap();
        let unregister = import_service
            .preview_unregister(&import.preview().project_id)
            .unwrap();
        let unregister_commit = import_service
            .apply(
                &unregister,
                &ApprovedProjectMutation::new(unregister.preview().plan_digest.clone(), true),
                103,
            )
            .unwrap();
        let document = import_service.store.load().unwrap();
        assert!(document.projects.is_empty());
        assert_eq!(document.registration_recovery_floor_revision, 0);
        assert_eq!(document.registration_tombstones.len(), 2);
        assert!(document.registration_tombstones.iter().all(|tombstone| {
            tombstone.unregistered_at_library_revision == unregister_commit.library_revision
        }));
        assert!(
            registration_recovery_is_blocked(
                &document,
                &destination,
                &import.preview().project_id,
                import.preview().expected_library_revision,
            )
            .unwrap()
        );
        let serialized = serde_json::to_string(&document).unwrap();
        assert!(!serialized.contains(destination.to_str().unwrap()));

        fs::rename(&parked, &destination).unwrap();
        assert_eq!(
            import_service.apply_portable(
                &import,
                &ApprovedProjectMutation::new(import.preview().plan_digest.clone(), true),
                104,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        let reregister = import_service
            .preview_register(&destination, ProjectRegistrationOptions::existing(), 105)
            .unwrap();
        import_service
            .apply(
                &reregister,
                &ApprovedProjectMutation::new(reregister.preview().plan_digest.clone(), true),
                105,
            )
            .unwrap();
        let document = import_service.store.load().unwrap();
        assert_eq!(document.projects.len(), 1);
        assert!(
            !registration_recovery_is_blocked(
                &document,
                &destination,
                &import.preview().project_id,
                document.revision,
            )
            .unwrap()
        );
        assert!(document.registration_tombstones.is_empty());
    }

    #[test]
    fn missing_root_unregister_blocks_an_old_migration_before_explicit_reregistration() {
        let (fixture, service) = fixture();
        let source = fixture.join("migration-missing-root-source");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("context")).unwrap();
        fs::write(
            source.join("context/research_state.md"),
            b"RQ: Can a missing migrated root stay unregistered?\n",
        )
        .unwrap();
        let destination = fixture.join("migration-missing-root-destination");
        let migration = service
            .preview_migrate(
                &source,
                &destination,
                ProjectRegistrationOptions::new("Migration missing root", ProjectKind::Article),
                110,
            )
            .unwrap();
        ensure_migration_files(&migration, 110).unwrap();
        let registration = service
            .preview_register(&destination, ProjectRegistrationOptions::existing(), 110)
            .unwrap();
        service
            .apply(
                &registration,
                &ApprovedProjectMutation::new(registration.preview().plan_digest.clone(), true),
                110,
            )
            .unwrap();

        let parked = fixture.join("migration-missing-root-parked");
        fs::rename(&destination, &parked).unwrap();
        let unregister = service
            .preview_unregister(&migration.preview().project_id)
            .unwrap();
        service
            .apply(
                &unregister,
                &ApprovedProjectMutation::new(unregister.preview().plan_digest.clone(), true),
                111,
            )
            .unwrap();
        fs::rename(&parked, &destination).unwrap();

        assert_eq!(
            service.apply_migration(
                &migration,
                &ApprovedProjectMutation::new(migration.preview().plan_digest.clone(), true),
                112,
            ),
            Err(ProjectError::RecoveryRequired)
        );
        let reregister = service
            .preview_register(&destination, ProjectRegistrationOptions::existing(), 113)
            .unwrap();
        service
            .apply(
                &reregister,
                &ApprovedProjectMutation::new(reregister.preview().plan_digest.clone(), true),
                113,
            )
            .unwrap();
        let document = service.store.load().unwrap();
        assert_eq!(document.projects.len(), 1);
        assert!(document.registration_tombstones.is_empty());
    }

    #[test]
    fn tombstones_match_either_identity_and_compact_without_filling_the_library() {
        let (fixture, service) = fixture();
        let id_a = service.generate_project_id().unwrap();
        let id_b = service.generate_project_id().unwrap();
        let root_x = fixture.join("identity-root-x");
        let root_y = fixture.join("identity-root-y");
        let mut document = ResearchLibraryDocumentV1::empty();
        document.revision = 4;
        document
            .registration_tombstones
            .push(ProjectRegistrationTombstoneV1 {
                identity_kind: ProjectRegistrationTombstoneIdentityKindV1::ProjectId,
                identity_value: id_a.as_str().to_string(),
                unregistered_at_library_revision: 3,
            });
        document
            .registration_tombstones
            .push(ProjectRegistrationTombstoneV1 {
                identity_kind: ProjectRegistrationTombstoneIdentityKindV1::RootReferenceDigest,
                identity_value: root_reference_digest(&root_x).unwrap(),
                unregistered_at_library_revision: 4,
            });
        assert!(document.validate().is_ok());
        assert!(registration_recovery_is_blocked(&document, &root_y, &id_a, 2).unwrap());
        assert!(registration_recovery_is_blocked(&document, &root_x, &id_b, 3).unwrap());
        assert!(!registration_recovery_is_blocked(&document, &root_y, &id_a, 3).unwrap());

        clear_registration_tombstones(&mut document, &root_y, &id_a).unwrap();
        assert!(registration_recovery_is_blocked(&document, &root_x, &id_b, 3).unwrap());
        assert_eq!(document.registration_recovery_floor_revision, 0);

        document.registration_tombstones.clear();
        document.revision = MAX_REGISTRATION_TOMBSTONES as u64;
        for index in 0..MAX_REGISTRATION_TOMBSTONES {
            document
                .registration_tombstones
                .push(ProjectRegistrationTombstoneV1 {
                    identity_kind: ProjectRegistrationTombstoneIdentityKindV1::RootReferenceDigest,
                    identity_value: format!("{index:064x}"),
                    unregistered_at_library_revision: index as u64 + 1,
                });
        }
        record_registration_tombstones(&mut document, &root_y, &id_b).unwrap();
        document.revision += 1;
        document.registration_tombstones.sort_by(|left, right| {
            left.identity_kind
                .cmp(&right.identity_kind)
                .then_with(|| left.identity_value.cmp(&right.identity_value))
        });
        assert_eq!(
            document.registration_tombstones.len(),
            MAX_REGISTRATION_TOMBSTONES
        );
        assert_eq!(document.registration_recovery_floor_revision, 2);
        document.validate().unwrap();
    }

    #[test]
    fn explicit_reregistration_does_not_block_an_unrelated_orphan_recovery() {
        let (fixture, service) = fixture();
        let source_a = fixture.join("unrelated-orphan-source-a");
        fs::create_dir(&source_a).unwrap();
        fs::create_dir(source_a.join("context")).unwrap();
        fs::write(
            source_a.join("context/research_state.md"),
            b"RQ: Can project A recover independently?\n",
        )
        .unwrap();
        let destination_a = fixture.join("unrelated-orphan-destination-a");
        let migration_a = service
            .preview_migrate(
                &source_a,
                &destination_a,
                ProjectRegistrationOptions::new("Orphan A", ProjectKind::Article),
                120,
            )
            .unwrap();
        ensure_migration_files(&migration_a, 120).unwrap();

        let root_b = fixture.join("reregistered-project-b");
        let create_b = service
            .preview_create(
                &root_b,
                ProjectRegistrationOptions::new("Project B", ProjectKind::Review),
                121,
            )
            .unwrap();
        service
            .apply(
                &create_b,
                &ApprovedProjectMutation::new(create_b.preview().plan_digest.clone(), true),
                121,
            )
            .unwrap();
        let unregister_b = service
            .preview_unregister(&create_b.preview().project_id)
            .unwrap();
        service
            .apply(
                &unregister_b,
                &ApprovedProjectMutation::new(unregister_b.preview().plan_digest.clone(), true),
                122,
            )
            .unwrap();
        let reregister_b = service
            .preview_register(&root_b, ProjectRegistrationOptions::existing(), 123)
            .unwrap();
        service
            .apply(
                &reregister_b,
                &ApprovedProjectMutation::new(reregister_b.preview().plan_digest.clone(), true),
                123,
            )
            .unwrap();
        let before_recovery = service.store.load().unwrap();
        assert_eq!(before_recovery.registration_recovery_floor_revision, 0);
        assert!(before_recovery.registration_tombstones.is_empty());

        let recovered = service
            .apply_migration(
                &migration_a,
                &ApprovedProjectMutation::new(migration_a.preview().plan_digest.clone(), true),
                124,
            )
            .unwrap();
        assert_eq!(recovered.library_revision, 4);
        assert_eq!(service.store.load().unwrap().projects.len(), 2);
    }

    #[test]
    fn legacy_library_documents_default_registration_recovery_state() {
        let document: ResearchLibraryDocumentV1 = serde_json::from_str(
            r#"{"schema_version":1,"document_kind":"qiongli-research-library","revision":0,"projects":[]}"#,
        )
        .unwrap();
        document.validate().unwrap();
        assert_eq!(document.registration_recovery_floor_revision, 0);
        assert!(document.registration_tombstones.is_empty());
    }
}
