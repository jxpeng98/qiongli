use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum CompanionError {
    #[error("Zotero connector URL must use localhost or 127.0.0.1")]
    NonLoopback,
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

#[derive(Debug, Clone)]
pub struct CompanionClient {
    base_url: Url,
}

impl CompanionClient {
    pub fn new(raw: &str) -> Result<Self, CompanionError> {
        let base_url = Url::parse(raw)?;
        let host = base_url.host_str().unwrap_or("");
        if host != "127.0.0.1" && host != "localhost" {
            return Err(CompanionError::NonLoopback);
        }
        Ok(Self { base_url })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}
