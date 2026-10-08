//! Terminal-approved native Plugin export and official AGY registration.
use std::fs;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use qiongli_config::WorkflowVariantStore;
use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    approve_antigravity_plugin_bundle_target, compose_local_antigravity_plugin_source,
    verify_cached_antigravity_plugin_source, verify_local_antigravity_plugin_source,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli_content::{confirm, line, show_json};
use crate::command::{CommandEnvironment, config_root};
use crate::install_output::InstallWriter;

#[cfg(test)]
#[path = "antigravity_tests.rs"]
mod tests;

#[derive(Eq, PartialEq, Serialize)]
struct Plan {
    executable: PathBuf,
    executable_sha256: String,
    source: PathBuf,
    source_receipt_sha256: Option<String>,
    binary_sha256: String,
    content_pack_sha256: String,
    workflow_variant_sha256: Option<String>,
    language: String,
    cache: PathBuf,
    cache_receipt_sha256: Option<String>,
    profile_files: Vec<(String, Option<String>)>,
    commands: Vec<Vec<String>>,
}

pub(crate) fn install(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    destination: Option<&Path>,
    language: &str,
    reader: &mut impl BufRead,
    writer: &mut impl InstallWriter,
) -> Result<bool, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let discovered = environment
        .client_executable("agy")
        .ok_or("host-plugin-executable-unavailable")?;
    let executable = crate::desktop::resolve_host_plugin_executable(home, "agy", &discovered)?;
    let version = run(environment, &executable, &["--version".into()])?;
    let version =
        semver::Version::parse(version.trim()).map_err(|_| "antigravity-version-unsupported")?;
    if version < semver::Version::new(1, 2, 17) {
        return Err("antigravity-version-unsupported");
    }
    let source = select_source(environment, destination)?;
    let plan = prepare(environment, content, &executable, &source, language)?;
    preview(writer, "antigravity-plugin-files", &plan)?;
    line(
        writer,
        "Export the qiongli research entry, the no-qiongli reply-only entry, internal workflows and the native Full MCP binary to this source. AGY will use its absolute binary path; retain this directory while installed.\n",
    )?;
    let reviewed = Instant::now();
    if !confirm(
        reader,
        writer,
        "Approve these Plugin source file changes? [y/N] ",
    )? {
        line(
            writer,
            "Cancelled; no files or Host configuration changed.\n",
        )?;
        return Ok(false);
    }
    {
        let root = config_root(environment).map_err(|e| e.reason_code())?;
        let _guard = crate::update_reconcile::acquire_managed_write_guard(home, root.clone())?;
        if reviewed.elapsed() > Duration::from_secs(600)
            || prepare(environment, content, &executable, &source, language)? != plan
        {
            return Err("local-host-precondition-changed");
        }
        let variant = WorkflowVariantStore::new(root)
            .load(content.pack())
            .map_err(|e| e.reason_code())?;
        let target =
            approve_antigravity_plugin_bundle_target(&source).map_err(|e| e.reason_code())?;
        let binary = std::env::current_exe().map_err(|_| "plugin-source-executable-unavailable")?;
        compose_local_antigravity_plugin_source(
            content.pack(),
            &binary,
            &plan.binary_sha256,
            &target,
            variant.overrides(),
            plan.source_receipt_sha256.as_deref(),
            Some(language),
        )
        .map_err(|e| e.reason_code())?;
    }
    line(writer, "Files: exported and receipt verified.\n")?;
    run(
        environment,
        &executable,
        &[
            "plugin".into(),
            "validate".into(),
            source
                .to_str()
                .ok_or("plugin-source-destination-invalid")?
                .into(),
        ],
    )?;
    let plan = prepare(environment, content, &executable, &source, language)?;
    preview(writer, "antigravity-plugin-registration", &plan)?;
    line(
        writer,
        "AGY will install/update and enable this Plugin. Other Plugins and model settings stay configured. Existing sessions need a restart.\n",
    )?;
    let reviewed = Instant::now();
    if !confirm(
        reader,
        writer,
        "Trust this Plugin and run the displayed AGY commands? [y/N] ",
    )? {
        line(
            writer,
            "Host registration skipped; exported files remain. Session tools: not checked.\n",
        )?;
        return Ok(false);
    }
    let root = config_root(environment).map_err(|e| e.reason_code())?;
    let _guard = crate::update_reconcile::acquire_managed_write_guard(home, root)?;
    if reviewed.elapsed() > Duration::from_secs(600)
        || prepare(environment, content, &executable, &source, language)? != plan
    {
        return Err("local-host-precondition-changed");
    }
    for (index, args) in plan.commands.iter().enumerate() {
        line(
            writer,
            &format!("  Manager step {}/{}\n", index + 1, plan.commands.len()),
        )?;
        super::installation_command::run(
            environment,
            &executable,
            args,
            Duration::from_secs(30),
            false,
            writer,
        )?;
    }
    let installed = prepare(environment, content, &executable, &source, language)?;
    if installed.cache_receipt_sha256 != installed.source_receipt_sha256
        || !enabled(
            &home.join(".gemini/config/config.json"),
            crate::plugin_source::plugin_name(),
        )?
        || !registered(
            &home.join(".gemini/config/import_manifest.json"),
            crate::plugin_source::plugin_name(),
        )?
    {
        return Err("local-host-registration-not-verified");
    }
    show_json(writer, &serde_json::json!({"host": "Antigravity", "version": version.to_string(), "source": source,
        "cache": installed.cache, "receipt_sha256": installed.cache_receipt_sha256, "session_tools": "not-checked"}).to_string())?;
    line(
        writer,
        "Antigravity: Plugin installed and enabled; qiongli and no-qiongli entries, internal workflows and Full MCP. Start a new AGY session and call qiongli_config_status to verify actual tool use. Keep the source directory; uninstall with agy plugin uninstall before removing it.\n",
    )?;
    Ok(true)
}

