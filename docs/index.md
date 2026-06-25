---
layout: home

hero:
  name: Qiongli
  text: "Research workflows that leave an audit trail."
  tagline: "Turn broad academic requests into task IDs, quality gates, literature diagnostics, role handoffs, and artifacts your team can inspect later."
  actions:
    - theme: brand
      text: Start With Quickstart
      link: /quickstart
    - theme: alt
      text: Choose A Workflow
      link: /guide/task-recipes
    - theme: alt
      text: Install
      link: /guide/install

features:
  - title: "Choose The Right Install"
    details: "Start with the smallest surface that fits your work: native plugin, bootstrap partial/full, npm, or pipx."
  - title: "Move From Topic To Workflow"
    details: "Route systematic reviews, empirical studies, qualitative work, RCTs, theory papers, and code-first methods papers."
  - title: "Keep Evidence Traceable"
    details: "Store claims, citations, search logs, diagnostics, methods, code, and review status under predictable artifact paths."
  - title: "Coordinate Agent Review"
    details: "Use solo, duo, or triad runs with explicit handoffs, disagreement records, and verification outcomes."
---

## Choose Your Entry Point

| You want to... | Start here | Why |
|---|---|---|
| Try Qiongli in one client | [Install](/guide/install) | Native plugin and extension paths keep setup small. |
| Figure out what to type after install | [Using Agent Skills](/guide/using-agent-skills) | Codex, Claude Code, Antigravity, Hermes, and shell each expose Qiongli differently. |
| Install workflows for several clients | [Quickstart](/quickstart) | Bootstrap `partial` installs workflow assets without requiring Python. |
| Run validators, `doctor`, or orchestrated tasks | [Multi-Agent Runtime](/guide/multi-agent) | `full` runtime explains Python, model CLIs, auth, runtime routing, and verification. |
| Choose a paper workflow | [Task Recipes](/guide/task-recipes) | Map your research goal to a paper type, stage, Task ID, and expected output. |
| Automate installs or upgrades | [CLI Reference](/reference/cli) | Covers `qiongli`, `ql`, npm/npx, pipx, compatibility aliases, and JSON checks. |

## Latest Stable Downloads

Current stable release: [v1.7.0](https://github.com/jxpeng98/qiongli/releases/tag/v1.7.0). These direct links cover the common install paths; use the download guide for subject-specific Desktop ZIPs and maintainer artifacts.

| Need | Link or command |
|---|---|
| npm CLI | [`qiongli@1.7.0`](https://www.npmjs.com/package/qiongli/v/1.7.0): `npm install -g qiongli@latest` |
| PyPI CLI | [`qiongli 1.7.0`](https://pypi.org/project/qiongli/1.7.0/): `pipx install qiongli` |
| Claude Desktop/Web core skill | [`qiongli-claude-desktop-skill-core-v1.7.0.zip`](https://github.com/jxpeng98/qiongli/releases/download/v1.7.0/qiongli-claude-desktop-skill-core-v1.7.0.zip) |
| Claude Desktop literature MCPB | [`qiongli-literature-provider-0.1.5.mcpb`](https://github.com/jxpeng98/qiongli/releases/download/v1.7.0/qiongli-literature-provider-0.1.5.mcpb) |
| Zotero Desktop companion | [`qiongli-zotero-companion-0.2.2.xpi`](https://github.com/jxpeng98/qiongli/releases/download/v1.7.0/qiongli-zotero-companion-0.2.2.xpi) |
| All release assets | [Download guide](https://github.com/jxpeng98/qiongli/releases/download/v1.7.0/qiongli-downloads-v1.7.0.md) and [GitHub Release](https://github.com/jxpeng98/qiongli/releases/tag/v1.7.0) |

## What The Current System Covers

Qiongli ships a portable workflow package, `qiongli-workflow`, plus optional local runtimes for literature search and orchestration. The docs are organized around the work a researcher or project owner actually needs to finish:

- **Frame the work:** refine the question, identify gaps, map theory, choose venues, and define the contribution.
- **Build the literature base:** plan provider-aware searches, materialize search bundles, run diagnostics, deduplicate results, screen papers, extract evidence, and snowball citations.
- **Design and execute the study:** specify variables, datasets, robustness checks, preregistration, ethics artifacts, and data management.
- **Write and audit the manuscript:** structure sections, keep claims tied to evidence, plan figures and tables, evaluate limitations, and prepare submission or rebuttal materials.
- **Handle research code:** use the Stage-I specification -> planning -> execution -> review path for code-first or methods-heavy work.
- **Coordinate models:** assign controller, primary, reviewer, and verifier roles across Codex, Claude Code, and Antigravity while preserving handoffs and verification status.

## Documentation Map

- [Guide](/guide/): operational path for installation, usage, upgrades, troubleshooting, and runtime choices.
- [Using Agent Skills](/guide/using-agent-skills): client-by-client invocation rules, including Codex `/skills` and `$qiongli`.
- [Task Recipes](/guide/task-recipes): scenario-driven routes for paper types and common research goals.
- [Examples](/examples/): concrete playbooks for systematic review, empirical, qualitative, methods, and theory papers.
- [Reference](/reference/): CLI behavior, skill catalog, and operator-facing conventions.
- [Architecture](/architecture): how contracts, skills, roles, pipelines, bridges, and package surfaces fit together.
- [Advanced](/advanced/): subject packaging, extension, MCP providers, Zotero, rigorous literature search, and plugin-first distribution.
- [Maintainer](/maintainer/): release policy, naming policy, and implementation guidance for contributors.

## Runtime Boundary

Installing workflow assets is intentionally lighter than running full orchestration. You can use `qiongli-workflow` without Python. You only need Python 3.12+, the relevant model CLIs, and matching authentication when you run `doctor`, validators, tests, or model-orchestrated task execution.

## Community Credit

Thanks to the [linux.do](https://linux.do/) community for being an open Chinese-language space for practical AI tooling, developer workflow, and local-first productivity discussions. Qiongli will be shared there as the workflow matures, both to reach more users and to collect direct feedback.
