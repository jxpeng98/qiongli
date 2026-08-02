use qiongli_config::{
    EmailAddress, GlobalSettings, MAX_SECRET_VALUE_BYTES, ProviderReadiness, SecretRef,
    SecretStore, SecretStoreError, SecretStoreStatus, SecretValue, UnavailableSecretStore,
};
use qiongli_content::ProfileId;

const SECRET_REF: &str = "qsr1_0123456789abcdef0123456789abcdef";

#[test]
fn secret_reference_is_strict_and_redacted() {
    let reference = SecretRef::parse(SECRET_REF).unwrap();
    assert_eq!(format!("{reference:?}"), "<redacted-secret-ref>");
    assert_eq!(format!("{reference}"), "<redacted-secret-ref>");
    for invalid in [
        "secret-key",
        "qsr1_0123456789ABCDEF0123456789ABCDEF",
        "qsr1_0123",
        " qsr1_0123456789abcdef0123456789abcdef",
        "qsr1_0123456789abcdef0123456789abcdef ",
    ] {
        assert!(SecretRef::parse(invalid).is_err(), "accepted {invalid:?}");
    }
}

#[test]
fn unavailable_secret_store_has_no_fallback() {
    let store = UnavailableSecretStore;
    let reference = SecretRef::parse(SECRET_REF).unwrap();
    assert_eq!(store.status(), SecretStoreStatus::Unavailable);
    assert!(matches!(
        store.resolve(&reference),
        Err(SecretStoreError::Unavailable)
    ));
    let value = SecretValue::new(b"secret-canary".to_vec()).unwrap();
    assert_eq!(
        store.store(&reference, &value),
        Err(SecretStoreError::Unavailable)
    );
    assert_eq!(store.remove(&reference), Err(SecretStoreError::Unavailable));
    assert_eq!(
        SecretStoreError::Unavailable.remediation_code(),
        "secure-store-unavailable"
    );
}

#[test]
fn secret_values_are_constructible_bounded_and_redacted() {
    let value = SecretValue::new(b"secret-canary".to_vec()).unwrap();
    assert_eq!(value.as_bytes(), b"secret-canary");
    assert_eq!(format!("{value:?}"), "<redacted-secret-value>");
    assert!(SecretValue::new(Vec::new()).is_err());
    assert!(SecretValue::new(vec![b'x'; MAX_SECRET_VALUE_BYTES + 1]).is_err());
}

#[test]
fn defaults_enable_only_arxiv_and_select_marketplace_lite() {
    let settings = GlobalSettings::default();
    assert_eq!(settings.default_profile, ProfileId::MarketplaceLite);
    assert!(!settings.providers.openalex.enabled);
    assert!(!settings.providers.semantic_scholar.enabled);
    assert!(!settings.providers.crossref.enabled);
    assert!(!settings.providers.pubmed.enabled);
    assert!(settings.providers.arxiv.enabled);
    assert_eq!(
        settings.providers.arxiv.readiness(),
        ProviderReadiness::Ready
    );
    assert!(!settings.agent_backends.openai.enabled);
    assert_eq!(
        settings.agent_backends.openai.readiness(),
        ProviderReadiness::Disabled
    );
}

#[test]
fn private_email_is_normalized_but_never_debugged() {
    let email = EmailAddress::parse("  researcher@example.org  ").unwrap();
    assert_eq!(email.as_str(), "researcher@example.org");
    assert_eq!(format!("{email:?}"), "<redacted-email>");
    assert!(EmailAddress::parse("").is_err());
    assert!(EmailAddress::parse("bad\nmail@example.org").is_err());
    assert!(EmailAddress::parse(&"a".repeat(321)).is_err());
}

#[test]
fn enabled_provider_readiness_is_typed() {
    let mut settings = GlobalSettings::default();
    settings.providers.openalex.enabled = true;
    assert_eq!(
        settings.providers.openalex.readiness(),
        ProviderReadiness::NeedsSecret
    );
    settings.providers.openalex.api_key_ref = Some(SecretRef::parse(SECRET_REF).unwrap());
    assert_eq!(
        settings.providers.openalex.readiness(),
        ProviderReadiness::Ready
    );

    settings.providers.semantic_scholar.enabled = true;
    assert_eq!(
        settings.providers.semantic_scholar.readiness(),
        ProviderReadiness::NeedsSecret
    );
    settings.providers.crossref.enabled = true;
    assert_eq!(
        settings.providers.crossref.readiness(),
        ProviderReadiness::NeedsPublicSetting
    );
    settings.providers.pubmed.enabled = true;
    assert_eq!(
        settings.providers.pubmed.readiness(),
        ProviderReadiness::NeedsSecret
    );
}

#[test]
fn enabled_agent_backend_requires_an_opaque_secret_reference() {
    let mut settings = GlobalSettings::default();
    settings.agent_backends.openai.enabled = true;
    assert_eq!(
        settings.agent_backends.openai.readiness(),
        ProviderReadiness::NeedsSecret
    );
    settings.agent_backends.openai.api_key_ref = Some(SecretRef::parse(SECRET_REF).unwrap());
    assert_eq!(
        settings.agent_backends.openai.readiness(),
        ProviderReadiness::Ready
    );
}

#[test]
fn settings_debug_redacts_every_private_value() {
    let mut settings = GlobalSettings::default();
    settings.providers.openalex.email = Some(EmailAddress::parse("canary@example.org").unwrap());
    settings.providers.openalex.api_key_ref = Some(SecretRef::parse(SECRET_REF).unwrap());
    settings.agent_backends.openai.api_key_ref = Some(SecretRef::parse(SECRET_REF).unwrap());
    let debug = format!("{settings:?}");
    assert!(!debug.contains("canary@example.org"));
    assert!(!debug.contains(SECRET_REF));
    assert!(debug.contains("<redacted-email>"));
    assert!(debug.contains("<redacted-secret-ref>"));
}
