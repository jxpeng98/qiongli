#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import unicodedata
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any, Mapping, Sequence

from tooling.scripts.validate_capability_contract import validate_instance


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RECORD = "tooling/migration/ctr-201-inventory.json"
DEFAULT_SCHEMA = "tooling/migration/ctr-201-inventory.schema.json"
DEFAULT_CLI_ARTIFACT = "tooling/migration/ctr-201-cli.json"
DEFAULT_CLI_SCHEMA = "tooling/migration/ctr-201-cli.schema.json"
DEFAULT_CLI_RUNTIME_ARTIFACT = "tooling/migration/ctr-201-cli-runtime.json"
DEFAULT_CLI_RUNTIME_SCHEMA = "tooling/migration/ctr-201-cli-runtime.schema.json"
DEFAULT_ORCHESTRATOR_ARTIFACT = "tooling/migration/ctr-201-orchestrator.json"
DEFAULT_ORCHESTRATOR_SCHEMA = "tooling/migration/ctr-201-orchestrator.schema.json"
DEFAULT_ORCHESTRATOR_RUNTIME_ARTIFACT = (
    "tooling/migration/ctr-201-orchestrator-runtime.json"
)
DEFAULT_ORCHESTRATOR_RUNTIME_SCHEMA = (
    "tooling/migration/ctr-201-orchestrator-runtime.schema.json"
)
DEFAULT_CONTENT_ARTIFACT = "tooling/migration/ctr-201-content.json"
DEFAULT_CONTENT_SCHEMA = "tooling/migration/ctr-201-content.schema.json"
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
    "a373e6727e639a07c48bfd3c7c2ec2c56a2a7ca5f713ebd5fa31c5b0dbf72a3e"
)
EXPECTED_CLI_SCHEMA_CANONICAL_SHA256 = (
    "173436615a8a26d45903cc7812a55f2e9ae094089f637bced0f418a3976456ad"
)
EXPECTED_CLI_RUNTIME_SCHEMA_CANONICAL_SHA256 = (
    "785b051f5d67900d43012b7f9574f43e7a2a1c63e3b4274a4814637d0623175b"
)
EXPECTED_CLI_RUNTIME_PAYLOAD_SHA256 = (
    "b82be3d7f1531a3fefdf3dd864c74042d2d3ecc806d38337f24e2b14d843f41c"
)
EXPECTED_ORCHESTRATOR_SCHEMA_CANONICAL_SHA256 = (
    "0473158288cf35d4a10e39cfc741fd5b4cb38a49c68209aaea48337d52782510"
)
EXPECTED_ORCHESTRATOR_PAYLOAD_SHA256 = (
    "508ed0f92a511a0a9a6daa33598ce891222540b15e5aa207984db97319fe2c5e"
)
EXPECTED_ORCHESTRATOR_RUNTIME_SCHEMA_CANONICAL_SHA256 = (
    "dff9d2226a4f3cb6ab068b158c7aecbff393c0c2a18a46058c5f587172a5178e"
)
EXPECTED_ORCHESTRATOR_RUNTIME_PAYLOAD_SHA256 = (
    "29bbb1c0cd042d469f55e93078a4d3b4494148f47a2bd66e568d097f83e6b5da"
)
EXPECTED_ORCHESTRATOR_RUNTIME_CASE_MANIFEST_SHA256 = (
    "6a930dd355eb57b0b6b1759f73dba9c7af4b115e1b0bdc576112e49c14cc20ee"
)
EXPECTED_CONTENT_SCHEMA_CANONICAL_SHA256 = (
    "6f88a56c2a88c51f68a6bb10bce05776d1e06f678ae916739a6e3de96d2b1704"
)
EXPECTED_CONTENT_PAYLOAD_SHA256 = (
    "d17f37aa96d1896d047b27d197d63f773ae1d644a875722f5262be39593ff304"
)
EXPECTED_CLI_CAPTURE_CONTRACT = {
    "source_mode": "accepted-tag-git-blobs",
    "python_version": "python3.12",
    "help_mode": "authored-help-only",
    "environment_mode": "dual-environment",
    "environment_count": 2,
    "dynamic_default_policy": "symbolic-context-values",
    "callable_policy": "allowlisted-symbols-no-repr",
    "ambient_dependency_policy": "disabled-with-deny-use-stubs",
    "side_effect_policy": "read-only-no-network-no-process",
}
EXPECTED_CLI_COUNTS = {
    "canonical_command_path_count": 46,
    "public_command_path_count": 49,
    "console_entrypoint_count": 5,
    "argument_action_count": 164,
    "cwd_default_count": 27,
}
EXPECTED_CLI_RUNTIME_COUNTS = {
    "public_command_path_count": 49,
    "console_entrypoint_count": 5,
    "case_count": 118,
    "formatted_help_observation_count": 245,
    "invalid_usage_observation_count": 49,
    "zero_argument_observation_count": 5,
    "json_canonical_path_count": 13,
    "dry_run_public_path_count": 11,
    "npm_alias_count": 5,
}
EXPECTED_ORCHESTRATOR_COUNTS = {
    "stage_count": 13,
    "task_count": 76,
    "required_dependency_edge_count": 104,
    "runtime_agent_count": 3,
    "functional_agent_count": 9,
    "routing_skill_id_count": 82,
    "logical_mcp_capability_count": 11,
    "quality_gate_count": 4,
    "declared_profile_count": 5,
    "team_run_task_count": 2,
    "worker_orchestration_task_count": 2,
}
EXPECTED_ORCHESTRATOR_RUNTIME_COUNTS = {
    "case_count": 44,
    "resolved_dimension_count": 6,
    "disposition_decision_count": 6,
}
EXPECTED_ORCHESTRATOR_RUNTIME_COVERAGE = {
    "accepted_a8_case_count": 1,
    "bounded_runtime_case_count": 43,
    "case_count": 44,
    "completion_ready": True,
    "cross_platform_runtime_parity": "not-claimed",
    "ctr_201": "complete",
    "ctr_202": "not-complete",
    "disposition_decision_count": 6,
    "fnd_202": "not-implemented",
    "real_agent_runtime_parity": "not-claimed",
    "required_not_fully_captured_count": 0,
    "resolved_dimension_count": 6,
    "rust_orchestrator": "not-implemented",
}
EXPECTED_ORCHESTRATOR_CHILD_COUNTS = {
    "stage_count": 13,
    "task_count": 76,
    "prerequisites_all_edge_count": 54,
    "prerequisites_any_edge_count": 50,
    "required_dependency_edge_count": 104,
    "recommended_prerequisite_edge_count": 44,
    "recommended_next_edge_count": 154,
    "task_output_assignment_count": 136,
    "unique_task_output_count": 118,
    "runtime_agent_count": 3,
    "functional_agent_count": 9,
    "functional_stage_default_count": 13,
    "functional_task_override_count": 16,
    "routing_skill_id_count": 82,
    "task_skill_assignment_count": 207,
    "logical_mcp_capability_count": 11,
    "task_mcp_assignment_count": 139,
    "quality_gate_count": 4,
    "task_quality_gate_assignment_count": 136,
    "declared_profile_count": 5,
    "team_run_task_count": 2,
    "worker_orchestration_task_count": 2,
    "source_anchor_count": 4,
}
EXPECTED_ORCHESTRATOR_CAPTURE_CONTRACT = {
    "source_mode": "accepted-tag-git-blobs",
    "python_version": "python3.12",
    "yaml_toolchain": "pyyaml-6.0.3-strict-safe-loader",
    "python_analysis": "stdlib-ast-no-import-no-compile-no-eval",
    "environment_mode": "environment-independent-static-analysis",
    "side_effect_policy": "git-read-only-until-explicit-artifact-write",
    "ordering_policy": (
        "source-order-for-primary-catalogs;canonical-sort-for-map-summaries;"
        "explicit-ordinals-where-order-preserved"
    ),
    "yaml_policy": (
        "strict-safe-loader-with-one-authenticated-identical-duplicate;"
        "reject-other-duplicates-merge-anchor-alias-custom-tag-timestamp-binary-"
        "or-nonfinite"
    ),
    "compatibility_policy": "preserve-external-and-normalized-spellings",
}
EXPECTED_ORCHESTRATOR_COVERAGE_BOUNDARY = {
    "static_contract": "captured",
    "runtime_behavior_matrix": "incomplete",
    "state_resume_behavior": "incomplete",
    "agent_launch_behavior": "incomplete",
    "solo_duo_triad_runtime_parity": "incomplete",
    "concurrency_timeout_cancellation": "incomplete",
    "failure_replay_behavior": "incomplete",
    "quality_gate_semantic_execution": "incomplete",
    "profile_override_runtime_behavior": "incomplete",
    "plugin_marketplace_behavior": "not-implemented",
    "materialized_content_closure": "incomplete",
    "rust_orchestrator": "not-implemented",
    "ctr_201": "in-progress",
    "fnd_202": "not-implemented",
    "completion_ready": False,
}
EXPECTED_ORCHESTRATOR_SOURCE_PATHS = (
    "packages/python-qiongli/src/qiongli/bridges/orchestrator.py",
    "packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py",
    "content/standards/research-workflow-contract.yaml",
    "content/standards/mcp-agent-capability-map.yaml",
)
EXPECTED_CLI_ALIASES = {
    ("qiongli", "self-update"): ("update",),
    ("qiongli", "remove"): ("uninstall", "delete"),
}
EXPECTED_EMPTY_ARGUMENT_COMMANDS = {
    ("qiongli", "provider"),
    ("qiongli", "guidance"),
    ("qiongli", "subject"),
    ("qiongli", "project"),
    ("qiongli", "mcp", "config"),
}
EXPECTED_CONSOLE_ENTRYPOINTS = (
    ("qiongli", "qiongli.cli:main", 0),
    ("ql", "qiongli.cli:main", 1),
    ("research-skills", "qiongli.cli:main", 2),
    ("rsk", "qiongli.cli:main", 3),
    ("rsw", "qiongli.cli:main", 4),
)
EXPECTED_PARSER_ROOTS = {
    "qiongli-cli": (
        ("qiongli",),
        "qiongli.cli:build_parser",
        "cli-parser",
        "Install/upgrade qiongli client skills without requiring a git fork.",
        ("cmd", True),
        0,
    ),
    "qiongli-mcp-cli": (
        ("qiongli", "mcp"),
        "bridges.mcp_cli:build_parser",
        "mcp-cli-parser",
        "Run and configure the Qiongli cross-platform MCP server.",
        ("cmd", True),
        1,
    ),
}
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
EXPECTED_CLI_CAPTURED_SCOPE = (
    "align-success-outcome",
    "installer-dry-run-success",
    "observed-success-exit-code-zero",
    "python-full-static-command-tree",
    "python-full-static-arguments-and-aliases",
    "python-full-authored-help-metadata",
    "python-full-console-entrypoints",
    "python-full-mounted-mcp-parser",
    "python-full-formatted-help-49-public-paths",
    "python-full-invalid-usage-49-public-paths",
    "python-full-five-console-entrypoint-root-help-and-align",
    "python-full-safe-json-handler-boundaries",
    "python-full-observable-error-taxonomy",
    "python-full-dry-run-explicit-dispositions",
    "python-full-side-effect-explicit-dispositions",
    "python-full-approved-leg-201-disposition-decisions",
    "python-full-handler-runtime-parity-not-claimed",
    "npm-accepted-parse-argv-dispatch",
    "npm-python-update-divergence",
)
EXPECTED_CLI_GAPS: tuple[str, ...] = ()
EXPECTED_ORCHESTRATOR_CAPTURED_SCOPE = (
    "task-run-preview",
    "duo-mode-preview",
    "python-full-static-declared-stage-task-graph",
    "python-full-static-declared-agent-contracts",
    "python-full-static-declared-routing-contracts",
    "python-full-static-declared-mcp-capability-contracts",
    "python-full-static-declared-quality-gates",
    "python-full-bounded-orchestrator-runtime-matrix",
    "python-full-state-and-resume-observed-boundary",
    "python-full-agent-launch-fake-boundary",
    "python-full-solo-duo-triad-controller-semantics",
    "python-full-failure-and-cancellation-observed-boundary",
    "python-full-quality-gate-artifact-existence-boundary",
    "python-full-approved-downstream-disposition-decisions",
    "python-full-real-agent-runtime-parity-not-claimed",
)
EXPECTED_ORCHESTRATOR_RUNTIME_DIMENSION_IDS = (
    "complete-runtime-behavior-matrix",
    "complete-state-and-resume",
    "complete-agent-launch-behavior",
    "complete-solo-duo-triad-runtime-parity",
    "complete-failure-and-cancellation",
    "complete-quality-gate-semantic-execution",
)
EXPECTED_ORCHESTRATOR_GAPS: tuple[str, ...] = ()
EXPECTED_ORCHESTRATION_ORACLE_OUTCOME = {
    "error": None,
    "exit_code": 0,
    "status": "success",
    "value": {
        "controller_metadata": {
            "controller": "codex",
            "execution_mode": "duo",
            "primary_agent": "",
            "review_agent": "",
            "solo_role_gates": "standard",
            "verifier_agent": "",
        },
        "effective_domain": "auto",
        "mode": "task-run-preview",
        "run_agents": False,
        "stderr_lines": [],
        "task": {
            "paper_type": "empirical",
            "task_id": "F3",
            "topic": "runtime-baseline",
        },
        "task_description": "F3 empirical runtime-baseline",
        "will_launch_agents": False,
    },
}
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
EXPECTED_PROFILES = ("skill-only", "marketplace-lite", "full")
EXPECTED_CONTENT_COUNTS = {
    "accepted_content_file_count": 377,
    "accepted_content_total_bytes": 1_761_400,
    "resource_root_count": 12,
    "resource_kind_count": 11,
    "portable_core_file_count": 263,
    "materialized_profile_count": 3,
    "materialized_output_file_count": 863,
    "identity_output_count": 850,
    "transformed_output_count": 6,
    "generated_output_count": 7,
}
EXPECTED_CONTENT_PROFILE_FACTS = (
    {
        "profile_id": "skill-only",
        "aliases": [],
        "variant_id": "qiongli-next-prerelease-core-desktop-focused",
        "subject": "core",
        "flavor": "desktop",
        "coverage": "focused",
        "skill_name": "qiongli-next",
        "source_file_count": 341,
        "source_total_bytes": 1_627_305,
        "source_tree_sha256": (
            "2283d6f5d284dde43225c5fb194e2e714b5e7b34e9c9bb97e753914d968acf26"
        ),
        "materialized_file_count": 178,
        "materialized_total_bytes": 708_608,
        "materialized_tree_sha256": (
            "5b76bc0c02cda7fc18adf2b1afd492e763392ed5fc2a05dac360d1221045f280"
        ),
        "origin_counts": {
            "identity-copy": 173,
            "content-transform": 2,
            "generated-metadata": 3,
        },
    },
    {
        "profile_id": "marketplace-lite",
        "aliases": ["lite"],
        "variant_id": "qiongli-next-prerelease-core-full-complete",
        "subject": "core",
        "flavor": "full",
        "coverage": "complete",
        "skill_name": "qiongli-next",
        "source_file_count": 377,
        "source_total_bytes": 1_761_400,
        "source_tree_sha256": EXPECTED_CONTENT_TREE_SHA256,
        "materialized_file_count": 342,
        "materialized_total_bytes": 1_600_064,
        "materialized_tree_sha256": (
            "a854fc61203883132041a43077cc9ea26e62aa28e2c2eeb266777f582b029c6c"
        ),
        "origin_counts": {
            "identity-copy": 338,
            "content-transform": 2,
            "generated-metadata": 2,
        },
    },
    {
        "profile_id": "full",
        "aliases": [],
        "variant_id": "qiongli-local-core-full-complete",
        "subject": "core",
        "flavor": "full",
        "coverage": "complete",
        "skill_name": "qiongli",
        "source_file_count": 377,
        "source_total_bytes": 1_761_400,
        "source_tree_sha256": EXPECTED_CONTENT_TREE_SHA256,
        "materialized_file_count": 343,
        "materialized_total_bytes": 1_602_568,
        "materialized_tree_sha256": (
            "b5612c713789bbd126829edc1e0646ec2c2387898aa2f5a4c812de0de5aad554"
        ),
        "origin_counts": {
            "identity-copy": 339,
            "content-transform": 2,
            "generated-metadata": 2,
        },
    },
)
EXPECTED_PORTABLE_CORE_FACTS = {
    "variant_id": "legacy-portable-core",
    "file_count": 263,
    "total_bytes": 1_442_456,
    "tree_sha256": (
        "21840d087bd18b1b9d37a03bddf6318a9023c69a0a320ff8bfcea843d4f5b48b"
    ),
    "origin_counts": {
        "identity-copy": 263,
        "content-transform": 0,
        "generated-metadata": 0,
    },
}
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
CALLABLE_REPR_PATTERN = re.compile(
    r"(?:<(?:(?:bound )?method|function|class)\b|\bat 0x[0-9a-f]+>)",
    re.IGNORECASE,
)
WINDOWS_RESERVED_COMPONENT = re.compile(
    r"^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\..*)?$",
    re.IGNORECASE,
)


