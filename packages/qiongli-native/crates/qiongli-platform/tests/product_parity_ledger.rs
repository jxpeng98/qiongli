use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

const REQUIRED_CAPABILITY_IDS: [&str; 16] = [
    "discovery-install-check",
    "doctor-product-health",
    "install-cli-wrapper",
    "install-copy-link-mode",
    "install-default-client-skills-paths",
    "install-global-project-skills",
    "install-local-plugin-lifecycle",
    "install-mcp-registration",
    "install-surface-selection",
    "install-target-auto-all",
    "orchestration-external-workers",
    "orchestration-full-doctor-run",
    "remove-receipt-owned-installation",
    "setup-literature-provider",
    "setup-subject-coverage-profile",
    "update-native-product",
];
const REQUIRED_DEFERRED_CAPABILITY_IDS: [&str; 6] = [
    "install-copy-link-mode",
    "install-global-project-skills",
    "install-surface-selection",
    "orchestration-external-workers",
    "orchestration-full-doctor-run",
    "setup-subject-coverage-profile",
];
const REQUIRED_BASELINE_DOMAINS: [&str; 6] = [
    "cli",
    "installers",
    "mcp",
    "mutable-state",
    "orchestrator-scenarios",
    "skills",
];
const REQUIRED_PYTHON_FULL_COVERAGE: [&str; 5] = [
    "cli-command",
    "installer-dry-run",
    "mcp-initialize-and-list",
    "mutable-provider-state",
    "orchestration-preview",
];
const ACCEPTANCE_EVIDENCE_PATTERN: &str =
    "^[A-Za-z0-9][A-Za-z0-9._+-]*(?:/[A-Za-z0-9][A-Za-z0-9._+-]*)*\\.rs#[a-z][a-z0-9_]*$";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParityLedger {
    #[serde(rename = "$schema")]
    schema: String,
    schema_version: String,
    record_type: String,
    source_release: String,
    baseline_plan: String,
    classification_status: ClassificationStatus,
    capabilities: Vec<Capability>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum ClassificationStatus {
    Tracked,
    Complete,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Capability {
    id: String,
    category: CapabilityCategory,
    source: Vec<String>,
    observed_outcome: String,
    target_surfaces: Vec<TargetSurface>,
    baseline_domains: Vec<String>,
    baseline_coverage: Vec<String>,
    disposition: Disposition,
    owning_batch: OwningBatch,
    implementation_evidence: Vec<String>,
    acceptance_evidence: Vec<String>,
    nonclaim: Option<String>,
    retirement_reason: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CapabilityCategory {
    Install,
    Setup,
    Discovery,
    Doctor,
    Update,
    Remove,
    Orchestration,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TargetSurface {
    NativeCli,
    DesktopApp,
    Codex,
    ClaudeCode,
    LiteMcp,
    FullOrchestrator,
}

#[derive(Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Disposition {
    Retain,
    Replace,
    DeferToR4,
    RetireWithReason,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum OwningBatch {
    R3qA,
    R3qB,
    R3qC,
    R3qD,
    R3qE,
    R4,
}

#[derive(Deserialize)]
struct BaselinePlan {
    release_lineage: ReleaseLineage,
    oracles: Vec<Oracle>,
    inventory: BaselineInventory,
}

#[derive(Deserialize)]
struct ReleaseLineage {
    source_release: String,
}

#[derive(Deserialize)]
struct Oracle {
    id: String,
    required_coverage: Vec<String>,
}

#[derive(Deserialize)]
struct BaselineInventory {
    domains: Vec<BaselineDomain>,
}

#[derive(Deserialize)]
struct BaselineDomain {
    id: String,
}

#[test]
fn accepted_1x_product_outcomes_have_explicit_2x_dispositions() {
    let root = repository_root();
    let ledger_path = root.join("tooling/migration/qiongli-1x-product-parity.json");
    let ledger: ParityLedger = parse_json(&ledger_path);

    assert_eq!(ledger.schema, "./qiongli-1x-product-parity.schema.json");
    assert_eq!(ledger.schema_version, "1.1");
    assert_eq!(ledger.record_type, "qiongli-1x-product-outcome-parity");
    assert_eq!(
        ledger.classification_status,
        ClassificationStatus::Complete,
        "classification completion must not be reported as implementation completion"
    );

    let baseline_path = root.join(&ledger.baseline_plan);
    let baseline: BaselinePlan = parse_json(&baseline_path);
    assert_eq!(
        ledger.source_release,
        baseline.release_lineage.source_release
    );

    let capability_ids = unique_set(ledger.capabilities.iter().map(|item| item.id.as_str()));
    assert_eq!(
        capability_ids,
        REQUIRED_CAPABILITY_IDS.into_iter().collect(),
        "the parity ledger must classify every accepted 1.x product outcome exactly once"
    );

    for capability in &ledger.capabilities {
        assert!(!capability.observed_outcome.trim().is_empty());
        assert!(!capability.source.is_empty());
        assert!(!capability.target_surfaces.is_empty());
        assert!(!capability.baseline_domains.is_empty());
        match capability.disposition {
            Disposition::DeferToR4 => {
                assert!(
                    capability
                        .nonclaim
                        .as_deref()
                        .is_some_and(|value| !value.trim().is_empty()),
                    "{} must state its R3Q nonclaim",
                    capability.id
                );
                assert!(
                    matches!(capability.owning_batch, OwningBatch::R4),
                    "{} is deferred but is not owned by R4",
                    capability.id
                );
            }
            Disposition::RetireWithReason => assert!(
                capability
                    .retirement_reason
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "{} must explain retirement",
                capability.id
            ),
            Disposition::Retain | Disposition::Replace => {
                assert!(
                    !capability.implementation_evidence.is_empty(),
                    "{} must identify direct implementation evidence",
                    capability.id
                );
                assert!(
                    !capability.acceptance_evidence.is_empty(),
                    "{} must identify direct acceptance evidence",
                    capability.id
                );
            }
        }
        for source in &capability.source {
            assert!(
                root.join(source).is_file(),
                "missing ledger source: {source}"
            );
        }
        for evidence in &capability.implementation_evidence {
            assert!(
                root.join(evidence).is_file(),
                "missing ledger evidence: {evidence}"
            );
        }
        for evidence in &capability.acceptance_evidence {
            let Some((repository_path, scenario)) = parse_acceptance_evidence(evidence) else {
                panic!(
                    "acceptance evidence must use repository/path.rs#test_or_scenario_name: {evidence}"
                );
            };
            let evidence_path = root.join(repository_path);
            assert!(
                evidence_path.is_file(),
                "missing ledger evidence: {evidence}"
            );
            assert!(
                acceptance_symbol_is_executable(&evidence_path, repository_path, scenario),
                "acceptance evidence must resolve to the exact executable test or scenario: {evidence}"
            );
        }
        assert_eq!(
            unique_set(
                capability
                    .implementation_evidence
                    .iter()
                    .map(String::as_str)
            )
            .len(),
            capability.implementation_evidence.len(),
            "{} repeats implementation evidence",
            capability.id
        );
        assert_eq!(
            unique_set(capability.acceptance_evidence.iter().map(String::as_str)).len(),
            capability.acceptance_evidence.len(),
            "{} repeats acceptance evidence",
            capability.id
        );
        let _ = (&capability.category, &capability.owning_batch);
    }

    let deferred_capability_ids = collect_set(
        ledger
            .capabilities
            .iter()
            .filter(|capability| matches!(capability.disposition, Disposition::DeferToR4))
            .map(|capability| capability.id.as_str()),
    );
    assert_eq!(
        deferred_capability_ids,
        REQUIRED_DEFERRED_CAPABILITY_IDS.into_iter().collect(),
        "classification completeness must preserve every deferred capability"
    );

    let baseline_domains = unique_set(
        baseline
            .inventory
            .domains
            .iter()
            .map(|item| item.id.as_str()),
    );
    let ledger_domains = collect_set(
        ledger
            .capabilities
            .iter()
            .flat_map(|item| item.baseline_domains.iter().map(String::as_str)),
    );
    for domain in REQUIRED_BASELINE_DOMAINS {
        assert!(baseline_domains.contains(domain));
        assert!(
            ledger_domains.contains(domain),
            "baseline domain {domain} is unclassified"
        );
    }

    let python_full = baseline
        .oracles
        .iter()
        .find(|oracle| oracle.id == "python-full")
        .expect("accepted baseline must retain the python-full oracle");
    let oracle_coverage = unique_set(python_full.required_coverage.iter().map(String::as_str));
    assert_eq!(
        oracle_coverage,
        REQUIRED_PYTHON_FULL_COVERAGE.into_iter().collect()
    );
    let ledger_coverage = collect_set(
        ledger
            .capabilities
            .iter()
            .flat_map(|item| item.baseline_coverage.iter().map(String::as_str)),
    );
    assert_eq!(ledger_coverage, oracle_coverage);
}

#[test]
fn parity_schema_requires_evidence_for_retained_and_replaced_outcomes() {
    let root = repository_root();
    let schema_path = root.join("tooling/migration/qiongli-1x-product-parity.schema.json");
    let schema: serde_json::Value = parse_json(&schema_path);
    let required = schema["required"]
        .as_array()
        .expect("parity schema must declare required root fields");
    assert!(required.contains(&serde_json::Value::from("classification_status")));
    assert!(!required.contains(&serde_json::Value::from("status")));
    assert_eq!(
        schema.pointer("/properties/schema_version/const"),
        Some(&serde_json::Value::from("1.1"))
    );
    assert!(schema.pointer("/properties/status").is_none());
    assert_eq!(
        schema.pointer("/properties/classification_status/enum"),
        Some(&serde_json::json!(["tracked", "complete"]))
    );
    let rules = schema["$defs"]["capability"]["allOf"]
        .as_array()
        .expect("capability schema must declare conditional rules");

    let evidence_rule = rules
        .iter()
        .find(|rule| {
            rule.pointer("/if/properties/disposition/enum")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|values| {
                    values
                        == &[
                            serde_json::Value::String("retain".to_string()),
                            serde_json::Value::String("replace".to_string()),
                        ]
                })
        })
        .expect("retain/replace evidence rule must exist");
    assert_eq!(
        evidence_rule.pointer("/then/properties/implementation_evidence/minItems"),
        Some(&serde_json::Value::from(1))
    );
    assert_eq!(
        evidence_rule.pointer("/then/properties/acceptance_evidence/minItems"),
        Some(&serde_json::Value::from(1))
    );
    assert_eq!(
        schema
            .pointer("/$defs/capability/properties/acceptance_evidence/items/pattern")
            .and_then(serde_json::Value::as_str),
        Some(ACCEPTANCE_EVIDENCE_PATTERN)
    );

    let deferred_rule = rules
        .iter()
        .find(|rule| {
            rule.pointer("/if/properties/disposition/const")
                == Some(&serde_json::Value::String("defer-to-r4".to_string()))
        })
        .expect("deferred-outcome rule must exist");
    assert_eq!(
        deferred_rule.pointer("/then/properties/owning_batch/const"),
        Some(&serde_json::Value::String("r4".to_string()))
    );
}

#[test]
fn acceptance_evidence_resolves_exact_executable_symbols() {
    let root = repository_root();
    let test_path = "packages/qiongli-native/crates/qiongli-platform/src/client_inventory.rs";
    let test_source = root.join(test_path);
    assert!(acceptance_symbol_is_executable(
        &test_source,
        test_path,
        "missing_clients_are_inspect_only_and_discovery_does_not_write"
    ));
    assert!(!acceptance_symbol_is_executable(
        &test_source,
        test_path,
        "inspect_path"
    ));
    assert!(!acceptance_symbol_is_executable(
        &test_source,
        test_path,
        "does_not_exist"
    ));

    let example_path =
        "packages/qiongli-native/apps/qiongli/examples/native_packaged_product_acceptance.rs";
    let example_source = root.join(example_path);
    assert!(acceptance_symbol_is_executable(
        &example_source,
        example_path,
        "exercise_lite_mcp_self_test"
    ));
    assert!(!acceptance_symbol_is_executable(
        &example_source,
        example_path,
        "read_json"
    ));

    assert!(parse_acceptance_evidence(
        "packages/qiongli-native/apps/qiongli/tests/cli.rs#status_and_doctor_are_read_only_redacted_and_explicit_about_limitations"
    )
    .is_some());
    assert!(parse_acceptance_evidence("../outside.rs#unrelated_test").is_none());
    assert!(parse_acceptance_evidence("packages/example.rs").is_none());
}

#[test]
fn example_scenarios_must_be_uncommented_calls_in_run() {
    let commented_call = r#"
fn run() {
    // exercise_probe();
    /* exercise_probe(); */
    let _description = "exercise_probe(";
}

fn exercise_probe() {}
"#;
    assert!(!example_scenario_is_called_from_run(
        commented_call,
        "exercise_probe"
    ));

    let non_run_call = r#"
fn run() {}

fn helper() {
    exercise_probe();
}

fn exercise_probe() {}
"#;
    assert!(!example_scenario_is_called_from_run(
        non_run_call,
        "exercise_probe"
    ));

    let run_call = r#"
fn run() {
    exercise_probe();
}

fn exercise_probe() {}
"#;
    assert!(example_scenario_is_called_from_run(
        run_call,
        "exercise_probe"
    ));
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

fn parse_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn parse_acceptance_evidence(evidence: &str) -> Option<(&str, &str)> {
    let (repository_path, scenario) = evidence.split_once('#')?;
    let path = Path::new(repository_path);
    if scenario.contains('#')
        || path.is_absolute()
        || path.extension().and_then(|extension| extension.to_str()) != Some("rs")
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        || !valid_scenario_identifier(scenario)
    {
        return None;
    }
    Some((repository_path, scenario))
}

fn valid_scenario_identifier(scenario: &str) -> bool {
    let mut bytes = scenario.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z'))
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn acceptance_symbol_is_executable(path: &Path, repository_path: &str, scenario: &str) -> bool {
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    if repository_path.contains("/examples/") {
        return example_scenario_is_called_from_run(&source, scenario);
    }

    let lines = source.lines().collect::<Vec<_>>();
    let mut declarations = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| declared_function_name(line) == Some(scenario));
    let Some((declaration_index, _)) = declarations.next() else {
        return false;
    };
    if declarations.next().is_some() {
        return false;
    }

    lines[..declaration_index]
        .iter()
        .rev()
        .take_while(|line| line.trim_start().starts_with("#["))
        .any(|line| {
            let attribute = line.trim();
            attribute == "#[test]" || attribute.starts_with("#[tokio::test")
        })
}

fn declared_function_name(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let declaration = line
        .strip_prefix("fn ")
        .or_else(|| line.strip_prefix("async fn "))?;
    let name_end =
        declaration.find(|character: char| character == '(' || character.is_whitespace())?;
    let name = &declaration[..name_end];
    valid_scenario_identifier(name).then_some(name)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RustToken<'a> {
    Identifier(&'a str),
    OpenBrace,
    CloseBrace,
    OpenParenthesis,
    Semicolon,
    Other,
}

fn example_scenario_is_called_from_run(source: &str, scenario: &str) -> bool {
    if !scenario.starts_with("exercise_") {
        return false;
    }
    let tokens = rust_code_tokens(source);
    let scenario_definitions = function_declarations(&tokens, scenario);
    if scenario_definitions.len() != 1 {
        return false;
    }
    let run_definitions = function_declarations(&tokens, "run");
    if run_definitions.len() != 1 {
        return false;
    }
    let Some((body_start, body_end)) = function_body(&tokens, run_definitions[0]) else {
        return false;
    };

    tokens[body_start + 1..body_end]
        .windows(2)
        .enumerate()
        .any(|(offset, pair)| {
            pair == [RustToken::Identifier(scenario), RustToken::OpenParenthesis]
                && (offset == 0 || tokens[body_start + offset] != RustToken::Identifier("fn"))
        })
}

fn function_declarations(tokens: &[RustToken<'_>], name: &str) -> Vec<usize> {
    tokens
        .windows(3)
        .enumerate()
        .filter_map(|(index, window)| {
            (window
                == [
                    RustToken::Identifier("fn"),
                    RustToken::Identifier(name),
                    RustToken::OpenParenthesis,
                ])
            .then_some(index)
        })
        .collect()
}

fn function_body(tokens: &[RustToken<'_>], declaration: usize) -> Option<(usize, usize)> {
    let body_start = tokens.iter().enumerate().skip(declaration + 3).find_map(
        |(index, token)| match token {
            RustToken::OpenBrace => Some(Some(index)),
            RustToken::Semicolon => Some(None),
            _ => None,
        },
    )??;
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().skip(body_start) {
        match token {
            RustToken::OpenBrace => depth += 1,
            RustToken::CloseBrace => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some((body_start, index));
                }
            }
            _ => {}
        }
    }
    None
}

fn rust_code_tokens(source: &str) -> Vec<RustToken<'_>> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0_usize;
    while index < bytes.len() {
        if let Some(end) = raw_string_end(bytes, index) {
            index = end;
            continue;
        }
        if bytes[index] == b'"' {
            index = quoted_string_end(bytes, index);
            continue;
        }
        if bytes[index] == b'\''
            && let Some(end) = char_literal_end(source, index)
        {
            index = end;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            index = bytes[index..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(bytes.len(), |offset| index + offset + 1);
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            index = block_comment_end(bytes, index);
            continue;
        }
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            tokens.push(RustToken::Identifier(&source[start..index]));
            continue;
        }
        let token = match bytes[index] {
            b'{' => Some(RustToken::OpenBrace),
            b'}' => Some(RustToken::CloseBrace),
            b'(' => Some(RustToken::OpenParenthesis),
            b';' => Some(RustToken::Semicolon),
            byte if byte.is_ascii_whitespace() => None,
            _ => Some(RustToken::Other),
        };
        if let Some(token) = token {
            tokens.push(token);
        }
        index += 1;
    }
    tokens
}

fn raw_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start;
    if matches!(bytes.get(cursor), Some(b'b' | b'c')) {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'r') {
        return None;
    }
    cursor += 1;
    let hashes_start = cursor;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    let hash_count = cursor - hashes_start;
    if bytes.get(cursor) != Some(&b'"') {
        return None;
    }
    cursor += 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'"'
            && bytes.get(cursor + 1..cursor + 1 + hash_count)
                == Some(&bytes[hashes_start..hashes_start + hash_count])
        {
            return Some(cursor + 1 + hash_count);
        }
        cursor += 1;
    }
    Some(bytes.len())
}

fn quoted_string_end(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor = (cursor + 2).min(bytes.len()),
            b'"' => return cursor + 1,
            _ => cursor += 1,
        }
    }
    bytes.len()
}

