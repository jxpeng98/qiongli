#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any, Mapping, Sequence

from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RECORD = "tooling/migration/ctr-201-inventory.json"
DEFAULT_SCHEMA = "tooling/migration/ctr-201-inventory.schema.json"
EXPECTED_TAG = "v1.19.0-beta.1"
EXPECTED_COMMIT = "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"
EXPECTED_MANIFEST_PATH = (
    "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
)
EXPECTED_MANIFEST_SHA256 = (
    "77bb7628d43a496c995e4b0a8daf6a624847b62e96948c0461affe89002da131"
)
EXPECTED_CORPUS_SHA256 = (
    "7fdd92894d88b221180e77ad73677cc158147cc861b17ba0245ea54f0127fbe2"
)
EXPECTED_CONTENT_TREE_SHA256 = (
    "4659cbcd839c3f8eb3798a64981b7ec2180cf766566fcf439ac892eb32a8a5a8"
)
EXPECTED_CONTENT_FILE_COUNT = 377
EXPECTED_REGISTRY_PATH = "content/mcp-contracts/v2/registry.json"
EXPECTED_REGISTRY_SHA256 = (
    "602d3faf525e2e5c938afb14f1b1d291f528240947b3df6ed9f56baeb73e7020"
)
EXPECTED_SCHEMA_CANONICAL_SHA256 = (
    "5b25a7b8901a1bdd2207f961336b73504a241dda23d4189b89ddc93899652e88"
)
EXPECTED_ALIAS = {
    "public_name": "qiongli_open_config_wizard",
    "canonical_name": "qiongli_configure_provider",
}
EXPECTED_LEGACY_ONLY = (
    "qiongli_zotero_search",
    "qiongli_zotero_upsert_references",
)
EXPECTED_RUNTIME_ORDER = ("node-mcpb", "python-full", "rust-lite")
EXPECTED_RUNTIME_METADATA = {
    "node-mcpb": ("node", "legacy-mcpb"),
    "python-full": ("python", "full"),
    "rust-lite": ("rust", "marketplace-lite"),
}
EXPECTED_CLI_GAPS = (
    "complete-command-tree",
    "complete-arguments-and-aliases",
    "complete-help-text",
    "complete-json-output",
    "complete-exit-code-matrix",
    "complete-dry-run-semantics",
    "complete-error-classes",
)
EXPECTED_ORCHESTRATOR_GAPS = (
    "complete-task-graph",
    "complete-state-and-resume",
    "all-solo-duo-triad-modes",
    "complete-primary-reviewer-verifier",
    "complete-profile-resolution",
    "complete-artifact-and-quality-gates",
    "complete-failure-and-cancellation",
)
EXPECTED_RESOURCE_ROOTS = (
    ("content/distribution/", "prefix", "target-metadata", 3),
    ("content/mcp-contracts/", "prefix", "mcp-contract", 28),
    ("content/roles/", "prefix", "role", 10),
    ("content/schemas/", "prefix", "schema", 5),
    ("content/skills/", "prefix", "skill", 97),
    ("content/skills-core.md", "exact", "skill-summary", 1),
    ("content/skills-summary.md", "exact", "skill-summary", 1),
    ("content/standards/", "prefix", "standard", 11),
    ("content/subjects/", "prefix", "subject", 77),
    ("content/templates/", "prefix", "template", 92),
    ("content/venue-profiles/", "prefix", "venue-profile", 6),
    ("content/workflow/", "prefix", "workflow", 46),
)
EXPECTED_PROFILES = ("skill-only", "lite", "full")
MACHINE_PATH_PATTERN = re.compile(
    r"(?:file://|(?<![A-Za-z0-9/])/(?:Users|home|root|Volumes|tmp|var/tmp|"
    r"var/folders|private/tmp|private/var/folders)/|"
    r"(?<![A-Za-z0-9+.-])[A-Za-z]:[\\/]|\\\\[^\\/\s]+[\\/][^\\/\s]+)",
    re.IGNORECASE,
)
SECRET_PATTERN = re.compile(
    r"(?:QIONGLI_CANARY_DO_NOT_ECHO|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|"
    r"\b(?:sk[-_]|ghp_|github_pat_)[A-Za-z0-9_-]{12,}\b)"
)


