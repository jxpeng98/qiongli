# Subject Runtime Smoke And Core V2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a preview-first real local smoke harness and continue the adaptive subject runtime so installed Qiongli can discover, refine, and apply subject-specific guidance during use without asking users to choose a subject at install time.

**Architecture:** Run two independent feature worktrees from `dev`. Track A treats the existing MCP `qiongli_task_run` preview path as the system under test, isolates all project and user config state in temporary directories, and emits reproducible JSON reports. Track B deepens the runtime contract with structured signal records, a resource activation plan, and cross-run evidence memory; after Track B lands, Track A rebases and expands the smoke assertions to cover the new packet fields.

**Tech Stack:** Python 3.11+, stdlib `unittest`, `tempfile`, JSON/YAML, existing `uv`, existing `node --test`, Qiongli bridge modules.

---

## Execution Model

Use `superpowers:subagent-driven-development` for implementation. Dispatch one worker for Track A and one worker for Track B. Each worker uses its own worktree and commits only coherent slices. The main agent reviews each slice, runs the targeted tests, then coordinates rebases and the final merge back to `dev`.

Track A should never launch local agents by default. The real smoke path is "real" because it calls the actual MCP tool handler and subject refinement runtime, but it remains preview-only unless `QIONGLI_SMOKE_RUN_AGENTS=1` and a command flag both request local agents.

Track B should not make automatic project-manifest changes. It may write trace files and proposal files. Promotion from repeated evidence should be a recommendation that the user or a future apply mode can accept.

## Worktrees And Branches

- Track A branch: `feature/real-smoke-subject-runtime`
- Track A worktree: `.worktrees/real-smoke-subject-runtime`
- Track B branch: `feature/subject-refinement-core-v2`
- Track B worktree: `.worktrees/subject-refinement-core-v2`
- Base branch: `dev`
- Integration order: merge Track B into `dev`, rebase Track A onto updated `dev`, update Track A smoke expectations for Track B packet fields, then merge Track A into `dev`.

## Files

Track A creates:

- `tooling/scripts/run_subject_runtime_smoke.py`: isolated smoke runner with preview and opt-in local-agent modes.
- `tests/fixtures/subject_runtime_smoke/no_subject_core_only.json`: fixture for core-only runtime.
- `tests/fixtures/subject_runtime_smoke/borrow_finance_lens.json`: fixture for method-lens borrowing.
- `tests/fixtures/subject_runtime_smoke/suggest_finance_subject.json`: fixture for finance subject suggestion.
- `tests/fixtures/subject_runtime_smoke/locked_economics_borrow_finance.json`: fixture for locked subject plus borrowed method pack.
- `tests/test_subject_runtime_smoke.py`: unit tests for fixture loading, isolation, report shape, preview assertions, and local-agent guard.

Track B creates or modifies:

- `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`: add structured signal records and attach `resource_activation_plan` to `SubjectRefinementPacket`.
- `packages/python-qiongli/src/qiongli/bridges/subject_resources.py`: central planner for adaptive resource activation.
- `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`: persist cross-run subject evidence memory and include promotion recommendations in guidance proposals.
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`: expose the richer packet through preview output without changing default preview safety.
- `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`: include resource activation plan and signal summaries in draft/review prompts.
- `content/standards/subject-refinement-contract.yaml`: document signal dimensions, activation levels, and promotion thresholds.
- `tests/test_subject_refinement.py`: signal ledger and packet compatibility tests.
- `tests/test_subject_resources.py`: resource planner tests.
- `tests/test_guidance_runtime.py`: evidence memory and promotion proposal tests.
- `tests/test_mcp_tool_handlers.py`: preview payload tests.
- `tests/test_orchestrator_subject_refinement.py`: real task-run packet and prompt tests.

## Shared Packet Contract

Track B extends the existing `subject_refinement` packet while preserving current keys. The packet remains backward-compatible with callers that only read `decision`, `primary_subject`, `loaded_resources`, or `domain`.

New `signals` records use this shape:

```json
{
  "id": "finance.method.event-study",
  "subject": "finance",
  "dimension": "method",
  "value": "event-study",
  "weight": 0.35,
  "source": "task_text",
  "snippet": "Use an event study with event windows around earnings announcements."
}
```

New `resource_activation_plan` uses this shape:

```json
{
  "decision": "suggest_subject",
  "active_subject": "auto",
  "primary_subject": "finance",
  "levels": ["core", "subject_overlay", "subject_skill", "method_pack"],
  "resources": [
    {
      "kind": "subject_overlay",
      "subject": "finance",
      "path": "overlays/finance.yaml",
      "activation": "suggested"
    },
    {
      "kind": "subject_skill",
      "subject": "finance",
      "path": "skills/finance/SKILL.md",
      "activation": "suggested"
    },
    {
      "kind": "method_pack",
      "subject": "finance",
      "lens": "event-study",
      "path": "method-packs/finance/event-study.yaml",
      "activation": "suggested"
    }
  ],
  "persistence_recommendation": {
    "status": "proposed",
    "write_manifest": false,
    "recommended_subject_mode": "suggested"
  }
}
```

## Task 0: Prepare Parallel Worktrees

**Files:**
- No source files changed in this task.

- [ ] **Step 1: Confirm `dev` is clean**

Run from repository root:

```bash
git status --short --branch
```

Expected output starts with:

```text
## dev...origin/dev
```

There should be no uncommitted source changes before creating worktrees.

- [ ] **Step 2: Create Track A worktree**

Run:

```bash
git worktree add .worktrees/real-smoke-subject-runtime -b feature/real-smoke-subject-runtime dev
```

Expected output includes:

```text
Preparing worktree
HEAD is now at
```

- [ ] **Step 3: Create Track B worktree**

Run:

```bash
git worktree add .worktrees/subject-refinement-core-v2 -b feature/subject-refinement-core-v2 dev
```

Expected output includes:

```text
Preparing worktree
HEAD is now at
```

- [ ] **Step 4: Record branch ownership**

Run:

```bash
git worktree list
```

Expected output includes the repository root on `dev`, Track A on `feature/real-smoke-subject-runtime`, and Track B on `feature/subject-refinement-core-v2`.

## Track A: Real Smoke Harness

### Task A1: Add Smoke Fixtures

**Files:**
- Create: `tests/fixtures/subject_runtime_smoke/no_subject_core_only.json`
- Create: `tests/fixtures/subject_runtime_smoke/borrow_finance_lens.json`
- Create: `tests/fixtures/subject_runtime_smoke/suggest_finance_subject.json`
- Create: `tests/fixtures/subject_runtime_smoke/locked_economics_borrow_finance.json`

- [ ] **Step 1: Create fixture directory**

Run in Track A worktree:

```bash
mkdir -p tests/fixtures/subject_runtime_smoke
```

Expected: command exits with status 0.

- [ ] **Step 2: Add core-only fixture**

Create `tests/fixtures/subject_runtime_smoke/no_subject_core_only.json`:

```json
{
  "name": "no_subject_core_only",
  "manifest": null,
  "args": {
    "task_id": "F3",
    "paper_type": "empirical",
    "topic": "revise introduction",
    "context": "Tighten the framing and improve transitions.",
    "domain": "auto",
    "guidance_mode": "propose",
    "run_agents": false
  },
  "expected": {
    "decision": "no_subject",
    "primary_subject": "auto",
    "effective_domain": "auto",
    "resource_levels": ["core_only"],
    "run_agents": false
  }
}
```

- [ ] **Step 3: Add borrowed-lens fixture**

Create `tests/fixtures/subject_runtime_smoke/borrow_finance_lens.json`:

```json
{
  "name": "borrow_finance_lens",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "args": {
    "task_id": "C1",
    "paper_type": "empirical",
    "topic": "policy announcement timing",
    "context": "Use an event-study timing lens around policy announcements with qualitative adoption outcomes.",
    "domain": "auto",
    "guidance_mode": "propose",
    "run_agents": false
  },
  "expected": {
    "decision": "borrow_lens",
    "primary_subject": "auto",
    "effective_domain": "auto",
    "borrowed_lens": "event-study",
    "borrowed_subject": "finance",
    "resource_levels": ["method_pack_only"],
    "run_agents": false
  }
}
```

- [ ] **Step 4: Add finance suggestion fixture**

Create `tests/fixtures/subject_runtime_smoke/suggest_finance_subject.json`:

```json
{
  "name": "suggest_finance_subject",
  "manifest": null,
  "args": {
    "task_id": "C1",
    "paper_type": "empirical",
    "topic": "earnings announcement stock market reaction",
    "context": "Estimate CRSP abnormal returns using an event study for Journal of Finance framing.",
    "domain": "auto",
    "guidance_mode": "propose",
    "run_agents": false
  },
  "expected": {
    "decision": "suggest_subject",
    "primary_subject": "finance",
    "effective_domain": "finance",
    "method_lens": "event-study",
    "resource_levels": ["subject_overlay", "subject_skill", "method_pack"],
    "run_agents": false
  }
}
```

- [ ] **Step 5: Add locked-economics fixture**

Create `tests/fixtures/subject_runtime_smoke/locked_economics_borrow_finance.json`:

```json
{
  "name": "locked_economics_borrow_finance",
  "manifest": {
    "active_subject": "economics",
    "subject_mode": "locked",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "args": {
    "task_id": "C1",
    "paper_type": "empirical",
    "topic": "treatment announcement",
    "context": "Use CRSP abnormal returns and an event study for Journal of Finance while keeping the economics framing.",
    "domain": "auto",
    "guidance_mode": "propose",
    "run_agents": false
  },
  "expected": {
    "decision": "lock_subject",
    "primary_subject": "economics",
    "effective_domain": "economics",
    "borrowed_lens": "event-study",
    "borrowed_subject": "finance",
    "resource_levels": ["subject_overlay", "subject_skill", "method_pack_only"],
    "run_agents": false
  }
}
```

- [ ] **Step 6: Commit fixtures**

Run:

```bash
git add tests/fixtures/subject_runtime_smoke
git commit -m "test(smoke): add subject runtime fixtures"
```

Expected: commit succeeds and only fixture files are included.

### Task A2: Write Failing Smoke Runner Tests

**Files:**
- Create: `tests/test_subject_runtime_smoke.py`
- Future create: `tooling/scripts/run_subject_runtime_smoke.py`

- [ ] **Step 1: Add failing test file**

Create `tests/test_subject_runtime_smoke.py`:

```python
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tooling.scripts.run_subject_runtime_smoke import (
    FIXTURE_DIR,
    SmokeCase,
    load_smoke_cases,
    run_smoke_suite,
)


class SubjectRuntimeSmokeTests(unittest.TestCase):
    def test_load_smoke_cases_reads_all_fixtures(self) -> None:
        cases = load_smoke_cases(FIXTURE_DIR)

        names = {case.name for case in cases}
        self.assertEqual(
            names,
            {
                "no_subject_core_only",
                "borrow_finance_lens",
                "suggest_finance_subject",
                "locked_economics_borrow_finance",
            },
        )
        self.assertTrue(all(isinstance(case, SmokeCase) for case in cases))

    def test_preview_suite_passes_and_writes_inside_isolated_project(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=[],
            )

        self.assertEqual(report["summary"]["failed"], 0)
        self.assertEqual(report["summary"]["passed"], 4)
        self.assertEqual(report["mode"], "preview")
        for case in report["cases"]:
            self.assertTrue(case["project_root"].startswith(str(Path(tmp_dir).resolve())))
            self.assertFalse(case["result"]["run_agents"])
            self.assertEqual(case["status"], "passed")

    def test_report_is_json_serializable(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        encoded = json.dumps(report, sort_keys=True)
        decoded = json.loads(encoded)
        self.assertEqual(decoded["summary"]["total"], 1)
        self.assertEqual(decoded["cases"][0]["name"], "suggest_finance_subject")

    def test_local_agent_mode_requires_environment_opt_in(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.dict("os.environ", {}, clear=True):
                with self.assertRaises(RuntimeError) as raised:
                    run_smoke_suite(
                        fixture_dir=FIXTURE_DIR,
                        workspace_root=Path(tmp_dir),
                        mode="local-agent",
                        selected_cases=["suggest_finance_subject"],
                    )

        self.assertIn("QIONGLI_SMOKE_RUN_AGENTS=1", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the new tests and verify the expected import failure**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: fail with an import error for `tooling.scripts.run_subject_runtime_smoke`.

### Task A3: Implement Preview Smoke Runner

**Files:**
- Create: `tooling/scripts/run_subject_runtime_smoke.py`
- Modify: `tests/test_subject_runtime_smoke.py` only if assertions need the final report field names from this task.

- [ ] **Step 1: Create script package path if needed**

Run:

```bash
mkdir -p tooling/scripts
```

Expected: command exits with status 0.

- [ ] **Step 2: Implement runner data model and fixture loading**

Create the top of `tooling/scripts/run_subject_runtime_smoke.py`:

```python
from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
import uuid
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SRC = REPO_ROOT / "packages" / "python-qiongli" / "src"
if str(PYTHON_SRC) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC))

from bridges.mcp_tool_handlers import call_qiongli_tool  # noqa: E402


FIXTURE_DIR = REPO_ROOT / "tests" / "fixtures" / "subject_runtime_smoke"
MANIFEST_REL = Path(".qiongli") / "guidance_manifest.yaml"


@dataclass(frozen=True)
class SmokeCase:
    name: str
    manifest: dict[str, Any] | None
    args: dict[str, Any]
    expected: dict[str, Any]
    source: Path


def load_smoke_cases(fixture_dir: Path = FIXTURE_DIR) -> list[SmokeCase]:
    cases: list[SmokeCase] = []
    for path in sorted(Path(fixture_dir).glob("*.json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        cases.append(
            SmokeCase(
                name=str(payload["name"]),
                manifest=payload.get("manifest"),
                args=dict(payload["args"]),
                expected=dict(payload["expected"]),
                source=path,
            )
        )
    return cases
```

- [ ] **Step 3: Implement isolated project setup**

Add to `tooling/scripts/run_subject_runtime_smoke.py`:

```python
def _write_manifest(project_root: Path, manifest: dict[str, Any] | None) -> None:
    if manifest is None:
        return
    manifest_path = project_root / MANIFEST_REL
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(
        yaml.safe_dump(manifest, sort_keys=False, allow_unicode=False),
        encoding="utf-8",
    )


def _isolated_env(project_root: Path) -> dict[str, str]:
    base = project_root / ".smoke-home"
    return {
        "QIONGLI_GUIDANCE_HOME": str(base / "qiongli-guidance"),
        "QIONGLI_CONFIG_HOME": str(base / "qiongli-config"),
        "CODEX_HOME": str(base / "codex"),
        "XDG_CONFIG_HOME": str(base / "xdg-config"),
    }
```

- [ ] **Step 4: Implement one-case execution and assertions**

Add to `tooling/scripts/run_subject_runtime_smoke.py`:

```python
def run_smoke_case(case: SmokeCase, *, workspace_root: Path, mode: str) -> dict[str, Any]:
    project_root = workspace_root.resolve() / case.name
    project_root.mkdir(parents=True, exist_ok=True)
    _write_manifest(project_root, case.manifest)

    args = dict(case.args)
    args["cwd"] = str(project_root)
    args["run_agents"] = mode == "local-agent"

    env_updates = _isolated_env(project_root)
    old_env = {key: os.environ.get(key) for key in env_updates}
    os.environ.update(env_updates)
    try:
        result = call_qiongli_tool("qiongli_task_run", args)
    finally:
        for key, value in old_env.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value

    failures = _assert_case(case, result)
    return {
        "name": case.name,
        "source": str(case.source.relative_to(REPO_ROOT)),
        "project_root": str(project_root),
        "status": "passed" if not failures else "failed",
        "failures": failures,
        "result": result.get("structuredContent", result),
    }


def _assert_case(case: SmokeCase, result: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if result.get("isError"):
        failures.append("tool returned isError=true")
        return failures

    payload = result.get("structuredContent", result)
    data = payload.get("data", {}) if isinstance(payload, dict) else {}
    preview = data.get("task_run_preview", {}) if isinstance(data, dict) else {}
    refinement = preview.get("subject_refinement", {}) if isinstance(preview, dict) else {}
    expected = case.expected

    _expect_equal(failures, "run_agents", payload.get("run_agents"), expected.get("run_agents"))
    _expect_equal(failures, "decision", refinement.get("decision"), expected.get("decision"))
    _expect_equal(
        failures,
        "primary_subject",
        refinement.get("primary_subject"),
        expected.get("primary_subject"),
    )
    _expect_equal(
        failures,
        "effective_domain",
        preview.get("effective_domain"),
        expected.get("effective_domain"),
    )
    loaded_resources = refinement.get("loaded_resources", {})
    for level in expected.get("resource_levels", []):
        if level not in loaded_resources.get("levels", []):
            failures.append(f"missing resource level {level!r}")
    if "method_lens" in expected and expected["method_lens"] not in refinement.get("method_lenses", []):
        failures.append(f"missing method lens {expected['method_lens']!r}")
    if "borrowed_lens" in expected:
        borrowed = refinement.get("borrowed_lenses", [])
        if not any(item.get("lens") == expected["borrowed_lens"] for item in borrowed):
            failures.append(f"missing borrowed lens {expected['borrowed_lens']!r}")
    return failures


def _expect_equal(failures: list[str], field: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        failures.append(f"{field}: expected {expected!r}, got {actual!r}")
```

- [ ] **Step 5: Implement suite execution and CLI**

Add to `tooling/scripts/run_subject_runtime_smoke.py`:

```python
def run_smoke_suite(
    *,
    fixture_dir: Path = FIXTURE_DIR,
    workspace_root: Path | None = None,
    mode: str = "preview",
    selected_cases: list[str] | None = None,
) -> dict[str, Any]:
    if mode == "local-agent" and os.environ.get("QIONGLI_SMOKE_RUN_AGENTS") != "1":
        raise RuntimeError(
            "local-agent smoke requires QIONGLI_SMOKE_RUN_AGENTS=1 and launches local runtime agents"
        )

    cases = load_smoke_cases(fixture_dir)
    selected = set(selected_cases or [])
    if selected:
        cases = [case for case in cases if case.name in selected]
    if selected and len(cases) != len(selected):
        found = {case.name for case in cases}
        missing = sorted(selected - found)
        raise ValueError("unknown smoke case(s): " + ", ".join(missing))

    with tempfile.TemporaryDirectory(prefix="qiongli-smoke-") as tmp_dir:
        root = Path(workspace_root).resolve() if workspace_root else Path(tmp_dir).resolve()
        root.mkdir(parents=True, exist_ok=True)
        case_results = [
            run_smoke_case(case, workspace_root=root, mode=mode)
            for case in cases
        ]

    failed = sum(1 for case in case_results if case["status"] != "passed")
    return {
        "schema_version": "1.0",
        "created_at": datetime.now(timezone.utc).isoformat(),
        "run_id": uuid.uuid4().hex,
        "mode": mode,
        "summary": {
            "total": len(case_results),
            "passed": len(case_results) - failed,
            "failed": failed,
        },
        "cases": case_results,
        "environment": {
            "repo_root": str(REPO_ROOT),
            "python": sys.executable,
        },
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run Qiongli subject runtime smoke cases.")
    parser.add_argument("--fixture-dir", type=Path, default=FIXTURE_DIR)
    parser.add_argument("--workspace-root", type=Path)
    parser.add_argument("--mode", choices=("preview", "local-agent"), default="preview")
    parser.add_argument("--case", action="append", default=[])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    report = run_smoke_suite(
        fixture_dir=args.fixture_dir,
        workspace_root=args.workspace_root,
        mode=args.mode,
        selected_cases=list(args.case),
    )
    print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    return 0 if report["summary"]["failed"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 6: Run targeted smoke tests**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected:

```text
Ran 4 tests
OK
```

- [ ] **Step 7: Run the smoke runner directly**

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --mode preview --json
```

Expected JSON includes:

```json
{
  "mode": "preview",
  "summary": {
    "total": 4,
    "passed": 4,
    "failed": 0
  }
}
```

- [ ] **Step 8: Commit runner**

Run:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/test_subject_runtime_smoke.py
git commit -m "test(smoke): add subject runtime smoke runner"
```

Expected: commit succeeds and includes the runner plus tests.

## Track B: Core Subject Runtime V2

### Task B1: Add Structured Signal Ledger

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
- Modify: `tests/test_subject_refinement.py`
- Modify: `content/standards/subject-refinement-contract.yaml`

- [ ] **Step 1: Add failing signal tests**

Append to `tests/test_subject_refinement.py` inside `SubjectRefinementTests`:

```python
    def test_finance_suggestion_includes_structured_signals(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "earnings announcement stock returns",
                "context": "Use CRSP abnormal returns and an event study for Journal of Finance.",
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        signals = packet["signals"]
        self.assertTrue(signals)
        self.assertTrue(
            any(
                signal["id"] == "finance.method.event-study"
                and signal["dimension"] == "method"
                and signal["weight"] > 0
                for signal in signals
            )
        )
        self.assertTrue(any(signal["dimension"] == "data_or_outcome" for signal in signals))
        self.assertTrue(any(signal["dimension"] == "venue" for signal in signals))
        self.assertIn("snippet", signals[0])

    def test_no_subject_packet_keeps_empty_signal_ledger(self) -> None:
        packet = infer_subject_refinement(
            {"topic": "revise introduction", "context": "Tighten the framing."},
            manifest_state=ProjectManifest(),
        ).to_packet()

        self.assertEqual(packet["signals"], [])
        self.assertEqual(packet["evidence"], [])
```

- [ ] **Step 2: Run signal tests and verify failure**

Run:

```bash
uv run python -m unittest tests.test_subject_refinement.SubjectRefinementTests.test_finance_suggestion_includes_structured_signals tests.test_subject_refinement.SubjectRefinementTests.test_no_subject_packet_keeps_empty_signal_ledger
```

Expected: fail with a missing `signals` key.

- [ ] **Step 3: Extend the dataclasses**

In `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`, add `signals` to `SubjectSignals` and `SubjectRefinementPacket`:

```python
@dataclass(frozen=True)
class SubjectSignals:
    finance_method_lenses: list[str]
    finance_data_outcomes: list[str]
    finance_venues: list[str]
    economics_method_lenses: list[str]
    economics_venues: list[str]
    evidence: list[str]
    signals: list[dict[str, Any]]
```

```python
@dataclass(frozen=True)
class SubjectRefinementPacket:
    decision: str
    mode: str
    active_subject: str
    primary_subject: str
    secondary_subjects: list[str]
    candidate_subjects: list[dict[str, Any]]
    method_lenses: list[str]
    borrowed_lenses: list[dict[str, Any]]
    loaded_resources: dict[str, Any]
    persistence: dict[str, Any]
    summary: str
    domain: str
    confidence: float = 0.0
    evidence: list[str] | None = None
    signals: list[dict[str, Any]] | None = None
```

Update `to_packet()` with:

```python
            "signals": [_copy_record(signal) for signal in self.signals or []],
```

- [ ] **Step 4: Build signal records**

Add helper functions in `subject_refinement.py` near `_detect_signals`:

```python
def _signal_record(
    *,
    subject: str,
    dimension: str,
    value: str,
    weight: float,
    text: str,
    pattern: re.Pattern[str],
) -> dict[str, Any]:
    return {
        "id": f"{subject}.{dimension}.{value}",
        "subject": subject,
        "dimension": dimension,
        "value": value,
        "weight": weight,
        "source": "task_text",
        "snippet": _first_snippet(text, pattern),
    }


def _first_snippet(text: str, pattern: re.Pattern[str]) -> str:
    match = pattern.search(text)
    if not match:
        return ""
    start = max(0, match.start() - 40)
    end = min(len(text), match.end() + 40)
    return " ".join(text[start:end].split())


def _detect_signal_records(text: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for lens, pattern in FINANCE_METHOD_PATTERNS.items():
        if pattern.search(text):
            records.append(
                _signal_record(
                    subject="finance",
                    dimension="method",
                    value=lens,
                    weight=0.35,
                    text=text,
                    pattern=pattern,
                )
            )
    for label, pattern in {
        "finance-data": FINANCE_DATA_OUTCOME_PATTERNS[0],
        "finance-outcome": FINANCE_DATA_OUTCOME_PATTERNS[1],
    }.items():
        if pattern.search(text):
            records.append(
                _signal_record(
                    subject="finance",
                    dimension="data_or_outcome",
                    value=label,
                    weight=0.30,
                    text=text,
                    pattern=pattern,
                )
            )
    for label, pattern in {
        "journal-of-finance": FINANCE_VENUE_PATTERNS[0],
        "journal-of-financial-economics": FINANCE_VENUE_PATTERNS[1],
        "review-of-financial-studies": FINANCE_VENUE_PATTERNS[2],
    }.items():
        if pattern.search(text):
            records.append(
                _signal_record(
                    subject="finance",
                    dimension="venue",
                    value=label,
                    weight=0.20,
                    text=text,
                    pattern=pattern,
                )
            )
    for lens, pattern in ECONOMICS_METHOD_PATTERNS.items():
        if pattern.search(text):
            records.append(
                _signal_record(
                    subject="economics",
                    dimension="method",
                    value=lens,
                    weight=0.40,
                    text=text,
                    pattern=pattern,
                )
            )
    for label, pattern in {
        "american-economic-review": ECONOMICS_VENUE_PATTERNS[0],
        "quarterly-journal-of-economics": ECONOMICS_VENUE_PATTERNS[1],
        "journal-of-political-economy": ECONOMICS_VENUE_PATTERNS[2],
    }.items():
        if pattern.search(text):
            records.append(
                _signal_record(
                    subject="economics",
                    dimension="venue",
                    value=label,
                    weight=0.20,
                    text=text,
                    pattern=pattern,
                )
            )
    return _unique_records(records, key="id")
```

- [ ] **Step 5: Attach signals in every packet path**

In `_detect_signals`, assign `signal_records = _detect_signal_records(text)` and include `signals=signal_records` in the returned `SubjectSignals`.

In each `SubjectRefinementPacket(...)` construction, pass:

```python
signals=signals.signals,
```

For the final `no_subject` construction, pass:

```python
signals=[],
```

- [ ] **Step 6: Extend contract documentation**

Add to `content/standards/subject-refinement-contract.yaml`:

```yaml
signal_dimensions:
  method:
    description: "Named method lens signals such as event-study, asset-pricing, DID, or causal-identification."
  data_or_outcome:
    description: "Dataset, measurement, or outcome language that strengthens subject attribution."
  venue:
    description: "Journal or venue language that strengthens subject attribution."

signal_weights:
  finance:
    method: 0.35
    data_or_outcome: 0.30
    venue: 0.20
  economics:
    method: 0.40
    venue: 0.20
```

- [ ] **Step 7: Run signal tests**

Run:

```bash
uv run python -m unittest tests.test_subject_refinement
```

Expected:

```text
OK
```

- [ ] **Step 8: Commit signal ledger**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_refinement.py tests/test_subject_refinement.py content/standards/subject-refinement-contract.yaml
git commit -m "feat(subjects): add refinement signal ledger"
```

Expected: commit succeeds with one feature slice.

### Task B2: Add Adaptive Resource Activation Planner

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/subject_resources.py`
- Create: `tests/test_subject_resources.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `tests/test_mcp_tool_handlers.py`
- Modify: `tests/test_orchestrator_subject_refinement.py`

- [ ] **Step 1: Add resource planner tests**

Create `tests/test_subject_resources.py`:

```python
from __future__ import annotations

import unittest

from bridges.subject_resources import build_resource_activation_plan


class SubjectResourcePlannerTests(unittest.TestCase):
    def test_suggest_subject_loads_core_subject_and_method_resources(self) -> None:
        plan = build_resource_activation_plan(
            decision="suggest_subject",
            active_subject="auto",
            primary_subject="finance",
            loaded_resources={
                "levels": ["subject_overlay", "subject_skill", "method_pack"],
                "overlays": ["overlays/finance.yaml"],
                "subject_skills": ["skills/finance/SKILL.md"],
                "method_packs": ["method-packs/finance/event-study.yaml"],
                "standards": ["subject-refinement-contract.yaml"],
                "contract_warnings": [],
            },
            method_lenses=["event-study"],
            borrowed_lenses=[],
            persistence={"status": "proposed"},
        )

        self.assertEqual(plan["levels"], ["core", "subject_overlay", "subject_skill", "method_pack"])
        self.assertEqual(plan["persistence_recommendation"]["recommended_subject_mode"], "suggested")
        self.assertFalse(plan["persistence_recommendation"]["write_manifest"])
        self.assertTrue(any(item["kind"] == "subject_skill" for item in plan["resources"]))
        self.assertTrue(any(item.get("lens") == "event-study" for item in plan["resources"]))

    def test_borrow_lens_activates_method_pack_without_subject_switch(self) -> None:
        plan = build_resource_activation_plan(
            decision="borrow_lens",
            active_subject="auto",
            primary_subject="auto",
            loaded_resources={
                "levels": ["method_pack_only"],
                "overlays": [],
                "subject_skills": [],
                "method_packs": ["method-packs/finance/event-study.yaml"],
                "standards": ["subject-refinement-contract.yaml"],
                "contract_warnings": [],
            },
            method_lenses=[],
            borrowed_lenses=[
                {
                    "source_subject": "finance",
                    "lens": "event-study",
                    "resource_level": "method_pack_only",
                    "reason": "finance method-only signal; keep active subject",
                }
            ],
            persistence={"status": "temporary"},
        )

        self.assertEqual(plan["levels"], ["core", "method_pack_only"])
        self.assertEqual(plan["primary_subject"], "auto")
        self.assertEqual(plan["resources"][0]["activation"], "temporary")
        self.assertEqual(plan["resources"][0]["subject"], "finance")


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run planner tests and verify failure**

Run:

```bash
uv run python -m unittest tests.test_subject_resources
```

Expected: fail with a missing `bridges.subject_resources` module.

- [ ] **Step 3: Implement `subject_resources.py`**

Create `packages/python-qiongli/src/qiongli/bridges/subject_resources.py`:

```python
from __future__ import annotations

from pathlib import PurePosixPath
from typing import Any


def build_resource_activation_plan(
    *,
    decision: str,
    active_subject: str,
    primary_subject: str,
    loaded_resources: dict[str, Any],
    method_lenses: list[str],
    borrowed_lenses: list[dict[str, Any]],
    persistence: dict[str, Any],
) -> dict[str, Any]:
    status = str(persistence.get("status") or "none")
    resources: list[dict[str, Any]] = []
    for path in loaded_resources.get("overlays", []):
        resources.append(_resource("subject_overlay", primary_subject, path, status))
    for path in loaded_resources.get("subject_skills", []):
        resources.append(_resource("subject_skill", primary_subject, path, status))
    method_paths = list(loaded_resources.get("method_packs", []))
    for index, path in enumerate(method_paths):
        lens = _lens_for_path(path)
        subject = _subject_for_method_pack(path)
        borrowed = _borrowed_lens_for_path(borrowed_lenses, lens)
        resources.append(
            {
                "kind": "method_pack_only" if borrowed else "method_pack",
                "subject": borrowed.get("source_subject", subject) if borrowed else subject,
                "lens": borrowed.get("lens", lens) if borrowed else lens,
                "path": path,
                "activation": status,
                "order": index,
            }
        )
    return {
        "decision": decision,
        "active_subject": active_subject,
        "primary_subject": primary_subject,
        "levels": _normalized_levels(loaded_resources.get("levels", [])),
        "resources": resources,
        "persistence_recommendation": _persistence_recommendation(status),
        "contract_warnings": list(loaded_resources.get("contract_warnings", [])),
    }


def _resource(kind: str, subject: str, path: str, status: str) -> dict[str, Any]:
    return {
        "kind": kind,
        "subject": subject,
        "path": path,
        "activation": status,
    }


def _normalized_levels(levels: list[str]) -> list[str]:
    normalized = ["core"]
    for level in levels:
        if level == "core_only":
            continue
        if level not in normalized:
            normalized.append(level)
    return normalized


def _persistence_recommendation(status: str) -> dict[str, Any]:
    mode = {
        "none": "auto",
        "temporary": "auto",
        "proposed": "suggested",
        "applied": "confirmed",
        "locked": "locked",
    }.get(status, "auto")
    return {
        "status": status,
        "write_manifest": False,
        "recommended_subject_mode": mode,
    }


def _lens_for_path(path: str) -> str:
    return PurePosixPath(path).stem


def _subject_for_method_pack(path: str) -> str:
    parts = PurePosixPath(path).parts
    if len(parts) >= 3 and parts[0] == "method-packs":
        return parts[1]
    return "auto"


def _borrowed_lens_for_path(
    borrowed_lenses: list[dict[str, Any]],
    lens: str,
) -> dict[str, Any]:
    for item in borrowed_lenses:
        if item.get("lens") == lens:
            return item
    return {}
```

- [ ] **Step 4: Attach planner to refinement packet**

In `subject_refinement.py`, import:

```python
from .subject_resources import build_resource_activation_plan
```

Add `resource_activation_plan` to `SubjectRefinementPacket`:

```python
    resource_activation_plan: dict[str, Any] | None = None
```

Update `to_packet()` with:

```python
            "resource_activation_plan": dict(self.resource_activation_plan or {}),
```

Add helper:

```python
def _activation_plan_for(packet: SubjectRefinementPacket) -> dict[str, Any]:
    return build_resource_activation_plan(
        decision=packet.decision,
        active_subject=packet.active_subject,
        primary_subject=packet.primary_subject,
        loaded_resources=packet.loaded_resources,
        method_lenses=packet.method_lenses,
        borrowed_lenses=packet.borrowed_lenses,
        persistence=packet.persistence,
    )
```

Because `SubjectRefinementPacket` is frozen, construct packets through a local helper instead of setting the field after creation:

```python
def _packet(**kwargs: Any) -> SubjectRefinementPacket:
    base = SubjectRefinementPacket(**kwargs)
    return SubjectRefinementPacket(
        **{
            **base.__dict__,
            "resource_activation_plan": build_resource_activation_plan(
                decision=base.decision,
                active_subject=base.active_subject,
                primary_subject=base.primary_subject,
                loaded_resources=base.loaded_resources,
                method_lenses=base.method_lenses,
                borrowed_lenses=base.borrowed_lenses,
                persistence=base.persistence,
            ),
        }
    )
```

Replace every direct `return SubjectRefinementPacket(` in `infer_subject_refinement` with `return _packet(`.

- [ ] **Step 5: Add packet tests**

Append to `tests/test_subject_refinement.py`:

```python
    def test_packet_includes_resource_activation_plan(self) -> None:
        packet = infer_subject_refinement(
            {
                "topic": "earnings announcement returns",
                "context": "Estimate abnormal returns using CRSP data and an event study for Journal of Finance.",
            },
            manifest_state=ProjectManifest(),
        ).to_packet()

        plan = packet["resource_activation_plan"]
        self.assertEqual(plan["decision"], "suggest_subject")
        self.assertEqual(plan["primary_subject"], "finance")
        self.assertIn("core", plan["levels"])
        self.assertTrue(any(item["kind"] == "subject_skill" for item in plan["resources"]))
```

- [ ] **Step 6: Improve prompt rendering**

In `orchestrator.py`, update both runtime subject refinement prompt blocks so they include the plan and signals:

```python
                + "; loaded_resources: "
                + str(subject_refinement.get("loaded_resources", {}))
                + "; resource_activation_plan: "
                + str(subject_refinement.get("resource_activation_plan", {}))
                + "; signals: "
                + str(subject_refinement.get("signals", []))
                + "\n"
```

- [ ] **Step 7: Add MCP preview assertion**

In `tests/test_mcp_tool_handlers.py`, add a test using `_call_task_run_preview`:

```python
    def test_task_run_preview_exposes_resource_activation_plan(self) -> None:
        result, _stub = self._call_task_run_preview(
            {
                "task_id": "C1",
                "paper_type": "empirical",
                "topic": "earnings announcement reaction",
                "context": "Use CRSP abnormal returns and an event study for Journal of Finance.",
                "domain": "auto",
            }
        )

        preview = result["structuredContent"]["data"]["task_run_preview"]
        refinement = preview["subject_refinement"]
        self.assertEqual(refinement["decision"], "suggest_subject")
        self.assertEqual(refinement["resource_activation_plan"]["primary_subject"], "finance")
        self.assertTrue(refinement["signals"])
```

- [ ] **Step 8: Add orchestrator prompt assertion**

In `tests/test_orchestrator_subject_refinement.py`, extend `test_builds_subject_refinement_for_real_task_run_packet`:

```python
        self.assertIn("resource_activation_plan", packet["subject_refinement"])
        self.assertIn("signals", packet["subject_refinement"])
        self.assertIn("resource_activation_plan", draft_prompt)
        self.assertIn("signals", review_prompt)
```

- [ ] **Step 9: Run planner and integration tests**

Run:

```bash
uv run python -m unittest tests.test_subject_resources tests.test_subject_refinement tests.test_mcp_tool_handlers tests.test_orchestrator_subject_refinement
```

Expected:

```text
OK
```

- [ ] **Step 10: Commit resource planner**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_resources.py packages/python-qiongli/src/qiongli/bridges/subject_refinement.py packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_subject_resources.py tests/test_subject_refinement.py tests/test_mcp_tool_handlers.py tests/test_orchestrator_subject_refinement.py
git commit -m "feat(runtime): plan adaptive subject resources"
```

Expected: commit succeeds with the planner, packet, prompt, and preview assertions.

### Task B3: Persist Cross-Run Subject Evidence Memory

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- Modify: `tests/test_guidance_runtime.py`
- Modify: `content/standards/subject-refinement-contract.yaml`

- [ ] **Step 1: Add guidance memory test**

Append to `tests/test_guidance_runtime.py` inside `GuidanceRuntimeTests`:

```python
    def test_repeated_subject_suggestions_update_subject_evidence_memory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)

            for run_id in ("finance-run-1", "finance-run-2"):
                state = effective_guidance(root, mode="propose", run_id=run_id)
                write_guidance_trace(
                    project_root=root,
                    guidance_state=state,
                    task_packet={
                        "task_id": "C1",
                        "paper_type": "empirical",
                        "topic": "earnings announcement stock market reaction",
                        "context": "Use CRSP abnormal returns and an event study for Journal of Finance.",
                    },
                    draft_content="draft",
                    review_content="review",
                    merged_analysis="merged",
                    validator_gate={"passed": True, "found": [], "missing": [], "checked": 0},
                    applied=False,
                )

            memory_path = root / ".qiongli" / "trace" / "subject_evidence.json"
            self.assertTrue(memory_path.is_file())
            memory = json.loads(memory_path.read_text(encoding="utf-8"))
            self.assertEqual(memory["subjects"]["finance"]["suggestion_count"], 2)
            self.assertEqual(memory["subjects"]["finance"]["last_decision"], "suggest_subject")

            proposal = (
                root
                / ".qiongli"
                / "trace"
                / "runs"
                / "finance-run-2"
                / "guidance_update_proposal.md"
            ).read_text(encoding="utf-8")
            self.assertIn("subject_mode: suggested", proposal)
            self.assertIn("confirm finance", proposal)
```

- [ ] **Step 2: Run guidance memory test and verify failure**

Run:

```bash
uv run python -m unittest tests.test_guidance_runtime.GuidanceRuntimeTests.test_repeated_subject_suggestions_update_subject_evidence_memory
```

Expected: fail because `subject_evidence.json` is not created.

- [ ] **Step 3: Add trace path constant and memory helpers**

In `guidance_runtime.py`, add:

```python
SUBJECT_EVIDENCE_REL = TRACE_REL / "subject_evidence.json"
```

Add helper functions near `_write_json`:

```python
def _subject_evidence_path(paths: GuidancePaths) -> Path:
    return paths.project_root / SUBJECT_EVIDENCE_REL


def _load_subject_evidence(paths: GuidancePaths) -> dict[str, Any]:
    path = _subject_evidence_path(paths)
    if not path.is_file():
        return {"schema_version": "1.0", "subjects": {}}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {"schema_version": "1.0", "subjects": {}, "warnings": ["invalid previous subject evidence"]}


def _update_subject_evidence(
    *,
    paths: GuidancePaths,
    run_id: str,
    subject_refinement: dict[str, Any],
) -> dict[str, Any]:
    memory = _load_subject_evidence(paths)
    subjects = memory.setdefault("subjects", {})
    subject = str(subject_refinement.get("primary_subject") or "auto")
    decision = str(subject_refinement.get("decision") or "")
    if subject in {"", "auto"} or decision not in {"suggest_subject", "confirm_subject", "lock_subject"}:
        _write_json(_subject_evidence_path(paths), memory)
        return memory
    record = subjects.setdefault(
        subject,
        {
            "suggestion_count": 0,
            "last_decision": "",
            "last_confidence": 0.0,
            "last_run_id": "",
            "signals": [],
        },
    )
    if decision == "suggest_subject":
        record["suggestion_count"] = int(record.get("suggestion_count", 0)) + 1
    record["last_decision"] = decision
    record["last_confidence"] = float(subject_refinement.get("confidence") or 0.0)
    record["last_run_id"] = run_id
    record["signals"] = list(subject_refinement.get("signals") or [])
    _write_json(_subject_evidence_path(paths), memory)
    return memory


def _subject_promotion_recommendation(memory: dict[str, Any], subject_refinement: dict[str, Any]) -> dict[str, Any]:
    subject = str(subject_refinement.get("primary_subject") or "auto")
    if subject in {"", "auto"}:
        return {"status": "none"}
    record = memory.get("subjects", {}).get(subject, {})
    confidence = float(subject_refinement.get("confidence") or 0.0)
    if int(record.get("suggestion_count", 0)) >= 2 and confidence >= 0.75:
        return {
            "status": "recommend_confirmation",
            "subject": subject,
            "subject_mode": "suggested",
            "message": f"Repeated evidence supports asking the user to confirm {subject}.",
        }
    return {"status": "none"}
```

- [ ] **Step 4: Wire memory into trace writing**

In `write_guidance_trace`, after `subject_refinement_packet = subject_refinement.to_packet()`, add:

```python
    subject_evidence = _update_subject_evidence(
        paths=paths,
        run_id=run_id,
        subject_refinement=subject_refinement_packet,
    )
    promotion_recommendation = _subject_promotion_recommendation(
        subject_evidence,
        subject_refinement_packet,
    )
    subject_refinement_packet["subject_evidence_memory"] = subject_evidence
    subject_refinement_packet["promotion_recommendation"] = promotion_recommendation
```

Keep the existing `_write_json(run_dir / "subject_refinement.json", subject_refinement_packet)` after these fields are attached so each run bundle captures the memory snapshot.

Add `subject_evidence_memory` and `promotion_recommendation` to the returned trace record if the current return payload does not already include the full `subject_refinement` packet.

- [ ] **Step 5: Include recommendation in proposal text**

In `_proposal_text`, append a subject section when the packet contains a promotion recommendation:

```python
    promotion = subject_refinement.get("promotion_recommendation", {})
    if promotion.get("status") == "recommend_confirmation":
        lines.extend(
            [
                "",
                "## Subject Confirmation Proposal",
                "",
                f"- active_subject: {promotion['subject']}",
                f"- subject_mode: {promotion['subject_mode']}",
                f"- action: ask the user to confirm {promotion['subject']} before writing the manifest",
            ]
        )
```

- [ ] **Step 6: Document promotion threshold**

Add to `content/standards/subject-refinement-contract.yaml`:

```yaml
subject_evidence_memory:
  path: ".qiongli/trace/subject_evidence.json"
  promotion_policy:
    minimum_repeated_suggestions: 2
    minimum_confidence: 0.75
    write_manifest_automatically: false
    recommendation_subject_mode: "suggested"
```

- [ ] **Step 7: Run guidance runtime tests**

Run:

```bash
uv run python -m unittest tests.test_guidance_runtime
```

Expected:

```text
OK
```

- [ ] **Step 8: Commit evidence memory**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py tests/test_guidance_runtime.py content/standards/subject-refinement-contract.yaml
git commit -m "feat(guidance): persist subject evidence memory"
```

Expected: commit succeeds with the memory and proposal behavior.

## Track Integration

### Task I1: Merge Core Track Into `dev`

**Files:**
- No direct source edit unless conflict resolution is required.

- [ ] **Step 1: Validate Track B before merge**

Run in Track B worktree:

```bash
uv run python -m unittest tests.test_subject_refinement tests.test_subject_resources tests.test_guidance_runtime tests.test_mcp_tool_handlers tests.test_orchestrator_subject_refinement
```

Expected:

```text
OK
```

- [ ] **Step 2: Switch to root worktree `dev`**

Run in repository root:

```bash
git switch dev
```

Expected: branch is `dev`.

- [ ] **Step 3: Merge Track B with a non-fast-forward feature commit**

Run:

```bash
git merge --no-ff feature/subject-refinement-core-v2
```

Expected: merge succeeds. If conflicts occur, resolve only files touched by Track B and rerun the tests from Step 1.

### Task I2: Rebase Track A And Expand Smoke Assertions

**Files:**
- Modify: `tooling/scripts/run_subject_runtime_smoke.py`
- Modify: `tests/test_subject_runtime_smoke.py`

- [ ] **Step 1: Rebase Track A on updated `dev`**

Run in Track A worktree:

```bash
git fetch origin
git rebase dev
```

Expected: rebase succeeds. If a conflict appears in smoke tests, keep Track B's new packet fields and Track A's fixture-driven execution.

- [ ] **Step 2: Assert signals and resource activation plan in smoke runner**

Add to `_assert_case` in `tooling/scripts/run_subject_runtime_smoke.py`:

```python
    if refinement.get("decision") != "no_subject":
        if "signals" not in refinement:
            failures.append("missing signals ledger")
        if "resource_activation_plan" not in refinement:
            failures.append("missing resource_activation_plan")
```

- [ ] **Step 3: Add smoke test assertion for richer packets**

Append to `tests/test_subject_runtime_smoke.py`:

```python
    def test_preview_suite_checks_runtime_packet_v2_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        refinement = report["cases"][0]["result"]["data"]["task_run_preview"]["subject_refinement"]
        self.assertTrue(refinement["signals"])
        self.assertEqual(refinement["resource_activation_plan"]["primary_subject"], "finance")
```

- [ ] **Step 4: Run Track A tests after rebase**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
uv run python tooling/scripts/run_subject_runtime_smoke.py --mode preview --json
```

Expected: both commands exit 0 and the JSON report shows four passed cases.

- [ ] **Step 5: Commit Track A v2 assertion update**

Run:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/test_subject_runtime_smoke.py
git commit -m "test(smoke): assert subject runtime v2 packet fields"
```

Expected: commit succeeds.

### Task I3: Merge Smoke Track Into `dev`

**Files:**
- No direct source edit unless conflict resolution is required.

- [ ] **Step 1: Merge Track A**

Run in repository root:

```bash
git switch dev
git merge --no-ff feature/real-smoke-subject-runtime
```

Expected: merge succeeds.

- [ ] **Step 2: Run combined targeted verification**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke tests.test_subject_refinement tests.test_subject_resources tests.test_guidance_runtime tests.test_mcp_tool_handlers tests.test_orchestrator_subject_refinement
```

Expected:

```text
OK
```

- [ ] **Step 3: Run real preview smoke report on merged `dev`**

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --mode preview --json
```

Expected JSON includes:

```json
{
  "mode": "preview",
  "summary": {
    "total": 4,
    "passed": 4,
    "failed": 0
  }
}
```

- [ ] **Step 4: Run full regression set**

Run:

```bash
uv run python -m unittest discover -s tests
node --test packages/npm-qiongli/test/*.test.mjs
git diff --check
```

Expected:

```text
OK
```

The Node command should report all npm package tests passed. `git diff --check` should print no whitespace errors.

## Commit Boundaries

Use these commit boundaries during implementation:

1. Track A: `test(smoke): add subject runtime fixtures`
2. Track A: `test(smoke): add subject runtime smoke runner`
3. Track B: `feat(subjects): add refinement signal ledger`
4. Track B: `feat(runtime): plan adaptive subject resources`
5. Track B: `feat(guidance): persist subject evidence memory`
6. Track A after rebase: `test(smoke): assert subject runtime v2 packet fields`

After both tracks land on `dev`, squash only if the history becomes harder to review than the feature boundaries above. If squashing, keep one core feature commit and one smoke-test commit:

- `feat(runtime): deepen adaptive subject refinement`
- `test(smoke): add real subject runtime smoke harness`

## Acceptance Criteria

- Default smoke execution calls the real `qiongli_task_run` MCP handler and never launches local agents.
- Local-agent smoke mode is guarded by both `--mode local-agent` and `QIONGLI_SMOKE_RUN_AGENTS=1`.
- Smoke fixtures cover core-only, borrowed-lens, suggested-finance, and locked-subject behavior.
- `subject_refinement` packets preserve all current fields and add `signals` plus `resource_activation_plan`.
- Borrowed lenses do not switch the active subject or write project manifest state.
- Repeated high-confidence suggestions write `.qiongli/trace/subject_evidence.json` and produce a confirmation proposal, not an automatic manifest update.
- MCP preview and real orchestrator task-run packets both expose the richer subject runtime packet.
- Full Python and npm test suites pass on merged `dev`.

## Reviewer Checklist

- Check that Track A uses only temporary project/config directories and does not write to the developer's real `~/.qiongli`, `~/.codex`, or client config.
- Check that `run_agents` remains false unless the explicit local-agent smoke mode is selected.
- Check that Track B does not break existing callers that only read old `subject_refinement` keys.
- Check that `resource_activation_plan` describes what should be loaded without pretending unavailable files were loaded.
- Check that evidence memory proposes subject confirmation without silently changing `.qiongli/guidance_manifest.yaml`.
- Check that prompt additions are compact enough for ordinary task runs.
