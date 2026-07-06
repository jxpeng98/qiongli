# Project Subject Guidance Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users install Qiongli once, use it with zero project configuration, and optionally let each project select or gradually learn its active subject, venue, method lenses, and local guidance through auditable project-local files.

**Architecture:** Keep global installation as the stable core runtime. Add a structured project manifest under `.qiongli/guidance_manifest.yaml` for machine-readable project state, keep `.qiongli/local_guidance.md` and `.qiongli/guidance.d/*.md` for human-readable rules, and keep `.qiongli/trace/` as the evidence trail. Missing manifest means `active_subject: auto`; task runs may infer temporary subject/method lenses, but persistent changes are written only through explicit proposals or `apply` mode.

**Tech Stack:** Python 3.12, `dataclasses`, `PyYAML`, existing `bridges.guidance_runtime`, existing `bridges.orchestrator`, existing `qiongli.cli`, existing MCP tool handler schema, `unittest`.

---

## Product Contract

The user-facing model is:

```text
qiongli install --target all
cd /path/to/research-project
qiongli project init
qiongli project set-subject finance
qiongli project status
```

Zero-config usage must also work:

```text
no .qiongli/guidance_manifest.yaml
=> implicit manifest: active_subject=auto
=> task-run infers temporary subject/method lens from the current request
=> task-run writes a guidance update proposal instead of silently changing project state
```

Persistent project state lives in:

```text
.qiongli/
  guidance_manifest.yaml
  local_guidance.md
  guidance.d/
  trace/
```

Precedence from strongest to weakest:

```text
canonical workflow contracts and safety constraints
> current task/user instruction
> configured project manifest subject layer
> temporary inferred subject layer
> project local guidance
> user global preferences
```

When any local or inferred guidance conflicts with task contracts, required outputs, evidence gates, quality gates, MCP evidence requirements, or safety constraints, the canonical requirement wins and the conflict is recorded in the task packet and trace.

## File Structure

- Create `packages/python-qiongli/src/qiongli/bridges/project_manifest.py`
  - Owns structured project manifest parsing, validation, defaults, updates, and serialization.
  - Exposes `ProjectManifest`, `ProjectManifestState`, `load_project_manifest`, `init_project_manifest`, `update_project_manifest`, and `manifest_to_guidance_section`.
- Modify `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
  - Include manifest state in `GuidanceState`.
  - Read manifest before local guidance fragments.
  - Add manifest path/status to bootstrap, trace, list, lint, and proposal apply flows.
- Create `packages/python-qiongli/src/qiongli/bridges/project_inference.py`
  - Provides deterministic first-pass subject/method inference from task packet and run text.
  - Produces conservative manifest update suggestions with confidence and evidence snippets.
- Create `packages/python-qiongli/src/qiongli/bridges/subject_runtime.py`
  - Maps active subject to runtime domain/venue defaults and compact official subject context.
  - Does not mutate installed skill packages.
- Modify `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
  - Inject project manifest and effective subject context into task packets and draft/review prompts.
  - Add `project` command handlers and route `guidance apply` through structured proposal application.
- Modify `packages/python-qiongli/src/qiongli/cli.py`
  - Add top-level `qiongli project` commands and keep `qiongli guidance` compatibility.
- Modify `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
  - Add project status/config tools or include project manifest status in `qiongli_task_run` preview.
- Modify `content/workflow/SKILL.md`
  - Teach skill-only usage to read `.qiongli/guidance_manifest.yaml` when present.
- Modify docs:
  - `README.md`
  - `docs/reference/cli.md`
  - `docs/zh/reference/cli.md`
  - `docs/advanced/subject-packaging-model.md`
- Add tests:
  - `tests/test_project_manifest.py`
  - `tests/test_project_inference.py`
  - Extend `tests/test_guidance_runtime.py`
  - Extend `tests/test_orchestrator_workflows.py`
  - Extend `tests/test_mcp_tool_handlers.py`
  - Extend `tests/test_cli.py`

---

### Task 1: Add Structured Project Manifest Runtime

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/project_manifest.py`
- Create: `tests/test_project_manifest.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`

- [ ] **Step 1: Write failing manifest tests**

Add `tests/test_project_manifest.py`:

```python
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.project_manifest import (
    ProjectManifestError,
    init_project_manifest,
    load_project_manifest,
    update_project_manifest,
)


class ProjectManifestTests(unittest.TestCase):
    def test_missing_manifest_returns_implicit_auto_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            state = load_project_manifest(Path(tmp_dir))

        self.assertFalse(state.exists)
        self.assertEqual(state.manifest.active_subject, "auto")
        self.assertEqual(state.manifest.strictness, "standard")
        self.assertEqual(state.manifest.secondary_subjects, [])
        self.assertEqual(state.manifest.venue_profiles, [])
        self.assertEqual(state.manifest.method_lenses, [])

    def test_init_project_manifest_writes_default_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            state = init_project_manifest(root)

            self.assertTrue(state.path.is_file())
            text = state.path.read_text(encoding="utf-8")
            self.assertIn("active_subject: auto", text)
            self.assertIn("strictness: standard", text)

    def test_update_project_manifest_sets_subject_venue_and_methods(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_manifest(root)

            state = update_project_manifest(
                root,
                active_subject="finance",
                venue_profiles=["journal-of-finance"],
                method_lenses=["asset-pricing", "event-study"],
                strictness="high",
            )

            self.assertEqual(state.manifest.active_subject, "finance")
            self.assertEqual(state.manifest.venue_profiles, ["journal-of-finance"])
            self.assertEqual(state.manifest.method_lenses, ["asset-pricing", "event-study"])
            self.assertEqual(state.manifest.strictness, "high")

    def test_invalid_subject_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: unknown-field\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ProjectManifestError, "Unsupported active_subject"):
                load_project_manifest(root)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the failing test**

Run:

```bash
python3 -m unittest tests.test_project_manifest -v
```

Expected: FAIL with `ModuleNotFoundError: No module named 'bridges.project_manifest'`.

- [ ] **Step 3: Implement `project_manifest.py`**

Create `packages/python-qiongli/src/qiongli/bridges/project_manifest.py`:

```python
from __future__ import annotations

