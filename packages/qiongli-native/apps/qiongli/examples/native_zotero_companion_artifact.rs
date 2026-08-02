use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{Read as _, Write as _};
use std::path::{Component, Path, PathBuf};

use qiongli_platform::{
    ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE, ZOTERO_COMPANION_PACKAGED_XPI_FILE,
    verify_zotero_companion_artifact,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_XPI_BYTES: u64 = 2 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

fn main() {
    if let Err(code) = run() {
        eprintln!("error: {code}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let arguments = Arguments::parse(env::args_os().skip(1))?;
    match arguments {
        Arguments::Materialize { output } => materialize(&output),
        Arguments::Verify { manifest, xpi } => verify(&manifest, &xpi),
    }
}

enum Arguments {
    Materialize { output: PathBuf },
    Verify { manifest: PathBuf, xpi: PathBuf },
}

impl Arguments {
    fn parse(values: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let values = values.into_iter().collect::<Vec<_>>();
        match values.as_slice() {
            [command, option, output]
                if command.to_str() == Some("materialize")
                    && option.to_str() == Some("--output") =>
            {
                let output = PathBuf::from(output);
                if !valid_existing_directory(&output) {
                    return Err("zotero-companion-artifact-output-invalid");
                }
                Ok(Self::Materialize { output })
            }
            [command, manifest_option, manifest, xpi_option, xpi]
                if command.to_str() == Some("verify")
                    && manifest_option.to_str() == Some("--manifest")
                    && xpi_option.to_str() == Some("--xpi") =>
            {
                let manifest = PathBuf::from(manifest);
                let xpi = PathBuf::from(xpi);
                if !valid_input_file(&manifest) || !valid_input_file(&xpi) {
                    return Err("zotero-companion-artifact-input-invalid");
                }
                Ok(Self::Verify { manifest, xpi })
            }
            _ => Err("zotero-companion-artifact-usage-invalid"),
        }
    }
}

fn materialize(output: &Path) -> Result<(), &'static str> {
    if fs::read_dir(output)
        .map_err(|_| "zotero-companion-artifact-output-invalid")?
        .next()
        .is_some()
    {
        return Err("zotero-companion-artifact-output-not-empty");
    }
    let artifact = qiongli::embedded_zotero_companion().map_err(|error| error.reason_code())?;
    let xpi = output.join(ZOTERO_COMPANION_PACKAGED_XPI_FILE);
    let manifest = output.join(ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE);
    write_new(&xpi, artifact.xpi_bytes())?;
    if let Err(error) = write_new(&manifest, artifact.manifest_bytes()) {
        let _ = fs::remove_file(&xpi);
        return Err(error);
    }
    let receipt = ArtifactReceiptV1 {
        schema_version: 1,
        status: "materialized",
        companion_version: &artifact.manifest().companion_version,
        endpoint_version: &artifact.manifest().endpoint_version,
        source_artifact_file: &artifact.manifest().artifact_file,
        materialized_file: ZOTERO_COMPANION_PACKAGED_XPI_FILE,
        artifact_size_bytes: artifact.manifest().artifact_size_bytes,
        artifact_sha256: &artifact.manifest().artifact_sha256,
        artifact_manifest_file: ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE,
        artifact_manifest_sha256: sha256_hex(artifact.manifest_bytes()),
    };
    println!(
        "{}",
        serde_json_canonicalizer::to_string(&receipt)
            .map_err(|_| "zotero-companion-artifact-receipt-invalid")?
    );
    Ok(())
}

fn verify(manifest: &Path, xpi: &Path) -> Result<(), &'static str> {
    let manifest_bytes = read_bounded(manifest, MAX_MANIFEST_BYTES)?;
    let xpi_bytes = read_bounded(xpi, MAX_XPI_BYTES)?;
    let artifact = verify_zotero_companion_artifact(&manifest_bytes, &xpi_bytes)
        .map_err(|error| error.reason_code())?;
    let receipt = ArtifactReceiptV1 {
        schema_version: 1,
        status: "verified",
        companion_version: &artifact.manifest().companion_version,
        endpoint_version: &artifact.manifest().endpoint_version,
        source_artifact_file: &artifact.manifest().artifact_file,
        materialized_file: ZOTERO_COMPANION_PACKAGED_XPI_FILE,
        artifact_size_bytes: artifact.manifest().artifact_size_bytes,
        artifact_sha256: &artifact.manifest().artifact_sha256,
        artifact_manifest_file: ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE,
        artifact_manifest_sha256: sha256_hex(artifact.manifest_bytes()),
    };
    println!(
        "{}",
        serde_json_canonicalizer::to_string(&receipt)
            .map_err(|_| "zotero-companion-artifact-receipt-invalid")?
    );
    Ok(())
}

#[derive(Serialize)]
struct ArtifactReceiptV1<'a> {
    schema_version: u32,
    status: &'static str,
    companion_version: &'a str,
    endpoint_version: &'a str,
    source_artifact_file: &'a str,
    materialized_file: &'static str,
    artifact_size_bytes: u64,
    artifact_sha256: &'a str,
    artifact_manifest_file: &'static str,
    artifact_manifest_sha256: String,
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, &'static str> {
    let file = fs::File::open(path).map_err(|_| "zotero-companion-artifact-read-failed")?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "zotero-companion-artifact-read-failed")?;
    if bytes.is_empty() || bytes.len() as u64 > limit {
        return Err("zotero-companion-artifact-read-failed");
    }
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "zotero-companion-artifact-write-failed")?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "zotero-companion-artifact-write-failed")
}

fn valid_existing_directory(path: &Path) -> bool {
    valid_absolute_path(path)
        && fs::symlink_metadata(path)
            .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
        && fs::canonicalize(path).ok().as_deref() == Some(path)
}

fn valid_input_file(path: &Path) -> bool {
    valid_absolute_path(path)
        && fs::symlink_metadata(path)
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn valid_absolute_path(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut value = String::with_capacity(64);
    for byte in digest {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    value
}
