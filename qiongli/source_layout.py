from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


REPO_MARKERS = ("pyproject.toml", ".git")
GENERATED_OUTPUT_ROOTS = (
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
GENERATED_OUTPUT_FILES = (
    Path("qiongli-workflow/skills-core.md"),
    Path("qiongli-workflow/skills-summary.md"),
)


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
        return GENERATED_OUTPUT_ROOTS

    @property
    def generated_output_files(self) -> tuple[Path, ...]:
        return GENERATED_OUTPUT_FILES
