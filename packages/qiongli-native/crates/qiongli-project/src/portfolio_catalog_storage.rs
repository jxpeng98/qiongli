use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use qiongli_config::ConfigRoot;

use crate::portfolio_catalog::{
    MAX_PORTFOLIO_CATALOG_MANIFEST_BYTES, MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES,
    MAX_PORTFOLIO_CONTRIBUTION_BYTES, PortfolioCatalogManifestV1, PortfolioCatalogSnapshotV1,
    PortfolioCatalogTransactionV1, PortfolioContributionV1,
};
use crate::storage::{
    acquire_lock, atomic_write, ensure_private_directory_beneath, prepare_private_state_directory,
    project_metadata_if_exists, read_bounded_project_file, remove_private_state_file, sha256_bytes,
};
use crate::{ProjectError, ProjectId};

const PORTFOLIO_CATALOG_DIRECTORY: &str = "portfolio-catalog";
const PORTFOLIO_CATALOG_STORAGE_VERSION: &str = "v1";
const PORTFOLIO_CONTRIBUTIONS_DIRECTORY: &str = "contributions";
const PORTFOLIO_TRANSACTIONS_DIRECTORY: &str = "transactions";
const PORTFOLIO_CATALOG_FILE: &str = "catalog.json";
const PORTFOLIO_CATALOG_LOCK_FILE: &str = ".catalog.lock";
const JSON_SUFFIX: &str = ".json";

#[derive(Clone)]
pub(crate) struct PortfolioCatalogStore {
    config_root: ConfigRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredPortfolioCatalog {
    pub(crate) manifest: PortfolioCatalogManifestV1,
    pub(crate) contributions: Vec<PortfolioContributionV1>,
    pub(crate) snapshot: PortfolioCatalogSnapshotV1,
}

struct CatalogPaths {
    state_root: PathBuf,
    root: PathBuf,
    contributions: PathBuf,
    transactions: PathBuf,
}

impl PortfolioCatalogStore {
    pub(crate) const fn new(config_root: ConfigRoot) -> Self {
        Self { config_root }
    }

    pub(crate) fn rebuild(&self) -> Result<Option<StoredPortfolioCatalog>, ProjectError> {
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(PORTFOLIO_CATALOG_LOCK_FILE))?;
        recover_transactions_locked(&paths)?;
        rebuild_catalog_locked(&paths)
    }

    // C3.2 calls this after authoritative-state reconciliation. C3.1 keeps
    // direct publication private and covers it through storage tests only.
    #[allow(dead_code)]
    pub(crate) fn commit(
        &self,
        transaction: &PortfolioCatalogTransactionV1,
    ) -> Result<StoredPortfolioCatalog, ProjectError> {
        transaction.validate()?;
        let paths = self.prepare()?;
        let _lock = acquire_lock(&paths.root.join(PORTFOLIO_CATALOG_LOCK_FILE))?;
        recover_transactions_locked(&paths)?;
        let current = rebuild_catalog_locked(&paths)?;
        if current
            .as_ref()
            .is_some_and(|catalog| catalog.manifest == transaction.next_manifest)
        {
            return current.ok_or(ProjectError::RecoveryRequired);
        }
        if current.as_ref().map(|catalog| &catalog.manifest)
            != transaction.previous_manifest.as_ref()
        {
            return Err(ProjectError::PortfolioCatalogConflict);
        }

        write_transaction(&paths, transaction)?;
        interrupt_after(CatalogDurableBoundary::Transaction)?;
        complete_transaction_locked(&paths, transaction)?;
        rebuild_catalog_locked(&paths)?.ok_or(ProjectError::RecoveryRequired)
    }

