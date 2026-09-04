use std::collections::BTreeSet;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use qiongli_config::resolve_config_root;
use serde::Serialize;

use crate::academic_graph::{MAX_GRAPH_EDGES, MAX_GRAPH_NODES, canonical_domain_digest};
use crate::academic_graph_index::AcademicGraphIndexV1;
use crate::academic_graph_portfolio::{
    MAX_PORTFOLIO_EDGES, MAX_PORTFOLIO_NODES, MAX_PORTFOLIO_OCCURRENCES, build_portfolio,
    validate_portfolio_capacity,
};
use crate::model::{MAX_LIBRARY_PROJECTS, RegisteredProjectV1};
use crate::portable::{MAX_PORTABLE_FILES, migration_inventory};
use crate::storage::{
    GRAPH_SEMANTIC_LINKS_RELATIVE_PATH, MAX_CAPTURE_DOCUMENTS, SEMANTIC_ARTIFACTS,
    create_project_root, empty_semantic_digest, list_capture_documents, project_root_string,
    sha256_bytes, write_manifest,
};
use crate::{
    ACADEMIC_GRAPH_DOCUMENT_KIND, ACADEMIC_GRAPH_SCHEMA_VERSION, AcademicGraphConfidence,
    AcademicGraphEdgeStatus, AcademicGraphEdgeV1, AcademicGraphIdentityScope, AcademicGraphLayer,
    AcademicGraphNodeType, AcademicGraphNodeV1, AcademicGraphQueryV1, AcademicGraphRelation,
    AcademicGraphSnapshotV1, AcademicGraphSourceKind, AcademicGraphSourceRefV1,
    AcademicInferenceStrength, ApprovedProjectMutation, ArticleProjectManifestV1,
    ArticleProjectSummaryV1, CaptureDelivery, CapturePolicy, CaptureSource, LibraryHealth,
    ProjectBindingV1, ProjectError, ProjectHealth, ProjectId, ProjectKind, ProjectLifecycle,
    ProjectMutationEffect, ProjectNextAction, ProjectOverviewV1, ProjectRegistrationOptions,
    ProjectStage, ProjectStateService, RESEARCH_LIBRARY_SCHEMA_VERSION, ResearchCaptureDraftV1,
    ResearchLibrarySnapshotV1,
};

const RECEIPT_VERSION: &str = "qiongli-platform-capacity/v1";
const FIXTURE_VERSION: &[u8] = b"qiongli-project-capacity-fixture/v1\0";
const SAMPLE_COUNT: usize = 20;
const PROFILE_NAMES: [&str; 3] = ["small", "medium", "product-limit"];
const LIBRARY_PROFILE_SIZES: [usize; 3] = [3, 64, 512];
const CAPTURE_PROFILE_SIZES: [usize; 3] = [8, 128, 1_024];
const GRAPH_PROFILE_SIZES: [(usize, usize); 3] = [(64, 64), (1_024, 1_024), (4_096, 4_096)];
const PORTFOLIO_PROFILE_SIZES: [usize; 3] = [3, 64, 512];
const PORTABLE_PROFILE_SIZES: [usize; 3] = [8, 128, 1_024];
const PROJECT_MANIFEST_PATH: &str = "context/project_manifest.json";

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Eq, PartialEq, Serialize)]
struct ProfileSizes {
    small: usize,
    medium: usize,
    product_limit: usize,
}

