# Real Local-Agent Smoke Runtime Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a release-grade, opt-in local-agent smoke path that proves materialized subject guidance is consumed by real task runs while keeping preview smoke as the safe default.

**Architecture:** Harden the existing `tooling/scripts/run_subject_runtime_smoke.py` runner instead of creating a parallel harness. Extend `qiongli_task_run` MCP argument mapping only where the smoke runner needs bounded real-run controls, then add report metadata, trace assertions, write-boundary checks, and release-readiness documentation around the existing real orchestrator path.

**Tech Stack:** Python 3.11+, stdlib `unittest`, `tempfile`, `pathlib`, existing Qiongli MCP handler/orchestrator bridge, existing `uv run python -m unittest`, JSON smoke reports.

---

## Source Spec

Implement:

- `docs/superpowers/specs/2026-07-02-real-local-agent-smoke-runtime-hardening-design.md`

Do not add new subjects. Do not make local-agent smoke part of the default test
or release gate. Keep preview mode safe and unchanged by default.

## File Map

- Modify `tooling/scripts/run_subject_runtime_smoke.py`
  - Owns smoke case selection, isolated environment setup, task-run argument
    construction, local-agent guardrails, local-agent assertions, write-boundary
    checks, rerun diagnostics, and JSON report shape.
- Modify `tests/test_subject_runtime_smoke.py`
  - Adds focused unit tests for local-agent default case selection, bounded
    task args, trace assertions, write-boundary failures, and error reports.
- Modify `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Extends `qiongli_task_run` input schema and `_task_run_kwargs()` to pass
    bounded real-run options through to `ModelOrchestrator.task_run()`.
- Modify `tests/test_mcp_tool_handlers.py`
  - Adds coverage that `max_revision_rounds`, `output_budget`, and
    `skip_validation` are accepted by MCP and passed to the orchestrator.
- Modify `docs/advanced/publish-pypi.md`
  - Documents the optional subject local-agent smoke command for maintainers
    and release candidates.
- Modify `docs/zh/advanced/publish-pypi.md`
  - Chinese documentation parity for the same maintainer-only smoke command.

## Task 0: Prepare Worktree And Baseline

**Files:**
- No source files changed.

- [ ] **Step 1: Confirm current branch and cleanliness**

Run from repository root:

```bash
git status --short --branch
```

Expected: current branch is `dev` and no uncommitted source changes are present.

- [ ] **Step 2: Confirm `.worktrees` is ignored**

Run:

```bash
git check-ignore -q .worktrees
```

Expected: exit code `0`.

- [ ] **Step 3: Create an isolated implementation worktree**

Run:

```bash
git worktree add .worktrees/real-local-agent-smoke-hardening -b feature/real-local-agent-smoke-hardening dev
```

Expected: output includes `Preparing worktree`.

- [ ] **Step 4: Run baseline focused tests in the worktree**

Run from `.worktrees/real-local-agent-smoke-hardening`:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke tests.test_mcp_tool_handlers
```

Expected: all selected tests pass before implementation.

- [ ] **Step 5: Run baseline subject smoke commands**

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: preview smoke exits `0` with `summary.failed == 0`; router eval exits
`0` with no threshold failures.

## Task 1: Smoke Runner Selection And Schema Helpers

**Files:**
- Modify: `tooling/scripts/run_subject_runtime_smoke.py`
- Modify: `tests/test_subject_runtime_smoke.py`

- [ ] **Step 1: Write failing tests for schema version and local-agent default case selection**

Add imports in `tests/test_subject_runtime_smoke.py`:

```python
from tooling.scripts import run_subject_runtime_smoke as smoke
```

Add these tests to `SubjectRuntimeSmokeTests`:

```python
    def test_report_schema_version_is_1_1(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["suggest_finance_subject"],
            )

        self.assertEqual(report["schema_version"], "1.1")

    def test_local_agent_mode_defaults_to_confirmed_finance_case(self) -> None:
        fake_result = {
            "name": "confirmed_finance_guidance_loaded",
            "source": "tests/fixtures/subject_runtime_smoke/confirmed_finance_guidance_loaded.json",
            "project_root": "/tmp/project",
            "status": "passed",
            "failures": [],
            "environment": {},
            "result": {"run_agents": True, "data": {}},
            "local_agent": {"requested": True, "env_opt_in": True},
            "trace_assertions": {},
            "write_boundary": {"known_paths_inside_project": True, "violations": []},
        }
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.dict("os.environ", {smoke.LOCAL_AGENT_ENV: "1"}):
                with mock.patch.object(smoke, "run_smoke_case", return_value=fake_result) as run_case:
                    report = run_smoke_suite(
                        fixture_dir=FIXTURE_DIR,
                        workspace_root=Path(tmp_dir),
                        mode="local-agent",
                        selected_cases=[],
                    )

        self.assertEqual(report["schema_version"], "1.1")
        self.assertEqual(report["summary"], {"total": 1, "passed": 1, "failed": 0})
        self.assertEqual(run_case.call_count, 1)
        selected_case = run_case.call_args.args[0]
        self.assertEqual(selected_case.name, "confirmed_finance_guidance_loaded")
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: failures mention `schema_version` still being `1.0` and local-agent
mode selecting all cases when no `--case` is provided.

- [ ] **Step 3: Add constants and a case selection helper**

In `tooling/scripts/run_subject_runtime_smoke.py`, replace the hard-coded schema
version with constants near the existing `LOCAL_AGENT_ENV` constant:

```python
REPORT_SCHEMA_VERSION = "1.1"
LOCAL_AGENT_DEFAULT_CASES = ("confirmed_finance_guidance_loaded",)
```

Add this helper below `load_smoke_cases()`:

```python
def _select_smoke_cases(
    cases: list[SmokeCase],
    *,
    mode: str,
    selected_cases: list[str] | None,
) -> list[SmokeCase]:
    selected = set(selected_cases or [])
    if mode == "local-agent" and not selected:
        selected = set(LOCAL_AGENT_DEFAULT_CASES)
    if selected:
        filtered = [case for case in cases if case.name in selected]
        found = {case.name for case in filtered}
        missing = sorted(selected - found)
        if missing:
            raise ValueError("unknown smoke case(s): " + ", ".join(missing))
        return filtered
    return list(cases)
```

In `run_smoke_suite()`, replace the inline selection block with:

```python
    cases = _select_smoke_cases(
        load_smoke_cases(fixture_dir),
        mode=mode,
        selected_cases=selected_cases,
    )
```

Set report schema version via the constant:

```python
        "schema_version": REPORT_SCHEMA_VERSION,
```

Update `_error_report()` to also use `REPORT_SCHEMA_VERSION`.

- [ ] **Step 4: Run tests to verify Task 1 passes**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: `test_report_schema_version_is_1_1` and
`test_local_agent_mode_defaults_to_confirmed_finance_case` pass, with no
regression in existing smoke tests.

- [ ] **Step 5: Commit Task 1**

Run:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/test_subject_runtime_smoke.py
git commit -m "test(smoke): default local-agent subject smoke case"
```

## Task 2: Pass Bounded Task-Run Options Through MCP

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Write failing MCP passthrough test**

Add this test near `test_task_run_tool_can_launch_agents_when_explicitly_enabled`
in `tests/test_mcp_tool_handlers.py`:

```python
    def test_task_run_tool_passes_bounded_runtime_options(self) -> None:
        class StubResult:
            mode = "task-run"
            confidence = 0.95
            merged_analysis = "run ok"
            recommendations: list[str] = []
            data = {"runtime_plan": {"draft": "codex", "review": "codex"}}

        class StubOrchestrator:
            def task_run(self, **kwargs: object) -> StubResult:
                self.kwargs = kwargs
                return StubResult()

        stub = StubOrchestrator()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=stub):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {
                        "cwd": str(root),
                        "task_id": "C1",
                        "paper_type": "empirical",
                        "topic": "smoke topic",
                        "run_agents": True,
                        "max_revision_rounds": 0,
                        "output_budget": 1,
                        "skip_validation": True,
                    },
                )

        self.assertFalse(result["isError"])
        self.assertEqual(stub.kwargs["max_revision_rounds"], 0)
        self.assertEqual(stub.kwargs["output_budget"], 1)
        self.assertIs(stub.kwargs["skip_validation"], True)
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_task_run_tool_passes_bounded_runtime_options
```

