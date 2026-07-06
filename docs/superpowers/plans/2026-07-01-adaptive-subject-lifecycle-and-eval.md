# Adaptive Subject Lifecycle and Evaluation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first operational layer for adaptive subject refinement: a measurable subject router evaluation suite plus CLI/MCP controls for inspecting, confirming, dismissing, resetting, locking, and unlocking inferred subject guidance.

**Architecture:** Keep the full install subject-agnostic. Core Qiongli remains the default runtime; subject packs are activated through evidence, project state, and user confirmation. The new lifecycle module owns user decisions and subject state transitions. The existing guidance runtime continues to infer subject refinements and generate preview-first local guidance, while the evaluation runner measures router behavior with curated fixtures.

**Tech Stack:** Python 3.11+, `uv`, `unittest`, existing `qiongli.bridges` modules, existing CLI parser in `qiongli/cli.py`, existing MCP handler registry in `qiongli/bridges/mcp_tool_handlers.py`, JSON fixtures under `tests/fixtures/`.

---

## Current Code Contracts

The implementation must preserve these existing contracts:

- `packages/python-qiongli/src/qiongli/bridges/project_manifest.py`
  - `OFFICIAL_SUBJECTS`
  - `ProjectManifest`
  - `ProjectManifestState`
  - `load_project_manifest(project_root)`
  - `init_project_manifest(project_root, overwrite=False)`
  - `update_project_manifest(project_root, active_subject=None, subject_mode=None, secondary_subjects=None, venue_profiles=None, method_lenses=None, strictness=None)`
  - Unknown manifest fields are preserved by the update path.

- `packages/python-qiongli/src/qiongli/bridges/subject_refinement.py`
  - `infer_subject_refinement(project_root, request, manifest)` returns preview-oriented subject recommendations.

- `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
  - `.qiongli/trace/subject_evidence.json` already stores subject evidence.
  - `_update_subject_evidence(...)` writes subject evidence during guidance preview/application runs.
  - `_subject_promotion_recommendation(...)` currently creates promotion recommendations from repeated evidence.
  - This task may expose small public helpers from this module, but must not move the guidance generation flow.

- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - `MCP_TOOL_DEFINITIONS` declares tool schemas.
  - `call_qiongli_tool(name, arguments)` dispatches tool calls.

- `packages/python-qiongli/src/qiongli/cli.py`
  - Existing top-level commands are defined with `argparse`.
  - New `subject` commands should follow the same parser style and return process status codes through command functions.

## New Files

- `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`
- `tooling/scripts/evaluate_subject_router.py`
- `tests/test_subject_lifecycle.py`
- `tests/test_subject_router_eval.py`
- `tests/fixtures/subject_router_eval/clear_finance.json`
- `tests/fixtures/subject_router_eval/clear_economics.json`
- `tests/fixtures/subject_router_eval/finance_method_only_borrow.json`
- `tests/fixtures/subject_router_eval/economics_method_only_borrow.json`
- `tests/fixtures/subject_router_eval/mixed_econ_finance.json`
- `tests/fixtures/subject_router_eval/weak_core_only.json`
- `tests/fixtures/subject_router_eval/near_miss_finance.json`
- `tests/fixtures/subject_router_eval/locked_subject_neighbor_lens.json`

## Modified Files

- `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- `packages/python-qiongli/src/qiongli/cli.py`
- `tests/test_cli.py`
- `tests/test_guidance_runtime.py`
- `tests/test_mcp_tool_handlers.py`

## Branch and Worktree Setup

- [ ] Start in the repository root and confirm the base branch:

```bash
git status --short --branch
```

Expected:

```text
## dev...origin/dev
```

Additional commits ahead of `origin/dev` are acceptable if they are intentional local development history.

- [ ] Create an isolated implementation worktree:

```bash
git worktree add .worktrees/adaptive-subject-lifecycle-eval -b feature/adaptive-subject-lifecycle-eval dev
```

Expected:

```text
Preparing worktree (new branch 'feature/adaptive-subject-lifecycle-eval')
HEAD is now at <sha> <message>
```

- [ ] Enter the worktree:

```bash
cd .worktrees/adaptive-subject-lifecycle-eval
```

- [ ] Run the focused baseline suite:

```bash
uv run python -m unittest \
  tests.test_subject_refinement \
  tests.test_guidance_runtime \
  tests.test_mcp_tool_handlers \
  tests.test_cli
```

Expected:

```text
OK
```

If the baseline fails, record the failing test names and stop before editing. Do not rewrite unrelated behavior to make a pre-existing failure disappear.

---

## Task 1: Add Router Evaluation Fixtures

**Purpose:** Create a stable, reviewable fixture corpus that captures clear matches, weak signals, neighbor-borrowing cases, mixed subject cases, near-misses, and locked-subject behavior.

### 1.1 Create the fixture directory

- [ ] Create:

```text
tests/fixtures/subject_router_eval/
```

### 1.2 Add `clear_finance.json`

- [ ] Create `tests/fixtures/subject_router_eval/clear_finance.json`:

```json
{
  "id": "clear_finance",
  "description": "Corporate finance request with valuation and risk signals.",
  "request": "Design a study of corporate bond spread reactions to earnings guidance surprises. I need asset pricing controls, event-study windows, liquidity filters, and robustness checks for abnormal returns.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "finance",
    "suggest_subjects": ["finance"],
    "forbidden_subjects": ["economics"],
    "method_lenses": ["asset_pricing", "event_study"]
  }
}
```

### 1.3 Add `clear_economics.json`

- [ ] Create `tests/fixtures/subject_router_eval/clear_economics.json`:

```json
{
  "id": "clear_economics",
  "description": "Labor economics request with identification and policy evaluation signals.",
  "request": "Help design a difference-in-differences study on minimum wage increases, employment effects, spillovers across commuting zones, pre-trends, and heterogeneous treatment effects.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "economics",
    "suggest_subjects": ["economics"],
    "forbidden_subjects": ["finance"],
    "method_lenses": ["causal_inference", "policy_evaluation"]
  }
}
```

### 1.4 Add `finance_method_only_borrow.json`

- [ ] Create `tests/fixtures/subject_router_eval/finance_method_only_borrow.json`:

```json
{
  "id": "finance_method_only_borrow",
  "description": "Finance remains primary while borrowing econometric identification language.",
  "request": "I am writing an empirical asset pricing paper and need guidance on portfolio sorts, factor regressions, Fama-MacBeth estimates, and whether a difference-in-differences appendix is credible after an index inclusion shock.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "finance",
    "suggest_subjects": ["finance"],
    "allowed_neighbor_subjects": ["economics"],
    "forbidden_subjects": [],
    "method_lenses": ["asset_pricing", "causal_inference"]
  }
}
```

### 1.5 Add `economics_method_only_borrow.json`

- [ ] Create `tests/fixtures/subject_router_eval/economics_method_only_borrow.json`:

```json
{
  "id": "economics_method_only_borrow",
  "description": "Economics remains primary while borrowing financial market measurement language.",
  "request": "I need a development economics study on credit access after a banking reform, with household welfare outcomes, quasi-experimental identification, and market-based measures of local credit supply.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "economics",
    "suggest_subjects": ["economics"],
    "allowed_neighbor_subjects": ["finance"],
    "forbidden_subjects": [],
    "method_lenses": ["causal_inference", "policy_evaluation"]
  }
}
```

### 1.6 Add `mixed_econ_finance.json`

- [ ] Create `tests/fixtures/subject_router_eval/mixed_econ_finance.json`:

```json
{
  "id": "mixed_econ_finance",
  "description": "Request genuinely spans economics and finance; primary is economics because policy identification dominates.",
  "request": "Build a paper on how monetary policy shocks affect bank lending, firm investment, and household employment. I need local projections, event windows around FOMC surprises, distributional effects, and financial intermediary balance-sheet channels.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "recommend",
    "primary_subject": "economics",
    "suggest_subjects": ["economics", "finance"],
    "allowed_neighbor_subjects": ["finance"],
    "forbidden_subjects": [],
    "method_lenses": ["causal_inference", "asset_pricing"]
  }
}
```

### 1.7 Add `weak_core_only.json`

- [ ] Create `tests/fixtures/subject_router_eval/weak_core_only.json`:

```json
{
  "id": "weak_core_only",
  "description": "General academic planning request should remain on core guidance.",
  "request": "Help me organize a research workflow, write a stronger introduction, plan the related work section, and prepare a reproducible folder structure for my draft.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "core_only",
    "primary_subject": "core",
    "suggest_subjects": [],
    "forbidden_subjects": ["finance", "economics"],
    "method_lenses": []
  }
}
```

### 1.8 Add `near_miss_finance.json`

- [ ] Create `tests/fixtures/subject_router_eval/near_miss_finance.json`:

```json
{
  "id": "near_miss_finance",
  "description": "Mentions budget and funding, but the task is project management rather than finance research.",
  "request": "Create a timeline and budget table for a grant-funded archive digitization project. I need staffing milestones, equipment cost categories, and a reporting calendar.",
  "manifest": {
    "active_subject": "auto",
    "subject_mode": "auto",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": [],
    "strictness": "standard"
  },
  "expected": {
    "decision": "core_only",
    "primary_subject": "core",
    "suggest_subjects": [],
    "forbidden_subjects": ["finance", "economics"],
    "method_lenses": []
  }
}
```

### 1.9 Add `locked_subject_neighbor_lens.json`

- [ ] Create `tests/fixtures/subject_router_eval/locked_subject_neighbor_lens.json`:

```json
{
  "id": "locked_subject_neighbor_lens",
  "description": "Locked economics project may borrow a finance lens without changing primary subject.",
  "request": "Within my economics project, add financial market event-study evidence around bank regulation announcements, but keep the core framing in policy evaluation and identification.",
  "manifest": {
    "active_subject": "economics",
    "subject_mode": "locked",
    "secondary_subjects": [],
    "venue_profiles": [],
    "method_lenses": ["causal_inference"],
    "strictness": "standard"
  },
  "expected": {
    "decision": "keep_locked",
    "primary_subject": "economics",
    "suggest_subjects": ["finance"],
    "allowed_neighbor_subjects": ["finance"],
    "forbidden_subjects": [],
    "method_lenses": ["causal_inference", "asset_pricing"]
  }
}
```

### 1.10 Verify and commit fixtures

- [ ] Run:

```bash
python -m json.tool tests/fixtures/subject_router_eval/clear_finance.json >/dev/null
python -m json.tool tests/fixtures/subject_router_eval/clear_economics.json >/dev/null
python -m json.tool tests/fixtures/subject_router_eval/finance_method_only_borrow.json >/dev/null
python -m json.tool tests/fixtures/subject_router_eval/economics_method_only_borrow.json >/dev/null
python -m json.tool tests/fixtures/subject_router_eval/mixed_econ_finance.json >/dev/null
python -m json.tool tests/fixtures/subject_router_eval/weak_core_only.json >/dev/null
python -m json.tool tests/fixtures/subject_router_eval/near_miss_finance.json >/dev/null
python -m json.tool tests/fixtures/subject_router_eval/locked_subject_neighbor_lens.json >/dev/null
```

Expected: all commands exit with code `0`.

- [ ] Commit:

```bash
git add tests/fixtures/subject_router_eval
git commit -m "test(subjects): add router evaluation fixtures"
```

---

## Task 2: Build the Subject Router Evaluation Runner

**Purpose:** Add a local quality gate that measures whether adaptive subject routing behaves correctly before broadening subject packs.

### 2.1 Write failing tests first

- [ ] Create `tests/test_subject_router_eval.py`:

```python
import json
import tempfile
import unittest
from pathlib import Path

from tooling.scripts.evaluate_subject_router import (
    DEFAULT_THRESHOLDS,
    EvalCase,
    evaluate_cases,
    load_eval_cases,
    threshold_failures,
)


FIXTURES = Path("tests/fixtures/subject_router_eval")


class SubjectRouterEvalTests(unittest.TestCase):
    def test_load_eval_cases_reads_all_fixture_files(self) -> None:
        cases = load_eval_cases(FIXTURES)

        self.assertEqual(len(cases), 8)
        self.assertEqual({case.case_id for case in cases}, {
            "clear_finance",
            "clear_economics",
            "finance_method_only_borrow",
            "economics_method_only_borrow",
            "mixed_econ_finance",
            "weak_core_only",
            "near_miss_finance",
            "locked_subject_neighbor_lens",
        })

    def test_evaluate_cases_reports_required_metrics(self) -> None:
        cases = load_eval_cases(FIXTURES)

        report = evaluate_cases(cases)

        self.assertEqual(report["case_count"], 8)
        self.assertIn("decision_accuracy", report["metrics"])
        self.assertIn("primary_subject_accuracy", report["metrics"])
        self.assertIn("suggest_subject_precision", report["metrics"])
        self.assertIn("near_miss_false_positives", report["metrics"])
        self.assertEqual(len(report["cases"]), 8)

    def test_threshold_failures_returns_named_failures(self) -> None:
        report = {
            "metrics": {
                "decision_accuracy": 0.5,
                "primary_subject_accuracy": 1.0,
                "suggest_subject_precision": 0.75,
                "near_miss_false_positives": 1,
            }
        }

        failures = threshold_failures(report, DEFAULT_THRESHOLDS)

        self.assertEqual(
            failures,
            [
                "decision_accuracy 0.500 below required 0.900",
                "suggest_subject_precision 0.750 below required 0.850",
                "near_miss_false_positives 1 above allowed 0",
            ],
        )

    def test_loader_rejects_duplicate_case_ids(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            payload = {
                "id": "duplicate",
                "description": "duplicate id",
                "request": "general research workflow",
                "manifest": {
                    "active_subject": "auto",
                    "subject_mode": "auto",
                    "secondary_subjects": [],
                    "venue_profiles": [],
                    "method_lenses": [],
                    "strictness": "standard",
                },
                "expected": {
                    "decision": "core_only",
                    "primary_subject": "core",
                    "suggest_subjects": [],
                    "forbidden_subjects": [],
                    "method_lenses": [],
                },
            }
            (root / "a.json").write_text(json.dumps(payload), encoding="utf-8")
            (root / "b.json").write_text(json.dumps(payload), encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "Duplicate eval case id"):
                load_eval_cases(root)


if __name__ == "__main__":
    unittest.main()
```

- [ ] Run:

```bash
uv run python -m unittest tests.test_subject_router_eval
```

Expected before implementation:

```text
ModuleNotFoundError: No module named 'tooling.scripts.evaluate_subject_router'
```

### 2.2 Implement the runner

- [ ] Create `tooling/scripts/evaluate_subject_router.py` with these public functions and CLI:

```python
#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from qiongli.bridges.project_manifest import ProjectManifest
from qiongli.bridges.subject_refinement import infer_subject_refinement


DEFAULT_FIXTURE_DIR = Path("tests/fixtures/subject_router_eval")
DEFAULT_THRESHOLDS = {
    "decision_accuracy": 0.90,
    "primary_subject_accuracy": 0.90,
    "suggest_subject_precision": 0.85,
    "near_miss_false_positives": 0,
}


@dataclass(frozen=True)
class EvalCase:
    case_id: str
    description: str
    request: str
    manifest: dict[str, Any]
    expected: dict[str, Any]
    source_path: Path


def load_eval_cases(root: Path) -> list[EvalCase]:
    if not root.exists():
        raise FileNotFoundError(f"Evaluation fixture directory not found: {root}")

    cases: list[EvalCase] = []
    seen: set[str] = set()
    for path in sorted(root.glob("*.json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        case_id = str(payload["id"])
        if case_id in seen:
            raise ValueError(f"Duplicate eval case id: {case_id}")
        seen.add(case_id)
        cases.append(
            EvalCase(
                case_id=case_id,
                description=str(payload.get("description", "")),
                request=str(payload["request"]),
                manifest=dict(payload["manifest"]),
                expected=dict(payload["expected"]),
                source_path=path,
            )
        )
    if not cases:
        raise ValueError(f"No evaluation fixtures found in {root}")
    return cases


def _manifest_from_payload(payload: dict[str, Any]) -> ProjectManifest:
    return ProjectManifest(
        active_subject=str(payload.get("active_subject", "auto")),
        subject_mode=str(payload.get("subject_mode", "auto")),
        secondary_subjects=list(payload.get("secondary_subjects", [])),
        venue_profiles=list(payload.get("venue_profiles", [])),
        method_lenses=list(payload.get("method_lenses", [])),
        strictness=str(payload.get("strictness", "standard")),
    )


def _actual_decision(refinement: dict[str, Any], manifest: ProjectManifest) -> str:
    if manifest.subject_mode == "locked":
        return "keep_locked"
    if refinement.get("suggest_subjects") or refinement.get("primary_subject") not in (None, "core", "auto"):
        return "recommend"
    return "core_only"


def _primary_subject(refinement: dict[str, Any], manifest: ProjectManifest) -> str:
    if manifest.subject_mode == "locked" and manifest.active_subject != "auto":
        return manifest.active_subject
    primary = refinement.get("primary_subject") or refinement.get("active_subject")
    if primary in (None, "", "auto"):
        return "core"
    return str(primary)


def _suggested_subjects(refinement: dict[str, Any]) -> list[str]:
    subjects = refinement.get("suggest_subjects", [])
    if isinstance(subjects, str):
        return [subjects]
    return [str(subject) for subject in subjects]


def _method_lenses(refinement: dict[str, Any]) -> list[str]:
    lenses = refinement.get("method_lenses", [])
    if isinstance(lenses, str):
        return [lenses]
    return [str(lens) for lens in lenses]


def run_eval_case(case: EvalCase) -> dict[str, Any]:
    manifest = _manifest_from_payload(case.manifest)
    with tempfile.TemporaryDirectory(prefix=f"qiongli-router-eval-{case.case_id}-") as tmpdir:
        refinement = infer_subject_refinement(Path(tmpdir), case.request, manifest)

    actual = {
        "decision": _actual_decision(refinement, manifest),
        "primary_subject": _primary_subject(refinement, manifest),
        "suggest_subjects": _suggested_subjects(refinement),
        "method_lenses": _method_lenses(refinement),
    }
    expected = case.expected
    forbidden = set(expected.get("forbidden_subjects", []))
    expected_suggestions = set(expected.get("suggest_subjects", []))
    actual_suggestions = set(actual["suggest_subjects"])

    return {
        "id": case.case_id,
        "description": case.description,
        "source": str(case.source_path),
        "expected": expected,
        "actual": actual,
        "passed": {
            "decision": actual["decision"] == expected.get("decision"),
            "primary_subject": actual["primary_subject"] == expected.get("primary_subject"),
            "suggest_subjects": expected_suggestions.issubset(actual_suggestions),
            "forbidden_subjects": not bool(forbidden & actual_suggestions),
        },
    }


def evaluate_cases(cases: list[EvalCase]) -> dict[str, Any]:
    results = [run_eval_case(case) for case in cases]
    decision_hits = sum(1 for result in results if result["passed"]["decision"])
    primary_hits = sum(1 for result in results if result["passed"]["primary_subject"])

    expected_suggestion_count = 0
    correct_suggestion_count = 0
    near_miss_false_positives = 0
    for result in results:
        expected = result["expected"]
        actual = result["actual"]
        expected_suggestions = set(expected.get("suggest_subjects", []))
        actual_suggestions = set(actual.get("suggest_subjects", []))
        expected_suggestion_count += len(expected_suggestions)
        correct_suggestion_count += len(expected_suggestions & actual_suggestions)
        if result["id"].startswith("near_miss") and actual_suggestions:
            near_miss_false_positives += 1

    return {
        "case_count": len(results),
        "metrics": {
            "decision_accuracy": decision_hits / len(results),
            "primary_subject_accuracy": primary_hits / len(results),
            "suggest_subject_precision": (
                correct_suggestion_count / expected_suggestion_count
                if expected_suggestion_count
                else 1.0
            ),
            "near_miss_false_positives": near_miss_false_positives,
        },
        "cases": results,
    }


def threshold_failures(report: dict[str, Any], thresholds: dict[str, float | int]) -> list[str]:
    metrics = report["metrics"]
    failures: list[str] = []
    for metric, required in thresholds.items():
        actual = metrics[metric]
        if metric == "near_miss_false_positives":
            if actual > required:
                failures.append(f"{metric} {actual} above allowed {required}")
        elif actual < required:
            failures.append(f"{metric} {actual:.3f} below required {required:.3f}")
    return failures


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Evaluate Qiongli subject router quality.")
    parser.add_argument("--fixtures", type=Path, default=DEFAULT_FIXTURE_DIR)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    report = evaluate_cases(load_eval_cases(args.fixtures))
    failures = threshold_failures(report, DEFAULT_THRESHOLDS)
    if args.json:
        print(json.dumps({**report, "threshold_failures": failures}, indent=2, sort_keys=True))
    else:
        metrics = report["metrics"]
        print(f"cases: {report['case_count']}")
        for name in sorted(metrics):
            print(f"{name}: {metrics[name]}")
        if failures:
            print("threshold failures:")
            for failure in failures:
                print(f"- {failure}")

    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
```