_CLI_EXTRACTION_CACHE: dict[str, bytes] = {}
_ORCHESTRATOR_EXTRACTION_CACHE: dict[str, bytes] = {}
_CONTENT_EXTRACTION_CACHE: dict[str, bytes] = {}


class InventoryConfigError(ValueError):
    """Raised when validator inputs cannot be loaded safely."""


class CliArtifactMismatch(ValueError):
    """Raised when accepted source extraction disagrees with the child artifact."""


class OrchestratorArtifactMismatch(ValueError):
    """Raised when accepted-source orchestrator extraction disagrees."""


class ContentArtifactMismatch(ValueError):
    """Raised when accepted-source content extraction disagrees."""


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


def _reject_non_finite_constant(_value: str) -> None:
    raise InventoryConfigError("JSON document contains a non-finite number")


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


def _contains_non_finite_number(value: Any) -> bool:
    if isinstance(value, float):
        return value != value or value in {float("inf"), float("-inf")}
    if isinstance(value, Mapping):
        return any(
            _contains_non_finite_number(key) or _contains_non_finite_number(item)
            for key, item in value.items()
        )
    if isinstance(value, list):
        return any(_contains_non_finite_number(item) for item in value)
    return False


def _canonical_json_bytes(value: Any) -> bytes:
    try:
        rendered = json.dumps(
            value,
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
            allow_nan=False,
        )
        return rendered.encode("utf-8")
    except (TypeError, UnicodeEncodeError, ValueError) as error:
        raise InventoryConfigError("JSON value cannot be serialized canonically") from error


def canonical_payload_bytes(record: Mapping[str, Any]) -> bytes:
    """Serialize the integrity-covered record without the integrity block."""

    payload = {key: value for key, value in record.items() if key != "integrity"}
    return _canonical_json_bytes(payload)


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
    if posix.is_absolute() or windows.is_absolute() or windows.drive or windows.root:
        return False
    if any(part in {"", ".", ".."} for part in posix.parts):
        return False
    if any(
        ":" in part
        or part.endswith((" ", "."))
        or WINDOWS_RESERVED_COMPONENT.fullmatch(part) is not None
        for part in posix.parts
    ):
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
            parse_constant=_reject_non_finite_constant,
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


def _validate_recursively_closed_schema(
    schema: Mapping[str, Any], *, label: str = "child"
) -> list[str]:
    """Require every object-shaped child schema to be closed and fully required."""

    stack: list[Any] = [schema]
    while stack:
        value = stack.pop()
        if isinstance(value, Mapping):
            if value.get("type") == "object":
                properties = value.get("properties")
                required = value.get("required")
                if (
                    not isinstance(properties, Mapping)
                    or not isinstance(required, list)
                    or not all(isinstance(item, str) for item in required)
                    or len(required) != len(set(required))
                    or set(required) != set(properties)
                    or value.get("additionalProperties") is not False
                ):
                    return [f"{label} schema must be recursively closed"]
            stack.extend(value.values())
        elif isinstance(value, list):
            stack.extend(value)
    return []


def _cli_path(value: Any) -> tuple[str, ...] | None:
    if not isinstance(value, list) or not value or not all(
        isinstance(item, str) for item in value
    ):
        return None
    return tuple(value)


def _validate_cli_repository_paths(artifact: Mapping[str, Any]) -> list[str]:
    source = artifact.get("source")
    if not isinstance(source, Mapping):
        return ["CLI child source bindings are invalid"]
    a8_manifest = source.get("a8_manifest")
    python_oracle = source.get("python_full_oracle")
    package_tree = source.get("package_tree")
    anchors = source.get("blob_anchors")
    checks: list[tuple[Any, bool]] = []
    for binding in (a8_manifest, python_oracle):
        checks.append((binding.get("path") if isinstance(binding, Mapping) else None, False))
    checks.append(
        (
            package_tree.get("root") if isinstance(package_tree, Mapping) else None,
            True,
        )
    )
    if isinstance(anchors, list):
        checks.extend(
            (anchor.get("path") if isinstance(anchor, Mapping) else None, False)
            for anchor in anchors
        )
    else:
        checks.append((None, False))
    if any(
        not isinstance(path, str)
        or not is_canonical_repository_path(path, allow_trailing_slash=trailing)
        for path, trailing in checks
    ):
        return ["CLI child contains a non-canonical repository path"]
    return []


