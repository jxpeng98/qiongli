# Cross-Platform Routing Grill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Qiongli route academic work reliably across Codex, Claude / Claude Code, Gemini, CLI, and portable installs, with stage-aware grill behavior and stronger academic analysis-code constraints.

**Architecture:** The canonical routing and grill behavior lives in `content/workflow/` and cross-cutting skills. Platform manifests expose the same discovery language, and materialization syncs the canonical package to plugin, npm, Python, and portable payloads. Tests assert both source and materialized copies contain the routing contract.

**Tech Stack:** Markdown skill packages, JSON plugin manifests, Python unittest regression tests, existing `scripts/materialize_distribution_payloads.py` payload sync.

---

### Task 1: Add Regression Tests For Routing, Grill, Stage-I, And Metadata

**Files:**
- Modify: `tests/test_plugin_manifests.py`
- Create: `tests/test_cross_platform_routing_grill_contract.py`

- [ ] **Step 1: Add manifest discovery assertions**

Add assertions to `tests/test_plugin_manifests.py` that check Codex, Claude, Gemini, and Next manifests expose discovery terms for natural academic workflows.

Expected terms:

```python
DISCOVERY_TERMS = (
    "academic",
    "research",
    "literature",
    "manuscript",
    "analysis",
    "statistics",
    "reproducibility",
    "rebuttal",
)
```

For the Codex manifest, check `description`, `keywords`, `interface.longDescription`, and `interface.defaultPrompt`.

For Claude and Gemini manifests, check `description` and `keywords`.

- [ ] **Step 2: Add canonical routing contract tests**

Create `tests/test_cross_platform_routing_grill_contract.py` with tests that read:

```python
WORKFLOW_SKILL = LAYOUT.workflow / "SKILL.md"
PLATFORM_ROUTING = LAYOUT.workflow / "references" / "platform-routing.md"
BOUNDARY_INTERVIEWER = LAYOUT.skills / "Z_cross_cutting" / "boundary-interviewer.md"
SELF_CRITIQUE = LAYOUT.skills / "Z_cross_cutting" / "self-critique.md"
STAGE_I = LAYOUT.workflow / "references" / "stage-I-code.md"
CODE_BUILD = LAYOUT.workflow / "workflows" / "code-build.md"
CODE_BUILDER = LAYOUT.skills / "I_code" / "code-builder.md"
CODE_SPEC = LAYOUT.skills / "I_code" / "code-specification.md"
CODE_PLAN = LAYOUT.skills / "I_code" / "code-planning.md"
```

Assert the canonical texts contain:

```python
"Cross-Platform Trigger Contract"
"Ambiguity Trigger"
"Stage-Aware Grill Contract"
"Cross-Stage Grill Memory"
"Academic Analysis Code"
"estimand"
"dataset lineage"
"model diagnostics"
"manuscript-facing"
```

- [ ] **Step 3: Add materialized payload tests**

In the same test file, materialize plugin output to a temp directory:

```bash
python scripts/materialize_distribution_payloads.py --target plugin --out <tmp>/dist-source --force
```

Assert the materialized plugin skill contains:

```python
"Cross-Platform Trigger Contract"
"Ambiguity Trigger"
"Stage-Aware Grill Contract"
```

- [ ] **Step 4: Run targeted tests and verify failure**

Run:

```bash
python3 -m unittest tests.test_cross_platform_routing_grill_contract tests.test_plugin_manifests -v
```

Expected: failure before implementation because the new contract terms are missing.

### Task 2: Update Canonical Skill Routing

**Files:**
- Modify: `content/workflow/SKILL.md`
- Modify: `content/workflow/references/platform-routing.md`

- [ ] **Step 1: Expand skill description**

Update `content/workflow/SKILL.md` frontmatter description to include multi-platform natural academic tasks: papers, literature, study design, manuscript writing, statistics, analysis code, reproducibility, proofread, rebuttal, and presentations.

- [ ] **Step 2: Add Cross-Platform Trigger Contract**

Add a `## Cross-Platform Trigger Contract` section near the workflow entry points. Include trigger cases, non-trigger cases, ambiguity triggers, and platform-specific routing notes for Codex, Claude / Claude Code, Gemini, CLI, and portable installs.

- [ ] **Step 3: Replace platform-routing command-only mapping with natural routing**

Update `content/workflow/references/platform-routing.md` so it keeps the task-ID mapping but adds:

- natural language routing examples
- ambiguity trigger phrases in English and Chinese
- non-trigger guardrails
- platform metadata expectations
- the rule that explicit workflow commands are optional, not required

### Task 3: Make Grill Stage-Aware And Cross-Stage

**Files:**
- Modify: `content/skills/Z_cross_cutting/boundary-interviewer.md`
- Modify: `content/skills/Z_cross_cutting/self-critique.md`
- Modify: `content/workflow/references/stage-handoff-contract.md`

- [ ] **Step 1: Upgrade boundary-interviewer stage coverage**

Change `Related Task IDs` from MVP/future language to full-stage coverage: A, B, C, D, E, F, G, J, H, I, K.

