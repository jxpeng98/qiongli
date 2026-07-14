use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use qiongli_config::{
    ConfigError, ConfigState, GlobalSettingsStore, RedactedConfigStatus, resolve_config_root,
};
use qiongli_content::{
    EmbeddedContent, MaterializationAuthorization, ProfileId, ProfileProjection,
    approve_materialization_target,
};
use qiongli_platform::{
    ARTIFACT_IDENTITY_SCHEMA_VERSION, Architecture, CODEX_ADAPTER_SCHEMA_VERSION,
    CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION, CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
    CodexDiscoverySummaryV1, INSTALL_PLAN_SCHEMA_VERSION, INSTALL_RECEIPT_SCHEMA_VERSION,
    LAUNCH_GRANT_SCHEMA_VERSION, LocalTargetFamily, OperatingSystem, discover_codex_user,
};
use serde::Serialize;

const OUTPUT_SCHEMA_VERSION: u32 = 1;

const USAGE: &str = "Qiongli native platform\n\nUsage:\n  qiongli --version\n  qiongli --help\n  qiongli content list\n  qiongli content materialize --profile <profile> --target <absolute-path>\n  qiongli config show\n  qiongli config set --expected-revision <revision> --default-profile <profile>\n  qiongli install status\n  qiongli install codex status\n  qiongli mcp serve --profile <lite|marketplace-lite> --transport stdio\n  qiongli status\n  qiongli doctor\n\nProfiles:\n  skill-only | marketplace-lite | lite | full\n\nOptions:\n  -h, --help  Print help\n  --version   Print the native product version\n";

const CONTENT_USAGE: &str = "Qiongli embedded content\n\nUsage:\n  qiongli content list\n  qiongli content materialize --profile <profile> --target <absolute-path>\n  qiongli content --help\n";

const CONFIG_USAGE: &str = "Qiongli global config\n\nUsage:\n  qiongli config show\n  qiongli config set --expected-revision <revision> --default-profile <profile>\n  qiongli config --help\n";

const MCP_USAGE: &str = "Qiongli native MCP\n\nUsage:\n  qiongli mcp serve --profile <lite|marketplace-lite> --transport stdio\n  qiongli mcp --help\n";

const INSTALL_USAGE: &str = "Qiongli native installation\n\nUsage:\n  qiongli install status\n  qiongli install codex status\n  qiongli install --help\n";

#[derive(Clone, Default)]
pub struct CommandEnvironment {
    configured_root: Option<OsString>,
    platform_home: Option<PathBuf>,
}

impl CommandEnvironment {
    #[must_use]
    pub fn from_process() -> Self {
        Self {
            configured_root: env::var_os("QIONGLI_CONFIG_HOME"),
            platform_home: process_platform_home(),
        }
    }
}

pub struct CliOutput {
    exit_code: u8,
    stdout: String,
    stderr: String,
}

pub enum ProductAction {
    Output(CliOutput),
    ServeLiteMcpStdio,
}

impl CliOutput {
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }

    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    fn success_text(stdout: impl Into<String>) -> Self {
        Self {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn operation_failure(reason_code: &'static str) -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("error: {reason_code}\n"),
        }
    }

    fn usage_failure(error: UsageError) -> Self {
        Self {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("error: {}\n\n{}", error.message, error.usage),
        }
    }
}

#[must_use]
pub fn failed_embedded_content_output() -> CliOutput {
    CliOutput::operation_failure("embedded-content-integrity-failed")
}