    fn prepare(&self) -> Result<CatalogPaths, ProjectError> {
        let root = prepare_private_state_directory(
            &self.config_root,
            &[
                PORTFOLIO_CATALOG_DIRECTORY,
                PORTFOLIO_CATALOG_STORAGE_VERSION,
            ],
        )?;
        let state_root = self.config_root.state_root().to_path_buf();
        let contributions = root.join(PORTFOLIO_CONTRIBUTIONS_DIRECTORY);
        let transactions = root.join(PORTFOLIO_TRANSACTIONS_DIRECTORY);
        ensure_private_directory_beneath(&state_root, &contributions)?;
        ensure_private_directory_beneath(&state_root, &transactions)?;
        Ok(CatalogPaths {
            state_root,
            root,
            contributions,
            transactions,
        })
    }

    #[cfg(test)]
    fn root_for_test(&self) -> PathBuf {
        self.config_root
            .state_root()
            .join(PORTFOLIO_CATALOG_DIRECTORY)
            .join(PORTFOLIO_CATALOG_STORAGE_VERSION)
    }
}

impl Debug for PortfolioCatalogStore {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PortfolioCatalogStore")
            .field("root", &"<private-derived-state>")
            .finish()
    }
}

fn recover_transactions_locked(paths: &CatalogPaths) -> Result<(), ProjectError> {
    cleanup_owned_staging_files(paths)?;
    validate_root_entries(paths)?;
    let _ = list_contribution_project_ids(paths)?;
    let transaction_ids = list_transaction_ids(paths)?;
    if transaction_ids.len() > 1 {
        return Err(ProjectError::RecoveryRequired);
    }
    let Some(transaction_id) = transaction_ids.first() else {
        return Ok(());
    };
    let transaction =
        read_transaction(paths, transaction_id)?.ok_or(ProjectError::RecoveryRequired)?;
    complete_transaction_locked(paths, &transaction)
}

fn complete_transaction_locked(
    paths: &CatalogPaths,
    transaction: &PortfolioCatalogTransactionV1,
) -> Result<(), ProjectError> {
    transaction.validate()?;
    let current = read_manifest(paths)?;
    if current.as_ref() != transaction.previous_manifest.as_ref()
        && current.as_ref() != Some(&transaction.next_manifest)
    {
        return Err(ProjectError::PortfolioCatalogConflict);
    }

    for contribution in &transaction.replacements {
        write_contribution(paths, contribution)?;
    }
    interrupt_after(CatalogDurableBoundary::Contributions)?;

    if current.as_ref() != Some(&transaction.next_manifest) {
        let bytes = transaction.next_manifest.to_canonical_json()?;
        atomic_write(&paths.root, PORTFOLIO_CATALOG_FILE, &bytes, true)?;
    }
    interrupt_after(CatalogDurableBoundary::Manifest)?;

    for project_id in &transaction.removals {
        remove_private_state_file(
            &paths.state_root,
            &paths.contributions.join(contribution_file_name(project_id)),
            MAX_PORTFOLIO_CONTRIBUTION_BYTES,
        )?;
    }
    interrupt_after(CatalogDurableBoundary::Removals)?;

    remove_private_state_file(
        &paths.state_root,
        &paths
            .transactions
            .join(transaction_file_name(&transaction.transaction_id)),
        MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES,
    )
}

