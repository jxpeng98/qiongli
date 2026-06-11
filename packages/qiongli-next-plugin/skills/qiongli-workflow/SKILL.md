---
name: qiongli-next
description: Qiongli Next version: v1.2.1. General-purpose Qiongli academic workflow across paper types and methods.
---

# Qiongli Core

General-purpose Qiongli academic workflow across paper types and methods.

Installed Qiongli workflow version: `v1.2.1`

## Core Workflow Map

### 1. Research Framing
Question, contribution, theory, and venue fit

### 2. Literature Grounding
Search, screening, extraction, citation, and mapping

### 3. Study Design
Design, variables, data, preregistration, and robustness

### 4. Ethics and Disclosure
IRB, participant protection, and disclosure statements

### 5. Evidence Synthesis
Effect sizes, synthesis, quality, and publication-bias checks

### 6. Manuscript Writing
Architecture, interpretation, tables, figures, and discussion

### 7. Reporting Compliance
PRISMA, reporting, and academic tone checks

### 8. Submission and Review
Packaging, rebuttal, reviewer simulation, and limitations

### 9. Research Code and Statistics
Code planning, execution, review, reproducibility, and stats

### 10. Proofreading and Similarity
Human voice, similarity, final proofread, and AI trace checks

### 11. Presentation
Scholarly slide, Slidev, and Beamer workflows

### 12. Cross-Cutting Governance
Context, metadata, collaboration, boundaries, and self-critique

## Required Behavior

- Use the canonical task and output definitions in `references/workflow-contract.md`.
- Keep stage labels and task IDs unchanged across models.
- When a workflow references `templates/<name>.md`, load the template from `templates/`.
- Use `skills/registry.yaml` as the active skill list for this subject package.

## Literature Provider Configuration

- CLI, Codex, and Claude Code installs can configure external literature providers with `qiongli provider setup` and audit them with `qiongli provider doctor`.
- In bundled MCP installs, do not expect the client MCP settings UI to inject provider keys into the plugin-bundled MCP server. Use `qiongli_config_status` to find the shared provider config path, then use `qiongli_configure_provider` to open a local browser setup page. Use `qiongli_save_provider_config` only for explicit scripted writes.
- Keep provider secrets out of `.mcp.json`, plugin manifests, release ZIPs, and research artifacts. The bundled provider server reads the shared provider config or explicit provider environment variables at runtime.
- Treat `provider_connected` as the only mode where configured external academic provider credentials are available to the local runtime.
- Treat `strategy_only` as a constrained mode: use platform search or user-supplied corpus, record the limitation, and do not claim review-grade external provider coverage.
- Claude Desktop/Web focused ZIPs are skill-only packages kept within the 180-file upload budget. They contain workflows/prompts/templates, store no secrets, and cannot execute OpenAlex, Semantic Scholar, Crossref, or PubMed API calls by themselves.
- For a manual Desktop install, upload the `qiongli-claude-desktop-skill-*.zip` first, then add a manual MCP install when provider calls or local orchestration are required. The skill ZIP supplies agent instructions, workflows/prompts/templates, and subject overlays; MCP supplies tool calls.
- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or platform-native search capability before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex and Semantic Scholar configuration. If no MCPB or platform-native search is available, record the run as `strategy_only`.
- The literature MCPB provides literature MCP tools only. It does not launch orchestrator agents. To expose the full agent runtime through MCP, manually install the full CLI MCP server with `qiongli mcp serve --transport stdio`; clients can then call tools such as `qiongli_task_run` after the local CLI runtime and model CLIs are configured.

## Skill Loading Strategy

Use `skills-summary.md` for quick lookup, `skills-core.md` for consolidated guidance, and detailed files under `skills/` for active subject skills.

## Prerelease Invocation

Invoke this beta package as `$qiongli-next` when testing the next Qiongli core workflow.
