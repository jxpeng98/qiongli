use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use unicode_normalization::UnicodeNormalization;

use crate::collector::expected_resource_kind;
use crate::manifest::{
    ManifestError, RESOURCE_PACK_FORMAT_VERSION, ResourceEntry, ResourceKind,
    ResourcePackManifestV1,
};
use crate::writer::{
    RESOURCE_PACK_HEADER_LEN, RESOURCE_PACK_MAGIC, ResourcePackWriterError, canonical_json,
    content_root_sha256, sha256_hex,
};

const DEFAULT_MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const DEFAULT_MAX_ENTRIES: usize = 4_096;
const DEFAULT_MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
const DEFAULT_MAX_PAYLOAD_BYTES: u64 = 128 * 1024 * 1024;
const DEFAULT_MAX_PATH_DEPTH: usize = 32;
const DEFAULT_MAX_PACK_BYTES: u64 =
    DEFAULT_MAX_MANIFEST_BYTES + DEFAULT_MAX_PAYLOAD_BYTES + RESOURCE_PACK_HEADER_LEN as u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourcePackLimits {
    pub max_pack_bytes: u64,
    pub max_manifest_bytes: u64,
    pub max_entries: usize,
    pub max_entry_bytes: u64,
    pub max_payload_bytes: u64,
    pub max_path_depth: usize,
}

impl Default for ResourcePackLimits {
    fn default() -> Self {
        Self {
            max_pack_bytes: DEFAULT_MAX_PACK_BYTES,
            max_manifest_bytes: DEFAULT_MAX_MANIFEST_BYTES,
            max_entries: DEFAULT_MAX_ENTRIES,
            max_entry_bytes: DEFAULT_MAX_ENTRY_BYTES,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
            max_path_depth: DEFAULT_MAX_PATH_DEPTH,
        }
    }
}

