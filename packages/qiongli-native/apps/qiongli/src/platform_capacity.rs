use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use qiongli_project::{
    ApprovedProjectMutation, ProjectId, ProjectKind, ProjectRegistrationOptions,
    ProjectStateService,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::CommandEnvironment;

const RECEIPT_VERSION: &str = "qiongli-platform-capacity/v1";
const FIXTURE_VERSION: &str = "qiongli-desktop-capacity-fixture/v1";
const SAMPLE_COUNT: usize = 20;
const PROFILES: [ProfileSpec; 3] = [
    ProfileSpec {
        name: "small",
        library_projects: 3,
    },
    ProfileSpec {
        name: "medium",
        library_projects: 64,
    },
    ProfileSpec {
        name: "product-limit",
        library_projects: 512,
    },
];

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Serialize)]
struct ProfileSpec {
    name: &'static str,
    library_projects: usize,
}

#[derive(Serialize)]
struct CapacityReceipt {
    receipt_version: &'static str,
    status: &'static str,
    source_commit: String,
    run_id: String,
    target: Target,
    rust_version: String,
    build_profile: &'static str,
    sample_count: usize,
    fixture_sha256: String,
    profiles: Vec<ProfileReceipt>,
}

#[derive(Serialize)]
struct Target {
    os: &'static str,
    arch: &'static str,
}

#[derive(Serialize)]
struct ProfileReceipt {
    name: &'static str,
    counts: ProfileCounts,
    metrics: DesktopMetrics,
}

#[derive(Serialize)]
struct ProfileCounts {
    library_projects: usize,
}

#[derive(Serialize)]
struct DesktopMetrics {
    native_startup_validation: SampleMetric,
    app_snapshot: SampleMetric,
    serialized_ipc_payload: SampleMetric,
}

#[derive(Debug, Serialize)]
struct SampleMetric {
    unit: &'static str,
    raw_samples: Vec<u64>,
    p50: u64,
    p95: u64,
}

