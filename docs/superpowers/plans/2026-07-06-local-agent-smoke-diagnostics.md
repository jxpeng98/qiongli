# Local Agent Smoke Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Strengthen Stage 3 local-agent smoke reports so maintainer failures include machine-readable runtime request, routing, path, and rerun diagnostics.

**Architecture:** Keep preview smoke unchanged and keep real local-agent execution opt-in. Extend `tooling/scripts/run_subject_runtime_smoke.py` reporting helpers to derive diagnostics from existing mocked or real `qiongli_task_run` payloads: task packet controller metadata, runtime plan, routing notes, local guidance trace paths, and write-boundary checked paths.

**Tech Stack:** Python stdlib, existing `unittest` smoke tests, existing MCP handler smoke fixtures.

---

## Files

- Modify: `tooling/scripts/run_subject_runtime_smoke.py`
  - Add local-agent requested runtime metadata.
  - Add routing/runtime notes to `local_agent`.
  - Add checked trace/Qiongli-visible paths to `write_boundary`.
  - Add failed-case `diagnostics` with case name, workspace root, project root, rerun command, and trace paths when available.
- Modify: `tests/test_subject_runtime_smoke.py`
  - Add failing tests for requested runtime metadata and runtime notes.
  - Add failing tests for checked write-boundary paths.
  - Add failing tests for failed-case diagnostics.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that Stage 3 local-agent smoke now has stronger machine-readable diagnostics while still remaining opt-in.

## Task 1: Add Failing Tests

- [x] **Step 1: Add requested runtime and routing notes test**

Add a test in `tests/test_subject_runtime_smoke.py` that fakes a successful
local-agent payload with `controller_metadata`, `runtime_plan`, and
`routing_notes`:

```python
    def test_local_agent_report_includes_requested_runtime_and_runtime_notes(self) -> None:
        case = next(
            item
            for item in load_smoke_cases(FIXTURE_DIR)
            if item.name == "confirmed_finance_guidance_loaded"
        )

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {
                "structuredContent": {
                    "mode": "task-run",
                    "run_agents": True,
                    "data": {
                        "task_packet": {
                            "controller_metadata": {
                                "controller": "codex",
                                "primary_agent": "codex",
                                "review_agent": "claude",
                            },
                            "local_guidance": {
                                "guidance_files_read": [
                                    ".qiongli/guidance.d/subject-runtime.md"
                                ]
                            },
                            "subject_refinement": {
                                "decision": "confirm_subject",
                                "primary_subject": "finance",
                                "loaded_resources": {
                                    "levels": ["subject_overlay", "subject_skill"]
                                },
                                "signals": [],
                                "resource_activation_plan": {},
                            },
                            "runtime_plan": {
                                "primary_agent": "codex",
                                "review_agent": "claude",
                                "fallback_agent": "antigravity",
                            },
                            "domain": "finance",
                        },
                        "local_guidance_trace": {
                            "run_dir": ".qiongli/trace/runs/run-1",
                            "trace_index": ".qiongli/trace/index.jsonl",
                            "guidance_files_read": [
                                ".qiongli/guidance.d/subject-runtime.md"
                            ],
                        },
                        "routing_notes": [
                            "Runtime plan: draft=codex, review=claude, fallback=antigravity.",
                            "Runtime preflight: claude unavailable; fallback retained.",
                            "Unrelated note.",
                        ],
                    },
                },
                "isError": False,
            }

        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "passed", result["failures"])
        self.assertEqual(
            result["local_agent"]["requested_runtime"],
            {"controller": "codex", "primary_agent": "codex", "review_agent": "claude"},
        )
        self.assertEqual(
            result["local_agent"]["runtime_plan"],
            {"primary_agent": "codex", "review_agent": "claude", "fallback_agent": "antigravity"},
        )
        self.assertIn("Runtime preflight: claude unavailable; fallback retained.", result["local_agent"]["runtime_notes"])
```

- [x] **Step 2: Add checked write-boundary paths test**

Add:

```python
    def test_write_boundary_reports_checked_qiongli_visible_paths(self) -> None:
        project_root = Path("/tmp/project").resolve()
        payload = {
            "data": {
                "local_guidance_trace": {
                    "run_dir": ".qiongli/trace/runs/run-1",
                    "trace_index": ".qiongli/trace/index.jsonl",
                    "guidance_proposal": ".qiongli/trace/runs/run-1/guidance-proposal.json",
                }
            }
        }

        result = smoke._write_boundary_report(payload, project_root)

        self.assertTrue(result["known_paths_inside_project"])
        self.assertIn(str(project_root / ".qiongli/trace/runs/run-1"), result["checked_paths"])
        self.assertIn(str(project_root / ".qiongli/trace/index.jsonl"), result["checked_paths"])
```

- [x] **Step 3: Add failed-case diagnostics test**

Add:

```python
    def test_local_agent_failure_diagnostics_include_roots_rerun_and_trace_paths(self) -> None:
        case = next(
            item
            for item in load_smoke_cases(FIXTURE_DIR)
            if item.name == "confirmed_finance_guidance_loaded"
        )

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {
                "structuredContent": {
                    "mode": "task-run",
                    "run_agents": True,
                    "data": {
                        "task_packet": {"domain": "finance"},
                        "local_guidance_trace": {
                            "run_dir": "/tmp/outside/run-1",
                            "trace_index": ".qiongli/trace/index.jsonl",
                        },
                    },
                },
                "isError": False,
            }

        with tempfile.TemporaryDirectory() as tmp_dir:
            workspace = Path(tmp_dir)
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, workspace, "local-agent")

        diagnostics = result["diagnostics"]
        self.assertEqual(diagnostics["case_name"], "confirmed_finance_guidance_loaded")
        self.assertEqual(diagnostics["workspace_root"], str(workspace.resolve()))
        self.assertEqual(diagnostics["project_root"], result["project_root"])
        self.assertIn("--case confirmed_finance_guidance_loaded", diagnostics["rerun_command"])
        self.assertEqual(diagnostics["trace_paths"]["run_dir"], "/tmp/outside/run-1")
```

- [x] **Step 4: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_local_agent_report_includes_requested_runtime_and_runtime_notes tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_write_boundary_reports_checked_qiongli_visible_paths tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_local_agent_failure_diagnostics_include_roots_rerun_and_trace_paths -q
```

Expected: FAIL because these report fields do not exist yet.

## Task 2: Implement Report Diagnostics

- [x] **Step 1: Add helper functions**

In `tooling/scripts/run_subject_runtime_smoke.py`, add:

```python
def _payload_string(value: Any) -> str:
    return value if isinstance(value, str) else ""


def _routing_notes_from_payload(payload: dict[str, Any]) -> list[str]:
    notes = _payload_data(payload).get("routing_notes", [])
    return [str(item) for item in notes] if isinstance(notes, list) else []


def _runtime_notes(notes: list[str]) -> list[str]:
    keywords = ("runtime", "preflight", "fallback", "agent")
    return [note for note in notes if any(keyword in note.lower() for keyword in keywords)]
```

- [x] **Step 2: Extend `_local_agent_metadata`**

Update `_local_agent_metadata(payload)` to include:

```python
controller_metadata = _payload_object(packet.get("controller_metadata", {}))
routing_notes = _routing_notes_from_payload(payload)
"requested_runtime": {
    "controller": _payload_string(packet.get("controller") or controller_metadata.get("controller")),
    "primary_agent": _payload_string(packet.get("primary_agent") or controller_metadata.get("primary_agent")),
    "review_agent": _payload_string(packet.get("review_agent") or controller_metadata.get("review_agent")),
},
"routing_notes": routing_notes,
"runtime_notes": _runtime_notes(routing_notes),
```

- [x] **Step 3: Add checked paths to `_write_boundary_report`**

Track every resolved known path and trace path in `checked_paths`, and return:

```python
"checked_paths": sorted(set(checked_paths)),
```

- [x] **Step 4: Add failed-case diagnostics**

Add:

```python
def _trace_paths(payload: dict[str, Any]) -> dict[str, str]:
    trace = _local_guidance_trace_from_payload(payload)
    return {key: value for key, value in trace.items() if key in {...} and isinstance(value, str) and value}


def _failure_diagnostics(...):
    return {...}
```

Call it in `run_smoke_case(...)` when `mode == "local-agent"` and the case is
not passed.

- [x] **Step 5: Run GREEN**

Run the focused tests from Task 1. Expected: PASS.

## Task 3: Verify And Document

- [x] **Step 1: Run full smoke test module**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_runtime_smoke -q
```

Expected: PASS.

- [x] **Step 2: Run preview smoke**

Run:

```bash
.venv/bin/python tooling/scripts/run_subject_runtime_smoke.py --json
```

Expected: JSON report exits 0 and summary failed is 0.

- [x] **Step 3: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [x] **Step 4: Update roadmap**

Update Stage 3 status to say local-agent smoke has strengthened machine-readable
diagnostics for runtime requests, routing notes, checked Qiongli-visible paths,
and failed-case rerun context while remaining opt-in.

- [x] **Step 5: Commit by content**

Implementation:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/test_subject_runtime_smoke.py
git commit -m "feat(smoke): strengthen local agent diagnostics"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-local-agent-smoke-diagnostics.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record local agent smoke diagnostics"
```

## Self-Review

- Spec coverage: Covers Stage 3 diagnostics and write-boundary report gaps without launching real agents.
- Placeholder scan: No TBD/TODO placeholders remain.
- Type consistency: New report fields are nested under existing `local_agent`, `write_boundary`, and failed-case `diagnostics` objects.