impl ResourcePackLimits {
    fn validate(self) -> Result<(), ResourcePackLoaderError> {
        if self.max_pack_bytes == 0 {
            return Err(ResourcePackLoaderError::InvalidLimits("max_pack_bytes"));
        }
        if self.max_manifest_bytes == 0 {
            return Err(ResourcePackLoaderError::InvalidLimits("max_manifest_bytes"));
        }
        if self.max_entries == 0 {
            return Err(ResourcePackLoaderError::InvalidLimits("max_entries"));
        }
        if self.max_entry_bytes == 0 {
            return Err(ResourcePackLoaderError::InvalidLimits("max_entry_bytes"));
        }
        if self.max_payload_bytes == 0 {
            return Err(ResourcePackLoaderError::InvalidLimits("max_payload_bytes"));
        }
        if self.max_path_depth == 0 {
            return Err(ResourcePackLoaderError::InvalidLimits("max_path_depth"));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ResourcePackLoaderError {
    InvalidLimits(&'static str),
    InvalidExpectedPackSha256,
    PackSizeOverflow,
    PackTooLarge {
        size_bytes: u64,
        limit: u64,
    },
    PackDigestMismatch,
    TruncatedHeader {
        actual_bytes: usize,
    },
    InvalidMagic,
    UnsupportedFormatVersion {
        found: u32,
    },
    ManifestTooLarge {
        size_bytes: u64,
        limit: u64,
    },
    ManifestLengthOverflow,
    TruncatedManifest {
        declared_bytes: u64,
        available_bytes: usize,
    },
    InvalidManifestUtf8(std::str::Utf8Error),
    InvalidManifest(ManifestError),
    CanonicalJson(ResourcePackWriterError),
    NonCanonicalManifest,
    EntryLimitExceeded {
        count: usize,
        limit: usize,
    },
    EntryTooLarge {
        path: String,
        size_bytes: u64,
        limit: u64,
    },
    PayloadSizeOverflow,
    PayloadTooLarge {
        size_bytes: u64,
        limit: u64,
    },
    PathDepthExceeded {
        path: String,
        depth: usize,
        limit: usize,
    },
    InvalidEntryPath {
        path: String,
    },
    EntryOutsideCanonicalSources {
        path: String,
    },
    ResourceKindMismatch {
        path: String,
        declared: ResourceKind,
        expected: ResourceKind,
    },
    PortablePathCollision {
        first: String,
        second: String,
    },
    PayloadLengthMismatch {
        declared_bytes: u64,
        actual_bytes: u64,
    },
    ContentRootMismatch,
    EntryDigestMismatch {
        path: String,
    },
    InvalidProfile(ManifestError),
    InvalidProfileProjection,
}

impl Display for ResourcePackLoaderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits(field) => {
                write!(
                    formatter,
                    "resource-pack limit {field} must be greater than zero"
                )
            }
            Self::InvalidExpectedPackSha256 => formatter.write_str(
                "expected resource-pack SHA-256 must be 64 lowercase hexadecimal characters",
            ),
            Self::PackSizeOverflow => {
                formatter.write_str("resource-pack byte length exceeds the supported range")
            }
            Self::PackTooLarge { size_bytes, limit } => write!(
                formatter,
                "resource pack is {size_bytes} bytes, above the {limit}-byte limit"
            ),
            Self::PackDigestMismatch => {
                formatter.write_str("resource-pack whole-core SHA-256 mismatch")
            }
            Self::TruncatedHeader { actual_bytes } => write!(
                formatter,
                "resource-pack header is truncated: expected {RESOURCE_PACK_HEADER_LEN} bytes, found {actual_bytes}"
            ),
            Self::InvalidMagic => formatter.write_str("resource-pack magic bytes are invalid"),
            Self::UnsupportedFormatVersion { found } => write!(
                formatter,
                "unsupported resource-pack format version: {found}"
            ),
            Self::ManifestTooLarge { size_bytes, limit } => write!(
                formatter,
                "resource-pack manifest is {size_bytes} bytes, above the {limit}-byte limit"
            ),
            Self::ManifestLengthOverflow => formatter
                .write_str("resource-pack manifest length exceeds the supported address range"),
            Self::TruncatedManifest {
                declared_bytes,
                available_bytes,
            } => write!(
                formatter,
                "resource-pack manifest declares {declared_bytes} bytes but only {available_bytes} remain"
            ),
            Self::InvalidManifestUtf8(error) => {
                write!(formatter, "resource-pack manifest is not UTF-8: {error}")
            }
            Self::InvalidManifest(error) => {
                write!(formatter, "invalid resource-pack manifest: {error}")
            }
            Self::CanonicalJson(error) => {
                write!(
                    formatter,
                    "resource-pack canonical JSON verification failed: {error}"
                )
            }
            Self::NonCanonicalManifest => {
                formatter.write_str("resource-pack manifest is not canonical RFC 8785 JSON")
            }
            Self::EntryLimitExceeded { count, limit } => write!(
                formatter,
                "resource-pack contains {count} entries, above the {limit}-entry limit"
            ),
            Self::EntryTooLarge {
                path,
                size_bytes,
                limit,
            } => write!(
                formatter,
                "resource-pack entry {path:?} is {size_bytes} bytes, above the {limit}-byte limit"
            ),
            Self::PayloadSizeOverflow => {
                formatter.write_str("resource-pack declared payload size overflow")
            }
            Self::PayloadTooLarge { size_bytes, limit } => write!(
                formatter,
                "resource-pack payload is {size_bytes} bytes, above the {limit}-byte limit"
            ),
            Self::PathDepthExceeded { path, depth, limit } => write!(
                formatter,
                "resource-pack path {path:?} has depth {depth}, above the {limit}-component limit"
            ),
            Self::InvalidEntryPath { path } => {
                write!(formatter, "resource-pack path {path:?} is not portable")
            }
            Self::EntryOutsideCanonicalSources { path } => write!(
                formatter,
                "resource-pack path {path:?} is outside the canonical source allowlist"
            ),
            Self::ResourceKindMismatch {
                path,
                declared,
                expected,
            } => write!(
                formatter,
                "resource-pack path {path:?} declares {declared:?}, expected {expected:?}"
            ),
            Self::PortablePathCollision { first, second } => write!(
                formatter,
                "resource-pack paths collide portably: {first:?} and {second:?}"
            ),
            Self::PayloadLengthMismatch {
                declared_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "resource-pack payload declares {declared_bytes} bytes but contains {actual_bytes}"
            ),
            Self::ContentRootMismatch => {
                formatter.write_str("resource-pack content-root SHA-256 mismatch")
            }
            Self::EntryDigestMismatch { path } => {
                write!(formatter, "resource-pack entry SHA-256 mismatch: {path}")
            }
            Self::InvalidProfile(error) => {
                write!(formatter, "invalid resource-pack profile: {error}")
            }
            Self::InvalidProfileProjection => {
                formatter.write_str("validated resource-pack profile projection is unavailable")
            }
        }
    }
}

impl Error for ResourcePackLoaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidManifestUtf8(error) => Some(error),
            Self::InvalidManifest(error) | Self::InvalidProfile(error) => Some(error),
            Self::CanonicalJson(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct LoadedResourcePack<'a> {
    core_bytes: &'a [u8],
    manifest_bytes: &'a [u8],
    payload_bytes: &'a [u8],
    manifest: ResourcePackManifestV1,
    entry_ranges: Box<[Range<usize>]>,
    pack_sha256: String,
}

impl<'a> LoadedResourcePack<'a> {
    #[must_use]
    pub fn core_bytes(&self) -> &'a [u8] {
        self.core_bytes
    }

