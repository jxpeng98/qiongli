<div align="center">
  <h1>Qiongli (穷理)</h1>
  <p><strong>Use AI agents for academic research without losing the evidence trail.</strong></p>
  <p>Qiongli turns a research goal into a paper route, task IDs, literature and citation evidence, quality gates, agent handoffs, and stable files under <code>RESEARCH/[topic]/</code>.</p>
  <p>
    <a href="https://www.npmjs.com/package/qiongli"><img alt="npm latest version" src="https://img.shields.io/npm/v/qiongli/latest?style=flat-square&amp;logo=npm&amp;label=npm%20latest"></a>
    <a href="https://www.npmjs.com/package/qiongli?activeTab=versions"><img alt="npm next version" src="https://img.shields.io/npm/v/qiongli/next?style=flat-square&amp;logo=npm&amp;label=npm%20next&amp;color=cb3837"></a>
    <a href="https://pypi.org/project/qiongli/"><img alt="PyPI latest version" src="https://img.shields.io/pypi/v/qiongli?style=flat-square&amp;logo=pypi&amp;label=PyPI%20latest"></a>
  </p>
  <p>
    <a href="README_CN.md">中文 README</a> ·
    <a href="docs/index.md">Docs</a> ·
    <a href="docs/zh/index.md">中文文档</a> ·
    <a href="docs/quickstart.md">Quickstart</a> ·
    <a href="docs/guide/cli-2x.md#standalone-binary-download">Install</a> ·
    <a href="docs/guide/cli-2x.md">CLI</a>
  </p>
</div>

## Qiongli 2.x

Native academic research CLI with embedded Skills, templates and Lite/Full MCP.
Keep your chosen Codex or Claude Code Host and model settings. Qiongli supplies
research records, source links, reviewable changes and stage handoffs; no Qiongli
desktop App is required.

Beta.5 unifies installation and updates: run `qiongli install` in a terminal,
or `qiongli install plugin` to reuse an existing Host registration. Check the local
MCP protocol with `qiongli mcp check`. A Plugin includes the research Skills and its
MCP runtime; standalone Skills are an optional export. The [2.x guide](docs/guide/cli-2x.md)
explains the choices and how to verify the first Host session.

## Qiongli 2.x Standalone Downloads

**Download, extract, and run. You do not need Python, Node.js, Rust or a package manager.**

