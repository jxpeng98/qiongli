# Full-Cycle Multi-Agent Workflow And Journal Fit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a preview-first full-cycle academic paper workflow harness and a reverse journal-fit recommender that can evaluate an existing manuscript before submission.

**Architecture:** Extend the canonical workflow contract with a Stage H journal-fit task, add a deterministic Python lifecycle harness for stage gates and drift checks, then add a local venue-profile based journal recommender. Keep `/paper` as the single-task router and expose the full-cycle path through a new workflow plus Python full MCP/CLI preview tools.

**Tech Stack:** Python 3.12 standard library, PyYAML already used by Qiongli, unittest, Markdown workflow docs, YAML venue profiles, existing Qiongli bridge modules, existing MCP tool handler patterns.

---

## Source Spec

Implement:

- `docs/superpowers/specs/2026-07-04-full-cycle-multiagent-workflow-and-journal-fit-design.md`

## Desired Final Commit Shape

Keep implementation commits grouped by capability:

1. `docs(roadmap): prioritize full-cycle workflow harness`
   - roadmap reconciliation and design/plan files
2. `feat(workflow): add lifecycle harness contract`
   - H5 workflow contract, references, docs tests
3. `feat(harness): add full-cycle preview gates`
   - lifecycle harness module, deterministic fixtures, smoke script
4. `feat(journals): recommend venues from manuscript fit`
   - journal-fit module, tests, H5 output contract
5. `feat(mcp): expose lifecycle and journal fit preview tools`
   - MCP tool schemas, handler tests, CLI/reference docs
6. `test(harness): add lifecycle release checks`
   - beta smoke integration, release-readiness docs

Do not squash the whole feature into one commit.

## File Structure

Create:

- `content/workflow/references/full-cycle-workflow-harness.md`
  - Contract for lifecycle stage gates, drift checks, strong judge gates, and
    reverse journal-fit checkpoints.
- `content/workflow/workflows/paper-lifecycle.md`
  - User-facing workflow entrypoint for end-to-end planning and review.
- `content/skills/H_submission/journal-fit-recommender.md`
  - Skill card for manuscript-first journal recommendation.
- `packages/python-qiongli/src/qiongli/bridges/lifecycle_harness.py`
  - Deterministic lifecycle state, stage gate, drift check, and report builder.
- `packages/python-qiongli/src/qiongli/bridges/journal_fit.py`
  - Venue profile loading and manuscript-to-venue scoring.
- `tooling/scripts/run_full_cycle_workflow_harness.py`
  - Local deterministic smoke harness for lifecycle fixtures.
- `tests/fixtures/full_cycle_harness/clean_empirical/`
  - Minimal passing project fixture.
- `tests/fixtures/full_cycle_harness/missing_claim_evidence/`
  - Fixture that must block journal-fit readiness.
- `tests/fixtures/full_cycle_harness/drifted_research_question/`
  - Fixture that must detect drift from locked Stage A decisions.
- `tests/fixtures/full_cycle_harness/journal_overreach/`
  - Fixture that must reject an over-optimistic top-venue recommendation.
- `tests/test_lifecycle_harness.py`
  - Unit tests for stage gates, drift checks, report shape, and next-task output.
- `tests/test_journal_fit.py`
  - Unit tests for venue-profile scoring, blocked inputs, and ranked outputs.
- `tests/test_full_cycle_harness_script.py`
  - Script-level tests for fixture execution and JSON reports.

Modify:

- `content/standards/research-workflow-contract.yaml`
  - Add `H5` and output artifacts.
- `content/workflow/references/workflow-contract.md`
  - Regenerate after contract update.
- `content/workflow/references/stage-H-submission.md`
  - Document `H5` and its definition of done.
- `content/workflow/SKILL.md`
  - Mention `/paper-lifecycle` and reverse journal fit routing.
- `content/workflow/skills/registry.yaml`
  - Register `journal-fit-recommender` if the registry exists in the package.
- `docs/reference/skills.md`
  - Add the H5 skill entry.
- `docs/reference/cli.md`
  - Document lifecycle/journal MCP or CLI preview tools.
- `docs/guide/task-recipes.md`
  - Add full-cycle and manuscript-first journal recommendation recipes.
- `docs/zh/guide/task-recipes.md`
  - Chinese parity for task recipes.
- `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Reconcile current status and insert the new priority stage.
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Add Python full MCP schemas and dispatch for preview lifecycle and journal fit.
- `tests/test_mcp_tool_handlers.py`
  - Add MCP schema and dispatch tests.
- `tests/test_workflow_contract_doc.py`
  - Add H5 contract assertions.
- `tests/test_command_workflow_alignment.py`
  - Add `/paper-lifecycle` workflow alignment checks.
- `tooling/scripts/run_beta_smoke.sh`
  - Add deterministic preview harness check after it is stable.

---

## Task 0: Roadmap Reconciliation Baseline

**Files:**

- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Confirm branch and working tree**

Run:

```bash
git status --short --branch
```

Expected: branch is `dev`. Any unrelated uncommitted files must be left
untouched.

- [ ] **Step 2: Write the roadmap reconciliation**

Update the roadmap so it states:

```markdown
## Priority Update: Full-Cycle Workflow Harness

Status: next priority before additional subject expansion.

The local-agent smoke and subject-gate foundations are now partially present on
`dev`, but Qiongli still lacks an end-to-end lifecycle harness that proves
topic framing, broad evidence search, data/methods, writing, compliance,
strong judge, reverse journal fit, and feedback loops preserve the same locked
research state.

