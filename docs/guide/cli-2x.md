# Qiongli 2 CLI: installation and command boundaries

Qiongli 2 is CLI-first. GitHub binary archives, npm and PyPI distribute the same
native executable for a given version and target. No Qiongli App is required.


## Installation and command improvements in beta.5

Terminal users can run `qiongli install` or `qiongli upgrade`
to open the guide. `qiongli install plugin` installs or updates; `upgrade plugin`
and `update plugin` use that same flow. The guide selects the Host and reuses its
registered directory. Use `--target codex`, `--target claude` or `--target all` to
skip Host selection. All handles each Host separately, stopping on cancellation
or failure; each new source needs its own directory with an existing parent.
`--destination` remains available for a single Host. Scripts keep the explicit
`--dry-run` plan, and redirected bare install keeps its read-only inventory.

Plugin source files do not belong in `~/.agents/skills`. Official registration
lets the Host load Skills and MCP from its Plugin cache. If another Qiongli Plugin
is enabled, the new flow names it before exporting. Disable it in the Host and
retry; no old Plugin files are removed. Codex's current CLI has no standalone
Plugin-disable command. Use the Plugin control in Codex rather than remove, which
would delete its cache. A completed export alone is not installation success.

Check `qiongli doctor` and `qiongli install list` for verified registration, then
start a new session and check actual tools. Skills-only installation still exports
`.qiongli-skills`; it does not connect a Host. Choose Plugin for automatic Host
registration.

Bare `config backend`, `project graph`, `project capture`, `project portfolio`
and `app plan` now show scoped help in a terminal. Script errors remain unchanged;
project writes still require explicit preview and approval arguments.


## From installation to first use {#first-use}

Run `qiongli install` in a terminal (`--interactive` remains supported). Choose
the recommended Plugin, then Codex or Claude. The default export is `qiongli-next` under your home directory; you can
enter another absolute path with an existing parent. For upgrades, the guide
reuses the verified registered source directory. A second Host needs a separate
destination. Menu choices do not approve writes: review the file plan and Host registration separately.
Choose Skills for guidance files only, or MCP configuration for an existing setup.

| Entry | Contents and connection |
|---|---|
| CLI package | Native program with embedded resources and MCP implementation; no automatic Host setup |
| CLI-installed Plugin | Skills + native program + Full MCP configuration, 32 tools |
| Native Marketplace platform Plugin | Skills + native program + Lite MCP configuration, 14 tools |
| Standalone Skills | Guidance and references; no running MCP or automatic Host registration |

MCP is compiled into `qiongli`; there is no separate server package to install.
The Host starts the stdio process from the Plugin configuration. You normally do
not keep a separate terminal running `mcp serve`. The full profile of a Skills
export selects content; it does not install Full MCP. Use `qiongli install list`
for the read-only inventory. Redirected bare `install` also retains that output.

Check installation in three steps:

1. Read the file and registration results. Declining registration keeps the export
   available for a retry at the same destination.
2. Run `qiongli mcp check` (or add `--profile lite`) for initialization, tool discovery
   and one read-only call in this CLI. This does not verify a Plugin cache, Host
   session or online service.
3. Open a new Host session. Ask it to list actual Qiongli tools and call
   `qiongli_config_status`. Inspect literature provider setup separately with
   `qiongli_literature_status`. Missing tools must remain a reported gap.

Then try a supplied document: read it with source locations, propose canonical
research records, review and save them, and inspect the Graph and a stage summary.
Saving retains the existing approval checks; summaries preserve the originals.

After updating the CLI, use `qiongli doctor` to identify Plugins needing refresh,
then `qiongli install plugin`; choose the Host there or specify it with `--target`.
Each Plugin
keeps its own executable; equal version strings do not replace receipt/hash checks.
Use `qiongli setup` for manual guidance on duplicate CLI installations.

## Everyday commands

Use short commands, focused help and readable terminal output.
`ql` and `qiongli` accept the same commands.

