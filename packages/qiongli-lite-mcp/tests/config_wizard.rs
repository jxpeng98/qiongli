use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use qiongli_lite_mcp::config::provider_config::ConfigError;
use qiongli_lite_mcp::config::wizard::{start_config_wizard, ConfigWizardOptions, WizardError};
use url::Url;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn rejects_non_loopback_host_and_unsupported_provider() {
    let _guard = ENV_LOCK.lock().unwrap();
    let non_loopback = start_config_wizard(ConfigWizardOptions {
        host: "0.0.0.0".to_string(),
        ..ConfigWizardOptions::default()
    });
    assert!(matches!(non_loopback, Err(WizardError::NonLoopbackHost)));

    let unknown_provider = start_config_wizard(ConfigWizardOptions {
        provider: Some("unknown".to_string()),
        ..ConfigWizardOptions::default()
    });
    assert!(matches!(
        unknown_provider,
        Err(WizardError::UnsupportedProvider(_))
    ));
}

#[test]
fn uses_distinct_random_tokens_and_normalizes_localhost() {
    let _guard = ENV_LOCK.lock().unwrap();
    let first = start_config_wizard(ConfigWizardOptions {
        host: "localhost".to_string(),
        ttl: Duration::from_secs(2),
        ..ConfigWizardOptions::default()
    })
    .unwrap();
    let second = start_config_wizard(ConfigWizardOptions {
        ttl: Duration::from_secs(2),
        ..ConfigWizardOptions::default()
    })
    .unwrap();

    assert_eq!(first.host(), "127.0.0.1");
    assert_ne!(token(first.url()), token(second.url()));
    assert!(token(first.url()).len() >= 32);

    first.stop();
    second.stop();
    assert!(first.wait_until_stopped(Duration::from_secs(1)));
    assert!(second.wait_until_stopped(Duration::from_secs(1)));
}