fn rebuild_catalog_locked(
    paths: &CatalogPaths,
) -> Result<Option<StoredPortfolioCatalog>, ProjectError> {
    cleanup_owned_staging_files(paths)?;
    validate_root_entries(paths)?;
    if !list_transaction_ids(paths)?.is_empty() {
        return Err(ProjectError::RecoveryRequired);
    }
    let contribution_ids = list_contribution_project_ids(paths)?;
    let Some(manifest) = read_manifest(paths)? else {
        if contribution_ids.is_empty() {
            return Ok(None);
        }
        return Err(ProjectError::InvalidPortfolioCatalog);
    };
    let expected_ids = manifest
        .contributions
        .iter()
        .map(|contribution| contribution.project_id.clone())
        .collect::<Vec<_>>();
    if contribution_ids != expected_ids {
        return Err(ProjectError::InvalidPortfolioCatalog);
    }

    let mut contributions = Vec::with_capacity(manifest.contributions.len());
    let mut node_count = 0usize;
    let mut edge_count = 0usize;
    let mut diagnostic_count = 0usize;
    for expected in &manifest.contributions {
        let (contribution, bytes) = read_contribution(paths, &expected.project_id)?
            .ok_or(ProjectError::InvalidPortfolioCatalog)?;
        if contribution.project_id != expected.project_id
            || contribution.semantic_revision != expected.semantic_revision
            || contribution.projection_id != expected.projection_id
            || contribution.contribution_id != expected.contribution_id
            || sha256_bytes(&bytes) != expected.contribution_sha256
        {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        node_count = node_count
            .checked_add(contribution.node_count)
            .ok_or(ProjectError::InvalidPortfolioCatalog)?;
        edge_count = edge_count
            .checked_add(contribution.edge_count)
            .ok_or(ProjectError::InvalidPortfolioCatalog)?;
        diagnostic_count = diagnostic_count
            .checked_add(contribution.diagnostic_count)
            .ok_or(ProjectError::InvalidPortfolioCatalog)?;
        contributions.push(contribution);
    }
    let snapshot = PortfolioCatalogSnapshotV1 {
        schema_version: manifest.schema_version,
        document_kind: crate::PORTFOLIO_CATALOG_SNAPSHOT_DOCUMENT_KIND.to_string(),
        catalog_id: manifest.catalog_id.clone(),
        generation: manifest.generation,
        library_revision: manifest.library_revision,
        created_at_unix: manifest.created_at_unix,
        contribution_count: contributions.len(),
        node_count,
        edge_count,
        diagnostic_count,
        contributions: manifest.contributions.clone(),
    };
    Ok(Some(StoredPortfolioCatalog {
        manifest,
        contributions,
        snapshot,
    }))
}

fn read_manifest(paths: &CatalogPaths) -> Result<Option<PortfolioCatalogManifestV1>, ProjectError> {
    let path = paths.root.join(PORTFOLIO_CATALOG_FILE);
    let Some(bytes) = read_private_document(paths, &path, MAX_PORTFOLIO_CATALOG_MANIFEST_BYTES)?
    else {
        return Ok(None);
    };
    PortfolioCatalogManifestV1::from_json_slice(&bytes).map(Some)
}

fn read_contribution(
    paths: &CatalogPaths,
    project_id: &ProjectId,
) -> Result<Option<(PortfolioContributionV1, Vec<u8>)>, ProjectError> {
    let path = paths.contributions.join(contribution_file_name(project_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_PORTFOLIO_CONTRIBUTION_BYTES)? else {
        return Ok(None);
    };
    let contribution = PortfolioContributionV1::from_json_slice(&bytes)?;
    if contribution.project_id != *project_id {
        return Err(ProjectError::InvalidPortfolioCatalog);
    }
    Ok(Some((contribution, bytes)))
}

fn read_transaction(
    paths: &CatalogPaths,
    transaction_id: &str,
) -> Result<Option<PortfolioCatalogTransactionV1>, ProjectError> {
    if !valid_transaction_id(transaction_id) {
        return Err(ProjectError::InvalidPortfolioCatalog);
    }
    let path = paths
        .transactions
        .join(transaction_file_name(transaction_id));
    let Some(bytes) = read_private_document(paths, &path, MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES)?
    else {
        return Ok(None);
    };
    let transaction = PortfolioCatalogTransactionV1::from_json_slice(&bytes)?;
    if transaction.transaction_id != transaction_id {
        return Err(ProjectError::InvalidPortfolioCatalog);
    }
    Ok(Some(transaction))
}

fn read_private_document(
    paths: &CatalogPaths,
    path: &Path,
    maximum_bytes: usize,
) -> Result<Option<Vec<u8>>, ProjectError> {
    let Some(metadata) = project_metadata_if_exists(&paths.state_root, path)? else {
        return Ok(None);
    };
    read_bounded_project_file(&paths.state_root, path, &metadata, maximum_bytes, true).map(Some)
}

fn write_contribution(
    paths: &CatalogPaths,
    contribution: &PortfolioContributionV1,
) -> Result<(), ProjectError> {
    let bytes = contribution.to_canonical_json()?;
    atomic_write(
        &paths.contributions,
        &contribution_file_name(&contribution.project_id),
        &bytes,
        true,
    )
}

#[allow(dead_code)]
fn write_transaction(
    paths: &CatalogPaths,
    transaction: &PortfolioCatalogTransactionV1,
) -> Result<(), ProjectError> {
    let bytes = transaction.to_canonical_json()?;
    let file_name = transaction_file_name(&transaction.transaction_id);
    let path = paths.transactions.join(&file_name);
    if let Some(existing) =
        read_private_document(paths, &path, MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES)?
    {
        if existing == bytes {
            return Ok(());
        }
        return Err(ProjectError::PortfolioCatalogConflict);
    }
    atomic_write(&paths.transactions, &file_name, &bytes, true)
}

fn validate_root_entries(paths: &CatalogPaths) -> Result<(), ProjectError> {
    for entry in read_directory(&paths.root)? {
        let name = entry
            .file_name()
            .to_str()
            .ok_or(ProjectError::InvalidPortfolioCatalog)?
            .to_string();
        if matches!(
            name.as_str(),
            PORTFOLIO_CONTRIBUTIONS_DIRECTORY
                | PORTFOLIO_TRANSACTIONS_DIRECTORY
                | PORTFOLIO_CATALOG_FILE
                | PORTFOLIO_CATALOG_LOCK_FILE
        ) {
            continue;
        }
        return Err(ProjectError::InvalidPortfolioCatalog);
    }
    Ok(())
}

fn list_contribution_project_ids(paths: &CatalogPaths) -> Result<Vec<ProjectId>, ProjectError> {
    let mut project_ids = Vec::new();
    for entry in read_directory(&paths.contributions)? {
        let name = entry
            .file_name()
            .to_str()
            .ok_or(ProjectError::InvalidPortfolioCatalog)?
            .to_string();
        let value = name
            .strip_suffix(JSON_SUFFIX)
            .ok_or(ProjectError::InvalidPortfolioCatalog)?;
        project_ids.push(
            ProjectId::parse(value.to_string())
                .map_err(|_| ProjectError::InvalidPortfolioCatalog)?,
        );
    }
    project_ids.sort_unstable();
    if project_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ProjectError::InvalidPortfolioCatalog);
    }
    Ok(project_ids)
}

