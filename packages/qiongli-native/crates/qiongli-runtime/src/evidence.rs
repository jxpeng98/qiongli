use std::io::{self, Write};

use serde::Serialize;
use serde_json::{Map, Value};
use thiserror::Error;

const MAX_CONTEXT_CHARS: usize = 4_096;
const MAX_QUERY_BYTES: usize = 4_096;
const MAX_RESULTS: usize = 1_000;
const MAX_JSON_DEPTH: usize = 32;
const MAX_JSON_VALUES: usize = 100_000;
const MAX_EVIDENCE_INPUT_BYTES: usize = 2 * 1024 * 1024;

const ALLOWED_ARGUMENTS: [&str; 9] = [
    "cwd",
    "query",
    "provider_status",
    "search_plan",
    "results",
    "diagnostics",
    "query_plan",
    "search_results",
    "search_diagnostics",
];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Error)]
pub enum EvidenceError {
    #[error("evidence arguments must be an object")]
    ArgumentsNotObject,
    #[error("Unsupported argument")]
    UnsupportedArgument,
    #[error("cwd must be a string")]
    ContextNotString,
    #[error("cwd must not be empty")]
    EmptyContext,
    #[error("cwd exceeds the character limit")]
    ContextTooLong,
    #[error("query must be a string")]
    QueryNotString,
    #[error("query exceeds the byte limit")]
    QueryTooLong,
    #[error("provider_status must be an object")]
    ProviderStatusNotObject,
    #[error("search_plan must be an object")]
    SearchPlanNotObject,
    #[error("diagnostics must be an object")]
    DiagnosticsNotObject,
    #[error("results must be an array")]
    ResultsNotArray,
    #[error("results must contain objects")]
    ResultNotObject,
    #[error("canonical evidence fields cannot be combined with compatibility aliases")]
    AmbiguousAlias,
    #[error("evidence results exceed the record limit")]
    TooManyResults,
    #[error("evidence input exceeds the byte limit")]
    InputTooLarge,
    #[error("evidence input exceeds the nesting limit")]
    InputTooDeep,
    #[error("evidence input exceeds the value-count limit")]
    InputTooComplex,
    #[error("evidence input could not be serialized")]
    InvalidJson,
}

#[derive(Clone)]
pub struct EvidenceInput {
    query: String,
    provider_status: Value,
    search_plan: Value,
    diagnostics: Value,
    results: Vec<Value>,
}

impl EvidenceInput {
    pub fn from_arguments(arguments: &Value) -> Result<Self, EvidenceError> {
        let entries = arguments
            .as_object()
            .ok_or(EvidenceError::ArgumentsNotObject)?;
        if entries
            .keys()
            .any(|key| !ALLOWED_ARGUMENTS.contains(&key.as_str()))
        {
            return Err(EvidenceError::UnsupportedArgument);
        }
        validate_json_shape(arguments)?;
        validate_serialized_size(arguments)?;
        validate_context(entries.get("cwd"))?;

        let query = match entries.get("query") {
            Some(value) => {
                let query = value.as_str().ok_or(EvidenceError::QueryNotString)?;
                if query.len() > MAX_QUERY_BYTES {
                    return Err(EvidenceError::QueryTooLong);
                }
                query.to_owned()
            }
            None => String::new(),
        };

        let provider_status = object_field(
            entries,
            "provider_status",
            None,
            EvidenceError::ProviderStatusNotObject,
        )?;
        let search_plan = object_field(
            entries,
            "search_plan",
            Some("query_plan"),
            EvidenceError::SearchPlanNotObject,
        )?;
        let diagnostics = object_field(
            entries,
            "diagnostics",
            Some("search_diagnostics"),
            EvidenceError::DiagnosticsNotObject,
        )?;
        let results = results_field(entries)?;

        Ok(Self {
            query,
            provider_status,
            search_plan,
            diagnostics,
            results,
        })
    }
}

#[derive(Clone, Serialize, PartialEq)]
pub struct LiteratureEvidenceSnapshot {
    pub status: &'static str,
    pub artifact_type: &'static str,
    pub query: String,
    pub provider_status: Value,
    pub search_plan: Value,
    pub result_count: usize,
    pub results: Vec<Value>,
    pub diagnostics: Value,
}

#[must_use]
pub fn build_evidence_snapshot(input: EvidenceInput) -> LiteratureEvidenceSnapshot {
    let results = input
        .results
        .into_iter()
        .map(redact_credentials)
        .collect::<Vec<_>>();
    LiteratureEvidenceSnapshot {
        status: "ok",
        artifact_type: "qiongli_literature_evidence_snapshot",
        query: input.query,
        provider_status: redact_credentials(input.provider_status),
        search_plan: redact_credentials(input.search_plan),
        result_count: results.len(),
        results,
        diagnostics: redact_credentials(input.diagnostics),
    }
}