Expected: failure because the MCP schema or `_task_run_kwargs()` does not accept
or pass the new fields.

- [ ] **Step 3: Extend the `qiongli_task_run` schema**

In `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`, add these
properties to the `qiongli_task_run` input schema:

```python
                "max_revision_rounds": {"type": "integer", "minimum": 0},
                "output_budget": {"type": "integer", "minimum": 1},
                "skip_validation": {"type": "boolean"},
```

- [ ] **Step 4: Add integer argument parsing helpers**

Near `_optional_bool()` in `mcp_tool_handlers.py`, add:

```python
def _optional_int(
    args: dict[str, Any],
    key: str,
    default: int | None = None,
    *,
    minimum: int | None = None,
) -> int | None:
    if key not in args or args[key] is None:
        return default
    raw = args[key]
    if isinstance(raw, bool) or not isinstance(raw, int):
        raise ValueError(f"{key} must be an integer")
    if minimum is not None and raw < minimum:
        raise ValueError(f"{key} must be >= {minimum}")
    return raw
```

If the file already has an equivalent integer helper, reuse it and do not add a
duplicate.

- [ ] **Step 5: Pass bounded options through `_task_run_kwargs()`**

In `_task_run_kwargs()`, add these fields to the returned dict:

```python
        "max_revision_rounds": _optional_int(args, "max_revision_rounds", 2, minimum=0),
        "output_budget": _optional_int(args, "output_budget", None, minimum=1),
        "skip_validation": _optional_bool(args, "skip_validation", default=False),
```

Do not pass these only in local-agent mode. The handler should map valid
arguments consistently; preview mode will ignore them because it calls
`task_plan()`.

- [ ] **Step 6: Run MCP tests**

Run:

```bash
uv run python -m unittest tests.test_mcp_tool_handlers
```

Expected: all MCP handler tests pass, including the new bounded runtime options
test.

- [ ] **Step 7: Commit Task 2**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "feat(mcp): pass bounded task-run options"
```

## Task 3: Bounded Local-Agent Task Arguments

**Files:**
- Modify: `tooling/scripts/run_subject_runtime_smoke.py`
- Modify: `tests/test_subject_runtime_smoke.py`

- [ ] **Step 1: Write failing tests for local-agent task arguments**

Add this test to `tests/test_subject_runtime_smoke.py`:

```python
    def test_local_agent_case_uses_bounded_task_arguments(self) -> None:
        captured_calls: list[tuple[str, dict[str, object]]] = []

        def fake_call(name: str, args: dict[str, object]) -> dict[str, object]:
            captured_calls.append((name, dict(args)))
            if name == "qiongli_subject_update":
                return {"structuredContent": {"ok": True}, "isError": False}
            return {
                "structuredContent": {
                    "mode": "task-run",
                    "run_agents": True,
                    "data": {
                        "task_packet": {
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
                                "review_agent": "codex",
                                "fallback_agent": "codex",
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
                        "routing_notes": ["Runtime plan: draft=codex, review=codex."],
                    },
                },
                "isError": False,
            }

        case = SmokeCase(
            name="confirmed_finance_guidance_loaded",
            manifest=None,
            args={
                "task_id": "C1",
                "paper_type": "empirical",
                "topic": "earnings announcement stock market reaction",
                "context": "Use event-study evidence and Journal of Finance standards.",
                "domain": "auto",
                "guidance_mode": "propose",
                "run_agents": False,
            },
            expected={
                "run_agents": False,
                "decision": "confirm_subject",
                "primary_subject": "finance",
                "effective_domain": "finance",
                "resource_levels": ["subject_overlay", "subject_skill"],
                "guidance_source": ".qiongli/guidance.d/subject-runtime.md",
            },
            source=Path("confirmed_finance_guidance_loaded.json"),
            setup_subject_action={
                "action": "confirm",
                "subject": "finance",
                "run_id": "setup-confirm-finance",
            },
        )
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "passed", result["failures"])
        task_call = captured_calls[-1]
        self.assertEqual(task_call[0], "qiongli_task_run")
        task_args = task_call[1]
        self.assertIs(task_args["run_agents"], True)
        self.assertEqual(task_args["max_revision_rounds"], 0)
        self.assertEqual(task_args["output_budget"], 1)
        self.assertIs(task_args["skip_validation"], True)
        self.assertEqual(task_args["execution_mode"], "solo")
        self.assertEqual(task_args["controller"], "codex")
        self.assertEqual(task_args["primary"], "codex")
        self.assertEqual(task_args["reviewer"], "codex")
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_local_agent_case_uses_bounded_task_arguments
```

Expected: failure because local-agent mode only sets `run_agents` today.

- [ ] **Step 3: Add local-agent bounded argument constants**

In `tooling/scripts/run_subject_runtime_smoke.py`, add near the other constants:

```python
SUBJECT_GUIDANCE_SOURCE = ".qiongli/guidance.d/subject-runtime.md"
LOCAL_AGENT_TASK_OVERRIDES: dict[str, Any] = {
    "run_agents": True,
    "max_revision_rounds": 0,
    "output_budget": 1,
    "skip_validation": True,
    "execution_mode": "solo",
    "controller": "codex",
    "primary": "codex",
    "reviewer": "codex",
    "solo_role_gates": "standard",
}
```

- [ ] **Step 4: Add task argument builder and local-agent run expectation**

Add below `_isolated_env()`:

```python
def _task_run_args_for_mode(case: SmokeCase, project_root: Path, mode: str) -> dict[str, Any]:
    args = dict(case.args)
    args["cwd"] = str(project_root)
    if mode == "local-agent":
        args.update(LOCAL_AGENT_TASK_OVERRIDES)
    else:
        args["run_agents"] = False
    return args
