//! Interactive presentation over the existing managed content write owner.
use std::io::{self, BufRead, IsTerminal, Read, Write};

use qiongli_content::EmbeddedContent;

use crate::install_output::{DisplayOptions, InstallOutput, InstallWriter};
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
    pub(crate) all_detected: bool,
    pub(crate) targets: Vec<PluginInstallHost>,
    pub(crate) destination: Option<std::path::PathBuf>,
    pub(crate) context_hooks: Option<bool>,
    pub(crate) language: Option<String>,
    pub(crate) display: DisplayOptions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PluginInstallHost {
    Managed(crate::managed_operation::ManagedIntegrationTargetV1),
    DeepSeek,
    Antigravity,
    Pi,
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
    (
        "5",
        "antigravity",
        "Antigravity",
        PluginInstallHost::Antigravity,
    ),
    ("7", "pi", "Pi", PluginInstallHost::Pi),
];

pub(crate) fn all_plugin_hosts_selection(selection: &str) -> bool {
    matches!(selection.trim(), "all" | "6")
}

pub(crate) fn plugin_hosts(selection: &str) -> Result<Vec<PluginInstallHost>, &'static str> {
    let selection = selection.trim();
    if all_plugin_hosts_selection(selection) {
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
                    .find(|entry| {
                        selected == entry.0
                            || selected == entry.1
                            || (selected == "agy" && entry.1 == "antigravity")
                    })
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

fn installed_plugin_hosts(
    mut detected: impl FnMut(&str) -> bool,
    writer: &mut impl Write,
) -> Result<Vec<PluginInstallHost>, &'static str> {
    let mut hosts = Vec::new();
    let mut names = Vec::new();
    for (_, _, name, host) in PLUGIN_HOSTS {
        let executable = match host {
            PluginInstallHost::Managed(ManagedHost::Codex) => "codex",
            PluginInstallHost::Managed(ManagedHost::ClaudeCode) => "claude",
            PluginInstallHost::DeepSeek => "dsh",
            PluginInstallHost::Antigravity => "agy",
            PluginInstallHost::Pi => "pi",
        };
        if detected(executable) {
            hosts.push(*host);
            names.push(*name);
        } else {
            line(
                writer,
                &format!("Skipping {name}: {executable} CLI not found.\n"),
            )?;
        }
    }
    if hosts.is_empty() {
        return Err("installation-no-hosts-detected");
    }
    line(
        writer,
        &format!(
            "Install all detected Hosts: {}. Each Plugin includes Skills and Full MCP.\n",
            names.join(", ")
        ),
    )?;
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
        || (targets
            .iter()
            .any(|host| matches!(host, PluginInstallHost::Antigravity | PluginInstallHost::Pi))
            && context_hooks.is_some())
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
                "installation requires a terminal; scripted --dry-run is available only for Codex/Claude source exports and standalone Skills",
            );
        }
        let reader = &mut io::stdin().lock();
        let writer = &mut InstallOutput::terminal(io::stdout().lock(), self.display);
        let result = if self.plugin {
            install_plugins(environment, content, self, reader, writer)
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
    writer: &mut impl InstallWriter,
) -> Result<(), &'static str> {
    use crate::managed_operation::ManagedSkillsPresetV1;
    let inventory = crate::cli_inventory::discover(environment);
    line(
        writer,
        &format!(
            "Qiongli {} — connect your research tools\nVisible CLI installations: {}. The running CLI supplies this installation.\n\n1. Plugin: choose specific Hosts; Skills + native program + Full MCP.\n2. Skills files only: export guidance; no MCP or automatic Host registration.\n3. MCP connection only: show configuration for your existing Host.\n4. Review CLI versions and manual cleanup guidance.\n5. Install all detected Hosts (recommended): each Plugin includes Skills + Full MCP.\n0. Cancel.\n",
            env!("CARGO_PKG_VERSION"),
            inventory.installations.len()
        ),
    )?;
    let select = crate::cli_inventory::choice(reader, writer, "Choose [5 — all detected Hosts]: ")
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
        "" | "1" | "5" | "all" => {
            return install_plugins(
                environment,
                content,
                InstallationGuide {
                    plugin: true,
                    all_detected: matches!(select.as_str(), "" | "5" | "all"),
                    ..Default::default()
                },
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

fn install_plugins(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    options: InstallationGuide,
    reader: &mut impl BufRead,
    writer: &mut impl InstallWriter,
) -> Result<(), &'static str> {
    let InstallationGuide {
        mut targets,
        mut all_detected,
        destination,
        context_hooks,
        language,
        ..
    } = options;
    if targets.is_empty() && !all_detected {
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
                "Hosts (comma/space separated): {entries}, 3 Codex+Claude, 6 All detected Hosts (all), 0 cancel [{default}]: "
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
        all_detected = all_plugin_hosts_selection(&selection);
    }
    if all_detected {
        // Validate the all-Host preset before discovery: finding only one Host
        // does not make shared destinations or Host-specific flags meaningful.
        validate_host_options(&plugin_hosts("all")?, destination.as_deref(), context_hooks)?;
        targets =
            installed_plugin_hosts(|name| environment.client_executable(name).is_some(), writer)?;
    }
    validate_host_options(&targets, destination.as_deref(), context_hooks)?;
    if writer.verbose() {
        line(
            writer,
            "\nQiongli Plugin installation\nEach Host asks for approval. Failed/skipped Hosts are reported at the end; later Hosts continue.\n\n",
        )?;
    }
    // A DSH-only install can reuse the selected profile's saved preference.
    // Explicit --language still overrides it; other Hosts keep their guide.
    let language = if targets == [PluginInstallHost::DeepSeek] && language.is_none() {
        None
    } else {
        let Some(language) =
            choose_skill_language(environment, language.as_deref(), reader, writer)?
        else {
            return line(writer, "Cancelled; no changes made.\n");
        };
        Some(language)
    };
    let options = InstallationGuide {
        plugin: true,
        all_detected,
        destination,
        context_hooks,
        language,
        ..Default::default()
    };
    install_batch(&targets, &options, writer, |host, writer| {
        install_one_plugin(environment, content, host, &options, reader, writer)
    })
}

fn install_one_plugin(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    selected: PluginInstallHost,
    options: &InstallationGuide,
    reader: &mut impl BufRead,
    writer: &mut impl InstallWriter,
) -> Result<bool, &'static str> {
    let destination = &options.destination;
    let target = match selected {
        PluginInstallHost::Pi => {
            return crate::plugin_host::pi::install(
                environment,
                content,
                destination.as_deref(),
                options
                    .language
                    .as_deref()
                    .ok_or("skill-language-invalid")?,
                reader,
                writer,
            );
        }
        PluginInstallHost::Antigravity => {
            return crate::plugin_host::antigravity::install(
                environment,
                content,
                destination.as_deref(),
                options
                    .language
                    .as_deref()
                    .ok_or("skill-language-invalid")?,
                reader,
                writer,
            );
        }
        PluginInstallHost::DeepSeek => {
            return crate::plugin_host::deepseek::install(
                environment,
                content,
                options.language.as_deref(),
                reader,
                writer,
            );
        }
        PluginInstallHost::Managed(target) => target,
    };
    let language = options.language.as_ref().ok_or("skill-language-invalid")?;
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
        let default = automatic_source_directory(environment, content, target)?;
        line(
            writer,
            "Plugin source files stay here. The Host loads its registered cache, including Skills and MCP; no copy to ~/.agents/skills is needed.\n",
        )?;
        show_json(
            writer,
            &serde_json::json!({"selected_source": default}).to_string(),
        )?;
        default
    };
    if !path.is_absolute() {
        return Err("plugin-source-destination-invalid");
    }
    let mut command = ManagedOperationCliCommand::PlanPluginSource {
        action: crate::plugin_source::PluginSourceAction::Install,
        target,
        destination: path,
        context_hooks: options.context_hooks,
        language: Some(language.clone()),
    };
    let mut plan_json = prepare_plan(&command, environment, content)?;
    if options.context_hooks.is_none() && !options.all_detected {
        let value: serde_json::Value =
            serde_json::from_str(&plan_json).map_err(|_| "managed-operation-plan-invalid")?;
        let current = value["operation"]["source"]["context_hooks"]
            .as_bool()
            .unwrap_or(false);
        let Some(selected) = choose_context_hooks(current, reader, writer)? else {
            return line(writer, "Skipped this Host; no source changes made.\n").map(|_| false);
        };
        if selected != current {
            if let ManagedOperationCliCommand::PlanPluginSource { context_hooks, .. } = &mut command
            {
                *context_hooks = Some(selected);
            }
            plan_json = prepare_plan(&command, environment, content)?;
        }
    }
    BundledContentReview { plan_json }.review(environment, content, reader, writer)
}