fn validate_context(value: Option<&Value>) -> Result<(), EvidenceError> {
    let Some(value) = value else {
        return Ok(());
    };
    let context = value.as_str().ok_or(EvidenceError::ContextNotString)?;
    if context.trim().is_empty() {
        return Err(EvidenceError::EmptyContext);
    }
    if context.chars().count() > MAX_CONTEXT_CHARS {
        return Err(EvidenceError::ContextTooLong);
    }
    Ok(())
}

fn object_field(
    entries: &Map<String, Value>,
    canonical: &str,
    alias: Option<&str>,
    type_error: EvidenceError,
) -> Result<Value, EvidenceError> {
    let value = select_field(entries, canonical, alias)?;
    match value {
        Some(value) if value.is_object() => Ok(value.clone()),
        Some(_) => Err(type_error),
        None => Ok(Value::Object(Map::new())),
    }
}

fn results_field(entries: &Map<String, Value>) -> Result<Vec<Value>, EvidenceError> {
    let Some(value) = select_field(entries, "results", Some("search_results"))? else {
        return Ok(Vec::new());
    };
    let values = value.as_array().ok_or(EvidenceError::ResultsNotArray)?;
    if values.len() > MAX_RESULTS {
        return Err(EvidenceError::TooManyResults);
    }
    if values.iter().any(|value| !value.is_object()) {
        return Err(EvidenceError::ResultNotObject);
    }
    Ok(values.clone())
}

fn select_field<'a>(
    entries: &'a Map<String, Value>,
    canonical: &str,
    alias: Option<&str>,
) -> Result<Option<&'a Value>, EvidenceError> {
    let canonical_value = entries.get(canonical);
    let alias_value = alias.and_then(|name| entries.get(name));
    if canonical_value.is_some() && alias_value.is_some() {
        return Err(EvidenceError::AmbiguousAlias);
    }
    Ok(canonical_value.or(alias_value))
}

fn validate_serialized_size(value: &Value) -> Result<(), EvidenceError> {
    let mut writer = BoundedWriter::new(MAX_EVIDENCE_INPUT_BYTES);
    match serde_json::to_writer(&mut writer, value) {
        Ok(()) => Ok(()),
        Err(_) if writer.exceeded => Err(EvidenceError::InputTooLarge),
        Err(_) => Err(EvidenceError::InvalidJson),
    }
}

