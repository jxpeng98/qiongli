use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "embedded-content")]
use qiongli_content::EmbeddedContent;

use crate::{RuntimeError, RuntimeErrorCode};

pub const LITE_TOOL_CONTRACT_RESOURCE_PATH: &str = "mcp-contracts/lite-tools.json";
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

const LITE_CONTRACT_SCHEMA_VERSION: &str = "1.0";
#[cfg(feature = "embedded-content")]
const MARKETPLACE_LITE_PROFILE: &str = "marketplace-lite";
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

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL_CONTRACT: &[u8] =
        include_bytes!("../../../../../content/mcp-contracts/lite-tools.json");
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
}
