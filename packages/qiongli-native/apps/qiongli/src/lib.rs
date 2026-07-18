mod application;
mod candidate_cli;
mod command;
mod credential_store;
mod desktop;
mod desktop_contract;
mod macos_update_stage;
mod managed_content;
mod mcp;
mod native_cli;
mod native_update_replace;
mod update_cli;
mod update_reconcile;

pub use application::{
    DesktopApplicationAssetError, DesktopApplicationError, desktop_application_icon_png,
    desktop_application_metadata, run_desktop_application,
};
pub use command::{
    CliOutput, CommandEnvironment, ProductAction, failed_embedded_content_output, prepare_action,
    run_cli,
};
#[doc(hidden)]
pub use credential_store::native_secret_store;
pub use desktop::{
    DesktopActivationSession, DesktopCandidateSession, DesktopLaunchError, run_desktop,
    run_desktop_with_activation_sessions, run_desktop_with_candidate_sessions,
};
pub use desktop_contract::{
    DESKTOP_APPLICATION_IDENTIFIER, DESKTOP_CONTENT_ERROR_CODE, DESKTOP_PRODUCT_LICENSE,
    DESKTOP_PRODUCT_NAME, DESKTOP_PRODUCT_VERSION, DESKTOP_RUNTIME_ERROR_CODE,
    DESKTOP_STARTUP_ERROR_CODE, DESKTOP_WINDOW_TITLE,
};
pub use mcp::serve_lite_mcp;
pub use native_update_replace::run_native_update_helper;
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

const EMBEDDED_MACOS_TEAM_ID: &str =
    include_str!(concat!(env!("OUT_DIR"), "/qiongli-macos-team-id.txt"));

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

#[must_use]
pub const fn embedded_macos_team_id() -> Option<&'static str> {
    if EMBEDDED_MACOS_TEAM_ID.is_empty() {
        None
    } else {
        Some(EMBEDDED_MACOS_TEAM_ID)
    }
}
