//! Terminal-approved local Pi package; Pi owns registration and MCP transport.
use std::fs;
use std::io::{BufRead, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use qiongli_config::WorkflowVariantStore;
use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    approve_pi_plugin_bundle_target, compose_local_pi_plugin_source, verify_local_pi_plugin_source,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli_content::{confirm, line, show_json};
use crate::command::{CommandEnvironment, config_root};

#[cfg(test)]
#[path = "pi_tests.rs"]
mod tests;

#[derive(Debug, Eq, PartialEq, Serialize)]
struct Plan {
    executable: PathBuf,
    executable_sha256: String,
    source: PathBuf,
    source_receipt_sha256: Option<String>,
    binary_sha256: String,
    content_pack_sha256: String,
    workflow_variant_sha256: Option<String>,
    language: String,
    agent_dir: PathBuf,
    profile_files: Vec<(String, Option<String>)>,
    commands: Vec<Vec<String>>,
}

pub(crate) fn install(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    destination: Option<&Path>,
    language: &str,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<bool, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let discovered = environment
        .client_executable("pi")
        .ok_or("host-plugin-executable-unavailable")?;
    let executable = crate::desktop::resolve_host_plugin_executable(home, "pi", &discovered)?;
    let version = run(environment, &executable, &["--version".into()])?;
    validate_version(&version)?;
    let source = select_source(environment, destination)?;
    let plan = prepare(environment, content, &executable, &source, language)?;
    preview(writer, "pi-plugin-files", &plan)?;
    line(
        writer,
        "Export qiongli and no-qiongli Skills, internal workflows and a native Full MCP extension. Pi loads this source directly; retain the directory while installed.\n",
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
        recheck(environment, content, &plan, reviewed)?;
        let variant = WorkflowVariantStore::new(root)
            .load(content.pack())
            .map_err(|e| e.reason_code())?;
        let target = approve_pi_plugin_bundle_target(&source).map_err(|e| e.reason_code())?;
        let binary = std::env::current_exe().map_err(|_| "plugin-source-executable-unavailable")?;
        compose_local_pi_plugin_source(
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
    let plan = prepare(environment, content, &executable, &source, language)?;
    if !plan.commands.is_empty() {
        preview(writer, "pi-plugin-registration", &plan)?;
        line(
            writer,
            "Pi will register this local package in its user settings. Its extension registers Full MCP inside sessions. New Unix profile files use private permissions (umask 077); existing permissions stay unchanged. No model session starts during installation.\n",
        )?;
        let reviewed = Instant::now();
        if !confirm(
            reader,
            writer,
            "Trust this package and run the displayed Pi command? [y/N] ",
        )? {
            line(
                writer,
                "Host registration skipped; exported files remain. Session tools: not checked.\n",
            )?;
            return Ok(false);
        }
        let root = config_root(environment).map_err(|e| e.reason_code())?;
        let _guard = crate::update_reconcile::acquire_managed_write_guard(home, root)?;
        recheck(environment, content, &plan, reviewed)?;
        let before = read_json(&plan.agent_dir.join("settings.json"))?;
        for args in &plan.commands {
            run(environment, &executable, args)?;
        }
        let after = read_json(&plan.agent_dir.join("settings.json"))?;
        verify_registration(&before, &after, &plan.agent_dir, home, &source)?;
    }
    let installed = prepare(environment, content, &executable, &source, language)?;
    if !installed.commands.is_empty()
        || installed.source_receipt_sha256 != plan.source_receipt_sha256
        || installed
            .profile_files
            .iter()
            .find(|(name, _)| name == "mcp.json")
            != plan
                .profile_files
                .iter()
                .find(|(name, _)| name == "mcp.json")
    {
        return Err("local-host-registration-not-verified");
    }
    show_json(
        writer,
        &serde_json::json!({"host":"Pi", "version":version.trim(),
        "source":source, "agent_dir":installed.agent_dir,
        "receipt_sha256":installed.source_receipt_sha256, "session_tools":"not-checked"})
        .to_string(),
    )?;
    line(
        writer,
        "Pi: local package registered; qiongli and no-qiongli Skills plus Full MCP extension. Start a new session or /reload, inspect /mcp, then call qiongli_config_status. Shell pi mcp list does not load extensions. Uninstall with pi remove <source> before removing the source directory.\n",
    )?;
    Ok(true)
}

fn validate_version(value: &str) -> Result<(), &'static str> {
    let version = semver::Version::parse(value.trim()).map_err(|_| "pi-version-unsupported")?;
    if version < semver::Version::new(0, 99, 0) || !version.pre.is_empty() {
        return Err("pi-version-unsupported");
    }
    Ok(())
}

fn agent_dir(environment: &CommandEnvironment) -> Result<PathBuf, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let path = environment
        .pi_config_root()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(".pi/agent"));
    approve_nearest(&path)?;
    Ok(path)
}

