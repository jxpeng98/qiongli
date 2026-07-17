use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use qiongli_config::{
    ConfigError, ConfigRoot, ConfigState, GlobalSettingsStore, RedactedConfigStatus,
    UpdateStateStore, UpdateStreamPreference, resolve_config_root,
};
use qiongli_content::{
    EmbeddedContent, MaterializationAuthorization, ProfileId, ProfileProjection,
    approve_materialization_target, verify_materialization,
};
use qiongli_platform::{
    ARTIFACT_IDENTITY_SCHEMA_VERSION, Architecture, CLAUDE_ADAPTER_SCHEMA_VERSION,
    CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION, CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
    CODEX_ADAPTER_SCHEMA_VERSION, CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION,
    CODEX_REGISTRATION_STATE_SCHEMA_VERSION, ClaudeDiscoverySummaryV1, ClientActivationTarget,
    CodexDiscoverySummaryV1, INSTALL_PLAN_SCHEMA_VERSION, INSTALL_RECEIPT_SCHEMA_VERSION,
    LAUNCH_GRANT_SCHEMA_VERSION, LocalTargetFamily, NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION,
    NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION, NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION,
    NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION, NativeReleaseAuthority, OperatingSystem,
    discover_claude_user_with_config, discover_codex_user,
};
use serde::Serialize;

use crate::candidate_cli::{CandidateCliCommand, CandidateReceiptOptions, CandidateReleaseOptions};
use crate::native_cli::{
    NativeCliCommand, NativeClientTarget, NativeReceiptOptions, NativeReleaseOptions,
};
use crate::update_cli::UpdateCliCommand;

const OUTPUT_SCHEMA_VERSION: u32 = 1;

const USAGE: &str = "Qiongli native platform\n\nUsage:\n  qiongli\n  qiongli --version\n  qiongli --help\n  qiongli ui [--startup-check]\n  qiongli ui --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude>\n  qiongli content list\n  qiongli content materialize --profile <profile> --target <absolute-path>\n  qiongli config show\n  qiongli config set --expected-revision <revision> --default-profile <profile>\n  qiongli update status\n  qiongli update channel --expected-revision <revision> --stream <stable|beta>\n  qiongli update check\n  qiongli update download --expected-revision <revision>\n  qiongli update verify --expected-revision <revision>\n  qiongli update stage --expected-revision <revision>\n  qiongli update install --expected-revision <revision>\n  qiongli update cancel --expected-revision <revision>\n  qiongli install status\n  qiongli install codex status\n  qiongli install claude status\n  qiongli install candidate <preview|apply|verify|remove> [options]\n  qiongli install native <preview|apply|verify|remove> [options]\n  qiongli mcp serve --profile <lite|marketplace-lite> --transport stdio\n  qiongli status\n  qiongli doctor\n\nProfiles:\n  skill-only | marketplace-lite | lite | full\n\nOptions:\n  -h, --help  Print help\n  --version   Print the native product version\n";

const CONTENT_USAGE: &str = "Qiongli embedded content\n\nUsage:\n  qiongli content list\n  qiongli content materialize --profile <profile> --target <absolute-path>\n  qiongli content --help\n";

const CONFIG_USAGE: &str = "Qiongli global config\n\nUsage:\n  qiongli config show\n  qiongli config set --expected-revision <revision> --default-profile <profile>\n  qiongli config --help\n";

const UPDATE_USAGE: &str = "Qiongli native update\n\nUsage:\n  qiongli update status\n  qiongli update channel --expected-revision <revision> --stream <stable|beta>\n  qiongli update check\n  qiongli update download --expected-revision <revision>\n  qiongli update verify --expected-revision <revision>\n  qiongli update stage --expected-revision <revision>\n  qiongli update install --expected-revision <revision>\n  qiongli update cancel --expected-revision <revision>\n  qiongli update --help\n";

const MCP_USAGE: &str = "Qiongli native MCP\n\nUsage:\n  qiongli mcp serve --profile <lite|marketplace-lite> --transport stdio\n  qiongli mcp --help\n";

const INSTALL_USAGE: &str = "Qiongli native installation\n\nUsage:\n  qiongli install status\n  qiongli install codex status\n  qiongli install claude status\n  qiongli install candidate preview --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude>\n  qiongli install candidate apply --candidate <candidate.json> --archive <archive> --release-notes <notes.md> --target <codex|claude> --expected-approval-digest <sha256> --approve-filesystem-write --approve-client-config-change --approve-host-trust\n  qiongli install candidate verify --target <codex|claude> --install-id <native-payload-id>\n  qiongli install candidate remove --target <codex|claude> --install-id <native-payload-id> --approve-filesystem-write --approve-client-config-change\n  qiongli install native preview --release <release.json> --archive <archive> --managed-root <absolute-path> --target <codex|claude>\n  qiongli install native apply --release <release.json> --archive <archive> --managed-root <absolute-path> --target <codex|claude> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli install native verify --managed-root <absolute-path> --install-id <native-payload-id>\n  qiongli install native remove --managed-root <absolute-path> --install-id <native-payload-id> --approve-filesystem-write\n  qiongli install --help\n";

#[derive(Clone, Default)]
pub struct CommandEnvironment {
    configured_root: Option<OsString>,
    platform_home: Option<PathBuf>,
    claude_config_root: Option<PathBuf>,
}