```

In `run_smoke_case()`, replace the inline task args block with:

```python
        args = _task_run_args_for_mode(case, project_root, mode)
        result = call_qiongli_tool("qiongli_task_run", args)
```

Change the assertion call in `run_smoke_case()` from:

```python
    failures = _assert_case(case, result)
```

to:

```python
    failures = _assert_case(case, result, mode=mode)
```

Change the helper signature:

```python
def _assert_case(
    case: SmokeCase,
    result: dict[str, Any],
    *,
    mode: str = "preview",
) -> list[str]:
```

Replace the `run_agents` comparison inside `_assert_case()` with:

```python
    expected_run_agents = True if mode == "local-agent" else expected.get("run_agents")
    _expect_equal(failures, "run_agents", payload.get("run_agents"), expected_run_agents)
```

- [ ] **Step 5: Run smoke runner tests**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: all subject runtime smoke tests pass.

- [ ] **Step 6: Commit Task 3**

Run:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/test_subject_runtime_smoke.py
git commit -m "feat(smoke): bound local-agent subject smoke runs"
```

## Task 4: Local-Agent Assertions And Report Metadata

**Files:**
- Modify: `tooling/scripts/run_subject_runtime_smoke.py`
- Modify: `tests/test_subject_runtime_smoke.py`

- [ ] **Step 1: Write failing tests for local-agent assertion failures and metadata**

Add these tests to `tests/test_subject_runtime_smoke.py`:

```python
    def test_local_agent_assertion_requires_guidance_trace(self) -> None:
        case = SmokeCase(
            name="confirmed_finance_guidance_loaded",
            manifest=None,
            args={},
            expected={
                "run_agents": True,
                "decision": "confirm_subject",
                "primary_subject": "finance",
                "effective_domain": "finance",
                "resource_levels": [],
                "guidance_source": ".qiongli/guidance.d/subject-runtime.md",
            },
            source=Path("confirmed_finance_guidance_loaded.json"),
        )
        result = {
            "structuredContent": {
                "run_agents": True,
                "data": {
                    "task_packet": {
                        "local_guidance": {
                            "guidance_files_read": [
                                ".qiongli/guidance.d/subject-runtime.md"
                            ]
                        },
                        "subject_refinement": {
                            "decision": "confirm_subject",
                            "primary_subject": "finance",
                            "loaded_resources": {"levels": []},
                            "signals": [],
                            "resource_activation_plan": {},
                        },
                        "domain": "finance",
                    }
                },
            }
        }

        failures = smoke._assert_case(case, result, mode="local-agent", project_root=Path("/tmp/project"))

        self.assertIn("missing local guidance trace", failures)

    def test_local_agent_report_includes_runtime_metadata(self) -> None:
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
                                "review_agent": "codex",
                                "fallback_agent": "codex",
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
                        "routing_notes": ["Runtime plan: draft=codex, review=codex."],
                    },
                },
                "isError": False,
            }

        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch.object(smoke, "call_qiongli_tool", side_effect=fake_call):
                result = smoke.run_smoke_case(case, Path(tmp_dir), "local-agent")

        self.assertEqual(result["status"], "passed", result["failures"])
        self.assertEqual(
            result["local_agent"]["runtime_plan"],
            {"primary_agent": "codex", "review_agent": "codex", "fallback_agent": "codex"},
        )
        self.assertTrue(result["trace_assertions"]["trace_written"])
        self.assertTrue(result["trace_assertions"]["subject_guidance_loaded"])
        self.assertTrue(result["trace_assertions"]["subject_refinement_persisted"])
```

- [ ] **Step 2: Run the focused tests to verify they fail**

Run:

```bash
uv run python -m unittest \
  tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_local_agent_assertion_requires_guidance_trace \
  tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_local_agent_report_includes_runtime_metadata
```

Expected: failures because `_assert_case()` does not accept `project_root` yet
and the case report has no `local_agent` or `trace_assertions` metadata.

- [ ] **Step 3: Extend `_assert_case()` with project-root context**

In `run_smoke_case()`, change the Task 3 assertion call from:

```python
    failures = _assert_case(case, result, mode=mode)
```

to:

```python
    payload = result.get("structuredContent", result)
    failures = _assert_case(case, result, mode=mode, project_root=project_root)
```

Change the helper signature from the Task 3 version to:

```python
def _assert_case(
    case: SmokeCase,
    result: dict[str, Any],
    *,
    mode: str = "preview",
    project_root: Path | None = None,
) -> list[str]:
```

Existing tests that call `_assert_case(case, result)` should continue to work
because both new arguments have defaults.

- [ ] **Step 4: Add local-agent metadata helpers**

Add these helpers below `_expect_equal()`:

```python
def _payload_data(payload: dict[str, Any]) -> dict[str, Any]:
    data = payload.get("data", {})
    return data if isinstance(data, dict) else {}


def _task_packet_from_payload(payload: dict[str, Any]) -> dict[str, Any]:
    data = _payload_data(payload)
    packet = data.get("task_packet", {})
    return packet if isinstance(packet, dict) else {}


def _local_guidance_trace_from_payload(payload: dict[str, Any]) -> dict[str, Any]:
    data = _payload_data(payload)
    trace = data.get("local_guidance_trace", {})
    return trace if isinstance(trace, dict) else {}


def _local_agent_metadata(payload: dict[str, Any]) -> dict[str, Any]:
    packet = _task_packet_from_payload(payload)
    return {
        "requested": True,
        "env_opt_in": os.environ.get(LOCAL_AGENT_ENV) == "1",
        "will_launch_agents": bool(payload.get("run_agents")),
        "runtime_plan": dict(packet.get("runtime_plan", {}) or {}),
    }


def _trace_assertions(payload: dict[str, Any]) -> dict[str, bool]:
    packet = _task_packet_from_payload(payload)
    guidance = packet.get("local_guidance", {})
    if not isinstance(guidance, dict):
        guidance = {}
    trace = _local_guidance_trace_from_payload(payload)
    files_read = list(guidance.get("guidance_files_read", []) or [])
    trace_files_read = list(trace.get("guidance_files_read", []) or [])
    subject_guidance_loaded = (
        SUBJECT_GUIDANCE_SOURCE in files_read
        or SUBJECT_GUIDANCE_SOURCE in trace_files_read
    )
    return {
        "trace_written": bool(trace),
        "subject_guidance_loaded": subject_guidance_loaded,
        "subject_refinement_persisted": isinstance(packet.get("subject_refinement"), dict),
    }
```

