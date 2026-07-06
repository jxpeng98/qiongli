# Coursework Dissertation Contracts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first verifiable Qiongli coursework/dissertation support slice: contract stages, routed workflow docs, skill registry entries, templates, generated references, and guardrail tests.

**Architecture:** This is the contracts/docs vertical slice from `docs/superpowers/specs/2026-07-06-coursework-dissertation-support-design.md`. It adds `L` and `M` as supplemental project-mode stages while preserving the existing `A-K` research lifecycle. Runtime execution, CLI command wiring, and MCP task-run behavior are deferred to later implementation plans.

**Tech Stack:** Python 3.14 test runner from `.venv/bin/python`, PyYAML, Markdown workflow/skill files, Qiongli contract generators.

---

## Scope Boundary

This plan implements only Slice 1 from the design spec:

- contract and generated workflow reference support for `L` and `M`,
- workflow docs for `/coursework` and `/dissertation`,
- routing documentation and skill package entrypoint updates,
- canonical source templates under `content/templates/`,
- skill cards and registry/schema support for `L_coursework` and `M_dissertation`,
- focused tests proving routes, artifacts, templates, and guardrails exist.

This plan intentionally does not implement CLI subcommands, MCP task-run wiring,
or automated artifact materialization. Those belong in the next coursework and
dissertation runtime plans.

## Files

- Create: `tests/test_coursework_dissertation_contract.py`
- Create: `content/workflow/workflows/coursework.md`
- Create: `content/workflow/workflows/dissertation.md`
- Create: `content/workflow/references/stage-L-coursework.md`
- Create: `content/workflow/references/stage-M-dissertation.md`
- Create: `content/skills/L_coursework/assignment-brief-analyzer.md`
- Create: `content/skills/L_coursework/rubric-mapper.md`
- Create: `content/skills/L_coursework/coursework-architect.md`
- Create: `content/skills/L_coursework/coursework-reviser.md`
- Create: `content/skills/M_dissertation/dissertation-planner.md`
- Create: `content/skills/M_dissertation/chapter-architect.md`
- Create: `content/skills/M_dissertation/supervisor-feedback-integrator.md`
- Create: `content/skills/M_dissertation/dissertation-readiness-checker.md`
- Create: `content/templates/assignment-brief.md`
- Create: `content/templates/rubric-map.md`
- Create: `content/templates/learning-outcomes.md`
- Create: `content/templates/academic-integrity-notes.md`
- Create: `content/templates/coursework-outline.md`
- Create: `content/templates/coursework-claim-evidence-plan.md`
- Create: `content/templates/coursework-revision-plan.md`
- Create: `content/templates/coursework-submission-checklist.md`
- Create: `content/templates/dissertation-plan.md`
- Create: `content/templates/dissertation-chapter-map.md`
- Create: `content/templates/dissertation-chapter-status.md`
- Create: `content/templates/supervisor-feedback-log.md`
- Create: `content/templates/dissertation-milestone-plan.md`
- Create: `content/templates/dissertation-final-readiness.md`
- Create: `content/templates/dissertation-defense-prep.md`
- Modify: `content/standards/research-workflow-contract.yaml`
- Modify: `content/workflow/references/workflow-contract.md`
- Modify: `content/workflow/references/platform-routing.md`
- Modify: `content/workflow/SKILL.md`
- Modify: `content/workflow/workflows/paper.md`
- Modify: `content/workflow/workflows/academic-write.md`
- Modify: `content/workflow/references/coverage-matrix.md`
- Modify: `content/schemas/skill.schema.json`
- Modify: `content/schemas/artifact-types.yaml`
- Modify: `content/skills/registry.yaml`
- Modify: `content/skills-summary.md`
- Modify: `content/skills-core.md`
- Modify: `docs/reference/skills.md`
- Modify: `docs/zh/reference/skills.md`
- Modify: `packages/python-qiongli/src/qiongli/skill_docs.py`
- Modify: `tooling/scripts/validate_research_standard.py`

## Baseline

Already verified in the isolated worktree:

```bash
.venv/bin/python -m pytest tests/test_workflow_contract_doc.py tests/test_skill_contract_alignment.py -q
```

Expected baseline:

```text
10 passed
```

## Task 1: Add Contract Tests And Stage IDs

