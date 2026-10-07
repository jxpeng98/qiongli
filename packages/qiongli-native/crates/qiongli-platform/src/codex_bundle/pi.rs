//! Local Pi package over the shared receipt-bound transaction owner.
use super::*;

pub const RECEIPT_FILE: &str = ".qiongli-pi-plugin-bundle.json";
pub(super) const EXTENSION_PATH: &str = "extensions/qiongli-mcp.js";

const ENTRY_GUIDANCE: &str = r#"

## Pi workflow entry

Use `/skill:qiongli` as the research entry. Select the relevant workflow from the
route table and read its resources within this package. The `qiongli-<workflow>`
shortcuts described for Codex are internal workflow names here, not separate Pi
Skills or shell commands. The independent `/skill:no-qiongli` reply-only entry
remains available; honor it before any resource read, routing or MCP call.

This package registers Full MCP through Pi's built-in MCP support. Discover its
tools using `tool_search` or `codemode` in the `mcp__qiongli` namespace (or
`mcp__qiongli_next` for a prerelease); keep the native tool arguments unchanged.
Use `/mcp` in the session to inspect the connection. Shell-level `pi mcp list`
does not load package extensions. If MCP is missing, report that limit; never
claim a tool ran or silently substitute a different server. File-configured
servers and project resource filters can override this package.
"#;

#[derive(Clone, Debug)]
pub struct PiPluginBundleTarget(CodexPluginBundleTarget);

impl PiPluginBundleTarget {
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
}

/// A verified local export, not evidence of installation or a running Host session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedPiPluginBundle(VerifiedCodexPluginBundle);

impl VerifiedPiPluginBundle {
    #[must_use]
    pub fn receipt(&self) -> &CodexPluginBundleReceiptV1 {
        self.0.receipt()
    }

    #[must_use]
    pub fn receipt_sha256(&self) -> &str {
        self.0.receipt_sha256()
    }
}

/// Caller-selected source directory, approved at the trusted terminal boundary.
pub fn approve_pi_plugin_bundle_target(
    path: impl AsRef<Path>,
) -> Result<PiPluginBundleTarget, CodexPluginBundleError> {
    let mut target = approve_codex_plugin_bundle_target(path)?;
    target.host = LocalBundleHost::Pi;
    Ok(PiPluginBundleTarget(target))
}

pub fn compose_local_pi_plugin_source(
    pack: &LoadedResourcePack<'_>,
    source_binary: &Path,
    expected_binary_sha256: &str,
    target: &PiPluginBundleTarget,
    overrides: Option<&WorkflowOverrides>,
    expected_receipt_sha256: Option<&str>,
    skill_language: Option<&str>,
) -> Result<VerifiedPiPluginBundle, CodexPluginBundleError> {
    compose_codex_plugin_bundle_internal(
        pack,
        None,
        expected_binary_sha256,
        source_binary,
        &target.0,
        overrides,
        expected_receipt_sha256.is_some(),
        expected_receipt_sha256,
        Some(false),
        skill_language,
    )
    .map(VerifiedPiPluginBundle)
}

pub fn verify_local_pi_plugin_source(
    target: &PiPluginBundleTarget,
) -> Result<VerifiedPiPluginBundle, CodexPluginBundleError> {
    revalidate_target(&target.0)?;
    verify_bundle_tree_for_target(target.path(), &target.0).map(VerifiedPiPluginBundle)
}

pub fn remove_local_pi_plugin_source(
    target: &PiPluginBundleTarget,
    expected_receipt_sha256: &str,
) -> Result<VerifiedPiPluginBundle, CodexPluginBundleError> {
    remove_bundle(
        &target.0,
        CodexPluginBundleKind::UserLocalPiFullMcp,
        Some(expected_receipt_sha256),
    )
    .map(VerifiedPiPluginBundle)
}

fn manifest(name: &str, version: &str) -> Value {
    serde_json::json!({
        "name": name, "version": version, "type": "module", "private": true,
        "description": "Qiongli research workflows and native Full MCP for Pi",
        "keywords": ["pi-package"],
        "pi": {"skills": ["./skills/qiongli-workflow", "./skills/no-qiongli"],
               "extensions": ["./extensions/qiongli-mcp.js"]}
    })
}