- [ ] **Step 5: Add local-agent assertion checks**

Near the end of `_assert_case()`, add:

```python
    if mode == "local-agent":
        trace = _local_guidance_trace_from_payload(payload)
        trace_assertions = _trace_assertions(payload)
        if not trace:
            failures.append("missing local guidance trace")
        if expected.get("guidance_source") and not trace_assertions["subject_guidance_loaded"]:
            failures.append(
                f"missing local-agent guidance source {expected['guidance_source']!r}"
            )
        if not trace_assertions["subject_refinement_persisted"]:
            failures.append("missing local-agent subject refinement packet")
```

- [ ] **Step 6: Include metadata in case reports**

In `run_smoke_case()`, after `payload = result.get("structuredContent", result)`,
build the base report:

```python
    report = {
        "name": case.name,
        "source": _repo_relative(case.source),
        "project_root": str(project_root),
        "status": "passed" if not failures else "failed",
        "failures": failures,
        "environment": env_updates,
        "result": payload,
    }
    if mode == "local-agent":
        report["local_agent"] = _local_agent_metadata(payload)
        report["trace_assertions"] = _trace_assertions(payload)
    return report
```

Remove or replace the existing direct return dict at the end of `run_smoke_case()`.

- [ ] **Step 7: Run smoke tests**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: all subject runtime smoke tests pass.

- [ ] **Step 8: Commit Task 4**

Run:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/test_subject_runtime_smoke.py
git commit -m "feat(smoke): report local-agent trace assertions"
```

## Task 5: Write-Boundary Checks And Rerun Diagnostics

**Files:**
- Modify: `tooling/scripts/run_subject_runtime_smoke.py`
- Modify: `tests/test_subject_runtime_smoke.py`

- [ ] **Step 1: Write failing tests for write-boundary checks**

Add these tests to `tests/test_subject_runtime_smoke.py`:

```python
    def test_write_boundary_detects_outside_trace_path(self) -> None:
        project_root = Path("/tmp/project").resolve()
        payload = {
            "data": {
                "local_guidance_trace": {
                    "run_dir": "/tmp/outside/run-1",
                    "trace_index": ".qiongli/trace/index.jsonl",
                }
            }
        }

        result = smoke._write_boundary_report(payload, project_root)

        self.assertFalse(result["known_paths_inside_project"])
        self.assertTrue(any("/tmp/outside/run-1" in item for item in result["violations"]))

    def test_error_report_includes_rerun_command(self) -> None:
        error = RuntimeError("local-agent smoke requires QIONGLI_SMOKE_RUN_AGENTS=1")

        report = smoke._error_report("local-agent", error)

        self.assertEqual(report["schema_version"], "1.1")
        self.assertIn("rerun_command", report)
        self.assertIn("QIONGLI_SMOKE_RUN_AGENTS=1", report["rerun_command"])
        self.assertIn("--mode local-agent", report["rerun_command"])
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
uv run python -m unittest \
  tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_write_boundary_detects_outside_trace_path \
  tests.test_subject_runtime_smoke.SubjectRuntimeSmokeTests.test_error_report_includes_rerun_command
```

Expected: failures because `_write_boundary_report()` does not exist and
`_error_report()` has no rerun command.

- [ ] **Step 3: Add path resolution helpers**

In `tooling/scripts/run_subject_runtime_smoke.py`, add below `_repo_relative()`:

```python
def _resolve_reported_path(project_root: Path, value: Any) -> Path | None:
    if not isinstance(value, str) or not value.strip():
        return None
    raw = Path(value)
    return raw.resolve() if raw.is_absolute() else (project_root / raw).resolve()


def _path_inside_project(project_root: Path, path: Path) -> bool:
    try:
        path.relative_to(project_root.resolve())
        return True
    except ValueError:
        return False
