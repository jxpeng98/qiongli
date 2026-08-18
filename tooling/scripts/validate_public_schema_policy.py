#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path, PurePosixPath
from typing import Any, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tooling.scripts.validate_arc_201_adrs import is_canonical_repository_path


DEFAULT_POLICY = REPO_ROOT / "tooling" / "architecture" / "public-schema-policy.json"
SCHEMA_DRAFT = "https://json-schema.org/draft/2020-12/schema"
ROOT_KEYS = {
    "schema_version",
    "record_type",
    "branch",
    "authority",
    "compatibility_classes",
    "contracts",
}
AUTHORITY_KEYS = {
    "language",
    "json_schema_draft",
    "generated_contract_role",
    "golden_fixture_owner",
}
CONTRACT_KEYS = {
    "id",
    "boundary",
    "baseline",
    "rust_sources",
    "consumers",
    "fixtures",
    "changes",
}
BASELINE_KEYS = {"adopted_at", "authority_state", "version_sources"}
CHANGE_KEYS = {
    "change_id",
    "contract_id",
    "from_version",
    "to_version",
    "classification",
    "rust_sources",
    "generated_schema",
    "golden_fixtures",
    "consumer_checks",
    "migration_path",
    "removal_gate",
}
COMPATIBILITY_CLASSES = (
    "additive",
    "migratable-breaking",
    "unsupported-breaking",
)
EXPECTED_CONTRACTS = (
    "app-ipc",
    "mcp-tools",
    "public-cli-json",
)
EXPECTED_BASELINE_STATES = {
    "app-ipc": "split-rust-zod",
    "mcp-tools": "checked-in-json-schema",
    "public-cli-json": "rust-serialize-without-generated-schema",
}
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9._/-]{0,127}$")


class PublicSchemaPolicyError(ValueError):
    pass


def load_policy(path: Path = DEFAULT_POLICY) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise PublicSchemaPolicyError(f"cannot load {path}: {error}") from error
    if not isinstance(value, dict):
        raise PublicSchemaPolicyError(f"{path} must contain a JSON object")
    return value


def resolve_repository_file(repo_root: Path, relative: str) -> Path:
    if not is_canonical_repository_path(relative):
        raise PublicSchemaPolicyError(
            "path must be a canonical repository-relative POSIX path"
        )
    root = repo_root.resolve(strict=True)
    candidate = repo_root
    for part in PurePosixPath(relative).parts:
        candidate = candidate / part
        if candidate.is_symlink():
            raise PublicSchemaPolicyError("path must not contain a symbolic link")
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, ValueError, RuntimeError) as error:
        raise PublicSchemaPolicyError(
            "path must resolve inside the repository"
        ) from error
    if not resolved.is_file():
        raise PublicSchemaPolicyError("path must resolve to a regular file")
    return resolved


def _exact_keys(value: object, keys: set[str], label: str, errors: list[str]) -> bool:
    if not isinstance(value, dict):
        errors.append(f"{label} must be an object")
        return False
    if set(value) != keys:
        errors.append(f"{label} must contain exactly {sorted(keys)}")
    return True


def _nonempty_text(value: object) -> bool:
    return isinstance(value, str) and 0 < len(value.strip()) <= 256


def _validate_path(
    repo_root: Path,
    value: object,
    label: str,
    errors: list[str],
) -> Path | None:
    if not isinstance(value, str):
        errors.append(f"{label} must be a string path")
        return None
    try:
        return resolve_repository_file(repo_root, value)
    except PublicSchemaPolicyError as error:
        errors.append(f"{label}: {error}")
        return None


def _validate_path_list(
    repo_root: Path,
    value: object,
    label: str,
    errors: list[str],
) -> None:
    if not isinstance(value, list) or not value:
        errors.append(f"{label} must be a non-empty path array")
        return
    if any(not isinstance(item, str) for item in value):
        errors.append(f"{label} must contain only string paths")
        return
    if len(set(value)) != len(value):
        errors.append(f"{label} must not contain duplicate paths")
    for index, item in enumerate(value):
        _validate_path(repo_root, item, f"{label}[{index}]", errors)


