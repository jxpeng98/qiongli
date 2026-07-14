use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use qiongli_content::ProfileId;
use serde::Serialize;

use crate::SecretRef;

#[derive(Clone, Eq, PartialEq)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn parse(value: &str) -> Result<Self, EmailAddressError> {
        let value = value.trim();
        let characters = value.chars().count();
        if !(1..=320).contains(&characters) || value.chars().any(char::is_control) {
            return Err(EmailAddressError);
        }
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for EmailAddress {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted-email>")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailAddressError;

impl Display for EmailAddressError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid-email-setting")
    }
}

impl Error for EmailAddressError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderReadiness {
    Disabled,
    Ready,
    NeedsSecret,
    NeedsPublicSetting,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OpenAlexSettings {
    pub enabled: bool,
    pub email: Option<EmailAddress>,
    pub api_key_ref: Option<SecretRef>,
}

impl OpenAlexSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.api_key_ref.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsSecret
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticScholarSettings {
    pub enabled: bool,
    pub api_key_ref: Option<SecretRef>,
}

impl SemanticScholarSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.api_key_ref.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsSecret
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CrossrefSettings {
    pub enabled: bool,
    pub email: Option<EmailAddress>,
}

impl CrossrefSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.email.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsPublicSetting
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PubmedSettings {
    pub enabled: bool,
    pub api_key_ref: Option<SecretRef>,
}

impl PubmedSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if !self.enabled {
            ProviderReadiness::Disabled
        } else if self.api_key_ref.is_some() {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::NeedsSecret
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArxivSettings {
    pub enabled: bool,
}

impl ArxivSettings {
    #[must_use]
    pub const fn readiness(&self) -> ProviderReadiness {
        if self.enabled {
            ProviderReadiness::Ready
        } else {
            ProviderReadiness::Disabled
        }
    }
}

impl Default for ArxivSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderSettings {
    pub openalex: OpenAlexSettings,
    pub semantic_scholar: SemanticScholarSettings,
    pub crossref: CrossrefSettings,
    pub pubmed: PubmedSettings,
    pub arxiv: ArxivSettings,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSettings {
    pub default_profile: ProfileId,
    pub providers: ProviderSettings,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            default_profile: ProfileId::MarketplaceLite,
            providers: ProviderSettings::default(),
        }
    }
}