    #[must_use]
    pub fn manifest_bytes(&self) -> &'a [u8] {
        self.manifest_bytes
    }

    #[must_use]
    pub fn manifest(&self) -> &ResourcePackManifestV1 {
        &self.manifest
    }

    #[must_use]
    pub fn pack_sha256(&self) -> &str {
        &self.pack_sha256
    }

    pub fn resources_for_profile<'pack>(
        &'pack self,
        profile: &str,
    ) -> Result<Vec<LoadedResource<'pack, 'a>>, ResourcePackLoaderError> {
        let profile_id = self
            .manifest
            .resolve_profile(profile)
            .map_err(ResourcePackLoaderError::InvalidProfile)?;
        let projection = self
            .manifest
            .profiles
            .iter()
            .find(|candidate| candidate.id == profile_id)
            .ok_or(ResourcePackLoaderError::InvalidProfileProjection)?;
        let included_kinds = projection
            .included_resource_kinds
            .iter()
            .copied()
            .collect::<BTreeSet<ResourceKind>>();

        Ok(self
            .manifest
            .entries
            .iter()
            .zip(self.entry_ranges.iter())
            .filter(|(entry, _)| included_kinds.contains(&entry.resource_kind))
            .map(|(entry, range)| LoadedResource {
                entry,
                bytes: &self.payload_bytes[range.clone()],
            })
            .collect())
    }

    pub fn resource_for_profile<'pack>(
        &'pack self,
        profile: &str,
        path: &str,
    ) -> Result<Option<LoadedResource<'pack, 'a>>, ResourcePackLoaderError> {
        let profile_id = self
            .manifest
            .resolve_profile(profile)
            .map_err(ResourcePackLoaderError::InvalidProfile)?;
        let projection = self
            .manifest
            .profiles
            .iter()
            .find(|candidate| candidate.id == profile_id)
            .ok_or(ResourcePackLoaderError::InvalidProfileProjection)?;
        let Some(index) = self
            .manifest
            .entries
            .binary_search_by(|entry| entry.path.as_str().cmp(path))
            .ok()
        else {
            return Ok(None);
        };
        let entry = &self.manifest.entries[index];
        if !projection
            .included_resource_kinds
            .contains(&entry.resource_kind)
        {
            return Ok(None);
        }
        Ok(Some(LoadedResource {
            entry,
            bytes: &self.payload_bytes[self.entry_ranges[index].clone()],
        }))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LoadedResource<'pack, 'bytes> {
    entry: &'pack ResourceEntry,
    bytes: &'bytes [u8],
}

impl<'pack, 'bytes> LoadedResource<'pack, 'bytes> {
    #[must_use]
    pub fn entry(&self) -> &'pack ResourceEntry {
        self.entry
    }

    #[must_use]
    pub fn bytes(&self) -> &'bytes [u8] {
        self.bytes
    }
}

pub fn load_resource_pack<'a>(
    core_bytes: &'a [u8],
    expected_pack_sha256: &str,
) -> Result<LoadedResourcePack<'a>, ResourcePackLoaderError> {
    load_resource_pack_with_limits(
        core_bytes,
        expected_pack_sha256,
        ResourcePackLimits::default(),
    )
}

