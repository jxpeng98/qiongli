#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, Iterable, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RECORD = REPO_ROOT / "tooling" / "migration" / "2x-branch-point.json"
EXPECTED_VALUES: dict[tuple[str, ...], Any] = {
    ("$schema",): "./2x-branch-point.schema.json",
    ("schema_version",): "1.0",
    ("record_type",): "qiongli-2x-branch-point",
    ("branch",): "2.x",
    ("source", "branch"): "dev",
    ("source", "baseline_commit"): "70c5bd9e5b098fbb61bd60934e8774b2eef44a01",
    ("source", "baseline_tree"): "c17f7612cdd6e69040332442fde7e6d44c0c24a0",
    ("source", "initialization_commit"): "974f3f254040aad064f06cce6b75fc4a9c950456",
    ("source", "initialization_tree"): "c17f7612cdd6e69040332442fde7e6d44c0c24a0",
    ("source", "initialization_tree_changed"): False,
    ("source", "accepted_release_tag"): "v1.19.0-beta.1",
    ("source", "accepted_release_commit"): "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f",
    ("source", "maintenance_branch"): "release/1.x-python",
    ("baseline", "manifest_path"): (
        "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
    ),
    ("baseline", "manifest_sha256"): (
        "77bb7628d43a496c995e4b0a8daf6a624847b62e96948c0461affe89002da131"
    ),
    ("baseline", "corpus_sha256"): (
        "7fdd92894d88b221180e77ad73677cc158147cc861b17ba0245ea54f0127fbe2"
    ),
    ("source_validation", "ci", "run_id"): 29141373649,
    ("source_validation", "ci", "job_count"): 5,
    ("source_validation", "checkout_install_check", "run_id"): 29141373644,
    ("source_validation", "checkout_install_check", "job_count"): 3,
    ("branch_validation", "record_commit"): (
        "69855fd50413ee6809baf21eb345fc0c55721de3"
    ),
    ("branch_validation", "ci", "run_id"): 29142556037,
    ("branch_validation", "ci", "job_count"): 5,
    ("branch_validation", "checkout_install_check", "run_id"): 29142556048,
    ("branch_validation", "checkout_install_check", "job_count"): 3,
    ("protection", "dev", "ruleset_id"): 17189053,
    ("protection", "dev", "name"): "dev protected 1.x handoff",
    ("protection", "native_development", "branch"): "2.x",
    ("protection", "native_development", "ruleset_id"): 18800504,
    ("protection", "native_development", "name"): (
        "2.x protected native development"
    ),
    ("protection", "maintenance", "branch"): "release/1.x-python",
    ("protection", "maintenance", "ruleset_id"): 18797579,
    ("protection", "maintenance", "name"): (
        "release/1.x-python critical-fix-only"
    ),
}
TOP_LEVEL_KEYS = {
    "$schema",
    "schema_version",
    "record_type",
    "branch",
    "source",
    "baseline",
    "source_validation",
    "branch_validation",
    "protection",
    "branch_validation_policy",
}


class BranchPointValidationError(ValueError):
    pass


