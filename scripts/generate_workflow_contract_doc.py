#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.workflow_contract_doc import generate_workflow_contract_reference
from qiongli.source_layout import RepoLayout


def main() -> int:
    target = RepoLayout(REPO_ROOT).workflow / "references" / "workflow-contract.md"
    target.write_text(generate_workflow_contract_reference(REPO_ROOT), encoding="utf-8")
    print(f"[WRITE] {target.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