pub fn run_cli(
    args: impl IntoIterator<Item = OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> CliOutput {
    match prepare_action(args, environment, content) {
        ProductAction::Output(output) => output,
        ProductAction::ServeLiteMcpStdio => {
            CliOutput::operation_failure("streaming-command-requires-product-entrypoint")
        }
    }
}

pub fn prepare_action(
    args: impl IntoIterator<Item = OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> ProductAction {
    let command = match parse_args(args) {
        Ok(command) => command,
        Err(error) => return ProductAction::Output(CliOutput::usage_failure(error)),
    };

    let output = match command {
        Command::Help => CliOutput::success_text(USAGE),
        Command::Version => {
            CliOutput::success_text(format!("qiongli {}\n", env!("CARGO_PKG_VERSION")))
        }
        Command::ContentHelp => CliOutput::success_text(CONTENT_USAGE),
        Command::ContentList => content_list(content),
        Command::ContentMaterialize { profile, target } => {
            content_materialize(content, profile, &target)
        }
        Command::ConfigHelp => CliOutput::success_text(CONFIG_USAGE),
        Command::ConfigShow => config_show(environment),
        Command::ConfigSet {
            expected_revision,
            default_profile,
        } => config_set(environment, expected_revision, default_profile),
        Command::InstallHelp => CliOutput::success_text(INSTALL_USAGE),
        Command::InstallStatus => install_status(),
        Command::InstallCodexStatus => install_codex_status(environment),
        Command::McpHelp => CliOutput::success_text(MCP_USAGE),
        Command::McpServeLiteStdio => return ProductAction::ServeLiteMcpStdio,
        Command::Status => status(environment, content),
        Command::Doctor => doctor(environment),
    };
    ProductAction::Output(output)
}

#[derive(Debug, Eq, PartialEq)]
enum Command {
    Help,
    Version,
    ContentHelp,
    ContentList,
    ContentMaterialize {
        profile: ProfileId,
        target: PathBuf,
    },
    ConfigHelp,
    ConfigShow,
    ConfigSet {
        expected_revision: u64,
        default_profile: ProfileId,
    },
    InstallHelp,
    InstallStatus,
    InstallCodexStatus,
    McpHelp,
    McpServeLiteStdio,
    Status,
    Doctor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UsageError {
    message: &'static str,
    usage: &'static str,
}

fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, UsageError> {
    let args = args.into_iter().collect::<Vec<_>>();
    let Some(command) = args.first().and_then(|value| value.to_str()) else {
        return Err(global_usage_error("a command or option is required"));
    };

    match command {
        "-h" | "--help" if args.len() == 1 => Ok(Command::Help),
        "--version" if args.len() == 1 => Ok(Command::Version),
        "content" => parse_content_args(&args[1..]),
        "config" => parse_config_args(&args[1..]),
        "install" => parse_install_args(&args[1..]),
        "mcp" => parse_mcp_args(&args[1..]),
        "status" if args.len() == 1 => Ok(Command::Status),
        "doctor" if args.len() == 1 => Ok(Command::Doctor),
        "-h" | "--help" | "--version" | "status" | "doctor" => {
            Err(global_usage_error("unexpected extra argument"))
        }
        _ => Err(global_usage_error("unknown command or option")),
    }
}

fn parse_install_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(install_usage_error("an install subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::InstallHelp),
        "status" if args.len() == 1 => Ok(Command::InstallStatus),
        "codex"
            if args.get(1).and_then(|value| value.to_str()) == Some("status")
                && args.len() == 2 =>
        {
            Ok(Command::InstallCodexStatus)
        }
        "--help" | "status" | "codex" => Err(install_usage_error("unexpected extra argument")),
        _ => Err(install_usage_error("unknown install subcommand")),
    }
}

fn parse_mcp_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(mcp_usage_error("an MCP subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::McpHelp),
        "serve"
            if args.get(1).and_then(|value| value.to_str()) == Some("--help")
                && args.len() == 2 =>
        {
            Ok(Command::McpHelp)
        }
        "serve" => parse_mcp_serve_options(&args[1..]),
        "--help" => Err(mcp_usage_error("unexpected extra argument")),
        _ => Err(mcp_usage_error("unknown MCP subcommand")),
    }
}

fn parse_mcp_serve_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut profile = None;
    let mut transport = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| mcp_usage_error("MCP option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| mcp_usage_error("MCP option value is required"))?
            .to_str()
            .ok_or_else(|| mcp_usage_error("MCP option value is not valid UTF-8"))?;
        match option {
            "--profile" if profile.is_none() => {
                if !matches!(value, "lite" | "marketplace-lite") {
                    return Err(mcp_usage_error("MCP profile is unavailable"));
                }
                profile = Some(value);
            }
            "--transport" if transport.is_none() => {
                if value != "stdio" {
                    return Err(mcp_usage_error("MCP transport is unavailable"));
                }
                transport = Some(value);
            }
            "--profile" | "--transport" => {
                return Err(mcp_usage_error("duplicate MCP option"));
            }
            _ => return Err(mcp_usage_error("unknown MCP option")),
        }
        index += 2;
    }
    if profile.is_none() {
        return Err(mcp_usage_error("MCP profile is required"));
    }
    if transport.is_none() {
        return Err(mcp_usage_error("MCP transport is required"));
    }
    Ok(Command::McpServeLiteStdio)
}

fn parse_content_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(content_usage_error("a content subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::ContentHelp),
        "list" if args.len() == 1 => Ok(Command::ContentList),
        "materialize" => parse_materialize_options(&args[1..]),
        "--help" | "list" => Err(content_usage_error("unexpected extra argument")),
        _ => Err(content_usage_error("unknown content subcommand")),
    }
}

fn parse_materialize_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut profile = None;
    let mut target = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| content_usage_error("content option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| content_usage_error("content option value is required"))?;
        match option {
            "--profile" if profile.is_none() => {
                profile = Some(
                    parse_profile(value)
                        .ok_or_else(|| content_usage_error("content profile is invalid"))?,
                );
            }
            "--target" if target.is_none() => target = Some(PathBuf::from(value)),
            "--profile" | "--target" => {
                return Err(content_usage_error("duplicate content option"));
            }
            _ => return Err(content_usage_error("unknown content option")),
        }
        index += 2;
    }
    Ok(Command::ContentMaterialize {
        profile: profile.ok_or_else(|| content_usage_error("content profile is required"))?,
        target: target.ok_or_else(|| content_usage_error("content target is required"))?,
    })
}

fn parse_config_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(config_usage_error("a config subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::ConfigHelp),
        "show" if args.len() == 1 => Ok(Command::ConfigShow),
        "set" => parse_config_set_options(&args[1..]),
        "--help" | "show" => Err(config_usage_error("unexpected extra argument")),
        _ => Err(config_usage_error("unknown config subcommand")),
    }
}

fn parse_config_set_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut expected_revision = None;
    let mut default_profile = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| config_usage_error("config option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| config_usage_error("config option value is required"))?;
        match option {
            "--expected-revision" if expected_revision.is_none() => {
                expected_revision = Some(
                    parse_revision(value)
                        .ok_or_else(|| config_usage_error("expected revision is invalid"))?,
                );
            }
            "--default-profile" if default_profile.is_none() => {
                default_profile = Some(
                    parse_profile(value)
                        .ok_or_else(|| config_usage_error("default profile is invalid"))?,
                );
            }
            "--expected-revision" | "--default-profile" => {
                return Err(config_usage_error("duplicate config option"));
            }
            _ => return Err(config_usage_error("unknown config option")),
        }
        index += 2;
    }
    Ok(Command::ConfigSet {
        expected_revision: expected_revision
            .ok_or_else(|| config_usage_error("expected revision is required"))?,
        default_profile: default_profile
            .ok_or_else(|| config_usage_error("default profile is required"))?,
    })
}