impl CommandEnvironment {
    #[must_use]
    pub fn from_process() -> Self {
        Self {
            configured_root: env::var_os("QIONGLI_CONFIG_HOME"),
            platform_home: process_platform_home(),
            claude_config_root: nonempty_environment_path("CLAUDE_CONFIG_DIR"),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_paths(
        configured_root: Option<OsString>,
        platform_home: Option<PathBuf>,
        claude_config_root: Option<PathBuf>,
    ) -> Self {
        Self {
            configured_root,
            platform_home,
            claude_config_root,
        }
    }

    pub(crate) fn platform_home(&self) -> Option<&Path> {
        self.platform_home.as_deref()
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn configured_root(&self) -> Option<&OsStr> {
        self.configured_root.as_deref()
    }

    pub(crate) fn claude_config_root(&self) -> Option<&Path> {
        self.claude_config_root.as_deref()
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
    LaunchDesktop,
    LaunchDesktopWithCandidate(Box<crate::DesktopCandidateSession>),
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
        ProductAction::LaunchDesktop => {
            CliOutput::operation_failure("desktop-command-requires-product-entrypoint")
        }
        ProductAction::LaunchDesktopWithCandidate(_) => {
            CliOutput::operation_failure("desktop-command-requires-product-entrypoint")
        }
    }
}

pub fn prepare_action(
    args: impl IntoIterator<Item = OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> ProductAction {
    let authority = match crate::embedded_release_authority() {
        Ok(authority) => authority,
        Err(_) => {
            return ProductAction::Output(CliOutput::operation_failure(
                "native-release-authority-invalid",
            ));
        }
    };
    prepare_action_with_release_authority(args, environment, content, authority.as_ref())
}

pub(crate) fn prepare_action_with_release_authority(
    args: impl IntoIterator<Item = OsString>,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    authority: Option<&NativeReleaseAuthority>,
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
        Command::Ui => return ProductAction::LaunchDesktop,
        Command::UiCandidate(options) => {
            let authority = match authority {
                Some(authority) => authority,
                None => {
                    return ProductAction::Output(CliOutput::operation_failure(
                        "native-release-authority-unavailable",
                    ));
                }
            };
            let source_commit = match crate::embedded_source_commit() {
                Some(source_commit) => source_commit,
                None => {
                    return ProductAction::Output(CliOutput::operation_failure(
                        "native-source-commit-unavailable",
                    ));
                }
            };
            let now_unix = match crate::candidate_cli::now_unix() {
                Ok(now_unix) => now_unix,
                Err(reason_code) => {
                    return ProductAction::Output(CliOutput::operation_failure(reason_code));
                }
            };
            let prepared = match crate::candidate_cli::prepare_candidate(
                &options,
                authority,
                source_commit,
                content,
                now_unix,
            ) {
                Ok(prepared) => prepared,
                Err(reason_code) => {
                    return ProductAction::Output(CliOutput::operation_failure(reason_code));
                }
            };
            return ProductAction::LaunchDesktopWithCandidate(Box::new(
                crate::DesktopCandidateSession::new(prepared.into_verified()),
            ));
        }
        Command::UiStartupCheck => ui_startup_check(environment, content),
        Command::ContentHelp => CliOutput::success_text(CONTENT_USAGE),
        Command::ContentList => content_list(content),
        Command::ContentMaterialize { profile, target } => {
            content_materialize(environment, content, profile, &target)
        }
        Command::ConfigHelp => CliOutput::success_text(CONFIG_USAGE),
        Command::ConfigShow => config_show(environment),
        Command::ConfigSet {
            expected_revision,
            default_profile,
        } => config_set(environment, expected_revision, default_profile),
        Command::UpdateHelp => CliOutput::success_text(UPDATE_USAGE),
        Command::Update(command) => {
            let store = match update_store(environment) {
                Ok(store) => store,
                Err(error) => {
                    return ProductAction::Output(CliOutput::operation_failure(
                        error.reason_code(),
                    ));
                }
            };
            match crate::update_cli::execute(
                command,
                &store,
                authority,
                crate::embedded_macos_team_id(),
                environment,
                content,
            ) {
                Ok(output) => json_output(&output, 0),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::InstallHelp => CliOutput::success_text(INSTALL_USAGE),
        Command::InstallStatus => install_status(authority),
        Command::InstallCodexStatus => install_codex_status(environment),
        Command::InstallClaudeStatus => install_claude_status(environment),
        Command::InstallCandidate(command) => {
            match crate::candidate_cli::execute(
                command,
                authority,
                crate::embedded_source_commit(),
                environment.platform_home(),
                content,
            ) {
                Ok(output) => json_output(&output, 0),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
        Command::InstallNative(command) => {
            match crate::native_cli::execute(command, authority, content) {
                Ok(output) => json_output(&output, 0),
                Err(reason_code) => CliOutput::operation_failure(reason_code),
            }
        }
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
    Ui,
    UiCandidate(CandidateReleaseOptions),
    UiStartupCheck,
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
    UpdateHelp,
    Update(UpdateCliCommand),
    InstallHelp,
    InstallStatus,
    InstallCodexStatus,
    InstallClaudeStatus,
    InstallCandidate(CandidateCliCommand),
    InstallNative(NativeCliCommand),
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
    if args.is_empty() {
        return Ok(Command::Ui);
    }
    let Some(command) = args.first().and_then(|value| value.to_str()) else {
        return Err(global_usage_error("command or option is not valid text"));
    };

    match command {
        "-h" | "--help" if args.len() == 1 => Ok(Command::Help),
        "--version" if args.len() == 1 => Ok(Command::Version),
        "content" => parse_content_args(&args[1..]),
        "config" => parse_config_args(&args[1..]),
        "update" => parse_update_args(&args[1..]),
        "install" => parse_install_args(&args[1..]),
        "mcp" => parse_mcp_args(&args[1..]),
        "ui" if args.len() == 1 => Ok(Command::Ui),
        "ui" if args.get(1).and_then(|value| value.to_str()) == Some("--startup-check")
            && args.len() == 2 =>
        {
            Ok(Command::UiStartupCheck)
        }
        "ui" if args.len() > 1 => parse_candidate_release_options(&args[1..], false)
            .map(|parsed| Command::UiCandidate(parsed.options)),
        "status" if args.len() == 1 => Ok(Command::Status),
        "doctor" if args.len() == 1 => Ok(Command::Doctor),
        "-h" | "--help" | "--version" | "ui" | "status" | "doctor" => {
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
        "claude"
            if args.get(1).and_then(|value| value.to_str()) == Some("status")
                && args.len() == 2 =>
        {
            Ok(Command::InstallClaudeStatus)
        }
        "candidate"
            if args.get(1).and_then(|value| value.to_str()) == Some("--help")
                && args.len() == 2 =>
        {
            Ok(Command::InstallHelp)
        }
        "candidate" => parse_candidate_install_args(&args[1..]).map(Command::InstallCandidate),
        "native"
            if args.get(1).and_then(|value| value.to_str()) == Some("--help")
                && args.len() == 2 =>
        {
            Ok(Command::InstallHelp)
        }
        "native" => parse_native_install_args(&args[1..]).map(Command::InstallNative),
        "--help" | "status" | "codex" | "claude" => {
            Err(install_usage_error("unexpected extra argument"))
        }
        _ => Err(install_usage_error("unknown install subcommand")),
    }
}

fn parse_candidate_install_args(args: &[OsString]) -> Result<CandidateCliCommand, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(install_usage_error(
            "a candidate install subcommand is required",
        ));
    };
    match subcommand {
        "preview" => parse_candidate_release_options(&args[1..], false)
            .map(|parsed| CandidateCliCommand::Preview(parsed.options)),
        "apply" => parse_candidate_release_options(&args[1..], true).and_then(|parsed| {
            Ok(CandidateCliCommand::Apply {
                options: parsed.options,
                expected_approval_digest: parsed.expected_approval_digest.ok_or_else(|| {
                    install_usage_error("expected candidate approval digest is required")
                })?,
            })
        }),
        "verify" => {
            parse_candidate_receipt_options(&args[1..], false).map(CandidateCliCommand::Verify)
        }
        "remove" => {
            parse_candidate_receipt_options(&args[1..], true).map(CandidateCliCommand::Remove)
        }
        "--help" if args.len() == 1 => Err(install_usage_error(
            "use qiongli install --help for candidate install options",
        )),
        "--help" => Err(install_usage_error("unexpected candidate install argument")),
        _ => Err(install_usage_error("unknown candidate install subcommand")),
    }
}

struct ParsedCandidateReleaseOptions {
    options: CandidateReleaseOptions,
    expected_approval_digest: Option<String>,
}

fn parse_candidate_release_options(
    args: &[OsString],
    apply: bool,
) -> Result<ParsedCandidateReleaseOptions, UsageError> {
    let mut candidate = None;
    let mut archive = None;
    let mut release_notes = None;
    let mut target = None;
    let mut expected_approval_digest = None;
    let mut filesystem_approved = false;
    let mut config_approved = false;
    let mut host_trust_approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("candidate install option is not valid UTF-8"))?;
        let approval = match option {
            "--approve-filesystem-write" => Some(&mut filesystem_approved),
            "--approve-client-config-change" => Some(&mut config_approved),
            "--approve-host-trust" => Some(&mut host_trust_approved),
            _ => None,
        };
        if let Some(approved) = approval {
            if !apply || *approved {
                return Err(install_usage_error(
                    "candidate install approval is unexpected or duplicate",
                ));
            }
            *approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("candidate install option value is required"))?;
        match option {
            "--candidate" if candidate.is_none() => candidate = nonempty_path(value),
            "--archive" if archive.is_none() => archive = nonempty_path(value),
            "--release-notes" if release_notes.is_none() => release_notes = nonempty_path(value),
            "--target" if target.is_none() => {
                target =
                    Some(parse_candidate_target(value).ok_or_else(|| {
                        install_usage_error("candidate install target is invalid")
                    })?);
            }
            "--expected-approval-digest" if apply && expected_approval_digest.is_none() => {
                expected_approval_digest =
                    Some(parse_sha256(value).ok_or_else(|| {
                        install_usage_error("candidate approval digest is invalid")
                    })?);
            }
            "--candidate"
            | "--archive"
            | "--release-notes"
            | "--target"
            | "--expected-approval-digest" => {
                return Err(install_usage_error(
                    "candidate install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown candidate install option")),
        }
        if matches!(option, "--candidate" | "--archive" | "--release-notes") && value.is_empty() {
            return Err(install_usage_error("candidate install path is empty"));
        }
        index += 2;
    }
    if apply && !(filesystem_approved && config_approved && host_trust_approved) {
        return Err(install_usage_error(
            "all candidate install approvals are required",
        ));
    }
    Ok(ParsedCandidateReleaseOptions {
        options: CandidateReleaseOptions {
            candidate: candidate
                .ok_or_else(|| install_usage_error("release candidate path is required"))?,
            archive: archive
                .ok_or_else(|| install_usage_error("candidate archive path is required"))?,
            release_notes: release_notes
                .ok_or_else(|| install_usage_error("release notes path is required"))?,
            target: target
                .ok_or_else(|| install_usage_error("candidate install target is required"))?,
        },
        expected_approval_digest,
    })
}

fn parse_candidate_receipt_options(
    args: &[OsString],
    remove: bool,
) -> Result<CandidateReceiptOptions, UsageError> {
    let mut target = None;
    let mut install_id = None;
    let mut filesystem_approved = false;
    let mut config_approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("candidate install option is not valid UTF-8"))?;
        let approval = match option {
            "--approve-filesystem-write" => Some(&mut filesystem_approved),
            "--approve-client-config-change" => Some(&mut config_approved),
            _ => None,
        };
        if let Some(approved) = approval {
            if !remove || *approved {
                return Err(install_usage_error(
                    "candidate remove approval is unexpected or duplicate",
                ));
            }
            *approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("candidate install option value is required"))?;
        match option {
            "--target" if target.is_none() => {
                target =
                    Some(parse_candidate_target(value).ok_or_else(|| {
                        install_usage_error("candidate install target is invalid")
                    })?);
            }
            "--install-id" if install_id.is_none() => {
                install_id = Some(
                    parse_native_install_id(value)
                        .ok_or_else(|| install_usage_error("candidate install ID is invalid"))?,
                );
            }
            "--target" | "--install-id" => {
                return Err(install_usage_error(
                    "candidate install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown candidate install option")),
        }
        index += 2;
    }
    if remove && !(filesystem_approved && config_approved) {
        return Err(install_usage_error(
            "all candidate remove approvals are required",
        ));
    }
    Ok(CandidateReceiptOptions {
        target: target
            .ok_or_else(|| install_usage_error("candidate install target is required"))?,
        install_id: install_id
            .ok_or_else(|| install_usage_error("candidate install ID is required"))?,
    })
}

fn parse_native_install_args(args: &[OsString]) -> Result<NativeCliCommand, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(install_usage_error(
            "a native install subcommand is required",
        ));
    };
    match subcommand {
        "preview" => parse_native_release_options(&args[1..], false)
            .map(|parsed| NativeCliCommand::Preview(parsed.options)),
        "apply" => parse_native_release_options(&args[1..], true).and_then(|parsed| {
            Ok(NativeCliCommand::Apply {
                options: parsed.options,
                expected_plan_digest: parsed.expected_plan_digest.ok_or_else(|| {
                    install_usage_error("expected native install plan digest is required")
                })?,
            })
        }),
        "verify" => parse_native_receipt_options(&args[1..], false)
            .map(|parsed| NativeCliCommand::Verify(parsed.options)),
        "remove" => parse_native_receipt_options(&args[1..], true)
            .map(|parsed| NativeCliCommand::Remove(parsed.options)),
        "--help" if args.len() == 1 => Err(install_usage_error(
            "use qiongli install --help for native install options",
        )),
        "--help" => Err(install_usage_error("unexpected native install argument")),
        _ => Err(install_usage_error("unknown native install subcommand")),
    }
}

struct ParsedNativeReleaseOptions {
    options: NativeReleaseOptions,
    expected_plan_digest: Option<String>,
}

fn parse_native_release_options(
    args: &[OsString],
    apply: bool,
) -> Result<ParsedNativeReleaseOptions, UsageError> {
    let mut release = None;
    let mut archive = None;
    let mut managed_root = None;
    let mut target = None;
    let mut expected_plan_digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("native install option is not valid UTF-8"))?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err(install_usage_error(
                    "native install approval option is unexpected or duplicate",
                ));
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("native install option value is required"))?;
        match option {
            "--release" if release.is_none() => release = nonempty_path(value),
            "--archive" if archive.is_none() => archive = nonempty_path(value),
            "--managed-root" if managed_root.is_none() => managed_root = nonempty_path(value),
            "--target" if target.is_none() => {
                target = Some(
                    parse_native_client_target(value)
                        .ok_or_else(|| install_usage_error("native install target is invalid"))?,
                );
            }
            "--expected-plan-digest" if apply && expected_plan_digest.is_none() => {
                expected_plan_digest =
                    Some(parse_sha256(value).ok_or_else(|| {
                        install_usage_error("native install plan digest is invalid")
                    })?);
            }
            "--release"
            | "--archive"
            | "--managed-root"
            | "--target"
            | "--expected-plan-digest" => {
                return Err(install_usage_error(
                    "native install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown native install option")),
        }
        if matches!(option, "--release" | "--archive" | "--managed-root") && value.is_empty() {
            return Err(install_usage_error("native install path is empty"));
        }
        index += 2;
    }
    if apply && !approved {
        return Err(install_usage_error(
            "explicit filesystem-write approval is required",
        ));
    }
    Ok(ParsedNativeReleaseOptions {
        options: NativeReleaseOptions {
            release: release
                .ok_or_else(|| install_usage_error("native release path is required"))?,
            archive: archive
                .ok_or_else(|| install_usage_error("native archive path is required"))?,
            managed_root: managed_root
                .ok_or_else(|| install_usage_error("native managed root is required"))?,
            target: target
                .ok_or_else(|| install_usage_error("native install target is required"))?,
        },
        expected_plan_digest,
    })
}

struct ParsedNativeReceiptOptions {
    options: NativeReceiptOptions,
}

fn parse_native_receipt_options(
    args: &[OsString],
    remove: bool,
) -> Result<ParsedNativeReceiptOptions, UsageError> {
    let mut managed_root = None;
    let mut install_id = None;
    let mut approved = false;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| install_usage_error("native install option is not valid UTF-8"))?;
        if option == "--approve-filesystem-write" {
            if !remove || approved {
                return Err(install_usage_error(
                    "native install approval option is unexpected or duplicate",
                ));
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| install_usage_error("native install option value is required"))?;
        match option {
            "--managed-root" if managed_root.is_none() => managed_root = nonempty_path(value),
            "--install-id" if install_id.is_none() => {
                install_id = Some(
                    parse_native_install_id(value)
                        .ok_or_else(|| install_usage_error("native install ID is invalid"))?,
                );
            }
            "--managed-root" | "--install-id" => {
                return Err(install_usage_error(
                    "native install option is unexpected or duplicate",
                ));
            }
            _ => return Err(install_usage_error("unknown native install option")),
        }
        if option == "--managed-root" && value.is_empty() {
            return Err(install_usage_error("native install path is empty"));
        }
        index += 2;
    }
    if remove && !approved {
        return Err(install_usage_error(
            "explicit filesystem-write approval is required",
        ));
    }
    Ok(ParsedNativeReceiptOptions {
        options: NativeReceiptOptions {
            managed_root: managed_root
                .ok_or_else(|| install_usage_error("native managed root is required"))?,
            install_id: install_id
                .ok_or_else(|| install_usage_error("native install ID is required"))?,
        },
    })
}