### 2.3 Verify runner behavior

- [ ] Run:

```bash
uv run python -m unittest tests.test_subject_router_eval
```

Expected:

```text
OK
```

- [ ] Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected:

```json
{
  "case_count": 8,
  "metrics": {
    "decision_accuracy": 1.0,
    "near_miss_false_positives": 0,
    "primary_subject_accuracy": 1.0,
    "suggest_subject_precision": 1.0
  },
  "threshold_failures": []
}
```

The exact ordering of JSON fields may differ. If metrics fall below thresholds, inspect each case result and adjust `subject_refinement.py` only when the fixture expectation is valid and the implementation is genuinely under-routing or over-routing.

- [ ] Commit:

```bash
git add tooling/scripts/evaluate_subject_router.py tests/test_subject_router_eval.py
git commit -m "test(subjects): add subject router evaluation runner"
```

---

## Task 3: Add Subject Lifecycle Runtime

**Purpose:** Centralize user subject decisions so CLI, MCP, and future clients can share the same lifecycle behavior.

### 3.1 Write failing lifecycle tests first

- [ ] Create `tests/test_subject_lifecycle.py`:

```python
import json
import tempfile
import unittest
from pathlib import Path

from qiongli.bridges.project_manifest import load_project_manifest
from qiongli.bridges.subject_lifecycle import (
    SubjectLifecycleError,
    apply_subject_action,
    subject_status,
)


class SubjectLifecycleTests(unittest.TestCase):
    def test_confirm_updates_manifest_and_records_event(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)

            result = apply_subject_action(root, "confirm", "finance", source="test")
            manifest = load_project_manifest(root)
            evidence = json.loads((root / ".qiongli/trace/subject_evidence.json").read_text())

            self.assertEqual(result["manifest"]["active_subject"], "finance")
            self.assertEqual(manifest.active_subject, "finance")
            self.assertEqual(manifest.subject_mode, "confirmed")
            self.assertEqual(evidence["lifecycle_events"][-1]["action"], "confirm")
            self.assertEqual(evidence["lifecycle_events"][-1]["subject"], "finance")

    def test_dismiss_only_updates_trace_memory(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)

            result = apply_subject_action(root, "dismiss", "finance", source="test")

            self.assertFalse((root / ".qiongli/guidance_manifest.yaml").exists())
            self.assertEqual(result["state"]["dismissed_subjects"]["finance"]["source"], "test")

    def test_lock_and_unlock_transition_manifest_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)

            apply_subject_action(root, "lock", "economics", source="test")
            locked = load_project_manifest(root)
            self.assertEqual(locked.active_subject, "economics")
            self.assertEqual(locked.subject_mode, "locked")

            result = apply_subject_action(root, "unlock", source="test")
            unlocked = load_project_manifest(root)
            self.assertEqual(unlocked.active_subject, "economics")
            self.assertEqual(unlocked.subject_mode, "confirmed")
            self.assertEqual(result["manifest"]["subject_mode"], "confirmed")

    def test_reset_returns_to_adaptive_core_and_clears_dismissals(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)

            apply_subject_action(root, "dismiss", "finance", source="test")
            apply_subject_action(root, "confirm", "economics", source="test")
            result = apply_subject_action(root, "reset", source="test")
            manifest = load_project_manifest(root)
            evidence = json.loads((root / ".qiongli/trace/subject_evidence.json").read_text())

            self.assertEqual(manifest.active_subject, "auto")
            self.assertEqual(manifest.subject_mode, "auto")
            self.assertEqual(manifest.secondary_subjects, [])
            self.assertEqual(manifest.method_lenses, [])
            self.assertEqual(result["state"]["dismissed_subjects"], {})
            self.assertEqual(evidence["subjects"], {})

    def test_status_reports_manifest_and_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            root = Path(tmpdir)
            apply_subject_action(root, "confirm", "finance", source="test")

            status = subject_status(root)

            self.assertEqual(status["manifest"]["active_subject"], "finance")
            self.assertEqual(status["manifest"]["subject_mode"], "confirmed")
            self.assertIn("lifecycle_events", status["state"])

    def test_invalid_subject_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            with self.assertRaisesRegex(SubjectLifecycleError, "Unknown subject"):
                apply_subject_action(Path(tmpdir), "confirm", "alchemy", source="test")

    def test_subject_required_for_confirm_dismiss_and_lock(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            for action in ("confirm", "dismiss", "lock"):
                with self.subTest(action=action):
                    with self.assertRaisesRegex(SubjectLifecycleError, "requires a subject"):
                        apply_subject_action(Path(tmpdir), action, source="test")


if __name__ == "__main__":
    unittest.main()
```