| Task | Short command | Earlier beta.2 equivalent |
|---|---|---|
| See common commands | `qiongli` | `qiongli --help` |
| Check local health | `qiongli doctor` | Same command, JSON output |
| Review installed CLI versions | `qiongli setup` | `qiongli install migrate --interactive` |
| List installations and Hosts | `qiongli install list` | `qiongli install inventory` |
| List research projects | `qiongli project` | `qiongli project list` |
| Read one project | `qiongli project show <id>` | `qiongli project show --project-id <id>` |
| Read configuration | `qiongli config` | `qiongli config show` |
| List embedded content profiles | `qiongli content` | `qiongli content list` |
| Connect Full MCP over stdio | `qiongli mcp serve --profile full` | Add `--transport stdio` |

Use `qiongli help project create` or `qiongli project create --help` to see one
operation's parameters. `qiongli help all` retains the complete reference, including
advanced managed installation commands. `qiongli update` shows managed update
status; npm, pip and Cargo packages are still upgraded with their own package manager.

Queries display readable summaries in a terminal. Redirected output retains its
previous format; use `--json` explicitly for structured data, or `--text` to save a
readable report. Put one output flag before the command or at the end:

```sh
qiongli status --json
qiongli doctor --text > qiongli-doctor.txt
```

The short `setup` entry borrows the familiar 1.x name but specifically reviews CLI
installations. It does not configure a Host, select a model or uninstall anything.
An empty-argument launch shows help instead of starting that review. Project writes
still use the same preview, approval and revision checks. Human-readable previews
retain their full plan values; `--json` is the exact machine-readable representation.

## Standalone binary download

**Recommended for direct use: download, extract, and run. No Python, Node.js,
Rust or package-manager setup is needed.** You can run from the extracted folder;
adding it to PATH is optional.

Download a platform archive from [Release v2.0.0-beta.5](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-beta.5).
It contains `qiongli` (Windows: `qiongli.exe`), `README.md` and `LICENSE`.
The executable embeds the research Skills, templates and Lite/Full MCP resources;
you do not need a separate resource directory or a checkout of this repository.
Configure models, Host applications and online literature services separately.

| Your platform | Complete CLI archive |
|---|---|
| macOS Apple Silicon / ARM64 | [qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip) |
| Linux x64 / glibc 2.35+ | [qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz) |

Windows beta releases statically link the C runtime, so you do not need to install the
Visual C++ runtime separately. Linux uses system libraries, including glibc 2.35+.

Choose these files under **Assets**. GitHub's **Source code** archives require a
build; the `.tgz` npm package, `.whl` Python packages and `qiongli-next-…-plugin-…`
archives serve different installation paths. Intel macOS, Linux ARM and native
Windows ARM builds are not part of this release.

### 1. Verify the download

Download [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/SHA256SUMS)
from the same release. In the directory containing your download, run the command
for your platform and compare its hash with the line for that exact filename in
`SHA256SUMS`. Continue only if they match.

```sh
# macOS
shasum -a 256 qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz
# Linux
sha256sum qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz
```