fn list_transaction_ids(paths: &CatalogPaths) -> Result<Vec<String>, ProjectError> {
    let mut transaction_ids = Vec::new();
    for entry in read_directory(&paths.transactions)? {
        let name = entry
            .file_name()
            .to_str()
            .ok_or(ProjectError::InvalidPortfolioCatalog)?
            .to_string();
        let value = name
            .strip_suffix(JSON_SUFFIX)
            .ok_or(ProjectError::InvalidPortfolioCatalog)?;
        if !valid_transaction_id(value) {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        transaction_ids.push(value.to_string());
    }
    transaction_ids.sort_unstable();
    transaction_ids.dedup();
    Ok(transaction_ids)
}

fn cleanup_owned_staging_files(paths: &CatalogPaths) -> Result<(), ProjectError> {
    cleanup_staging_in_directory(
        paths,
        &paths.root,
        &[PORTFOLIO_CATALOG_FILE],
        MAX_PORTFOLIO_CATALOG_MANIFEST_BYTES,
    )?;
    cleanup_staging_in_directory(
        paths,
        &paths.contributions,
        &[],
        MAX_PORTFOLIO_CONTRIBUTION_BYTES,
    )?;
    cleanup_staging_in_directory(
        paths,
        &paths.transactions,
        &[],
        MAX_PORTFOLIO_CATALOG_TRANSACTION_BYTES,
    )
}

fn cleanup_staging_in_directory(
    paths: &CatalogPaths,
    directory: &Path,
    fixed_targets: &[&str],
    maximum_bytes: usize,
) -> Result<(), ProjectError> {
    for entry in read_directory(directory)? {
        let name = entry
            .file_name()
            .to_str()
            .ok_or(ProjectError::InvalidPortfolioCatalog)?
            .to_string();
        let Some(target) = owned_staging_target(&name) else {
            continue;
        };
        let valid_target = fixed_targets.contains(&target)
            || target.strip_suffix(JSON_SUFFIX).is_some_and(|identity| {
                ProjectId::parse(identity.to_string()).is_ok() || valid_transaction_id(identity)
            });
        if !valid_target {
            return Err(ProjectError::InvalidPortfolioCatalog);
        }
        remove_private_state_file(&paths.state_root, &directory.join(name), maximum_bytes)?;
    }
    Ok(())
}

fn owned_staging_target(name: &str) -> Option<&str> {
    let without_dot = name.strip_prefix('.')?;
    let (target, token) = without_dot.rsplit_once(".qiongli-stage-")?;
    (token.len() == 24
        && token
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()))
    .then_some(target)
}