from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import yaml


OFFICIAL_SUBJECTS = (
    "auto",
    "core",
    "economics",
    "accounting",
    "business",
    "finance",
    "political-economy",
    "geoeconomics",
    "economics-accounting",
)
STRICTNESS_CHOICES = ("standard", "high")
MANIFEST_REL = Path(".qiongli") / "guidance_manifest.yaml"


class ProjectManifestError(ValueError):
    """Raised when project guidance manifest metadata is invalid."""


@dataclass(frozen=True)
class ProjectManifest:
    active_subject: str = "auto"
    secondary_subjects: list[str] | None = None
    venue_profiles: list[str] | None = None
    method_lenses: list[str] | None = None
    strictness: str = "standard"

    def normalized(self) -> "ProjectManifest":
        active_subject = _normalize_subject(self.active_subject, "active_subject")
        secondary_subjects = [_normalize_subject(item, "secondary_subjects") for item in self.secondary_subjects or []]
        venue_profiles = _normalize_string_list(self.venue_profiles or [], "venue_profiles")
        method_lenses = _normalize_string_list(self.method_lenses or [], "method_lenses")
        strictness = str(self.strictness or "standard").strip().lower()
        if strictness not in STRICTNESS_CHOICES:
            raise ProjectManifestError(
                f"Unsupported strictness '{self.strictness}'. Available: {', '.join(STRICTNESS_CHOICES)}"
            )
        return ProjectManifest(
            active_subject=active_subject,
            secondary_subjects=secondary_subjects,
            venue_profiles=venue_profiles,
            method_lenses=method_lenses,
            strictness=strictness,
        )

    def to_dict(self) -> dict[str, Any]:
        normalized = self.normalized()
        return {
            "active_subject": normalized.active_subject,
            "secondary_subjects": list(normalized.secondary_subjects or []),
            "venue_profiles": list(normalized.venue_profiles or []),
            "method_lenses": list(normalized.method_lenses or []),
            "strictness": normalized.strictness,
        }


@dataclass(frozen=True)
class ProjectManifestState:
    exists: bool
    path: Path
    project_root: Path
    manifest: ProjectManifest
    warnings: list[str]

    def to_packet(self) -> dict[str, Any]:
        return {
            "exists": self.exists,
            "path": _rel(self.project_root, self.path),
            "manifest": self.manifest.to_dict(),
            "warnings": list(self.warnings),
        }


def load_project_manifest(project_root: Path) -> ProjectManifestState:
    root = Path(project_root).expanduser().resolve()
    path = root / MANIFEST_REL
    if not path.is_file():
        return ProjectManifestState(
            exists=False,
            path=path,
            project_root=root,
            manifest=ProjectManifest().normalized(),
            warnings=[],
        )
    try:
        raw = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except yaml.YAMLError as exc:
        raise ProjectManifestError(f"Malformed project guidance manifest {path}: {exc}") from exc
    if not isinstance(raw, dict):
        raise ProjectManifestError(f"Project guidance manifest must be a YAML object: {path}")
    manifest = ProjectManifest(
        active_subject=str(raw.get("active_subject", "auto")),
        secondary_subjects=_raw_list(raw.get("secondary_subjects"), "secondary_subjects"),
        venue_profiles=_raw_list(raw.get("venue_profiles"), "venue_profiles"),
        method_lenses=_raw_list(raw.get("method_lenses"), "method_lenses"),
        strictness=str(raw.get("strictness", "standard")),
    ).normalized()
    return ProjectManifestState(exists=True, path=path, project_root=root, manifest=manifest, warnings=[])


def init_project_manifest(project_root: Path, *, overwrite: bool = False) -> ProjectManifestState:
    root = Path(project_root).expanduser().resolve()
    path = root / MANIFEST_REL
    if path.exists() and not overwrite:
        return load_project_manifest(root)
    path.parent.mkdir(parents=True, exist_ok=True)
    manifest = ProjectManifest().normalized()
    path.write_text(_render_manifest(manifest), encoding="utf-8")
    return load_project_manifest(root)


def update_project_manifest(
    project_root: Path,
    *,
    active_subject: str | None = None,
    secondary_subjects: list[str] | None = None,
    venue_profiles: list[str] | None = None,
    method_lenses: list[str] | None = None,
    strictness: str | None = None,
) -> ProjectManifestState:
    state = init_project_manifest(project_root)
    current = state.manifest
    next_manifest = ProjectManifest(
        active_subject=active_subject if active_subject is not None else current.active_subject,
        secondary_subjects=secondary_subjects if secondary_subjects is not None else list(current.secondary_subjects or []),
        venue_profiles=venue_profiles if venue_profiles is not None else list(current.venue_profiles or []),
        method_lenses=method_lenses if method_lenses is not None else list(current.method_lenses or []),
        strictness=strictness if strictness is not None else current.strictness,
    ).normalized()
    state.path.write_text(_render_manifest(next_manifest), encoding="utf-8")
    return load_project_manifest(state.project_root)


def manifest_to_guidance_section(state: ProjectManifestState) -> str:
    data = state.manifest.to_dict()
    origin = "configured" if state.exists else "implicit"
    lines = [
        f"Project manifest ({origin}):",
        f"- active_subject: {data['active_subject']}",
        f"- secondary_subjects: {', '.join(data['secondary_subjects']) or 'none'}",
        f"- venue_profiles: {', '.join(data['venue_profiles']) or 'none'}",
        f"- method_lenses: {', '.join(data['method_lenses']) or 'none'}",
        f"- strictness: {data['strictness']}",
    ]
    return "\n".join(lines)


