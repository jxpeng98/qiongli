use std::sync::Arc;

use qiongli_config::SecretStore;
#[cfg(not(target_os = "macos"))]
use qiongli_config::UnavailableSecretStore;
#[cfg(target_os = "macos")]
use qiongli_config::{SecretRef, SecretStoreError, SecretStoreStatus, SecretValue};

#[cfg(target_os = "macos")]
const KEYCHAIN_SERVICE: &str = "io.github.jxpeng98.qiongli.providers.v2";
#[cfg(target_os = "macos")]
const ERR_SEC_ITEM_NOT_FOUND: i32 = -25_300;

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, Default)]
struct MacOsKeychainSecretStore;

#[cfg(target_os = "macos")]
impl SecretStore for MacOsKeychainSecretStore {
    fn status(&self) -> SecretStoreStatus {
        SecretStoreStatus::Available
    }

    fn resolve(&self, secret_ref: &SecretRef) -> Result<SecretValue, SecretStoreError> {
        let bytes = security_framework::passwords::get_generic_password(
            KEYCHAIN_SERVICE,
            secret_ref.storage_key(),
        )
        .map_err(map_keychain_read_error)?;
        SecretValue::new(bytes).map_err(|_| SecretStoreError::PersistenceFailed)
    }

    fn store(&self, secret_ref: &SecretRef, value: &SecretValue) -> Result<(), SecretStoreError> {
        security_framework::passwords::set_generic_password(
            KEYCHAIN_SERVICE,
            secret_ref.storage_key(),
            value.as_bytes(),
        )
        .map_err(|_| SecretStoreError::PersistenceFailed)
    }

    fn remove(&self, secret_ref: &SecretRef) -> Result<(), SecretStoreError> {
        security_framework::passwords::delete_generic_password(
            KEYCHAIN_SERVICE,
            secret_ref.storage_key(),
        )
        .map_err(map_keychain_read_error)
    }
}

#[cfg(target_os = "macos")]
fn map_keychain_read_error(error: security_framework::base::Error) -> SecretStoreError {
    if error.code() == ERR_SEC_ITEM_NOT_FOUND {
        SecretStoreError::NotFound
    } else {
        SecretStoreError::Unavailable
    }
}

#[doc(hidden)]
pub fn native_secret_store() -> Arc<dyn SecretStore> {
    #[cfg(target_os = "macos")]
    {
        Arc::new(MacOsKeychainSecretStore)
    }
    #[cfg(not(target_os = "macos"))]
    {
        Arc::new(UnavailableSecretStore)
    }
}
