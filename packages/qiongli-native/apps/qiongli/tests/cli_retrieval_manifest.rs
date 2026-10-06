#![allow(clippy::disallowed_methods)]

use qiongli_config::resolve_config_root;
use qiongli_project::{
    ApprovedCaptureConsolidation, ApprovedCaptureIntake, ApprovedProjectMutation,
    CaptureConsolidationDrafts, CaptureDelivery, CapturePolicy, CaptureSource, ProjectBindingV1,
    ProjectId, ProjectKind, ProjectRegistrationOptions, ProjectStage, ProjectStateService,
    ResearchCaptureDraftV1, SourcePacketDraftV1,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    base: PathBuf,
    root: PathBuf,
    config: PathBuf,
    home: PathBuf,
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
            "qiongli-manifest-cli-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&base).unwrap();
        let base = fs::canonicalize(base).unwrap();
        let root = base.join("paper");
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
        let registration = service
            .preview_register(
                &root,
                ProjectRegistrationOptions::new(
                    "Synthetic retrieval history",
                    ProjectKind::Article,
                )
                .with_project_id(id.clone())
                .with_stage(ProjectStage::Literature),
                100,
            )
            .unwrap();
        service
            .apply(
                &registration,
                &ApprovedProjectMutation::new(registration.preview().plan_digest.clone(), true),
                100,
            )
            .unwrap();
        Self {
            base,
            root,
            config,
            home,
            service,
            id,
        }
    }
    fn capture(&self, revision: u64) -> String {
        let capture = ResearchCaptureDraftV1 {
            binding: ProjectBindingV1::new(
                self.id.clone(),
                revision,
                ProjectStage::Literature,
                "Preserve actual retrieval attempts",
                CapturePolicy::ReviewRequired,
            )
            .unwrap(),
            source: CaptureSource::Codex,
            delivery: CaptureDelivery::Portable,
            captured_at_unix: 100 + revision,
            summary: format!("Reviewed synthetic retrieval attempt {revision}"),
            changes: vec![],
            decisions: vec![],
            evidence: vec![],
            contradictions: vec![],
            next_actions: vec![],
        }
        .into_capture()
        .unwrap();
        let plan = self.service.preview_capture(capture.clone()).unwrap();
        self.service
            .apply_capture(
                &plan,
                &ApprovedCaptureIntake::new(plan.preview().plan_digest.clone(), true),
                110 + revision,
            )
            .unwrap();
        capture.capture_id.as_str().to_owned()
    }
    fn run(&self, capture: &str, file: &Path, apply: Option<(&str, bool, bool)>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_qiongli"));
        command
            .current_dir(&self.base)
            .env("QIONGLI_CONFIG_HOME", &self.config)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("PATH", "")
            .args([
                "project",
                "capture",
                "consolidate",
                if apply.is_some() { "apply" } else { "preview" },
                "--project-id",
                self.id.as_str(),
                "--capture-id",
                capture,
                "--reviewed-at-unix",
                "200",
                "--retrieval-manifest-file",
            ])
            .arg(file);
        if let Some((digest, academic, filesystem)) = apply {
            command.args(["--expected-plan-digest", digest]);
            if academic {
                command.arg("--approve-academic-review");
            }
            if filesystem {
                command.arg("--approve-filesystem-write");
            }
        }
        command.arg("--json").output().unwrap()
    }
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files = vec![];
    if root.exists() {
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                files.extend(snapshot(&path));
            } else {
                files.push((path.clone(), fs::read(path).unwrap()));
            }
        }
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}
fn attempt(provider: &str, status: &str, notes: &str) -> Value {
    json!({"recordId":"R-001", "citekey":"Synthetic2026", "doi":"", "retrievalStatus":status, "versionLabel":"unknown", "sourceProvider":provider, "retrievedAt":"", "fulltextPath":"", "accessUrl":"https://example.org/article", "license":"", "notes":notes})
}
fn write_draft(path: &Path, previous: Option<String>, attempts: Vec<Value>) {
    fs::write(
        path,
        serde_json::to_vec(
            &json!({"schemaVersion":1,"previousSha256":previous,"attempts":attempts}),
        )
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn cli_preserves_attempt_history_and_restarts_without_implicit_approval() {
    let fixture = Fixture::new();
    let packet_capture = fixture.capture(1);
    let packet = SourcePacketDraftV1 {
        schema_version: 1,
        citekey: "Synthetic2026".into(),
        content: "{\"coverage\":\"body excerpt only\",\"segments\":[\"synthetic passage\"]}".into(),
    };
    let packet_plan = fixture
        .service
        .preview_capture_consolidation_with_drafts(
            &fixture.id,
            &qiongli_project::CaptureId::parse(packet_capture).unwrap(),
            150,
            CaptureConsolidationDrafts {
                source_packet: Some(&packet),
                ..Default::default()
            },
        )
        .unwrap();
    fixture
        .service
        .apply_capture_consolidation(
            &packet_plan,
            &ApprovedCaptureConsolidation::new(
                packet_plan.preview().plan_digest.clone(),
                true,
                true,
            ),
        )
        .unwrap();
    let capture = fixture.capture(2);
    let draft = fixture.base.join("manifest.json");
    let native = attempt(
        "native fulltext",
        "not_retrieved:broken_link",
        "DNS lookup failed; access state unknown",
    );
    let mut host = attempt(
        "Host public retrieval",
        "retrieved_oa",
        "Excerpt only, \"quoted\"\nIdentity remains reviewed",
    );
    host["sourcePacket"] =
        json!({"relativePath":packet.relative_path(),"sha256":sha(packet.content.as_bytes())});
    write_draft(&draft, None, vec![native, host]);
    let before = snapshot(&fixture.base);
    let preview = success(fixture.run(&capture, &draft, None));
    assert_eq!(snapshot(&fixture.base), before);
    let content = preview["retrievalManifestContent"].as_str().unwrap();
    assert!(content.contains("native fulltext"));
    assert!(content.contains("Host public retrieval"));
    assert!(content.contains("not_retrieved:broken_link,unknown"));
    assert!(content.contains("retrieved_oa,unknown"));
    assert!(content.contains("\"\"quoted\"\""));
    assert!(content.contains("not a PDF digest"));
    let digest = preview["preview"]["planDigest"].as_str().unwrap();
    for (academic, filesystem) in [(false, false), (true, false), (false, true)] {
        assert!(
            !fixture
                .run(&capture, &draft, Some((digest, academic, filesystem)))
                .status
                .success()
        );
        assert_eq!(snapshot(&fixture.base), before);
    }
    success(fixture.run(&capture, &draft, Some((digest, true, true))));
    let prior = fs::read(fixture.root.join("retrieval_manifest.csv")).unwrap();
    assert_eq!(prior, content.as_bytes());
    assert!(
        !fixture
            .run(&capture, &draft, Some((digest, true, true)))
            .status
            .success()
    );
    let later = fixture.capture(3);
    write_draft(
        &draft,
        Some(sha(&prior)),
        vec![attempt(
            "Host retry",
            "abstract_only",
            "Later session retains limits",
        )],
    );
    let later_preview = success(fixture.run(&later, &draft, None));
    let next = later_preview["retrievalManifestContent"].as_str().unwrap();
    assert!(next.as_bytes().starts_with(&prior));
    success(fixture.run(
        &later,
        &draft,
        Some((
            later_preview["preview"]["planDigest"].as_str().unwrap(),
            true,
            true,
        )),
    ));
    assert_eq!(
        fs::read(fixture.root.join("retrieval_manifest.csv")).unwrap(),
        next.as_bytes()
    );
}

#[test]
fn cli_refuses_changed_drafts_packet_impersonation_and_manifest_collision() {
    let fixture = Fixture::new();
    let capture = fixture.capture(1);
    let draft = fixture.base.join("manifest.json");
    write_draft(
        &draft,
        None,
        vec![attempt(
            "native",
            "not_retrieved:broken_link",
            "Original reviewed failure",
        )],
    );
    let preview = success(fixture.run(&capture, &draft, None));
    let digest = preview["preview"]["planDigest"].as_str().unwrap();
    write_draft(
        &draft,
        None,
        vec![attempt("Host", "abstract_only", "Changed after review")],
    );
    let before = snapshot(&fixture.base);
    assert!(
        !fixture
            .run(&capture, &draft, Some((digest, true, true)))
            .status
            .success()
    );
    assert_eq!(snapshot(&fixture.base), before);
    let mut forged = attempt(
        "Host",
        "retrieved_oa",
        "Raw packet is not a local fulltext file",
    );
    forged["fulltextPath"] = json!(format!("sources/Synthetic2026/{}.json", "a".repeat(64)));
    forged["fulltextSha256"] = json!("a".repeat(64));
    write_draft(&draft, None, vec![forged]);
    let before = snapshot(&fixture.base);
    assert!(!fixture.run(&capture, &draft, None).status.success());
    assert_eq!(snapshot(&fixture.base), before);
    write_draft(
        &draft,
        None,
        vec![attempt(
            "native",
            "not_retrieved:broken_link",
            "Original reviewed failure",
        )],
    );
    fs::write(
        fixture.root.join("retrieval_manifest.csv"),
        b"Competing user file\n",
    )
    .unwrap();
    let before = snapshot(&fixture.base);
    assert!(
        !fixture
            .run(&capture, &draft, Some((digest, true, true)))
            .status
            .success()
    );
    assert_eq!(snapshot(&fixture.base), before);
}
