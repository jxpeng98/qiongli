use std::fs;
use std::path::{Path, PathBuf};

use qiongli_config::{
    ConfigRootSource, ConfigState, GLOBAL_SETTINGS_FILE, GlobalSettingsStore, ProviderReadiness,
    SecretStoreStatus, UPDATE_STATE_FILE, UpdateStateStore, UpdateStreamPreference,
};
use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    ClientActionReadiness, ClientDiscoveryState, ClientInventory, ClientInventoryEntryV1,
    ClientKind, ClientPathCandidateV1, ClientPathId, ClientPathManagement, ClientPathScope,
    ClientPathSource, ClientPathState,
};
use qiongli_runtime::LITE_PUBLIC_TOOL_NAMES;
use serde::Serialize;

use crate::command::{CommandEnvironment, config_root};
use crate::managed_content::{load_managed_content_registry, managed_content_registry_path};

pub(crate) const PRODUCT_INSPECTION_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductDoctorCheckId {
    EmbeddedContent,
    GlobalConfig,
    SecureStore,
    ManagedContent,
    CodexLocal,
    ClaudeCodeLocal,
    LiteMcp,
    LiteratureProviders,
    UpdateRecovery,
    FullRuntime,
}

impl ProductDoctorCheckId {
    pub(crate) const ALL: [Self; 10] = [
        Self::EmbeddedContent,
        Self::GlobalConfig,
        Self::SecureStore,
        Self::ManagedContent,
        Self::CodexLocal,
        Self::ClaudeCodeLocal,
        Self::LiteMcp,
        Self::LiteratureProviders,
        Self::UpdateRecovery,
        Self::FullRuntime,
    ];

    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::EmbeddedContent => "embedded-content",
            Self::GlobalConfig => "global-config",
            Self::SecureStore => "secure-store",
            Self::ManagedContent => "managed-content",
            Self::CodexLocal => "codex-local",
            Self::ClaudeCodeLocal => "claude-code-local",
            Self::LiteMcp => "lite-mcp",
            Self::LiteratureProviders => "literature-providers",
            Self::UpdateRecovery => "update-recovery",
            Self::FullRuntime => "full-runtime",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductDoctorStatus {
    Ready,
    Attention,
    Missing,
    Unavailable,
    Invalid,
    FutureSchema,
    Insecure,
    Busy,
    WriteUnsupported,
    RecoveryRequired,
    Deferred,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProductDoctorCheckV1 {
    pub(crate) id: ProductDoctorCheckId,
    #[serde(rename = "state")]
    pub(crate) status: ProductDoctorStatus,
    pub(crate) blocking: bool,
    pub(crate) code: &'static str,
    pub(crate) remediation: &'static str,
    pub(crate) section: &'static str,
    pub(crate) path_id: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductPathGroup {
    Product,
    Configuration,
    Content,
    Update,
    Project,
    Codex,
    ClaudeCode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductPathScope {
    Runtime,
    User,
    Project,
    Managed,
    Custom,
    Legacy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductPathSource {
    ProcessEnvironment,
    OfficialDefault,
    EnvironmentOverride,
    CurrentProject,
    QiongliManaged,
    ClientAdapter,
    Runtime,
    LegacyObserved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductPathFileType {
    Missing,
    Directory,
    File,
    Symlink,
    Other,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductPathOwner {
    #[cfg_attr(
        not(unix),
        expect(
            dead_code,
            reason = "non-Unix targets preserve the shared owner schema but report unknown"
        )
    )]
    CurrentUser,
    #[cfg_attr(
        not(unix),
        expect(
            dead_code,
            reason = "non-Unix targets preserve the shared owner schema but report unknown"
        )
    )]
    OtherUser,
    Missing,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductPathWritability {
    Writable,
    ReadOnly,
    MissingParentWritable,
    MissingParentReadOnly,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ProductPathSafety {
    Supported,
    InspectOnly,
    LegacyOnly,
    Unsafe,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedPathType {
    Directory,
    File,
    Any,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProductPathInspectionV1 {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) group: ProductPathGroup,
    pub(crate) scope: ProductPathScope,
    pub(crate) source: ProductPathSource,
    pub(crate) selected: bool,
    pub(crate) symbolic_path: String,
    pub(crate) exact_path: String,
    pub(crate) exists: bool,
    pub(crate) file_type: ProductPathFileType,
    pub(crate) expected_type: &'static str,
    pub(crate) type_matches_expected: Option<bool>,
    pub(crate) owner: ProductPathOwner,
    pub(crate) writability: ProductPathWritability,
    pub(crate) safety: ProductPathSafety,
    pub(crate) resolved_target: Option<String>,
}

impl std::fmt::Debug for ProductPathInspectionV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductPathInspectionV1")
            .field("id", &self.id)
            .field("group", &self.group)
            .field("scope", &self.scope)
            .field("source", &self.source)
            .field("selected", &self.selected)
            .field("symbolic_path", &self.symbolic_path)
            .field("exists", &self.exists)
            .field("file_type", &self.file_type)
            .field("expected_type", &self.expected_type)
            .field("type_matches_expected", &self.type_matches_expected)
            .field("owner", &self.owner)
            .field("writability", &self.writability)
            .field("safety", &self.safety)
            .field("resolved_target_present", &self.resolved_target.is_some())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProductInspectionSnapshotV1 {
    pub(crate) schema_version: u32,
    pub(crate) product_version: &'static str,
    pub(crate) checks: [ProductDoctorCheckV1; 10],
    pub(crate) paths: Vec<ProductPathInspectionV1>,
}

impl ProductInspectionSnapshotV1 {
    pub(crate) fn blocking(&self) -> bool {
        self.checks.iter().any(|check| check.blocking)
    }

    pub(crate) fn requires_attention(&self) -> bool {
        self.checks.iter().any(|check| {
            !matches!(
                check.status,
                ProductDoctorStatus::Ready
                    | ProductDoctorStatus::Missing
                    | ProductDoctorStatus::Deferred
            )
        })
    }
}

pub(crate) fn inspect_product(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    secret_store_status: SecretStoreStatus,
) -> ProductInspectionSnapshotV1 {
    let inventory = environment.client_inventory();
    let root = config_root(environment).ok();
    let mut paths = Vec::new();

    if let Some(home) = environment.platform_home() {
        push_path(
            &mut paths,
            PathDescriptor::new(
                "user-home",
                "User home",
                ProductPathGroup::Product,
                ProductPathScope::User,
                ProductPathSource::ProcessEnvironment,
                "<user-home>",
                home,
                ExpectedPathType::Directory,
            ),
        );
        push_path(
            &mut paths,
            PathDescriptor::new(
                "product-home",
                "Qiongli product home",
                ProductPathGroup::Product,
                ProductPathScope::Managed,
                ProductPathSource::QiongliManaged,
                "<user-home>/.qiongli",
                &home.join(".qiongli"),
                ExpectedPathType::Directory,
            ),
        );
        push_path(
            &mut paths,
            PathDescriptor::new(
                "codex-registration-receipt",
                "Codex registration receipt",
                ProductPathGroup::Codex,
                ProductPathScope::Managed,
                ProductPathSource::ClientAdapter,
                "<user-home>/.qiongli/plugins/codex/.qiongli-next-codex-registration.json",
                &home
                    .join(".qiongli/plugins/codex")
                    .join(".qiongli-next-codex-registration.json"),
                ExpectedPathType::File,
            ),
        );
        push_path(
            &mut paths,
            PathDescriptor::new(
                "claude-code-registration-receipt",
                "Claude Code registration receipt",
                ProductPathGroup::ClaudeCode,
                ProductPathScope::Managed,
                ProductPathSource::ClientAdapter,
                "<user-home>/.qiongli/plugins/claude-code/.qiongli-next-claude-registration.json",
                &home
                    .join(".qiongli/plugins/claude-code")
                    .join(".qiongli-next-claude-registration.json"),
                ExpectedPathType::File,
            ),
        );
    }

    if let Ok(executable) = std::env::current_exe() {
        push_path(
            &mut paths,
            PathDescriptor::new(
                "current-executable",
                "Current Qiongli executable",
                ProductPathGroup::Product,
                ProductPathScope::Runtime,
                ProductPathSource::Runtime,
                "<current-executable>",
                &executable,
                ExpectedPathType::File,
            ),
        );
        let manifest = if cfg!(target_os = "macos") {
            executable
                .parent()
                .and_then(Path::parent)
                .unwrap_or_else(|| executable.parent().unwrap_or(&executable))
                .join("Resources")
                .join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
        } else {
            executable
                .parent()
                .unwrap_or(&executable)
                .join(qiongli_platform::DESKTOP_PACKAGE_MANIFEST_FILE)
        };
        push_path(
            &mut paths,
            PathDescriptor::new(
                "desktop-package-manifest",
                "Desktop package manifest",
                ProductPathGroup::Product,
                ProductPathScope::Runtime,
                ProductPathSource::Runtime,
                "<application-resources>/.qiongli-desktop-package.json",
                &manifest,
                ExpectedPathType::File,
            ),
        );
    }

    if let Some(root) = root.as_ref() {
        let source = match root.source() {
            ConfigRootSource::Default => ProductPathSource::OfficialDefault,
            ConfigRootSource::Override => ProductPathSource::EnvironmentOverride,
        };
        push_path(
            &mut paths,
            PathDescriptor::new(
                "config-root",
                "Qiongli 2 configuration root",
                ProductPathGroup::Configuration,
                ProductPathScope::Managed,
                source,
                root.symbolic_state_root(),
                root.state_root(),
                ExpectedPathType::Directory,
            ),
        );
        push_path(
            &mut paths,
            PathDescriptor::new(
                "global-config",
                "Global settings",
                ProductPathGroup::Configuration,
                ProductPathScope::Managed,
                source,
                "<qiongli-config-root>/settings.json",
                &root.state_root().join(GLOBAL_SETTINGS_FILE),
                ExpectedPathType::File,
            ),
        );
        push_path(
            &mut paths,
            PathDescriptor::new(
                "managed-content-receipts",
                "Managed content receipts",
                ProductPathGroup::Content,
                ProductPathScope::Managed,
                ProductPathSource::QiongliManaged,
                "<qiongli-config-root>/managed-content.json",
                &managed_content_registry_path(root.state_root()),
                ExpectedPathType::File,
            ),
        );
        push_path(
            &mut paths,
            PathDescriptor::new(
                "update-state",
                "Update state",
                ProductPathGroup::Update,
                ProductPathScope::Managed,
                ProductPathSource::QiongliManaged,
                "<qiongli-config-root>/update-state.json",
                &root.state_root().join(UPDATE_STATE_FILE),
                ExpectedPathType::File,
            ),
        );
        push_path(
            &mut paths,
            PathDescriptor::new(
                "update-staging",
                "Update staging root",
                ProductPathGroup::Update,
                ProductPathScope::Managed,
                ProductPathSource::QiongliManaged,
                "<qiongli-config-root>/updates/staging",
                &root.state_root().join("updates/staging"),
                ExpectedPathType::Directory,
            ),
        );
        if let Ok(registry) = load_managed_content_registry(root.state_root()) {
            for (index, entry) in registry.entries.iter().enumerate() {
                let target = PathBuf::from(&entry.target);
                push_path(
                    &mut paths,
                    PathDescriptor::new(
                        format!("managed-content-target-{index}"),
                        format!("Managed Skills target {}", index + 1),
                        ProductPathGroup::Content,
                        ProductPathScope::Managed,
                        ProductPathSource::QiongliManaged,
                        "<receipt-owned-skills-target>",
                        &target,
                        ExpectedPathType::Directory,
                    ),
                );
            }
        }
    }

    if let Some(project) = environment.project_root() {
        push_path(
            &mut paths,
            PathDescriptor::new(
                "project-root",
                "Current project",
                ProductPathGroup::Project,
                ProductPathScope::Project,
                ProductPathSource::CurrentProject,
                "<project-root>",
                project,
                ExpectedPathType::Directory,
            ),
        );
    }

    if let Some(inventory) = inventory.as_ref() {
        append_client_paths(&mut paths, inventory);
    }

    let checks = build_checks(
        environment,
        content,
        secret_store_status,
        inventory.as_ref(),
        root.as_ref(),
    );
    debug_assert_eq!(
        checks.each_ref().map(|check| check.id),
        ProductDoctorCheckId::ALL
    );
    debug_assert!(checks.iter().all(|check| !check.id.code().is_empty()));
    ProductInspectionSnapshotV1 {
        schema_version: PRODUCT_INSPECTION_SCHEMA_VERSION,
        product_version: env!("CARGO_PKG_VERSION"),
        checks,
        paths,
    }
}

fn build_checks(
    _environment: &CommandEnvironment,
    content: &EmbeddedContent,
    secret_store_status: SecretStoreStatus,
    inventory: Option<&ClientInventory>,
    root: Option<&qiongli_config::ConfigRoot>,
) -> [ProductDoctorCheckV1; 10] {
    let content_ready = !content.pack().manifest().entries.is_empty();
    let config_status = root.map(|root| GlobalSettingsStore::new(root.clone()).status());
    let config = config_status.as_ref().map_or_else(
        || ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::GlobalConfig,
            status: ProductDoctorStatus::Unavailable,
            blocking: true,
            code: "global-config-home-unavailable",
            remediation: "inspect-global-config",
            section: "global-settings",
            path_id: Some("global-config"),
        },
        |status| config_doctor_check(status.state),
    );
    let managed_content = root.map_or_else(
        || ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::ManagedContent,
            status: ProductDoctorStatus::Unavailable,
            blocking: true,
            code: "managed-content-home-unavailable",
            remediation: "inspect-managed-content",
            section: "skills",
            path_id: Some("managed-content-receipts"),
        },
        |root| match load_managed_content_registry(root.state_root()) {
            Ok(_) => ProductDoctorCheckV1 {
                id: ProductDoctorCheckId::ManagedContent,
                status: ProductDoctorStatus::Ready,
                blocking: false,
                code: "managed-content-ready",
                remediation: "none",
                section: "skills",
                path_id: Some("managed-content-receipts"),
            },
            Err(code) => ProductDoctorCheckV1 {
                id: ProductDoctorCheckId::ManagedContent,
                status: ProductDoctorStatus::RecoveryRequired,
                blocking: true,
                code,
                remediation: "inspect-managed-content",
                section: "skills",
                path_id: Some("managed-content-receipts"),
            },
        },
    );
    let update = root.map_or_else(
        || ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::UpdateRecovery,
            status: ProductDoctorStatus::Unavailable,
            blocking: false,
            code: "update-state-home-unavailable",
            remediation: "inspect-update-state",
            section: "about",
            path_id: Some("update-state"),
        },
        |root| {
            let store = UpdateStateStore::new(root.clone(), UpdateStreamPreference::Beta);
            match store.load() {
                Ok(_) => ProductDoctorCheckV1 {
                    id: ProductDoctorCheckId::UpdateRecovery,
                    status: ProductDoctorStatus::Ready,
                    blocking: false,
                    code: "update-state-ready",
                    remediation: "none",
                    section: "about",
                    path_id: Some("update-state"),
                },
                Err(error) => ProductDoctorCheckV1 {
                    id: ProductDoctorCheckId::UpdateRecovery,
                    status: if error == qiongli_config::ConfigError::RecoveryRequired {
                        ProductDoctorStatus::RecoveryRequired
                    } else {
                        ProductDoctorStatus::Attention
                    },
                    blocking: error == qiongli_config::ConfigError::RecoveryRequired,
                    code: error.reason_code(),
                    remediation: "inspect-update-state",
                    section: "about",
                    path_id: Some("update-state"),
                },
            }
        },
    );
    let providers = provider_doctor_check(config_status.as_ref());
    let [codex, claude] = client_doctor_checks(inventory);
    [
        ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::EmbeddedContent,
            status: if content_ready {
                ProductDoctorStatus::Ready
            } else {
                ProductDoctorStatus::RecoveryRequired
            },
            blocking: !content_ready,
            code: if content_ready {
                "embedded-content-ready"
            } else {
                "embedded-content-empty"
            },
            remediation: if content_ready {
                "none"
            } else {
                "reinstall-qiongli"
            },
            section: "about",
            path_id: None,
        },
        config,
        ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::SecureStore,
            status: if secret_store_status == SecretStoreStatus::Available {
                ProductDoctorStatus::Ready
            } else {
                ProductDoctorStatus::Unavailable
            },
            blocking: false,
            code: if secret_store_status == SecretStoreStatus::Available {
                "secure-store-ready"
            } else {
                "secure-store-unavailable"
            },
            remediation: if secret_store_status == SecretStoreStatus::Available {
                "none"
            } else {
                "use-supported-secure-store"
            },
            section: "literature-providers",
            path_id: None,
        },
        managed_content,
        codex,
        claude,
        ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::LiteMcp,
            status: if LITE_PUBLIC_TOOL_NAMES.is_empty() {
                ProductDoctorStatus::RecoveryRequired
            } else {
                ProductDoctorStatus::Ready
            },
            blocking: LITE_PUBLIC_TOOL_NAMES.is_empty(),
            code: if LITE_PUBLIC_TOOL_NAMES.is_empty() {
                "lite-mcp-tool-registry-empty"
            } else {
                "lite-mcp-offline-contract-ready"
            },
            remediation: if LITE_PUBLIC_TOOL_NAMES.is_empty() {
                "reinstall-qiongli"
            } else {
                "none"
            },
            section: "mcp",
            path_id: None,
        },
        providers,
        update,
        ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::FullRuntime,
            status: ProductDoctorStatus::Deferred,
            blocking: false,
            code: "full-runtime-not-available-in-lite",
            remediation: "upgrade-to-r4-full-runtime",
            section: "diagnostics",
            path_id: None,
        },
    ]
}