def _validate_cli_artifact_semantics(artifact: Mapping[str, Any]) -> list[str]:
    """Validate cross-field facts which JSON Schema cannot express."""

    errors: list[str] = []
    if _contains_unicode_surrogate(artifact):
        return ["CLI child contains invalid Unicode scalar data"]
    if _contains_non_finite_number(artifact):
        return ["CLI child contains invalid numeric data"]

    strings = _iter_strings(artifact)
    if any(MACHINE_PATH_PATTERN.search(value) for value in strings):
        errors.append("CLI child contains a forbidden machine-local path")
    if any(SECRET_PATTERN.search(value) for value in strings):
        errors.append("CLI child contains forbidden secret-shaped data")
    if any(CALLABLE_REPR_PATTERN.search(value) for value in strings):
        errors.append("CLI child contains an unstable callable representation")
    errors.extend(_validate_cli_repository_paths(artifact))
    if artifact.get("capture_contract") != EXPECTED_CLI_CAPTURE_CONTRACT:
        errors.append("CLI child capture-isolation contract is invalid")

    integrity = artifact.get("integrity")
    if not isinstance(integrity, Mapping) or (
        integrity.get("algorithm") != "sha256"
        or integrity.get("canonicalization")
        != "utf-8-json-sorted-keys-compact-excluding-integrity"
        or integrity.get("payload_sha256") != canonical_payload_sha256(artifact)
    ):
        errors.append("CLI child canonical payload digest does not match")

    entrypoints = artifact.get("console_entrypoints")
    projected_entrypoints = (
        tuple(
            (
                item.get("name"),
                item.get("target"),
                item.get("declaration_ordinal"),
            )
            for item in entrypoints
            if isinstance(item, Mapping)
        )
        if isinstance(entrypoints, list)
        else ()
    )
    if projected_entrypoints != EXPECTED_CONSOLE_ENTRYPOINTS:
        errors.append("CLI child console entrypoints are not exact")

    parser_roots = artifact.get("parser_roots")
    projected_roots: dict[Any, Any] = {}
    duplicate_root = False
    if isinstance(parser_roots, list):
        for root in parser_roots:
            if not isinstance(root, Mapping):
                duplicate_root = True
                continue
            root_id = root.get("root_id")
            if root_id in projected_roots:
                duplicate_root = True
            subcommands = root.get("subcommand_metadata")
            projected_roots[root_id] = (
                _cli_path(root.get("path")),
                root.get("builder"),
                root.get("source_anchor_role"),
                root.get("description"),
                (
                    subcommands.get("destination"),
                    subcommands.get("required"),
                )
                if isinstance(subcommands, Mapping)
                else None,
                root.get("declaration_ordinal"),
            )
    if duplicate_root or projected_roots != EXPECTED_PARSER_ROOTS:
        errors.append("CLI child parser roots are not exact")

    commands = artifact.get("commands")
    if not isinstance(commands, list):
        return sorted(set([*errors, "CLI child command inventory is invalid"]))

    canonical_paths: set[tuple[str, ...]] = set()
    public_paths: set[tuple[str, ...]] = set()
    alias_projection: dict[tuple[str, ...], tuple[str, ...]] = {}
    parser_root_ordinals: dict[str, list[int]] = {
        "qiongli-cli": [],
        "qiongli-mcp-cli": [],
    }
    empty_argument_paths: set[tuple[str, ...]] = set()
    total_arguments = 0
    cwd_defaults = 0
    mounted_command_count = 0

    for command in commands:
        if not isinstance(command, Mapping):
            errors.append("CLI child command inventory is invalid")
            continue
        path = _cli_path(command.get("path"))
        segment = command.get("segment")
        aliases = command.get("aliases")
        ordinal = command.get("declaration_ordinal")
        if path is None or len(path) < 2 or path[0] != "qiongli":
            errors.append("CLI child command path is invalid")
            continue
        if path in canonical_paths:
            errors.append("CLI child contains a duplicate canonical command path")
        canonical_paths.add(path)
        if segment != path[-1]:
            errors.append("CLI child command segment does not match its path")
        if not isinstance(ordinal, int) or isinstance(ordinal, bool) or ordinal < 0:
            errors.append("CLI child command declaration ordinal is invalid")
        else:
            parser_root_id = (
                "qiongli-mcp-cli"
                if len(path) > 2 and path[:2] == ("qiongli", "mcp")
                else "qiongli-cli"
            )
            parser_root_ordinals[parser_root_id].append(ordinal)

        alias_values = (
            tuple(alias for alias in aliases if isinstance(alias, str))
            if isinstance(aliases, list)
            else ()
        )
        if not isinstance(aliases, list) or len(alias_values) != len(aliases):
            errors.append("CLI child command aliases are invalid")
        if len(alias_values) != len(set(alias_values)) or segment in alias_values:
            errors.append("CLI child command aliases are not unique")
        alias_projection[path] = alias_values
        for public_path in (path, *(path[:-1] + (alias,) for alias in alias_values)):
            if public_path in public_paths:
                errors.append("CLI child contains a projected public command collision")
            public_paths.add(public_path)

        expected_delegate = (
            {
                "kind": "parser-root",
                "parser_root_id": "qiongli-mcp-cli",
                "argument_destination": "mcp_args",
            }
            if path == ("qiongli", "mcp")
            else None
        )
        if command.get("delegate") != expected_delegate:
            errors.append("CLI child parser-root delegation is invalid")
        if len(path) > 2 and path[:2] == ("qiongli", "mcp"):
            mounted_command_count += 1

        arguments = command.get("arguments")
        if not isinstance(arguments, list):
            errors.append("CLI child command arguments are invalid")
            continue
        if not arguments:
            empty_argument_paths.add(path)
        total_arguments += len(arguments)
        argument_ordinals: list[int] = []
        option_strings: set[str] = set()
        positional_destinations: set[str] = set()
        delegate_arguments = 0
        for argument in arguments:
            if not isinstance(argument, Mapping):
                errors.append("CLI child command argument is invalid")
                continue
            argument_ordinal = argument.get("declaration_ordinal")
            if (
                not isinstance(argument_ordinal, int)
                or isinstance(argument_ordinal, bool)
                or argument_ordinal < 0
            ):
                errors.append("CLI child argument declaration ordinal is invalid")
            else:
                argument_ordinals.append(argument_ordinal)
            options = argument.get("option_strings")
            positional = argument.get("positional")
            if not isinstance(options, list) or not all(
                isinstance(option, str) for option in options
            ):
                errors.append("CLI child option-string inventory is invalid")
                options = []
            if positional is not (len(options) == 0):
                errors.append("CLI child positional and option-string metadata disagree")
            for option in options:
                if option in option_strings:
                    errors.append("CLI child reuses an option string within a command")
                option_strings.add(option)

            destination = argument.get("destination")
            action = argument.get("action")
            nargs = argument.get("nargs")
            type_name = argument.get("type")
            if action == "help" or not isinstance(destination, str):
                errors.append("CLI child includes a non-callable argument action")
            if positional is True:
                if destination in positional_destinations:
                    errors.append("CLI child reuses a positional destination")
                if isinstance(destination, str):
                    positional_destinations.add(destination)
            if action in {"store-const", "store-false", "store-true"} and (
                positional is not False or nargs != "zero" or type_name != "none"
            ):
                errors.append("CLI child constant action metadata is inconsistent")
            if action == "store" and nargs == "zero":
                errors.append("CLI child store action cannot use zero arguments")
            if type_name == "integer" and action != "store":
                errors.append("CLI child integer type is bound to an invalid action")
            default = argument.get("default")
            if isinstance(default, Mapping) and default.get("kind") == "context":
                if default.get("source") != "cwd":
                    errors.append("CLI child dynamic default context is invalid")
                cwd_defaults += 1
            if destination == "mcp_args":
                delegate_arguments += 1
                if (
                    path != ("qiongli", "mcp")
                    or positional is not True
                    or action != "store"
                    or nargs != "remainder"
                ):
                    errors.append("CLI child delegated argument metadata is invalid")
        if sorted(argument_ordinals) != list(range(len(arguments))):
            errors.append("CLI child argument ordinals must be contiguous per command")
        if path == ("qiongli", "mcp") and delegate_arguments != 1:
            errors.append("CLI child MCP delegate argument must be unique")
        if path != ("qiongli", "mcp") and delegate_arguments:
            errors.append("CLI child delegate argument is attached to the wrong command")

    if len(commands) != EXPECTED_CLI_COUNTS["canonical_command_path_count"] or len(
        canonical_paths
    ) != EXPECTED_CLI_COUNTS["canonical_command_path_count"]:
        errors.append("CLI child canonical command count does not match")
    if len(public_paths) != EXPECTED_CLI_COUNTS["public_command_path_count"]:
        errors.append("CLI child public command count does not match")
    if mounted_command_count != 7:
        errors.append("CLI child mounted parser-root command count does not match")
    if any(
        sorted(ordinals) != list(range(len(ordinals)))
        for ordinals in parser_root_ordinals.values()
    ):
        errors.append("CLI child command ordinals must be contiguous per parser root")
    nonempty_aliases = {
        path: aliases for path, aliases in alias_projection.items() if aliases
    }
    if nonempty_aliases != EXPECTED_CLI_ALIASES:
        errors.append("CLI child alias inventory is not exact")
    if empty_argument_paths != EXPECTED_EMPTY_ARGUMENT_COMMANDS:
        errors.append("CLI child zero-argument command inventory is not exact")
    if total_arguments != EXPECTED_CLI_COUNTS["argument_action_count"]:
        errors.append("CLI child argument action count does not match")
    if cwd_defaults != EXPECTED_CLI_COUNTS["cwd_default_count"]:
        errors.append("CLI child cwd-default count does not match")

    coverage = artifact.get("coverage")
    expected_coverage = {
        "canonical_command_count": 46,
        "public_command_count": 49,
        "console_entrypoint_count": 5,
        "argument_action_count": 164,
        "cwd_default_count": 27,
        "static_semantics": "captured",
        "formatted_help_output": "incomplete",
        "json_output": "incomplete",
        "runtime_behavior_matrix": "incomplete",
        "exit_code_matrix": "incomplete",
        "dry_run_semantics": "incomplete",
        "error_matrix": "incomplete",
        "side_effect_matrix": "incomplete",
        "legacy_npm_compatibility_surface": "incomplete",
        "ctr_201": "in-progress",
        "fnd_202": "not-implemented",
        "completion_ready": False,
    }
    if not isinstance(coverage, Mapping) or dict(coverage) != expected_coverage:
        errors.append("CLI child coverage boundary is invalid")
    return sorted(set(errors))


def _validate_orchestrator_repository_paths(
    artifact: Mapping[str, Any],
) -> list[str]:
    checks: list[tuple[Any, bool]] = []
    source = artifact.get("source")
    if isinstance(source, Mapping):
        for key in ("a8_manifest", "python_full_oracle"):
            binding = source.get(key)
            checks.append(
                (binding.get("path") if isinstance(binding, Mapping) else None, False)
            )
        trees = source.get("package_trees")
        if isinstance(trees, list):
            checks.extend(
                (
                    tree.get("root") if isinstance(tree, Mapping) else None,
                    True,
                )
                for tree in trees
            )
        else:
            checks.append((None, True))
        anchors = source.get("blob_anchors")
        if isinstance(anchors, list):
            checks.extend(
                (
                    anchor.get("path") if isinstance(anchor, Mapping) else None,
                    False,
                )
                for anchor in anchors
            )
        else:
            checks.append((None, False))
    else:
        checks.append((None, False))

    workflow = artifact.get("workflow")
    if isinstance(workflow, Mapping):
        checks.append((workflow.get("artifact_root"), True))
        for collection in (workflow.get("stages"), workflow.get("tasks")):
            if not isinstance(collection, list):
                checks.append((None, True))
                continue
            for item in collection:
                outputs = item.get("outputs") if isinstance(item, Mapping) else None
                if not isinstance(outputs, list):
                    checks.append((None, True))
                    continue
                checks.extend((output, output.endswith("/") if isinstance(output, str) else False) for output in outputs)

    routing = artifact.get("routing")
    if isinstance(routing, Mapping):
        for skill in routing.get("skills", []):
            if not isinstance(skill, Mapping):
                checks.append((None, False))
                continue
            checks.append((skill.get("file"), False))
            outputs = skill.get("default_outputs")
            if isinstance(outputs, list):
                checks.extend(
                    (
                        output,
                        output.endswith("/") if isinstance(output, str) else False,
                    )
                    for output in outputs
                )
            else:
                checks.append((None, True))
        for agent in routing.get("functional_agents", []):
            checks.append(
                (
                    agent.get("role_file") if isinstance(agent, Mapping) else None,
                    False,
                )
            )
        for gate in routing.get("quality_gates", []):
            contract_ref = gate.get("contract_ref") if isinstance(gate, Mapping) else None
            checks.append(
                (
                    contract_ref.split("#", 1)[0]
                    if isinstance(contract_ref, str) and contract_ref.count("#") == 1
                    else None,
                    False,
                )
            )
        for team_run in routing.get("team_runs", []):
            if not isinstance(team_run, Mapping):
                checks.append((None, True))
                continue
            for key in ("shard_outputs", "canonical_outputs"):
                outputs = team_run.get(key)
                if isinstance(outputs, list):
                    checks.extend(
                        (
                            output,
                            output.endswith("/") if isinstance(output, str) else False,
                        )
                        for output in outputs
                    )
                else:
                    checks.append((None, True))

    compatibility = artifact.get("compatibility")
    indirect = (
        compatibility.get("indirect_content_dependencies")
        if isinstance(compatibility, Mapping)
        else None
    )
    if isinstance(indirect, Mapping):
        checks.append((indirect.get("skill_registry_path"), False))
        role_paths = indirect.get("functional_role_paths")
        if isinstance(role_paths, list):
            checks.extend((path, False) for path in role_paths)
        else:
            checks.append((None, False))

    oracle = artifact.get("oracle")
    source_paths = oracle.get("source_paths") if isinstance(oracle, Mapping) else None
    if isinstance(source_paths, list):
        checks.extend((path, False) for path in source_paths)
    else:
        checks.append((None, False))
    if any(
        not isinstance(path, str)
        or not is_canonical_repository_path(path, allow_trailing_slash=trailing)
        for path, trailing in checks
    ):
        return ["orchestrator child contains a non-canonical portable path"]
    return []


def _orchestrator_required_graph_has_cycle(
    graph: Mapping[str, Sequence[str]],
) -> bool:
    visited: set[str] = set()
    visiting: set[str] = set()

    def visit(task_id: str) -> bool:
        if task_id in visited:
            return False
        if task_id in visiting:
            return True
        visiting.add(task_id)
        for dependency in graph.get(task_id, ()):
            if dependency not in graph or visit(dependency):
                return True
        visiting.remove(task_id)
        visited.add(task_id)
        return False

    return any(visit(task_id) for task_id in graph)


