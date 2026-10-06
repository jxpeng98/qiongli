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
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[allow(dead_code)]
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
            json_pointer: None,
        }
    }
}

use qiongli_project::ProjectError;
#[test]
fn saved_documents_preserve_whole_hash_and_utf8_windows() {
    let f = Fixture::new();
    let bytes = "abé中xyz".as_bytes();
    f.put("notes/Test2026.md", bytes);
    let mut r = f.request("notes/Test2026.md", bytes);
    r.max_bytes = 4;
    let v = f.service.read_saved_document(&r).unwrap();
    assert_eq!(v.content, "abé");
    assert_eq!(v.sha256, sha(bytes));
    assert_eq!(v.next_offset_bytes, Some(4));
    assert!(v.truncated_after);
    r.offset_bytes = 4;
    let v = f.service.read_saved_document(&r).unwrap();
    assert_eq!(v.content, "中x");
    assert!(v.truncated_before);
    r.offset_bytes = bytes.len() as u64;
    let v = f.service.read_saved_document(&r).unwrap();
    assert_eq!(v.content, "");
    assert_eq!(v.next_offset_bytes, None);
    r.offset_bytes = 3;
    assert_eq!(
        f.service.read_saved_document(&r).unwrap_err(),
        ProjectError::InvalidProjectDocument
    );
    for path in [
        "retrieval_manifest.csv",
        "sources/Test2026/PLACEHOLDER.json",
    ] {
        let data = b"{\"public\":true}";
        let path = path.replace("PLACEHOLDER", &sha(data));
        f.put(&path, data);
        assert_eq!(
            f.service
                .read_saved_document(&f.request(&path, data))
                .unwrap()
                .sha256,
            sha(data)
        );
    }
}
#[test]
fn saved_document_rejects_stale_identity_hash_and_unsafe_paths() {
    let f = Fixture::new();
    let bytes = b"original";
    f.put("notes/Test2026.md", bytes);
    let r = f.request("notes/Test2026.md", bytes);
    let mut stale = r.clone();
    stale.expected_project_revision = 2;
    assert_eq!(
        f.service.read_saved_document(&stale).unwrap_err(),
        ProjectError::RevisionConflict
    );
    f.put("notes/Test2026.md", b"changed");
    assert_eq!(
        f.service.read_saved_document(&r).unwrap_err(),
        ProjectError::RevisionConflict
    );
    for path in [
        "../notes/Test2026.md",
        "/notes/Test2026.md",
        "notes/nested/Test.md",
        "context/research_state.md",
        "paper.pdf",
        "sources/Test2026/not-a-hash.json",
    ] {
        assert!(
            f.service
                .read_saved_document(&f.request(path, bytes))
                .is_err(),
            "{path}"
        );
    }
    let bad_packet = format!("sources/Test2026/{}.json", sha(bytes));
    f.put(&bad_packet, b"changed");
    assert_eq!(
        f.service
            .read_saved_document(&f.request(&bad_packet, b"changed"))
            .unwrap_err(),
        ProjectError::RevisionConflict
    );
    f.put("notes/Invalid.md", &[255]);
    assert_eq!(
        f.service
            .read_saved_document(&f.request("notes/Invalid.md", &[255]))
            .unwrap_err(),
        ProjectError::ProjectArtifactContentInvalid
    );
    let large = vec![b'x'; 4 * 1024 * 1024 + 1];
    f.put("notes/Large.md", &large);
    assert_eq!(
        f.service
            .read_saved_document(&f.request("notes/Large.md", &large))
            .unwrap_err(),
        ProjectError::DocumentTooLarge
    );
}
#[cfg(unix)]
#[test]
fn saved_document_rejects_links_and_non_regular_files() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    f.put("notes/Real.md", b"safe");
    symlink(f.root.join("notes/Real.md"), f.root.join("notes/Link.md")).unwrap();
    assert!(
        f.service
            .read_saved_document(&f.request("notes/Link.md", b"safe"))
            .is_err()
    );
    fs::create_dir(f.root.join("notes/Directory.md")).unwrap();
    assert!(
        f.service
            .read_saved_document(&f.request("notes/Directory.md", b""))
            .is_err()
    );
    fs::rename(f.root.join("notes"), f.root.join("outside-notes")).unwrap();
    symlink(f.root.join("outside-notes"), f.root.join("notes")).unwrap();
    assert!(
        f.service
            .read_saved_document(&f.request("notes/Real.md", b"safe"))
            .is_err()
    );
}

