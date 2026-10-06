#![allow(clippy::disallowed_methods)]
use qiongli_config::resolve_config_root;
use qiongli_project::{
    ApprovedCaptureConsolidation, ApprovedCaptureIntake, ApprovedProjectMutation,
    CaptureConsolidationDrafts, CaptureDelivery, CapturePolicy, CaptureSource, ProjectBindingV1,
    ProjectId, ProjectKind, ProjectRegistrationOptions, ProjectStage, ProjectStateService,
    ResearchCaptureDraftV1, SavedDocumentReadRequest, SourcePacketDraftV1,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
struct Fixture {
    base: PathBuf,
    root: PathBuf,
    home: PathBuf,
    config: PathBuf,
    service: ProjectStateService,
    id: ProjectId,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "qiongli-saved-read-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let base = fs::canonicalize(base).unwrap();
        let root = base.join("project");
        let home = base.join("home");
        let config = base.join("config");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir(&home).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&base, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let service =
            ProjectStateService::new(resolve_config_root(Some(config.as_os_str()), &home).unwrap());
        let id = ProjectId::parse("prj_abcdef0123456789abcdef0123456789").unwrap();
        let plan = service
            .preview_register(
                &root,
                ProjectRegistrationOptions::new("Public synthetic reader", ProjectKind::Article)
                    .with_project_id(id.clone())
                    .with_stage(ProjectStage::Literature),
                100,
            )
            .unwrap();
        service
            .apply(
                &plan,
                &ApprovedProjectMutation::new(plan.preview().plan_digest.clone(), true),
                100,
            )
            .unwrap();
        Self {
            base,
            root,
            home,
            config,
            service,
            id,
        }
    }
    fn put(&self, path: &str, bytes: &[u8]) {
        let p = self.root.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    }
    fn request(&self, path: &str, bytes: &[u8]) -> SavedDocumentReadRequest {
        SavedDocumentReadRequest {
            project_id: self.id.clone(),
            expected_project_revision: 1,
            relative_path: path.into(),
            expected_sha256: sha(bytes),
            offset_bytes: 0,
            max_bytes: 16384,
            json_pointer: None,
        }
    }
}

use std::process::{Command, Output};
impl Fixture {
    fn run(&self, r: &SavedDocumentReadRequest) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_qiongli"));
        command
            .current_dir(&self.base)
            .env("QIONGLI_CONFIG_HOME", &self.config)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("PATH", "")
            .args([
                "project",
                "document",
                "read",
                "--project-id",
                r.project_id.as_str(),
                "--expected-project-revision",
                &r.expected_project_revision.to_string(),
                "--relative-path",
                &r.relative_path,
                "--expected-sha256",
                &r.expected_sha256,
                "--offset-bytes",
                &r.offset_bytes.to_string(),
                "--max-bytes",
                &r.max_bytes.to_string(),
            ]);
        if let Some(pointer) = &r.json_pointer {
            command.args(["--json-pointer", pointer]);
        }
        command.arg("--json").output().unwrap()
    }
}
#[test]
fn fresh_cli_reads_match_shared_owner_and_reject_drift() {
    let f = Fixture::new();
    let bytes = "public évidence\nnext".as_bytes();
    f.put("notes/Test2026.md", bytes);
    let mut r = f.request("notes/Test2026.md", bytes);
    r.max_bytes = 8;
    let output = f.run(&r);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        actual,
        serde_json::to_value(f.service.read_saved_document(&r).unwrap()).unwrap()
    );
    r.offset_bytes = actual["nextOffsetBytes"].as_u64().unwrap();
    assert!(f.run(&r).status.success());
    f.put("notes/Test2026.md", b"externally changed");
    let output = f.run(&r);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("project-revision-conflict"));
    assert_eq!(
        fs::read(f.root.join("notes/Test2026.md")).unwrap(),
        b"externally changed"
    );
}

#[test]
fn full_mcp_matches_cli_and_lite_refuses_saved_reader() {
    use std::io::Write;
    use std::process::Stdio;
    let f = Fixture::new();
    let bytes = b"public saved note";
    f.put("notes/Test2026.md", bytes);
    let r = f.request("notes/Test2026.md", bytes);
    let packet = br#"{"passages":[{"a/b~c":"ab\u00e9\u4e2d\n\"quoted\""},{"a/b~c":"ab\u00e9\u4e2d\n\"quoted\""}]}"#;
    let path = format!("sources/Public2026/{}.json", sha(packet));
    f.put(&path, packet);
    let mut selected = f.request(&path, packet);
    selected.json_pointer = Some("/passages/1/a~1b~0c".into());
    selected.max_bytes = 4;
    let output = f.run(&selected);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(first["content"], "abé");
    assert_eq!(first["sha256"], sha(packet));
    assert_eq!(first["sourceSizeBytes"], packet.len());
    selected.offset_bytes = first["nextOffsetBytes"].as_u64().unwrap();
    for r in [r, selected.clone()] {
        let args = serde_json::to_value(&r).unwrap();
        let expected: serde_json::Value = serde_json::from_slice(&f.run(&r).stdout).unwrap();
        for profile in ["full", "lite"] {
            let mut child = Command::new(env!("CARGO_BIN_EXE_qiongli"))
                .current_dir(&f.base)
                .env("QIONGLI_CONFIG_HOME", &f.config)
                .env("HOME", &f.home)
                .env("USERPROFILE", &f.home)
                .env("PATH", "")
                .args(["mcp", "serve", "--profile", profile, "--transport", "stdio"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            {
                let mut input = child.stdin.take().unwrap();
                for request in [
                    serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{}}}),
                    serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}),
                    serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
                    serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"qiongli_project_document_read","arguments":args}}),
                ] {
                    writeln!(input, "{request}").unwrap();
                }
            }
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let responses: Vec<serde_json::Value> = String::from_utf8(output.stdout)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let tools = responses.iter().find(|v| v["id"] == 2).unwrap()["result"]["tools"]
                .as_array()
                .unwrap();
            let reader = tools
                .iter()
                .find(|v| v["name"] == "qiongli_project_document_read");
            let call = responses.iter().find(|v| v["id"] == 3).unwrap();
            assert_eq!(tools.len(), if profile == "full" { 35 } else { 15 });
            if profile == "full" {
                let tool = reader.unwrap();
                assert_eq!(
                    tool["inputSchema"]["properties"]["json_pointer"]["type"],
                    "string"
                );
                assert!(
                    !tool["inputSchema"]["required"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|v| v == "json_pointer")
                );
                assert_eq!(tool["annotations"]["readOnlyHint"], true);
                assert_eq!(tool["annotations"]["destructiveHint"], false);
                let text = call["result"]["content"][0]["text"].as_str().unwrap();
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(text).unwrap(),
                    expected
                );
            } else {
                assert!(reader.is_none());
                assert!(call.get("error").is_some() || call["result"]["isError"] == true);
            }
        }
    }
    let mut rejected = selected.clone();
    rejected.json_pointer = Some("/passages/0".into());
    assert!(!f.run(&rejected).status.success());
    rejected = selected.clone();
    rejected.expected_project_revision = 2;
    assert!(!f.run(&rejected).status.success());
    f.put(&selected.relative_path, br#"{"passages":[]}"#);
    assert!(!f.run(&selected).status.success());
}

