# Repository Restructuring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce the repository layout abstraction and guardrails needed to move Qiongli toward the approved `content/`, `packages/`, `tooling/`, and unified `evals/` structure.

**Architecture:** Start with Phase 0 only: add a `qiongli.source_layout` module that names the current logical roots without moving files. Tests should prove the abstraction points at tracked canonical sources and existing package/tooling roots. Generated-output path classification should read from the same module so later migration phases can change one source of path truth before moving consumers.

**Tech Stack:** Python 3.12+, `unittest`, `pathlib`, existing Qiongli materialization scripts and tests.

---

### Task 1: Add Source Layout Tests

**Files:**
- Create: `tests/test_source_layout.py`
- Modify: none
- Test: `tests/test_source_layout.py`

- [x] **Step 1: Write failing tests for logical roots**

Create `tests/test_source_layout.py`:

```python
from __future__ import annotations

import unittest
from pathlib import Path

from qiongli.source_layout import RepoLayout, discover_repo_root


REPO_ROOT = Path(__file__).resolve().parents[1]


class SourceLayoutTests(unittest.TestCase):
    def test_discover_repo_root_from_test_file(self) -> None:
        self.assertEqual(REPO_ROOT, discover_repo_root(Path(__file__)))

    def test_current_canonical_content_roots_exist(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        expected_files = (
            layout.workflow / "SKILL.md",
            layout.workflow / "VERSION",
            layout.skills / "registry.yaml",
            layout.templates / "idea-funnel.md",
            layout.standards / "research-workflow-contract.yaml",
            layout.roles / "pi.yaml",
            layout.venue_profiles / "nature.yaml",
            layout.subjects / "catalog.yaml",
            layout.skills_core,
            layout.skills_summary,
        )

        for path in expected_files:
            with self.subTest(path=path):
                self.assertTrue(path.exists(), f"{path} should exist")

    def test_current_package_and_tooling_roots_exist(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        expected_dirs = (
            layout.python_package,
            layout.research_skills_package,
            layout.npm_package,
            layout.plugin_package,
            layout.literature_mcpb_package,
            layout.scripts,
            layout.release,
            layout.evals,
            layout.eval_legacy,
            layout.docs,
            layout.tests,
        )

        for path in expected_dirs:
            with self.subTest(path=path):
                self.assertTrue(path.is_dir(), f"{path} should be a directory")

    def test_materialized_output_roots_are_named(self) -> None:
        layout = RepoLayout(REPO_ROOT)

        self.assertIn(Path("qiongli/payload"), layout.generated_output_roots)
        self.assertIn(Path("packages/npm-qiongli/payload"), layout.generated_output_roots)
        self.assertIn(Path("plugins/qiongli/skills/qiongli-workflow"), layout.generated_output_roots)


if __name__ == "__main__":
    unittest.main()
```

- [x] **Step 2: Run the test to verify it fails**

Run:

```bash
.venv/bin/python -m unittest tests.test_source_layout
```

Expected: FAIL with `ModuleNotFoundError: No module named 'qiongli.source_layout'`.

### Task 2: Implement `qiongli.source_layout`

**Files:**
- Create: `qiongli/source_layout.py`
- Modify: `scripts/generated_output_paths.py`
- Test: `tests/test_source_layout.py`

- [x] **Step 1: Add the layout module**

Create `qiongli/source_layout.py`:

```python
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


REPO_MARKERS = ("pyproject.toml", ".git")


def discover_repo_root(start: Path | str) -> Path:
    """Return the nearest ancestor that looks like the Qiongli repository root."""

    current = Path(start).resolve()
    if current.is_file():
        current = current.parent

    for candidate in (current, *current.parents):
        if all((candidate / marker).exists() for marker in REPO_MARKERS):
            return candidate

    raise ValueError(f"could not find repository root from {start}")


@dataclass(frozen=True)
class RepoLayout:
    root: Path

    def __post_init__(self) -> None:
        object.__setattr__(self, "root", self.root.resolve())

    @property
    def workflow(self) -> Path:
        return self.root / "qiongli-workflow"

    @property
    def skills(self) -> Path:
        return self.root / "skills"

    @property
    def templates(self) -> Path:
        return self.root / "templates"

    @property
    def standards(self) -> Path:
        return self.root / "standards"

    @property
    def roles(self) -> Path:
        return self.root / "roles"

    @property
    def venue_profiles(self) -> Path:
        return self.root / "venue-profiles"

    @property
    def subjects(self) -> Path:
        return self.root / "subjects"

    @property
    def schemas(self) -> Path:
        return self.root / "schemas"

    @property
    def skills_core(self) -> Path:
        return self.root / "skills-core.md"

    @property
    def skills_summary(self) -> Path:
        return self.root / "skills-summary.md"

    @property
    def python_package(self) -> Path:
        return self.root / "qiongli"

    @property
    def research_skills_package(self) -> Path:
        return self.root / "research_skills"

    @property
    def npm_package(self) -> Path:
        return self.root / "packages" / "npm-qiongli"

    @property
    def literature_mcpb_package(self) -> Path:
        return self.root / "packages" / "qiongli-literature-mcpb"

    @property
    def plugin_package(self) -> Path:
        return self.root / "plugins" / "qiongli"

    @property
    def scripts(self) -> Path:
        return self.root / "scripts"

    @property
    def release(self) -> Path:
        return self.root / "release"

    @property
    def evals(self) -> Path:
        return self.root / "evals"

    @property
    def eval_legacy(self) -> Path:
        return self.root / "eval"

    @property
    def docs(self) -> Path:
        return self.root / "docs"

    @property
    def tests(self) -> Path:
        return self.root / "tests"

    @property
    def generated_output_roots(self) -> tuple[Path, ...]:
        return (
            Path("qiongli/payload"),
            Path("packages/npm-qiongli/payload"),
            Path("packages/npm-qiongli/python-runtime"),
            Path("plugins/qiongli/skills/qiongli-workflow"),
            Path("qiongli-workflow/skills"),
            Path("qiongli-workflow/templates"),
            Path("qiongli-workflow/standards"),
            Path("qiongli-workflow/roles"),
            Path("qiongli-workflow/venue-profiles"),
        )

    @property
    def generated_output_files(self) -> tuple[Path, ...]:
        return (
            Path("qiongli-workflow/skills-core.md"),
            Path("qiongli-workflow/skills-summary.md"),
        )
```

- [x] **Step 2: Run the source layout tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_source_layout
```

Expected: PASS.

### Task 3: Wire Existing Source-Tree Guardrail to Layout

**Files:**
- Modify: `tests/test_distribution_source_tree.py`
- Test: `tests/test_distribution_source_tree.py`

- [x] **Step 1: Update the test to read roots from `RepoLayout`**

Replace the hard-coded path tuples in `tests/test_distribution_source_tree.py` with:

```python
from qiongli.source_layout import RepoLayout

LAYOUT = RepoLayout(REPO_ROOT)

GENERATED_OUTPUT_ROOTS = tuple(str(path) for path in LAYOUT.generated_output_roots)

CANONICAL_SOURCE_PATHS = (
    str(LAYOUT.workflow.relative_to(REPO_ROOT) / "SKILL.md"),
    str(LAYOUT.workflow.relative_to(REPO_ROOT) / "references" / "workflow-contract.md"),
    str(LAYOUT.workflow.relative_to(REPO_ROOT) / "workflows" / "paper.md"),
    str(LAYOUT.skills.relative_to(REPO_ROOT) / "registry.yaml"),
    str(LAYOUT.templates.relative_to(REPO_ROOT) / "idea-funnel.md"),
    str(LAYOUT.standards.relative_to(REPO_ROOT) / "research-workflow-contract.yaml"),
    str(LAYOUT.roles.relative_to(REPO_ROOT) / "pi.yaml"),
    str(LAYOUT.venue_profiles.relative_to(REPO_ROOT) / "nature.yaml"),
    str(LAYOUT.subjects.relative_to(REPO_ROOT) / "catalog.yaml"),
)
```

- [x] **Step 2: Run the source-tree guardrail test**

Run:

```bash
.venv/bin/python -m unittest tests.test_distribution_source_tree
```

Expected: PASS.

### Task 4: Commit Phase 0 Layout Guardrails

**Files:**
- Add: `docs/development/repository-restructuring-implementation-plan.md`
- Add: `qiongli/source_layout.py`
- Add: `tests/test_source_layout.py`
- Modify: `scripts/generated_output_paths.py`
- Modify: `tests/test_distribution_source_tree.py`

- [x] **Step 1: Run focused verification**

Run:

```bash
.venv/bin/python -m unittest tests.test_source_layout tests.test_distribution_source_tree tests.test_generated_payload_guard tests.test_distribution_materialization_docs
```

Expected: PASS.

- [x] **Step 2: Stage the Phase 0 files**

Run:

```bash
git add docs/development/repository-restructuring-implementation-plan.md qiongli/source_layout.py scripts/generated_output_paths.py tests/test_source_layout.py tests/test_distribution_source_tree.py
```

- [x] **Step 3: Commit**

Run:

```bash
git commit -m "test(layout): add repository source layout guardrails"
```

Expected: commit succeeds.