def _render_manifest(manifest: ProjectManifest) -> str:
    payload = manifest.to_dict()
    return yaml.safe_dump(payload, allow_unicode=True, sort_keys=False)


def _normalize_subject(value: str, field: str) -> str:
    normalized = str(value or "auto").strip().lower()
    if normalized not in OFFICIAL_SUBJECTS:
        raise ProjectManifestError(
            f"Unsupported {field} '{value}'. Available: {', '.join(OFFICIAL_SUBJECTS)}"
        )
    return normalized


def _normalize_string_list(values: list[str], field: str) -> list[str]:
    result: list[str] = []
    for value in values:
        normalized = str(value).strip().lower()
        if not normalized:
            continue
        if normalized not in result:
            result.append(normalized)
    return result


def _raw_list(value: object, field: str) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        raise ProjectManifestError(f"{field} must be a list")
    return [str(item) for item in value]


def _rel(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return str(path)
```

- [ ] **Step 4: Run manifest tests**

Run:

```bash
python3 -m unittest tests.test_project_manifest -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/project_manifest.py tests/test_project_manifest.py
git commit -m "feat(guidance): add project manifest runtime"
```

---

### Task 2: Wire Manifest Into Guidance State And Trace

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- Modify: `tests/test_guidance_runtime.py`

- [ ] **Step 1: Add failing guidance runtime tests**

Append to `GuidanceRuntimeTests`:

```python
    def test_effective_guidance_includes_implicit_project_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            state = effective_guidance(root, mode="read")

            self.assertEqual(state.project_manifest["manifest"]["active_subject"], "auto")
            self.assertFalse(state.project_manifest["exists"])
            self.assertIn("Project manifest", state.guidance_context)

    def test_init_project_guidance_creates_manifest_with_auto_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)

            paths = init_project_guidance(root)

            self.assertTrue(paths.project_guidance_manifest.is_file())
            self.assertIn("active_subject: auto", paths.project_guidance_manifest.read_text(encoding="utf-8"))

    def test_guidance_trace_records_project_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            state = effective_guidance(root, mode="propose", run_id="manifest-run")

            trace = write_guidance_trace(
                project_root=root,
                guidance_state=state,
                task_packet={"task_id": "F3", "paper_type": "empirical", "topic": "ai-writing"},
                draft_content="draft",
                review_content="review",
                merged_analysis="merged",
                validator_gate={"passed": True, "found": [], "missing": [], "checked": 0},
                applied=False,
            )

            self.assertEqual(trace["project_manifest"]["manifest"]["active_subject"], "auto")
            self.assertTrue((root / ".qiongli" / "trace" / "runs" / "manifest-run" / "project_manifest.json").is_file())
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
python3 -m unittest tests.test_guidance_runtime.GuidanceRuntimeTests.test_effective_guidance_includes_implicit_project_manifest tests.test_guidance_runtime.GuidanceRuntimeTests.test_init_project_guidance_creates_manifest_with_auto_subject tests.test_guidance_runtime.GuidanceRuntimeTests.test_guidance_trace_records_project_manifest -v
```

Expected: FAIL because `GuidanceState` has no `project_manifest`.

- [ ] **Step 3: Extend `GuidanceState` and initialization**

In `guidance_runtime.py`:

```python
from .project_manifest import (
    init_project_manifest,
    load_project_manifest,
    manifest_to_guidance_section,
)
```

Add a field to `GuidanceState`:

```python
    project_manifest: dict[str, Any] | None = None
```

Update `to_packet()`:

```python
        payload["project_manifest"] = dict(self.project_manifest or {})
```

Inside `init_project_guidance()` after directories are created:

```python
    init_project_manifest(paths.project_root)
```

Inside `effective_guidance()` before source files are read:

```python
    manifest_state = load_project_manifest(paths.project_root)
    sections.append("## Project Manifest\n\n" + manifest_to_guidance_section(manifest_state))
    files_read.append(_rel(paths.project_root, manifest_state.path) if manifest_state.exists else "<implicit-project-manifest>")
    guidance_sources.append({
        "kind": "project-manifest",
        "path": files_read[-1],
        "label": "Project Manifest",
    })
    source_order.append("project-manifest")
```

Set `project_manifest=manifest_state.to_packet()` in every `GuidanceState` return. For `mode="off"`, use:

```python
            project_manifest={},
```

- [ ] **Step 4: Write manifest trace bundle**

Inside `write_guidance_trace()` after `task_packet.json`:

```python
    _write_json(run_dir / "project_manifest.json", guidance_state.project_manifest or {})
```

Add to `record`:

```python
        "project_manifest": dict(guidance_state.project_manifest or {}),
