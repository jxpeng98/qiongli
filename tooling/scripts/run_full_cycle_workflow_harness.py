#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SRC = REPO_ROOT / "packages" / "python-qiongli" / "src"
if str(PYTHON_SRC) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC))

from qiongli.bridges.journal_fit import recommend_journals  # noqa: E402
from qiongli.bridges.lifecycle_harness import build_lifecycle_report  # noqa: E402


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run deterministic full-cycle Qiongli harness fixtures."
    )
    parser.add_argument("--fixture", required=True, help="Fixture project directory.")
    parser.add_argument("--json-report", required=True, help="Output JSON report path.")
    parser.add_argument("--topic", default="full-cycle-fixture")
    parser.add_argument("--paper-type", default="empirical")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    fixture = Path(args.fixture).resolve()
    if not fixture.is_dir():
        parser.error(f"--fixture must be an existing directory: {fixture}")

    report_path = Path(args.json_report).resolve()
    with tempfile.TemporaryDirectory() as tmp_dir:
        project = Path(tmp_dir) / "project"
        shutil.copytree(fixture, project)
        report = build_lifecycle_report(
            project,
            topic=args.topic,
            paper_type=args.paper_type,
        )

        venues = project / "venues"
        if venues.exists():
            report["journal_fit"] = recommend_journals(project, venue_roots=[venues])
            _normalize_journal_fit_sources(report, project)

        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(
            json.dumps(report, indent=2, ensure_ascii=False),
            encoding="utf-8",
        )

    if report.get("blocking_reasons"):
        return 1
    if _journal_fit_blocks_submission(report.get("journal_fit")):
        return 1
    return 0


def _journal_fit_blocks_submission(journal_fit: Any) -> bool:
    if not isinstance(journal_fit, dict):
        return False
    if journal_fit.get("status") == "blocked":
        return True
    return bool(journal_fit.get("blocking_reasons"))


def _normalize_journal_fit_sources(report: dict[str, Any], project: Path) -> None:
    journal_fit = report.get("journal_fit")
    if not isinstance(journal_fit, dict):
        return

    project_root = project.resolve()
    for venue in journal_fit.get("ranked_venues", []):
        if not isinstance(venue, dict):
            continue
        source = venue.get("source")
        if not isinstance(source, str) or not source:
            continue

        source_path = Path(source)
        if not source_path.is_absolute():
            continue
        try:
            venue["source"] = source_path.resolve().relative_to(project_root).as_posix()
        except ValueError:
            venue["source"] = source


if __name__ == "__main__":
    raise SystemExit(main())
