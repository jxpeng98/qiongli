#!/usr/bin/env python3
"""Simple eval runner for qiongli golden tests.

Usage:
    python evals/runner/run_eval.py evals/cases/sr-social-media-mental-health.yaml

This runner validates that skill outputs match expected structure.
It does NOT execute skills — it checks existing outputs against expectations.
"""
from __future__ import annotations

import csv
import hashlib
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tooling.scripts.audit_citation_risk import (  # noqa: E402
    CITABLE_EVIDENCE_TYPES,
    audit_citation_integrity,
)
from tooling.scripts.validate_capability_contract import (  # noqa: E402
    _load_json,
    validate_instance,
)


SUPPORTED_ASSERTIONS = {
    "contains_all",
    "contains_any",
    "schema",
    "field_constraint",
    "count_conservation",
    "cross_artifact_consistency",
    "locator_syntax",
    "citation_identity",
    "file_digest",
}
ASSERTION_KEYS = {
    "contains_all": {"type", "values"},
    "contains_any": {"type", "values"},
    "schema": {"type", "schema"},
    "field_constraint": {"type", "field", "allowed_values"},
    "count_conservation": {"type", "total", "parts"},
    "cross_artifact_consistency": {
        "type",
        "field",
        "other_artifact",
        "other_field",
        "relation",
    },
    "locator_syntax": {"type", "field"},
    "citation_identity": {"type", "bibliography"},
    "file_digest": {"type", "sha256"},
}
SUPPORTED_ARTIFACT_SUFFIXES = (".md", ".py", ".r")
SUPPORTED_SCHEMA_CONSTRAINTS = {
    "$ref",
    "oneOf",
    "const",
    "enum",
    "type",
    "required",
    "properties",
    "additionalProperties",
    "minProperties",
    "minItems",
    "maxItems",
    "uniqueItems",
    "items",
    "minLength",
    "maxLength",
    "pattern",
    "format",
    "minimum",
    "maximum",
}
LOCATOR_PATTERN = re.compile(
    r"(?:p\.\s*\d+|pp\.\s*\d+\s*[-–]\s*\d+|"
    r"[A-Za-z0-9][A-Za-z0-9_.-]*:\S(?:.*\S)?)",
    flags=re.IGNORECASE,
)


