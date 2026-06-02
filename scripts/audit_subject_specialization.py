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


REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.subject_materializer import (  # noqa: E402
    MaterializeOptions,
    SubjectDefinition,
    materialize_subject_package,
    validate_subject_catalog,
)
from qiongli.source_layout import RepoLayout  # noqa: E402


SUBJECT_TERMS = {
    "business": ("business", "management", "theory contribution", "doctoral-level journal"),
    "economics": ("identification", "estimand", "robustness", "causal"),
    "accounting": ("accrual", "disclosure", "measurement", "audit"),
    "economics-accounting": ("identification", "disclosure", "measurement", "causal"),
    "finance": ("asset pricing", "risk-adjusted", "look-ahead bias", "finance"),
    "political-economy": ("political mechanism", "institution", "distributional conflict", "policy"),
    "geoeconomics": ("sanctions", "statecraft", "supply chain", "strategic competition"),
}


@dataclass(frozen=True)
class SubjectSpecializationFinding:
    subject: str
    code: str
    message: str


def audit_subject_specialization(root: Path, subjects: list[str] | None = None) -> list[SubjectSpecializationFinding]:
    root = Path(root).resolve()
    catalog = validate_subject_catalog(root)
    subject_ids = subjects if subjects is not None else sorted(subject_id for subject_id in catalog.subjects if subject_id != "core")
    _validate_requested_subjects(subject_ids, catalog.subjects)

    findings: list[SubjectSpecializationFinding] = []
    for subject_id in subject_ids:
        if subject_id == "core":
            continue
        subject = catalog.subjects[subject_id]
        findings.extend(_audit_subject_definition(subject))
        findings.extend(_audit_materialized_outputs(root, subject))
    return findings


def _validate_requested_subjects(subject_ids: list[str], available_subjects: dict[str, SubjectDefinition]) -> None:
    unknown = sorted(set(subject_ids) - set(available_subjects))
    if not unknown:
        return
    available = ", ".join(sorted(available_subjects))
    requested = ", ".join(unknown)
    raise ValueError(f"unknown subject(s): {requested}. Available subjects: {available}")


def _audit_subject_definition(subject: SubjectDefinition) -> list[SubjectSpecializationFinding]:
    findings: list[SubjectSpecializationFinding] = []
    if not subject.domain_profiles:
        findings.append(
            SubjectSpecializationFinding(
                subject=subject.id,
                code="missing-domain-profiles",
                message="non-core subject has no domain profiles",
            )
        )
    if not subject.venue_profiles:
        findings.append(
            SubjectSpecializationFinding(
                subject=subject.id,
                code="missing-venue-profiles",
                message="non-core subject has no venue profiles",
            )
        )
    layer_size = len(subject.skill_overrides) + len(subject.subject_specific_skill_refs)
    if layer_size < 2:
        findings.append(
            SubjectSpecializationFinding(
                subject=subject.id,
                code="thin-subject-layer",
                message="subject layer must define at least two skill overrides or subject-specific skill refs",
            )
        )
    return findings


def _audit_materialized_outputs(root: Path, subject: SubjectDefinition) -> list[SubjectSpecializationFinding]:
    with tempfile.TemporaryDirectory(prefix=f"qiongli-subject-audit-{subject.id}-") as tmp_dir:
        tmp_root = Path(tmp_dir)
        focused = tmp_root / "focused" / "qiongli-workflow"
        complete = tmp_root / "complete" / "qiongli-workflow"

        materialize_subject_package(
            MaterializeOptions(
                source=root,
                out=focused,
                subject=subject.id,
                flavor="full",
                coverage="focused",
            )
        )
        materialize_subject_package(
            MaterializeOptions(
                source=root,
                out=complete,
                subject=subject.id,
                flavor="full",
                coverage="complete",
            )
        )

        findings = _audit_focused_domain_profiles(focused, subject)
        findings.extend(_audit_overlay_subject_terms(root, subject))
        findings.extend(_audit_materialized_subject_terms(complete, subject))
        return findings


def _audit_focused_domain_profiles(
    package_root: Path,
    subject: SubjectDefinition,
) -> list[SubjectSpecializationFinding]:
    profile_root = package_root / "skills" / "domain-profiles"
    present = {path.name for path in sorted(profile_root.glob("*.yaml"))}
    allowed = {f"{profile}.yaml" for profile in subject.domain_profiles}
    unrelated = sorted(present - allowed)
    if not unrelated:
        return []
    return [
        SubjectSpecializationFinding(
            subject=subject.id,
            code="unrelated-focused-profile",
            message=f"focused output includes undeclared domain profiles: {', '.join(unrelated)}",
        )
    ]


