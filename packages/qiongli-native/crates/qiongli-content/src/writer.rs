use std::error::Error;
use std::fmt::{self, Display, Formatter};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::collector::CollectedResource;
use crate::manifest::{
    CompatibleProduct, JCS_MAX_SAFE_INTEGER, ManifestError,
    RESOURCE_PACK_COMPILER_CONTRACT_VERSION, RESOURCE_PACK_FORMAT_VERSION, ResourceEntry,
    ResourcePackManifestV1, canonical_profile_projections,
};

pub const RESOURCE_PACK_MAGIC: [u8; 8] = *b"QLPACK\0\0";
pub const RESOURCE_PACK_HEADER_LEN: usize =
    RESOURCE_PACK_MAGIC.len() + size_of::<u32>() + size_of::<u64>();
pub const RESOURCE_PACK_CONTENT_ROOT_DOMAIN_V1: &[u8] = b"qiongli:resource-pack:content-root:v1\0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourcePackBuildMetadata {
    pub pack_id: String,
    pub content_version: String,
    pub source_commit: String,
    pub compatible_product: CompatibleProduct,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuiltResourcePack {
    core_bytes: Box<[u8]>,
    manifest_bytes: Box<[u8]>,
    manifest: ResourcePackManifestV1,
    pack_sha256: String,
}

impl BuiltResourcePack {
    #[must_use]
    pub fn core_bytes(&self) -> &[u8] {
        &self.core_bytes
    }

    #[must_use]
    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest_bytes
    }

    #[must_use]
    pub fn manifest(&self) -> &ResourcePackManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn pack_sha256(&self) -> &str {
        &self.pack_sha256
    }

    #[must_use]
    pub fn into_core_bytes(self) -> Box<[u8]> {
        self.core_bytes
    }
}

#[derive(Debug)]
pub enum ResourcePackWriterError {
    EmptyResources,
    DuplicatePath { path: String },
    PayloadSizeOverflow,
    JcsNumberOutOfRange,
    CanonicalJson(serde_json::Error),
    InvalidManifest(ManifestError),
}

impl Display for ResourcePackWriterError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyResources => formatter.write_str("resource-pack input must not be empty"),
            Self::DuplicatePath { path } => {
                write!(formatter, "duplicate resource-pack path: {path}")
            }
            Self::PayloadSizeOverflow => formatter.write_str("resource-pack payload size overflow"),
            Self::JcsNumberOutOfRange => formatter
                .write_str("resource-pack numeric fields exceed the JCS safe-integer range"),
            Self::CanonicalJson(error) => {
                write!(formatter, "resource-pack JCS serialization failed: {error}")
            }
            Self::InvalidManifest(error) => {
                write!(formatter, "invalid resource-pack manifest: {error}")
            }
        }
    }
}

impl Error for ResourcePackWriterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalJson(error) => Some(error),
            Self::InvalidManifest(error) => Some(error),
            Self::EmptyResources
            | Self::DuplicatePath { .. }
            | Self::PayloadSizeOverflow
            | Self::JcsNumberOutOfRange => None,
        }
    }
}

