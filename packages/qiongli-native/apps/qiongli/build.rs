use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use qiongli_content::{
    QIONGLI_CORE_RESOURCE_PACK_LOCK_V1, ResourcePackLockV1, build_resource_pack,
    collect_canonical_sources,
};

const PACK_FILE: &str = "qiongli-core.qlpack";
const DIGEST_FILE: &str = "qiongli-core.qlpack.sha256";

fn main() {
    if let Err(error) = build_embedded_pack() {
        panic!("failed to build verified Qiongli embedded content: {error}");
    }
}

fn build_embedded_pack() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let content_root = manifest_dir.join("../../../../content");
    let lock_path =
        manifest_dir.join("../../crates/qiongli-content/resources/qiongli-core.lock.json");
    println!("cargo:rerun-if-changed={}", content_root.display());
    println!("cargo:rerun-if-changed={}", lock_path.display());

    let lock = ResourcePackLockV1::from_json(QIONGLI_CORE_RESOURCE_PACK_LOCK_V1)?;
    if lock.to_canonical_json()?.as_slice() != QIONGLI_CORE_RESOURCE_PACK_LOCK_V1.as_bytes() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "qiongli-core resource-pack lock must use canonical JSON",
        )
        .into());
    }
    let resources = collect_canonical_sources(&content_root)?;
    let built = build_resource_pack(&lock.metadata()?, &resources)?;
    lock.verify(&built)?;

    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo OUT_DIR is unavailable"))?;
    fs::write(out_dir.join(PACK_FILE), built.core_bytes())?;
    fs::write(out_dir.join(DIGEST_FILE), lock.pack_sha256.as_bytes())?;
    Ok(())
}
