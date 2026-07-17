use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParityLedger {
    #[serde(rename = "$schema")]
    schema: String,
    schema_version: String,
    record_type: String,
    source_release: String,
    baseline_plan: String,
    status: LedgerStatus,
    capabilities: Vec<Capability>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum LedgerStatus {
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
    assert_eq!(ledger.schema_version, "1.0");
    assert_eq!(ledger.record_type, "qiongli-1x-product-outcome-parity");
    assert!(matches!(
        ledger.status,
        LedgerStatus::Tracked | LedgerStatus::Complete
    ));

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
            Disposition::DeferToR4 => assert!(
                capability
                    .nonclaim
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "{} must state its R3Q nonclaim",
                capability.id
            ),
            Disposition::RetireWithReason => assert!(
                capability
                    .retirement_reason
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "{} must explain retirement",
                capability.id
            ),
            Disposition::Retain | Disposition::Replace => {}
        }
        for source in &capability.source {
            assert!(
                root.join(source).is_file(),
                "missing ledger source: {source}"
            );
        }
        for evidence in capability
            .implementation_evidence
            .iter()
            .chain(&capability.acceptance_evidence)
        {
            assert!(
                root.join(evidence).exists(),
                "missing ledger evidence: {evidence}"
            );
        }
        let _ = (&capability.category, &capability.owning_batch);
    }

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

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

fn parse_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
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