fn read_directory(path: &Path) -> Result<Vec<fs::DirEntry>, ProjectError> {
    fs::read_dir(path)
        .map_err(map_io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_io)
}

fn contribution_file_name(project_id: &ProjectId) -> String {
    format!("{}{JSON_SUFFIX}", project_id.as_str())
}

fn transaction_file_name(transaction_id: &str) -> String {
    format!("{transaction_id}{JSON_SUFFIX}")
}

fn valid_transaction_id(value: &str) -> bool {
    value.strip_prefix("ptx_").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn map_io(error: io::Error) -> ProjectError {
    ProjectError::PersistenceFailed(error.kind())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CatalogDurableBoundary {
    #[allow(dead_code)]
    Transaction,
    Contributions,
    Manifest,
    Removals,
}

#[cfg(not(test))]
fn interrupt_after(_boundary: CatalogDurableBoundary) -> Result<(), ProjectError> {
    Ok(())
}

#[cfg(test)]
thread_local! {
    static INTERRUPT_AFTER: std::cell::Cell<Option<CatalogDurableBoundary>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn interrupt_after(boundary: CatalogDurableBoundary) -> Result<(), ProjectError> {
    INTERRUPT_AFTER.with(|selected| {
        if selected.get() == Some(boundary) {
            selected.set(None);
            Err(ProjectError::RecoveryRequired)
        } else {
            Ok(())
        }
    })
}

#[cfg(test)]
fn set_interrupt_after(boundary: CatalogDurableBoundary) {
    INTERRUPT_AFTER.with(|selected| selected.set(Some(boundary)));
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use qiongli_config::{ConfigRoot, resolve_config_root};

    use super::*;
    use crate::{
        AcademicGraphService, ApprovedProjectMutation, PortfolioContributionV1, ProjectHealth,
        ProjectKind, ProjectRegistrationOptions, ProjectStateService,
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        config: ConfigRoot,
        projects: ProjectStateService,
        store: PortfolioCatalogStore,
    }

    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is available")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "qiongli-portfolio-catalog-{}-{nonce}-{}",
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
            let store = PortfolioCatalogStore::new(config.clone());
            Self {
                root,
                config,
                projects,
                store,
            }
        }

        fn create_project(&self, name: &str, now_unix: u64) -> (ProjectId, PathBuf) {
            let project_root = self.root.join(name.to_lowercase().replace(' ', "-"));
            let plan = self
                .projects
                .preview_create(
                    &project_root,
                    ProjectRegistrationOptions::new(name, ProjectKind::Article),
                    now_unix,
                )
                .expect("project create can be previewed");
            let project_id = plan.preview().project_id.clone();
            self.projects
                .apply(
                    &plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    now_unix,
                )
                .expect("project create can be applied");
            (project_id, project_root)
        }

        fn contribution(&self, project_id: &ProjectId) -> PortfolioContributionV1 {
            let graph = AcademicGraphService::new(self.projects.clone())
                .rebuild(project_id)
                .expect("graph can be rebuilt");
            PortfolioContributionV1::from_graph(graph, ProjectHealth::Ready)
                .expect("ready graph is a valid contribution")
        }

        fn refresh(
            &self,
            project_id: &ProjectId,
            project_root: &Path,
            now_unix: u64,
            marker: &str,
        ) {
            fs::write(
                project_root.join("context/research_state.md"),
                format!("- main_question_or_thesis: {marker}\n"),
            )
            .expect("semantic artifact can be updated");
            let plan = self
                .projects
                .preview_refresh(project_id, now_unix)
                .expect("refresh can be previewed");
            self.projects
                .apply(
                    &plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    now_unix,
                )
                .expect("refresh can be applied");
        }

        fn transaction(
            &self,
            previous: Option<PortfolioCatalogManifestV1>,
            replacements: Vec<PortfolioContributionV1>,
            removals: Vec<ProjectId>,
            now_unix: u64,
        ) -> PortfolioCatalogTransactionV1 {
            PortfolioCatalogTransactionV1::new(
                previous,
                replacements,
                removals,
                self.projects
                    .snapshot()
                    .expect("library can be inspected")
                    .revision,
                now_unix,
            )
            .expect("catalog transaction is valid")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn catalog_inserts_replays_reopens_replaces_and_removes_exact_contributions() {
        let fixture = Fixture::new();
        assert!(
            fixture
                .store
                .rebuild()
                .expect("empty catalog is valid")
                .is_none()
        );
        let (project_id, project_root) = fixture.create_project("Project A", 1);
        let initial = fixture.contribution(&project_id);
        let insert = fixture.transaction(None, vec![initial.clone()], Vec::new(), 2);

        let inserted = fixture.store.commit(&insert).expect("insert commits");
        assert_eq!(inserted.snapshot.contribution_count, 1);
        assert_eq!(inserted.contributions, vec![initial.clone()]);
        assert!(
            !serde_json::to_string(&inserted.snapshot)
                .expect("snapshot serializes")
                .contains(fixture.root.to_string_lossy().as_ref())
        );
        assert_eq!(
            fixture
                .store
                .commit(&insert)
                .expect("exact replay succeeds"),
            inserted
        );
        let reopened = PortfolioCatalogStore::new(fixture.config.clone())
            .rebuild()
            .expect("catalog reopens")
            .expect("catalog remains present");
        assert_eq!(reopened, inserted);

        fixture.refresh(&project_id, &project_root, 3, "What changed?");
        let replacement = fixture.contribution(&project_id);
        assert_ne!(replacement.contribution_id, initial.contribution_id);
        let replace = fixture.transaction(
            Some(inserted.manifest),
            vec![replacement.clone()],
            Vec::new(),
            4,
        );
        let replaced = fixture.store.commit(&replace).expect("replacement commits");
        assert_eq!(replaced.contributions, vec![replacement]);
        assert_eq!(replaced.snapshot.generation, 2);

        let remove = fixture.transaction(Some(replaced.manifest), Vec::new(), vec![project_id], 5);
        let removed = fixture.store.commit(&remove).expect("removal commits");
        assert_eq!(removed.snapshot.contribution_count, 0);
        assert!(removed.contributions.is_empty());
        assert!(
            fs::read_dir(
                fixture
                    .store
                    .root_for_test()
                    .join(PORTFOLIO_CONTRIBUTIONS_DIRECTORY)
            )
            .expect("contribution directory is readable")
            .next()
            .is_none()
        );
    }

    #[test]
    fn every_durable_catalog_boundary_recovers_to_the_exact_next_manifest() {
        for boundary in [
            CatalogDurableBoundary::Transaction,
            CatalogDurableBoundary::Contributions,
            CatalogDurableBoundary::Manifest,
            CatalogDurableBoundary::Removals,
        ] {
            let fixture = Fixture::new();
            let (project_id, _) = fixture.create_project("Boundary Project", 1);
            let transaction =
                fixture.transaction(None, vec![fixture.contribution(&project_id)], Vec::new(), 2);
            set_interrupt_after(boundary);
            assert_eq!(
                fixture.store.commit(&transaction).unwrap_err(),
                ProjectError::RecoveryRequired
            );

            let recovered = PortfolioCatalogStore::new(fixture.config.clone())
                .rebuild()
                .expect("restart completes staged transaction")
                .expect("recovered catalog is present");
            assert_eq!(recovered.manifest, transaction.next_manifest);
            assert_eq!(recovered.snapshot.contribution_count, 1);
            assert!(
                fs::read_dir(
                    fixture
                        .store
                        .root_for_test()
                        .join(PORTFOLIO_TRANSACTIONS_DIRECTORY)
                )
                .expect("transaction directory is readable")
                .next()
                .is_none()
            );
        }
    }

    #[test]
    fn catalog_rejects_corruption_unknown_files_and_noncanonical_json() {
        let fixture = Fixture::new();
        let (project_id, _) = fixture.create_project("Corruption Project", 1);
        let transaction =
            fixture.transaction(None, vec![fixture.contribution(&project_id)], Vec::new(), 2);
        fixture.store.commit(&transaction).expect("catalog commits");
        let root = fixture.store.root_for_test();
        fs::write(root.join("unexpected.txt"), b"unexpected").expect("unknown file can be created");
        assert_eq!(
            fixture.store.rebuild().unwrap_err(),
            ProjectError::InvalidPortfolioCatalog
        );
        fs::remove_file(root.join("unexpected.txt")).expect("unknown file can be removed");

        let contribution_path = root
            .join(PORTFOLIO_CONTRIBUTIONS_DIRECTORY)
            .join(contribution_file_name(&project_id));
        let canonical = fs::read(&contribution_path).expect("contribution is readable");
        fs::remove_file(&contribution_path).expect("contribution can be removed");
        assert_eq!(
            fixture.store.rebuild().unwrap_err(),
            ProjectError::InvalidPortfolioCatalog
        );
        atomic_write(
            contribution_path
                .parent()
                .expect("contribution has a parent"),
            contribution_path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("contribution filename is UTF-8"),
            &canonical,
            true,
        )
        .expect("contribution can be restored");

        let (extra_project_id, _) = fixture.create_project("Unexpected Project", 3);
        let extra = fixture.contribution(&extra_project_id);
        let extra_path = root
            .join(PORTFOLIO_CONTRIBUTIONS_DIRECTORY)
            .join(contribution_file_name(&extra_project_id));
        fs::write(
            &extra_path,
            extra
                .to_canonical_json()
                .expect("extra contribution serializes"),
        )
        .expect("extra contribution can be written");
        assert_eq!(
            fixture.store.rebuild().unwrap_err(),
            ProjectError::InvalidPortfolioCatalog
        );
        fs::remove_file(extra_path).expect("extra contribution can be removed");

        let mut value: serde_json::Value =
            serde_json::from_slice(&canonical).expect("contribution is JSON");
        value
            .as_object_mut()
            .expect("contribution is an object")
            .insert("unknown".to_string(), serde_json::Value::Bool(true));
        fs::write(
            &contribution_path,
            serde_json::to_vec_pretty(&value).expect("tampered JSON serializes"),
        )
        .expect("contribution can be tampered");
        assert_eq!(
            fixture.store.rebuild().unwrap_err(),
            ProjectError::InvalidPortfolioCatalog
        );
    }

    #[test]
    fn stale_catalog_generation_conflicts_without_leaving_a_transaction() {
        let fixture = Fixture::new();
        let (project_id, _) = fixture.create_project("Generation Project", 1);
        let contribution = fixture.contribution(&project_id);
        let first = fixture.transaction(None, vec![contribution.clone()], Vec::new(), 2);
        fixture.store.commit(&first).expect("first catalog commits");
        let stale = fixture.transaction(None, vec![contribution], Vec::new(), 3);

        assert_eq!(
            fixture.store.commit(&stale).unwrap_err(),
            ProjectError::PortfolioCatalogConflict
        );
        assert!(
            fs::read_dir(
                fixture
                    .store
                    .root_for_test()
                    .join(PORTFOLIO_TRANSACTIONS_DIRECTORY)
            )
            .expect("transaction directory is readable")
            .next()
            .is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    fn catalog_rejects_links_and_broadened_private_file_permissions() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        for fault in ["symlink", "hardlink", "permissions"] {
            let fixture = Fixture::new();
            let (project_id, _) = fixture.create_project("Private Project", 1);
            let transaction =
                fixture.transaction(None, vec![fixture.contribution(&project_id)], Vec::new(), 2);
            fixture.store.commit(&transaction).expect("catalog commits");
            let contribution_path = fixture
                .store
                .root_for_test()
                .join(PORTFOLIO_CONTRIBUTIONS_DIRECTORY)
                .join(contribution_file_name(&project_id));
            match fault {
                "symlink" => {
                    let outside = fixture.root.join("outside.json");
                    fs::copy(&contribution_path, &outside).expect("copy succeeds");
                    fs::remove_file(&contribution_path).expect("contribution can be removed");
                    symlink(&outside, &contribution_path).expect("symlink can be created");
                }
                "hardlink" => {
                    fs::hard_link(&contribution_path, fixture.root.join("alias.json"))
                        .expect("hard link can be created");
                }
                "permissions" => {
                    let mut permissions = fs::metadata(&contribution_path)
                        .expect("metadata is available")
                        .permissions();
                    permissions.set_mode(0o644);
                    fs::set_permissions(&contribution_path, permissions)
                        .expect("permissions can be broadened");
                }
                _ => unreachable!(),
            }
            assert_eq!(
                fixture.store.rebuild().unwrap_err(),
                ProjectError::UnsafeProjectRoot
            );
        }
    }

    #[test]
    fn catalog_lock_contention_is_reported_without_publishing() {
        let fixture = Fixture::new();
        fixture
            .store
            .rebuild()
            .expect("catalog layout can be prepared");
        let root = fixture.store.root_for_test();
        let _lock =
            acquire_lock(&root.join(PORTFOLIO_CATALOG_LOCK_FILE)).expect("test lock is acquired");
        assert_eq!(fixture.store.rebuild().unwrap_err(), ProjectError::LockBusy);
        assert!(!root.join(PORTFOLIO_CATALOG_FILE).exists());
    }

    #[test]
    fn deleting_private_catalog_state_changes_no_library_or_project_bytes() {
        let fixture = Fixture::new();
        let (project_id, project_root) = fixture.create_project("Deletion Project", 1);
        let transaction =
            fixture.transaction(None, vec![fixture.contribution(&project_id)], Vec::new(), 2);
        fixture.store.commit(&transaction).expect("catalog commits");
        let library_before = fixture.projects.snapshot().expect("library is readable");
        let manifest_path = project_root.join("context/project_manifest.json");
        let manifest_before = fs::read(&manifest_path).expect("manifest is readable");

        fs::remove_dir_all(fixture.store.root_for_test())
            .expect("private derived catalog can be deleted");

        assert_eq!(
            fixture
                .projects
                .snapshot()
                .expect("library remains readable"),
            library_before
        );
        assert_eq!(
            fs::read(&manifest_path).expect("manifest remains readable"),
            manifest_before
        );
        assert!(
            fixture
                .projects
                .portfolio_catalog_snapshot()
                .expect("deleted catalog is rebuildable empty state")
                .is_none()
        );
    }
}