pub fn load_resource_pack_with_limits<'a>(
    core_bytes: &'a [u8],
    expected_pack_sha256: &str,
    limits: ResourcePackLimits,
) -> Result<LoadedResourcePack<'a>, ResourcePackLoaderError> {
    limits.validate()?;
    let pack_size =
        u64::try_from(core_bytes.len()).map_err(|_| ResourcePackLoaderError::PackSizeOverflow)?;
    if pack_size > limits.max_pack_bytes {
        return Err(ResourcePackLoaderError::PackTooLarge {
            size_bytes: pack_size,
            limit: limits.max_pack_bytes,
        });
    }
    if !is_lower_hex(expected_pack_sha256, 64) {
        return Err(ResourcePackLoaderError::InvalidExpectedPackSha256);
    }
    let pack_sha256 = sha256_hex(core_bytes);
    if pack_sha256 != expected_pack_sha256 {
        return Err(ResourcePackLoaderError::PackDigestMismatch);
    }
    if core_bytes.len() < RESOURCE_PACK_HEADER_LEN {
        return Err(ResourcePackLoaderError::TruncatedHeader {
            actual_bytes: core_bytes.len(),
        });
    }
    if core_bytes[..RESOURCE_PACK_MAGIC.len()] != RESOURCE_PACK_MAGIC {
        return Err(ResourcePackLoaderError::InvalidMagic);
    }

    let version_end = RESOURCE_PACK_MAGIC.len() + size_of::<u32>();
    let format_version = u32::from_le_bytes(
        core_bytes[RESOURCE_PACK_MAGIC.len()..version_end]
            .try_into()
            .expect("resource-pack version width is fixed after header validation"),
    );
    if format_version != RESOURCE_PACK_FORMAT_VERSION {
        return Err(ResourcePackLoaderError::UnsupportedFormatVersion {
            found: format_version,
        });
    }

    let manifest_len = u64::from_le_bytes(
        core_bytes[version_end..RESOURCE_PACK_HEADER_LEN]
            .try_into()
            .expect("resource-pack manifest-length width is fixed after header validation"),
    );
    if manifest_len > limits.max_manifest_bytes {
        return Err(ResourcePackLoaderError::ManifestTooLarge {
            size_bytes: manifest_len,
            limit: limits.max_manifest_bytes,
        });
    }
    let manifest_len_usize = usize::try_from(manifest_len)
        .map_err(|_| ResourcePackLoaderError::ManifestLengthOverflow)?;
    let manifest_end = RESOURCE_PACK_HEADER_LEN
        .checked_add(manifest_len_usize)
        .ok_or(ResourcePackLoaderError::ManifestLengthOverflow)?;
    if manifest_end > core_bytes.len() {
        return Err(ResourcePackLoaderError::TruncatedManifest {
            declared_bytes: manifest_len,
            available_bytes: core_bytes.len() - RESOURCE_PACK_HEADER_LEN,
        });
    }

    let manifest_bytes = &core_bytes[RESOURCE_PACK_HEADER_LEN..manifest_end];
    let manifest_text = std::str::from_utf8(manifest_bytes)
        .map_err(ResourcePackLoaderError::InvalidManifestUtf8)?;
    let manifest = ResourcePackManifestV1::from_json(manifest_text)
        .map_err(ResourcePackLoaderError::InvalidManifest)?;
    manifest
        .validate()
        .map_err(ResourcePackLoaderError::InvalidManifest)?;
    let canonical_manifest =
        canonical_json(&manifest).map_err(ResourcePackLoaderError::CanonicalJson)?;
    if canonical_manifest != manifest_bytes {
        return Err(ResourcePackLoaderError::NonCanonicalManifest);
    }
    if manifest.entries.len() > limits.max_entries {
        return Err(ResourcePackLoaderError::EntryLimitExceeded {
            count: manifest.entries.len(),
            limit: limits.max_entries,
        });
    }

    let mut declared_payload_bytes = 0_u64;
    let mut portable_paths = BTreeMap::<String, String>::new();
    for entry in &manifest.entries {
        if entry.size_bytes > limits.max_entry_bytes {
            return Err(ResourcePackLoaderError::EntryTooLarge {
                path: entry.path.clone(),
                size_bytes: entry.size_bytes,
                limit: limits.max_entry_bytes,
            });
        }
        validate_loaded_path(&entry.path, limits.max_path_depth)?;
        let expected_kind = expected_resource_kind(&entry.path).ok_or_else(|| {
            ResourcePackLoaderError::EntryOutsideCanonicalSources {
                path: entry.path.clone(),
            }
        })?;
        if entry.resource_kind != expected_kind {
            return Err(ResourcePackLoaderError::ResourceKindMismatch {
                path: entry.path.clone(),
                declared: entry.resource_kind,
                expected: expected_kind,
            });
        }
        let portable_key = portable_path_key(&entry.path);
        if let Some(first) = portable_paths.insert(portable_key, entry.path.clone()) {
            return Err(ResourcePackLoaderError::PortablePathCollision {
                first,
                second: entry.path.clone(),
            });
        }
        declared_payload_bytes = declared_payload_bytes
            .checked_add(entry.size_bytes)
            .ok_or(ResourcePackLoaderError::PayloadSizeOverflow)?;
    }
    if declared_payload_bytes > limits.max_payload_bytes {
        return Err(ResourcePackLoaderError::PayloadTooLarge {
            size_bytes: declared_payload_bytes,
            limit: limits.max_payload_bytes,
        });
    }

    let payload_bytes = &core_bytes[manifest_end..];
    let actual_payload_bytes = u64::try_from(payload_bytes.len())
        .map_err(|_| ResourcePackLoaderError::PackSizeOverflow)?;
    if declared_payload_bytes != actual_payload_bytes {
        return Err(ResourcePackLoaderError::PayloadLengthMismatch {
            declared_bytes: declared_payload_bytes,
            actual_bytes: actual_payload_bytes,
        });
    }

    let canonical_entries =
        canonical_json(&manifest.entries).map_err(ResourcePackLoaderError::CanonicalJson)?;
    let actual_content_root =
        content_root_sha256(&canonical_entries).map_err(ResourcePackLoaderError::CanonicalJson)?;
    if actual_content_root != manifest.content_root_sha256 {
        return Err(ResourcePackLoaderError::ContentRootMismatch);
    }

    let mut entry_ranges = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let start = usize::try_from(entry.payload_offset)
            .map_err(|_| ResourcePackLoaderError::PayloadSizeOverflow)?;
        let size = usize::try_from(entry.size_bytes)
            .map_err(|_| ResourcePackLoaderError::PayloadSizeOverflow)?;
        let end = start
            .checked_add(size)
            .ok_or(ResourcePackLoaderError::PayloadSizeOverflow)?;
        let entry_bytes = payload_bytes.get(start..end).ok_or(
            ResourcePackLoaderError::PayloadLengthMismatch {
                declared_bytes: declared_payload_bytes,
                actual_bytes: actual_payload_bytes,
            },
        )?;
        if sha256_hex(entry_bytes) != entry.sha256 {
            return Err(ResourcePackLoaderError::EntryDigestMismatch {
                path: entry.path.clone(),
            });
        }
        entry_ranges.push(start..end);
    }

    Ok(LoadedResourcePack {
        core_bytes,
        manifest_bytes,
        payload_bytes,
        manifest,
        entry_ranges: entry_ranges.into_boxed_slice(),
        pack_sha256,
    })
}

