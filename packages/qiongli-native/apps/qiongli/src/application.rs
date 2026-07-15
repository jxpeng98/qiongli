use qiongli_content::EmbeddedContent;
use qiongli_ui::DesktopApplicationMetadata;

use crate::{
    CommandEnvironment, DESKTOP_APPLICATION_IDENTIFIER, DESKTOP_CONTENT_ERROR_CODE,
    DESKTOP_PRODUCT_LICENSE, DESKTOP_PRODUCT_NAME, DESKTOP_PRODUCT_VERSION,
    DESKTOP_STARTUP_ERROR_CODE, DESKTOP_WINDOW_TITLE, DesktopLaunchError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopApplicationError {
    EmbeddedContent,
    Window,
}

impl DesktopApplicationError {
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::EmbeddedContent => DESKTOP_CONTENT_ERROR_CODE,
            Self::Window => DESKTOP_STARTUP_ERROR_CODE,
        }
    }
}

#[must_use]
pub const fn desktop_application_metadata() -> DesktopApplicationMetadata {
    DesktopApplicationMetadata::new(
        DESKTOP_PRODUCT_NAME,
        DESKTOP_WINDOW_TITLE,
        DESKTOP_PRODUCT_VERSION,
        DESKTOP_APPLICATION_IDENTIFIER,
        DESKTOP_PRODUCT_LICENSE,
        DESKTOP_STARTUP_ERROR_CODE,
    )
}

pub fn run_desktop_application() -> Result<(), DesktopApplicationError> {
    run_desktop_application_with(
        CommandEnvironment::from_process(),
        crate::embedded_content(),
        crate::run_desktop,
    )
}

fn run_desktop_application_with<E>(
    environment: CommandEnvironment,
    content: Result<EmbeddedContent, E>,
    launch: impl FnOnce(CommandEnvironment, EmbeddedContent) -> Result<(), DesktopLaunchError>,
) -> Result<(), DesktopApplicationError> {
    let content = content.map_err(|_| DesktopApplicationError::EmbeddedContent)?;
    launch(environment, content).map_err(|_| DesktopApplicationError::Window)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_is_stable_and_packaging_ready() {
        let metadata = desktop_application_metadata();
        assert_eq!(metadata.product_name(), DESKTOP_PRODUCT_NAME);
        assert_eq!(metadata.window_title(), DESKTOP_WINDOW_TITLE);
        assert_eq!(metadata.version(), DESKTOP_PRODUCT_VERSION);
        assert_eq!(
            metadata.application_identifier(),
            DESKTOP_APPLICATION_IDENTIFIER
        );
        assert_eq!(metadata.license(), DESKTOP_PRODUCT_LICENSE);
        assert_eq!(metadata.startup_error_code(), DESKTOP_STARTUP_ERROR_CODE);
    }

    #[test]
    fn invalid_embedded_content_has_a_fixed_public_error() {
        let result = run_desktop_application_with(
            CommandEnvironment::default(),
            Err::<EmbeddedContent, _>(()),
            |_environment, _content| Ok(()),
        );
        assert_eq!(result, Err(DesktopApplicationError::EmbeddedContent));
        assert_eq!(
            DesktopApplicationError::EmbeddedContent.reason_code(),
            DESKTOP_CONTENT_ERROR_CODE
        );
    }

    #[test]
    fn renderer_failure_has_a_fixed_public_error() {
        let content = crate::embedded_content().expect("embedded content must load");
        let result = run_desktop_application_with(
            CommandEnvironment::default(),
            Ok::<EmbeddedContent, ()>(content),
            |_environment, _content| Err(DesktopLaunchError),
        );
        assert_eq!(result, Err(DesktopApplicationError::Window));
        assert_eq!(
            DesktopApplicationError::Window.reason_code(),
            DESKTOP_STARTUP_ERROR_CODE
        );
    }
}