fn nonempty_path(value: &OsStr) -> Option<PathBuf> {
    (!value.is_empty()).then(|| PathBuf::from(value))
}

fn parse_native_client_target(value: &OsStr) -> Option<NativeClientTarget> {
    match value.to_str()? {
        "codex" => Some(NativeClientTarget::Codex),
        "claude" => Some(NativeClientTarget::Claude),
        _ => None,
    }
}

fn parse_candidate_target(value: &OsStr) -> Option<ClientActivationTarget> {
    match value.to_str()? {
        "codex" => Some(ClientActivationTarget::Codex),
        "claude" => Some(ClientActivationTarget::ClaudeCode),
        _ => None,
    }
}

fn parse_sha256(value: &OsStr) -> Option<String> {
    let value = value.to_str()?;
    (value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then(|| value.to_string())
}

fn parse_native_install_id(value: &OsStr) -> Option<String> {
    let value = value.to_str()?;
    let digest = value.strip_prefix("native-payload-")?;
    (digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then(|| value.to_string())
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

fn parse_update_args(args: &[OsString]) -> Result<Command, UsageError> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err(update_usage_error("an update subcommand is required"));
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(Command::UpdateHelp),
        "status" if args.len() == 1 => Ok(Command::Update(UpdateCliCommand::Status)),
        "check" if args.len() == 1 => Ok(Command::Update(UpdateCliCommand::Check)),
        "channel" => parse_update_channel_options(&args[1..]),
        "download" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Download { expected_revision })
        }),
        "verify" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Verify { expected_revision })
        }),
        "stage" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Stage { expected_revision })
        }),
        "install" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Install { expected_revision })
        }),
        "reconcile" => parse_update_health_options(&args[1..]).map(|command| match command {
            Command::Update(UpdateCliCommand::Health { transaction_id }) => {
                Command::Update(UpdateCliCommand::Reconcile { transaction_id })
            }
            _ => unreachable!("health parser always returns an update health command"),
        }),
        "health" => parse_update_health_options(&args[1..]),
        "cancel" => parse_update_expected_revision(&args[1..]).map(|expected_revision| {
            Command::Update(UpdateCliCommand::Cancel { expected_revision })
        }),
        "--help" | "status" | "check" => Err(update_usage_error("unexpected extra argument")),
        _ => Err(update_usage_error("unknown update subcommand")),
    }
}