fn parse_profile(value: &OsStr) -> Option<ProfileId> {
    match value.to_str()? {
        "skill-only" => Some(ProfileId::SkillOnly),
        "marketplace-lite" | "lite" => Some(ProfileId::MarketplaceLite),
        "full" => Some(ProfileId::Full),
        _ => None,
    }
}

fn parse_revision(value: &OsStr) -> Option<u64> {
    value
        .to_str()?
        .parse::<u64>()
        .ok()
        .filter(|revision| *revision <= qiongli_config::MAX_GLOBAL_SETTINGS_REVISION)
}

const fn global_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: USAGE,
    }
}

const fn content_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: CONTENT_USAGE,
    }
}

const fn config_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: CONFIG_USAGE,
    }
}

const fn mcp_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: MCP_USAGE,
    }
}

const fn install_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: INSTALL_USAGE,
    }
}

fn content_list(content: &EmbeddedContent) -> CliOutput {
    let manifest = content.pack().manifest();
    json_output(
        &ContentListOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "content-list",
            pack_id: &manifest.pack_id,
            content_version: &manifest.content_version,
            source_commit: &manifest.source_commit,
            pack_sha256: content.pack().pack_sha256(),
            content_root_sha256: &manifest.content_root_sha256,
            profiles: content.profiles(),
        },
        0,
    )
}

fn content_materialize(content: &EmbeddedContent, profile: ProfileId, path: &Path) -> CliOutput {
    let target = match approve_materialization_target(path) {
        Ok(target) => target,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let receipt = match content.materialize_profile(profile_name(profile), &target) {
        Ok(receipt) => receipt,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &MaterializeOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "content-materialize",
            profile: receipt.profile,
            authorization: receipt.authorization,
            entry_count: receipt.entries.len(),
            pack_sha256: &receipt.pack_sha256,
            content_root_sha256: &receipt.content_root_sha256,
        },
        0,
    )
}