def _validate_orchestrator_artifact_semantics(
    artifact: Mapping[str, Any],
) -> list[str]:
    errors: list[str] = []
    if _contains_unicode_surrogate(artifact):
        return ["orchestrator child contains invalid Unicode scalar data"]
    if _contains_non_finite_number(artifact):
        return ["orchestrator child contains invalid numeric data"]
    strings = _iter_strings(artifact)
    if any(MACHINE_PATH_PATTERN.search(value) for value in strings):
        errors.append("orchestrator child contains a forbidden machine-local path")
    if any(SECRET_PATTERN.search(value) for value in strings):
        errors.append("orchestrator child contains forbidden secret-shaped data")
    if any(CALLABLE_REPR_PATTERN.search(value) for value in strings):
        errors.append("orchestrator child contains an unstable callable representation")
    errors.extend(_validate_orchestrator_repository_paths(artifact))
    if artifact.get("task_id") != "CTR-201C" or artifact.get("status") != (
        "static-contract-captured"
    ):
        errors.append("orchestrator child status boundary is invalid")
    if artifact.get("capture_contract") != EXPECTED_ORCHESTRATOR_CAPTURE_CONTRACT:
        errors.append("orchestrator child capture-isolation contract is invalid")
    integrity = artifact.get("integrity")
    if not isinstance(integrity, Mapping) or (
        integrity.get("algorithm") != "sha256"
        or integrity.get("canonicalization")
        != "utf-8-json-sorted-keys-compact-excluding-integrity"
        or integrity.get("payload_sha256") != canonical_payload_sha256(artifact)
    ):
        errors.append("orchestrator child canonical payload digest does not match")

    source = artifact.get("source")
    source_ok = False
    if isinstance(source, Mapping):
        manifest = source.get("a8_manifest")
        python_oracle = source.get("python_full_oracle")
        trees = source.get("package_trees")
        anchors = source.get("blob_anchors")
        source_ok = (
            source.get("accepted_tag") == EXPECTED_TAG
            and source.get("accepted_commit") == EXPECTED_COMMIT
            and manifest
            == {"path": EXPECTED_MANIFEST_PATH, "sha256": EXPECTED_MANIFEST_SHA256}
            and python_oracle
            == {
                "path": (
                    "tooling/migration/baselines/v1.19.0-beta.1/oracles/"
                    "python-full.json"
                ),
                "sha256": (
                    "26d247c9268c3166c98080aef420acfdb8248f62b11cc69420250f6e493a23e3"
                ),
                "case_id": "python.orchestration-preview",
            }
            and trees
            == [
                {
                    "root": "packages/python-qiongli/",
                    "file_count": 76,
                    "tree_sha256": (
                        "3a91a6dde9a78116fed73358275b2797c3ce7bf3d9a54894e7dbd11d2f0f9781"
                    ),
                },
                {
                    "root": "content/",
                    "file_count": 377,
                    "tree_sha256": EXPECTED_CONTENT_TREE_SHA256,
                },
            ]
            and isinstance(anchors, list)
            and tuple(
                anchor.get("path")
                for anchor in anchors
                if isinstance(anchor, Mapping)
            )
            == EXPECTED_ORCHESTRATOR_SOURCE_PATHS
            and len(anchors) == len(EXPECTED_ORCHESTRATOR_SOURCE_PATHS)
        )
    if not source_ok:
        errors.append("orchestrator child frozen-source binding is invalid")

    coverage = artifact.get("coverage")
    if not isinstance(coverage, Mapping) or any(
        coverage.get(key) != value
        for key, value in {
            **EXPECTED_ORCHESTRATOR_CHILD_COUNTS,
            **EXPECTED_ORCHESTRATOR_COVERAGE_BOUNDARY,
        }.items()
    ) or set(coverage) != set(EXPECTED_ORCHESTRATOR_CHILD_COUNTS) | set(
        EXPECTED_ORCHESTRATOR_COVERAGE_BOUNDARY
    ):
        errors.append("orchestrator child coverage boundary is invalid")

    workflow = artifact.get("workflow")
    routing = artifact.get("routing")
    if not isinstance(workflow, Mapping) or not isinstance(routing, Mapping):
        return sorted(
            set([*errors, "orchestrator child workflow or routing inventory is invalid"])
        )
    stages = workflow.get("stages")
    tasks = workflow.get("tasks")
    if not isinstance(stages, list) or not isinstance(tasks, list):
        return sorted(set([*errors, "orchestrator child task inventory is invalid"]))

    stage_ids = [
        item.get("stage_id") for item in stages if isinstance(item, Mapping)
    ]
    if (
        len(stage_ids) != len(stages)
        or len(stage_ids) != len(set(stage_ids))
        or len(stage_ids) != EXPECTED_ORCHESTRATOR_COUNTS["stage_count"]
        or [item.get("declaration_ordinal") for item in stages if isinstance(item, Mapping)]
        != list(range(len(stages)))
    ):
        errors.append("orchestrator child stage inventory is not unique and ordered")

    def registry_ids(key: str, field: str) -> list[Any]:
        values = routing.get(key)
        return (
            [item.get(field) for item in values if isinstance(item, Mapping)]
            if isinstance(values, list)
            else []
        )

    runtime_agents = registry_ids("runtime_agents", "agent_id")
    functional_agents = registry_ids("functional_agents", "agent_id")
    skills = registry_ids("skills", "skill_id")
    mcp_capabilities = registry_ids("logical_mcp_capabilities", "capability_id")
    quality_gates = registry_ids("quality_gates", "gate_id")
    registries = (
        (runtime_agents, EXPECTED_ORCHESTRATOR_COUNTS["runtime_agent_count"]),
        (functional_agents, EXPECTED_ORCHESTRATOR_COUNTS["functional_agent_count"]),
        (skills, EXPECTED_ORCHESTRATOR_COUNTS["routing_skill_id_count"]),
        (
            mcp_capabilities,
            EXPECTED_ORCHESTRATOR_COUNTS["logical_mcp_capability_count"],
        ),
        (quality_gates, EXPECTED_ORCHESTRATOR_COUNTS["quality_gate_count"]),
    )
    if any(
        len(values) != expected or len(values) != len(set(values))
        for values, expected in registries
    ):
        errors.append("orchestrator child registry identities are invalid")

    expected_task_keys = {
        "task_id",
        "declaration_ordinal",
        "stage_id",
        "title",
        "purpose",
        "outputs",
        "dependencies",
        "required_skills",
        "required_mcp",
        "quality_gates",
        "runtime_plan",
        "functional_plan",
    }
    expected_dependency_keys = {
        "prerequisites_all",
        "prerequisites_any",
        "recommended_prerequisites",
        "recommended_next",
    }
    expected_runtime_keys = {"primary_agent", "review_agent", "fallback_agent"}
    expected_functional_keys = {
        "owner",
        "source",
        "stage_default_owner",
        "role_id",
        "role_file",
    }
    task_ids = [item.get("task_id") for item in tasks if isinstance(item, Mapping)]
    task_id_set = set(task_ids)
    if (
        len(task_ids) != len(tasks)
        or len(task_ids) != len(task_id_set)
        or len(task_ids) != EXPECTED_ORCHESTRATOR_COUNTS["task_count"]
    ):
        errors.append("orchestrator child task identities are invalid")
    required_graph: dict[str, Sequence[str]] = {}
    task_key_error = False
    ordinal_error = False
    reference_error = False
    for ordinal, task in enumerate(tasks):
        if not isinstance(task, Mapping):
            task_key_error = True
            continue
        if set(task) != expected_task_keys:
            task_key_error = True
        if task.get("declaration_ordinal") != ordinal:
            ordinal_error = True
        task_id = task.get("task_id")
        if task.get("stage_id") not in set(stage_ids):
            reference_error = True
        dependencies = task.get("dependencies")
        if not isinstance(dependencies, Mapping) or set(dependencies) != expected_dependency_keys:
            task_key_error = True
            continue
        dependency_values: dict[str, Sequence[str]] = {}
        for key in expected_dependency_keys:
            values = dependencies.get(key)
            if not isinstance(values, list) or not all(
                isinstance(value, str) for value in values
            ):
                task_key_error = True
                dependency_values[key] = ()
            else:
                dependency_values[key] = values
                if not set(values).issubset(task_id_set):
                    reference_error = True
        if isinstance(task_id, str):
            required_graph[task_id] = dependency_values.get("prerequisites_all", ())
        runtime_plan = task.get("runtime_plan")
        functional_plan = task.get("functional_plan")
        if not isinstance(runtime_plan, Mapping) or set(runtime_plan) != expected_runtime_keys:
            task_key_error = True
        elif not set(runtime_plan.values()).issubset(set(runtime_agents)):
            reference_error = True
        if (
            not isinstance(functional_plan, Mapping)
            or set(functional_plan) != expected_functional_keys
        ):
            task_key_error = True
        elif {
            functional_plan.get("owner"),
            functional_plan.get("stage_default_owner"),
        } - set(functional_agents):
            reference_error = True
        for key, allowed in (
            ("required_skills", set(skills)),
            ("required_mcp", set(mcp_capabilities)),
            ("quality_gates", set(quality_gates)),
        ):
            values = task.get(key)
            if not isinstance(values, list) or not all(
                isinstance(value, str) for value in values
            ):
                task_key_error = True
            elif not set(values).issubset(allowed):
                reference_error = True
    if task_key_error:
        errors.append("orchestrator child task key closure is invalid")
    if ordinal_error:
        errors.append("orchestrator child task ordinals are not contiguous")
    if reference_error:
        errors.append("orchestrator child task reference closure is invalid")
    if (
        len(required_graph) != len(tasks)
        or _orchestrator_required_graph_has_cycle(required_graph)
    ):
        errors.append("orchestrator child prerequisites_all graph is not a DAG")

    expected_oracle = {
        "oracle_id": "python-full",
        "case_id": "python.orchestration-preview",
        "source_paths": list(EXPECTED_ORCHESTRATOR_SOURCE_PATHS),
        "operation": "tools/call qiongli_task_run",
        "transport": "jsonrpc-stdio",
        "task": {
            "task_id": "F3",
            "paper_type": "empirical",
            "topic": "runtime-baseline",
            "guidance_mode": "off",
            "run_agents": False,
        },
        "outcome": {
            "status": "success",
            "exit_code": 0,
            "mode": "task-run-preview",
            "will_launch_agents": False,
            "controller": "codex",
            "execution_mode": "duo",
            "primary_agent": "",
            "review_agent": "",
            "verifier_agent": "",
            "solo_role_gates": "standard",
            "effective_domain": "auto",
            "task_description": "F3 empirical runtime-baseline",
            "stderr_lines": [],
        },
        "filesystem_delta": {
            "before_tree_sha256": (
                "4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945"
            ),
            "after_tree_sha256": (
                "4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945"
            ),
            "created": [],
            "modified": [],
            "deleted": [],
            "writes_outside_sandbox": False,
        },
    }
    if artifact.get("oracle") != expected_oracle:
        errors.append("orchestrator child frozen oracle outcome is not exact")
    return sorted(set(errors))


def _accepted_cli_extraction_bytes(repo_root: Path) -> bytes:
    try:
        cache_key = str(repo_root.resolve(strict=True))
    except (OSError, RuntimeError) as error:
        raise InventoryConfigError("accepted CLI extraction root is unavailable") from error
    cached = _CLI_EXTRACTION_CACHE.get(cache_key)
    if cached is not None:
        return cached
    try:
        from tooling.scripts import extract_ctr_201_cli_inventory as extractor
    except ImportError as error:
        raise InventoryConfigError("accepted CLI extractor is unavailable") from error
    try:
        extracted = extractor.extract_cli_inventory(repo_root)
    except Exception as error:
        mismatch_type = getattr(extractor, "InventoryMismatch", ())
        unavailable_type = getattr(extractor, "ExtractorError", ())
        if mismatch_type and isinstance(error, mismatch_type):
            raise CliArtifactMismatch("accepted CLI extraction does not match") from error
        if unavailable_type and isinstance(error, unavailable_type):
            raise InventoryConfigError("accepted CLI extraction is unavailable") from error
        raise InventoryConfigError("accepted CLI extraction failed safely") from error
    if (
        not isinstance(extracted, Mapping)
        or _contains_unicode_surrogate(extracted)
        or _contains_non_finite_number(extracted)
    ):
        raise InventoryConfigError("accepted CLI extraction returned invalid data")
    canonical = _canonical_json_bytes(extracted)
    _CLI_EXTRACTION_CACHE[cache_key] = canonical
    return canonical


