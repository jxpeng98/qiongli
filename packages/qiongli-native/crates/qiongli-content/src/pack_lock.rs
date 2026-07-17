use std::error::Error;
use std::fmt::{self, Display, Formatter};

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::manifest::{CompatibleProduct, JCS_MAX_SAFE_INTEGER};
use crate::writer::{BuiltResourcePack, ResourcePackBuildMetadata};

pub const RESOURCE_PACK_LOCK_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePackLockV1 {
    pub lock_version: u32,
    pub pack_id: String,
    pub content_version: String,
    pub source_commit: String,
    pub compatible_product: CompatibleProduct,
    pub entry_count: u64,
    pub content_root_sha256: String,
    pub pack_sha256: String,
}

impl ResourcePackLockV1 {
    pub fn from_json(input: &str) -> Result<Self, ResourcePackLockError> {
        let lock =
            serde_json::from_str::<Self>(input).map_err(ResourcePackLockError::InvalidJson)?;
        lock.validate()?;
        Ok(lock)
    }

    #[must_use]
    pub fn from_built(pack: &BuiltResourcePack) -> Self {
        let manifest = pack.manifest();
        Self {
            lock_version: RESOURCE_PACK_LOCK_VERSION,
            pack_id: manifest.pack_id.clone(),
            content_version: manifest.content_version.clone(),
            source_commit: manifest.source_commit.clone(),
            compatible_product: manifest.compatible_product.clone(),
            entry_count: manifest.entries.len() as u64,
            content_root_sha256: manifest.content_root_sha256.clone(),
            pack_sha256: pack.pack_sha256().to_string(),
        }
    }

    pub fn to_canonical_json(&self) -> Result<Vec<u8>, ResourcePackLockError> {
        self.validate()?;
        serde_json_canonicalizer::to_vec(self).map_err(ResourcePackLockError::CanonicalJson)
    }

    pub fn metadata(&self) -> Result<ResourcePackBuildMetadata, ResourcePackLockError> {
        self.validate()?;
        Ok(ResourcePackBuildMetadata {
            pack_id: self.pack_id.clone(),
            content_version: self.content_version.clone(),
            source_commit: self.source_commit.clone(),
            compatible_product: self.compatible_product.clone(),
        })
    }