fn validate_json_shape(root: &Value) -> Result<(), EvidenceError> {
    let mut stack = vec![(root, 0_usize)];
    let mut values = 0_usize;
    while let Some((value, depth)) = stack.pop() {
        values = values.saturating_add(1);
        if values > MAX_JSON_VALUES {
            return Err(EvidenceError::InputTooComplex);
        }
        match value {
            Value::Array(items) => {
                if depth >= MAX_JSON_DEPTH && !items.is_empty() {
                    return Err(EvidenceError::InputTooDeep);
                }
                if values
                    .saturating_add(stack.len())
                    .saturating_add(items.len())
                    > MAX_JSON_VALUES
                {
                    return Err(EvidenceError::InputTooComplex);
                }
                stack.extend(items.iter().map(|item| (item, depth + 1)));
            }
            Value::Object(entries) => {
                if depth >= MAX_JSON_DEPTH && !entries.is_empty() {
                    return Err(EvidenceError::InputTooDeep);
                }
                if values
                    .saturating_add(stack.len())
                    .saturating_add(entries.len())
                    > MAX_JSON_VALUES
                {
                    return Err(EvidenceError::InputTooComplex);
                }
                stack.extend(entries.values().map(|item| (item, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}

fn redact_credentials(value: Value) -> Value {
    match value {
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .filter_map(|(key, value)| {
                    (!credential_bearing_key(&key)).then(|| (key, redact_credentials(value)))
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(redact_credentials).collect()),
        value => value,
    }
}

fn credential_bearing_key(key: &str) -> bool {
    let normalized = key.trim().to_ascii_lowercase().replace(['-', ' '], "_");
    let compact = normalized.replace('_', "");
    let padded = format!("_{normalized}_");
    let has_sensitive_segment = normalized.split('_').any(|segment| {
        matches!(
            segment,
            "secret" | "password" | "passwd" | "credential" | "credentials" | "auth" | "bearer"
        )
    });
    let has_sensitive_marker = [
        "api_key",
        "access_key",
        "authorization",
        "cookie",
        "private_key",
        "client_secret",
        "access_token",
        "refresh_token",
        "auth_token",
        "id_token",
    ]
    .iter()
    .any(|marker| padded.contains(&format!("_{marker}_")));
    let has_sensitive_suffix = [
        "secret",
        "password",
        "passwd",
        "credential",
        "credentials",
        "authorization",
        "cookie",
        "bearer",
        "auth",
        "token",
        "apikey",
        "accesskey",
        "privatekey",
        "clientsecret",
    ]
    .iter()
    .any(|suffix| compact.ends_with(suffix));

    has_sensitive_segment
        || normalized == "token"
        || normalized == "authorization"
        || normalized.ends_with("_token")
        || has_sensitive_suffix
        || has_sensitive_marker
}

struct BoundedWriter {
    remaining: usize,
    exceeded: bool,
}

impl BoundedWriter {
    const fn new(limit: usize) -> Self {
        Self {
            remaining: limit,
            exceeded: false,
        }
    }
}

impl Write for BoundedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.len() > self.remaining {
            self.exceeded = true;
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "serialized evidence limit exceeded",
            ));
        }
        self.remaining -= buffer.len();
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn builds_canonical_and_compatibility_snapshots() {
        let canonical = EvidenceInput::from_arguments(&json!({
            "query": "governance",
            "provider_status": {"arxiv": "ready"},
            "search_plan": {"mode": "provider_connected"},
            "results": [{"title": "Paper"}],
            "diagnostics": {"status": "complete"}
        }))
        .unwrap();
        let canonical = build_evidence_snapshot(canonical);
        assert_eq!(canonical.query, "governance");
        assert_eq!(canonical.result_count, 1);
        assert_eq!(canonical.provider_status["arxiv"], "ready");

        let compatibility = EvidenceInput::from_arguments(&json!({
            "query_plan": {"mode": "legacy"},
            "search_results": [{"title": "Legacy"}],
            "search_diagnostics": {"status": "partial"}
        }))
        .unwrap();
        let compatibility = build_evidence_snapshot(compatibility);
        assert_eq!(compatibility.search_plan["mode"], "legacy");
        assert_eq!(compatibility.results[0]["title"], "Legacy");
        assert_eq!(compatibility.diagnostics["status"], "partial");
    }

    #[test]
    fn rejects_ambiguous_canonical_and_alias_fields() {
        let error = EvidenceInput::from_arguments(&json!({
            "search_plan": {},
            "query_plan": {}
        }))
        .err()
        .unwrap();
        assert_eq!(error, EvidenceError::AmbiguousAlias);
    }

    #[test]
    fn recursively_redacts_credentials_and_keeps_benign_keys() {
        const CANARY: &str = "direct-runtime-secret-canary";
        let input = EvidenceInput::from_arguments(&json!({
            "provider_status": {"api_key": CANARY, "arxiv": "ready"},
            "results": [{
                "title": "Paper",
                "metadata": {
                    "accessToken": CANARY,
                    "serviceAuthorization": CANARY,
                    "sessionCookie": CANARY,
                    "providerSecret": CANARY,
                    "serviceClientSecret": CANARY,
                    "token_budget": 2048,
                    "public_key": "kept"
                }
            }],
            "diagnostics": {"password": CANARY, "status": "complete"}
        }))
        .unwrap();
        let snapshot = build_evidence_snapshot(input);
        let rendered = serde_json::to_string(&snapshot).unwrap();
        assert!(!rendered.contains(CANARY));
        assert_eq!(snapshot.provider_status["arxiv"], "ready");
        assert_eq!(snapshot.results[0]["metadata"]["token_budget"], 2048);
        assert_eq!(snapshot.results[0]["metadata"]["public_key"], "kept");
        assert_eq!(snapshot.diagnostics["status"], "complete");
    }

    #[test]
    fn rejects_query_result_size_depth_and_complexity_overruns() {
        let query_error = EvidenceInput::from_arguments(&json!({
            "query": "x".repeat(MAX_QUERY_BYTES + 1)
        }))
        .err()
        .unwrap();
        assert_eq!(query_error, EvidenceError::QueryTooLong);

        let result_error = EvidenceInput::from_arguments(&json!({
            "results": vec![json!({}); MAX_RESULTS + 1]
        }))
        .err()
        .unwrap();
        assert_eq!(result_error, EvidenceError::TooManyResults);

        let size_error = EvidenceInput::from_arguments(&json!({
            "diagnostics": {"note": "x".repeat(MAX_EVIDENCE_INPUT_BYTES)}
        }))
        .err()
        .unwrap();
        assert_eq!(size_error, EvidenceError::InputTooLarge);

        let mut nested = json!({"leaf": true});
        for _ in 0..=MAX_JSON_DEPTH {
            nested = json!({"nested": nested});
        }
        let depth_error = EvidenceInput::from_arguments(&json!({
            "diagnostics": nested
        }))
        .err()
        .unwrap();
        assert_eq!(depth_error, EvidenceError::InputTooDeep);

        let complexity_error = EvidenceInput::from_arguments(&json!({
            "diagnostics": {"values": vec![Value::Null; MAX_JSON_VALUES]}
        }))
        .err()
        .unwrap();
        assert_eq!(complexity_error, EvidenceError::InputTooComplex);
    }

    #[test]
    fn ignores_valid_context_without_copying_it_to_output() {
        let input = EvidenceInput::from_arguments(&json!({
            "cwd": "/private/research/project",
            "query": "governance"
        }))
        .unwrap();
        let snapshot = build_evidence_snapshot(input);
        let rendered = serde_json::to_string(&snapshot).unwrap();
        assert!(!rendered.contains("/private/research/project"));
    }
}
