use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "embedded-content")]
use qiongli_content::EmbeddedContent;

use crate::{RuntimeError, RuntimeErrorCode};

pub const LITE_TOOL_CONTRACT_RESOURCE_PATH: &str = "mcp-contracts/lite-tools.json";
pub const FULL_PROJECT_TOOL_CONTRACT_RESOURCE_PATH: &str = "mcp-contracts/full-project-tools.json";
pub const LITE_PUBLIC_TOOL_NAMES: [&str; 12] = [
    "qiongli_config_status",
    "qiongli_save_provider_config",
    "qiongli_configure_provider",
    "qiongli_open_config_wizard",
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
    "qiongli_zotero_status",
    "qiongli_zotero_export_import_files",
    "qiongli_orchestrator_route",
    "qiongli_task_plan",
];
pub const FULL_PROJECT_PUBLIC_TOOL_NAMES: [&str; 9] = [
    "qiongli_project_list",
    "qiongli_project_read",
    "qiongli_project_graph_snapshot",
    "qiongli_project_graph_portfolio",
    "qiongli_project_graph_query",
    "qiongli_project_artifact_changes",
    "qiongli_project_capture_coverage",
    "qiongli_project_capture_preview",
    "qiongli_project_capture_apply",
];

const LITE_CONTRACT_SCHEMA_VERSION: &str = "1.0";
#[cfg(feature = "embedded-content")]
const MARKETPLACE_LITE_PROFILE: &str = "marketplace-lite";
#[cfg(feature = "embedded-content")]
const FULL_PROFILE: &str = "full";
const MAX_LITE_CONTRACT_BYTES: usize = 1024 * 1024;
const MAX_TOOL_DESCRIPTION_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteToolId {
    ConfigStatus,
    SaveProviderConfig,
    ConfigureProvider,
    LiteratureStatus,
    SearchPlan,
    LiteratureSearch,
    LiteratureExportEvidence,
    ZoteroStatus,
    ZoteroExportImportFiles,
    OrchestratorRoute,
    TaskPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FullProjectToolId {
    List,
    Read,
    GraphSnapshot,
    GraphPortfolio,
    GraphQuery,
    ArtifactChanges,
    CaptureCoverage,
    CapturePreview,
    CaptureApply,
}

impl FullProjectToolId {
    #[must_use]
    pub fn from_public_name(name: &str) -> Option<Self> {
        match name {
            "qiongli_project_list" => Some(Self::List),
            "qiongli_project_read" => Some(Self::Read),
            "qiongli_project_graph_snapshot" => Some(Self::GraphSnapshot),
            "qiongli_project_graph_portfolio" => Some(Self::GraphPortfolio),
            "qiongli_project_graph_query" => Some(Self::GraphQuery),
            "qiongli_project_artifact_changes" => Some(Self::ArtifactChanges),
            "qiongli_project_capture_coverage" => Some(Self::CaptureCoverage),
            "qiongli_project_capture_preview" => Some(Self::CapturePreview),
            "qiongli_project_capture_apply" => Some(Self::CaptureApply),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteConfigHandler {
    Status,
    SaveProvider,
    ConfigureProvider,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteLiteratureHandler {
    Status,
    SearchPlan,
    Search,
    ExportEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteZoteroHandler {
    Status,
    ExportImportFiles,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteOrchestrationHandler {
    Route,
    TaskPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteDispatchTarget {
    Config(LiteConfigHandler),
    Literature(LiteLiteratureHandler),
    Zotero(LiteZoteroHandler),
    Orchestration(LiteOrchestrationHandler),
}

impl LiteToolId {
    #[must_use]
    pub fn from_public_name(name: &str) -> Option<Self> {
        match name {
            "qiongli_config_status" => Some(Self::ConfigStatus),
            "qiongli_save_provider_config" => Some(Self::SaveProviderConfig),
            "qiongli_configure_provider" | "qiongli_open_config_wizard" => {
                Some(Self::ConfigureProvider)
            }
            "qiongli_literature_status" => Some(Self::LiteratureStatus),
            "qiongli_search_plan" => Some(Self::SearchPlan),
            "qiongli_literature_search" => Some(Self::LiteratureSearch),
            "qiongli_literature_export_evidence" => Some(Self::LiteratureExportEvidence),
            "qiongli_zotero_status" => Some(Self::ZoteroStatus),
            "qiongli_zotero_export_import_files" => Some(Self::ZoteroExportImportFiles),
            "qiongli_orchestrator_route" => Some(Self::OrchestratorRoute),
            "qiongli_task_plan" => Some(Self::TaskPlan),
            _ => None,
        }
    }

    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::ConfigStatus => "qiongli_config_status",
            Self::SaveProviderConfig => "qiongli_save_provider_config",
            Self::ConfigureProvider => "qiongli_configure_provider",
            Self::LiteratureStatus => "qiongli_literature_status",
            Self::SearchPlan => "qiongli_search_plan",
            Self::LiteratureSearch => "qiongli_literature_search",
            Self::LiteratureExportEvidence => "qiongli_literature_export_evidence",
            Self::ZoteroStatus => "qiongli_zotero_status",
            Self::ZoteroExportImportFiles => "qiongli_zotero_export_import_files",
            Self::OrchestratorRoute => "qiongli_orchestrator_route",
            Self::TaskPlan => "qiongli_task_plan",
        }
    }

    #[must_use]
    pub const fn dispatch_target(self) -> LiteDispatchTarget {
        match self {
            Self::ConfigStatus => LiteDispatchTarget::Config(LiteConfigHandler::Status),
            Self::SaveProviderConfig => LiteDispatchTarget::Config(LiteConfigHandler::SaveProvider),
            Self::ConfigureProvider => {
                LiteDispatchTarget::Config(LiteConfigHandler::ConfigureProvider)
            }
            Self::LiteratureStatus => LiteDispatchTarget::Literature(LiteLiteratureHandler::Status),
            Self::SearchPlan => LiteDispatchTarget::Literature(LiteLiteratureHandler::SearchPlan),
            Self::LiteratureSearch => LiteDispatchTarget::Literature(LiteLiteratureHandler::Search),
            Self::LiteratureExportEvidence => {
                LiteDispatchTarget::Literature(LiteLiteratureHandler::ExportEvidence)
            }
            Self::ZoteroStatus => LiteDispatchTarget::Zotero(LiteZoteroHandler::Status),
            Self::ZoteroExportImportFiles => {
                LiteDispatchTarget::Zotero(LiteZoteroHandler::ExportImportFiles)
            }
            Self::OrchestratorRoute => {
                LiteDispatchTarget::Orchestration(LiteOrchestrationHandler::Route)
            }
            Self::TaskPlan => LiteDispatchTarget::Orchestration(LiteOrchestrationHandler::TaskPlan),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolContract {
    schema_version: String,
    tools: Vec<ToolDefinition>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiteToolRegistry {
    tools: Vec<ToolDefinition>,
}

impl LiteToolRegistry {
    pub fn from_json(bytes: &[u8]) -> Result<Self, RuntimeError> {
        if bytes.len() > MAX_LITE_CONTRACT_BYTES {
            return Err(RuntimeError::new(RuntimeErrorCode::LiteContractTooLarge));
        }
        let contract = serde_json::from_slice::<ToolContract>(bytes)
            .map_err(|_| RuntimeError::new(RuntimeErrorCode::InvalidLiteContract))?;
        validate_contract(&contract)?;
        Ok(Self {
            tools: contract.tools,
        })
    }

    #[cfg(feature = "embedded-content")]
    pub fn from_embedded_content(content: &EmbeddedContent) -> Result<Self, RuntimeError> {
        let resource = content
            .read_profile_resource(MARKETPLACE_LITE_PROFILE, LITE_TOOL_CONTRACT_RESOURCE_PATH)
            .map_err(|_| RuntimeError::new(RuntimeErrorCode::LiteContractUnavailable))?
            .ok_or_else(|| RuntimeError::new(RuntimeErrorCode::LiteContractUnavailable))?;
        Self::from_json(resource.bytes())
    }

    #[must_use]
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    #[must_use]
    pub fn into_tools(self) -> Vec<ToolDefinition> {
        self.tools
    }

    #[must_use]
    pub fn resolve(&self, public_name: &str) -> Option<LiteToolId> {
        LiteToolId::from_public_name(public_name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FullProjectToolRegistry {
    tools: Vec<ToolDefinition>,
}

impl FullProjectToolRegistry {
    pub fn from_json(bytes: &[u8]) -> Result<Self, RuntimeError> {
        if bytes.len() > MAX_LITE_CONTRACT_BYTES {
            return Err(RuntimeError::new(
                RuntimeErrorCode::FullProjectContractTooLarge,
            ));
        }
        let contract = serde_json::from_slice::<ToolContract>(bytes)
            .map_err(|_| RuntimeError::new(RuntimeErrorCode::InvalidFullProjectContract))?;
        validate_full_project_contract(&contract)?;
        Ok(Self {
            tools: contract.tools,
        })
    }

    #[cfg(feature = "embedded-content")]
    pub fn from_embedded_content(content: &EmbeddedContent) -> Result<Self, RuntimeError> {
        let resource = content
            .read_profile_resource(FULL_PROFILE, FULL_PROJECT_TOOL_CONTRACT_RESOURCE_PATH)
            .map_err(|_| RuntimeError::new(RuntimeErrorCode::FullProjectContractUnavailable))?
            .ok_or_else(|| RuntimeError::new(RuntimeErrorCode::FullProjectContractUnavailable))?;
        Self::from_json(resource.bytes())
    }

    #[must_use]
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    #[must_use]
    pub fn resolve(&self, public_name: &str) -> Option<FullProjectToolId> {
        FullProjectToolId::from_public_name(public_name)
    }
}

fn validate_contract(contract: &ToolContract) -> Result<(), RuntimeError> {
    if contract.schema_version != LITE_CONTRACT_SCHEMA_VERSION
        || contract.tools.len() != LITE_PUBLIC_TOOL_NAMES.len()
    {
        return Err(RuntimeError::new(RuntimeErrorCode::InvalidLiteContract));
    }

    for (tool, expected_name) in contract.tools.iter().zip(LITE_PUBLIC_TOOL_NAMES) {
        if tool.name != expected_name
            || tool.description.trim().is_empty()
            || tool.description.len() > MAX_TOOL_DESCRIPTION_BYTES
            || !tool.input_schema.is_object()
        {
            return Err(RuntimeError::new(RuntimeErrorCode::InvalidLiteContract));
        }
    }
    Ok(())
}

fn validate_full_project_contract(contract: &ToolContract) -> Result<(), RuntimeError> {
    if contract.schema_version != LITE_CONTRACT_SCHEMA_VERSION
        || contract.tools.len() != FULL_PROJECT_PUBLIC_TOOL_NAMES.len()
    {
        return Err(RuntimeError::new(
            RuntimeErrorCode::InvalidFullProjectContract,
        ));
    }
    for (tool, expected_name) in contract.tools.iter().zip(FULL_PROJECT_PUBLIC_TOOL_NAMES) {
        if tool.name != expected_name
            || tool.description.trim().is_empty()
            || tool.description.len() > MAX_TOOL_DESCRIPTION_BYTES
            || !tool.input_schema.is_object()
        {
            return Err(RuntimeError::new(
                RuntimeErrorCode::InvalidFullProjectContract,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL_CONTRACT: &[u8] =
        include_bytes!("../../../../../content/mcp-contracts/lite-tools.json");
    const FULL_PROJECT_CONTRACT: &[u8] =
        include_bytes!("../../../../../content/mcp-contracts/full-project-tools.json");
    const CANARY: &str = "private-contract-canary";

    #[test]
    fn canonical_contract_has_frozen_public_and_typed_identities() {
        let registry = LiteToolRegistry::from_json(CANONICAL_CONTRACT).unwrap();
        let names = registry
            .tools()
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, LITE_PUBLIC_TOOL_NAMES);
        assert_eq!(
            registry.resolve("qiongli_configure_provider"),
            Some(LiteToolId::ConfigureProvider)
        );
        assert_eq!(
            registry.resolve("qiongli_open_config_wizard"),
            Some(LiteToolId::ConfigureProvider)
        );
        assert_eq!(
            LiteToolId::ConfigureProvider.canonical_name(),
            "qiongli_configure_provider"
        );
        assert_eq!(registry.resolve("unknown"), None);
    }

    #[test]
    fn every_canonical_identity_has_an_exhaustive_domain_target() {
        let expected = [
            (
                LiteToolId::ConfigStatus,
                LiteDispatchTarget::Config(LiteConfigHandler::Status),
            ),
            (
                LiteToolId::SaveProviderConfig,
                LiteDispatchTarget::Config(LiteConfigHandler::SaveProvider),
            ),
            (
                LiteToolId::ConfigureProvider,
                LiteDispatchTarget::Config(LiteConfigHandler::ConfigureProvider),
            ),
            (
                LiteToolId::LiteratureStatus,
                LiteDispatchTarget::Literature(LiteLiteratureHandler::Status),
            ),
            (
                LiteToolId::SearchPlan,
                LiteDispatchTarget::Literature(LiteLiteratureHandler::SearchPlan),
            ),
            (
                LiteToolId::LiteratureSearch,
                LiteDispatchTarget::Literature(LiteLiteratureHandler::Search),
            ),
            (
                LiteToolId::LiteratureExportEvidence,
                LiteDispatchTarget::Literature(LiteLiteratureHandler::ExportEvidence),
            ),
            (
                LiteToolId::ZoteroStatus,
                LiteDispatchTarget::Zotero(LiteZoteroHandler::Status),
            ),
            (
                LiteToolId::ZoteroExportImportFiles,
                LiteDispatchTarget::Zotero(LiteZoteroHandler::ExportImportFiles),
            ),
            (
                LiteToolId::OrchestratorRoute,
                LiteDispatchTarget::Orchestration(LiteOrchestrationHandler::Route),
            ),
            (
                LiteToolId::TaskPlan,
                LiteDispatchTarget::Orchestration(LiteOrchestrationHandler::TaskPlan),
            ),
        ];

        for (tool, target) in expected {
            assert_eq!(tool.dispatch_target(), target);
        }

        assert_eq!(
            LiteToolId::from_public_name("qiongli_open_config_wizard")
                .unwrap()
                .dispatch_target(),
            LiteDispatchTarget::Config(LiteConfigHandler::ConfigureProvider)
        );
    }

    #[test]
    fn rejects_oversized_and_malformed_contracts_with_redacted_codes() {
        let oversized = vec![b' '; MAX_LITE_CONTRACT_BYTES + 1];
        let oversized_error = LiteToolRegistry::from_json(&oversized).unwrap_err();
        assert_eq!(
            oversized_error.code(),
            RuntimeErrorCode::LiteContractTooLarge
        );

        let malformed = format!(r#"{{"schema_version":"{CANARY}""#);
        let malformed_error = LiteToolRegistry::from_json(malformed.as_bytes()).unwrap_err();
        assert_eq!(
            malformed_error.code(),
            RuntimeErrorCode::InvalidLiteContract
        );
        assert!(!malformed_error.to_string().contains(CANARY));
    }

    #[test]
    fn rejects_schema_name_count_and_order_drift() {
        for mutation in [
            |value: &mut Value| value["schema_version"] = Value::String("2.0".to_string()),
            |value: &mut Value| {
                value["tools"].as_array_mut().unwrap().remove(0);
            },
            |value: &mut Value| {
                let tools = value["tools"].as_array_mut().unwrap();
                tools.swap(0, 1);
            },
            |value: &mut Value| {
                value["tools"].as_array_mut().unwrap()[0]["name"] =
                    Value::String(CANARY.to_string());
            },
        ] {
            let mut value: Value = serde_json::from_slice(CANONICAL_CONTRACT).unwrap();
            mutation(&mut value);
            let bytes = serde_json::to_vec(&value).unwrap();
            let error = LiteToolRegistry::from_json(&bytes).unwrap_err();
            assert_eq!(error.code(), RuntimeErrorCode::InvalidLiteContract);
            assert!(!error.to_string().contains(CANARY));
        }

        let mut extra: Value = serde_json::from_slice(CANONICAL_CONTRACT).unwrap();
        let duplicate = extra["tools"].as_array().unwrap()[0].clone();
        extra["tools"].as_array_mut().unwrap().push(duplicate);
        let error = LiteToolRegistry::from_json(&serde_json::to_vec(&extra).unwrap()).unwrap_err();
        assert_eq!(error.code(), RuntimeErrorCode::InvalidLiteContract);
    }

    #[test]
    fn rejects_unknown_fields_and_invalid_definition_shapes() {
        for mutation in [
            |value: &mut Value| {
                value["private"] = Value::String(CANARY.to_string());
            },
            |value: &mut Value| {
                value["tools"][0]["private"] = Value::String(CANARY.to_string());
            },
            |value: &mut Value| {
                value["tools"][0]["description"] = Value::String(" ".to_string());
            },
            |value: &mut Value| {
                value["tools"][0]["inputSchema"] = Value::String(CANARY.to_string());
            },
        ] {
            let mut value: Value = serde_json::from_slice(CANONICAL_CONTRACT).unwrap();
            mutation(&mut value);
            let error =
                LiteToolRegistry::from_json(&serde_json::to_vec(&value).unwrap()).unwrap_err();
            assert_eq!(error.code(), RuntimeErrorCode::InvalidLiteContract);
            assert!(!error.to_string().contains(CANARY));
        }
    }

    #[test]
    fn full_project_contract_has_a_closed_project_and_capture_inventory() {
        let registry = FullProjectToolRegistry::from_json(FULL_PROJECT_CONTRACT).unwrap();
        let names = registry
            .tools()
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, FULL_PROJECT_PUBLIC_TOOL_NAMES);
        assert_eq!(
            registry.resolve("qiongli_project_list"),
            Some(FullProjectToolId::List)
        );
        assert_eq!(
            registry.resolve("qiongli_project_read"),
            Some(FullProjectToolId::Read)
        );
        assert_eq!(
            registry.resolve("qiongli_project_graph_snapshot"),
            Some(FullProjectToolId::GraphSnapshot)
        );
        assert_eq!(
            registry.resolve("qiongli_project_graph_portfolio"),
            Some(FullProjectToolId::GraphPortfolio)
        );
        assert_eq!(
            registry.resolve("qiongli_project_graph_query"),
            Some(FullProjectToolId::GraphQuery)
        );
        assert_eq!(
            registry.resolve("qiongli_project_artifact_changes"),
            Some(FullProjectToolId::ArtifactChanges)
        );
        assert_eq!(
            registry.resolve("qiongli_project_capture_coverage"),
            Some(FullProjectToolId::CaptureCoverage)
        );
        assert_eq!(
            registry.resolve("qiongli_project_capture_preview"),
            Some(FullProjectToolId::CapturePreview)
        );
        assert_eq!(
            registry.resolve("qiongli_project_capture_apply"),
            Some(FullProjectToolId::CaptureApply)
        );
        assert_eq!(
            registry.resolve("qiongli_project_capture_consolidate"),
            None
        );
    }

    #[test]
    fn full_project_contract_rejects_drift_with_redacted_codes() {
        let mut value: Value = serde_json::from_slice(FULL_PROJECT_CONTRACT).unwrap();
        value["tools"][0]["name"] = Value::String(CANARY.to_string());
        let error =
            FullProjectToolRegistry::from_json(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert_eq!(error.code(), RuntimeErrorCode::InvalidFullProjectContract);
        assert!(!error.to_string().contains(CANARY));

        let oversized = vec![b' '; MAX_LITE_CONTRACT_BYTES + 1];
        assert_eq!(
            FullProjectToolRegistry::from_json(&oversized)
                .unwrap_err()
                .code(),
            RuntimeErrorCode::FullProjectContractTooLarge
        );
    }
}
