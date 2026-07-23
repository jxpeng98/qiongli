use std::fmt::{self, Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    ClaudeMarketplaceState, ClaudeRegistrationState, ClaudeSkillsPluginState, ClaudeSourceState,
    CodexMarketplaceState, CodexRegistrationState, CodexSourceState,
    discover_claude_user_with_config, discover_codex_user,
};

pub const CLIENT_INVENTORY_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Copy)]
pub struct ClientInventoryInput<'a> {
    home: &'a Path,
    codex_config_root: Option<&'a Path>,
    claude_config_root: Option<&'a Path>,
    project_root: Option<&'a Path>,
    codex_custom_skills_root: Option<&'a Path>,
    claude_custom_skills_root: Option<&'a Path>,
    codex_host_present: bool,
    claude_host_present: bool,
}

impl<'a> ClientInventoryInput<'a> {
    #[must_use]
    pub const fn new(home: &'a Path) -> Self {
        Self {
            home,
            codex_config_root: None,
            claude_config_root: None,
            project_root: None,
            codex_custom_skills_root: None,
            claude_custom_skills_root: None,
            codex_host_present: false,
            claude_host_present: false,
        }
    }

    #[must_use]
    pub const fn with_codex_config_root(mut self, path: Option<&'a Path>) -> Self {
        self.codex_config_root = path;
        self
    }

    #[must_use]
    pub const fn with_claude_config_root(mut self, path: Option<&'a Path>) -> Self {
        self.claude_config_root = path;
        self
    }

    #[must_use]
    pub const fn with_project_root(mut self, path: Option<&'a Path>) -> Self {
        self.project_root = path;
        self
    }

    #[must_use]
    pub const fn with_custom_skills_roots(
        mut self,
        codex: Option<&'a Path>,
        claude: Option<&'a Path>,
    ) -> Self {
        self.codex_custom_skills_root = codex;
        self.claude_custom_skills_root = claude;
        self
    }

