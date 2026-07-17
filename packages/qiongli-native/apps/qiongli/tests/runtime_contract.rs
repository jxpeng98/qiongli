use qiongli_runtime::{LITE_PUBLIC_TOOL_NAMES, LiteToolId, LiteToolRegistry};

#[test]
fn verified_embedded_pack_supplies_the_frozen_lite_registry() {
    let content = qiongli::embedded_content().expect("embedded content must verify");
    let registry = LiteToolRegistry::from_embedded_content(&content)
        .expect("verified pack must contain the Lite contract");
    let names = registry
        .tools()
        .iter()
        .map(|tool| tool.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, LITE_PUBLIC_TOOL_NAMES);
    assert_eq!(
        registry.resolve("qiongli_open_config_wizard"),
        Some(LiteToolId::ConfigureProvider)
    );
}