fn select_source(
    environment: &CommandEnvironment,
    destination: Option<&Path>,
) -> Result<PathBuf, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let agent = agent_dir(environment)?;
    let settings = read_json(&agent.join("settings.json"))?;
    let mut sources = Vec::new();
    for package in packages(&settings)? {
        let raw = package_source(package)?;
        if is_qiongli_source(raw) {
            let path = resolve_source(&agent, home, raw).ok_or("pi-package-conflict")?;
            if !sources.contains(&path) {
                sources.push(path);
            }
        }
    }
    if sources.len() > 1 {
        return Err("pi-package-conflict");
    }
    let source = destination
        .map(Path::to_path_buf)
        .or_else(|| sources.first().cloned())
        .unwrap_or_else(|| home.join("qiongli-pi"));
    if sources.first().is_some_and(|old| old != &source) {
        return Err("pi-package-conflict");
    }
    Ok(source)
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
            Some("qiongli" | "qiongli-next" | "qiongli-pi")
        )
    {
        return Err("plugin-source-destination-invalid");
    }
    if !qiongli_content::skill_language_valid(language) {
        return Err("skill-language-invalid");
    }
    let agent = agent_dir(environment)?;
    if agent.starts_with(source) {
        return Err("plugin-source-destination-reserved");
    }
    for reserved in [
        home.join(".pi"),
        agent.clone(),
        home.join(".gemini"),
        home.join(".agents"),
        home.join(".qiongli"),
        config_root(environment)
            .map_err(|e| e.reason_code())?
            .compatibility_root()
            .to_path_buf(),
        environment
            .codex_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".codex")),
        environment
            .claude_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".claude")),
        environment
            .dsh_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".dsh")),
    ] {
        if source.starts_with(reserved) {
            return Err("plugin-source-destination-reserved");
        }
    }
    let target = approve_pi_plugin_bundle_target(source).map_err(|e| e.reason_code())?;
    let source_receipt_sha256 = match fs::symlink_metadata(source) {
        Ok(_) => Some(
            verify_local_pi_plugin_source(&target)
                .map_err(|e| e.reason_code())?
                .receipt_sha256()
                .to_owned(),
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => return Err("plugin-source-destination-invalid"),
    };
    let settings = read_json(&agent.join("settings.json"))?;
    let mcp = read_json(&agent.join("mcp.json"))?;
    if mcp.get("mcpServers").is_some_and(|v| !v.is_object()) {
        return Err("pi-profile-invalid");
    }
    if ["qiongli", "qiongli-next", "qiongli_next"]
        .iter()
        .any(|name| mcp["mcpServers"].get(name).is_some())
    {
        return Err("pi-mcp-conflict");
    }
    if let Some(extensions) = settings.get("extensions") {
        let extensions = extensions.as_array().ok_or("pi-profile-invalid")?;
        if extensions.iter().any(|v| v == "-builtin:mcp") {
            return Err("pi-package-filtered");
        }
    }
    let registered = registered(&settings, &agent, home, source)?;
    let profile_files = ["settings.json", "mcp.json"]
        .into_iter()
        .map(|name| {
            Ok((
                name.into(),
                read_optional(&agent.join(name))?.map(|b| format!("{:x}", Sha256::digest(b))),
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
        agent_dir: agent,
        profile_files,
        commands: if registered {
            vec![]
        } else {
            vec![vec![
                "install".into(),
                source
                    .to_str()
                    .ok_or("plugin-source-destination-invalid")?
                    .into(),
            ]]
        },
    })
}

fn recheck(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    plan: &Plan,
    reviewed: Instant,
) -> Result<(), &'static str> {
    if reviewed.elapsed() > Duration::from_secs(600)
        || prepare(
            environment,
            content,
            &plan.executable,
            &plan.source,
            &plan.language,
        )? != *plan
    {
        return Err("local-host-precondition-changed");
    }
    Ok(())
}

fn packages(settings: &Value) -> Result<&[Value], &'static str> {
    match settings.get("packages") {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        _ => Err("pi-profile-invalid"),
    }
}

fn package_source(package: &Value) -> Result<&str, &'static str> {
    package
        .as_str()
        .or_else(|| package.get("source").and_then(Value::as_str))
        .filter(|s| !s.is_empty() && !s.chars().any(char::is_control))
        .ok_or("pi-profile-invalid")
}

