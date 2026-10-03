//! Interactive presentation over the existing managed content write owner.
use std::io::{self, BufRead, IsTerminal, Read, Write};

use qiongli_content::EmbeddedContent;

use crate::managed_operation::ManagedOperationCliCommand;
use crate::{CliOutput, CommandEnvironment};

pub(crate) fn system_skill_language(environment: &CommandEnvironment) -> &'static str {
    #[cfg(not(any(target_os = "macos", windows)))]
    let _ = environment;
    for key in ["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"] {
        if let Ok(value) = std::env::var(key)
            && !value.trim().is_empty()
            && !matches!(value.as_str(), "C" | "POSIX" | "C.UTF-8")
        {
            return qiongli_content::skill_language_for_locale(&value);
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(output) = locale_command(
        environment,
        std::path::Path::new("/usr/bin/defaults"),
        &["read", "-g", "AppleLanguages"],
    ) && let Some(locale) = output
        .split(['"', '(', ')', ',', '\n'])
        .map(str::trim)
        .find(|s| !s.is_empty())
    {
        return qiongli_content::skill_language_for_locale(locale);
    }
    #[cfg(windows)]
    if let Some(root) = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
        && let Some(output) = locale_command(
            environment,
            &root.join("System32/WindowsPowerShell/v1.0/powershell.exe"),
            &[
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[System.Globalization.CultureInfo]::CurrentUICulture.Name",
            ],
        )
    {
        return qiongli_content::skill_language_for_locale(&output);
    }
    "en"
}

#[cfg(any(target_os = "macos", windows))]
fn locale_command(
    environment: &CommandEnvironment,
    executable: &std::path::Path,
    args: &[&str],
) -> Option<String> {
    crate::desktop::bounded_host_os_command_with_timeout(
        environment,
        executable,
        &args
            .iter()
            .map(std::ffi::OsString::from)
            .collect::<Vec<_>>(),
        std::time::Duration::from_secs(2),
    )
    .ok()
}

fn choose_skill_language(
    environment: &CommandEnvironment,
    selection: Option<&str>,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Option<String>, &'static str> {
    let selected = match selection {
        Some(s) => s.to_owned(),
        None => crate::cli_inventory::choice(
            reader,
            writer,
            &format!(
                "Skill descriptions: 1 Auto ({}), 2 中文, 3 English, 0 cancel [1]: ",
                environment.skill_language()
            ),
        )
        .map_err(|_| "installation-input-failed")?,
    };
    let language = match selected.as_str() {
        "0" => return Ok(None),
        "" | "1" | "auto" => environment.skill_language(),
        "2" | "zh" => "zh",
        "3" | "en" => "en",
        _ => return Err("skill-language-invalid"),
    };
    line(
        writer,
        &format!("Skill description language: {language}. Invocation names stay the same.\n"),
    )?;
    Ok(Some(language.into()))
}

pub(crate) fn prepare_plan(
    command: &ManagedOperationCliCommand,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<String, &'static str> {
    let result = crate::managed_operation::execute(command, environment, content);
    if let (
        ManagedOperationCliCommand::PlanPluginSource {
            target,
            destination,
            context_hooks,
            language,
            ..
        },
        Err("plugin-source-already-exists"),
    ) = (command, &result)
    {
        return crate::managed_operation::execute(
            &ManagedOperationCliCommand::PlanPluginSource {
                action: crate::plugin_source::PluginSourceAction::Update,
                target: *target,
                destination: destination.clone(),
                context_hooks: *context_hooks,
                language: language.clone(),
            },
            environment,
            content,
        );
    }
    result
}

pub struct BundledContentReview {
    pub(crate) plan_json: String,
}

#[derive(Default, Debug, Eq, PartialEq)]
pub struct InstallationGuide {
    pub(crate) plugin: bool,
    pub(crate) targets: Vec<PluginInstallHost>,
    pub(crate) destination: Option<std::path::PathBuf>,
    pub(crate) context_hooks: Option<bool>,
    pub(crate) language: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PluginInstallHost {
    Managed(crate::managed_operation::ManagedIntegrationTargetV1),
    DeepSeek,
}

use crate::managed_operation::ManagedIntegrationTargetV1 as ManagedHost;

// Keep 3 as the existing Codex/Claude shortcut; new Hosts get their own entry.
const PLUGIN_HOSTS: &[(&str, &str, &str, PluginInstallHost)] = &[
    (
        "1",
        "codex",
        "Codex",
        PluginInstallHost::Managed(ManagedHost::Codex),
    ),
    (
        "2",
        "claude",
        "Claude Code",
        PluginInstallHost::Managed(ManagedHost::ClaudeCode),
    ),
    (
        "4",
        "deepseek",
        "DeepSeek Harness",
        PluginInstallHost::DeepSeek,
    ),
];

pub(crate) fn plugin_hosts(selection: &str) -> Result<Vec<PluginInstallHost>, &'static str> {
    let selection = selection.trim();
    if selection == "all" {
        return Ok(PLUGIN_HOSTS.iter().map(|entry| entry.3).collect());
    }
    let mut hosts = Vec::new();
    for selected in selection
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
    {
        let entries = if selected == "3" || selected == "both" {
            &PLUGIN_HOSTS[..2]
        } else {
            std::slice::from_ref(
                PLUGIN_HOSTS
                    .iter()
                    .find(|entry| selected == entry.0 || selected == entry.1)
                    .ok_or("installation-selection-invalid")?,
            )
        };
        for entry in entries {
            if !hosts.contains(&entry.3) {
                hosts.push(entry.3);
            }
        }
    }
    if hosts.is_empty() {
        return Err("installation-selection-invalid");
    }
    Ok(hosts)
}

pub(crate) fn validate_host_options(
    targets: &[PluginInstallHost],
    destination: Option<&std::path::Path>,
    context_hooks: Option<bool>,
) -> Result<(), &'static str> {
    if (targets.len() > 1 && destination.is_some())
        || (targets.contains(&PluginInstallHost::DeepSeek)
            && (destination.is_some() || context_hooks.is_some()))
    {
        return Err("installation-host-options-invalid");
    }
    Ok(())
}