def _accepted_orchestrator_extraction_bytes(repo_root: Path) -> bytes:
    try:
        cache_key = str(repo_root.resolve(strict=True))
    except (OSError, RuntimeError) as error:
        raise InventoryConfigError(
            "accepted orchestrator extraction root is unavailable"
        ) from error
    cached = _ORCHESTRATOR_EXTRACTION_CACHE.get(cache_key)
    if cached is not None:
        return cached
    try:
        from tooling.scripts import extract_ctr_201_orchestrator_inventory as extractor
    except ImportError as error:
        raise InventoryConfigError(
            "accepted orchestrator extractor is unavailable"
        ) from error
    try:
        extracted = extractor.extract_orchestrator_inventory(repo_root)
    except Exception as error:
        mismatch_type = getattr(extractor, "InventoryMismatch", ())
        unavailable_type = getattr(extractor, "ExtractorError", ())
        if mismatch_type and isinstance(error, mismatch_type):
            raise OrchestratorArtifactMismatch(
                "accepted orchestrator extraction does not match"
            ) from error
        if unavailable_type and isinstance(error, unavailable_type):
            raise InventoryConfigError(
                "accepted orchestrator extraction is unavailable"
            ) from error
        raise InventoryConfigError(
            "accepted orchestrator extraction failed safely"
        ) from error
    if (
        not isinstance(extracted, Mapping)
        or _contains_unicode_surrogate(extracted)
        or _contains_non_finite_number(extracted)
    ):
        raise InventoryConfigError(
            "accepted orchestrator extraction returned invalid data"
        )
    canonical = _canonical_json_bytes(extracted)
    _ORCHESTRATOR_EXTRACTION_CACHE[cache_key] = canonical
    return canonical


def _accepted_content_extraction_bytes(repo_root: Path) -> bytes:
    try:
        cache_key = str(repo_root.resolve(strict=True))
    except (OSError, RuntimeError) as error:
        raise InventoryConfigError(
            "accepted content extraction root is unavailable"
        ) from error
    cached = _CONTENT_EXTRACTION_CACHE.get(cache_key)
    if cached is not None:
        return cached
    try:
        from tooling.scripts import extract_ctr_201_content_inventory as extractor
    except ImportError as error:
        raise InventoryConfigError(
            "accepted content extractor is unavailable"
        ) from error
    try:
        extracted = extractor.extract_content_inventory(repo_root)
    except Exception as error:
        mismatch_type = getattr(extractor, "InventoryMismatch", ())
        unavailable_type = getattr(extractor, "ExtractorError", ())
        if mismatch_type and isinstance(error, mismatch_type):
            raise ContentArtifactMismatch(
                "accepted content extraction does not match"
            ) from error
        if unavailable_type and isinstance(error, unavailable_type):
            raise InventoryConfigError(
                "accepted content extraction is unavailable"
            ) from error
        raise InventoryConfigError(
            "accepted content extraction failed safely"
        ) from error
    if (
        not isinstance(extracted, Mapping)
        or _contains_unicode_surrogate(extracted)
        or _contains_non_finite_number(extracted)
    ):
        raise InventoryConfigError(
            "accepted content extraction returned invalid data"
        )
    canonical = _canonical_json_bytes(extracted)
    _CONTENT_EXTRACTION_CACHE[cache_key] = canonical
    return canonical


def _validate_cli_static_semantics(
    repo_root: Path, record: Mapping[str, Any]
) -> list[str]:
    cli = record.get("cli")
    binding = cli.get("static_semantics") if isinstance(cli, Mapping) else None
    if not isinstance(binding, Mapping):
        return ["CLI static-semantics binding is missing"]
    expected_binding = {
        "task_id": "CTR-201B",
        "status": "static-semantics-captured",
        "artifact_path": DEFAULT_CLI_ARTIFACT,
        "schema_path": DEFAULT_CLI_SCHEMA,
        "schema_canonical_sha256": EXPECTED_CLI_SCHEMA_CANONICAL_SHA256,
        **EXPECTED_CLI_COUNTS,
        "capture_ready": True,
    }
    errors = [
        "CLI static-semantics master binding is invalid"
        for key, value in expected_binding.items()
        if binding.get(key) != value
    ]
    if errors:
        return sorted(set(errors))

    child_schema = _load_json_file(
        repo_root, DEFAULT_CLI_SCHEMA, label="CLI child schema"
    )
    artifact = _load_json_file(
        repo_root, DEFAULT_CLI_ARTIFACT, label="CLI child artifact"
    )
    if _sha256(_canonical_json_bytes(child_schema)) != EXPECTED_CLI_SCHEMA_CANONICAL_SHA256:
        errors.append("CLI child schema canonical digest is invalid")
    if (
        child_schema.get("$schema")
        != "https://json-schema.org/draft/2020-12/schema"
        or child_schema.get("$id")
        != "https://qiongli.dev/schemas/ctr-201-cli-static-semantics-v1.json"
    ):
        errors.append("CLI child schema identity is invalid")
    errors.extend(_validate_recursively_closed_schema(child_schema, label="CLI child"))
    if validate_instance(artifact, child_schema):
        errors.append("CLI child artifact does not satisfy its closed schema")
        return sorted(set(errors))

    errors.extend(_validate_cli_artifact_semantics(artifact))
    integrity = artifact.get("integrity")
    coverage = artifact.get("coverage")
    if not isinstance(integrity, Mapping) or binding.get("payload_sha256") != integrity.get(
        "payload_sha256"
    ):
        errors.append("CLI child payload digest does not match the master binding")
    child_count_keys = {
        "canonical_command_path_count": "canonical_command_count",
        "public_command_path_count": "public_command_count",
        "console_entrypoint_count": "console_entrypoint_count",
        "argument_action_count": "argument_action_count",
        "cwd_default_count": "cwd_default_count",
    }
    if not isinstance(coverage, Mapping) or any(
        binding.get(master_key) != coverage.get(child_key)
        for master_key, child_key in child_count_keys.items()
    ):
        errors.append("CLI child counts do not match the master binding")
    if errors:
        return sorted(set(errors))
    try:
        extracted = _accepted_cli_extraction_bytes(repo_root)
    except CliArtifactMismatch:
        return ["accepted CLI extraction does not match its frozen source"]
    if extracted != _canonical_json_bytes(artifact):
        errors.append("CLI child artifact differs from accepted-source extraction")
    return sorted(set(errors))


def _validate_cli_runtime_freeze(
    repo_root: Path, record: Mapping[str, Any]
) -> list[str]:
    cli = record.get("cli")
    binding = cli.get("runtime_freeze") if isinstance(cli, Mapping) else None
    if not isinstance(binding, Mapping):
        return ["CLI runtime-freeze binding is missing"]
    expected_binding = {
        "task_id": "CTR-201E",
        "status": "runtime-inventory-freeze-captured",
        "artifact_path": DEFAULT_CLI_RUNTIME_ARTIFACT,
        "schema_path": DEFAULT_CLI_RUNTIME_SCHEMA,
        "schema_canonical_sha256": EXPECTED_CLI_RUNTIME_SCHEMA_CANONICAL_SHA256,
        "payload_sha256": EXPECTED_CLI_RUNTIME_PAYLOAD_SHA256,
        **EXPECTED_CLI_RUNTIME_COUNTS,
        "capture_ready": True,
    }
    if dict(binding) != expected_binding:
        return ["CLI runtime-freeze master binding is invalid"]

    child_schema = _load_json_file(
        repo_root, DEFAULT_CLI_RUNTIME_SCHEMA, label="CLI runtime child schema"
    )
    artifact = _load_json_file(
        repo_root, DEFAULT_CLI_RUNTIME_ARTIFACT, label="CLI runtime child artifact"
    )
    errors: list[str] = []
    if (
        _sha256(_canonical_json_bytes(child_schema))
        != EXPECTED_CLI_RUNTIME_SCHEMA_CANONICAL_SHA256
    ):
        errors.append("CLI runtime child schema canonical digest is invalid")
    if (
        child_schema.get("$schema")
        != "https://json-schema.org/draft/2020-12/schema"
        or child_schema.get("$id")
        != "https://qiongli.dev/schemas/ctr-201-cli-runtime.schema.json"
    ):
        errors.append("CLI runtime child schema identity is invalid")
    errors.extend(
        _validate_recursively_closed_schema(child_schema, label="CLI runtime child")
    )
    if validate_instance(artifact, child_schema):
        errors.append("CLI runtime child artifact does not satisfy its closed schema")
        return sorted(set(errors))
    try:
        from tooling.scripts import extract_ctr_201_cli_runtime_inventory as extractor

        extractor._validate_expected_artifact(artifact, repo_root)
    except Exception:
        errors.append("CLI runtime child semantic validation failed")
        return sorted(set(errors))
    integrity = artifact.get("integrity")
    coverage = artifact.get("coverage")
    if (
        not isinstance(integrity, Mapping)
        or integrity.get("payload_sha256") != EXPECTED_CLI_RUNTIME_PAYLOAD_SHA256
        or binding.get("payload_sha256") != integrity.get("payload_sha256")
    ):
        errors.append("CLI runtime child payload digest does not match the master binding")
    child_count_keys = {
        "public_command_path_count": "public_commands",
        "console_entrypoint_count": "console_entrypoints",
        "formatted_help_observation_count": "help_observations",
        "invalid_usage_observation_count": "invalid_usage_observations",
        "zero_argument_observation_count": "zero_argument_observations",
        "json_canonical_path_count": "json_canonical_commands",
        "dry_run_public_path_count": "dry_run_public_commands",
        "npm_alias_count": "npm_aliases",
    }
    if not isinstance(coverage, Mapping) or any(
        binding.get(master_key) != coverage.get(child_key)
        for master_key, child_key in child_count_keys.items()
    ):
        errors.append("CLI runtime child counts do not match the master binding")
    cases = artifact.get("cases")
    if not isinstance(cases, list) or binding.get("case_count") != len(cases):
        errors.append("CLI runtime child case count does not match the master binding")
    return sorted(set(errors))


def _validate_orchestrator_static_contract(
    repo_root: Path, record: Mapping[str, Any]
) -> list[str]:
    orchestrator = record.get("orchestrator")
    binding = (
        orchestrator.get("static_contract")
        if isinstance(orchestrator, Mapping)
        else None
    )
    if not isinstance(binding, Mapping):
        return ["orchestrator static-contract binding is missing"]
    expected_binding = {
        "task_id": "CTR-201C",
        "status": "static-contract-captured",
        "artifact_path": DEFAULT_ORCHESTRATOR_ARTIFACT,
        "schema_path": DEFAULT_ORCHESTRATOR_SCHEMA,
        "schema_canonical_sha256": EXPECTED_ORCHESTRATOR_SCHEMA_CANONICAL_SHA256,
        "payload_sha256": EXPECTED_ORCHESTRATOR_PAYLOAD_SHA256,
        **EXPECTED_ORCHESTRATOR_COUNTS,
        "capture_ready": True,
    }
    if dict(binding) != expected_binding:
        return ["orchestrator static-contract master binding is invalid"]

    child_schema = _load_json_file(
        repo_root, DEFAULT_ORCHESTRATOR_SCHEMA, label="orchestrator child schema"
    )
    artifact = _load_json_file(
        repo_root, DEFAULT_ORCHESTRATOR_ARTIFACT, label="orchestrator child artifact"
    )
    errors: list[str] = []
    if (
        _sha256(_canonical_json_bytes(child_schema))
        != EXPECTED_ORCHESTRATOR_SCHEMA_CANONICAL_SHA256
    ):
        errors.append("orchestrator child schema canonical digest is invalid")
    if (
        child_schema.get("$schema")
        != "https://json-schema.org/draft/2020-12/schema"
        or child_schema.get("$id")
        != (
            "https://qiongli.dev/schemas/"
            "ctr-201-orchestrator-static-semantics-v1.json"
        )
    ):
        errors.append("orchestrator child schema identity is invalid")
    errors.extend(
        _validate_recursively_closed_schema(
            child_schema, label="orchestrator child"
        )
    )
    if validate_instance(artifact, child_schema):
        errors.append("orchestrator child artifact does not satisfy its closed schema")
        return sorted(set(errors))
    errors.extend(_validate_orchestrator_artifact_semantics(artifact))
    integrity = artifact.get("integrity")
    coverage = artifact.get("coverage")
    if (
        not isinstance(integrity, Mapping)
        or integrity.get("payload_sha256") != EXPECTED_ORCHESTRATOR_PAYLOAD_SHA256
        or binding.get("payload_sha256") != integrity.get("payload_sha256")
    ):
        errors.append(
            "orchestrator child payload digest does not match the master binding"
        )
    if not isinstance(coverage, Mapping) or any(
        binding.get(key) != coverage.get(key)
        for key in EXPECTED_ORCHESTRATOR_COUNTS
    ):
        errors.append("orchestrator child counts do not match the master binding")
    if errors:
        return sorted(set(errors))
    try:
        extracted = _accepted_orchestrator_extraction_bytes(repo_root)
    except OrchestratorArtifactMismatch:
        return ["accepted orchestrator extraction does not match its frozen source"]
    if extracted != _canonical_json_bytes(artifact):
        errors.append(
            "orchestrator child artifact differs from accepted-source extraction"
        )
    return sorted(set(errors))


