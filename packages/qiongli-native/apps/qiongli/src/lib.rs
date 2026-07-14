mod command;
mod desktop;
mod mcp;

pub use command::{
    CliOutput, CommandEnvironment, ProductAction, failed_embedded_content_output, prepare_action,
    run_cli,
};
pub use desktop::{DesktopLaunchError, run_desktop};
pub use mcp::serve_lite_mcp;
use qiongli_content::{EmbeddedContent, ResourcePackLoaderError};

pub const EMBEDDED_PACK_SHA256: &str =
    include_str!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack.sha256"));

static EMBEDDED_PACK_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack"));

pub fn embedded_content() -> Result<EmbeddedContent, ResourcePackLoaderError> {
    EmbeddedContent::load(EMBEDDED_PACK_BYTES, EMBEDDED_PACK_SHA256)
}
