//! A confirmed Plugin migration uses Codex's version-checked configuration API.
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{Value, json};

use crate::command::CommandEnvironment;

const RPC_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Eq, PartialEq, Serialize)]
pub(super) struct PluginMigration {
    pub plugins: Vec<String>,
    file: PathBuf,
    version: String,
    file_sha256: String,
}

impl PluginMigration {
    pub fn read(
        environment: &CommandEnvironment,
        executable: &Path,
        root: &Path,
        plugins: Vec<String>,
        writer: &mut impl Write,
    ) -> Result<Option<Self>, &'static str> {
        let file = root.join("config.toml");
        let metadata = match std::fs::symlink_metadata(&file) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && plugins.is_empty() => {
                return Ok(None);
            }
            Err(_) => return Err("local-host-migration-config-unavailable"),
        };
        if !metadata.is_file() || metadata.len() > 1_048_576 {
            return Err("local-host-migration-config-unavailable");
        }
        let file_sha256 = crate::cli_install::regular_file_sha256(&file)?;
        let response = rpc(
            environment,
            executable,
            "config/read",
            json!({"includeLayers": true}),
            writer,
        )?;
        Self::from_config(file, plugins, &response, file_sha256)
    }

    fn from_config(
        file: PathBuf,
        mut plugins: Vec<String>,
        response: &Value,
        file_sha256: String,
    ) -> Result<Option<Self>, &'static str> {
        if plugins.len() > 16
            || plugins
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != plugins.len()
            || plugins.iter().any(|id| {
                id.len() > 160
                    || id.matches('@').count() != 1
                    || id.ends_with('@')
                    || !id
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"@._-".contains(&c))
                    || id == super::plugin_id()
                    || !matches!(
                        id.split('@').next(),
                        Some(
                            "qiongli"
                                | "qiongli-macos-arm64"
                                | "qiongli-windows-x64"
                                | "qiongli-linux-x64"
                                | "qiongli-next"
                                | "qiongli-next-macos-arm64"
                                | "qiongli-next-windows-x64"
                                | "qiongli-next-linux-x64"
                        )
                    )
            })
        {
            return Err("local-host-inventory-invalid");
        }
        let layers = response["layers"]
            .as_array()
            .ok_or("local-host-migration-config-unavailable")?;
        let mut users = layers.iter().filter(|layer| {
            layer["name"]["type"] == "user"
                && layer["name"]["profile"].is_null()
                && super::same_path(&file, &layer["name"]["file"])
        });
        let layer = users
            .next()
            .ok_or("local-host-migration-config-unavailable")?;
        if users.next().is_some()
            || plugins
                .iter()
                .any(|id| layer["config"]["plugins"][id]["enabled"] != true)
        {
            return Err("local-host-migration-config-unavailable");
        }
        // A renamed local catalog hides the old ID from `plugin list`, while
        // its user configuration remains enabled. Include that exact sibling.
        let previous = if super::plugin_name() == "qiongli" {
            "qiongli-next@qiongli-cli-local"
        } else {
            "qiongli@qiongli-cli-local"
        };
        if layer["config"]["plugins"][previous]["enabled"] == true
            && !plugins.iter().any(|id| id == previous)
        {
            plugins.push(previous.into());
        }
        if plugins.len() > 16 {
            return Err("local-host-inventory-invalid");
        }
        plugins.sort();
        let version = layer["version"]
            .as_str()
            .filter(|version| !version.is_empty() && version.len() <= 128)
            .ok_or("local-host-migration-config-unavailable")?;
        // Host versions are opaque and can describe normalized configuration,
        // rather than raw TOML bytes. Bind both identities without conflating them.
        if file_sha256 != crate::cli_install::regular_file_sha256(&file)? {
            return Err("local-host-precondition-changed");
        }
        if plugins.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            plugins,
            file,
            version: version.into(),
            file_sha256,
        }))
    }

    pub fn request(&self) -> Value {
        json!({
            "method": "config/batchWrite",
            "params": {
                "filePath": self.file,
                "expectedVersion": self.version,
                "edits": self.plugins.iter().map(|id| json!({
                    "keyPath": format!("plugins.{}.enabled", json!(id)),
                    "value": false,
                    "mergeStrategy": "replace"
                })).collect::<Vec<_>>()
            }
        })
    }

    pub fn apply(
        &self,
        environment: &CommandEnvironment,
        executable: &Path,
        writer: &mut impl Write,
    ) -> Result<(), &'static str> {
        if self.file_sha256 != crate::cli_install::regular_file_sha256(&self.file)? {
            return Err("local-host-precondition-changed");
        }
        let response = rpc(
            environment,
            executable,
            "config/batchWrite",
            self.request()["params"].clone(),
            writer,
        )?;
        if response["status"] != "ok" || !super::same_path(&self.file, &response["filePath"]) {
            return Err("local-host-migration-not-verified");
        }
        Ok(())
    }
}