The next implementation priority is therefore a full-cycle multi-agent workflow
harness with reverse journal-fit recommendation. Subject expansion resumes
after this harness can detect stage drift, unsupported claims, unresolved
fatal flaws, and over-optimistic journal recommendations.
```

- [ ] **Step 3: Run markdown scan**

Run:

```bash
rg -n "Full-Cycle Workflow Harness|reverse journal-fit|subject expansion resumes" docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
```

Expected: all three phrases are found.

- [ ] **Step 4: Commit roadmap-only update if executed separately**

Run:

```bash
git add docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): prioritize full-cycle workflow harness"
```

Expected: commit succeeds.

## Task 1: Add H5 To The Workflow Contract

**Files:**

- Modify: `content/standards/research-workflow-contract.yaml`
- Modify: `content/workflow/references/workflow-contract.md`
- Modify: `content/workflow/references/stage-H-submission.md`
- Modify: `tests/test_workflow_contract_doc.py`

- [ ] **Step 1: Write failing contract tests**

Append tests to `tests/test_workflow_contract_doc.py`:

```python
    def test_stage_h_includes_reverse_journal_fit_task(self) -> None:
        contract = CONTRACT_PATH.read_text(encoding="utf-8")
        self.assertIn('id: "H5"', contract)
        self.assertIn("Reverse journal-fit recommendation", contract)
        self.assertIn("submission/journal_fit_recommendation.md", contract)

    def test_generated_workflow_doc_includes_h5(self) -> None:
        doc = GENERATED_DOC_PATH.read_text(encoding="utf-8")
        self.assertIn("`H5`", doc)
        self.assertIn("Reverse journal-fit recommendation", doc)
        self.assertIn("`submission/journal_fit_recommendation.md`", doc)
```

If the existing test file uses different constants, use the existing contract
and generated-doc path constants in that file instead of introducing duplicate
paths.

- [ ] **Step 2: Run failing tests**

Run:

```bash
uv run python -m unittest tests.test_workflow_contract_doc -v
```

Expected: FAIL because `H5` is not in the contract yet.

- [ ] **Step 3: Add H5 to YAML contract**

In `content/standards/research-workflow-contract.yaml`, add the H5 task under
Stage H:

```yaml
      - id: "H5"
        stage: "H"
        purpose: "Reverse journal-fit recommendation"
        primary_output:
          - "submission/journal_fit_recommendation.md"
          - "submission/journal_fit_recommendation.json"
```

Also add both files to `artifacts.required_core` if the contract keeps H-stage
submission artifacts there:

```yaml
    - "submission/journal_fit_recommendation.md"
    - "submission/journal_fit_recommendation.json"
```

- [ ] **Step 4: Regenerate generated workflow doc**

Run:

```bash
python3 tooling/scripts/generate_workflow_contract_doc.py
```

Expected: `content/workflow/references/workflow-contract.md` updates with H5.

- [ ] **Step 5: Update Stage H reference**

In `content/workflow/references/stage-H-submission.md`, add:

```markdown
## H5 - Reverse Journal-Fit Recommendation

Use H5 when a manuscript already exists and the question is which journal is
the best fit. H5 is manuscript-first. It must read the draft, contribution,
methods or evidence design, limitations, claim-evidence map, and venue profiles
before ranking journals.

**Definition of done**
- At least three candidate venues are assessed when the venue catalog permits.
- The report distinguishes primary, stretch, safe, fallback, and do-not-submit
  venues.
- Each recommendation states scope fit, contribution fit, method/evidence fit,
  reviewer risk, desk-reject risk, and required revisions.
- The report blocks a best-journal claim when the manuscript, methods, or
  claim-evidence map is missing.

Write into:
- `submission/journal_fit_recommendation.md`
- `submission/journal_fit_recommendation.json`
```

- [ ] **Step 6: Run contract tests**

Run:

```bash
uv run python -m unittest tests.test_workflow_contract_doc -v
```

Expected: PASS.

- [ ] **Step 7: Commit**

Run:

```bash
git add content/standards/research-workflow-contract.yaml content/workflow/references/workflow-contract.md content/workflow/references/stage-H-submission.md tests/test_workflow_contract_doc.py
git commit -m "feat(workflow): add reverse journal fit task"
```

Expected: commit succeeds.

## Task 2: Add Lifecycle Harness Contract And Workflow Entry

**Files:**

- Create: `content/workflow/references/full-cycle-workflow-harness.md`
- Create: `content/workflow/workflows/paper-lifecycle.md`
- Create: `content/skills/H_submission/journal-fit-recommender.md`
- Modify: `content/workflow/SKILL.md`
- Modify: `docs/reference/skills.md`
- Modify: `tests/test_command_workflow_alignment.py`

- [ ] **Step 1: Write failing workflow alignment test**

Add to `tests/test_command_workflow_alignment.py`:

```python
    def test_paper_lifecycle_workflow_is_packaged(self) -> None:
        workflow = WORKFLOW_ROOT / "workflows" / "paper-lifecycle.md"
        self.assertTrue(workflow.exists(), "missing /paper-lifecycle workflow")
        text = workflow.read_text(encoding="utf-8")
        self.assertIn("Full-Cycle", text)
        self.assertIn("stage_handoff.md", text)
        self.assertIn("journal_fit_recommendation.md", text)

    def test_journal_fit_recommender_skill_is_documented(self) -> None:
        skill = CONTENT_ROOT / "skills" / "H_submission" / "journal-fit-recommender.md"
        self.assertTrue(skill.exists(), "missing journal-fit-recommender skill")
        text = skill.read_text(encoding="utf-8")
        self.assertIn("manuscript-first", text)
        self.assertIn("do_not_submit", text)
```

Use the existing root constants in this test file. If the file does not define
`CONTENT_ROOT`, derive it from the same repository root constant used for
`WORKFLOW_ROOT`.

- [ ] **Step 2: Run failing tests**

Run:

```bash
uv run python -m unittest tests.test_command_workflow_alignment -v
```

Expected: FAIL because the files do not exist.

- [ ] **Step 3: Create lifecycle harness reference**

Create `content/workflow/references/full-cycle-workflow-harness.md`:

```markdown
# Full-Cycle Workflow Harness