pub fn guide_installation(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> CliOutput {
    InstallationGuide::default().run(environment, content)
}

impl InstallationGuide {
    pub fn run(self, environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return CliOutput::usage_text(
                "installation requires a terminal; use --dry-run with an explicit target/destination for scripts",
            );
        }
        let reader = &mut io::stdin().lock();
        let writer = &mut io::stdout().lock();
        let result = if self.plugin {
            install_plugins(
                environment,
                content,
                self.targets,
                self.destination,
                self.context_hooks,
                self.language,
                reader,
                writer,
            )
        } else {
            guide(environment, content, reader, writer)
        };
        match result {
            Ok(()) => CliOutput::success_text(""),
            Err(code) => installation_failure(code),
        }
    }
}

fn guide(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<(), &'static str> {
    use crate::managed_operation::ManagedSkillsPresetV1;
    let inventory = crate::cli_inventory::discover(environment);
    line(
        writer,
        &format!(
            "Qiongli {} — connect your research tools\nVisible CLI installations: {}. The running CLI supplies this installation.\n\n1. Plugin (recommended): Skills + native program + Full MCP (33 tools).\n2. Skills files only: export guidance; no MCP or automatic Host registration.\n3. MCP connection only: show configuration for your existing Host.\n4. Review CLI versions and manual cleanup guidance.\n0. Cancel.\n",
            env!("CARGO_PKG_VERSION"),
            inventory.installations.len()
        ),
    )?;
    let select = crate::cli_inventory::choice(reader, writer, "Choose [1]: ")
        .map_err(|_| "installation-input-failed")?;
    let command = match select.as_str() {
        "0" => return line(writer, "Cancelled; no changes made.\n"),
        "4" => {
            return crate::cli_inventory::review(&inventory, reader, writer)
                .map_err(|_| "installation-review-failed");
        }
        "3" => {
            let executable =
                std::env::current_exe().map_err(|_| "plugin-source-executable-unavailable")?;
            let config = serde_json::json!({"mcpServers":{(crate::plugin_source::plugin_name()):{
                "command":executable,"args":["mcp","serve","--profile","full","--transport","stdio"]}}});
            line(
                writer,
                &serde_json::to_string_pretty(&config)
                    .map_err(|_| "installation-preview-invalid")?,
            )?;
            return line(
                writer,
                "\nNo settings changed. Add this command through your Host's MCP configuration.\nThe Host starts the bundled MCP implementation; no separate server package or background terminal is needed.\nCheck this CLI with qiongli mcp check. Then open a new Host session, list Qiongli tools and call qiongli_config_status.\n",
            );
        }
        "2" => ManagedOperationCliCommand::PlanSkillsReconcile {
            language: choose_skill_language(environment, None, reader, writer)?,
            preset: ManagedSkillsPresetV1::QiongliManaged,
            profile: qiongli_content::ProfileId::Full,
        },
        "" | "1" => {
            return install_plugins(
                environment,
                content,
                Vec::new(),
                None,
                None,
                None,
                reader,
                writer,
            );
        }
        _ => return Err("installation-selection-invalid"),
    };
    if matches!(
        &command,
        ManagedOperationCliCommand::PlanSkillsReconcile { language: None, .. }
    ) {
        return line(writer, "Cancelled; no changes made.\n");
    }
    BundledContentReview {
        plan_json: prepare_plan(&command, environment, content)?,
    }
    .review(environment, content, reader, writer)
    .map(|_| ())
}

