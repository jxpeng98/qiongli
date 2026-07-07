---
name: qiongli-next
description: "Qiongli Next version: v1.17.0-beta.1. Cross-platform academic research workflow for Codex, Claude / Claude Code, and CLI. Use for academic research lifecycle work: paper planning, literature review, paper reading, gap finding, study design, manuscript writing, statistics, analysis code, reproducibility, proofread, rebuttal, submission, presentation, and stage-aware grill / critique. Route natural academic requests even when the user does not explicitly invoke $qiongli-next or a slash command."
---

# Qiongli Core

General-purpose Qiongli academic workflow across paper types and methods.

Installed Qiongli workflow version: `v1.17.0-beta.1`

## Cross-Platform Trigger Contract

Qiongli should be considered whenever a request belongs to the academic research lifecycle. This does not require explicit `$qiongli-next`, `/paper`, `/lit-review`, or slash-command invocation.

Use Qiongli for:

- framing a research topic, research question, hypothesis, contribution, theory, or venue fit
- reading papers, PDFs, notes, citations, bibliographies, literature folders, or review matrices
- searching, screening, mapping, extracting, or synthesizing literature
- designing studies, variables, instruments, robustness checks, data plans, preregistration, or ethics materials
- writing or revising proposals, manuscript sections, abstracts, tables, figures, claims maps, or discussion text
- interpreting statistics, effect sizes, models, diagnostics, robustness checks, or analysis outputs
- reading or editing academic analysis code, notebooks, Stata scripts, R scripts, Python scripts, Quarto files, or replication packages
- checking PRISMA, reporting compliance, tone, citation support, originality, submission packages, rebuttals, peer review responses, or presentations

## Workflow Entry Points

Explicit workflow commands are optional entry points. In Codex, users can invoke this skill with `/skills` or `$qiongli-next`, but natural academic requests should also route here. Claude Code surfaces may expose the same workflows as slash-style command wrappers:

```
/paper [topic] [venue]                # Master router - choose paper type + task ID
/lit-review [topic] [year range]     # Systematic literature review (PRISMA)
/paper-read [URL or DOI]             # Deep paper analysis
/find-gap [research area]            # Identify research gaps
/build-framework [theory/concept]    # Build theoretical framework
/academic-write [section] [topic]    # Academic writing assistance
/synthesize [topic] [outcome_id]     # Evidence synthesis / meta-analysis
/paper-write [topic] [type] [venue]  # Full manuscript drafting
/study-design [topic]                # Empirical study design
/ethics-check [topic]                # Ethics / IRB pack
/submission-prep [topic] [venue]     # Submission package
/rebuttal [topic]                    # Rebuttal / response to reviewers
/code-build [method] --domain ...    # Build academic research code
/proofread [topic]                   # AI de-trace / final proofreading
/academic-present [topic]            # Academic presentation preparation
```

Full workflow definitions are included in `workflows/`. When a user names a workflow above, read `workflows/<command-name>.md` for the complete execution instructions.

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

- CLI, Codex, Claude Code, Antigravity, and Hermes installs can configure external literature providers with `qiongli provider setup` and audit them with `qiongli provider doctor`.
- In bundled MCP installs, do not expect the client MCP settings UI to inject provider keys into the plugin-bundled MCP server. Use `qiongli_config_status` to find the shared provider config path, then use `qiongli_configure_provider` to open a local browser setup page. Use `qiongli_save_provider_config` only for explicit scripted writes.
- Keep provider secrets out of `.mcp.json`, plugin manifests, release ZIPs, and research artifacts. The bundled provider server reads the shared provider config or explicit provider environment variables at runtime.
- Do not use `qiongli_collect_evidence` to judge built-in literature provider configuration. That tool is a filesystem/builtin/external-command evidence adapter; direct provider names such as `openalex` require a separate `RESEARCH_MCP_OPENALEX_CMD`. Use `qiongli_literature_status`, `qiongli_config_status`, `qiongli_test_provider`, and `qiongli_literature_search` to judge OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv provider availability.
- Treat `provider_connected` as the only mode where configured external academic provider credentials are available to the local runtime.
- Treat `strategy_only` as a constrained mode: draft the search strategy or use user-supplied corpus, record the limitation, and do not claim review-grade external provider or native-search coverage.
- Claude Desktop/Web focused ZIPs are skill-only packages kept within the 180-file upload budget. They contain workflows/prompts/templates, store no secrets, and cannot execute OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv API calls by themselves.
- For a manual Desktop install, upload the `qiongli-claude-desktop-skill-*.zip` first, then add a manual MCP install when provider calls or local orchestration are required. The skill ZIP supplies agent instructions, workflows/prompts/templates, and subject overlays; MCP supplies tool calls.
- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or another configured provider MCP before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv configuration/search. arXiv is enabled without credentials. Platform-native search alone is `native_only`, not `provider_connected`; if no provider MCP/MCPB and no platform-native search is available, record the run as `strategy_only`.
- The literature MCPB provides literature MCP tools only. It does not launch orchestrator agents. To expose the full agent runtime through MCP, manually install the full CLI MCP server with `qiongli mcp serve --transport stdio`; clients can then call tools such as `qiongli_task_run` after the local CLI runtime and model CLIs are configured.

## Runtime Subject Refinement

- Qiongli installs as an adaptive core workflow with `active_subject: auto` unless project guidance says otherwise.
- Use `standards/subject-refinement-contract.yaml` to classify no-subject, borrowed-lens, suggested, confirmed, and locked subject states.
- Treat a borrowed method lens as temporary method guidance. Do not switch the whole project subject from a single method signal.
- Use `subject_refinement.primary_subject` as the temporary subject only for `suggest_subject`, `confirm_subject`, or `lock_subject` decisions.
- Persist subject changes only through project-local guidance proposals, `subject_mode: confirmed`, or `subject_mode: locked`.

## Skill Loading Strategy

Use `skills-summary.md` for quick lookup, `skills-core.md` for consolidated guidance, and detailed files under `skills/` for active subject skills.
