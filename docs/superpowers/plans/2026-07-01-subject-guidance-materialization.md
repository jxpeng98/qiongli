# Subject Guidance Materialization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist explicitly confirmed or locked runtime subject state into project-local managed guidance so later Qiongli runs automatically load the right subject layer from adaptive core installs.

**Architecture:** Add a focused `subject_guidance.py` bridge that owns `.qiongli/guidance.d/subject-runtime.md` inspection, rendering, replacement, and disabling. `subject_lifecycle.py` remains the lifecycle coordinator and calls the materializer after manifest state changes; `guidance_runtime.py` continues to load `.qiongli/guidance.d/*.md` generically, so subject guidance behaves like any other project fragment.

**Tech Stack:** Python 3.11+, stdlib `unittest`, dataclasses, JSON/YAML bridge modules, existing CLI/MCP handlers, existing preview smoke runner, `uv run python -m unittest`.

---

## Source Spec

Implement the approved spec:

- `docs/superpowers/specs/2026-07-01-subject-guidance-materialization-design.md`

Do not expand the subject catalog. Do not change install-time subject selection. Keep all writes project-local.

## File Map

- Create `packages/python-qiongli/src/qiongli/bridges/subject_guidance.py`
  - Owns managed subject guidance path, markers, rendering, block replacement, disabled block rendering, and status inspection.
- Create `tests/test_subject_guidance.py`
  - Unit tests for the materializer without lifecycle, CLI, or MCP.
- Modify `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`
  - Calls `subject_guidance` from confirm, lock, unlock, and reset.
  - Adds `subject_guidance` to status packets.
- Modify `tests/test_subject_lifecycle.py`
  - Integration tests for lifecycle actions and guidance materialization.
- Modify `packages/python-qiongli/src/qiongli/cli.py`
  - Human output includes subject guidance state.
  - JSON output remains the raw lifecycle packet.
- Modify `tests/test_cli.py`
  - CLI JSON and human output coverage for subject guidance.
- Modify `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Update tool descriptions only if needed; handlers should inherit lifecycle packet changes.
- Modify `tests/test_mcp_tool_handlers.py`
  - MCP status and update tests assert `subject_guidance`.
- Modify `tests/test_guidance_runtime.py`
  - Proves `effective_guidance()` and trace records load generated subject guidance.
- Modify `tooling/scripts/run_subject_runtime_smoke.py`
  - Adds optional per-fixture lifecycle setup and expected guidance source assertions.
- Create `tests/fixtures/subject_runtime_smoke/confirmed_finance_guidance_loaded.json`
  - Smoke fixture for confirm finance then preview task run.
- Modify `tests/test_subject_runtime_smoke.py`
  - Updates expected case count and asserts confirmed-subject guidance loading.
- Modify `docs/advanced/subject-packaging-model.md`
  - Documents that confirmed/locked project subject state materializes to `.qiongli/guidance.d/subject-runtime.md`.
- Modify `docs/zh/advanced/subject-packaging-model.md`
  - Chinese parity update for the same user-facing behavior.

## Task 0: Prepare Worktree And Baseline

**Files:**
- No source files changed.

- [ ] **Step 1: Confirm current branch and cleanliness**

Run from repository root:

```bash
git status --short --branch
```

Expected: branch is `dev` and there are no unstaged or staged source changes before creating the worktree.

- [ ] **Step 2: Confirm `.worktrees` is ignored**

Run:

```bash
git check-ignore -q .worktrees
```

Expected: exit code `0`. If it fails, add `.worktrees/` to `.gitignore`, commit that docs/chore change, then continue.

- [ ] **Step 3: Create isolated feature worktree**

Run:

```bash
git worktree add .worktrees/subject-guidance-materialization -b feature/subject-guidance-materialization dev
```

Expected: output includes `Preparing worktree` and the new worktree path.

- [ ] **Step 4: Run baseline focused tests in the worktree**

Run from `.worktrees/subject-guidance-materialization`:

```bash
uv run python -m unittest tests.test_subject_lifecycle tests.test_guidance_runtime tests.test_mcp_tool_handlers tests.test_cli tests.test_subject_runtime_smoke
```

Expected: all selected tests pass before implementation. If they fail, stop and report the baseline failure.

## Task 1: Subject Guidance Materializer

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/subject_guidance.py`
- Create: `tests/test_subject_guidance.py`