    pub fn verify(&self, pack: &BuiltResourcePack) -> Result<(), ResourcePackLockError> {
        self.validate()?;
        let manifest = pack.manifest();
        verify_text("pack_id", &self.pack_id, &manifest.pack_id)?;
        verify_text(
            "content_version",
            &self.content_version,
            &manifest.content_version,
        )?;
        verify_text(
            "source_commit",
            &self.source_commit,
            &manifest.source_commit,
        )?;
        if self.compatible_product != manifest.compatible_product {
            return Err(ResourcePackLockError::MetadataMismatch {
                field: "compatible_product",
            });
        }

        let actual_entry_count = u64::try_from(manifest.entries.len())
            .map_err(|_| ResourcePackLockError::EntryCountOverflow)?;
        if self.entry_count != actual_entry_count {
            return Err(ResourcePackLockError::EntryCountMismatch {
                expected: self.entry_count,
                actual: actual_entry_count,
            });
        }
        if self.content_root_sha256 != manifest.content_root_sha256 {
            return Err(ResourcePackLockError::ContentRootMismatch {
                expected: self.content_root_sha256.clone(),
                actual: manifest.content_root_sha256.clone(),
            });
        }
        if self.pack_sha256 != pack.pack_sha256() {
            return Err(ResourcePackLockError::PackDigestMismatch {
                expected: self.pack_sha256.clone(),
                actual: pack.pack_sha256().to_string(),
            });
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), ResourcePackLockError> {
        if self.lock_version != RESOURCE_PACK_LOCK_VERSION {
            return Err(ResourcePackLockError::UnsupportedVersion {
                found: self.lock_version,
            });
        }
        if self.pack_id.is_empty()
            || !self
                .pack_id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(ResourcePackLockError::InvalidField {
                field: "pack_id",
                reason: "must use lowercase ASCII identifiers",
            });
        }
        Version::parse(&self.content_version).map_err(|_| ResourcePackLockError::InvalidField {
            field: "content_version",
            reason: "must be SemVer",
        })?;
        if !is_lower_hex(&self.source_commit, 40) {
            return Err(ResourcePackLockError::InvalidField {
                field: "source_commit",
                reason: "must be 40 lowercase hexadecimal characters",
            });
        }
        let minimum = Version::parse(&self.compatible_product.minimum).map_err(|_| {
            ResourcePackLockError::InvalidField {
                field: "compatible_product.minimum",
                reason: "must be SemVer",
            }
        })?;
        let maximum = Version::parse(&self.compatible_product.maximum_exclusive).map_err(|_| {
            ResourcePackLockError::InvalidField {
                field: "compatible_product.maximum_exclusive",
                reason: "must be SemVer",
            }
        })?;
        if minimum >= maximum {
            return Err(ResourcePackLockError::InvalidField {
                field: "compatible_product",
                reason: "must describe a non-empty range",
            });
        }
        if self.entry_count == 0 || self.entry_count > JCS_MAX_SAFE_INTEGER {
            return Err(ResourcePackLockError::InvalidField {
                field: "entry_count",
                reason: "must be within the positive JCS safe-integer range",
            });
        }
        for (field, digest) in [
            ("content_root_sha256", self.content_root_sha256.as_str()),
            ("pack_sha256", self.pack_sha256.as_str()),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err(ResourcePackLockError::InvalidField {
                    field,
                    reason: "must be 64 lowercase hexadecimal characters",
                });
            }
        }
        Ok(())
    }
}

fn verify_text(
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), ResourcePackLockError> {
    if expected == actual {
        Ok(())
    } else {
        Err(ResourcePackLockError::MetadataMismatch { field })
    }
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug)]
pub enum ResourcePackLockError {
    InvalidJson(serde_json::Error),
    CanonicalJson(serde_json::Error),
    UnsupportedVersion {
        found: u32,
    },
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
    MetadataMismatch {
        field: &'static str,
    },
    EntryCountOverflow,
    EntryCountMismatch {
        expected: u64,
        actual: u64,
    },
    ContentRootMismatch {
        expected: String,
        actual: String,
    },
    PackDigestMismatch {
        expected: String,
        actual: String,
    },
}

impl Display for ResourcePackLockError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid resource-pack lock: {error}"),
            Self::CanonicalJson(error) => {
                write!(
                    formatter,
                    "resource-pack lock canonicalization failed: {error}"
                )
            }
            Self::UnsupportedVersion { found } => {
                write!(formatter, "unsupported resource-pack lock version: {found}")
            }
            Self::InvalidField { field, reason } => {
                write!(formatter, "resource-pack lock field {field} {reason}")
            }
            Self::MetadataMismatch { field } => {
                write!(formatter, "resource-pack lock metadata drifted at {field}")
            }
            Self::EntryCountOverflow => {
                formatter.write_str("resource-pack entry count exceeds the supported range")
            }
            Self::EntryCountMismatch { expected, actual } => write!(
                formatter,
                "resource-pack entry count drifted: expected {expected}, found {actual}"
            ),
            Self::ContentRootMismatch { expected, actual } => write!(
                formatter,
                "resource-pack content root drifted: expected {expected}, found {actual}"
            ),
            Self::PackDigestMismatch { expected, actual } => write!(
                formatter,
                "resource-pack digest drifted: expected {expected}, found {actual}"
            ),
        }
    }
}

impl Error for ResourcePackLockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidJson(error) | Self::CanonicalJson(error) => Some(error),
            _ => None,
        }
    }
}
