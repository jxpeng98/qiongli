# Experience Record Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement roadmap Stage 7 by writing stable local experience records for every guidance trace bundle.

**Architecture:** Add a focused `experience_runtime` module that builds compact experience records from task packets, guidance trace records, validator gates, subject refinement, and worker orchestration state. Integrate it from `guidance_runtime.write_guidance_trace()` so the existing trace path remains the single write point and older trace index behavior stays stable.

**Implementation status:** Completed on `dev` and extended into the planned
Stage 8-11 minimum viable slice: local experience query/replay, MCP query/show/
lessons tools, bounded planner injection, skill reinforcement candidate
generation, promotion candidate gates, and experience metrics.

**Tech Stack:** Python 3.12, unittest, JSON/JSONL trace files, existing Qiongli bridge modules.

---

## File Map

- Create: `packages/python-qiongli/src/qiongli/bridges/experience_runtime.py`
  - Owns experience record construction, path normalization, failure-mode extraction, JSONL append, and summary helpers.
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
  - Calls `write_experience_record()` after the existing trace index record is built.
  - Adds returned experience paths/status to the guidance trace result.
- Create: `tests/test_experience_runtime.py`
  - Unit tests for record construction, failure modes, worker state, path normalization, malformed JSONL tolerance, and JSONL append.
- Modify: `tests/test_guidance_runtime.py`
  - Integration test proving `write_guidance_trace()` writes `experience_record.json` and `.qiongli/trace/experience.jsonl`.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark Stage 7 status once implementation is complete.

## Task 1: Add Failing Experience Runtime Unit Tests

**Files:**
- Create: `tests/test_experience_runtime.py`

- [x] **Step 1: Write tests for record construction and failure modes**

Create `tests/test_experience_runtime.py` with:

```python
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from bridges.experience_runtime import (
    build_experience_record,
    experience_summary,
    write_experience_record,
)


class ExperienceRuntimeTests(unittest.TestCase):
    def test_build_experience_record_captures_task_quality_and_failure_modes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            run_dir = root / ".qiongli" / "trace" / "runs" / "run-1"
            run_dir.mkdir(parents=True)
            guidance_trace = {
                "run_id": "run-1",
                "created_at": "2026-07-06T12:00:00Z",
                "run_dir": ".qiongli/trace/runs/run-1",
                "guidance_mode": "propose",
                "guidance_files_read": [".qiongli/local_guidance.md"],
                "guidance_sources": [{"kind": "project-local", "path": ".qiongli/local_guidance.md"}],
                "project_manifest": {"manifest": {"active_subject": "auto"}},
                "subject_refinement": {"decision": "no_subject", "summary": "Core guidance only."},
            }
            task_packet = {
                "task_id": "B1",
                "paper_type": "systematic-review",
                "topic": "ai-writing",
                "required_outputs": ["search_diagnostics.md"],
                "worker_orchestration": {"status": "disabled", "mode": "none"},
                "controller_metadata": {
                    "execution_mode": "solo",
                    "controller": "codex",
                    "primary_agent": "codex",
                    "review_agent": "claude",
                    "verifier_agent": "",
                },
            }
            validator_gate = {
                "passed": False,
                "found": [],
                "missing": ["search_diagnostics.md"],
                "checked": 1,
            }

            record = build_experience_record(
                project_root=root,
                run_dir=run_dir,
                guidance_trace=guidance_trace,
                task_packet=task_packet,
                validator_gate=validator_gate,
            )

        self.assertEqual(record["schema_version"], "1.0")
        self.assertEqual(record["run_id"], "run-1")
        self.assertEqual(record["task"]["task_id"], "B1")
        self.assertEqual(record["execution"]["execution_mode"], "solo")
        self.assertEqual(record["quality"]["validator_status"], "failed")
        self.assertIn("missing_required_output:search_diagnostics.md", record["experience"]["failure_modes"])
        self.assertEqual(record["outputs"]["missing_outputs"], ["search_diagnostics.md"])
        self.assertEqual(record["privacy"]["redaction_status"], "not_needed")

    def test_write_experience_record_writes_run_file_and_jsonl_index(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            run_dir = root / ".qiongli" / "trace" / "runs" / "run-2"
            run_dir.mkdir(parents=True)
            trace = {"run_id": "run-2", "created_at": "2026-07-06T12:00:00Z", "run_dir": ".qiongli/trace/runs/run-2"}
            packet = {"task_id": "F3", "paper_type": "empirical", "topic": "demo"}
            gate = {"passed": True, "found": ["manuscript/manuscript.md"], "missing": [], "checked": 1}

            result = write_experience_record(
                project_root=root,
                run_dir=run_dir,
                guidance_trace=trace,
                task_packet=packet,
                validator_gate=gate,
            )

            record_path = root / result["experience_record"]
            index_path = root / result["experience_index"]
            record = json.loads(record_path.read_text(encoding="utf-8"))
            rows = [json.loads(line) for line in index_path.read_text(encoding="utf-8").splitlines()]

        self.assertEqual(record["run_id"], "run-2")
        self.assertEqual(rows[0]["run_id"], "run-2")
        self.assertEqual(result["experience_status"], "written")

    def test_experience_summary_tolerates_malformed_jsonl_rows(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            trace_root = root / ".qiongli" / "trace"
            trace_root.mkdir(parents=True)
            (trace_root / "experience.jsonl").write_text(
                '{"run_id": "ok", "task": {"task_id": "B1"}}\nnot-json\n',
                encoding="utf-8",
            )

            summary = experience_summary(root)

        self.assertEqual(summary["run_count"], 1)
        self.assertEqual(summary["malformed_count"], 1)
        self.assertEqual(summary["runs"][0]["run_id"], "ok")


if __name__ == "__main__":
    unittest.main()
```