class InventoryConfigError(ValueError):
    """Raised when validator inputs cannot be loaded safely."""


class SafeArgumentParser(argparse.ArgumentParser):
    """Reject invalid arguments without echoing attacker-controlled values."""

    def error(self, _message: str) -> None:
        raise InventoryConfigError("command-line arguments are invalid")


def _unique_json_object(pairs: Sequence[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise InventoryConfigError("JSON object contains duplicate keys")
        value[key] = item
    return value


def _contains_unicode_surrogate(value: Any) -> bool:
    if isinstance(value, str):
        return any(0xD800 <= ord(character) <= 0xDFFF for character in value)
    if isinstance(value, Mapping):
        return any(
            _contains_unicode_surrogate(key) or _contains_unicode_surrogate(item)
            for key, item in value.items()
        )
    if isinstance(value, list):
        return any(_contains_unicode_surrogate(item) for item in value)
    return False


def canonical_payload_bytes(record: Mapping[str, Any]) -> bytes:
    """Serialize the integrity-covered record without the integrity block."""

    payload = {key: value for key, value in record.items() if key != "integrity"}
    return json.dumps(
        payload,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def canonical_payload_sha256(record: Mapping[str, Any]) -> str:
    return hashlib.sha256(canonical_payload_bytes(record)).hexdigest()


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def is_canonical_repository_path(value: str, *, allow_trailing_slash: bool = False) -> bool:
    if not value or "\\" in value or any(ord(character) < 32 for character in value):
        return False
    if value.endswith("/"):
        if not allow_trailing_slash:
            return False
        value = value[:-1]
    if not value or "//" in value:
        return False
    posix = PurePosixPath(value)
    windows = PureWindowsPath(value)
    if posix.is_absolute() or windows.is_absolute():
        return False
    if any(part in {"", ".", ".."} for part in posix.parts):
        return False
    return posix.as_posix() == value


def _safe_file(repo_root: Path, relative: str, *, label: str) -> Path:
    if not is_canonical_repository_path(relative):
        raise InventoryConfigError(f"{label} must be a canonical repository path")
    root = repo_root.resolve(strict=True)
    candidate = repo_root
    for component in PurePosixPath(relative).parts:
        candidate = candidate / component
        if candidate.is_symlink():
            raise InventoryConfigError(f"{label} must not traverse a symbolic link")
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
    except (OSError, RuntimeError, ValueError) as error:
        raise InventoryConfigError(f"{label} is unavailable") from error
    if not resolved.is_file():
        raise InventoryConfigError(f"{label} must be a regular file")
    return resolved


def _load_json_file(repo_root: Path, relative: str, *, label: str) -> dict[str, Any]:
    path = _safe_file(repo_root, relative, label=label)
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=_unique_json_object,
        )
    except (
        InventoryConfigError,
        OSError,
        UnicodeDecodeError,
        json.JSONDecodeError,
    ) as error:
        raise InventoryConfigError(f"{label} must be canonical UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise InventoryConfigError(f"{label} must contain a JSON object")
    if _contains_unicode_surrogate(value):
        raise InventoryConfigError(f"{label} must contain Unicode scalar values")
    return value


def load_inventory_documents(
    repo_root: Path,
    *,
    record_path: str = DEFAULT_RECORD,
    schema_path: str = DEFAULT_SCHEMA,
) -> tuple[dict[str, Any], dict[str, Any]]:
    record = _load_json_file(repo_root, record_path, label="inventory record")
    schema = _load_json_file(repo_root, schema_path, label="inventory schema")
    return record, schema


def _iter_strings(value: Any) -> Sequence[str]:
    strings: list[str] = []
    if isinstance(value, str):
        strings.append(value)
    elif isinstance(value, Mapping):
        for key, item in value.items():
            if isinstance(key, str):
                strings.append(key)
            strings.extend(_iter_strings(item))
    elif isinstance(value, list):
        for item in value:
            strings.extend(_iter_strings(item))
    return strings


def _validate_schema_contract(schema: Mapping[str, Any]) -> list[str]:
    errors: list[str] = []
    schema_bytes = json.dumps(
        schema,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    if _sha256(schema_bytes) != EXPECTED_SCHEMA_CANONICAL_SHA256:
        errors.append("inventory schema canonical digest is invalid")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        errors.append("inventory schema must use JSON Schema Draft 2020-12")
    if schema.get("$id") != "https://qiongli.dev/schemas/ctr-201-semantic-inventory-v1.json":
        errors.append("inventory schema identity is invalid")
    if schema.get("type") != "object" or schema.get("additionalProperties") is not False:
        errors.append("inventory schema must be a closed object")
    properties = schema.get("properties")
    required = schema.get("required")
    if not isinstance(properties, Mapping) or not isinstance(required, list):
        errors.append("inventory schema structure is invalid")
    elif set(required) != set(properties):
        errors.append("inventory schema must require every top-level field")
    return errors


def _validate_completion_claims(record: Mapping[str, Any]) -> list[str]:
    errors: list[str] = []
    completion = record.get("completion")
    if (
        record.get("task_id") != "CTR-201A"
        or record.get("status") != "in-progress"
        or not isinstance(completion, Mapping)
        or completion.get("ctr_201") != "in-progress"
        or completion.get("fnd_202") != "not-implemented"
        or completion.get("completion_ready") is not False
    ):
        errors.append("CTR-201A must remain in progress and FND-202 not implemented")
    return errors


def _load_bound_json(
    repo_root: Path,
    relative: Any,
    expected_sha256: Any,
    *,
    label: str,
) -> tuple[dict[str, Any] | None, list[str]]:
    if not isinstance(relative, str) or not is_canonical_repository_path(relative):
        return None, [f"{label} path binding is invalid"]
    if not isinstance(expected_sha256, str) or not re.fullmatch(r"[0-9a-f]{64}", expected_sha256):
        return None, [f"{label} digest binding is invalid"]
    try:
        path = _safe_file(repo_root, relative, label=label)
        data = path.read_bytes()
        value = json.loads(
            data.decode("utf-8"),
            object_pairs_hook=_unique_json_object,
        )
    except (InventoryConfigError, OSError, UnicodeDecodeError, json.JSONDecodeError):
        return None, [f"{label} cannot be verified"]
    if not isinstance(value, dict):
        return None, [f"{label} must contain a JSON object"]
    if _contains_unicode_surrogate(value):
        return None, [f"{label} contains invalid Unicode scalar data"]
    errors = []
    if _sha256(data) != expected_sha256:
        errors.append(f"{label} digest does not match its binding")
    return value, errors


def _validate_frozen_source(
    repo_root: Path, record: Mapping[str, Any]
) -> tuple[dict[str, Any] | None, list[str]]:
    errors: list[str] = []
    source = record.get("frozen_source")
    if not isinstance(source, Mapping):
        return None, ["frozen source binding is missing"]
    if source.get("manifest_path") != EXPECTED_MANIFEST_PATH:
        errors.append("frozen manifest path is not the accepted A8 anchor")
    if source.get("manifest_sha256") != EXPECTED_MANIFEST_SHA256:
        errors.append("frozen manifest digest is not the accepted A8 digest")
    manifest, load_errors = _load_bound_json(
        repo_root,
        source.get("manifest_path"),
        source.get("manifest_sha256"),
        label="frozen A8 manifest",
    )
    errors.extend(load_errors)
    if manifest is None:
        return None, errors

    manifest_source = manifest.get("source")
    integrity = manifest.get("integrity")
    if (
        source.get("accepted_tag") != EXPECTED_TAG
        or source.get("accepted_commit") != EXPECTED_COMMIT
        or not isinstance(manifest_source, Mapping)
        or manifest_source.get("tag") != EXPECTED_TAG
        or manifest_source.get("peeled_commit") != EXPECTED_COMMIT
    ):
        errors.append("accepted tag lineage does not match the frozen A8 manifest")
    if (
        source.get("corpus_sha256") != EXPECTED_CORPUS_SHA256
        or not isinstance(integrity, Mapping)
        or integrity.get("corpus_sha256") != EXPECTED_CORPUS_SHA256
    ):
        errors.append("frozen A8 corpus digest does not match")

    package_trees = manifest.get("package_trees")
    content_trees = (
        [
            item
            for item in package_trees
            if isinstance(item, Mapping) and item.get("root") == "content/"
        ]
        if isinstance(package_trees, list)
        else []
    )
    content_binding = source.get("content_tree")
    if len(content_trees) != 1 or not isinstance(content_binding, Mapping):
        errors.append("frozen content tree binding is missing or ambiguous")
    else:
        tree = content_trees[0]
        expected = ("content/", EXPECTED_CONTENT_FILE_COUNT, EXPECTED_CONTENT_TREE_SHA256)
        actual = (tree.get("root"), tree.get("file_count"), tree.get("tree_sha256"))
        recorded = (
            content_binding.get("root"),
            content_binding.get("file_count"),
            content_binding.get("tree_sha256"),
        )
        if actual != expected or recorded != expected:
            errors.append("frozen content tree identity does not match")
    return manifest, errors


def _mcp_case(oracle: Mapping[str, Any]) -> Mapping[str, Any] | None:
    cases = oracle.get("cases")
    if not isinstance(cases, list):
        return None
    matches = [
        case
        for case in cases
        if isinstance(case, Mapping)
        and isinstance(case.get("coverage"), list)
        and "mcp-initialize-and-list" in case["coverage"]
    ]
    return matches[0] if len(matches) == 1 else None


def _case_ids_for_coverage(oracle: Mapping[str, Any], coverage: str) -> list[str]:
    result: list[str] = []
    cases = oracle.get("cases")
    if not isinstance(cases, list):
        return result
    for case in cases:
        if not isinstance(case, Mapping) or not isinstance(case.get("coverage"), list):
            continue
        case_id = case.get("id")
        if coverage in case["coverage"] and isinstance(case_id, str):
            result.append(case_id)
    return result


def _validate_runtime_surfaces(
    repo_root: Path,
    record: Mapping[str, Any],
    manifest: Mapping[str, Any],
) -> tuple[dict[str, dict[str, Any]], list[str]]:
    errors: list[str] = []
    mcp = record.get("mcp")
    surfaces = mcp.get("runtime_surfaces") if isinstance(mcp, Mapping) else None
    if not isinstance(surfaces, list):
        return {}, ["runtime MCP surfaces are missing"]
    ids = [item.get("oracle_id") for item in surfaces if isinstance(item, Mapping)]
    if tuple(ids) != EXPECTED_RUNTIME_ORDER or len(ids) != len(set(ids)):
        errors.append("runtime MCP surfaces must contain each frozen oracle exactly once")

    manifest_fixtures = manifest.get("oracle_fixtures")
    fixture_by_id = {
        item.get("oracle_id"): item
        for item in manifest_fixtures
        if isinstance(item, Mapping) and isinstance(item.get("oracle_id"), str)
    } if isinstance(manifest_fixtures, list) else {}
    oracle_documents: dict[str, dict[str, Any]] = {}
    for surface in surfaces:
        if not isinstance(surface, Mapping):
            errors.append("runtime MCP surface entry is invalid")
            continue
        oracle_id = surface.get("oracle_id")
        if not isinstance(oracle_id, str) or oracle_id not in EXPECTED_RUNTIME_METADATA:
            errors.append("runtime MCP surface has an unknown oracle identity")
            continue
        expected_runtime, expected_profile = EXPECTED_RUNTIME_METADATA[oracle_id]
        if (surface.get("runtime"), surface.get("profile")) != (
            expected_runtime,
            expected_profile,
        ):
            errors.append(f"{oracle_id} runtime metadata does not match the frozen oracle")
        binding = surface.get("oracle")
        fixture = fixture_by_id.get(oracle_id)
        if not isinstance(binding, Mapping) or not isinstance(fixture, Mapping):
            errors.append(f"{oracle_id} oracle binding is missing")
            continue
        expected_path = (
            "tooling/migration/baselines/v1.19.0-beta.1/" + str(fixture.get("path", ""))
        )
        expected_binding = (
            expected_path,
            fixture.get("sha256"),
            fixture.get("case_count"),
        )
        recorded_binding = (
            binding.get("path"),
            binding.get("sha256"),
            binding.get("case_count"),
        )
        if recorded_binding != expected_binding or binding.get("case_count") != 5:
            errors.append(f"{oracle_id} binding does not match the frozen manifest")
        oracle, load_errors = _load_bound_json(
            repo_root,
            binding.get("path"),
            binding.get("sha256"),
            label=f"{oracle_id} oracle",
        )
        errors.extend(load_errors)
        if oracle is None:
            continue
        oracle_documents[oracle_id] = oracle
        cases = oracle.get("cases")
        if (
            oracle.get("oracle_id") != oracle_id
            or not isinstance(cases, list)
            or len(cases) != binding.get("case_count")
        ):
            errors.append(f"{oracle_id} case inventory does not match its binding")
        case = _mcp_case(oracle)
        outcome = case.get("outcome") if isinstance(case, Mapping) else None
        value = outcome.get("value") if isinstance(outcome, Mapping) else None
        names = value.get("tool_names") if isinstance(value, Mapping) else None
        count = value.get("tool_count") if isinstance(value, Mapping) else None
        recorded_names = surface.get("public_names")
        if (
            not isinstance(names, list)
            or not all(isinstance(name, str) for name in names)
            or len(names) != len(set(names))
            or recorded_names != names
            or count != len(names)
            or surface.get("public_name_count") != len(names)
        ):
            errors.append(f"{oracle_id} public MCP surface does not match its oracle")
    return oracle_documents, errors


def _ordered_union(*values: Sequence[str]) -> list[str]:
    result: list[str] = []
    seen: set[str] = set()
    for sequence in values:
        for value in sequence:
            if value not in seen:
                seen.add(value)
                result.append(value)
    return result


def _validate_contract_and_target(
    repo_root: Path,
    record: Mapping[str, Any],
) -> list[str]:
    errors: list[str] = []
    contract = record.get("contract_v2")
    mcp = record.get("mcp")
    if not isinstance(contract, Mapping) or not isinstance(mcp, Mapping):
        return ["Contract v2 or MCP inventory is missing"]
    if (
        contract.get("registry_path") != EXPECTED_REGISTRY_PATH
        or contract.get("registry_sha256") != EXPECTED_REGISTRY_SHA256
    ):
        errors.append("Contract v2 registry binding is invalid")
    registry, load_errors = _load_bound_json(
        repo_root,
        contract.get("registry_path"),
        contract.get("registry_sha256"),
        label="Contract v2 registry",
    )
    errors.extend(load_errors)
    if registry is None:
        return errors
    tools = registry.get("tools")
    coverage = registry.get("coverage")
    if not isinstance(tools, list) or not isinstance(coverage, Mapping):
        return [*errors, "Contract v2 pilot structure is invalid"]
    canonical: list[str] = []
    public: list[str] = []
    for tool in tools:
        if not isinstance(tool, Mapping) or not isinstance(tool.get("name"), str):
            errors.append("Contract v2 pilot contains an invalid tool entry")
            continue
        canonical.append(tool["name"])
        public.append(tool["name"])
        aliases = tool.get("aliases", [])
        if not isinstance(aliases, list):
            errors.append("Contract v2 pilot contains an invalid alias inventory")
            continue
        for alias in aliases:
            if isinstance(alias, Mapping) and isinstance(alias.get("name"), str):
                public.append(alias["name"])
            else:
                errors.append("Contract v2 pilot contains an invalid alias entry")
    actual_contract = (
        registry.get("status"),
        coverage.get("mode"),
        len(canonical),
        len(public),
        coverage.get("target_canonical_tool_count"),
        coverage.get("target_public_name_count"),
    )
    recorded_contract = (
        contract.get("status"),
        contract.get("coverage_mode"),
        contract.get("canonical_tool_count"),
        contract.get("public_name_count"),
        contract.get("target_canonical_tool_count"),
        contract.get("target_public_name_count"),
    )
    if actual_contract != ("pilot", "pilot", 6, 7, 23, 24) or recorded_contract != actual_contract:
        errors.append("Contract v2 pilot coverage does not match the current registry")
    if contract.get("completion_ready") is not False:
        errors.append("Contract v2 pilot cannot be marked complete")

    surfaces = mcp.get("runtime_surfaces")
    surface_by_id = {
        item.get("oracle_id"): item
        for item in surfaces
        if isinstance(item, Mapping) and isinstance(item.get("oracle_id"), str)
    } if isinstance(surfaces, list) else {}
    python_names = surface_by_id.get("python-full", {}).get("public_names", [])
    rust_names = surface_by_id.get("rust-lite", {}).get("public_names", [])
    node_names = surface_by_id.get("node-mcpb", {}).get("public_names", [])
    if not all(
        isinstance(names, list) and all(isinstance(name, str) for name in names)
        for names in (python_names, rust_names, node_names)
    ):
        return [*errors, "runtime surface names are invalid"]
    target_public = _ordered_union(python_names, rust_names)
    if target_public != mcp.get("target_public_names") or len(target_public) != 24:
        errors.append(
            "target MCP public-name inventory must be the Python Full and Rust Lite union"
        )
    aliases = mcp.get("aliases")
    if aliases != [EXPECTED_ALIAS]:
        errors.append("target MCP compatibility alias inventory is invalid")
    alias_names = {EXPECTED_ALIAS["public_name"]}
    target_canonical = [name for name in target_public if name not in alias_names]
    if target_canonical != mcp.get("target_canonical_names") or len(target_canonical) != 23:
        errors.append("target MCP canonical-name inventory is invalid")
    if not set(canonical).issubset(target_canonical) or not set(public).issubset(target_public):
        errors.append("Contract v2 pilot names must be contained in the target inventory")
    target_set = set(target_public)
    derived_legacy = [name for name in node_names if name not in target_set]
    legacy = mcp.get("legacy_only")
    recorded_legacy = (
        [item.get("public_name") for item in legacy if isinstance(item, Mapping)]
        if isinstance(legacy, list)
        else []
    )
    if (
        tuple(derived_legacy) != EXPECTED_LEGACY_ONLY
        or tuple(recorded_legacy) != EXPECTED_LEGACY_ONLY
    ):
        errors.append("Node-only legacy MCP inventory is invalid")
    if not isinstance(legacy, list) or any(
        not isinstance(item, Mapping)
        or item.get("source_oracle") != "node-mcpb"
        or item.get("disposition") != "pending-LEG-201"
        for item in legacy
    ):
        errors.append("Node-only MCP names must remain pending LEG-201 disposition")
    if (
        mcp.get("target_public_name_count") != len(target_public)
        or mcp.get("target_canonical_tool_count") != len(target_canonical)
    ):
        errors.append("target MCP counts do not match their inventories")
    return errors


def _validate_coverage_gaps(
    record: Mapping[str, Any], oracle_documents: Mapping[str, Mapping[str, Any]]
) -> list[str]:
    errors: list[str] = []
    python_oracle = oracle_documents.get("python-full")
    if not isinstance(python_oracle, Mapping):
        return ["Python Full oracle is unavailable for CLI and orchestrator coverage"]
    expected = (
        (
            "cli",
            ("cli-command", "installer-dry-run"),
            ["python.cli-align", "python.installer-dry-run"],
            [
                "align-success-outcome",
                "installer-dry-run-success",
                "observed-success-exit-code-zero",
            ],
            list(EXPECTED_CLI_GAPS),
        ),
        (
            "orchestrator",
            ("orchestration-preview",),
            ["python.orchestration-preview"],
            ["task-run-preview", "duo-mode-preview"],
            list(EXPECTED_ORCHESTRATOR_GAPS),
        ),
    )
    for section_name, coverages, case_ids, captured_scope, gaps in expected:
        section = record.get(section_name)
        actual_cases = [
            case_id
            for coverage in coverages
            for case_id in _case_ids_for_coverage(python_oracle, coverage)
        ]
        if not isinstance(section, Mapping) or (
            section.get("status") != "incomplete"
            or section.get("captured_oracle_cases") != case_ids
            or actual_cases != case_ids
            or section.get("captured_scope") != captured_scope
            or section.get("required_not_fully_captured") != gaps
            or section.get("completion_ready") is not False
        ):
            errors.append(f"{section_name} coverage must remain explicit and incomplete")
    return errors


def _matches_root(path: str, source: str, match: str) -> bool:
    return path == source if match == "exact" else path.startswith(source)


def _validate_content(record: Mapping[str, Any], manifest: Mapping[str, Any]) -> list[str]:
    errors: list[str] = []
    content = record.get("content")
    if not isinstance(content, Mapping):
        return ["content resource inventory is missing"]
    roots = content.get("resource_roots")
    if not isinstance(roots, list):
        return ["content resource roots are missing"]
    recorded_roots = [
        (
            item.get("source"),
            item.get("match"),
            item.get("resource_kind"),
            item.get("file_count"),
        )
        for item in roots
        if isinstance(item, Mapping)
    ]
    if tuple(recorded_roots) != EXPECTED_RESOURCE_ROOTS:
        errors.append("content resource roots or logical kinds do not match CTR-201A")
    sources = [item[0] for item in recorded_roots]
    if len(sources) != len(set(sources)):
        errors.append("content resource roots contain a duplicate source")
    for source, match, _kind, _count in recorded_roots:
        if not isinstance(source, str) or not is_canonical_repository_path(
            source, allow_trailing_slash=match == "prefix"
        ):
            errors.append("content resource root contains a non-canonical path")

    package_trees = manifest.get("package_trees")
    content_trees = [
        item
        for item in package_trees
        if isinstance(item, Mapping) and item.get("root") == "content/"
    ] if isinstance(package_trees, list) else []
    files = content_trees[0].get("files") if len(content_trees) == 1 else None
    paths = [
        item.get("path")
        for item in files
        if isinstance(item, Mapping) and isinstance(item.get("path"), str)
    ] if isinstance(files, list) else []
    if len(paths) != EXPECTED_CONTENT_FILE_COUNT or len(paths) != len(set(paths)):
        errors.append("frozen content file inventory is invalid")
    for path in paths:
        matches = [
            item
            for item in recorded_roots
            if isinstance(item[0], str)
            and isinstance(item[1], str)
            and _matches_root(path, item[0], item[1])
        ]
        if len(matches) != 1:
            errors.append("every frozen content file must have exactly one logical resource kind")
            break
    for source, match, _kind, expected_count in recorded_roots:
        count = sum(_matches_root(path, source, match) for path in paths)
        if count != expected_count:
            errors.append("content resource root file count does not match the frozen tree")
            break
    if (
        content.get("source_file_count") != EXPECTED_CONTENT_FILE_COUNT
        or content.get("source_tree_sha256") != EXPECTED_CONTENT_TREE_SHA256
    ):
        errors.append("content source identity does not match the frozen tree")

    profiles = content.get("profiles")
    profile_names = [
        item.get("profile") for item in profiles if isinstance(item, Mapping)
    ] if isinstance(profiles, list) else []
    if tuple(profile_names) != EXPECTED_PROFILES or len(profile_names) != len(set(profile_names)):
        errors.append("content profiles must be unique and ordered")
    if not isinstance(profiles, list) or any(
        not isinstance(item, Mapping)
        or item.get("status") != "not-ready"
        or item.get("included_resource_kinds") != []
        or item.get("expected_materialized_tree_sha256") is not None
        for item in profiles
    ):
        errors.append("content profile mappings must remain explicitly not ready")
    materialization = content.get("materialization")
    if (
        not isinstance(materialization, Mapping)
        or materialization.get("status") != "not-ready"
        or materialization.get("mapping_policy") != "not-frozen"
        or materialization.get("expected_tree_sha256") is not None
        or content.get("completion_ready") is not False
    ):
        errors.append("content materialization must remain explicitly not ready")
    return errors


def validate_inventory(
    repo_root: Path,
    record: Mapping[str, Any],
    schema: Mapping[str, Any],
) -> list[str]:
    if _contains_unicode_surrogate(schema):
        return ["inventory schema contains invalid Unicode scalar data"]
    if _contains_unicode_surrogate(record):
        return ["inventory contains invalid Unicode scalar data"]
    errors = _validate_schema_contract(schema)
    schema_errors = validate_instance(record, schema)
    if schema_errors:
        # Schema errors may contain attacker-controlled values. Keep the diagnostic
        # deliberately generic so a malformed record cannot become an exfiltration path.
        errors.append("inventory record does not satisfy its closed schema")
        return sorted(set(errors))

    strings = _iter_strings(record)
    if any(MACHINE_PATH_PATTERN.search(value) for value in strings):
        errors.append("inventory contains a forbidden machine-local path")
    if any(SECRET_PATTERN.search(value) for value in strings):
        errors.append("inventory contains forbidden secret-shaped data")

    errors.extend(_validate_completion_claims(record))
    integrity = record.get("integrity")
    if not isinstance(integrity, Mapping) or (
        integrity.get("algorithm") != "sha256"
        or integrity.get("canonicalization")
        != "utf-8-json-sorted-keys-compact-excluding-integrity"
        or integrity.get("payload_sha256") != canonical_payload_sha256(record)
    ):
        errors.append("inventory canonical payload digest does not match")

    manifest, source_errors = _validate_frozen_source(repo_root, record)
    errors.extend(source_errors)
    if manifest is None:
        return sorted(set(errors))
    oracle_documents, surface_errors = _validate_runtime_surfaces(
        repo_root, record, manifest
    )
    errors.extend(surface_errors)
    errors.extend(_validate_contract_and_target(repo_root, record))
    errors.extend(_validate_coverage_gaps(record, oracle_documents))
    errors.extend(_validate_content(record, manifest))
    return sorted(set(errors))


def _parser() -> argparse.ArgumentParser:
    parser = SafeArgumentParser(
        description="Validate the derived CTR-201A semantic inventory."
    )
    parser.add_argument("--root", type=Path, default=REPO_ROOT)
    parser.add_argument("--record", default=DEFAULT_RECORD)
    parser.add_argument("--schema", default=DEFAULT_SCHEMA)
    parser.add_argument("--json", action="store_true", help="Emit JSON only")
    return parser


def _emit(payload: Mapping[str, Any], *, as_json: bool) -> None:
    if as_json:
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
        return
    status = payload["status"]
    if status == "pass":
        print(
            "[ctr-201] PASS: semantic inventory is in progress; "
            "FND-202 is not implemented"
        )
        return
    print(f"[ctr-201] {status.upper()}: {payload['error_count']} finding(s)", file=sys.stderr)
    for error in payload.get("errors", []):
        print(f"[ctr-201] {error}", file=sys.stderr)


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    as_json = "--json" in arguments
    try:
        args = _parser().parse_args(arguments)
        root = args.root.resolve(strict=True)
        if not root.is_dir():
            raise InventoryConfigError("repository root must be a directory")
        if not isinstance(args.record, str) or not isinstance(args.schema, str):
            raise InventoryConfigError("inventory paths must be strings")
        record, schema = load_inventory_documents(
            root, record_path=args.record, schema_path=args.schema
        )
    except (InventoryConfigError, OSError, RuntimeError):
        payload = {
            "status": "error",
            "exit_code": 2,
            "error_count": 1,
            "errors": ["validator configuration could not be loaded safely"],
        }
        _emit(payload, as_json=as_json)
        return 2

    errors = validate_inventory(root, record, schema)
    if errors:
        payload = {
            "status": "fail",
            "exit_code": 1,
            "error_count": len(errors),
            "errors": errors,
        }
        _emit(payload, as_json=args.json)
        return 1
    payload = {
        "status": "pass",
        "exit_code": 0,
        "error_count": 0,
        "ctr_201": "in-progress",
        "fnd_202": "not-implemented",
    }
    _emit(payload, as_json=args.json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