#[test]
fn saved_document_list_cli_and_full_mcp_match_without_unreceipted_discovery() {
    use std::io::Write;
    use std::process::Stdio;
    let f = Fixture::new();
    f.put(
        "notes/Test2026.md",
        b"unreceipted fixture must not be listed",
    );
    let capture = ResearchCaptureDraftV1 {
        binding: ProjectBindingV1::new(
            f.id.clone(),
            1,
            ProjectStage::Literature,
            "Save public synthetic packet",
            CapturePolicy::ReviewRequired,
        )
        .unwrap(),
        source: CaptureSource::Codex,
        delivery: CaptureDelivery::Portable,
        captured_at_unix: 110,
        summary: "Bound public fixture packet".into(),
        changes: vec![],
        decisions: vec![],
        evidence: vec![],
        contradictions: vec![],
        next_actions: vec![],
    }
    .into_capture()
    .unwrap();
    let intake = f.service.preview_capture(capture.clone()).unwrap();
    f.service
        .apply_capture(
            &intake,
            &ApprovedCaptureIntake::new(intake.preview().plan_digest.clone(), true),
            111,
        )
        .unwrap();
    let packet = SourcePacketDraftV1 {
        schema_version: 1,
        citekey: "Public2026".into(),
        content: "{\"public_fixture\":true}".into(),
    };
    let preview = f
        .service
        .preview_capture_consolidation_with_drafts(
            &f.id,
            &capture.capture_id,
            120,
            CaptureConsolidationDrafts {
                source_packet: Some(&packet),
                ..Default::default()
            },
        )
        .unwrap();
    f.service
        .apply_capture_consolidation(
            &preview,
            &ApprovedCaptureConsolidation::new(preview.preview().plan_digest.clone(), true, true),
        )
        .unwrap();
    let args = serde_json::json!({"project_id":f.id,"expected_project_revision":2});
    let output = Command::new(env!("CARGO_BIN_EXE_qiongli"))
        .current_dir(&f.base)
        .env("QIONGLI_CONFIG_HOME", &f.config)
        .env("HOME", &f.home)
        .env("USERPROFILE", &f.home)
        .env("PATH", "")
        .args([
            "project",
            "document",
            "list",
            "--project-id",
            f.id.as_str(),
            "--expected-project-revision",
            "2",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected["totalDocuments"], 1);
    assert_eq!(
        expected["documents"][0]["relativePath"],
        packet.relative_path()
    );
    let read: SavedDocumentReadRequest =
        serde_json::from_value(expected["documents"][0]["readArguments"].clone()).unwrap();
    assert_eq!(
        f.service.read_saved_document(&read).unwrap().content,
        packet.content
    );
    for profile in ["full", "lite"] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_qiongli"))
            .current_dir(&f.base)
            .env("QIONGLI_CONFIG_HOME", &f.config)
            .env("HOME", &f.home)
            .env("USERPROFILE", &f.home)
            .env("PATH", "")
            .args(["mcp", "serve", "--profile", profile, "--transport", "stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        {
            let mut input = child.stdin.take().unwrap();
            for request in [
                serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{}}}),
                serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}),
                serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
                serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"qiongli_project_document_list","arguments":args}}),
            ] {
                writeln!(input, "{request}").unwrap();
            }
        }
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let responses: Vec<serde_json::Value> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let tools = responses.iter().find(|v| v["id"] == 2).unwrap()["result"]["tools"]
            .as_array()
            .unwrap();
        let tool = tools
            .iter()
            .find(|v| v["name"] == "qiongli_project_document_list");
        let call = responses.iter().find(|v| v["id"] == 3).unwrap();
        if profile == "full" {
            assert_eq!(tool.unwrap()["annotations"]["readOnlyHint"], true);
            let actual: serde_json::Value =
                serde_json::from_str(call["result"]["content"][0]["text"].as_str().unwrap())
                    .unwrap();
            assert_eq!(actual, expected);
        } else {
            assert!(tool.is_none());
            assert!(call.get("error").is_some() || call["result"]["isError"] == true);
        }
    }
}
