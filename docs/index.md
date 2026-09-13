---
layout: home
hero:
  name: Qiongli
  text: "Research with an evidence trail."
  tagline: "A Rust-native CLI, Skills and MCP sharing research records and sources. Keep your chosen Host and model."
  actions:
    - theme: brand
      text: Download 2.x CLI
      link: /guide/cli-2x#standalone-binary-download
    - theme: alt
      text: Quickstart
      link: /quickstart
    - theme: alt
      text: Choose a workflow
      link: /guide/task-recipes
features:
  - title: Extract and run
    details: "The standalone binary includes research resources and needs no separate Python, Node.js or Rust runtime."
  - title: Start with your task
    details: "Shared Skills cover reading, reviews, design, writing and polish. Load only the guidance the request needs."
  - title: Follow the sources
    details: "Graph connects claims, sources and locations in canonical records. Stage summaries retain findings and changes."
  - title: Review before writing
    details: "Project changes require previews, explicit approval and revision checks. Summaries never automatically delete source files."
---

## Start here

These pages cover **2.0.0**, the first stable native CLI release from `main`.
Start with guided installation, the offline Graph, reply-only Skill or optional hooks.
Coming from 1.x? Read [what changed and how to migrate](guide/whats-new-2.md).

| What you need | Entry |
|---|---|
| Download, extract and run | [Standalone binary](guide/cli-2x.md#standalone-binary-download) |
| Install through npm, PyPI or Cargo | [Installation and commands](guide/cli-2x.md) |
| Install or refresh bundled Plugins and Skills | [Plugin installation flow](guide/cli-2x.md#install-and-upgrade-bundled-content) |
| Review CLI copies and integration state | `qiongli setup`, `qiongli install`, `qiongli doctor` |
| Understand the research Graph | [Graph scope and checks](guide/cli-2x.md#research-graph) |
| Maintain an older Python/npm installation | [1.x reference](reference/cli.md) |

## Runtime boundaries

The 2.x CLI, Lite/Full MCP and exported native Plugins need no additional language
runtime. The npm entry needs Node, PyPI needs Python, and Cargo needs Rust build
tools. Choose the standalone binary to avoid those installation prerequisites.
Configure Host applications and online service accounts when the task requires them.

Update packages through their original channel. `upgrade plugin` refreshes content
from the running CLI; `upgrade cli` explains the package update commands. After
registration, start a new Host session to check actual tools. Native Marketplace
platform packages use Lite MCP; local CLI-exported Plugins use Full MCP.

[Research workflows](guide/task-recipes.md) · [Architecture](architecture.md) · [Observed Host capabilities](guide/agent-host-capability-matrix.md)
