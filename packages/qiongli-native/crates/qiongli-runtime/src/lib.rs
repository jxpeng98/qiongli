mod contract;
mod error;
pub mod evidence;
pub mod mcp;
pub mod orchestration;
pub mod protocol;
pub mod providers;
pub mod searchplan;
pub mod zotero;

pub use contract::{
    FULL_PROJECT_PUBLIC_TOOL_NAMES, FULL_PROJECT_TOOL_CONTRACT_RESOURCE_PATH, FullProjectToolId,
    FullProjectToolRegistry, LITE_PUBLIC_TOOL_NAMES, LITE_TOOL_CONTRACT_RESOURCE_PATH,
    LiteConfigHandler, LiteDispatchTarget, LiteLiteratureHandler, LiteOrchestrationHandler,
    LiteToolId, LiteToolRegistry, LiteZoteroHandler, ToolDefinition,
};
pub use error::{RuntimeError, RuntimeErrorCode};
