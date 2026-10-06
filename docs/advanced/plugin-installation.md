# Plugin setup

For a first installation, run `qiongli install` and follow the [quickstart](../quickstart.md). This page covers options and custom exports.

## Choose Hosts

```sh
qiongli install plugin --target codex
qiongli install plugin --target claude
qiongli install plugin --target deepseek
qiongli install plugin --target antigravity
qiongli install plugin --target codex,deepseek
```

Omit `--target` for a menu; `--target all` processes each Host separately. Cancellation or failure stops later steps and keeps completed installations. Each Host needs its own approval. Codex/Claude use separate source directories; `--destination` selects one Host's directory with an existing parent.

File export and official Host registration have separate confirmations. Cancelling registration keeps exported files and leaves Host settings unchanged. Retry for the same Host after resolving a failure. Keep old sources and caches for recovery. Known Codex conflicts can be disabled through a confirmed migration; Claude conflicts need manual disabling.

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

To install without a global Qiongli CLI, use DeepSeek Desktop's **Add plugin → Official npm registry**, entering `qiongli@2.3.0`, or run:

```sh
dsh plugin --profile desktop add qiongli@2.3.0
```

Replace `desktop` with your profile. The bundle includes 22 Skill entries, Full MCP and the platform executable. Keep one Qiongli bundle per profile, reload it and check the actual tools.

The installer prefers an existing Desktop profile, otherwise `web`. New CLI profiles use the official `web` template; Desktop initializes its reserved profile itself. DSH uses its package manager and profile, so `--destination` and context `--hooks` require separate Codex/Claude selections.

Use the installer or manager to update an exact version. In the Desktop dialog, follow its remove/add instructions while retaining your profile and model settings. For developer bundle exports and external proposals, see [external Agent coordination](external-host-coordination.md).

## Antigravity CLI {#antigravity}

The current development CLI adds Antigravity support; the previously tagged
2.3.0 binary does not contain this installer.

```sh
qiongli install plugin --target antigravity --language en
# agy is also accepted as the target name
qiongli update plugin --target agy
```

Install `agy` 1.2.17 or newer first. The terminal guide separately confirms the
local export and `agy plugin install` / `enable`. Its default source directory is
`~/qiongli-antigravity`; `--destination` can select an absolute secure directory
ending in `qiongli`, `qiongli-next` or `qiongli-antigravity`. Reuse the same custom
destination when updating. The export contains 22 Skill entries, the native
binary and Full MCP. It uses Antigravity's root `plugin.json` and `mcp_config.json`.

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
