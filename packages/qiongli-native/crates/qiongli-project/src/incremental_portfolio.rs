use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::academic_graph_portfolio::{build_portfolio, portfolio_project_is_included};
use crate::model::{MAX_SEMANTIC_REVISION, RegisteredProjectV1, ResearchLibraryDocumentV1};
use crate::portfolio_catalog::{
    PortfolioCatalogSnapshotV1, PortfolioCatalogTransactionV1, PortfolioContributionRefV1,
    PortfolioContributionV1,
};
use crate::portfolio_catalog_storage::StoredPortfolioCatalog;
use crate::{
    AcademicGraphPortfolioService, AcademicGraphPortfolioSnapshotV1, AcademicGraphService,
    ArticleProjectSummaryV1, PortfolioCancellationToken, ProjectError, ProjectHealth, ProjectId,
    ProjectStateService, ResearchLibrarySnapshotV1,
};

pub const INCREMENTAL_PORTFOLIO_SCHEMA_VERSION: u32 = 1;
pub const INCREMENTAL_PORTFOLIO_SNAPSHOT_DOCUMENT_KIND: &str =
    "qiongli-incremental-portfolio-snapshot";
pub const PORTFOLIO_RECONCILIATION_DOCUMENT_KIND: &str = "qiongli-portfolio-reconciliation";
pub const PORTFOLIO_MAINTENANCE_PREVIEW_DOCUMENT_KIND: &str =
    "qiongli-portfolio-maintenance-preview";