fn config_doctor_check(state: ConfigState) -> ProductDoctorCheckV1 {
    let (status, blocking, code, remediation) = match state {
        ConfigState::Missing => (
            ProductDoctorStatus::Missing,
            false,
            "global-config-missing",
            "create-global-config",
        ),
        ConfigState::Ready => (
            ProductDoctorStatus::Ready,
            false,
            "global-config-ready",
            "none",
        ),
        ConfigState::Busy => (
            ProductDoctorStatus::Busy,
            false,
            "global-config-busy",
            "retry-global-config",
        ),
        ConfigState::Invalid => (
            ProductDoctorStatus::Invalid,
            true,
            "global-config-invalid",
            "inspect-global-config",
        ),
        ConfigState::FutureSchema => (
            ProductDoctorStatus::FutureSchema,
            true,
            "global-config-future-schema",
            "upgrade-qiongli",
        ),
        ConfigState::Insecure => (
            ProductDoctorStatus::Insecure,
            true,
            "global-config-insecure",
            "repair-global-config-permissions",
        ),
        ConfigState::RecoveryRequired => (
            ProductDoctorStatus::RecoveryRequired,
            true,
            "global-config-recovery-required",
            "recover-global-config",
        ),
        ConfigState::WriteUnsupported => (
            ProductDoctorStatus::WriteUnsupported,
            true,
            "global-config-write-unsupported",
            "use-supported-platform",
        ),
    };
    ProductDoctorCheckV1 {
        id: ProductDoctorCheckId::GlobalConfig,
        status,
        blocking,
        code,
        remediation,
        section: "global-settings",
        path_id: Some("global-config"),
    }
}