- [ ] Run:

```bash
uv run python -m unittest tests.test_subject_lifecycle
```

Expected before implementation:

```text
ModuleNotFoundError: No module named 'qiongli.bridges.subject_lifecycle'
```

### 3.2 Implement lifecycle module

- [ ] Create `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`:

```python
from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .project_manifest import OFFICIAL_SUBJECTS, load_project_manifest, update_project_manifest


TRACE_REL = Path(".qiongli") / "trace"
SUBJECT_EVIDENCE_REL = TRACE_REL / "subject_evidence.json"
ACTIONS = {"confirm", "dismiss", "reset", "lock", "unlock"}


class SubjectLifecycleError(ValueError):
    """Raised when a subject lifecycle action is invalid."""


def _now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def _evidence_path(project_root: Path) -> Path:
    return project_root / SUBJECT_EVIDENCE_REL


def _load_state(project_root: Path) -> dict[str, Any]:
    path = _evidence_path(project_root)
    if not path.exists():
        return {"subjects": {}, "dismissed_subjects": {}, "lifecycle_events": []}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        payload = {}
    if not isinstance(payload, dict):
        payload = {}
    payload.setdefault("subjects", {})
    payload.setdefault("dismissed_subjects", {})
    payload.setdefault("lifecycle_events", [])
    if not isinstance(payload["subjects"], dict):
        payload["subjects"] = {}
    if not isinstance(payload["dismissed_subjects"], dict):
        payload["dismissed_subjects"] = {}
    if not isinstance(payload["lifecycle_events"], list):
        payload["lifecycle_events"] = []
    return payload


def _write_state(project_root: Path, state: dict[str, Any]) -> dict[str, Any]:
    path = _evidence_path(project_root)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return state


def _manifest_dict(project_root: Path) -> dict[str, Any]:
    manifest = load_project_manifest(project_root)
    return {
        "active_subject": manifest.active_subject,
        "subject_mode": manifest.subject_mode,
        "secondary_subjects": manifest.secondary_subjects,
        "venue_profiles": manifest.venue_profiles,
        "method_lenses": manifest.method_lenses,
        "strictness": manifest.strictness,
    }


def _validate_action(action: str) -> None:
    if action not in ACTIONS:
        raise SubjectLifecycleError(f"Unknown subject action: {action}")


def _validate_subject(action: str, subject: str | None) -> str | None:
    if action in {"confirm", "dismiss", "lock"} and not subject:
        raise SubjectLifecycleError(f"Action '{action}' requires a subject")
    if subject and subject not in OFFICIAL_SUBJECTS:
        raise SubjectLifecycleError(f"Unknown subject: {subject}")
    return subject


def _append_event(
    state: dict[str, Any],
    *,
    action: str,
    subject: str | None,
    source: str,
    run_id: str | None,
) -> None:
    state["lifecycle_events"].append(
        {
            "action": action,
            "subject": subject,
            "source": source,
            "run_id": run_id or "",
            "created_at": _now_iso(),
        }
    )


def subject_status(project_root: Path) -> dict[str, Any]:
    project_root = Path(project_root)
    return {
        "project_root": str(project_root),
        "manifest": _manifest_dict(project_root),
        "state": _load_state(project_root),
    }


def apply_subject_action(
    project_root: Path,
    action: str,
    subject: str | None = None,
    *,
    source: str = "user",
    run_id: str | None = None,
) -> dict[str, Any]:
    project_root = Path(project_root)
    _validate_action(action)
    subject = _validate_subject(action, subject)
    state = _load_state(project_root)

    if action == "confirm":
        update_project_manifest(project_root, active_subject=subject, subject_mode="confirmed")
    elif action == "lock":
        update_project_manifest(project_root, active_subject=subject, subject_mode="locked")
    elif action == "unlock":
        manifest = load_project_manifest(project_root)
        next_mode = "confirmed" if manifest.active_subject != "auto" else "auto"
        update_project_manifest(project_root, subject_mode=next_mode)
    elif action == "reset":
        update_project_manifest(
            project_root,
            active_subject="auto",
            subject_mode="auto",
            secondary_subjects=[],
            venue_profiles=[],
            method_lenses=[],
            strictness="standard",
        )
        state["subjects"] = {}
        state["dismissed_subjects"] = {}
    elif action == "dismiss":
        state["dismissed_subjects"][subject] = {
            "source": source,
            "run_id": run_id or "",
            "created_at": _now_iso(),
            "last_suggestion_count": int(
                state.get("subjects", {}).get(subject, {}).get("suggestion_count", 0)
            ),
        }

    _append_event(state, action=action, subject=subject, source=source, run_id=run_id)
    _write_state(project_root, state)
    return subject_status(project_root)
```

### 3.3 Verify lifecycle runtime

- [ ] Run:

```bash
uv run python -m unittest tests.test_subject_lifecycle
```

Expected:

```text
OK
```

- [ ] Commit:

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py tests/test_subject_lifecycle.py
git commit -m "feat(subjects): add subject lifecycle runtime"
```

---

## Task 4: Respect Dismissed Subject Recommendations

**Purpose:** Prevent the system from repeatedly recommending a subject the user dismissed, while allowing the recommendation to return after new evidence accumulates.

### 4.1 Add focused guidance runtime tests

- [ ] Add tests to `tests/test_guidance_runtime.py`:

```python
def test_subject_promotion_recommendation_respects_dismissed_subject(self) -> None:
    memory = {
        "subjects": {
            "finance": {
                "suggestion_count": 3,
                "preview_count": 3,
                "apply_count": 0,
                "last_suggested_at": "2026-07-01T00:00:00+00:00",
                "last_run_id": "run-3",
            }
        },
        "dismissed_subjects": {
            "finance": {
                "last_suggestion_count": 3,
                "created_at": "2026-07-01T00:01:00+00:00",
                "source": "test",
                "run_id": "run-3",
            }
        },
    }
    refinement = {"suggest_subjects": ["finance"]}

    recommendation = guidance_runtime._subject_promotion_recommendation(memory, refinement)

    self.assertEqual(recommendation["status"], "dismissed")
    self.assertEqual(recommendation["subject"], "finance")


