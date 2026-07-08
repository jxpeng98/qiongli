use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SearchPlanInput {
    pub query: String,
    pub search_mode: Option<String>,
    pub provider_connected: bool,
    pub native_search_usable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchPlan {
    pub query: String,
    pub search_mode: String,
    pub search_execution_mode: String,
    pub provider_capability_mode: String,
    pub native_search_queries: Vec<String>,
}

pub fn build_search_plan(input: SearchPlanInput) -> SearchPlan {
    let search_execution_mode = match (input.provider_connected, input.native_search_usable) {
        (true, true) => "hybrid_search",
        (true, false) => "provider_connected",
        (false, true) => "native_only",
        (false, false) => "strategy_only",
    };
    let provider_capability_mode = if input.provider_connected {
        "provider_connected"
    } else {
        "strategy_only"
    };
    SearchPlan {
        native_search_queries: vec![input.query.clone()],
        query: input.query,
        search_mode: input.search_mode.unwrap_or_else(|| "topic".to_string()),
        search_execution_mode: search_execution_mode.to_string(),
        provider_capability_mode: provider_capability_mode.to_string(),
    }
}
