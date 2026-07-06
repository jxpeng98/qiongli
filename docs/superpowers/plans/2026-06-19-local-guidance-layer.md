# Local Guidance Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the project-local Qiongli guidance and trace layer from the approved spec, and make current task-run output behavior auditable when formal `RESEARCH/[topic]/...` artifacts are missing.

**Architecture:** Add a focused guidance runtime module that owns `.qiongli/local_guidance.md`, `.qiongli/trace/index.jsonl`, and per-run trace bundles. Inject effective guidance into `ModelOrchestrator.task_run`, write trace bundles after validator evaluation, and expose CLI/MCP guidance controls without mutating canonical skills or release payloads.

**Tech Stack:** Python 3.12, `unittest`, existing `bridges.orchestrator`, existing `qiongli.cli`, existing MCP tool handler schema.

---

## Root Cause Summary

Current `v1.5.0b2` / `v1.5.0-beta.2` behavior:

- `task_run` builds a task packet with `artifact_root="RESEARCH/[topic]/"` and `required_outputs`, prompts the runtime to produce those outputs, then validates the disk.
- It does not have a generic writer that materializes draft sections into `RESEARCH/[topic]/...`.
- Literature search is different because `bridges.providers.literature_artifacts.materialize_search_bundle()` explicitly writes that bundle.
- `qiongli_task_run` over the full MCP server defaults to preview and does not launch agents unless `run_agents` is the JSON boolean `true`.
- The bundled `qiongli` / `qiongli-next` plugin MCP focuses on provider tools and does not run Python orchestrator agents.

This plan does not introduce a generic formal artifact materializer. It implements the approved trace bundle so printed task-run content becomes project-local, linked evidence even when formal outputs are missing.

## File Structure

- Create `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
  - Owns guidance path resolution, scaffold text, effective guidance merging, trace bundle writing, index writing, proposal creation, and proposal application.
- Create `tests/test_guidance_runtime.py`
  - Unit tests for guidance path resolution, init/show/trace/apply, merge precedence, and trace bundle writing.
- Modify `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
  - Add `guidance_mode` argument and parser option.
  - Prepare local guidance before draft prompt construction.
  - Inject `packet["local_guidance"]`.
  - Include guidance context in draft and review prompts.
  - Write trace bundles after validator gate evaluation and include trace metadata in result data.
- Modify `tests/test_orchestrator_workflows.py`
  - Add failing tests for guidance injection, trace bundle writing on missing formal outputs, and `--guidance-mode off`.
- Modify `packages/python-qiongli/src/qiongli/cli.py`
  - Add `qiongli guidance init/show/trace/apply`.
- Modify `tests/test_cli.py`
  - Add CLI coverage for guidance commands.
- Modify `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Accept `guidance_mode` on `qiongli_task_run` and pass it through.
- Modify `tests/test_mcp_tool_handlers.py`
  - Add MCP preview/run argument coverage for `guidance_mode`.
- Modify `docs/reference/cli.md` and `docs/zh/reference/cli.md`
  - Document `--guidance-mode` and `qiongli guidance`.

## Task 1: Guidance Runtime Unit Tests

**Files:**
- Create: `tests/test_guidance_runtime.py`
- Create later: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`

- [ ] **Step 1: Write failing tests for init and effective guidance**

