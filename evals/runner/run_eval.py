#!/usr/bin/env python3
"""Simple eval runner for qiongli golden tests.

Usage:
    python evals/runner/run_eval.py evals/cases/sr-social-media-mental-health.yaml

This runner validates that skill outputs match expected structure.
It does NOT execute skills — it checks existing outputs against expectations.
"""
import sys
import os
import yaml
import re

SUPPORTED_ASSERTIONS = {"contains_all", "contains_any"}
SUPPORTED_ARTIFACT_SUFFIXES = (".md", ".py", ".r", ".R")


def load_case(path: str) -> object:
    with open(path, encoding="utf-8") as f:
        return yaml.safe_load(f)


def check_assertion(content: str, assertion_type: str, values: list[str]) -> list[str]:
    folded = content.casefold()
    if assertion_type == "contains_all":
        return [f"Missing: {value}" for value in values if value.casefold() not in folded]
    if any(value.casefold() in folded for value in values):
        return []
    return [f"Missing any of: {', '.join(values)}"]


def run_case(case_path: str, output_dir: str = None) -> bool:
    try:
        case = load_case(case_path)
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
        print(f"\n[FAIL] Eval case {case_id} requires a non-empty pipeline")
        return False

    print(f"\n{'='*60}")
    print(f"Eval Case: {case_id}")
    print(f"Pipeline:  {pipeline}")
    print(f"{'='*60}")

    if case.get("schema_version") != "1.0":
        print(f"  [case] BLOCKED — unsupported schema_version: {case.get('schema_version')!r}")
        return False

    expected_outputs = case.get("expected_outputs")
    if not isinstance(expected_outputs, dict) or not expected_outputs:
        print("  [case] BLOCKED — expected_outputs must be a non-empty object")
        return False

    if output_dir is None:
        case_input = case.get("input")
        topic = case_input.get("topic") if isinstance(case_input, dict) else None
        if not isinstance(topic, str) or not topic.strip():
            print("  [case] BLOCKED — input.topic must be a non-empty string")
            return False
        topic_slug = re.sub(r"[^a-z0-9]+", "_", topic.lower())[:40]
        output_dir = f"RESEARCH/{topic_slug}"

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
            print(f"  [{skill_id}] BLOCKED — expected output must be an object")
            continue

        config_errors = []
        artifact = expected.get("artifact")
        required = expected.get("required")
        raw_assertions = expected.get("assertions")

        if "must_contain" in expected or "validation" in expected:
            config_errors.append("legacy must_contain/validation fields are unsupported")
            blocked_assertions += 1
        if not isinstance(artifact, str) or not artifact.strip() or os.path.isabs(artifact):
            config_errors.append("artifact must be a non-empty relative path")
            blocked_assertions += 1
        if type(required) is not bool:
            config_errors.append("required must be true or false")
            blocked_assertions += 1

        assertions = []
        if not isinstance(raw_assertions, list) or not raw_assertions:
            config_errors.append("assertions must be a non-empty list")
            blocked_assertions += 1
        else:
            for index, assertion in enumerate(raw_assertions):
                if not isinstance(assertion, dict):
                    config_errors.append(f"assertion {index} must be an object")
                    blocked_assertions += 1
                    continue
                assertion_type = assertion.get("type")
                values = assertion.get("values")
                if not isinstance(assertion_type, str) or not assertion_type:
                    config_errors.append(f"assertion {index} requires a type")
                    blocked_assertions += 1
                    continue
                if assertion_type not in SUPPORTED_ASSERTIONS:
                    config_errors.append(f"assertion {index} has unknown type: {assertion_type}")
                    blocked_assertions += 1
                    unknown_validation_types += 1
                    continue
                if (
                    not isinstance(values, list)
                    or not values
                    or not all(isinstance(value, str) and value.strip() for value in values)
                ):
                    config_errors.append(f"assertion {index} values must be non-empty strings")
                    blocked_assertions += 1
                    continue
                assertions.append((assertion_type, [value.strip() for value in values]))

        if config_errors:
            failed += 1
            print(f"  [{skill_id}] BLOCKED")
            for error in config_errors:
                print(f"    x {error}")
            continue

        artifact_path = os.path.join(output_dir, artifact)
        if not os.path.exists(artifact_path):
            if required:
                failed += 1
                required_missing += 1
                print(f"  [{skill_id}] FAIL — required artifact not found: {artifact}")
            else:
                skipped += 1
                print(f"  [{skill_id}] SKIP — optional artifact not found: {artifact}")
            continue

        try:
            if os.path.isdir(artifact_path):
                files = sorted(
                    filename
                    for filename in os.listdir(artifact_path)
                    if filename.endswith(SUPPORTED_ARTIFACT_SUFFIXES)
                )
                if not files:
                    if required:
                        failed += 1
                        required_missing += 1
                        print(f"  [{skill_id}] FAIL — required artifact directory empty: {artifact}")
                    else:
                        skipped += 1
                        print(f"  [{skill_id}] SKIP — optional artifact directory empty: {artifact}")
                    continue
                artifact_path = os.path.join(artifact_path, files[0])
            with open(artifact_path, encoding="utf-8") as f:
                content = f.read()
        except (OSError, UnicodeError) as exc:
            failed += 1
            blocked_assertions += len(assertions)
            print(f"  [{skill_id}] BLOCKED — artifact unreadable: {exc}")
            continue

        failures = []
        for assertion_type, values in assertions:
            executed_assertions += 1
            assertion_failures = check_assertion(content, assertion_type, values)
            if assertion_failures:
                failed_assertions += 1
                failures.extend(assertion_failures)

        if failures:
            failed += 1
            print(f"  [{skill_id}] FAIL")
            for f in failures:
                print(f"    x {f}")
        else:
            passed += 1
            print(f"  [{skill_id}] PASS")

    print(f"\n{'-'*40}")
    print(f"Results: {passed}/{total} passed, {failed} failed, {skipped} skipped")
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
        print("Usage: python evals/runner/run_eval.py <case.yaml> [output_dir]")
        sys.exit(1)

    case_path = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else None
    success = run_case(case_path, output_dir)
    sys.exit(0 if success else 1)