```powershell
# Windows
Get-FileHash .\qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

### 2. Extract and run

Extract into a new directory, keeping existing installations and research files
intact. On macOS:

```sh
mkdir qiongli-2.0.0-beta.5-macos-arm64
tar -xzf qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz -C qiongli-2.0.0-beta.5-macos-arm64
cd qiongli-2.0.0-beta.5-macos-arm64
./qiongli --version
./qiongli --help
./qiongli content list
```

On Linux:

```sh
mkdir qiongli-2.0.0-beta.5-linux-x64
tar -xzf qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz -C qiongli-2.0.0-beta.5-linux-x64
cd qiongli-2.0.0-beta.5-linux-x64
./qiongli --version
./qiongli --help
./qiongli content list
```

On Windows, open PowerShell in your download directory:

```powershell
Expand-Archive -Path .\qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip -DestinationPath .\qiongli-2.0.0-beta.5-windows-x64
Set-Location .\qiongli-2.0.0-beta.5-windows-x64
.\qiongli.exe --version
.\qiongli.exe --help
.\qiongli.exe content list
```

The version output should be `qiongli 2.0.0-beta.5`. The archive supplies the
`qiongli` executable; npm/PyPI/Cargo additionally provide the `ql` command alias.

### 3. Optional PATH and Host setup

You can keep using the absolute executable path. To run `qiongli` from any
directory, add the extracted directory to your user PATH using your shell profile
or Windows user environment settings, then open a new terminal. Use
`type -a qiongli` on macOS/Linux or `Get-Command qiongli -All` in PowerShell to
check which installation will run. Verify `qiongli --version` again.

[Connect a Host](#connect-a-host) using the absolute executable path for Lite or
Full MCP, or [export a local Plugin source](#export-a-local-plugin-source).
Downloading and running the CLI does not automatically activate a Host Plugin.

To upgrade, extract the new version into a separate directory, test it, then
update PATH or the Host command. Keep the previous binary and research data for
rollback; switching binaries does not reverse data migrations.

## Review existing CLI installations

Beta.2 can show which Qiongli installations are visible and help you review a
migration. Run the newly installed executable by its full path if an older
version appears first on PATH:

```sh
qiongli install inventory --paths exact
qiongli install migrate --interactive
```

The inventory groups aliases and reports package metadata separately from the
running executable's version. Discovery covers PATH, known user prefixes and
configured Cargo/Python/npm locations; it does not search every environment or
resolve shell functions. Unknown executables are never run. `doctor` includes a
redacted overview; exact paths are opt-in.

The interactive review lets you choose a preferred installation and request
archive or uninstall guidance for each remaining copy. Enter keeps the current
setup. Choosing a preferred installation prepares guidance; it does not edit
PATH or Host settings. No file is deleted, moved or archived. Review shared
command files before uninstalling: an old package may own the same entry as a
new one. Research files, configuration and Plugin caches are excluded.

Beta.3 uses `qiongli setup` to open this review and shows help on an empty launch.
Beta.2 opened the review on an empty-argument terminal launch. To open
the review during npm installation, allow Qiongli's script for that invocation and connect it to the
terminal:

```sh
npm install -g qiongli@next --allow-scripts=qiongli --foreground-scripts
```

Recent npm versions warn when this `postinstall` script has not been explicitly
allowed. The warning alone does not mean installation failed or the script was
blocked; strict script policy can make it an error. `--allow-scripts=qiongli`
records permission for this invocation without changing your saved configuration.
`--foreground-scripts` enables interaction when input and output are terminals;
otherwise Qiongli skips the review. See [npm's script settings](https://docs.npmjs.com/cli/v11/commands/npm-install/#allow-scripts).

Disabling install scripts with `--ignore-scripts` still leaves a working CLI.
If Qiongli is already installed, run the newly installed executable's
`setup` command; reinstalling is unnecessary. pip and Cargo users also run
the review after installation. Scripts, help/version commands and MCP do not
prompt. Package-manager installations stay in place for backup; only a confirmed
standalone release bundle is suitable for a separate, checksum-verified copy.


## Package managers

Alternatively, choose one package manager for the command on your PATH:

```sh
npm install --global qiongli@next
```

Or install in a Python virtual environment:

```sh
python -m pip install --upgrade "qiongli==2.0.0b5"
```

Both expose `qiongli` and `ql`. Check both with `--version` before comparing
behavior. Beta.2 targets macOS ARM64, Windows x64, and Linux x64/glibc 2.35+.
npm needs Node 18+; PyPI needs Python 3.9+.

Cargo builds the CLI from source and requires Rust 1.97+ and the target's native
linker. It provides both `qiongli` and `ql`.

```sh
cargo install qiongli --version 2.0.0-beta.5 --locked
```

Use the exact SemVer prerelease version; Cargo has no `next` channel. Choose the
standalone archive if you want to run immediately without a compiler.

If npm reports a successful install but the command is missing, inspect
`npm prefix -g`. On Unix the command directory is `<prefix>/bin`; on Windows
it is `<prefix>`. Compare it with PATH. A malformed npm prefix is a local
configuration problem; reinstalling into the same prefix does not fix PATH.
Use `type -a qiongli ql` on Unix or `Get-Command qiongli,ql -All` in PowerShell
to detect an older binary shadowing the new installation. Review the npm
configuration before changing a prefix shared by other globally installed tools.

## Which surface owns which command?

| Outcome | Native entry | Boundary |
|---|---|---|
| Inspect installation and resolved paths | `qiongli --version`, `paths`, `doctor`, `install status`, `install inventory` | Inspection is not Host activation |
| Register, inspect, refresh, import or export a project | `qiongli project --help` | Project writes retain revision and approval checks |
| Inspect embedded research content | `qiongli content list` | Does not install a Plugin |
| Inspect global configuration | `qiongli config show`, `config backend status` | The Host owns model configuration and execution |
| Start tool transport | `qiongli mcp serve --profile full --transport stdio` | Full exposes project/graph/handoff tools; Lite exposes the bounded literature subset |
| Plan Skills installation or maintenance | `qiongli app plan skills-reconcile --preset qiongli-managed --profile full` | `app` is the retained command namespace; no GUI is needed; apply requires the returned plan/digest and explicit approval |
| Register or repair a managed Plugin | `qiongli app plan integrations-install --target codex` (or `claude`) | Standalone/registry binaries lack the packaged-product authority required by this owner; use the separate user-approved local source export below |
| Research routing, literature review, writing and critique | Host workflow entries such as `/paper`, `/lit-review`, `/paper-read` | Host instructions, not standalone shell subcommands |
| Inspect/update a managed installation | `qiongli update --help`, `migrate-1x --help` | Managed authority remains required; use the package manager to upgrade a registry-installed CLI |

Use `--help` on the owning command group for exact arguments. Legacy commands
such as `check`, `provider setup`, `provider doctor`,
`install --target ... --parts mcp`, and `project init` are not the native 2.x
contract. They remain documented in the explicitly labeled 1.x reference.

## Install and upgrade bundled content

Install bundled Plugin or standalone Skills without an App package. The guide
can choose the Host and directory; for an explicit Codex destination, run:

```sh
mkdir -p "$HOME/qiongli-plugins/codex"
qiongli install plugin --target codex \
  --destination "$HOME/qiongli-plugins/codex/qiongli-next"
