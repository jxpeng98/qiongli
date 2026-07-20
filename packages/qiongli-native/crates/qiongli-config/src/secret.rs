use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

use zeroize::Zeroizing;

pub const MAX_SECRET_VALUE_BYTES: usize = 16 * 1024;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SecretRef(String);

impl SecretRef {
    pub fn parse(value: &str) -> Result<Self, SecretRefError> {
        let identifier = value.strip_prefix("qsr1_").ok_or(SecretRefError)?;
        if identifier.len() != 32
            || !identifier
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(SecretRefError);
        }
        Ok(Self(value.to_owned()))
    }

    #[cfg(any(unix, windows, test))]
    pub(crate) fn raw(&self) -> &str {
        &self.0
    }

    /// Returns the opaque identifier used only at a trusted secret-store boundary.
    ///
    /// Callers must not display, log, or serialize this value outside the versioned
    /// configuration document.
    #[must_use]
    pub fn storage_key(&self) -> &str {
        &self.0
    }
}

impl Debug for SecretRef {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted-secret-ref>")
    }
}

impl Display for SecretRef {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted-secret-ref>")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretRefError;

impl Display for SecretRefError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid-secret-ref")
    }
}

impl Error for SecretRefError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretStoreStatus {
    Available,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecretStoreError {
    Unavailable,
    NotFound,
    PersistenceFailed,
}

impl SecretStoreError {
    #[must_use]
    pub const fn remediation_code(self) -> &'static str {
        match self {
            Self::Unavailable => "secure-store-unavailable",
            Self::NotFound => "secure-secret-not-found",
            Self::PersistenceFailed => "secure-store-write-failed",
        }
    }
}

impl Display for SecretStoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.remediation_code())
    }
}

impl Error for SecretStoreError {}

pub struct SecretValue {
    bytes: Zeroizing<Vec<u8>>,
}

impl SecretValue {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Result<Self, SecretValueError> {
        let bytes = bytes.into();
        if bytes.is_empty() || bytes.len() > MAX_SECRET_VALUE_BYTES {
            return Err(SecretValueError);
        }
        Ok(Self {
            bytes: Zeroizing::new(bytes),
        })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl Debug for SecretValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted-secret-value>")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecretValueError;

impl Display for SecretValueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid-secret-value")
    }
}

impl Error for SecretValueError {}

pub trait SecretStore: Send + Sync {
    fn status(&self) -> SecretStoreStatus;

    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError>;

    fn store(&self, _secret_ref: &SecretRef, _value: &SecretValue) -> Result<(), SecretStoreError> {
        Err(SecretStoreError::Unavailable)
    }

    fn remove(&self, _secret_ref: &SecretRef) -> Result<(), SecretStoreError> {
        Err(SecretStoreError::Unavailable)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UnavailableSecretStore;

impl SecretStore for UnavailableSecretStore {
    fn status(&self) -> SecretStoreStatus {
        SecretStoreStatus::Unavailable
    }

    fn resolve(&self, _secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
        Err(SecretStoreError::Unavailable)
    }
}