The full-cycle harness keeps long-running paper work aligned across topic
framing, literature search, design, data, writing, compliance, review, journal
fit, and feedback.

## Required State Files

- `context/research_state.md`
- `context/decision_log.md`
- `context/boundary_review.md`
- `context/stage_handoff.md`
- `evidence/claim-evidence-ledger.csv`

## Gate Decisions

- `passed`: required artifacts and drift checks are satisfied.
- `blocked_missing_artifact`: a required artifact is absent.
- `blocked_unresolved_boundary`: a locked decision or boundary is missing.
- `blocked_unresolved_judge`: H3 or H4 found a major issue that remains open.
- `reopened_by_revisit_trigger`: new evidence or feedback invalidated an
  earlier decision.

## Drift Checks

- The manuscript preserves the locked research question.
- Claims in `manuscript/manuscript.md` map to
  `evidence/claim-evidence-ledger.csv`.
- Journal recommendations account for fatal flaws and limitations.
- Revision promises point to feasible artifacts or are marked as blocked.

## Strong Judge Rule

The strong judge can pass, revise, block submission, or reopen a stage. It must
not draft manuscript text directly.
```

- [ ] **Step 4: Create `/paper-lifecycle` workflow**

Create `content/workflow/workflows/paper-lifecycle.md`:

```markdown
---
description: Full-cycle academic paper workflow harness from topic selection to journal fit and feedback.
---

# Full-Cycle Paper Lifecycle Workflow

Use this workflow when the user wants Qiongli to coordinate the whole paper
pipeline rather than one isolated task.

## Inputs

$ARGUMENTS

## Contract

Read `references/workflow-contract.md`,
`references/stage-handoff-contract.md`, and
`references/full-cycle-workflow-harness.md` before producing a lifecycle plan.

## Default Mode

Default to preview. Do not launch local agents unless the caller explicitly
sets `run_agents: true`.

## Required Checkpoints

1. Stage A: topic, research question, contribution, boundary review, initial
   venue assumptions.
2. Stage B: broad literature search, search logs, full-text status, Zotero or
   retrieval evidence where available.
3. Stage C/I: study design, data plan, analysis or reproducibility status.
4. Stage F: manuscript outline, draft, claim-evidence map.
5. Stage G/J: compliance, cross-section integrity, proofreading.
6. Stage H: peer review simulation, fatal flaw analysis, H5 reverse journal fit.
7. Feedback loop: response matrix, revision plan, stage reopen decisions.

## Output

Produce a lifecycle plan that lists:

- current lifecycle status,
- passed and blocked stage gates,
- missing artifacts,
- drift risks,
- recommended next task IDs,
- whether H5 journal fit is ready.
```

- [ ] **Step 5: Create journal-fit skill card**

Create `content/skills/H_submission/journal-fit-recommender.md`:

```markdown
---
id: journal-fit-recommender
stage: H_submission
description: "Recommend journals from an existing manuscript using venue profile, contribution, methods, evidence, and reviewer-risk fit."
inputs:
  - type: Manuscript
    description: "Current manuscript draft or structured manuscript sections"
  - type: ClaimEvidenceMap
    description: "Claim to evidence mapping for manuscript claims"
  - type: VenueProfileCatalog
    description: "Local venue profiles and subject-specific venue profiles"
outputs:
  - type: JournalFitRecommendation
    artifact: "submission/journal_fit_recommendation.md"
constraints:
  - "Must be manuscript-first, not target-first"
  - "Must block best-journal claims when manuscript evidence is missing"
  - "Must classify venues as primary, stretch, safe, fallback, or do_not_submit"
tools: [filesystem]
tags: [submission, journal-selection, venue-fit, manuscript-review]
domain_aware: true
---

# Journal Fit Recommender

Recommend journals for an existing manuscript.

## Required Inputs

- Manuscript draft.
- Research question and contribution statement.
- Methods, data, or evidence design summary.
- Claim-evidence map.
- Limitations or fatal flaw report when available.
- Venue profiles.

## Output Contract

Write `RESEARCH/[topic]/submission/journal_fit_recommendation.md` with:

| Venue | Class | Scope fit | Contribution fit | Method/evidence fit | Reviewer risk | Required revision |
|---|---|---|---|---|---|---|

Use these classes:

- `primary`
- `stretch`
- `safe`
- `fallback`
- `do_not_submit`

Block recommendation when the manuscript or claim-evidence map is missing.
```

- [ ] **Step 6: Update skill and reference docs**

In `content/workflow/SKILL.md`, add `/paper-lifecycle` to the workflow entry
points and explain that H5 handles manuscript-first journal recommendation.

In `docs/reference/skills.md`, add `journal-fit-recommender` under Stage H.

- [ ] **Step 7: Run workflow alignment tests**

Run:

```bash
uv run python -m unittest tests.test_command_workflow_alignment -v
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add content/workflow/references/full-cycle-workflow-harness.md content/workflow/workflows/paper-lifecycle.md content/skills/H_submission/journal-fit-recommender.md content/workflow/SKILL.md docs/reference/skills.md tests/test_command_workflow_alignment.py
git commit -m "feat(workflow): add full-cycle lifecycle entrypoint"
```

Expected: commit succeeds.

## Task 3: Lifecycle Harness Core

**Files:**

- Create: `packages/python-qiongli/src/qiongli/bridges/lifecycle_harness.py`
- Create: `tests/test_lifecycle_harness.py`

- [ ] **Step 1: Write failing lifecycle tests**

Create `tests/test_lifecycle_harness.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from qiongli.bridges.lifecycle_harness import (
    build_lifecycle_report,
    evaluate_stage_gate,
)


