# Read-Only Subject Proposed Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let read-only marketplace, plugin, and client-native installs export subject lifecycle proposed actions without writing `.qiongli` project files.

**Architecture:** Add a shared `propose_subject_action` path in `bridges.subject_lifecycle` that validates the same actions as the write path, reads current status, and returns a structured `proposed_action` packet. Wire CLI `--propose-only` and MCP `read_only: true` to that shared function so CLI and MCP preserve parity while default lifecycle writes remain unchanged.

**Tech Stack:** Python standard library, existing Qiongli CLI/MCP handlers, `unittest`.

---

### Task 1: Add Failing Lifecycle Tests

**Files:**
- Modify: `tests/test_subject_lifecycle.py`

- [x] **Step 1: Write the proposed-action lifecycle test**

Add:

```python
from bridges.subject_lifecycle import propose_subject_action
```

Add this test:

```python
def test_propose_subject_action_exports_action_without_writing_project_files(self) -> None:
    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)

        result = propose_subject_action(
            root,
            "confirm",
            "finance",
            source="marketplace-readonly",
            run_id="run-1",
        )

        self.assertEqual(result["write_mode"], "proposed")
        self.assertFalse(self._manifest_path(root).exists())
        self.assertFalse(self._state_path(root).exists())
        self.assertFalse(self._guidance_path(root).exists())
        self.assertEqual(result["manifest"]["active_subject"], "auto")
        proposed = result["proposed_action"]
        self.assertEqual(proposed["action"], "confirm")
        self.assertEqual(proposed["subject"], "finance")
        self.assertEqual(proposed["source"], "marketplace-readonly")
        self.assertEqual(proposed["run_id"], "run-1")
        self.assertEqual(proposed["write_mode"], "proposed")
        self.assertIn(".qiongli/guidance_manifest.yaml", proposed["target_files"])
        self.assertIn(".qiongli/trace/subject_evidence.json", proposed["target_files"])
        self.assertIn("qiongli subject confirm finance", proposed["apply_command"])
```

- [x] **Step 2: Run the failing lifecycle test**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_lifecycle.SubjectLifecycleTests.test_propose_subject_action_exports_action_without_writing_project_files -q
```

Expected: FAIL because `propose_subject_action` does not exist yet.

### Task 2: Add Failing MCP And CLI Tests

**Files:**
- Modify: `tests/test_mcp_tool_handlers.py`
- Modify: `tests/test_cli.py`

- [x] **Step 1: Add MCP read-only test**

Add:

```python
def test_subject_update_read_only_exports_proposed_action_without_writing_manifest(self) -> None:
    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)

        result = call_qiongli_tool(
            "qiongli_subject_update",
            {
                "cwd": str(root),
                "action": "confirm",
                "subject": "finance",
                "read_only": True,
                "run_id": "run-1",
            },
        )

        payload = result["structuredContent"]
        self.assertFalse(result["isError"])
        self.assertEqual(payload["write_mode"], "proposed")
        self.assertFalse((root / ".qiongli" / "guidance_manifest.yaml").exists())
        self.assertEqual(payload["proposed_action"]["action"], "confirm")
        self.assertEqual(payload["proposed_action"]["subject"], "finance")
        self.assertEqual(payload["proposed_action"]["source"], "mcp")
        self.assertIn("qiongli subject confirm finance", payload["proposed_action"]["apply_command"])
```

Also assert the tool schema contains `read_only`.

- [x] **Step 2: Add CLI propose-only test**

Add:

```python
def test_subject_confirm_propose_only_json_exports_action_without_writing_manifest(self) -> None:
    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        stdout = io.StringIO()

        with mock.patch.object(
            cli_module.sys,
            "argv",
            [
                "qiongli",
                "subject",
                "confirm",
                "finance",
                "--cwd",
                str(root),
                "--propose-only",
                "--json",
            ],
        ), contextlib.redirect_stdout(stdout):
            exit_code = cli_module.main()

        payload = json.loads(stdout.getvalue())

    self.assertEqual(exit_code, 0)
    self.assertFalse((root / ".qiongli" / "guidance_manifest.yaml").exists())
    self.assertEqual(payload["write_mode"], "proposed")
    self.assertEqual(payload["proposed_action"]["action"], "confirm")
    self.assertEqual(payload["proposed_action"]["subject"], "finance")
    self.assertEqual(payload["proposed_action"]["source"], "cli")