def _validate_orchestrator_runtime_freeze(
    repo_root: Path, record: Mapping[str, Any]
) -> list[str]:
    orchestrator = record.get("orchestrator")
    binding = (
        orchestrator.get("runtime_freeze")
        if isinstance(orchestrator, Mapping)
        else None
    )
    if not isinstance(binding, Mapping):
        return ["orchestrator runtime-freeze binding is missing"]
    expected_binding = {
        "task_id": "CTR-201F",
        "status": "runtime-inventory-freeze-captured",
        "artifact_path": DEFAULT_ORCHESTRATOR_RUNTIME_ARTIFACT,
        "schema_path": DEFAULT_ORCHESTRATOR_RUNTIME_SCHEMA,
        "schema_canonical_sha256": (
            EXPECTED_ORCHESTRATOR_RUNTIME_SCHEMA_CANONICAL_SHA256
        ),
        "payload_sha256": EXPECTED_ORCHESTRATOR_RUNTIME_PAYLOAD_SHA256,
        "case_manifest_sha256": (
            EXPECTED_ORCHESTRATOR_RUNTIME_CASE_MANIFEST_SHA256
        ),
        **EXPECTED_ORCHESTRATOR_RUNTIME_COUNTS,
        "capture_ready": True,
    }
    if dict(binding) != expected_binding:
        return ["orchestrator runtime-freeze master binding is invalid"]

    child_schema = _load_json_file(
        repo_root,
        DEFAULT_ORCHESTRATOR_RUNTIME_SCHEMA,
        label="orchestrator runtime child schema",
    )
    artifact = _load_json_file(
        repo_root,
        DEFAULT_ORCHESTRATOR_RUNTIME_ARTIFACT,
        label="orchestrator runtime child artifact",
    )
    errors: list[str] = []
    if (
        _sha256(_canonical_json_bytes(child_schema))
        != EXPECTED_ORCHESTRATOR_RUNTIME_SCHEMA_CANONICAL_SHA256
    ):
        errors.append("orchestrator runtime child schema canonical digest is invalid")
    if (
        child_schema.get("$schema")
        != "https://json-schema.org/draft/2020-12/schema"
        or child_schema.get("$id")
        != "https://qiongli.dev/schemas/ctr-201-orchestrator-runtime.schema.json"
    ):
        errors.append("orchestrator runtime child schema identity is invalid")
    errors.extend(
        _validate_recursively_closed_schema(
            child_schema, label="orchestrator runtime child"
        )
    )
    if validate_instance(artifact, child_schema):
        errors.append(
            "orchestrator runtime child artifact does not satisfy its closed schema"
        )
        return sorted(set(errors))

    try:
        from tooling.scripts import (
            extract_ctr_201_orchestrator_runtime_inventory as extractor,
        )

        extractor.validate_runtime_artifact(artifact)
    except Exception:
        errors.append("orchestrator runtime child semantic validation failed")
        return sorted(set(errors))

    static_artifact = _load_json_file(
        repo_root, DEFAULT_ORCHESTRATOR_ARTIFACT, label="orchestrator child artifact"
    )
    static_source = static_artifact.get("source")
    expected_source = (
        {
            "accepted_tag": EXPECTED_TAG,
            "accepted_commit": EXPECTED_COMMIT,
            "a8_manifest": {
                "path": EXPECTED_MANIFEST_PATH,
                "sha256": EXPECTED_MANIFEST_SHA256,
            },
            "python_full_oracle": static_source.get("python_full_oracle"),
            "package_trees": static_source.get("package_trees"),
            "blob_anchors": static_source.get("blob_anchors"),
            "ctr_201c": {
                "artifact_path": DEFAULT_ORCHESTRATOR_ARTIFACT,
                "schema_path": DEFAULT_ORCHESTRATOR_SCHEMA,
                "schema_canonical_sha256": (
                    EXPECTED_ORCHESTRATOR_SCHEMA_CANONICAL_SHA256
                ),
                "payload_sha256": EXPECTED_ORCHESTRATOR_PAYLOAD_SHA256,
            },
            "ctr_201d": {
                "artifact_path": DEFAULT_CONTENT_ARTIFACT,
                "payload_sha256": EXPECTED_CONTENT_PAYLOAD_SHA256,
            },
            "accepted_manifest_corpus_sha256": EXPECTED_CORPUS_SHA256,
        }
        if isinstance(static_source, Mapping)
        else None
    )
    if artifact.get("source") != expected_source:
        errors.append("orchestrator runtime child frozen-source binding is invalid")

    integrity = artifact.get("integrity")
    coverage = artifact.get("coverage")
    cases = artifact.get("cases")
    dimensions = artifact.get("behavior_dimensions")
    decisions = artifact.get("disposition_decisions")
    if not isinstance(integrity, Mapping) or (
        integrity.get("payload_sha256")
        != EXPECTED_ORCHESTRATOR_RUNTIME_PAYLOAD_SHA256
        or integrity.get("case_manifest_sha256")
        != EXPECTED_ORCHESTRATOR_RUNTIME_CASE_MANIFEST_SHA256
        or binding.get("payload_sha256") != integrity.get("payload_sha256")
        or binding.get("case_manifest_sha256")
        != integrity.get("case_manifest_sha256")
    ):
        errors.append(
            "orchestrator runtime child integrity does not match the master binding"
        )
    if not isinstance(coverage, Mapping) or dict(coverage) != (
        EXPECTED_ORCHESTRATOR_RUNTIME_COVERAGE
    ):
        errors.append("orchestrator runtime child coverage boundary is invalid")
    if (
        not isinstance(cases, list)
        or binding.get("case_count") != len(cases)
        or not isinstance(dimensions, list)
        or binding.get("resolved_dimension_count") != len(dimensions)
        or [
            item.get("id") for item in dimensions if isinstance(item, Mapping)
        ]
        != list(EXPECTED_ORCHESTRATOR_RUNTIME_DIMENSION_IDS)
        or not isinstance(decisions, list)
        or binding.get("disposition_decision_count") != len(decisions)
    ):
        errors.append("orchestrator runtime child counts do not match the master binding")
    return sorted(set(errors))


def _validate_completion_claims(record: Mapping[str, Any]) -> list[str]:
    errors: list[str] = []
    completion = record.get("completion")
    cli = record.get("cli")
    orchestrator = record.get("orchestrator")
    content = record.get("content")
    if (
        record.get("task_id") != "CTR-201A"
        or record.get("status") != "complete"
        or not isinstance(completion, Mapping)
        or completion.get("ctr_201") != "complete"
        or completion.get("fnd_202") != "not-implemented"
        or completion.get("completion_ready") is not True
        or not isinstance(cli, Mapping)
        or cli.get("completion_ready") is not True
        or not isinstance(orchestrator, Mapping)
        or orchestrator.get("completion_ready") is not True
        or orchestrator.get("required_not_fully_captured") != []
        or not isinstance(content, Mapping)
        or content.get("completion_ready") is not True
    ):
        errors.append(
            "CTR-201 accepted-source inventory must be complete while FND-202 "
            "remains not implemented"
        )
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
            parse_constant=_reject_non_finite_constant,
        )
    except (InventoryConfigError, OSError, UnicodeDecodeError, json.JSONDecodeError):
        return None, [f"{label} cannot be verified"]
    if not isinstance(value, dict):
        return None, [f"{label} must contain a JSON object"]
    if _contains_unicode_surrogate(value):
        return None, [f"{label} contains invalid Unicode scalar data"]
    if _contains_non_finite_number(value):
        return None, [f"{label} contains invalid numeric data"]
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
    manifest: Mapping[str, Any],
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
    manifest_domains = manifest.get("domains")
    mcp_domains = (
        [
            domain
            for domain in manifest_domains
            if isinstance(domain, Mapping) and domain.get("id") == "mcp"
        ]
        if isinstance(manifest_domains, list)
        else []
    )
    registry_entries: list[Mapping[str, Any]] = []
    if len(mcp_domains) == 1:
        files = mcp_domains[0].get("files")
        if isinstance(files, list):
            registry_entries = [
                item
                for item in files
                if isinstance(item, Mapping)
                and item.get("path") == EXPECTED_REGISTRY_PATH
            ]
    if len(registry_entries) != 1 or (
        registry_entries[0].get("path"),
        registry_entries[0].get("sha256"),
    ) != (EXPECTED_REGISTRY_PATH, EXPECTED_REGISTRY_SHA256):
        errors.append(
            "Contract v2 registry binding is absent from accepted A8 MCP metadata"
        )
    elif (
        contract.get("registry_path"),
        contract.get("registry_sha256"),
    ) != (
        registry_entries[0].get("path"),
        registry_entries[0].get("sha256"),
    ):
        errors.append("Contract v2 registry binding differs from accepted A8 metadata")

    # CTR-201 is a historical freeze. Its six-tool pilot facts remain recorded in
    # the immutable ledger even after the live Contract v2 registry advances.
    actual_contract = (
        contract.get("status"),
        contract.get("coverage_mode"),
        contract.get("canonical_tool_count"),
        contract.get("public_name_count"),
        contract.get("target_canonical_tool_count"),
        contract.get("target_public_name_count"),
    )
    if actual_contract != ("pilot", "pilot", 6, 7, 23, 24):
        errors.append("Contract v2 pilot coverage does not match the frozen CTR-201 facts")
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
    oracle_cases = python_oracle.get("cases")
    orchestration_cases = (
        [
            case
            for case in oracle_cases
            if isinstance(case, Mapping)
            and case.get("id") == "python.orchestration-preview"
        ]
        if isinstance(oracle_cases, list)
        else []
    )
    if (
        len(orchestration_cases) != 1
        or orchestration_cases[0].get("outcome")
        != EXPECTED_ORCHESTRATION_ORACLE_OUTCOME
    ):
        errors.append("Python Full orchestration oracle outcome is not exact")
    cli = record.get("cli")
    cli_case_ids = ["python.cli-align", "python.installer-dry-run"]
    actual_cli_cases = [
        case_id
        for coverage in ("cli-command", "installer-dry-run")
        for case_id in _case_ids_for_coverage(python_oracle, coverage)
    ]
    if not isinstance(cli, Mapping) or (
        cli.get("status") != "runtime-inventory-frozen"
        or cli.get("captured_oracle_cases") != cli_case_ids
        or actual_cli_cases != cli_case_ids
        or cli.get("captured_scope") != list(EXPECTED_CLI_CAPTURED_SCOPE)
        or cli.get("required_not_fully_captured") != list(EXPECTED_CLI_GAPS)
        or cli.get("completion_ready") is not True
    ):
        errors.append("CLI coverage must remain explicit and runtime-inventory-frozen")

    orchestrator = record.get("orchestrator")
    orchestrator_case_ids = ["python.orchestration-preview"]
    actual_orchestrator_cases = _case_ids_for_coverage(
        python_oracle, "orchestration-preview"
    )
    if not isinstance(orchestrator, Mapping) or (
        orchestrator.get("status") != "runtime-inventory-frozen"
        or orchestrator.get("captured_oracle_cases") != orchestrator_case_ids
        or actual_orchestrator_cases != orchestrator_case_ids
        or orchestrator.get("captured_scope")
        != list(EXPECTED_ORCHESTRATOR_CAPTURED_SCOPE)
        or orchestrator.get("required_not_fully_captured")
        != list(EXPECTED_ORCHESTRATOR_GAPS)
        or orchestrator.get("completion_ready") is not True
    ):
        errors.append(
            "orchestrator coverage must remain explicit and runtime-inventory-frozen"
        )
    return errors