```

- [ ] **Step 4: Add write-boundary report helper**

Add below the path helpers:

```python
def _write_boundary_report(payload: dict[str, Any], project_root: Path) -> dict[str, Any]:
    violations: list[str] = []
    expected_paths = [
        ".qiongli/guidance_manifest.yaml",
        SUBJECT_GUIDANCE_SOURCE,
        ".qiongli/trace",
    ]
    for rel_path in expected_paths:
        resolved = (project_root / rel_path).resolve()
        if not _path_inside_project(project_root, resolved):
            violations.append(str(resolved))

    trace = _local_guidance_trace_from_payload(payload)
    for key in ("run_dir", "trace_index", "proposal_path"):
        resolved = _resolve_reported_path(project_root, trace.get(key))
        if resolved is not None and not _path_inside_project(project_root, resolved):
            violations.append(str(resolved))

    return {
        "known_paths_inside_project": not violations,
        "violations": violations,
    }
```

- [ ] **Step 5: Include write-boundary report in local-agent case reports and failures**

In `run_smoke_case()`, when `mode == "local-agent"`, add:

```python
        write_boundary = _write_boundary_report(payload, project_root)
        report["write_boundary"] = write_boundary
        if not write_boundary["known_paths_inside_project"]:
            report["status"] = "failed"
            report["failures"].extend(
                f"write boundary violation: {item}"
                for item in write_boundary["violations"]
            )
```

Ensure this runs after the base report is built.

- [ ] **Step 6: Add rerun command helper**

Add near `_error_report()`:

```python
def _rerun_command(mode: str, case_name: str | None = None) -> str:
    parts = [
        "uv",
        "run",
        "python",
        "tooling/scripts/run_subject_runtime_smoke.py",
        "--mode",
        mode,
    ]
    if case_name:
        parts.extend(["--case", case_name])
    parts.append("--json")
    command = " ".join(parts)
    if mode == "local-agent":
        command = f"{LOCAL_AGENT_ENV}=1 {command}"
    return command
```

Update `_error_report()`:

```python
        "rerun_command": _rerun_command(mode),
```

When a specific local-agent case fails inside `run_smoke_case()`, include:

```python
        report["rerun_command"] = _rerun_command(mode, case.name)
```

- [ ] **Step 7: Run smoke tests**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: all smoke tests pass.

- [ ] **Step 8: Commit Task 5**

Run:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/test_subject_runtime_smoke.py
git commit -m "feat(smoke): audit local-agent write boundaries"
```

## Task 6: Release-Readiness Documentation

**Files:**
- Modify: `docs/advanced/publish-pypi.md`
- Modify: `docs/zh/advanced/publish-pypi.md`

- [ ] **Step 1: Add English maintainer smoke documentation**

In `docs/advanced/publish-pypi.md`, add a short subsection near the existing
release smoke documentation:

```markdown
### Optional subject runtime local-agent smoke

The default release smoke remains preview-first and does not launch local
agents. Before a release candidate, maintainers can additionally verify the
adaptive subject runtime with a real local-agent run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
uv run python tooling/scripts/evaluate_subject_router.py --json
QIONGLI_SMOKE_RUN_AGENTS=1 \
uv run python tooling/scripts/run_subject_runtime_smoke.py \
  --mode local-agent \
  --case confirmed_finance_guidance_loaded \
  --json
```

The local-agent command is opt-in and should be treated as a maintainer
confidence check. It verifies that confirmed subject guidance is loaded through
`.qiongli/guidance.d/subject-runtime.md`, that a local guidance trace is
written, and that Qiongli-visible paths remain inside the isolated smoke root.
```

- [ ] **Step 2: Add Chinese maintainer smoke documentation**

In `docs/zh/advanced/publish-pypi.md`, add the Chinese equivalent near the
existing release smoke documentation:

```markdown
### 可选的 subject runtime 本地 agent smoke

默认 release smoke 仍然是 preview-first，不会启动本地 agent。发布候选版本前，
维护者可以额外运行一次真实本地 agent smoke 来验证 adaptive subject runtime：

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
uv run python tooling/scripts/evaluate_subject_router.py --json
QIONGLI_SMOKE_RUN_AGENTS=1 \
uv run python tooling/scripts/run_subject_runtime_smoke.py \
  --mode local-agent \
  --case confirmed_finance_guidance_loaded \
  --json
