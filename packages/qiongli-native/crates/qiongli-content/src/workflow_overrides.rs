use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use sha2::{Digest, Sha256};

use crate::{LoadedResourcePack, LogicalMode, ResourceKind, ResourcePackLoaderError};

pub const MAX_WORKFLOW_OVERRIDE_BYTES: usize = 128 * 1024;
pub const MAX_WORKFLOW_OVERRIDE_TOTAL_BYTES: usize = 2 * 1024 * 1024;
const MAX_WORKFLOW_OVERRIDES: usize = 512;
const VARIANT_DIGEST_DOMAIN: &[u8] = b"qiongli-workflow-variant-v1\0";

#[derive(Debug)]
pub enum WorkflowOverrideError {
    Profile(ResourcePackLoaderError),
    ParentMismatch,
    PathNotAllowed(String),
    InvalidMarkdown(String),
    EntryTooLarge(String),
    VariantTooLarge,
}

impl Display for WorkflowOverrideError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Profile(error) => write!(formatter, "profile projection failed: {error}"),
            Self::ParentMismatch => formatter.write_str("workflow variant parent pack changed"),
            Self::PathNotAllowed(path) => {
                write!(formatter, "workflow override is not allowed: {path}")
            }
            Self::InvalidMarkdown(path) => write!(
                formatter,
                "workflow override is not valid Markdown text: {path}"
            ),
            Self::EntryTooLarge(path) => write!(
                formatter,
                "workflow override exceeds the per-file limit: {path}"
            ),
            Self::VariantTooLarge => {
                formatter.write_str("workflow variant exceeds its bounded limits")
            }
        }
    }
}