def _audit_materialized_subject_terms(
    package_root: Path,
    subject: SubjectDefinition,
) -> list[SubjectSpecializationFinding]:
    expected_terms = SUBJECT_TERMS.get(subject.id)
    if not expected_terms:
        return []

    findings: list[SubjectSpecializationFinding] = []
    generated_skill_paths = _subject_layer_skill_paths(package_root, subject)
    for path in generated_skill_paths:
        text = path.read_text(encoding="utf-8").lower()
        if not any(term in text for term in expected_terms):
            rel_path = path.relative_to(package_root)
            findings.append(
                SubjectSpecializationFinding(
                    subject=subject.id,
                    code="missing-subject-term",
                    message=f"{rel_path} does not include any expected subject terms: {', '.join(expected_terms)}",
                )
            )

    skill_text = "\n".join(
        path.read_text(encoding="utf-8").lower()
        for path in sorted((package_root / "skills").glob("**/*.md"))
        if path.is_file()
    )
    missing_terms = [term for term in expected_terms if term not in skill_text]
    if missing_terms:
        findings.append(
            SubjectSpecializationFinding(
                subject=subject.id,
                code="missing-subject-term",
                message=f"complete output skills are missing expected subject terms: {', '.join(missing_terms)}",
            )
        )
    return findings


def _audit_overlay_subject_terms(root: Path, subject: SubjectDefinition) -> list[SubjectSpecializationFinding]:
    expected_terms = SUBJECT_TERMS.get(subject.id)
    if not expected_terms:
        return []

    findings: list[SubjectSpecializationFinding] = []
    overlay_root = RepoLayout(root).subjects / subject.id
    for override in subject.skill_overrides:
        overlay_rel = override.get("overlay")
        if not isinstance(overlay_rel, str) or not overlay_rel.strip():
            continue
        overlay_path = overlay_root / overlay_rel
        if not overlay_path.is_file():
            continue
        text = overlay_path.read_text(encoding="utf-8").lower()
        if any(term in text for term in expected_terms):
            continue
        rel_path = overlay_path.relative_to(root)
        findings.append(
            SubjectSpecializationFinding(
                subject=subject.id,
                code="missing-subject-term",
                message=f"{rel_path} does not include any expected subject terms: {', '.join(expected_terms)}",
            )
        )
    return findings


def _subject_layer_skill_paths(package_root: Path, subject: SubjectDefinition) -> list[Path]:
    registry_by_id = _materialized_registry_by_id(package_root)
    skill_ids = {
        *(
            str(override.get("skill"))
            for override in subject.skill_overrides
            if isinstance(override.get("skill"), str) and str(override.get("skill")).strip()
        ),
        *subject.subject_specific_skill_refs,
    }
    paths: list[Path] = []
    for skill_id in sorted(skill_ids):
        entry = registry_by_id.get(skill_id)
        if not entry:
            continue
        rel_file = entry.get("file")
        if not isinstance(rel_file, str):
            continue
        path = package_root / rel_file
        if path.is_file():
            paths.append(path)
    return paths


def _materialized_registry_by_id(package_root: Path) -> dict[str, dict[str, Any]]:
    registry_path = package_root / "skills" / "registry.yaml"
    payload = yaml.safe_load(registry_path.read_text(encoding="utf-8")) or {}
    skills = payload.get("skills")
    if not isinstance(skills, list):
        return {}
    registry: dict[str, dict[str, Any]] = {}
    for entry in skills:
        if isinstance(entry, dict) and isinstance(entry.get("id"), str):
            registry[entry["id"]] = entry
    return registry


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Audit Qiongli subject specialization depth and materialized output.")
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--subject", action="append", dest="subjects", help="Subject id to audit. May be repeated.")
    parser.add_argument("--json", action="store_true", help="Emit findings as JSON.")
    args = parser.parse_args(argv)

    try:
        findings = audit_subject_specialization(args.root, args.subjects)
    except ValueError as exc:
        print(f"[FAIL] subject specialization audit: {exc}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps([asdict(finding) for finding in findings], indent=2, sort_keys=True))
    else:
        for finding in findings:
            print(f"{finding.subject}: {finding.code}: {finding.message}")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
