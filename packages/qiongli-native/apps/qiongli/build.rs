use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Read as _};
use std::path::PathBuf;

use qiongli_content::{
    QIONGLI_CORE_RESOURCE_PACK_LOCK_V1, ResourcePackLockV1, build_resource_pack,
    collect_canonical_sources,
};
use qiongli_platform::{MAX_NATIVE_RELEASE_AUTHORITY_BYTES, NativeReleaseAuthority};

const PACK_FILE: &str = "qiongli-core.qlpack";
const DIGEST_FILE: &str = "qiongli-core.qlpack.sha256";
const RELEASE_AUTHORITY_FILE: &str = "qiongli-native-release-authority.json";
const RELEASE_AUTHORITY_ENV: &str = "QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE";

fn main() {
    if let Err(error) = build_embedded_assets() {
        panic!("failed to build verified Qiongli embedded assets: {error}");
    }
}

fn build_embedded_assets() -> Result<(), Box<dyn Error>> {
    build_embedded_pack()?;
    build_embedded_release_authority()?;
    Ok(())
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

fn build_embedded_release_authority() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed={RELEASE_AUTHORITY_ENV}");
    let bytes = match env::var_os(RELEASE_AUTHORITY_ENV) {
        None => Vec::new(),
        Some(value) if value.is_empty() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native release authority path is empty",
            )
            .into());
        }
        Some(value) => {
            let path = PathBuf::from(value);
            println!("cargo:rerun-if-changed={}", path.display());
            let file = fs::File::open(path)?;
            let limit = u64::try_from(MAX_NATIVE_RELEASE_AUTHORITY_BYTES)
                .map_err(|_| io::Error::other("native release authority limit is invalid"))?;
            let mut bounded = file.take(limit.saturating_add(1));
            let mut input = Vec::new();
            bounded.read_to_end(&mut input)?;
            if input.len() > MAX_NATIVE_RELEASE_AUTHORITY_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native release authority is too large",
                )
                .into());
            }
            let authority = NativeReleaseAuthority::from_json(&input)?;
            authority.validate_product_version(env!("CARGO_PKG_VERSION"))?;
            authority.to_canonical_json()?
        }
    };
    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo OUT_DIR is unavailable"))?;
    fs::write(out_dir.join(RELEASE_AUTHORITY_FILE), bytes)?;
    Ok(())
}
