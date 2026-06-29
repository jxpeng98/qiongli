---
layout: home

hero:
  name: Qiongli
  text: "Research workflows with an audit trail."
  tagline: "Choose a paper route, install the smallest useful surface, and keep literature, writing, code, and review evidence inspectable."
  actions:
    - theme: brand
      text: Quickstart
      link: /quickstart
    - theme: alt
      text: Install
      link: /guide/install
    - theme: alt
      text: Choose A Workflow
      link: /guide/task-recipes

features:
  - title: "Start Small"
    details: "Use a native plugin, Desktop ZIP, bootstrap, npm, or pipx path based on the task instead of installing every runtime up front."
  - title: "Route The Work"
    details: "Map research goals to paper types, stages, Task IDs, expected outputs, and quality gates."
  - title: "Keep Evidence Visible"
    details: "Track claims, citations, search logs, diagnostics, methods, code, review status, and handoffs under predictable paths."
  - title: "Separate Update From Refresh"
    details: "`qiongli update` upgrades the package; `qiongli upgrade` refreshes local assets. The boundary is explicit."
---

## Choose Your Entry Point

| You want to... | Start here |
|---|---|
| Try Qiongli in one client | [Install](/guide/install) |
| Get from no setup to a first workspace | [Quickstart](/quickstart) |
| Know what to type after install | [Using Agent Skills](/guide/using-agent-skills) |
| Choose a paper workflow | [Task Recipes](/guide/task-recipes) |
| Run validators, `doctor`, or orchestrated tasks | [Multi-Agent Runtime](/guide/multi-agent) |
| Automate installs, checks, updates, or release work | [CLI Reference](/reference/cli) |

## Latest Stable Downloads

Current stable release: [v1.14.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.14.0). These direct links cover the common install paths; use the download guide for subject-specific Desktop ZIPs and maintainer artifacts.

| Need | Link or command |
|---|---|
| npm CLI | [`qiongli@1.14.0`](https://www.npmjs.com/package/qiongli/v/1.14.0): `npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.14.0`](https://pypi.org/project/qiongli/1.14.0/): `pipx install qiongli` |
| Claude Desktop/Web core skill | [`qiongli-claude-desktop-skill-core-v1.14.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.14.0/qiongli-claude-desktop-skill-core-v1.14.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.14.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.14.0/qiongli-zotero-companion-0.2.2.xpi) |
| All release assets | [Download guide](https://github.com/jxpeng98/qiongli/releases/download/v1.14.0/qiongli-downloads-v1.14.0.md) and [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.14.0) |

## What The System Covers

Qiongli ships the portable `qiongli-workflow` package plus optional local runtimes for literature search and orchestration.

- **Framing:** questions, gaps, contribution claims, venues, and boundaries.
- **Literature:** provider-aware searches, diagnostics, bundles, screening, extraction, and snowballing.
- **Design:** variables, datasets, robustness, preregistration, ethics, and data management.
- **Writing:** claim-evidence mapping, tables, figures, limitations, proofreading, submission, and rebuttal.
- **Code:** Stage-I specification, planning, execution, and review for methods-heavy work.
- **Coordination:** solo, duo, or triad roles across local agent tools with recorded handoffs.

## Runtime Boundary

Installing workflow assets does not imply local agent execution. You can use Qiongli as a skill/plugin without Python. Python 3.12+, model CLIs, and matching authentication are only required for `doctor`, validators, MCP orchestration, or actual task execution.

## Documentation Map

- [Guide](/guide/): install, usage, upgrades, troubleshooting, and runtime choices.
- [Examples](/examples/): paper-type playbooks.
- [Reference](/reference/): CLI behavior and skill catalog.
- [Architecture](/architecture): package surfaces, contracts, roles, and bridges.
- [Advanced](/advanced/): MCP providers, Zotero, subject packaging, and plugin-first distribution.
- [Maintainer](/maintainer/): release policy, naming policy, and contributor guidance.
