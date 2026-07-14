//! Deterministic provider/native-search planning shared by Rust entrypoints.

use std::collections::{BTreeMap, HashSet};
use std::fmt::{self, Display, Formatter};

use serde::Serialize;
use serde_json::{Value, json};

pub const PLAN_PROVIDER_ORDER: [&str; 5] = [
    "semantic_scholar",
    "openalex",
    "crossref",
    "pubmed",
    "arxiv",
];

const MAX_CONTEXT_LENGTH: usize = 4_096;
const MAX_QUERY_LENGTH: usize = 4_096;
const MAX_PLATFORM_LENGTH: usize = 64;
const MAX_NATIVE_TOOLS: usize = 8;
const MAX_QUERY_VARIANTS: usize = 16;
const MAX_DOCUMENT_TYPES: usize = 32;
const MAX_FILTER_VALUE_LENGTH: usize = 256;
const MIN_SEARCH_YEAR: u16 = 1_000;
const MAX_SEARCH_YEAR: u16 = 9_999;

const ALLOWED_ARGUMENTS: [&str; 22] = [
    "cwd",
    "query",
    "platform",
    "search_mode",
    "searchMode",
    "native_search_available",
    "native_search_usable",
    "nativeSearchAvailable",
    "native_search_tools",
    "nativeSearchTools",
    "query_variants",
    "queryVariants",
    "include_working_papers",
    "includeWorkingPapers",
    "from_year",
    "fromYear",
    "to_year",
    "toYear",
    "venue_filter",
    "venueFilter",
    "document_types",
    "documentTypes",
];

pub fn normalize_identifier(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_separator = false;
    for character in value.trim().chars() {
        if character.is_whitespace() || matches!(character, '-' | '_') {
            if !normalized.is_empty() && !previous_separator {
                normalized.push('_');
            }
            previous_separator = true;
        } else {
            normalized.extend(character.to_lowercase());
            previous_separator = false;
        }
    }
    if normalized.ends_with('_') {
        normalized.pop();
    }
    normalized
}

const AGENT_INSTRUCTIONS: [&str; 6] = [
    "MCP servers must not call Codex or Claude native search directly.",
    "The active agent executes native_search_queries only when the platform exposes native search.",
    "Do not treat native-search results as provider-reproducible records.",
    "Write provider, native, and user-corpus records with distinct provenance labels.",
    "Use native_fulltext_queries only to discover candidate URLs; do not mark full text as retrieved from search snippets.",
    "Write native_fulltext_candidates with candidate_only status until retrieval_manifest.csv verifies readable text.",
];

#[derive(Debug, Clone)]
pub struct SearchPlanInput {
    pub query: String,
    pub search_mode: String,
    pub platform: String,
    pub native_search_available: bool,
    pub native_search_tools: Vec<String>,
    pub query_variants: Vec<String>,
    pub include_working_papers: Option<bool>,
    pub from_year: Option<u16>,
    pub to_year: Option<u16>,
    pub venue_filter: Option<String>,
    pub document_types: Vec<String>,
    pub active_providers: Vec<String>,
}

