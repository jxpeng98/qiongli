# Paper Reading Summary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend Qiongli B2 `/paper-read` so targeted paper reading produces grounded single-paper notes plus project-level paper reading summary and matrix artifacts.

**Architecture:** Keep B2 as the canonical workflow. Add summary/matrix artifacts to the contract, teach `/paper-read` to update them, and add templates that force source anchors, evidence limits, and uncertainty handling.

**Tech Stack:** Markdown workflow specs, YAML contract, Python `unittest`, existing sync and validation scripts.

---

### Task 1: Add Failing Contract Tests

**Files:**
- Create: `tests/test_paper_reading_summary_contract.py`

- [ ] **Step 1: Write failing tests**

```python
from __future__ import annotations

import unittest
from pathlib import Path

import yaml


REPO_ROOT = Path(__file__).resolve().parents[1]


class PaperReadingSummaryContractTests(unittest.TestCase):
    def test_b2_contract_declares_project_summary_outputs(self) -> None:
        contract = yaml.safe_load(
            (REPO_ROOT / "standards" / "research-workflow-contract.yaml").read_text(
                encoding="utf-8"
            )
        )

        outputs = set(contract["task_catalog"]["B2"]["outputs"])

        self.assertIn("literature/paper_reading_summary.md", outputs)
        self.assertIn("literature/paper_reading_matrix.md", outputs)

    def test_paper_read_workflow_enforces_truthful_summary_boundaries(self) -> None:
        content = (
            REPO_ROOT / "qiongli-workflow" / "workflows" / "paper-read.md"
        ).read_text(encoding="utf-8")

        for token in (
            "literature/paper_reading_summary.md",
            "literature/paper_reading_matrix.md",
            "evidence_limit: abstract_only",
            "Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications.",
            "direct_evidence",
            "reasonable_inference",
            "unsupported_gap",
        ):
            self.assertIn(token, content)

    def test_paper_note_template_requires_source_anchors_and_uncertainty(self) -> None:
        content = (REPO_ROOT / "templates" / "paper-note.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "Evidence Limit",
            "Source Anchors",
            "Author Claims",
            "Agent Interpretation",
            "Reusable Citation Points",
            "Uncertainty Register",
        ):
            self.assertIn(token, content)

    def test_project_summary_templates_exist_and_block_unsupported_claims(self) -> None:
        summary = (REPO_ROOT / "templates" / "paper-reading-summary.md").read_text(
            encoding="utf-8"
        )
        matrix = (REPO_ROOT / "templates" / "paper-reading-matrix.md").read_text(
            encoding="utf-8"
        )

        for token in (
            "source_anchor",
            "evidence_limit",
            "unsupported_gap",
            "Do not upgrade an inference into a fact",
        ):
            self.assertIn(token, summary)
            self.assertIn(token, matrix)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
python3 -m unittest tests.test_paper_reading_summary_contract -v
```

Expected: failures because B2 does not yet declare the new outputs and the new templates do not yet exist.

### Task 2: Update Canonical Contract And Generated Reference

**Files:**
- Modify: `standards/research-workflow-contract.yaml`
- Regenerate: `qiongli-workflow/references/workflow-contract.md`

- [ ] **Step 1: Add B2 outputs**

In `standards/research-workflow-contract.yaml`, under `task_catalog.B2.outputs`, add:

```yaml
      - "literature/paper_reading_summary.md"
      - "literature/paper_reading_matrix.md"
```

- [ ] **Step 2: Regenerate workflow contract doc**

Run:

```bash
python3 scripts/generate_workflow_contract_doc.py
```

Expected: `qiongli-workflow/references/workflow-contract.md` updates the B2 primary output list.

- [ ] **Step 3: Re-run focused test**

Run:

```bash
python3 -m unittest tests.test_paper_reading_summary_contract.PaperReadingSummaryContractTests.test_b2_contract_declares_project_summary_outputs -v
```

Expected: PASS.

### Task 3: Add Grounded Reading Templates

**Files:**
- Modify: `templates/paper-note.md`
- Create: `templates/paper-reading-summary.md`
- Create: `templates/paper-reading-matrix.md`

- [ ] **Step 1: Extend `templates/paper-note.md`**

