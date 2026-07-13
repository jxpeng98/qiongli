use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use semver::Version;
use serde::{Deserialize, Serialize};

pub const RESOURCE_PACK_FORMAT_VERSION: u32 = 1;
pub const RESOURCE_PACK_COMPILER_CONTRACT_VERSION: u32 = 1;
pub const JCS_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestError(String);

impl ManifestError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for ManifestError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ManifestError {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileId {
    SkillOnly,
    MarketplaceLite,
    Full,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceKind {
    TargetMetadata,
    McpContract,
    Role,
    Schema,
    Skill,
    SkillSummary,
    Standard,
    Subject,
    Template,
    VenueProfile,
    Workflow,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LogicalMode {
    #[serde(rename = "0644")]
    Regular,
    #[serde(rename = "0755")]
    Executable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibleProduct {
    pub minimum: String,
    pub maximum_exclusive: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileProjection {
    pub id: ProfileId,
    pub aliases: Vec<String>,
    pub included_resource_kinds: Vec<ResourceKind>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceEntry {
    pub path: String,
    pub resource_kind: ResourceKind,
    pub mode: LogicalMode,
    pub size_bytes: u64,
    pub payload_offset: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePackManifestV1 {
    pub format_version: u32,
    pub compiler_contract_version: u32,
    pub pack_id: String,
    pub content_version: String,
    pub source_commit: String,
    pub compatible_product: CompatibleProduct,
    pub profiles: Vec<ProfileProjection>,
    pub entries: Vec<ResourceEntry>,
    pub content_root_sha256: String,
}

impl ResourcePackManifestV1 {
    pub fn from_json(input: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(input)
            .map_err(|error| ManifestError::new(format!("invalid resource-pack manifest: {error}")))
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.format_version != RESOURCE_PACK_FORMAT_VERSION {
            return Err(ManifestError::new("format_version must be 1"));
        }
        if self.compiler_contract_version != RESOURCE_PACK_COMPILER_CONTRACT_VERSION {
            return Err(ManifestError::new("compiler_contract_version must be 1"));
        }
        if self.pack_id.is_empty()
            || !self
                .pack_id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(ManifestError::new(
                "pack_id must use lowercase ASCII identifiers",
            ));
        }

        Version::parse(&self.content_version)
            .map_err(|_| ManifestError::new("content_version must be SemVer"))?;
        let minimum = Version::parse(&self.compatible_product.minimum)
            .map_err(|_| ManifestError::new("compatible_product.minimum must be SemVer"))?;
        let maximum = Version::parse(&self.compatible_product.maximum_exclusive).map_err(|_| {
            ManifestError::new("compatible_product.maximum_exclusive must be SemVer")
        })?;
        if minimum >= maximum {
            return Err(ManifestError::new(
                "compatible product range must be non-empty",
            ));
        }

        require_lower_hex(&self.source_commit, 40, "source_commit")?;
        require_lower_hex(&self.content_root_sha256, 64, "content_root_sha256")?;
        self.validate_profiles()?;
        self.validate_entries()?;
        Ok(())
    }

    pub fn resolve_profile(&self, profile: &str) -> Result<ProfileId, ManifestError> {
        self.profiles
            .iter()
            .find(|candidate| {
                profile_id(candidate.id) == profile
                    || candidate.aliases.iter().any(|alias| alias == profile)
            })
            .map(|candidate| candidate.id)
            .ok_or_else(|| ManifestError::new("unknown resource-pack profile"))
    }

    pub fn entries_for_profile(&self, profile: &str) -> Result<Vec<&ResourceEntry>, ManifestError> {
        let profile_id = self.resolve_profile(profile)?;
        let projection = self
            .profiles
            .iter()
            .find(|candidate| candidate.id == profile_id)
            .ok_or_else(|| ManifestError::new("resource-pack profile is unavailable"))?;
        let kinds: BTreeSet<_> = projection.included_resource_kinds.iter().copied().collect();
        Ok(self
            .entries
            .iter()
            .filter(|entry| kinds.contains(&entry.resource_kind))
            .collect())
    }

    fn validate_profiles(&self) -> Result<(), ManifestError> {
        let profiles: BTreeMap<_, _> = self
            .profiles
            .iter()
            .map(|profile| (profile.id, profile))
            .collect();
        if profiles.len() != self.profiles.len() || profiles.len() != 3 {
            return Err(ManifestError::new(
                "profiles must contain skill-only, marketplace-lite, and full exactly once",
            ));
        }

        let profile_order = self
            .profiles
            .iter()
            .map(|profile| profile.id)
            .collect::<Vec<_>>();
        if profile_order
            != [
                ProfileId::SkillOnly,
                ProfileId::MarketplaceLite,
                ProfileId::Full,
            ]
        {
            return Err(ManifestError::new(
                "resource-pack profiles must use canonical order",
            ));
        }

        for profile_id in [
            ProfileId::SkillOnly,
            ProfileId::MarketplaceLite,
            ProfileId::Full,
        ] {
            let profile = profiles
                .get(&profile_id)
                .ok_or_else(|| ManifestError::new("required resource-pack profile is missing"))?;
            let expected_aliases: &[&str] = if profile_id == ProfileId::MarketplaceLite {
                &["lite"]
            } else {
                &[]
            };
            if profile
                .aliases
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != expected_aliases
            {
                return Err(ManifestError::new("resource-pack profile aliases drifted"));
            }
            let expected_kinds = expected_resource_kinds(profile_id);
            if profile.included_resource_kinds != expected_kinds {
                return Err(ManifestError::new(
                    "resource-pack profile projection drifted",
                ));
            }
        }
        Ok(())
    }

    fn validate_entries(&self) -> Result<(), ManifestError> {
        if self.entries.is_empty() {
            return Err(ManifestError::new(
                "resource-pack entries must not be empty",
            ));
        }
        let mut expected_offset = 0_u64;
        let mut previous_path: Option<&str> = None;
        for entry in &self.entries {
            validate_pack_path(&entry.path)?;
            if previous_path.is_some_and(|previous| previous >= entry.path.as_str()) {
                return Err(ManifestError::new(
                    "resource-pack entries must use unique ascending paths",
                ));
            }
            if entry.payload_offset != expected_offset {
                return Err(ManifestError::new(
                    "resource-pack payload offsets must be contiguous",
                ));
            }
            if entry.size_bytes > JCS_MAX_SAFE_INTEGER
                || entry.payload_offset > JCS_MAX_SAFE_INTEGER
            {
                return Err(ManifestError::new(
                    "resource-pack numeric fields exceed the JCS safe-integer range",
                ));
            }
            require_lower_hex(&entry.sha256, 64, "entry sha256")?;
            expected_offset = expected_offset
                .checked_add(entry.size_bytes)
                .ok_or_else(|| ManifestError::new("resource-pack payload size overflow"))?;
            previous_path = Some(&entry.path);
        }
        Ok(())
    }
}

fn profile_id(profile: ProfileId) -> &'static str {
    match profile {
        ProfileId::SkillOnly => "skill-only",
        ProfileId::MarketplaceLite => "marketplace-lite",
        ProfileId::Full => "full",
    }
}

fn expected_resource_kinds(profile: ProfileId) -> Vec<ResourceKind> {
    let all = vec![
        ResourceKind::TargetMetadata,
        ResourceKind::McpContract,
        ResourceKind::Role,
        ResourceKind::Schema,
        ResourceKind::Skill,
        ResourceKind::SkillSummary,
        ResourceKind::Standard,
        ResourceKind::Subject,
        ResourceKind::Template,
        ResourceKind::VenueProfile,
        ResourceKind::Workflow,
    ];
    if profile == ProfileId::SkillOnly {
        all.into_iter()
            .filter(|kind| {
                !matches!(
                    kind,
                    ResourceKind::TargetMetadata | ResourceKind::McpContract | ResourceKind::Schema
                )
            })
            .collect()
    } else {
        all
    }
}

pub(crate) fn canonical_profile_projections() -> Vec<ProfileProjection> {
    [
        ProfileId::SkillOnly,
        ProfileId::MarketplaceLite,
        ProfileId::Full,
    ]
    .into_iter()
    .map(|id| ProfileProjection {
        id,
        aliases: if id == ProfileId::MarketplaceLite {
            vec!["lite".to_string()]
        } else {
            Vec::new()
        },
        included_resource_kinds: expected_resource_kinds(id),
    })
    .collect()
}

fn validate_pack_path(path: &str) -> Result<(), ManifestError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "..")
    {
        return Err(ManifestError::new(
            "resource-pack entry path is not portable",
        ));
    }
    Ok(())
}

fn require_lower_hex(value: &str, length: usize, label: &str) -> Result<(), ManifestError> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ManifestError::new(format!(
            "{label} must be {length} lowercase hexadecimal characters"
        )));
    }
    Ok(())
}