def load_record(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise BranchPointValidationError(f"cannot load {path}: {error}") from error
    if not isinstance(value, dict):
        raise BranchPointValidationError(f"{path} must contain a JSON object")
    return value


def _nested(record: dict[str, Any], path: tuple[str, ...]) -> Any:
    value: Any = record
    for component in path:
        if not isinstance(value, dict) or component not in value:
            raise KeyError(".".join(path))
        value = value[component]
    return value


def _expect_keys(
    value: Any, expected: Iterable[str], label: str, errors: list[str]
) -> None:
    if not isinstance(value, dict):
        errors.append(f"{label} must be an object")
        return
    expected_set = set(expected)
    actual = set(value)
    if actual != expected_set:
        errors.append(
            f"{label} keys differ: missing={sorted(expected_set - actual)}, "
            f"extra={sorted(actual - expected_set)}"
        )


def _as_object(value: Any) -> dict[str, Any]:
    return value if isinstance(value, dict) else {}


def _git(repo_root: Path, *arguments: str) -> str:
    environment = os.environ.copy()
    for name in (
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_COMMON_DIR",
        "GIT_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_WORK_TREE",
    ):
        environment.pop(name, None)
    result = subprocess.run(
        ["git", "-C", str(repo_root), *arguments],
        check=False,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip()
        raise BranchPointValidationError(
            f"git {' '.join(arguments)} failed: {detail}"
        )
    return result.stdout.strip()


def git_blob_bytes(repo_root: Path, relative: str, ref: str = "HEAD") -> bytes:
    if not relative or Path(relative).is_absolute() or ".." in Path(relative).parts:
        raise BranchPointValidationError("Git blob path must be repository-relative")
    environment = os.environ.copy()
    for name in (
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_COMMON_DIR",
        "GIT_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_WORK_TREE",
    ):
        environment.pop(name, None)
    result = subprocess.run(
        ["git", "-C", str(repo_root), "show", f"{ref}:{relative}"],
        check=False,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise BranchPointValidationError(f"cannot read Git blob {ref}:{relative}: {detail}")
    return result.stdout


def _validate_workflow_pair(
    pair: Any, *, expected_head: str, include_record: bool, errors: list[str]
) -> None:
    expected_keys = {"ci", "checkout_install_check"}
    if include_record:
        expected_keys.add("record_commit")
    _expect_keys(pair, expected_keys, "workflow evidence", errors)
    if not isinstance(pair, dict):
        return
    for name in ("ci", "checkout_install_check"):
        evidence = pair.get(name)
        _expect_keys(
            evidence,
            {"run_id", "head_sha", "job_count", "conclusion", "url"},
            f"workflow evidence {name}",
            errors,
        )
        if not isinstance(evidence, dict):
            continue
        run_id = evidence.get("run_id")
        if evidence.get("head_sha") != expected_head:
            errors.append(f"{name} head_sha does not match its evidenced commit")
        if evidence.get("conclusion") != "success":
            errors.append(f"{name} conclusion must be success")
        if not isinstance(evidence.get("job_count"), int) or evidence["job_count"] < 1:
            errors.append(f"{name} job_count must be a positive integer")
        expected_url = f"https://github.com/jxpeng98/qiongli/actions/runs/{run_id}"
        if evidence.get("url") != expected_url:
            errors.append(f"{name} URL does not match run_id")


def validate_record(repo_root: Path, record: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    _expect_keys(record, TOP_LEVEL_KEYS, "branch-point record", errors)
    for path, expected in EXPECTED_VALUES.items():
        try:
            actual = _nested(record, path)
        except KeyError:
            errors.append(f"missing frozen evidence field {'.'.join(path)}")
            continue
        if actual != expected:
            errors.append(
                f"frozen evidence {'.'.join(path)} must be {expected!r}, "
                f"found {actual!r}"
            )

    source = record.get("source")
    baseline = record.get("baseline")
    protection = record.get("protection")
    policy = record.get("branch_validation_policy")
    branch_validation = record.get("branch_validation")
    _expect_keys(
        source,
        {
            "branch",
            "baseline_commit",
            "baseline_tree",
            "initialization_commit",
            "initialization_tree",
            "initialization_tree_changed",
            "accepted_release_tag",
            "accepted_release_commit",
            "maintenance_branch",
        },
        "source",
        errors,
    )
    _expect_keys(
        baseline,
        {"manifest_path", "manifest_sha256", "corpus_sha256"},
        "baseline",
        errors,
    )
    _expect_keys(
        protection,
        {"development_handoff_policy", "dev", "native_development", "maintenance"},
        "protection",
        errors,
    )
    _expect_keys(
        policy,
        {"required_workflows", "evidence_binding", "ruleset_activation_gate"},
        "branch_validation_policy",
        errors,
    )

    if isinstance(source, dict):
        baseline_commit = source.get("baseline_commit")
        initialization_commit = source.get("initialization_commit")
        record_commit = _as_object(branch_validation).get("record_commit")
        try:
            if _git(repo_root, "rev-parse", f"{baseline_commit}^{{tree}}") != source.get(
                "baseline_tree"
            ):
                errors.append("baseline_tree does not match baseline_commit")
            if _git(
                repo_root, "rev-parse", f"{initialization_commit}^{{tree}}"
            ) != source.get("initialization_tree"):
                errors.append("initialization_tree does not match initialization_commit")
            if _git(repo_root, "rev-parse", f"{initialization_commit}^") != baseline_commit:
                errors.append("initialization_commit is not a child of baseline_commit")
            if _git(repo_root, "rev-parse", f"{record_commit}^") != initialization_commit:
                errors.append("record_commit is not a child of initialization_commit")
            tag_commit = _git(
                repo_root, "rev-parse", f"{source.get('accepted_release_tag')}^{{}}"
            )
            if tag_commit != source.get("accepted_release_commit"):
                errors.append("accepted release tag does not peel to recorded commit")
        except BranchPointValidationError as error:
            errors.append(str(error))

    if isinstance(baseline, dict):
        relative = baseline.get("manifest_path")
        if isinstance(relative, str):
            try:
                payload = git_blob_bytes(repo_root, relative)
                manifest_data = json.loads(payload)
            except (BranchPointValidationError, json.JSONDecodeError) as error:
                errors.append(f"cannot verify baseline manifest: {error}")
            else:
                digest = hashlib.sha256(payload).hexdigest()
                if digest != baseline.get("manifest_sha256"):
                    errors.append("baseline manifest SHA-256 does not match record")
                integrity = _as_object(manifest_data.get("integrity"))
                corpus = integrity.get("corpus_sha256")
                if corpus != baseline.get("corpus_sha256"):
                    errors.append("baseline corpus SHA-256 does not match manifest")

    source_head = _as_object(source).get("baseline_commit")
    branch_head = _as_object(branch_validation).get("record_commit")
    _validate_workflow_pair(
        record.get("source_validation"),
        expected_head=source_head,
        include_record=False,
        errors=errors,
    )
    _validate_workflow_pair(
        branch_validation,
        expected_head=branch_head,
        include_record=True,
        errors=errors,
    )

    handoff = (
        protection.get("development_handoff_policy")
        if isinstance(protection, dict)
        else None
    )
    expected_handoff = {
        "applies_to": ["dev", "2.x"],
        "pull_request_required": True,
        "dismiss_stale_reviews": True,
        "required_review_thread_resolution": True,
        "required_checks_strict": True,
        "required_check_count": 8,
        "deletion_blocked": True,
        "non_fast_forward_blocked": True,
        "bypass_actors": [],
        "current_user_can_bypass": "never",
    }
    if handoff != expected_handoff:
        errors.append("development handoff policy does not match accepted protection")
    if isinstance(protection, dict):
        ruleset_keys = {
            "dev": {"ruleset_id", "name", "enforcement", "url"},
            "native_development": {
                "branch",
                "ruleset_id",
                "name",
                "enforcement",
                "url",
            },
            "maintenance": {
                "branch",
                "ruleset_id",
                "name",
                "enforcement",
                "url",
            },
        }
        for name, expected_keys in ruleset_keys.items():
            ruleset = protection.get(name)
            if not isinstance(ruleset, dict):
                errors.append(f"protection.{name} must be an object")
                continue
            _expect_keys(ruleset, expected_keys, f"protection.{name}", errors)
            ruleset_id = ruleset.get("ruleset_id")
            if ruleset.get("enforcement") != "active":
                errors.append(f"protection.{name} must be active")
            expected_url = f"https://github.com/jxpeng98/qiongli/rules/{ruleset_id}"
            if ruleset.get("url") != expected_url:
                errors.append(f"protection.{name} URL does not match ruleset_id")
    if isinstance(policy, dict) and policy.get("required_workflows") != [
        "CI",
        "Checkout Install Check",
    ]:
        errors.append("branch validation workflows do not match accepted policy")
    if isinstance(policy, dict):
        for name in ("evidence_binding", "ruleset_activation_gate"):
            value = policy.get(name)
            if not isinstance(value, str) or len(value.strip()) < 20:
                errors.append(f"branch_validation_policy.{name} must be descriptive text")

    schema_path = repo_root / "tooling" / "migration" / "2x-branch-point.schema.json"
    try:
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        errors.append(f"cannot load branch-point schema: {error}")
    else:
        if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
            errors.append("branch-point schema must use JSON Schema Draft 2020-12")
        if schema.get("additionalProperties") is not False:
            errors.append("branch-point schema must reject unknown top-level fields")
    return errors


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate the frozen Qiongli 2.x branch-point evidence."
    )
    parser.add_argument("--record", type=Path, default=DEFAULT_RECORD)
    args = parser.parse_args(argv)
    try:
        record = load_record(args.record)
    except BranchPointValidationError as error:
        print(f"[2x-branch-point] {error}", file=sys.stderr)
        return 2
    errors = validate_record(REPO_ROOT, record)
    if errors:
        for error in errors:
            print(f"[2x-branch-point] FAIL: {error}", file=sys.stderr)
        print(
            f"[2x-branch-point] {len(errors)} validation error(s)", file=sys.stderr
        )
        return 1
    print("[2x-branch-point] PASS: branch lineage, baseline, CI, and rulesets agree")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