pub const PORTFOLIO_DELETION_DOCUMENT_KIND: &str = "qiongli-portfolio-derived-state-deletion";
pub const PORTFOLIO_DOCTOR_DOCUMENT_KIND: &str = "qiongli-portfolio-doctor";
const PORTFOLIO_MAINTENANCE_APPROVAL: &str = "derived-state-write";
const PORTFOLIO_CANCELLATION_BATCH_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortfolioReconciliationMode {
    Incremental,
    Full,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortfolioMaintenanceOperation {
    Reconcile,
    FullRebuild,
    DeleteDerivedState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioMaintenancePreviewV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub plan_digest: String,
    pub operation: PortfolioMaintenanceOperation,
    pub expected_library_revision: u64,
    pub expected_catalog_id: Option<String>,
    pub expected_catalog_generation: Option<u64>,
    pub current_contribution_count: usize,
    pub derived_state_only: bool,
    pub approvals_required: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovedPortfolioMaintenance {
    expected_plan_digest: String,
    derived_state_write: bool,
}

impl ApprovedPortfolioMaintenance {
    #[must_use]
    pub fn new(expected_plan_digest: impl Into<String>, derived_state_write: bool) -> Self {
        Self {
            expected_plan_digest: expected_plan_digest.into(),
            derived_state_write,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPortfolioMaintenance {
    preview: PortfolioMaintenancePreviewV1,
}

impl VerifiedPortfolioMaintenance {
    #[must_use]
    pub const fn preview(&self) -> &PortfolioMaintenancePreviewV1 {
        &self.preview
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioDerivedStateDeletionV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub plan_digest: String,
    pub library_revision: u64,
    pub removed_catalog_id: Option<String>,
    pub removed_contribution_count: usize,
    pub derived_state_only: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortfolioDoctorStatus {
    Missing,
    Equivalent,
    Divergent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioDoctorV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub status: PortfolioDoctorStatus,
    pub library_revision: u64,
    pub catalog_id: Option<String>,
    pub incremental_portfolio_id: Option<String>,
    pub clean_portfolio_id: String,
    pub byte_equivalent: bool,
    pub contribution_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalPortfolioSnapshotV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub catalog: PortfolioCatalogSnapshotV1,
    pub portfolio: AcademicGraphPortfolioSnapshotV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioReconciliationV1 {
    pub schema_version: u32,
    pub document_kind: String,
    pub mode: PortfolioReconciliationMode,
    pub reconciled_at_unix: u64,
    pub catalog_changed: bool,
    pub rebuilt_project_count: usize,
    pub reused_project_count: usize,
    pub removed_project_count: usize,
    pub rebuilt_project_ids: Vec<ProjectId>,
    pub reused_project_ids: Vec<ProjectId>,
    pub removed_project_ids: Vec<ProjectId>,
    pub snapshot: IncrementalPortfolioSnapshotV1,
}

#[derive(Clone)]
pub struct IncrementalPortfolioService {
    projects: ProjectStateService,
}

impl IncrementalPortfolioService {
    #[must_use]
    pub const fn new(projects: ProjectStateService) -> Self {
        Self { projects }
    }

    pub fn current(&self) -> Result<IncrementalPortfolioSnapshotV1, ProjectError> {
        let initial_document = self.projects.store.load()?;
        initial_document.validate()?;
        let guard = self.projects.store.lock(initial_document.revision)?;
        if guard.document != initial_document {
            return Err(ProjectError::RevisionConflict);
        }
        let library = self.projects.snapshot()?;
        validate_library_snapshot(&guard.document, &library)?;
        let catalog = self
            .projects
            .portfolio_catalog_store
            .rebuild()?
            .ok_or(ProjectError::RecoveryRequired)?;
        validate_catalog_authority(&guard.document, &library, &catalog)?;
        let portfolio = portfolio_from_catalog(&library, &catalog)?;

        let confirmed_library = self.projects.snapshot()?;
        if confirmed_library != library || self.projects.store.load()? != guard.document {
            return Err(ProjectError::RevisionConflict);
        }
        let confirmed_catalog = self
            .projects
            .portfolio_catalog_store
            .rebuild()?
            .ok_or(ProjectError::RevisionConflict)?;
        if confirmed_catalog.manifest != catalog.manifest {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(IncrementalPortfolioSnapshotV1 {
            schema_version: INCREMENTAL_PORTFOLIO_SCHEMA_VERSION,
            document_kind: INCREMENTAL_PORTFOLIO_SNAPSHOT_DOCUMENT_KIND.to_string(),
            catalog: catalog.snapshot,
            portfolio,
        })
    }

    pub fn reconcile(&self, now_unix: u64) -> Result<PortfolioReconciliationV1, ProjectError> {
        self.reconcile_with_cancellation(now_unix, &PortfolioCancellationToken::new())
    }

    pub fn rebuild_full(&self, now_unix: u64) -> Result<PortfolioReconciliationV1, ProjectError> {
        self.rebuild_full_with_cancellation(now_unix, &PortfolioCancellationToken::new())
    }

    pub fn reconcile_with_cancellation(
        &self,
        now_unix: u64,
        cancellation: &PortfolioCancellationToken,
    ) -> Result<PortfolioReconciliationV1, ProjectError> {
        self.reconcile_with_mode(
            PortfolioReconciliationMode::Incremental,
            now_unix,
            cancellation,
        )
    }

    pub fn rebuild_full_with_cancellation(
        &self,
        now_unix: u64,
        cancellation: &PortfolioCancellationToken,
    ) -> Result<PortfolioReconciliationV1, ProjectError> {
        self.reconcile_with_mode(PortfolioReconciliationMode::Full, now_unix, cancellation)
    }

    pub fn preview_reconcile(&self) -> Result<VerifiedPortfolioMaintenance, ProjectError> {
        self.preview_maintenance(PortfolioMaintenanceOperation::Reconcile)
    }

    pub fn preview_full_rebuild(&self) -> Result<VerifiedPortfolioMaintenance, ProjectError> {
        self.preview_maintenance(PortfolioMaintenanceOperation::FullRebuild)
    }

    pub fn preview_delete_derived_state(
        &self,
    ) -> Result<VerifiedPortfolioMaintenance, ProjectError> {
        self.preview_maintenance(PortfolioMaintenanceOperation::DeleteDerivedState)
    }

    pub fn apply_reconcile(
        &self,
        plan: &VerifiedPortfolioMaintenance,
        approval: &ApprovedPortfolioMaintenance,
        now_unix: u64,
        cancellation: &PortfolioCancellationToken,
    ) -> Result<PortfolioReconciliationV1, ProjectError> {
        self.validate_maintenance(plan, approval, PortfolioMaintenanceOperation::Reconcile)?;
        self.reconcile_with_cancellation(now_unix, cancellation)
    }

    pub fn apply_full_rebuild(
        &self,
        plan: &VerifiedPortfolioMaintenance,
        approval: &ApprovedPortfolioMaintenance,
        now_unix: u64,
        cancellation: &PortfolioCancellationToken,
    ) -> Result<PortfolioReconciliationV1, ProjectError> {
        self.validate_maintenance(plan, approval, PortfolioMaintenanceOperation::FullRebuild)?;
        self.rebuild_full_with_cancellation(now_unix, cancellation)
    }

    pub fn apply_delete_derived_state(
        &self,
        plan: &VerifiedPortfolioMaintenance,
        approval: &ApprovedPortfolioMaintenance,
    ) -> Result<PortfolioDerivedStateDeletionV1, ProjectError> {
        self.validate_maintenance(
            plan,
            approval,
            PortfolioMaintenanceOperation::DeleteDerivedState,
        )?;
        let preview = plan.preview();
        let removed_contribution_count = self.projects.portfolio_catalog_store.delete(
            preview.expected_catalog_id.as_deref(),
            preview.expected_catalog_generation,
        )?;
        if self.projects.snapshot()?.revision != preview.expected_library_revision {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(PortfolioDerivedStateDeletionV1 {
            schema_version: INCREMENTAL_PORTFOLIO_SCHEMA_VERSION,
            document_kind: PORTFOLIO_DELETION_DOCUMENT_KIND.to_string(),
            plan_digest: preview.plan_digest.clone(),
            library_revision: preview.expected_library_revision,
            removed_catalog_id: preview.expected_catalog_id.clone(),
            removed_contribution_count,
            derived_state_only: true,
        })
    }

    pub fn doctor_compare(&self) -> Result<PortfolioDoctorV1, ProjectError> {
        let library = self.projects.snapshot()?;
        let clean = AcademicGraphPortfolioService::new(self.projects.clone()).rebuild()?;
        let current = match self.current() {
            Ok(current) => Some(current),
            Err(ProjectError::RecoveryRequired) => None,
            Err(error) => return Err(error),
        };
        let (status, catalog_id, incremental_portfolio_id, contribution_count, byte_equivalent) =
            if let Some(current) = current {
                let byte_equivalent = serde_json_canonicalizer::to_vec(&current.portfolio)
                    .map_err(|_| ProjectError::InvalidPortfolioCatalog)?
                    == serde_json_canonicalizer::to_vec(&clean)
                        .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
                (
                    if byte_equivalent {
                        PortfolioDoctorStatus::Equivalent
                    } else {
                        PortfolioDoctorStatus::Divergent
                    },
                    Some(current.catalog.catalog_id),
                    Some(current.portfolio.portfolio_id),
                    current.catalog.contribution_count,
                    byte_equivalent,
                )
            } else {
                (PortfolioDoctorStatus::Missing, None, None, 0, false)
            };
        if self.projects.snapshot()? != library {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(PortfolioDoctorV1 {
            schema_version: INCREMENTAL_PORTFOLIO_SCHEMA_VERSION,
            document_kind: PORTFOLIO_DOCTOR_DOCUMENT_KIND.to_string(),
            status,
            library_revision: library.revision,
            catalog_id,
            incremental_portfolio_id,
            clean_portfolio_id: clean.portfolio_id,
            byte_equivalent,
            contribution_count,
        })
    }

    fn preview_maintenance(
        &self,
        operation: PortfolioMaintenanceOperation,
    ) -> Result<VerifiedPortfolioMaintenance, ProjectError> {
        let library = self.projects.snapshot()?;
        let catalog = match self.projects.portfolio_catalog_store.rebuild() {
            Ok(catalog) => catalog,
            Err(ProjectError::InvalidPortfolioCatalog)
                if operation == PortfolioMaintenanceOperation::DeleteDerivedState =>
            {
                None
            }
            Err(error) => return Err(error),
        };
        let mut preview = PortfolioMaintenancePreviewV1 {
            schema_version: INCREMENTAL_PORTFOLIO_SCHEMA_VERSION,
            document_kind: PORTFOLIO_MAINTENANCE_PREVIEW_DOCUMENT_KIND.to_string(),
            plan_digest: String::new(),
            operation,
            expected_library_revision: library.revision,
            expected_catalog_id: catalog
                .as_ref()
                .map(|catalog| catalog.manifest.catalog_id.clone()),
            expected_catalog_generation: catalog
                .as_ref()
                .map(|catalog| catalog.manifest.generation),
            current_contribution_count: catalog
                .as_ref()
                .map_or(0, |catalog| catalog.contributions.len()),
            derived_state_only: true,
            approvals_required: vec![PORTFOLIO_MAINTENANCE_APPROVAL.to_string()],
        };
        preview.plan_digest = maintenance_digest(&preview)?;
        Ok(VerifiedPortfolioMaintenance { preview })
    }

    fn validate_maintenance(
        &self,
        plan: &VerifiedPortfolioMaintenance,
        approval: &ApprovedPortfolioMaintenance,
        operation: PortfolioMaintenanceOperation,
    ) -> Result<(), ProjectError> {
        if plan.preview.operation != operation
            || !approval.derived_state_write
            || approval.expected_plan_digest != plan.preview.plan_digest
        {
            return Err(if !approval.derived_state_write {
                ProjectError::ApprovalRequired
            } else {
                ProjectError::PlanMismatch
            });
        }
        let current = self.preview_maintenance(operation)?;
        if current.preview != plan.preview {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(())
    }

    fn reconcile_with_mode(
        &self,
        mode: PortfolioReconciliationMode,
        now_unix: u64,
        cancellation: &PortfolioCancellationToken,
    ) -> Result<PortfolioReconciliationV1, ProjectError> {
        cancellation.check()?;
        if now_unix > MAX_SEMANTIC_REVISION {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        let initial_document = self.projects.store.load()?;
        initial_document.validate()?;
        let guard = self.projects.store.lock(initial_document.revision)?;
        if guard.document != initial_document {
            return Err(ProjectError::RevisionConflict);
        }
        let library = self.projects.snapshot()?;
        validate_library_snapshot(&guard.document, &library)?;
        let current = self.projects.portfolio_catalog_store.rebuild()?;
        let current_by_project = current
            .as_ref()
            .map(|catalog| {
                catalog
                    .contributions
                    .iter()
                    .cloned()
                    .map(|contribution| (contribution.project_id.clone(), contribution))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let summary_by_project = library
            .projects
            .iter()
            .map(|project| (project.project_id.clone(), project))
            .collect::<BTreeMap<_, _>>();
        let entry_by_project = guard
            .document
            .projects
            .iter()
            .map(|project| (project.project_id.clone(), project))
            .collect::<BTreeMap<_, _>>();

        let graph_service = AcademicGraphService::new(self.projects.clone());
        let mut desired = Vec::new();
        let mut replacements = Vec::new();
        let mut rebuilt_project_ids = Vec::new();
        let mut reused_project_ids = Vec::new();
        for summary in library
            .projects
            .iter()
            .filter(|project| portfolio_project_is_included(project))
        {
            cancellation.check()?;
            let entry = entry_by_project
                .get(&summary.project_id)
                .copied()
                .ok_or(ProjectError::RevisionConflict)?;
            let reusable = current_by_project
                .get(&summary.project_id)
                .filter(|contribution| contribution_is_current(contribution, summary, entry))
                .cloned();
            let contribution = if mode == PortfolioReconciliationMode::Incremental {
                if let Some(contribution) = reusable {
                    reused_project_ids.push(summary.project_id.clone());
                    contribution
                } else {
                    let contribution =
                        rebuild_contribution(&graph_service, summary, entry, cancellation)?;
                    rebuilt_project_ids.push(summary.project_id.clone());
                    replacements.push(contribution.clone());
                    contribution
                }
            } else {
                let contribution =
                    rebuild_contribution(&graph_service, summary, entry, cancellation)?;
                rebuilt_project_ids.push(summary.project_id.clone());
                replacements.push(contribution.clone());
                contribution
            };
            desired.push(contribution);
        }
        desired.sort_by(|left, right| left.project_id.cmp(&right.project_id));
        replacements.sort_by(|left, right| left.project_id.cmp(&right.project_id));
        rebuilt_project_ids.sort_unstable();
        reused_project_ids.sort_unstable();

        let desired_ids = desired
            .iter()
            .map(|contribution| contribution.project_id.clone())
            .collect::<BTreeSet<_>>();
        let mut removed_project_ids = current_by_project
            .keys()
            .filter(|project_id| !desired_ids.contains(*project_id))
            .cloned()
            .collect::<Vec<_>>();
        removed_project_ids.sort_unstable();
        if summary_by_project.len() != guard.document.projects.len() {
            return Err(ProjectError::RevisionConflict);
        }

        let confirmed_library = self.projects.snapshot()?;
        if confirmed_library != library || self.projects.store.load()? != guard.document {
            return Err(ProjectError::RevisionConflict);
        }
        let desired_refs = desired
            .iter()
            .map(PortfolioContributionRefV1::from_contribution)
            .collect::<Result<Vec<_>, _>>()?;
        let catalog_unchanged = current.as_ref().is_some_and(|catalog| {
            catalog.manifest.library_revision == library.revision
                && catalog.manifest.contributions == desired_refs
        });
        cancellation.check()?;
        let (catalog, catalog_changed) = if catalog_unchanged {
            (current.ok_or(ProjectError::RecoveryRequired)?, false)
        } else {
            let transaction = PortfolioCatalogTransactionV1::new(
                current.as_ref().map(|catalog| catalog.manifest.clone()),
                replacements,
                removed_project_ids.clone(),
                library.revision,
                now_unix,
            )?;
            if transaction.next_manifest.contributions != desired_refs {
                return Err(ProjectError::PortfolioCatalogConflict);
            }
            (
                self.projects.portfolio_catalog_store.commit(&transaction)?,
                true,
            )
        };
        validate_catalog_authority(&guard.document, &library, &catalog)?;
        let portfolio = portfolio_from_catalog_with_cancellation(&library, &catalog, cancellation)?;

        let post_library = self.projects.snapshot()?;
        if post_library != library || self.projects.store.load()? != guard.document {
            return Err(ProjectError::RevisionConflict);
        }
        let post_catalog = self
            .projects
            .portfolio_catalog_store
            .rebuild()?
            .ok_or(ProjectError::RecoveryRequired)?;
        if post_catalog.manifest != catalog.manifest {
            return Err(ProjectError::RevisionConflict);
        }
        Ok(PortfolioReconciliationV1 {
            schema_version: INCREMENTAL_PORTFOLIO_SCHEMA_VERSION,
            document_kind: PORTFOLIO_RECONCILIATION_DOCUMENT_KIND.to_string(),
            mode,
            reconciled_at_unix: now_unix,
            catalog_changed,
            rebuilt_project_count: rebuilt_project_ids.len(),
            reused_project_count: reused_project_ids.len(),
            removed_project_count: removed_project_ids.len(),
            rebuilt_project_ids,
            reused_project_ids,
            removed_project_ids,
            snapshot: IncrementalPortfolioSnapshotV1 {
                schema_version: INCREMENTAL_PORTFOLIO_SCHEMA_VERSION,
                document_kind: INCREMENTAL_PORTFOLIO_SNAPSHOT_DOCUMENT_KIND.to_string(),
                catalog: catalog.snapshot,
                portfolio,
            },
        })
    }
}

fn rebuild_contribution(
    graph_service: &AcademicGraphService,
    summary: &ArticleProjectSummaryV1,
    entry: &RegisteredProjectV1,
    cancellation: &PortfolioCancellationToken,
) -> Result<PortfolioContributionV1, ProjectError> {
    cancellation.check()?;
    let graph = graph_service.rebuild(&summary.project_id)?;
    for batch in graph.nodes.chunks(PORTFOLIO_CANCELLATION_BATCH_SIZE) {
        let _ = batch;
        cancellation.check()?;
    }
    for batch in graph.edges.chunks(PORTFOLIO_CANCELLATION_BATCH_SIZE) {
        let _ = batch;
        cancellation.check()?;
    }
    let contribution = PortfolioContributionV1::from_graph(graph, ProjectHealth::Ready)?;
    if !contribution_is_current(&contribution, summary, entry) {
        return Err(ProjectError::RevisionConflict);
    }
    Ok(contribution)
}

fn contribution_is_current(
    contribution: &PortfolioContributionV1,
    summary: &ArticleProjectSummaryV1,
    entry: &RegisteredProjectV1,
) -> bool {
    contribution.project_id == summary.project_id
        && contribution.project_id == entry.project_id
        && contribution.health == ProjectHealth::Ready
        && summary.health == ProjectHealth::Ready
        && contribution.lifecycle == summary.lifecycle
        && contribution.lifecycle == entry.lifecycle
        && contribution.semantic_revision == summary.semantic_revision
        && contribution.semantic_revision == entry.semantic_revision
        && contribution.semantic_digest == entry.semantic_digest
        && contribution.graph.project_stage == summary.stage
        && contribution.graph.project_stage == entry.stage
        && contribution.graph.project_lifecycle == summary.lifecycle
        && contribution.graph.project_revision == summary.semantic_revision
        && contribution.graph.project_semantic_digest == entry.semantic_digest
        && contribution.graph.projection_id == contribution.projection_id
}

fn validate_library_snapshot(
    document: &ResearchLibraryDocumentV1,
    snapshot: &ResearchLibrarySnapshotV1,
) -> Result<(), ProjectError> {
    if snapshot.revision != document.revision || snapshot.projects.len() != document.projects.len()
    {
        return Err(ProjectError::RevisionConflict);
    }
    let summaries = snapshot
        .projects
        .iter()
        .map(|summary| (&summary.project_id, summary))
        .collect::<BTreeMap<_, _>>();
    for entry in &document.projects {
        let summary = summaries
            .get(&entry.project_id)
            .copied()
            .ok_or(ProjectError::RevisionConflict)?;
        if summary.health == ProjectHealth::Ready
            && (summary.display_name != entry.display_name
                || summary.project_kind != entry.project_kind
                || summary.stage != entry.stage
                || summary.lifecycle != entry.lifecycle
                || summary.semantic_revision != entry.semantic_revision
                || summary.academically_updated_at_unix != entry.academically_updated_at_unix)
        {
            return Err(ProjectError::RevisionConflict);
        }
    }
    Ok(())
}

fn validate_catalog_authority(
    document: &ResearchLibraryDocumentV1,
    library: &ResearchLibrarySnapshotV1,
    catalog: &StoredPortfolioCatalog,
) -> Result<(), ProjectError> {
    if catalog.manifest.library_revision != library.revision {
        return Err(ProjectError::RevisionConflict);
    }
    let entries = document
        .projects
        .iter()
        .map(|entry| (&entry.project_id, entry))
        .collect::<BTreeMap<_, _>>();
    let expected = library
        .projects
        .iter()
        .filter(|summary| portfolio_project_is_included(summary))
        .map(|summary| (&summary.project_id, summary))
        .collect::<BTreeMap<_, _>>();
    if catalog.contributions.len() != expected.len() {
        return Err(ProjectError::RevisionConflict);
    }
    for contribution in &catalog.contributions {
        let summary = expected
            .get(&contribution.project_id)
            .copied()
            .ok_or(ProjectError::RevisionConflict)?;
        let entry = entries
            .get(&contribution.project_id)
            .copied()
            .ok_or(ProjectError::RevisionConflict)?;
        if !contribution_is_current(contribution, summary, entry) {
            return Err(ProjectError::RevisionConflict);
        }
    }
    Ok(())
}

fn portfolio_from_catalog(
    library: &ResearchLibrarySnapshotV1,
    catalog: &StoredPortfolioCatalog,
) -> Result<AcademicGraphPortfolioSnapshotV1, ProjectError> {
    let graphs = catalog
        .contributions
        .iter()
        .map(|contribution| contribution.graph.clone())
        .collect::<Vec<_>>();
    build_portfolio(library, &graphs)
}

fn portfolio_from_catalog_with_cancellation(
    library: &ResearchLibrarySnapshotV1,
    catalog: &StoredPortfolioCatalog,
    cancellation: &PortfolioCancellationToken,
) -> Result<AcademicGraphPortfolioSnapshotV1, ProjectError> {
    for contribution in &catalog.contributions {
        for batch in contribution
            .graph
            .nodes
            .chunks(PORTFOLIO_CANCELLATION_BATCH_SIZE)
        {
            let _ = batch;
            cancellation.check()?;
        }
        for batch in contribution
            .graph
            .edges
            .chunks(PORTFOLIO_CANCELLATION_BATCH_SIZE)
        {
            let _ = batch;
            cancellation.check()?;
        }
    }
    cancellation.check()?;
    portfolio_from_catalog(library, catalog)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortfolioMaintenanceIdentity<'a> {
    schema_version: u32,
    document_kind: &'a str,
    operation: PortfolioMaintenanceOperation,
    expected_library_revision: u64,
    expected_catalog_id: &'a Option<String>,
    expected_catalog_generation: Option<u64>,
    current_contribution_count: usize,
    derived_state_only: bool,
    approvals_required: &'a [String],
}

fn maintenance_digest(preview: &PortfolioMaintenancePreviewV1) -> Result<String, ProjectError> {
    let identity = PortfolioMaintenanceIdentity {
        schema_version: preview.schema_version,
        document_kind: &preview.document_kind,
        operation: preview.operation,
        expected_library_revision: preview.expected_library_revision,
        expected_catalog_id: &preview.expected_catalog_id,
        expected_catalog_generation: preview.expected_catalog_generation,
        current_contribution_count: preview.current_contribution_count,
        derived_state_only: preview.derived_state_only,
        approvals_required: &preview.approvals_required,
    };
    let bytes = serde_json_canonicalizer::to_vec(&identity)
        .map_err(|_| ProjectError::InvalidPortfolioCatalog)?;
    let mut digest = Sha256::new();
    digest.update(b"qiongli-portfolio-maintenance-preview-v1\0");
    digest.update(bytes);
    Ok(format!("{:x}", digest.finalize()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_config::{ConfigRoot, resolve_config_root};

    use super::*;
    use crate::{
        AcademicGraphPortfolioService, ApprovedCaptureConsolidation, ApprovedCaptureIntake,
        ApprovedProjectMutation, CaptureArea, CaptureDelivery, CapturePolicy, CaptureSource,
        DecisionCandidateV1, DecisionRelation, EvidenceLocatorKind, EvidenceReferenceV1,
        ProjectBindingV1, ProjectKind, ProjectRegistrationOptions, ResearchCaptureDraftV1,
        SemanticChangeV1, VerifiedProjectMutation,
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        config: ConfigRoot,
        projects: ProjectStateService,
        service: IncrementalPortfolioService,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is available")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "qiongli-incremental-portfolio-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).expect("fixture root can be created");
            let root = fs::canonicalize(root).expect("fixture root can be canonicalized");
            let home = root.join("home");
            fs::create_dir(&home).expect("fixture home can be created");
            let config = resolve_config_root(Some(root.join("config").as_os_str()), &home)
                .expect("config root is valid");
            let projects = ProjectStateService::new(config.clone());
            let service = IncrementalPortfolioService::new(projects.clone());
            Self {
                root,
                config,
                projects,
                service,
            }
        }

        fn create_project(&self, name: &str, now_unix: u64) -> (ProjectId, PathBuf) {
            let root = self.root.join(name.to_lowercase().replace(' ', "-"));
            let plan = self
                .projects
                .preview_create(
                    &root,
                    ProjectRegistrationOptions::new(name, ProjectKind::Article),
                    now_unix,
                )
                .expect("create can be previewed");
            let project_id = plan.preview().project_id.clone();
            self.apply(&plan, now_unix);
            (project_id, root)
        }

        fn apply(&self, plan: &VerifiedProjectMutation, now_unix: u64) {
            self.projects
                .apply(
                    plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    now_unix,
                )
                .expect("mutation can be applied");
        }

        fn refresh(
            &self,
            project_id: &ProjectId,
            project_root: &Path,
            now_unix: u64,
            question: &str,
        ) {
            fs::write(
                project_root.join("context/research_state.md"),
                format!("- main_question_or_thesis: {question}\n"),
            )
            .expect("research state can be changed");
            let plan = self
                .projects
                .preview_refresh(project_id, now_unix)
                .expect("refresh can be previewed");
            self.apply(&plan, now_unix);
        }

        fn assert_matches_clean_full(&self, observed: &AcademicGraphPortfolioSnapshotV1) {
            let clean = AcademicGraphPortfolioService::new(self.projects.clone())
                .rebuild()
                .expect("clean portfolio rebuild succeeds");
            assert_eq!(observed, &clean);
            assert_eq!(
                serde_json_canonicalizer::to_vec(observed)
                    .expect("incremental portfolio serializes"),
                serde_json_canonicalizer::to_vec(&clean).expect("clean portfolio serializes")
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn incremental_reconcile_reuses_unchanged_projects_and_matches_clean_full_bytes() {
        let fixture = Fixture::new();
        assert_eq!(
            fixture.service.current().unwrap_err(),
            ProjectError::RecoveryRequired
        );
        let (project_a, root_a) = fixture.create_project("Project A", 1);
        let (project_b, _) = fixture.create_project("Project B", 2);

        let initial = fixture.service.reconcile(3).expect("initial reconcile");
        assert_eq!(initial.rebuilt_project_count, 2);
        assert_eq!(initial.reused_project_count, 0);
        assert!(initial.catalog_changed);
        fixture.assert_matches_clean_full(&initial.snapshot.portfolio);

        let noop = fixture.service.reconcile(4).expect("no-op reconcile");
        assert_eq!(noop.rebuilt_project_count, 0);
        assert_eq!(noop.reused_project_count, 2);
        assert!(!noop.catalog_changed);
        assert_eq!(
            noop.snapshot.catalog.catalog_id,
            initial.snapshot.catalog.catalog_id
        );
        assert_eq!(
            noop.snapshot.catalog.generation,
            initial.snapshot.catalog.generation
        );

        fixture.refresh(&project_a, &root_a, 5, "What changed in Project A?");
        let refreshed = fixture.service.reconcile(6).expect("refresh reconcile");
        assert_eq!(refreshed.rebuilt_project_ids, vec![project_a.clone()]);
        assert_eq!(refreshed.reused_project_ids, vec![project_b]);
        assert_eq!(refreshed.removed_project_count, 0);
        fixture.assert_matches_clean_full(&refreshed.snapshot.portfolio);

        let full = fixture.service.rebuild_full(7).expect("full rebuild");
        assert_eq!(full.rebuilt_project_count, 2);
        assert_eq!(full.reused_project_count, 0);
        assert!(!full.catalog_changed);
        assert_eq!(full.snapshot.portfolio, refreshed.snapshot.portfolio);
    }

    #[test]
    fn archive_restore_unregister_and_restart_reconcile_only_owned_contributions() {
        let fixture = Fixture::new();
        let (project_a, _) = fixture.create_project("Lifecycle A", 1);
        let (project_b, _) = fixture.create_project("Lifecycle B", 2);
        fixture.service.reconcile(3).expect("initial reconcile");

        let archive = fixture
            .projects
            .preview_archive(&project_a)
            .expect("archive can be previewed");
        fixture.apply(&archive, 4);
        let archived = fixture.service.reconcile(5).expect("archive reconcile");
        assert_eq!(archived.removed_project_ids, vec![project_a.clone()]);
        assert_eq!(archived.reused_project_ids, vec![project_b.clone()]);
        assert_eq!(archived.snapshot.portfolio.included_project_count, 1);
        assert_eq!(archived.snapshot.portfolio.skipped_project_count, 1);
        fixture.assert_matches_clean_full(&archived.snapshot.portfolio);

        let restore = fixture
            .projects
            .preview_restore(&project_a)
            .expect("restore can be previewed");
        fixture.apply(&restore, 6);
        let restored = fixture.service.reconcile(7).expect("restore reconcile");
        assert_eq!(restored.rebuilt_project_ids, vec![project_a.clone()]);
        assert_eq!(restored.reused_project_ids, vec![project_b.clone()]);
        assert_eq!(restored.snapshot.portfolio.included_project_count, 2);
        fixture.assert_matches_clean_full(&restored.snapshot.portfolio);

        let unregister = fixture
            .projects
            .preview_unregister(&project_b)
            .expect("unregister can be previewed");
        fixture.apply(&unregister, 8);
        let unregistered = fixture.service.reconcile(9).expect("unregister reconcile");
        assert_eq!(unregistered.removed_project_ids, vec![project_b]);
        assert_eq!(unregistered.snapshot.portfolio.project_count, 1);
        fixture.assert_matches_clean_full(&unregistered.snapshot.portfolio);

        let restarted_projects = ProjectStateService::new(fixture.config.clone());
        let restarted = IncrementalPortfolioService::new(restarted_projects)
            .current()
            .expect("restarted service reads the same current catalog");
        assert_eq!(restarted, unregistered.snapshot);
    }

    #[test]
    fn drifted_project_is_never_returned_as_a_current_mixed_revision() {
        let fixture = Fixture::new();
        let (project_id, project_root) = fixture.create_project("Drift Project", 1);
        fixture.service.reconcile(2).expect("initial reconcile");
        fs::write(
            project_root.join("context/research_state.md"),
            "- main_question_or_thesis: unregistered drift\n",
        )
        .expect("project can drift outside the control plane");

        assert_eq!(
            fixture.service.current().unwrap_err(),
            ProjectError::RevisionConflict
        );
        let reconciled = fixture.service.reconcile(3).expect("drift is skipped");
        assert_eq!(reconciled.removed_project_ids, vec![project_id]);
        assert_eq!(reconciled.snapshot.portfolio.included_project_count, 0);
        assert_eq!(reconciled.snapshot.portfolio.skipped_project_count, 1);
        fixture.assert_matches_clean_full(&reconciled.snapshot.portfolio);
    }

    #[test]
    fn migration_import_and_rollback_replace_only_their_derived_contribution() {
        let fixture = Fixture::new();
        let source = fixture.root.join("legacy-source");
        fs::create_dir(&source).expect("legacy source can be created");
        fs::create_dir(source.join("context")).expect("legacy context can be created");
        fs::write(
            source.join("context/research_state.md"),
            "RQ: Can legacy evidence migrate incrementally?\n",
        )
        .expect("legacy state can be written");
        let destination = fixture.root.join("migrated-project");
        let migration = fixture
            .projects
            .preview_migrate(
                &source,
                &destination,
                ProjectRegistrationOptions::new("Migrated project", ProjectKind::Article),
                1,
            )
            .expect("migration can be previewed");
        let project_id = migration.preview().project_id.clone();
        fixture
            .projects
            .apply_migration(
                &migration,
                &ApprovedProjectMutation::new(migration.preview().plan_digest.clone(), true),
                2,
            )
            .expect("migration can be applied");

        let imported = fixture.service.reconcile(3).expect("migration reconcile");
        assert_eq!(imported.rebuilt_project_ids, vec![project_id.clone()]);
        fixture.assert_matches_clean_full(&imported.snapshot.portfolio);

        let rollback = fixture
            .projects
            .preview_migration_rollback(&source, &destination)
            .expect("rollback can be previewed");
        fixture
            .projects
            .apply_migration_rollback(
                &rollback,
                &ApprovedProjectMutation::new(rollback.preview().plan_digest.clone(), true),
            )
            .expect("rollback can be applied");
        let rolled_back = fixture.service.reconcile(4).expect("rollback reconcile");
        assert_eq!(rolled_back.removed_project_ids, vec![project_id]);
        assert_eq!(rolled_back.snapshot.portfolio.project_count, 0);
        assert!(source.is_dir());
        assert!(!destination.exists());
        fixture.assert_matches_clean_full(&rolled_back.snapshot.portfolio);
    }

    #[test]
    fn accepted_consolidation_replaces_one_revision_bound_contribution() {
        let fixture = Fixture::new();
        let (project_id, _) = fixture.create_project("Consolidated Project", 1);
        fixture.service.reconcile(2).expect("initial reconcile");
        let summary = fixture
            .projects
            .snapshot()
            .expect("library is readable")
            .projects
            .into_iter()
            .find(|project| project.project_id == project_id)
            .expect("project is registered");
        let capture = ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                project_id.clone(),
                summary.semantic_revision,
                summary.stage,
                "Reconcile the measurement literature",
                CapturePolicy::ReviewRequired,
            )
            .expect("capture binding is valid"),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Connected,
            captured_at_unix: 3,
            summary: "Validity and reliability remain distinct constructs.".to_string(),
            changes: vec![SemanticChangeV1 {
                area: CaptureArea::Literature,
                summary: "Separate validity and reliability evidence streams.".to_string(),
            }],
            decisions: vec![DecisionCandidateV1 {
                relation: DecisionRelation::Candidate,
                statement: "Organize the review around construct validity.".to_string(),
                rationale: "This distinction explains disagreement across sources.".to_string(),
                target: None,
            }],
            evidence: vec![EvidenceReferenceV1 {
                locator_kind: EvidenceLocatorKind::Doi,
                locator: "10.1000/incremental".to_string(),
                relevance: "Defines the construct-validity distinction.".to_string(),
                limitation: Some("Conceptual evidence only.".to_string()),
            }],
            contradictions: Vec::new(),
            next_actions: vec!["Test the distinction against empirical papers.".to_string()],
        }
        .into_capture()
        .expect("capture is valid");
        let intake = fixture
            .projects
            .preview_capture(capture.clone())
            .expect("capture can be previewed");
        fixture
            .projects
            .apply_capture(
                &intake,
                &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
                4,
            )
            .expect("capture can be accepted");
        let before_consolidation = fixture
            .service
            .reconcile(5)
            .expect("history-only capture reconcile");
        assert_eq!(before_consolidation.rebuilt_project_count, 0);
        assert_eq!(before_consolidation.reused_project_count, 1);

        let consolidation = fixture
            .projects
            .preview_capture_consolidation(&project_id, &capture.capture_id, 6)
            .expect("consolidation can be previewed");
        fixture
            .projects
            .apply_capture_consolidation(
                &consolidation,
                &ApprovedCaptureConsolidation::new(
                    consolidation.preview().plan_digest.clone(),
                    true,
                    true,
                ),
            )
            .expect("consolidation can be applied");
        let reconciled = fixture
            .service
            .reconcile(7)
            .expect("consolidation reconcile");
        assert_eq!(reconciled.rebuilt_project_ids, vec![project_id]);
        assert_eq!(reconciled.reused_project_count, 0);
        fixture.assert_matches_clean_full(&reconciled.snapshot.portfolio);
    }

    #[test]
    fn rel_904_missing_index_rebuilds_only_derived_state() {
        let fixture = Fixture::new();
        let (_project_a, root_a) = fixture.create_project("Delete Derived A", 1);
        let (_project_b, root_b) = fixture.create_project("Delete Derived B", 2);
        let reconciled = fixture.service.reconcile(3).expect("catalog reconciles");
        let library_before = fixture.projects.snapshot().expect("library is readable");
        let project_a_before = project_bytes(&root_a);
        let project_b_before = project_bytes(&root_b);

        let plan = fixture
            .service
            .preview_delete_derived_state()
            .expect("delete can be previewed");
        assert_eq!(
            plan.preview().operation,
            PortfolioMaintenanceOperation::DeleteDerivedState
        );
        assert!(plan.preview().derived_state_only);
        assert_eq!(
            plan.preview().current_contribution_count,
            reconciled.snapshot.catalog.contribution_count
        );
        assert_eq!(
            fixture
                .service
                .apply_delete_derived_state(
                    &plan,
                    &ApprovedPortfolioMaintenance::new(plan.preview().plan_digest.clone(), false,),
                )
                .unwrap_err(),
            ProjectError::ApprovalRequired
        );
        let deletion = fixture
            .service
            .apply_delete_derived_state(
                &plan,
                &ApprovedPortfolioMaintenance::new(plan.preview().plan_digest.clone(), true),
            )
            .expect("derived state can be deleted");
        assert_eq!(
            deletion.removed_catalog_id.as_deref(),
            Some(reconciled.snapshot.catalog.catalog_id.as_str())
        );
        assert_eq!(deletion.removed_contribution_count, 2);
        assert_eq!(
            fixture.service.current().unwrap_err(),
            ProjectError::RecoveryRequired
        );
        assert_eq!(
            fixture
                .projects
                .snapshot()
                .expect("library remains readable"),
            library_before
        );
        assert_eq!(project_bytes(&root_a), project_a_before);
        assert_eq!(project_bytes(&root_b), project_b_before);

        let restarted_projects = ProjectStateService::new(fixture.config.clone());
        let restarted = IncrementalPortfolioService::new(restarted_projects);
        let rebuilt = restarted
            .reconcile(4)
            .expect("deleted derived state rebuilds after restart");
        assert_eq!(
            rebuilt.snapshot.portfolio.portfolio_id,
            reconciled.snapshot.portfolio.portfolio_id
        );
        assert_eq!(rebuilt.rebuilt_project_count, 2);
    }

    #[test]
    fn rel_904_corrupt_derived_state_can_be_deleted_and_rebuilt() {
        let fixture = Fixture::new();
        let (project_id, project_root) = fixture.create_project("Corrupt Derived", 1);
        let initial = fixture.service.reconcile(2).expect("catalog reconciles");
        let library_before = fixture.projects.snapshot().expect("library is readable");
        let project_before = project_bytes(&project_root);
        let contribution = fixture
            .config
            .state_root()
            .join("portfolio-catalog/v1/contributions")
            .join(format!("{}.json", project_id.as_str()));
        let contribution_before = fs::read(&contribution).expect("contribution is readable");
        fs::write(&contribution, b"{").expect("derived contribution can be corrupted");

        assert_eq!(
            fixture.service.current().unwrap_err(),
            ProjectError::InvalidPortfolioCatalog
        );
        let stale = fixture
            .service
            .preview_delete_derived_state()
            .expect("corrupt derived state can be previewed for deletion");
        fs::write(&contribution, contribution_before)
            .expect("valid derived contribution can be restored");
        assert_eq!(
            fixture
                .service
                .apply_delete_derived_state(
                    &stale,
                    &ApprovedPortfolioMaintenance::new(stale.preview().plan_digest.clone(), true),
                )
                .unwrap_err(),
            ProjectError::RevisionConflict
        );
        fs::write(&contribution, b"{").expect("derived contribution can be corrupted again");
        let plan = fixture
            .service
            .preview_delete_derived_state()
            .expect("current corruption can be previewed for deletion");
        assert_eq!(plan.preview().expected_catalog_id, None);
        assert_eq!(plan.preview().expected_catalog_generation, None);
        assert_eq!(
            fixture
                .service
                .apply_delete_derived_state(
                    &plan,
                    &ApprovedPortfolioMaintenance::new(plan.preview().plan_digest.clone(), false,),
                )
                .unwrap_err(),
            ProjectError::ApprovalRequired
        );
        fixture
            .service
            .apply_delete_derived_state(
                &plan,
                &ApprovedPortfolioMaintenance::new(plan.preview().plan_digest.clone(), true),
            )
            .expect("approved corrupt derived state can be deleted");
        assert_eq!(
            fixture.service.current().unwrap_err(),
            ProjectError::RecoveryRequired
        );
        assert_eq!(fixture.projects.snapshot().unwrap(), library_before);
        assert_eq!(project_bytes(&project_root), project_before);

        let restarted =
            IncrementalPortfolioService::new(ProjectStateService::new(fixture.config.clone()));
        let rebuilt = restarted
            .reconcile(3)
            .expect("catalog rebuilds from canonical projects");
        assert_eq!(rebuilt.snapshot.portfolio, initial.snapshot.portfolio);
        assert_eq!(fixture.projects.snapshot().unwrap(), library_before);
        assert_eq!(project_bytes(&project_root), project_before);
    }

    #[test]
    fn cancelled_and_stale_maintenance_publish_no_catalog_change() {
        let fixture = Fixture::new();
        fixture.create_project("Cancelled Maintenance", 1);
        let cancellation = PortfolioCancellationToken::new();
        cancellation.cancel();
        assert_eq!(
            fixture
                .service
                .reconcile_with_cancellation(2, &cancellation)
                .unwrap_err(),
            ProjectError::OperationCancelled
        );
        assert!(
            fixture
                .projects
                .portfolio_catalog_snapshot()
                .expect("catalog state is readable")
                .is_none()
        );

        let plan = fixture
            .service
            .preview_reconcile()
            .expect("reconcile can be previewed");
        fixture.create_project("Preview Drift", 3);
        assert_eq!(
            fixture
                .service
                .apply_reconcile(
                    &plan,
                    &ApprovedPortfolioMaintenance::new(plan.preview().plan_digest.clone(), true),
                    4,
                    &PortfolioCancellationToken::new(),
                )
                .unwrap_err(),
            ProjectError::RevisionConflict
        );
        assert!(
            fixture
                .projects
                .portfolio_catalog_snapshot()
                .expect("catalog state is readable")
                .is_none()
        );
    }

    #[test]
    fn maintenance_apply_and_doctor_compare_incremental_with_clean_bytes() {
        let fixture = Fixture::new();
        fixture.create_project("Maintenance Doctor", 1);
        let plan = fixture
            .service
            .preview_reconcile()
            .expect("reconcile can be previewed");
        let reconciled = fixture
            .service
            .apply_reconcile(
                &plan,
                &ApprovedPortfolioMaintenance::new(plan.preview().plan_digest.clone(), true),
                2,
                &PortfolioCancellationToken::new(),
            )
            .expect("approved reconcile succeeds");
        assert_eq!(reconciled.rebuilt_project_count, 1);
        let doctor = fixture
            .service
            .doctor_compare()
            .expect("doctor comparison succeeds");
        assert_eq!(doctor.status, PortfolioDoctorStatus::Equivalent);
        assert!(doctor.byte_equivalent);
        assert_eq!(
            doctor.incremental_portfolio_id.as_deref(),
            Some(reconciled.snapshot.portfolio.portfolio_id.as_str())
        );
        assert_eq!(
            doctor.clean_portfolio_id,
            reconciled.snapshot.portfolio.portfolio_id
        );
    }

    fn project_bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
            let mut entries = fs::read_dir(directory)
                .expect("project directory is readable")
                .collect::<Result<Vec<_>, _>>()
                .expect("project entries are readable");
            entries.sort_by_key(fs::DirEntry::file_name);
            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    collect(root, &path, files);
                } else {
                    files.insert(
                        path.strip_prefix(root)
                            .expect("path is beneath project")
                            .to_path_buf(),
                        fs::read(path).expect("project file is readable"),
                    );
                }
            }
        }
        let mut files = BTreeMap::new();
        collect(root, root, &mut files);
        files
    }
}
