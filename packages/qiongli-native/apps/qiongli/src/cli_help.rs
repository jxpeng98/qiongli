//! Short entry pages; detailed syntax comes from the existing command owners.
use std::ffi::OsString;

use crate::{command, project_cli};

pub(crate) const CONTENT_INSTALL: &str = "Install or update Qiongli content\n\nUsage:\n  qiongli install\n  qiongli install plugin [--target <codex|claude|all>] [--destination <absolute-path/qiongli-next>]\n  qiongli install skills [--preset <qiongli-managed|current-project>] [--profile <skill-only|lite|full>]\n  qiongli install list\n\nIn a terminal, install opens the guide. install --interactive is also supported.\nPlugin: choose a Host once; later installs reuse its registered source directory.\nExisting verified files are updated. upgrade/update plugin are aliases of this\nsame flow. --target all handles Codex and Claude separately; omit --destination.\nA new source needs an absolute path ending in qiongli-next with an existing,\nsecure parent. Each Host needs its own directory. Unknown or changed files refuse.\n\nThe Host loads Skills and Full MCP from its registered Plugin cache. The source\nfolder is retained for updates; nothing needs copying to ~/.agents/skills.\nAnother enabled Qiongli Plugin is reported before export. Disable it in the Host\nand retry; this command never uninstalls or archives another Plugin.\n\nConfirm file changes and Host registration separately. Enter cancels.\nSuccess reports the registered source, Host cache and loaded Skill location.\nRun qiongli doctor or qiongli install list to verify registration, then start a\nnew Host session to check actual tools. A local mcp check does not attest it.\n\nSkills exports update .qiongli-skills under home (default) or the current project.\nThey export guidance only; they do not register a Host or connect MCP.\nupgrade/update skills use the same operation.\n\nScripts: add --dry-run to emit the unchanged canonical file plan. Plugin previews\nrequire an explicit single --target and --destination. Review it before app apply.\nBare install with redirected output or --json/--text retains its inventory.\n\nUpdate the CLI through its original installation channel first. This command\ninstalls embedded content; it does not download a newer CLI.\n";

pub(crate) const UPGRADE: &str = "Upgrade Qiongli\n\n  upgrade plugin         Refresh bundled files and official Host registration\n  upgrade skills         Refresh standalone Skills in the selected preset\n  upgrade cli            Show CLI package upgrade commands\n\nIn a terminal, bare upgrade opens the installation guide.\nUse qiongli install plugin to install or update; Host and path are discovered.\nupgrade/update plugin and skills remain aliases. Bare update and update status keep their\nmanaged-installation meaning; registry packages use their own package manager.\n";

pub(crate) const CLI_UPGRADE: &str = "Upgrade the CLI using its original installation channel:\n\n  npm (2.x prerelease):   npm install --global qiongli@next\n  pip (same environment): python -m pip install --upgrade --pre qiongli\n  Cargo:                 cargo install qiongli --version <2.x-version> --locked --force\n  GitHub Release:        Download/extract the new archive and switch PATH to it\n\nUse qiongli install list --paths exact to check which executable is active.\nThen rerun install plugin or install skills to refresh the bundled content.\nNo package manager has been invoked by this command.\n";

pub(crate) const HOME: &str = "Qiongli — research from your terminal\n\nUsage: qiongli [command] [--json | --text]\n       ql is the same command\n\nStart here:\n  status                 Show version, content and configuration status\n  doctor                 Check the installation and explain problems\n  install                Install or update Plugin/Skills with guided confirmation\n  setup                  Review installed CLI versions interactively\n\nResearch and configuration:\n  project                List your registered research projects\n  project show <id>       Show one project\n  content                List embedded research content profiles\n  config                 Show current configuration\n  install                Install Plugin/Skills; list existing CLI versions\n  paths                  Show resolved file locations\n  mcp                    Show how to connect an AI Host\n\nMore commands:\n  app                    Manage Plugin, Skills and CLI installation plans\n  upgrade                Refresh Plugin/Skills or show CLI upgrade steps\n  update                 Managed status; update plugin/skills aliases\n  migrate-1x             Preview and migrate legacy configuration\n\nHelp and output:\n  help <command>         Show help, e.g. qiongli help project create\n  help all               Show the complete command reference\n  -h, --help             Show help for a command\n  -V, --version          Show the installed version\n  --json                 Keep structured output for scripts\n  --text                 Show readable output even when redirected\n\nTerminal queries show summaries; redirected queries retain their existing output.\nProject changes still require preview and explicit approval.\n";

const PROJECT: &str = "Research projects\n\nUsage: qiongli project [command]\n\n  list, ls               List registered projects (default)\n  show <id>              Show one project\n  create                 Preview creating a project, then apply it\n  register               Preview registering an existing project\n  doctor                 Check project health and repair options\n  graph                  Inspect source-linked research relationships\n  capture                Review research notes and capture proposals\n  portfolio              Query research across projects\n  import, export         Preview and transfer portable projects\n  migrate                Copy legacy research into a new project\n  archive, restore       Change a project's library lifecycle\n  refresh, unregister    Refresh or remove a library registration\n\nExample: qiongli project create --help\nWrites retain their preview, digest and approval requirements.\n";

const INSTALL: &str = "Installed CLI versions and Hosts\n\nUsage: qiongli install [command]\n\n  --interactive          Guided Plugin, Skills or MCP connection setup\n  list, inventory        List detected CLI installations and Hosts (default)\n  review                 Review versions and manual cleanup choices\n  codex status           Inspect the local Codex integration\n  claude status          Inspect the local Claude integration\n  status                 Inspect signed payload capabilities\n\nExamples:\n  qiongli install list --paths exact\n  qiongli setup\n\nReview never deletes, moves or archives files or changes PATH.\nAdvanced signed payload tools: qiongli install candidate --help\n                              qiongli install native --help\n";