- [ ] **Step 1: Write failing materializer tests**

Create `tests/test_subject_guidance.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.subject_guidance import (
    END_MARKER,
    START_MARKER,
    SUBJECT_GUIDANCE_REL,
    SubjectGuidanceError,
    disable_subject_guidance,
    inspect_subject_guidance,
    write_subject_guidance,
)


class SubjectGuidanceTests(unittest.TestCase):
    def test_inspect_missing_subject_guidance_does_not_create_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            status = inspect_subject_guidance(root)

            self.assertFalse(status["exists"])
            self.assertEqual(status["managed_block"], "missing")
            self.assertEqual(status["path"], ".qiongli/guidance.d/subject-runtime.md")
            self.assertFalse((root / ".qiongli").exists())

    def test_write_confirmed_subject_guidance_creates_active_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            status = write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
                run_id="run-1",
                method_lenses=["event-study", "asset-pricing"],
                resource_activation_plan={
                    "levels": ["core", "subject_overlay", "subject_skill", "method_pack"]
                },
            )

            path = root / SUBJECT_GUIDANCE_REL
            text = path.read_text(encoding="utf-8")
            self.assertTrue(path.is_file())
            self.assertEqual(status["managed_block"], "active")
            self.assertEqual(status["active_subject"], "finance")
            self.assertEqual(status["subject_mode"], "confirmed")
            self.assertIn(START_MARKER, text)
            self.assertIn(END_MARKER, text)
            self.assertIn("active_subject: finance", text)
            self.assertIn("subject_mode: confirmed", text)
            self.assertIn("- event-study", text)
            self.assertIn("- subject_overlay: confirmed", text)

    def test_write_locked_subject_guidance_marks_replacement_protection(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            write_subject_guidance(
                root,
                active_subject="economics",
                subject_mode="locked",
                lifecycle_action="lock",
                source="mcp",
            )

            text = (root / SUBJECT_GUIDANCE_REL).read_text(encoding="utf-8")
            self.assertIn("active_subject: economics", text)
            self.assertIn("subject_mode: locked", text)
            self.assertIn("Do not automatically replace the active subject.", text)

    def test_disable_subject_guidance_writes_disabled_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )

            status = disable_subject_guidance(root, lifecycle_action="reset", source="cli")

            text = (root / SUBJECT_GUIDANCE_REL).read_text(encoding="utf-8")
            self.assertEqual(status["managed_block"], "disabled")
            self.assertEqual(status["active_subject"], "auto")
            self.assertEqual(status["subject_mode"], "auto")
            self.assertIn("status: disabled", text)
            self.assertIn("Use adaptive core inference for future runs.", text)

    def test_rewrite_preserves_user_text_outside_managed_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(
                "user prefix\n"
                f"{START_MARKER}\nold block\n{END_MARKER}\n"
                "user suffix\n",
                encoding="utf-8",
            )

            write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )

            text = path.read_text(encoding="utf-8")
            self.assertTrue(text.startswith("user prefix\n"))
            self.assertTrue(text.rstrip().endswith("user suffix"))
            self.assertNotIn("old block", text)
            self.assertEqual(text.count(START_MARKER), 1)
            self.assertEqual(text.count(END_MARKER), 1)

    def test_append_managed_block_when_user_file_has_no_markers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text("# User Subject Notes\n\n- Keep this note.\n", encoding="utf-8")

            status = write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )

            text = path.read_text(encoding="utf-8")
            self.assertEqual(status["managed_block"], "appended")
            self.assertIn("# User Subject Notes", text)
            self.assertIn(START_MARKER, text)

    def test_multiple_managed_blocks_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(
                f"{START_MARKER}\none\n{END_MARKER}\n{START_MARKER}\ntwo\n{END_MARKER}\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SubjectGuidanceError, "multiple managed blocks"):
                write_subject_guidance(
                    root,
                    active_subject="finance",
                    subject_mode="confirmed",
                    lifecycle_action="confirm",
                    source="cli",
                )

    def test_invalid_marker_order_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            path = root / SUBJECT_GUIDANCE_REL
            path.parent.mkdir(parents=True)
            path.write_text(f"{END_MARKER}\n{START_MARKER}\n", encoding="utf-8")

            status = inspect_subject_guidance(root)
            self.assertEqual(status["managed_block"], "invalid")
            with self.assertRaisesRegex(SubjectGuidanceError, "invalid marker order"):
                disable_subject_guidance(root, lifecycle_action="reset", source="cli")


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the new test to verify it fails**

Run:

```bash
uv run python -m unittest tests.test_subject_guidance
```

Expected: failure because `bridges.subject_guidance` does not exist.

- [ ] **Step 3: Implement the materializer module**

Create `packages/python-qiongli/src/qiongli/bridges/subject_guidance.py` with these public names:

```python
SUBJECT_GUIDANCE_REL = Path(".qiongli") / "guidance.d" / "subject-runtime.md"
START_MARKER = "<!-- qiongli:subject-runtime:start -->"
END_MARKER = "<!-- qiongli:subject-runtime:end -->"