impl ProfileSizes {
    const fn new(values: [usize; 3]) -> Self {
        Self {
            small: values[0],
            medium: values[1],
            product_limit: values[2],
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct GraphProfileSize {
    nodes: usize,
    edges: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct GraphProfiles {
    small: GraphProfileSize,
    medium: GraphProfileSize,
    product_limit: GraphProfileSize,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct PortfolioBounds {
    nodes: usize,
    edges: usize,
    occurrences: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct CapacityProfiles {
    library_projects: ProfileSizes,
    capture_documents: ProfileSizes,
    graph: GraphProfiles,
    portfolio_projects: ProfileSizes,
    portable_files: ProfileSizes,
    portfolio_bounds: PortfolioBounds,
}

#[derive(Serialize)]
struct ReceiptTarget {
    os: &'static str,
    arch: &'static str,
}

#[derive(Serialize)]
struct MetricReceipt {
    operation: &'static str,
    profile: &'static str,
    metric: &'static str,
    unit: &'static str,
    raw_samples: Vec<u64>,
    p50: u64,
    p95: u64,
}

#[derive(Serialize)]
struct ProjectCapacityReceipt {
    receipt_version: &'static str,
    status: &'static str,
    source_commit: String,
    run_id: String,
    target: ReceiptTarget,
    rust_version: String,
    sample_count: usize,
    profiles: CapacityProfiles,
    fixture_sha256: String,
    metrics: Vec<MetricReceipt>,
}

#[derive(Serialize)]
struct GraphSemantics<'a> {
    schema_version: u32,
    project_id: &'a ProjectId,
    project_revision: u64,
    project_stage: ProjectStage,
    project_lifecycle: ProjectLifecycle,
    project_manifest_digest: &'a str,
    project_semantic_digest: &'a str,
    graph_source_digest: &'a str,
    sources: &'a [AcademicGraphSourceRefV1],
    nodes: &'a [AcademicGraphNodeV1],
    edges: &'a [AcademicGraphEdgeV1],
    diagnostics: &'a [crate::AcademicGraphDiagnosticV1],
}

struct LibraryFixture {
    service: ProjectStateService,
    project_ids: Vec<ProjectId>,
    project_roots: Vec<PathBuf>,
}

struct FixtureRoot(PathBuf);

impl std::ops::Deref for FixtureRoot {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn profiles() -> CapacityProfiles {
    CapacityProfiles {
        library_projects: ProfileSizes::new(LIBRARY_PROFILE_SIZES),
        capture_documents: ProfileSizes::new(CAPTURE_PROFILE_SIZES),
        graph: GraphProfiles {
            small: GraphProfileSize {
                nodes: GRAPH_PROFILE_SIZES[0].0,
                edges: GRAPH_PROFILE_SIZES[0].1,
            },
            medium: GraphProfileSize {
                nodes: GRAPH_PROFILE_SIZES[1].0,
                edges: GRAPH_PROFILE_SIZES[1].1,
            },
            product_limit: GraphProfileSize {
                nodes: GRAPH_PROFILE_SIZES[2].0,
                edges: GRAPH_PROFILE_SIZES[2].1,
            },
        },
        portfolio_projects: ProfileSizes::new(PORTFOLIO_PROFILE_SIZES),
        portable_files: ProfileSizes::new(PORTABLE_PROFILE_SIZES),
        portfolio_bounds: PortfolioBounds {
            nodes: MAX_PORTFOLIO_NODES,
            edges: MAX_PORTFOLIO_EDGES,
            occurrences: MAX_PORTFOLIO_OCCURRENCES,
        },
    }
}

fn fixture_sha256(profiles: &CapacityProfiles) -> String {
    let mut identity = FIXTURE_VERSION.to_vec();
    identity.extend(
        serde_json_canonicalizer::to_vec(profiles)
            .expect("capacity fixture identity must serialize"),
    );
    sha256_bytes(&identity)
}

fn fixture_root() -> FixtureRoot {
    let root = std::env::temp_dir().join(format!(
        "qiongli-platform-capacity-{}-{}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir(&root).expect("capacity fixture root must be writable");
    FixtureRoot(fs::canonicalize(root).expect("capacity fixture root must be canonical"))
}

fn project_id(value: usize) -> ProjectId {
    ProjectId::parse(format!("prj_{value:032x}")).expect("capacity project id must be valid")
}

fn create_library_fixture(base: &Path, name: &str, count: usize) -> LibraryFixture {
    let fixture = base.join(name);
    let home = fixture.join("home");
    let projects = fixture.join("projects");
    fs::create_dir_all(&home).expect("library home must be writable");
    fs::create_dir_all(&projects).expect("library project parent must be writable");
    let config = resolve_config_root(Some(fixture.join("config").as_os_str()), &home)
        .expect("isolated config root must resolve");
    let service = ProjectStateService::new(config);
    let mut entries = Vec::with_capacity(count);
    let mut project_ids = Vec::with_capacity(count);
    let mut project_roots = Vec::with_capacity(count);

    for index in 0..count {
        let project_id = project_id(index + 1);
        let project_root = projects.join(format!("project-{index:04}"));
        create_project_root(&project_root).expect("capacity project root must be created");
        let manifest = ArticleProjectManifestV1::new(
            project_id.clone(),
            format!("Capacity project {index}"),
            ProjectKind::Article,
            ProjectStage::Writing,
            empty_semantic_digest(),
            1,
        )
        .expect("capacity manifest must be valid");
        write_manifest(&project_root, &manifest, None).expect("capacity manifest must be written");
        entries.push(RegisteredProjectV1 {
            project_id: project_id.clone(),
            display_name: manifest.display_name.clone(),
            project_kind: manifest.project_kind,
            stage: manifest.stage,
            lifecycle: manifest.lifecycle,
            semantic_revision: manifest.semantic_revision,
            semantic_digest: manifest.semantic_digest.clone(),
            root_path: project_root_string(&project_root)
                .expect("capacity project root must be representable"),
            registered_at_unix: 1,
            last_opened_at_unix: None,
            academically_updated_at_unix: 1,
        });
        project_ids.push(project_id);
        project_roots.push(project_root);
    }

    let mut mutation = service
        .store
        .begin(0)
        .expect("empty capacity library must open");
    mutation.document.projects = entries;
    mutation
        .commit()
        .expect("capacity library must be committed once");
    LibraryFixture {
        service,
        project_ids,
        project_roots,
    }
}

fn create_capture_fixture(base: &Path, name: &str, count: usize) -> PathBuf {
    let root = base.join(name);
    create_project_root(&root).expect("capture fixture root must be created");
    fs::create_dir_all(root.join("context/captures"))
        .expect("capture history directory must be created");
    for index in 0..count {
        write_capture(&root, index);
    }
    root
}

fn write_capture(root: &Path, index: usize) {
    let draft = ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            project_id(1),
            1,
            ProjectStage::Writing,
            "capacity",
            CapturePolicy::HistoryOnly,
        )
        .expect("capture binding must be valid"),
        source: CaptureSource::Cli,
        delivery: CaptureDelivery::Connected,
        captured_at_unix: u64::try_from(index + 1).expect("capture index must fit u64"),
        summary: "capacity".to_string(),
        changes: Vec::new(),
        decisions: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        next_actions: Vec::new(),
    };
    let capture = draft.into_capture().expect("capture fixture must be valid");
    let bytes = capture
        .to_canonical_json()
        .expect("capture fixture must serialize");
    fs::write(
        root.join("context/captures")
            .join(format!("{}.json", capture.capture_id.as_str())),
        bytes,
    )
    .expect("capture fixture must be written");
}

fn graph_snapshot(
    project_id: &ProjectId,
    node_count: usize,
    edge_count: usize,
) -> Result<AcademicGraphSnapshotV1, ProjectError> {
    let mut nodes = Vec::with_capacity(node_count);
    nodes.push(AcademicGraphNodeV1::new(
        project_id,
        AcademicGraphNodeType::Project,
        AcademicGraphIdentityScope::Project,
        project_id.as_str(),
        "p",
        vec![AcademicGraphLayer::Combined],
        PROJECT_MANIFEST_PATH,
        "#/project_id",
    )?);
    for index in 1..node_count {
        nodes.push(AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Claim,
            AcademicGraphIdentityScope::Project,
            format!("c{index:x}"),
            "n",
            vec![AcademicGraphLayer::Combined],
            PROJECT_MANIFEST_PATH,
            format!("n{index:x}"),
        )?);
    }
    let mut edges = Vec::with_capacity(edge_count);
    if edge_count > 0 && nodes.len() < 2 {
        return Err(ProjectError::InvalidGraphDocument);
    }
    for index in 0..edge_count {
        let source = &nodes[index % nodes.len()];
        let target = &nodes[(index + 1) % nodes.len()];
        edges.push(AcademicGraphEdgeV1::new(
            project_id,
            &source.node_id,
            AcademicGraphRelation::Contains,
            &target.node_id,
            vec![AcademicGraphLayer::Combined],
            "r",
            PROJECT_MANIFEST_PATH,
            format!("e{index:x}"),
            "e",
            AcademicInferenceStrength::DirectEvidence,
            AcademicGraphConfidence::High,
            AcademicGraphEdgeStatus::Observed,
            None,
        )?);
    }
    finish_graph_snapshot(project_id, nodes, edges)
}

fn portfolio_graph(
    project_id: &ProjectId,
    shared_identity: usize,
) -> Result<AcademicGraphSnapshotV1, ProjectError> {
    let nodes = vec![
        AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Project,
            AcademicGraphIdentityScope::Project,
            project_id.as_str(),
            "p",
            vec![AcademicGraphLayer::Combined],
            PROJECT_MANIFEST_PATH,
            "#/project_id",
        )?,
        AcademicGraphNodeV1::new(
            project_id,
            AcademicGraphNodeType::Concept,
            AcademicGraphIdentityScope::Global,
            format!("g{shared_identity:x}"),
            "g",
            vec![AcademicGraphLayer::Combined],
            PROJECT_MANIFEST_PATH,
            "g",
        )?,
    ];
    finish_graph_snapshot(project_id, nodes, Vec::new())
}

fn finish_graph_snapshot(
    project_id: &ProjectId,
    mut nodes: Vec<AcademicGraphNodeV1>,
    mut edges: Vec<AcademicGraphEdgeV1>,
) -> Result<AcademicGraphSnapshotV1, ProjectError> {
    nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    let mut sources = std::iter::once(AcademicGraphSourceRefV1 {
        source_kind: AcademicGraphSourceKind::ProjectManifest,
        artifact_path: PROJECT_MANIFEST_PATH.to_string(),
        present: true,
        content_digest: Some("1".repeat(64)),
        size_bytes: 1,
    })
    .chain(
        SEMANTIC_ARTIFACTS
            .into_iter()
            .map(|path| AcademicGraphSourceRefV1 {
                source_kind: AcademicGraphSourceKind::RegisteredArtifact,
                artifact_path: path.to_string(),
                present: false,
                content_digest: None,
                size_bytes: 0,
            }),
    )
    .chain(std::iter::once(AcademicGraphSourceRefV1 {
        source_kind: AcademicGraphSourceKind::SemanticLinks,
        artifact_path: GRAPH_SEMANTIC_LINKS_RELATIVE_PATH.to_string(),
        present: false,
        content_digest: None,
        size_bytes: 0,
    }))
    .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.artifact_path.cmp(&right.artifact_path));
    let graph_source_digest =
        canonical_domain_digest(b"qiongli-academic-graph-sources-v1\0", &sources)?;
    let project_manifest_digest = "1".repeat(64);
    let project_semantic_digest = "2".repeat(64);
    let diagnostics = Vec::new();
    let semantics = GraphSemantics {
        schema_version: ACADEMIC_GRAPH_SCHEMA_VERSION,
        project_id,
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        project_lifecycle: ProjectLifecycle::Active,
        project_manifest_digest: &project_manifest_digest,
        project_semantic_digest: &project_semantic_digest,
        graph_source_digest: &graph_source_digest,
        sources: &sources,
        nodes: &nodes,
        edges: &edges,
        diagnostics: &diagnostics,
    };
    let projection_digest =
        canonical_domain_digest(b"qiongli-academic-graph-projection-v1\0", &semantics)?;
    let snapshot = AcademicGraphSnapshotV1 {
        schema_version: ACADEMIC_GRAPH_SCHEMA_VERSION,
        document_kind: ACADEMIC_GRAPH_DOCUMENT_KIND.to_string(),
        projection_id: format!("grp_{projection_digest}"),
        projection_digest,
        project_id: project_id.clone(),
        project_revision: 1,
        project_stage: ProjectStage::Writing,
        project_lifecycle: ProjectLifecycle::Active,
        project_manifest_digest,
        project_semantic_digest,
        graph_source_digest,
        source_count: sources.len(),
        present_source_count: 1,
        node_count: nodes.len(),
        edge_count: edges.len(),
        diagnostic_count: 0,
        sources,
        nodes,
        edges,
        diagnostics,
    };
    snapshot.validate()?;
    Ok(snapshot)
}

fn portfolio_fixture(
    project_count: usize,
) -> (ResearchLibrarySnapshotV1, Vec<AcademicGraphSnapshotV1>) {
    let project_ids = (0..project_count)
        .map(|index| project_id(index + 1))
        .collect::<Vec<_>>();
    let library = ResearchLibrarySnapshotV1 {
        schema_version: RESEARCH_LIBRARY_SCHEMA_VERSION,
        revision: u64::try_from(project_count).expect("portfolio project count must fit u64"),
        health: LibraryHealth::Ready,
        projects: project_ids
            .iter()
            .enumerate()
            .map(|(index, project_id)| ArticleProjectSummaryV1 {
                project_id: project_id.clone(),
                display_name: format!("Portfolio project {index}"),
                project_kind: ProjectKind::Article,
                stage: ProjectStage::Writing,
                lifecycle: ProjectLifecycle::Active,
                semantic_revision: 1,
                registered_at_unix: 1,
                last_opened_at_unix: None,
                academically_updated_at_unix: 1,
                health: ProjectHealth::Ready,
                next_action: ProjectNextAction::Open,
                root_label: format!("project-{index}"),
                overview: ProjectOverviewV1::empty(),
            })
            .collect(),
    };
    let graphs = project_ids
        .iter()
        .map(|project_id| {
            portfolio_graph(project_id, 1).expect("portfolio graph fixture must be valid")
        })
        .collect();
    (library, graphs)
}

fn create_portable_fixture(base: &Path, profile_name: &str, file_count: usize) -> LibraryFixture {
    let fixture = create_library_fixture(base, &format!("portable-{profile_name}"), 1);
    let project_root = &fixture.project_roots[0];
    let content = project_root.join("files");
    fs::create_dir(&content).expect("portable content directory must be created");
    for index in 0..file_count.saturating_sub(1) {
        fs::write(content.join(format!("file-{index:04}.txt")), b"capacity\n")
            .expect("portable fixture file must be written");
    }
    let (inventory, excluded) =
        migration_inventory(project_root).expect("portable fixture inventory must load");
    assert_eq!(inventory.len(), file_count);
    assert_eq!(excluded, 0);
    fixture
}

fn measure<F>(
    metrics: &mut Vec<MetricReceipt>,
    operation: &'static str,
    profile: &'static str,
    mut action: F,
) where
    F: FnMut(usize),
{
    action(0);
    let mut elapsed = Vec::with_capacity(SAMPLE_COUNT);
    let mut resident_set = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 1..=SAMPLE_COUNT {
        let started = Instant::now();
        action(sample);
        elapsed.push(
            u64::try_from(started.elapsed().as_nanos()).expect("elapsed nanoseconds must fit u64"),
        );
        resident_set.push(resident_memory_bytes());
    }
    metrics.push(metric_receipt(
        operation,
        profile,
        "elapsed_time",
        "nanoseconds",
        elapsed,
    ));
    metrics.push(metric_receipt(
        operation,
        profile,
        "resident_memory",
        "bytes",
        resident_set,
    ));
}

fn metric_receipt(
    operation: &'static str,
    profile: &'static str,
    metric: &'static str,
    unit: &'static str,
    raw_samples: Vec<u64>,
) -> MetricReceipt {
    assert_eq!(raw_samples.len(), SAMPLE_COUNT);
    MetricReceipt {
        operation,
        profile,
        metric,
        unit,
        p50: nearest_rank(&raw_samples, 50),
        p95: nearest_rank(&raw_samples, 95),
        raw_samples,
    }
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = percentile.saturating_mul(sorted.len()).div_ceil(100);
    sorted[rank - 1]
}

#[cfg(target_os = "linux")]
fn resident_memory_bytes() -> u64 {
    let value = fs::read_to_string("/proc/self/statm")
        .expect("Linux resident memory source must be readable");
    let pages = value
        .split_ascii_whitespace()
        .nth(1)
        .expect("Linux resident memory pages must be present")
        .parse::<u64>()
        .expect("Linux resident memory pages must be numeric");
    pages
        .checked_mul(u64::try_from(rustix::param::page_size()).expect("page size must fit u64"))
        .expect("Linux resident memory bytes must fit u64")
}

#[cfg(target_os = "macos")]
#[allow(clippy::disallowed_methods)]
fn resident_memory_bytes() -> u64 {
    let output = Command::new("/bin/ps")
        .args(["-o", "rss=", "-p"])
        .arg(std::process::id().to_string())
        .output()
        .expect("macOS resident memory command must launch");
    assert!(
        output.status.success(),
        "macOS resident memory command failed"
    );
    let kibibytes = std::str::from_utf8(&output.stdout)
        .expect("macOS resident memory output must be UTF-8")
        .trim()
        .parse::<u64>()
        .expect("macOS resident memory output must be numeric");
    kibibytes
        .checked_mul(1_024)
        .expect("macOS resident memory bytes must fit u64")
}

#[cfg(target_os = "windows")]
#[allow(clippy::disallowed_methods)]
fn resident_memory_bytes() -> u64 {
    let command = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &command])
        .output()
        .expect("Windows resident memory command must launch");
    assert!(
        output.status.success(),
        "Windows resident memory command failed"
    );
    std::str::from_utf8(&output.stdout)
        .expect("Windows resident memory output must be UTF-8")
        .trim()
        .parse::<u64>()
        .expect("Windows resident memory output must be numeric")
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn resident_memory_bytes() -> u64 {
    panic!("platform capacity receipts support Linux, macOS, and Windows only")
}

#[allow(clippy::disallowed_methods)]
fn rust_version() -> String {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("rustc version command must launch");
    assert!(output.status.success(), "rustc version command failed");
    let version = String::from_utf8(output.stdout)
        .expect("rustc version must be UTF-8")
        .trim()
        .to_string();
    assert!(version.starts_with("rustc "), "rustc version is malformed");
    version
}

fn required_source_commit() -> String {
    let value = std::env::var("QIONGLI_CAPACITY_SOURCE_COMMIT")
        .expect("QIONGLI_CAPACITY_SOURCE_COMMIT is required");
    assert!(
        crate::model::valid_lower_hex(&value, 40),
        "QIONGLI_CAPACITY_SOURCE_COMMIT must be 40 lowercase hex characters"
    );
    value
}

fn required_run_id() -> String {
    let value =
        std::env::var("QIONGLI_CAPACITY_RUN_ID").expect("QIONGLI_CAPACITY_RUN_ID is required");
    assert!(
        !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()),
        "QIONGLI_CAPACITY_RUN_ID must be positive decimal"
    );
    assert!(
        value.parse::<u64>().is_ok_and(|value| value > 0),
        "QIONGLI_CAPACITY_RUN_ID must be positive decimal"
    );
    value
}

fn validate_receipt(receipt: &ProjectCapacityReceipt) {
    assert_eq!(receipt.receipt_version, RECEIPT_VERSION);
    assert_eq!(receipt.status, "observation-only");
    assert!(crate::model::valid_lower_hex(&receipt.source_commit, 40));
    assert!(
        receipt.run_id.parse::<u64>().is_ok_and(|value| value > 0)
            && receipt.run_id.bytes().all(|byte| byte.is_ascii_digit())
    );
    assert!(matches!(receipt.target.os, "linux" | "macos" | "windows"));
    assert!(!receipt.target.arch.is_empty());
    assert!(receipt.rust_version.starts_with("rustc "));
    assert_eq!(receipt.sample_count, SAMPLE_COUNT);
    assert_eq!(receipt.profiles, profiles());
    assert!(crate::model::valid_lower_hex(&receipt.fixture_sha256, 64));
    assert_eq!(receipt.fixture_sha256, fixture_sha256(&receipt.profiles));

    let operations = [
        "project_snapshot",
        "project_refresh",
        "capture_load",
        "graph_build",
        "graph_query",
        "portfolio_rebuild",
        "portable_export",
        "portable_import",
    ];
    let mut expected = BTreeSet::new();
    for operation in operations {
        for profile in PROFILE_NAMES {
            expected.insert((operation, profile, "elapsed_time"));
            expected.insert((operation, profile, "resident_memory"));
        }
    }
    let observed = receipt
        .metrics
        .iter()
        .map(|metric| (metric.operation, metric.profile, metric.metric))
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, expected);
    assert_eq!(receipt.metrics.len(), expected.len());
    for metric in &receipt.metrics {
        assert_eq!(metric.raw_samples.len(), SAMPLE_COUNT);
        assert!(metric.raw_samples.iter().all(|sample| *sample > 0));
        assert_eq!(metric.p50, nearest_rank(&metric.raw_samples, 50));
        assert_eq!(metric.p95, nearest_rank(&metric.raw_samples, 95));
        assert!(matches!(
            (metric.metric, metric.unit),
            ("elapsed_time", "nanoseconds") | ("resident_memory", "bytes")
        ));
    }
}

