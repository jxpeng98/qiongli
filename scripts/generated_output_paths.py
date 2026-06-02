from __future__ import annotations

from pathlib import Path


GENERATED_OUTPUT_DIRECTORIES = (
    "qiongli/payload",
    "packages/npm-qiongli/payload",
    "packages/npm-qiongli/python-runtime",
    "plugins/qiongli/skills/qiongli-workflow",
    "qiongli-workflow/skills",
    "qiongli-workflow/templates",
    "qiongli-workflow/standards",
    "qiongli-workflow/roles",
    "qiongli-workflow/venue-profiles",
)

GENERATED_OUTPUT_FILES = (
    "qiongli-workflow/skills-core.md",
    "qiongli-workflow/skills-summary.md",
)

GENERATED_OUTPUT_PATHS = GENERATED_OUTPUT_DIRECTORIES + GENERATED_OUTPUT_FILES


def normalize_generated_path(path: Path | str) -> str:
    return Path(str(path).strip()).as_posix().lstrip("./")


def is_generated_output_path(path: Path | str) -> bool:
    rel = normalize_generated_path(path)
    return rel in GENERATED_OUTPUT_FILES or any(
        rel == directory or rel.startswith(f"{directory}/") for directory in GENERATED_OUTPUT_DIRECTORIES
    )