def load_case(path: str | Path) -> object:
    with Path(path).open(encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def _clean_string(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{label} must be a non-empty string")
    return value.strip()


def _clean_string_list(value: object, label: str) -> list[str]:
    if not isinstance(value, list) or not value:
        raise ValueError(f"{label} must be a non-empty list of strings")
    cleaned = [_clean_string(item, label) for item in value]
    if len(cleaned) != len(set(cleaned)):
        raise ValueError(f"{label} must not contain duplicates")
    return cleaned


def _normalize_assertion(
    assertion: object, index: int
) -> tuple[dict[str, Any] | None, str | None, bool]:
    if not isinstance(assertion, dict):
        return None, f"assertion {index} must be an object", False

    raw_type = assertion.get("type")
    if not isinstance(raw_type, str) or not raw_type.strip():
        return None, f"assertion {index} requires a type", False
    assertion_type = raw_type.strip()
    if assertion_type not in SUPPORTED_ASSERTIONS:
        return None, f"assertion {index} has unknown type: {assertion_type}", True

    expected_keys = ASSERTION_KEYS[assertion_type]
    missing = expected_keys - assertion.keys()
    extra = assertion.keys() - expected_keys
    if missing or extra:
        details = []
        if missing:
            details.append("missing " + ", ".join(sorted(missing)))
        if extra:
            details.append("unexpected " + ", ".join(sorted(map(str, extra))))
        return None, f"assertion {index} fields: {'; '.join(details)}", False

    try:
        normalized: dict[str, Any] = {"type": assertion_type}
        if assertion_type in {"contains_all", "contains_any"}:
            normalized["values"] = _clean_string_list(assertion["values"], "values")
        elif assertion_type == "schema":
            normalized["schema"] = _clean_string(assertion["schema"], "schema")
        elif assertion_type == "field_constraint":
            normalized["field"] = _clean_string(assertion["field"], "field")
            normalized["allowed_values"] = _clean_string_list(
                assertion["allowed_values"], "allowed_values"
            )
        elif assertion_type == "count_conservation":
            normalized["total"] = _clean_string(assertion["total"], "total")
            normalized["parts"] = _clean_string_list(assertion["parts"], "parts")
            if normalized["total"] in normalized["parts"]:
                raise ValueError("total must not also appear in parts")
        elif assertion_type == "cross_artifact_consistency":
            normalized["field"] = _clean_string(assertion["field"], "field")
            normalized["other_artifact"] = _clean_string(
                assertion["other_artifact"], "other_artifact"
            )
            normalized["other_field"] = _clean_string(
                assertion["other_field"], "other_field"
            )
            relation = _clean_string(assertion["relation"], "relation")
            if relation not in {"equal", "subset"}:
                raise ValueError("relation must be equal or subset")
            normalized["relation"] = relation
        elif assertion_type == "locator_syntax":
            normalized["field"] = _clean_string(assertion["field"], "field")
        elif assertion_type == "citation_identity":
            normalized["bibliography"] = _clean_string(
                assertion["bibliography"], "bibliography"
            )
        else:
            digest = _clean_string(assertion["sha256"], "sha256").lower()
            if re.fullmatch(r"[0-9a-f]{64}", digest) is None:
                raise ValueError(
                    "sha256 must contain exactly 64 hexadecimal characters"
                )
            normalized["sha256"] = digest
    except ValueError as exc:
        return None, f"assertion {index}: {exc}", False
    return normalized, None, False


def _resolve_relative(root: Path, reference: str) -> Path:
    relative = Path(reference)
    if relative.is_absolute():
        raise ValueError(f"path must be relative: {reference}")
    resolved_root = root.resolve()
    resolved = (resolved_root / relative).resolve()
    if not resolved.is_relative_to(resolved_root):
        raise ValueError(f"path escapes its root: {reference}")
    return resolved


def _require_file(path: Path, label: str) -> Path:
    if not path.is_file():
        raise ValueError(f"{label} not found: {path.name}")
    return path


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _read_csv_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        raw_fields = reader.fieldnames
        if not raw_fields:
            raise ValueError(f"CSV has no header: {path.name}")
        fields = [_clean_string(field, "CSV header") for field in raw_fields]
        if len(fields) != len(set(fields)):
            raise ValueError(f"CSV has duplicate headers: {path.name}")

        rows: list[dict[str, str]] = []
        for row_number, row in enumerate(reader, start=2):
            if None in row or any(value is None for value in row.values()):
                raise ValueError(
                    f"CSV row {row_number} has the wrong number of fields"
                )
            rows.append(
                {
                    field: str(row[raw_field]).strip()
                    for raw_field, field in zip(raw_fields, fields, strict=True)
                }
            )
    if not rows:
        raise ValueError(f"CSV has no data rows: {path.name}")
    return rows


def _read_csv_column(
    path: Path, field: str, *, allow_blank: bool = False
) -> list[str]:
    rows = _read_csv_rows(path)
    if field not in rows[0]:
        raise ValueError(f"CSV field not found: {field}")
    values = [row[field] for row in rows]
    if allow_blank:
        values = [value for value in values if value]
        if not values:
            raise ValueError(f"CSV field has no applicable values: {field}")
    elif any(not value for value in values):
        raise ValueError(f"CSV field contains an empty value: {field}")
    return values


def _extract_count(content: str, label: str) -> int:
    pattern = re.compile(
        rf"^\s*(?:[-*+]\s+)?(?:\*\*)?{re.escape(label)}\s*:\s*"
        rf"n\s*=\s*(\d+)\s*(?:\*\*)?\s*$",
        flags=re.IGNORECASE | re.MULTILINE,
    )
    matches = pattern.findall(content)
    if len(matches) != 1:
        raise ValueError(f"count label must occur exactly once: {label}")
    return int(matches[0])


def _load_structured_artifact(path: Path) -> object:
    suffix = path.suffix.casefold()
    if suffix == ".json":
        return _load_json(path)
    if suffix in {".yaml", ".yml"}:
        return yaml.safe_load(_read_text(path))
    raise ValueError("schema assertions require a JSON or YAML artifact")


def _check_assertion(
    artifact_path: Path,
    assertion: dict[str, Any],
    *,
    case_root: Path,
    output_root: Path,
) -> list[str]:
    assertion_type = assertion["type"]
    if assertion_type in {"contains_all", "contains_any"}:
        folded = _read_text(artifact_path).casefold()
        values = assertion["values"]
        if assertion_type == "contains_all":
            return [
                f"Missing: {value}"
                for value in values
                if value.casefold() not in folded
            ]
        if any(value.casefold() in folded for value in values):
            return []
        return [f"Missing any of: {', '.join(values)}"]

    if assertion_type == "schema":
        schema_path = _require_file(
            _resolve_relative(case_root, assertion["schema"]), "schema"
        )
        schema = _load_json(schema_path)
        if not isinstance(schema, dict):
            raise ValueError("schema must be a JSON object")
        if not SUPPORTED_SCHEMA_CONSTRAINTS.intersection(schema):
            raise ValueError("schema has no supported constraints")
        value = _load_structured_artifact(artifact_path)
        return [
            f"Schema: {failure}" for failure in validate_instance(value, schema)
        ]

    if assertion_type == "field_constraint":
        values = _read_csv_column(artifact_path, assertion["field"])
        allowed = set(assertion["allowed_values"])
        invalid = sorted({value for value in values if value not in allowed})
        if not invalid:
            return []
        return [
            f"Disallowed {assertion['field']} value(s): {', '.join(invalid)}"
        ]

    if assertion_type == "count_conservation":
        content = _read_text(artifact_path)
        total = _extract_count(content, assertion["total"])
        parts = [
            _extract_count(content, label) for label in assertion["parts"]
        ]
        if total == sum(parts):
            return []
        return [
            f"Count mismatch: {assertion['total']}={total}, parts sum={sum(parts)}"
        ]

    if assertion_type == "cross_artifact_consistency":
        primary = Counter(
            _read_csv_column(artifact_path, assertion["field"])
        )
        other_path = _require_file(
            _resolve_relative(output_root, assertion["other_artifact"]),
            "other artifact",
        )
        other = Counter(
            _read_csv_column(other_path, assertion["other_field"])
        )
        relation = assertion["relation"]
        matches = (
            primary == other if relation == "equal" else not (primary - other)
        )
        if matches:
            return []
        return [f"Cross-artifact {relation} relation failed"]

    if assertion_type == "locator_syntax":
        values = _read_csv_column(
            artifact_path, assertion["field"], allow_blank=True
        )
        invalid = sorted(
            {
                value
                for value in values
                if LOCATOR_PATTERN.fullmatch(value) is None
            }
        )
        if not invalid:
            return []
        return [f"Invalid locator(s): {', '.join(invalid)}"]

    if assertion_type == "citation_identity":
        rows = _read_csv_rows(artifact_path)
        required_fields = {"evidence_type", "source_id"}
        if not required_fields.issubset(rows[0]):
            missing = ", ".join(
                sorted(required_fields - rows[0].keys())
            )
            raise ValueError(f"citation ledger missing field(s): {missing}")
        if not any(
            row["evidence_type"] in CITABLE_EVIDENCE_TYPES for row in rows
        ):
            raise ValueError(
                "citation ledger has no citable paper/theory rows"
            )
        bibliography = _require_file(
            _resolve_relative(output_root, assertion["bibliography"]),
            "bibliography",
        )
        result = audit_citation_integrity(artifact_path, bibliography)
        return [
            f"Citation identity: {error}" for error in result.errors
        ]

    actual = hashlib.sha256(artifact_path.read_bytes()).hexdigest()
    if actual == assertion["sha256"]:
        return []
    return [
        f"SHA-256 mismatch: expected {assertion['sha256']}, got {actual}"
    ]


def run_case(
    case_path: str | Path, output_dir: str | Path | None = None
) -> bool:
    case_file = Path(case_path)
    try:
        case = load_case(case_file)
    except (OSError, UnicodeError, yaml.YAMLError) as exc:
        print(f"\n[FAIL] Unable to load eval case: {exc}")
        return False

    if not isinstance(case, dict):
        print("\n[FAIL] Eval case must be a YAML object")
        return False

    case_id = case.get("case_id")
    pipeline = case.get("pipeline")
    if not isinstance(case_id, str) or not case_id.strip():
        print("\n[FAIL] Eval case requires a non-empty case_id")
        return False
    if not isinstance(pipeline, str) or not pipeline.strip():
        print(
            f"\n[FAIL] Eval case {case_id} requires a non-empty pipeline"
        )
        return False

    print(f"\n{'=' * 60}")
    print(f"Eval Case: {case_id}")
    print(f"Pipeline:  {pipeline}")
    print(f"{'=' * 60}")

    if case.get("schema_version") != "1.0":
        print(
            "  [case] BLOCKED — unsupported schema_version: "
            f"{case.get('schema_version')!r}"
        )
        return False

    expected_outputs = case.get("expected_outputs")
    if not isinstance(expected_outputs, dict) or not expected_outputs:
        print(
            "  [case] BLOCKED — expected_outputs must be a non-empty object"
        )
        return False

    if output_dir is None:
        case_input = case.get("input")
        topic = (
            case_input.get("topic")
            if isinstance(case_input, dict)
            else None
        )
        if not isinstance(topic, str) or not topic.strip():
            print(
                "  [case] BLOCKED — input.topic must be a non-empty string"
            )
            return False
        topic_slug = re.sub(r"[^a-z0-9]+", "_", topic.lower())[:40]
        output_dir = Path("RESEARCH") / topic_slug

    output_root = Path(output_dir).resolve()
    case_root = case_file.resolve().parent
    total = len(expected_outputs)
    passed = 0
    failed = 0
    skipped = 0
    required_missing = 0
    executed_assertions = 0
    failed_assertions = 0
    blocked_assertions = 0
    unknown_validation_types = 0

    for skill_id, expected in expected_outputs.items():
        if not isinstance(expected, dict):
            failed += 1
            blocked_assertions += 1
            print(
                f"  [{skill_id}] BLOCKED — expected output must be an object"
            )
            continue

        fatal_errors: list[str] = []
        blocked_reasons: list[str] = []
        artifact = expected.get("artifact")
        required = expected.get("required")
        raw_assertions = expected.get("assertions")

        if "must_contain" in expected or "validation" in expected:
            blocked_reasons.append(
                "legacy must_contain/validation fields are unsupported"
            )
            blocked_assertions += 1
        if not isinstance(artifact, str) or not artifact.strip():
            fatal_errors.append(
                "artifact must be a non-empty relative path"
            )
            blocked_assertions += 1
        else:
            artifact = artifact.strip()
        if type(required) is not bool:
            fatal_errors.append("required must be true or false")
            blocked_assertions += 1

        assertions: list[tuple[int, dict[str, Any]]] = []
        if not isinstance(raw_assertions, list) or not raw_assertions:
            fatal_errors.append("assertions must be a non-empty list")
            blocked_assertions += 1
        else:
            for index, assertion in enumerate(raw_assertions):
                normalized, error, unknown = _normalize_assertion(
                    assertion, index
                )
                if error is not None:
                    blocked_reasons.append(error)
                    blocked_assertions += 1
                    unknown_validation_types += int(unknown)
                elif normalized is not None:
                    assertions.append((index, normalized))

        if fatal_errors:
            blocked_assertions += len(assertions)
            failed += 1
            print(f"  [{skill_id}] BLOCKED")
            for error in fatal_errors + blocked_reasons:
                print(f"    x {error}")
            continue

        try:
            artifact_path = _resolve_relative(output_root, artifact)
        except ValueError as exc:
            blocked_assertions += len(assertions)
            failed += 1
            print(f"  [{skill_id}] BLOCKED")
            for error in blocked_reasons + [str(exc)]:
                print(f"    x {error}")
            continue

        if not artifact_path.exists():
            if blocked_reasons:
                failed += 1
                print(f"  [{skill_id}] BLOCKED")
                for error in blocked_reasons:
                    print(f"    x {error}")
            elif required:
                failed += 1
                required_missing += 1
                print(
                    f"  [{skill_id}] FAIL — required artifact not found: "
                    f"{artifact}"
                )
            else:
                skipped += 1
                print(
                    f"  [{skill_id}] SKIP — optional artifact not found: "
                    f"{artifact}"
                )
            continue

        if artifact_path.is_dir():
            files = sorted(
                path
                for path in artifact_path.iterdir()
                if path.is_file()
                and path.suffix.casefold() in SUPPORTED_ARTIFACT_SUFFIXES
                and path.resolve().is_relative_to(output_root)
            )
            if not files:
                if required:
                    failed += 1
                    required_missing += 1
                    print(
                        f"  [{skill_id}] FAIL — required artifact "
                        f"directory empty: {artifact}"
                    )
                else:
                    skipped += 1
                    print(
                        f"  [{skill_id}] SKIP — optional artifact "
                        f"directory empty: {artifact}"
                    )
                continue
            artifact_path = files[0].resolve()
        elif not artifact_path.is_file():
            blocked_assertions += len(assertions)
            failed += 1
            print(
                f"  [{skill_id}] BLOCKED — artifact is not a regular "
                f"file: {artifact}"
            )
            continue

        try:
            artifact_empty = artifact_path.stat().st_size == 0
        except OSError as exc:
            blocked_assertions += len(assertions)
            failed += 1
            print(f"  [{skill_id}] BLOCKED — artifact unreadable: {exc}")
            continue
        if required and artifact_empty:
            failed += 1
            required_missing += 1
            print(
                f"  [{skill_id}] FAIL — required artifact is empty: "
                f"{artifact}"
            )
            continue

        failures: list[str] = []
        for index, assertion in assertions:
            try:
                assertion_failures = _check_assertion(
                    artifact_path,
                    assertion,
                    case_root=case_root,
                    output_root=output_root,
                )
            except (
                OSError,
                UnicodeError,
                ValueError,
                TypeError,
                csv.Error,
                yaml.YAMLError,
                re.error,
            ) as exc:
                blocked_assertions += 1
                blocked_reasons.append(
                    f"assertion {index} ({assertion['type']}) "
                    f"unavailable: {exc}"
                )
                continue
            executed_assertions += 1
            if assertion_failures:
                failed_assertions += 1
                failures.extend(assertion_failures)

        if blocked_reasons:
            failed += 1
            print(f"  [{skill_id}] BLOCKED")
            for error in blocked_reasons + failures:
                print(f"    x {error}")
        elif failures:
            failed += 1
            print(f"  [{skill_id}] FAIL")
            for failure in failures:
                print(f"    x {failure}")
        else:
            passed += 1
            print(f"  [{skill_id}] PASS")

    print(f"\n{'-' * 40}")
    print(
        f"Results: {passed}/{total} passed, {failed} failed, "
        f"{skipped} skipped"
    )
    print(
        "Truth: "
        f"required_missing={required_missing}, "
        f"executed_assertions={executed_assertions}, "
        f"failed_assertions={failed_assertions}, "
        f"blocked_assertions={blocked_assertions}, "
        f"unknown_validation_types={unknown_validation_types}"
    )
    if executed_assertions == 0:
        print("  [case] BLOCKED — no assertions executed")
    return (
        required_missing == 0
        and executed_assertions > 0
        and failed_assertions == 0
        and blocked_assertions == 0
        and unknown_validation_types == 0
    )


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(
            "Usage: python evals/runner/run_eval.py "
            "<case.yaml> [output_dir]"
        )
        sys.exit(1)

    requested_case = sys.argv[1]
    requested_output = sys.argv[2] if len(sys.argv) > 2 else None
    success = run_case(requested_case, requested_output)
    sys.exit(0 if success else 1)
