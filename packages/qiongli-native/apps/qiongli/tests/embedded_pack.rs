use qiongli::{EMBEDDED_PACK_SHA256, embedded_content};
use qiongli_content::ProfileId;

#[test]
fn product_binary_contains_the_frozen_verified_resource_pack() {
    let content = embedded_content().expect("product embedded pack must verify");

    assert_eq!(content.pack().pack_sha256(), EMBEDDED_PACK_SHA256);
    assert_eq!(content.pack().manifest().pack_id, "qiongli-core");
    assert_eq!(
        content.pack().manifest().content_version,
        env!("CARGO_PKG_VERSION")
    );
    assert_eq!(
        content
            .profiles()
            .iter()
            .map(|profile| profile.id)
            .collect::<Vec<_>>(),
        vec![
            ProfileId::SkillOnly,
            ProfileId::MarketplaceLite,
            ProfileId::Full,
        ]
    );
    assert_eq!(
        content
            .read_profile_resource("skill-only", "workflow/VERSION")
            .expect("skill-only profile must resolve")
            .expect("workflow version must be embedded")
            .bytes(),
        format!("v{}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    let codex_manifest = content
        .read_profile_resource("marketplace-lite", ".codex-plugin/plugin.json")
        .expect("marketplace-lite profile must resolve")
        .expect("Codex plugin manifest must be embedded");
    let codex_manifest: serde_json::Value = serde_json::from_slice(codex_manifest.bytes())
        .expect("Codex plugin manifest must be valid JSON");
    assert_eq!(codex_manifest["name"], "qiongli");
    assert_eq!(codex_manifest["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(codex_manifest["skills"], "./");
    assert!(codex_manifest.get("mcpServers").is_none());

    let claude_manifest = content
        .read_profile_resource("marketplace-lite", ".claude-plugin/plugin.json")
        .expect("marketplace-lite profile must resolve")
        .expect("Claude plugin manifest must be embedded");
    let claude_manifest: serde_json::Value = serde_json::from_slice(claude_manifest.bytes())
        .expect("Claude plugin manifest must be valid JSON");
    assert_eq!(claude_manifest["name"], "qiongli");
    assert_eq!(claude_manifest["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(claude_manifest["skills"], "./skills/");
    assert_eq!(claude_manifest["mcpServers"], "./.mcp.json");
}
