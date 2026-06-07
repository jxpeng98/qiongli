---
layout: home

hero:
  name: Qiongli
  text: Contract-bound research workflows for AI coding agents.
  tagline: Install once, then use Codex, Claude Code, or Gemini to run academic workflows with explicit task IDs, quality gates, literature diagnostics, role handoffs, and auditable artifacts.
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
  - title: Install Without Guesswork
    details: Choose native plugin, bootstrap partial/full, npm, or pipx based on what runtime surface you actually need.
  - title: Research Routes
    details: Start from systematic review, empirical, qualitative, RCT, theory, or code-first methods workflows.
  - title: Evidence Contracts
    details: Keep claims, citations, search logs, diagnostics, methods, code, and review status traceable under standard artifact paths.
  - title: Multi-Agent Control
    details: Use solo, duo, and triad execution modes with explicit handoffs, disagreement records, and verification outcomes.
---

## Choose Your Entry Point

| You want to... | Start here | Why |
|---|---|---|
| Try Qiongli in one client | [Install](/guide/install) | Native plugin / extension paths keep setup small. |
| Know what to type after install | [Using Agent Skills](/guide/using-agent-skills) | Codex, Claude Code, Gemini, and shell expose Qiongli differently. |
| Install global workflows for several clients | [Quickstart](/quickstart) | Bootstrap `partial` installs workflow assets without requiring Python. |
| Run validators, `doctor`, or orchestrated tasks | [Multi-Agent Runtime](/guide/multi-agent) | `full` runtime explains Python, model CLIs, auth, broker/direct Gemini modes, and verification. |
| Pick a paper workflow | [Task Recipes](/guide/task-recipes) | Maps real research goals to paper types, stages, Task IDs, and expected outputs. |
| Automate installs or upgrades | [CLI Reference](/reference/cli) | Covers `qiongli`, `ql`, npm/npx, pipx, compatibility aliases, and JSON checks. |

## What The Current System Covers

Qiongli ships a single portable workflow package, `qiongli-workflow`, with staged research skills and a shared task contract. The current documentation is organized around what a researcher or project owner needs to do:

- **Frame the work:** refine questions, identify gaps, map theory, choose venues, and define contribution claims.
- **Build the literature base:** plan provider-aware searches, materialize search bundles, run diagnostics, deduplicate results, screen papers, extract evidence, and snowball citations.
- **Design and execute the study:** specify variables, datasets, robustness checks, preregistration, ethics artifacts, and data management.
- **Write and audit the manuscript:** structure sections, maintain claim-evidence integrity, generate figures/tables, evaluate limitations, and prepare submission/rebuttal materials.
- **Handle research code:** use the Stage-I specification -> planning -> execution -> review path for code-first or methods-heavy work.
- **Coordinate models:** assign controller, primary, reviewer, and verifier roles across Codex, Claude Code, and Gemini while preserving handoffs and verification status.

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

Asset installation and workflow discovery are intentionally lighter than full orchestration. You can install `qiongli-workflow` without Python, but `doctor`, validators, tests, and model-orchestrated task execution require Python 3.12+, the relevant model CLIs, and matching authentication.
