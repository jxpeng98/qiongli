//! Terminal-approved local sources registered exclusively through official Host commands.
use std::ffi::OsString;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use qiongli_content::EmbeddedContent;
use qiongli_platform::{
    approve_claude_plugin_bundle_target, approve_codex_plugin_bundle_target,
    verify_local_claude_plugin_source, verify_local_codex_plugin_source,
};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli_content::{confirm, line, show_json};
use crate::command::CommandEnvironment;
use crate::managed_operation::ManagedIntegrationTargetV1;
use crate::plugin_source::PluginSourcePlan;

const MARKETPLACE: &str = "qiongli-cli-local";
const PLUGIN: &str = "qiongli-next@qiongli-cli-local";

#[derive(Eq, PartialEq, Serialize)]
struct HostPlan {
    executable: PathBuf,
    executable_sha256: String,
    home: PathBuf,
    config_root: PathBuf,
    source: Value,
    marketplaces: Value,
    plugins: Value,
    cache_receipt_sha256: Option<String>,
    commands: Vec<Vec<String>>,
}

pub(crate) fn register(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    source: &PluginSourcePlan,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<(), &'static str> {
    line(
        writer,
        "\nPlugin files are ready. Checking official Host registration…\n",
    )?;
    let plan = prepare(environment, content, source)?;
    if !plan.commands.is_empty() {
        let bytes =
            serde_json_canonicalizer::to_vec(&plan).map_err(|_| "local-host-plan-invalid")?;
        let preview = serde_json::json!({
            "command": "plugin-host-registration",
            "executable": plan.executable,
            "config_root": plan.config_root,
            "source": source.destination,
            "source_receipt_sha256": plan.source["source"]["receipt_sha256"],
            "cache_receipt_sha256": plan.cache_receipt_sha256,
            "plugin": PLUGIN,
            "commands": plan.commands.iter().map(|args| serde_json::json!({"arguments": serde_json::json!(args).to_string()})).collect::<Vec<_>>(),
            "plan_digest_sha256": format!("{:x}", Sha256::digest(&bytes)),
            "approvals_required": ["client-config-change", "host-trust"],
        });
        show_json(writer, &preview.to_string())?;
        line(
            writer,
            "The Host will register/enable this Plugin and may replace its verified old cache. Existing sessions need a restart.\n",
        )?;
        let reviewed_at = Instant::now();
        if !confirm(
            reader,
            writer,
            "Trust this local Plugin and run exactly these Host commands? [y/N] ",
        )? {
            return line(
                writer,
                "Host registration skipped; exported files remain available.\n",
            );
        }
        if reviewed_at.elapsed() > Duration::from_secs(600) {
            return Err("local-host-precondition-changed");
        }
        // Reuse the cross-process content guard while the Host consumes this source.
        let root = crate::command::config_root(environment).map_err(|e| e.reason_code())?;
        let _guard = crate::update_reconcile::acquire_managed_write_guard(&plan.home, root)?;
        if prepare(environment, content, source)? != plan {
            return Err("local-host-precondition-changed");
        }
        for arguments in &plan.commands {
            show_json(
                writer,
                &serde_json::json!({"running_arguments": serde_json::json!(arguments).to_string()})
                    .to_string(),
            )?;
            run(environment, &plan.executable, arguments)?;
        }
    }
    let verified = prepare(environment, content, source)?;
    if !verified.commands.is_empty() {
        return Err("local-host-registration-not-verified");
    }
    line(
        writer,
        "Plugin registration, enabled state and cached files verified. Start a new Host session to load the Skills and Full MCP. Live tools have not been checked here.\n",
    )
}

fn run(
    environment: &CommandEnvironment,
    executable: &Path,
    args: &[String],
) -> Result<String, &'static str> {
    crate::desktop::bounded_host_os_command_with_timeout(
        environment,
        executable,
        &args.iter().map(OsString::from).collect::<Vec<_>>(),
        Duration::from_secs(30),
    )
    .map_err(|e| e.reason_code())
}