fn config_show(environment: &CommandEnvironment) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &ConfigShowOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "config-show",
            config: store.status(),
        },
        0,
    )
}

fn config_set(
    environment: &CommandEnvironment,
    expected_revision: u64,
    default_profile: ProfileId,
) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let mut loaded = match store.load() {
        Ok(loaded) => loaded,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    loaded.settings.default_profile = default_profile;
    let outcome = match store.replace(expected_revision, loaded.settings) {
        Ok(outcome) => outcome,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &ConfigSetOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "config-set",
            revision: outcome.revision,
            default_profile,
            cleanup_required: outcome.cleanup_required,
        },
        0,
    )
}

fn install_status() -> CliOutput {
    let (Some(os), Some(arch)) = (OperatingSystem::current(), Architecture::current()) else {
        return CliOutput::operation_failure("unsupported-build-target");
    };
    json_output(
        &InstallStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-status",
            contracts: InstallContractVersions {
                artifact_identity: ARTIFACT_IDENTITY_SCHEMA_VERSION,
                launch_grant: LAUNCH_GRANT_SCHEMA_VERSION,
                install_plan: INSTALL_PLAN_SCHEMA_VERSION,
                install_receipt: INSTALL_RECEIPT_SCHEMA_VERSION,
                codex_adapter: CODEX_ADAPTER_SCHEMA_VERSION,
                codex_registration_receipt: CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION,
                codex_registration_state: CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
            },
            current_target: InstallBuildTarget { os, arch },
            transaction_engine: "grant-and-approval-gated",
            launch_grant: "unavailable",
            preview: "unavailable",
            apply: "unavailable",
            targets: [
                InstallTargetStatus {
                    family: LocalTargetFamily::CodexLocal,
                    state: "adapter-engine-ready",
                },
                InstallTargetStatus {
                    family: LocalTargetFamily::ClaudeCodeLocal,
                    state: "contract-only",
                },
            ],
        },
        0,
    )
}

fn install_codex_status(environment: &CommandEnvironment) -> CliOutput {
    let Some(home) = environment.platform_home.as_deref() else {
        return CliOutput::operation_failure("codex-home-unavailable");
    };
    let target = match discover_codex_user(home) {
        Ok(target) => target,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &InstallCodexStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-codex-status",
            target: target.summary(),
            launch_grant: "unavailable",
            preview: "unavailable",
            apply: "unavailable",
            activation: "client-action-required",
        },
        0,
    )
}

fn status(environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &StatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "status",
            product_version: env!("CARGO_PKG_VERSION"),
            content: content_summary(content),
            config: store.status(),
        },
        0,
    )
}

fn doctor(environment: &CommandEnvironment) -> CliOutput {
    let store = match config_store(environment) {
        Ok(store) => store,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let config = store.status();
    let blocking = is_blocking_config_state(config.state);
    let checks = [
        DoctorCheck {
            id: "embedded-content",
            state: "ready",
            blocking: false,
            remediation_code: "none",
        },
        DoctorCheck {
            id: "global-config",
            state: config_state_code(config.state),
            blocking,
            remediation_code: config_remediation_code(config.state),
        },
        DoctorCheck {
            id: "secure-store",
            state: "unavailable",
            blocking: false,
            remediation_code: "secure-store-not-implemented",
        },
    ];
    json_output(
        &DoctorOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "doctor",
            overall: if blocking { "attention" } else { "ready" },
            checks,
        },
        u8::from(blocking),
    )
}

pub(crate) fn config_store(
    environment: &CommandEnvironment,
) -> Result<GlobalSettingsStore, ConfigError> {
    let home = environment
        .platform_home
        .as_deref()
        .ok_or(ConfigError::HomeUnavailable)?;
    let root = resolve_config_root(environment.configured_root.as_deref(), home)?;
    Ok(GlobalSettingsStore::new(root))
}

fn content_summary(content: &EmbeddedContent) -> ContentSummary<'_> {
    let manifest = content.pack().manifest();
    ContentSummary {
        state: "ready",
        pack_id: &manifest.pack_id,
        content_version: &manifest.content_version,
        pack_sha256: content.pack().pack_sha256(),
        content_root_sha256: &manifest.content_root_sha256,
        profiles: content.profiles(),
    }
}

const fn profile_name(profile: ProfileId) -> &'static str {
    match profile {
        ProfileId::SkillOnly => "skill-only",
        ProfileId::MarketplaceLite => "marketplace-lite",
        ProfileId::Full => "full",
    }
}

