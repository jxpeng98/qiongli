from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


REPO_MARKERS = ("pyproject.toml", ".git")
GENERATED_OUTPUT_ROOTS = (
    Path(".agent"),
    Path(".gemini"),
    Path("packages/python-qiongli/src/qiongli/payload"),
    Path("packages/npm-qiongli/payload"),
    Path("packages/npm-qiongli/python-runtime"),
    Path("plugins/qiongli"),
    Path("qiongli-workflow"),
    Path("content/workflow/skills"),
    Path("content/workflow/templates"),
    Path("content/workflow/standards"),
    Path("content/workflow/roles"),
    Path("content/workflow/venue-profiles"),
)
GENERATED_OUTPUT_FILES = (
    Path("content/workflow/skills-core.md"),
    Path("content/workflow/skills-summary.md"),
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
        return self._content_path("workflow", legacy_name="qiongli-workflow")

    @property
    def content(self) -> Path:
        return self.root / "content"

    @property
    def skills(self) -> Path:
        return self._content_path("skills")

    @property
    def templates(self) -> Path:
        return self._content_path("templates")

    @property
    def standards(self) -> Path:
        return self._content_path("standards")

    @property
    def roles(self) -> Path:
        return self._content_path("roles")

    @property
    def venue_profiles(self) -> Path:
        return self._content_path("venue-profiles")

    @property
    def subjects(self) -> Path:
        return self._content_path("subjects")

    @property
    def schemas(self) -> Path:
        return self._content_path("schemas")

    @property
    def skills_core(self) -> Path:
        return self._content_path("skills-core.md")

    @property
    def skills_summary(self) -> Path:
        return self._content_path("skills-summary.md")

    @property
    def python_source_root(self) -> Path:
        return self.root / "packages" / "python-qiongli" / "src"

    @property
    def python_package(self) -> Path:
        package_path = self.python_source_root / "qiongli"
        legacy_path = self.root / "qiongli"
        return package_path if package_path.exists() else legacy_path

    @property
    def research_skills_package(self) -> Path:
        package_path = self.python_source_root / "research_skills"
        legacy_path = self.root / "research_skills"
        return package_path if package_path.exists() else legacy_path

    @property
    def bridges_package(self) -> Path:
        package_path = self.python_package / "bridges"
        legacy_path = self.root / "bridges"
        return package_path if package_path.exists() else legacy_path

    @property
    def bridges_compat_package(self) -> Path:
        return self.python_source_root / "bridges"

    @property
    def npm_package(self) -> Path:
        return self.root / "packages" / "npm-qiongli"

    @property
    def literature_mcpb_package(self) -> Path:
        return self.root / "packages" / "qiongli-literature-mcpb"

    @property
    def plugin_package(self) -> Path:
        package_path = self.root / "packages" / "qiongli-plugin"
        legacy_path = self.root / "plugins" / "qiongli"
        return package_path if package_path.exists() else legacy_path

    @property
    def next_plugin_package(self) -> Path:
        return self.root / "packages" / "qiongli-next-plugin"

    @property
    def plugin_artifact_package(self) -> Path:
        return self.root / "plugins" / "qiongli"

    @property
    def agent_platform(self) -> Path:
        path = self.plugin_package / "platforms" / "agent"
        legacy_path = self.root / ".agent"
        return path if path.exists() else legacy_path

    @property
    def gemini_platform(self) -> Path:
        path = self.plugin_package / "platforms" / "gemini"
        legacy_path = self.root / ".gemini"
        return path if path.exists() else legacy_path

    @property
    def agent_platform_artifact(self) -> Path:
        return self.root / ".agent"

    @property
    def gemini_platform_artifact(self) -> Path:
        return self.root / ".gemini"

    @property
    def scripts(self) -> Path:
        path = self.tooling / "scripts"
        legacy_path = self.root / "scripts"
        return path if path.exists() else legacy_path

    @property
    def tooling(self) -> Path:
        return self.root / "tooling"

    @property
    def pipelines(self) -> Path:
        path = self.tooling / "pipelines"
        legacy_path = self.root / "pipelines"
        return path if path.exists() else legacy_path

    @property
    def install(self) -> Path:
        path = self.tooling / "install"
        legacy_path = self.root / "install"
        return path if path.exists() else legacy_path

    @property
    def release(self) -> Path:
        path = self.tooling / "release"
        legacy_path = self.root / "release"
        return path if path.exists() else legacy_path

    @property
    def evals(self) -> Path:
        return self.root / "evals"

    @property
    def eval_legacy(self) -> Path:
        return self.evals

    @property
    def eval_cases(self) -> Path:
        return self.evals / "cases"

    @property
    def eval_rubrics(self) -> Path:
        return self.evals / "rubrics"

    @property
    def eval_runner(self) -> Path:
        return self.evals / "runner"

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

    def resolve_source_path(self, relative_path: Path | str) -> Path:
        rel = Path(str(relative_path))
        if rel.is_absolute():
            return rel
        parts = rel.parts
        if not parts:
            return self.root

        first = parts[0]
        rest = Path(*parts[1:]) if len(parts) > 1 else Path()
        source_roots = {
            ".agent": self.agent_platform,
            ".gemini": self.gemini_platform,
            "qiongli-workflow": self.workflow,
            "skills": self.skills,
            "templates": self.templates,
            "standards": self.standards,
            "roles": self.roles,
            "venue-profiles": self.venue_profiles,
            "subjects": self.subjects,
            "schemas": self.schemas,
            "scripts": self.scripts,
            "pipelines": self.pipelines,
            "install": self.install,
            "release": self.release,
        }
        if first in source_roots:
            return source_roots[first] / rest
        if first == "skills-core.md":
            return self.skills_core
        if first == "skills-summary.md":
            return self.skills_summary
        return self.root / rel

    def _content_path(self, name: str, *, legacy_name: str | None = None) -> Path:
        content_path = self.content / name
        legacy_path = self.root / (legacy_name or name)
        return content_path if content_path.exists() else legacy_path