fn char_literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut cursor = start + 1;
    if bytes.get(cursor) == Some(&b'\\') {
        cursor += 1;
        match bytes.get(cursor)? {
            b'x' => cursor += 3,
            b'u' if bytes.get(cursor + 1) == Some(&b'{') => {
                cursor += 2;
                cursor += bytes[cursor..].iter().position(|byte| *byte == b'}')? + 1;
            }
            _ => cursor += source[cursor..].chars().next()?.len_utf8(),
        }
    } else {
        cursor += source[cursor..].chars().next()?.len_utf8();
    }
    (bytes.get(cursor) == Some(&b'\'')).then_some(cursor + 1)
}

fn block_comment_end(bytes: &[u8], start: usize) -> usize {
    let mut cursor = start + 2;
    let mut depth = 1_usize;
    while cursor < bytes.len() {
        if bytes[cursor..].starts_with(b"/*") {
            depth += 1;
            cursor += 2;
        } else if bytes[cursor..].starts_with(b"*/") {
            depth -= 1;
            cursor += 2;
            if depth == 0 {
                return cursor;
            }
        } else {
            cursor += 1;
        }
    }
    bytes.len()
}

fn unique_set<'a>(values: impl IntoIterator<Item = &'a str>) -> BTreeSet<&'a str> {
    let values: Vec<_> = values.into_iter().collect();
    let unique: BTreeSet<_> = values.iter().copied().collect();
    assert_eq!(unique.len(), values.len(), "duplicate ledger value");
    unique
}

fn collect_set<'a>(values: impl IntoIterator<Item = &'a str>) -> BTreeSet<&'a str> {
    values.into_iter().collect()
}