```

- [x] **Step 3: Run the failing CLI/MCP tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_subject_update_read_only_exports_proposed_action_without_writing_manifest tests.test_cli.InstallerCliTests.test_subject_confirm_propose_only_json_exports_action_without_writing_manifest -q
```

Expected: FAIL because the tool schema, handler, and CLI parser do not support read-only proposed actions yet.

### Task 3: Implement Shared Proposed Action Packets

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py`

- [x] **Step 1: Add the public helper**

Implement:

```python
def propose_subject_action(
    project_root: Path,
    action: str,
    subject: str | None = None,
    *,
    source: str = "user",
    run_id: str | None = None,
) -> dict[str, Any]:
    root = _normalize_project_root(project_root)
    normalized_action = _validate_action(action)
    normalized_subject = _validate_subject_for_action(normalized_action, subject)
    manifest_state = load_project_manifest(root)
    state = _load_state(root)
    packet = _status_packet(root, manifest_state=manifest_state, state=state)
    packet["write_mode"] = "proposed"
    packet["proposed_action"] = _proposed_action_packet(
        root,
        action=normalized_action,
        subject=normalized_subject,
        source=source,
        run_id=run_id,
    )
    return packet
```

- [x] **Step 2: Add proposal packet helpers**

Add `_proposed_action_packet`, `_target_files_for_action`, and `_apply_command_for_action` so the packet contains `schema_version`, `action`, `subject`, `source`, `run_id`, `project_root`, `created_at`, `write_mode`, `target_files`, and `apply_command`.

- [x] **Step 3: Mark write path packets as applied**

Set `packet["write_mode"] = "applied"` before returning from `apply_subject_action`.

- [x] **Step 4: Run lifecycle test**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_lifecycle.SubjectLifecycleTests.test_propose_subject_action_exports_action_without_writing_project_files -q
```

Expected: PASS.

### Task 4: Wire MCP And CLI

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `packages/python-qiongli/src/qiongli/cli.py`

- [x] **Step 1: Add MCP schema field and handler routing**

Add `read_only` to `qiongli_subject_update` input schema. In `_tool_subject_update`, call `propose_subject_action` when `read_only` is truthy; otherwise call `apply_subject_action`.

- [x] **Step 2: Add CLI parser flags**

For subject update commands, add:

```python
subject_action.add_argument(
    "--propose-only",
    action="store_true",
    help="Return a proposed subject action without writing .qiongli project files",
)
```

- [x] **Step 3: Add CLI handler routing**

In `cmd_subject`, call `propose_subject_action` when `args.propose_only` is true.

- [x] **Step 4: Run CLI/MCP tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_subject_update_read_only_exports_proposed_action_without_writing_manifest tests.test_cli.InstallerCliTests.test_subject_confirm_propose_only_json_exports_action_without_writing_manifest -q
```

Expected: PASS.

### Task 5: Document Stage 6 Fallback Contract And Verify

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-06-read-only-subject-proposed-actions.md`

- [x] **Step 1: Update CLI docs**

Document `--propose-only` and MCP `read_only: true` under adaptive subject lifecycle controls. State that read-only clients can export the JSON packet and apply it later with the normal `qiongli subject ...` command in a writable project.

- [x] **Step 2: Update roadmap**

Mark Stage 6 as partially implemented for read-only proposed-action export through CLI/MCP, leaving release receipts and full marketplace/install packaging checks as follow-up.

- [x] **Step 3: Mark this plan complete**

Change checklist items in this plan to `[x]` after verification passes.

- [x] **Step 4: Run focused verification**

Run:

```bash
.venv/bin/python -m unittest tests.test_subject_lifecycle tests.test_mcp_tool_handlers tests.test_cli -q
git diff --check
```

Expected: tests pass and no whitespace errors.

- [x] **Step 5: Commit implementation and docs separately**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_lifecycle.py packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py packages/python-qiongli/src/qiongli/cli.py tests/test_subject_lifecycle.py tests/test_mcp_tool_handlers.py tests/test_cli.py
git commit -m "feat(subjects): export read-only lifecycle proposals" -m "Add CLI and MCP proposed-action fallback packets for clients that cannot write project-local .qiongli files."
git add docs/reference/cli.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md docs/superpowers/plans/2026-07-06-read-only-subject-proposed-actions.md
git commit -m "docs(roadmap): record read-only subject action fallback" -m "Document the Stage 6 CLI/MCP proposed-action fallback and remaining marketplace release-readiness work."
```