fn provider_doctor_check(
    config: Option<&qiongli_config::RedactedConfigStatus>,
) -> ProductDoctorCheckV1 {
    let Some(providers) = config.and_then(|config| config.providers.as_ref()) else {
        return ProductDoctorCheckV1 {
            id: ProductDoctorCheckId::LiteratureProviders,
            status: ProductDoctorStatus::Unavailable,
            blocking: false,
            code: "literature-providers-config-unavailable",
            remediation: "inspect-global-config",
            section: "literature-providers",
            path_id: Some("global-config"),
        };
    };
    let statuses = [
        &providers.openalex,
        &providers.semantic_scholar,
        &providers.crossref,
        &providers.pubmed,
        &providers.arxiv,
    ];
    let enabled = statuses.iter().filter(|status| status.enabled).count();
    let ready = statuses
        .iter()
        .filter(|status| status.enabled && status.readiness == ProviderReadiness::Ready)
        .count();
    ProductDoctorCheckV1 {
        id: ProductDoctorCheckId::LiteratureProviders,
        status: if enabled == ready {
            ProductDoctorStatus::Ready
        } else {
            ProductDoctorStatus::Attention
        },
        blocking: false,
        code: if enabled == ready {
            "literature-providers-ready"
        } else {
            "literature-providers-configuration-required"
        },
        remediation: if enabled == ready {
            "none"
        } else {
            "configure-literature-providers"
        },
        section: "literature-providers",
        path_id: Some("global-config"),
    }
}