class SubjectGuidanceError(ValueError):
    pass


def inspect_subject_guidance(project_root: Path) -> dict[str, Any]: ...


def write_subject_guidance(
    project_root: Path,
    *,
    active_subject: str,
    subject_mode: str,
    lifecycle_action: str,
    source: str,
    run_id: str | None = None,
    method_lenses: Sequence[str] | None = None,
    resource_activation_plan: Mapping[str, Any] | None = None,
) -> dict[str, Any]: ...


def disable_subject_guidance(
    project_root: Path,
    *,
    lifecycle_action: str,
    source: str,
    run_id: str | None = None,
) -> dict[str, Any]: ...
```

Implementation requirements:

- Normalize `project_root` with `Path(project_root).expanduser().resolve()`.
- `inspect_subject_guidance()` never creates files.
- Use UTC timestamps from `datetime.now(UTC).isoformat()`.
- Use ASCII text in generated guidance.
- Active block statuses:
  - `managed_block: "active"` for `subject_mode` `confirmed` or `locked`.
  - `managed_block: "disabled"` for disabled `auto` guidance.
  - `managed_block: "missing"` when file does not exist.
  - `managed_block: "absent"` when file exists without markers.
  - `managed_block: "invalid"` when marker counts or marker order are invalid.
- `_replace_managed_block(existing, block)` must:
  - create new file content when existing text is empty,
  - replace exactly one valid block,
  - append a block when no markers exist,
  - raise `SubjectGuidanceError("subject guidance contains multiple managed blocks")` for multiple starts or ends,
  - raise `SubjectGuidanceError("subject guidance has invalid marker order")` when the end marker precedes the start marker.
- Render active resource lines from `resource_activation_plan["levels"]` when present. If no levels are available, render `- core: active`.
- Render method lenses as `- none` when the list is empty.

- [ ] **Step 4: Run materializer tests**

Run:

```bash
uv run python -m unittest tests.test_subject_guidance
```

Expected: all tests pass.

- [ ] **Step 5: Commit Task 1**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_guidance.py tests/test_subject_guidance.py
git commit -m "feat(subjects): add subject guidance materializer"
```

Expected: commit succeeds.