fn install_batch<W: InstallWriter>(
    targets: &[PluginInstallHost],
    options: &InstallationGuide,
    writer: &mut W,
    mut install: impl FnMut(PluginInstallHost, &mut W) -> Result<bool, &'static str>,
) -> Result<(), &'static str> {
    let mut results = Vec::new();
    for (index, host) in targets.iter().enumerate() {
        let name = plugin_host_label(*host);
        line(
            writer,
            &format!(
                "\n[{}/{}] {name} — install or update\n",
                index + 1,
                targets.len()
            ),
        )?;
        writer.flush().map_err(|_| "installation-output-failed")?;
        let result = install(*host, writer);
        let state = match result {
            Ok(true) => "installed",
            Ok(false) => "skipped",
            Err(_) => "failed",
        };
        if writer.verbose() {
            line(
                writer,
                &format!("[{}/{}] {name}: {state}\n", index + 1, targets.len()),
            )?;
        }
        let interrupted = matches!(
            result,
            Err("installation-input-failed" | "installation-output-failed")
        );
        results.push((*host, result));
        if interrupted {
            break;
        }
    }
    line(
        writer,
        "\nInstallation summary (registration only; session tools not checked):\n",
    )?;
    for (host, result) in &results {
        let name = plugin_host_label(*host);
        match result {
            Ok(true) => line(writer, &format!("  OK      {name}\n"))?,
            Ok(false) => line(
                writer,
                &format!(
                    "  SKIPPED {name}: not approved/completed; any exported files are retained.\n"
                ),
            )?,
            Err(code) => line(
                writer,
                &format!(
                    "  FAILED  {name}: {code}\n    {}\n",
                    installation_hint(code)
                ),
            )?,
        }
        if !matches!(result, Ok(true)) {
            line(
                writer,
                &format!("    Retry: {}\n", plugin_retry_command(*host, options)),
            )?;
        }
    }
    for host in &targets[results.len()..] {
        line(
            writer,
            &format!(
                "  NOT RUN {}: input/output interrupted.\n    Retry: {}\n",
                plugin_host_label(*host),
                plugin_retry_command(*host, options)
            ),
        )?;
    }
    let installed = results
        .iter()
        .filter(|(_, r)| matches!(r, Ok(true)))
        .count();
    let failed = results.iter().filter(|(_, r)| r.is_err()).count();
    let skipped = results.len() - installed - failed;
    line(
        writer,
        &format!(
            "Processed {}/{} Hosts: {installed} installed, {failed} failed, {skipped} skipped, {} not run.\n",
            results.len(),
            targets.len(),
            targets.len() - results.len()
        ),
    )?;
    if failed > 0 {
        if targets.len() == 1 {
            return results[0].1.map(|_| ());
        }
        Err("installation-batch-incomplete")
    } else {
        Ok(())
    }
}

