use std::io::{BufRead, Write};

use qiongli_config::UnavailableSecretStore;
use qiongli_content::EmbeddedContent;
use qiongli_runtime::mcp::LiteMcpServer;
use qiongli_runtime::providers::ProviderAccess;
use qiongli_runtime::{LiteToolRegistry, RuntimeError};

use crate::command::{CommandEnvironment, config_store};

pub fn serve_lite_mcp<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<(), RuntimeError> {
    let registry = LiteToolRegistry::from_embedded_content(content)?;
    let server = match config_store(environment).and_then(|store| store.load()) {
        Ok(loaded) => {
            let access =
                ProviderAccess::from_global_settings(&loaded.settings, &UnavailableSecretStore);
            LiteMcpServer::production("qiongli", env!("CARGO_PKG_VERSION"), registry, access)
        }
        Err(_) => LiteMcpServer::config_unavailable("qiongli", env!("CARGO_PKG_VERSION"), registry),
    };
    server.serve(reader, writer)
}