#[allow(clippy::too_many_arguments)] // Shared terminal installer options, including the selected metadata language.
fn install_plugins(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    mut targets: Vec<PluginInstallHost>,
    destination: Option<std::path::PathBuf>,
    context_hooks: Option<bool>,
    language: Option<String>,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<(), &'static str> {
    if targets.is_empty() {
        let default = if environment.client_executable("codex").is_none()
            && environment.client_executable("claude").is_some()
        {
            "2"
        } else {
            "1"
        };
        let entries = PLUGIN_HOSTS
            .iter()
            .map(|entry| format!("{} {}", entry.0, entry.2))
            .collect::<Vec<_>>()
            .join(", ");
        let selection = crate::cli_inventory::choice(
            reader,
            writer,
            &format!(
                "Hosts (comma/space separated): {entries}, 3 Codex+Claude, all, 0 cancel [{default}]: "
            ),
        )
        .map_err(|_| "installation-input-failed")?;
        if selection == "0" {
            return line(writer, "Cancelled; no changes made.\n");
        }
        targets = plugin_hosts(if selection.is_empty() {
            default
        } else {
            &selection
        })?;
    }
    validate_host_options(&targets, destination.as_deref(), context_hooks)?;
    line(
        writer,
        "Each selected Host has its own preview and confirmation. Cancellation or failure stops here; completed Host steps remain installed.\n",
    )?;
    let Some(language) = choose_skill_language(environment, language.as_deref(), reader, writer)?
    else {
        return line(writer, "Cancelled; no changes made.\n");
    };
    let multiple = targets.len() > 1;
    for selected in targets {
        let host = PLUGIN_HOSTS
            .iter()
            .find(|entry| entry.3 == selected)
            .ok_or("installation-selection-invalid")?
            .2;
        line(writer, &format!("\n{host} Plugin — install or update\n"))?;
        let target = match selected {
            PluginInstallHost::DeepSeek => {
                if !crate::plugin_host::deepseek::install(
                    environment,
                    content,
                    &language,
                    reader,
                    writer,
                )? {
                    break;
                }
                continue;
            }
            PluginInstallHost::Managed(target) => target,
        };
        let registered = crate::plugin_host::installation_source(
            environment,
            target,
            destination.as_deref(),
            writer,
        )?;
        let path = if let Some(path) = &destination {
            path.clone()
        } else if let Some(path) = registered {
            line(
                writer,
                "Using the source directory registered with this Host.\n",
            )?;
            path
        } else {
            let default = environment
                .platform_home()
                .ok_or("plugin-source-home-unavailable")?
                .join(crate::plugin_source::plugin_name());
            line(
                writer,
                "Plugin source files stay here. The Host loads its registered cache, including Skills and MCP; no copy to ~/.agents/skills is needed.\n",
            )?;
            let usable_default =
                crate::plugin_source::status(environment, content, target, &default).is_ok();
            if !usable_default {
                line(
                    writer,
                    "The default directory belongs to another Host or has unverified files. Enter a different path ending in qiongli or qiongli-next with an existing parent; Enter cancels.\n",
                )?;
            }
            let selected = crate::cli_inventory::choice(
                reader,
                writer,
                &format!(
                    "Source directory (absolute, existing parent; 0 cancels) [{}]: ",
                    serde_json::to_string(&default).map_err(|_| "installation-preview-invalid")?
                ),
            )
            .map_err(|_| "installation-input-failed")?;
            if selected == "0" || (selected.is_empty() && !usable_default) {
                return line(
                    writer,
                    "Cancelled this installation. Previously completed Host steps remain installed.\n",
                );
            }
            if selected.is_empty() {
                default
            } else {
                selected.into()
            }
        };
        if !path.is_absolute() {
            return Err("plugin-source-destination-invalid");
        }
        let mut command = ManagedOperationCliCommand::PlanPluginSource {
            action: crate::plugin_source::PluginSourceAction::Install,
            target,
            destination: path,
            context_hooks,
            language: Some(language.clone()),
        };
        let mut plan_json = prepare_plan(&command, environment, content)?;
        if context_hooks.is_none() {
            let value: serde_json::Value =
                serde_json::from_str(&plan_json).map_err(|_| "managed-operation-plan-invalid")?;
            let current = value["operation"]["source"]["context_hooks"]
                .as_bool()
                .unwrap_or(false);
            let Some(selected) = choose_context_hooks(current, reader, writer)? else {
                return line(
                    writer,
                    "Cancelled this installation; no files changed for this Host.\n",
                );
            };
            if selected != current {
                if let ManagedOperationCliCommand::PlanPluginSource { context_hooks, .. } =
                    &mut command
                {
                    *context_hooks = Some(selected);
                }
                plan_json = prepare_plan(&command, environment, content)?;
            }
        }
        let review = BundledContentReview { plan_json };
        if !review.review(environment, content, reader, writer)? {
            break;
        }
        if multiple {
            line(
                writer,
                &format!("{host} step finished. Each Host uses its own source and confirmation.\n"),
            )?;
        }
    }
    Ok(())
}