#[test]
fn saved_document_strict_request_and_bounds() {
    let f = Fixture::new();
    let bytes = b"safe";
    f.put("notes/Test2026.md", bytes);
    let r = f.request("notes/Test2026.md", bytes);
    for (offset, max) in [(0, 0), (0, 3), (0, 65537), (5, 4), (u64::MAX, 4)] {
        let mut bad = r.clone();
        bad.offset_bytes = offset;
        bad.max_bytes = max;
        assert_eq!(
            f.service.read_saved_document(&bad).unwrap_err(),
            ProjectError::InvalidProjectDocument
        );
    }
    let mut bad = r.clone();
    bad.expected_project_revision = 0;
    assert_eq!(
        f.service.read_saved_document(&bad).unwrap_err(),
        ProjectError::InvalidProjectDocument
    );
    let mut value = serde_json::json!({"project_id":r.project_id,"expected_project_revision":r.expected_project_revision,"relative_path":r.relative_path,"expected_sha256":r.expected_sha256,"offset_bytes":r.offset_bytes,"max_bytes":r.max_bytes});
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SavedDocumentReadRequest>(value).is_err());
    let mut value = serde_json::json!({"project_id":r.project_id,"expected_project_revision":r.expected_project_revision,"relative_path":r.relative_path,"expected_sha256":r.expected_sha256,"offset_bytes":r.offset_bytes,"max_bytes":r.max_bytes});
    value.as_object_mut().unwrap().remove("expected_sha256");
    assert!(serde_json::from_value::<SavedDocumentReadRequest>(value).is_err());
    let missing = f.request("notes/Missing.md", bytes);
    assert_eq!(
        f.service.read_saved_document(&missing).unwrap_err(),
        ProjectError::RevisionConflict
    );
}