```python
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from bridges.guidance_runtime import (
    GUIDANCE_MODES,
    apply_guidance_proposal,
    effective_guidance,
    guidance_trace_summary,
    init_project_guidance,
    write_guidance_trace,
)


class GuidanceRuntimeTests(unittest.TestCase):
    def test_init_project_guidance_creates_project_local_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            paths = init_project_guidance(root)

            self.assertEqual(paths.project_guidance, root / ".qiongli" / "local_guidance.md")
            self.assertTrue(paths.project_guidance.is_file())
            self.assertTrue((root / ".qiongli" / "trace").is_dir())
            self.assertIn("# Qiongli Local Guidance", paths.project_guidance.read_text(encoding="utf-8"))

    def test_effective_guidance_project_overrides_global_preferences(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir) / "project"
            root.mkdir()
            home = Path(tmp_dir) / "home"
            home.mkdir()
            (home / "preferences.md").write_text(
                "# Qiongli User Preferences\n\n## Artifact Preferences\n- Prefer compact outputs.\n",
                encoding="utf-8",
            )
            init_project_guidance(root)
            (root / ".qiongli" / "local_guidance.md").write_text(
                "# Qiongli Local Guidance\n\n## Artifact Policy\n- Keep trace bundles in the project.\n",
                encoding="utf-8",
            )

            with mock.patch.dict("os.environ", {"QIONGLI_GUIDANCE_HOME": str(home)}):
                state = effective_guidance(root, mode="read")

            self.assertTrue(state.enabled)
            self.assertEqual(state.mode, "read")
            self.assertIn("Prefer compact outputs", state.guidance_context)
            self.assertIn("Keep trace bundles in the project", state.guidance_context)
            self.assertEqual(state.project_guidance_file, ".qiongli/local_guidance.md")
            self.assertEqual(state.trace_dir, "")

    def test_effective_guidance_off_mode_skips_guidance_reads(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)

            state = effective_guidance(root, mode="off")

            self.assertFalse(state.enabled)
            self.assertEqual(state.guidance_context, "")
            self.assertEqual(state.guidance_files_read, [])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `python3 -m unittest tests.test_guidance_runtime -v`

Expected: FAIL or ERROR with `ModuleNotFoundError` or missing symbols from `bridges.guidance_runtime`.

## Task 2: Minimal Guidance Runtime Implementation

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- Test: `tests/test_guidance_runtime.py`

- [ ] **Step 1: Implement data classes, init, and effective guidance**

Create `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py` with:

```python
from __future__ import annotations

import hashlib
import json
import os
import uuid
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


GUIDANCE_MODES = ("off", "read", "propose", "apply")
LOCAL_GUIDANCE_REL = Path(".qiongli") / "local_guidance.md"
TRACE_REL = Path(".qiongli") / "trace"
TRACE_INDEX_REL = TRACE_REL / "index.jsonl"


@dataclass(frozen=True)
class GuidancePaths:
    project_root: Path
    project_guidance: Path
    trace_root: Path
    trace_index: Path
    global_preferences: Path


@dataclass(frozen=True)
class GuidanceState:
    enabled: bool
    mode: str
    project_guidance_file: str
    global_preferences_file: str
    trace_dir: str
    summary: str
    guidance_context: str
    guidance_files_read: list[str]
    run_id: str = ""
    warnings: list[str] | None = None

    def to_packet(self) -> dict[str, Any]:
        return asdict(self)
```

Then implement:

```python
def resolve_guidance_paths(project_root: Path) -> GuidancePaths:
    root = Path(project_root).resolve()
    global_home = os.environ.get("QIONGLI_GUIDANCE_HOME") or os.environ.get("QIONGLI_CONFIG_HOME")
    global_root = Path(global_home).expanduser().resolve() if global_home else Path.home() / ".qiongli"
    return GuidancePaths(
        project_root=root,
        project_guidance=root / LOCAL_GUIDANCE_REL,
        trace_root=root / TRACE_REL,
        trace_index=root / TRACE_INDEX_REL,
        global_preferences=global_root / "preferences.md",
    )


def init_project_guidance(project_root: Path) -> GuidancePaths:
    paths = resolve_guidance_paths(project_root)
    paths.trace_root.mkdir(parents=True, exist_ok=True)
    if not paths.project_guidance.exists():
        paths.project_guidance.parent.mkdir(parents=True, exist_ok=True)
        paths.project_guidance.write_text(_default_local_guidance(), encoding="utf-8")
    return paths