/// Observe the same local source/cache contract used during registration. Never apply its commands.
pub(crate) fn inspect_inventory(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    inventory: &qiongli_platform::ClientInventory,
) -> qiongli_platform::ClientInventorySummaryV1 {
    use qiongli_platform::{
        ClientActionReadiness as Readiness, ClientComponentState as State, ClientHostPresence,
        ClientKind, ClientOwnershipState,
    };
    let mut summary = inventory.summary().clone();
    for entry in &mut summary.clients {
        if entry.host_presence != ClientHostPresence::Observed {
            continue;
        }
        let target = match entry.client {
            ClientKind::Codex => ManagedIntegrationTargetV1::Codex,
            ClientKind::ClaudeCode => ManagedIntegrationTargetV1::ClaudeCode,
        };
        match inspect_registration(environment, content, target) {
            Ok(None) => {}
            Ok(Some((version, current, cached))) => {
                entry.installed_plugin_version = cached.then_some(version);
                entry.ownership = ClientOwnershipState::QiongliManaged;
                entry.components.plugin_source = State::Ready;
                entry.components.marketplace = State::Ready;
                entry.components.registration = if cached { State::Ready } else { State::Missing };
                entry.components.skills = if cached { State::Ready } else { State::Missing };
                entry.components.full_mcp = entry.components.skills;
                entry.readiness = if current && cached {
                    Readiness::Current
                } else {
                    Readiness::RepairReady
                };
                entry.reason_code = if current && cached {
                    "local-host-registered-session-unchecked"
                } else {
                    "local-host-refresh-required"
                }
                .into();
            }
            Err(_) => {
                // Do not turn an unsupported, failed or ambiguous observation into an install suggestion.
                entry.readiness = Readiness::Unavailable;
                entry.reason_code = "local-host-observation-unavailable".into();
            }
        }
    }
    summary
}

fn inspect_registration(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    target: ManagedIntegrationTargetV1,
) -> Result<Option<(String, bool, bool)>, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let root = match target {
        ManagedIntegrationTargetV1::Codex => environment
            .codex_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".codex")),
        ManagedIntegrationTargetV1::ClaudeCode => environment
            .claude_config_root()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(".claude")),
    };
    // A read-only command must not initialize an unrelated Host. This bounded
    // hint only selects a probe; official inventories and receipts still verify it.
    let probe_config = match std::fs::symlink_metadata(root.join("plugins/cache").join(MARKETPLACE))
    {
        Ok(_) => false,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => return Err("local-host-config-unavailable"),
    };
    if probe_config {
        let registration = root.join(if target == ManagedIntegrationTargetV1::Codex {
            "config.toml"
        } else {
            "plugins/known_marketplaces.json"
        });
        match std::fs::symlink_metadata(&registration) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Ok(meta) if meta.is_file() && meta.len() <= 1_048_576 => {
                let mut text = String::new();
                std::fs::File::open(registration)
                    .map_err(|_| "local-host-config-unavailable")?
                    .take(1_048_577)
                    .read_to_string(&mut text)
                    .map_err(|_| "local-host-config-unavailable")?;
                if text.len() > 1_048_576 {
                    return Err("local-host-config-unavailable");
                }
                if !text.contains(MARKETPLACE) {
                    return Ok(None);
                }
            }
            _ => return Err("local-host-config-unavailable"),
        }
    }
    let (executable, root) = host_context(environment, target)?;
    let (marketplaces, plugins) = read_inventory(environment, &executable, Duration::from_secs(5))?;
    let codex = target == ManagedIntegrationTargetV1::Codex;
    let markets = if codex {
        &marketplaces["marketplaces"]
    } else {
        &marketplaces
    }
    .as_array()
    .ok_or("local-host-inventory-invalid")?;
    let entries = if codex {
        &plugins["installed"]
    } else {
        &plugins
    }
    .as_array()
    .ok_or("local-host-inventory-invalid")?;
    let Some(market) = markets.iter().find(|m| m["name"] == MARKETPLACE) else {
        if entries
            .iter()
            .any(|p| p[if codex { "pluginId" } else { "id" }] == PLUGIN)
        {
            return Err("local-host-marketplace-conflict");
        }
        return Ok(None);
    };
    let path = if codex {
        &market["marketplaceSource"]["source"]
    } else {
        &market["path"]
    };
    let destination = Path::new(path.as_str().ok_or("local-host-inventory-invalid")?);
    let source = crate::plugin_source::plan(
        environment,
        content,
        crate::plugin_source::PluginSourceAction::Update,
        target,
        destination,
    )?;
    let current: Value = serde_json::from_str(&crate::plugin_source::status(
        environment,
        content,
        target,
        destination,
    )?)
    .map_err(|_| "plugin-source-status-invalid")?;
    let (actions, _) = commands(&source, &root, &current, &marketplaces, &plugins)?;
    let version = current["source"]["version"]
        .as_str()
        .ok_or("plugin-source-status-invalid")?;
    Ok(Some((
        version.into(),
        current["state"] == "source-current",
        actions.is_empty(),
    )))
}