def test_subject_promotion_recommendation_reopens_after_new_evidence(self) -> None:
    memory = {
        "subjects": {
            "finance": {
                "suggestion_count": 4,
                "preview_count": 4,
                "apply_count": 0,
                "last_suggested_at": "2026-07-01T00:02:00+00:00",
                "last_run_id": "run-4",
            }
        },
        "dismissed_subjects": {
            "finance": {
                "last_suggestion_count": 3,
                "created_at": "2026-07-01T00:01:00+00:00",
                "source": "test",
                "run_id": "run-3",
            }
        },
    }
    refinement = {"suggest_subjects": ["finance"]}

    recommendation = guidance_runtime._subject_promotion_recommendation(memory, refinement)

    self.assertEqual(recommendation["status"], "recommend_confirm")
    self.assertEqual(recommendation["subject"], "finance")
```

If `tests/test_guidance_runtime.py` imports individual helpers instead of the module object, adapt the import once at the top:

```python
from qiongli.bridges import guidance_runtime
```

- [ ] Run:

```bash
uv run python -m unittest tests.test_guidance_runtime
```

Expected before implementation:

```text
FAIL: test_subject_promotion_recommendation_respects_dismissed_subject
```

### 4.2 Update recommendation suppression logic

- [ ] In `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`, update `_subject_promotion_recommendation(memory, subject_refinement)` so it checks dismissal state before returning a new confirmation recommendation:

```python
def _dismissed_subject_record(memory: Mapping[str, Any], subject: str) -> Mapping[str, Any] | None:
    dismissed_subjects = memory.get("dismissed_subjects", {})
    if not isinstance(dismissed_subjects, Mapping):
        return None
    record = dismissed_subjects.get(subject)
    return record if isinstance(record, Mapping) else None
```

Inside the loop that selects a promoted subject:

```python
dismissed = _dismissed_subject_record(memory, subject)
if dismissed is not None:
    last_suggestion_count = _safe_int(dismissed.get("last_suggestion_count"), default=0)
    if suggestion_count <= last_suggestion_count:
        return {
            "status": "dismissed",
            "subject": subject,
            "suggestion_count": suggestion_count,
            "dismissed_at": dismissed.get("created_at", ""),
            "source": dismissed.get("source", ""),
        }
```

The existing positive recommendation remains unchanged when `suggestion_count > last_suggestion_count`.

### 4.3 Verify dismissal behavior

- [ ] Run:

```bash
uv run python -m unittest tests.test_guidance_runtime
```

Expected:

```text
OK
```

- [ ] Run the lifecycle tests again to ensure memory shape remains compatible:

```bash
uv run python -m unittest tests.test_subject_lifecycle
```

Expected:

```text
OK
```

- [ ] Commit:

```bash
git add packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py tests/test_guidance_runtime.py
git commit -m "feat(guidance): respect dismissed subject recommendations"
```

---

## Task 5: Expose Lifecycle Controls Through MCP

**Purpose:** Let clients and marketplace-style integrations use the same adaptive subject lifecycle without relying on CLI-only flows.

### 5.1 Write MCP handler tests first

- [ ] Add tests to `tests/test_mcp_tool_handlers.py`:

```python
def test_subject_status_tool_reports_manifest(tmp_path: Path) -> None:
    result = call_qiongli_tool("qiongli_subject_status", {"cwd": str(tmp_path)})

    self.assertEqual(result["manifest"]["active_subject"], "auto")
    self.assertEqual(result["manifest"]["subject_mode"], "auto")
    self.assertIn("state", result)


def test_subject_update_tool_can_confirm_and_dismiss(tmp_path: Path) -> None:
    confirmed = call_qiongli_tool(
        "qiongli_subject_update",
        {"cwd": str(tmp_path), "action": "confirm", "subject": "finance"},
    )
    dismissed = call_qiongli_tool(
        "qiongli_subject_update",
        {"cwd": str(tmp_path), "action": "dismiss", "subject": "economics"},
    )

    self.assertEqual(confirmed["manifest"]["active_subject"], "finance")
    self.assertEqual(confirmed["manifest"]["subject_mode"], "confirmed")
    self.assertEqual(
        dismissed["state"]["dismissed_subjects"]["economics"]["source"],
        "mcp",
    )


def test_subject_update_tool_rejects_invalid_action(tmp_path: Path) -> None:
    with self.assertRaises(ValueError):
        call_qiongli_tool("qiongli_subject_update", {"cwd": str(tmp_path), "action": "merge"})
```

If the file uses `unittest.TestCase`, place these methods inside the existing test class and replace `tmp_path` with `tempfile.TemporaryDirectory()` as the local pattern requires.

- [ ] Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers
```

Expected before implementation:

```text
ValueError: Unknown MCP tool: qiongli_subject_status
```

### 5.2 Add MCP schemas and dispatch

- [ ] In `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`, import lifecycle helpers:

```python
from .subject_lifecycle import apply_subject_action, subject_status
```

- [ ] Add this tool definition to `MCP_TOOL_DEFINITIONS`:

```python
{
    "name": "qiongli_subject_status",
    "description": "Inspect adaptive subject state and local guidance manifest for a project.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "cwd": {
                "type": "string",
                "description": "Project directory. Defaults to the current working directory.",
            }
        },
        "additionalProperties": False,
    },
}
```

- [ ] Add this second tool definition:

```python
{
    "name": "qiongli_subject_update",
    "description": "Confirm, dismiss, reset, lock, or unlock adaptive subject guidance for a project.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "cwd": {
                "type": "string",
                "description": "Project directory. Defaults to the current working directory.",
            },
            "action": {
                "type": "string",
                "enum": ["confirm", "dismiss", "reset", "lock", "unlock"],
            },
            "subject": {
                "type": "string",
                "enum": ["core", "economics", "finance"],
                "description": "Required for confirm, dismiss, and lock. Omit for reset and unlock.",
            },
            "run_id": {
                "type": "string",
                "description": "Optional run identifier to link the lifecycle event to a guidance run.",
            },
        },
        "required": ["action"],
        "additionalProperties": False,
    },
}
```