fn parse_update_health_options(args: &[OsString]) -> Result<Command, UsageError> {
    if args.len() != 2 || args.first().and_then(|value| value.to_str()) != Some("--transaction-id")
    {
        return Err(update_usage_error(
            "exactly one transaction id option is required",
        ));
    }
    let transaction_id = args[1]
        .to_str()
        .ok_or_else(|| update_usage_error("transaction id is invalid"))?;
    Ok(Command::Update(UpdateCliCommand::Health {
        transaction_id: transaction_id.to_string(),
    }))
}

fn parse_update_expected_revision(args: &[OsString]) -> Result<u64, UsageError> {
    if args.len() != 2
        || args.first().and_then(|value| value.to_str()) != Some("--expected-revision")
    {
        return Err(update_usage_error(
            "exactly one expected revision option is required",
        ));
    }
    parse_revision(&args[1]).ok_or_else(|| update_usage_error("expected revision is invalid"))
}

fn parse_update_channel_options(args: &[OsString]) -> Result<Command, UsageError> {
    let mut expected_revision = None;
    let mut stream = None;
    let mut index = 0;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or_else(|| update_usage_error("update option is not valid UTF-8"))?;
        let value = args
            .get(index + 1)
            .ok_or_else(|| update_usage_error("update option value is required"))?;
        match option {
            "--expected-revision" if expected_revision.is_none() => {
                expected_revision = Some(
                    parse_revision(value)
                        .ok_or_else(|| update_usage_error("expected revision is invalid"))?,
                );
            }
            "--stream" if stream.is_none() => {
                stream = Some(match value.to_str() {
                    Some("stable") => UpdateStreamPreference::Stable,
                    Some("beta") => UpdateStreamPreference::Beta,
                    _ => return Err(update_usage_error("update stream is invalid")),
                });
            }
            "--expected-revision" | "--stream" => {
                return Err(update_usage_error("duplicate update option"));
            }
            _ => return Err(update_usage_error("unknown update option")),
        }
        index += 2;
    }
    Ok(Command::Update(UpdateCliCommand::Channel {
        expected_revision: expected_revision
            .ok_or_else(|| update_usage_error("expected revision is required"))?,
        stream: stream.ok_or_else(|| update_usage_error("update stream is required"))?,
    }))
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