fn rpc(
    environment: &CommandEnvironment,
    executable: &Path,
    method: &'static str,
    params: Value,
    writer: &mut impl Write,
) -> Result<Value, &'static str> {
    let args = ["app-server", "--listen", "stdio://"];
    let result = crate::desktop::official_host_command(
        environment,
        executable,
        &args.map(std::ffi::OsString::from),
    )
    .map_err(|error| error.reason_code())
    .and_then(|command| run_rpc(command, method, params, RPC_TIMEOUT));
    if let Err(code) = result {
        let command = super::installation_command::display_command(
            &executable.to_string_lossy(),
            &super::words(&args),
        );
        crate::cli_content::line(
            writer,
            &format!(
                "Codex Plugin migration check failed at {method} ({code}, limit {}s).\n  Command: {command}\n",
                RPC_TIMEOUT.as_secs(),
            ),
        )?;
        if method == "config/batchWrite" {
            crate::cli_content::line(
                writer,
                "Check which Qiongli Plugins are enabled in Codex before retrying. The configuration request may already have taken effect.\n",
            )?;
        }
    }
    result
}

fn run_rpc(
    mut command: Command,
    method: &'static str,
    params: Value,
    timeout: Duration,
) -> Result<Value, &'static str> {
    // Only the resolved official Host is launched. No model/session request is sent.
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "local-host-migration-config-unavailable")?;
    let input = child.stdin.take().expect("configured piped stdin");
    let output = child.stdout.take().expect("configured piped stdout");
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let deadline = Instant::now() + timeout;
    std::thread::spawn(move || {
        let mut input = input;
        let mut output = BufReader::new(output.take(2 * 1024 * 1024));
        let result = (|| {
            exchange(
                &mut output,
                &mut input,
                1,
                "initialize",
                json!({"clientInfo":{"name":"qiongli-plugin-install","version":env!("CARGO_PKG_VERSION")}}),
            )?;
            if Instant::now() >= deadline {
                return Err("host-command-timeout");
            }
            input
                .write_all(b"{\"method\":\"initialized\"}\n")
                .map_err(|_| "local-host-migration-config-unavailable")?;
            if Instant::now() >= deadline {
                return Err("host-command-timeout");
            }
            exchange(&mut output, &mut input, 2, method, params)
        })();
        let _ = sender.send(result);
    });
    let result = match receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err("host-command-timeout"),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err("local-host-migration-config-unavailable")
        }
    };
    let _ = child.kill();
    let _ = child.wait();
    // A helper process can retain a pipe after the direct app-server child
    // stops. Let the I/O worker finish when the pipe closes, without blocking
    // the installer or sending a late migration request after the deadline.
    result
}

