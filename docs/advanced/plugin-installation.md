# Plugin setup

For a first installation, run `qiongli install` and follow the [quickstart](../quickstart.md). This page covers options and custom exports.

## Choose Hosts

```sh
qiongli install all
qiongli install plugin --target codex
qiongli install plugin --target claude
qiongli install plugin --target deepseek
qiongli install plugin --target antigravity
qiongli install plugin --target pi
qiongli install plugin --target codex,deepseek
```

**Qiongli 2.5.0:** `qiongli install all`, main guide choice **5**, and
Plugin Host choice **6 / all** install Plugins for every detected supported Host
CLI (`codex`, `claude`, `dsh`, `agy`, `pi`). Each Plugin includes Skills and Full MCP;
there is no separate component install. The guide lists selected Hosts and missing
CLIs it skips. If none are found, it stops without installing anything. It does
not install the Host applications. `upgrade all` and `update all` refresh the
same detected set; `--language auto|zh|en` chooses descriptions once for the batch.

**Development branch (after 2.5.0):** Plugin commands without `--target` select
detected Hosts automatically; Enter in the main guide also selects all. Explicit
destination/Hook overrides without a target retain the Host menu. Detection uses
PATH and existing common CLI installation directories; an application or config
directory alone is insufficient. `--target all` uses the same selection. An
explicit list such as `--target codex,claude` still requires both clients. Each
selected Host retains its own required file and registration approvals. A failure
or explicit decline is recorded and later Hosts continue; closed/incomplete input
or output failure stops the batch. Completed installations remain. Source directories are selected without a path prompt:

- Codex/Claude reuse registered paths or that Host's verified legacy home/qiongli
  export. Fresh installs use `~/qiongli-codex` and `~/qiongli-claude`; prereleases
  use `qiongli-next-*`.
- AGY recovers the original source from a verified cache, otherwise using
  `~/qiongli-antigravity`.
- Pi reuses its registered local package, otherwise using `~/qiongli-pi`.
- DeepSeek retains its profile choice and official package-manager flow.

All-detected mode preserves existing Hook settings (off for new sources).
Language and file/registration approvals remain explicit. Unknown, changed or
unsafe default paths refuse; they are not overwritten or replaced by numbered
fallback directories.

`--destination` selects one Host's directory with
an existing parent. Codex/Claude accept historical `qiongli`/`qiongli-next`
basenames and their matching Host-scoped defaults. The all-Host preset does not
accept `--destination`, `--hooks`
or `--dry-run`; use an explicit compatible Host selection for those options.

File export and official Host registration have separate confirmations. Cancelling registration keeps exported files and leaves Host settings unchanged. Retry for the same Host after resolving a failure. Keep old sources and caches for recovery. Known Codex conflicts can be disabled through a confirmed migration; Claude conflicts need manual disabling.

## Progress and failed installations

On the development branch, the installer shows `[1/5]` Host progress, official
manager command steps and elapsed time every five seconds while a command runs.
The final summary separates installed, failed, skipped and not-run Hosts. A
batch containing a failure exits nonzero even when other Hosts succeed. Replying
`n` skips that Host; each later Host still requires its own approval. Close input
or interrupt the command to stop the batch. Exported files are retained when
registration fails or is declined.

Retry only the unfinished client using the command printed in the summary, for example:

```sh
qiongli install plugin --target deepseek
```

`host-command-nonzero-exit` means the official manager failed, not a specific
network/version/permission diagnosis. New diagnostics include its exit code,
recognized error category when present and the exact command. For DSH this
preserves the selected profile, pinned package version, registry and flags.
Check the reported issue and retained files first; choose the same profile when
retrying. Run the printed DSH command directly if its unrecognized error needs
full diagnosis. Raw manager output is neither echoed nor saved by Qiongli because
it may include credentials or configuration. Do not post secrets from direct output.

A timeout does not undo changes and may leave package-manager descendants running.
Check their state before retrying. Unknown files, changed receipts and profile
conflicts remain refusals for that Host; continuation does not bypass them.

## Description language {#language}

```sh
qiongli install plugin --target codex --language zh
qiongli install plugin --target claude --language en
```