```

First confirm the displayed file plan. Then review and confirm the separate Host
plan: Qiongli calls Codex's official Plugin commands to register and enable the
local Plugin. The export contains the native executable, workflow Skills and Full
MCP; it needs no Python, Node or Cargo runtime. Codex itself must be installed.
For Claude Code, use `--target claude` and a separate directory, such as
`$HOME/qiongli-plugins/claude/qiongli-next`, with its parent created first.

Development builds after beta.5 also offer optional context reminders during
Plugin installation. First install defaults to off; updates preserve your choice.
Use `qiongli install plugin --hooks context` to include them or `--hooks off` to
remove their Plugin configuration. The preview shows the exact commands; Host
trust and live delivery remain separate checks. See [Hook setup and verification](/advanced/agent-skill-collaboration#optional-context-hooks).
Standalone Skills exports do not install hooks.

After updating your CLI through its original channel, refresh the registered source:

```sh
qiongli install plugin --target codex
```

The source directory is discovered from the Host. `--target all` handles both Hosts
separately; each new Host needs its own directory. Omit `--destination` with all.

`update plugin` is an alias. Repeating `install plugin` reconciles an existing
verified export. Before replacing an old Plugin cache, Qiongli verifies its
receipt and shows the exact Host remove/install sequence for confirmation. Other
enabled Qiongli Plugins, a marketplace pointing elsewhere, changed files or an
unexpected scope stop registration. Resolve those conflicts in the Host yourself.

Enter declines each confirmation. Declining the second step leaves the exported
files in place. A Host failure also retains the files and stops at that step;
after resolving it, rerun `install plugin` for the same Host.
Success means the official inventory and cached files match. Start a new Host
session to load Skills and Full MCP; live research-tool readiness is separate.

For standalone content files:

```sh
qiongli install skills
qiongli upgrade skills --preset current-project --profile full
```

The default is the full profile under `$HOME/.qiongli-skills`. The
`current-project` preset uses `.qiongli-skills` in the current directory.
These files are not automatically registered as a Host Plugin. An existing
profile is preserved; specify that same profile when refreshing it.

For scripts, add `--dry-run --json` to produce the canonical file plan, review it,
then use `qiongli app apply` with its exact digest and filesystem approval.
This scripted path exports files only. Host registration requires the interactive
shortcut or explicit official Host commands. Piped input never grants approval.

`qiongli upgrade cli` shows upgrade commands for npm, pip, Cargo and GitHub
archives. It does not run a package manager. The Plugin/Skills commands refresh
content from the running CLI; they do not download a newer CLI release.

## Connect a Host

A CLI package installation does not register or upgrade a Plugin. A bundled
Plugin uses its own executable; check the Plugin version independently of PATH.
The native Plugin source is projected from `content/`; installed caches are
derived outputs and must not be edited as source.

The Codex Plugin includes the general `$qiongli` entry and workflow
shortcuts such as `$qiongli-paper-read`, `$qiongli-lit-review` and
`$qiongli-academic-write`. These entries share the same research instructions;
internal skill cards and templates are loaded as needed. CLI exports and
Marketplace packages generate the shortcuts automatically. An already installed
older Plugin gains them only after updating its source/package and refreshing
it through Codex.

Both Hosts also receive the independent `no-qiongli` Skill for replies without
tool calls. Use `$no-qiongli` in Codex or say “reply only”. See
[reply-only scope and installation](../advanced/agent-skill-collaboration.md#reply-only).

For an explicit standalone MCP connection, configure the Host to launch the
absolute installed `qiongli` path with these arguments:

```text
mcp serve --profile full --transport stdio
```

Use a command path and runtime PATH visible to the actual Host process. A GUI
Host may not inherit your terminal's PATH; npm wrappers additionally need Node.
The connection is ready only when real Qiongli tools are visible and a read-only
call succeeds. Configure literature providers through the visible
`qiongli_config_status` / `qiongli_configure_provider` tools, then inspect
`qiongli_literature_status`. Models and credentials remain owned by the Host.

## Export a local Plugin source

Alpha.8 supports a user-approved local Plugin source without a signed App
package. It bundles the
running native executable, Full MCP and canonical workflow content, so the Plugin
does not depend on npm's Node/PATH wrapper after export.

Choose a secure directory outside Host configuration/cache and `.qiongli` paths.
For Codex, create an export parent and inspect the plan:

```sh
mkdir -p "$HOME/qiongli-plugins/codex"
qiongli app plan plugin-source-install --target codex \
  --destination "$HOME/qiongli-plugins/codex/qiongli-next" > "$HOME/plugin-plan.json"