    #[must_use]
    pub const fn with_host_presence(mut self, codex: bool, claude: bool) -> Self {
        self.codex_host_present = codex;
        self.claude_host_present = claude;
        self
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientKind {
    Codex,
    ClaudeCode,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientDiscoveryState {
    NotDetected,
    Detected,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientOwnershipState {
    NotInstalled,
    QiongliManaged,
    Unmanaged,
    Mixed,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientActionReadiness {
    InspectOnly,
    InstallReady,
    Current,
    RepairReady,
    ResolveConflict,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientHostPresence {
    NotObserved,
    Observed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientComponentState {
    Missing,
    Ready,
    Conflict,
    Drifted,
    RecoveryRequired,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientPathSurface {
    ClientConfig,
    SkillsRoot,
    SkillsPackage,
    PluginMarketplace,
    PluginSource,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientPathScope {
    User,
    Project,
    Managed,
    Custom,
    Legacy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientPathSource {
    EnvironmentOverride,
    OfficialDefault,
    ProjectContext,
    QiongliManaged,
    ExplicitCustom,
    LegacyObserved,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientPathState {
    Missing,
    Directory,
    File,
    Symlink,
    Invalid,
    Unsafe,
    Unavailable,
}

impl ClientPathState {
    const fn is_observed(self) -> bool {
        matches!(
            self,
            Self::Directory | Self::File | Self::Symlink | Self::Invalid
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientPathManagement {
    Supported,
    InspectOnly,
    LegacyOnly,
    Unsafe,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientPathId {
    CodexConfig,
    CodexUserSkills,
    CodexProjectSkills,
    CodexCustomSkills,
    CodexMarketplace,
    CodexPluginSource,
    CodexLegacyPluginSource,
    CodexLegacySkills,
    ClaudeConfig,
    ClaudeUserSkills,
    ClaudeProjectSkills,
    ClaudeCustomSkills,
    ClaudeMarketplace,
    ClaudePluginSource,
    ClaudeLegacyPluginSource,
    ClaudeDirectSkills,
    ClaudeLegacySkills,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClientSymbolicPath {
    CodexConfig,
    CodexConfigOverride,
    CodexUserSkills,
    CodexProjectSkills,
    CodexCustomSkills,
    CodexMarketplace,
    CodexPluginSource,
    CodexLegacyPluginSource,
    CodexLegacySkills,
    ClaudeConfig,
    ClaudeConfigOverride,
    ClaudeUserSkills,
    ClaudeProjectSkills,
    ClaudeCustomSkills,
    ClaudeMarketplace,
    ClaudePluginSource,
    ClaudeLegacyPluginSource,
    ClaudeDirectSkills,
    ClaudeLegacySkills,
}

impl ClientSymbolicPath {
    #[must_use]
    pub const fn display(self) -> &'static str {
        match self {
            Self::CodexConfig => "<user-home>/.codex",
            Self::CodexConfigOverride => "<codex-config>",
            Self::CodexUserSkills => "<user-home>/.agents/skills",
            Self::CodexProjectSkills => "<project-root>/.agents/skills",
            Self::CodexCustomSkills => "<custom-codex-skills-root>",
            Self::CodexMarketplace => "<user-home>/.agents/plugins/marketplace.json",
            Self::CodexPluginSource => "<user-home>/.qiongli/plugins/codex/qiongli-next",
            Self::CodexLegacyPluginSource => "<user-home>/.qiongli/plugins/codex/qiongli",
            Self::CodexLegacySkills => "<codex-config>/skills/qiongli-workflow",
            Self::ClaudeConfig => "<user-home>/.claude",
            Self::ClaudeConfigOverride => "<claude-config>",
            Self::ClaudeUserSkills => "<claude-config>/skills",
            Self::ClaudeProjectSkills => "<project-root>/.claude/skills",
            Self::ClaudeCustomSkills => "<custom-claude-skills-root>",
            Self::ClaudeMarketplace => {
                "<user-home>/.qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json"
            }
            Self::ClaudePluginSource => {
                "<user-home>/.qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next"
            }
            Self::ClaudeLegacyPluginSource => {
                "<user-home>/.qiongli/plugins/claude-code/qiongli-local/plugins/qiongli"
            }
            Self::ClaudeDirectSkills => "<claude-config>/skills/qiongli-next",
            Self::ClaudeLegacySkills => "<claude-config>/skills/qiongli-workflow",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientPathCandidateV1 {
    pub id: ClientPathId,
    pub surface: ClientPathSurface,
    pub scope: ClientPathScope,
    pub source: ClientPathSource,
    pub state: ClientPathState,
    pub management: ClientPathManagement,
    pub selected: bool,
    pub symbolic_path: ClientSymbolicPath,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientComponentInventoryV1 {
    pub skills: ClientComponentState,
    pub plugin_source: ClientComponentState,
    pub full_mcp: ClientComponentState,
    pub marketplace: ClientComponentState,
    pub registration: ClientComponentState,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientInventoryEntryV1 {
    pub client: ClientKind,
    pub discovery: ClientDiscoveryState,
    pub ownership: ClientOwnershipState,
    pub readiness: ClientActionReadiness,
    pub host_presence: ClientHostPresence,
    pub installed_plugin_version: Option<String>,
    pub reason_code: String,
    pub components: ClientComponentInventoryV1,
    pub paths: Vec<ClientPathCandidateV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClientInventorySummaryV1 {
    pub schema_version: u32,
    pub clients: [ClientInventoryEntryV1; 2],
}

#[derive(Clone)]
pub struct ClientInventory {
    summary: ClientInventorySummaryV1,
    private_paths: Vec<(ClientPathId, PathBuf)>,
}

impl ClientInventory {
    #[must_use]
    pub const fn summary(&self) -> &ClientInventorySummaryV1 {
        &self.summary
    }

    #[must_use]
    pub fn exact_path(&self, id: ClientPathId) -> Option<&Path> {
        self.private_paths
            .iter()
            .find_map(|(candidate, path)| (*candidate == id).then_some(path.as_path()))
    }
}

impl Debug for ClientInventory {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientInventory")
            .field("summary", &self.summary)
            .field("private_path_count", &self.private_paths.len())
            .finish_non_exhaustive()
    }
}

#[must_use]
pub fn discover_client_inventory(input: ClientInventoryInput<'_>) -> ClientInventory {
    let mut private_paths = Vec::new();
    let codex = discover_codex_inventory(input, &mut private_paths);
    let claude = discover_claude_inventory(input, &mut private_paths);
    ClientInventory {
        summary: ClientInventorySummaryV1 {
            schema_version: CLIENT_INVENTORY_SCHEMA_VERSION,
            clients: [codex, claude],
        },
        private_paths,
    }
}

fn discover_codex_inventory(
    input: ClientInventoryInput<'_>,
    private_paths: &mut Vec<(ClientPathId, PathBuf)>,
) -> ClientInventoryEntryV1 {
    let config_root = input
        .codex_config_root
        .map_or_else(|| input.home.join(".codex"), Path::to_path_buf);
    let mut paths = vec![candidate(
        private_paths,
        ClientPathId::CodexConfig,
        config_root.clone(),
        ClientPathSurface::ClientConfig,
        ClientPathScope::User,
        if input.codex_config_root.is_some() {
            ClientPathSource::EnvironmentOverride
        } else {
            ClientPathSource::OfficialDefault
        },
        if input.codex_config_root.is_some() {
            ClientSymbolicPath::CodexConfigOverride
        } else {
            ClientSymbolicPath::CodexConfig
        },
        ExpectedPathKind::Directory,
        true,
        false,
    )];
    paths.push(candidate(
        private_paths,
        ClientPathId::CodexUserSkills,
        input.home.join(".agents/skills"),
        ClientPathSurface::SkillsRoot,
        ClientPathScope::User,
        ClientPathSource::OfficialDefault,
        ClientSymbolicPath::CodexUserSkills,
        ExpectedPathKind::Directory,
        true,
        false,
    ));
    if let Some(project_root) = input.project_root {
        paths.push(candidate(
            private_paths,
            ClientPathId::CodexProjectSkills,
            project_root.join(".agents/skills"),
            ClientPathSurface::SkillsRoot,
            ClientPathScope::Project,
            ClientPathSource::ProjectContext,
            ClientSymbolicPath::CodexProjectSkills,
            ExpectedPathKind::Directory,
            true,
            false,
        ));
    }
    if let Some(custom_root) = input.codex_custom_skills_root {
        paths.push(candidate(
            private_paths,
            ClientPathId::CodexCustomSkills,
            custom_root.to_path_buf(),
            ClientPathSurface::SkillsRoot,
            ClientPathScope::Custom,
            ClientPathSource::ExplicitCustom,
            ClientSymbolicPath::CodexCustomSkills,
            ExpectedPathKind::Directory,
            true,
            false,
        ));
    }
    paths.push(candidate(
        private_paths,
        ClientPathId::CodexMarketplace,
        input.home.join(".agents/plugins/marketplace.json"),
        ClientPathSurface::PluginMarketplace,
        ClientPathScope::User,
        ClientPathSource::OfficialDefault,
        ClientSymbolicPath::CodexMarketplace,
        ExpectedPathKind::File,
        true,
        false,
    ));
    paths.push(candidate(
        private_paths,
        ClientPathId::CodexPluginSource,
        input.home.join(".qiongli/plugins/codex/qiongli-next"),
        ClientPathSurface::PluginSource,
        ClientPathScope::Managed,
        ClientPathSource::QiongliManaged,
        ClientSymbolicPath::CodexPluginSource,
        ExpectedPathKind::Directory,
        true,
        false,
    ));
    paths.push(candidate(
        private_paths,
        ClientPathId::CodexLegacyPluginSource,
        input.home.join(".qiongli/plugins/codex/qiongli"),
        ClientPathSurface::PluginSource,
        ClientPathScope::Legacy,
        ClientPathSource::LegacyObserved,
        ClientSymbolicPath::CodexLegacyPluginSource,
        ExpectedPathKind::Directory,
        false,
        true,
    ));
    paths.push(candidate(
        private_paths,
        ClientPathId::CodexLegacySkills,
        config_root.join("skills/qiongli-workflow"),
        ClientPathSurface::SkillsPackage,
        ClientPathScope::Legacy,
        ClientPathSource::LegacyObserved,
        ClientSymbolicPath::CodexLegacySkills,
        ExpectedPathKind::Directory,
        false,
        true,
    ));

    let (components, installed_plugin_version, adapter_reason) =
        match discover_codex_user(input.home) {
            Ok(target) => {
                let summary = target.summary();
                (
                    ClientComponentInventoryV1 {
                        skills: component_from_path(&paths, ClientPathId::CodexLegacySkills),
                        plugin_source: match summary.source {
                            CodexSourceState::Missing => ClientComponentState::Missing,
                            CodexSourceState::Ready => ClientComponentState::Ready,
                        },
                        full_mcp: match summary.source {
                            CodexSourceState::Missing => ClientComponentState::Missing,
                            CodexSourceState::Ready => ClientComponentState::Ready,
                        },
                        marketplace: match summary.marketplace {
                            CodexMarketplaceState::Missing => ClientComponentState::Missing,
                            CodexMarketplaceState::Ready => ClientComponentState::Ready,
                        },
                        registration: codex_registration(summary.registration),
                    },
                    target
                        .registration_state()
                        .and_then(|state| state.active.as_ref())
                        .map(|receipt| receipt.artifact.version.clone()),
                    None,
                )
            }
            Err(error) => (
                fallback_components(
                    &paths,
                    ClientPathId::CodexLegacySkills,
                    ClientPathId::CodexPluginSource,
                    ClientPathId::CodexMarketplace,
                ),
                None,
                Some(error.reason_code()),
            ),
        };
    finish_entry(
        ClientKind::Codex,
        input.codex_host_present,
        paths,
        components,
        installed_plugin_version,
        adapter_reason,
    )
}

fn discover_claude_inventory(
    input: ClientInventoryInput<'_>,
    private_paths: &mut Vec<(ClientPathId, PathBuf)>,
) -> ClientInventoryEntryV1 {
    let config_root = input
        .claude_config_root
        .map_or_else(|| input.home.join(".claude"), Path::to_path_buf);
    let mut paths = vec![candidate(
        private_paths,
        ClientPathId::ClaudeConfig,
        config_root.clone(),
        ClientPathSurface::ClientConfig,
        ClientPathScope::User,
        if input.claude_config_root.is_some() {
            ClientPathSource::EnvironmentOverride
        } else {
            ClientPathSource::OfficialDefault
        },
        if input.claude_config_root.is_some() {
            ClientSymbolicPath::ClaudeConfigOverride
        } else {
            ClientSymbolicPath::ClaudeConfig
        },
        ExpectedPathKind::Directory,
        true,
        false,
    )];
    paths.push(candidate(
        private_paths,
        ClientPathId::ClaudeUserSkills,
        config_root.join("skills"),
        ClientPathSurface::SkillsRoot,
        ClientPathScope::User,
        if input.claude_config_root.is_some() {
            ClientPathSource::EnvironmentOverride
        } else {
            ClientPathSource::OfficialDefault
        },
        ClientSymbolicPath::ClaudeUserSkills,
        ExpectedPathKind::Directory,
        true,
        false,
    ));
    if let Some(project_root) = input.project_root {
        paths.push(candidate(
            private_paths,
            ClientPathId::ClaudeProjectSkills,
            project_root.join(".claude/skills"),
            ClientPathSurface::SkillsRoot,
            ClientPathScope::Project,
            ClientPathSource::ProjectContext,
            ClientSymbolicPath::ClaudeProjectSkills,
            ExpectedPathKind::Directory,
            true,
            false,
        ));
    }
    if let Some(custom_root) = input.claude_custom_skills_root {
        paths.push(candidate(
            private_paths,
            ClientPathId::ClaudeCustomSkills,
            custom_root.to_path_buf(),
            ClientPathSurface::SkillsRoot,
            ClientPathScope::Custom,
            ClientPathSource::ExplicitCustom,
            ClientSymbolicPath::ClaudeCustomSkills,
            ExpectedPathKind::Directory,
            true,
            false,
        ));
    }
    paths.push(candidate(
        private_paths,
        ClientPathId::ClaudeMarketplace,
        input
            .home
            .join(".qiongli/plugins/claude-code/qiongli-local/.claude-plugin/marketplace.json"),
        ClientPathSurface::PluginMarketplace,
        ClientPathScope::Managed,
        ClientPathSource::QiongliManaged,
        ClientSymbolicPath::ClaudeMarketplace,
        ExpectedPathKind::File,
        true,
        false,
    ));
    paths.push(candidate(
        private_paths,
        ClientPathId::ClaudePluginSource,
        input
            .home
            .join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli-next"),
        ClientPathSurface::PluginSource,
        ClientPathScope::Managed,
        ClientPathSource::QiongliManaged,
        ClientSymbolicPath::ClaudePluginSource,
        ExpectedPathKind::Directory,
        true,
        false,
    ));
    paths.push(candidate(
        private_paths,
        ClientPathId::ClaudeLegacyPluginSource,
        input
            .home
            .join(".qiongli/plugins/claude-code/qiongli-local/plugins/qiongli"),
        ClientPathSurface::PluginSource,
        ClientPathScope::Legacy,
        ClientPathSource::LegacyObserved,
        ClientSymbolicPath::ClaudeLegacyPluginSource,
        ExpectedPathKind::Directory,
        false,
        true,
    ));
    paths.push(candidate(
        private_paths,
        ClientPathId::ClaudeDirectSkills,
        config_root.join("skills/qiongli-next"),
        ClientPathSurface::SkillsPackage,
        ClientPathScope::User,
        ClientPathSource::QiongliManaged,
        ClientSymbolicPath::ClaudeDirectSkills,
        ExpectedPathKind::Directory,
        true,
        false,
    ));
    paths.push(candidate(
        private_paths,
        ClientPathId::ClaudeLegacySkills,
        config_root.join("skills/qiongli-workflow"),
        ClientPathSurface::SkillsPackage,
        ClientPathScope::Legacy,
        ClientPathSource::LegacyObserved,
        ClientSymbolicPath::ClaudeLegacySkills,
        ExpectedPathKind::Directory,
        false,
        true,
    ));

    let (components, installed_plugin_version, adapter_reason) =
        match discover_claude_user_with_config(input.home, &config_root) {
            Ok(target) => {
                let summary = target.summary();
                (
                    ClientComponentInventoryV1 {
                        skills: match summary.skills_plugin {
                            ClaudeSkillsPluginState::Missing => {
                                component_from_path(&paths, ClientPathId::ClaudeLegacySkills)
                            }
                            ClaudeSkillsPluginState::Ready => ClientComponentState::Ready,
                            ClaudeSkillsPluginState::Conflict => ClientComponentState::Conflict,
                        },
                        plugin_source: match summary.source {
                            ClaudeSourceState::Missing => ClientComponentState::Missing,
                            ClaudeSourceState::Ready => ClientComponentState::Ready,
                        },
                        full_mcp: match summary.source {
                            ClaudeSourceState::Missing => ClientComponentState::Missing,
                            ClaudeSourceState::Ready => ClientComponentState::Ready,
                        },
                        marketplace: match summary.marketplace {
                            ClaudeMarketplaceState::Missing => ClientComponentState::Missing,
                            ClaudeMarketplaceState::Ready => ClientComponentState::Ready,
                        },
                        registration: claude_registration(summary.registration),
                    },
                    target
                        .registration_state()
                        .and_then(|state| state.active.as_ref())
                        .map(|receipt| receipt.artifact.version.clone()),
                    None,
                )
            }
            Err(error) => (
                fallback_components(
                    &paths,
                    ClientPathId::ClaudeDirectSkills,
                    ClientPathId::ClaudePluginSource,
                    ClientPathId::ClaudeMarketplace,
                ),
                None,
                Some(error.reason_code()),
            ),
        };
    finish_entry(
        ClientKind::ClaudeCode,
        input.claude_host_present,
        paths,
        components,
        installed_plugin_version,
        adapter_reason,
    )
}

fn finish_entry(
    client: ClientKind,
    host_present: bool,
    paths: Vec<ClientPathCandidateV1>,
    components: ClientComponentInventoryV1,
    installed_plugin_version: Option<String>,
    adapter_reason: Option<&'static str>,
) -> ClientInventoryEntryV1 {
    let config_unsafe = paths.iter().any(|path| {
        path.surface == ClientPathSurface::ClientConfig
            && matches!(
                path.state,
                ClientPathState::Symlink
                    | ClientPathState::Invalid
                    | ClientPathState::Unsafe
                    | ClientPathState::Unavailable
            )
    });
    let observed = host_present || paths.iter().any(|path| path.state.is_observed());
    let discovery = if config_unsafe || adapter_reason.is_some() {
        ClientDiscoveryState::Unavailable
    } else if observed {
        ClientDiscoveryState::Detected
    } else {
        ClientDiscoveryState::NotDetected
    };
    let legacy_observed = paths.iter().any(|path| {
        path.management == ClientPathManagement::LegacyOnly && path.state.is_observed()
    });
    let ownership = match (ownership(components), legacy_observed) {
        (ClientOwnershipState::NotInstalled, true) => ClientOwnershipState::Unmanaged,
        (ClientOwnershipState::QiongliManaged, true) => ClientOwnershipState::Mixed,
        (ownership, _) => ownership,
    };
    let readiness = if discovery == ClientDiscoveryState::Unavailable {
        ClientActionReadiness::Unavailable
    } else {
        match components.registration {
            ClientComponentState::Ready => ClientActionReadiness::Current,
            ClientComponentState::Drifted | ClientComponentState::RecoveryRequired => {
                ClientActionReadiness::RepairReady
            }
            ClientComponentState::Conflict => ClientActionReadiness::ResolveConflict,
            ClientComponentState::Unavailable => ClientActionReadiness::Unavailable,
            ClientComponentState::Missing if discovery == ClientDiscoveryState::Detected => {
                ClientActionReadiness::InstallReady
            }
            ClientComponentState::Missing => ClientActionReadiness::InspectOnly,
        }
    };
    let reason_code = adapter_reason.unwrap_or_else(|| readiness_reason(readiness));
    ClientInventoryEntryV1 {
        client,
        discovery,
        ownership,
        readiness,
        host_presence: if host_present {
            ClientHostPresence::Observed
        } else {
            ClientHostPresence::NotObserved
        },
        installed_plugin_version,
        reason_code: reason_code.to_owned(),
        components,
        paths,
    }
}

const fn ownership(components: ClientComponentInventoryV1) -> ClientOwnershipState {
    match components.registration {
        ClientComponentState::Ready
        | ClientComponentState::Drifted
        | ClientComponentState::RecoveryRequired => ClientOwnershipState::QiongliManaged,
        ClientComponentState::Conflict => {
            if matches!(components.plugin_source, ClientComponentState::Ready) {
                ClientOwnershipState::Mixed
            } else {
                ClientOwnershipState::Unmanaged
            }
        }
        ClientComponentState::Unavailable => ClientOwnershipState::Unknown,
        ClientComponentState::Missing => {
            if matches!(
                components.skills,
                ClientComponentState::Ready | ClientComponentState::Conflict
            ) || matches!(components.plugin_source, ClientComponentState::Ready)
                || matches!(components.marketplace, ClientComponentState::Ready)
            {
                ClientOwnershipState::Unmanaged
            } else {
                ClientOwnershipState::NotInstalled
            }
        }
    }
}

const fn readiness_reason(readiness: ClientActionReadiness) -> &'static str {
    match readiness {
        ClientActionReadiness::InspectOnly => "client-not-detected",
        ClientActionReadiness::InstallReady => "client-detected-install-ready",
        ClientActionReadiness::Current => "client-managed-current",
        ClientActionReadiness::RepairReady => "client-managed-repair-ready",
        ClientActionReadiness::ResolveConflict => "client-registration-conflict",
        ClientActionReadiness::Unavailable => "client-inventory-unavailable",
    }
}

const fn codex_registration(state: CodexRegistrationState) -> ClientComponentState {
    match state {
        CodexRegistrationState::Absent => ClientComponentState::Missing,
        CodexRegistrationState::Registered => ClientComponentState::Ready,
        CodexRegistrationState::Conflict => ClientComponentState::Conflict,
        CodexRegistrationState::Drifted => ClientComponentState::Drifted,
        CodexRegistrationState::RecoveryRequired => ClientComponentState::RecoveryRequired,
    }
}

const fn claude_registration(state: ClaudeRegistrationState) -> ClientComponentState {
    match state {
        ClaudeRegistrationState::Absent => ClientComponentState::Missing,
        ClaudeRegistrationState::Registered => ClientComponentState::Ready,
        ClaudeRegistrationState::Conflict => ClientComponentState::Conflict,
        ClaudeRegistrationState::Drifted => ClientComponentState::Drifted,
        ClaudeRegistrationState::RecoveryRequired => ClientComponentState::RecoveryRequired,
    }
}

fn fallback_components(
    paths: &[ClientPathCandidateV1],
    skills: ClientPathId,
    source: ClientPathId,
    marketplace: ClientPathId,
) -> ClientComponentInventoryV1 {
    ClientComponentInventoryV1 {
        skills: component_from_path(paths, skills),
        plugin_source: component_from_path(paths, source),
        full_mcp: ClientComponentState::Unavailable,
        marketplace: component_from_path(paths, marketplace),
        registration: ClientComponentState::Unavailable,
    }
}

fn component_from_path(paths: &[ClientPathCandidateV1], id: ClientPathId) -> ClientComponentState {
    paths
        .iter()
        .find(|path| path.id == id)
        .map_or(ClientComponentState::Unavailable, |path| match path.state {
            ClientPathState::Missing => ClientComponentState::Missing,
            ClientPathState::Directory | ClientPathState::File | ClientPathState::Symlink => {
                ClientComponentState::Ready
            }
            ClientPathState::Invalid | ClientPathState::Unsafe | ClientPathState::Unavailable => {
                ClientComponentState::Unavailable
            }
        })
}

#[derive(Clone, Copy)]
enum ExpectedPathKind {
    Directory,
    File,
}

#[allow(clippy::too_many_arguments)]
fn candidate(
    private_paths: &mut Vec<(ClientPathId, PathBuf)>,
    id: ClientPathId,
    path: PathBuf,
    surface: ClientPathSurface,
    scope: ClientPathScope,
    source: ClientPathSource,
    symbolic_path: ClientSymbolicPath,
    expected: ExpectedPathKind,
    selected: bool,
    legacy: bool,
) -> ClientPathCandidateV1 {
    let state = inspect_path(&path, expected);
    private_paths.push((id, path));
    ClientPathCandidateV1 {
        id,
        surface,
        scope,
        source,
        state,
        management: path_management(state, surface, legacy),
        selected,
        symbolic_path,
    }
}

fn inspect_path(path: &Path, expected: ExpectedPathKind) -> ClientPathState {
    if !path.is_absolute() || has_lexical_traversal(path) {
        return ClientPathState::Unsafe;
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || is_reparse_point(&metadata) => {
            ClientPathState::Symlink
        }
        Ok(metadata) => match expected {
            ExpectedPathKind::Directory if metadata.is_dir() => ClientPathState::Directory,
            ExpectedPathKind::File if metadata.is_file() => ClientPathState::File,
            ExpectedPathKind::Directory | ExpectedPathKind::File => ClientPathState::Invalid,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ClientPathState::Missing,
        Err(_) => ClientPathState::Unavailable,
    }
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
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    attributes_are_reparse_point(metadata.file_attributes())
}

#[cfg(windows)]
const fn attributes_are_reparse_point(attributes: u32) -> bool {
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
const fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
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
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::CurDir | std::path::Component::ParentDir
        )
    })
}

const fn path_management(
    state: ClientPathState,
    surface: ClientPathSurface,
    legacy: bool,
) -> ClientPathManagement {
    if legacy {
        return ClientPathManagement::LegacyOnly;
    }
    match state {
        ClientPathState::Unsafe | ClientPathState::Invalid => ClientPathManagement::Unsafe,
        ClientPathState::Unavailable => ClientPathManagement::Unavailable,
        ClientPathState::Symlink if matches!(surface, ClientPathSurface::ClientConfig) => {
            ClientPathManagement::InspectOnly
        }
        ClientPathState::Missing
        | ClientPathState::Directory
        | ClientPathState::File
        | ClientPathState::Symlink => ClientPathManagement::Supported,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        home: PathBuf,
        project: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/qiongli-client-inventory-tests")
                .join(format!(
                    "{name}-{}-{}",
                    std::process::id(),
                    NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
                ));
            let requested_home = root.join("home");
            let requested_project = root.join("project");
            fs::create_dir_all(&requested_home).expect("fixture home must exist");
            fs::create_dir_all(&requested_project).expect("fixture project must exist");
            let root = fs::canonicalize(root).expect("fixture root must canonicalize");
            let home = fs::canonicalize(requested_home).expect("fixture home must canonicalize");
            let project =
                fs::canonicalize(requested_project).expect("fixture project must canonicalize");
            make_adapter_safe(&home);
            Self {
                root,
                home,
                project,
            }
        }

        fn inventory(&self) -> ClientInventory {
            discover_client_inventory(
                ClientInventoryInput::new(&self.home).with_project_root(Some(&self.project)),
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("fixture cleanup must succeed");
        }
    }

    #[test]
    fn missing_clients_are_inspect_only_and_discovery_does_not_write() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/qiongli-client-inventory-tests")
            .join(format!(
                "missing-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
        let requested_home = root.join("home");
        fs::create_dir_all(&requested_home).expect("fixture home must exist");
        let home = fs::canonicalize(requested_home).expect("fixture home must canonicalize");
        make_adapter_safe(&home);
        let children_before = fs::read_dir(&home)
            .expect("fixture home must be readable")
            .count();
        let inventory = discover_client_inventory(ClientInventoryInput::new(&home));

        assert_eq!(
            fs::read_dir(&home)
                .expect("fixture home must remain readable")
                .count(),
            children_before
        );
        for client in &inventory.summary().clients {
            assert_eq!(client.discovery, ClientDiscoveryState::NotDetected);
            assert_eq!(client.readiness, ClientActionReadiness::InspectOnly);
            assert_eq!(client.ownership, ClientOwnershipState::NotInstalled);
        }
        fs::remove_dir_all(root).expect("fixture cleanup must succeed");
    }

    #[test]
    fn config_and_host_presence_are_independent_discovery_signals() {
        let fixture = Fixture::new("multi-signal");
        fs::create_dir_all(fixture.home.join(".codex")).expect("Codex config fixture must exist");
        let inventory = discover_client_inventory(
            ClientInventoryInput::new(&fixture.home).with_host_presence(false, true),
        );

        assert_eq!(
            inventory.summary().clients[0].discovery,
            ClientDiscoveryState::Detected,
            "{:?}",
            inventory.summary().clients[0]
        );
        assert_eq!(
            inventory.summary().clients[1].host_presence,
            ClientHostPresence::Observed
        );
        assert_eq!(
            inventory.summary().clients[1].discovery,
            ClientDiscoveryState::Detected
        );
    }

    #[test]
    fn official_user_project_and_override_paths_are_symbolic_and_private() {
        let fixture = Fixture::new("paths");
        let codex_override = fixture.root.join("codex-config");
        let claude_override = fixture.root.join("claude-config");
        let codex_custom = fixture.root.join("custom-codex-skills");
        let claude_custom = fixture.root.join("custom-claude-skills");
        let inventory = discover_client_inventory(
            ClientInventoryInput::new(&fixture.home)
                .with_codex_config_root(Some(&codex_override))
                .with_claude_config_root(Some(&claude_override))
                .with_project_root(Some(&fixture.project))
                .with_custom_skills_roots(Some(&codex_custom), Some(&claude_custom)),
        );
        let summary_json =
            serde_json::to_string(inventory.summary()).expect("inventory summary must serialize");

        assert!(summary_json.contains("codex-config-override"));
        assert!(summary_json.contains("codex-project-skills"));
        assert!(summary_json.contains("claude-project-skills"));
        assert!(summary_json.contains("codex-custom-skills"));
        assert!(summary_json.contains("claude-custom-skills"));
        assert!(!summary_json.contains(fixture.root.to_string_lossy().as_ref()));
        assert_eq!(
            inventory.exact_path(ClientPathId::CodexConfig),
            Some(codex_override.as_path())
        );
    }

    #[test]
    fn legacy_install_is_detected_but_never_selected_for_replacement() {
        let fixture = Fixture::new("legacy");
        let legacy = fixture.home.join(".codex/skills/qiongli-workflow");
        fs::create_dir_all(&legacy).expect("legacy fixture must exist");
        let inventory = fixture.inventory();
        let codex = &inventory.summary().clients[0];
        let legacy = codex
            .paths
            .iter()
            .find(|candidate| candidate.id == ClientPathId::CodexLegacySkills)
            .expect("legacy candidate must be reported");

        assert_eq!(codex.discovery, ClientDiscoveryState::Detected, "{codex:?}");
        assert_eq!(codex.ownership, ClientOwnershipState::Unmanaged);
        assert_eq!(legacy.management, ClientPathManagement::LegacyOnly);
        assert!(!legacy.selected);
    }

    #[test]
    fn relative_override_is_unsafe_and_does_not_leak_the_path() {
        let fixture = Fixture::new("unsafe");
        let relative = Path::new("relative-codex-home");
        let inventory = discover_client_inventory(
            ClientInventoryInput::new(&fixture.home).with_codex_config_root(Some(relative)),
        );
        let codex = &inventory.summary().clients[0];

        assert_eq!(codex.discovery, ClientDiscoveryState::Unavailable);
        assert_eq!(codex.readiness, ClientActionReadiness::Unavailable);
        assert_eq!(codex.paths[0].state, ClientPathState::Unsafe);
        assert!(!format!("{inventory:?}").contains("relative-codex-home"));

        let mut traversal = fixture.root.as_os_str().to_os_string();
        for component in ["safe", "..", "unsafe-codex-home"] {
            traversal.push(std::path::MAIN_SEPARATOR_STR);
            traversal.push(component);
        }
        let traversal = PathBuf::from(traversal);
        assert!(has_lexical_traversal(&traversal));
        let inventory = discover_client_inventory(
            ClientInventoryInput::new(&fixture.home).with_codex_config_root(Some(&traversal)),
        );
        assert_eq!(
            inventory.summary().clients[0].paths[0].state,
            ClientPathState::Unsafe
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_reparse_attribute_is_classified_without_following_the_target() {
        assert!(attributes_are_reparse_point(0x0400));
        assert!(attributes_are_reparse_point(0x0400 | 0x0010));
        assert!(!attributes_are_reparse_point(0x0010));
    }

    #[test]
    fn managed_drift_recovery_and_conflict_have_distinct_next_actions() {
        let missing_paths = Vec::new();
        let components = |registration, plugin_source| ClientComponentInventoryV1 {
            skills: ClientComponentState::Missing,
            plugin_source,
            full_mcp: ClientComponentState::Ready,
            marketplace: ClientComponentState::Ready,
            registration,
        };
        let current = finish_entry(
            ClientKind::Codex,
            true,
            missing_paths.clone(),
            components(ClientComponentState::Ready, ClientComponentState::Ready),
            None,
            None,
        );
        let drifted = finish_entry(
            ClientKind::Codex,
            true,
            missing_paths.clone(),
            components(ClientComponentState::Drifted, ClientComponentState::Ready),
            None,
            None,
        );
        let recovery = finish_entry(
            ClientKind::Codex,
            true,
            missing_paths.clone(),
            components(
                ClientComponentState::RecoveryRequired,
                ClientComponentState::Ready,
            ),
            None,
            None,
        );
        let conflict = finish_entry(
            ClientKind::Codex,
            true,
            missing_paths,
            components(ClientComponentState::Conflict, ClientComponentState::Ready),
            None,
            None,
        );

        assert_eq!(current.readiness, ClientActionReadiness::Current);
        assert_eq!(current.ownership, ClientOwnershipState::QiongliManaged);
        assert_eq!(drifted.readiness, ClientActionReadiness::RepairReady);
        assert_eq!(recovery.readiness, ClientActionReadiness::RepairReady);
        assert_eq!(conflict.readiness, ClientActionReadiness::ResolveConflict);
        assert_eq!(conflict.ownership, ClientOwnershipState::Mixed);
    }

    #[cfg(unix)]
    #[test]
    fn config_symlink_is_reported_without_following_it() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new("symlink");
        let real = fixture.root.join("real-codex");
        let link = fixture.root.join("codex-link");
        fs::create_dir(&real).expect("real config must exist");
        symlink(&real, &link).expect("config symlink must exist");
        let inventory = discover_client_inventory(
            ClientInventoryInput::new(&fixture.home).with_codex_config_root(Some(&link)),
        );
        let codex = &inventory.summary().clients[0];

        assert_eq!(codex.paths[0].state, ClientPathState::Symlink);
        assert_eq!(codex.discovery, ClientDiscoveryState::Unavailable);
        assert_eq!(codex.readiness, ClientActionReadiness::Unavailable);
    }

    #[cfg(unix)]
    fn make_adapter_safe(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .expect("fixture permissions must be adapter-safe");
    }

    #[cfg(not(unix))]
    fn make_adapter_safe(_path: &Path) {}
}
