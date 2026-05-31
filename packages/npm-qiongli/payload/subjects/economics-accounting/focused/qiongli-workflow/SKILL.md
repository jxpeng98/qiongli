---
name: qiongli
description: Cross-disciplinary economics and accounting workflow for archival, causal, and reporting-setting research.
---

# Qiongli Economics + Accounting

Cross-disciplinary economics and accounting workflow for archival, causal, and reporting-setting research.

## Economics + Accounting Workflow Map

### 1. Research Framing
Question, contribution, theory, institutional setting, and venue fit

### 2. Economics and Accounting Literature
Search, mapping, extraction, citation grounding, and construct positioning

### 3. Identification, Measurement, and Design
Causal design, disclosure setting, accounting constructs, variables, and robustness

### 4. Evidence and Results
Economic interpretation, accounting measurement diagnostics, tables, and figures

### 5. Manuscript and Reproducibility
Paper architecture, reporting transparency, reproducibility, and submission readiness

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
