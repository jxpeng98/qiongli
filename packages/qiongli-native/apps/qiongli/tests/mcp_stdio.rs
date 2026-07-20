#![allow(clippy::disallowed_methods)]

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_config::resolve_config_root;
use qiongli_project::{
    ApprovedProjectMutation, CaptureArea, CaptureDelivery, CapturePolicy, CaptureSource,
    EvidenceLocatorKind, EvidenceReferenceV1, ProjectBindingV1, ProjectKind,
    ProjectRegistrationOptions, ProjectStage, ProjectStateService, ResearchCaptureDraftV1,
    SemanticChangeV1,
};
use qiongli_runtime::{FULL_PROJECT_PUBLIC_TOOL_NAMES, LITE_PUBLIC_TOOL_NAMES};
use serde_json::{Value, json};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
const SECRET_CANARY: &str = "copied-native-mcp-secret-canary";

struct Fixture {
    root: PathBuf,
    home: PathBuf,
    config_root: PathBuf,
    executable: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let native_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("app crate must live below the native workspace");
        let test_base = native_root.join("target/qiongli-native-mcp-tests");
        fs::create_dir_all(&test_base).expect("MCP test base must be created");
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock must follow the Unix epoch")
            .as_nanos();
        let root = test_base.join(format!(
            "copied-binary-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("isolated MCP root must be created");
        set_private_directory_mode(&root);
        let home = root.join("home");
        fs::create_dir(&home).expect("isolated MCP home must be created");
        set_private_directory_mode(&home);
        let config_root = root.join("private-config-path-canary");
        let executable = root.join(format!("copied-qiongli{}", std::env::consts::EXE_SUFFIX));
        fs::copy(env!("CARGO_BIN_EXE_qiongli"), &executable)
            .expect("canonical binary must be copied");
        set_executable_mode(&executable);
        Self {
            root,
            home,
            config_root,
            executable,
        }
    }

    fn command(&self) -> Command {
        self.command_with_profile("marketplace-lite")
    }

    fn command_with_profile(&self, profile: &str) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&self.root)
            .env("PATH", "")
            .env("QIONGLI_CONFIG_HOME", &self.config_root)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .args(["mcp", "serve", "--transport", "stdio", "--profile", profile])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
fn set_private_directory_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("fixture directory mode must be private");
}

#[cfg(not(unix))]
fn set_private_directory_mode(_path: &Path) {}

#[cfg(unix)]
fn set_executable_mode(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("copied binary must be executable");
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) {}

fn rpc(id: u64, method: &str, params: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
}

