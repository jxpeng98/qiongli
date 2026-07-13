use qiongli_content::{
    ProfileId, RESOURCE_PACK_MANIFEST_SCHEMA_V1, ResourceKind, ResourcePackManifestV1,
};
use serde_json::Value;

const GOLDEN: &str = include_str!("fixtures/resource-pack-manifest-v1.golden.json");

fn golden() -> ResourcePackManifestV1 {
    ResourcePackManifestV1::from_json(GOLDEN).expect("golden manifest must parse")
}

#[test]
fn golden_manifest_parses_and_validates() {
    golden().validate().expect("golden manifest must validate");
}

#[test]
fn schema_closes_versions_profiles_kinds_and_unknown_fields() {
    let schema: Value = serde_json::from_str(RESOURCE_PACK_MANIFEST_SCHEMA_V1)
        .expect("resource-pack schema must be JSON");
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["format_version"]["const"], 1);
    assert_eq!(
        schema["$defs"]["resourceKind"]["enum"]
            .as_array()
            .expect("resource kind enum")
            .len(),
        11
    );

    let mut value: Value = serde_json::from_str(GOLDEN).expect("golden JSON");
    value["unexpected"] = Value::Bool(true);
    assert!(ResourcePackManifestV1::from_json(&value.to_string()).is_err());
}

#[test]
fn profile_alias_and_projections_match_the_frozen_contract() {
    let manifest = golden();
    assert_eq!(
        manifest.resolve_profile("lite").expect("lite alias"),
        ProfileId::MarketplaceLite
    );
    assert_eq!(
        manifest.resolve_profile("skill-only").expect("skill-only"),
        ProfileId::SkillOnly
    );
    assert!(manifest.resolve_profile("unknown").is_err());

    let skill_only = manifest
        .entries_for_profile("skill-only")
        .expect("skill-only entries");
    assert_eq!(skill_only.len(), 9);
    assert!(skill_only.iter().all(|entry| {
        !matches!(
            entry.resource_kind,
            ResourceKind::TargetMetadata | ResourceKind::McpContract | ResourceKind::Schema
        )
    }));
    assert_eq!(
        manifest
            .entries_for_profile("lite")
            .expect("lite entries")
            .len(),
        manifest.entries.len()
    );
    assert_eq!(
        manifest
            .entries_for_profile("full")
            .expect("full entries")
            .len(),
        manifest.entries.len()
    );
}

#[test]
fn semantic_mutations_fail_closed() {
    let mut manifest = golden();
    manifest.format_version = 2;
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.source_commit = "ABC".to_string();
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.compatible_product.maximum_exclusive = "1.0.0".to_string();
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.profiles.push(manifest.profiles[0].clone());
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.profiles[1].aliases.clear();
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.profiles[0]
        .included_resource_kinds
        .push(ResourceKind::Schema);
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.entries.swap(0, 1);
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.entries[1].payload_offset = 99;
    assert!(manifest.validate().is_err());

    let mut manifest = golden();
    manifest.entries[0].path = "../escape".to_string();
    assert!(manifest.validate().is_err());
}