```

这个 local-agent 命令必须显式 opt in，只作为维护者信心检查。它会验证已确认
subject 的 `.qiongli/guidance.d/subject-runtime.md` 被真实 task run 加载、
local guidance trace 已写入，并且 Qiongli 可见路径仍在隔离 smoke root 内。
```

- [ ] **Step 3: Run doc checks**

Run:

```bash
rg -n "QIONGLI_SMOKE_RUN_AGENTS=1|confirmed_finance_guidance_loaded" docs/advanced/publish-pypi.md docs/zh/advanced/publish-pypi.md
git diff --check
```

Expected: both docs contain the command and `git diff --check` exits `0`.

- [ ] **Step 4: Commit Task 6**

Run:

```bash
git add docs/advanced/publish-pypi.md docs/zh/advanced/publish-pypi.md
git commit -m "docs(release): document subject local-agent smoke"
```

## Task 7: Final Verification And Integration

**Files:**
- No new source files beyond previous tasks.

- [ ] **Step 1: Run focused unit tests**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke tests.test_mcp_tool_handlers
```

Expected: all focused tests pass.

- [ ] **Step 2: Run full subject runtime verification**

Run:

```bash
uv run python -m unittest tests.test_subject_guidance tests.test_subject_lifecycle tests.test_guidance_runtime tests.test_subject_runtime_smoke tests.test_subject_refinement tests.test_subject_router_eval
```

Expected: all selected subject runtime tests pass.

- [ ] **Step 3: Run preview smoke and router eval**

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: preview smoke exits `0` with all cases passing; router eval exits `0`
with no threshold failures.

- [ ] **Step 4: Verify local-agent guard failure remains machine-readable**

Run without the environment variable:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --mode local-agent --json
```

Expected: command exits non-zero and prints JSON with:

```json
{
  "schema_version": "1.1",
  "mode": "local-agent",
  "summary": {"total": 0, "passed": 0, "failed": 1},
  "error": "local-agent smoke requires QIONGLI_SMOKE_RUN_AGENTS=1 and launches local runtime agents"
}
```

The JSON should also contain a `rerun_command` including
`QIONGLI_SMOKE_RUN_AGENTS=1`.

- [ ] **Step 5: Optionally run real local-agent smoke when local runtime is available**

Run only when the maintainer environment has the selected local runtime
configured:

```bash
QIONGLI_SMOKE_RUN_AGENTS=1 \
uv run python tooling/scripts/run_subject_runtime_smoke.py \
  --mode local-agent \
  --case confirmed_finance_guidance_loaded \
  --json
```

Expected if local runtime is available: command exits `0`, `summary.failed == 0`,
`local_agent.will_launch_agents == true`, `trace_assertions.trace_written ==
true`, and `write_boundary.known_paths_inside_project == true`.

Expected if local runtime is unavailable: command exits non-zero with a failed
case that includes runtime preflight or execution diagnostics. Do not treat an
unavailable local runtime as a blocker for merging this implementation unless
the failure is caused by Qiongli smoke runner logic.

- [ ] **Step 6: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: exits `0`.

- [ ] **Step 7: Final status check**

Run:

```bash
git status --short --branch
git log --oneline --max-count=8
```

Expected: implementation branch is clean after all commits. Recent commits show
the task-by-task smoke hardening work.

## Self-Review Checklist

- Spec coverage:
  - Opt-in local-agent smoke double gate: Task 1, Task 7.
  - Bounded real-run arguments: Task 2, Task 3.
  - Loaded subject guidance assertions: Task 4.
  - Local guidance trace assertions: Task 4.
  - Write-boundary checks: Task 5.
  - Rerun diagnostics: Task 5.
  - Preview smoke remains default: Task 1, Task 7.
  - Release-readiness documentation: Task 6.
- Open-item scan:
  - No unresolved markers or open-ended edge-case instructions.
- Type consistency:
  - Smoke report schema uses `schema_version: "1.1"`.
  - Local-agent env var remains `QIONGLI_SMOKE_RUN_AGENTS`.
  - Subject guidance source remains `.qiongli/guidance.d/subject-runtime.md`.
  - MCP bounded args use `max_revision_rounds`, `output_budget`, and
    `skip_validation`.