**Files:**
- Create: `tests/test_coursework_dissertation_contract.py`
- Modify: `content/standards/research-workflow-contract.yaml`
- Modify: `tooling/scripts/validate_research_standard.py`
- Modify: `content/workflow/references/workflow-contract.md`

- [ ] **Step 1: Write the failing contract tests**

Create `tests/test_coursework_dissertation_contract.py`:

```python
from __future__ import annotations

from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "content" / "standards" / "research-workflow-contract.yaml"
WORKFLOW_REFERENCE = ROOT / "content" / "workflow" / "references" / "workflow-contract.md"


def load_contract() -> dict:
    return yaml.safe_load(CONTRACT.read_text(encoding="utf-8"))


def test_contract_declares_coursework_and_dissertation_stages() -> None:
    contract = load_contract()
    stages = {stage["id"]: stage for stage in contract["stages"]}

    assert stages["L"]["name"] == "coursework-learning-assessment"
    assert stages["L"]["phase_type"] == "project-mode"
    assert "assignment/brief.md" in stages["L"]["outputs"]
    assert "coursework/claim_evidence_plan.md" in stages["L"]["outputs"]

    assert stages["M"]["name"] == "dissertation-major-project"
    assert stages["M"]["phase_type"] == "project-mode"
    assert "dissertation/dissertation_plan.md" in stages["M"]["outputs"]
    assert "dissertation/defense_prep.md" in stages["M"]["outputs"]


def test_contract_declares_coursework_and_dissertation_task_ids() -> None:
    task_catalog = load_contract()["task_catalog"]

    expected_l = {
        "L1": "assignment-brief-intake",
        "L2": "rubric-learning-outcome-map",
        "L3": "coursework-outline",
        "L4": "coursework-claim-evidence-plan",
        "L5": "coursework-draft",
        "L6": "coursework-revision",
        "L7": "coursework-final-readiness",
    }
    expected_m = {
        "M1": "dissertation-planning",
        "M2": "dissertation-chapter-architecture",
        "M3": "dissertation-chapter-drafting",
        "M4": "supervisor-feedback-integration",
        "M5": "dissertation-milestone-risk-plan",
        "M6": "dissertation-final-readiness",
        "M7": "viva-defense-preparation",
    }

    for task_id, title in expected_l.items():
        assert task_catalog[task_id]["stage"] == "L"
        assert task_catalog[task_id]["title"] == title
        assert task_catalog[task_id]["outputs"]

    for task_id, title in expected_m.items():
        assert task_catalog[task_id]["stage"] == "M"
        assert task_catalog[task_id]["title"] == title
        assert task_catalog[task_id]["outputs"]


def test_contract_declares_project_types_and_context_refresh_points() -> None:
    contract = load_contract()

    assert "coursework" in contract["academic_project_types"]
    assert "dissertation" in contract["academic_project_types"]
    assert "thesis" in contract["academic_project_types"]

    refresh_points = contract["academic_context_continuity"]["refresh_points"]
    assert "rubric criteria" in refresh_points["L"]
    assert "chapter status" in refresh_points["M"]


def test_generated_workflow_reference_mentions_new_stages_and_tasks() -> None:
    text = WORKFLOW_REFERENCE.read_text(encoding="utf-8")

    assert "- `L`: Coursework and learning assessment" in text
    assert "- `M`: Dissertation and major project" in text
    assert "| `L1` | L | Assignment brief intake | `assignment/brief.md` |" in text
    assert "| `M7` | M | Viva or defense preparation | `dissertation/defense_prep.md` |" in text
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py -q
```

Expected: FAIL with `KeyError: 'L'`, `KeyError: 'academic_project_types'`, or missing generated-reference assertions.

- [ ] **Step 3: Add `L` and `M` to the workflow contract**

Modify `content/standards/research-workflow-contract.yaml`:

- add assignment/coursework/dissertation artifacts to `artifacts.required_core`,
- add this top-level section after `paper_types`:

```yaml
academic_project_types:
  - "journal_manuscript"
  - "research_paper"
  - "coursework"
  - "capstone"
  - "dissertation"
  - "thesis"
  - "presentation"
```

- add refresh points:

```yaml
    L: "Preserve assignment brief, rubric criteria, learning outcomes, AI-policy status, word count, user-supplied evidence, and unresolved submission risks."
    M: "Preserve degree level, dissertation type, chapter status, supervisor feedback, milestone constraints, ethics dependencies, and unresolved evidence gaps."
```