class LifecycleHarnessTests(unittest.TestCase):
    def test_stage_gate_blocks_missing_required_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            result = evaluate_stage_gate(project, "B")

        self.assertEqual(result["stage"], "B")
        self.assertEqual(result["status"], "blocked_missing_artifact")
        self.assertIn("search_strategy.md", result["missing_artifacts"])
        self.assertIn("search_log.md", result["missing_artifacts"])

    def test_clean_empirical_report_recommends_next_h5_when_submission_ready(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write(project / "context" / "research_state.md", "RQ: Does X affect Y?")
            _write(project / "context" / "decision_log.md", "decision_id,stage,decision\nA1,A,RQ locked\n")
            _write(project / "context" / "boundary_review.md", "claim strength: associative")
            _write(project / "context" / "stage_handoff.md", "Completed Artifacts\n")
            _write(project / "search_strategy.md", "query: x y")
            _write(project / "search_log.md", "provider: fixture")
            _write(project / "search_results.csv", "title,doi\nA,10.1/a\n")
            _write(project / "dedup_log.csv", "record_id,decision\n1,keep\n")
            _write(project / "retrieval_manifest.csv", "record_id,retrieval_status\n1,abstract_only\n")
            _write(project / "study_design.md", "empirical design")
            _write(project / "analysis_plan.md", "model: y = x")
            _write(project / "manuscript" / "manuscript.md", "# Manuscript\nDoes X affect Y?")
            _write(project / "evidence" / "claim-evidence-ledger.csv", "claim_id,claim,evidence_status\nc1,Does X affect Y?,supported\n")
            _write(project / "reporting_checklist.md", "ready")
            _write(project / "proofread" / "proofread_checklist.md", "ready")
            _write(project / "revision" / "peer_review_simulation.md", "no major flaws")
            _write(project / "revision" / "fatal_flaw_analysis.md", "Decision: pass")

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertEqual(report["schema_version"], "1.0")
        self.assertEqual(report["lifecycle_status"], "ready_for_h5")
        self.assertIn("H5", report["recommended_next_tasks"])

    def test_report_detects_research_question_drift(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project = Path(tmp_dir)
            _write(project / "context" / "research_state.md", "RQ: Does X affect Y?")
            _write(project / "manuscript" / "manuscript.md", "# Manuscript\nThis paper studies A and B.")
            _write(project / "evidence" / "claim-evidence-ledger.csv", "claim_id,claim,evidence_status\nc1,A and B,supported\n")

            report = build_lifecycle_report(project, topic="demo", paper_type="empirical")

        self.assertFalse(report["drift_checks"]["locked_question_preserved"])
        self.assertIn("research_question_drift", report["blocking_reasons"])


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run failing tests**

Run:

```bash
uv run python -m unittest tests.test_lifecycle_harness -v
```

Expected: FAIL because `lifecycle_harness` does not exist.

- [ ] **Step 3: Implement lifecycle harness module**

Create `packages/python-qiongli/src/qiongli/bridges/lifecycle_harness.py`:

```python
from __future__ import annotations

import csv
from pathlib import Path
from typing import Any


STAGE_REQUIRED_ARTIFACTS: dict[str, list[str]] = {
    "A": [
        "context/research_state.md",
        "context/decision_log.md",
        "context/boundary_review.md",
        "framing/research_question.md",
        "framing/contribution_statement.md",
    ],
    "B": [
        "search_strategy.md",
        "search_log.md",
        "search_results.csv",
        "dedup_log.csv",
        "retrieval_manifest.csv",
    ],
    "C": [
        "study_design.md",
        "analysis_plan.md",
    ],
    "F": [
        "manuscript/manuscript.md",
        "evidence/claim-evidence-ledger.csv",
    ],
    "GJ": [
        "reporting_checklist.md",
        "proofread/proofread_checklist.md",
    ],
    "H": [
        "revision/peer_review_simulation.md",
        "revision/fatal_flaw_analysis.md",
    ],
}


def evaluate_stage_gate(project_root: Path | str, stage: str) -> dict[str, Any]:
    root = Path(project_root)
    required = STAGE_REQUIRED_ARTIFACTS.get(stage, [])
    missing = [path for path in required if not (root / path).exists()]
    status = "passed" if not missing else "blocked_missing_artifact"
    return {
        "stage": stage,
        "status": status,
        "required_artifacts": list(required),
        "missing_artifacts": missing,
        "warnings": [],
    }


def build_lifecycle_report(
    project_root: Path | str,
    *,
    topic: str,
    paper_type: str,
    mode: str = "preview",
) -> dict[str, Any]:
    root = Path(project_root)
    gates = [
        evaluate_stage_gate(root, stage)
        for stage in ("A", "B", "C", "F", "GJ", "H")
    ]
    blocking_reasons: list[str] = []
    for gate in gates:
        if gate["status"] != "passed":
            blocking_reasons.append(f"{gate['stage']}:missing_artifact")

    drift_checks = _drift_checks(root)
    if not drift_checks["locked_question_preserved"]:
        blocking_reasons.append("research_question_drift")
    if drift_checks["claim_evidence_coverage"] == "missing":
        blocking_reasons.append("missing_claim_evidence")
    if drift_checks["unresolved_judge_blocks"] > 0:
        blocking_reasons.append("unresolved_judge_blocks")

    if blocking_reasons:
        lifecycle_status = "blocked_missing_artifact"
        recommended_next = _recommended_next_tasks(gates, drift_checks)
    else:
        lifecycle_status = "ready_for_h5"
        recommended_next = ["H5"]

    return {
        "schema_version": "1.0",
        "mode": mode,
        "topic": topic,
        "paper_type": paper_type,
        "lifecycle_status": lifecycle_status,
        "stage_gates": gates,
        "drift_checks": drift_checks,
        "journal_fit": {
            "status": "not_run",
            "primary": None,
            "blocking_reasons": [],
        },
        "blocking_reasons": blocking_reasons,
        "recommended_next_tasks": recommended_next,
    }


def _drift_checks(root: Path) -> dict[str, Any]:
    research_state = _read(root / "context" / "research_state.md")
    manuscript = _read(root / "manuscript" / "manuscript.md")
    locked_question_preserved = True
    if "RQ:" in research_state and manuscript:
        question = research_state.split("RQ:", 1)[1].splitlines()[0].strip()
        tokens = [token.lower() for token in question.replace("?", "").split() if len(token) > 2]
        matched = sum(1 for token in tokens if token in manuscript.lower())
        locked_question_preserved = matched >= max(1, len(tokens) // 2)

    ledger_path = root / "evidence" / "claim-evidence-ledger.csv"
    coverage = _claim_evidence_coverage(ledger_path)
    judge_blocks = _unresolved_judge_blocks(root)
    return {
        "locked_question_preserved": locked_question_preserved,
        "claim_evidence_coverage": coverage,
        "unresolved_judge_blocks": judge_blocks,
    }


def _claim_evidence_coverage(path: Path) -> str:
    if not path.exists():
        return "missing"
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        return "missing"
    statuses = {str(row.get("evidence_status", "")).strip().lower() for row in rows}
    if statuses <= {"supported", "ready", "verified"}:
        return "complete"
    return "partial"


def _unresolved_judge_blocks(root: Path) -> int:
    fatal = _read(root / "revision" / "fatal_flaw_analysis.md").lower()
    if "block_submission" in fatal or "decision: block" in fatal:
        return 1
    return 0


def _recommended_next_tasks(gates: list[dict[str, Any]], drift_checks: dict[str, Any]) -> list[str]:
    if not drift_checks["locked_question_preserved"]:
        return ["A1", "A2"]
    if drift_checks["claim_evidence_coverage"] == "missing":
        return ["F4"]
    if drift_checks["unresolved_judge_blocks"] > 0:
        return ["H4"]
    for gate in gates:
        if gate["status"] != "passed":
            return [_first_task_for_stage(str(gate["stage"]))]
    return ["H5"]


def _first_task_for_stage(stage: str) -> str:
    return {
        "A": "A1",
        "B": "B1",
        "C": "C1",
        "F": "F3",
        "GJ": "G3",
        "H": "H3",
    }.get(stage, "A1")


def _read(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="replace")
```

- [ ] **Step 4: Run lifecycle tests**

Run:

```bash
uv run python -m unittest tests.test_lifecycle_harness -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/lifecycle_harness.py tests/test_lifecycle_harness.py
git commit -m "feat(harness): add lifecycle preview gates"
```

Expected: commit succeeds.

## Task 4: Reverse Journal-Fit Recommender

**Files:**

- Create: `packages/python-qiongli/src/qiongli/bridges/journal_fit.py`
- Create: `tests/test_journal_fit.py`

- [ ] **Step 1: Write failing journal fit tests**

Create `tests/test_journal_fit.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import yaml

from qiongli.bridges.journal_fit import recommend_journals


class JournalFitTests(unittest.TestCase):
    def test_blocks_when_manuscript_is_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write_venue(root / "venues" / "journal-a.yaml", "journal-a", ["finance"])

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "blocked")
        self.assertIn("missing manuscript/manuscript.md", report["blocking_reasons"])
        self.assertEqual(report["ranked_venues"], [])

    def test_ranks_primary_and_safe_venues_from_profile_fit(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(root / "manuscript" / "manuscript.md", "Finance event study with abnormal returns and CRSP data.")
            _write(root / "framing" / "contribution_statement.md", "Contribution to finance evidence.")
            _write(root / "study_design.md", "Event-study design with abnormal returns.")
            _write(root / "evidence" / "claim-evidence-ledger.csv", "claim_id,claim,evidence_status\nc1,abnormal returns,supported\n")
            _write_venue(root / "venues" / "journal-of-finance.yaml", "journal-of-finance", ["finance", "event study", "abnormal returns"])
            _write_venue(root / "venues" / "general-management.yaml", "general-management", ["management", "qualitative"])

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "ok")
        self.assertEqual(report["ranked_venues"][0]["venue_id"], "journal-of-finance")
        self.assertEqual(report["ranked_venues"][0]["class"], "primary")
        self.assertEqual(report["ranked_venues"][-1]["class"], "do_not_submit")

    def test_marks_stretch_when_fit_is_high_but_fatal_flaw_exists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            _write(root / "manuscript" / "manuscript.md", "Finance event study with abnormal returns and CRSP data.")
            _write(root / "framing" / "contribution_statement.md", "Contribution to finance evidence.")
            _write(root / "study_design.md", "Event-study design.")
            _write(root / "evidence" / "claim-evidence-ledger.csv", "claim_id,claim,evidence_status\nc1,abnormal returns,partial\n")
            _write(root / "revision" / "fatal_flaw_analysis.md", "Decision: block_submission")
            _write_venue(root / "venues" / "journal-of-finance.yaml", "journal-of-finance", ["finance", "event study", "abnormal returns"])

            report = recommend_journals(root, venue_roots=[root / "venues"])

        self.assertEqual(report["status"], "ok")
        self.assertEqual(report["ranked_venues"][0]["class"], "stretch")
        self.assertIn("unresolved fatal flaw", report["ranked_venues"][0]["reviewer_risk"])


def _write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _write_venue(path: Path, venue_id: str, keywords: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        yaml.safe_dump(
            {
                "venue_id": venue_id,
                "community": keywords[0],
                "article_types": ["research article"],
                "contribution_expectations": keywords,
                "methods_expectations": keywords,
                "evidence_standards": keywords,
                "writing_style": ["direct"],
                "common_reviewer_objections": ["weak fit"],
                "formatting_constraints": {"word_limit": 12000},
                "required_reporting_standards": [],
            },
            sort_keys=False,
        ),
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run failing journal tests**

Run:

```bash
uv run python -m unittest tests.test_journal_fit -v
```

Expected: FAIL because `journal_fit` does not exist.

- [ ] **Step 3: Implement journal fit module**

Create `packages/python-qiongli/src/qiongli/bridges/journal_fit.py`:

```python
from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml


REQUIRED_INPUTS = [
    "manuscript/manuscript.md",
    "framing/contribution_statement.md",
    "study_design.md",
    "evidence/claim-evidence-ledger.csv",
]


def recommend_journals(
    project_root: Path | str,
    *,
    venue_roots: list[Path | str],
    limit: int = 5,
) -> dict[str, Any]:
    root = Path(project_root)
    missing = [path for path in REQUIRED_INPUTS if not (root / path).exists()]
    if "manuscript/manuscript.md" in missing:
        return {
            "schema_version": "1.0",
            "status": "blocked",
            "blocking_reasons": [f"missing {path}" for path in missing],
            "ranked_venues": [],
        }

    manuscript_text = _project_text(root)
    fatal_flaw = _has_unresolved_fatal_flaw(root)
    venues = _load_venues([Path(path) for path in venue_roots])
    ranked = [
        _score_venue(venue, manuscript_text, fatal_flaw=fatal_flaw)
        for venue in venues
    ]
    ranked.sort(key=lambda item: item["score"], reverse=True)
    return {
        "schema_version": "1.0",
        "status": "ok",
        "blocking_reasons": [f"missing {path}" for path in missing],
        "ranked_venues": ranked[: max(1, limit)],
    }


def _project_text(root: Path) -> str:
    parts = []
    for rel in REQUIRED_INPUTS:
        path = root / rel
        if path.exists():
            parts.append(path.read_text(encoding="utf-8", errors="replace"))
    return "\n".join(parts).lower()


def _load_venues(roots: list[Path]) -> list[dict[str, Any]]:
    venues: list[dict[str, Any]] = []
    for root in roots:
        if root.is_file():
            paths = [root]
        else:
            paths = sorted(root.glob("*.yaml"))
        for path in paths:
            payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
            if isinstance(payload, dict) and payload.get("venue_id"):
                payload["_source"] = str(path)
                venues.append(payload)
    return venues


def _score_venue(
    venue: dict[str, Any],
    manuscript_text: str,
    *,
    fatal_flaw: bool,
) -> dict[str, Any]:
    keywords = _venue_keywords(venue)
    matched = [keyword for keyword in keywords if keyword and keyword.lower() in manuscript_text]
    denominator = max(1, len(keywords))
    score = round(len(matched) / denominator, 3)
    classification = _classify(score, fatal_flaw=fatal_flaw)
    reviewer_risk = "unresolved fatal flaw" if fatal_flaw else _reviewer_risk(score)
    return {
        "venue_id": str(venue["venue_id"]),
        "class": classification,
        "score": score,
        "scope_fit": _fit_label(score),
        "contribution_fit": _fit_label(score),
        "method_evidence_fit": _fit_label(score),
        "reviewer_risk": reviewer_risk,
        "desk_reject_risk": "high" if score < 0.25 else "medium" if score < 0.60 else "low",
        "matched_terms": matched,
        "required_revision": _required_revision(score, fatal_flaw=fatal_flaw),
        "source": str(venue.get("_source", "")),
    }


def _venue_keywords(venue: dict[str, Any]) -> list[str]:
    values: list[str] = []
    for key in (
        "community",
        "article_types",
        "contribution_expectations",
        "methods_expectations",
        "evidence_standards",
        "writing_style",
    ):
        raw = venue.get(key, [])
        if isinstance(raw, str):
            values.append(raw)
        elif isinstance(raw, list):
            values.extend(str(item) for item in raw)
    return sorted({value.strip().lower() for value in values if value.strip()})


def _classify(score: float, *, fatal_flaw: bool) -> str:
    if score < 0.25:
        return "do_not_submit"
    if fatal_flaw and score >= 0.60:
        return "stretch"
    if score >= 0.70:
        return "primary"
    if score >= 0.45:
        return "safe"
    return "fallback"


def _fit_label(score: float) -> str:
    if score >= 0.70:
        return "strong"
    if score >= 0.45:
        return "moderate"
    if score >= 0.25:
        return "weak"
    return "poor"


def _reviewer_risk(score: float) -> str:
    if score >= 0.70:
        return "fit risk is low; evaluate contribution strength"
    if score >= 0.45:
        return "fit is plausible but positioning revision is needed"
    return "scope or method fit is weak"


def _required_revision(score: float, *, fatal_flaw: bool) -> str:
    if fatal_flaw:
        return "Resolve fatal flaw before submission."
    if score >= 0.70:
        return "Tighten venue-facing contribution and formatting."
    if score >= 0.45:
        return "Revise framing to match venue scope and methods expectations."
    return "Choose a different venue or substantially re-scope the manuscript."


def _has_unresolved_fatal_flaw(root: Path) -> bool:
    path = root / "revision" / "fatal_flaw_analysis.md"
    if not path.exists():
        return False
    text = path.read_text(encoding="utf-8", errors="replace").lower()
    return "block_submission" in text or "decision: block" in text
```

- [ ] **Step 4: Run journal tests**

Run:

```bash
uv run python -m unittest tests.test_journal_fit -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/journal_fit.py tests/test_journal_fit.py
git commit -m "feat(journals): recommend venues from manuscript fit"
```

Expected: commit succeeds.

## Task 5: Full-Cycle Harness Script And Fixtures

**Files:**

- Create: `tooling/scripts/run_full_cycle_workflow_harness.py`
- Create: `tests/test_full_cycle_harness_script.py`
- Create: `tests/fixtures/full_cycle_harness/clean_empirical/`
- Create: `tests/fixtures/full_cycle_harness/missing_claim_evidence/`
- Create: `tests/fixtures/full_cycle_harness/drifted_research_question/`
- Create: `tests/fixtures/full_cycle_harness/journal_overreach/`

- [ ] **Step 1: Write failing script tests**

Create `tests/test_full_cycle_harness_script.py`:

```python
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.run_full_cycle_workflow_harness import main


class FullCycleHarnessScriptTests(unittest.TestCase):
    def test_clean_fixture_returns_zero(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = Path(tmp_dir) / "report.json"
            exit_code = main([
                "--fixture",
                "tests/fixtures/full_cycle_harness/clean_empirical",
                "--json-report",
                str(report),
            ])

        self.assertEqual(exit_code, 0)
        payload = json.loads(report.read_text(encoding="utf-8"))
        self.assertEqual(payload["lifecycle_status"], "ready_for_h5")

    def test_drift_fixture_returns_one(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = Path(tmp_dir) / "report.json"
            exit_code = main([
                "--fixture",
                "tests/fixtures/full_cycle_harness/drifted_research_question",
                "--json-report",
                str(report),
            ])

        self.assertEqual(exit_code, 1)
        payload = json.loads(report.read_text(encoding="utf-8"))
        self.assertIn("research_question_drift", payload["blocking_reasons"])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Create minimal fixtures**

Create these files under `tests/fixtures/full_cycle_harness/clean_empirical/`:

```text
context/research_state.md
context/decision_log.md
context/boundary_review.md
context/stage_handoff.md
framing/research_question.md
framing/contribution_statement.md
search_strategy.md
search_log.md
search_results.csv
dedup_log.csv
retrieval_manifest.csv
study_design.md
analysis_plan.md
manuscript/manuscript.md
evidence/claim-evidence-ledger.csv
reporting_checklist.md
proofread/proofread_checklist.md
revision/peer_review_simulation.md
revision/fatal_flaw_analysis.md
venues/journal-of-finance.yaml
venues/general-management.yaml
```

Use these contents for the key files:

```markdown
# context/research_state.md
RQ: Does CRSP event exposure affect abnormal returns?
```

```markdown
# manuscript/manuscript.md
# Manuscript

This manuscript asks whether CRSP event exposure affects abnormal returns.
```

```csv
claim_id,claim,evidence_status
c1,CRSP event exposure affects abnormal returns,supported
```

Copy this fixture to `drifted_research_question/` and change only
`manuscript/manuscript.md` to:

```markdown
# Manuscript

This manuscript studies employee wellbeing in remote organizations.
```

Copy this fixture to `missing_claim_evidence/` and remove
`evidence/claim-evidence-ledger.csv`.

Copy this fixture to `journal_overreach/` and change
`revision/fatal_flaw_analysis.md` to:

```markdown
# Fatal Flaw Analysis

Decision: block_submission
```

- [ ] **Step 3: Implement script**

Create `tooling/scripts/run_full_cycle_workflow_harness.py`:

```python
#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SRC = REPO_ROOT / "packages" / "python-qiongli" / "src"
if str(PYTHON_SRC) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC))

from qiongli.bridges.journal_fit import recommend_journals  # noqa: E402
from qiongli.bridges.lifecycle_harness import build_lifecycle_report  # noqa: E402


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run deterministic full-cycle Qiongli harness fixtures.")
    parser.add_argument("--fixture", required=True, help="Fixture project directory.")
    parser.add_argument("--json-report", required=True, help="Output JSON report path.")
    parser.add_argument("--topic", default="full-cycle-fixture")
    parser.add_argument("--paper-type", default="empirical")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    fixture = Path(args.fixture).resolve()
    report_path = Path(args.json_report).resolve()
    with tempfile.TemporaryDirectory() as tmp_dir:
        project = Path(tmp_dir) / "project"
        shutil.copytree(fixture, project)
        report = build_lifecycle_report(project, topic=args.topic, paper_type=args.paper_type)
        venues = project / "venues"
        if venues.exists():
            report["journal_fit"] = recommend_journals(project, venue_roots=[venues])
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    return 0 if not report.get("blocking_reasons") else 1


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Run script tests**

Run:

```bash
uv run python -m unittest tests.test_full_cycle_harness_script -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add tooling/scripts/run_full_cycle_workflow_harness.py tests/test_full_cycle_harness_script.py tests/fixtures/full_cycle_harness
git commit -m "test(harness): add full-cycle workflow fixtures"
```

Expected: commit succeeds.

## Task 6: MCP Preview Tools

**Files:**

- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`
- Modify: `docs/reference/cli.md`

- [ ] **Step 1: Write failing MCP tests**

Add tests to `tests/test_mcp_tool_handlers.py`:

```python
    def test_lifecycle_plan_tool_returns_preview_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / "manuscript").mkdir()
            (root / "manuscript" / "manuscript.md").write_text("draft", encoding="utf-8")

            result = call_qiongli_tool(
                "qiongli_lifecycle_plan",
                {"cwd": str(root), "topic": "demo", "paper_type": "empirical"},
            )

        self.assertFalse(result.get("isError"), result)
        payload = result["structuredContent"]
        self.assertEqual(payload["schema_version"], "1.0")
        self.assertEqual(payload["mode"], "preview")

    def test_journal_fit_tool_blocks_missing_manuscript(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            result = call_qiongli_tool(
                "qiongli_journal_fit_recommend",
                {"cwd": str(root), "venue_roots": []},
            )

        self.assertFalse(result.get("isError"), result)
        payload = result["structuredContent"]
        self.assertEqual(payload["status"], "blocked")
        self.assertIn("missing manuscript/manuscript.md", payload["blocking_reasons"])
```

If `call_qiongli_tool` returns a different wrapper shape in the current file,
match the existing tests' expected wrapper.

- [ ] **Step 2: Run failing MCP tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers -v
```

Expected: FAIL because the tools are not registered.

- [ ] **Step 3: Add MCP tool schemas**

In `mcp_tool_handlers.py`, add schemas for:

```python
{
    "name": "qiongli_lifecycle_plan",
    "description": "Build a preview full-cycle paper lifecycle gate report without launching agents.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "cwd": {"type": "string"},
            "topic": {"type": "string"},
            "paper_type": {"type": "string"},
            "mode": {"type": "string", "enum": ["preview"]},
        },
    },
}
```

and:

```python
{
    "name": "qiongli_journal_fit_recommend",
    "description": "Recommend journals from an existing manuscript using local venue profiles.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "cwd": {"type": "string"},
            "venue_roots": {
                "type": "array",
                "items": {"type": "string"},
            },
            "limit": {"type": "integer"},
        },
    },
}
```

- [ ] **Step 4: Add MCP dispatch**

In the tool dispatch function, add:

```python
    if name == "qiongli_lifecycle_plan":
        from .lifecycle_harness import build_lifecycle_report

        cwd = Path(str(arguments.get("cwd") or Path.cwd()))
        payload = build_lifecycle_report(
            cwd,
            topic=str(arguments.get("topic") or cwd.name),
            paper_type=str(arguments.get("paper_type") or "empirical"),
            mode="preview",
        )
        return _structured_tool_result(payload)

    if name == "qiongli_journal_fit_recommend":
        from .journal_fit import recommend_journals

        cwd = Path(str(arguments.get("cwd") or Path.cwd()))
        raw_roots = arguments.get("venue_roots") or []
        venue_roots = [Path(str(item)) for item in raw_roots]
        if not venue_roots:
            venue_roots = [cwd / "venues"]
        payload = recommend_journals(
            cwd,
            venue_roots=venue_roots,
            limit=int(arguments.get("limit") or 5),
        )
        return _structured_tool_result(payload)