fn is_qiongli_source(source: &str) -> bool {
    let npm_name = source
        .strip_prefix("npm:")
        .unwrap_or(source)
        .split('@')
        .next()
        .unwrap_or_default();
    matches!(npm_name, "qiongli" | "qiongli-next")
        || matches!(
            Path::new(source).file_name().and_then(|s| s.to_str()),
            Some("qiongli" | "qiongli-next" | "qiongli-pi")
        )
}

fn resolve_source(agent: &Path, home: &Path, source: &str) -> Option<PathBuf> {
    if source.starts_with("npm:") || source.starts_with("git:") || source.contains("://") {
        return None;
    }
    let path = if let Some(relative) = source.strip_prefix("~/") {
        home.join(relative)
    } else {
        agent.join(source)
    };
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            c => normalized.push(c.as_os_str()),
        }
    }
    normalized.is_absolute().then_some(normalized)
}

fn registered(
    settings: &Value,
    agent: &Path,
    home: &Path,
    source: &Path,
) -> Result<bool, &'static str> {
    let mut found = false;
    for package in packages(settings)? {
        let raw = package_source(package)?;
        if resolve_source(agent, home, raw).as_deref() == Some(source) {
            // Respect resource filters; installing the same path must not silently undo them.
            if package.as_object().is_some_and(|o| o.len() != 1) {
                return Err("pi-package-filtered");
            }
            if found {
                return Err("pi-package-conflict");
            }
            found = true;
        } else if is_qiongli_source(raw) {
            return Err("pi-package-conflict");
        }
    }
    Ok(found)
}

fn verify_registration(
    before: &Value,
    after: &Value,
    agent: &Path,
    home: &Path,
    source: &Path,
) -> Result<(), &'static str> {
    if !registered(after, agent, home, source)? {
        return Err("local-host-registration-not-verified");
    }
    let without_package = |value: &Value| -> Result<Value, &'static str> {
        let mut value = value.clone();
        let keep = packages(&value)?
            .iter()
            .filter(|p| {
                package_source(p)
                    .ok()
                    .and_then(|s| resolve_source(agent, home, s))
                    .as_deref()
                    != Some(source)
            })
            .cloned()
            .collect::<Vec<_>>();
        let object = value.as_object_mut().ok_or("pi-profile-invalid")?;
        object.remove("packages");
        if !keep.is_empty() {
            object.insert("packages".into(), Value::Array(keep));
        }
        Ok(value)
    };
    if without_package(before)? != without_package(after)? {
        return Err("local-host-registration-not-verified");
    }
    Ok(())
}

fn preview(writer: &mut impl Write, operation: &str, plan: &Plan) -> Result<(), &'static str> {
    let bytes =
        serde_json_canonicalizer::to_vec(plan).map_err(|_| "installation-preview-invalid")?;
    show_json(writer, &serde_json::json!({"operation":operation, "plan":plan,
        "plan_digest_sha256":format!("{:x}", Sha256::digest(bytes)),
        "unix_creation_mask": if cfg!(unix) { Some("077") } else { None },
        "approvals_required": if operation.ends_with("files") {vec!["filesystem-write"]} else {vec!["client-config-change", "host-trust"]}}).to_string())
}

fn approve_nearest(path: &Path) -> Result<(), &'static str> {
    if !path.is_absolute() {
        return Err("pi-profile-unsafe");
    }
    let mut target = path;
    while target.parent().is_some_and(|p| !p.exists()) {
        target = target.parent().ok_or("pi-profile-unsafe")?;
    }
    qiongli_content::approve_materialization_target(target).map_err(|_| "pi-profile-unsafe")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::symlink_metadata(target).is_ok_and(|m| m.permissions().mode() & 0o022 != 0) {
            return Err("pi-profile-unsafe");
        }
    }
    Ok(())
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, &'static str> {
    approve_nearest(path.parent().ok_or("pi-profile-unsafe")?)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("pi-profile-invalid"),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 1024 * 1024 {
        return Err("pi-profile-invalid");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 || metadata.mode() & 0o022 != 0 {
            return Err("pi-profile-invalid");
        }
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| "pi-profile-invalid")?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "pi-profile-invalid")?;
    if bytes.len() as u64 != metadata.len() {
        return Err("local-host-precondition-changed");
    }
    Ok(Some(bytes))
}

fn read_json(path: &Path) -> Result<Value, &'static str> {
    let Some(bytes) = read_optional(path)? else {
        return Ok(serde_json::json!({}));
    };
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "pi-profile-invalid")?;
    if !value.is_object() {
        return Err("pi-profile-invalid");
    }
    Ok(value)
}

fn run(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
) -> Result<String, &'static str> {
    crate::desktop::bounded_private_host_os_command_with_timeout(
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