- add boundary refresh points:

```yaml
    L: "Refresh rubric, learning-outcome, permitted-assistance, source-use, and personal-evidence boundaries."
    M: "Refresh chapter, supervisor-feedback, ethics, milestone, claim-evidence, and defense-readiness boundaries."
```

- add stages:

```yaml
  - id: "L"
    name: "coursework-learning-assessment"
    sequence_index: 12
    phase_type: "project-mode"
    outputs:
      - "assignment/brief.md"
      - "assignment/rubric_map.md"
      - "assignment/learning_outcomes.md"
      - "assignment/academic_integrity_notes.md"
      - "assignment/submission_checklist.md"
      - "coursework/outline.md"
      - "coursework/claim_evidence_plan.md"
      - "coursework/citation_plan.md"
      - "coursework/draft.md"
      - "coursework/revision_plan.md"
      - "coursework/final_response.md"
  - id: "M"
    name: "dissertation-major-project"
    sequence_index: 13
    phase_type: "project-mode"
    outputs:
      - "assignment/brief.md"
      - "assignment/rubric_map.md"
      - "assignment/learning_outcomes.md"
      - "assignment/academic_integrity_notes.md"
      - "dissertation/dissertation_plan.md"
      - "dissertation/chapter_map.md"
      - "dissertation/chapter_status.md"
      - "dissertation/milestone_plan.md"
      - "dissertation/supervisor_feedback_log.md"
      - "dissertation/revision_plan.md"
      - "dissertation/final_readiness.md"
      - "dissertation/defense_prep.md"
      - "dissertation/chapters/"
```

- add `L1-L7` and `M1-M7` to `task_catalog` with the titles and outputs asserted by the tests.

- [ ] **Step 4: Update contract validation constants**

Modify `tooling/scripts/validate_research_standard.py`:

```python
EXPECTED_STAGE_IDS = {stage for stage in "ABCDEFGHIJKLM"}
EXPECTED_TASK_IDS = {
    "A1", "A1_5", "A2", "A3", "A4", "A5",
    "B1", "B1_5", "B2", "B3", "B4", "B5", "B6",
    "C1", "C1_5", "C2", "C3", "C3_5", "C4", "C5",
    "D1", "D2", "D3",
    "E1", "E2", "E3", "E3_5", "E4", "E5",
    "F1", "F2", "F3", "F4", "F5", "F6",
    "G1", "G2", "G3", "G4",
    "H1", "H2", "H2_5", "H3", "H4", "H5",
    "I1", "I2", "I3", "I4", "I5", "I6", "I7", "I8", "I9",
    "J1", "J2", "J3", "J4",
    "K1", "K2", "K3", "K4",
    "L1", "L2", "L3", "L4", "L5", "L6", "L7",
    "M1", "M2", "M3", "M4", "M5", "M6", "M7",
}
```

Also widen the stage/task regexes in `validate_contract` from `[A-K]` to `[A-M]`.

- [ ] **Step 5: Regenerate the workflow contract reference**

Run:

```bash
.venv/bin/python scripts/generate_workflow_contract_doc.py
```

Expected: command exits 0 and updates `content/workflow/references/workflow-contract.md`.

- [ ] **Step 6: Run tests to verify Task 1 passes**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py tests/test_workflow_contract_doc.py -q
```

Expected: PASS.

## Task 2: Add Workflow Entry Points And Routing Docs

**Files:**
- Modify: `tests/test_coursework_dissertation_contract.py`
- Create: `content/workflow/workflows/coursework.md`
- Create: `content/workflow/workflows/dissertation.md`
- Create: `content/workflow/references/stage-L-coursework.md`
- Create: `content/workflow/references/stage-M-dissertation.md`
- Modify: `content/workflow/references/platform-routing.md`
- Modify: `content/workflow/SKILL.md`
- Modify: `content/workflow/workflows/paper.md`
- Modify: `content/workflow/workflows/academic-write.md`
- Modify: `content/workflow/references/coverage-matrix.md`

- [ ] **Step 1: Add failing workflow/routing tests**

Append to `tests/test_coursework_dissertation_contract.py`:

```python
def test_coursework_and_dissertation_workflow_docs_exist() -> None:
    coursework = (ROOT / "content" / "workflow" / "workflows" / "coursework.md").read_text(encoding="utf-8")
    dissertation = (ROOT / "content" / "workflow" / "workflows" / "dissertation.md").read_text(encoding="utf-8")

    assert "Canonical Task IDs" in coursework
    assert "`L1` assignment brief intake" in coursework
    assert "academic integrity" in coursework.lower()
    assert "`M1` dissertation project planning" in dissertation
    assert "supervisor feedback" in dissertation.lower()