impl SearchPlanInput {
    pub fn from_arguments(
        arguments: &Value,
        active_providers: Vec<String>,
    ) -> Result<Self, SearchPlanInputError> {
        let entries = arguments
            .as_object()
            .ok_or_else(|| SearchPlanInputError::new("search plan arguments must be an object"))?;
        if entries
            .keys()
            .any(|key| !ALLOWED_ARGUMENTS.contains(&key.as_str()))
        {
            return Err(SearchPlanInputError::new("Unsupported argument"));
        }
        validate_optional_context(arguments)?;
        let query = required_bounded_string(arguments, "query", MAX_QUERY_LENGTH)?;
        let platform = optional_alias_string(
            arguments,
            &["platform"],
            "platform",
            MAX_PLATFORM_LENGTH,
            false,
        )?;
        let platform = match platform {
            Some(value) if !valid_platform_identifier(&value) => {
                return Err(SearchPlanInputError::new(
                    "platform must be an ASCII identifier",
                ));
            }
            Some(value) => normalize_identifier(&value),
            None => "unknown".to_string(),
        };
        let native_search_available = optional_alias_bool(
            arguments,
            &[
                "native_search_available",
                "native_search_usable",
                "nativeSearchAvailable",
            ],
            "native_search_available",
        )?
        .unwrap_or(false);
        let native_search_tools = optional_alias_string_list(
            arguments,
            &["native_search_tools", "nativeSearchTools"],
            "native_search_tools",
            MAX_NATIVE_TOOLS,
            MAX_FILTER_VALUE_LENGTH,
            true,
        )?
        .unwrap_or_default();
        let query_variants = optional_alias_string_list(
            arguments,
            &["query_variants", "queryVariants"],
            "query_variants",
            MAX_QUERY_VARIANTS,
            MAX_QUERY_LENGTH,
            false,
        )?
        .unwrap_or_default();
        let include_working_papers = optional_alias_bool(
            arguments,
            &["include_working_papers", "includeWorkingPapers"],
            "include_working_papers",
        )?;
        let from_year = optional_alias_year(arguments, &["from_year", "fromYear"], "from_year")?;
        let to_year = optional_alias_year(arguments, &["to_year", "toYear"], "to_year")?;
        if from_year.zip(to_year).is_some_and(|(from, to)| from > to) {
            return Err(SearchPlanInputError::new(
                "from_year must be less than or equal to to_year",
            ));
        }
        let search_mode =
            optional_alias_search_mode(arguments, &["search_mode", "searchMode"], "search_mode")?
                .unwrap_or_else(|| "topic".to_string());
        let venue_filter = optional_alias_string(
            arguments,
            &["venue_filter", "venueFilter"],
            "venue_filter",
            MAX_FILTER_VALUE_LENGTH,
            true,
        )?
        .filter(|value| !value.is_empty());
        let document_types = optional_alias_string_list(
            arguments,
            &["document_types", "documentTypes"],
            "document_types",
            MAX_DOCUMENT_TYPES,
            MAX_FILTER_VALUE_LENGTH,
            false,
        )?
        .unwrap_or_default();

        Ok(Self {
            query,
            search_mode,
            platform,
            native_search_available,
            native_search_tools,
            query_variants,
            include_working_papers,
            from_year,
            to_year,
            venue_filter,
            document_types,
            active_providers,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchPlanInputError {
    message: String,
}

impl SearchPlanInputError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for SearchPlanInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SearchPlanInputError {}

fn validate_optional_context(arguments: &Value) -> Result<(), SearchPlanInputError> {
    let Some(value) = arguments.get("cwd") else {
        return Ok(());
    };
    let cwd = value
        .as_str()
        .ok_or_else(|| SearchPlanInputError::new("cwd must be a string"))?;
    if cwd.trim().is_empty() {
        return Err(SearchPlanInputError::new("cwd must not be empty"));
    }
    if cwd.chars().count() > MAX_CONTEXT_LENGTH {
        return Err(SearchPlanInputError::new(format!(
            "cwd must be at most {MAX_CONTEXT_LENGTH} characters"
        )));
    }
    Ok(())
}

fn required_bounded_string(
    arguments: &Value,
    name: &str,
    maximum: usize,
) -> Result<String, SearchPlanInputError> {
    let value = arguments
        .get(name)
        .ok_or_else(|| SearchPlanInputError::new(format!("Missing {name}")))?
        .as_str()
        .ok_or_else(|| SearchPlanInputError::new(format!("{name} must be a string")))?;
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SearchPlanInputError::new(format!(
            "{name} must not be empty"
        )));
    }
    if value.chars().count() > maximum {
        return Err(SearchPlanInputError::new(format!(
            "{name} must be at most {maximum} characters"
        )));
    }
    Ok(normalized.to_string())
}

fn one_alias_value<'a>(
    arguments: &'a Value,
    names: &[&str],
    canonical_name: &str,
) -> Result<Option<&'a Value>, SearchPlanInputError> {
    let mut found = None;
    for name in names {
        if let Some(value) = arguments.get(name) {
            if found.is_some() {
                return Err(SearchPlanInputError::new(format!(
                    "conflicting aliases for {canonical_name}"
                )));
            }
            found = Some(value);
        }
    }
    Ok(found)
}