- [x] **Step 2: Run tests and verify RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_experience_runtime -q
```

Expected: fail with `ModuleNotFoundError: No module named 'bridges.experience_runtime'`.

## Task 2: Implement Minimal Experience Runtime

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/experience_runtime.py`

- [x] **Step 1: Add the runtime module**

Create `packages/python-qiongli/src/qiongli/bridges/experience_runtime.py` with:

```python
from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCHEMA_VERSION = "1.0"
EXPERIENCE_INDEX_REL = Path(".qiongli") / "trace" / "experience.jsonl"


def build_experience_record(
    *,
    project_root: Path,
    run_dir: Path,
    guidance_trace: dict[str, Any],
    task_packet: dict[str, Any],
    validator_gate: dict[str, Any],
) -> dict[str, Any]:
    root = Path(project_root).resolve()
    worker_state = task_packet.get("worker_orchestration", {})
    controller = task_packet.get("controller_metadata", {})
    subject_refinement = guidance_trace.get("subject_refinement", task_packet.get("subject_refinement", {}))
    required_outputs = list(task_packet.get("required_outputs", []) or [])
    found_outputs = list(validator_gate.get("found", []) or [])
    missing_outputs = list(validator_gate.get("missing", []) or [])
    return {
        "schema_version": SCHEMA_VERSION,
        "run_id": str(guidance_trace.get("run_id", "")),
        "created_at": str(guidance_trace.get("created_at") or _utc_now()),
        "project_root": str(root),
        "task": {
            "task_id": str(task_packet.get("task_id", guidance_trace.get("task_id", ""))),
            "paper_type": str(task_packet.get("paper_type", guidance_trace.get("paper_type", ""))),
            "topic": str(task_packet.get("topic", guidance_trace.get("topic", ""))),
            "workflow": str(task_packet.get("workflow", "")),
            "stage": str(task_packet.get("stage", "")),
        },
        "execution": {
            "run_agents": bool(task_packet.get("run_agents", False)),
            "execution_mode": str(controller.get("execution_mode") or _execution_mode_from_worker(worker_state)),
            "controller": str(controller.get("controller", "")),
            "primary_agent": str(controller.get("primary_agent", "")),
            "review_agent": str(controller.get("review_agent", "")),
            "verifier_agent": str(controller.get("verifier_agent", "")),
            "worker_mode": str(worker_state.get("mode", worker_state.get("orchestration_mode", "none"))),
        },
        "inputs": {
            "guidance_files_read": list(guidance_trace.get("guidance_files_read", []) or []),
            "guidance_sources": list(guidance_trace.get("guidance_sources", []) or []),
            "project_manifest": dict(guidance_trace.get("project_manifest", {}) or {}),
            "subject_refinement": dict(subject_refinement or {}),
            "provider_status": dict(task_packet.get("provider_status", {}) or {}),
            "mcp_evidence": list(task_packet.get("mcp_evidence", []) or []),
        },
        "outputs": {
            "required_outputs": required_outputs,
            "found_outputs": found_outputs,
            "missing_outputs": missing_outputs,
            "artifacts_written": list(task_packet.get("artifacts_written", []) or []),
            "trace_files": _trace_files(root, run_dir),
        },
        "quality": {
            "validator_status": _validator_status(validator_gate),
            "review_status": _review_status(worker_state),
            "blocking_issues": _blocking_issues(validator_gate, worker_state),
            "warnings": list(task_packet.get("warnings", []) or []),
            "confidence": float(task_packet.get("confidence", 0.0) or 0.0),
        },
        "experience": {
            "lessons": [],
            "failure_modes": _failure_modes(missing_outputs, validator_gate, worker_state),
            "reusable_guidance": [],
            "promotion_candidates": [],
        },
        "privacy": {
            "redaction_status": "not_needed",
            "contains_user_corpus": False,
            "contains_provider_metadata": bool(task_packet.get("mcp_evidence")),
        },
    }


def write_experience_record(
    *,
    project_root: Path,
    run_dir: Path,
    guidance_trace: dict[str, Any],
    task_packet: dict[str, Any],
    validator_gate: dict[str, Any],
) -> dict[str, str]:
    root = Path(project_root).resolve()
    target_run_dir = Path(run_dir).resolve()
    target_run_dir.mkdir(parents=True, exist_ok=True)
    record = build_experience_record(
        project_root=root,
        run_dir=target_run_dir,
        guidance_trace=guidance_trace,
        task_packet=task_packet,
        validator_gate=validator_gate,
    )
    record_path = target_run_dir / "experience_record.json"
    record_path.write_text(json.dumps(record, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    index_path = root / EXPERIENCE_INDEX_REL
    index_path.parent.mkdir(parents=True, exist_ok=True)
    compact = {
        "schema_version": record["schema_version"],
        "run_id": record["run_id"],
        "created_at": record["created_at"],
        "task": record["task"],
        "execution": record["execution"],
        "quality": record["quality"],
        "experience": record["experience"],
        "experience_record": _rel(root, record_path),
    }
    with index_path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(compact, ensure_ascii=False, sort_keys=True) + "\n")
    return {
        "experience_status": "written",
        "experience_record": _rel(root, record_path),
        "experience_index": _rel(root, index_path),
    }


def experience_summary(project_root: Path, *, limit: int = 20) -> dict[str, Any]:
    root = Path(project_root).resolve()
    index_path = root / EXPERIENCE_INDEX_REL
    rows: list[dict[str, Any]] = []
    malformed_count = 0
    if index_path.is_file():
        for line in index_path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            try:
                parsed = json.loads(line)
            except json.JSONDecodeError:
                malformed_count += 1
                continue
            if isinstance(parsed, dict):
                rows.append(parsed)
    return {
        "project_dir": str(root),
        "experience_index": _rel(root, index_path),
        "run_count": len(rows),
        "malformed_count": malformed_count,
        "runs": rows[-max(0, limit):],
    }


def _validator_status(validator_gate: dict[str, Any]) -> str:
    if validator_gate.get("skipped"):
        return "skipped"
    if validator_gate.get("passed") is True:
        return "passed"
    if validator_gate.get("passed") is False:
        return "failed"
    return "blocked" if validator_gate.get("reason") else "skipped"


def _review_status(worker_state: Any) -> str:
    if not isinstance(worker_state, dict) or worker_state.get("status") in {None, "disabled"}:
        return "skipped"
    status = str(worker_state.get("merge_review_status") or worker_state.get("status") or "")
    if status in {"passed", "failed", "blocked"}:
        return status
    return "skipped"


def _failure_modes(
    missing_outputs: list[Any],
    validator_gate: dict[str, Any],
    worker_state: Any,
) -> list[str]:
    modes = [f"missing_required_output:{item}" for item in missing_outputs if str(item).strip()]
    if validator_gate.get("reason"):
        modes.append("validator_reason:" + str(validator_gate["reason"]))
    if isinstance(worker_state, dict):
        for key in ("barrier_status", "merge_status", "merge_review_status"):
            value = str(worker_state.get(key, "")).strip()
            if value and value not in {"ok", "passed", "disabled"}:
                modes.append(f"worker_{key}:{value}")
    return _unique(modes)


def _blocking_issues(validator_gate: dict[str, Any], worker_state: Any) -> list[str]:
    issues = []
    if validator_gate.get("reason"):
        issues.append(str(validator_gate["reason"]))
    if isinstance(worker_state, dict):
        issues.extend(str(item) for item in worker_state.get("blocking_issues", []) or [])
    return _unique(issues)


def _trace_files(root: Path, run_dir: Path) -> list[str]:
    if not run_dir.is_dir():
        return []
    return [_rel(root, path) for path in sorted(run_dir.iterdir()) if path.is_file()]


def _execution_mode_from_worker(worker_state: Any) -> str:
    if isinstance(worker_state, dict) and worker_state.get("status") not in {None, "disabled"}:
        return "worker"
    return "preview"


def _rel(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return str(path)


def _unique(values: list[str]) -> list[str]:
    seen = set()
    result = []
    for value in values:
        if value not in seen:
            seen.add(value)
            result.append(value)
    return result


def _utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
```