#[test]
fn platform_capacity_contract_identifies_limits_and_percentiles() {
    assert_eq!(LIBRARY_PROFILE_SIZES[2], MAX_LIBRARY_PROJECTS);
    assert_eq!(CAPTURE_PROFILE_SIZES[2], MAX_CAPTURE_DOCUMENTS);
    assert_eq!(GRAPH_PROFILE_SIZES[2], (MAX_GRAPH_NODES, MAX_GRAPH_EDGES));
    assert_eq!(PORTFOLIO_PROFILE_SIZES[2], MAX_LIBRARY_PROJECTS);
    assert_eq!(PORTABLE_PROFILE_SIZES[2], MAX_PORTABLE_FILES);
    assert_eq!(MAX_PORTFOLIO_NODES, 16_384);
    assert_eq!(MAX_PORTFOLIO_EDGES, 32_768);
    assert_eq!(MAX_PORTFOLIO_OCCURRENCES, 65_536);
    assert_portfolio_independent_capacity_bounds();
    let samples = (1..=20).collect::<Vec<_>>();
    assert_eq!(nearest_rank(&samples, 50), 10);
    assert_eq!(nearest_rank(&samples, 95), 19);
    let graph = graph_snapshot(&project_id(1), 2, 1).expect("small graph fixture must validate");
    assert_eq!((graph.node_count, graph.edge_count), (2, 1));
    assert_eq!(fixture_sha256(&profiles()), fixture_sha256(&profiles()));
}