impl SampleMetric {
    fn new(unit: &'static str, raw_samples: Vec<u64>) -> Result<Self, &'static str> {
        if raw_samples.len() != SAMPLE_COUNT {
            return Err("capacity-sample-count-invalid");
        }
        if raw_samples.contains(&0) {
            return Err("capacity-sample-non-positive");
        }
        Ok(Self {
            p50: nearest_rank(&raw_samples, 50)?,
            p95: nearest_rank(&raw_samples, 95)?,
            unit,
            raw_samples,
        })
    }
}

struct CapacityInputs {
    output_dir: PathBuf,
    source_commit: String,
    run_id: String,
}

struct CapacityFixture {
    root: FixtureRoot,
    environment: CommandEnvironment,
    service: ProjectStateService,
    project_count: usize,
}

struct FixtureRoot(PathBuf);

impl std::ops::Deref for FixtureRoot {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FixtureRoot {
    fn cleanup(self) {
        fs::remove_dir_all(&self.0).expect("capacity fixture must be removed");
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl CapacityFixture {
    fn new() -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let fixture_base = native_root.join("target/qiongli-platform-capacity-tests");
        fs::create_dir_all(&fixture_base).expect("capacity fixture base must be created");
        let root = fixture_base.join(format!(
            "desktop-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        create_private_directory(&root);
        let root = FixtureRoot(root);
        let home = root.join("home");
        let projects = root.join("projects");
        create_private_directory(&home);
        create_private_directory(&projects);
        let environment = CommandEnvironment::with_paths(
            Some(OsString::from(root.join("config"))),
            Some(home),
            None,
        )
        .without_client_discovery();
        let config_root = crate::command::config_root(&environment)
            .expect("isolated capacity config root must resolve");

        Self {
            root,
            environment,
            service: ProjectStateService::new(config_root),
            project_count: 0,
        }
    }

    fn extend_to(&mut self, project_count: usize) {
        assert!(project_count >= self.project_count);
        for index in self.project_count..project_count {
            let project_root = self
                .root
                .join("projects")
                .join(format!("project-{index:04}"));
            let project_id = ProjectId::parse(format!("prj_{:032x}", index + 1))
                .expect("capacity project ID must be valid");
            let options = ProjectRegistrationOptions::new(
                format!("Capacity Project {index:04}"),
                ProjectKind::Article,
            )
            .with_project_id(project_id);
            let now_unix = 1_700_000_000 + index as u64;
            let plan = self
                .service
                .preview_create(&project_root, options, now_unix)
                .expect("capacity project must preview");
            let approval = ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true);
            self.service
                .apply(&plan, &approval, now_unix)
                .expect("capacity project must be created");
        }
        self.project_count = project_count;
        assert_eq!(
            self.service
                .snapshot()
                .expect("capacity library must load")
                .projects
                .len(),
            project_count
        );
    }

    fn cleanup(self) {
        self.root.cleanup();
    }
}

#[test]
fn platform_capacity_contract() {
    let samples = (1..=SAMPLE_COUNT as u64).collect::<Vec<_>>();
    let metric = SampleMetric::new("nanoseconds", samples).expect("samples must be valid");
    assert_eq!((metric.p50, metric.p95), (10, 19));
    assert_eq!(
        SampleMetric::new("nanoseconds", vec![0; SAMPLE_COUNT]).unwrap_err(),
        "capacity-sample-non-positive"
    );
    assert_eq!(
        PROFILES.map(|profile| profile.library_projects),
        [3, 64, 512]
    );
    assert_eq!(fixture_sha256(), fixture_sha256());
    assert!(valid_source_commit(
        "0123456789abcdef0123456789abcdef01234567"
    ));
    assert!(!valid_source_commit(
        "0123456789ABCDEF0123456789abcdef01234567"
    ));
    assert!(valid_run_id("1"));
    assert!(!valid_run_id("0"));

    let detected = crate::command::DetectedClientVersion {
        major: 1,
        minor: 2,
        patch: 3,
    };
    let mut environment = CommandEnvironment::with_paths(None, None, None)
        .with_client_versions(Some(detected), Some(detected))
        .without_client_discovery();
    environment.detect_client_versions();
    assert_eq!(environment.codex_host_version(), None);
    assert_eq!(environment.claude_host_version(), None);
}

#[test]
#[ignore = "manual release-mode three-target capacity observation"]
fn platform_capacity_baseline() {
    ensure_release_profile();
    let inputs = capacity_inputs().expect("capacity inputs must be valid");
    let content = crate::embedded_content().expect("embedded content must load");
    let mut fixture = CapacityFixture::new();
    let mut profiles = Vec::with_capacity(PROFILES.len());

    for profile in PROFILES {
        fixture.extend_to(profile.library_projects);
        profiles.push(measure_profile(profile, &fixture.environment, &content));
    }

    let receipt = CapacityReceipt {
        receipt_version: RECEIPT_VERSION,
        status: "observation-only",
        source_commit: inputs.source_commit,
        run_id: inputs.run_id,
        target: Target {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        },
        rust_version: rust_version().expect("rustc version must be available"),
        build_profile: "release",
        sample_count: SAMPLE_COUNT,
        fixture_sha256: fixture_sha256(),
        profiles,
    };
    let mut rendered =
        serde_json_canonicalizer::to_vec(&receipt).expect("capacity receipt must serialize");
    let receipt_text =
        std::str::from_utf8(&rendered).expect("capacity receipt must contain UTF-8 JSON");
    assert!(
        !receipt_text.contains(fixture.root.to_string_lossy().as_ref()),
        "capacity receipt must not expose its fixture path"
    );
    for forbidden in [
        "\"path\"",
        "\"home\"",
        "\"hostname\"",
        "\"username\"",
        "credential",
    ] {
        assert!(!receipt_text.contains(forbidden));
    }
    rendered.push(b'\n');
    fs::create_dir_all(&inputs.output_dir).expect("capacity output directory must be created");
    fs::write(
        inputs.output_dir.join("qiongli-desktop-capacity.json"),
        rendered,
    )
    .expect("capacity receipt must be written");
    fixture.cleanup();
}

fn measure_profile(
    profile: ProfileSpec,
    environment: &CommandEnvironment,
    content: &qiongli_content::EmbeddedContent,
) -> ProfileReceipt {
    crate::desktop::validate_desktop_startup(environment, content)
        .expect("desktop startup warm-up must pass");
    let mut startup_samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        crate::desktop::validate_desktop_startup(environment, content)
            .expect("desktop startup sample must pass");
        startup_samples.push(elapsed_nanoseconds(started));
    }

    let warm_snapshot = crate::desktop::app_snapshot_json(environment, content)
        .expect("App snapshot warm-up must pass");
    assert_snapshot_project_count(&warm_snapshot, profile.library_projects);
    let mut snapshot_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut payload_samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        let snapshot = crate::desktop::app_snapshot_json(environment, content)
            .expect("App snapshot sample must pass");
        snapshot_samples.push(elapsed_nanoseconds(started));
        payload_samples.push(snapshot.len() as u64);
    }