- [x] **Step 2: Run RED tests and verify GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_experience_runtime -q
```

Expected: `OK`.

## Task 3: Integrate Experience Records Into Guidance Trace Writing

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- Modify: `tests/test_guidance_runtime.py`

- [x] **Step 1: Add failing integration assertion**

In `tests/test_guidance_runtime.py`, extend
`test_write_guidance_trace_creates_linked_bundle_and_index` after the existing
trace file assertions with:

```python
            experience_path = run_dir / "experience_record.json"
            self.assertTrue(experience_path.is_file())
            experience = json.loads(experience_path.read_text(encoding="utf-8"))
            self.assertEqual(experience["run_id"], "run-123")
            self.assertEqual(experience["task"]["task_id"], "F3")
            self.assertEqual(experience["quality"]["validator_status"], "failed")
            self.assertIn(
                "missing_required_output:manuscript/manuscript.md",
                experience["experience"]["failure_modes"],
            )
            experience_rows = [
                json.loads(line)
                for line in (root / ".qiongli" / "trace" / "experience.jsonl")
                .read_text(encoding="utf-8")
                .splitlines()
                if line.strip()
            ]
            self.assertEqual(experience_rows[0]["run_id"], "run-123")
            self.assertEqual(trace["experience_status"], "written")
            self.assertEqual(
                trace["experience_record"],
                ".qiongli/trace/runs/run-123/experience_record.json",
            )