fn plugin_host_label(host: PluginInstallHost) -> &'static str {
    PLUGIN_HOSTS
        .iter()
        .find(|entry| entry.3 == host)
        .expect("known Plugin Host")
        .2
}

fn plugin_retry_command(host: PluginInstallHost, options: &InstallationGuide) -> String {
    let target = PLUGIN_HOSTS
        .iter()
        .find(|entry| entry.3 == host)
        .expect("known Plugin Host")
        .1;
    let mut args = vec![
        "install".to_owned(),
        "plugin".to_owned(),
        "--target".to_owned(),
        target.to_owned(),
    ];
    if let Some(path) = &options.destination {
        args.extend([
            "--destination".to_owned(),
            path.to_string_lossy().into_owned(),
        ]);
    }
    if let Some(language) = &options.language {
        args.extend(["--language".to_owned(), language.clone()]);
    }
    if let Some(hooks) = options.context_hooks {
        args.extend([
            "--hooks".to_owned(),
            if hooks { "context" } else { "off" }.to_owned(),
        ]);
    }
    crate::plugin_host::installation_command::display_command("qiongli", &args)
}

fn automatic_source_directory(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    target: ManagedHost,
) -> Result<std::path::PathBuf, &'static str> {
    let home = environment
        .platform_home()
        .ok_or("plugin-source-home-unavailable")?;
    // Reuse a verified legacy export for this Host, including an export whose
    // registration was cancelled. Unrelated home/qiongli files are never adopted.
    let legacy = home.join(crate::plugin_source::plugin_name());
    if let Ok(status) = crate::plugin_source::status(environment, content, target, &legacy) {
        let status: serde_json::Value =
            serde_json::from_str(&status).map_err(|_| "installation-preview-invalid")?;
        if status["source"].is_object() {
            return Ok(legacy);
        }
    }
    let path = home.join(crate::plugin_source::default_directory_name(target));
    // Unknown, changed, linked or unsafe default destinations refuse rather
    // than silently choosing yet another directory and multiplying installs.
    crate::plugin_source::status(environment, content, target, &path)?;
    Ok(path)
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
        &format!("Hooks: 1 context reminders, 2 off, 0 skip this Host [{default}]: "),
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
            &mut InstallOutput::terminal(io::stdout().lock(), DisplayOptions::default()),
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
        writer: &mut impl InstallWriter,
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
                "mcp":"Full, 35 tools; started by the Host from the bundled native program"}).to_string())?;
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
        line(writer, "\nFile changes to approve\n")?;
        show_json(writer, &self.plan_json)?;
        if !confirm(
            reader,
            writer,
            "Apply these bundled-content file changes? [y/N] ",
        )? {
            line(writer, "Cancelled; no changes made.\n")?;
            return Ok(false);
        }
        line(
            writer,
            "Exporting source files and verifying their receipt…\n",
        )?;
        writer.flush().map_err(|_| "installation-output-failed")?;
        let result =
            crate::managed_operation::apply_reviewed_plan(environment, content, &self.plan_json)?;
        if writer.verbose() {
            show_json(writer, &result)?;
        }
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

