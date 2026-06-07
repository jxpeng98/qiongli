#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package  # noqa: E402


REQUIRED_FIELDS = (
    "id",
    "subject",
    "coverage",
    "prompt",
    "expected_skill_refs",
    "expected_terms",
    "expected_domain_profiles",
    "forbidden_domain_profiles",
)
STRING_FIELDS = ("id", "subject", "coverage", "prompt")
STRING_LIST_FIELDS = (
    "expected_skill_refs",
    "expected_terms",
    "expected_domain_profiles",
    "forbidden_domain_profiles",
)


@dataclass(frozen=True)
class SubjectEvalCase:
    id: str
    subject: str
    coverage: str
    prompt: str
    expected_skill_refs: tuple[str, ...]
    expected_terms: tuple[str, ...]
    expected_domain_profiles: tuple[str, ...]
    forbidden_domain_profiles: tuple[str, ...]


@dataclass(frozen=True)
class SubjectEvalFinding:
    case_id: str
    code: str
    message: str


def load_subject_eval_cases(case_dir: Path) -> list[SubjectEvalCase]:
    case_dir = Path(case_dir)
    cases: list[SubjectEvalCase] = []
    for path in sorted(case_dir.glob("*.yaml"), key=lambda item: item.name):
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
        if not isinstance(payload, dict):
            raise ValueError(f"{path} must contain a YAML mapping")
        _validate_case_payload(path, payload)
        cases.append(
            SubjectEvalCase(
                id=payload["id"],
                subject=payload["subject"],
                coverage=payload["coverage"],
                prompt=payload["prompt"],
                expected_skill_refs=tuple(payload["expected_skill_refs"]),
                expected_terms=tuple(payload["expected_terms"]),
                expected_domain_profiles=tuple(payload["expected_domain_profiles"]),
                forbidden_domain_profiles=tuple(payload["forbidden_domain_profiles"]),
            )
        )
    return cases


def audit_subject_eval_cases(root: Path, case_dir: Path | None = None) -> list[SubjectEvalFinding]:
    root = Path(root).resolve()
    case_dir = Path(case_dir) if case_dir is not None else root / "evals" / "subject-specialization" / "cases"
    findings: list[SubjectEvalFinding] = []
    for case in load_subject_eval_cases(case_dir):
        findings.extend(_audit_case(root, case))
    return findings


def _validate_case_payload(path: Path, payload: dict[str, Any]) -> None:
    missing = [field for field in REQUIRED_FIELDS if field not in payload]
    if missing:
        raise ValueError(f"{path} missing required field(s): {', '.join(missing)}")

    for field in STRING_FIELDS:
        value = payload[field]
        if not isinstance(value, str) or not value.strip():
            raise ValueError(f"{path} field {field} must be a non-empty string")

    for field in STRING_LIST_FIELDS:
        value = payload[field]
        if not isinstance(value, list) or not all(isinstance(item, str) and item.strip() for item in value):
            raise ValueError(f"{path} field {field} must be a list of non-empty strings")


def _audit_case(root: Path, case: SubjectEvalCase) -> list[SubjectEvalFinding]:
    with tempfile.TemporaryDirectory(prefix=f"qiongli-subject-eval-{case.id}-") as tmp_dir:
        package_root = Path(tmp_dir) / "qiongli-workflow"
        materialize_subject_package(
            MaterializeOptions(
                source=root,
                out=package_root,
                subject=case.subject,
                flavor="full",
                coverage=case.coverage,
            )
        )

        findings: list[SubjectEvalFinding] = []
        registry_ids = _load_registry_ids(package_root)
        domain_profiles = {path.name for path in (package_root / "skills" / "domain-profiles").glob("*.yaml")}
        skills_text = _load_skills_text(package_root)

        for expected in case.expected_skill_refs:
            if expected not in registry_ids:
                findings.append(
                    SubjectEvalFinding(
                        case_id=case.id,
                        code="missing-expected-skill",
                        message=f"materialized registry is missing expected skill ref: {expected}",
                    )
                )

        for expected in case.expected_domain_profiles:
            if expected not in domain_profiles:
                findings.append(
                    SubjectEvalFinding(
                        case_id=case.id,
                        code="missing-expected-profile",
                        message=f"materialized domain profiles are missing expected profile: {expected}",
                    )
                )

        for forbidden in case.forbidden_domain_profiles:
            if forbidden in domain_profiles:
                findings.append(
                    SubjectEvalFinding(
                        case_id=case.id,
                        code="forbidden-profile-present",
                        message=f"materialized domain profiles include forbidden profile: {forbidden}",
                    )
                )

        for expected in case.expected_terms:
            if expected.lower() not in skills_text:
                findings.append(
                    SubjectEvalFinding(
                        case_id=case.id,
                        code="missing-expected-term",
                        message=f"materialized skill markdown is missing expected term: {expected}",
                    )
                )

        return findings


def _load_registry_ids(package_root: Path) -> set[str]:
    registry_path = package_root / "skills" / "registry.yaml"
    payload = yaml.safe_load(registry_path.read_text(encoding="utf-8")) or {}
    skills = payload.get("skills")
    if not isinstance(skills, list):
        return set()
    return {entry["id"] for entry in skills if isinstance(entry, dict) and isinstance(entry.get("id"), str)}


def _load_skills_text(package_root: Path) -> str:
    return "\n".join(
        path.read_text(encoding="utf-8").lower()
        for path in sorted((package_root / "skills").glob("**/*.md"))
        if path.is_file()
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Audit Qiongli subject specialization eval cases.")
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--case-dir", type=Path, help="Directory containing subject eval case YAML files.")
    parser.add_argument("--json", action="store_true", help="Emit findings as JSON.")
    args = parser.parse_args(argv)

    try:
        findings = audit_subject_eval_cases(args.root, args.case_dir)
    except ValueError as exc:
        print(f"[FAIL] subject eval case audit: {exc}", file=sys.stderr)
        return 2

    if args.json:
        print(json.dumps([asdict(finding) for finding in findings], indent=2, sort_keys=True))
    else:
        for finding in findings:
            print(f"{finding.case_id}: {finding.code}: {finding.message}")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
