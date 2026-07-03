from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SRC = REPO_ROOT / "packages" / "python-qiongli" / "src"
if str(PYTHON_SRC) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC))

from qiongli.bridges.project_manifest import ProjectManifest  # noqa: E402
from qiongli.bridges.subject_contracts import load_runtime_subject_contracts  # noqa: E402
from qiongli.bridges.subject_refinement import infer_subject_refinement  # noqa: E402


FIXTURE_DIR = REPO_ROOT / "tests" / "fixtures" / "subject_router_eval"
DEFAULT_THRESHOLDS = {
    "decision_accuracy": 0.90,
    "primary_subject_accuracy": 0.90,
    "suggest_subject_precision": 0.85,
    "near_miss_false_positives": 0,
    "forbidden_subject_accuracy": 1.0,
    "method_lens_accuracy": 1.0,
    "all_case_checks_passed": 1.0,
}
CONCRETE_CORE_SUBJECTS = {"auto", "core"}
REQUIRED_GATE_TAGS = {
    "clear_positive",
    "method_only_borrow",
    "near_miss",
}


@dataclass(frozen=True)
class EvalCase:
    id: str
    description: str
    request: str
    manifest: dict[str, Any]
    expected: dict[str, Any]
    source: str
    subject_under_test: str = ""
    tags: list[str] | None = None