fn client_doctor_checks(inventory: Option<&ClientInventory>) -> [ProductDoctorCheckV1; 2] {
    let Some(inventory) = inventory else {
        return [
            unavailable_client_check(ProductDoctorCheckId::CodexLocal, "codex-local"),
            unavailable_client_check(ProductDoctorCheckId::ClaudeCodeLocal, "claude-code-local"),
        ];
    };
    let clients = &inventory.summary().clients;
    [
        client_doctor_check(&clients[0]),
        client_doctor_check(&clients[1]),
    ]
}

fn unavailable_client_check(
    id: ProductDoctorCheckId,
    section: &'static str,
) -> ProductDoctorCheckV1 {
    ProductDoctorCheckV1 {
        id,
        status: ProductDoctorStatus::Unavailable,
        blocking: true,
        code: "client-inventory-home-unavailable",
        remediation: "inspect-client-paths",
        section,
        path_id: None,
    }
}

fn client_doctor_check(client: &ClientInventoryEntryV1) -> ProductDoctorCheckV1 {
    let (id, section, path_id, client_code) = match client.client {
        ClientKind::Codex => (
            ProductDoctorCheckId::CodexLocal,
            "integrations",
            "codex-config",
            "codex",
        ),
        ClientKind::ClaudeCode => (
            ProductDoctorCheckId::ClaudeCodeLocal,
            "integrations",
            "claude-config",
            "claude-code",
        ),
    };
    let (status, blocking, remediation) = match (client.discovery, client.readiness) {
        (ClientDiscoveryState::NotDetected, _) => (
            ProductDoctorStatus::Missing,
            false,
            "install-supported-client",
        ),
        (ClientDiscoveryState::Unavailable, _) | (_, ClientActionReadiness::Unavailable) => (
            ProductDoctorStatus::Unavailable,
            true,
            "inspect-client-paths",
        ),
        (_, ClientActionReadiness::Current) => (ProductDoctorStatus::Ready, false, "none"),
        (_, ClientActionReadiness::ResolveConflict) => (
            ProductDoctorStatus::Attention,
            true,
            "resolve-client-conflict",
        ),
        (_, ClientActionReadiness::RepairReady) => (
            ProductDoctorStatus::RecoveryRequired,
            true,
            "repair-client-integration",
        ),
        (_, ClientActionReadiness::InstallReady | ClientActionReadiness::InspectOnly) => (
            ProductDoctorStatus::Attention,
            false,
            "install-client-integration",
        ),
    };
    ProductDoctorCheckV1 {
        id,
        status,
        blocking,
        code: match (client_code, status) {
            ("codex", ProductDoctorStatus::Ready) => "codex-local-ready",
            ("codex", ProductDoctorStatus::Missing) => "codex-client-not-detected",
            ("codex", ProductDoctorStatus::Unavailable) => "codex-local-unavailable",
            ("codex", ProductDoctorStatus::RecoveryRequired) => "codex-local-recovery-required",
            ("codex", _) => "codex-local-attention",
            ("claude-code", ProductDoctorStatus::Ready) => "claude-code-local-ready",
            ("claude-code", ProductDoctorStatus::Missing) => "claude-code-client-not-detected",
            ("claude-code", ProductDoctorStatus::Unavailable) => "claude-code-local-unavailable",
            ("claude-code", ProductDoctorStatus::RecoveryRequired) => {
                "claude-code-local-recovery-required"
            }
            ("claude-code", _) => "claude-code-local-attention",
            _ => "client-local-attention",
        },
        remediation,
        section,
        path_id: Some(path_id),
    }
}