```

Use the existing helper name for structured results if it differs from
`_structured_tool_result`.

- [ ] **Step 5: Update CLI/reference docs**

In `docs/reference/cli.md`, add a short section:

```markdown
### Full-cycle preview tools

- `qiongli_lifecycle_plan`: builds a preview stage-gate report for an existing
  paper project. It does not launch agents.
- `qiongli_journal_fit_recommend`: ranks journals from an existing manuscript
  and local venue profiles. It blocks when manuscript evidence is missing.
```

- [ ] **Step 6: Run MCP tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers -v
```

Expected: PASS.

- [ ] **Step 7: Commit**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py docs/reference/cli.md
git commit -m "feat(mcp): expose lifecycle and journal fit previews"
```

Expected: commit succeeds.

## Task 7: Release Harness Integration

**Files:**

- Modify: `tooling/scripts/run_beta_smoke.sh`
- Modify: `docs/advanced/publish-pypi.md`
- Modify: `docs/zh/advanced/publish-pypi.md`

- [ ] **Step 1: Add beta smoke command**

In `tooling/scripts/run_beta_smoke.sh`, add a deterministic preview command:

```bash
python3 tooling/scripts/run_full_cycle_workflow_harness.py \
  --fixture tests/fixtures/full_cycle_harness/clean_empirical \
  --json-report "${TMPDIR:-/tmp}/qiongli-full-cycle-harness.json"
