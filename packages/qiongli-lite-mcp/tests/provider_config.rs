use std::sync::Mutex;

use qiongli_lite_mcp::config::provider_config::{
    provider_config_path, provider_field_specs, resolve_provider_config, save_provider_value,
    summary, ConfigError, ProviderFieldRole, ResolvedProviderConfig,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn provider_config_path_uses_qiongli_config_home() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);

    let path = provider_config_path().unwrap();
    assert_eq!(path, temp.join("providers.json"));

    std::env::remove_var("QIONGLI_CONFIG_HOME");
}

#[cfg(any(unix, windows))]
#[test]
fn default_and_tilde_paths_resolve_against_the_platform_home() {
    let _guard = ENV_LOCK.lock().unwrap();
    let home = tempfile_dir().join("platform-home");
    std::fs::create_dir_all(&home).unwrap();
    let previous_config_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let previous_home: Vec<_> = platform_home_variables()
        .iter()
        .map(|name| (*name, std::env::var_os(name)))
        .collect();
    std::env::remove_var("QIONGLI_CONFIG_HOME");
    set_platform_home(&home);

    assert_eq!(
        provider_config_path().unwrap(),
        home.join(".config/qiongli/providers.json")
    );
    std::env::set_var("QIONGLI_CONFIG_HOME", "~");
    assert_eq!(provider_config_path().unwrap(), home.join("providers.json"));
    std::env::set_var("QIONGLI_CONFIG_HOME", "~/nested/config");
    assert_eq!(
        provider_config_path().unwrap(),
        home.join("nested/config/providers.json")
    );
    for invalid in ["~//abs", r"~/\abs", r"~/C:\abs", "~/C:relative"] {
        std::env::set_var("QIONGLI_CONFIG_HOME", invalid);
        let error = provider_config_path().expect_err("portable absolute form must fail");
        assert!(matches!(error, ConfigError::InvalidConfigHome));
        assert_eq!(
            error.to_string(),
            "provider config home must be a fully qualified absolute path or start with ~/"
        );
        assert!(!error.to_string().contains(invalid));
    }
    assert_eq!(std::fs::read_dir(&home).unwrap().count(), 0);

    restore_env("QIONGLI_CONFIG_HOME", previous_config_home);
    for (name, value) in previous_home {
        restore_env(name, value);
    }
}

#[test]
fn relative_config_home_is_rejected_without_writing_to_cwd() {
    let _guard = ENV_LOCK.lock().unwrap();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let relative = format!(
        "qiongli-relative-config-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let cwd_root = std::path::PathBuf::from(&relative);
    let cwd_target = cwd_root.join("providers.json");
    assert!(!cwd_root.exists());
    std::env::set_var("QIONGLI_CONFIG_HOME", &relative);

    let path_error = provider_config_path().expect_err("relative path must fail");
    let save_error = save_provider_value("crossref", "email", "person@example.com")
        .expect_err("relative path must not be written");
    let summary_error = summary().expect_err("relative path must not be read");

    assert!(matches!(path_error, ConfigError::InvalidConfigHome));
    assert!(matches!(save_error, ConfigError::InvalidConfigHome));
    assert!(matches!(summary_error, ConfigError::InvalidConfigHome));
    assert_eq!(
        path_error.to_string(),
        "provider config home must be a fully qualified absolute path or start with ~/"
    );
    assert!(!path_error.to_string().contains(&relative));
    assert!(!cwd_root.exists());
    assert!(!cwd_target.exists());
    restore_env("QIONGLI_CONFIG_HOME", previous_home);
}

#[cfg(any(unix, windows))]
#[test]
fn missing_platform_home_fails_closed() {
    let _guard = ENV_LOCK.lock().unwrap();
    let previous_config_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let previous_home: Vec<_> = platform_home_variables()
        .iter()
        .map(|name| (*name, std::env::var_os(name)))
        .collect();
    std::env::remove_var("QIONGLI_CONFIG_HOME");
    for name in platform_home_variables() {
        std::env::remove_var(name);
    }

    assert!(matches!(
        provider_config_path(),
        Err(ConfigError::HomeUnavailable)
    ));
    assert!(matches!(
        save_provider_value("crossref", "email", "person@example.com"),
        Err(ConfigError::HomeUnavailable)
    ));
    assert!(matches!(summary(), Err(ConfigError::HomeUnavailable)));

    restore_env("QIONGLI_CONFIG_HOME", previous_config_home);
    for (name, value) in previous_home {
        restore_env(name, value);
    }
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
fn environment_credentials_enable_a_persisted_disabled_provider() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let aliases = [
        "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
        "SEMANTIC_SCHOLAR_API_KEY",
        "S2_API_KEY",
        "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
    ];
    let previous_aliases = aliases.map(|name| (name, std::env::var_os(name)));
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);
    for alias in aliases {
        std::env::remove_var(alias);
    }
    std::fs::write(
        temp.join("providers.json"),
        r#"{"version":1,"providers":{"semantic_scholar":{"enabled":false}}}"#,
    )
    .unwrap();
    std::env::set_var("QIONGLI_SEMANTIC_SCHOLAR_API_KEY", "environment-secret");

    let resolved = resolve_provider_config().unwrap();

    assert!(resolved.is_enabled("semantic_scholar"));
    assert!(resolved.is_configured("semantic_scholar"));
    assert!(resolved.is_active("semantic_scholar"));
    assert_eq!(
        resolved.value("semantic_scholar", "api_key"),
        Some("environment-secret")
    );
    restore_env("QIONGLI_CONFIG_HOME", previous_home);
    for (name, value) in previous_aliases {
        restore_env(name, value);
    }
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
        r#"{
            "version": 1,
            "providers": {
                "crossref": {"future_provider_field": {"mode": "next"}},
                "future_provider": ["opaque", 2]
            },
            "search": {
                "minimum_productive_providers": 3,
                "allow_platform_search_supplement": false,
                "future_search_field": {"mode": "next"}
            },
            "future_extension": {"enabled": true}
        }"#,
    )
    .unwrap();

    save_provider_value("crossref", "email", "person@example.com").unwrap();
    let payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

    assert_eq!(payload["future_extension"]["enabled"], true);
    assert_eq!(
        payload["providers"]["crossref"]["future_provider_field"]["mode"],
        "next"
    );
    assert_eq!(payload["providers"]["future_provider"][0], "opaque");
    assert_eq!(payload["search"]["minimum_productive_providers"], 3);
    assert_eq!(payload["search"]["allow_platform_search_supplement"], false);
    assert_eq!(payload["search"]["future_search_field"]["mode"], "next");
    restore_env("QIONGLI_CONFIG_HOME", previous_home);
}