fn choose_context_hooks(
    current: bool,
    reader: &mut impl BufRead,
    writer: &mut impl Write,
) -> Result<Option<bool>, &'static str> {
    line(
        writer,
        "Optional context hooks: remind the model after resume/compaction or child-agent startup. First install defaults to off; updates keep the existing choice.\n",
    )?;
    let default = if current { "1" } else { "2" };
    let selected = crate::cli_inventory::choice(
        reader,
        writer,
        &format!("Hooks: 1 context reminders, 2 off, 0 cancel [{default}]: "),
    )
    .map_err(|_| "installation-input-failed")?;
    match if selected.is_empty() {
        default
    } else {
        &selected
    } {
        "1" => Ok(Some(true)),
        "2" => Ok(Some(false)),
        "0" => Ok(None),
        _ => Err("installation-selection-invalid"),
    }
}

impl BundledContentReview {
    pub fn run(self, environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return CliOutput::usage_text(
                "installation requires a terminal; use --dry-run, then qiongli app apply for scripts",
            );
        }
        match self.review(
            environment,
            content,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
        ) {
            Ok(_) => CliOutput::success_text(""),
            Err(code) => installation_failure(code),
        }
    }

    fn review(
        &self,
        environment: &CommandEnvironment,
        content: &EmbeddedContent,
        reader: &mut impl BufRead,
        writer: &mut impl Write,
    ) -> Result<bool, &'static str> {
        let value: serde_json::Value =
            serde_json::from_str(&self.plan_json).map_err(|_| "managed-operation-plan-invalid")?;
        if value["operation"]["kind"] == "skills-reconcile-preset"
            && value["operation"]["skill_language"].is_null()
        {
            let Some(language) = choose_skill_language(environment, None, reader, writer)? else {
                return Ok(false);
            };
            let command = ManagedOperationCliCommand::PlanSkillsReconcile {
                preset: serde_json::from_value(value["operation"]["preset"].clone())
                    .map_err(|_| "managed-operation-plan-invalid")?,
                profile: serde_json::from_value(value["operation"]["profile"].clone())
                    .map_err(|_| "managed-operation-plan-invalid")?,
                language: Some(language),
            };
            return Self {
                plan_json: prepare_plan(&command, environment, content)?,
            }
            .review(environment, content, reader, writer);
        }
        if value["operation"]["kind"] == "plugin-source" {
            let source: crate::plugin_source::PluginSourcePlan =
                serde_json::from_value(value["operation"]["source"].clone())
                    .map_err(|_| "managed-operation-plan-invalid")?;
            crate::plugin_host::check_context_hook_support(environment, &source)?;
            let previous: serde_json::Value = serde_json::from_str(&crate::plugin_source::status(
                environment,
                content,
                source.target,
                &source.destination,
            )?)
            .map_err(|_| "plugin-source-status-invalid")?;
            show_json(writer, &serde_json::json!({"installation":"Plugin (Skills included)",
                "host":source.target,"destination":source.destination,"cli_version":env!("CARGO_PKG_VERSION"),
                "previous_export_version":previous["source"]["version"],"export_state":previous["state"],
                "skill_language": source.skill_language,
                "context_hooks": if source.context_hooks {"include context reminders; Host trust and execution not verified"} else {"off in this Plugin"},
                "mcp":"Full, 33 tools; started by the Host from the bundled native program"}).to_string())?;
            line(
                writer,
                "This also installs the research Skills. A separate Skills installation or MCP package is unnecessary.\nFile changes and Host registration are confirmed separately below.\n",
            )?;
            if source.context_hooks {
                use qiongli_platform::{ClientKind, OperatingSystem, plugin_context_hooks};
                let host = match source.target {
                    crate::managed_operation::ManagedIntegrationTargetV1::Codex => {
                        ClientKind::Codex
                    }
                    crate::managed_operation::ManagedIntegrationTargetV1::ClaudeCode => {
                        ClientKind::ClaudeCode
                    }
                };
                line(
                    writer,
                    "Hook configuration in the Plugin manifest (uses the bundled binary):\n",
                )?;
                line(
                    writer,
                    &serde_json::to_string_pretty(&plugin_context_hooks(
                        host,
                        OperatingSystem::current().ok_or("plugin-source-platform-unsupported")?,
                    ))
                    .map_err(|_| "installation-preview-invalid")?,
                )?;
                line(
                    writer,
                    "\nOnly context reminders; no project reads, writes or approval decisions. Host/global hook settings are kept. Existing manual Qiongli hooks may produce duplicate reminders.\n",
                )?;
            } else {
                line(
                    writer,
                    "This export will contain no context hooks. Existing Host/global hooks are kept.\n",
                )?;
            }
        } else {
            line(
                writer,
                "Export Skills and research references only. The profile selects content, not a running MCP service.\n",
            )?;
        }
        if let Some(preset) = value["operation"].get("preset") {
            let preset = serde_json::from_value(preset.clone())
                .map_err(|_| "managed-operation-plan-invalid")?;
            let destination = crate::managed_operation::skills_preset_path(environment, preset)?;
            show_json(
                writer,
                &serde_json::json!({"destination": destination}).to_string(),
            )?;
        }
        show_json(writer, &self.plan_json)?;
        if !confirm(
            reader,
            writer,
            "Apply these bundled-content file changes? [y/N] ",
        )? {
            line(writer, "Cancelled; no changes made.\n")?;
            return Ok(false);
        }
        let result =
            crate::managed_operation::apply_reviewed_plan(environment, content, &self.plan_json)?;
        show_json(writer, &result)?;
        line(writer, "Files: exported and receipt verified.\n")?;
        if value["operation"]["kind"] == "plugin-source" {
            let source: crate::plugin_source::PluginSourcePlan =
                serde_json::from_value(value["operation"]["source"].clone())
                    .map_err(|_| "managed-operation-plan-invalid")?;
            match crate::plugin_host::register(environment, content, &source, reader, writer) {
                Ok(true) => {}
                Ok(false) => return Ok(false),
                Err(code) => {
                    line(
                        writer,
                        "Host registration did not finish. Exported files remain available. Resolve the reported Host error, then rerun qiongli install plugin; other Plugins are kept.\n",
                    )?;
                    return Err(code);
                }
            }
        } else {
            line(
                writer,
                "Skills: exported to .qiongli-skills under the selected home/project.\nHost registration: not performed. MCP: not installed by this export.\nUse install plugin to load the workflow and Full MCP together in a Host.\n",
            )?;
        }
        Ok(true)
    }
}

