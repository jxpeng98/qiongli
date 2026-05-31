# Academic Grill-Me Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Adapt the external grill-me pattern into Qiongli as an academic idea-discovery and boundary-critique loop, with explicit credit and workflow triggers.

**Architecture:** Extend the existing `boundary-interviewer` capability instead of adding a second generic interview skill. The contract defines an academic grill loop and stage question banks, while Stage A and workflow entry points invoke it before idea framing, gap discovery, and downstream scholarly commitments. Generated portable, plugin, Python, and npm payloads are synced from the canonical source files.

**Tech Stack:** Markdown skill cards, YAML workflow contracts, Python unittest/pytest contract tests, existing sync scripts.

---

### Task 1: Contract Tests

**Files:**
- Modify: `tests/test_boundary_interviewer_contract.py`
- Test: `tests/test_boundary_interviewer_contract.py`

- [ ] **Step 1: Write failing tests**

Add tests that assert:
- `standards/boundary-review-contract.yaml` declares an `academic_grill_loop`.
- Stage A contains idea-discovery prompts for moving from vague topic to defensible idea.
- `skills/Z_cross_cutting/boundary-interviewer.md` documents the academic adaptation and source credit.
- `/paper`, `/find-gap`, and Stage A reference docs trigger the academic grill loop.

- [ ] **Step 2: Verify red**

Run:

```bash
uv run --with pytest pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: FAIL because the academic grill loop and credit text do not exist yet.

### Task 2: Canonical Academic Adaptation

**Files:**
- Modify: `standards/boundary-review-contract.yaml`
- Modify: `skills/Z_cross_cutting/boundary-interviewer.md`
- Modify: `skills-core.md`
- Modify: `skills-summary.md`
- Modify: `README.md`
- Modify: `qiongli-workflow/references/stage-A-framing.md`
- Modify: `qiongli-workflow/workflows/paper.md`
- Modify: `qiongli-workflow/workflows/find-gap.md`

- [ ] **Step 1: Implement contract and source docs**

Add `academic_grill_loop` with academic principles:
- inspect artifacts before asking
- one scholarly question at a time
- recommended answer required
- adapt the question to paper type, claim strength, evidence threshold, rival explanations, and venue risk
- record credit to Matt Pocock's grill-me skill as an inspiration, not a copied workflow

- [ ] **Step 2: Implement workflow triggers**

Add Stage A and `/find-gap` triggers so vague topics run an academic grill loop before producing RQs, gap analyses, or contribution statements.

- [ ] **Step 3: Verify green**

Run:

```bash
uv run --with pytest pytest tests/test_boundary_interviewer_contract.py -q
```

Expected: PASS.

### Task 3: Sync Generated Packages

**Files:**
- Generated mirrors under `qiongli-workflow/`, `plugins/qiongli/skills/qiongli-workflow/`, `qiongli/payload/`, and `packages/npm-qiongli/`.

- [ ] **Step 1: Sync portable and plugin packages**

Run:

```bash
./scripts/sync_skill_package.sh --target all
```

- [ ] **Step 2: Sync npm and Python payloads**

Run:

```bash
uv run python scripts/sync_npm_package_payload.py
```

- [ ] **Step 3: Verify generated payloads**

Run:

```bash
uv run --with pytest pytest tests/test_distribution_payloads.py -q
```

Expected: PASS.

### Task 4: Final Verification And PR

**Files:**
- All modified source, tests, and generated package mirrors.

- [ ] **Step 1: Run targeted tests**

Run:

```bash
uv run --with pytest pytest tests/test_boundary_interviewer_contract.py tests/test_distribution_payloads.py -q
```

Expected: PASS.

- [ ] **Step 2: Inspect git diff**

Run:

```bash
git status --short
git diff --stat
```

Expected: only academic grill workflow, contract, tests, and synced payload files changed.

- [ ] **Step 3: Commit**

Use a Conventional Commit:

```bash
git add <changed-files>
git commit -m "feat(workflow): adapt grill-me for academic idea discovery"
```

- [ ] **Step 4: Push and open PR**

Run:

```bash
git push -u origin feat/academic-grill-me-workflow
gh pr create --base dev --head feat/academic-grill-me-workflow --title "feat(workflow): adapt grill-me for academic idea discovery" --body-file <generated-pr-body>
```
