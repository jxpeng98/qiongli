# Upgrade and rollback

[Back up projects and settings](data-lifecycle.md#backup-and-restore) before updating the CLI and Plugin.

## Update the CLI

Use your original installation channel; choose the matching command:

```sh
npm install --global qiongli@latest
python -m pip install --upgrade "qiongli==2.2.0"
cargo install qiongli --version 2.2.0 --locked
```

For direct downloads, get the new version from the [installation page](install.md), verify its checksum and extract into a new directory. Then switch PATH or the executable path. Keep the old program and backup.

Check `qiongli --version` for 2.2.0. `qiongli upgrade cli` displays update instructions without running a package manager.

## Refresh the Plugin

```sh
qiongli install plugin
qiongli mcp check
```

Choose the Host, reuse its registered directory, and review and confirm the file and registration changes. `qiongli upgrade plugin` uses the same flow. Open a new Host session, check the tools and call `qiongli_config_status`.

DeepSeek uses its own plugin manager; the installer selects the npm version matching the CLI. For direct DSH installation, see [DeepSeek setup](../advanced/plugin-installation.md#deepseek).

## Move from 1.x

1. Back up projects, configuration and the old version. Try a project copy first.
2. Follow the [installation guide](install.md) for 2.2.0. Run `qiongli setup` to review duplicate CLIs; it gives manual guidance without moving or deleting files.
3. Run `qiongli install plugin` to connect your Host, keeping your model settings. Codex can migrate known old Plugins after confirmation; conflicting Claude Plugins need manual disabling.
4. Open a new session, check the tools and use [2.x commands](cli-2x.md). Old commands such as `project init`, `provider setup` and `check` remain in the [1.x reference](../reference/cli.md).

Existing project formats still require their supported migration flow; installing a new CLI does not migrate projects automatically. See [what changed in 2.x](whats-new-2.md).

## Roll back

Reinstall the original version, then export and enable its matching Plugin. The CLI and Plugin carry separate executables; check both.

**Projects using newer save features may need the newer CLI.** Version 2.2.0 reads old receipts, but older programs reject receipts with new saved-artifact types. Keep using the newer CLI for those projects, or restore a pre-upgrade backup into a separate directory. Do not delete or rewrite receipts to bypass the checks.
