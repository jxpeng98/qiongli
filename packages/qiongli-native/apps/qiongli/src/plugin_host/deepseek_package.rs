//! Resolve a release before approval; verify that release without the CLI's pack.
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path};
use std::time::Duration;

use qiongli_content::{ResourcePackManifestV1, load_resource_pack};
use serde_json::Value;

const REGISTRY_LATEST: &str = "https://registry.npmjs.org/qiongli/latest";
const INVALID: &str = "deepseek-install-not-verified";
const LOOKUP: &str = "deepseek-latest-version-unavailable";
const MANIFEST_LIMIT: usize = 4 * 1024 * 1024;
const PAYLOAD_LIMIT: usize = 128 * 1024 * 1024;

pub(super) fn stable_version(value: &str) -> Result<(), &'static str> {
    let version = semver::Version::parse(value).map_err(|_| LOOKUP)?;
    if !version.pre.is_empty() || !version.build.is_empty() || version.to_string() != value {
        return Err(LOOKUP);
    }
    Ok(())
}

pub(super) fn latest_version() -> Result<String, &'static str> {
    resolve_latest(REGISTRY_LATEST)
}

fn resolve_latest(endpoint: &str) -> Result<String, &'static str> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("qiongli/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|_| LOOKUP)?;
    let response = client
        .get(endpoint)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .send()
        .map_err(|_| LOOKUP)?;
    if response.status() != reqwest::StatusCode::OK
        || response
            .content_length()
            .is_some_and(|size| size > 1024 * 1024)
        || response
            .headers()
            .contains_key(reqwest::header::CONTENT_ENCODING)
    {
        return Err(LOOKUP);
    }
    let mut bytes = Vec::new();
    response
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| LOOKUP)?;
    if bytes.len() > 1024 * 1024 {
        return Err(LOOKUP);
    }
    let metadata: Value = serde_json::from_slice(&bytes).map_err(|_| LOOKUP)?;
    let version = metadata["version"].as_str().ok_or(LOOKUP)?;
    stable_version(version)?;
    if metadata["name"] != "qiongli"
        || metadata["main"] != "dsh/index.mjs"
        || metadata["dsh"]["bundle"]["patch"] != "./dsh/cordis.patch.yml"
    {
        return Err(LOOKUP);
    }
    Ok(version.into())
}

fn regular_bytes(root: &Path, relative: &str, maximum: u64) -> Result<Vec<u8>, &'static str> {
    if relative.contains('\\') || relative.chars().any(char::is_control) {
        return Err(INVALID);
    }
    let mut file = root.to_path_buf();
    for component in Path::new(relative).components() {
        let Component::Normal(name) = component else {
            return Err(INVALID);
        };
        file.push(name);
        // The npm package root may be a pnpm store link. Its contents must not
        // introduce any additional links or escape the package through them.
        if fs::symlink_metadata(&file)
            .map_err(|_| INVALID)?
            .file_type()
            .is_symlink()
        {
            return Err(INVALID);
        }
    }
    let metadata = fs::symlink_metadata(&file).map_err(|_| INVALID)?;
    if !metadata.is_file() || metadata.len() > maximum {
        return Err(INVALID);
    }
    let mut bytes = Vec::new();
    fs::File::open(file)
        .map_err(|_| INVALID)?
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| INVALID)?;
    if bytes.len() as u64 > maximum {
        return Err(INVALID);
    }
    Ok(bytes)
}

/// This checks installed content consistency. Registry origin, package download
/// integrity and package scripts remain the official DSH/pnpm manager's owner;
/// a self-consistent receipt is not an independent registry signature.
pub(super) fn verify_content(
    root: &Path,
    version: &str,
    receipt: &Value,
) -> Result<(), &'static str> {
    let source = &receipt["source"];
    if receipt["schema_version"] != 1
        || receipt["platform"] != "deepseek-npm"
        || source["schema_version"] != 1
        || source["version"] != version
        || fs::symlink_metadata(root)
            .map_err(|_| INVALID)?
            .file_type()
            .is_symlink()
    {
        return Err(INVALID);
    }
    let manifest_json = source["pack_manifest_json"].as_str().ok_or(INVALID)?;
    if manifest_json.len() > MANIFEST_LIMIT {
        return Err(INVALID);
    }
    let manifest = ResourcePackManifestV1::from_json(manifest_json).map_err(|_| INVALID)?;
    manifest.validate().map_err(|_| INVALID)?;
    if manifest.content_version != version
        || manifest.entries.len() > 4096
        || source["content_source_commit"] != manifest.source_commit
        || source["content_root_sha256"] != manifest.content_root_sha256
    {
        return Err(INVALID);
    }
    let entries = source["entries"].as_array().ok_or(INVALID)?;
    if entries.len() != manifest.entries.len() {
        return Err(INVALID);
    }
    let mut recorded_entries = BTreeMap::new();
    for entry in entries {
        let path = entry["path"].as_str().ok_or(INVALID)?;
        if recorded_entries.insert(path, entry).is_some() {
            return Err(INVALID);
        }
    }
    let mut pack = Vec::new();
    pack.extend_from_slice(b"QLPACK\0\0");
    pack.extend_from_slice(&1u32.to_le_bytes());
    pack.extend_from_slice(&(manifest_json.len() as u64).to_le_bytes());
    pack.extend_from_slice(manifest_json.as_bytes());
    let mut total = 0usize;
    for entry in &manifest.entries {
        let recorded = recorded_entries.get(entry.path.as_str()).ok_or(INVALID)?;
        if recorded["path"] != entry.path
            || recorded["size_bytes"] != entry.size_bytes
            || recorded["sha256"] != entry.sha256
            || entry.size_bytes > 16 * 1024 * 1024
        {
            return Err(INVALID);
        }
        total = total
            .checked_add(entry.size_bytes as usize)
            .ok_or(INVALID)?;
        if total > PAYLOAD_LIMIT {
            return Err(INVALID);
        }
        let bytes = match entry.path.as_str() {
            ".codex-plugin/plugin.json" | ".claude-plugin/plugin.json" => {
                receipt["source_manifest_bytes"][&entry.path]
                    .as_str()
                    .ok_or(INVALID)?
                    .as_bytes()
                    .to_vec()
            }
            "workflow/no-qiongli/SKILL.md" => {
                regular_bytes(root, "skills/no-qiongli/SKILL.md", entry.size_bytes)?
            }
            path => {
                // Match the canonical npm projection: only remove an existing
                // workflow prefix; other canonical resources retain their path.
                let suffix = path.strip_prefix("workflow/").unwrap_or(path);
                regular_bytes(
                    root,
                    &format!("skills/qiongli-workflow/{suffix}"),
                    entry.size_bytes,
                )?
            }
        };
        if bytes.len() as u64 != entry.size_bytes {
            return Err(INVALID);
        }
        pack.extend_from_slice(&bytes);
    }
    load_resource_pack(&pack, source["pack_sha256"].as_str().ok_or(INVALID)?)
        .map_err(|_| INVALID)?;
    for entry in ["index.mjs", "cordis.patch.yml", "skills.json"] {
        if regular_bytes(root, entry, MANIFEST_LIMIT as u64)?.is_empty() {
            return Err(INVALID);
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod tests;