def load_eval_cases(fixture_dir: Path = FIXTURE_DIR) -> list[EvalCase]:
    cases: list[EvalCase] = []
    seen: dict[str, Path] = {}
    for path in sorted(Path(fixture_dir).rglob("*.json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        case_id = str(payload["id"])
        if case_id in seen:
            raise ValueError(
                "duplicate fixture id "
                f"{case_id!r}: {_repo_relative(seen[case_id])}, {_repo_relative(path)}"
            )
        seen[case_id] = path
        subject_under_test = str(
            payload.get("subject_under_test", "") or path.parent.name
        )
        tags = [
            tag
            for tag in list(payload.get("tags", []) or [])
            if isinstance(tag, str)
        ]
        cases.append(
            EvalCase(
                id=case_id,
                description=str(payload.get("description", "")),
                request=str(payload["request"]),
                manifest=dict(payload["manifest"]),
                expected=dict(payload["expected"]),
                source=_repo_relative(path),
                subject_under_test=subject_under_test,
                tags=tags,
            )
        )
    if not cases:
        raise ValueError(f"no subject router eval fixtures found in {fixture_dir}")
    return sorted(cases, key=lambda case: case.id)


def run_eval_case(case: EvalCase) -> dict[str, Any]:
    manifest = ProjectManifest(**case.manifest).normalized()
    packet = infer_subject_refinement(
        {"topic": case.request, "context": case.request},
        manifest_state=manifest,
    )
    refinement = packet.to_packet()
    actual = _actual_eval_result(manifest, refinement)
    expected = _normalized_expected(case.expected)
    passed = {
        "decision": actual["decision"] == expected["decision"],
        "primary_subject": actual["primary_subject"] == expected["primary_subject"],
        "suggest_subjects": set(expected["suggest_subjects"]).issubset(
            set(actual["suggest_subjects"])
        ),
        "forbidden_subjects": not (
            set(expected["forbidden_subjects"]) & set(actual["suggest_subjects"])
        ),
        "method_lenses": set(actual["method_lenses"]) == set(expected["method_lenses"]),
    }
    return {
        "id": case.id,
        "description": case.description,
        "source": case.source,
        "expected": expected,
        "actual": actual,
        "passed": passed,
    }


def evaluate_cases(
    cases: list[EvalCase],
    thresholds: Mapping[str, float] = DEFAULT_THRESHOLDS,
) -> dict[str, Any]:
    if not cases:
        raise ValueError("cannot evaluate an empty case list")
    case_results = [run_eval_case(case) for case in cases]
    metrics = _metrics(case_results, cases)
    return {
        "case_count": len(case_results),
        "metrics": metrics,
        "cases": case_results,
        "threshold_failures": threshold_failures(metrics, thresholds),
    }


def subject_gate_report(subject: str, cases: list[EvalCase]) -> dict[str, Any]:
    contracts = load_runtime_subject_contracts()
    contract = contracts.get(subject)
    activation_status = contract.activation_status if contract else "candidate"
    thresholds = _contract_thresholds(contract)
    subject_cases = [
        case
        for case in cases
        if case.subject_under_test == subject or subject in list(case.tags or [])
    ]
    report = (
        evaluate_cases(subject_cases, thresholds=thresholds)
        if subject_cases
        else _empty_eval_report()
    )
    subject_tags = {
        tag
        for case in subject_cases
        for tag in list(case.tags or [])
    }
    blocking_failures: list[str] = []
    if contract is None:
        blocking_failures.append("missing runtime subject contract")
    if activation_status != "runtime_enabled":
        blocking_failures.append(f"activation_status is {activation_status}")
    if contract is not None and activation_status == "runtime_enabled":
        blocking_failures.extend(_missing_resource_failures(contract))
    missing_tags = sorted(REQUIRED_GATE_TAGS - subject_tags)
    for tag in missing_tags:
        blocking_failures.append(f"missing {tag} fixtures")
    for failure in report["threshold_failures"]:
        metric = failure.get("metric", "unknown")
        blocking_failures.append(f"threshold failure: {metric}")
    return {
        "subject": subject,
        "activation_status": activation_status,
        "eligible_for_runtime_enabled": not blocking_failures,
        "case_count": len(subject_cases),
        "required_tags": sorted(REQUIRED_GATE_TAGS),
        "present_tags": sorted(subject_tags),
        "metrics": report["metrics"],
        "blocking_failures": blocking_failures,
    }


def _contract_thresholds(contract: Any | None) -> Mapping[str, float]:
    if contract is None:
        return DEFAULT_THRESHOLDS
    activation_gate = getattr(contract, "activation_gate", {})
    if not isinstance(activation_gate, Mapping):
        return DEFAULT_THRESHOLDS
    required_metrics = activation_gate.get("required_metrics", {})
    if not isinstance(required_metrics, Mapping):
        return DEFAULT_THRESHOLDS
    thresholds: dict[str, float] = {}
    for metric, threshold in required_metrics.items():
        if metric not in DEFAULT_THRESHOLDS:
            continue
        if isinstance(threshold, bool) or not isinstance(threshold, int | float):
            continue
        thresholds[str(metric)] = float(threshold)
    return thresholds or DEFAULT_THRESHOLDS


def _missing_resource_failures(contract: Any) -> list[str]:
    resource_root = _contract_resource_root(getattr(contract, "source", ""))
    failures: list[str] = []
    for field in (
        "domain_profile",
        "overlay",
        "subject_skill",
        "evaluation_pack",
    ):
        resource = getattr(contract, field, "")
        if _resource_is_missing(resource_root, resource):
            failures.append(f"missing resource: {field} {resource}")

    method_lenses = getattr(contract, "method_lenses", {})
    if isinstance(method_lenses, Mapping):
        for lens, config in method_lenses.items():
            if not isinstance(config, Mapping):
                continue
            resource = config.get("resource", "")
            if _resource_is_missing(resource_root, resource):
                failures.append(
                    f"missing resource: method_lenses[{lens}].resource {resource}"
                )
    return failures


def _contract_resource_root(source: str) -> Path:
    source_path = Path(source).expanduser()
    if not source_path.is_absolute():
        source_path = REPO_ROOT / source_path
    source_path = source_path.resolve()
    for parent in source_path.parents:
        if parent.name != "subjects":
            continue
        if parent.parent.name == "content":
            return parent.parent.parent
        return parent.parent
    return REPO_ROOT


def _resource_is_missing(resource_root: Path, resource: Any) -> bool:
    if not isinstance(resource, str) or not resource.strip():
        return True
    return not (resource_root / resource.strip()).exists()


def threshold_failures(
    metrics: Mapping[str, float],
    thresholds: Mapping[str, float] = DEFAULT_THRESHOLDS,
) -> list[dict[str, Any]]:
    failures: list[dict[str, Any]] = []
    for metric, threshold in thresholds.items():
        actual = metrics[metric]
        if metric == "near_miss_false_positives":
            if actual > threshold:
                failures.append(
                    {
                        "metric": metric,
                        "actual": actual,
                        "threshold": threshold,
                        "comparator": "<=",
                    }
                )
            continue
        if actual < threshold:
            failures.append(
                {
                    "metric": metric,
                    "actual": actual,
                    "threshold": threshold,
                    "comparator": ">=",
                }
            )
    return failures


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Evaluate adaptive subject router fixtures.")
    parser.add_argument("--fixture-dir", "--fixtures", type=Path, default=FIXTURE_DIR)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--subject", default="")
    parser.add_argument("--gate", choices=["runtime-enabled"], default="")
    args = parser.parse_args(argv)

    try:
        cases = load_eval_cases(args.fixture_dir)
        report = evaluate_cases(cases)
        if args.subject and args.gate:
            report["subject_gate"] = subject_gate_report(args.subject, cases)
    except Exception as exc:  # noqa: BLE001 - keep CLI failures machine-readable with --json.
        report = {
            "case_count": 0,
            "metrics": {},
            "cases": [],
            "threshold_failures": [{"metric": "runner_error", "error": str(exc)}],
        }

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    else:
        _print_text_report(report)
    gate = report.get("subject_gate")
    if isinstance(gate, Mapping) and not gate.get("eligible_for_runtime_enabled"):
        return 1
    return 0 if not report["threshold_failures"] else 1


def _actual_eval_result(
    manifest: ProjectManifest,
    refinement: Mapping[str, Any],
) -> dict[str, Any]:
    method_lenses = _effective_method_lenses(refinement)
    suggest_subjects = _suggest_subjects(manifest, refinement)
    decision = _eval_decision(
        manifest=manifest,
        runtime_decision=str(refinement.get("decision", "")),
        suggest_subjects=suggest_subjects,
        refinement=refinement,
    )
    primary_subject = _eval_primary_subject(manifest, refinement, decision)
    return {
        "decision": decision,
        "primary_subject": primary_subject,
        "suggest_subjects": suggest_subjects,
        "method_lenses": method_lenses,
    }


def _eval_decision(
    *,
    manifest: ProjectManifest,
    runtime_decision: str,
    suggest_subjects: list[str],
    refinement: Mapping[str, Any],
) -> str:
    if manifest.subject_mode == "locked":
        return "keep_locked"
    if runtime_decision in {"suggest_subject", "confirm_subject"}:
        return "recommend"
    if runtime_decision == "borrow_lens":
        if (
            suggest_subjects
            or _has_candidate_subjects(refinement)
            or _has_borrowed_lenses(refinement)
        ):
            return "recommend"
        return "core_only"
    if runtime_decision == "no_subject":
        return "core_only"
    return runtime_decision


def _eval_primary_subject(
    manifest: ProjectManifest,
    refinement: Mapping[str, Any],
    decision: str,
) -> str:
    if manifest.subject_mode == "locked" and manifest.active_subject not in CONCRETE_CORE_SUBJECTS:
        return manifest.active_subject
    primary = str(refinement.get("primary_subject", "auto"))
    if primary == "auto" and decision == "core_only":
        return "core"
    return primary


def _suggest_subjects(
    manifest: ProjectManifest,
    refinement: Mapping[str, Any],
) -> list[str]:
    subjects: list[str] = []
    runtime_decision = str(refinement.get("decision", ""))
    locked_active_subject = (
        manifest.active_subject
        if (
            manifest.subject_mode == "locked"
            and manifest.active_subject not in CONCRETE_CORE_SUBJECTS
        )
        else None
    )
    candidates = refinement.get("candidate_subjects", [])
    if isinstance(candidates, list):
        for candidate in candidates:
            if isinstance(candidate, Mapping):
                subject = candidate.get("subject")
                if isinstance(subject, str):
                    subjects.append(subject)

    primary = refinement.get("primary_subject")
    if (
        runtime_decision == "suggest_subject"
        and isinstance(primary, str)
        and primary not in subjects
    ):
        subjects.append(primary)

    excluded_subjects = set(CONCRETE_CORE_SUBJECTS)
    if runtime_decision == "lock_subject" or manifest.subject_mode == "locked":
        if locked_active_subject:
            excluded_subjects.add(locked_active_subject)
    return _unique([subject for subject in subjects if subject not in excluded_subjects])


def _effective_method_lenses(refinement: Mapping[str, Any]) -> list[str]:
    lenses = [
        lens
        for lens in list(refinement.get("method_lenses", []) or [])
        if isinstance(lens, str)
    ]
    borrowed = refinement.get("borrowed_lenses", [])
    if isinstance(borrowed, list):
        for item in borrowed:
            if isinstance(item, Mapping) and isinstance(item.get("lens"), str):
                lenses.append(str(item["lens"]))
    return _unique(lenses)


def _normalized_expected(expected: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "decision": str(expected.get("decision", "")),
        "primary_subject": str(expected.get("primary_subject", "")),
        "suggest_subjects": list(expected.get("suggest_subjects", []) or []),
        "allowed_neighbor_subjects": list(
            expected.get("allowed_neighbor_subjects", []) or []
        ),
        "forbidden_subjects": list(expected.get("forbidden_subjects", []) or []),
        "method_lenses": list(expected.get("method_lenses", []) or []),
    }


def _empty_eval_report() -> dict[str, Any]:
    return {
        "case_count": 0,
        "metrics": _empty_metrics(),
        "cases": [],
        "threshold_failures": [],
    }


def _empty_metrics() -> dict[str, float]:
    return {
        "decision_accuracy": 0.0,
        "primary_subject_accuracy": 0.0,
        "suggest_subject_precision": 0.0,
        "near_miss_false_positives": 0,
        "forbidden_subject_accuracy": 0.0,
        "method_lens_accuracy": 0.0,
        "all_case_checks_passed": 0.0,
    }


def _metrics(
    case_results: list[dict[str, Any]],
    cases: list[EvalCase] | None = None,
) -> dict[str, float]:
    total = len(case_results)
    actual_suggestion_total = 0
    accepted_suggestion_hits = 0
    for case in case_results:
        expected_subjects = set(case["expected"].get("suggest_subjects", []))
        allowed_neighbor_subjects = set(
            case["expected"].get("allowed_neighbor_subjects", [])
        )
        accepted_subjects = expected_subjects | allowed_neighbor_subjects
        actual_subjects = set(case["actual"]["suggest_subjects"])
        actual_suggestion_total += len(actual_subjects)
        accepted_suggestion_hits += len(actual_subjects & accepted_subjects)

    suggest_subject_precision = (
        1.0
        if actual_suggestion_total == 0
        else accepted_suggestion_hits / actual_suggestion_total
    )
    return {
        "decision_accuracy": _pass_rate(case_results, "decision", total),
        "primary_subject_accuracy": _pass_rate(case_results, "primary_subject", total),
        "suggest_subject_precision": suggest_subject_precision,
        "near_miss_false_positives": sum(
            1
            for index, case in enumerate(case_results)
            if _is_near_miss_case(case, cases[index] if cases else None)
            and case["actual"]["suggest_subjects"]
        ),
        "forbidden_subject_accuracy": _pass_rate(
            case_results,
            "forbidden_subjects",
            total,
        ),
        "method_lens_accuracy": _pass_rate(case_results, "method_lenses", total),
        "all_case_checks_passed": (
            1.0
            if all(all(case["passed"].values()) for case in case_results)
            else 0.0
        ),
    }


def _is_near_miss_case(
    case_result: Mapping[str, Any],
    eval_case: EvalCase | None = None,
) -> bool:
    if str(case_result["id"]).startswith("near_miss"):
        return True
    result_tags = case_result.get("tags", [])
    if isinstance(result_tags, list) and "near_miss" in result_tags:
        return True
    if eval_case is not None:
        return "near_miss" in list(eval_case.tags or [])
    return False


def _pass_rate(case_results: list[dict[str, Any]], flag: str, total: int) -> float:
    return sum(1 for case in case_results if case["passed"][flag]) / total


def _has_candidate_subjects(refinement: Mapping[str, Any]) -> bool:
    candidates = refinement.get("candidate_subjects", [])
    if not isinstance(candidates, list):
        return False
    return any(
        isinstance(candidate, Mapping)
        and isinstance(candidate.get("subject"), str)
        and candidate["subject"] not in CONCRETE_CORE_SUBJECTS
        for candidate in candidates
    )


def _has_borrowed_lenses(refinement: Mapping[str, Any]) -> bool:
    borrowed = refinement.get("borrowed_lenses", [])
    return isinstance(borrowed, list) and bool(borrowed)


def _print_text_report(report: Mapping[str, Any]) -> None:
    print(f"case_count: {report['case_count']}")
    metrics = report.get("metrics", {})
    if isinstance(metrics, Mapping):
        for metric in DEFAULT_THRESHOLDS:
            if metric in metrics:
                print(f"{metric}: {metrics[metric]}")
    failures = report.get("threshold_failures", [])
    if failures:
        print("threshold_failures:")
        for failure in failures:
            print(f"- {failure}")


def _repo_relative(path: Path) -> str:
    resolved = path.resolve()
    try:
        return str(resolved.relative_to(REPO_ROOT))
    except ValueError:
        return str(resolved)


def _unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    unique_values: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        unique_values.append(value)
    return unique_values


if __name__ == "__main__":
    raise SystemExit(main())