def effective_guidance(project_root: Path, *, mode: str = "propose", run_id: str = "") -> GuidanceState:
    normalized_mode = _normalize_mode(mode)
    paths = resolve_guidance_paths(project_root)
    if normalized_mode == "off":
        return GuidanceState(
            enabled=False,
            mode="off",
            project_guidance_file=_rel(paths.project_root, paths.project_guidance),
            global_preferences_file=str(paths.global_preferences),
            trace_dir="",
            summary="Local guidance disabled for this run.",
            guidance_context="",
            guidance_files_read=[],
            run_id=run_id,
            warnings=[],
        )
    sections: list[str] = []
    files_read: list[str] = []
    warnings: list[str] = []
    for label, path in (("Global Preferences", paths.global_preferences), ("Project Local Guidance", paths.project_guidance)):
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8").strip()
        except OSError as exc:
            warnings.append(f"Failed to read {path}: {exc}")
            continue
        if text:
            sections.append(f"## {label}\n\n{text}")
            files_read.append(str(path) if label == "Global Preferences" else _rel(paths.project_root, path))
    context = "\n\n".join(sections)
    return GuidanceState(
        enabled=bool(context),
        mode=normalized_mode,
        project_guidance_file=_rel(paths.project_root, paths.project_guidance),
        global_preferences_file=str(paths.global_preferences),
        trace_dir="",
        summary=_summarize_guidance(context, files_read),
        guidance_context=context,
        guidance_files_read=files_read,
        run_id=run_id,
        warnings=warnings,
    )
