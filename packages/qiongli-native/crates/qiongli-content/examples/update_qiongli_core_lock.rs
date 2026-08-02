use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use qiongli_content::{
    CompatibleProduct, ResourcePackBuildMetadata, ResourcePackLockV1, build_resource_pack,
    collect_canonical_sources,
};

fn main() -> Result<(), Box<dyn Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let content_root = crate_root.join("../../../../content");
    let lock_path = crate_root.join("resources/qiongli-core.lock.json");
    let resources = collect_canonical_sources(&content_root)?;
    let source_commit = env::var("QIONGLI_NATIVE_SOURCE_COMMIT")?;
    if !matches!(source_commit.len(), 40 | 64)
        || !source_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(
            "QIONGLI_NATIVE_SOURCE_COMMIT must be a lowercase 40- or 64-byte hex digest".into(),
        );
    }
    let built = build_resource_pack(
        &ResourcePackBuildMetadata {
            pack_id: "qiongli-core".to_string(),
            content_version: env!("CARGO_PKG_VERSION").to_string(),
            source_commit,
            compatible_product: CompatibleProduct {
                minimum: "2.0.0-alpha.1".to_string(),
                maximum_exclusive: "3.0.0".to_string(),
            },
        },
        &resources,
    )?;
    let lock = ResourcePackLockV1::from_built(&built);
    let lock_bytes = lock.to_canonical_json()?;

    let parent = lock_path
        .parent()
        .expect("resource-pack lock path must have a parent");
    fs::create_dir_all(parent)?;
    fs::write(&lock_path, lock_bytes)?;

    println!("updated {}", lock_path.display());
    println!("entry_count={}", lock.entry_count);
    println!("content_root_sha256={}", lock.content_root_sha256);
    println!("pack_sha256={}", lock.pack_sha256);
    Ok(())
}