const fn is_blocking_config_state(state: ConfigState) -> bool {
    !matches!(state, ConfigState::Missing | ConfigState::Ready)
}

const fn config_state_code(state: ConfigState) -> &'static str {
    match state {
        ConfigState::Missing => "missing",
        ConfigState::Ready => "ready",
        ConfigState::Invalid => "invalid",
        ConfigState::FutureSchema => "future-schema",
        ConfigState::Insecure => "insecure",
        ConfigState::Busy => "busy",
        ConfigState::RecoveryRequired => "recovery-required",
        ConfigState::WriteUnsupported => "write-unsupported",
    }
}

const fn config_remediation_code(state: ConfigState) -> &'static str {
    match state {
        ConfigState::Missing | ConfigState::Ready => "none",
        ConfigState::Invalid => "inspect-global-config",
        ConfigState::FutureSchema => "upgrade-qiongli",
        ConfigState::Insecure => "repair-global-config-permissions",
        ConfigState::Busy => "retry-global-config",
        ConfigState::RecoveryRequired => "recover-global-config",
        ConfigState::WriteUnsupported => "use-supported-platform",
    }
}

fn json_output(value: &impl Serialize, exit_code: u8) -> CliOutput {
    match serde_json::to_string_pretty(value) {
        Ok(mut stdout) => {
            stdout.push('\n');
            CliOutput {
                exit_code,
                stdout,
                stderr: String::new(),
            }
        }
        Err(_) => CliOutput::operation_failure("output-serialization-failed"),
    }
}

#[derive(Serialize)]
struct ContentListOutput<'a> {
    schema_version: u32,
    command: &'static str,
    pack_id: &'a str,
    content_version: &'a str,
    source_commit: &'a str,
    pack_sha256: &'a str,
    content_root_sha256: &'a str,
    profiles: &'a [ProfileProjection],
}

#[derive(Serialize)]
struct MaterializeOutput<'a> {
    schema_version: u32,
    command: &'static str,
    profile: ProfileId,
    authorization: MaterializationAuthorization,
    entry_count: usize,
    pack_sha256: &'a str,
    content_root_sha256: &'a str,
}

#[derive(Serialize)]
struct ConfigShowOutput {
    schema_version: u32,
    command: &'static str,
    config: RedactedConfigStatus,
}

#[derive(Serialize)]
struct ConfigSetOutput {
    schema_version: u32,
    command: &'static str,
    revision: u64,
    default_profile: ProfileId,
    cleanup_required: bool,
}

#[derive(Serialize)]
struct InstallStatusOutput {
    schema_version: u32,
    command: &'static str,
    contracts: InstallContractVersions,
    current_target: InstallBuildTarget,
    transaction_engine: &'static str,
    launch_grant: &'static str,
    preview: &'static str,
    apply: &'static str,
    targets: [InstallTargetStatus; 2],
}

#[derive(Serialize)]
struct InstallCodexStatusOutput<'a> {
    schema_version: u32,
    command: &'static str,
    target: &'a CodexDiscoverySummaryV1,
    launch_grant: &'static str,
    preview: &'static str,
    apply: &'static str,
    activation: &'static str,
}

#[derive(Serialize)]
struct InstallContractVersions {
    artifact_identity: u32,
    launch_grant: u32,
    install_plan: u32,
    install_receipt: u32,
    codex_adapter: u32,
    codex_registration_receipt: u32,
    codex_registration_state: u32,
}

#[derive(Serialize)]
struct InstallBuildTarget {
    os: OperatingSystem,
    arch: Architecture,
}

#[derive(Serialize)]
struct InstallTargetStatus {
    family: LocalTargetFamily,
    state: &'static str,
}

#[derive(Serialize)]
struct ContentSummary<'a> {
    state: &'static str,
    pack_id: &'a str,
    content_version: &'a str,
    pack_sha256: &'a str,
    content_root_sha256: &'a str,
    profiles: &'a [ProfileProjection],
}

#[derive(Serialize)]
struct StatusOutput<'a> {
    schema_version: u32,
    command: &'static str,
    product_version: &'static str,
    content: ContentSummary<'a>,
    config: RedactedConfigStatus,
}

#[derive(Serialize)]
struct DoctorOutput {
    schema_version: u32,
    command: &'static str,
    overall: &'static str,
    checks: [DoctorCheck; 3],
}