#[test]
fn requires_token_limits_body_and_expires() {
    let _guard = ENV_LOCK.lock().unwrap();
    let wizard = start_config_wizard(ConfigWizardOptions {
        ttl: Duration::from_millis(150),
        max_body_bytes: 24,
        ..ConfigWizardOptions::default()
    })
    .unwrap();

    let forbidden = send_request(
        wizard.url(),
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(forbidden.starts_with("HTTP/1.1 403"));

    let wrong_token = send_request(
        wizard.url(),
        "GET /?token=wrong-token HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(wrong_token.starts_with("HTTP/1.1 403"));

    let target = target_with_path(wizard.url(), "/save");
    let oversized_body = "semantic_scholar.api_key=secret-value";
    let oversized = send_request(
        wizard.url(),
        &format!(
            "POST {target} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{oversized_body}",
            oversized_body.len()
        ),
    );
    assert!(oversized.starts_with("HTTP/1.1 413"));
    assert!(!oversized.contains("secret-value"));

    assert!(wizard.wait_until_stopped(Duration::from_secs(1)));
    assert!(!wizard.is_completed());
}

#[test]
fn rejects_relative_config_home_before_starting_the_wizard() {
    let _guard = ENV_LOCK.lock().unwrap();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    std::env::set_var("QIONGLI_CONFIG_HOME", "relative/config");

    let result = start_config_wizard(ConfigWizardOptions::default());

    assert!(matches!(
        result,
        Err(WizardError::Config(ConfigError::InvalidConfigHome))
    ));
    restore_env("QIONGLI_CONFIG_HOME", previous_home);
}

#[test]
fn saves_once_without_echoing_secret_and_stops_after_redirect() {
    let _guard = ENV_LOCK.lock().unwrap();
    let config_home = temp_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    std::env::set_var("QIONGLI_CONFIG_HOME", &config_home);

    let wizard = start_config_wizard(ConfigWizardOptions {
        provider: Some("semantic-scholar".to_string()),
        ttl: Duration::from_secs(3),
        ..ConfigWizardOptions::default()
    })
    .unwrap();

    let form_target = target_with_path(wizard.url(), "/");
    let form = send_request(
        wizard.url(),
        &format!("GET {form_target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
    );
    assert!(form.starts_with("HTTP/1.1 200"));
    assert!(form.contains("Semantic Scholar API key"));
    assert!(!form.contains("OpenAlex API key"));

    let secret = "wizard-secret-value";
    let body = format!("semantic_scholar.api_key={secret}");
    let save_target = target_with_path(wizard.url(), "/save");
    let saved = send_request(
        wizard.url(),
        &format!(
            "POST {save_target} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(saved.starts_with("HTTP/1.1 303"));
    assert!(!saved.contains(secret));
    assert!(wizard.is_completed());

    let repeated = send_request(
        wizard.url(),
        &format!(
            "POST {save_target} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(repeated.starts_with("HTTP/1.1 410"));
    assert!(!repeated.contains(secret));

    let saved_target = target_with_path(wizard.url(), "/saved");
    let saved_page = send_request(
        wizard.url(),
        &format!("GET {saved_target} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
    );
    assert!(saved_page.starts_with("HTTP/1.1 200"));
    assert!(saved_page.contains("Configuration saved"));
    assert!(!saved_page.contains(secret));
    assert!(wizard.wait_until_stopped(Duration::from_secs(1)));

    let config = fs::read_to_string(config_home.join("providers.json")).unwrap();
    assert!(config.contains(secret));
    assert_eq!(wizard.config_path(), &config_home.join("providers.json"));

    restore_env("QIONGLI_CONFIG_HOME", previous_home);
}

#[test]
fn malformed_config_remains_byte_identical_after_failed_submission() {
    let _guard = ENV_LOCK.lock().unwrap();
    let config_home = temp_dir();
    let previous_home = std::env::var_os("QIONGLI_CONFIG_HOME");
    std::env::set_var("QIONGLI_CONFIG_HOME", &config_home);
    let config_path = config_home.join("providers.json");
    let malformed_secret = "malformed-rust-wizard-secret-canary";
    let submitted_secret = "submitted-rust-wizard-secret-canary";
    let original = format!("{{not-json {malformed_secret}").into_bytes();
    fs::write(&config_path, &original).unwrap();

    let wizard = start_config_wizard(ConfigWizardOptions {
        ttl: Duration::from_secs(3),
        ..ConfigWizardOptions::default()
    })
    .unwrap();
    let body = format!("openalex.api_key={submitted_secret}");
    let save_target = target_with_path(wizard.url(), "/save");
    let response = send_request(
        wizard.url(),
        &format!(
            "POST {save_target} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );

    assert!(response.starts_with("HTTP/1.1 500"), "{response}");
    assert!(response.contains("Unable to save configuration."));
    assert!(!response.contains(malformed_secret));
    assert!(!response.contains(submitted_secret));
    assert!(!response.contains(config_home.to_string_lossy().as_ref()));
    assert_eq!(fs::read(&config_path).unwrap(), original);
    assert!(!wizard.is_completed());

    wizard.stop();
    assert!(wizard.wait_until_stopped(Duration::from_secs(1)));
    restore_env("QIONGLI_CONFIG_HOME", previous_home);
    fs::remove_dir_all(config_home).unwrap();
}

fn send_request(base_url: &str, request: &str) -> String {
    let parsed = Url::parse(base_url).unwrap();
    let mut stream =
        TcpStream::connect((parsed.host_str().unwrap(), parsed.port().unwrap())).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn token(url: &str) -> String {
    Url::parse(url)
        .unwrap()
        .query_pairs()
        .find_map(|(key, value)| (key == "token").then(|| value.into_owned()))
        .unwrap()
}

fn target_with_path(url: &str, path: &str) -> String {
    format!("{path}?token={}", token(url))
}

fn temp_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "qiongli-config-wizard-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn restore_env(name: &str, value: Option<std::ffi::OsString>) {
    if let Some(value) = value {
        std::env::set_var(name, value);
    } else {
        std::env::remove_var(name);
    }
}