fn optional_alias_bool(
    arguments: &Value,
    names: &[&str],
    canonical_name: &str,
) -> Result<Option<bool>, SearchPlanInputError> {
    one_alias_value(arguments, names, canonical_name)?
        .map(|value| {
            value.as_bool().ok_or_else(|| {
                SearchPlanInputError::new(format!("{canonical_name} must be a boolean"))
            })
        })
        .transpose()
}

fn optional_alias_string(
    arguments: &Value,
    names: &[&str],
    canonical_name: &str,
    maximum: usize,
    allow_empty: bool,
) -> Result<Option<String>, SearchPlanInputError> {
    one_alias_value(arguments, names, canonical_name)?
        .map(|value| {
            let value = value.as_str().ok_or_else(|| {
                SearchPlanInputError::new(format!("{canonical_name} must be a string"))
            })?;
            if value.chars().count() > maximum {
                return Err(SearchPlanInputError::new(format!(
                    "{canonical_name} must be at most {maximum} characters"
                )));
            }
            let normalized = value.trim();
            if !allow_empty && normalized.is_empty() {
                return Err(SearchPlanInputError::new(format!(
                    "{canonical_name} must not be empty"
                )));
            }
            Ok(normalized.to_string())
        })
        .transpose()
}

fn valid_platform_identifier(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphanumeric())
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '_' | '-')
        })
}

fn optional_alias_string_list(
    arguments: &Value,
    names: &[&str],
    canonical_name: &str,
    maximum_items: usize,
    maximum_item_length: usize,
    normalize_tools: bool,
) -> Result<Option<Vec<String>>, SearchPlanInputError> {
    let Some(value) = one_alias_value(arguments, names, canonical_name)? else {
        return Ok(None);
    };
    let values = value
        .as_array()
        .ok_or_else(|| SearchPlanInputError::new(format!("{canonical_name} must be an array")))?;
    if values.len() > maximum_items {
        return Err(SearchPlanInputError::new(format!(
            "{canonical_name} must contain at most {maximum_items} items"
        )));
    }
    let mut normalized = Vec::with_capacity(values.len());
    let mut raw_values = Vec::with_capacity(values.len());
    for value in values {
        let raw_value = value.as_str().ok_or_else(|| {
            SearchPlanInputError::new(format!("{canonical_name} must contain strings"))
        })?;
        if raw_value.chars().count() > maximum_item_length {
            return Err(SearchPlanInputError::new(format!(
                "{canonical_name} items must be at most {maximum_item_length} characters"
            )));
        }
        if raw_values.contains(&raw_value) {
            return Err(SearchPlanInputError::new(format!(
                "{canonical_name} must contain unique values"
            )));
        }
        raw_values.push(raw_value);
        let value = raw_value.trim();
        if value.is_empty() {
            return Err(SearchPlanInputError::new(format!(
                "{canonical_name} must not contain empty values"
            )));
        }
        let value = if normalize_tools {
            normalize_identifier(value)
        } else {
            value.to_string()
        };
        if value.is_empty() {
            return Err(SearchPlanInputError::new(format!(
                "{canonical_name} must contain valid identifiers"
            )));
        }
        let duplicate_key = value.to_lowercase();
        if normalized
            .iter()
            .any(|candidate: &String| candidate.to_lowercase() == duplicate_key)
        {
            return Err(SearchPlanInputError::new(format!(
                "{canonical_name} must contain unique values"
            )));
        }
        normalized.push(value);
    }
    Ok(Some(normalized))
}