def test_platform_routing_mentions_coursework_and_dissertation() -> None:
    routing = (ROOT / "content" / "workflow" / "references" / "platform-routing.md").read_text(encoding="utf-8")
    skill = (ROOT / "content" / "workflow" / "SKILL.md").read_text(encoding="utf-8")

    assert "/coursework" in routing
    assert "/dissertation" in routing
    assert "assignment brief" in routing
    assert "supervisor feedback" in routing
    assert "/coursework [assignment brief, task, or topic]" in skill
    assert "/dissertation [topic, program, or level]" in skill
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py -q
```

Expected: FAIL because the workflow files and routing references do not exist yet.

- [ ] **Step 3: Create workflow docs**

Create `content/workflow/workflows/coursework.md` with sections:

```markdown
---
description: Coursework and learning-assessment workflow for assignment briefs, rubrics, drafts, and final readiness checks
---

# Coursework Workflow

Support coursework planning, drafting, revision, and final readiness while preserving rubric fit, learning outcomes, source integrity, and institutional AI-policy limits.

Canonical Task IDs:
- `L1` assignment brief intake
- `L2` rubric and learning-outcome mapping
- `L3` coursework outline and structure plan
- `L4` coursework claim-evidence and citation plan
- `L5` coursework draft or section draft
- `L6` coursework revision against rubric
- `L7` coursework final readiness check

## Request

$ARGUMENTS

## Required Boundaries

- Do not promise marks or grades.
- Do not invent module rules, citations, sources, data, fieldwork, personal experience, or supervisor comments.
- Record missing rubric, learning outcome, source, and AI-policy information in `assignment/academic_integrity_notes.md`.
- Treat timed exams, quizzes, and assessed problem sets as concept-explanation requests, not coursework drafting requests.

## Workflow

1. Run `L1` to write `assignment/brief.md`.
2. Run `L2` to write `assignment/rubric_map.md` and `assignment/learning_outcomes.md`.
3. Run `L3` to write `coursework/outline.md` with structure matched to the assignment type.
4. Run `L4` to write `coursework/claim_evidence_plan.md` and `coursework/citation_plan.md`.
5. Run `L5` only after missing user facts, personal experience, data, and source gaps are marked.
6. Run `L6` to compare the draft against the rubric and learning outcomes.
7. Run `L7` to write `assignment/submission_checklist.md`.

Begin in preview mode. Ask for missing assignment rules rather than inventing them.
```

Create `content/workflow/workflows/dissertation.md` with sections:

```markdown
---
description: Dissertation and major-project workflow for planning, chapter architecture, supervisor feedback, readiness, and defense preparation
---

# Dissertation Workflow

Support undergraduate-and-above dissertations, theses, capstones, and major projects while reusing the existing research lifecycle for framing, literature, methods, writing, proofread, and presentation tasks.

Canonical Task IDs:
- `M1` dissertation project planning
- `M2` dissertation chapter architecture
- `M3` dissertation chapter drafting
- `M4` supervisor feedback integration
- `M5` dissertation milestone and risk planning
- `M6` dissertation final readiness check
- `M7` viva or defense preparation

## Request

$ARGUMENTS

## Required Boundaries

- Calibrate expectations to undergraduate, taught master, professional master, research master, or doctoral level.
- Do not imply supervisor approval.
- Do not invent ethics approval, data access, fieldwork, interview material, or institutional rules.
- Preserve chapter status, supervisor feedback, milestone risks, and unresolved evidence gaps across handoffs.

## Workflow

1. Run `M1` to write `dissertation/dissertation_plan.md`.
2. Run `M2` to write `dissertation/chapter_map.md`.
3. Route chapter drafting through existing `A-K` tasks where appropriate, then write chapter material under `dissertation/chapters/`.
4. Run `M4` whenever supervisor feedback is supplied.
5. Run `M5` to write `dissertation/milestone_plan.md`.
6. Run `M6` to write `dissertation/final_readiness.md`.
7. Run `M7` to write `dissertation/defense_prep.md` when viva or defense preparation is requested.