fn installation_hint(code: &str) -> &'static str {
    match code {
        "installation-batch-incomplete" => {
            "Some Hosts did not finish. Review the per-Host results and run only their retry commands."
        }
        "installation-input-failed" => {
            "Input closed or incomplete. Start a terminal and retry the unfinished Host."
        }
        "host-command-nonzero-exit" => {
            "The official manager failed. Review its exit code, recognized error and command above; partial files may remain."
        }
        "host-command-timeout" => {
            "The official manager exceeded its time limit. Review retained files and manager state before retrying."
        }
        "installation-no-hosts-detected" => {
            "No supported Host CLI was found. Install codex, claude, dsh, agy or pi, or make its CLI available on PATH, then rerun qiongli install all. No Plugins were installed."
        }
        "plugin-source-destination-invalid" => {
            "Use an absolute directory with an existing parent and a supported Qiongli name, or omit --destination for automatic source selection."
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
            "Install or update the selected Host CLI (codex, claude, dsh, agy or pi), then retry this command."
        }
        "pi-version-unsupported" => {
            "Update Pi coding agent to 0.99.0 or newer with built-in MCP enabled, then retry."
        }
        "pi-package-conflict" => {
            "Another Qiongli package/source is configured in Pi. Review pi list and pi remove <source> before retrying; no conflicting package was replaced."
        }
        "pi-package-filtered" => {
            "Pi has filtered or disabled this package's resources or built-in MCP. Review pi config and enable its Skills, extension and built-in MCP, then retry."
        }
        "pi-mcp-conflict" => {
            "Pi already has a standalone Qiongli MCP entry which overrides the package. Review it before retrying; no existing server was changed."
        }
        "pi-profile-invalid" | "pi-profile-unsafe" => {
            "Review Pi's settings.json, mcp.json and PI_CODING_AGENT_DIR. Use a safe absolute user configuration path and bounded JSON objects; existing linked or group/world-writable profiles must be reviewed before retrying."
        }
        "antigravity-version-unsupported" => {
            "Update Antigravity CLI to 1.2.17 or newer, then retry."
        }
        "antigravity-plugin-conflict" => {
            "The AGY cache contains another enabled Qiongli Plugin or unverified files. Review it with agy plugin before retrying."
        }
        "antigravity-mcp-conflict" => {
            "An existing standalone Qiongli MCP entry would duplicate the Plugin. Review it in AGY before retrying."
        }
        "deepseek-version-unsupported" => "Update DeepSeek Harness to 0.2 or newer, then retry.",
        "deepseek-latest-version-unavailable" => {
            "Could not resolve a stable Qiongli release from registry.npmjs.org. Check network access and retry; no cached version was installed."
        }
        "deepseek-install-not-verified" => {
            "DSH returned, but the installed registration/version/content did not verify. Review the failed step and selected profile before retrying; existing files were retained."
        }
        "deepseek-desktop-profile-unavailable" => {
            "Open DeepSeek Desktop once to initialize its profile, or choose a CLI profile."
        }
        "deepseek-plugin-conflict" => {
            "Review the old dsh-qiongli-* bundle in this profile and remove it through the official DSH manager before installing qiongli."
        }
        "installation-host-options-invalid" => {
            "Use a single Host for --destination. Hooks support Codex/Claude only; DeepSeek uses its own npm profile."
        }
        "local-host-plugin-scope-conflict" => {
            "Review duplicate or project-scoped Qiongli Plugins in the Host; this command uses user scope."
        }
        _ => {
            "Review the reported step and retained files before retrying. Use --help for the installation workflow."
        }
    }
}