#[test]
#[ignore = "manual release-mode platform capacity observation"]
#[allow(clippy::assertions_on_constants)]
fn platform_capacity_baseline_writes_project_receipt() {
    assert!(
        !cfg!(debug_assertions),
        "platform capacity baseline must run with --release"
    );
    let source_commit = required_source_commit();
    let run_id = required_run_id();
    let output_dir = PathBuf::from(
        std::env::var("QIONGLI_CAPACITY_OUTPUT_DIR")
            .expect("QIONGLI_CAPACITY_OUTPUT_DIR is required"),
    );
    let fixture_root = fixture_root();
    let profiles = profiles();
    let fixture_sha256 = fixture_sha256(&profiles);
    let mut metrics = Vec::new();

    let library_fixtures = LIBRARY_PROFILE_SIZES
        .into_iter()
        .enumerate()
        .map(|(index, count)| {
            create_library_fixture(
                &fixture_root,
                &format!("library-{}", PROFILE_NAMES[index]),
                count,
            )
        })
        .collect::<Vec<_>>();
    for (index, fixture) in library_fixtures.iter().enumerate() {
        let profile = PROFILE_NAMES[index];
        let expected_count = LIBRARY_PROFILE_SIZES[index];
        measure(&mut metrics, "project_snapshot", profile, |_| {
            let snapshot = fixture
                .service
                .snapshot()
                .expect("capacity library snapshot must load");
            assert_eq!(snapshot.projects.len(), expected_count);
            black_box(snapshot);
        });
        let project_id = &fixture.project_ids[0];
        measure(&mut metrics, "project_refresh", profile, |_| {
            let refresh = fixture
                .service
                .preview_refresh(project_id, 2)
                .expect("capacity project refresh must preview");
            assert_eq!(refresh.preview().effect, ProjectMutationEffect::NoChange);
            let commit = fixture
                .service
                .apply(
                    &refresh,
                    &ApprovedProjectMutation::new(refresh.preview().plan_digest.clone(), true),
                    2,
                )
                .expect("capacity project refresh must apply");
            black_box(commit);
        });
    }
    assert_library_boundaries(&library_fixtures[2], &fixture_root);

    let capture_roots = CAPTURE_PROFILE_SIZES
        .into_iter()
        .enumerate()
        .map(|(index, count)| {
            create_capture_fixture(
                &fixture_root,
                &format!("capture-{}", PROFILE_NAMES[index]),
                count,
            )
        })
        .collect::<Vec<_>>();
    for (index, root) in capture_roots.iter().enumerate() {
        let profile = PROFILE_NAMES[index];
        let expected_count = CAPTURE_PROFILE_SIZES[index];
        measure(&mut metrics, "capture_load", profile, |_| {
            let captures =
                list_capture_documents(root).expect("capacity capture history must load");
            assert_eq!(captures.len(), expected_count);
            black_box(captures);
        });
    }
    let capture_limit_root = &capture_roots[2];
    assert_eq!(
        list_capture_documents(capture_limit_root)
            .expect("exact capture limit must load")
            .len(),
        MAX_CAPTURE_DOCUMENTS
    );
    write_capture(capture_limit_root, MAX_CAPTURE_DOCUMENTS);
    assert!(matches!(
        list_capture_documents(capture_limit_root),
        Err(ProjectError::DocumentTooLarge)
    ));

    let graph_fixtures = GRAPH_PROFILE_SIZES
        .into_iter()
        .enumerate()
        .map(|(index, (nodes, edges))| {
            graph_snapshot(&project_id(10_000 + index), nodes, edges)
                .expect("capacity graph fixture must validate")
        })
        .collect::<Vec<_>>();
    for (index, graph) in graph_fixtures.iter().enumerate() {
        let profile = PROFILE_NAMES[index];
        measure(&mut metrics, "graph_build", profile, |_| {
            black_box(
                AcademicGraphIndexV1::from_snapshot(graph.clone())
                    .expect("capacity graph index must build"),
            );
        });
        let index = AcademicGraphIndexV1::from_snapshot(graph.clone())
            .expect("capacity graph query index must build");
        let query = AcademicGraphQueryV1::new(graph.projection_id.clone());
        measure(&mut metrics, "graph_query", profile, |_| {
            black_box(index.query(&query).expect("capacity graph query must run"));
        });
    }
    assert_eq!(graph_fixtures[2].node_count, MAX_GRAPH_NODES);
    assert_eq!(graph_fixtures[2].edge_count, MAX_GRAPH_EDGES);
    assert!(matches!(
        graph_snapshot(&project_id(20_001), MAX_GRAPH_NODES + 1, 0),
        Err(ProjectError::InvalidGraphDocument)
    ));
    assert!(matches!(
        graph_snapshot(&project_id(20_002), 2, MAX_GRAPH_EDGES + 1),
        Err(ProjectError::InvalidGraphDocument)
    ));

    let portfolio_fixtures = PORTFOLIO_PROFILE_SIZES
        .into_iter()
        .map(portfolio_fixture)
        .collect::<Vec<_>>();
    for (index, (library, graphs)) in portfolio_fixtures.iter().enumerate() {
        let profile = PROFILE_NAMES[index];
        let expected_count = PORTFOLIO_PROFILE_SIZES[index];
        measure(&mut metrics, "portfolio_rebuild", profile, |_| {
            let portfolio =
                build_portfolio(library, graphs).expect("capacity portfolio must rebuild");
            assert_eq!(portfolio.project_count, expected_count);
            black_box(portfolio);
        });
    }
    assert_portfolio_independent_capacity_bounds();

    for (index, file_count) in PORTABLE_PROFILE_SIZES.into_iter().enumerate() {
        let profile = PROFILE_NAMES[index];
        let fixture = create_portable_fixture(&fixture_root, profile, file_count);
        let project_id = &fixture.project_ids[0];
        let project_root = &fixture.project_roots[0];
        let export_parent = fixture_root.join(format!("portable-runs-{profile}/exports"));
        fs::create_dir_all(&export_parent).expect("portable export parent must be created");
        measure(&mut metrics, "portable_export", profile, |sample| {
            let destination = export_parent.join(format!("export-{sample}"));
            let plan = fixture
                .service
                .preview_export(project_id, &destination)
                .expect("capacity portable export must preview");
            fixture
                .service
                .apply_portable(
                    &plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    3,
                )
                .expect("capacity portable export must apply");
            black_box(plan);
        });

        let canonical_export = export_parent.join("export-0");
        let import_cases = (0..=SAMPLE_COUNT)
            .map(|sample| {
                let case = fixture_root.join(format!("portable-runs-{profile}/import-{sample}"));
                let home = case.join("home");
                fs::create_dir_all(&home).expect("portable import home must be created");
                let config = resolve_config_root(Some(case.join("config").as_os_str()), &home)
                    .expect("portable import config must resolve");
                (ProjectStateService::new(config), case.join("project"))
            })
            .collect::<Vec<_>>();
        measure(&mut metrics, "portable_import", profile, |sample| {
            let (service, destination) = &import_cases[sample];
            let plan = service
                .preview_import(&canonical_export, destination)
                .expect("capacity portable import must preview");
            service
                .apply_portable(
                    &plan,
                    &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                    4,
                )
                .expect("capacity portable import must apply");
            black_box(plan);
        });

        let (inventory, excluded) =
            migration_inventory(project_root).expect("exact portable limit must load");
        assert_eq!(inventory.len(), file_count);
        assert_eq!(excluded, 0);
        if file_count == MAX_PORTABLE_FILES {
            fs::write(project_root.join("files/overflow.txt"), b"capacity\n")
                .expect("portable overflow fixture must be written");
            assert!(matches!(
                migration_inventory(project_root),
                Err(ProjectError::DocumentTooLarge)
            ));
        }
    }

    let receipt = ProjectCapacityReceipt {
        receipt_version: RECEIPT_VERSION,
        status: "observation-only",
        source_commit,
        run_id,
        target: ReceiptTarget {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        },
        rust_version: rust_version(),
        sample_count: SAMPLE_COUNT,
        profiles,
        fixture_sha256,
        metrics,
    };
    validate_receipt(&receipt);
    let mut bytes = serde_json_canonicalizer::to_vec(&receipt)
        .expect("project capacity receipt must serialize");
    let receipt_text = std::str::from_utf8(&bytes).expect("capacity receipt must be UTF-8");
    assert!(!receipt_text.contains(fixture_root.to_string_lossy().as_ref()));
    for forbidden in [
        "\"path\"",
        "\"home\"",
        "\"hostname\"",
        "\"username\"",
        "credential",
    ] {
        assert!(!receipt_text.contains(forbidden));
    }
    bytes.push(b'\n');
    fs::create_dir_all(&output_dir).expect("capacity output directory must be created");
    fs::write(output_dir.join("qiongli-project-capacity.json"), bytes)
        .expect("project capacity receipt must be written");
}