const fn update_usage_error(message: &'static str) -> UsageError {
    UsageError {
        message,
        usage: UPDATE_USAGE,
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

fn ui_startup_check(environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
    if crate::desktop::validate_desktop_startup(environment, content).is_err() {
        return CliOutput::operation_failure("desktop-startup-check-failed");
    }
    let (Some(os), Some(arch)) = (OperatingSystem::current(), Architecture::current()) else {
        return CliOutput::operation_failure("unsupported-build-target");
    };
    json_output(
        &UiStartupCheckOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "ui-startup-check",
            product_version: env!("CARGO_PKG_VERSION"),
            current_target: InstallBuildTarget { os, arch },
            service: "ready",
            snapshot: "ready",
            app_state: "ready",
            update_surface: "ready",
            window_entrypoint: "available",
            window: "not-opened",
        },
        0,
    )
}

fn content_materialize(
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
    profile: ProfileId,
    path: &Path,
) -> CliOutput {
    let root = match config_root(environment) {
        Ok(root) => root,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let target = match approve_materialization_target(path) {
        Ok(target) => target,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let previous = verify_materialization(&target).ok();
    let receipt = match content.materialize_profile(profile_name(profile), &target) {
        Ok(receipt) => receipt,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    if let Err(reason_code) = crate::managed_content::register_managed_materialization(
        root.state_root(),
        &target,
        &receipt,
    ) {
        return match crate::managed_content::compensate_unregistered_materialization(
            content,
            &target,
            &receipt,
            previous.as_ref(),
        ) {
            Ok(()) => CliOutput::operation_failure(reason_code),
            Err(recovery) => CliOutput::operation_failure(recovery),
        };
    }
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

fn install_status(authority: Option<&NativeReleaseAuthority>) -> CliOutput {
    let (Some(os), Some(arch)) = (OperatingSystem::current(), Architecture::current()) else {
        return CliOutput::operation_failure("unsupported-build-target");
    };
    let candidate_ready = authority.is_some() && crate::embedded_source_commit().is_some();
    json_output(
        &InstallStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-status",
            contracts: InstallContractVersions {
                artifact_identity: ARTIFACT_IDENTITY_SCHEMA_VERSION,
                launch_grant: LAUNCH_GRANT_SCHEMA_VERSION,
                release_authority: NATIVE_RELEASE_AUTHORITY_SCHEMA_VERSION,
                release_envelope: NATIVE_RELEASE_ENVELOPE_SCHEMA_VERSION,
                release_candidate: NATIVE_RELEASE_CANDIDATE_SCHEMA_VERSION,
                install_plan: INSTALL_PLAN_SCHEMA_VERSION,
                install_receipt: INSTALL_RECEIPT_SCHEMA_VERSION,
                native_payload_install_receipt: NATIVE_PAYLOAD_INSTALL_RECEIPT_SCHEMA_VERSION,
                codex_adapter: CODEX_ADAPTER_SCHEMA_VERSION,
                codex_registration_receipt: CODEX_REGISTRATION_RECEIPT_SCHEMA_VERSION,
                codex_registration_state: CODEX_REGISTRATION_STATE_SCHEMA_VERSION,
                claude_adapter: CLAUDE_ADAPTER_SCHEMA_VERSION,
                claude_registration_receipt: CLAUDE_REGISTRATION_RECEIPT_SCHEMA_VERSION,
                claude_registration_state: CLAUDE_REGISTRATION_STATE_SCHEMA_VERSION,
            },
            current_target: InstallBuildTarget { os, arch },
            transaction_engine: "grant-and-approval-gated",
            release_authority: if authority.is_some() {
                "embedded"
            } else {
                "unavailable"
            },
            launch_grant: if authority.is_some() {
                "release-bound"
            } else {
                "unavailable"
            },
            source_commit: if crate::embedded_source_commit().is_some() {
                "embedded"
            } else {
                "unavailable"
            },
            candidate: if candidate_ready {
                "signed-current-target-required"
            } else {
                "unavailable"
            },
            preview: if candidate_ready {
                "signed-candidate-required"
            } else {
                "unavailable"
            },
            apply: if candidate_ready {
                "signed-candidate-and-approval-required"
            } else {
                "unavailable"
            },
            verify: "receipt-backed",
            remove: "receipt-backed-explicit-approval",
            targets: [
                InstallTargetStatus {
                    family: LocalTargetFamily::CodexLocal,
                    state: "adapter-engine-ready",
                },
                InstallTargetStatus {
                    family: LocalTargetFamily::ClaudeCodeLocal,
                    state: "adapter-engine-ready",
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

fn install_claude_status(environment: &CommandEnvironment) -> CliOutput {
    let Some(home) = environment.platform_home.as_deref() else {
        return CliOutput::operation_failure("claude-home-unavailable");
    };
    let claude_config_root = environment
        .claude_config_root
        .clone()
        .unwrap_or_else(|| home.join(".claude"));
    let target = match discover_claude_user_with_config(home, &claude_config_root) {
        Ok(target) => target,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    json_output(
        &InstallClaudeStatusOutput {
            schema_version: OUTPUT_SCHEMA_VERSION,
            command: "install-claude-status",
            target: target.summary(),
            launch_grant: "unavailable",
            preview: "unavailable",
            apply: "unavailable",
            activation: "reload-or-client-action-required",
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
    Ok(GlobalSettingsStore::new(config_root(environment)?))
}

pub(crate) fn config_root(environment: &CommandEnvironment) -> Result<ConfigRoot, ConfigError> {
    let home = environment
        .platform_home
        .as_deref()
        .ok_or(ConfigError::HomeUnavailable)?;
    resolve_config_root(environment.configured_root.as_deref(), home)
}

fn update_store(environment: &CommandEnvironment) -> Result<UpdateStateStore, ConfigError> {
    Ok(UpdateStateStore::new(
        config_root(environment)?,
        default_update_stream(),
    ))
}

fn default_update_stream() -> UpdateStreamPreference {
    let version = env!("CARGO_PKG_VERSION");
    if version.contains("-alpha.") || version.contains("-beta.") {
        UpdateStreamPreference::Beta
    } else {
        UpdateStreamPreference::Stable
    }
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
struct UiStartupCheckOutput {
    schema_version: u32,
    command: &'static str,
    product_version: &'static str,
    current_target: InstallBuildTarget,
    service: &'static str,
    snapshot: &'static str,
    app_state: &'static str,
    update_surface: &'static str,
    window_entrypoint: &'static str,
    window: &'static str,
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
    release_authority: &'static str,
    source_commit: &'static str,
    candidate: &'static str,
    launch_grant: &'static str,
    preview: &'static str,
    apply: &'static str,
    verify: &'static str,
    remove: &'static str,
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
struct InstallClaudeStatusOutput<'a> {
    schema_version: u32,
    command: &'static str,
    target: &'a ClaudeDiscoverySummaryV1,
    launch_grant: &'static str,
    preview: &'static str,
    apply: &'static str,
    activation: &'static str,
}

#[derive(Serialize)]
struct InstallContractVersions {
    artifact_identity: u32,
    launch_grant: u32,
    release_authority: u32,
    release_envelope: u32,
    release_candidate: u32,
    install_plan: u32,
    install_receipt: u32,
    native_payload_install_receipt: u32,
    codex_adapter: u32,
    codex_registration_receipt: u32,
    codex_registration_state: u32,
    claude_adapter: u32,
    claude_registration_receipt: u32,
    claude_registration_state: u32,
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
        assert_eq!(parse_args(args(&[])), Ok(Command::Ui));
        assert_eq!(parse_args(args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse_args(args(&["--version"])), Ok(Command::Version));
        assert_eq!(parse_args(args(&["ui"])), Ok(Command::Ui));
        assert_eq!(
            parse_args(args(&["ui", "--startup-check"])),
            Ok(Command::UiStartupCheck)
        );
        assert_eq!(
            parse_args(args(&[
                "ui",
                "--candidate",
                "/approved/qiongli.candidate.json",
                "--archive",
                "/approved/qiongli.zip",
                "--release-notes",
                "/approved/qiongli.release-notes.md",
                "--target",
                "codex",
            ])),
            Ok(Command::UiCandidate(CandidateReleaseOptions {
                candidate: PathBuf::from("/approved/qiongli.candidate.json"),
                archive: PathBuf::from("/approved/qiongli.zip"),
                release_notes: PathBuf::from("/approved/qiongli.release-notes.md"),
                target: ClientActivationTarget::Codex,
            }))
        );
        assert_eq!(
            parse_args(args(&["content", "list"])),
            Ok(Command::ContentList)
        );
        assert_eq!(
            parse_args(args(&["config", "show"])),
            Ok(Command::ConfigShow)
        );
        assert_eq!(
            parse_args(args(&["update", "status"])),
            Ok(Command::Update(UpdateCliCommand::Status))
        );
        assert_eq!(
            parse_args(args(&["update", "check"])),
            Ok(Command::Update(UpdateCliCommand::Check))
        );
        assert_eq!(
            parse_args(args(&["update", "download", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Download {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "verify", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Verify {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "stage", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Stage {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "install", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Install {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&[
                "update",
                "health",
                "--transaction-id",
                "update-0123456789abcdef0123456789abcdef",
            ])),
            Ok(Command::Update(UpdateCliCommand::Health {
                transaction_id: "update-0123456789abcdef0123456789abcdef".to_string(),
            }))
        );
        assert_eq!(
            parse_args(args(&["update", "cancel", "--expected-revision", "3",])),
            Ok(Command::Update(UpdateCliCommand::Cancel {
                expected_revision: 3,
            }))
        );
        assert_eq!(
            parse_args(args(&[
                "update",
                "channel",
                "--expected-revision",
                "3",
                "--stream",
                "stable",
            ])),
            Ok(Command::Update(UpdateCliCommand::Channel {
                expected_revision: 3,
                stream: UpdateStreamPreference::Stable,
            }))
        );
        assert_eq!(parse_args(args(&["status"])), Ok(Command::Status));
        assert_eq!(parse_args(args(&["doctor"])), Ok(Command::Doctor));
        assert_eq!(
            parse_args(args(&["install", "codex", "status"])),
            Ok(Command::InstallCodexStatus)
        );
        assert_eq!(
            parse_args(args(&["install", "claude", "status"])),
            Ok(Command::InstallClaudeStatus)
        );
        assert_eq!(
            parse_args(args(&[
                "install",
                "native",
                "preview",
                "--release",
                "/approved/release.json",
                "--archive",
                "/approved/archive.zip",
                "--managed-root",
                "/approved/managed",
                "--target",
                "codex",
            ])),
            Ok(Command::InstallNative(NativeCliCommand::Preview(
                NativeReleaseOptions {
                    release: PathBuf::from("/approved/release.json"),
                    archive: PathBuf::from("/approved/archive.zip"),
                    managed_root: PathBuf::from("/approved/managed"),
                    target: NativeClientTarget::Codex,
                }
            )))
        );
        assert_eq!(
            parse_args(args(&[
                "install",
                "candidate",
                "preview",
                "--candidate",
                "/approved/qiongli.candidate.json",
                "--archive",
                "/approved/qiongli.zip",
                "--release-notes",
                "/approved/qiongli.release-notes.md",
                "--target",
                "claude",
            ])),
            Ok(Command::InstallCandidate(CandidateCliCommand::Preview(
                CandidateReleaseOptions {
                    candidate: PathBuf::from("/approved/qiongli.candidate.json"),
                    archive: PathBuf::from("/approved/qiongli.zip"),
                    release_notes: PathBuf::from("/approved/qiongli.release-notes.md"),
                    target: ClientActivationTarget::ClaudeCode,
                }
            )))
        );
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

        let digest = "a".repeat(64);
        let first = parse_args(args(&[
            "install",
            "candidate",
            "apply",
            "--candidate",
            "/approved/qiongli.candidate.json",
            "--archive",
            "/approved/qiongli.zip",
            "--release-notes",
            "/approved/qiongli.release-notes.md",
            "--target",
            "codex",
            "--expected-approval-digest",
            &digest,
            "--approve-filesystem-write",
            "--approve-client-config-change",
            "--approve-host-trust",
        ]));
        let second = parse_args(args(&[
            "install",
            "candidate",
            "apply",
            "--approve-host-trust",
            "--target",
            "codex",
            "--release-notes",
            "/approved/qiongli.release-notes.md",
            "--approve-filesystem-write",
            "--expected-approval-digest",
            &digest,
            "--archive",
            "/approved/qiongli.zip",
            "--approve-client-config-change",
            "--candidate",
            "/approved/qiongli.candidate.json",
        ]));
        assert_eq!(first, second);

        let first = parse_args(args(&[
            "install",
            "native",
            "apply",
            "--release",
            "/approved/release.json",
            "--archive",
            "/approved/archive.zip",
            "--managed-root",
            "/approved/managed",
            "--target",
            "claude",
            "--expected-plan-digest",
            &digest,
            "--approve-filesystem-write",
        ]));
        let second = parse_args(args(&[
            "install",
            "native",
            "apply",
            "--approve-filesystem-write",
            "--target",
            "claude",
            "--managed-root",
            "/approved/managed",
            "--expected-plan-digest",
            &digest,
            "--archive",
            "/approved/archive.zip",
            "--release",
            "/approved/release.json",
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
    fn library_cli_boundary_does_not_launch_product_entrypoint_modes() {
        let content = crate::embedded_content().unwrap();
        let environment = CommandEnvironment::default();
        let no_args = run_cli(args(&[]), &environment, &content);
        assert_eq!(no_args.exit_code(), 1);
        assert_eq!(
            no_args.stderr(),
            "error: desktop-command-requires-product-entrypoint\n"
        );

        let ui = run_cli(args(&["ui"]), &environment, &content);
        assert_eq!(ui.exit_code(), 1);
        assert_eq!(
            ui.stderr(),
            "error: desktop-command-requires-product-entrypoint\n"
        );

        let startup_check = run_cli(args(&["ui", "--startup-check"]), &environment, &content);
        assert_eq!(startup_check.exit_code(), 0);
        assert!(startup_check.stderr().is_empty());
        let startup_check: serde_json::Value =
            serde_json::from_str(startup_check.stdout()).unwrap();
        assert_eq!(startup_check["command"], "ui-startup-check");
        assert_eq!(startup_check["service"], "ready");
        assert_eq!(startup_check["snapshot"], "ready");
        assert_eq!(startup_check["app_state"], "ready");
        assert_eq!(startup_check["update_surface"], "ready");
        assert_eq!(startup_check["window_entrypoint"], "available");
        assert_eq!(startup_check["window"], "not-opened");

        let mcp = run_cli(
            args(&["mcp", "serve", "--profile", "lite", "--transport", "stdio"]),
            &environment,
            &content,
        );
        assert_eq!(mcp.exit_code(), 1);
        assert_eq!(
            mcp.stderr(),
            "error: streaming-command-requires-product-entrypoint\n"
        );
    }

    #[test]
    fn parser_rejects_missing_duplicate_unknown_and_private_values() {
        for values in [
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
            vec!["update"],
            vec!["update", "status", "extra"],
            vec!["update", "channel", "--expected-revision", "0"],
            vec!["update", "download"],
            vec!["update", "verify"],
            vec!["update", "cancel", "--expected-revision", "not-a-number"],
            vec![
                "update",
                "channel",
                "--expected-revision",
                "0",
                "--stream",
                "alpha",
            ],
            vec!["update", "check", "--url", "https://private-canary"],
            vec!["status", "extra"],
            vec!["mcp"],
            vec!["mcp", "serve", "--profile", "lite"],
            vec!["mcp", "serve", "--transport", "stdio"],
            vec!["mcp", "serve", "--profile", "full", "--transport", "stdio"],
            vec![
                "install",
                "native",
                "apply",
                "--release",
                "/approved/release.json",
                "--archive",
                "/approved/archive.zip",
                "--managed-root",
                "/approved/managed",
                "--target",
                "codex",
                "--expected-plan-digest",
                "a",
                "--approve-filesystem-write",
            ],
            vec![
                "install",
                "candidate",
                "apply",
                "--candidate",
                "/approved/qiongli.candidate.json",
                "--archive",
                "/approved/qiongli.zip",
                "--release-notes",
                "/approved/qiongli.release-notes.md",
                "--target",
                "codex",
                "--expected-approval-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write",
                "--approve-client-config-change",
            ],
            vec![
                "install",
                "candidate",
                "remove",
                "--target",
                "codex",
                "--install-id",
                "native-payload-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write",
            ],
            vec![
                "install",
                "native",
                "remove",
                "--managed-root",
                "/approved/managed",
                "--install-id",
                "native-payload-invalid",
                "--approve-filesystem-write",
            ],
            vec![
                "install",
                "native",
                "remove",
                "--managed-root",
                "/approved/managed",
                "--install-id",
                "native-payload-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ],
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
