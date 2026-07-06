#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any, Mapping


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (Path(__file__).resolve().parent, PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from evaluate_subject_router import evaluate_cases, load_eval_cases  # noqa: E402
from run_subject_runtime_smoke import run_smoke_suite  # noqa: E402


METRIC_ORDER = (
    "decision_accuracy",
    "primary_subject_accuracy",
    "suggest_subject_precision",
    "near_miss_false_positives",
    "forbidden_subject_accuracy",
    "method_lens_accuracy",
    "all_case_checks_passed",
)


def build_acceptance_evidence(repo_root: Path = REPO_ROOT) -> str:
    root = Path(repo_root).resolve()
    eval_report = evaluate_cases(
        load_eval_cases(root / "tests" / "fixtures" / "subject_router_eval"),
    )
    smoke_report = run_smoke_suite(
        fixture_dir=root / "tests" / "fixtures" / "subject_runtime_smoke",
        mode="preview",
    )
    return render_acceptance_evidence(eval_report, smoke_report)


def render_acceptance_evidence(
    eval_report: Mapping[str, Any],
    smoke_report: Mapping[str, Any],
) -> str:
    eval_summary = _subject_eval_summary(eval_report)
    smoke_summary = _subject_smoke_summary(smoke_report)

    lines = [
        "## Subject Runtime Evidence",
        "",
        (
            "- Subject router eval: "
            f"{eval_summary['status']} "
            f"(cases: {eval_summary['case_count']}, "
            f"threshold_failures: {eval_summary['threshold_failures']})"
        ),
    ]
    if eval_summary["metrics"]:
        lines.append(f"  - metrics: {eval_summary['metrics']}")
    lines.extend(
        [
            (
                "- Subject runtime smoke: "
                f"{smoke_summary['status']} "
                f"(mode: {smoke_summary['mode']}, "
                f"passed: {smoke_summary['passed']}/{smoke_summary['total']}, "
                f"failed: {smoke_summary['failed']})"
            ),
            "",
        ]
    )
    return "\n".join(lines)


def _subject_eval_summary(report: Mapping[str, Any]) -> dict[str, Any]:
    threshold_failures = report.get("threshold_failures", [])
    failure_count = len(threshold_failures) if isinstance(threshold_failures, list) else 0
    return {
        "status": "passed" if failure_count == 0 else "failed",
        "case_count": int(report.get("case_count", 0) or 0),
        "threshold_failures": failure_count,
        "metrics": _format_metrics(report.get("metrics", {})),
    }


def _subject_smoke_summary(report: Mapping[str, Any]) -> dict[str, Any]:
    summary = report.get("summary", {})
    if not isinstance(summary, Mapping):
        summary = {}
    failed = int(summary.get("failed", 0) or 0)
    return {
        "status": "passed" if failed == 0 else "failed",
        "mode": str(report.get("mode", "preview") or "preview"),
        "total": int(summary.get("total", 0) or 0),
        "passed": int(summary.get("passed", 0) or 0),
        "failed": failed,
    }


def _format_metrics(raw_metrics: Any) -> str:
    if not isinstance(raw_metrics, Mapping):
        return ""
    parts: list[str] = []
    for metric in METRIC_ORDER:
        value = raw_metrics.get(metric)
        if isinstance(value, bool) or not isinstance(value, int | float):
            continue
        parts.append(f"{metric}={value:.3f}")
    return ", ".join(parts)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Generate release acceptance subject runtime evidence.",
    )
    parser.add_argument("--root", type=Path, default=REPO_ROOT)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args(argv)

    rendered = build_acceptance_evidence(args.root)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(rendered, encoding="utf-8")
    else:
        print(rendered, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