Choose the complete CLI from [GitHub Release v2.0.0-beta.5](https://github.com/jxpeng98/qiongli/releases/tag/v2.0.0-beta.5).
After extraction, run `./qiongli --help` (PowerShell: `.\qiongli.exe --help`).
The program already includes the research Skills, templates and Lite/Full MCP
resources. You can use it from that folder; installing an App or changing PATH is optional.

| Platform | Binary archive |
|---|---|
| macOS Apple Silicon (ARM64) | [Download `.tar.gz`](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [Download `.zip`](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-x86_64-pc-windows-msvc.zip) |
| Linux x64 (glibc 2.35+) | [Download `.tar.gz`](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/qiongli-2.0.0-beta.5-x86_64-unknown-linux-gnu.tar.gz) |

Windows beta releases include the C runtime in the executable; no Visual C++ runtime
installation is needed. Linux uses system libraries with glibc 2.35+.

Verify with the release's [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.0.0-beta.5/SHA256SUMS)
before running. See [extraction, PATH and MCP setup](docs/guide/cli-2x.md#standalone-binary-download)
for step-by-step instructions. Choose these platform archives from **Assets**, rather than
GitHub's automatic **Source code** downloads. Host applications and online services remain separate.

Package-manager users can use `npm install --global qiongli@next` with Node.js 18+,
or follow the [PyPI instructions](docs/guide/cli-2x.md#package-managers) with Python 3.9+.
Both distribute the native executable. Cargo builds it from source with Rust 1.97+
and a native linker.

```sh
cargo install qiongli --version 2.0.0-beta.5 --locked
```

Cargo provides `qiongli` and `ql`. Choose the archives above to skip compilation.

Beta.3 adds shorter commands and readable terminal output. Run `qiongli` for
help, `qiongli project` to list projects, or `qiongli setup` to review installed
CLI versions and manual archive/uninstall steps. npm can show the review during
a foreground install; pip and Cargo users run it afterward. No files or settings are changed. See the
[installation review guide](docs/guide/cli-2x.md#review-existing-cli-installations).

`install plugin` and `upgrade plugin` use the same flow, with file previews
and separate confirmation for official Codex/Claude registration. See [bundled content installation](docs/guide/cli-2x.md#install-and-upgrade-bundled-content).

## After installation

```sh
qiongli --version
qiongli install
qiongli mcp check
qiongli doctor
qiongli setup
qiongli content
qiongli help install plugin
```

`setup` reviews visible CLI copies and gives manual removal or archive guidance.
It does not uninstall programs, move files or change PATH. `qiongli` and `ql` use
the same commands; direct downloads provide `qiongli`, while package-manager
installs also provide `ql`. Use `--json` in scripts.

`install plugin` previews bundled files and asks for confirmation before exporting.
It then asks separately for official Host registration. Later runs discover and
refresh that source; `upgrade plugin` is an alias. `install skills` exports
`.qiongli-skills`; use the Plugin installation path to load the workflow in a Host. `upgrade cli` explains how to update
through the original channel without running a package manager.

Follow the [installation and upgrade examples](docs/guide/cli-2x.md#install-and-upgrade-bundled-content).
After a Plugin update, start a new Host session and check that its tools are available.
In development builds after beta.5, the Plugin guide also offers optional context
hooks (off by default). `install plugin --hooks context` includes them;
`--hooks off` removes their Plugin configuration. See [Hook setup and verification](docs/advanced/agent-skill-collaboration.md#optional-context-hooks).

## Skills, MCP and research records

| Part | Purpose |
|---|---|
| Skills / Plugin | Route the requested reading, review, study design, writing, polish or stage summary through shared guidance |
| Lite MCP | 14 tools for bounded literature, configuration, search planning and Zotero operations |
| Full MCP | 32 tools, adding projects, Graph and Host handoffs; writes retain preview, approval and revision checks |
| Research Graph | Rebuild links between claims, sources and locations from canonical research records; unsupported relations remain unconfirmed |
| Stage summaries | Keep substantive findings, sources, predecessors and changes; optional retention review lists individual files for the user to select and delete personally |

New Codex Plugin builds have 20 workflow shortcuts and the general `$qiongli` entry;
82 internal skill cards stay available on demand. Claude keeps one main Skill.
Native Marketplace platform packages start Lite MCP; CLI-exported local Plugins
start Full MCP. A shared Skill name does not imply the same available tools.

Graph improvements are checked through stable identities, multiple sources, resolvable
evidence, deterministic rebuilding and stale-read refusal. The Host still needs to
normalize authorized material into canonical records. This is not an arbitrary-folder
or PDF semantic scanner. See [Graph scope and checks](docs/guide/cli-2x.md#research-graph).

## Development and documentation

Shared research content lives in `content/`; native services live in
`packages/qiongli-native/`. npm, PyPI and Cargo keep their own launchers and installation
instructions while sharing the CLI description and release identity. Generate packages
from their canonical sources; installed caches and generated mirrors are not edit targets.

- [Quickstart](docs/quickstart.md) and [2.x command guide](docs/guide/cli-2x.md)
- [Architecture](docs/architecture.md) and [contribution workflow](CONTRIBUTING.md)
- [1.x command reference](docs/reference/cli.md), retained for older installations and migration
- [Distribution materialization](docs/development/distribution-materialization.md), including staged materialization and npm package contract tests

Reuse existing approval, rollback, package-identity and content checks. A Rust rewrite
alone does not establish a speed or maintenance-cost improvement; those claims need
measurements against the same task and environment.

## Credit

The Academic Idea Funnel and Academic Grill Loop are an academic adaptation of
Matt Pocock's `grill-me` pattern for academic idea-discovery, evidence, rival explanations
and feasibility. Thanks to the [linux.do](https://linux.do/) community for practical
discussion and feedback.