def _validate_generated_schema(
    repo_root: Path,
    value: object,
    label: str,
    errors: list[str],
) -> None:
    path = _validate_path(repo_root, value, label, errors)
    if path is None:
        return
    try:
        schema = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        errors.append(f"{label} must contain JSON: {error}")
        return
    if not isinstance(schema, dict) or schema.get("$schema") != SCHEMA_DRAFT:
        errors.append(f"{label} must declare Draft 2020-12 JSON Schema")


def _validate_change(
    repo_root: Path,
    family_id: str,
    value: object,
    index: int,
    prior_versions: dict[str, str],
    seen_change_ids: set[str],
    errors: list[str],
) -> None:
    label = f"{family_id}.changes[{index}]"
    if not _exact_keys(value, CHANGE_KEYS, label, errors):
        return
    assert isinstance(value, dict)

    change_id = value.get("change_id")
    if not isinstance(change_id, str) or not IDENTIFIER.fullmatch(change_id):
        errors.append(f"{label}.change_id is invalid")
    elif change_id in seen_change_ids:
        errors.append(f"{label}.change_id {change_id!r} is duplicated")
    else:
        seen_change_ids.add(change_id)

    contract_id = value.get("contract_id")
    if not isinstance(contract_id, str) or not (
        contract_id == family_id or contract_id.startswith(f"{family_id}/")
    ):
        errors.append(f"{label}.contract_id must belong to {family_id}")
        contract_id = ""

    from_version = value.get("from_version")
    to_version = value.get("to_version")
    if not _nonempty_text(from_version):
        errors.append(f"{label}.from_version must name a predecessor")
    if not _nonempty_text(to_version):
        errors.append(f"{label}.to_version must name a successor")
    if from_version == to_version and isinstance(from_version, str):
        errors.append(f"{label} must change the public version")
    if contract_id and isinstance(from_version, str) and isinstance(to_version, str):
        prior = prior_versions.get(contract_id)
        if prior is not None and from_version != prior:
            errors.append(
                f"{label}.from_version must equal prior to_version {prior!r}"
            )
        prior_versions[contract_id] = to_version

    classification = value.get("classification")
    if classification not in COMPATIBILITY_CLASSES:
        errors.append(f"{label}.classification is unknown")

    _validate_path_list(repo_root, value.get("rust_sources"), f"{label}.rust_sources", errors)
    _validate_generated_schema(
        repo_root, value.get("generated_schema"), f"{label}.generated_schema", errors
    )
    _validate_path_list(
        repo_root, value.get("golden_fixtures"), f"{label}.golden_fixtures", errors
    )
    _validate_path_list(
        repo_root, value.get("consumer_checks"), f"{label}.consumer_checks", errors
    )

    migration_path = value.get("migration_path")
    removal_gate = value.get("removal_gate")
    if classification == "additive":
        if migration_path is not None or removal_gate is not None:
            errors.append(f"{label} additive changes need no breaking-change control")
    elif classification == "migratable-breaking":
        if migration_path is None:
            errors.append(f"{label} requires a migration_path")
        else:
            _validate_path(repo_root, migration_path, f"{label}.migration_path", errors)
        if removal_gate is not None:
            errors.append(f"{label} migratable changes must not claim a removal gate")
    elif classification == "unsupported-breaking":
        if migration_path is not None:
            errors.append(f"{label} unsupported changes cannot claim a migration path")
        if removal_gate is None:
            errors.append(f"{label} requires a separate removal_gate")
        else:
            _validate_path(repo_root, removal_gate, f"{label}.removal_gate", errors)


