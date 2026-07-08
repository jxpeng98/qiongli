use std::sync::Mutex;

use qiongli_lite_mcp::config::provider_config::{
    provider_config_path, save_provider_value, summary,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn provider_config_path_uses_qiongli_config_home() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);

    let path = provider_config_path();
    assert_eq!(path, temp.join("providers.json"));

    std::env::remove_var("QIONGLI_CONFIG_HOME");
}

#[test]
fn save_and_summarize_provider_value_without_exposing_secret() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);

    save_provider_value("openalex", "api-key", "openalex-secret-key").unwrap();
    let status = summary().unwrap();
    let serialized = serde_json::to_string(&status).unwrap();

    assert_eq!(status.providers["openalex"], "configured");
    assert_eq!(status.providers["arxiv"], "configured");
    assert!(!serialized.contains("openalex-secret-key"));

    std::env::remove_var("QIONGLI_CONFIG_HOME");
}

fn tempfile_dir() -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "qiongli-lite-mcp-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

