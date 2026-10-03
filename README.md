<div align="center">
  <h1>Qiongli (穷理)</h1>
  <p><strong>Use AI agents for research, with sources you can check.</strong></p>
  <p>Read papers, design studies and write with Codex, Claude Code or DeepSeek Harness. Keep your model settings and research records.</p>
  <p>
    <a href="https://www.npmjs.com/package/qiongli"><img alt="npm version" src="https://img.shields.io/npm/v/qiongli/latest?style=flat-square&amp;logo=npm"></a>
    <a href="https://pypi.org/project/qiongli/"><img alt="PyPI version" src="https://img.shields.io/pypi/v/qiongli?style=flat-square&amp;logo=pypi"></a>
  </p>
  <p><a href="README_CN.md">中文</a> · <a href="docs/index.md">Docs</a> · <a href="tooling/release/v2.2.0.md">2.2.0 release notes</a></p>
</div>

## Install

Download your platform's CLI from [Release v2.2.0](https://github.com/jxpeng98/qiongli/releases/tag/v2.2.0), verify its `SHA256SUMS`, and extract it. Run `./qiongli --version` (PowerShell: `.\qiongli.exe --version`). No Python, Node.js or Rust installation is needed.

Supports macOS ARM64, Windows x64, and Linux x64/ARM64 with glibc 2.35+. [Download and package-manager instructions](docs/guide/install.md).

With Node.js 18+, you can also install through npm:

```sh
npm install --global qiongli@latest
```

## Start using Qiongli

Run in a terminal; use `./qiongli` or `.\qiongli.exe` if it is not on PATH:

```sh
qiongli install
qiongli mcp check
```

Choose your Host and confirm the file changes and Plugin registration. The Plugin includes Skills and Full MCP. Open a new Host session, check the Qiongli tools, then ask:

> Read this paper. Explain the findings, point to the evidence and flag what remains uncertain.

In Codex, you can also use `$qiongli` or `$qiongli-paper-read`. For a reply without tools or file work, use `$no-qiongli` or say “reply only”. See [Skills and optional hooks](docs/advanced/agent-skill-collaboration.md).

Project writes require a preview, approval and revision checks. Literature services and Zotero are configured separately. Marketplace Plugins use Lite MCP; CLI-installed Plugins use Full MCP.

## Upgrade

Update the CLI through the channel you used to install it, then refresh the Plugin:

```sh
qiongli install plugin
```

Start a new Host session afterward. Coming from 1.x? Back up your projects and settings, install 2.2.0, then connect your Host. New saved-artifact receipts may require the newer CLI; switching binaries alone does not downgrade project data. See [upgrade and rollback](docs/guide/upgrade.md) and [what changed in 2.x](docs/guide/whats-new-2.md).

## Documentation

- [Quickstart](docs/quickstart.md) · [CLI commands](docs/guide/cli-2x.md)
- [Research tasks](docs/guide/task-recipes.md) · [Research Graph example](docs/examples/research-graph.md)
- [Provider and Zotero setup](docs/advanced/index.md) · [Troubleshooting](docs/guide/troubleshooting.md)
- [Development](CONTRIBUTING.md) · [1.x and historical guides](docs/legacy/index.md)

## Credit

The Academic Idea Funnel and Academic Grill Loop are an academic adaptation of Matt Pocock's `grill-me` pattern. Thanks to the [linux.do](https://linux.do/) community for discussion and feedback.
