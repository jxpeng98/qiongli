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
DEFAULT_CLI_ARTIFACT = "tooling/migration/ctr-201-cli.json"
DEFAULT_CLI_SCHEMA = "tooling/migration/ctr-201-cli.schema.json"
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
    "33403b58331a946b266a5e60360ac376c8be0119f4cf565f1544fa11f0b7d02b"
)
EXPECTED_CLI_SCHEMA_CANONICAL_SHA256 = (
    "173436615a8a26d45903cc7812a55f2e9ae094089f637bced0f418a3976456ad"
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
)
EXPECTED_CLI_GAPS = (
    "complete-formatted-help-output",
    "complete-json-output",
    "complete-exit-code-matrix",
    "complete-dry-run-semantics",
    "complete-error-classes",
    "complete-legacy-npm-compatibility-surface",
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
CALLABLE_REPR_PATTERN = re.compile(
    r"(?:<(?:(?:bound )?method|function|class)\b|\bat 0x[0-9a-f]+>)",
    re.IGNORECASE,
)
WINDOWS_RESERVED_COMPONENT = re.compile(
    r"^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\..*)?$",
    re.IGNORECASE,
)


_CLI_EXTRACTION_CACHE: dict[str, bytes] = {}


class InventoryConfigError(ValueError):
    """Raised when validator inputs cannot be loaded safely."""


class CliArtifactMismatch(ValueError):
    """Raised when accepted source extraction disagrees with the child artifact."""


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


def _validate_recursively_closed_schema(schema: Mapping[str, Any]) -> list[str]:
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
                    return ["CLI child schema must be recursively closed"]
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
    errors.extend(_validate_recursively_closed_schema(child_schema))
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
            list(EXPECTED_CLI_CAPTURED_SCOPE),
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
    errors.extend(_validate_contract_and_target(repo_root, record))
    errors.extend(_validate_coverage_gaps(record, oracle_documents))
    errors.extend(_validate_cli_static_semantics(repo_root, record))
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
        "ctr_201": "in-progress",
        "fnd_202": "not-implemented",
    }
    _emit(payload, as_json=args.json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