fn read_inventory(
    environment: &CommandEnvironment,
    executable: &Path,
    timeout: Duration,
) -> Result<(Value, Value), &'static str> {
    let read = |args: &[&str]| {
        let output = crate::desktop::bounded_host_os_command_with_timeout(
            environment,
            executable,
            &args.iter().map(OsString::from).collect::<Vec<_>>(),
            timeout,
        )
        .map_err(|e| e.reason_code())?;
        serde_json::from_str(&output).map_err(|_| "local-host-inventory-invalid")
    };
    Ok((
        read(&["plugin", "marketplace", "list", "--json"])?,
        read(&["plugin", "list", "--json"])?,
    ))
}

fn host_context(
    environment: &CommandEnvironment,
    target: ManagedIntegrationTargetV1,
) -> Result<(PathBuf, PathBuf), &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let (name, configured_root, client) = match target {
        ManagedIntegrationTargetV1::Codex => (
            "codex",
            environment.codex_config_root(),
            qiongli_platform::ClientActivationTarget::Codex,
        ),
        ManagedIntegrationTargetV1::ClaudeCode => (
            "claude",
            environment.claude_config_root(),
            qiongli_platform::ClientActivationTarget::ClaudeCode,
        ),
    };
    if crate::desktop::managed_integration_version_is_unsupported(environment, client) {
        return Err("local-host-version-unsupported");
    }
    let executable = environment
        .client_executable(name)
        .ok_or("host-plugin-executable-unavailable")?;
    let executable = crate::desktop::resolve_host_plugin_executable(home, name, &executable)?;
    let root = configured_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| home.join(format!(".{name}")));
    qiongli_content::approve_materialization_target(&root)
        .map_err(|_| "local-host-config-root-unsafe")?;
    Ok((executable, root))
}

fn prepare(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    source: &PluginSourcePlan,
) -> Result<HostPlan, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("host-plugin-home-unavailable")?;
    let (executable, config_root) = host_context(environment, source.target)?;
    let source_status =
        crate::plugin_source::status(environment, content, source.target, &source.destination)?;
    let current: Value =
        serde_json::from_str(&source_status).map_err(|_| "plugin-source-status-invalid")?;
    if current["state"] != "source-current" {
        return Err("local-host-source-not-current");
    }
    let (marketplaces, plugins) =
        read_inventory(environment, &executable, Duration::from_secs(30))?;
    let (commands, cache_receipt_sha256) =
        commands(source, &config_root, &current, &marketplaces, &plugins)?;
    Ok(HostPlan {
        executable_sha256: crate::cli_install::regular_file_sha256(&executable)?,
        executable,
        home: home.to_owned(),
        config_root,
        source: current,
        marketplaces,
        plugins,
        cache_receipt_sha256,
        commands,
    })
}