impl Error for WorkflowOverrideError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Profile(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowOverrideEntry {
    path: String,
    base_sha256: String,
    current_sha256: String,
    bytes: Vec<u8>,
}

impl WorkflowOverrideEntry {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn base_sha256(&self) -> &str {
        &self.base_sha256
    }

    #[must_use]
    pub fn current_sha256(&self) -> &str {
        &self.current_sha256
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn size_bytes(&self) -> u64 {
        u64::try_from(self.bytes.len()).unwrap_or(u64::MAX)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowOverrides {
    pack_sha256: String,
    content_root_sha256: String,
    variant_sha256: String,
    entries: Vec<WorkflowOverrideEntry>,
}

impl WorkflowOverrides {
    pub fn new(
        pack: &LoadedResourcePack<'_>,
        contents: BTreeMap<String, Vec<u8>>,
    ) -> Result<Option<Self>, WorkflowOverrideError> {
        if contents.len() > MAX_WORKFLOW_OVERRIDES {
            return Err(WorkflowOverrideError::VariantTooLarge);
        }
        let mut total_bytes = 0_usize;
        let mut entries = Vec::with_capacity(contents.len());
        for (path, bytes) in contents {
            let resource = pack
                .resource_for_profile("full", &path)
                .map_err(WorkflowOverrideError::Profile)?
                .filter(|resource| {
                    workflow_resource_is_editable(resource.entry().resource_kind, &path)
                })
                .ok_or_else(|| WorkflowOverrideError::PathNotAllowed(path.clone()))?;
            validate_markdown(&path, &bytes)?;
            total_bytes = total_bytes
                .checked_add(bytes.len())
                .filter(|total| *total <= MAX_WORKFLOW_OVERRIDE_TOTAL_BYTES)
                .ok_or(WorkflowOverrideError::VariantTooLarge)?;
            let current_sha256 = sha256_hex(&bytes);
            if current_sha256 == resource.entry().sha256 {
                continue;
            }
            entries.push(WorkflowOverrideEntry {
                path,
                base_sha256: resource.entry().sha256.clone(),
                current_sha256,
                bytes,
            });
        }
        if entries.is_empty() {
            return Ok(None);
        }
        let pack_sha256 = pack.pack_sha256().to_owned();
        let content_root_sha256 = pack.manifest().content_root_sha256.clone();
        let variant_sha256 = variant_sha256(&pack_sha256, &content_root_sha256, &entries);
        Ok(Some(Self {
            pack_sha256,
            content_root_sha256,
            variant_sha256,
            entries,
        }))
    }

    #[must_use]
    pub fn pack_sha256(&self) -> &str {
        &self.pack_sha256
    }

    #[must_use]
    pub fn content_root_sha256(&self) -> &str {
        &self.content_root_sha256
    }

    #[must_use]
    pub fn variant_sha256(&self) -> &str {
        &self.variant_sha256
    }

    #[must_use]
    pub fn entries(&self) -> &[WorkflowOverrideEntry] {
        &self.entries
    }

    #[must_use]
    pub fn entry(&self, path: &str) -> Option<&WorkflowOverrideEntry> {
        self.entries
            .binary_search_by(|entry| entry.path.as_str().cmp(path))
            .ok()
            .map(|index| &self.entries[index])
    }

    pub fn validate_parent(
        &self,
        pack: &LoadedResourcePack<'_>,
    ) -> Result<(), WorkflowOverrideError> {
        if self.pack_sha256 != pack.pack_sha256()
            || self.content_root_sha256 != pack.manifest().content_root_sha256
        {
            return Err(WorkflowOverrideError::ParentMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedResource {
    path: String,
    resource_kind: ResourceKind,
    mode: LogicalMode,
    canonical_sha256: String,
    current_sha256: String,
    bytes: Vec<u8>,
}

impl ProjectedResource {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn resource_kind(&self) -> ResourceKind {
        self.resource_kind
    }

    #[must_use]
    pub const fn mode(&self) -> LogicalMode {
        self.mode
    }

    #[must_use]
    pub fn canonical_sha256(&self) -> &str {
        &self.canonical_sha256
    }

    #[must_use]
    pub fn current_sha256(&self) -> &str {
        &self.current_sha256
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn size_bytes(&self) -> u64 {
        u64::try_from(self.bytes.len()).unwrap_or(u64::MAX)
    }
}

pub fn project_profile(
    pack: &LoadedResourcePack<'_>,
    profile: &str,
    overrides: Option<&WorkflowOverrides>,
) -> Result<Vec<ProjectedResource>, WorkflowOverrideError> {
    if let Some(overrides) = overrides {
        overrides.validate_parent(pack)?;
    }
    pack.resources_for_profile(profile)
        .map_err(WorkflowOverrideError::Profile)?
        .into_iter()
        .map(|resource| {
            let override_entry = overrides.and_then(|value| value.entry(&resource.entry().path));
            let bytes = override_entry
                .map_or_else(|| resource.bytes().to_vec(), |entry| entry.bytes.clone());
            Ok(ProjectedResource {
                path: resource.entry().path.clone(),
                resource_kind: resource.entry().resource_kind,
                mode: resource.entry().mode,
                canonical_sha256: resource.entry().sha256.clone(),
                current_sha256: override_entry.map_or_else(
                    || resource.entry().sha256.clone(),
                    |entry| entry.current_sha256.clone(),
                ),
                bytes,
            })
        })
        .collect()
}

#[must_use]
pub fn workflow_resource_is_editable(kind: ResourceKind, path: &str) -> bool {
    (path == "workflow/SKILL.md" && kind == ResourceKind::Workflow)
        || (kind == ResourceKind::Skill && path.ends_with(".md"))
}

fn validate_markdown(path: &str, bytes: &[u8]) -> Result<(), WorkflowOverrideError> {
    if bytes.len() > MAX_WORKFLOW_OVERRIDE_BYTES {
        return Err(WorkflowOverrideError::EntryTooLarge(path.to_owned()));
    }
    let content = std::str::from_utf8(bytes)
        .map_err(|_| WorkflowOverrideError::InvalidMarkdown(path.to_owned()))?;
    if content.chars().any(|character| {
        character == '\r' || (character.is_control() && !matches!(character, '\n' | '\t'))
    }) {
        return Err(WorkflowOverrideError::InvalidMarkdown(path.to_owned()));
    }
    Ok(())
}

fn variant_sha256(
    pack_sha256: &str,
    content_root_sha256: &str,
    entries: &[WorkflowOverrideEntry],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(VARIANT_DIGEST_DOMAIN);
    hash_field(&mut hasher, pack_sha256.as_bytes());
    hash_field(&mut hasher, content_root_sha256.as_bytes());
    for entry in entries {
        hash_field(&mut hasher, entry.path.as_bytes());
        hash_field(&mut hasher, entry.base_sha256.as_bytes());
        hash_field(&mut hasher, entry.current_sha256.as_bytes());
        hasher.update(entry.size_bytes().to_be_bytes());
    }
    encode_hex(&hasher.finalize())
}

fn hash_field(hasher: &mut Sha256, value: &[u8]) {
    hasher.update(u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(value);
}

fn sha256_hex(bytes: &[u8]) -> String {
    encode_hex(&Sha256::digest(bytes))
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}