fn optional_alias_year(
    arguments: &Value,
    names: &[&str],
    canonical_name: &str,
) -> Result<Option<u16>, SearchPlanInputError> {
    let Some(value) = one_alias_value(arguments, names, canonical_name)? else {
        return Ok(None);
    };
    let parsed = if let Some(value) = value.as_u64() {
        u16::try_from(value).ok()
    } else if let Some(value) = value.as_str() {
        (value.len() == 4 && value.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| value.parse::<u16>().ok())
            .flatten()
    } else {
        None
    }
    .ok_or_else(|| {
        SearchPlanInputError::new(format!("{canonical_name} must be a four-digit year"))
    })?;
    if !(MIN_SEARCH_YEAR..=MAX_SEARCH_YEAR).contains(&parsed) {
        return Err(SearchPlanInputError::new(format!(
            "{canonical_name} must be between {MIN_SEARCH_YEAR} and {MAX_SEARCH_YEAR}"
        )));
    }
    Ok(Some(parsed))
}

fn optional_alias_search_mode(
    arguments: &Value,
    names: &[&str],
    canonical_name: &str,
) -> Result<Option<String>, SearchPlanInputError> {
    let Some(value) = one_alias_value(arguments, names, canonical_name)? else {
        return Ok(None);
    };
    let value = value
        .as_str()
        .ok_or_else(|| SearchPlanInputError::new(format!("{canonical_name} must be a string")))?;
    if ![
        "auto",
        "topic",
        "title",
        "doi",
        "review",
        "systematic_review",
    ]
    .contains(&value)
    {
        return Err(SearchPlanInputError::new("unsupported search_mode"));
    }
    Ok(Some(value.to_string()))
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProviderQuery {
    pub provider: String,
    pub query_id: String,
    pub query: String,
    pub source: String,
    pub filters: BTreeMap<String, Value>,
    pub provenance_label: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NativeQuery {
    pub tool: String,
    pub platform: String,
    pub query_id: String,
    pub query: String,
    pub source: String,
    pub filters: BTreeMap<String, Value>,
    pub provenance_label: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NativeFulltextQuery {
    pub tool: String,
    pub platform: String,
    pub query_id: String,
    pub query: String,
    pub source: String,
    pub purpose: String,
    pub candidate_status: String,
    pub filters: BTreeMap<String, Value>,
    pub expected_candidate_fields: Vec<String>,
    pub provenance_label: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NativeFulltextCandidateSchema {
    pub artifact_type: String,
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub status_values: Vec<String>,
    pub evidence_rule: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProvenanceLabels {
    pub provider: Vec<String>,
    pub native: Vec<String>,
    pub user_corpus: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MergePolicy {
    pub dedupe_keys: Vec<String>,
    pub provider_records: String,
    pub native_records: String,
    pub fulltext_candidate_records: String,
    pub user_corpus_records: String,
    pub search_log: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchPlan {
    pub artifact_type: String,
    pub query: String,
    pub search_mode: String,
    pub platform: String,
    pub search_execution_mode: String,
    pub provider_capability_mode: String,
    pub native_search_available: bool,
    pub native_search_tools: Vec<String>,
    pub provider_queries: Vec<ProviderQuery>,
    pub native_search_queries: Vec<NativeQuery>,
    pub native_fulltext_queries: Vec<NativeFulltextQuery>,
    pub native_fulltext_candidate_schema: NativeFulltextCandidateSchema,
    pub provenance_labels: ProvenanceLabels,
    pub execution_sequence: Vec<Value>,
    pub agent_instructions: Vec<String>,
    pub merge_policy: MergePolicy,
    pub limitations: Vec<String>,
}

pub fn build_search_plan(input: SearchPlanInput) -> SearchPlan {
    let active_providers = ordered_active_providers(&input.active_providers);
    let provider_connected = !active_providers.is_empty();
    let platform = match normalize_identifier(&input.platform) {
        value if value.is_empty() => "unknown".to_string(),
        value => value,
    };
    let native_search_tools = if input.native_search_available {
        let tools = normalized_unique_identifiers(&input.native_search_tools);
        if tools.is_empty() {
            vec![default_native_search_tool(&platform).to_string()]
        } else {
            tools
        }
    } else {
        Vec::new()
    };
    let search_execution_mode = match (provider_connected, input.native_search_available) {
        (true, true) => "hybrid_search",
        (true, false) => "provider_connected",
        (false, true) => "native_only",
        (false, false) => "strategy_only",
    };
    let provider_capability_mode = if provider_connected {
        "provider_connected"
    } else {
        "strategy_only"
    };
    let query_entries = query_entries(&input.query, &input.query_variants);
    let filters = search_filters(&input);

    let mut provider_queries = Vec::new();
    if provider_connected {
        for provider in &active_providers {
            for entry in &query_entries {
                provider_queries.push(ProviderQuery {
                    provider: provider.clone(),
                    query_id: entry.query_id.clone(),
                    query: entry.query.clone(),
                    source: entry.source.clone(),
                    filters: filters.clone(),
                    provenance_label: format!("mcp:{provider}"),
                });
            }
        }
    }
    let mut native_search_queries = Vec::new();
    if input.native_search_available {
        for tool in &native_search_tools {
            for entry in &query_entries {
                native_search_queries.push(NativeQuery {
                    tool: tool.clone(),
                    platform: platform.clone(),
                    query_id: entry.query_id.clone(),
                    query: entry.query.clone(),
                    source: entry.source.clone(),
                    filters: filters.clone(),
                    provenance_label: format!("native:{tool}"),
                });
            }
        }
    }
    let mut native_fulltext_queries = Vec::new();
    if input.native_search_available {
        for tool in &native_search_tools {
            for entry in &query_entries {
                native_fulltext_queries.push(NativeFulltextQuery {
                    tool: tool.clone(),
                    platform: platform.clone(),
                    query_id: entry.query_id.clone(),
                    query: fulltext_candidate_query(&entry.query),
                    source: entry.source.clone(),
                    purpose: "fulltext_candidate_discovery".to_string(),
                    candidate_status: "candidate_only".to_string(),
                    filters: filters.clone(),
                    expected_candidate_fields: expected_candidate_fields(),
                    provenance_label: format!("native:{tool}"),
                });
            }
        }
    }
    let limitations = match (provider_connected, input.native_search_available) {
        (true, true) => Vec::new(),
        (true, false) => vec!["Platform-native search was not declared available.".to_string()],
        (false, true) => vec![
            "Provider MCP search is unavailable; native results require explicit provenance labels."
                .to_string(),
        ],
        (false, false) => {
            vec!["No provider MCP search or platform-native search is available.".to_string()]
        }
    };

    SearchPlan {
        artifact_type: "qiongli_hybrid_search_plan".to_string(),
        query: input.query,
        search_mode: input.search_mode,
        platform,
        search_execution_mode: search_execution_mode.to_string(),
        provider_capability_mode: provider_capability_mode.to_string(),
        native_search_available: input.native_search_available,
        native_search_tools: native_search_tools.clone(),
        provider_queries,
        native_search_queries: native_search_queries.clone(),
        native_fulltext_queries: native_fulltext_queries.clone(),
        native_fulltext_candidate_schema: native_fulltext_candidate_schema(),
        provenance_labels: ProvenanceLabels {
            provider: active_providers
                .iter()
                .map(|provider| format!("mcp:{provider}"))
                .collect(),
            native: native_search_tools
                .iter()
                .map(|tool| format!("native:{tool}"))
                .collect(),
            user_corpus: vec!["user_corpus".to_string()],
        },
        execution_sequence: execution_sequence(
            &native_search_queries,
            &native_fulltext_queries,
            provider_connected,
        ),
        agent_instructions: AGENT_INSTRUCTIONS
            .iter()
            .map(|instruction| (*instruction).to_string())
            .collect(),
        merge_policy: merge_policy(),
        limitations,
    }
}

#[derive(Debug, Clone)]
struct QueryEntry {
    query_id: String,
    query: String,
    source: String,
}

fn query_entries(query: &str, variants: &[String]) -> Vec<QueryEntry> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for candidate in std::iter::once(query).chain(variants.iter().map(String::as_str)) {
        let key = candidate.to_lowercase();
        if !seen.insert(key) {
            continue;
        }
        entries.push(QueryEntry {
            query_id: format!("Q{}", entries.len() + 1),
            query: candidate.to_string(),
            source: if entries.is_empty() {
                "primary".to_string()
            } else {
                "variant".to_string()
            },
        });
    }
    entries
}

fn ordered_active_providers(providers: &[String]) -> Vec<String> {
    PLAN_PROVIDER_ORDER
        .iter()
        .filter(|provider| providers.iter().any(|candidate| candidate == *provider))
        .map(|provider| (*provider).to_string())
        .collect()
}

fn normalized_unique_identifiers(values: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .iter()
        .map(|value| normalize_identifier(value))
        .filter(|value| !value.is_empty())
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn search_filters(input: &SearchPlanInput) -> BTreeMap<String, Value> {
    let mut filters = BTreeMap::new();
    filters.insert("search_mode".to_string(), json!(input.search_mode));
    if let Some(value) = input.include_working_papers {
        filters.insert("include_working_papers".to_string(), json!(value));
    }
    if let Some(value) = input.from_year {
        filters.insert("from_year".to_string(), json!(value));
        filters.insert("fromYear".to_string(), json!(value));
    }
    if let Some(value) = input.to_year {
        filters.insert("to_year".to_string(), json!(value));
        filters.insert("toYear".to_string(), json!(value));
    }
    if let Some(value) = input.venue_filter.as_ref() {
        filters.insert("venue_filter".to_string(), json!(value));
    }
    if !input.document_types.is_empty() {
        filters.insert("document_types".to_string(), json!(input.document_types));
    }
    filters
}

fn default_native_search_tool(platform: &str) -> &'static str {
    match platform {
        "codex" => "codex_web_search",
        "claude" | "claude_code" | "claudecode" => "claude_web_search",
        "antigravity" => "antigravity_search",
        _ => "platform_native_search",
    }
}

fn fulltext_candidate_query(query: &str) -> String {
    format!(
        "{query} (PDF OR \"full text\" OR preprint OR \"author manuscript\" OR repository OR PMC OR arXiv)"
    )
}

fn expected_candidate_fields() -> Vec<String> {
    [
        "query_id",
        "source_agent",
        "url",
        "title",
        "doi",
        "access_type",
        "snippet",
        "candidate_status",
        "retrieved_at",
    ]
    .iter()
    .map(|field| (*field).to_string())
    .collect()
}

fn native_fulltext_candidate_schema() -> NativeFulltextCandidateSchema {
    NativeFulltextCandidateSchema {
        artifact_type: "qiongli_native_fulltext_candidate_schema".to_string(),
        required: [
            "query_id",
            "source_agent",
            "url",
            "title",
            "candidate_status",
            "retrieved_at",
        ]
        .iter()
        .map(|field| (*field).to_string())
        .collect(),
        optional: ["doi", "access_type", "snippet", "license", "version_label"]
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        status_values: vec!["candidate_only".to_string()],
        evidence_rule: "Search snippets and URLs do not prove retrieved full text. Upgrade only through retrieval_manifest.csv."
            .to_string(),
    }
}

fn execution_sequence(
    native_queries: &[NativeQuery],
    native_fulltext_queries: &[NativeFulltextQuery],
    provider_connected: bool,
) -> Vec<Value> {
    let mut sequence = vec![
        json!({
            "actor": "agent",
            "action": "call qiongli_literature_status",
            "tool": "qiongli_literature_status"
        }),
        json!({
            "actor": "agent",
            "action": "call qiongli_search_plan",
            "tool": "qiongli_search_plan"
        }),
    ];
    if provider_connected {
        sequence.push(json!({
            "actor": "agent",
            "action": "call qiongli_literature_search",
            "tool": "qiongli_literature_search",
            "queries": "provider_queries"
        }));
    }
    if !native_queries.is_empty() {
        sequence.push(json!({
            "actor": "agent",
            "action": "execute platform-native search",
            "queries": "native_search_queries"
        }));
    }
    if !native_fulltext_queries.is_empty() {
        sequence.push(json!({
            "actor": "agent",
            "action": "execute platform-native full-text candidate search",
            "queries": "native_fulltext_queries"
        }));
    }
    sequence.push(json!({
        "actor": "agent",
        "action": "merge/dedupe/search_log",
        "inputs": [
            "provider_queries",
            "native_search_queries",
            "native_fulltext_candidates",
            "user_corpus"
        ]
    }));
    sequence
}

fn merge_policy() -> MergePolicy {
    MergePolicy {
        dedupe_keys: ["doi", "title", "year", "provider_record_id", "native_url"]
            .iter()
            .map(|key| (*key).to_string())
            .collect(),
        provider_records:
            "Prefer provider MCP metadata for reproducible bibliographic fields.".to_string(),
        native_records:
            "Keep native-search records only with native provenance labels and source URLs."
                .to_string(),
        fulltext_candidate_records: "Keep native full-text search outputs as candidate_only until retrieval_manifest.csv verifies readable text."
            .to_string(),
        user_corpus_records:
            "Keep user-corpus records separate from provider and native search records."
                .to_string(),
        search_log:
            "Record provider and native query execution separately before merge and dedupe."
                .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argument_parser_normalizes_aliases_and_retains_year_compatibility() {
        let input = SearchPlanInput::from_arguments(
            &json!({
                "cwd": "/project",
                "query": " governance ",
                "platform": "Claude Code",
                "nativeSearchAvailable": true,
                "nativeSearchTools": [" Web Search "],
                "queryVariants": ["institutions"],
                "includeWorkingPapers": true,
                "from_year": 2020,
                "toYear": "2026",
                "searchMode": "review",
                "venueFilter": "journal",
                "documentTypes": ["article"]
            }),
            vec!["arxiv".to_string()],
        )
        .unwrap();
        assert_eq!(input.query, "governance");
        assert_eq!(input.platform, "claude_code");
        assert_eq!(input.native_search_tools, ["web_search"]);
        assert_eq!(input.from_year, Some(2020));
        assert_eq!(input.to_year, Some(2026));

        let plan = build_search_plan(input);
        assert_eq!(
            plan.provider_queries[0].filters["from_year"],
            plan.provider_queries[0].filters["fromYear"]
        );
        assert_eq!(
            plan.provider_queries[0].filters["to_year"],
            plan.provider_queries[0].filters["toYear"]
        );
    }

    #[test]
    fn argument_parser_rejects_shape_alias_bounds_and_unknowns_without_echo() {
        const CANARY: &str = "private-search-plan-canary";
        let mut unknown = json!({"query": "topic"});
        unknown
            .as_object_mut()
            .unwrap()
            .insert(CANARY.to_string(), json!(true));
        for arguments in [
            json!([]),
            json!({}),
            json!({"query": " "}),
            json!({"query": "topic", "from_year": 2026, "to_year": 2020}),
            json!({
                "query": "topic",
                "native_search_available": true,
                "nativeSearchAvailable": true
            }),
            json!({"query": "topic", "query_variants": vec!["x"; 17]}),
            unknown,
        ] {
            let error = SearchPlanInput::from_arguments(&arguments, Vec::new()).unwrap_err();
            assert!(!error.to_string().contains(CANARY));
        }
    }
}