## Task 2: Lifecycle Integration

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`
- Modify: `tests/test_subject_lifecycle.py`

- [ ] **Step 1: Add failing lifecycle tests**

Append tests to `tests/test_subject_lifecycle.py`:

```python
    def test_confirm_materializes_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = apply_subject_action(root, "confirm", "finance", source="cli", run_id="run-1")

            guidance = result["subject_guidance"]
            self.assertTrue((root / ".qiongli" / "guidance.d" / "subject-runtime.md").is_file())
            self.assertTrue(guidance["exists"])
            self.assertEqual(guidance["managed_block"], "active")
            self.assertEqual(guidance["active_subject"], "finance")
            self.assertEqual(guidance["subject_mode"], "confirmed")

    def test_lock_materializes_locked_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = apply_subject_action(root, "lock", "economics", source="mcp")

            guidance = result["subject_guidance"]
            self.assertEqual(guidance["active_subject"], "economics")
            self.assertEqual(guidance["subject_mode"], "locked")
            text = (root / ".qiongli" / "guidance.d" / "subject-runtime.md").read_text(encoding="utf-8")
            self.assertIn("Do not automatically replace the active subject.", text)

    def test_unlock_rewrites_locked_guidance_to_confirmed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            apply_subject_action(root, "lock", "finance")

            result = apply_subject_action(root, "unlock", source="cli", run_id="unlock-1")

            guidance = result["subject_guidance"]
            self.assertEqual(guidance["active_subject"], "finance")
            self.assertEqual(guidance["subject_mode"], "confirmed")
            text = (root / ".qiongli" / "guidance.d" / "subject-runtime.md").read_text(encoding="utf-8")
            self.assertIn("subject_mode: confirmed", text)

    def test_reset_disables_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            apply_subject_action(root, "confirm", "finance")

            result = apply_subject_action(root, "reset", source="cli")

            guidance = result["subject_guidance"]
            self.assertEqual(guidance["managed_block"], "disabled")
            self.assertEqual(guidance["active_subject"], "auto")
            self.assertEqual(guidance["subject_mode"], "auto")

    def test_dismiss_does_not_create_or_modify_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = apply_subject_action(root, "dismiss", "finance", source="cli")

            self.assertFalse((root / ".qiongli" / "guidance.d" / "subject-runtime.md").exists())
            self.assertEqual(result["subject_guidance"]["managed_block"], "missing")

    def test_status_reports_subject_guidance_without_creating_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            status = subject_status(root)

            self.assertEqual(status["subject_guidance"]["managed_block"], "missing")
            self.assertFalse((root / ".qiongli").exists())
```

- [ ] **Step 2: Run lifecycle tests to verify failure**

Run:

```bash
uv run python -m unittest tests.test_subject_lifecycle
```

Expected: failures because lifecycle packets do not include `subject_guidance` and lifecycle actions do not materialize the fragment.

- [ ] **Step 3: Integrate `subject_guidance` into lifecycle**

Patch `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`:

```python
from .subject_guidance import (
    SubjectGuidanceError,
    disable_subject_guidance,
    inspect_subject_guidance,
    write_subject_guidance,
)
```

Add `subject_guidance` to `_status_packet()`:

```python
return {
    "project_root": str(project_root.resolve()),
    "manifest": packet["manifest"],
    "manifest_exists": manifest_state.exists,
    "state": state,
    "subject_guidance": inspect_subject_guidance(project_root),
}
```

In `apply_subject_action()`:

- For `confirm`, after `update_project_manifest(...)`, call `write_subject_guidance(...)` with the concrete subject, `subject_mode="confirmed"`, action, source, run_id, and `manifest_state.manifest.method_lenses`.
- For `lock`, call `write_subject_guidance(...)` with `subject_mode="locked"`.
- For `unlock`, call `disable_subject_guidance(...)` when the active subject resolves to `auto`; otherwise call `write_subject_guidance(...)` with `subject_mode="confirmed"`.
- For `reset`, call `disable_subject_guidance(...)`.
- For `dismiss`, do not write subject guidance.
- Convert `SubjectGuidanceError` into `SubjectLifecycleError` with a message that starts with `Failed to update subject guidance:`.
- Append lifecycle events and write subject evidence memory only after the guidance update succeeds for write actions.

The action order should avoid reporting success when guidance writing fails:

```python
try:
    # validate and write manifest as currently done
    # write or disable subject guidance for confirm/lock/unlock/reset
