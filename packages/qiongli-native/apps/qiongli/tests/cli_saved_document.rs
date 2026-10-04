#![allow(clippy::disallowed_methods)]
use qiongli_config::resolve_config_root;
use qiongli_project::{
    ApprovedProjectMutation, ProjectId, ProjectKind, ProjectRegistrationOptions,
    ProjectStateService, SavedDocumentReadRequest,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
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
        let base =
            std::env::temp_dir().join(format!("qiongli-saved-read-{}-{nonce}", std::process::id()));
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
                    .with_project_id(id.clone()),
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
        }
    }
}

use std::process::{Command, Output};
impl Fixture {
    fn run(&self, r: &SavedDocumentReadRequest) -> Output {
        Command::new(env!("CARGO_BIN_EXE_qiongli"))
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
                "--json",
            ])
            .output()
            .unwrap()
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
    let args = serde_json::json!({"project_id":r.project_id,"expected_project_revision":1,"relative_path":r.relative_path,"expected_sha256":r.expected_sha256});
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
        if profile == "full" {
            let tool = reader.unwrap();
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