fn exchange(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    id: u64,
    method: &str,
    params: Value,
) -> Result<Value, &'static str> {
    serde_json::to_writer(
        &mut *writer,
        &json!({"id":id,"method":method,"params":params}),
    )
    .map_err(|_| "local-host-migration-config-unavailable")?;
    writer
        .write_all(b"\n")
        .and_then(|()| writer.flush())
        .map_err(|_| "local-host-migration-config-unavailable")?;
    loop {
        let mut line = String::new();
        if reader
            .read_line(&mut line)
            .map_err(|_| "local-host-migration-config-unavailable")?
            == 0
        {
            return Err("local-host-migration-config-unavailable");
        }
        let response: Value =
            serde_json::from_str(&line).map_err(|_| "local-host-migration-config-unavailable")?;
        if response["id"] == id {
            return if response.get("error").is_none() && response["result"].is_object() {
                Ok(response["result"].clone())
            } else {
                Err("local-host-migration-config-unavailable")
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    #[allow(
        clippy::disallowed_methods,
        reason = "fixed synthetic app-server process"
    )]
    fn rpc_timeout_returns_while_a_helper_still_holds_the_pipe() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "(sleep 3) & IFS= read -r request; wait"]);
        let start = Instant::now();
        assert_eq!(
            run_rpc(
                command,
                "config/read",
                json!({}),
                Duration::from_millis(250)
            ),
            Err("host-command-timeout")
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn migration_rejects_unowned_entries_injection_duplicates_and_stale_config() {
        let file =
            std::env::temp_dir().join(format!("qiongli-migration-{}.toml", std::process::id()));
        let bytes = "model = 'preserve-model'\n[plugins.'qiongli-next@personal']\nenabled = true\n";
        std::fs::write(&file, bytes).unwrap();
        let id = "qiongli-next@personal";
        let digest = crate::cli_install::regular_file_sha256(&file).unwrap();
        let response = json!({"layers":[{
            "name":{"type":"user","file":file},
            "version":"opaque-host-config-version",
            "config":{"plugins":{id:{"enabled":true}}}
        }]});
        let migration =
            PluginMigration::from_config(file.clone(), vec![id.into()], &response, digest.clone())
                .unwrap()
                .unwrap();
        assert_eq!(
            migration.request()["params"]["edits"],
            json!([{
                "keyPath":"plugins.\"qiongli-next@personal\".enabled", "value":false, "mergeStrategy":"replace"
            }])
        );
        for ids in [
            vec![id, id],
            vec![super::super::plugin_id()],
            vec!["other@personal"],
            vec!["qiongli-next@x\".model"],
            vec!["qiongli-next@"],
            vec!["qiongli-next@x@y"],
        ] {
            assert!(
                PluginMigration::from_config(
                    file.clone(),
                    ids.into_iter().map(str::to_owned).collect(),
                    &response,
                    digest.clone()
                )
                .is_err()
            );
        }
        assert!(
            PluginMigration::from_config(file.clone(), vec![], &response, digest.clone())
                .unwrap()
                .is_none()
        );
        let previous = if super::super::plugin_name() == "qiongli" {
            "qiongli-next@qiongli-cli-local"
        } else {
            "qiongli@qiongli-cli-local"
        };
        let mut hidden = response.clone();
        hidden["layers"][0]["config"]["plugins"][previous] = json!({"enabled": true});
        let hidden_migration =
            PluginMigration::from_config(file.clone(), vec![], &hidden, digest.clone())
                .unwrap()
                .unwrap();
        assert_eq!(hidden_migration.plugins, vec![previous]);
        hidden["layers"][0]["config"]["plugins"][previous]["enabled"] = json!(false);
        assert!(
            PluginMigration::from_config(file.clone(), vec![], &hidden, digest.clone())
                .unwrap()
                .is_none()
        );
        for pointer in [
            "/layers/0/name/type",
            "/layers/0/name/profile",
            "/layers/0/config/plugins/qiongli-next@personal/enabled",
        ] {
            let mut invalid = response.clone();
            // profile is absent in the valid fixture; set it explicitly for this case.
            if pointer.ends_with("/profile") {
                invalid["layers"][0]["name"]["profile"] = json!("different-profile");
            } else {
                *invalid.pointer_mut(pointer).unwrap() = json!(false);
            }
            assert!(
                PluginMigration::from_config(
                    file.clone(),
                    vec![id.into()],
                    &invalid,
                    digest.clone()
                )
                .is_err()
            );
        }
        assert_eq!(std::fs::read_to_string(&file).unwrap(), bytes);
        std::fs::write(&file, format!("{bytes}# concurrent edit\n")).unwrap();
        assert!(matches!(
            PluginMigration::from_config(file.clone(), vec![id.into()], &response, digest.clone()),
            Err("local-host-precondition-changed")
        ));
        std::fs::remove_file(file).unwrap();
    }

    #[test]
    fn rpc_requires_a_matching_success_response_without_echoing_host_errors() {
        let mut output = Vec::new();
        let mut input =
            b"{\"method\":\"notification\"}\n{\"id\":2,\"result\":{\"status\":\"ok\"}}\n"
                .as_slice();
        assert_eq!(
            exchange(&mut input, &mut output, 2, "config/batchWrite", json!({})).unwrap()["status"],
            "ok"
        );
        for invalid in [
            "",
            "not json\n",
            "{\"id\":1,\"result\":{}}\n",
            "{\"id\":2,\"error\":{\"message\":\"private config\"}}\n",
        ] {
            assert_eq!(
                exchange(
                    &mut invalid.as_bytes(),
                    &mut Vec::new(),
                    2,
                    "config/read",
                    json!({})
                )
                .unwrap_err(),
                "local-host-migration-config-unavailable"
            );
        }
    }
}