```

- [ ] **Step 2: Run guidance runtime tests**

Run: `python3 -m unittest tests.test_guidance_runtime -v`

Expected: the first three tests pass; later tests added in Task 3 are not present yet.

## Task 3: Trace Bundle and Proposal Tests

**Files:**
- Modify: `tests/test_guidance_runtime.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`

- [ ] **Step 1: Add failing tests for trace writing and proposal apply**

Append to `GuidanceRuntimeTests`:

```python
    def test_write_guidance_trace_creates_linked_bundle_and_index(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            state = effective_guidance(root, mode="propose", run_id="run-123")

            trace = write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={"task_id": "F3", "paper_type": "empirical", "topic": "ai-writing", "required_outputs": ["manuscript/manuscript.md"]},
                draft_content="draft body",
                review_content="review body",
                merged_analysis="merged body",
                validator_gate={"passed": False, "found": [], "missing": ["manuscript/manuscript.md"], "checked": 1},
                applied=False,
            )

            run_dir = root / ".qiongli" / "trace" / "runs" / "run-123"
            self.assertEqual(trace["run_dir"], ".qiongli/trace/runs/run-123")
            for filename in (
                "task_packet.json",
                "guidance_context.md",
                "draft.md",
                "review.md",
                "merged_analysis.md",
                "validator_gate.json",
                "guidance_update_proposal.md",
            ):
                self.assertTrue((run_dir / filename).is_file(), filename)
            index_rows = [
                json.loads(line)
                for line in (root / ".qiongli" / "trace" / "index.jsonl").read_text(encoding="utf-8").splitlines()
            ]
            self.assertEqual(index_rows[0]["run_id"], "run-123")
            self.assertEqual(index_rows[0]["missing_outputs"], ["manuscript/manuscript.md"])

    def test_apply_guidance_proposal_appends_revision_history_only_to_project_file(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            proposal = root / ".qiongli" / "trace" / "runs" / "run-1" / "guidance_update_proposal.md"
            proposal.parent.mkdir(parents=True)
            proposal.write_text(
                "# Guidance Update Proposal\n\n## Proposed Changes\n\n- Prefer project-local trace bundles.\n",
                encoding="utf-8",
            )

            result = apply_guidance_proposal(root, proposal)

            text = (root / ".qiongli" / "local_guidance.md").read_text(encoding="utf-8")
            self.assertTrue(result["applied"])
            self.assertIn("Prefer project-local trace bundles", text)
            self.assertIn("run-1", text)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `python3 -m unittest tests.test_guidance_runtime -v`

Expected: FAIL for missing `write_guidance_trace` / `apply_guidance_proposal` behavior.

- [ ] **Step 3: Implement trace writer and proposal apply**

Add to `guidance_runtime.py`:

```python
def write_guidance_trace(
    *,
    project_root: Path,
    guidance_state: GuidanceState,
    task_packet: dict[str, Any],
    draft_content: str,
    review_content: str,
    merged_analysis: str,
    validator_gate: dict[str, Any],
    applied: bool,
) -> dict[str, Any]:
    paths = init_project_guidance(project_root)
    run_id = guidance_state.run_id or uuid.uuid4().hex
    run_dir = paths.trace_root / "runs" / run_id
    run_dir.mkdir(parents=True, exist_ok=True)
    _write_json(run_dir / "task_packet.json", task_packet)
    (run_dir / "guidance_context.md").write_text(guidance_state.guidance_context or "Local guidance disabled or empty.\n", encoding="utf-8")
    (run_dir / "draft.md").write_text(draft_content or "[no draft produced]\n", encoding="utf-8")
    (run_dir / "review.md").write_text(review_content or "[no review produced]\n", encoding="utf-8")
    (run_dir / "merged_analysis.md").write_text(merged_analysis or "[no merged analysis]\n", encoding="utf-8")
    _write_json(run_dir / "validator_gate.json", validator_gate)
    proposal_text = _proposal_text(task_packet, validator_gate, applied)
    (run_dir / "guidance_update_proposal.md").write_text(proposal_text, encoding="utf-8")
    record = {
        "run_id": run_id,
        "created_at": _utc_now(),
        "task_id": str(task_packet.get("task_id", "")),
        "paper_type": str(task_packet.get("paper_type", "")),
        "topic": str(task_packet.get("topic", "")),
        "cwd": str(paths.project_root),
        "guidance_mode": guidance_state.mode,
        "run_dir": _rel(paths.project_root, run_dir),
        "required_outputs": list(task_packet.get("required_outputs", []) or []),
        "found_outputs": list(validator_gate.get("found", []) or []),
        "missing_outputs": list(validator_gate.get("missing", []) or []),
        "guidance_files_read": list(guidance_state.guidance_files_read),
        "guidance_proposal": _rel(paths.project_root, run_dir / "guidance_update_proposal.md"),
        "applied_guidance_update": bool(applied),
    }
    with paths.trace_index.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
    return record
```

- [ ] **Step 4: Run tests to verify pass**

Run: `python3 -m unittest tests.test_guidance_runtime -v`

Expected: PASS.

## Task 4: Orchestrator Guidance Injection Tests

**Files:**
- Modify: `tests/test_orchestrator_workflows.py`
- Modify later: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`

- [ ] **Step 1: Add failing tests for task packet injection and prompt text**

Add tests near existing task prompt tests:

```python
    def test_task_run_injects_project_local_guidance_into_packet_and_prompt(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "local_guidance.md").write_text(
                "# Qiongli Local Guidance\n\n## Artifact Policy\n- Keep helper traces in .qiongli/trace.\n",
                encoding="utf-8",
            )
            orchestrator = MockOrchestrator()

            result = orchestrator.task_run(
                task_id="F3",
                paper_type="empirical",
                topic="ai-writing",
                cwd=root,
                guidance_mode="read",
                skip_validation=True,
            )

            packet = result.data["task_packet"]
            self.assertEqual(packet["local_guidance"]["mode"], "read")
            self.assertIn("Keep helper traces", packet["local_guidance"]["guidance_context"])
            draft_prompt = next(call["prompt"] for call in orchestrator.runtime_calls if call["agent"])
            self.assertIn("Local guidance context", draft_prompt)
            self.assertIn("Keep helper traces", draft_prompt)

    def test_task_run_guidance_off_does_not_read_local_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "local_guidance.md").write_text(
                "# Qiongli Local Guidance\n\n## Active Guidance\n- This text must not appear.\n",
                encoding="utf-8",
            )
            orchestrator = MockOrchestrator()

            result = orchestrator.task_run(
                task_id="F3",
                paper_type="empirical",
                topic="ai-writing",
                cwd=root,
                guidance_mode="off",
                skip_validation=True,
            )

            self.assertFalse(result.data["task_packet"]["local_guidance"]["enabled"])
            self.assertNotIn("This text must not appear", result.merged_analysis)
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `python3 -m unittest tests.test_orchestrator_workflows.OrchestratorWorkflowTests.test_task_run_injects_project_local_guidance_into_packet_and_prompt tests.test_orchestrator_workflows.OrchestratorWorkflowTests.test_task_run_guidance_off_does_not_read_local_guidance -v`

