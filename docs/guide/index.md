# Guide

Use this section when you are operating Qiongli as a user or project owner. It keeps the path short: install the smallest useful surface, learn what to type, choose a research route, then add runtime checks only when needed.

## Start Here

- [Quickstart](/quickstart): smallest install surface, first workspace, paper routes, and quality-gate context.
- [Install](/guide/install): native plugin, bootstrap, npm, and pipx install surfaces.
- [Using Agent Skills](/guide/using-agent-skills): what to type after install in Codex, Claude Code, Antigravity, Hermes, and the shell.
- [Task Recipes](/guide/task-recipes): scenario-based routes for literature review, empirical design, writing, code, and rebuttal.
- [Multi-Agent Runtime Guide](/guide/multi-agent): runtime routing, local agent execution, and auth rules.
- [Examples](/examples/): paper-type playbooks for systematic review, empirical, qualitative, methods, and theory workflows.
- [Upgrade](/guide/upgrade): npm asset refreshes, full-runtime package updates, shell bootstrap, Python CLI, and long-lived clone paths.
- [Data Ownership and Lifecycle](/guide/data-lifecycle): ownership, backup, export, uninstall, deletion, and 1.x end-of-support policy.
- [Troubleshooting](/guide/troubleshooting): unified error-code guide.

## Recommended Reading Order

1. [Quickstart](/quickstart)
2. [Install](/guide/install)
3. [Using Agent Skills](/guide/using-agent-skills)
4. [Task Recipes](/guide/task-recipes)
5. [Multi-Agent Runtime Guide](/guide/multi-agent)
6. [CLI Reference](/reference/cli)
7. [Data Ownership and Lifecycle](/guide/data-lifecycle)
8. [Troubleshooting](/guide/troubleshooting)

## Update Boundary

- On npm/npx, use `qiongli update` or `qiongli refresh` to reapply assets from the current package.
- On npm/npx, use `qiongli upgrade` for an overwrite asset refresh from the current package.
- Full runtime package self-update, including `qiongli self-update`, requires the Python runtime first: `pipx install qiongli`.

## When To Leave This Section

- Need the layer model or dependency rules: go to [Architecture](/architecture) or [Conventions](/conventions).
- Need MCP/Zotero/provider setup: go to [Advanced](/advanced/).
- Need to modify the system itself: go to [Maintainer](/maintainer/).