fn append_client_paths(paths: &mut Vec<ProductPathInspectionV1>, inventory: &ClientInventory) {
    for client in &inventory.summary().clients {
        let group = match client.client {
            ClientKind::Codex => ProductPathGroup::Codex,
            ClientKind::ClaudeCode => ProductPathGroup::ClaudeCode,
        };
        for candidate in &client.paths {
            let Some(exact) = inventory.exact_path(candidate.id) else {
                continue;
            };
            let mut descriptor = PathDescriptor::new(
                client_path_id(candidate.id),
                client_path_label(candidate.id),
                group,
                client_path_scope(candidate.scope),
                client_path_source(candidate.source),
                candidate.symbolic_path.display(),
                exact,
                ExpectedPathType::Any,
            );
            descriptor.selected = candidate.selected;
            descriptor.safety = client_path_safety(candidate);
            push_path(paths, descriptor);
        }
    }
}

fn client_path_id(id: ClientPathId) -> &'static str {
    match id {
        ClientPathId::CodexConfig => "codex-config",
        ClientPathId::CodexUserSkills => "codex-user-skills",
        ClientPathId::CodexProjectSkills => "codex-project-skills",
        ClientPathId::CodexCustomSkills => "codex-custom-skills",
        ClientPathId::CodexMarketplace => "codex-marketplace",
        ClientPathId::CodexPluginSource => "codex-plugin-source",
        ClientPathId::CodexLegacyPluginSource => "codex-legacy-plugin-source",
        ClientPathId::CodexLegacySkills => "codex-legacy-skills",
        ClientPathId::CodexLegacyMcpConfig => "codex-legacy-mcp-config",
        ClientPathId::ClaudeConfig => "claude-config",
        ClientPathId::ClaudeUserSkills => "claude-user-skills",
        ClientPathId::ClaudeProjectSkills => "claude-project-skills",
        ClientPathId::ClaudeCustomSkills => "claude-custom-skills",
        ClientPathId::ClaudeMarketplace => "claude-marketplace",
        ClientPathId::ClaudePluginSource => "claude-plugin-source",
        ClientPathId::ClaudeLegacyPluginSource => "claude-legacy-plugin-source",
        ClientPathId::ClaudeDirectSkills => "claude-direct-skills",
        ClientPathId::ClaudeLegacySkills => "claude-legacy-skills",
        ClientPathId::ClaudeLegacyMcpConfig => "claude-legacy-mcp-config",
    }
}