fn preview(writer: &mut impl Write, operation: &str, plan: &Plan) -> Result<(), &'static str> {
    let bytes =
        serde_json_canonicalizer::to_vec(plan).map_err(|_| "installation-preview-invalid")?;
    line(
        writer,
        if operation.ends_with("files") {
            "\nFile changes to approve\n"
        } else {
            "\nHost registration to approve\n"
        },
    )?;
    show_json(writer, &serde_json::json!({"operation": operation, "plan": plan,
        "plan_digest_sha256": format!("{:x}", Sha256::digest(bytes)),
        "approvals_required": if operation.ends_with("files") { vec!["filesystem-write"] } else { vec!["client-config-change", "host-trust"] }}).to_string())
}

fn prepare(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    executable: &Path,
    source: &Path,
    language: &str,
) -> Result<Plan, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    if !source.is_absolute()
        || !matches!(
            source.file_name().and_then(|s| s.to_str()),
            Some("qiongli" | "qiongli-next" | "qiongli-antigravity")
        )
    {
        return Err("plugin-source-destination-invalid");
    }
    if !qiongli_content::skill_language_valid(language) {
        return Err("skill-language-invalid");
    }
    for reserved in [
        home.join(".gemini"),
        home.join(".agents"),
        home.join(".qiongli"),
        environment
            .codex_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".codex")),
        environment
            .claude_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".claude")),
    ] {
        if source.starts_with(reserved) {
            return Err("plugin-source-destination-reserved");
        }
    }
    let source_target =
        approve_antigravity_plugin_bundle_target(source).map_err(|e| e.reason_code())?;
    let source_receipt_sha256 = if fs::symlink_metadata(source).is_ok() {
        Some(
            verify_local_antigravity_plugin_source(&source_target)
                .map_err(|e| e.reason_code())?
                .receipt_sha256()
                .to_owned(),
        )
    } else {
        None
    };
    let config = home.join(".gemini/config");
    approve_nearest(&config)?;
    let name = crate::plugin_source::plugin_name();
    let cache = config.join("plugins").join(name);
    approve_nearest(&cache)?;
    let cache_receipt_sha256 = if fs::symlink_metadata(&cache).is_ok() {
        let target =
            approve_antigravity_plugin_bundle_target(&cache).map_err(|e| e.reason_code())?;
        Some(
            verify_cached_antigravity_plugin_source(&target)
                .map_err(|_| "antigravity-plugin-conflict")?
                .receipt_sha256()
                .to_owned(),
        )
    } else {
        None
    };
    let mcp = optional_json(&config.join("mcp_config.json"))?;
    if ["qiongli", "qiongli-next"]
        .iter()
        .any(|key| mcp["mcpServers"].get(key).is_some())
    {
        return Err("antigravity-mcp-conflict");
    }
    let other = if name == "qiongli" {
        "qiongli-next"
    } else {
        "qiongli"
    };
    enabled(&config.join("config.json"), name)?;
    if config.join("plugins").join(other).exists() && enabled(&config.join("config.json"), other)? {
        return Err("antigravity-plugin-conflict");
    }
    let profile_files = ["config.json", "import_manifest.json", "mcp_config.json"]
        .into_iter()
        .map(|name| {
            optional_json(&config.join(name))?;
            Ok((
                name.into(),
                read_optional(&config.join(name))?.map(|b| format!("{:x}", Sha256::digest(b))),
            ))
        })
        .collect::<Result<_, &'static str>>()?;
    let variant = WorkflowVariantStore::new(config_root(environment).map_err(|e| e.reason_code())?)
        .load(content.pack())
        .map_err(|e| e.reason_code())?;
    let binary = std::env::current_exe().map_err(|_| "plugin-source-executable-unavailable")?;
    Ok(Plan {
        executable: executable.into(),
        executable_sha256: crate::cli_install::regular_file_sha256(executable)?,
        source: source.into(),
        source_receipt_sha256,
        binary_sha256: crate::cli_install::regular_file_sha256(&binary)?,
        content_pack_sha256: content.pack().pack_sha256().into(),
        workflow_variant_sha256: variant.variant_sha256().map(str::to_owned),
        language: language.into(),
        cache,
        cache_receipt_sha256,
        profile_files,
        commands: vec![
            vec![
                "plugin".into(),
                "install".into(),
                source
                    .to_str()
                    .ok_or("plugin-source-destination-invalid")?
                    .into(),
            ],
            vec!["plugin".into(), "enable".into(), name.into()],
        ],
    })
}