except SubjectGuidanceError as exc:
    raise SubjectLifecycleError(f"Failed to update subject guidance: {exc}") from exc
```

- [ ] **Step 4: Run lifecycle tests**

Run:

```bash
uv run python -m unittest tests.test_subject_lifecycle tests.test_subject_guidance
```

Expected: all tests pass.

- [ ] **Step 5: Commit Task 2**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py tests/test_subject_lifecycle.py
git commit -m "feat(subjects): materialize guidance during lifecycle actions"
```

Expected: commit succeeds.

## Task 3: CLI And MCP Surface

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `tests/test_cli.py`
- Modify: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Add failing CLI tests**

In `tests/test_cli.py`, add tests near existing subject command tests:

```python
    def test_subject_confirm_json_reports_materialized_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            result = self.run_cli("subject", "confirm", "finance", "--cwd", tmp_dir, "--json")

            self.assertEqual(result.returncode, 0, result.stderr)
            payload = json.loads(result.stdout)
            self.assertTrue(payload["subject_guidance"]["exists"])
            self.assertEqual(payload["subject_guidance"]["managed_block"], "active")
            self.assertEqual(payload["subject_guidance"]["active_subject"], "finance")

    def test_subject_status_human_output_includes_subject_guidance_line(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            confirm = self.run_cli("subject", "confirm", "finance", "--cwd", tmp_dir)
            self.assertEqual(confirm.returncode, 0, confirm.stderr)

            result = self.run_cli("subject", "status", "--cwd", tmp_dir)

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("subject guidance: active (.qiongli/guidance.d/subject-runtime.md)", result.stdout)
```

If the local CLI helper is not named `self.run_cli`, use the existing helper in that test class and keep the assertions unchanged.

- [ ] **Step 2: Add failing MCP tests**

In `tests/test_mcp_tool_handlers.py`, add tests near existing subject lifecycle tests:

```python
    def test_subject_update_returns_materialized_guidance_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            result = call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "confirm", "subject": "finance"},
            )

            guidance = result["subject_guidance"]
            self.assertTrue(guidance["exists"])
            self.assertEqual(guidance["managed_block"], "active")
            self.assertEqual(guidance["active_subject"], "finance")

    def test_subject_status_returns_materialized_guidance_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            call_qiongli_tool(
                "qiongli_subject_update",
                {"cwd": str(root), "action": "lock", "subject": "economics"},
            )

            result = call_qiongli_tool("qiongli_subject_status", {"cwd": str(root)})

            guidance = result["subject_guidance"]
            self.assertEqual(guidance["active_subject"], "economics")
            self.assertEqual(guidance["subject_mode"], "locked")
```

- [ ] **Step 3: Run CLI/MCP tests to verify failure**

Run:

```bash
uv run python -m unittest tests.test_cli tests.test_mcp_tool_handlers
```

Expected: CLI human output test fails until `_print_subject_result()` prints subject guidance. MCP tests may already pass after Task 2; keep them as regression coverage.

- [ ] **Step 4: Update CLI human output and MCP descriptions**

Patch `_print_subject_result()` in `packages/python-qiongli/src/qiongli/cli.py`:

```python
    guidance = payload.get("subject_guidance", {})
    if isinstance(guidance, dict):
        block = str(guidance.get("managed_block") or "missing")
        path = str(guidance.get("path") or ".qiongli/guidance.d/subject-runtime.md")
        print(f"subject guidance: {block} ({path})")
        warnings = guidance.get("warnings", [])
        if isinstance(warnings, list) and warnings:
            print("subject guidance warnings: " + "; ".join(str(item) for item in warnings))
```

Patch MCP tool descriptions in `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`:

```python
"description": "Inspect adaptive subject state, project manifest, evidence memory, and managed subject guidance for a project."
```

and:

```python
"description": "Confirm, dismiss, reset, lock, or unlock adaptive subject guidance and managed project subject guidance."
```

- [ ] **Step 5: Run CLI/MCP tests**

Run:

```bash
uv run python -m unittest tests.test_cli tests.test_mcp_tool_handlers
```

