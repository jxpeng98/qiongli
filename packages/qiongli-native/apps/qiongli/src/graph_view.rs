//! Offline presentation of an existing projection, with no project write authority.
use std::io;
use std::path::Path;

use qiongli_config::{ConfigRoot, GlobalSettingsStore};
use qiongli_project::{
    AcademicGraphProjectionV1, AcademicGraphService, ProjectId, ProjectStateService,
};
use serde_json::json;

use crate::CliOutput;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GraphViewMode {
    Html,
    Save,
    Open,
}

pub(crate) fn execute(
    projects: &ProjectStateService,
    project_id: &ProjectId,
    root: &ConfigRoot,
    mode: GraphViewMode,
) -> CliOutput {
    match AcademicGraphService::new(projects.clone()).rebuild_projection(project_id) {
        Ok(projection) => match render(&projection) {
            Ok(html) => deliver(&html, root, mode, |path| open::that_detached(path)),
            Err(_) => CliOutput::operation_failure("output-serialization-failed"),
        },
        Err(error) => CliOutput::operation_failure(error.reason_code()),
    }
}

fn deliver(
    html: &str,
    root: &ConfigRoot,
    mode: GraphViewMode,
    opener: impl FnOnce(&Path) -> io::Result<()>,
) -> CliOutput {
    if mode == GraphViewMode::Html {
        return CliOutput::success_text(html);
    }
    if let Err(error) = GlobalSettingsStore::new(root.clone()).prepare_store() {
        return CliOutput::operation_failure(error.reason_code());
    }
    let mut nonce = [0_u8; 16];
    if getrandom::fill(&mut nonce).is_err() {
        return CliOutput::operation_failure("graph-export-identity-unavailable");
    }
    let nonce = u128::from_be_bytes(nonce);
    // A fresh private derived file: no canonical write, overwrite or automatic deletion.
    let path = root
        .state_root()
        .join(format!("graph-view-{nonce:032x}.html"));
    if crate::managed_content::write_new_private_file(&path, html.as_bytes()).is_err() {
        return CliOutput::operation_failure("graph-export-write-failed");
    }
    let message = format!(
        "Graph snapshot saved: {path:?}\nThis is an offline snapshot; save a new view after project changes.\n"
    );
    if mode == GraphViewMode::Open {
        if opener(&path).is_err() {
            return CliOutput::operation_failure("graph-open-unavailable").with_stdout(format!(
                "{message}The saved file is available. Open it manually in your browser.\n"
            ));
        }
        return CliOutput::success_text(format!(
            "{message}Open request sent to the default HTML application. If no window appears, open the saved file manually.\n"
        ));
    }
    CliOutput::success_text(format!(
        "{message}Open this file in your browser when ready.\n"
    ))
}

fn render(projection: &AcademicGraphProjectionV1) -> Result<String, serde_json::Error> {
    let data = serde_json::to_string(&json!({
        "snapshot": projection.graph, "readiness": projection.readiness
    }))?;
    Ok(document(&data))
}

fn document(data: &str) -> String {
    // JSON escaping alone does not prevent an HTML script end tag.
    let data = data
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    include_str!("graph_view.html").replace("__QIONGLI_GRAPH_DATA__", &data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_views_are_private_unique_and_preserved_when_opening_fails() {
        use std::fs;
        let temporary = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "qiongli-graph-open-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&temporary).unwrap();
        let root = qiongli_config::resolve_config_root(
            Some(temporary.join("config").as_os_str()),
            &temporary,
        )
        .unwrap();
        let html = "<!doctype html><title>Research snapshot</title>";
        let plain = deliver(html, &root, GraphViewMode::Html, |_| {
            panic!("stdout must not open a browser")
        });
        assert_eq!(plain.stdout(), html);
        assert!(!root.state_root().exists());
        let saved = deliver(html, &root, GraphViewMode::Save, |_| {
            panic!("save must not open a browser")
        });
        assert_eq!(saved.exit_code(), 0);
        let failed = deliver(html, &root, GraphViewMode::Open, |path| {
            assert_eq!(fs::read_to_string(path).unwrap(), html);
            Err(io::Error::other("test opener unavailable"))
        });
        assert_ne!(failed.exit_code(), 0);
        assert!(failed.stdout().contains("Open it manually"));
        let files = fs::read_dir(root.state_root())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 2);
        for path in &files {
            assert_eq!(fs::read_to_string(path).unwrap(), html);
            #[cfg(windows)]
            assert!(qiongli_windows_security::open_owner_only_file(path).is_ok());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(path).unwrap().permissions().mode() & 0o777,
                    0o600
                );
            }
        }
        assert!(crate::managed_content::write_new_private_file(&files[0], b"overwrite").is_err());
        assert_eq!(fs::read_to_string(&files[0]).unwrap(), html);
        let opened = deliver(html, &root, GraphViewMode::Open, |path| {
            assert_eq!(fs::read_to_string(path).unwrap(), html);
            Ok(())
        });
        assert_eq!(opened.exit_code(), 0);
        assert!(opened.stdout().contains("Open request sent"));
        #[cfg(unix)]
        {
            let linked = temporary.join("linked");
            std::os::unix::fs::symlink(root.compatibility_root(), &linked).unwrap();
            let unsafe_root =
                qiongli_config::resolve_config_root(Some(linked.as_os_str()), &temporary).unwrap();
            let denied = deliver(html, &unsafe_root, GraphViewMode::Open, |_| {
                panic!("unsafe root must not launch")
            });
            assert_ne!(denied.exit_code(), 0);
        }
        fs::remove_dir_all(temporary).unwrap();
    }

    #[test]
    fn offline_graph_embeds_data_without_html_or_script_injection() {
        let data = json!({"label": "</script><script>alert(1)</script>&\u{2028}",
                          "value": "中文 __QIONGLI_GRAPH_DATA__"});
        let html = document(&data.to_string());
        let serialized = html
            .split("<script id=\"graph-data\" type=\"application/json\">")
            .nth(1)
            .unwrap()
            .split("</script>")
            .next()
            .unwrap();
        assert!(!serialized.contains('<'));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(serialized).unwrap(),
            data
        );
        assert!(html.contains("connect-src 'none'"));
        assert!(!html.contains("<script src="));
        assert!(!html.contains(".innerHTML"));
    }
}
