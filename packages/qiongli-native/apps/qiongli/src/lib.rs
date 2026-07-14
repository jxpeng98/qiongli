mod command;

pub use command::{CliOutput, CommandEnvironment, failed_embedded_content_output, run_cli};
use qiongli_content::{EmbeddedContent, ResourcePackLoaderError};

pub const EMBEDDED_PACK_SHA256: &str =
    include_str!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack.sha256"));

static EMBEDDED_PACK_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack"));

pub fn embedded_content() -> Result<EmbeddedContent, ResourcePackLoaderError> {
    EmbeddedContent::load(EMBEDDED_PACK_BYTES, EMBEDDED_PACK_SHA256)
}