fn assert_library_boundaries(fixture: &LibraryFixture, base: &Path) {
    let document = fixture
        .service
        .store
        .load()
        .expect("capacity library document must load");
    assert_eq!(document.projects.len(), MAX_LIBRARY_PROJECTS);
    document
        .validate()
        .expect("exact library limit must validate");
    let mut over_limit = document.clone();
    let mut extra = over_limit.projects[0].clone();
    extra.project_id = project_id(MAX_LIBRARY_PROJECTS + 1);
    extra.root_path = "capacity-over-limit".to_string();
    over_limit.projects.push(extra);
    assert_eq!(
        over_limit.validate(),
        Err(ProjectError::InvalidLibraryDocument)
    );
    let overflow_root = base.join("library-overflow-project");
    assert!(matches!(
        fixture.service.preview_create(
            &overflow_root,
            ProjectRegistrationOptions::new("Overflow project", ProjectKind::Article)
                .with_project_id(project_id(MAX_LIBRARY_PROJECTS + 1)),
            2,
        ),
        Err(ProjectError::LibraryFull)
    ));
}

fn assert_portfolio_independent_capacity_bounds() {
    for (exact, over_limit) in [
        ((MAX_PORTFOLIO_NODES, 0, 0), (MAX_PORTFOLIO_NODES + 1, 0, 0)),
        ((0, MAX_PORTFOLIO_EDGES, 0), (0, MAX_PORTFOLIO_EDGES + 1, 0)),
        (
            (0, 0, MAX_PORTFOLIO_OCCURRENCES),
            (0, 0, MAX_PORTFOLIO_OCCURRENCES + 1),
        ),
    ] {
        validate_portfolio_capacity(exact.0, exact.1, exact.2)
            .expect("independent exact portfolio limit must pass its count owner");
        assert_eq!(
            validate_portfolio_capacity(over_limit.0, over_limit.1, over_limit.2),
            Err(ProjectError::InvalidGraphDocument)
        );
    }
}