fn is_lower_hex(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_loaded_path(path: &str, max_depth: usize) -> Result<(), ResourcePackLoaderError> {
    let components = path.split('/').collect::<Vec<_>>();
    if components.len() > max_depth {
        return Err(ResourcePackLoaderError::PathDepthExceeded {
            path: path.to_string(),
            depth: components.len(),
            limit: max_depth,
        });
    }
    for component in components {
        let normalized = component.nfc().collect::<String>();
        if normalized != component
            || component.is_empty()
            || component == "."
            || component == ".."
            || component.ends_with('.')
            || component.ends_with(' ')
            || component.chars().any(|character| {
                character.is_control()
                    || matches!(character, ':' | '*' | '?' | '"' | '<' | '>' | '|')
            })
            || is_windows_device_name(component)
        {
            return Err(ResourcePackLoaderError::InvalidEntryPath {
                path: path.to_string(),
            });
        }
    }
    Ok(())
}

fn portable_path_key(path: &str) -> String {
    path.chars().flat_map(char::to_lowercase).nfc().collect()
}

fn is_windows_device_name(component: &str) -> bool {
    let base = component.split('.').next().unwrap_or(component);
    let upper = base.to_ascii_uppercase();
    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && matches!(&upper[..3], "COM" | "LPT")
            && matches!(upper.as_bytes()[3], b'1'..=b'9'))
}
