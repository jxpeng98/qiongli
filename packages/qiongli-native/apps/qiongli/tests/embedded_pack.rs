use qiongli::{EMBEDDED_PACK_SHA256, embedded_content};
use qiongli_content::ProfileId;

#[test]
fn product_binary_contains_the_frozen_verified_resource_pack() {
    let content = embedded_content().expect("product embedded pack must verify");

    assert_eq!(content.pack().pack_sha256(), EMBEDDED_PACK_SHA256);
    assert_eq!(content.pack().manifest().pack_id, "qiongli-core");
    assert_eq!(content.pack().manifest().content_version, "1.19.0-beta.1");
    assert_eq!(
        content.pack().manifest().source_commit,
        "ff2c4f35cd1ee5df78a04ff90a0325273917eed8"
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
        b"v1.19.0-beta.1\n"
    );
}
