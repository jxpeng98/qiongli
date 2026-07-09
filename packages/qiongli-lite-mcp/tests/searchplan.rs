use qiongli_lite_mcp::searchplan::{build_search_plan, SearchPlanInput};

#[test]
fn search_plan_records_hybrid_mode_when_provider_and_native_are_available() {
    let plan = build_search_plan(SearchPlanInput {
        query: "platform governance".to_string(),
        search_mode: Some("review".to_string()),
        provider_connected: true,
        native_search_available: true,
    });

    assert_eq!(plan.search_execution_mode, "hybrid_search");
    assert_eq!(plan.provider_capability_mode, "provider_connected");
    assert_eq!(plan.native_search_queries, vec!["platform governance"]);
}

#[test]
fn provider_only_plan_does_not_claim_native_queries() {
    let plan = build_search_plan(SearchPlanInput {
        query: "platform governance".to_string(),
        search_mode: None,
        provider_connected: true,
        native_search_available: false,
    });

    assert_eq!(plan.search_execution_mode, "provider_connected");
    assert!(!plan.native_search_available);
    assert!(plan.native_search_queries.is_empty());
}
