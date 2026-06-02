from __future__ import annotations

import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import GENERATED_OUTPUT_FILES as GENERATED_OUTPUT_FILE_PATHS
from qiongli.source_layout import GENERATED_OUTPUT_ROOTS

GENERATED_OUTPUT_DIRECTORIES = tuple(path.as_posix() for path in GENERATED_OUTPUT_ROOTS)
GENERATED_OUTPUT_FILES = tuple(path.as_posix() for path in GENERATED_OUTPUT_FILE_PATHS)

GENERATED_OUTPUT_PATHS = GENERATED_OUTPUT_DIRECTORIES + GENERATED_OUTPUT_FILES


def normalize_generated_path(path: Path | str) -> str:
    return Path(str(path).strip()).as_posix().lstrip("./")


def is_generated_output_path(path: Path | str) -> bool:
    rel = normalize_generated_path(path)
    return rel in GENERATED_OUTPUT_FILES or any(
        rel == directory or rel.startswith(f"{directory}/") for directory in GENERATED_OUTPUT_DIRECTORIES
    )