fn client_path_label(id: ClientPathId) -> &'static str {
    match id {
        ClientPathId::CodexConfig => "Codex configuration",
        ClientPathId::CodexUserSkills => "Codex user Skills",
        ClientPathId::CodexProjectSkills => "Codex project Skills",
        ClientPathId::CodexCustomSkills => "Codex custom Skills",
        ClientPathId::CodexMarketplace => "Codex personal marketplace",
        ClientPathId::CodexPluginSource => "Codex Qiongli plugin source",
        ClientPathId::CodexLegacyPluginSource => "Codex legacy plugin source",
        ClientPathId::CodexLegacySkills => "Codex legacy Skills",
        ClientPathId::CodexLegacyMcpConfig => "Codex legacy standalone MCP",
        ClientPathId::ClaudeConfig => "Claude Code configuration",
        ClientPathId::ClaudeUserSkills => "Claude Code user Skills",
        ClientPathId::ClaudeProjectSkills => "Claude Code project Skills",
        ClientPathId::ClaudeCustomSkills => "Claude Code custom Skills",
        ClientPathId::ClaudeMarketplace => "Claude Code marketplace",
        ClientPathId::ClaudePluginSource => "Claude Code Qiongli plugin source",
        ClientPathId::ClaudeLegacyPluginSource => "Claude Code legacy plugin source",
        ClientPathId::ClaudeDirectSkills => "Claude Code direct Skills",
        ClientPathId::ClaudeLegacySkills => "Claude Code legacy Skills",
        ClientPathId::ClaudeLegacyMcpConfig => "Claude Code legacy standalone MCP",
    }
}

const fn client_path_scope(scope: ClientPathScope) -> ProductPathScope {
    match scope {
        ClientPathScope::User => ProductPathScope::User,
        ClientPathScope::Project => ProductPathScope::Project,
        ClientPathScope::Managed => ProductPathScope::Managed,
        ClientPathScope::Custom => ProductPathScope::Custom,
        ClientPathScope::Legacy => ProductPathScope::Legacy,
    }
}