#[test]
fn saving_migrates_unique_legacy_aliases_to_canonical_keys() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    let aliases = [
        "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
        "SEMANTIC_SCHOLAR_API_KEY",
        "S2_API_KEY",
        "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY",
    ];
    let previous_aliases = aliases.map(|name| (name, std::env::var_os(name)));
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);
    for alias in aliases {
        std::env::remove_var(alias);
    }
    let path = temp.join("providers.json");
    std::fs::write(
        &path,
        r#"{
            "version": 1,
            "providers": {
                "semantic-scholar": {
                    "enabled": false,
                    "api-key": "legacy-key",
                    "future-field": {"keep": true}
                },
                "future-provider": {"keep": true}
            },
            "search": {
                "minimum-productive-providers": 2,
                "future-setting": {"keep": true}
            }
        }"#,
    )
    .unwrap();

    save_provider_value("s2", "api-key", "replacement-key").unwrap();
    let resolved = resolve_provider_config().unwrap();
    let payload: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

    assert!(resolved.is_active("semantic_scholar"));
    assert_eq!(
        resolved.value("semantic_scholar", "api_key"),
        Some("replacement-key")
    );
    assert!(payload["providers"].get("semantic-scholar").is_none());
    assert!(payload["providers"]["semantic_scholar"]
        .get("api-key")
        .is_none());
    assert_eq!(
        payload["providers"]["semantic_scholar"]["api_key"],
        "replacement-key"
    );
    assert_eq!(
        payload["providers"]["semantic_scholar"]["future-field"]["keep"],
        true
    );
    assert_eq!(payload["providers"]["future-provider"]["keep"], true);
    assert!(payload["search"]
        .get("minimum-productive-providers")
        .is_none());
    assert_eq!(payload["search"]["minimum_productive_providers"], 2);
    assert_eq!(payload["search"]["future-setting"]["keep"], true);

    restore_env("QIONGLI_CONFIG_HOME", previous_home);
    for (name, value) in previous_aliases {
        restore_env(name, value);
    }
}

#[test]
fn malformed_known_config_fails_closed_without_overwriting_original() {
    let _guard = ENV_LOCK.lock().unwrap();
    let temp = tempfile_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    std::env::set_var("QIONGLI_CONFIG_HOME", &temp);
    let path = temp.join("providers.json");
    let malformed_payloads = [
        r#"[]"#,
        r#"{"version":0}"#,
        r#"{"version":2}"#,
        r#"{"version":true}"#,
        r#"{"providers":[]}"#,
        r#"{"providers":{"openalex":[]}}"#,
        r#"{"providers":{"openalex":{"enabled":"credential-canary"}}}"#,
        r#"{"providers":{"openalex":{"api_key":{"secret":"credential-canary"}}}}"#,
        r#"{"providers":{"semantic-scholar":{"api_key":"first"},"semantic_scholar":{"api_key":"second"}}}"#,
        r#"{"providers":{"openalex":{"api-key":"first","api_key":"second"}}}"#,
        r#"{"search":[]}"#,
        r#"{"search":{"minimum_productive_providers":true}}"#,
        r#"{"search":{"minimum_productive_providers":0}}"#,
        r#"{"search":{"minimum_productive_providers":-1}}"#,
        r#"{"search":{"minimum-productive-providers":2,"minimum_productive_providers":3}}"#,
        r#"{"search":{"allow_platform_search_supplement":1}}"#,
    ];

    for payload in malformed_payloads {
        std::fs::write(&path, payload).unwrap();
        let original = std::fs::read(&path).unwrap();

        let read_error = resolve_provider_config().err().expect("config must fail");
        let save_error =
            save_provider_value("crossref", "email", "new-value").expect_err("save must fail");

        assert!(!read_error.to_string().contains("credential-canary"));
        assert!(!save_error.to_string().contains("credential-canary"));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        let entries: Vec<_> = std::fs::read_dir(&temp)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, vec![std::ffi::OsString::from("providers.json")]);
    }

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

#[cfg(unix)]
fn platform_home_variables() -> &'static [&'static str] {
    &["HOME"]
}

#[cfg(windows)]
fn platform_home_variables() -> &'static [&'static str] {
    &["USERPROFILE", "HOMEDRIVE", "HOMEPATH"]
}

#[cfg(unix)]
fn set_platform_home(path: &std::path::Path) {
    std::env::set_var("HOME", path);
}

#[cfg(windows)]
fn set_platform_home(path: &std::path::Path) {
    std::env::set_var("USERPROFILE", path);
    std::env::remove_var("HOMEDRIVE");
    std::env::remove_var("HOMEPATH");
}

fn restore_env(name: &str, value: Option<std::ffi::OsString>) {
    if let Some(value) = value {
        std::env::set_var(name, value);
    } else {
        std::env::remove_var(name);
    }
}