fn select_source(
    environment: &CommandEnvironment,
    destination: Option<&Path>,
) -> Result<PathBuf, &'static str> {
    if let Some(destination) = destination {
        return Ok(destination.to_path_buf());
    }
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let cache = home
        .join(".gemini/config/plugins")
        .join(crate::plugin_source::plugin_name());
    match fs::symlink_metadata(&cache) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(home.join("qiongli-antigravity"));
        }
        Err(_) => return Err("antigravity-plugin-conflict"),
        Ok(_) => {}
    }
    // The verified cache binds the absolute source executable. Do not infer
    // ownership from a directory name or adopt an unverified import record.
    let target = approve_antigravity_plugin_bundle_target(&cache).map_err(|e| e.reason_code())?;
    let verified = verify_cached_antigravity_plugin_source(&target)
        .map_err(|_| "antigravity-plugin-conflict")?;
    let bytes =
        read_optional(&cache.join("mcp_config.json"))?.ok_or("antigravity-plugin-conflict")?;
    if format!("{:x}", Sha256::digest(&bytes)) != verified.receipt().mcp_sha256 {
        return Err("local-host-precondition-changed");
    }
    let mcp: Value = serde_json::from_slice(&bytes).map_err(|_| "antigravity-plugin-conflict")?;
    let command = mcp["mcpServers"][verified.receipt().plugin_name()]["command"]
        .as_str()
        .ok_or("antigravity-plugin-conflict")?;
    let executable = Path::new(command);
    let relative = Path::new(&verified.receipt().binary_path);
    if !executable.is_absolute() || !executable.ends_with(relative) {
        return Err("antigravity-plugin-conflict");
    }
    executable
        .ancestors()
        .nth(relative.components().count())
        .map(Path::to_path_buf)
        .ok_or("antigravity-plugin-conflict")
}

fn approve_nearest(path: &Path) -> Result<(), &'static str> {
    let mut target = path;
    while target.parent().is_some_and(|p| !p.exists()) {
        target = target.parent().ok_or("antigravity-profile-unsafe")?;
    }
    qiongli_content::approve_materialization_target(target)
        .map_err(|_| "antigravity-profile-unsafe")?;
    Ok(())
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, &'static str> {
    approve_nearest(path.parent().ok_or("antigravity-profile-unsafe")?)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("antigravity-profile-invalid"),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return Err("antigravity-profile-invalid");
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| "antigravity-profile-invalid")?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "antigravity-profile-invalid")?;
    if bytes.len() as u64 != metadata.len() {
        return Err("local-host-precondition-changed");
    }
    Ok(Some(bytes))
}

fn optional_json(path: &Path) -> Result<Value, &'static str> {
    let Some(bytes) = read_optional(path)? else {
        return Ok(Value::Null);
    };
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "antigravity-profile-invalid")?;
    if !value.is_object() {
        return Err("antigravity-profile-invalid");
    }
    Ok(value)
}

fn enabled(path: &Path, name: &str) -> Result<bool, &'static str> {
    match &optional_json(path)?["plugins"][name]["enabled"] {
        Value::Bool(value) => Ok(*value),
        Value::Null => Ok(true),
        _ => Err("antigravity-profile-invalid"),
    }
}

fn registered(path: &Path, name: &str) -> Result<bool, &'static str> {
    Ok(optional_json(path)?["imports"]
        .as_array()
        .is_some_and(|imports| {
            imports
                .iter()
                .any(|p| p["name"] == name && p["source"] == "antigravity")
        }))
}

fn run(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
) -> Result<String, &'static str> {
    crate::desktop::bounded_host_os_command_with_timeout(
        environment,
        executable,
        &args
            .iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>(),
        Duration::from_secs(30),
    )
    .map_err(|e| e.reason_code())
}
