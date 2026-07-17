use qiongli_config::{GlobalSettings, SecretRef, UnavailableSecretStore};
use qiongli_runtime::providers::search::{SearchMode, SearchRequest};
use qiongli_runtime::providers::{ProviderAccess, ProviderAvailability, ProviderId};

#[test]
fn native_settings_compose_with_redacted_provider_runtime_status() {
    let secret_ref = "qsr1_0123456789abcdef0123456789abcdef";
    let mut settings = GlobalSettings::default();
    settings.providers.openalex.enabled = true;
    settings.providers.openalex.api_key_ref = Some(SecretRef::parse(secret_ref).unwrap());

    let access = ProviderAccess::from_global_settings(&settings, &UnavailableSecretStore);
    let serialized = serde_json::to_string(&access.status()).unwrap();

    assert_eq!(
        access.availability(ProviderId::OpenAlex),
        ProviderAvailability::SecretStoreUnavailable
    );
    assert!(access.is_active(ProviderId::Arxiv));
    assert!(!serialized.contains(secret_ref));
}

#[test]
fn canonical_app_can_construct_a_bounded_search_request() {
    let providers = vec!["openalex".to_owned(), "arxiv".to_owned()];
    let request = SearchRequest::from_raw(
        "platform governance",
        Some("review"),
        Some(&providers),
        None,
        Some(40),
        Some(75),
    )
    .unwrap();

    assert_eq!(request.mode(), SearchMode::Review);
    assert_eq!(request.per_provider_limit(), 40);
    assert_eq!(request.total_limit(), 75);
}