fn extension(name: &str, executable: &Path) -> Result<Vec<u8>, CodexPluginBundleError> {
    let executable = executable
        .to_str()
        .filter(|s| !s.chars().any(char::is_control))
        .ok_or(CodexPluginBundleError::ProjectionInvalid)?;
    let name =
        serde_json::to_string(name).map_err(|_| CodexPluginBundleError::ProjectionInvalid)?;
    let config = serde_json::json!({"command": executable, "args": expected_mcp_args(),
        "exposure": "deferred", "description": "Qiongli research workflows, traceable sources and approved project writes"});
    Ok(format!(
        r#"// Generated Qiongli adapter; Pi owns the MCP transport and model session.
export default function (pi) {{
  if (typeof pi.registerMcpServer !== "function") {{
    throw new Error("Qiongli requires Pi 0.99.0 or newer with built-in MCP enabled");
  }}
  pi.registerMcpServer({name}, {config});
}}
"#
    )
    .into_bytes())
}

pub(super) fn project(
    pack: &LoadedResourcePack<'_>,
    artifact: &ArtifactIdentityV1,
    executable: &Path,
    overrides: Option<&WorkflowOverrides>,
    language: Option<&str>,
) -> Result<BTreeMap<String, BundleFile>, CodexPluginBundleError> {
    let name = artifact.channel.plugin_name();
    let mut files = project_bundle_files(
        pack,
        artifact,
        binary_relative_path(artifact.os),
        overrides,
        false,
        name,
        language,
    )?;
    files.remove(PLUGIN_MANIFEST_PATH);
    files.remove(MCP_MANIFEST_PATH);
    files.retain(|path, _| {
        !path
            .strip_prefix("skills/qiongli-")
            .and_then(|path| path.strip_suffix("/SKILL.md"))
            .is_some_and(workflow_slug_is_valid)
    });
    let skill = files
        .get_mut(SKILL_MANIFEST_PATH)
        .ok_or(CodexPluginBundleError::ProjectionInvalid)?;
    let text =
        std::str::from_utf8(&skill.bytes).map_err(|_| CodexPluginBundleError::ProjectionInvalid)?;
    let guidance = CODEX_HOST_ADAPTER_GUIDANCE
        .replace("Codex", "Pi")
        .replace(
            "one `codex` host descriptor",
            "one `other-local` host descriptor",
        )
        .replace("`codex`", "`pi`");
    skill.bytes = text
        .replace(CODEX_HOST_ADAPTER_GUIDANCE, &guidance)
        .into_bytes();
    skill.bytes.extend_from_slice(ENTRY_GUIDANCE.as_bytes());
    for (path, bytes) in [
        (
            "package.json",
            canonical_json(&manifest(name, &artifact.version))?,
        ),
        (EXTENSION_PATH, extension(name, executable)?),
    ] {
        files.insert(
            path.into(),
            BundleFile {
                mode: LogicalMode::Regular,
                bytes,
            },
        );
    }
    Ok(files)
}

pub(super) fn verify_contract(
    root: &Path,
    receipt: &CodexPluginBundleReceiptV1,
    source_root: &Path,
) -> Result<(), CodexPluginBundleError> {
    // Staging/backup/quarantine move the tree, not its bound execution source.
    for (path, expected) in [
        (
            "package.json",
            canonical_json(&manifest(receipt.plugin_name(), &receipt.artifact.version))?,
        ),
        (
            EXTENSION_PATH,
            extension(
                receipt.plugin_name(),
                &source_root.join(&receipt.binary_path),
            )?,
        ),
    ] {
        let actual = read_bounded_managed_file(
            &root.join(path),
            MAX_MANIFEST_BYTES,
            LogicalMode::Regular,
            CodexPluginBundleError::BundleDrift,
        )?;
        if actual != expected {
            return Err(CodexPluginBundleError::ManifestInvalid);
        }
    }
    Ok(())
}