def _is_portable_content_path(value: Any, *, allow_trailing_slash: bool = False) -> bool:
    if not isinstance(value, str):
        return False
    candidate = value[:-1] if allow_trailing_slash and value.endswith("/") else value
    return (
        unicodedata.normalize("NFC", candidate) == candidate
        and len(candidate.encode("utf-8")) <= 512
        and len(PurePosixPath(candidate).parts) <= 32
        and is_canonical_repository_path(
            value, allow_trailing_slash=allow_trailing_slash
        )
    )


def _validate_content_materialized_tree(
    tree: Any,
    *,
    expected: Mapping[str, Any] | None = None,
) -> list[str]:
    if not isinstance(tree, Mapping):
        return ["content child materialized tree is invalid"]
    entries = tree.get("entries")
    if not isinstance(entries, list) or not all(
        isinstance(entry, Mapping) for entry in entries
    ):
        return ["content child materialized tree entries are invalid"]
    errors: list[str] = []
    paths = [entry.get("path") for entry in entries]
    if any(not _is_portable_content_path(path) for path in paths):
        errors.append("content child contains a non-canonical materialized path")
    string_paths = [path for path in paths if isinstance(path, str)]
    if (
        len(string_paths) != len(set(string_paths))
        or len({unicodedata.normalize("NFC", path).casefold() for path in string_paths})
        != len(string_paths)
        or string_paths != sorted(string_paths, key=lambda path: path.encode("utf-8"))
    ):
        errors.append("content child materialized paths are not unique and ordered")

    total_bytes = 0
    origin_counts = {
        "identity-copy": 0,
        "content-transform": 0,
        "generated-metadata": 0,
    }
    top_level: dict[str, int] = {}
    digest_rows: list[dict[str, Any]] = []
    for entry in entries:
        size = entry.get("size_bytes")
        digest = entry.get("sha256")
        mode = entry.get("mode")
        origin = entry.get("origin")
        path = entry.get("path")
        if (
            not isinstance(size, int)
            or isinstance(size, bool)
            or size < 0
            or not isinstance(digest, str)
            or re.fullmatch(r"[0-9a-f]{64}", digest) is None
            or mode != "0644"
            or not isinstance(path, str)
            or not isinstance(origin, Mapping)
        ):
            errors.append("content child materialized entry metadata is invalid")
            continue
        origin_class = origin.get("origin_class")
        source_paths = origin.get("source_paths")
        if (
            origin_class not in origin_counts
            or not isinstance(source_paths, list)
            or not source_paths
            or any(not _is_portable_content_path(item) for item in source_paths)
        ):
            errors.append("content child materialized provenance is invalid")
        else:
            origin_counts[str(origin_class)] += 1
        total_bytes += size
        top = path.split("/", 1)[0]
        top_level[top] = top_level.get(top, 0) + 1
        digest_rows.append(
            {
                "path": path,
                "mode": mode,
                "size_bytes": size,
                "sha256": digest,
            }
        )
    expected_top = [
        {"name": name, "file_count": top_level[name]}
        for name in sorted(top_level, key=lambda value: value.encode("utf-8"))
    ]
    if (
        tree.get("root") != "normalized-skill-root"
        or tree.get("file_count") != len(entries)
        or tree.get("total_bytes") != total_bytes
        or tree.get("tree_sha256") != _sha256(_canonical_json_bytes(digest_rows))
        or tree.get("origin_counts") != origin_counts
        or tree.get("top_level_counts") != expected_top
    ):
        errors.append("content child materialized tree summary is invalid")
    if expected is not None and (
        tree.get("file_count") != expected.get("file_count")
        or tree.get("total_bytes") != expected.get("total_bytes")
        or tree.get("tree_sha256") != expected.get("tree_sha256")
        or tree.get("origin_counts") != expected.get("origin_counts")
    ):
        errors.append("content child materialized tree identity is not exact")
    return sorted(set(errors))


def _validate_content_repository_paths(artifact: Mapping[str, Any]) -> list[str]:
    checks: list[tuple[Any, bool]] = []
    source = artifact.get("source")
    if isinstance(source, Mapping):
        checks.append((source.get("manifest_path"), False))
        content_tree = source.get("content_tree")
        if isinstance(content_tree, Mapping):
            checks.append((content_tree.get("root"), True))
            files = content_tree.get("files")
            if isinstance(files, list):
                checks.extend(
                    (
                        item.get("path") if isinstance(item, Mapping) else None,
                        False,
                    )
                    for item in files
                )
            else:
                checks.append((None, False))
        anchors = source.get("materializer_anchors")
        if isinstance(anchors, list):
            checks.extend(
                (
                    item.get("path") if isinstance(item, Mapping) else None,
                    False,
                )
                for item in anchors
            )
        else:
            checks.append((None, False))
    else:
        checks.append((None, False))

    catalog = artifact.get("resource_catalog")
    if isinstance(catalog, Mapping):
        for root in catalog.get("roots", []):
            if isinstance(root, Mapping):
                checks.append((root.get("source"), root.get("match") == "prefix"))
            else:
                checks.append((None, False))
        for kind in catalog.get("kinds", []):
            sources = kind.get("source_roots") if isinstance(kind, Mapping) else None
            if isinstance(sources, list):
                checks.extend((value, value.endswith("/") if isinstance(value, str) else False) for value in sources)
            else:
                checks.append((None, False))
    else:
        checks.append((None, False))

    for projection in [artifact.get("portable_core"), *(
        artifact.get("profiles") if isinstance(artifact.get("profiles"), list) else []
    )]:
        tree = projection.get("materialized_tree") if isinstance(projection, Mapping) else None
        entries = tree.get("entries") if isinstance(tree, Mapping) else None
        if not isinstance(entries, list):
            checks.append((None, False))
            continue
        for entry in entries:
            if not isinstance(entry, Mapping):
                checks.append((None, False))
                continue
            checks.append((entry.get("path"), False))
            origin = entry.get("origin")
            sources = origin.get("source_paths") if isinstance(origin, Mapping) else None
            if isinstance(sources, list):
                checks.extend((path, False) for path in sources)
            else:
                checks.append((None, False))
    if any(
        not _is_portable_content_path(path, allow_trailing_slash=trailing)
        for path, trailing in checks
    ):
        return ["content child contains a non-canonical portable path"]
    return []


def _validate_content_artifact_semantics(
    artifact: Mapping[str, Any],
) -> list[str]:
    if _contains_unicode_surrogate(artifact):
        return ["content child contains invalid Unicode scalar data"]
    if _contains_non_finite_number(artifact):
        return ["content child contains invalid numeric data"]
    errors: list[str] = []
    strings = _iter_strings(artifact)
    if any(MACHINE_PATH_PATTERN.search(value) for value in strings):
        errors.append("content child contains a forbidden machine-local path")
    if any(SECRET_PATTERN.search(value) for value in strings):
        errors.append("content child contains forbidden secret-shaped data")
    if any(CALLABLE_REPR_PATTERN.search(value) for value in strings):
        errors.append("content child contains an unstable callable representation")
    errors.extend(_validate_content_repository_paths(artifact))
    if (
        artifact.get("task_id") != "CTR-201D"
        or artifact.get("status") != "content-materialization-captured"
    ):
        errors.append("content child status boundary is invalid")
    integrity = artifact.get("integrity")
    if (
        not isinstance(integrity, Mapping)
        or integrity.get("algorithm") != "sha256"
        or integrity.get("canonicalization")
        != "utf-8-json-sorted-keys-compact-excluding-integrity"
        or integrity.get("payload_sha256") != canonical_payload_sha256(artifact)
    ):
        errors.append("content child canonical payload digest does not match")
    coverage = artifact.get("coverage")
    if not isinstance(coverage, Mapping) or dict(coverage) != {
        **EXPECTED_CONTENT_COUNTS,
        "capture_ready": True,
    }:
        errors.append("content child coverage boundary is invalid")
    source = artifact.get("source")
    content_tree = source.get("content_tree") if isinstance(source, Mapping) else None
    files = content_tree.get("files") if isinstance(content_tree, Mapping) else None
    if (
        not isinstance(source, Mapping)
        or source.get("accepted_tag") != EXPECTED_TAG
        or source.get("accepted_commit") != EXPECTED_COMMIT
        or source.get("manifest_path") != EXPECTED_MANIFEST_PATH
        or source.get("manifest_sha256") != EXPECTED_MANIFEST_SHA256
        or not isinstance(content_tree, Mapping)
        or content_tree.get("root") != "content/"
        or content_tree.get("file_count") != EXPECTED_CONTENT_FILE_COUNT
        or content_tree.get("total_bytes") != 1_761_400
        or content_tree.get("tree_sha256") != EXPECTED_CONTENT_TREE_SHA256
        or not isinstance(files, list)
        or len(files) != EXPECTED_CONTENT_FILE_COUNT
        or _sha256(_canonical_json_bytes(files)) != EXPECTED_CONTENT_TREE_SHA256
    ):
        errors.append("content child frozen-source binding is invalid")

    catalog = artifact.get("resource_catalog")
    roots = catalog.get("roots") if isinstance(catalog, Mapping) else None
    recorded_roots = [
        (
            item.get("source"),
            item.get("match"),
            item.get("resource_kind"),
            item.get("file_count"),
        )
        for item in roots
        if isinstance(item, Mapping)
    ] if isinstance(roots, list) else []
    if (
        not isinstance(catalog, Mapping)
        or catalog.get("resource_root_count") != 12
        or catalog.get("resource_kind_count") != 11
        or tuple(recorded_roots) != EXPECTED_RESOURCE_ROOTS
    ):
        errors.append("content child resource catalog is invalid")
    if isinstance(files, list) and isinstance(roots, list):
        for root in roots:
            if not isinstance(root, Mapping):
                errors.append("content child resource catalog is invalid")
                continue
            selected = [
                item
                for item in files
                if isinstance(item, Mapping)
                and isinstance(item.get("path"), str)
                and isinstance(root.get("source"), str)
                and _matches_root(
                    str(item["path"]),
                    str(root["source"]),
                    str(root.get("match")),
                )
            ]
            if (
                root.get("file_count") != len(selected)
                or root.get("total_bytes")
                != sum(int(item.get("size_bytes", -1)) for item in selected)
                or root.get("entries_sha256")
                != _sha256(_canonical_json_bytes(selected))
            ):
                errors.append("content child resource-root digest is invalid")
        if any(
            sum(
                _matches_root(
                    str(item.get("path")),
                    str(root.get("source")),
                    str(root.get("match")),
                )
                for root in roots
                if isinstance(root, Mapping)
            )
            != 1
            for item in files
            if isinstance(item, Mapping)
        ):
            errors.append("content child resource-root partition is invalid")

    contract = artifact.get("materialization_contract")
    if not isinstance(contract, Mapping) or dict(contract) != {
        "python_requirement": ">=3.12",
        "pyyaml_version": "6.0.3",
        "execution_model": (
            "authenticated-accepted-materializer-in-ephemeral-isolated-subprocess"
        ),
        "input_policy": "A8-authenticated-regular-blobs-only",
        "path_policy": "relative-posix-utf8-nfc-portable-casefold-unique",
        "mode_policy": "regular-skill-resources-normalized-to-0644",
        "entry_order": "ascending-utf8-path-bytes",
        "tree_canonicalization": (
            "sha256-of-utf8-compact-sorted-key-json-over-utf8-byte-sorted-"
            "path-mode-size_bytes-sha256-entry-array"
        ),
        "worker_network": (
            "accepted-reference-code-static-allowlist-plus-python-audit-denial;"
            "os-network-sandbox-not-proven"
        ),
        "worker_write_scope": "ephemeral-temporary-root-only",
        "host_cache_writes": "forbidden",
    }:
        errors.append("content child materialization contract is invalid")

    portable = artifact.get("portable_core")
    portable_tree = portable.get("materialized_tree") if isinstance(portable, Mapping) else None
    if not isinstance(portable, Mapping) or portable.get("variant_id") != EXPECTED_PORTABLE_CORE_FACTS["variant_id"]:
        errors.append("content child portable-core identity is invalid")
    errors.extend(
        _validate_content_materialized_tree(
            portable_tree,
            expected=EXPECTED_PORTABLE_CORE_FACTS,
        )
    )

    profiles = artifact.get("profiles")
    if not isinstance(profiles, list) or len(profiles) != len(EXPECTED_CONTENT_PROFILE_FACTS):
        errors.append("content child profile inventory is invalid")
        profiles = []
    for profile, expected in zip(profiles, EXPECTED_CONTENT_PROFILE_FACTS):
        if not isinstance(profile, Mapping):
            errors.append("content child profile inventory is invalid")
            continue
        closure = profile.get("source_closure")
        tree = profile.get("materialized_tree")
        if any(profile.get(key) != expected[key] for key in (
            "profile_id", "aliases", "variant_id", "subject", "flavor", "coverage", "skill_name"
        )):
            errors.append("content child profile identity is not exact")
        if (
            not isinstance(closure, Mapping)
            or closure.get("file_count") != expected["source_file_count"]
            or closure.get("total_bytes") != expected["source_total_bytes"]
            or closure.get("tree_sha256") != expected["source_tree_sha256"]
        ):
            errors.append("content child profile source closure is not exact")
        errors.extend(
            _validate_content_materialized_tree(
                tree,
                expected={
                    "file_count": expected["materialized_file_count"],
                    "total_bytes": expected["materialized_total_bytes"],
                    "tree_sha256": expected["materialized_tree_sha256"],
                    "origin_counts": expected["origin_counts"],
                },
            )
        )
        if (
            profile.get("evidence_scope")
            != "authenticated-accepted-source-skill-subtree"
            or profile.get("published_archive_member_parity") != "not-captured"
        ):
            errors.append("content child profile evidence boundary is invalid")

    compatibility = artifact.get("compatibility_boundary")
    if not isinstance(compatibility, Mapping) or (
        compatibility.get("a8_generated_tree_evidence") is not False
        or compatibility.get("published_archive_member_parity") != "not-captured"
        or compatibility.get("complete_plugin_wrapper_parity") != "not-captured"
        or compatibility.get("complete_native_binary_parity") != "not-captured"
        or compatibility.get("complete_subject_matrix_parity") != "not-captured"
        or compatibility.get("extraction_network_sandbox") != "not-proven"
        or compatibility.get("extraction_filesystem_sandbox")
        != "python-audit-write-confined;host-read-isolation-not-proven;os-sandbox-not-proven"
        or compatibility.get("fnd_202_implemented") is not False
    ):
        errors.append("content child compatibility boundary is invalid")
    return sorted(set(errors))


