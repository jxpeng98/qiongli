use std::ops::Deref;

use reqwest::blocking::Client;

use crate::config::provider_config::ResolvedProviderConfig;

pub use qiongli_runtime::providers::{CancellationToken, ProviderEndpoints, ProviderRuntimeError};

#[derive(Clone)]
pub struct ProviderRuntime {
    inner: qiongli_runtime::providers::ProviderRuntime,
    config: ResolvedProviderConfig,
}

impl ProviderRuntime {
    pub fn production(config: ResolvedProviderConfig) -> Result<Self, ProviderRuntimeError> {
        let inner =
            qiongli_runtime::providers::ProviderRuntime::production(config.runtime_access())?;
        Ok(Self { inner, config })
    }

    #[doc(hidden)]
    pub fn with_client(
        client: Client,
        endpoints: ProviderEndpoints,
        config: ResolvedProviderConfig,
    ) -> Self {
        let inner = qiongli_runtime::providers::ProviderRuntime::with_client(
            client,
            endpoints,
            config.runtime_access(),
        );
        Self { inner, config }
    }

    #[doc(hidden)]
    pub fn with_endpoints(
        endpoints: ProviderEndpoints,
        config: ResolvedProviderConfig,
    ) -> Result<Self, ProviderRuntimeError> {
        let inner = qiongli_runtime::providers::ProviderRuntime::with_endpoints(
            endpoints,
            config.runtime_access(),
        )?;
        Ok(Self { inner, config })
    }

    pub fn config(&self) -> &ResolvedProviderConfig {
        &self.config
    }
}

impl Deref for ProviderRuntime {
    type Target = qiongli_runtime::providers::ProviderRuntime;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