    ProfileReceipt {
        name: profile.name,
        counts: ProfileCounts {
            library_projects: profile.library_projects,
        },
        metrics: DesktopMetrics {
            native_startup_validation: SampleMetric::new("nanoseconds", startup_samples)
                .expect("startup samples must be complete"),
            app_snapshot: SampleMetric::new("nanoseconds", snapshot_samples)
                .expect("snapshot samples must be complete"),
            serialized_ipc_payload: SampleMetric::new("bytes", payload_samples)
                .expect("payload samples must be complete"),
        },
    }
}

fn assert_snapshot_project_count(snapshot: &str, expected: usize) {
    let snapshot: serde_json::Value =
        serde_json::from_str(snapshot).expect("App snapshot must be valid JSON");
    assert_eq!(
        snapshot["researchLibrary"]["projects"]
            .as_array()
            .expect("App snapshot must contain Research Library projects")
            .len(),
        expected
    );
}

fn elapsed_nanoseconds(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> Result<u64, &'static str> {
    if samples.is_empty() || !(1..=100).contains(&percentile) {
        return Err("capacity-percentile-invalid");
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (percentile * sorted.len()).div_ceil(100);
    Ok(sorted[rank - 1])
}

fn fixture_sha256() -> String {
    let bytes = serde_json_canonicalizer::to_vec(&serde_json::json!({
        "fixture_version": FIXTURE_VERSION,
        "embedded_pack_sha256": crate::EMBEDDED_PACK_SHA256.trim(),
        "profiles": PROFILES,
    }))
    .expect("fixture identity must serialize");
    format!("{:x}", Sha256::digest(bytes))
}

fn capacity_inputs() -> Result<CapacityInputs, &'static str> {
    let output_dir = std::env::var_os("QIONGLI_CAPACITY_OUTPUT_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or("capacity-output-directory-required")?;
    let source_commit = std::env::var("QIONGLI_CAPACITY_SOURCE_COMMIT")
        .map_err(|_| "capacity-source-commit-required")?;
    if !valid_source_commit(&source_commit) {
        return Err("capacity-source-commit-invalid");
    }
    let run_id =
        std::env::var("QIONGLI_CAPACITY_RUN_ID").map_err(|_| "capacity-run-id-required")?;
    if !valid_run_id(&run_id) {
        return Err("capacity-run-id-invalid");
    }
    if !matches!(std::env::consts::OS, "linux" | "macos" | "windows") {
        return Err("capacity-target-unsupported");
    }
    Ok(CapacityInputs {
        output_dir,
        source_commit,
        run_id,
    })
}

fn valid_source_commit(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_run_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok_and(|run_id| run_id > 0)
}

#[cfg(debug_assertions)]
fn ensure_release_profile() {
    panic!("capacity baseline must run with --release");
}

#[cfg(not(debug_assertions))]
const fn ensure_release_profile() {}

/// The capacity receipt records the compiler identity without shipping a process launcher.
#[allow(clippy::disallowed_methods)]
fn rust_version() -> Result<String, &'static str> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|_| "capacity-rustc-unavailable")?;
    if !output.status.success() {
        return Err("capacity-rustc-failed");
    }
    let version = String::from_utf8(output.stdout).map_err(|_| "capacity-rustc-invalid")?;
    let version = version.trim();
    if !version.starts_with("rustc ") || version.chars().any(char::is_control) {
        return Err("capacity-rustc-invalid");
    }
    Ok(version.to_owned())
}

#[cfg(unix)]
fn create_private_directory(path: &Path) {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder
        .create(path)
        .expect("private capacity directory must be created");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("private capacity directory mode must be retained");
}

#[cfg(windows)]
fn create_private_directory(path: &Path) {
    qiongli_windows_security::create_owner_only_directory(path)
        .expect("owner-only capacity directory must be created");
}

#[cfg(not(any(unix, windows)))]
fn create_private_directory(path: &Path) {
    fs::create_dir(path).expect("private capacity directory must be created");
}
