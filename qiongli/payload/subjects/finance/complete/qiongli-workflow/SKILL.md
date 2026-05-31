---
name: qiongli
description: Finance-focused corporate finance, asset pricing, market microstructure, and risk workflow for undergraduate-and-above researchers, with doctoral-level journal manuscripts as the quality target.
---

# Qiongli Finance

Finance-focused corporate finance, asset pricing, market microstructure, and risk workflow for undergraduate-and-above researchers, with doctoral-level journal manuscripts as the quality target.

## Finance Workflow Map

### 1. Finance Research Framing
Finance question, economic mechanism, contribution, and venue fit

### 2. Finance Literature Positioning
Asset pricing, corporate finance, market microstructure, and risk literature grounding

### 3. Finance Design and Risk Diagnostics
Return construction, identification, risk adjustment, event windows, and robustness

### 4. Finance Evidence and Results
Risk-adjusted interpretation, factor diagnostics, event-study evidence, tables, and figures

### 5. Finance Manuscript and Reproducibility
Paper architecture, data-code transparency, reporting checks, and review readiness

## Required Behavior

- Use the canonical task and output definitions in `references/workflow-contract.md`.
- Keep stage labels and task IDs unchanged across models.
- When a workflow references `templates/<name>.md`, load the template from `templates/`.
- Use `skills/registry.yaml` as the active skill list for this subject package.

## Literature Provider Configuration

- CLI, Codex, and Claude Code installs can configure external literature providers with `qiongli provider setup` and audit them with `qiongli provider doctor`.
- Treat `provider_connected` as the only mode where configured external academic provider credentials are available to the local runtime.
- Treat `strategy_only` as a constrained mode: use platform search or user-supplied corpus, record the limitation, and do not claim review-grade external provider coverage.
- Claude Desktop/Web focused ZIPs are skill-only packages kept within the 180-file upload budget. They contain workflows/prompts/templates, store no secrets, and cannot execute OpenAlex, Semantic Scholar, Crossref, or PubMed API calls by themselves.
- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or platform-native search capability before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex and Semantic Scholar configuration. If no MCPB or platform-native search is available, record the run as `strategy_only`.

## Skill Loading Strategy

Use `skills-summary.md` for quick lookup, `skills-core.md` for consolidated guidance, and detailed files under `skills/` for active subject skills.