#[test]
fn saved_document_identity_and_recovery_are_not_bypassed() {
    let f = Fixture::new();
    f.put("notes/Test2026.md", b"safe");
    let r = f.request("notes/Test2026.md", b"safe");
    let mut wrong = r.clone();
    wrong.project_id = ProjectId::parse("prj_0123456789abcdef0123456789abcdef").unwrap();
    assert_eq!(
        f.service.read_saved_document(&wrong).unwrap_err(),
        ProjectError::ProjectNotRegistered
    );
    let manifest = f.root.join("context/project_manifest.json");
    let original = fs::read(&manifest).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    value["projectId"] = serde_json::json!(wrong.project_id);
    fs::write(&manifest, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(f.service.read_saved_document(&r).is_err());
    fs::write(&manifest, &original).unwrap();
    fs::create_dir_all(f.root.join(".qiongli/consolidation-transaction")).unwrap();
    assert_eq!(
        f.service.read_saved_document(&r).unwrap_err(),
        ProjectError::RecoveryRequired
    );
    assert!(f.root.join(".qiongli/consolidation-transaction").exists());
}
#[cfg(unix)]
#[test]
fn saved_document_rejects_hardlinked_file() {
    let f = Fixture::new();
    f.put("notes/Original.md", b"safe");
    fs::hard_link(
        f.root.join("notes/Original.md"),
        f.root.join("notes/Linked.md"),
    )
    .unwrap();
    assert!(
        f.service
            .read_saved_document(&f.request("notes/Linked.md", b"safe"))
            .is_err()
    );
}

#[test]
fn packet_selector_decodes_strings_and_pages_the_selected_utf8_identity() {
    let f = Fixture::new();
    let text = "abé中\n\"quoted\" / ~";
    let bytes = br#"{"passages":[{"a/b~c":"ab\u00e9\u4e2d\n\"quoted\" / ~"},{"a/b~c":"ab\u00e9\u4e2d\n\"quoted\" / ~"}]}"#;
    let path = format!("sources/Public2026/{}.json", sha(bytes));
    f.put(&path, bytes);
    let mut r = f.request(&path, bytes);
    let plain = serde_json::to_value(f.service.read_saved_document(&r).unwrap()).unwrap();
    assert_eq!(plain["content"], std::str::from_utf8(bytes).unwrap());
    assert!(plain.get("jsonPointer").is_none());
    assert!(plain.get("selectedTextSizeBytes").is_none());
    assert!(
        serde_json::to_value(&r)
            .unwrap()
            .get("json_pointer")
            .is_none()
    );
    for pointer in ["/passages/0/a~1b~0c", "/passages/1/a~1b~0c"] {
        r.json_pointer = Some(pointer.into());
        r.offset_bytes = 0;
        r.max_bytes = 4;
        let first = serde_json::to_value(f.service.read_saved_document(&r).unwrap()).unwrap();
        assert_eq!(first["content"], "abé");
        assert_eq!(first["jsonPointer"], pointer);
        assert_eq!(first["selectedTextSizeBytes"], text.len());
        assert_eq!(first["sourceSizeBytes"], bytes.len());
        assert_eq!(first["sha256"], sha(bytes));
        let mut reconstructed = first["content"].as_str().unwrap().to_owned();
        let mut next = first["nextOffsetBytes"].as_u64();
        while let Some(offset) = next {
            r.offset_bytes = offset;
            let page = f.service.read_saved_document(&r).unwrap();
            assert_eq!(page.sha256, sha(bytes));
            assert!(page.truncated_before);
            reconstructed.push_str(&page.content);
            next = page.next_offset_bytes;
        }
        assert_eq!(reconstructed, text);
        r.offset_bytes = text.len() as u64;
        let end = f.service.read_saved_document(&r).unwrap();
        assert_eq!(end.content, "");
        assert_eq!(end.next_offset_bytes, None);
        r.offset_bytes = 3;
        assert!(f.service.read_saved_document(&r).is_err());
        r.offset_bytes = text.len() as u64 + 1;
        assert!(f.service.read_saved_document(&r).is_err());
    }
    let array = br#"["first","\u4e2d","",{"":"empty key"}]"#;
    let path = format!("sources/Array2026/{}.json", sha(array));
    f.put(&path, array);
    for (pointer, expected) in [
        ("/0", "first"),
        ("/1", "中"),
        ("/2", ""),
        ("/3/", "empty key"),
    ] {
        let mut r = f.request(&path, array);
        r.json_pointer = Some(pointer.into());
        assert_eq!(f.service.read_saved_document(&r).unwrap().content, expected);
    }
}

#[test]
fn packet_selector_refuses_ambiguous_json_invalid_pointers_and_non_string_targets() {
    let f = Fixture::new();
    let bytes = br#"{"text":"safe","null":null,"number":1,"object":{},"array":["safe"]}"#;
    let path = format!("sources/Public2026/{}.json", sha(bytes));
    f.put(&path, bytes);
    for pointer in [
        "",
        "text",
        "/text~",
        "/text~2",
        "/text\n",
        "/missing",
        "/null",
        "/number",
        "/object",
        "/array",
        "/array/01",
        "/array/-",
        "/array/1",
    ] {
        let mut r = f.request(&path, bytes);
        r.json_pointer = Some(pointer.into());
        assert!(
            f.service.read_saved_document(&r).is_err(),
            "pointer {pointer:?}"
        );
    }
    let mut r = f.request(&path, bytes);
    r.json_pointer = Some(format!("/{}", "é".repeat(2048)));
    assert!(f.service.read_saved_document(&r).is_err());
    for malformed in [
        br#"{"text":"one","text":"two"}"#.as_slice(),
        br#"{"text":"one","te\u0078t":"two"}"#.as_slice(),
        br#"{"text":"safe","nested":{"x":1,"x":2}}"#.as_slice(),
        br#"{"text":"safe"} trailing"#.as_slice(),
        br#"{"text":"\ud800"}"#.as_slice(),
    ] {
        let path = format!("sources/Invalid2026/{}.json", sha(malformed));
        f.put(&path, malformed);
        let mut r = f.request(&path, malformed);
        r.json_pointer = Some("/text".into());
        assert!(f.service.read_saved_document(&r).is_err());
    }
    // Count the bound in UTF-8 bytes and permit an exactly 4096-byte pointer.
    let key = format!("{}x", "é".repeat(2047));
    let boundary = serde_json::to_vec(&serde_json::json!({key.clone(): "safe"})).unwrap();
    let boundary_path = format!("sources/Boundary2026/{}.json", sha(&boundary));
    f.put(&boundary_path, &boundary);
    let mut boundary_request = f.request(&boundary_path, &boundary);
    boundary_request.json_pointer = Some(format!("/{key}"));
    assert_eq!(boundary_request.json_pointer.as_ref().unwrap().len(), 4096);
    assert_eq!(
        f.service
            .read_saved_document(&boundary_request)
            .unwrap()
            .content,
        "safe"
    );
    for other in ["notes/Public2026.md", "retrieval_manifest.csv"] {
        f.put(other, bytes);
        let mut r = f.request(other, bytes);
        r.json_pointer = Some("/text".into());
        assert!(f.service.read_saved_document(&r).is_err());
    }
}

#[test]
fn packet_selector_keeps_revision_hash_and_path_refusals() {
    let f = Fixture::new();
    let bytes = br#"{"text":"safe"}"#;
    let path = format!("sources/Public2026/{}.json", sha(bytes));
    f.put(&path, bytes);
    let mut r = f.request(&path, bytes);
    r.json_pointer = Some("/text".into());
    let mut bad = r.clone();
    bad.expected_project_revision = 2;
    assert_eq!(
        f.service.read_saved_document(&bad).unwrap_err(),
        ProjectError::RevisionConflict
    );
    bad = r.clone();
    bad.expected_sha256 = sha(b"different");
    assert_eq!(
        f.service.read_saved_document(&bad).unwrap_err(),
        ProjectError::RevisionConflict
    );
    for path in [
        format!("../{path}"),
        "sources/Public2026/not-a-hash.json".into(),
        "context/project_manifest.json".into(),
    ] {
        bad = r.clone();
        bad.relative_path = path;
        assert!(f.service.read_saved_document(&bad).is_err());
    }
    f.put(&r.relative_path, br#"{"text":"changed"}"#);
    assert_eq!(
        f.service.read_saved_document(&r).unwrap_err(),
        ProjectError::RevisionConflict
    );
    assert_eq!(
        fs::read(f.root.join(&r.relative_path)).unwrap(),
        br#"{"text":"changed"}"#
    );
}

#[cfg(unix)]
#[test]
fn packet_selector_preserves_link_and_owner_refusals() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let f = Fixture::new();
    let bytes = br#"{"text":"safe"}"#;
    let path = format!("sources/Public2026/{}.json", sha(bytes));
    f.put(&path, bytes);
    let mut r = f.request(&path, bytes);
    r.json_pointer = Some("/text".into());
    let outside = f.base.join("outside.json");
    fs::write(&outside, bytes).unwrap();
    fs::remove_file(f.root.join(&path)).unwrap();
    symlink(&outside, f.root.join(&path)).unwrap();
    assert!(f.service.read_saved_document(&r).is_err());
    fs::remove_file(f.root.join(&path)).unwrap();
    fs::hard_link(&outside, f.root.join(&path)).unwrap();
    assert!(f.service.read_saved_document(&r).is_err());
    fs::remove_file(f.root.join(&path)).unwrap();
    f.put(&path, bytes);
    fs::set_permissions(f.root.join(&path), fs::Permissions::from_mode(0o666)).unwrap();
    assert!(f.service.read_saved_document(&r).is_err());
    fs::set_permissions(f.root.join(&path), fs::Permissions::from_mode(0o600)).unwrap();
    fs::rename(f.root.join("sources"), f.root.join("outside-sources")).unwrap();
    symlink(f.root.join("outside-sources"), f.root.join("sources")).unwrap();
    assert!(f.service.read_saved_document(&r).is_err());
    assert_eq!(fs::read(&outside).unwrap(), bytes);
}