const fn client_path_source(source: ClientPathSource) -> ProductPathSource {
    match source {
        ClientPathSource::EnvironmentOverride => ProductPathSource::EnvironmentOverride,
        ClientPathSource::OfficialDefault => ProductPathSource::OfficialDefault,
        ClientPathSource::ProjectContext => ProductPathSource::CurrentProject,
        ClientPathSource::QiongliManaged => ProductPathSource::QiongliManaged,
        ClientPathSource::ExplicitCustom => ProductPathSource::ClientAdapter,
        ClientPathSource::LegacyObserved => ProductPathSource::LegacyObserved,
    }
}

const fn client_path_safety(candidate: &ClientPathCandidateV1) -> ProductPathSafety {
    if matches!(candidate.state, ClientPathState::Unsafe) {
        return ProductPathSafety::Unsafe;
    }
    match candidate.management {
        ClientPathManagement::Supported => ProductPathSafety::Supported,
        ClientPathManagement::InspectOnly => ProductPathSafety::InspectOnly,
        ClientPathManagement::LegacyOnly => ProductPathSafety::LegacyOnly,
        ClientPathManagement::Unsafe => ProductPathSafety::Unsafe,
        ClientPathManagement::Unavailable => ProductPathSafety::Unavailable,
    }
}

struct PathDescriptor<'a> {
    id: String,
    label: String,
    group: ProductPathGroup,
    scope: ProductPathScope,
    source: ProductPathSource,
    symbolic_path: String,
    exact_path: &'a Path,
    expected: ExpectedPathType,
    selected: bool,
    safety: ProductPathSafety,
}

impl<'a> PathDescriptor<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        group: ProductPathGroup,
        scope: ProductPathScope,
        source: ProductPathSource,
        symbolic_path: impl Into<String>,
        exact_path: &'a Path,
        expected: ExpectedPathType,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            group,
            scope,
            source,
            symbolic_path: symbolic_path.into(),
            exact_path,
            expected,
            selected: true,
            safety: ProductPathSafety::Supported,
        }
    }
}

fn push_path(paths: &mut Vec<ProductPathInspectionV1>, descriptor: PathDescriptor<'_>) {
    let metadata = fs::symlink_metadata(descriptor.exact_path);
    let (file_type, exists, owner, writability, resolved_target) = match metadata {
        Ok(metadata) => {
            let file_type = if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                ProductPathFileType::Symlink
            } else if metadata.is_dir() {
                ProductPathFileType::Directory
            } else if metadata.is_file() {
                ProductPathFileType::File
            } else {
                ProductPathFileType::Other
            };
            (
                file_type,
                true,
                path_owner(&metadata),
                if metadata.permissions().readonly() {
                    ProductPathWritability::ReadOnly
                } else {
                    ProductPathWritability::Writable
                },
                (file_type == ProductPathFileType::Symlink)
                    .then(|| resolved_link_target(descriptor.exact_path))
                    .flatten(),
            )
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let writable = nearest_existing_parent(descriptor.exact_path).map_or(
                ProductPathWritability::Unknown,
                |metadata| {
                    if metadata.permissions().readonly() {
                        ProductPathWritability::MissingParentReadOnly
                    } else {
                        ProductPathWritability::MissingParentWritable
                    }
                },
            );
            (
                ProductPathFileType::Missing,
                false,
                ProductPathOwner::Missing,
                writable,
                None,
            )
        }
        Err(_) => (
            ProductPathFileType::Unavailable,
            false,
            ProductPathOwner::Unknown,
            ProductPathWritability::Unknown,
            None,
        ),
    };
    let safety = if !descriptor.exact_path.is_absolute()
        || path_has_lexical_traversal(descriptor.exact_path)
    {
        ProductPathSafety::Unsafe
    } else {
        descriptor.safety
    };
    let type_matches_expected = if !exists {
        None
    } else {
        Some(match descriptor.expected {
            ExpectedPathType::Any => true,
            ExpectedPathType::Directory => file_type == ProductPathFileType::Directory,
            ExpectedPathType::File => file_type == ProductPathFileType::File,
        })
    };
    paths.push(ProductPathInspectionV1 {
        id: descriptor.id,
        label: descriptor.label,
        group: descriptor.group,
        scope: descriptor.scope,
        source: descriptor.source,
        selected: descriptor.selected,
        symbolic_path: descriptor.symbolic_path,
        exact_path: display_path(descriptor.exact_path),
        exists,
        file_type,
        expected_type: match descriptor.expected {
            ExpectedPathType::Directory => "directory",
            ExpectedPathType::File => "file",
            ExpectedPathType::Any => "any",
        },
        type_matches_expected,
        owner,
        writability,
        safety,
        resolved_target: resolved_target.as_deref().map(display_path),
    });
}