fn installation_failure(code: &'static str) -> CliOutput {
    let hint = installation_hint(code);
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
    if !response.ends_with('\n') {
        return Err("installation-input-failed");
    }
    Ok(matches!(response.trim(), "y" | "Y" | "yes" | "YES"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_continues_after_failure_and_decline_and_reports_targeted_retries() {
        let hosts = plugin_hosts("deepseek,agy,pi").unwrap();
        let mut visited = Vec::new();
        let mut output = Vec::new();
        let result = install_batch(
            &hosts,
            &InstallationGuide::default(),
            &mut output,
            |host, _| {
                visited.push(host);
                match host {
                    PluginInstallHost::DeepSeek => Err("host-command-nonzero-exit"),
                    PluginInstallHost::Antigravity => Ok(false),
                    _ => Ok(true),
                }
            },
        );
        assert_eq!(result.unwrap_err(), "installation-batch-incomplete");
        assert_eq!(visited, hosts);
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("  OK      Pi\n"));
        assert!(!output.contains("[3/3] Pi: installed"));
        assert!(output.contains("FAILED  DeepSeek Harness"));
        assert!(output.contains("SKIPPED Antigravity"));
        assert!(output.contains("qiongli install plugin --target deepseek"));
        assert!(output.contains("qiongli install plugin --target antigravity"));
        assert!(!output.contains("qiongli install plugin --target pi"));
        assert!(output.contains("1 installed, 1 failed, 1 skipped, 0 not run"));
    }

    #[test]
    fn batch_input_failure_stops_later_hosts_and_never_claims_success() {
        let hosts = plugin_hosts("deepseek,pi").unwrap();
        let mut calls = 0;
        let mut output = Vec::new();
        assert!(
            install_batch(
                &hosts,
                &InstallationGuide::default(),
                &mut output,
                |_, _| {
                    calls += 1;
                    Err("installation-input-failed")
                }
            )
            .is_err()
        );
        assert_eq!(calls, 1);
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("NOT RUN Pi"));
        assert!(output.contains("0 installed, 1 failed, 0 skipped, 1 not run"));
        for input in ["", "yes", "y"] {
            assert_eq!(
                confirm(&mut input.as_bytes(), &mut Vec::new(), "Approve? ").unwrap_err(),
                "installation-input-failed"
            );
        }
    }

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
            "1,2,4,5,7",
            "1 2 4 5 7",
            "codex,claude,deepseek,antigravity,pi",
            "3,4,5,7",
            "both 4 agy pi",
            "all",
            "6",
            "1,1,2,4,4,5,7,pi",
        ] {
            assert_eq!(plugin_hosts(selection).unwrap(), all);
        }
        assert_eq!(plugin_hosts("pi 7").unwrap(), vec![PluginInstallHost::Pi]);
        assert_eq!(plugin_hosts("3").unwrap(), all[..2]);
        assert!(validate_host_options(&[PluginInstallHost::Pi], None, Some(false)).is_err());
        assert_eq!(plugin_hosts("4 1").unwrap(), vec![all[2], all[0]]);
        for selection in ["", "0", "1,unknown", "1 all", "8"] {
            assert!(plugin_hosts(selection).is_err());
        }
        assert!(validate_host_options(&all, None, None).is_ok());
        assert!(validate_host_options(&all, None, Some(false)).is_err());
        assert!(validate_host_options(&all, Some(std::path::Path::new("/source")), None).is_err());
    }

    #[test]
    fn all_detected_selection_preserves_explicit_list_strictness() {
        for selection in ["all", "6", " all ", " 6 "] {
            assert!(all_plugin_hosts_selection(selection));
            assert_eq!(plugin_hosts(selection).unwrap().len(), 5);
        }
        for selection in [
            "1,2,4,5",
            "codex,claude,deepseek,antigravity",
            "both",
            "5",
            "agy",
            "ALL",
            "all,1",
            "6 1",
            "",
        ] {
            assert!(!all_plugin_hosts_selection(selection));
        }
    }

    #[test]
    fn all_detected_hosts_filter_in_order_without_executing_clients() {
        for present in [
            vec!["codex", "agy"],
            vec!["claude", "dsh"],
            vec!["codex", "claude", "dsh", "agy", "pi"],
            vec!["pi"],
            vec![],
        ] {
            let mut checked = Vec::new();
            let mut output = Vec::new();
            let actual = installed_plugin_hosts(
                |name| {
                    checked.push(name.to_owned());
                    present.contains(&name)
                },
                &mut output,
            );
            assert_eq!(checked, ["codex", "claude", "dsh", "agy", "pi"]);
            let expected = PLUGIN_HOSTS
                .iter()
                .zip(&checked)
                .filter(|(_, executable)| present.contains(&executable.as_str()))
                .map(|(entry, _)| entry.3)
                .collect::<Vec<_>>();
            let text = String::from_utf8(output).unwrap();
            for (entry, executable) in PLUGIN_HOSTS.iter().zip(&checked) {
                assert_eq!(
                    text.contains(&format!(
                        "Skipping {}: {executable} CLI not found.",
                        entry.2
                    )),
                    !present.contains(&executable.as_str())
                );
            }
            if expected.is_empty() {
                assert_eq!(actual.unwrap_err(), "installation-no-hosts-detected");
                assert!(!text.contains("Install all detected Hosts:"));
            } else {
                assert_eq!(actual.unwrap(), expected);
                let labels = PLUGIN_HOSTS
                    .iter()
                    .zip(&checked)
                    .filter(|(_, executable)| present.contains(&executable.as_str()))
                    .map(|(entry, _)| entry.2)
                    .collect::<Vec<_>>()
                    .join(", ");
                assert!(text.contains(&format!("Install all detected Hosts: {labels}.")));
                assert!(text.contains("Each Plugin includes Skills and Full MCP."));
            }
        }
    }

    #[test]
    fn automatic_sources_separate_hosts_reuse_verified_legacy_and_refuse_conflicts() {
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/automatic-source-tests");
        std::fs::create_dir_all(&base).unwrap();
        let root = base
            .canonicalize()
            .unwrap()
            .join(std::process::id().to_string());
        #[cfg(windows)]
        qiongli_windows_security::create_owner_only_directory(&root).unwrap();
        #[cfg(not(windows))]
        std::fs::create_dir(&root).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let environment = CommandEnvironment::with_paths(
            Some(root.join("config").into_os_string()),
            Some(root.clone()),
            None,
        );
        let content = crate::embedded_content().unwrap();
        let codex = automatic_source_directory(&environment, &content, ManagedHost::Codex).unwrap();
        let claude =
            automatic_source_directory(&environment, &content, ManagedHost::ClaudeCode).unwrap();
        assert_ne!(codex, claude);
        assert!(!codex.exists() && !claude.exists());
        let legacy = root.join(crate::plugin_source::plugin_name());
        let plan = crate::plugin_source::plan(
            &environment,
            &content,
            crate::plugin_source::PluginSourceAction::Install,
            ManagedHost::Codex,
            &legacy,
            Some(true),
            Some("zh"),
        )
        .unwrap();
        crate::plugin_source::apply(&environment, &content, &plan).unwrap();
        assert_eq!(
            automatic_source_directory(&environment, &content, ManagedHost::Codex).unwrap(),
            legacy
        );
        assert_eq!(
            automatic_source_directory(&environment, &content, ManagedHost::ClaudeCode).unwrap(),
            claude
        );
        let plan = crate::plugin_source::plan(
            &environment,
            &content,
            crate::plugin_source::PluginSourceAction::Install,
            ManagedHost::ClaudeCode,
            &claude,
            None,
            Some("en"),
        )
        .unwrap();
        plan.validate().unwrap();
        std::fs::create_dir(&claude).unwrap();
        std::fs::write(claude.join("canary"), b"keep").unwrap();
        assert!(
            automatic_source_directory(&environment, &content, ManagedHost::ClaudeCode).is_err()
        );
        assert_eq!(std::fs::read(claude.join("canary")).unwrap(), b"keep");
        assert!(
            crate::plugin_source::plan(
                &environment,
                &content,
                crate::plugin_source::PluginSourceAction::Install,
                ManagedHost::Codex,
                &claude,
                None,
                None,
            )
            .is_err()
        );
        std::fs::remove_dir_all(&root).unwrap();
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
        for response in ["0\n", "2\n\nn\n", "3\n", "1\n0\n"] {
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
            let result = review.review(&environment, &content, &mut answer.as_bytes(), &mut output);
            if answer.ends_with('\n') {
                assert!(!result.unwrap());
            } else {
                assert_eq!(result.unwrap_err(), "installation-input-failed");
            }
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
