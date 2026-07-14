pub use qiongli_runtime::zotero::companion::*;

pub fn probe_zotero_from_env() -> Result<ZoteroStatus, CompanionError> {
    let enabled = read_env_boolean("QIONGLI_ZOTERO_LOCAL_ENABLED", true);
    if !enabled {
        return Ok(ZoteroStatus::disabled());
    }
    let connector_url = std::env::var("QIONGLI_ZOTERO_CONNECTOR_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_CONNECTOR_URL.to_owned());
    Ok(CompanionClient::new(&connector_url)?.probe(true))
}

fn read_env_boolean(name: &str, fallback: bool) -> bool {
    let Ok(value) = std::env::var(name) else {
        return fallback;
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => fallback,
    }
}
