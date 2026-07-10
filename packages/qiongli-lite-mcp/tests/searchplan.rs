use qiongli_lite_mcp::searchplan::{build_search_plan, normalize_identifier, SearchPlanInput};

#[test]
fn search_plan_builds_complete_hybrid_routes_with_provenance() {
    let plan = build_search_plan(SearchPlanInput {
        query: "platform governance".to_string(),
        search_mode: "review".to_string(),
        platform: "codex".to_string(),
        native_search_available: true,
        native_search_tools: Vec::new(),
        query_variants: vec!["digital platform governance".to_string()],
        include_working_papers: Some(true),
        from_year: Some(2020),
        to_year: Some(2026),
        venue_filter: Some("information systems".to_string()),
        document_types: vec!["journal-article".to_string()],
        active_providers: vec![
            "arxiv".to_string(),
            "openalex".to_string(),
            "semantic_scholar".to_string(),
        ],
    });

    assert_eq!(plan.artifact_type, "qiongli_hybrid_search_plan");
    assert_eq!(plan.search_execution_mode, "hybrid_search");
    assert_eq!(plan.provider_capability_mode, "provider_connected");
    assert_eq!(plan.native_search_tools, vec!["codex_web_search"]);
    assert_eq!(plan.provider_queries.len(), 6);
    assert_eq!(plan.native_search_queries.len(), 2);
    assert_eq!(plan.native_fulltext_queries.len(), 2);
    assert_eq!(plan.provider_queries[0].provider, "semantic_scholar");
    assert_eq!(plan.provider_queries[2].provider, "openalex");
    assert_eq!(plan.provider_queries[4].provider, "arxiv");
    assert_eq!(plan.provider_queries[0].query_id, "Q1");
    assert_eq!(plan.provider_queries[1].source, "variant");
    assert_eq!(plan.provider_queries[0].filters["from_year"], 2020);
    assert_eq!(plan.provider_queries[0].filters["fromYear"], 2020);
    assert_eq!(plan.provider_queries[0].filters["to_year"], 2026);
    assert_eq!(plan.provider_queries[0].filters["toYear"], 2026);
    assert_eq!(
        plan.native_search_queries[0].provenance_label,
        "native:codex_web_search"
    );
    assert_eq!(
        plan.native_fulltext_queries[0].candidate_status,
        "candidate_only"
    );
    assert!(plan.native_fulltext_queries[0]
        .query
        .contains("author manuscript"));
    assert_eq!(
        plan.provenance_labels.provider,
        vec!["mcp:semantic_scholar", "mcp:openalex", "mcp:arxiv"]
    );
    assert!(plan.limitations.is_empty());
    assert_eq!(
        plan.execution_sequence.last().unwrap()["action"],
        "merge/dedupe/search_log"
    );
}

#[test]
fn provider_only_plan_does_not_claim_native_queries() {
    let plan = build_search_plan(base_input(vec!["arxiv"], false));

    assert_eq!(plan.search_execution_mode, "provider_connected");
    assert!(!plan.native_search_available);
    assert!(plan.native_search_tools.is_empty());
    assert!(plan.native_search_queries.is_empty());
    assert!(plan.native_fulltext_queries.is_empty());
    assert_eq!(plan.provider_queries.len(), 1);
    assert_eq!(
        plan.limitations,
        vec!["Platform-native search was not declared available."]
    );
}

#[test]
fn native_only_and_strategy_only_modes_remain_distinct() {
    let native_only = build_search_plan(base_input(Vec::new(), true));
    let strategy_only = build_search_plan(base_input(Vec::new(), false));

    assert_eq!(native_only.search_execution_mode, "native_only");
    assert_eq!(native_only.provider_capability_mode, "strategy_only");
    assert_eq!(native_only.native_search_tools, vec!["codex_web_search"]);
    assert!(native_only.provider_queries.is_empty());
    assert_eq!(strategy_only.search_execution_mode, "strategy_only");
    assert!(strategy_only.provider_queries.is_empty());
    assert!(strategy_only.native_search_queries.is_empty());
    assert_eq!(strategy_only.execution_sequence.len(), 3);
}

#[test]
fn duplicate_query_variants_are_removed_without_reordering_unique_values() {
    let mut input = base_input(vec!["arxiv"], false);
    input.query_variants = vec![
        "Platform Governance".to_string(),
        "digital governance".to_string(),
        "DIGITAL GOVERNANCE".to_string(),
    ];

    let plan = build_search_plan(input);

    assert_eq!(plan.provider_queries.len(), 2);
    assert_eq!(plan.provider_queries[0].query, "platform governance");
    assert_eq!(plan.provider_queries[1].query, "digital governance");
    assert_eq!(plan.provider_queries[1].query_id, "Q2");
}

#[test]
fn platform_and_native_tool_identifiers_share_canonical_normalization() {
    assert_eq!(normalize_identifier(" Codex- "), "codex");
    assert_eq!(
        normalize_identifier("codex  web---search___"),
        "codex_web_search"
    );

    let mut default_tool = base_input(Vec::new(), true);
    default_tool.platform = "Codex-".to_string();
    let default_plan = build_search_plan(default_tool);
    assert_eq!(default_plan.platform, "codex");
    assert_eq!(default_plan.native_search_tools, vec!["codex_web_search"]);

    let mut explicit_tool = base_input(Vec::new(), true);
    explicit_tool.native_search_tools = vec![
        "codex  web search".to_string(),
        "CODEX--web__search_".to_string(),
    ];
    let explicit_plan = build_search_plan(explicit_tool);
    assert_eq!(explicit_plan.native_search_tools, vec!["codex_web_search"]);
}

fn base_input(active_providers: Vec<&str>, native_search_available: bool) -> SearchPlanInput {
    SearchPlanInput {
        query: "platform governance".to_string(),
        search_mode: "topic".to_string(),
        platform: "codex".to_string(),
        native_search_available,
        native_search_tools: Vec::new(),
        query_variants: Vec::new(),
        include_working_papers: None,
        from_year: None,
        to_year: None,
        venue_filter: None,
        document_types: Vec::new(),
        active_providers: active_providers.into_iter().map(str::to_string).collect(),
    }
}