def _matches_root(path: str, source: str, match: str) -> bool:
    return path == source if match == "exact" else path.startswith(source)


def _validate_content_materialization_contract(
    repo_root: Path,
    record: Mapping[str, Any],
    manifest: Mapping[str, Any] | None = None,
) -> list[str]:
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

    package_trees = manifest.get("package_trees") if isinstance(manifest, Mapping) else None
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
        content.get("status") != "content-materialization-captured"
        or content.get("source_file_count") != EXPECTED_CONTENT_FILE_COUNT
        or content.get("source_total_bytes") != 1_761_400
        or content.get("source_tree_sha256") != EXPECTED_CONTENT_TREE_SHA256
    ):
        errors.append("content source identity does not match the frozen tree")

    profiles = content.get("profiles")
    profile_names = [
        item.get("profile_id") for item in profiles if isinstance(item, Mapping)
    ] if isinstance(profiles, list) else []
    if tuple(profile_names) != EXPECTED_PROFILES or len(profile_names) != len(set(profile_names)):
        errors.append("content profiles must be unique and ordered")
    expected_parent_profiles = [
        {
            "profile_id": expected["profile_id"],
            "aliases": expected["aliases"],
            "variant_id": expected["variant_id"],
            "source_file_count": expected["source_file_count"],
            "materialized_file_count": expected["materialized_file_count"],
            "materialized_total_bytes": expected["materialized_total_bytes"],
            "expected_materialized_tree_sha256": expected[
                "materialized_tree_sha256"
            ],
            "identity_output_count": expected["origin_counts"]["identity-copy"],
            "transformed_output_count": expected["origin_counts"][
                "content-transform"
            ],
            "generated_output_count": expected["origin_counts"][
                "generated-metadata"
            ],
        }
        for expected in EXPECTED_CONTENT_PROFILE_FACTS
    ]
    if profiles != expected_parent_profiles:
        errors.append("content profile materialization summaries are not exact")
    materialization = content.get("materialization")
    if (
        not isinstance(materialization, Mapping)
        or dict(materialization)
        != {
            "status": "content-materialization-captured",
            "mapping_policy": (
                "authenticated-accepted-materializer-in-ephemeral-isolated-subprocess"
            ),
            "published_archive_parity": "not-captured",
            "extraction_network_sandbox": "not-proven",
            "extraction_filesystem_sandbox": (
                "python-audit-write-confined;host-read-isolation-not-proven;os-sandbox-not-proven"
            ),
            "rust_materializer": "not-implemented",
        }
        or content.get("completion_ready") is not True
    ):
        errors.append("content materialization boundary is invalid")

    binding = content.get("static_inventory")
    expected_binding = {
        "task_id": "CTR-201D",
        "status": "content-materialization-captured",
        "artifact_path": DEFAULT_CONTENT_ARTIFACT,
        "schema_path": DEFAULT_CONTENT_SCHEMA,
        "schema_canonical_sha256": EXPECTED_CONTENT_SCHEMA_CANONICAL_SHA256,
        "payload_sha256": EXPECTED_CONTENT_PAYLOAD_SHA256,
        **EXPECTED_CONTENT_COUNTS,
        "capture_ready": True,
    }
    if not isinstance(binding, Mapping) or dict(binding) != expected_binding:
        errors.append("content static-inventory master binding is invalid")
        return sorted(set(errors))

    child_schema = _load_json_file(
        repo_root, DEFAULT_CONTENT_SCHEMA, label="content child schema"
    )
    artifact = _load_json_file(
        repo_root, DEFAULT_CONTENT_ARTIFACT, label="content child artifact"
    )
    if (
        _sha256(_canonical_json_bytes(child_schema))
        != EXPECTED_CONTENT_SCHEMA_CANONICAL_SHA256
    ):
        errors.append("content child schema canonical digest is invalid")
    if (
        child_schema.get("$schema")
        != "https://json-schema.org/draft/2020-12/schema"
        or child_schema.get("$id")
        != "https://qiongli.dev/schemas/ctr-201-content.schema.json"
    ):
        errors.append("content child schema identity is invalid")
    errors.extend(
        _validate_recursively_closed_schema(child_schema, label="content child")
    )
    if validate_instance(artifact, child_schema):
        errors.append("content child artifact does not satisfy its closed schema")
        return sorted(set(errors))
    errors.extend(_validate_content_artifact_semantics(artifact))

    child_integrity = artifact.get("integrity")
    child_coverage = artifact.get("coverage")
    if (
        not isinstance(child_integrity, Mapping)
        or child_integrity.get("payload_sha256") != EXPECTED_CONTENT_PAYLOAD_SHA256
        or binding.get("payload_sha256") != child_integrity.get("payload_sha256")
    ):
        errors.append("content child payload digest does not match the master binding")
    if not isinstance(child_coverage, Mapping) or any(
        binding.get(key) != child_coverage.get(key)
        for key in (*EXPECTED_CONTENT_COUNTS, "capture_ready")
    ):
        errors.append("content child counts do not match the master binding")
    child_source = artifact.get("source")
    child_tree = (
        child_source.get("content_tree") if isinstance(child_source, Mapping) else None
    )
    child_files = child_tree.get("files") if isinstance(child_tree, Mapping) else None
    if isinstance(files, list) and child_files != files:
        errors.append("content child source inventory differs from the frozen manifest")
    child_profiles = artifact.get("profiles")
    if isinstance(child_profiles, list) and len(child_profiles) == len(expected_parent_profiles):
        projected = []
        for profile in child_profiles:
            if not isinstance(profile, Mapping):
                projected = []
                break
            closure = profile.get("source_closure")
            tree = profile.get("materialized_tree")
            origins = tree.get("origin_counts") if isinstance(tree, Mapping) else None
            if not isinstance(closure, Mapping) or not isinstance(tree, Mapping) or not isinstance(origins, Mapping):
                projected = []
                break
            projected.append(
                {
                    "profile_id": profile.get("profile_id"),
                    "aliases": profile.get("aliases"),
                    "variant_id": profile.get("variant_id"),
                    "source_file_count": closure.get("file_count"),
                    "materialized_file_count": tree.get("file_count"),
                    "materialized_total_bytes": tree.get("total_bytes"),
                    "expected_materialized_tree_sha256": tree.get("tree_sha256"),
                    "identity_output_count": origins.get("identity-copy"),
                    "transformed_output_count": origins.get("content-transform"),
                    "generated_output_count": origins.get("generated-metadata"),
                }
            )
        if projected != expected_parent_profiles or projected != profiles:
            errors.append("content child profiles do not match the master summaries")
    else:
        errors.append("content child profiles do not match the master summaries")
    if errors:
        return sorted(set(errors))
    try:
        extracted = _accepted_content_extraction_bytes(repo_root)
    except ContentArtifactMismatch:
        return ["accepted content extraction does not match its frozen source"]
    if extracted != _canonical_json_bytes(artifact):
        errors.append("content child artifact differs from accepted-source extraction")
    return sorted(set(errors))


def _validate_content(
    repo_root: Path,
    record: Mapping[str, Any],
    manifest: Mapping[str, Any],
) -> list[str]:
    return _validate_content_materialization_contract(repo_root, record, manifest)


def validate_inventory(
    repo_root: Path,
    record: Mapping[str, Any],
    schema: Mapping[str, Any],
) -> list[str]:
    if _contains_unicode_surrogate(schema):
        return ["inventory schema contains invalid Unicode scalar data"]
    if _contains_unicode_surrogate(record):
        return ["inventory contains invalid Unicode scalar data"]
    if _contains_non_finite_number(schema):
        return ["inventory schema contains invalid numeric data"]
    if _contains_non_finite_number(record):
        return ["inventory contains invalid numeric data"]
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
    errors.extend(_validate_contract_and_target(repo_root, record, manifest))
    errors.extend(_validate_coverage_gaps(record, oracle_documents))
    errors.extend(_validate_cli_static_semantics(repo_root, record))
    errors.extend(_validate_cli_runtime_freeze(repo_root, record))
    errors.extend(_validate_orchestrator_static_contract(repo_root, record))
    errors.extend(_validate_orchestrator_runtime_freeze(repo_root, record))
    errors.extend(_validate_content(repo_root, record, manifest))
    return sorted(set(errors))


def _parser() -> argparse.ArgumentParser:
    parser = SafeArgumentParser(
        description=(
            "Validate the derived and accepted-source CTR-201A/B/C/D/E/F "
            "inventories."
        )
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
            "[ctr-201] PASS: accepted-source semantic inventory is complete; "
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

    try:
        errors = validate_inventory(root, record, schema)
    except (InventoryConfigError, OSError, RuntimeError):
        payload = {
            "status": "error",
            "exit_code": 2,
            "error_count": 1,
            "errors": ["validator configuration could not be loaded safely"],
        }
        _emit(payload, as_json=args.json)
        return 2
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
        "ctr_201": "complete",
        "fnd_202": "not-implemented",
    }
    _emit(payload, as_json=args.json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