Begin in preview mode and keep missing handbook, rubric, ethics, or supervisor constraints visible.
```

- [ ] **Step 4: Create stage references and update routing docs**

Create concise stage reference files:

- `content/workflow/references/stage-L-coursework.md`
- `content/workflow/references/stage-M-dissertation.md`

Update `content/workflow/references/platform-routing.md`:

- add coursework/dissertation to the trigger contract,
- add natural routing rows for `/coursework` and `/dissertation`,
- add Claude Code mappings for `L1-L7` and `M1-M7`,
- add CLI task packet guidance preserving `academic_project_type`.

Update `content/workflow/SKILL.md` workflow entry points with:

```text
/coursework [assignment brief, task, or topic] # Coursework, rubric, and learning-assessment support
/dissertation [topic, program, or level]       # Dissertation / thesis / major-project support
```

Update `content/workflow/workflows/paper.md`, `content/workflow/workflows/academic-write.md`, and `content/workflow/references/coverage-matrix.md` with short cross-links to `/coursework` and `/dissertation`.

- [ ] **Step 5: Run tests to verify Task 2 passes**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py -q
```

Expected: PASS.

## Task 3: Add Skill Stages, Registry Entries, And Skill Cards

**Files:**
- Modify: `tests/test_coursework_dissertation_contract.py`
- Modify: `content/schemas/skill.schema.json`
- Modify: `content/schemas/artifact-types.yaml`
- Modify: `content/skills/registry.yaml`
- Create: `content/skills/L_coursework/*.md`
- Create: `content/skills/M_dissertation/*.md`
- Modify: `packages/python-qiongli/src/qiongli/skill_docs.py`
- Modify: `tooling/scripts/validate_research_standard.py`

- [ ] **Step 1: Add failing registry/schema tests**

Append to `tests/test_coursework_dissertation_contract.py`:

```python
def test_skill_schema_and_registry_include_coursework_and_dissertation_stages() -> None:
    schema = yaml.safe_load((ROOT / "content" / "schemas" / "skill.schema.json").read_text(encoding="utf-8"))
    allowed_stages = set(schema["properties"]["stage"]["enum"])
    assert "L_coursework" in allowed_stages
    assert "M_dissertation" in allowed_stages

    registry = yaml.safe_load((ROOT / "content" / "skills" / "registry.yaml").read_text(encoding="utf-8"))
    entries = {entry["id"]: entry for entry in registry["skills"]}
    for skill_id in {
        "assignment-brief-analyzer",
        "rubric-mapper",
        "coursework-architect",
        "coursework-reviser",
        "dissertation-planner",
        "chapter-architect",
        "supervisor-feedback-integrator",
        "dissertation-readiness-checker",
    }:
        assert skill_id in entries
        skill_path = ROOT / "content" / entries[skill_id]["file"]
        assert skill_path.is_file()
        assert skill_path.read_text(encoding="utf-8").startswith("---\n")


def test_skill_docs_generator_knows_new_stages() -> None:
    skill_docs = (ROOT / "packages" / "python-qiongli" / "src" / "qiongli" / "skill_docs.py").read_text(encoding="utf-8")

    assert '"L_coursework"' in skill_docs
    assert '"M_dissertation"' in skill_docs
    assert "Coursework" in skill_docs
    assert "Dissertation" in skill_docs
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py -q
```

Expected: FAIL because schema, registry, and skill cards are missing.

- [ ] **Step 3: Update schema and validation stage sets**

Add `"L_coursework"` and `"M_dissertation"` to:

- `content/schemas/skill.schema.json`
- `tooling/scripts/validate_research_standard.py` `EXPECTED_SKILL_STAGES`
- `packages/python-qiongli/src/qiongli/skill_docs.py` `STAGE_ORDER`, `STAGE_META_EN`, and `STAGE_META_ZH`

- [ ] **Step 4: Add artifact types**

Add these artifact types to `content/schemas/artifact-types.yaml`:

```yaml
  - name: AssignmentBrief
    description: "Parsed coursework, capstone, or dissertation assignment brief with constraints and missing information"
    format: markdown
    produced_by: [assignment-brief-analyzer]
    consumed_by: [rubric-mapper, coursework-architect, dissertation-planner]
```

