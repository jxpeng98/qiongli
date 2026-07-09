use std::sync::Mutex;

use qiongli_lite_mcp::config::provider_config::{
    provider_config_path, provider_field_specs, resolve_provider_config, save_provider_value,
    summary, ProviderFieldRole, ResolvedProviderConfig,
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
    assert_eq!(status.config_path, temp.join("providers.json"));
    assert_eq!(
        status.redacted_config.providers["openalex"].fields["api_key"],
        "configured"
    );
    assert_eq!(
        status.redacted_config.providers["openalex"].fields["email"],
        "missing"
    );
    assert!(!serialized.contains("openalex-secret-key"));

    std::env::remove_var("QIONGLI_CONFIG_HOME");
}

#[test]
fn optional_openalex_email_does_not_activate_provider() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let aliases = [
        "QIONGLI_OPENALEX_API_KEY",
        "OPENALEX_API_KEY",
        "QIONGLI_MCPB_OPENALEX_API_KEY",
        "QIONGLI_OPENALEX_EMAIL",
        "OPENALEX_EMAIL",
        "QIONGLI_MCPB_OPENALEX_EMAIL",
    ];
    let previous_aliases = aliases.map(|name| (name, std::env::var_os(name)));
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);
    for alias in aliases {
        std::env::remove_var(alias);
    }

    save_provider_value("openalex", "email", "person@example.com").unwrap();
    let resolved = resolve_provider_config().unwrap();
    let status = summary().unwrap();

    assert_eq!(
        resolved.value("openalex", "email"),
        Some("person@example.com")
    );
    assert!(!resolved.is_configured("openalex"));
    assert_eq!(status.providers["openalex"], "missing");
    assert!(status.missing.contains(&"openalex.api_key".to_string()));
    assert!(!status.missing.contains(&"openalex.email".to_string()));

    restore_env("QIONGLI_CONFIG_HOME", previous_home);
    for (name, value) in previous_aliases {
        restore_env(name, value);
    }
}

#[test]
fn mcpb_environment_aliases_resolve_without_leaking_values() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let previous_key = std::env::var_os("QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY");
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);
    std::env::set_var("QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY", "semantic-secret");

    let resolved = resolve_provider_config().unwrap();
    let status = summary().unwrap();
    let rendered = serde_json::to_string(&status).unwrap();

    assert!(resolved.is_active("semantic-scholar"));
    assert_eq!(
        resolved.value("semantic_scholar", "api_key"),
        Some("semantic-secret")
    );
    assert_eq!(status.providers["semantic_scholar"], "configured");
    assert!(!rendered.contains("semantic-secret"));

    restore_env("QIONGLI_CONFIG_HOME", previous_home);
    restore_env("QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY", previous_key);
}

#[test]
fn every_mcpb_environment_alias_matches_the_shared_provider_contract() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let alias_names = [
        "QIONGLI_MCPB_OPENALEX_API_KEY",
        "QIONGLI_MCPB_OPENALEX_EMAIL",
        "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
        "QIONGLI_MCPB_CROSSREF_EMAIL",
        "QIONGLI_MCPB_PUBMED_API_KEY",
    ];
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let previous_aliases = alias_names.map(|name| (name, std::env::var_os(name)));
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);
    for (name, value) in [
        (alias_names[0], "openalex-key"),
        (alias_names[1], "openalex@example.com"),
        (alias_names[2], "semantic-key"),
        (alias_names[3], "crossref@example.com"),
        (alias_names[4], "pubmed-key"),
    ] {
        std::env::set_var(name, value);
    }

    let resolved = resolve_provider_config().unwrap();
    let rendered = serde_json::to_string(&summary().unwrap()).unwrap();

    assert_eq!(resolved.value("openalex", "api_key"), Some("openalex-key"));
    assert_eq!(
        resolved.value("openalex", "email"),
        Some("openalex@example.com")
    );
    assert_eq!(
        resolved.value("semantic_scholar", "api_key"),
        Some("semantic-key")
    );
    assert_eq!(
        resolved.value("crossref", "email"),
        Some("crossref@example.com")
    );
    assert_eq!(resolved.value("pubmed", "api_key"), Some("pubmed-key"));
    for secret in [
        "openalex-key",
        "openalex@example.com",
        "semantic-key",
        "crossref@example.com",
        "pubmed-key",
    ] {
        assert!(!rendered.contains(secret));
    }

    restore_env("QIONGLI_CONFIG_HOME", previous_home);
    for (name, value) in previous_aliases {
        restore_env(name, value);
    }
}

#[cfg(unix)]
#[test]
fn saved_provider_config_uses_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);

    let path = save_provider_value("pubmed", "api_key", "permission-secret").unwrap();
    let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;

    assert_eq!(mode, 0o600);
    restore_env("QIONGLI_CONFIG_HOME", previous_home);
}

#[test]
fn saving_a_provider_value_preserves_unknown_future_top_level_fields() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);
    let path = temp.join("providers.json");
    std::fs::write(
        &path,
        r#"{"version":1,"providers":{},"future_extension":{"enabled":true}}"#,
    )
    .unwrap();

    save_provider_value("crossref", "email", "person@example.com").unwrap();
    let payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

    assert_eq!(payload["future_extension"]["enabled"], true);
    restore_env("QIONGLI_CONFIG_HOME", previous_home);
}

#[test]
fn provider_field_specs_distinguish_activation_from_optional_values() {
    let openalex: Vec<_> = provider_field_specs()
        .iter()
        .filter(|spec| spec.provider == "openalex")
        .map(|spec| (spec.field, spec.role))
        .collect();

    assert_eq!(
        openalex,
        vec![
            ("api_key", ProviderFieldRole::ActivationRequired),
            ("email", ProviderFieldRole::Optional),
        ]
    );
}

#[test]
fn injected_provider_values_support_runtime_tests_without_process_environment() {
    let resolved = ResolvedProviderConfig::from_values(&[
        ("s2", "api-key", "semantic-secret"),
        ("openalex", "email", "person@example.com"),
    ])
    .unwrap();

    assert!(resolved.is_active("semantic_scholar"));
    assert!(!resolved.is_configured("openalex"));
    assert_eq!(
        resolved.value("openalex", "email"),
        Some("person@example.com")
    );
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

fn restore_env(name: &str, value: Option<std::ffi::OsString>) {
    if let Some(value) = value {
        std::env::set_var(name, value);
    } else {
        std::env::remove_var(name);
    }
}