const APP: &str = "Managed Plugin, Skills and CLI operations\n\nUsage: qiongli app <command>\n\n  snapshot               Inspect managed components\n  verify-integrations    Check Host integration readiness\n  verify-skills          Check managed Skills\n  plugin-source-status   Inspect a local Plugin source\n  plan                   Preview an installation, update or removal\n  apply                  Apply an explicitly approved, digest-bound plan\n  read-project-artifact  Read a revision-bound research artifact\n\nExample: qiongli app plan --help\nThese commands run without opening an App window.\n";

const SETUP: &str = "Review installed CLI versions\n\nUsage: qiongli setup\n       qiongli install review\n\nChoose a preferred installation and review manual archive/uninstall guidance.\nPress Enter to keep the current setup. Requires an interactive terminal.\nNo files or settings are changed. Host registration remains a separate operation.\nFor a non-interactive inventory: qiongli install list --paths exact\n";

fn reference() -> String {
    [
        command::APP_USAGE,
        command::CONTENT_USAGE,
        command::CONFIG_USAGE,
        command::UPDATE_USAGE,
        command::MCP_USAGE,
        command::INSTALL_USAGE,
        CONTENT_INSTALL,
        command::MIGRATION_USAGE,
        project_cli::PROJECT_USAGE,
        project_cli::GRAPH_NEIGHBOURHOOD_USAGE,
        crate::portfolio_cli::USAGE,
        crate::capture_cli::CAPTURE_USAGE,
        crate::capture_delivery_cli::USAGE,
        crate::capture_assignment_cli::USAGE,
        crate::capture_resolution_cli::USAGE,
        crate::repository_capture_cli::USAGE,
    ]
    .join("\n")
}

pub(crate) fn topic(args: &[OsString]) -> Option<String> {
    let words = args
        .iter()
        .map(|arg| arg.to_str())
        .collect::<Option<Vec<_>>>()?;
    let page = match words.as_slice() {
        [] => HOME,
        ["all"] => return Some(format!("{HOME}\n{}", reference())),
        ["project"] => PROJECT,
        ["install"] => {
            return Some(format!(
                "{INSTALL}\nInstall bundled Plugin/Skills: qiongli install plugin --help\nRefresh bundled content: qiongli upgrade --help\n"
            ));
        }
        ["install" | "upgrade" | "update", "plugin" | "skills"] => CONTENT_INSTALL,
        ["upgrade"] => UPGRADE,
        ["upgrade" | "update", "cli"] => CLI_UPGRADE,
        ["app"] => APP,
        ["setup"] | ["install", "review"] => SETUP,
        ["install", "list"] | ["install", "inventory"] => {
            "Usage: qiongli install list [--paths exact] [--json | --text]\n       qiongli install inventory [--paths exact] [--json | --text]\n\nShow detected CLI versions and Hosts. Use --paths exact to show actual locations.\n"
        }
        ["project", "ls"] | ["project", "list"] => {
            "Usage: qiongli project [list | ls] [--json | --text]\n\nList registered research projects.\n"
        }
        ["project", "show"] => {
            "Usage: qiongli project show <id>\n       qiongli project show --project-id <id>\n\nRead a registered project. Get its ID from qiongli project.\n"
        }
        ["status"] => {
            "Usage: qiongli status [--json | --text]\n\nShow the running version, embedded content and redacted configuration status.\n"
        }
        ["doctor"] => {
            "Usage: qiongli doctor [--paths exact] [--json | --text]\n\nShow health checks and suggested next steps. Paths are redacted by default.\n"
        }
        ["paths"] => {
            "Usage: qiongli paths [--json | --text]\n\nShow exact resolved paths. Review before sharing this output.\n"
        }
        ["content"] => command::CONTENT_USAGE,
        ["config"] => command::CONFIG_USAGE,
        ["update"] => return Some(format!("{UPGRADE}\n{}", command::UPDATE_USAGE)),
        ["mcp"] | ["mcp", "check"] => command::MCP_USAGE,
        ["migrate-1x"] => command::MIGRATION_USAGE,
        _ => "",
    };
    if !page.is_empty() {
        return Some(page.to_owned());
    }
    if words.iter().any(|word| word.starts_with('-')) {
        return None;
    }
    let reference = reference();
    let mut lines = Vec::new();
    for line in reference
        .lines()
        .filter(|line| line.starts_with("  qiongli "))
    {
        let syntax = line.split_whitespace().skip(1).collect::<Vec<_>>();
        if words.iter().enumerate().all(|(index, word)| {
            syntax.get(index).is_some_and(|token| {
                token == word
                    || token
                        .trim_matches(['<', '>'])
                        .split('|')
                        .any(|option| option == *word)
            })
        }) && !line.ends_with("--help")
            && !lines.contains(&line)
        {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        return None;
    }
    // Graph query help also needs the filter names and limits.
    let mut output = format!("Usage:\n{}\n", lines.join("\n"));
    if words.starts_with(&["project", "graph"]) {
        output.push_str("\nGraph filters:\n  --focus-node-id <nod_id> --direction <incoming|outgoing|both>\n  --max-depth <1..3> (requires --focus-node-id)\n  --node-type <type> --relation <relation> --layer <layer>\n  --canonical-id <id> --text <text> --max-nodes <1..256> --max-edges <1..512>\n");
    }
    output.push_str("\nUse --json for complete structured results. Preview and approval requirements still apply.\n");
    Some(output)
}