Expected: FAIL with `unexpected keyword argument guidance_mode` or missing `local_guidance`.

## Task 5: Orchestrator Guidance Injection Implementation

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Test: `tests/test_orchestrator_workflows.py`

- [ ] **Step 1: Import guidance runtime and add argument**

Add import:

```python
from .guidance_runtime import GUIDANCE_MODES, effective_guidance, write_guidance_trace
```

Add `guidance_mode: str = "propose"` to `ModelOrchestrator.task_run(...)`.

- [ ] **Step 2: Prepare guidance state after packet construction**

After `packet.update(self._build_domain_packet_fields(domain_context))`, add:

```python
        guidance_run_id = uuid.uuid4().hex
        guidance_state = effective_guidance(cwd, mode=guidance_mode, run_id=guidance_run_id)
        packet["local_guidance"] = guidance_state.to_packet()
        if guidance_state.warnings:
            routing_notes.extend(f"Local guidance warning: {item}" for item in guidance_state.warnings)
        if guidance_state.enabled:
            routing_notes.append(
                "Local guidance ACTIVE: "
                f"mode={guidance_state.mode}, files={', '.join(guidance_state.guidance_files_read) or 'none'}."
            )
        elif guidance_state.mode == "off":
            routing_notes.append("Local guidance disabled by --guidance-mode=off.")
```

- [ ] **Step 3: Add guidance text to draft and review prompts**

In `_build_task_draft_prompt`, read:

```python
        local_guidance = task_packet.get("local_guidance", {})
        local_guidance_section = ""
        local_guidance_rules = ""
        if isinstance(local_guidance, dict) and local_guidance.get("enabled"):
            return_sections.append("- Local Guidance Compliance")
            local_guidance_rules = """
25. Local guidance is active. Treat it as advisory project-local context.
26. Do not follow local guidance when it conflicts with required_outputs, quality_gates, evidence requirements, strict validation, or safety constraints.
"""
            local_guidance_section = (
                "Local guidance context:\n"
                + str(local_guidance.get("guidance_context", "")).strip()
                + "\n"
            )
```

Add `{local_guidance_rules}` to execution rules and `{local_guidance_section}` before additional context.

In `_build_task_review_prompt`, add a shorter local guidance section and checklist item to block conflicts.

- [ ] **Step 4: Run targeted orchestrator tests**

Run the two tests from Task 4.

Expected: PASS.

## Task 6: Orchestrator Trace Bundle Tests

**Files:**
- Modify: `tests/test_orchestrator_workflows.py`
- Modify later: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`

- [ ] **Step 1: Add failing test that validator failure still writes trace bundle**

Add:

```python
    def test_task_run_writes_guidance_trace_when_formal_outputs_are_missing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            orchestrator = MockOrchestrator()

            result = orchestrator.task_run(
                task_id="F3",
                paper_type="empirical",
                topic="ai-writing",
                cwd=root,
                guidance_mode="propose",
                focus_outputs=["manuscript/manuscript.md"],
            )

            trace = result.data["local_guidance_trace"]
            self.assertEqual(trace["missing_outputs"], ["manuscript/manuscript.md", "context/boundary_review.md"])
            run_dir = root / trace["run_dir"]
            self.assertTrue((run_dir / "task_packet.json").is_file())
            self.assertTrue((run_dir / "draft.md").is_file())
            self.assertTrue((run_dir / "validator_gate.json").is_file())
            self.assertTrue((root / ".qiongli" / "trace" / "index.jsonl").is_file())
            self.assertIn("Local guidance trace written", result.merged_analysis)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `python3 -m unittest tests.test_orchestrator_workflows.OrchestratorWorkflowTests.test_task_run_writes_guidance_trace_when_formal_outputs_are_missing -v`