fn installation_failure(code: &'static str) -> CliOutput {
    let hint = match code {
        "plugin-source-destination-invalid" => {
            "Choose an absolute directory ending in qiongli or qiongli-next with an existing parent, or omit --destination to reuse the registered source."
        }
        "plugin-source-destination-reserved" => {
            "Keep Plugin source files outside ~/.agents, Host configuration/cache directories and Qiongli's private state. The Host discovers Skills through Plugin registration."
        }

        "local-host-other-qiongli-enabled" => {
            "Disable the other Qiongli Plugin in the Host, then retry this command."
        }
        "local-host-migration-config-unavailable" | "local-host-migration-not-verified" => {
            "Check the listed Qiongli Plugins in Codex before retrying. Migration needs a supported Codex configuration API and user-scoped enabled entries. Update Codex or disable the previous Plugin there; its source and cache are kept."
        }
        "local-host-marketplace-conflict" => {
            "Review the Host's qiongli-cli-local marketplace path; choose the matching export destination."
        }
        "local-host-context-hooks-unsupported" => {
            "Context hooks need Claude Code 2.1.139 or newer. Update Claude Code, or rerun install plugin --hooks off."
        }
        "host-plugin-executable-unavailable" | "local-host-version-unsupported" => {
            "Install or update the selected Host CLI (codex, claude or dsh), then retry this command."
        }
        "deepseek-version-unsupported" => "Update DeepSeek Harness to 0.2 or newer, then retry.",
        "deepseek-desktop-profile-unavailable" => {
            "Open DeepSeek Desktop once to initialize its profile, or choose a CLI profile."
        }
        "deepseek-plugin-conflict" => {
            "Review the old dsh-qiongli-* bundle in this profile and remove it through the official DSH manager before installing qiongli."
        }
        "installation-host-options-invalid" => {
            "Use separate Codex/Claude selections for --hooks/--destination. DeepSeek uses its own npm profile."
        }
        "local-host-plugin-scope-conflict" => {
            "Review duplicate or project-scoped Qiongli Plugins in the Host; this command uses user scope."
        }
        _ => {
            "Review the reported step and retained files before retrying. Use --help for the installation workflow."
        }
    };
    CliOutput::operation_failure(code).with_stderr(format!("error: {code}\n{hint}\n"))
}