Also add `RubricMap`, `LearningOutcomeMap`, `AcademicIntegrityNotes`,
`CourseworkOutline`, `CourseworkClaimEvidencePlan`, `CourseworkRevisionPlan`,
`CourseworkSubmissionChecklist`, `DissertationPlan`, `DissertationChapterMap`,
`DissertationFeedbackLog`, and `DissertationReadinessReport` with matching
producer/consumer skill IDs.

- [ ] **Step 5: Add skill cards and registry entries**

Create the eight skill files with frontmatter matching their registry IDs and
stage names. Each card must include Purpose, When to Use, Inputs, Process,
Output Contract, Quality Bar, and Common Pitfalls sections.

Add registry entries with version `"1.15.0"`, canonical `true`, compatible
models `[codex, claude, gpt]`, and tooling requirements `[filesystem]`.

- [ ] **Step 6: Run tests to verify Task 3 passes**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py -q
```

Expected: PASS.

## Task 4: Add Templates And Regenerate Skill Docs

**Files:**
- Modify: `tests/test_coursework_dissertation_contract.py`
- Create: `content/templates/*.md`
- Modify: `content/skills-summary.md`
- Modify: `content/skills-core.md`
- Modify: `docs/reference/skills.md`
- Modify: `docs/zh/reference/skills.md`

- [ ] **Step 1: Add failing template tests**

Append to `tests/test_coursework_dissertation_contract.py`:

```python
def test_coursework_and_dissertation_templates_exist_with_missing_information_fields() -> None:
    template_names = [
        "assignment-brief.md",
        "rubric-map.md",
        "learning-outcomes.md",
        "academic-integrity-notes.md",
        "coursework-outline.md",
        "coursework-claim-evidence-plan.md",
        "coursework-revision-plan.md",
        "coursework-submission-checklist.md",
        "dissertation-plan.md",
        "dissertation-chapter-map.md",
        "dissertation-chapter-status.md",
        "supervisor-feedback-log.md",
        "dissertation-milestone-plan.md",
        "dissertation-final-readiness.md",
        "dissertation-defense-prep.md",
    ]

    for name in template_names:
        text = (ROOT / "content" / "templates" / name).read_text(encoding="utf-8")
        assert "Missing Information" in text
        assert "Do not invent" in text
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py -q
```

Expected: FAIL because templates are missing.

- [ ] **Step 3: Create templates**

Create each template under `content/templates/`. Each file must include:

```markdown
## Missing Information

- [ ] Assignment, handbook, rubric, policy, source, data, or user-material gap:

## Integrity Boundary

Do not invent citations, data, supervisor comments, personal experience, grades, or institutional rules.
```

Add template-specific headings after those shared sections.

- [ ] **Step 4: Regenerate skill docs**

Run:

```bash
.venv/bin/python scripts/generate_skill_docs.py
```

Expected: command exits 0 and updates `content/skills-summary.md`, `content/skills-core.md`, `docs/reference/skills.md`, and `docs/zh/reference/skills.md`.

- [ ] **Step 5: Run tests to verify Task 4 passes**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py tests/test_skill_contract_alignment.py -q
```

Expected: PASS.

## Task 5: Final Validation And Boundary Review

**Files:**
- Inspect all files changed by Tasks 1-4.

- [ ] **Step 1: Run focused tests**

Run:

```bash
.venv/bin/python -m pytest tests/test_coursework_dissertation_contract.py tests/test_workflow_contract_doc.py tests/test_skill_contract_alignment.py -q
```

Expected: PASS.

- [ ] **Step 2: Run repository validation**

Run:

```bash
.venv/bin/python tooling/scripts/validate_research_standard.py
```

Expected: PASS with no `[FAIL]` lines.

- [ ] **Step 3: Run diff and whitespace checks**

Run:

```bash
git diff --check
git status --short
```

Expected: no whitespace errors. Status should show only coursework/dissertation support files and the copied design spec.

- [ ] **Step 4: Repo boundary review**

Inspect changed paths and confirm:

- no generated mirrors under `qiongli-workflow/`, `plugins/`, or `packages/*/payload/` were edited,
- no marketplace manifests were copied,
- no local absolute paths were added,
- no secrets or institution-private material were added,
- templates live under canonical `content/templates/`, not ignored `content/workflow/templates/`.

- [ ] **Step 5: Report remaining implementation slices**

Report that this branch completes the contracts/docs vertical slice and adds
runtime task-plan/MCP preview coverage for coursework and dissertation task IDs.