Use the actual official subjects present in `OFFICIAL_SUBJECTS`. If `core` is not accepted as a lifecycle subject, omit it from the enum.

- [ ] Add handlers:

```python
def _tool_subject_status(args: Mapping[str, Any]) -> dict[str, Any]:
    return subject_status(_cwd_from_args(args))


def _tool_subject_update(args: Mapping[str, Any]) -> dict[str, Any]:
    action = str(args.get("action", ""))
    subject = args.get("subject")
    run_id = args.get("run_id")
    return apply_subject_action(
        _cwd_from_args(args),
        action,
        str(subject) if subject else None,
        source="mcp",
        run_id=str(run_id) if run_id else None,
    )
```

- [ ] Register the handlers in `call_qiongli_tool(...)` dispatch.

### 5.3 Verify MCP controls

- [ ] Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers tests.test_subject_lifecycle
```

Expected:

```text
OK
```

- [ ] Commit:

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "feat(mcp): expose subject lifecycle tools"
```

---

## Task 6: Add Top-Level CLI Subject Commands

**Purpose:** Let local users inspect and control subject state after full installation without choosing a subject at install time.

### 6.1 Write CLI tests first

- [ ] Add tests to `tests/test_cli.py` following the file's existing parser/command style:

```python
def test_subject_status_command_prints_json(self) -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            exit_code = cli.main(["subject", "status", "--cwd", tmpdir, "--json"])

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["manifest"]["active_subject"], "auto")
        self.assertEqual(payload["manifest"]["subject_mode"], "auto")


def test_subject_confirm_command_updates_manifest(self) -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            exit_code = cli.main(["subject", "confirm", "finance", "--cwd", tmpdir, "--json"])

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["manifest"]["active_subject"], "finance")
        self.assertEqual(payload["manifest"]["subject_mode"], "confirmed")


def test_subject_reset_command_returns_to_auto(self) -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        cli.main(["subject", "confirm", "finance", "--cwd", tmpdir, "--json"])
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            exit_code = cli.main(["subject", "reset", "--cwd", tmpdir, "--json"])

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["manifest"]["active_subject"], "auto")
        self.assertEqual(payload["manifest"]["subject_mode"], "auto")
```

Add imports only if absent:

```python
import contextlib
import io
import json
import tempfile
```

- [ ] Run:

```bash
uv run python -m unittest tests.test_cli
```

Expected before implementation:

```text
invalid choice: 'subject'
```

### 6.2 Implement CLI command group

- [ ] In `packages/python-qiongli/src/qiongli/cli.py`, add helper functions near other command handlers:

```python
def _print_subject_result(payload: dict[str, object], as_json: bool) -> None:
    if as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return
    manifest = payload["manifest"]
    state = payload["state"]
    print(f"active_subject: {manifest['active_subject']}")
    print(f"subject_mode: {manifest['subject_mode']}")
    dismissed = state.get("dismissed_subjects", {})
    if dismissed:
        print("dismissed_subjects: " + ", ".join(sorted(dismissed)))


def cmd_subject(args: argparse.Namespace) -> int:
    from qiongli.bridges.subject_lifecycle import apply_subject_action, subject_status

    project_root = Path(args.cwd).expanduser().resolve()
    try:
        if args.subject_command == "status":
            payload = subject_status(project_root)
        else:
            payload = apply_subject_action(
                project_root,
                args.subject_command,
                getattr(args, "subject", None),
                source="cli",
            )
    except ValueError as exc:
        print(f"qiongli subject: {exc}", file=sys.stderr)
        return 2

    _print_subject_result(payload, args.json)
    return 0
```

If `json`, `Path`, or `sys` are already imported, reuse the existing imports.

- [ ] Add parser setup in the same area as other top-level subcommands:

```python
subject_parser = subparsers.add_parser(
    "subject",
    help="Inspect and control adaptive subject guidance.",
)
subject_subparsers = subject_parser.add_subparsers(dest="subject_command", required=True)

subject_status_parser = subject_subparsers.add_parser("status", help="Show subject state.")
subject_status_parser.add_argument("--cwd", default=".", help="Project directory.")
subject_status_parser.add_argument("--json", action="store_true", help="Print structured JSON.")
subject_status_parser.set_defaults(func=cmd_subject)

for action in ("confirm", "dismiss", "lock"):
    action_parser = subject_subparsers.add_parser(action, help=f"{action.title()} a subject.")
    action_parser.add_argument("subject", choices=["economics", "finance"])
    action_parser.add_argument("--cwd", default=".", help="Project directory.")
    action_parser.add_argument("--json", action="store_true", help="Print structured JSON.")
    action_parser.set_defaults(func=cmd_subject)

for action in ("reset", "unlock"):
    action_parser = subject_subparsers.add_parser(action, help=f"{action.title()} subject state.")
    action_parser.add_argument("--cwd", default=".", help="Project directory.")
    action_parser.add_argument("--json", action="store_true", help="Print structured JSON.")
    action_parser.set_defaults(func=cmd_subject)
```

The user-facing examples this must support:

```bash
qiongli subject status --cwd /path/to/project
qiongli subject confirm finance --cwd /path/to/project
qiongli subject dismiss finance --cwd /path/to/project
qiongli subject reset --cwd /path/to/project
qiongli subject lock economics --cwd /path/to/project
qiongli subject unlock --cwd /path/to/project
```

### 6.3 Verify CLI controls

- [ ] Run:

```bash
uv run python -m unittest tests.test_cli tests.test_subject_lifecycle
```

Expected:

```text
OK
```

- [ ] Run manual CLI smoke:

```bash
tmpdir="$(mktemp -d)"
uv run qiongli subject status --cwd "$tmpdir" --json
uv run qiongli subject confirm finance --cwd "$tmpdir" --json
uv run qiongli subject dismiss economics --cwd "$tmpdir" --json
uv run qiongli subject reset --cwd "$tmpdir" --json
```

Expected:

- First command prints `active_subject` as `auto`.
- Second command prints `active_subject` as `finance` and `subject_mode` as `confirmed`.
- Third command includes `dismissed_subjects.economics`.
- Fourth command prints `active_subject` as `auto` and `subject_mode` as `auto`.

- [ ] Commit:

```bash
git add packages/python-qiongli/src/qiongli/cli.py tests/test_cli.py
git commit -m "feat(cli): add subject lifecycle commands"
```

---

## Task 7: End-to-End Verification

**Purpose:** Confirm the feature works through tests, evaluation, and the existing local runtime smoke layer.

### 7.1 Run focused Python tests

- [ ] Run:

```bash
uv run python -m unittest \
  tests.test_subject_lifecycle \
  tests.test_subject_router_eval \
  tests.test_guidance_runtime \
  tests.test_mcp_tool_handlers \
  tests.test_cli
```

Expected:

```text
OK
```

### 7.2 Run subject router evaluation

- [ ] Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected:

```json
{
  "case_count": 8,
  "threshold_failures": []
}
```

Metrics must satisfy:

- `decision_accuracy >= 0.90`
- `primary_subject_accuracy >= 0.90`
- `suggest_subject_precision >= 0.85`
- `near_miss_false_positives == 0`

### 7.3 Run local subject runtime smoke

- [ ] Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py
```

Expected:

```text
Subject runtime smoke passed
```

If the script uses a different exact success line, record the emitted success line in the final implementation report.

### 7.4 Run broader test suite

- [ ] Run:

```bash
uv run python -m unittest discover tests
```

Expected:

```text
OK
```

### 7.5 Run repository validation commands

- [ ] Run:

```bash
git diff --check
```

Expected: no output and exit code `0`.

- [ ] If the repository has Node/package validation, run the existing command shown in `package.json`. Prefer an already documented validation command. Examples:

```bash
pnpm test
```

or:

```bash
pnpm run validate
```

Expected: command exits with code `0`.

Do not add a new package manager or lockfile for this feature.

---

## Task 8: Final Review, Squash Option, and Merge Preparation

**Purpose:** Make the implementation easy to review and safe to merge into `dev`.

### 8.1 Inspect final changes

- [ ] Run:

```bash
git status --short
git log --oneline --decorate --max-count=8
git diff dev...HEAD --stat
```

Expected:

- Working tree is clean.
- Feature commits are grouped by fixture, eval runner, lifecycle runtime, guidance suppression, MCP, and CLI.
- Diff only touches files listed in this plan, unless a test revealed a necessary adjacent file.

### 8.2 Self-review checklist

- [ ] Confirm `subject_lifecycle.py` is the only new state-transition owner.
- [ ] Confirm CLI and MCP both call `subject_lifecycle.py`, rather than implementing separate state logic.
- [ ] Confirm `dismiss` does not create `.qiongli/guidance_manifest.yaml`.
- [ ] Confirm `confirm` and `lock` create or update `.qiongli/guidance_manifest.yaml`.
- [ ] Confirm `reset` returns manifest state to adaptive core.
- [ ] Confirm evaluation fixtures are deterministic and do not call network services.
- [ ] Confirm `evaluate_subject_router.py` exits with code `1` when thresholds fail.
- [ ] Confirm existing preview-first guidance behavior remains intact.

### 8.3 Prepare final branch history

Keep commits separated while implementation is active. After all checks pass, use one of these two histories:

- Preferred for review:

```text
test(subjects): add router evaluation fixtures
test(subjects): add subject router evaluation runner
feat(subjects): add subject lifecycle runtime
feat(guidance): respect dismissed subject recommendations
feat(mcp): expose subject lifecycle tools
feat(cli): add subject lifecycle commands
```

- Preferred before direct merge if the user asks to squash:

```text
feat(subjects): add adaptive subject lifecycle controls
```

Squashed commit body:

```text
Add adaptive subject lifecycle controls for full-install Qiongli usage.

Introduce a curated subject router evaluation runner, local subject
state transitions, dismissal-aware promotion recommendations, MCP tools,
and top-level CLI commands for inspecting and controlling subject state.
```

### 8.4 Merge back to dev

Only after verification passes:

```bash
cd /Users/pengjiaxin/Work/utility/cli-tools/research-skills
git switch dev
git merge --no-ff feature/adaptive-subject-lifecycle-eval
```

Expected:

```text
Merge made by the 'ort' strategy.
```

If the user requests a squash merge instead:

```bash
cd /Users/pengjiaxin/Work/utility/cli-tools/research-skills
git switch dev
git merge --squash feature/adaptive-subject-lifecycle-eval
git commit
```

Use the squashed commit message from section 8.3.

---

## Acceptance Criteria

- [ ] `uv run python tooling/scripts/evaluate_subject_router.py --json` reports 8 cases and no threshold failures.
- [ ] Evaluation runner exits nonzero when any configured threshold fails.
- [ ] `qiongli subject status --cwd <project>` reports manifest and lifecycle state.
- [ ] `qiongli subject confirm finance --cwd <project>` writes `.qiongli/guidance_manifest.yaml` with `active_subject: finance` and `subject_mode: confirmed`.
- [ ] `qiongli subject dismiss finance --cwd <project>` writes `.qiongli/trace/subject_evidence.json` and does not create `.qiongli/guidance_manifest.yaml`.
- [ ] `qiongli subject reset --cwd <project>` restores adaptive core state and clears dismissal/promotion memory.
- [ ] `qiongli subject lock economics --cwd <project>` writes locked economics state.
- [ ] `qiongli subject unlock --cwd <project>` changes locked state to confirmed when a concrete subject is active.
- [ ] `qiongli_subject_status` and `qiongli_subject_update` MCP tools expose the same behavior as the CLI.
- [ ] Dismissed subject recommendations remain suppressed until new evidence increases the subject's suggestion count.
- [ ] Existing guidance preview behavior stays preview-first and does not run local agents.
- [ ] `uv run python -m unittest discover tests` passes.
- [ ] `uv run python tooling/scripts/run_subject_runtime_smoke.py` passes.

## Non-Goals

- Do not add a subject-selection step to install.
- Do not make economics or finance mandatory during full install.
- Do not run local agents from preview or subject status commands.
- Do not add new external dependencies.
- Do not broaden subject packs beyond economics and finance in this feature.
- Do not change marketplace packaging semantics in this feature; MCP and CLI controls are the cross-platform surface for clients.
