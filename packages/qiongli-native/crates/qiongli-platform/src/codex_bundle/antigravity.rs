//! User-local Antigravity layout over the existing receipt/CAS transaction owner.
//! No signed-product integration or Codex/Claude configuration is implied.
use super::*;

pub const RECEIPT_FILE: &str = ".qiongli-antigravity-plugin-bundle.json";

const ENTRY_GUIDANCE: &str = r#"

## Antigravity workflow entry

Use `qiongli` as the single research entry. Choose the relevant internal workflow
from the route table above and read only the resources needed for this request.
The separate `qiongli-<workflow>` shortcuts described for Codex are not installed
in Antigravity. If the user names one of those workflows, treat it as task intent
and load `workflows/<name>.md` from this package; do not look for another Skill or
try to run the shortcut as a shell command. All internal workflow guidance remains
available. The independent `no-qiongli` reply-only entry is also retained; honor
an active reply-only choice before any resource read, routing or MCP call.
"#;

#[derive(Clone, Debug)]
pub struct AntigravityPluginBundleTarget(CodexPluginBundleTarget);

impl AntigravityPluginBundleTarget {
    #[must_use]
    pub fn path(&self) -> &Path {
        self.0.path()
    }
}

/// A verified local export, not evidence of installation or a running Host session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedAntigravityPluginBundle(VerifiedCodexPluginBundle);

impl VerifiedAntigravityPluginBundle {
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
pub fn approve_antigravity_plugin_bundle_target(
    path: impl AsRef<Path>,
) -> Result<AntigravityPluginBundleTarget, CodexPluginBundleError> {
    let mut target = approve_codex_plugin_bundle_target(path)?;
    target.host = LocalBundleHost::Antigravity;
    Ok(AntigravityPluginBundleTarget(target))
}

pub fn compose_local_antigravity_plugin_source(
    pack: &LoadedResourcePack<'_>,
    source_binary: &Path,
    expected_binary_sha256: &str,
    target: &AntigravityPluginBundleTarget,
    overrides: Option<&WorkflowOverrides>,
    expected_receipt_sha256: Option<&str>,
    skill_language: Option<&str>,
) -> Result<VerifiedAntigravityPluginBundle, CodexPluginBundleError> {
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
    .map(VerifiedAntigravityPluginBundle)
}

pub fn verify_local_antigravity_plugin_source(
    target: &AntigravityPluginBundleTarget,
) -> Result<VerifiedAntigravityPluginBundle, CodexPluginBundleError> {
    revalidate_target(&target.0)?;
    verify_bundle_tree_for_target(target.path(), &target.0).map(VerifiedAntigravityPluginBundle)
}

/// Read an AGY-owned cache with canonical files and canonical or owner-only
/// directories. This does not authorize source updates or cache mutation.
pub fn verify_cached_antigravity_plugin_source(
    target: &AntigravityPluginBundleTarget,
) -> Result<VerifiedAntigravityPluginBundle, CodexPluginBundleError> {
    revalidate_target(&target.0)?;
    verify_bundle_tree_with_receipt(
        target.path(),
        BundleReadLayout::AntigravityCache,
        target.path(),
    )
    .map(VerifiedAntigravityPluginBundle)
}

pub fn remove_local_antigravity_plugin_source(
    target: &AntigravityPluginBundleTarget,
    expected_receipt_sha256: &str,
) -> Result<VerifiedAntigravityPluginBundle, CodexPluginBundleError> {
    remove_bundle(
        &target.0,
        CodexPluginBundleKind::UserLocalAntigravityFullMcp,
        Some(expected_receipt_sha256),
    )
    .map(VerifiedAntigravityPluginBundle)
}

fn manifest(plugin_name: &str) -> Value {
    serde_json::json!({"name": plugin_name, "description": "Qiongli research workflows and native Full MCP"})
}

fn mcp_manifest(plugin_name: &str, executable: &str) -> Value {
    serde_json::json!({"mcpServers": {plugin_name: {"command": executable, "args": expected_mcp_args()}}})
}

pub(super) fn project(
    pack: &LoadedResourcePack<'_>,
    artifact: &ArtifactIdentityV1,
    executable: &Path,
    overrides: Option<&WorkflowOverrides>,
    language: Option<&str>,
) -> Result<BTreeMap<String, BundleFile>, CodexPluginBundleError> {
    let plugin_name = artifact.channel.plugin_name();
    let mut files = project_bundle_files(
        pack,
        artifact,
        binary_relative_path(artifact.os),
        overrides,
        false,
        plugin_name,
        language,
    )?;
    files.remove(PLUGIN_MANIFEST_PATH);
    files.remove(MCP_MANIFEST_PATH);
    // AGY discovers every wrapper as a public Skill. Keep the shared resource
    // library and the self-contained reply-only entry, with one research router.
    // Receipt readers still accept legacy wrappers for verified update/removal.
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
        .replace("Codex", "Antigravity")
        .replace(
            "one `codex` host descriptor",
            "one `other-local` host descriptor",
        )
        .replace("`codex`", "`agy`");
    skill.bytes = text
        .replace(CODEX_HOST_ADAPTER_GUIDANCE, &guidance)
        .into_bytes();
    skill.bytes.extend_from_slice(ENTRY_GUIDANCE.as_bytes());
    let executable = executable
        .to_str()
        .filter(|s| !s.chars().any(char::is_control))
        .ok_or(CodexPluginBundleError::ProjectionInvalid)?;
    for (path, value) in [
        ("plugin.json", manifest(plugin_name)),
        ("mcp_config.json", mcp_manifest(plugin_name, executable)),
    ] {
        files.insert(
            path.into(),
            BundleFile {
                mode: LogicalMode::Regular,
                bytes: canonical_json(&value)?,
            },
        );
    }
    Ok(files)
}

pub(super) fn verify_contract(
    root: &Path,
    receipt: &CodexPluginBundleReceiptV1,
) -> Result<(), CodexPluginBundleError> {
    let read = |name: &str| -> Result<Value, CodexPluginBundleError> {
        let bytes = read_bounded_managed_file(
            &root.join(name),
            MAX_MANIFEST_BYTES,
            LogicalMode::Regular,
            CodexPluginBundleError::BundleDrift,
        )?;
        serde_json::from_slice(&bytes).map_err(|_| CodexPluginBundleError::ManifestInvalid)
    };
    if read("plugin.json")? != manifest(receipt.plugin_name()) {
        return Err(CodexPluginBundleError::ManifestInvalid);
    }
    let mcp = read("mcp_config.json")?;
    let executable = mcp["mcpServers"][receipt.plugin_name()]["command"]
        .as_str()
        .ok_or(CodexPluginBundleError::ManifestInvalid)?;
    // AGY 1.2 resolves relative commands against its launch directory. Bind an
    // absolute source binary instead; source directories must remain available.
    if !Path::new(executable).is_absolute()
        || !Path::new(executable).ends_with(&receipt.binary_path)
        || executable.chars().any(char::is_control)
        || mcp != mcp_manifest(receipt.plugin_name(), executable)
    {
        return Err(CodexPluginBundleError::ManifestInvalid);
    }
    Ok(())
}