Expected: FAIL because `local_guidance_trace` is missing.

- [ ] **Step 3: Write trace after merged analysis is assembled**

In `task_run`, after `merged = "\n".join(merged_parts)` and before confidence calculation, add:

```python
        local_guidance_trace: dict[str, Any] = {}
        if guidance_state.mode != "off":
            try:
                local_guidance_trace = write_guidance_trace(
                    project_root=cwd,
                    guidance_state=guidance_state,
                    task_packet=packet,
                    draft_content=draft_resp.content if draft_resp.success else f"[FAILED] {draft_resp.error}",
                    review_content=review_resp.content if review_resp and review_resp.success else "",
                    merged_analysis=merged,
                    validator_gate=validator_gate_result,
                    applied=guidance_state.mode == "apply",
                )
                routing_notes.append(f"Local guidance trace written: {local_guidance_trace['run_dir']}.")
                merged += "\n\n## Local Guidance Trace\n- " + local_guidance_trace["run_dir"]
            except OSError as exc:
                routing_notes.append(f"Local guidance trace write failed: {exc}")
```

Add `"local_guidance_trace": dict(local_guidance_trace)` to `result.data`.

- [ ] **Step 4: Run trace test**

Run the test from Step 2.

Expected: PASS.

## Task 7: Parser, CLI, and MCP Surface Tests

**Files:**
- Modify: `tests/test_cli.py`
- Modify: `tests/test_mcp_tool_handlers.py`
- Modify later: `packages/python-qiongli/src/qiongli/cli.py`
- Modify later: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify later: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`

- [ ] **Step 1: Add CLI tests**

Add tests that call `qiongli.cli.build_parser()` and command functions directly:

```python
def test_parser_accepts_guidance_command_group(self) -> None:
    from qiongli.cli import build_parser

    parser = build_parser()
    args = parser.parse_args(["guidance", "init", "--project-dir", "."])

    assert args.cmd == "guidance"
    assert args.guidance_cmd == "init"


def test_guidance_init_command_creates_project_files(self) -> None:
    from qiongli.cli import build_parser, cmd_guidance

    with tempfile.TemporaryDirectory() as tmp_dir:
        parser = build_parser()
        args = parser.parse_args(["guidance", "init", "--project-dir", tmp_dir])
        rc = cmd_guidance(args)

        assert rc == 0
        assert (Path(tmp_dir) / ".qiongli" / "local_guidance.md").is_file()
```

- [ ] **Step 2: Add MCP schema test for guidance_mode**

In `tests/test_mcp_tool_handlers.py`, add:

```python
def test_task_run_tool_accepts_guidance_mode_in_preview(self) -> None:
    result = call_qiongli_tool(
        "qiongli_task_run",
        {
            "task_id": "F3",
            "paper_type": "empirical",
            "topic": "ai-writing",
            "guidance_mode": "read",
        },
    )

    payload = result["structuredContent"]
    assert payload["data"]["task_run_preview"]["task_run_arguments"]["guidance_mode"] == "read"
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
python3 -m unittest tests.test_cli tests.test_mcp_tool_handlers -v
```

Expected: FAIL because `guidance` command and `guidance_mode` schema are missing.

## Task 8: CLI and MCP Implementation

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`

- [ ] **Step 1: Add task-run parser option**

In `bridges.orchestrator`, add `--guidance-mode` to task-run parser:

```python
task_run.add_argument(
    "--guidance-mode",
    choices=GUIDANCE_MODES,
    default="propose",
    help="Project-local guidance mode: off, read, propose, or apply.",
)
```

Pass `guidance_mode=getattr(args, "guidance_mode", "propose")` into `orchestrator.task_run`.

- [ ] **Step 2: Add `qiongli guidance` parser and command**

In `qiongli.cli`, import:

```python
from bridges.guidance_runtime import (
    apply_guidance_proposal,
    effective_guidance,
    guidance_trace_summary,
    init_project_guidance,
)
```

Add `guidance` subparser with `init`, `show`, `trace`, `apply`.

Add:

```python
def cmd_guidance(args: argparse.Namespace) -> int:
    project_dir = Path(args.project_dir).expanduser().resolve()
    action = getattr(args, "guidance_cmd", "")
    if action == "init":
        paths = init_project_guidance(project_dir)
        print(f"Created project guidance at {paths.project_guidance}")
        return 0
    if action == "show":
        state = effective_guidance(project_dir, mode="read")
        print(state.guidance_context or "No local guidance configured.")
        return 0
    if action == "trace":
        summary = guidance_trace_summary(project_dir)
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 0
    if action == "apply":
        result = apply_guidance_proposal(project_dir, Path(args.proposal))
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0
    raise RuntimeError(f"Unhandled guidance command: {action}")
```

Dispatch from `main()` when `args.cmd == "guidance"`.

- [ ] **Step 3: Add MCP schema and pass-through**

Add `"guidance_mode": {"type": "string", "enum": ["off", "read", "propose", "apply"]}` to `qiongli_task_run`.

In `_tool_task_run`, include `guidance_mode` in `task_run_kwargs` and preview arguments.

- [ ] **Step 4: Run CLI/MCP tests**

Run:

```bash
python3 -m unittest tests.test_cli tests.test_mcp_tool_handlers -v
```

Expected: PASS.

## Task 9: Documentation and Version Behavior Note

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/zh/reference/cli.md`
- Modify: `docs/advanced/cross-platform-mcp.md`

- [ ] **Step 1: Document guidance commands**

Add a CLI reference section:

```markdown
### `guidance`

Project-local guidance and trace helpers:

```bash
qiongli guidance init --project-dir .
qiongli guidance show --project-dir .
qiongli guidance trace --project-dir .
qiongli guidance apply --project-dir . --proposal .qiongli/trace/runs/<run_id>/guidance_update_proposal.md
```

Task runs default to `--guidance-mode propose`, which reads project guidance and writes a trace bundle under `.qiongli/trace/` without changing formal `RESEARCH/[topic]/...` outputs.
```

- [ ] **Step 2: Clarify MCP preview behavior**

In `docs/advanced/cross-platform-mcp.md`, add that `qiongli_task_run` preview creates no `RESEARCH/[topic]` files, and bundled `qiongli-next` literature MCP does not launch orchestrator agents.

- [ ] **Step 3: Run docs grep check**

Run: `rg -n "guidance-mode|qiongli guidance|task-run-preview" docs/reference/cli.md docs/zh/reference/cli.md docs/advanced/cross-platform-mcp.md`

Expected: all three docs contain the new guidance/preview notes.

## Task 10: Final Verification

**Files:**
- No new files.

- [ ] **Step 1: Run targeted test suite**

Run:

```bash
python3 -m unittest \
  tests.test_guidance_runtime \
  tests.test_orchestrator_workflows \
  tests.test_cli \
  tests.test_mcp_tool_handlers \
  tests.test_mcp_stdio_server \
  -v
```

Expected: PASS.

- [ ] **Step 2: Run strict validator if targeted suite passes**

Run: `python3 scripts/validate_research_standard.py --strict`

Expected: PASS.

- [ ] **Step 3: Inspect git diff**

Run:

```bash
git status --short
git diff --stat
```

Expected: only intended guidance runtime, tests, docs, and plan files are modified.