```

- [x] **Step 2: Run integration test and verify RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_guidance_runtime.GuidanceRuntimeTests.test_write_guidance_trace_creates_linked_bundle_and_index -q
```

Expected: fail because `experience_record.json` is not written yet.

- [x] **Step 3: Wire `write_experience_record()` into `write_guidance_trace()`**

In `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`, add:

```python
from .experience_runtime import write_experience_record
```

After appending the existing `index.jsonl` record, add:

```python
    experience_result = write_experience_record(
        project_root=paths.project_root,
        run_dir=run_dir,
        guidance_trace=record,
        task_packet=task_packet,
        validator_gate=validator_gate,
    )
    record.update(experience_result)
```

The experience write should happen before `return record` so callers receive
the experience status and paths.

- [x] **Step 4: Run integration test and verify GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_guidance_runtime.GuidanceRuntimeTests.test_write_guidance_trace_creates_linked_bundle_and_index -q
```

Expected: `OK`.

## Task 4: Verify Existing Orchestrator Trace Behavior

**Files:**
- Modify only if tests expose a missing field:
  `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Test: `tests/test_orchestrator_workflows.py`

- [x] **Step 1: Run local guidance trace orchestrator tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_orchestrator_workflows -q
```

Expected: existing tests remain `OK`. If failures show missing expected trace
keys, update the test expectation only when the old contract truly changed; the
intended behavior is additive.

- [x] **Step 2: Add focused orchestrator assertion if no existing test covers it**

If no orchestrator test reads `local_guidance_trace["experience_record"]`, add a
focused assertion to the nearest local-guidance trace test:

```python
            self.assertEqual(
                trace["experience_record"],
                ".qiongli/trace/runs/" + trace["run_id"] + "/experience_record.json",
            )