Expected: all tests pass.

- [ ] **Step 6: Commit Task 3**

Run:

```bash
git add packages/python-qiongli/src/qiongli/cli.py packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_cli.py tests/test_mcp_tool_handlers.py
git commit -m "feat(subjects): expose materialized guidance status"
```

Expected: commit succeeds.

## Task 4: Guidance Runtime Trace Coverage

**Files:**
- Modify: `tests/test_guidance_runtime.py`

- [ ] **Step 1: Add failing guidance runtime test**

Append this test to `tests/test_guidance_runtime.py`:

```python
    def test_effective_guidance_reads_materialized_subject_runtime_fragment(self) -> None:
        from bridges.subject_guidance import write_subject_guidance

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
                method_lenses=["event-study"],
            )

            state = effective_guidance(root, mode="read")

            self.assertIn(".qiongli/guidance.d/subject-runtime.md", state.guidance_files_read)
            self.assertTrue(
                any(
                    source["path"] == ".qiongli/guidance.d/subject-runtime.md"
                    and source["kind"] == "project-fragment"
                    for source in state.guidance_sources
                )
            )
            self.assertIn("active_subject: finance", state.guidance_context)

    def test_trace_records_materialized_subject_guidance_source(self) -> None:
        from bridges.subject_guidance import write_subject_guidance

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            write_subject_guidance(
                root,
                active_subject="finance",
                subject_mode="confirmed",
                lifecycle_action="confirm",
                source="cli",
            )
            state = effective_guidance(root, mode="propose", run_id="subject-guidance-run")

            trace = write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={"task_id": "C1", "paper_type": "empirical", "topic": "returns"},
                draft_content="draft",
                review_content="review",
                merged_analysis="merged",
                validator_gate={"passed": True, "found": [], "missing": [], "checked": 0},
                applied=False,
            )

            self.assertIn(".qiongli/guidance.d/subject-runtime.md", trace["guidance_files_read"])
            self.assertTrue(
                any(
                    source["path"] == ".qiongli/guidance.d/subject-runtime.md"
                    for source in trace["guidance_sources"]
                )
            )
```

- [ ] **Step 2: Run guidance runtime tests**

Run:

```bash
uv run python -m unittest tests.test_guidance_runtime
```

Expected: tests may already pass because `effective_guidance()` reads fragments generically. If they fail, fix only the generic fragment loading or trace recording needed for these assertions.

- [ ] **Step 3: Commit Task 4**

Run:

```bash
git add tests/test_guidance_runtime.py
git commit -m "test(guidance): cover materialized subject fragments"
```

Expected: commit succeeds.

## Task 5: Runtime Smoke Coverage

**Files:**
- Modify: `tooling/scripts/run_subject_runtime_smoke.py`
- Create: `tests/fixtures/subject_runtime_smoke/confirmed_finance_guidance_loaded.json`
- Modify: `tests/test_subject_runtime_smoke.py`

- [ ] **Step 1: Add failing smoke fixture**

Create `tests/fixtures/subject_runtime_smoke/confirmed_finance_guidance_loaded.json`:

```json
{
  "name": "confirmed_finance_guidance_loaded",
  "manifest": null,
  "setup_subject_action": {
    "action": "confirm",
    "subject": "finance",
    "run_id": "setup-confirm-finance"
  },
  "args": {
    "task_id": "C1",
    "paper_type": "empirical",
    "topic": "earnings announcement stock market reaction",
    "context": "Use event-study evidence and Journal of Finance standards for this empirical paper.",
    "domain": "auto",
    "guidance_mode": "propose",
    "run_agents": false
  },
  "expected": {
    "decision": "confirm_subject",
    "primary_subject": "finance",
    "effective_domain": "finance",
    "guidance_source": ".qiongli/guidance.d/subject-runtime.md",
    "resource_levels": ["subject_overlay", "subject_skill", "method_pack"],
    "run_agents": false
  }
}
```

- [ ] **Step 2: Add failing smoke runner tests**

Patch `tests/test_subject_runtime_smoke.py`:

```python
        self.assertEqual(
            names,
            {
                "no_subject_core_only",
                "borrow_finance_lens",
                "suggest_finance_subject",
                "locked_economics_borrow_finance",
                "confirmed_finance_guidance_loaded",
            },
        )
```

Change the preview suite expected pass count from `4` to `5`.

Add this test:

```python
    def test_confirmed_finance_case_loads_materialized_subject_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            report = run_smoke_suite(
                fixture_dir=FIXTURE_DIR,
                workspace_root=Path(tmp_dir),
                mode="preview",
                selected_cases=["confirmed_finance_guidance_loaded"],
            )

        case = report["cases"][0]
        self.assertEqual(case["status"], "passed", case["failures"])
        guidance = case["result"]["data"]["task_packet"]["local_guidance"]
        self.assertIn(
            ".qiongli/guidance.d/subject-runtime.md",
            guidance["guidance_files_read"],
        )
```

- [ ] **Step 3: Run smoke tests to verify failure**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: failure because the runner does not yet process `setup_subject_action` or assert `guidance_source`.

- [ ] **Step 4: Extend smoke runner schema and assertions**

Patch `tooling/scripts/run_subject_runtime_smoke.py`:

```python
from bridges.mcp_tool_handlers import call_qiongli_tool
```

Extend `SmokeCase`:

```python
@dataclass(frozen=True)
class SmokeCase:
    name: str
    manifest: dict[str, Any] | None
    args: dict[str, Any]
    expected: dict[str, Any]
    source: Path
    setup_subject_action: dict[str, Any] | None = None
```

In `load_smoke_cases()` pass:

```python
setup_subject_action=(
    dict(payload["setup_subject_action"])
    if isinstance(payload.get("setup_subject_action"), dict)
    else None
),
```

In `run_smoke_case()`, after `_write_manifest(...)` and before building `args`, add:

```python
    if case.setup_subject_action:
        setup_args = dict(case.setup_subject_action)
        setup_args["cwd"] = str(project_root)
        setup_result = call_qiongli_tool("qiongli_subject_update", setup_args)
        if setup_result.get("isError"):
            return {
                "name": case.name,
                "source": _repo_relative(case.source),
                "project_root": str(project_root),
                "status": "failed",
                "failures": [f"setup_subject_action failed: {setup_result}"],
                "environment": {},
                "result": setup_result,
            }
```

In `_assert_case()`, after `task_packet` is normalized, add:

```python
    guidance_source = expected.get("guidance_source")
    if guidance_source is not None:
        local_guidance = task_packet.get("local_guidance", {})
        if not isinstance(local_guidance, dict):
            local_guidance = {}
        files_read = list(local_guidance.get("guidance_files_read", []) or [])
        if guidance_source not in files_read:
            failures.append(f"missing guidance source {guidance_source!r}")
```

- [ ] **Step 5: Run smoke tests and script**

Run:

```bash
uv run python -m unittest tests.test_subject_runtime_smoke
```