fn nearest_existing_parent(path: &Path) -> Option<fs::Metadata> {
    let mut candidate = path.parent();
    while let Some(parent) = candidate {
        match fs::symlink_metadata(parent) {
            Ok(metadata) => return Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                candidate = parent.parent();
            }
            Err(_) => return None,
        }
    }
    None
}

fn resolved_link_target(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path).ok().or_else(|| {
        fs::read_link(path).ok().map(|target| {
            if target.is_absolute() {
                target
            } else {
                path.parent().unwrap_or(path).join(target)
            }
        })
    })
}

#[cfg(unix)]
fn path_owner(metadata: &fs::Metadata) -> ProductPathOwner {
    use std::os::unix::fs::MetadataExt;

    if metadata.uid() == rustix::process::geteuid().as_raw() {
        ProductPathOwner::CurrentUser
    } else {
        ProductPathOwner::OtherUser
    }
}

#[cfg(not(unix))]
fn path_owner(_metadata: &fs::Metadata) -> ProductPathOwner {
    ProductPathOwner::Unknown
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

fn path_has_lexical_traversal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::CurDir | std::path::Component::ParentDir
        )
    })
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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use qiongli_config::SecretStoreStatus;
    use qiongli_content::EmbeddedContent;

    use super::*;

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn isolated_snapshot_reports_exact_adapter_paths_and_redacted_debug() {
        let root = test_root("exact-paths");
        let home = root.join("home");
        let project = root.join("project");
        let config = root.join("config");
        fs::create_dir_all(home.join(".agents/skills")).unwrap();
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(project.join(".qiongli/all-chat")).unwrap();
        fs::write(
            project.join(".qiongli/all-chat/run_00000000000000000000000000000000.json"),
            "PRIVATE_CHAT_DIAGNOSTIC_CANARY",
        )
        .unwrap();
        let environment =
            CommandEnvironment::with_paths(Some(OsString::from(&config)), Some(home.clone()), None)
                .with_inventory_context(None, Some(project.clone()), true, true);
        let content = test_content();
        let snapshot = inspect_product(&environment, &content, SecretStoreStatus::Unavailable);

        assert!(
            !serde_json::to_string(&snapshot)
                .unwrap()
                .contains("PRIVATE_CHAT_DIAGNOSTIC_CANARY")
        );
        assert!(!format!("{snapshot:?}").contains("PRIVATE_CHAT_DIAGNOSTIC_CANARY"));
        assert_eq!(snapshot.schema_version, PRODUCT_INSPECTION_SCHEMA_VERSION);
        let codex = snapshot
            .paths
            .iter()
            .find(|path| path.id == "codex-user-skills")
            .unwrap();
        assert_eq!(codex.exact_path, display_path(&home.join(".agents/skills")));
        assert!(codex.exists);
        assert_eq!(codex.file_type, ProductPathFileType::Directory);
        assert_eq!(codex.type_matches_expected, Some(true));
        assert_eq!(codex.source, ProductPathSource::OfficialDefault);
        assert!(format!("{codex:?}").contains("symbolic_path"));
        assert!(!format!("{codex:?}").contains(&root.to_string_lossy().into_owned()));
        assert!(
            snapshot
                .paths
                .iter()
                .any(|path| path.id == "project-root" && path.exact_path == display_path(&project))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[cfg(windows)]
    #[test]
    fn windows_reparse_attribute_is_reported_as_a_link() {
        assert!(attributes_are_reparse_point(0x0400));
        assert!(attributes_are_reparse_point(0x0400 | 0x0020));
        assert!(!attributes_are_reparse_point(0x0020));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_target_is_reported_but_never_followed_for_state_classification() {
        use std::os::unix::fs::symlink;

        let root = test_root("symlink");
        let target = root.join("target");
        let link = root.join("link");
        fs::create_dir_all(&target).unwrap();
        symlink(&target, &link).unwrap();
        let mut paths = Vec::new();
        push_path(
            &mut paths,
            PathDescriptor::new(
                "test-link",
                "Test link",
                ProductPathGroup::Product,
                ProductPathScope::User,
                ProductPathSource::Runtime,
                "<test-link>",
                &link,
                ExpectedPathType::Directory,
            ),
        );
        assert_eq!(paths[0].file_type, ProductPathFileType::Symlink);
        let canonical_target = fs::canonicalize(&target).unwrap();
        assert_eq!(
            paths[0].resolved_target.as_deref(),
            Some(display_path(&canonical_target).as_str())
        );
        let _ = fs::remove_dir_all(root);
    }

    fn test_content() -> EmbeddedContent {
        crate::embedded_content().expect("embedded test content")
    }

    fn test_root(name: &str) -> PathBuf {
        let nonce = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "qiongli-product-inspection-{name}-{}-{nonce}",
            std::process::id()
        ))
    }
}