#[derive(Clone, Copy, Serialize)]
struct DoctorCheck {
    id: &'static str,
    state: &'static str,
    blocking: bool,
    remediation_code: &'static str,
}

#[cfg(unix)]
fn process_platform_home() -> Option<PathBuf> {
    nonempty_environment_path("HOME")
}

#[cfg(windows)]
fn process_platform_home() -> Option<PathBuf> {
    nonempty_environment_path("USERPROFILE")
        .or_else(windows_drive_home)
        .or_else(|| nonempty_environment_path("HOME"))
}

#[cfg(not(any(unix, windows)))]
fn process_platform_home() -> Option<PathBuf> {
    nonempty_environment_path("HOME")
}

fn nonempty_environment_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[cfg(windows)]
fn windows_drive_home() -> Option<PathBuf> {
    let mut drive = env::var_os("HOMEDRIVE").filter(|value| !value.is_empty())?;
    let path = env::var_os("HOMEPATH").filter(|value| !value.is_empty())?;
    drive.push(path);
    Some(PathBuf::from(drive))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_accepts_the_frozen_command_families() {
        assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
        assert_eq!(
            parse_args(args(&["content", "list"])),
            Ok(Command::ContentList)
        );
        assert_eq!(
            parse_args(args(&["config", "show"])),
            Ok(Command::ConfigShow)
        );
        assert_eq!(parse_args(args(&["status"])), Ok(Command::Status));
        assert_eq!(parse_args(args(&["doctor"])), Ok(Command::Doctor));
        assert_eq!(
            parse_args(args(&[
                "mcp",
                "serve",
                "--profile",
                "lite",
                "--transport",
                "stdio",
            ])),
            Ok(Command::McpServeLiteStdio)
        );
    }

    #[test]
    fn parser_accepts_both_option_orders_and_the_lite_alias() {
        let first = parse_args(args(&[
            "content",
            "materialize",
            "--profile",
            "lite",
            "--target",
            "/approved/target",
        ]));
        let second = parse_args(args(&[
            "content",
            "materialize",
            "--target",
            "/approved/target",
            "--profile",
            "marketplace-lite",
        ]));
        assert_eq!(first, second);

        let first = parse_args(args(&[
            "mcp",
            "serve",
            "--profile",
            "lite",
            "--transport",
            "stdio",
        ]));
        let second = parse_args(args(&[
            "mcp",
            "serve",
            "--transport",
            "stdio",
            "--profile",
            "marketplace-lite",
        ]));
        assert_eq!(first, second);

        let first = parse_args(args(&[
            "config",
            "set",
            "--expected-revision",
            "7",
            "--default-profile",
            "full",
        ]));
        let second = parse_args(args(&[
            "config",
            "set",
            "--default-profile",
            "full",
            "--expected-revision",
            "7",
        ]));
        assert_eq!(first, second);
    }

    #[test]
    fn parser_rejects_missing_duplicate_unknown_and_private_values() {
        for values in [
            vec![],
            vec!["content"],
            vec!["content", "list", "extra"],
            vec!["content", "materialize", "--profile", "full"],
            vec![
                "content",
                "materialize",
                "--profile",
                "full",
                "--profile",
                "full",
                "--target",
                "/approved/target",
            ],
            vec!["config", "set", "--expected-revision", "not-a-number"],
            vec!["config", "set", "--api-key", "private-canary"],
            vec!["status", "extra"],
            vec!["mcp"],
            vec!["mcp", "serve", "--profile", "lite"],
            vec!["mcp", "serve", "--transport", "stdio"],
            vec!["mcp", "serve", "--profile", "full", "--transport", "stdio"],
            vec!["mcp", "serve", "--profile", "lite", "--transport", "http"],
            vec![
                "mcp",
                "serve",
                "--profile",
                "lite",
                "--profile",
                "lite",
                "--transport",
                "stdio",
            ],
        ] {
            assert!(parse_args(args(&values)).is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn parser_rejects_non_utf8_unix_control_tokens() {
        use std::os::unix::ffi::OsStringExt;

        let invalid = OsString::from_vec(vec![0xff, 0xfe]);
        assert!(parse_args([invalid]).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn parser_rejects_unpaired_windows_surrogates_in_control_tokens() {
        use std::os::windows::ffi::OsStringExt;

        let invalid = OsString::from_wide(&[0xd800]);
        assert!(parse_args([invalid]).is_err());
    }
}