- [ ] **Step 2: Add Stage-Aware Grill Contract**

Add rules for:

- light automatic grill on ambiguity, stage start, risky claim/method/evidence/code/submission changes, and stale handoff risks
- deep grill on explicit grill/stress-test/reviewer/fatal-flaw requests
- one question at a time
- recommended answer with academic rationale
- artifact inspection before asking

- [ ] **Step 3: Add stage lenses**

Add concise stage lens bullets for A, B, C, D, E, F, G, J, H, I, K matching the design spec.

- [ ] **Step 4: Add cross-stage memory rules**

Document how `boundary_review.md`, `decision_log.md`, `stage_handoff.md`, and `self_critique_log.md` carry resolved decisions and open issues forward.

- [ ] **Step 5: Align self-critique**

Update `self-critique.md` so stage-specific critique can be triggered by the same light/deep grill rules and must preserve issue lineage across stage handoffs.

- [ ] **Step 6: Update stage handoff contract**

Add grill-related required sections or rules: `Open Grill Issues`, `Resolved Grill Decisions`, and `Revisit Triggers`.

### Task 4: Strengthen Stage-I Academic Analysis Code Constraints

**Files:**
- Modify: `content/workflow/references/stage-I-code.md`
- Modify: `content/workflow/workflows/code-build.md`
- Modify: `content/skills/I_code/code-builder.md`
- Modify: `content/skills/I_code/code-specification.md`
- Modify: `content/skills/I_code/code-planning.md`

- [ ] **Step 1: Add Academic Analysis Code section**

Add guidance that academic code starts from estimand, hypothesis, analysis plan, or manuscript artifact, not application architecture.

- [ ] **Step 2: Add analysis pipeline constraints**

Require dataset lineage, sample construction, variable derivation, model diagnostics, robustness outputs, manuscript-facing tables/figures, seeds, dependency notes, and rerun commands.

- [ ] **Step 3: Add anti-patterns**

Explicitly discourage service layers, controllers, unnecessary classes, framework scaffolding, and backend-style abstraction when a script/notebook is the research-appropriate form.

- [ ] **Step 4: Update code specification template**

Add fields for `estimand`, `analysis_plan_source`, `dataset_lineage`, `manuscript_outputs`, `diagnostics`, and `robustness_checks` to the JSON contract block and headings.

- [ ] **Step 5: Update code planning template**

Add plan sections for data lineage checks, diagnostics, manuscript artifact generation, and rerun evidence.

### Task 5: Update Multi-Platform Metadata

**Files:**
- Modify: `packages/qiongli-plugin/.codex-plugin/plugin.json`
- Modify: `packages/qiongli-plugin/.claude-plugin/plugin.json`
- Modify: `packages/qiongli-plugin/gemini-extension.json`
- Modify: `packages/qiongli-next-plugin/.codex-plugin/plugin.json`

- [ ] **Step 1: Expand stable plugin metadata**

Update descriptions, keywords, and Codex default prompts to mention academic analysis, statistics, reproducibility, manuscripts, rebuttal, and stage-aware grill.

- [ ] **Step 2: Expand Claude metadata**

Update Claude plugin description and keywords with the same discovery terms.

- [ ] **Step 3: Expand Gemini metadata**

Update Gemini extension description and keywords with the same discovery terms.

- [ ] **Step 4: Expand Next metadata**

Update Qiongli Next Codex metadata with the same route/discovery language while preserving prerelease wording.

### Task 6: Materialize Distribution Payloads

**Files:**
- Generated / synced by script:
  - `qiongli-workflow/**`
  - `packages/qiongli-plugin/**`
  - `packages/qiongli-next-plugin/**`
  - `packages/npm-qiongli/**`
  - `packages/python-qiongli/src/qiongli/payload/**`

- [ ] **Step 1: Run in-place materialization**

Run:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --in-place
```

Expected: generated payloads sync and audit passes.

- [ ] **Step 2: Inspect changed files**

Run:

```bash
git status --short
```

Expected: canonical files, manifests, and materialized payloads changed; no unrelated files.

### Task 7: Verify And Commit

**Files:**
- All modified implementation, docs, metadata, generated payload, and tests.

- [ ] **Step 1: Run targeted tests**

Run:

```bash
python3 -m unittest tests.test_cross_platform_routing_grill_contract tests.test_plugin_manifests -v
```

Expected: pass.

- [ ] **Step 2: Run payload guard tests**

Run:

```bash
python3 -m unittest tests.test_generated_payload_guard tests.test_plugin_distribution_contract tests.test_plugin_manifests -v
```

Expected: pass.

- [ ] **Step 3: Run diff whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [ ] **Step 4: Review diff**

Run:

```bash
git diff --stat
git diff --name-only
```

Expected: only routing, grill, Stage-I, manifest, materialized payload, tests, and plan files.

- [ ] **Step 5: Commit**

Run:

```bash
git add content packages qiongli-workflow tests
git add -f docs/superpowers/plans/2026-06-12-cross-platform-routing-grill.md
git commit -m "feat(qiongli): add cross-platform routing grill contract"
```

Expected: one implementation commit after the design commit.