```

- [ ] **Step 5: Run guidance runtime tests**

Run:

```bash
python3 -m unittest tests.test_guidance_runtime -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py tests/test_guidance_runtime.py
git commit -m "feat(guidance): include project manifest in guidance state"
```

---

### Task 3: Add Project CLI Commands

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `tests/test_cli.py`

- [ ] **Step 1: Add failing CLI delegation tests**

Add to `tests/test_cli.py` near existing guidance tests:

```python
    def test_project_set_subject_delegates_to_orchestrator(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            completed = mock.Mock(returncode=0, stdout="project set-subject ok\n")
            args = argparse.Namespace(project_cmd="set-subject", project_dir=str(project_dir), subject="finance")

            with mock.patch("subprocess.run", return_value=completed) as run:
                exit_code = cli_module.cmd_project(args)

        self.assertEqual(exit_code, 0)
        command = run.call_args.args[0]
        self.assertEqual(
            command[-5:],
            ["set-subject", "--project-dir", str(project_dir.resolve()), "--subject", "finance"],
        )

    def test_project_status_delegates_to_orchestrator(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            completed = mock.Mock(returncode=0, stdout="project status ok\n")
            args = argparse.Namespace(project_cmd="status", project_dir=tmp_dir)

            with mock.patch("subprocess.run", return_value=completed) as run:
                exit_code = cli_module.cmd_project(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("project", run.call_args.args[0])
        self.assertIn("status", run.call_args.args[0])
```

- [ ] **Step 2: Run failing CLI tests**

Run:

```bash
python3 -m unittest tests.test_cli.CliTests.test_project_set_subject_delegates_to_orchestrator tests.test_cli.CliTests.test_project_status_delegates_to_orchestrator -v
```

Expected: FAIL because `cmd_project` is missing.

- [ ] **Step 3: Add CLI dispatcher**

In `cli.py`, add:

```python
def _run_orchestrator_project(args: argparse.Namespace) -> int:
    env = os.environ.copy()
    repo_root = _find_repo_root(Path.cwd())
    if repo_root is not None:
        existing_pythonpath = env.get("PYTHONPATH", "")
        layout = RepoLayout(repo_root)
        import_roots = (layout.python_source_root, repo_root)
        env["PYTHONPATH"] = os.pathsep.join(
            [*(str(root) for root in import_roots), *([existing_pythonpath] if existing_pythonpath else [])]
        )

    command = [
        sys.executable,
        "-m",
        "bridges.orchestrator",
        "project",
        str(args.project_cmd),
        "--project-dir",
        str(Path(args.project_dir).expanduser().resolve()),
    ]
    if args.project_cmd == "set-subject":
        command.extend(["--subject", str(args.subject)])
    if args.project_cmd == "set-venue":
        command.extend(["--venue", str(args.venue)])
    if args.project_cmd == "set-method-lens":
        command.extend(["--method-lens", str(args.method_lens)])

    result = subprocess.run(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
        env=env,
    )
    if result.stdout:
        print(result.stdout.rstrip())
    return result.returncode


def cmd_project(args: argparse.Namespace) -> int:
    return _run_orchestrator_project(args)
```

Register a top-level `project` parser:

```python
    project = subparsers.add_parser("project", help="Manage project-local Qiongli subject and guidance state")
    project_subparsers = project.add_subparsers(dest="project_cmd", required=True)
    project_init = project_subparsers.add_parser("init", help="Create .qiongli project state")
    project_init.add_argument("--project-dir", default=str(Path.cwd()), help="Project directory")
    project_status = project_subparsers.add_parser("status", help="Show effective project state")
    project_status.add_argument("--project-dir", default=str(Path.cwd()), help="Project directory")
    project_set_subject = project_subparsers.add_parser("set-subject", help="Set active project subject")
    project_set_subject.add_argument("--project-dir", default=str(Path.cwd()), help="Project directory")
    project_set_subject.add_argument("subject", nargs="?", help="Subject, e.g. finance")
    project_set_subject.add_argument("--subject", dest="subject_flag", help="Subject, e.g. finance")
    project_set_venue = project_subparsers.add_parser("set-venue", help="Set a single active project venue profile")
    project_set_venue.add_argument("--project-dir", default=str(Path.cwd()), help="Project directory")
    project_set_venue.add_argument("venue", nargs="?", help="Venue profile, e.g. journal-of-finance")
    project_set_venue.add_argument("--venue", dest="venue_flag", help="Venue profile, e.g. journal-of-finance")
    project_set_method = project_subparsers.add_parser("set-method-lens", help="Set a single active project method lens")
    project_set_method.add_argument("--project-dir", default=str(Path.cwd()), help="Project directory")
    project_set_method.add_argument("method_lens", nargs="?", help="Method lens, e.g. did")
    project_set_method.add_argument("--method-lens", dest="method_lens_flag", help="Method lens, e.g. did")
```

In main command routing:

```python
    if args.cmd == "project":
        if getattr(args, "subject_flag", None):
            args.subject = args.subject_flag
        if getattr(args, "venue_flag", None):
            args.venue = args.venue_flag
        if getattr(args, "method_lens_flag", None):
            args.method_lens = args.method_lens_flag
        return cmd_project(args)
```

- [ ] **Step 4: Add orchestrator project handler**

In `orchestrator.py`, import:

```python
from .project_manifest import init_project_manifest, load_project_manifest, update_project_manifest
```

Add:

```python
def _run_project_command(args: argparse.Namespace) -> CollaborationResult:
    project_dir = Path(getattr(args, "project_dir", Path.cwd())).expanduser().resolve()
    action = str(getattr(args, "project_cmd", "") or "").strip()
    if action == "init":
        init_project_guidance(project_dir)
        state = init_project_manifest(project_dir)
        data = {"action": "init", "project_dir": str(project_dir), "project_manifest": state.to_packet()}
    elif action == "status":
        guidance = effective_guidance(project_dir, mode="read")
        data = {
            "action": "status",
            "project_dir": str(project_dir),
            "project_manifest": guidance.project_manifest,
            "guidance_files": list(guidance.guidance_files_read),
        }
    elif action == "set-subject":
        subject = str(getattr(args, "subject", "") or "").strip()
        state = update_project_manifest(project_dir, active_subject=subject)
        data = {"action": "set-subject", "project_dir": str(project_dir), "project_manifest": state.to_packet()}
    elif action == "set-venue":
        venue = str(getattr(args, "venue", "") or "").strip()
        current = load_project_manifest(project_dir).manifest
        state = update_project_manifest(project_dir, venue_profiles=[venue], method_lenses=list(current.method_lenses or []))
        data = {"action": "set-venue", "project_dir": str(project_dir), "project_manifest": state.to_packet()}
    elif action == "set-method-lens":
        method_lens = str(getattr(args, "method_lens", "") or "").strip()
        current = load_project_manifest(project_dir).manifest
        state = update_project_manifest(project_dir, venue_profiles=list(current.venue_profiles or []), method_lenses=[method_lens])
        data = {"action": "set-method-lens", "project_dir": str(project_dir), "project_manifest": state.to_packet()}
    else:
        raise ValueError(f"Unhandled project command: {action}")
    return CollaborationResult(
        mode="project",
        task_description=f"project {action}".strip(),
        merged_analysis=json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True),
        confidence=1.0,
        recommendations=[],
        data=data,
    )
```

Register orchestrator parser `project` with subcommands matching CLI, and route:

```python
    if args.mode == "project":
        result = _run_project_command(args)
```

- [ ] **Step 5: Run CLI tests**

Run:

```bash
python3 -m unittest tests.test_cli -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py packages/python-qiongli/src/qiongli/cli.py tests/test_cli.py
git commit -m "feat(cli): add project guidance commands"
```

---

### Task 4: Resolve Effective Subject And Inject It Into Task Runs

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/subject_runtime.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `tests/test_orchestrator_workflows.py`

- [ ] **Step 1: Add failing task-run tests**

Append to `OrchestratorWorkflowTests`:

```python
    def test_task_run_uses_project_manifest_subject_when_domain_is_auto(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: finance\nmethod_lenses:\n  - event-study\nstrictness: high\n",
                encoding="utf-8",
            )
            orchestrator = MockOrchestrator()

            result = orchestrator.task_run(
                task_id="F3",
                paper_type="empirical",
                topic="earnings-announcement",
                cwd=root,
                guidance_mode="read",
                domain="auto",
                skip_validation=True,
            )

            packet = result.data["task_packet"]
            self.assertEqual(packet["project_subject"]["effective_subject"], "finance")
            self.assertEqual(packet["domain"], "finance")
            self.assertIn("event-study", packet["project_subject"]["method_lenses"])
            draft_prompt = next(call["prompt"] for call in orchestrator.runtime_calls if call["agent"])
            self.assertIn("Project subject context", draft_prompt)
            self.assertIn("finance", draft_prompt)

    def test_task_run_domain_argument_overrides_project_subject_for_current_run(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text(
                "active_subject: finance\n",
                encoding="utf-8",
            )
            orchestrator = MockOrchestrator()

            result = orchestrator.task_run(
                task_id="F3",
                paper_type="empirical",
                topic="minimum-wage",
                cwd=root,
                guidance_mode="read",
                domain="economics",
                skip_validation=True,
            )

            packet = result.data["task_packet"]
            self.assertEqual(packet["project_subject"]["effective_subject"], "finance")
            self.assertEqual(packet["domain"], "economics")
            self.assertEqual(packet["project_subject"]["domain_source"], "task-argument")
```

- [ ] **Step 2: Run failing task-run tests**

Run:

```bash
python3 -m unittest tests.test_orchestrator_workflows.OrchestratorWorkflowTests.test_task_run_uses_project_manifest_subject_when_domain_is_auto tests.test_orchestrator_workflows.OrchestratorWorkflowTests.test_task_run_domain_argument_overrides_project_subject_for_current_run -v
```

Expected: FAIL because `project_subject` is missing.

- [ ] **Step 3: Add subject runtime resolver**

Create `subject_runtime.py`:

```python
from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .project_manifest import ProjectManifestState


SUBJECT_TO_DOMAIN = {
    "auto": "auto",
    "core": "auto",
    "economics": "economics",
    "accounting": "accounting",
    "business": "business-management",
    "finance": "finance",
    "political-economy": "political-economy",
    "geoeconomics": "geoeconomics",
    "economics-accounting": "economics",
}


@dataclass(frozen=True)
class ProjectSubjectState:
    effective_subject: str
    domain: str
    domain_source: str
    venue_profiles: list[str]
    method_lenses: list[str]
    strictness: str
    summary: str

    def to_packet(self) -> dict[str, Any]:
        return {
            "effective_subject": self.effective_subject,
            "domain": self.domain,
            "domain_source": self.domain_source,
            "venue_profiles": list(self.venue_profiles),
            "method_lenses": list(self.method_lenses),
            "strictness": self.strictness,
            "summary": self.summary,
        }


def resolve_project_subject(
    manifest_state: ProjectManifestState,
    *,
    requested_domain: str | None,
) -> ProjectSubjectState:
    manifest = manifest_state.manifest
    subject = manifest.active_subject
    requested = str(requested_domain or "auto").strip().lower() or "auto"
    if requested != "auto":
        domain = requested
        domain_source = "task-argument"
    else:
        domain = SUBJECT_TO_DOMAIN.get(subject, "auto")
        domain_source = "project-manifest" if manifest_state.exists and subject not in {"auto", "core"} else "auto"
    summary = (
        f"Project subject context: effective_subject={subject}; "
        f"domain={domain}; domain_source={domain_source}; "
        f"venue_profiles={', '.join(manifest.venue_profiles or []) or 'none'}; "
        f"method_lenses={', '.join(manifest.method_lenses or []) or 'none'}; "
        f"strictness={manifest.strictness}."
    )
    return ProjectSubjectState(
        effective_subject=subject,
        domain=domain,
        domain_source=domain_source,
        venue_profiles=list(manifest.venue_profiles or []),
        method_lenses=list(manifest.method_lenses or []),
        strictness=manifest.strictness,
        summary=summary,
    )
```

- [ ] **Step 4: Inject project subject before domain profile loading**

In `ModelOrchestrator.task_run()`, after `guidance_state = effective_guidance(...)` and before `_load_domain_profile_context(domain)`, add:

```python
        manifest_packet = guidance_state.project_manifest or {}
        manifest_state = load_project_manifest(cwd)
        project_subject = resolve_project_subject(manifest_state, requested_domain=domain)
        effective_domain = project_subject.domain if str(domain or "auto").strip().lower() == "auto" else domain
        domain_context = self._load_domain_profile_context(effective_domain)
```

Then after packet creation:

```python
        packet["project_subject"] = project_subject.to_packet()
        packet["domain"] = str(effective_domain or "auto")
```

Append routing note:

```python
        routing_notes.append(project_subject.summary)
```

Add prompt section in `_build_task_draft_prompt()` and `_build_task_review_prompt()`:

```python
        project_subject = task_packet.get("project_subject", {})
        project_subject_section = ""
        if isinstance(project_subject, dict) and project_subject:
            project_subject_section = "\nProject subject context:\n" + str(project_subject.get("summary", "")).strip() + "\n"
```

Include `{project_subject_section}` before local guidance context.

- [ ] **Step 5: Run orchestrator workflow tests**

Run:

```bash
python3 -m unittest tests.test_orchestrator_workflows -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/subject_runtime.py packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_orchestrator_workflows.py
git commit -m "feat(orchestrator): route task runs through project subject state"
```

---

### Task 5: Add Deterministic Subject Inference And Conservative Proposals

**Files:**
- Create: `packages/python-qiongli/src/qiongli/bridges/project_inference.py`
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- Modify: `tests/test_project_inference.py`
- Modify: `tests/test_guidance_runtime.py`

- [ ] **Step 1: Add inference tests**

Create `tests/test_project_inference.py`:

```python
from __future__ import annotations

import unittest

from bridges.project_inference import infer_project_manifest_suggestion


class ProjectInferenceTests(unittest.TestCase):
    def test_detects_finance_event_study(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {
                "topic": "earnings announcement returns",
                "context": "event study abnormal returns factor exposure",
            },
            draft_content="Use an event window and check leakage.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "finance")
        self.assertIn("event-study", suggestion["method_lenses"])
        self.assertGreaterEqual(suggestion["confidence"], 0.6)

    def test_detects_economics_did(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {"topic": "minimum wage DID", "context": "parallel trends causal identification"},
            draft_content="Difference-in-differences design needs pre-trends.",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "economics")
        self.assertIn("did", suggestion["method_lenses"])

    def test_returns_auto_when_evidence_is_weak(self) -> None:
        suggestion = infer_project_manifest_suggestion(
            {"topic": "writing introduction", "context": "revise paragraph"},
            draft_content="",
            review_content="",
            merged_analysis="",
        )

        self.assertEqual(suggestion["active_subject"], "auto")
        self.assertEqual(suggestion["confidence"], 0.0)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run failing inference tests**

Run:

```bash
python3 -m unittest tests.test_project_inference -v
```

Expected: FAIL because `bridges.project_inference` is missing.

- [ ] **Step 3: Implement inference module**

Create `project_inference.py`:

```python
from __future__ import annotations

import re
from typing import Any


FINANCE_PATTERNS = {
    "asset-pricing": re.compile(r"\b(asset pricing|factor model|factor exposure|portfolio|returns?)\b", re.I),
    "event-study": re.compile(r"\b(event study|event window|abnormal returns?|leakage)\b", re.I),
}
ECONOMICS_PATTERNS = {
    "did": re.compile(r"\b(did|difference[- ]in[- ]differences|parallel trends?|pre[- ]trends?)\b", re.I),
    "causal-identification": re.compile(r"\b(causal identification|instrumental variable|regression discontinuity|identification)\b", re.I),
}


def infer_project_manifest_suggestion(
    task_packet: dict[str, Any],
    *,
    draft_content: str,
    review_content: str,
    merged_analysis: str,
) -> dict[str, Any]:
    text = " ".join(
        [
            str(task_packet.get("topic", "")),
            str(task_packet.get("context", "")),
            draft_content or "",
            review_content or "",
            merged_analysis or "",
        ]
    )
    finance_hits = _hits(FINANCE_PATTERNS, text)
    economics_hits = _hits(ECONOMICS_PATTERNS, text)
    if len(finance_hits) > len(economics_hits) and finance_hits:
        return {
            "active_subject": "finance",
            "method_lenses": finance_hits,
            "confidence": min(0.95, 0.55 + 0.15 * len(finance_hits)),
            "evidence": _evidence(text, FINANCE_PATTERNS),
        }
    if len(economics_hits) >= len(finance_hits) and economics_hits:
        return {
            "active_subject": "economics",
            "method_lenses": economics_hits,
            "confidence": min(0.95, 0.55 + 0.15 * len(economics_hits)),
            "evidence": _evidence(text, ECONOMICS_PATTERNS),
        }
    return {"active_subject": "auto", "method_lenses": [], "confidence": 0.0, "evidence": []}


def _hits(patterns: dict[str, re.Pattern[str]], text: str) -> list[str]:
    return [name for name, pattern in patterns.items() if pattern.search(text)]


def _evidence(text: str, patterns: dict[str, re.Pattern[str]]) -> list[str]:
    snippets: list[str] = []
    for pattern in patterns.values():
        match = pattern.search(text)
        if match:
            start = max(0, match.start() - 40)
            end = min(len(text), match.end() + 40)
            snippet = " ".join(text[start:end].split())
            if snippet not in snippets:
                snippets.append(snippet)
    return snippets[:3]
```

- [ ] **Step 4: Extend proposal text**

In `guidance_runtime.write_guidance_trace()`, call inference before writing `guidance_update_proposal.md`:

```python
    suggestion = infer_project_manifest_suggestion(
        task_packet,
        draft_content=draft_content,
        review_content=review_content,
        merged_analysis=merged_analysis,
    )
```

Change `_proposal_text(...)` signature to accept `manifest_suggestion`. Include this section:

````markdown
## Proposed Manifest Changes

```yaml
active_subject: finance
method_lenses:
  - event-study
```

## Manifest Evidence

- confidence: 0.7
- evidence: event study abnormal returns factor exposure
````

Rules:

```python
if suggestion["active_subject"] == "auto" or suggestion["confidence"] < 0.6:
    render "No structured manifest change proposed."
```

- [ ] **Step 5: Run inference and guidance tests**

Run:

```bash
python3 -m unittest tests.test_project_inference tests.test_guidance_runtime -v
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/project_inference.py packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py tests/test_project_inference.py tests/test_guidance_runtime.py
git commit -m "feat(guidance): propose project manifest updates from task evidence"
```

---

### Task 6: Apply Structured Manifest Proposals Safely

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py`
- Modify: `tests/test_guidance_runtime.py`

- [ ] **Step 1: Add failing apply tests**

Add to `GuidanceRuntimeTests`:

```python
    def test_apply_guidance_proposal_updates_manifest_when_yaml_block_exists(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            init_project_guidance(root)
            proposal = root / ".qiongli" / "trace" / "runs" / "run-manifest" / "guidance_update_proposal.md"
            proposal.parent.mkdir(parents=True)
            proposal.write_text(
                "# Guidance Update Proposal\n\n"
                "## Proposed Manifest Changes\n\n"
                "```yaml\n"
                "active_subject: finance\n"
                "method_lenses:\n"
                "  - event-study\n"
                "strictness: high\n"
                "```\n\n"
                "## Proposed Changes\n\n"
                "- Treat event-window leakage as a recurring project risk.\n",
                encoding="utf-8",
            )

            result = apply_guidance_proposal(root, proposal)

            manifest_text = (root / ".qiongli" / "guidance_manifest.yaml").read_text(encoding="utf-8")
            guidance_text = (root / ".qiongli" / "local_guidance.md").read_text(encoding="utf-8")
            self.assertTrue(result["applied"])
            self.assertEqual(result["manifest_update"]["active_subject"], "finance")
            self.assertIn("active_subject: finance", manifest_text)
            self.assertIn("event-window leakage", guidance_text)
```

- [ ] **Step 2: Run failing apply test**

Run:

```bash
python3 -m unittest tests.test_guidance_runtime.GuidanceRuntimeTests.test_apply_guidance_proposal_updates_manifest_when_yaml_block_exists -v
```

Expected: FAIL because `apply_guidance_proposal()` ignores manifest YAML.

- [ ] **Step 3: Parse and apply manifest YAML**

In `guidance_runtime.py`, import:

```python
import yaml
from .project_manifest import update_project_manifest
```

Add:

```python
def _extract_manifest_changes(proposal_text: str) -> dict[str, Any]:
    match = re.search(
        r"## Proposed Manifest Changes\s+```yaml\s+(.*?)```",
        proposal_text,
        re.S,
    )
    if not match:
        return {}
    raw = yaml.safe_load(match.group(1)) or {}
    if not isinstance(raw, dict):
        return {}
    allowed = {"active_subject", "secondary_subjects", "venue_profiles", "method_lenses", "strictness"}
    return {key: raw[key] for key in allowed if key in raw}
```

Inside `apply_guidance_proposal()` before writing local guidance:

```python
    manifest_changes = _extract_manifest_changes(proposal_text)
    manifest_update: dict[str, Any] = {}
    if manifest_changes:
        state = update_project_manifest(
            paths.project_root,
            active_subject=manifest_changes.get("active_subject"),
            secondary_subjects=manifest_changes.get("secondary_subjects"),
            venue_profiles=manifest_changes.get("venue_profiles"),
            method_lenses=manifest_changes.get("method_lenses"),
            strictness=manifest_changes.get("strictness"),
        )
        manifest_update = state.manifest.to_dict()
```

Add to return payload:

```python
        "manifest_update": manifest_update,
```

- [ ] **Step 4: Run guidance runtime tests**

Run:

```bash
python3 -m unittest tests.test_guidance_runtime -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/guidance_runtime.py tests/test_guidance_runtime.py
git commit -m "feat(guidance): apply structured project manifest proposals"
```

---

### Task 7: Expose Project State In MCP Preview

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- Modify: `tests/test_mcp_tool_handlers.py`

- [ ] **Step 1: Add failing MCP preview test**

Add to `MCPToolHandlerTests`:

```python
    def test_task_run_preview_reports_project_manifest_state(self) -> None:
        class StubResult:
            mode = "task-plan"
            confidence = 0.8
            merged_analysis = "preview"
            recommendations: list[str] = []
            data = {
                "task_id": "F3",
                "paper_type": "empirical",
                "topic": "my-topic",
                "artifact_root": "RESEARCH/[topic]/",
                "runtime_plan": {"primary_agent": "codex", "review_agent": "claude"},
            }

        class StubOrchestrator:
            def task_plan(self, **_kwargs: object) -> StubResult:
                return StubResult()

            def _build_controller_metadata(self, **_kwargs: object) -> dict[str, str]:
                return {"execution_mode": "duo", "controller": "codex", "primary_agent": "", "review_agent": "", "verifier_agent": "", "solo_role_gates": "standard"}

            def _controller_runtime_overrides(self, _metadata: dict[str, str]) -> dict[str, str]:
                return {}

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            (root / ".qiongli").mkdir()
            (root / ".qiongli" / "guidance_manifest.yaml").write_text("active_subject: finance\n", encoding="utf-8")
            with mock.patch.object(tool_handlers, "ModelOrchestrator", return_value=StubOrchestrator()):
                result = call_qiongli_tool(
                    "qiongli_task_run",
                    {"task_id": "F3", "paper_type": "empirical", "topic": "my-topic", "cwd": str(root)},
                )

        preview = result["structuredContent"]["data"]["task_run_preview"]
        self.assertEqual(preview["project_manifest"]["manifest"]["active_subject"], "finance")
```

- [ ] **Step 2: Run failing MCP test**

Run:

```bash
python3 -m unittest tests.test_mcp_tool_handlers.MCPToolHandlerTests.test_task_run_preview_reports_project_manifest_state -v
```

Expected: FAIL because preview has no `project_manifest`.

- [ ] **Step 3: Include project manifest in preview**

In `mcp_tool_handlers.py`, import:

```python
from bridges.project_manifest import load_project_manifest
```

Inside `_task_run_preview()` return dict:

```python
        "project_manifest": load_project_manifest(task_run_kwargs["cwd"]).to_packet(),
```

- [ ] **Step 4: Run MCP handler tests**

Run:

```bash
python3 -m unittest tests.test_mcp_tool_handlers -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py tests/test_mcp_tool_handlers.py
git commit -m "feat(mcp): report project manifest in task previews"
```

---

### Task 8: Document User Workflow And Compatibility

**Files:**
- Modify: `README.md`
- Modify: `docs/reference/cli.md`
- Modify: `docs/zh/reference/cli.md`
- Modify: `docs/advanced/subject-packaging-model.md`
- Modify: `content/workflow/SKILL.md`

- [ ] **Step 1: Update user-facing install model**

In `README.md`, replace the specialized CLI install emphasis with:

````markdown
For CLI and local plugin users, install Qiongli once and select project subject behavior per project:

```bash
qiongli install --target all
cd /path/to/research-project
qiongli project init
qiongli project set-subject finance
qiongli project status
```

If no project subject is configured, Qiongli runs in `active_subject: auto` mode. It uses core guidance, infers temporary subject/method lenses from the current task, and writes auditable proposals before changing project-local state.
````

Keep the existing `--subject` install docs as legacy/advanced:

```markdown
Subject-specific install flags remain available for Desktop ZIPs, compatibility testing, and deliberately materialized focused packages.
```

- [ ] **Step 2: Add CLI reference section**

In `docs/reference/cli.md`, add:

````markdown
### `qiongli project`

Project commands manage `.qiongli/guidance_manifest.yaml`, the structured project-level subject and method state.

```bash
qiongli project init --project-dir .
qiongli project set-subject finance --project-dir .
qiongli project set-venue journal-of-finance --project-dir .
qiongli project set-method-lens event-study --project-dir .
qiongli project status --project-dir .
```

Missing project manifests are treated as `active_subject: auto`. Auto mode is usable without setup; it never silently commits a subject switch. Persistent changes are written through explicit project commands or accepted guidance proposals.
````

Add equivalent Chinese content to `docs/zh/reference/cli.md`.

- [ ] **Step 3: Update skill-only hook**

In `content/workflow/SKILL.md`, expand project-local guidance:

```markdown
Before skill-only execution, check the current project root for `.qiongli/guidance_manifest.yaml`, `.qiongli/local_guidance.md`, and `.qiongli/guidance.d/*.md`. Treat a missing manifest as `active_subject: auto`. Use configured subject, venue, method lens, and strictness as project-local context only; never let them override canonical workflow contracts, required outputs, evidence gates, quality gates, MCP evidence requirements, or safety constraints.
```

- [ ] **Step 4: Run documentation-sensitive tests**

Run:

```bash
python3 -m unittest tests.test_skill_contract_alignment tests.test_cli_setup_docs tests.test_package_readmes -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/reference/cli.md docs/zh/reference/cli.md docs/advanced/subject-packaging-model.md content/workflow/SKILL.md
git commit -m "docs: document project-level subject guidance"
```

---

### Task 9: Release And Regression Verification

**Files:**
- No source files unless verification finds a regression.

- [ ] **Step 1: Run targeted unit tests**

Run:

```bash
python3 -m unittest tests.test_project_manifest tests.test_project_inference tests.test_guidance_runtime tests.test_orchestrator_workflows tests.test_mcp_tool_handlers tests.test_cli -v
```

Expected: PASS.

- [ ] **Step 2: Run contract tests likely affected by package/skill text**

Run:

```bash
python3 -m unittest tests.test_skill_contract_alignment tests.test_package_readmes tests.test_cli_setup_docs tests.test_mcp_cli -v
```

Expected: PASS.

- [ ] **Step 3: Run release readiness subset**

Run:

```bash
python3 scripts/validate_project_artifacts.py --cwd .
python3 scripts/audit_skill_sections.py --strict
```

Expected: both commands exit 0.

- [ ] **Step 4: Inspect working tree**

Run:

```bash
git status --short
```

Expected: only intentional source, test, and documentation files are modified.

- [ ] **Step 5: Commit final fixes**

If verification required small fixes:

```bash
git add <fixed-files>
git commit -m "fix(guidance): stabilize project subject routing"
```

---

## Migration Notes

Existing users keep working:

- No `.qiongli/guidance_manifest.yaml` means implicit `active_subject: auto`.
- Existing `.qiongli/local_guidance.md` remains valid.
- Existing `qiongli guidance` commands remain valid.
- Existing `qiongli install --subject economics` remains valid for compatibility, Desktop ZIPs, and deliberately materialized subject packages.

New preferred local workflow:

```bash
qiongli install --target all
qiongli project init --project-dir .
qiongli project set-subject economics --project-dir .
qiongli task-run --task-id F3 --paper-type empirical --topic minimum-wage --cwd .
```

Zero-config workflow:

```bash
qiongli task-run --task-id C1 --paper-type empirical --topic asset-pricing --cwd .
qiongli guidance trace --project-dir .
qiongli guidance apply --project-dir . --proposal .qiongli/trace/runs/<run_id>/guidance_update_proposal.md
```

## Self-Review

Spec coverage:

- Zero-config use is covered by Task 1 and Task 2 through implicit `active_subject: auto`.
- Per-project subject selection is covered by Task 3 and Task 4.
- Runtime subject/domain adaptation is covered by Task 4.
- Continuous local guidance improvement is covered by Task 5 and Task 6.
- MCP and skill-only surfaces are covered by Task 7 and Task 8.
- Backward compatibility is covered in Migration Notes and documentation updates.

Placeholder scan:

- The plan contains no deferred implementation placeholders.
- Every introduced module has concrete tests and implementation snippets.
- Commands include expected outcomes.

Type consistency:

- `ProjectManifestState.to_packet()` is used consistently as `project_manifest`.
- `ProjectSubjectState.to_packet()` is used consistently as `project_subject`.
- `active_subject`, `venue_profiles`, `method_lenses`, and `strictness` names match across YAML, packets, tests, and docs.