def validate_policy(repo_root: Path, policy: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    _exact_keys(policy, ROOT_KEYS, "public schema policy", errors)
    if policy.get("schema_version") != "1.0":
        errors.append("public schema policy schema_version must be '1.0'")
    if policy.get("record_type") != "qiongli-public-schema-policy":
        errors.append("public schema policy record_type is invalid")
    if policy.get("branch") != "2.x":
        errors.append("public schema policy must be bound to branch '2.x'")

    authority = policy.get("authority")
    if _exact_keys(authority, AUTHORITY_KEYS, "authority", errors):
        assert isinstance(authority, dict)
        expected = {
            "language": "rust",
            "json_schema_draft": SCHEMA_DRAFT,
            "generated_contract_role": "derived",
            "golden_fixture_owner": "rust",
        }
        if authority != expected:
            errors.append("authority must declare the exact Rust-owned generated contract")

    classes = policy.get("compatibility_classes")
    if not isinstance(classes, list) or tuple(classes) != COMPATIBILITY_CLASSES:
        errors.append(
            "compatibility_classes must contain additive, migratable-breaking, "
            "and unsupported-breaking exactly once and in order"
        )

    contracts = policy.get("contracts")
    if not isinstance(contracts, list):
        return [*errors, "public schema policy contracts must be an array"]
    ids = [item.get("id") for item in contracts if isinstance(item, dict)]
    if tuple(ids) != EXPECTED_CONTRACTS:
        errors.append(
            "contracts must contain app-ipc, mcp-tools, and public-cli-json "
            "exactly once and in order"
        )

    seen_change_ids: set[str] = set()
    for index, contract in enumerate(contracts):
        label = f"contracts[{index}]"
        if not _exact_keys(contract, CONTRACT_KEYS, label, errors):
            continue
        assert isinstance(contract, dict)
        family_id = contract.get("id")
        if not isinstance(family_id, str) or family_id not in EXPECTED_CONTRACTS:
            errors.append(f"{label}.id is unknown")
            family_id = f"unknown-{index}"
        if not _nonempty_text(contract.get("boundary")):
            errors.append(f"{label}.boundary must be bounded non-empty text")

        baseline = contract.get("baseline")
        if _exact_keys(baseline, BASELINE_KEYS, f"{family_id}.baseline", errors):
            assert isinstance(baseline, dict)
            adopted_at = baseline.get("adopted_at")
            if not isinstance(adopted_at, str) or adopted_at != "2026-08-18":
                errors.append(f"{family_id}.baseline.adopted_at is invalid")
            expected_state = EXPECTED_BASELINE_STATES.get(family_id)
            if baseline.get("authority_state") != expected_state:
                errors.append(f"{family_id}.baseline.authority_state is invalid")
            _validate_path_list(
                repo_root,
                baseline.get("version_sources"),
                f"{family_id}.baseline.version_sources",
                errors,
            )

        for key in ("rust_sources", "consumers", "fixtures"):
            _validate_path_list(
                repo_root, contract.get(key), f"{family_id}.{key}", errors
            )

        changes = contract.get("changes")
        if not isinstance(changes, list):
            errors.append(f"{family_id}.changes must be an array")
            continue
        prior_versions: dict[str, str] = {}
        for change_index, change in enumerate(changes):
            _validate_change(
                repo_root,
                family_id,
                change,
                change_index,
                prior_versions,
                seen_change_ids,
                errors,
            )
    return errors


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate Rust-owned public schema authority and compatibility records."
    )
    parser.add_argument("--policy", type=Path, default=DEFAULT_POLICY)
    args = parser.parse_args(argv)
    try:
        policy = load_policy(args.policy)
    except PublicSchemaPolicyError as error:
        print(f"[public-schema] {error}", file=sys.stderr)
        return 2
    errors = validate_policy(REPO_ROOT, policy)
    if errors:
        for error in errors:
            print(f"[public-schema] FAIL: {error}", file=sys.stderr)
        print(f"[public-schema] {len(errors)} validation error(s)", file=sys.stderr)
        return 1
    print("[public-schema] PASS: 3 public boundaries and 3 compatibility classes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
