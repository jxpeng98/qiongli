//! Offline presentation of an existing projection, with no project write authority.
use qiongli_project::{
    AcademicGraphProjectionV1, AcademicGraphService, ProjectId, ProjectStateService,
};
use serde_json::json;

use crate::CliOutput;

pub(crate) fn execute(projects: &ProjectStateService, project_id: &ProjectId) -> CliOutput {
    match AcademicGraphService::new(projects.clone()).rebuild_projection(project_id) {
        Ok(projection) => match render(&projection) {
            Ok(html) => CliOutput::success_text(html),
            Err(_) => CliOutput::operation_failure("output-serialization-failed"),
        },
        Err(error) => CliOutput::operation_failure(error.reason_code()),
    }
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
