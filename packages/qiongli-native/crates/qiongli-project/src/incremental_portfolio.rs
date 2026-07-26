use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::academic_graph_portfolio::{build_portfolio, portfolio_project_is_included};
use crate::model::{MAX_SEMANTIC_REVISION, RegisteredProjectV1, ResearchLibraryDocumentV1};
use crate::portfolio_catalog::{
    PortfolioCatalogSnapshotV1, PortfolioCatalogTransactionV1, PortfolioContributionRefV1,
    PortfolioContributionV1,
};
use crate::portfolio_catalog_storage::StoredPortfolioCatalog;
use crate::{
    AcademicGraphPortfolioSnapshotV1, AcademicGraphService, ArticleProjectSummaryV1, ProjectError,
    ProjectHealth, ProjectId, ProjectStateService, ResearchLibrarySnapshotV1,
};

pub const INCREMENTAL_PORTFOLIO_SCHEMA_VERSION: u32 = 1;
pub const INCREMENTAL_PORTFOLIO_SNAPSHOT_DOCUMENT_KIND: &str =
    "qiongli-incremental-portfolio-snapshot";
pub const PORTFOLIO_RECONCILIATION_DOCUMENT_KIND: &str = "qiongli-portfolio-reconciliation";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortfolioReconciliationMode {
    Incremental,
    Full,
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
        self.reconcile_with_mode(PortfolioReconciliationMode::Incremental, now_unix)
    }

    pub fn rebuild_full(&self, now_unix: u64) -> Result<PortfolioReconciliationV1, ProjectError> {
        self.reconcile_with_mode(PortfolioReconciliationMode::Full, now_unix)
    }

    fn reconcile_with_mode(
        &self,
        mode: PortfolioReconciliationMode,
        now_unix: u64,
    ) -> Result<PortfolioReconciliationV1, ProjectError> {
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
                    let contribution = rebuild_contribution(&graph_service, summary, entry)?;
                    rebuilt_project_ids.push(summary.project_id.clone());
                    replacements.push(contribution.clone());
                    contribution
                }
            } else {
                let contribution = rebuild_contribution(&graph_service, summary, entry)?;
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
        let portfolio = portfolio_from_catalog(&library, &catalog)?;

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
) -> Result<PortfolioContributionV1, ProjectError> {
    let graph = graph_service.rebuild(&summary.project_id)?;
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
}