fn tool_call(id: u64, name: &str, arguments: Value) -> Value {
    rpc(
        id,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
}

fn full_tool_response(fixture: &Fixture, id: u64, name: &str, arguments: Value) -> (String, Value) {
    let mut child = fixture
        .command_with_profile("full")
        .spawn()
        .expect("copied canonical binary must start in full profile");
    {
        let stdin = child.stdin.as_mut().expect("MCP stdin must be piped");
        for request in [
            rpc(0, "initialize", json!({})),
            tool_call(id, name, arguments),
        ] {
            serde_json::to_writer(&mut *stdin, &request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let rendered = String::from_utf8(output.stdout).unwrap();
    let responses = rendered
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    let response = responses
        .into_iter()
        .find(|response| response["id"] == id)
        .expect("tool response ID must exist");
    (rendered, response)
}

#[test]
fn copied_binary_serves_initialize_list_and_bounded_calls_without_path_runtime() {
    let fixture = Fixture::new();
    let mut child = fixture
        .command()
        .spawn()
        .expect("copied canonical binary must start");
    let requests = [
        rpc(
            1,
            "initialize",
            json!({"protocolVersion": "2025-11-25", "capabilities": {}}),
        ),
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
        rpc(2, "tools/list", json!({})),
        tool_call(3, "qiongli_config_status", json!({})),
        tool_call(
            4,
            "qiongli_search_plan",
            json!({
                "query": "copied binary planning",
                "from_year": 2020,
                "toYear": "2026"
            }),
        ),
        tool_call(5, "qiongli_literature_search", json!({})),
        tool_call(
            6,
            "qiongli_literature_export_evidence",
            json!({"query": "copied binary", "results": [], "diagnostics": {}}),
        ),
        tool_call(7, "qiongli_zotero_status", json!({})),
        tool_call(
            8,
            "qiongli_zotero_export_import_files",
            json!({"records": [], "formats": []}),
        ),
        tool_call(
            9,
            "qiongli_orchestrator_route",
            json!({"request": "plan a review", "platform": "codex"}),
        ),
        tool_call(
            10,
            "qiongli_task_plan",
            json!({"task_id": "B1", "paper_type": "review", "topic": "AI"}),
        ),
        tool_call(
            11,
            "qiongli_save_provider_config",
            json!({
                "provider": "semantic_scholar",
                "field": "api_key",
                "value": SECRET_CANARY
            }),
        ),
        tool_call(
            12,
            "qiongli_configure_provider",
            json!({"host": "example.invalid", "port": 0}),
        ),
    ];
    {
        let stdin = child.stdin.as_mut().expect("MCP stdin must be piped");
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request).expect("request must serialize");
            stdin
                .write_all(b"\n")
                .expect("request delimiter must write");
        }
    }
    drop(child.stdin.take());
    let output = child
        .wait_with_output()
        .expect("copied MCP process must exit on EOF");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let rendered = String::from_utf8(output.stdout).expect("MCP stdout must be UTF-8 JSON lines");
    assert!(!rendered.contains(SECRET_CANARY));
    assert!(!rendered.contains("private-config-path-canary"));
    let responses = rendered
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("response must be JSON"))
        .collect::<Vec<_>>();
    assert_eq!(
        responses.len(),
        12,
        "notification must not produce a response"
    );

    let by_id = |id: u64| {
        responses
            .iter()
            .find(|response| response["id"] == id)
            .expect("response ID must exist")
    };
    assert_eq!(by_id(1)["result"]["serverInfo"]["name"], "qiongli");
    let names = by_id(2)["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(names, LITE_PUBLIC_TOOL_NAMES);
    assert_eq!(
        by_id(3)["result"]["structuredContent"]["config_path"],
        "<managed-native-config>"
    );
    let filters = &by_id(4)["result"]["structuredContent"]["provider_queries"][0]["filters"];
    assert_eq!(filters["from_year"], filters["fromYear"]);
    assert_eq!(filters["to_year"], filters["toYear"]);
    assert_eq!(by_id(5)["error"]["code"], -32602);
    assert_eq!(
        by_id(6)["result"]["structuredContent"]["artifact_type"],
        "qiongli_literature_evidence_snapshot"
    );
    assert_eq!(
        by_id(7)["result"]["structuredContent"]["status"],
        "disabled"
    );
    assert_eq!(by_id(8)["result"]["structuredContent"]["status"], "ok");
    assert_eq!(
        by_id(9)["result"]["structuredContent"]["run_agents_allowed"],
        false
    );
    assert_eq!(
        by_id(10)["result"]["structuredContent"]["preview_only"],
        true
    );
    assert_eq!(
        by_id(11)["result"]["structuredContent"]["reason_code"],
        "capability-unavailable"
    );
    assert_eq!(by_id(12)["error"]["code"], -32602);
}

#[test]
fn full_profile_reuses_redacted_project_state_and_accepts_connected_capture() {
    let fixture = Fixture::new();
    let config = resolve_config_root(Some(fixture.config_root.as_os_str()), &fixture.home).unwrap();
    let service = ProjectStateService::new(config);
    let project_root = fixture.root.join("full-project-path-canary");
    let create = service
        .preview_create(
            &project_root,
            ProjectRegistrationOptions::new("Full MCP Article", ProjectKind::Article),
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
    let project_id_string = project_id.as_str().to_string();
    let capture = ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            project_id.clone(),
            1,
            ProjectStage::Idea,
            "Preserve the article argument across connected agents.",
            CapturePolicy::ReviewRequired,
        )
        .unwrap(),
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Connected,
        captured_at_unix: 2,
        summary: "The article project, rather than a client session, owns durable research memory."
            .to_string(),
        changes: vec![SemanticChangeV1 {
            area: CaptureArea::Thesis,
            summary: "Use one cross-platform article project as the continuity boundary."
                .to_string(),
        }],
        decisions: vec![],
        evidence: vec![EvidenceReferenceV1 {
            locator_kind: EvidenceLocatorKind::Doi,
            locator: "10.1000/full-mcp-capture".to_string(),
            relevance: "Anchors the connected capture acceptance fixture.".to_string(),
            limitation: Some("Fixture evidence is not a publication claim.".to_string()),
        }],
        contradictions: vec![],
        next_actions: vec!["Review the normalized capture before consolidation.".to_string()],
    }
    .into_capture()
    .unwrap();
    let capture_id = capture.capture_id.as_str().to_string();
    let disconnected_capture = ResearchCaptureDraftV1 {
        binding: capture.binding.clone(),
        source: capture.source,
        delivery: CaptureDelivery::Portable,
        captured_at_unix: capture.captured_at_unix,
        summary: capture.summary.clone(),
        changes: capture.changes.clone(),
        decisions: capture.decisions.clone(),
        evidence: capture.evidence.clone(),
        contradictions: capture.contradictions.clone(),
        next_actions: capture.next_actions.clone(),
    }
    .into_capture()
    .unwrap();

    let mut child = fixture
        .command_with_profile("full")
        .spawn()
        .expect("copied canonical binary must start in full profile");
    let requests = [
        rpc(1, "initialize", json!({})),
        rpc(2, "tools/list", json!({})),
        tool_call(3, "qiongli_project_list", json!({})),
        tool_call(
            4,
            "qiongli_project_read",
            json!({"project_id": project_id_string}),
        ),
        tool_call(
            5,
            "qiongli_project_read",
            json!({"project_id": "invalid-project-id"}),
        ),
        tool_call(6, "qiongli_project_list", json!({(SECRET_CANARY): true})),
        tool_call(
            7,
            "qiongli_project_capture_preview",
            json!({"capture": capture}),
        ),
        tool_call(
            8,
            "qiongli_project_capture_preview",
            json!({"capture": capture, "capture_path": SECRET_CANARY}),
        ),
        tool_call(
            9,
            "qiongli_project_capture_preview",
            json!({"capture": disconnected_capture}),
        ),
    ];
    {
        let stdin = child.stdin.as_mut().expect("MCP stdin must be piped");
        for request in requests {
            serde_json::to_writer(&mut *stdin, &request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let rendered = String::from_utf8(output.stdout).unwrap();
    assert!(!rendered.contains(SECRET_CANARY));
    assert!(!rendered.contains(project_root.to_string_lossy().as_ref()));
    assert!(!rendered.contains("private-config-path-canary"));
    let responses = rendered
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    let by_id = |id: u64| {
        responses
            .iter()
            .find(|response| response["id"] == id)
            .unwrap()
    };
    let names = by_id(2)["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    let expected = LITE_PUBLIC_TOOL_NAMES
        .into_iter()
        .chain(FULL_PROJECT_PUBLIC_TOOL_NAMES)
        .collect::<Vec<_>>();
    assert_eq!(names, expected);
    assert_eq!(
        by_id(3)["result"]["structuredContent"]["projects"][0]["displayName"],
        "Full MCP Article"
    );
    assert_eq!(
        by_id(4)["result"]["structuredContent"]["project"]["projectId"],
        project_id_string
    );
    assert_eq!(by_id(5)["error"]["code"], -32602);
    assert_eq!(by_id(6)["error"]["code"], -32602);
    assert_eq!(
        by_id(7)["result"]["structuredContent"]["captureId"],
        capture_id
    );
    assert_eq!(
        by_id(7)["result"]["structuredContent"]["projectId"],
        project_id_string
    );
    assert_eq!(
        by_id(7)["result"]["structuredContent"]["effect"],
        "append-pending-history"
    );
    assert_eq!(by_id(8)["error"]["code"], -32602);
    assert_eq!(by_id(9)["error"]["code"], -32602);

    let plan_digest = by_id(7)["result"]["structuredContent"]["planDigest"]
        .as_str()
        .unwrap()
        .to_string();
    let apply_arguments = |digest: &str, approve: bool| {
        json!({
            "capture": capture,
            "plan_digest": digest,
            "approve_filesystem_write": approve
        })
    };
    let (denied_rendered, denied) = full_tool_response(
        &fixture,
        10,
        "qiongli_project_capture_apply",
        apply_arguments(&plan_digest, false),
    );
    assert_eq!(
        denied["result"]["structuredContent"]["reason_code"],
        "project-filesystem-approval-required"
    );
    let (mismatch_rendered, mismatch) = full_tool_response(
        &fixture,
        11,
        "qiongli_project_capture_apply",
        apply_arguments(&"0".repeat(64), true),
    );
    assert_eq!(
        mismatch["result"]["structuredContent"]["reason_code"],
        "project-plan-mismatch"
    );
    let (applied_rendered, applied) = full_tool_response(
        &fixture,
        12,
        "qiongli_project_capture_apply",
        apply_arguments(&plan_digest, true),
    );
    assert_eq!(
        applied["result"]["structuredContent"]["captureId"],
        capture_id
    );
    assert_eq!(
        applied["result"]["structuredContent"]["projectId"],
        project_id_string
    );
    assert!(
        applied["result"]["structuredContent"]["acknowledgement"]
            .as_str()
            .unwrap()
            .starts_with("ack_")
    );
    let (replay_rendered, replay) = full_tool_response(
        &fixture,
        13,
        "qiongli_project_capture_apply",
        apply_arguments(&plan_digest, true),
    );
    assert_eq!(
        replay["result"]["structuredContent"]["reason_code"],
        "research-capture-already-applied"
    );
    let (coverage_rendered, coverage) = full_tool_response(
        &fixture,
        14,
        "qiongli_project_capture_coverage",
        json!({"project_id": project_id_string}),
    );
    let coverage = &coverage["result"]["structuredContent"];
    assert_eq!(coverage["captureCount"], 1);
    assert_eq!(coverage["connectedCount"], 1);
    assert_eq!(coverage["pendingReviewCount"], 1);
    assert_eq!(coverage["unknownSourceCount"], 6);
    assert_eq!(coverage["sources"].as_array().unwrap().len(), 7);
    assert_eq!(coverage["sources"][0]["source"], "codex");
    assert_eq!(coverage["sources"][0]["delivery"], "connected");
    assert_eq!(coverage["sources"][0]["state"], "pending-review");
    assert_eq!(coverage["sources"][1]["delivery"], "unknown");
    assert_eq!(coverage["sources"][1]["state"], "unknown");
    for response in [
        denied_rendered,
        mismatch_rendered,
        applied_rendered,
        replay_rendered,
        coverage_rendered,
    ] {
        assert!(!response.contains(SECRET_CANARY));
        assert!(!response.contains(project_root.to_string_lossy().as_ref()));
        assert!(!response.contains("private-config-path-canary"));
    }
    let inbox = service.capture_inbox(&project_id).unwrap();
    assert_eq!(inbox.entries.len(), 1);
    assert_eq!(inbox.entries[0].capture_id.as_str(), capture_id);
    assert!(
        service
            .read_capture(&project_id, &capture.capture_id)
            .unwrap()
            .is_some()
    );
}

#[test]
fn invalid_or_escalating_mcp_cli_modes_fail_before_stdio_serving() {
    for args in [
        ["mcp", "serve", "--profile", "lite", "--transport", "http"].as_slice(),
        ["mcp", "serve", "--profile", "lite"].as_slice(),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_qiongli"))
            .args(args)
            .output()
            .expect("invalid native MCP command must exit");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Qiongli native MCP"));
    }
}