```

Run the single test first and verify it passes.

## Task 5: Documentation Status And Final Verification

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [x] **Step 1: Mark Stage 7 implementation status**

Under `## Stage 7: Experience Record Contract`, add:

```markdown
Status: implemented on `dev` for local experience record writing, query/replay,
planner injection, skill reinforcement candidates, promotion candidate gates,
and metrics. Release-readiness compatibility checks remain follow-up work.
```

- [x] **Step 2: Run targeted verification**

Run:

```bash
.venv/bin/python -m unittest tests.test_experience_runtime tests.test_guidance_runtime -q
.venv/bin/python -m unittest tests.test_worker_orchestration_runtime tests.test_agent_run_contract -q
git diff --check
```

Expected:

- experience and guidance runtime tests pass
- worker and agent-run contract tests pass
- whitespace check exits 0

- [ ] **Step 3: Commit the Stage 7 slice**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/experience_runtime.py \
  packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py \
  tests/test_experience_runtime.py \
  tests/test_guidance_runtime.py \
  docs/superpowers/specs/2026-07-06-experience-promotion-loop-design.md \
  docs/superpowers/plans/2026-07-06-experience-record-contract.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "feat(experience): record task-run experience traces"
```

## Completed Minimum Slices And Remaining Follow-On

Stage 8 minimum slice is implemented: CLI list/show/search/lessons/replay-plan,
MCP query/show/lessons tools, and bounded `prior_experience` injection for
`task-plan` and `task-run`. Remaining follow-up: richer older-trace fallback,
read-only export formatting, and prompt wording hardening.

Stage 9 minimum slice is implemented: repeated failure evidence can generate a
skill reinforcement candidate under `.qiongli/trace/promotion/` without editing
`content/skills/**`. Remaining follow-up: maintainer-applied source updates and
skill/core eval expansion.

Stage 10 minimum slice is implemented: promotion scopes are explicit, canonical
candidates require a test plan, user-global promotion requires approval, and
canonical source edits are never automatic. Remaining follow-up: local guidance
proposal acceptance UX and user-global redaction review.

Stage 11 minimum slice is implemented: `experience metrics` summarizes
validator pass rate, missing artifact rate, failure modes, and worker merge/final
review blockers. Remaining follow-up: release-readiness schema compatibility
checks and docs for metrics interpretation.