```

Review the JSON, including destination, binary hash, expected receipt and
`plan_digest_sha256`, then apply that exact digest:

```sh
qiongli app apply --plan "$HOME/plugin-plan.json" \
  --expected-plan-digest <plan_digest_sha256> --approve-filesystem-write
qiongli app plugin-source-status --target codex \
  --destination "$HOME/qiongli-plugins/codex/qiongli-next"
```

Use `--target claude` with a separate export parent for Claude Code. The destination
must end in `qiongli-next`; its parent must already exist and reject writes by
other users. These commands never write Host configuration or install into caches.
A local source is user-approved content, not a signed publisher attestation.

To update after upgrading your CLI, preview `plugin-source-update` with the same
target/destination, review and apply its new digest. Only a fully matching receipt
may be replaced. To remove an export, first unregister its Plugin through the Host,
then preview/apply `plugin-source-remove`. Unknown files, changed content, symlinks
and stale receipts refuse; there is no force-delete option.

The exported marketplace is `qiongli-cli-local` and the selector is
`qiongli-next@qiongli-cli-local`. Use the Host's official local marketplace and
Plugin commands to register the export, avoiding duplicate enabled Qiongli Plugins.
For Codex, the isolated compatibility check used:

```sh
codex plugin marketplace add "$HOME/qiongli-plugins/codex/qiongli-next"
codex plugin add qiongli-next@qiongli-cli-local
```

For Claude Code, validate the export with `claude plugin validate <export-path>`.
The interactive installation flow has been checked against isolated Codex and Claude
configurations on macOS. Loading the Plugin in an actual research session remains
a separate check.
Source updates require a Host refresh/new session; deleting a source does not
unregister its Host Plugin. `source-current` and `source-ready-host-action-required`
only describe the exported files. The interactive shortcuts above now offer separately approved Host registration.
The lower-level source commands still only export files; actual session tool
visibility/read/handoff/approved-write checks remain separate.

## Installation state and version identity (beta.5) {#installation-state}

`qiongli install list` and `qiongli doctor` read official Host inventories and reuse
the local source/cache receipts checked during registration. Uncertain source,
cache, version or enabled state is reported as requiring refresh or unavailable;
it is not automatically turned into an installation suggestion. Read-only Host
commands have time and output limits and never retry a write.

Registered state and matching cache bytes describe installation, not live tools.
Start a new Host session to check Skills and MCP. CLI and Plugin installations
carry separate executables, so refresh the original Plugin directory after updating
the CLI. Compare `qiongli --version`, `qiongli content --json` and the Host's Plugin inventory.

Package builds bind CLI and content versions to the target. npm and PyPI carry
their respective launchers; Cargo carries buildable Rust source. Their shared CLI
description comes from one package metadata field. Executable/resource hashes and
release receipts identify the build; an equal version string alone does not prove
that cached bytes match.

## Research Graph: checking the improvement {#research-graph}

Start with a readable summary, then open an offline view when the relationships
need a closer look:

```bash
qiongli project graph snapshot --project-id <prj_id> --text
qiongli project graph view --project-id <prj_id> --open
```

`--open` saves a new private snapshot in Qiongli's configuration directory and
asks the default HTML application, usually a browser, to open it. The CLI prints
the saved location; open it manually if no window appears. Use `--save` to save
without opening. Earlier snapshots stay until you remove them; project files and
previous exports are not overwritten. The native CLI needs no extra runtime or
local server. Filter records by type and search labels, IDs or source files. Record
pages keep larger projects accessible. Filter relations by type or recorded
status, then page through them: arrows show direction, dashed lines mark
proposals and dotted lines mark rejected relations. **Back to record** retraces
your exploration. The map scrolls horizontally on smaller screens so labels
remain readable; the relation list provides the same actions with full labels.

Source checks link to the available records from an affected file. Missing
sources stay visible even when no record can be shown. Each record or relation
provides a `qiongli project graph source`
command bound to its project revision and projection ID. This reads the recorded
source excerpt; follow its file/page references to inspect the original material.
Use **Copy command** to copy it; when clipboard access is unavailable, the page
selects the command for manual copying. Changed sources require a refreshed
project and a new view. Filters affect only the display; JSON download retains
the complete exported snapshot, including proposed and rejected relations.

The page is a snapshot, not a live editor. Its **Save snapshot JSON** button saves
the same projection for a stage record. Keep previous snapshots and original
sources; an export does not archive or delete them. Both exports contain research
content, so share them within the project's access boundaries.

To choose your own location, retain the stdout form:
`qiongli project graph view --project-id <prj_id> > research-graph-new.html`.
Choose a new filename: shell redirection can overwrite an existing file.

For review or venue decisions, the decision log's optional `Related Claims`
column links existing claim IDs with `informs`. A `locked` decision produces a
reviewed relation; tentative, blocked or revisit decisions remain proposed.
These links never count as supporting evidence. The source record retains the
report location, rationale, applicable venue requirements and manuscript impact.

Request collaboration in natural language, such as “ask another agent to review
this evidence independently.” The Skill uses available Host tools, records their
returned task identities, collects actual results and checks source versions.
Full MCP routing states its limits: it does not create Host subagents or provide
general automatic cross-Host communication. Without a communication tool, it
prepares a packet for you to transfer. Duplicate, stale or unfinished results do
not count as additional independent reviews. The originating coordinator retains
integration responsibility and the existing project-write approvals.

The 1.x citation graph expands references and citations from literature seeds and
deduplicates candidate papers. The 2.x local Research Graph adds relationships
within research records: stable claim identities, source locations and continuity
between literature and manuscript records. These graphs serve different purposes.
More nodes do not demonstrate better literature search or faster execution.

| Quality check | Required outcome |
|---|---|
| Multiple sources for a claim | One claim ID retains every distinct support record and location |
| Source navigation | Each support resolves to its current record, even after CSV rows are reordered |
| Repeated import and rebuild | Deduplicated nodes and edges stay identical |
| Unsupported relationships | Missing locations, unreviewed support, identity conflicts and unsafe paths never create reviewed support |
| Continuation across stages | Reuse claim IDs, citekeys, predecessors and source records; a summary does not replace evidence |
| Freshness and write permission | Shared project revisions reject stale reads and unapproved changes across CLI/MCP |

Existing Graph, project-service and Full MCP tests check these outcomes. Raw notes
and PDFs still need Host-assisted reading and proposed canonical records within
an authorized scope. Review and save those records before rebuilding Graph.
Use `qiongli project graph --help` for available operations; missing tools do not
permit bypassing approval with direct file edits.

Structural checks establish record and source consistency, not semantic research
quality. Compare the same corpus in a fresh Host session for extraction fidelity,
unresolved evidence and user steps. Speed claims require a measured comparison of
the same task and environment.