pub fn build_resource_pack(
    metadata: &ResourcePackBuildMetadata,
    resources: &[CollectedResource],
) -> Result<BuiltResourcePack, ResourcePackWriterError> {
    if resources.is_empty() {
        return Err(ResourcePackWriterError::EmptyResources);
    }

    let mut ordered_resources = resources.iter().collect::<Vec<_>>();
    ordered_resources.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    if let Some(duplicate) = ordered_resources
        .windows(2)
        .find(|pair| pair[0].path == pair[1].path)
    {
        return Err(ResourcePackWriterError::DuplicatePath {
            path: duplicate[0].path.clone(),
        });
    }

    let mut payload_offset = 0_u64;
    let mut entries = Vec::with_capacity(ordered_resources.len());
    for resource in &ordered_resources {
        let size_bytes = resource.size_bytes();
        if size_bytes > JCS_MAX_SAFE_INTEGER || payload_offset > JCS_MAX_SAFE_INTEGER {
            return Err(ResourcePackWriterError::JcsNumberOutOfRange);
        }
        entries.push(ResourceEntry {
            path: resource.path.clone(),
            resource_kind: resource.resource_kind,
            mode: resource.mode,
            size_bytes,
            payload_offset,
            sha256: sha256_hex(resource.bytes()),
        });
        payload_offset = payload_offset
            .checked_add(size_bytes)
            .ok_or(ResourcePackWriterError::PayloadSizeOverflow)?;
    }
    if payload_offset > JCS_MAX_SAFE_INTEGER {
        return Err(ResourcePackWriterError::JcsNumberOutOfRange);
    }

    let canonical_entries = canonical_json(&entries)?;
    let content_root_sha256 = content_root_sha256(&canonical_entries)?;
    let manifest = ResourcePackManifestV1 {
        format_version: RESOURCE_PACK_FORMAT_VERSION,
        compiler_contract_version: RESOURCE_PACK_COMPILER_CONTRACT_VERSION,
        pack_id: metadata.pack_id.clone(),
        content_version: metadata.content_version.clone(),
        source_commit: metadata.source_commit.clone(),
        compatible_product: metadata.compatible_product.clone(),
        profiles: canonical_profile_projections(),
        entries,
        content_root_sha256,
    };
    manifest
        .validate()
        .map_err(ResourcePackWriterError::InvalidManifest)?;

    let manifest_bytes = canonical_json(&manifest)?;
    let manifest_len = u64::try_from(manifest_bytes.len())
        .map_err(|_| ResourcePackWriterError::PayloadSizeOverflow)?;
    let payload_len = usize::try_from(payload_offset)
        .map_err(|_| ResourcePackWriterError::PayloadSizeOverflow)?;
    let core_capacity = RESOURCE_PACK_HEADER_LEN
        .checked_add(manifest_bytes.len())
        .and_then(|size| size.checked_add(payload_len))
        .ok_or(ResourcePackWriterError::PayloadSizeOverflow)?;

    let mut core_bytes = Vec::with_capacity(core_capacity);
    core_bytes.extend_from_slice(&RESOURCE_PACK_MAGIC);
    core_bytes.extend_from_slice(&RESOURCE_PACK_FORMAT_VERSION.to_le_bytes());
    core_bytes.extend_from_slice(&manifest_len.to_le_bytes());
    core_bytes.extend_from_slice(&manifest_bytes);
    for resource in ordered_resources {
        core_bytes.extend_from_slice(resource.bytes());
    }
    debug_assert_eq!(core_bytes.len(), core_capacity);

    let pack_sha256 = sha256_hex(&core_bytes);
    Ok(BuiltResourcePack {
        core_bytes: core_bytes.into_boxed_slice(),
        manifest_bytes: manifest_bytes.into_boxed_slice(),
        manifest,
        pack_sha256,
    })
}

pub(crate) fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ResourcePackWriterError> {
    serde_json_canonicalizer::to_vec(value).map_err(ResourcePackWriterError::CanonicalJson)
}

pub(crate) fn content_root_sha256(
    canonical_entries: &[u8],
) -> Result<String, ResourcePackWriterError> {
    let entries_len = u64::try_from(canonical_entries.len())
        .map_err(|_| ResourcePackWriterError::PayloadSizeOverflow)?;
    let mut hasher = Sha256::new();
    hasher.update(RESOURCE_PACK_CONTENT_ROOT_DOMAIN_V1);
    hasher.update(entries_len.to_le_bytes());
    hasher.update(canonical_entries);
    Ok(lower_hex(&hasher.finalize()))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    lower_hex(&Sha256::digest(bytes))
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{LogicalMode, ResourceKind};

    #[test]
    fn content_root_v1_matches_the_frozen_preimage_vector() {
        let entries = vec![ResourceEntry {
            path: "skills/example.md".to_string(),
            resource_kind: ResourceKind::Skill,
            mode: LogicalMode::Regular,
            size_bytes: 5,
            payload_offset: 0,
            sha256: sha256_hex(b"alpha"),
        }];
        let canonical_entries = canonical_json(&entries).expect("entry vector must canonicalize");

        assert_eq!(
            canonical_entries,
            br#"[{"mode":"0644","path":"skills/example.md","payload_offset":0,"resource_kind":"skill","sha256":"8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8","size_bytes":5}]"#
        );
        assert_eq!(
            content_root_sha256(&canonical_entries).expect("content root must hash"),
            "25410ed79c0a7d8a635d8e67a5acf536e6aff8be6a2f8ac7e6ece686409dfcb5"
        );
    }
}