Expected: all smoke unit tests pass.

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --case confirmed_finance_guidance_loaded --json
```

Expected: summary reports `total: 1`, `passed: 1`, `failed: 0`, and `mode: preview`.

- [ ] **Step 6: Commit Task 5**

Run:

```bash
git add tooling/scripts/run_subject_runtime_smoke.py tests/fixtures/subject_runtime_smoke/confirmed_finance_guidance_loaded.json tests/test_subject_runtime_smoke.py
git commit -m "test(smoke): cover materialized subject guidance"
```

Expected: commit succeeds.

## Task 6: Docs And Verification

**Files:**
- Modify: `docs/advanced/subject-packaging-model.md`
- Modify: `docs/zh/advanced/subject-packaging-model.md`

- [ ] **Step 1: Update English subject packaging docs**

Patch `docs/advanced/subject-packaging-model.md` in the "User Model" section after the paragraph about implicit `active_subject: auto`:

```markdown
When a user or client confirms or locks a runtime subject, Qiongli writes a
managed project fragment at `.qiongli/guidance.d/subject-runtime.md`. Future
task runs read that fragment through the local guidance layer, so the installed
adaptive core workflow can keep using core guidance while applying the confirmed
subject layer for this project. `qiongli subject reset --cwd .` disables the
managed subject fragment and returns the project to adaptive core inference.
```

- [ ] **Step 2: Update Chinese subject packaging docs**

Patch `docs/zh/advanced/subject-packaging-model.md` in the matching user model section:

```markdown
当用户或客户端确认、锁定某个运行时学科后，Qiongli 会在
`.qiongli/guidance.d/subject-runtime.md` 写入一个受管理的项目级片段。之后的任务运行会通过
local guidance 读取这个片段，因此安装好的 adaptive core 工作流仍然以 core 为基础，同时为当前项目叠加已确认的学科层。
执行 `qiongli subject reset --cwd .` 会禁用这个受管理片段，并让项目回到 adaptive core 推断模式。
```

- [ ] **Step 3: Run focused verification**

Run:

```bash
uv run python -m unittest tests.test_subject_guidance tests.test_subject_lifecycle tests.test_guidance_runtime tests.test_mcp_tool_handlers tests.test_cli tests.test_subject_runtime_smoke
```

Expected: all selected tests pass.

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: exit code `0`, `threshold_failures: []`.

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
```

Expected: preview smoke summary reports zero failures.

Run:

```bash
git diff --check
```

Expected: no output and exit code `0`.

- [ ] **Step 4: Commit Task 6**

Run:

```bash
git add docs/advanced/subject-packaging-model.md docs/zh/advanced/subject-packaging-model.md
git commit -m "docs(subjects): document materialized subject guidance"
```

Expected: commit succeeds.

## Task 7: Final Review And Integration

**Files:**
- No new source files required unless review finds issues.

- [ ] **Step 1: Run final focused suite**

Run from `.worktrees/subject-guidance-materialization`:

```bash
uv run python -m unittest tests.test_subject_guidance tests.test_subject_lifecycle tests.test_guidance_runtime tests.test_mcp_tool_handlers tests.test_cli tests.test_subject_runtime_smoke tests.test_subject_refinement tests.test_subject_router_eval
```

Expected: all tests pass.

- [ ] **Step 2: Run router eval**

Run:

```bash
uv run python tooling/scripts/evaluate_subject_router.py --json
```

Expected: `threshold_failures` is an empty list.

- [ ] **Step 3: Run preview smoke**

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
```

Expected: `summary.failed` is `0`.

- [ ] **Step 4: Run final diff hygiene**

Run:

```bash
git diff --check
```

Expected: no output and exit code `0`.

- [ ] **Step 5: Final code review**

Dispatch a fresh review subagent with this scope:

```text
Review the entire feature branch against
docs/superpowers/specs/2026-07-01-subject-guidance-materialization-design.md.
Focus on managed block safety, lifecycle semantics, project-local write
boundaries, CLI/MCP parity, and smoke coverage. Report blockers first with
file/line references. Do not edit files.
```

Expected: reviewer says ready to merge or reports concrete blockers. Fix blockers before merging.

- [ ] **Step 6: Merge back to dev**

From repository root:

```bash
git switch dev
git merge --no-ff feature/subject-guidance-materialization
```

Expected: merge succeeds.

- [ ] **Step 7: Post-merge verification**

Run from repository root:

```bash
uv run python -m unittest tests.test_subject_guidance tests.test_subject_lifecycle tests.test_guidance_runtime tests.test_mcp_tool_handlers tests.test_cli tests.test_subject_runtime_smoke
```

Expected: all selected tests pass on `dev`.

Run:

```bash
uv run python tooling/scripts/run_subject_runtime_smoke.py --json
```

Expected: preview smoke summary reports zero failures on `dev`.

- [ ] **Step 8: Cleanup**

Run:

```bash
git worktree remove .worktrees/subject-guidance-materialization
git branch -d feature/subject-guidance-materialization
git worktree prune
```

Expected: feature worktree and branch are removed after merge.