fn commands(
    source: &PluginSourcePlan,
    root: &Path,
    current: &Value,
    marketplaces: &Value,
    plugins: &Value,
) -> Result<(Vec<Vec<String>>, Option<String>), &'static str> {
    let codex = source.target == ManagedIntegrationTargetV1::Codex;
    let markets = if codex {
        &marketplaces["marketplaces"]
    } else {
        marketplaces
    }
    .as_array()
    .ok_or("local-host-inventory-invalid")?;
    let entries = if codex {
        &plugins["installed"]
    } else {
        plugins
    }
    .as_array()
    .ok_or("local-host-inventory-invalid")?;
    let mut registered = false;
    for market in markets {
        let name = market["name"]
            .as_str()
            .ok_or("local-host-inventory-invalid")?;
        if name == MARKETPLACE {
            let (kind, path) = if codex {
                (
                    &market["marketplaceSource"]["sourceType"],
                    &market["marketplaceSource"]["source"],
                )
            } else {
                (&market["source"], &market["path"])
            };
            if registered
                || kind != if codex { "local" } else { "directory" }
                || !same_path(&source.destination, path)
            {
                return Err("local-host-marketplace-conflict");
            }
            registered = true;
        }
    }
    let mut installed = None;
    for plugin in entries {
        let id = plugin[if codex { "pluginId" } else { "id" }]
            .as_str()
            .ok_or("local-host-inventory-invalid")?;
        if plugin["enabled"].as_bool().is_none() {
            return Err("local-host-inventory-invalid");
        }
        if id == PLUGIN {
            if installed.replace(plugin).is_some() || (!codex && plugin["scope"] != "user") {
                return Err("local-host-plugin-scope-conflict");
            }
        } else if matches!(id.split('@').next(), Some("qiongli" | "qiongli-next"))
            && plugin["enabled"] != false
        {
            return Err("local-host-other-qiongli-enabled");
        }
    }
    let mut commands = Vec::new();
    let mut cached_receipt = None;
    if let Some(plugin) = installed {
        if !registered
            || (codex
                && (plugin["installed"] != true
                    || plugin["source"]["source"] != "local"
                    || !same_path(&source.destination, &plugin["source"]["path"])))
        {
            return Err("local-host-plugin-source-conflict");
        }
        let version = plugin["version"]
            .as_str()
            .filter(|v| semver::Version::parse(v).is_ok())
            .ok_or("local-host-inventory-invalid")?;
        let cache = root
            .join("plugins/cache")
            .join(MARKETPLACE)
            .join("qiongli-next")
            .join(version);
        if !codex && !same_path(&cache, &plugin["installPath"]) {
            return Err("local-host-cache-conflict");
        }
        let receipt = cache_receipt(source.target, &cache, version)?;
        if current["source"]["receipt_sha256"] == receipt && plugin["enabled"] == true {
            return Ok((commands, Some(receipt)));
        }
        cached_receipt = Some(receipt);
        // Only the exact verified local Plugin cache can be replaced by its Host.
        commands.push(if codex {
            words(&["plugin", "remove", PLUGIN])
        } else {
            words(&["plugin", "uninstall", PLUGIN, "--scope", "user"])
        });
    }
    if !registered {
        commands.push(vec![
            "plugin".into(),
            "marketplace".into(),
            "add".into(),
            source
                .destination
                .to_str()
                .ok_or("plugin-source-destination-invalid")?
                .into(),
        ]);
    }
    commands.push(if codex {
        words(&["plugin", "add", PLUGIN])
    } else {
        words(&["plugin", "install", PLUGIN, "--scope", "user"])
    });
    Ok((commands, cached_receipt))
}