pub(crate) fn show_json(writer: &mut impl Write, json: &str) -> Result<(), &'static str> {
    let value = serde_json::from_str(json).map_err(|_| "installation-preview-invalid")?;
    let mut text = String::new();
    crate::cli_presentation::fields(&mut text, &value, 0);
    line(writer, &text)
}

pub(crate) fn line(writer: &mut impl Write, text: &str) -> Result<(), &'static str> {
    writer
        .write_all(text.as_bytes())
        .map_err(|_| "installation-output-failed")
}

pub(crate) fn confirm(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    prompt: &str,
) -> Result<bool, &'static str> {
    line(writer, prompt)?;
    writer.flush().map_err(|_| "installation-output-failed")?;
    let mut response = String::new();
    reader
        .take(128)
        .read_line(&mut response)
        .map_err(|_| "installation-input-failed")?;
    Ok(response.ends_with('\n') && matches!(response.trim(), "y" | "Y" | "yes" | "YES"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_language_selection_supports_auto_explicit_and_cancel() {
        let environment = CommandEnvironment::with_paths(None, None, None);
        for (input, expected) in [
            ("\n", Some("en")),
            ("2\n", Some("zh")),
            ("3\n", Some("en")),
            ("0\n", None),
        ] {
            assert_eq!(
                choose_skill_language(&environment, None, &mut input.as_bytes(), &mut Vec::new())
                    .unwrap()
                    .as_deref(),
                expected
            );
        }
        assert_eq!(
            choose_skill_language(
                &environment,
                Some("zh"),
                &mut "".as_bytes(),
                &mut Vec::new()
            )
            .unwrap()
            .as_deref(),
            Some("zh")
        );
        assert!(
            choose_skill_language(
                &environment,
                Some("fr"),
                &mut "".as_bytes(),
                &mut Vec::new()
            )
            .is_err()
        );
    }
    use crate::managed_operation::ManagedSkillsPresetV1;
    use qiongli_content::ProfileId;

    #[test]
    fn plugin_selection_supports_multiple_hosts_without_duplicate_steps() {
        let all = PLUGIN_HOSTS.iter().map(|entry| entry.3).collect::<Vec<_>>();
        for selection in [
            "1,2,4",
            "1 2 4",
            "codex,claude,deepseek",
            "3,4",
            "both 4",
            "all",
            "1,1,2,4,4",
        ] {
            assert_eq!(plugin_hosts(selection).unwrap(), all);
        }
        assert_eq!(plugin_hosts("4 1").unwrap(), vec![all[2], all[0]]);
        for selection in ["", "0", "1,unknown", "1 all", "5"] {
            assert!(plugin_hosts(selection).is_err());
        }
        assert!(validate_host_options(&all, None, None).is_ok());
        assert!(validate_host_options(&all, None, Some(false)).is_err());
        assert!(validate_host_options(&all, Some(std::path::Path::new("/source")), None).is_err());
    }

    #[test]
    fn context_hook_choice_preserves_defaults_and_requires_complete_input() {
        for (current, input, expected) in [
            (false, "\n", Some(false)),
            (true, "\n", Some(true)),
            (false, "1\n", Some(true)),
            (true, "2\n", Some(false)),
            (true, "0\n", None),
        ] {
            assert_eq!(
                choose_context_hooks(current, &mut input.as_bytes(), &mut Vec::new()).unwrap(),
                expected
            );
        }
        for input in ["", "1", "yes\n", "3\n"] {
            assert!(choose_context_hooks(false, &mut input.as_bytes(), &mut Vec::new()).is_err());
        }
    }

    #[test]
    fn content_review_requires_confirmation_and_revalidates_the_original_plan() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/cli-content-review")
            .canonicalize()
            .unwrap_or_else(|_| {
                let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../target/cli-content-review");
                std::fs::create_dir_all(&path).unwrap();
                path.canonicalize().unwrap()
            })
            .join(std::process::id().to_string());
        #[cfg(windows)]
        qiongli_windows_security::create_owner_only_directory(&root).unwrap();
        #[cfg(not(windows))]
        std::fs::create_dir(&root).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(root.clone()), None);
        let content = crate::embedded_content().unwrap();
        for response in ["0\n", "2\n\nn\n", "3\n", "\n0\n"] {
            let mut output = Vec::new();
            guide(
                &environment,
                &content,
                &mut response.as_bytes(),
                &mut output,
            )
            .unwrap();
            assert!(!root.join(".qiongli-skills").exists());
            assert!(!root.join("qiongli-next").exists());
        }
        for response in ["", "wrong\n", "1\nwrong\n"] {
            assert!(
                guide(
                    &environment,
                    &content,
                    &mut response.as_bytes(),
                    &mut Vec::new()
                )
                .is_err()
            );
        }
        let command = ManagedOperationCliCommand::PlanSkillsReconcile {
            language: Some("en".into()),
            preset: ManagedSkillsPresetV1::QiongliManaged,
            profile: ProfileId::Full,
        };
        let review = BundledContentReview {
            plan_json: prepare_plan(&command, &environment, &content).unwrap(),
        };
        let mut output = Vec::new();
        for answer in ["\n", "n\n", "yes", "", "not-approval\n"] {
            review
                .review(&environment, &content, &mut answer.as_bytes(), &mut output)
                .unwrap();
            assert!(!root.join(".qiongli-skills").exists());
        }
        review
            .review(&environment, &content, &mut "yes\n".as_bytes(), &mut output)
            .unwrap();
        assert!(root.join(".qiongli-skills").is_dir());
        assert!(
            review
                .review(&environment, &content, &mut "yes\n".as_bytes(), &mut output)
                .is_err()
        );
        let refreshed = BundledContentReview {
            plan_json: prepare_plan(&command, &environment, &content).unwrap(),
        };
        refreshed
            .review(&environment, &content, &mut "y\n".as_bytes(), &mut output)
            .unwrap();
        let unexpected = root.join(".qiongli-skills/user-notes.txt");
        std::fs::write(&unexpected, "retain user content").unwrap();
        assert!(prepare_plan(&command, &environment, &content).is_err());
        assert_eq!(
            std::fs::read_to_string(&unexpected).unwrap(),
            "retain user content"
        );
        let displayed = String::from_utf8(output).unwrap();
        assert!(displayed.contains("Plan digest sha256"));
        assert!(displayed.contains(".qiongli-skills"));
        for target in [
            crate::managed_operation::ManagedIntegrationTargetV1::Codex,
            crate::managed_operation::ManagedIntegrationTargetV1::ClaudeCode,
        ] {
            let command = ManagedOperationCliCommand::PlanPluginSource {
                language: None,
                action: crate::plugin_source::PluginSourceAction::Install,
                target,
                destination: root.join("qiongli-next"),
                context_hooks: Some(true),
            };
            let review = BundledContentReview {
                plan_json: prepare_plan(&command, &environment, &content).unwrap(),
            };
            let mut output = Vec::new();
            let result = review.review(&environment, &content, &mut "n\n".as_bytes(), &mut output);
            if target == crate::managed_operation::ManagedIntegrationTargetV1::Codex {
                assert!(!result.unwrap());
                let text = String::from_utf8(output).unwrap();
                assert!(
                    text.find("SessionStart").unwrap()
                        < text
                            .find("Apply these bundled-content file changes?")
                            .unwrap()
                );
                assert!(text.contains("hooks context"));
            } else {
                assert_eq!(result.unwrap_err(), "local-host-context-hooks-unsupported");
                assert!(output.is_empty());
            }
            assert!(!root.join("qiongli-next").exists());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