Add fields for retrieval/evidence limits, source anchors, author claims, agent interpretation, reusable citation points, and uncertainty register. Use controlled labels:

```markdown
| **Evidence Limit** | full_text / abstract_only / metadata_only / unavailable |
| **Retrieval Status** | retrieved_oa / retrieved_preprint / abstract_only / not_retrieved:<reason> |
```

- [ ] **Step 2: Create `templates/paper-reading-summary.md`**

Include sections for corpus overview, grounded themes, method/data patterns, stable findings, contradictions, gaps, writing-ready citation points, and uncertainty register. Require `source_anchor`, `evidence_limit`, and inference strength for each major claim.

- [ ] **Step 3: Create `templates/paper-reading-matrix.md`**

Create a compact table with citekey, evidence limit, theory, method, data source, main finding, limitation, relevance, source anchors, and inference strength.

- [ ] **Step 4: Run focused template tests**

Run:

```bash
python3 -m unittest tests.test_paper_reading_summary_contract.PaperReadingSummaryContractTests.test_paper_note_template_requires_source_anchors_and_uncertainty tests.test_paper_reading_summary_contract.PaperReadingSummaryContractTests.test_project_summary_templates_exist_and_block_unsupported_claims -v
```

Expected: PASS.

### Task 4: Update B2 Workflow And Stage B Reference

**Files:**
- Modify: `qiongli-workflow/workflows/paper-read.md`
- Modify: `qiongli-workflow/references/stage-B-literature.md`

- [ ] **Step 1: Update `/paper-read` workflow**

Add directory creation for `literature/`, a truthfulness boundary section, and a project-level summary integration step that updates:

```text
RESEARCH/[topic]/literature/paper_reading_summary.md
RESEARCH/[topic]/literature/paper_reading_matrix.md
```

The workflow must explicitly say to mark `evidence_limit: abstract_only` when full text is unavailable and to write gaps instead of unsupported facts.

- [ ] **Step 2: Update Stage B reference**

In the B2 canonical outputs and B2 definition-of-done, add the summary and matrix artifacts. State that B2 organizes targeted reading notes but does not make systematic-review-grade claims.

- [ ] **Step 3: Run workflow boundary test**

Run:

```bash
python3 -m unittest tests.test_paper_reading_summary_contract.PaperReadingSummaryContractTests.test_paper_read_workflow_enforces_truthful_summary_boundaries -v
```

Expected: PASS.

### Task 5: Sync Package Payloads And Validate

**Files:**
- Generated/mirrored package files under `qiongli-workflow/`, `plugins/qiongli/`, `qiongli/payload/`, and `packages/npm-qiongli/` as produced by existing sync scripts.

- [ ] **Step 1: Sync skill package**

Run:

```bash
./scripts/sync_skill_package.sh --target all
```

Expected: portable package and plugin package mirror the canonical templates, skills, standards, roles, and venue profiles.

- [ ] **Step 2: Sync npm/python payloads**

Run:

```bash
python3 scripts/sync_npm_package_payload.py
```

Expected: npm payload and Python payload include the new B2 contract outputs and templates.

- [ ] **Step 3: Run focused and standard validation**

Run:

```bash
python3 -m unittest tests.test_paper_reading_summary_contract tests.test_workflow_contract_doc tests.test_literature_contract -v
python3 scripts/validate_research_standard.py --strict
```

Expected: all tests pass and strict validation reports no errors.

### Task 6: Review Diff For Generated Churn

**Files:**
- All modified files.

- [ ] **Step 1: Inspect status**

Run:

```bash
git status --short
```

Expected: changes are limited to B2 workflow/reference/contract/templates, package mirrors, tests, and superpowers docs.

- [ ] **Step 2: Inspect focused diff**

Run:

```bash
git diff -- standards/research-workflow-contract.yaml qiongli-workflow/workflows/paper-read.md qiongli-workflow/references/stage-B-literature.md templates/paper-note.md templates/paper-reading-summary.md templates/paper-reading-matrix.md tests/test_paper_reading_summary_contract.py
```

Expected: every behavior change supports the approved B2 paper-reading summary scope and truthfulness boundary.