fn cache_receipt(
    target: ManagedIntegrationTargetV1,
    path: &Path,
    version: &str,
) -> Result<String, &'static str> {
    match target {
        ManagedIntegrationTargetV1::Codex => {
            let target = approve_codex_plugin_bundle_target(path).map_err(|e| e.reason_code())?;
            verify_local_codex_plugin_source(&target)
                .map_err(|e| e.reason_code())
                .and_then(|v| {
                    if v.receipt().artifact.version != version {
                        return Err("local-host-cache-version-mismatch");
                    }
                    Ok(v.receipt_sha256().to_owned())
                })
        }
        ManagedIntegrationTargetV1::ClaudeCode => {
            let target = approve_claude_plugin_bundle_target(path).map_err(|e| e.reason_code())?;
            verify_local_claude_plugin_source(&target)
                .map_err(|e| e.reason_code())
                .and_then(|v| {
                    if v.receipt().artifact.version != version {
                        return Err("local-host-cache-version-mismatch");
                    }
                    Ok(v.receipt_sha256().to_owned())
                })
        }
    }
}

fn same_path(expected: &Path, observed: &Value) -> bool {
    observed
        .as_str()
        .map(Path::new)
        .filter(|p| p.is_absolute())
        .and_then(|p| std::fs::canonicalize(p).ok())
        .zip(std::fs::canonicalize(expected).ok())
        .is_some_and(|(a, b)| a == b)
}

