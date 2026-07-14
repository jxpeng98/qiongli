use std::ffi::OsStr;
use std::fmt::{self, Debug, Formatter};
use std::path::{Component, Path, PathBuf};

use crate::ConfigError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigRootSource {
    Default,
    Override,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ConfigRoot {
    compatibility_root: PathBuf,
    state_root: PathBuf,
    source: ConfigRootSource,
}

impl ConfigRoot {
    #[must_use]
    pub fn compatibility_root(&self) -> &Path {
        &self.compatibility_root
    }

    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    #[must_use]
    pub const fn source(&self) -> ConfigRootSource {
        self.source
    }

    #[must_use]
    pub const fn symbolic_state_root(&self) -> &'static str {
        match self.source {
            ConfigRootSource::Default => "<user-home>/.config/qiongli/v2",
            ConfigRootSource::Override => "<configured-root>/v2",
        }
    }
}

impl Debug for ConfigRoot {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigRoot")
            .field("source", &self.source)
            .field("state_root", &self.symbolic_state_root())
            .finish()
    }
}

pub fn resolve_config_root(
    configured: Option<&OsStr>,
    platform_home: &Path,
) -> Result<ConfigRoot, ConfigError> {
    validate_home(platform_home)?;
    let (compatibility_root, source) = match configured {
        None => (
            platform_home.join(".config").join("qiongli"),
            ConfigRootSource::Default,
        ),
        Some(value) => (
            resolve_configured_root(value, platform_home)?,
            ConfigRootSource::Override,
        ),
    };
    let state_root = compatibility_root.join("v2");
    Ok(ConfigRoot {
        compatibility_root,
        state_root,
        source,
    })
}

fn validate_home(platform_home: &Path) -> Result<(), ConfigError> {
    if !platform_home.is_absolute() || has_lexical_traversal(platform_home) {
        return Err(ConfigError::HomeUnavailable);
    }
    Ok(())
}

fn resolve_configured_root(value: &OsStr, platform_home: &Path) -> Result<PathBuf, ConfigError> {
    if value.is_empty() {
        return Err(ConfigError::InvalidConfigHome);
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return (!has_lexical_traversal(path))
            .then(|| path.to_path_buf())
            .ok_or(ConfigError::InvalidConfigHome);
    }
    if value == OsStr::new("~") {
        return Ok(platform_home.to_path_buf());
    }
    let configured = value.to_str().ok_or(ConfigError::InvalidConfigHome)?;
    let suffix = home_relative_suffix(configured).ok_or(ConfigError::InvalidConfigHome)?;
    if !is_portable_relative_suffix(suffix) {
        return Err(ConfigError::InvalidConfigHome);
    }
    let suffix = Path::new(suffix);
    if suffix.has_root()
        || matches!(suffix.components().next(), Some(Component::Prefix(_)))
        || has_lexical_traversal(suffix)
    {
        return Err(ConfigError::InvalidConfigHome);
    }
    Ok(platform_home.join(suffix))
}

#[cfg(windows)]
fn home_relative_suffix(configured: &str) -> Option<&str> {
    configured
        .strip_prefix("~/")
        .or_else(|| configured.strip_prefix(r"~\"))
}

#[cfg(not(windows))]
fn home_relative_suffix(configured: &str) -> Option<&str> {
    configured.strip_prefix("~/")
}

fn is_portable_relative_suffix(suffix: &str) -> bool {
    let bytes = suffix.as_bytes();
    !matches!(bytes.first(), Some(b'/' | b'\\'))
        && !matches!(bytes, [drive, b':', ..] if drive.is_ascii_alphabetic())
}

#[cfg(unix)]
fn has_lexical_traversal(path: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;

    path.as_os_str()
        .as_bytes()
        .split(|byte| *byte == b'/')
        .any(|component| component == b"." || component == b"..")
}

#[cfg(windows)]
fn has_lexical_traversal(path: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;

    path.as_os_str()
        .encode_wide()
        .collect::<Vec<_>>()
        .split(|unit| matches!(*unit, 47 | 92))
        .any(|component| component == [46] || component == [46, 46])
}

#[cfg(not(any(unix, windows)))]
fn has_lexical_traversal(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}
