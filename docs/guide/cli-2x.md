# CLI commands

For Qiongli **2.2.1**. Start with [installation](install.md) or the [quickstart](../quickstart.md).

## Everyday commands

| Task | Command |
|---|---|
| Show version | `qiongli --version` |
| Show common commands | `qiongli` |
| Check local health | `qiongli doctor` |
| Connect or refresh a Plugin | `qiongli install plugin` |
| Check Host registrations | `qiongli install list` |
| Review duplicate CLI installations | `qiongli setup` |
| List research projects | `qiongli project` |
| Read one project | `qiongli project show <id>` |
| Read configuration | `qiongli config` |
| List embedded content | `qiongli content` |
| Check local MCP | `qiongli mcp check` |

Package-manager installations also provide `ql` with the same commands. Direct downloads provide `qiongli`.

Use `qiongli help project create` or `qiongli project create --help` for one operation. `qiongli help all` lists the full reference. Add `--json` for structured output or `--text` for a readable report:

```sh
qiongli status --json
qiongli content --json
qiongli doctor --text
```

## Installation and upgrades

### Download {#standalone-binary-download}

See [platform downloads, checksums and extraction](install.md#standalone-binary-download). The standalone program needs no separate language runtime.

### Package managers {#package-managers}

Choose one channel; see [requirements](install.md#package-managers):

```sh
npm install --global qiongli@latest
python -m pip install --upgrade "qiongli==2.2.1"
cargo install qiongli --version 2.2.1 --locked
```

### Review existing CLI installations {#review-existing-cli-installations}

`qiongli setup` lists visible copies and gives manual archive or uninstall guidance. It does not move files, delete data or change PATH. Run the new executable by its full path if an older copy appears first on PATH. Use `qiongli install inventory --paths exact` to opt into exact paths.

### Install and upgrade bundled content {#install-and-upgrade-bundled-content}

Run `qiongli install plugin` and confirm the file and Host plans separately. Updates reuse the verified source directory. `upgrade plugin` and `update plugin` are aliases. The Plugin contains Skills, its own executable and Full MCP.

`qiongli install skills` exports guidance only. `qiongli upgrade cli` explains package updates without running them. See [Plugin options](../advanced/plugin-installation.md) and [upgrade and rollback](upgrade.md).

### Connect a Host {#connect-a-host}

Open a new Host session after registration, list actual tools and call `qiongli_config_status`. A successful `qiongli mcp check` verifies only this CLI's local protocol. See [MCP connections](../advanced/cross-platform-mcp.md).

### First use {#first-use}

Follow the [quickstart](../quickstart.md). Describe a task naturally or use a visible Skill entry such as `$qiongli-paper-read` in Codex. Workflow names are Host instructions, not shell subcommands.

### Skill description language {#skill-description-language}

The installer offers Auto, 中文 and English. Updates preserve a saved language unless you select another. See [language options](../advanced/plugin-installation.md#language).

### Stable and Next Plugin identities {#stable-and-next-plugin-identities}

Since 2.1, stable Plugins use `qiongli`; Alpha/Beta use `qiongli-next`. The CLI version selects the identity. Existing verified source directories can retain their earlier names. The npm package name is always `qiongli`.

### DeepSeek Harness {#deepseek-harness-installation-21}

Use `qiongli install plugin --target deepseek` or DeepSeek's own manager. See [DeepSeek setup](../advanced/plugin-installation.md#deepseek).

### Export a local Plugin source {#export-a-local-plugin-source}

For scripts and custom registration, use the [preview/apply commands](../advanced/plugin-installation.md#scripted-export). Exported files still need official Host registration.

## Installation state and version identity {#installation-state}

`qiongli install list` and `qiongli doctor` check official Host inventories and source/cache receipts. Unavailable or changed installations remain reported gaps. Matching registration does not prove a live session has loaded tools.

The CLI and each Plugin carry separate executables. Refresh the Plugin after updating the CLI. Compare the CLI version, `qiongli content --json` and the Host inventory; matching version text alone does not prove matching cached files.

## Project changes and advanced commands

Project writes require a preview, approval and current revision. Inspect `qiongli project --help` for create, capture, import, export and other operations. Use [data ownership and backup](data-lifecycle.md) before moving a project.

For a standalone MCP client, launch the absolute executable path with:

```sh
qiongli mcp serve --profile full --transport stdio
```

Lite has 15 tools; Full has 33, including projects, Graph and handoffs. Models and execution remain owned by the Host.

The `app` namespace retains lower-level installation plans. `app apply` requires the plan digest and explicit filesystem approval. Managed-product installation/update commands require their own package authority; registry-installed CLIs update through their package manager. `qiongli update` reports managed-update state.

Old commands such as `project init`, `check` and `provider setup` belong to the [1.x reference](../reference/cli.md).

## Research Graph {#research-graph}

Graph links claims, sources, decisions and locations from saved canonical records. The Host must first read material, propose those records and save them through the approved flow.

```sh
qiongli project graph snapshot --project-id <prj_id> --text
qiongli project graph view --project-id <prj_id> --open
```

`--open` saves a new private offline HTML snapshot and asks the system to open it. Use `--save` to save without opening. If no window appears, open the printed path manually. Existing snapshots and project files stay intact.

Search and filter records, inspect proposed or rejected links, and copy source commands tied to that revision. After sources change, refresh the project and export again. Filters change the display; downloaded JSON still contains the full snapshot. Treat exported research as private when its sources are private.

Graph checks record consistency, source locations and revisions. Missing evidence remains unresolved. The worked [Graph example](../examples/research-graph.md) shows inputs and a reproducible run; Graph does not establish scientific validity or automatically understand arbitrary PDFs.