fn words(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn local_host_cache_replacement_requires_a_complete_matching_local_receipt() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .canonicalize()
            .unwrap()
            .join(format!("local-host-cache-{}", std::process::id()));
        std::fs::create_dir(&base).unwrap();
        let binary = base.join("binary");
        std::fs::write(&binary, b"local-source-fixture").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let digest = format!("{:x}", Sha256::digest(b"local-source-fixture"));
        let content = crate::embedded_content().unwrap();
        for target in [
            ManagedIntegrationTargetV1::Codex,
            ManagedIntegrationTargetV1::ClaudeCode,
        ] {
            let codex = target == ManagedIntegrationTargetV1::Codex;
            let root = base.join(if codex { "codex" } else { "claude" });
            let cache = root
                .join("plugins/cache/qiongli-cli-local/qiongli-next")
                .join(env!("CARGO_PKG_VERSION"));
            std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
            let exported = root.join("qiongli-next");
            if codex {
                let target = approve_codex_plugin_bundle_target(&exported).unwrap();
                qiongli_platform::compose_local_codex_plugin_source(
                    content.pack(),
                    &binary,
                    &digest,
                    &target,
                    None,
                    None,
                )
                .unwrap();
            } else {
                let target = approve_claude_plugin_bundle_target(&exported).unwrap();
                qiongli_platform::compose_local_claude_plugin_source(
                    content.pack(),
                    &binary,
                    &digest,
                    &target,
                    None,
                    None,
                )
                .unwrap();
            }
            std::fs::rename(exported, &cache).unwrap();
            let receipt = cache_receipt(target, &cache, env!("CARGO_PKG_VERSION")).unwrap();
            assert_eq!(
                cache_receipt(target, &cache, "2.0.0-beta.999").unwrap_err(),
                "local-host-cache-version-mismatch"
            );
            let mut source = crate::plugin_source::contract_source();
            source.target = target;
            source.destination = base.clone();
            let markets = if codex {
                json!({"marketplaces":[{"name":MARKETPLACE,"marketplaceSource":{"sourceType":"local","source":base}}]})
            } else {
                json!([{"name":MARKETPLACE,"source":"directory","path":base}])
            };
            let entry = if codex {
                json!({"pluginId":PLUGIN,"version":env!("CARGO_PKG_VERSION"),"installed":true,"enabled":true,"source":{"source":"local","path":base}})
            } else {
                json!({"id":PLUGIN,"version":env!("CARGO_PKG_VERSION"),"scope":"user","enabled":true,"installPath":cache})
            };
            let plugins = if codex {
                json!({"installed":[entry]})
            } else {
                json!([entry])
            };
            let current = json!({"source":{"receipt_sha256":receipt}});
            assert!(
                commands(&source, &root, &current, &markets, &plugins)
                    .unwrap()
                    .0
                    .is_empty()
            );
            let updated = json!({"source":{"receipt_sha256":"a".repeat(64)}});
            let (replacement, observed) =
                commands(&source, &root, &updated, &markets, &plugins).unwrap();
            assert_eq!(observed.as_deref(), Some(receipt.as_str()));
            assert_eq!(replacement.len(), 2);
            assert_eq!(
                replacement[0][1],
                if codex { "remove" } else { "uninstall" }
            );
            std::fs::write(cache.join("user-notes.txt"), "preserve").unwrap();
            assert!(commands(&source, &root, &updated, &markets, &plugins).is_err());
            assert_eq!(
                std::fs::read_to_string(cache.join("user-notes.txt")).unwrap(),
                "preserve"
            );
        }
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn local_host_commands_reject_foreign_sources_duplicates_and_malformed_inventories() {
        let mut source = crate::plugin_source::contract_source();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .canonicalize()
            .unwrap();
        source.destination = root.clone();
        for target in [
            ManagedIntegrationTargetV1::Codex,
            ManagedIntegrationTargetV1::ClaudeCode,
        ] {
            source.target = target;
            let codex = target == ManagedIntegrationTargetV1::Codex;
            let empty_markets = if codex {
                json!({"marketplaces": []})
            } else {
                json!([])
            };
            let empty_plugins = if codex {
                json!({"installed": []})
            } else {
                json!([])
            };
            let (install, receipt) =
                commands(&source, &root, &Value::Null, &empty_markets, &empty_plugins).unwrap();
            assert_eq!(
                install[0],
                vec!["plugin", "marketplace", "add", root.to_str().unwrap()]
            );
            assert_eq!(install[1][1], if codex { "add" } else { "install" });
            assert!(receipt.is_none());
            assert!(commands(&source, &root, &Value::Null, &Value::Null, &empty_plugins).is_err());
            assert!(commands(&source, &root, &Value::Null, &empty_markets, &Value::Null).is_err());
            let foreign = json!({if codex {"pluginId"} else {"id"}: "qiongli-next@personal", "enabled": true});
            let plugins = if codex {
                json!({"installed": [foreign]})
            } else {
                json!([foreign])
            };
            assert_eq!(
                commands(&source, &root, &Value::Null, &empty_markets, &plugins).unwrap_err(),
                "local-host-other-qiongli-enabled"
            );
            let market = if codex {
                json!({"name": MARKETPLACE, "marketplaceSource": {"sourceType":"local", "source":root}})
            } else {
                json!({"name":MARKETPLACE, "source":"directory", "path":root})
            };
            let duplicate = if codex {
                json!({"marketplaces":[market,market]})
            } else {
                json!([market, market])
            };
            assert_eq!(
                commands(&source, &root, &Value::Null, &duplicate, &empty_plugins).unwrap_err(),
                "local-host-marketplace-conflict"
            );
            let markets = if codex {
                json!({"marketplaces":[market]})
            } else {
                json!([market])
            };
            let (registered, _) =
                commands(&source, &root, &Value::Null, &markets, &empty_plugins).unwrap();
            assert_eq!(registered.len(), 1);
            let plugin = json!({if codex {"pluginId"} else {"id"}: PLUGIN, "scope":"project", "enabled":true});
            let plugins = if codex {
                json!({"installed":[plugin,plugin]})
            } else {
                json!([plugin])
            };
            assert_eq!(
                commands(&source, &root, &Value::Null, &markets, &plugins).unwrap_err(),
                "local-host-plugin-scope-conflict"
            );
        }
    }
}
