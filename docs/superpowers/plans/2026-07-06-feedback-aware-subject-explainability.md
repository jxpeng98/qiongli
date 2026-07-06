# Feedback-Aware Subject Explainability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the Stage 5 explainability slice by making subject refinement outputs separate task text, manifest state, trace memory, and user-action evidence.

**Architecture:** Keep routing behavior conservative and explicit: task-text and manifest evidence are emitted by `subject_refinement`, while trace-memory and user-action evidence are added by `guidance_runtime` after the project memory is loaded and updated. The proposal writer renders the same structure so human users can inspect why a subject was suggested, confirmed, dismissed, or left as core.

**Tech Stack:** Python 3.12, existing bridge modules, `unittest`, JSON trace artifacts.

---

## File Map

- Modify: `tests/test_subject_refinement.py`
  - Add RED tests for base `evidence_sources.task_text` and `evidence_sources.manifest_state`.
- Modify: `tests/test_guidance_runtime.py`
  - Add RED tests for trace-enriched `evidence_sources.trace_memory`, `evidence_sources.user_action`, and proposal rendering.
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
  - Add `evidence_sources` to `SubjectRefinementPacket`.
  - Populate base task-text and manifest-state source details for every packet.
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
  - Enrich `evidence_sources` with trace memory and latest lifecycle action.
  - Render a `Subject Evidence Sources` proposal section.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark Stage 5 implemented for this explainability and cooldown slice after verification.

## Task 1: Add Failing Subject Refinement Evidence Source Tests

**Files:**
- Modify: `tests/test_subject_refinement.py`

- [x] **Step 1: Add packet evidence source expectations**

Add a test that calls `infer_subject_refinement()` with finance task text and an auto manifest. Assert:

```python
sources = packet["evidence_sources"]
self.assertEqual(sources["manifest_state"]["active_subject"], "auto")
self.assertEqual(sources["manifest_state"]["subject_mode"], "auto")
self.assertEqual(sources["task_text"]["status"], "present")
self.assertIn("finance.method.event-study", sources["task_text"]["signal_ids"])
self.assertIn("trace_memory", sources)
self.assertIn("user_action", sources)
```

- [x] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_refinement.SubjectRefinementTests.test_subject_refinement_packet_separates_task_text_and_manifest_sources -q
```

Expected: fails with missing `evidence_sources`.

## Task 2: Implement Base Packet Evidence Sources

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`

- [x] **Step 1: Add `evidence_sources` to `SubjectRefinementPacket`**

Add an optional `evidence_sources` dict field and include a copied value in `to_packet()`.

- [x] **Step 2: Populate base sources in `_packet()`**

Add helper functions:

```python
def _base_evidence_sources(manifest_input: object, manifest: ProjectManifest, signals: SubjectSignals) -> dict[str, Any]
def _task_text_evidence_source(signals: SubjectSignals) -> dict[str, Any]
def _manifest_state_evidence_source(manifest_input: object, manifest: ProjectManifest) -> dict[str, Any]
```

Every call to `_packet()` should receive `evidence_sources=_base_evidence_sources(...)`.

- [x] **Step 3: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_refinement.SubjectRefinementTests.test_subject_refinement_packet_separates_task_text_and_manifest_sources -q
```

Expected: `OK`.

## Task 3: Add Failing Trace-Memory And User-Action Tests

**Files:**
- Modify: `tests/test_guidance_runtime.py`

- [x] **Step 1: Add trace enrichment test**

Add a test that:

1. Initializes project guidance.
2. Applies `dismiss finance` through `apply_subject_action()`.
3. Runs `write_guidance_trace()` on a finance-signaled task.
4. Reads `.qiongli/trace/runs/<run_id>/subject_refinement.json`.

Assert:

```python
sources = subject_refinement["evidence_sources"]
self.assertEqual(sources["trace_memory"]["status"], "present")
self.assertEqual(sources["trace_memory"]["subjects"]["finance"]["suggestion_count"], 1)
self.assertEqual(sources["user_action"]["status"], "present")
self.assertEqual(sources["user_action"]["latest_action"]["action"], "dismiss")
self.assertEqual(sources["user_action"]["latest_action"]["source"], "cli")
```

Also assert the proposal contains `## Subject Evidence Sources`, `task_text`, `manifest_state`, `trace_memory`, and `user_action`.

- [x] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_guidance_runtime.GuidanceRuntimeTests.test_guidance_trace_explains_subject_evidence_sources -q
```

Expected: fails with missing trace/user evidence source details.

## Task 4: Implement Trace Enrichment And Proposal Rendering

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`

- [x] **Step 1: Enrich evidence sources**

Add helper functions:

```python
def _enrich_subject_evidence_sources(subject_refinement: dict[str, Any], memory: Mapping[str, Any]) -> None
def _trace_memory_evidence_source(memory: Mapping[str, Any], primary_subject: str) -> dict[str, Any]
def _user_action_evidence_source(memory: Mapping[str, Any], primary_subject: str) -> dict[str, Any]
```

Call enrichment after `_update_subject_evidence()` and before writing `subject_refinement.json`.

- [x] **Step 2: Render proposal section**

Add `_subject_evidence_sources_section()` and call it from `_proposal_text()` after the subject decision section.

- [x] **Step 3: Run GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_guidance_runtime.GuidanceRuntimeTests.test_guidance_trace_explains_subject_evidence_sources -q
```

Expected: `OK`.

## Task 5: Verify And Update Roadmap

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-06-feedback-aware-subject-explainability.md`

- [x] **Step 1: Mark Stage 5 status**

Add a Stage 5 status noting implemented feedback-aware explainability, dismiss cooldowns, lifecycle event memory, and explicit evidence-source rendering.

- [x] **Step 2: Run verification**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_refinement tests.test_guidance_runtime -q
git diff --check
```

Expected: tests pass and whitespace check exits 0.