Auto follows the environment/system locale and falls back to English. Updates preserve a saved choice unless you supply another. Only descriptions and supported display metadata change; entry names and workflow bodies stay the same. Refresh the Host or open a new session after changing language.

## Optional context hooks

Hooks are off on first installation. Updates preserve the selection.

```sh
qiongli install plugin --target codex --hooks context
qiongli install plugin --target codex --hooks off
```

The preview shows commands. Host trust and actual event delivery need their own checks. See [hook setup and verification](agent-skill-collaboration.md#optional-context-hooks).

## DeepSeek Harness {#deepseek}

`qiongli install plugin --target deepseek` selects a profile, previews official DSH commands and asks for trust before installing the npm version matching your CLI. It verifies registration and the content receipt. It does not select a model.

To install without a global Qiongli CLI, use DeepSeek Desktop's **Add plugin → Official npm registry**, entering `qiongli@2.5.0`, or run:

```sh
dsh plugin --profile desktop add qiongli@2.5.0
```

Replace `desktop` with your profile. The bundle includes 22 Skill entries, Full MCP and the platform executable. Keep one Qiongli bundle per profile, reload it and check the actual tools.

The installer prefers an existing Desktop profile, otherwise `web`. New CLI profiles use the official `web` template; Desktop initializes its reserved profile itself. DSH uses its package manager and profile, so `--destination` and context `--hooks` require separate Codex/Claude selections.

Use the installer or manager to update an exact version. In the Desktop dialog, follow its remove/add instructions while retaining your profile and model settings. For developer bundle exports and external proposals, see [external Agent coordination](external-host-coordination.md).

## Pi coding agent {#pi}

**Qiongli 2.5.0:** experimental Pi support requires the official `pi`
CLI 0.99.0 or newer with built-in MCP enabled. Use the resolved official executable
on PATH; Pi's mise shim is not supported by this adapter. Select **7** in the Host menu,
or run:

```sh
qiongli install plugin --target pi --language en
qiongli update plugin --target pi
```

Pi is also detected by `qiongli install all`. The package exposes two Skills,
`/skill:qiongli` and `/skill:no-qiongli`; all research workflows remain inside
the shared library. A dependency-free package extension registers native Full
MCP with Pi, using deferred tool discovery. No third-party MCP bridge is needed.

The default source is `~/qiongli-pi`. A single-Host `--destination` can select a
secure absolute path ending in `qiongli`, `qiongli-next` or `qiongli-pi`. Updates
reuse the verified source in Pi's user settings, including relative declarations.
The installer honors `PI_CODING_AGENT_DIR` (default `~/.pi/agent`), previews source
changes and separately confirms the official `pi install <source>` command when
registration is needed. Model settings and unrelated packages are preserved.
On Unix, new profile files use private permissions. Existing linked or group/world
writable profiles refuse; the installer does not change their permissions.
Filtered package resources, disabled built-in MCP, duplicate Qiongli packages or
standalone Qiongli MCP entries require review; they are not silently reset.

Keep the source directory: Pi loads it directly. Start a new session or `/reload`,
use `/mcp` to inspect the connection, then ask Pi to discover and call
`qiongli_config_status`. **Shell `pi mcp list` does not load package extensions.**
Project settings, disabled resources or a replacement MCP extension can affect
session availability. Registration alone does not establish a working model
session. Pi context hooks, signed Host integration and App `--dry-run` are outside
this adapter. Remove the package with `pi remove <source>` before deleting its
source; removal leaves source files available for recovery.

See Pi's official [packages](https://pi.dev/docs/latest/packages) and
[MCP](https://pi.dev/docs/latest/mcp) documentation.

## Antigravity CLI {#antigravity}

Qiongli 2.4.0 adds experimental Antigravity CLI support; the 2.3.0 binary does
not contain this installer. Codex remains the primary research Host. Scoped
AGY 1.3.0 observations cover installation, a normally completed status call and
document-body recovery with supplied tool schemas. Automatic discovery, complete
research-session recovery and original MCP response capture remain unqualified.
The experimental label retains those failed checks for later promotion; it does
not establish a passed academic workflow or IDE support.

```sh
qiongli install plugin --target antigravity --language en
# agy is also accepted as the target name
qiongli update plugin --target agy
```

Install `agy` 1.2.17 or newer first. The terminal guide separately confirms the
local export and `agy plugin install` / `enable`. Its default source directory is
`~/qiongli-antigravity`; `--destination` can select an absolute secure directory
ending in `qiongli`, `qiongli-next` or `qiongli-antigravity`. On the development
branch, updates can omit the destination to recover it from the verified cache.
The export includes the native binary and Full MCP,
using Antigravity's root `plugin.json` and `mcp_config.json`.

**Qiongli 2.5.0:** the AGY Skill list has one research entry, `qiongli`,
plus the independent `no-qiongli` reply-only entry. Reading, literature review,
writing and other workflows remain internal resources selected by your request;
you can still name a workflow in the prompt. The 2.4.0 export has 22 public Skills.
Once the CLI update is available, upgrade the CLI, then run the Plugin update
above and reopen the AGY session. The verified update replaces the old wrapper
entries through the official manager. Do not delete cached Skills manually;
modified or unknown files block receipt verification and need separate review.

**Keep the exported source directory.** MCP starts its binary by absolute path;
no Plugin-root variable or working-directory expansion is assumed. In the
observed AGY 1.2.17, the official manager copies the Plugin to
`~/.gemini/config/plugins/<name>`. The installer verifies that entire cache
against the export receipt and checks registration and enablement separately.
Unknown/modified caches, another enabled Qiongli Plugin and an existing
standalone Qiongli MCP entry stop installation for review. The manager owns
configuration writes; Qiongli does not select a model or migrate credentials.

Complete AGY's normal workspace trust review, then start a fresh session and
check `/mcp` for the Qiongli server and tools. Invoke the `qiongli` Skill, then ask it to call
`qiongli_config_status` and list the available Qiongli tools. Registration alone
does not prove those session behaviors. Review Plugin state with `agy plugin
list`; imported MCP definitions need not appear in standalone `agy mcp list`.
Use an interactive session to review and approve the requested MCP call. In the
observed headless `--print` session, an unapproved call was denied even though
AGY exited with status 0; inspect the actual tool result and `denied_actions`.
To remove it, run `agy plugin uninstall qiongli` (or `qiongli-next` for a
prerelease) before removing its retained source. Context hooks, managed signed
integration, `app plan` and `--dry-run` for this Host remain outside this adapter.

The native layout follows the official [Plugin format](https://www.antigravity.google/docs/plugins?tab=cli)
and [MCP configuration](https://antigravity.google/docs/mcp). IDE installation and
other operating systems need separate qualification.

## Standalone Skills

```sh
qiongli install skills
qiongli upgrade skills --preset current-project --profile full
```

The default full-profile export is `$HOME/.qiongli-skills`; `current-project` uses `.qiongli-skills` in the current directory. Existing profiles remain in force. These are guidance files without automatic Host registration, MCP or hooks.

## Scripted Plugin export {#scripted-export}

For Codex or Claude, inspect a read-only file plan:

```sh
qiongli install plugin --target codex --destination /absolute/qiongli --dry-run --json
```

To apply a reviewed plan, use the lower-level owner. The destination must end in `qiongli` or `qiongli-next`, and its parent must exist and be secure; use a separate directory outside Host caches and `.qiongli` state. On Unix, every parent directory must also disallow group and other writes. An `insecure-materialization-parent` error can therefore come from `/tmp` or a shared workspace ancestor even when the immediate parent is private. Choose a private export location with secure ancestors, then preview again:

```sh
qiongli app plan plugin-source-install --target codex \
  --destination /absolute/qiongli > plugin-plan.json
qiongli app apply --plan plugin-plan.json \
  --expected-plan-digest <plan_digest_sha256> --approve-filesystem-write
qiongli app plugin-source-status --target codex --destination /absolute/qiongli
```

Use a new plan filename to avoid overwriting a previous plan. Review destination, binary hash, receipt and digest before applying. The export does not register the Host; use its official Plugin manager. Stable exports select `qiongli@qiongli-cli-local`; prereleases select `qiongli-next@qiongli-cli-local`.

Use `plugin-source-update` for a matching export and `plugin-source-remove` only after unregistering its Host Plugin. Changed files, unknown files, symlinks and stale receipts refuse replacement/removal. Piped input does not grant approval. Start a new Host session after registration or refresh and check actual tools.