```

Place it after existing deterministic smoke checks and before any optional
local-agent or provider-connected checks.

- [ ] **Step 2: Document maintainer command**

In `docs/advanced/publish-pypi.md`, add:

```markdown
Optional full-cycle workflow harness:

```bash
python3 tooling/scripts/run_full_cycle_workflow_harness.py \
  --fixture tests/fixtures/full_cycle_harness/clean_empirical \
  --json-report /tmp/qiongli-full-cycle-harness.json
```

This is preview-only. It verifies stage gates, drift checks, and journal-fit
readiness without launching local agents.
```

Use the same content in Chinese in `docs/zh/advanced/publish-pypi.md`.

- [ ] **Step 3: Run focused harness verification**

Run:

```bash
python3 tooling/scripts/run_full_cycle_workflow_harness.py --fixture tests/fixtures/full_cycle_harness/clean_empirical --json-report /tmp/qiongli-full-cycle-harness.json
python3 tooling/scripts/run_full_cycle_workflow_harness.py --fixture tests/fixtures/full_cycle_harness/drifted_research_question --json-report /tmp/qiongli-full-cycle-harness-drift.json
```

Expected: first command exits `0`; second exits `1`.

- [ ] **Step 4: Run beta smoke**

Run:

```bash
./scripts/run_beta_smoke.sh
```

Expected: PASS.

- [ ] **Step 5: Commit**

Run:

```bash
git add tooling/scripts/run_beta_smoke.sh docs/advanced/publish-pypi.md docs/zh/advanced/publish-pypi.md
git commit -m "test(harness): add full-cycle release smoke"
```

Expected: commit succeeds.

## Task 8: Final Verification

**Files:**

- No new source ownership.

- [ ] **Step 1: Run focused tests**

Run:

```bash
uv run python -m unittest tests.test_lifecycle_harness tests.test_journal_fit tests.test_full_cycle_harness_script tests.test_workflow_contract_doc tests.test_command_workflow_alignment -v
```

Expected: PASS.

- [ ] **Step 2: Run MCP tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers -v
```

Expected: PASS.

- [ ] **Step 3: Run deterministic smoke**

Run:

```bash
./scripts/run_beta_smoke.sh
```

Expected: PASS.

- [ ] **Step 4: Run standards validation**

Run:

```bash
python3 scripts/validate_research_standard.py --strict
```

Expected: `0 failed`.

- [ ] **Step 5: Boundary review**

Run:

```bash
rg -n "api[_-]key|s[e]cret|/U[s]ers/|/p[r]ivate/" docs/superpowers/specs/2026-07-04-full-cycle-multiagent-workflow-and-journal-fit-design.md docs/superpowers/plans/2026-07-04-full-cycle-multiagent-workflow-and-journal-fit.md content/workflow/references/full-cycle-workflow-harness.md content/workflow/workflows/paper-lifecycle.md content/skills/H_submission/journal-fit-recommender.md
```

Expected: no sensitive values or machine-specific paths in committed docs.

- [ ] **Step 6: Final status**

Run:

```bash
git status --short
```

Expected: only intended files are modified before final commit or PR.
