mod candidate_cli;
mod command;
mod desktop;
mod mcp;
mod native_cli;

pub use command::{
    CliOutput, CommandEnvironment, ProductAction, failed_embedded_content_output, prepare_action,
    run_cli,
};
pub use desktop::{
    DesktopActivationSession, DesktopCandidateSession, DesktopLaunchError, run_desktop,
    run_desktop_with_activation_sessions, run_desktop_with_candidate_sessions,
};
pub use mcp::serve_lite_mcp;
use qiongli_content::{EmbeddedContent, ResourcePackLoaderError};
use qiongli_platform::{NativeReleaseAuthority, NativeReleaseAuthorityError};

pub const EMBEDDED_PACK_SHA256: &str =
    include_str!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack.sha256"));

static EMBEDDED_PACK_BYTES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/qiongli-core.qlpack"));

static EMBEDDED_RELEASE_AUTHORITY_BYTES: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/qiongli-native-release-authority.json"
));

const EMBEDDED_SOURCE_COMMIT: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/qiongli-native-source-commit.txt"
));

pub fn embedded_content() -> Result<EmbeddedContent, ResourcePackLoaderError> {
    EmbeddedContent::load(EMBEDDED_PACK_BYTES, EMBEDDED_PACK_SHA256)
}

pub fn embedded_release_authority()
-> Result<Option<NativeReleaseAuthority>, NativeReleaseAuthorityError> {
    if EMBEDDED_RELEASE_AUTHORITY_BYTES.is_empty() {
        Ok(None)
    } else {
        let authority = NativeReleaseAuthority::from_json(EMBEDDED_RELEASE_AUTHORITY_BYTES)?;
        authority.validate_product_version(env!("CARGO_PKG_VERSION"))?;
        Ok(Some(authority))
    }
}

#[must_use]
pub const fn embedded_source_commit() -> Option<&'static str> {
    if EMBEDDED_SOURCE_COMMIT.is_empty() {
        None
    } else {
        Some(EMBEDDED_SOURCE_COMMIT)
    }
}
