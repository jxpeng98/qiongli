#!/usr/bin/env python3
from __future__ import annotations

import argparse
from datetime import datetime
import importlib
import json
import math
import re
import sys
from pathlib import Path
from typing import Any, Mapping
from urllib.parse import urlsplit


REPO_ROOT = Path(__file__).resolve().parents[2]
REGISTRY_RELATIVE = Path("content/mcp-contracts/v2/registry.json")
LITE_TOOLS_RELATIVE = Path("content/mcp-contracts/lite-tools.json")
COMPLETE_SMOKE_CALLS_RELATIVE = Path(
    "content/mcp-contracts/fixtures/capability-contract-v2-smoke-calls.json"
)
LEGACY_SMOKE_CALLS_RELATIVE = Path(
    "content/mcp-contracts/fixtures/lite-tool-smoke-calls.json"
)
MCPB_MANIFEST_RELATIVE = Path("packages/qiongli-literature-mcpb/manifest.json")
FULL_SOURCE_RELATIVE = Path("packages/python-qiongli/src/qiongli")
EXPECTED_PROFILES = ("skill-only", "marketplace-lite", "full")
RUNTIME_PROFILES = ("marketplace-lite", "full")
CONTRACT_SCHEMA_DIALECT = "https://json-schema.org/draft/2020-12/schema"
FORBIDDEN_LITE_SIDE_EFFECTS = {"project-write", "process-launch", "agent-launch"}
COMPATIBILITY_ALIAS_PATTERN = re.compile(
    r"^Compatibility alias for (?P<canonical>[A-Za-z0-9_]+)\.$"
)

# These are the fail-closed materialized-root fallback and the independent
# integrity oracle for a checked-in CTR-201 ledger. Do not derive them from the
# live Full/Lite implementations: deletion from both runtimes must still fail.
EXPECTED_TARGET_CANONICAL_NAMES = (
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
    "qiongli_config_status",
    "qiongli_save_provider_config",
    "qiongli_configure_provider",
    "qiongli_collect_evidence",
    "qiongli_list_provider_env",
    "qiongli_test_provider",
    "qiongli_subject_status",
    "qiongli_subject_update",
    "qiongli_orchestrator_route",
    "qiongli_orchestrator_doctor",
    "qiongli_lifecycle_plan",
    "qiongli_journal_fit_recommend",
    "qiongli_experience_query",
    "qiongli_experience_show",
    "qiongli_experience_lessons",
    "qiongli_task_plan",
    "qiongli_task_run",
    "qiongli_zotero_status",
    "qiongli_zotero_search",
    "qiongli_zotero_upsert_references",
    "qiongli_zotero_export_import_files",
)
EXPECTED_TARGET_PUBLIC_NAMES = (
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
    "qiongli_config_status",
    "qiongli_save_provider_config",
    "qiongli_configure_provider",
    "qiongli_collect_evidence",
    "qiongli_list_provider_env",
    "qiongli_test_provider",
    "qiongli_subject_status",
    "qiongli_subject_update",
    "qiongli_open_config_wizard",
    "qiongli_orchestrator_route",
    "qiongli_orchestrator_doctor",
    "qiongli_lifecycle_plan",
    "qiongli_journal_fit_recommend",
    "qiongli_experience_query",
    "qiongli_experience_show",
    "qiongli_experience_lessons",
    "qiongli_task_plan",
    "qiongli_task_run",
    "qiongli_zotero_status",
    "qiongli_zotero_search",
    "qiongli_zotero_upsert_references",
    "qiongli_zotero_export_import_files",
)
EXPECTED_FULL_PUBLIC_NAMES = (
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
    "qiongli_config_status",
    "qiongli_save_provider_config",
    "qiongli_configure_provider",
    "qiongli_collect_evidence",
    "qiongli_list_provider_env",
    "qiongli_test_provider",
    "qiongli_subject_status",
    "qiongli_subject_update",
    "qiongli_open_config_wizard",
    "qiongli_orchestrator_route",
    "qiongli_orchestrator_doctor",
    "qiongli_lifecycle_plan",
    "qiongli_journal_fit_recommend",
    "qiongli_experience_query",
    "qiongli_experience_show",
    "qiongli_experience_lessons",
    "qiongli_task_plan",
    "qiongli_task_run",
    "qiongli_zotero_status",
    "qiongli_zotero_search",
    "qiongli_zotero_upsert_references",
    "qiongli_zotero_export_import_files",
)
EXPECTED_LITE_PUBLIC_NAMES = (
    "qiongli_config_status",
    "qiongli_save_provider_config",
    "qiongli_configure_provider",
    "qiongli_open_config_wizard",
    "qiongli_literature_status",
    "qiongli_search_plan",
    "qiongli_literature_search",
    "qiongli_literature_export_evidence",
    "qiongli_zotero_status",
    "qiongli_zotero_search",
    "qiongli_zotero_upsert_references",
    "qiongli_zotero_export_import_files",
    "qiongli_orchestrator_route",
    "qiongli_task_plan",
)
EXPECTED_FROZEN_MCPB_PUBLIC_NAMES = tuple(
    name
    for name in EXPECTED_LITE_PUBLIC_NAMES
    if name not in {"qiongli_zotero_search", "qiongli_zotero_upsert_references"}
)
EXPECTED_INPUT_ERROR_SMOKE_PAIRS = {
    ("marketplace-lite", "qiongli_configure_provider"),
    ("marketplace-lite", "qiongli_open_config_wizard"),
    ("full", "qiongli_configure_provider"),
    ("full", "qiongli_open_config_wizard"),
    ("marketplace-lite", "qiongli_zotero_status"),
    ("full", "qiongli_zotero_status"),
    ("marketplace-lite", "qiongli_zotero_search"),
    ("full", "qiongli_zotero_search"),
    ("marketplace-lite", "qiongli_zotero_upsert_references"),
    ("full", "qiongli_zotero_upsert_references"),
}
EXPECTED_SMOKE_RESPONSE_CLASSES = {
    "input_error",
    "success",
    "bounded_local_result",
}
EXPECTED_ALIASES = {
    "qiongli_open_config_wizard": "qiongli_configure_provider",
}


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError("duplicate JSON object key")
        value[key] = item
    return value


def _reject_non_finite_constant(_: str) -> Any:
    raise ValueError("non-finite JSON constant")


def _contains_non_finite_number(value: Any) -> bool:
    if isinstance(value, float):
        return not math.isfinite(value)
    if isinstance(value, Mapping):
        return any(_contains_non_finite_number(item) for item in value.values())
    if isinstance(value, (list, tuple)):
        return any(_contains_non_finite_number(item) for item in value)
    return False


def _load_json(path: Path) -> Any:
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=_unique_json_object,
            parse_constant=_reject_non_finite_constant,
        )
    except FileNotFoundError as exc:
        raise ValueError(f"missing JSON file: {path}") from exc
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as exc:
        raise ValueError(f"invalid JSON in {path}: {exc}") from exc
    if _contains_non_finite_number(value):
        raise ValueError(f"invalid JSON in {path}: non-finite numeric value")
    return value


def _json_type_matches(value: Any, expected: str) -> bool:
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "string":
        return isinstance(value, str)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "null":
        return value is None
    return False


def _resolve_internal_ref(root_schema: Mapping[str, Any], ref: str) -> Mapping[str, Any]:
    if not ref.startswith("#/"):
        raise ValueError(f"unsupported non-local schema reference: {ref}")
    current: Any = root_schema
    for token in ref[2:].split("/"):
        token = token.replace("~1", "/").replace("~0", "~")
        if not isinstance(current, Mapping) or token not in current:
            raise ValueError(f"unresolved schema reference: {ref}")
        current = current[token]
    if not isinstance(current, Mapping):
        raise ValueError(f"schema reference does not resolve to an object: {ref}")
    return current


def validate_instance(
    value: Any,
    schema: Mapping[str, Any],
    *,
    path: str = "$",
    root_schema: Mapping[str, Any] | None = None,
) -> list[str]:
    root = root_schema or schema
    if "$ref" in schema:
        try:
            target = _resolve_internal_ref(root, str(schema["$ref"]))
        except ValueError as exc:
            return [f"{path}: {exc}"]
        return validate_instance(value, target, path=path, root_schema=root)

    failures: list[str] = []
    one_of = schema.get("oneOf")
    if isinstance(one_of, list):
        matches = [
            option
            for option in one_of
            if isinstance(option, Mapping)
            and not validate_instance(value, option, path=path, root_schema=root)
        ]
        if len(matches) != 1:
            failures.append(f"{path}: expected exactly one oneOf branch to match")
    if "const" in schema and value != schema["const"]:
        failures.append(f"{path}: expected constant {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        failures.append(f"{path}: value {value!r} is not in the allowed enum")

    expected_type = schema.get("type")
    if expected_type is not None:
        expected_types = [expected_type] if isinstance(expected_type, str) else expected_type
        if not isinstance(expected_types, list) or not any(
            isinstance(item, str) and _json_type_matches(value, item)
            for item in expected_types
        ):
            failures.append(f"{path}: expected type {expected_type!r}")
            return failures

    if isinstance(value, dict):
        required = schema.get("required", [])
        if isinstance(required, list):
            for key in required:
                if key not in value:
                    failures.append(f"{path}: missing required property {key!r}")
        properties = schema.get("properties", {})
        properties = properties if isinstance(properties, Mapping) else {}
        for key, item in value.items():
            child_path = f"{path}.{key}"
            child_schema = properties.get(key)
            if isinstance(child_schema, Mapping):
                failures.extend(
                    validate_instance(item, child_schema, path=child_path, root_schema=root)
                )
                continue
            additional = schema.get("additionalProperties", True)
            if additional is False:
                failures.append(f"{child_path}: additional property is not allowed")
            elif isinstance(additional, Mapping):
                failures.extend(
                    validate_instance(item, additional, path=child_path, root_schema=root)
                )
        minimum_properties = schema.get("minProperties")
        if isinstance(minimum_properties, int) and len(value) < minimum_properties:
            failures.append(f"{path}: expected at least {minimum_properties} properties")

    if isinstance(value, list):
        minimum_items = schema.get("minItems")
        if isinstance(minimum_items, int) and len(value) < minimum_items:
            failures.append(f"{path}: expected at least {minimum_items} items")
        maximum_items = schema.get("maxItems")
        if isinstance(maximum_items, int) and len(value) > maximum_items:
            failures.append(f"{path}: expected at most {maximum_items} items")
        if schema.get("uniqueItems") is True:
            serialized = [json.dumps(item, sort_keys=True) for item in value]
            if len(serialized) != len(set(serialized)):
                failures.append(f"{path}: array items must be unique")
        item_schema = schema.get("items")
        if isinstance(item_schema, Mapping):
            for index, item in enumerate(value):
                failures.extend(
                    validate_instance(
                        item,
                        item_schema,
                        path=f"{path}[{index}]",
                        root_schema=root,
                    )
                )

    if isinstance(value, str):
        minimum_length = schema.get("minLength")
        if isinstance(minimum_length, int) and len(value) < minimum_length:
            failures.append(f"{path}: string is shorter than {minimum_length}")
        maximum_length = schema.get("maxLength")
        if isinstance(maximum_length, int) and len(value) > maximum_length:
            failures.append(f"{path}: string is longer than {maximum_length}")
        pattern = schema.get("pattern")
        if isinstance(pattern, str) and re.search(pattern, value) is None:
            failures.append(f"{path}: value does not match pattern {pattern!r}")
        declared_format = schema.get("format")
        if declared_format == "date-time" and not _is_rfc3339_datetime(value):
            failures.append(f"{path}: value is not a valid RFC 3339 date-time")
        elif declared_format == "uri" and not _is_absolute_uri(value):
            failures.append(f"{path}: value is not an absolute URI")

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        minimum = schema.get("minimum")
        maximum = schema.get("maximum")
        if isinstance(minimum, (int, float)) and value < minimum:
            failures.append(f"{path}: value is below minimum {minimum}")
        if isinstance(maximum, (int, float)) and value > maximum:
            failures.append(f"{path}: value exceeds maximum {maximum}")

    return failures


def _is_rfc3339_datetime(value: str) -> bool:
    if re.fullmatch(
        r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})",
        value,
    ) is None:
        return False
    normalized = value[:-1] + "+00:00" if value.endswith("Z") else value
    try:
        return datetime.fromisoformat(normalized).tzinfo is not None
    except ValueError:
        return False


def _is_absolute_uri(value: str) -> bool:
    if not value or any(character.isspace() for character in value):
        return False
    try:
        parsed = urlsplit(value)
    except ValueError:
        return False
    return bool(parsed.scheme) and (bool(parsed.netloc) or bool(parsed.path))


def _safe_schema_path(registry_path: Path, reference: str) -> Path:
    raw_path = reference.split("#", 1)[0]
    relative = Path(raw_path)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError(f"schema reference must stay inside the v2 contract root: {reference}")
    contract_root = registry_path.parent.resolve()
    target = (registry_path.parent / relative).resolve()
    if not target.is_relative_to(contract_root):
        raise ValueError(f"schema reference escapes the v2 contract root: {reference}")
    if not target.is_file():
        raise ValueError(f"schema reference does not exist: {reference}")
    return target


def _closed_output_envelope_failures(
    name: str,
    profile_name: str,
    schema: Mapping[str, Any],
) -> list[str]:
    """Validate the fail-closed top-level envelope promised by Contract v2."""

    label = f"{name}.{profile_name}: output schema"
    failures: list[str] = []
    if schema.get("$schema") != CONTRACT_SCHEMA_DIALECT:
        failures.append(
            f"{label} must declare the Contract v2 JSON Schema dialect"
        )
    if schema.get("type") != "object":
        failures.append(f"{label} must declare top-level type 'object'")
    if schema.get("additionalProperties") is not False:
        failures.append(f"{label} must close its top-level additional properties")

    properties = schema.get("properties")
    if not isinstance(properties, Mapping) or not properties:
        failures.append(f"{label} must declare non-empty top-level properties")
        properties = {}
    elif any(
        not isinstance(key, str) or not isinstance(value, Mapping)
        for key, value in properties.items()
    ):
        failures.append(f"{label} top-level properties must map names to schema objects")

    required = schema.get("required")
    if not isinstance(required, list) or not required:
        failures.append(f"{label} must declare non-empty top-level required properties")
    else:
        if any(not isinstance(key, str) for key in required):
            failures.append(f"{label} required properties must be strings")
        elif len(required) != len(set(required)):
            failures.append(f"{label} required properties must be unique")
        missing = sorted(key for key in required if key not in properties)
        if missing:
            failures.append(
                f"{label} requires undeclared top-level properties {missing!r}"
            )
    return failures


def _load_full_tool_definitions(root: Path) -> dict[str, dict[str, Any]]:
    source_root = (root / FULL_SOURCE_RELATIVE).resolve()
    if not source_root.is_dir():
        raise ValueError(f"missing Python Full source root: {source_root}")
    if str(source_root) not in sys.path:
        sys.path.insert(0, str(source_root))
    module = importlib.import_module("bridges.mcp_tool_handlers")
    definitions = getattr(module, "MCP_TOOL_DEFINITIONS", None)
    if not isinstance(definitions, list):
        raise ValueError("Python Full MCP_TOOL_DEFINITIONS is not a list")
    tools: dict[str, dict[str, Any]] = {}
    for item in definitions:
        if not isinstance(item, dict) or not isinstance(item.get("name"), str):
            raise ValueError("Python Full contains an invalid MCP tool definition")
        name = item["name"]
        if name in tools:
            raise ValueError(f"Python Full contains duplicate MCP tool {name!r}")
        tools[name] = item
    return tools


def _tool_map(document: Any, *, label: str) -> dict[str, Mapping[str, Any]]:
    values = document.get("tools") if isinstance(document, Mapping) else None
    if not isinstance(values, list):
        raise ValueError(f"{label} tools must be a list")
    result: dict[str, Mapping[str, Any]] = {}
    for item in values:
        if not isinstance(item, Mapping) or not isinstance(item.get("name"), str):
            raise ValueError(f"{label} contains an invalid tool definition")
        name = item["name"]
        if name in result:
            raise ValueError(f"{label} contains duplicate tool {name!r}")
        result[name] = item
    return result


def runtime_schema_projection(schema: Mapping[str, Any]) -> dict[str, Any]:
    return {
        key: value
        for key, value in schema.items()
        if key not in {"$schema", "$id", "title"}
    }


def _derive_runtime_coverage(
    full_tools: Mapping[str, Mapping[str, Any]],
    lite_tools: Mapping[str, Mapping[str, Any]],
) -> tuple[int, int, list[str]]:
    """Derive canonical/public totals from the union of shipped runtime names."""
    public_names = set(full_tools) | set(lite_tools)
    alias_targets: dict[str, dict[str, str | None]] = {}
    failures: list[str] = []

    for profile_name, runtime_tools in (
        ("full", full_tools),
        ("marketplace-lite", lite_tools),
    ):
        for name, definition in runtime_tools.items():
            description = definition.get("description") if isinstance(definition, Mapping) else None
            match = (
                COMPATIBILITY_ALIAS_PATTERN.fullmatch(description)
                if isinstance(description, str)
                else None
            )
            alias_targets.setdefault(name, {})[profile_name] = (
                match.group("canonical") if match is not None else None
            )

    aliases: set[str] = set()
    for name, declarations in alias_targets.items():
        declared_targets = {target for target in declarations.values() if target is not None}
        if not declared_targets:
            continue
        aliases.add(name)
        if len(declared_targets) != 1:
            failures.append(
                f"{name}: runtime compatibility alias targets disagree across Full and Lite"
            )
            continue
        canonical = next(iter(declared_targets))
        non_alias_profiles = sorted(
            profile for profile, target in declarations.items() if target is None
        )
        if non_alias_profiles:
            failures.append(
                f"{name}: compatibility alias classification disagrees in "
                f"{', '.join(non_alias_profiles)}"
            )
        if canonical == name:
            failures.append(f"{name}: runtime compatibility alias cannot target itself")
        elif canonical not in public_names:
            failures.append(
                f"{name}: runtime compatibility alias targets missing canonical tool "
                f"{canonical!r}"
            )

    return len(public_names - aliases), len(public_names), failures


def _name_set_failure(label: str, actual: set[str], expected: set[str]) -> str:
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    return f"{label} differs from frozen CTR-201 inventory (missing={missing!r}, extra={extra!r})"


def _load_frozen_name_oracle(
    root: Path,
) -> tuple[dict[str, tuple[str, ...]], list[str]]:
    del root
    fallback = {
        "canonical": EXPECTED_TARGET_CANONICAL_NAMES,
        "public": EXPECTED_TARGET_PUBLIC_NAMES,
        "full": EXPECTED_FULL_PUBLIC_NAMES,
        "marketplace-lite": EXPECTED_LITE_PUBLIC_NAMES,
    }
    # CTR-201 is immutable historical evidence. The current target promotes the
    # two Zotero names it recorded as legacy-only, so the live oracle stays here.
    return fallback, []


def validate_capability_contract(
    root: Path,
    *,
    full_tool_definitions: Mapping[str, Mapping[str, Any]] | None = None,
    lite_tool_definitions: Mapping[str, Mapping[str, Any]] | None = None,
    mcpb_tool_definitions: Mapping[str, Mapping[str, Any]] | None = None,
    require_complete: bool = False,
) -> list[str]:
    root = root.resolve()
    registry_path = root / REGISTRY_RELATIVE
    failures: list[str] = []
    try:
        registry = _load_json(registry_path)
        registry_schema_ref = registry.get("$schema") if isinstance(registry, dict) else None
        if not isinstance(registry_schema_ref, str):
            raise ValueError("registry must declare a relative $schema")
        registry_schema_path = _safe_schema_path(registry_path, registry_schema_ref)
        registry_schema = _load_json(registry_schema_path)
    except ValueError as exc:
        return [str(exc)]

    if not isinstance(registry, dict) or not isinstance(registry_schema, dict):
        return ["registry and registry schema must be JSON objects"]
    failures.extend(validate_instance(registry, registry_schema))

    oracle, oracle_failures = _load_frozen_name_oracle(root)
    failures.extend(oracle_failures)
    expected_canonical = set(oracle["canonical"])
    expected_public = set(oracle["public"])
    expected_runtime_public = {
        "marketplace-lite": set(oracle["marketplace-lite"]),
        "full": set(oracle["full"]),
    }

    tools = registry.get("tools", [])
    tools = tools if isinstance(tools, list) else []
    canonical_names: list[str] = []
    public_names: list[str] = []
    smoke_ids: list[str] = []
    tool_aliases: dict[str, tuple[str, ...]] = {}
    exposed_public_names: dict[str, set[str]] = {
        profile: set() for profile in RUNTIME_PROFILES
    }
    taxonomy = registry.get("error_taxonomy", {})
    taxonomy = taxonomy if isinstance(taxonomy, Mapping) else {}
    side_effect_classes = registry.get("side_effect_classes", [])
    side_effect_classes = (
        set(side_effect_classes) if isinstance(side_effect_classes, list) else set()
    )

    schema_claims: dict[tuple[str, str, str], tuple[str, Mapping[str, Any]]] = {}
    schema_usage: dict[tuple[str, str], dict[str, set[str]]] = {}
    schema_ids: dict[str, str] = {}
    for tool in tools:
        if not isinstance(tool, dict):
            continue
        name = tool.get("name")
        if not isinstance(name, str):
            continue
        canonical_names.append(name)
        public_names.append(name)
        aliases_value = tool.get("aliases", [])
        aliases = tuple(
            alias["name"]
            for alias in aliases_value
            if isinstance(alias, Mapping) and isinstance(alias.get("name"), str)
        ) if isinstance(aliases_value, list) else ()
        tool_aliases[name] = aliases
        public_names.extend(aliases)
        for error_code in tool.get("errors", []):
            if error_code not in taxonomy:
                failures.append(f"{name}: unknown error taxonomy code {error_code!r}")
        effect_names: set[str] = set()
        for effect in tool.get("side_effects", []):
            if not isinstance(effect, dict):
                continue
            effect_name = effect.get("class")
            if effect_name not in side_effect_classes:
                failures.append(f"{name}: unknown side-effect class {effect_name!r}")
            if isinstance(effect_name, str):
                effect_names.add(effect_name)
            if effect.get("mode") in {"conditional", "deferred"} and not effect.get("trigger"):
                failures.append(
                    f"{name}: {effect.get('mode')} side effect {effect_name!r} requires a trigger"
                )
        lifecycle = tool.get("lifecycle", {})
        if isinstance(lifecycle, Mapping):
            if lifecycle.get("remove_after") is not None and lifecycle.get("deprecated_in") is None:
                failures.append(f"{name}: remove_after requires deprecated_in")
        if isinstance(aliases_value, list):
            for alias in aliases_value:
                if not isinstance(alias, Mapping):
                    continue
                if alias.get("remove_after") is not None and alias.get("deprecated_in") is None:
                    failures.append(f"{name}: alias remove_after requires deprecated_in")
        profiles = tool.get("profiles", {})
        profiles = profiles if isinstance(profiles, Mapping) else {}
        security = tool.get("security", {})
        security = security if isinstance(security, Mapping) else {}
        sensitive_output_paths = security.get("sensitive_output_paths", [])
        sensitive_output_paths = (
            set(sensitive_output_paths) if isinstance(sensitive_output_paths, list) else set()
        )
        profile_sensitive_output_paths = security.get(
            "profile_sensitive_output_paths", {}
        )
        profile_sensitive_output_paths = (
            profile_sensitive_output_paths
            if isinstance(profile_sensitive_output_paths, Mapping)
            else {}
        )
        exposed_output_refs = {
            profile_name: profile.get("output_schema_ref")
            for profile_name, profile in profiles.items()
            if isinstance(profile, Mapping) and profile.get("exposure") == "tool"
        }
        if len(set(exposed_output_refs.values())) > 1:
            missing_profile_paths = set(exposed_output_refs) - set(
                profile_sensitive_output_paths
            )
            if missing_profile_paths:
                failures.append(
                    f"{name}: divergent output schemas require profile-sensitive "
                    f"paths for {sorted(missing_profile_paths)!r}"
                )
        unexpected_profile_paths = set(profile_sensitive_output_paths) - set(
            exposed_output_refs
        )
        if unexpected_profile_paths:
            failures.append(
                f"{name}: profile-sensitive output paths target unavailable profiles "
                f"{sorted(unexpected_profile_paths)!r}"
            )
        if set(profiles) != set(EXPECTED_PROFILES):
            failures.append(f"{name}: profiles must explicitly declare {EXPECTED_PROFILES!r}")
        lite_profile = profiles.get("marketplace-lite", {})
        if (
            isinstance(lite_profile, Mapping)
            and lite_profile.get("exposure") == "tool"
            and effect_names & FORBIDDEN_LITE_SIDE_EFFECTS
        ):
            failures.append(
                f"{name}: Marketplace Lite declares forbidden side effects "
                f"{sorted(effect_names & FORBIDDEN_LITE_SIDE_EFFECTS)!r}"
            )

        for profile_name in EXPECTED_PROFILES:
            profile = profiles.get(profile_name, {})
            if not isinstance(profile, Mapping):
                continue
            exposure = profile.get("exposure")
            if profile_name == "skill-only":
                if exposure != "metadata-only":
                    failures.append(f"{name}.skill-only: exposure must be metadata-only")
            else:
                expected_exposure = (
                    "tool" if name in expected_runtime_public[profile_name] else "unavailable"
                )
                if exposure != expected_exposure:
                    failures.append(
                        f"{name}.{profile_name}: exposure must be {expected_exposure!r} "
                        "according to frozen CTR-201 inventory"
                    )
                alias_presence = {
                    alias in expected_runtime_public[profile_name] for alias in aliases
                }
                if alias_presence and alias_presence != {expected_exposure == "tool"}:
                    failures.append(
                        f"{name}.{profile_name}: alias availability disagrees with the "
                        "frozen canonical profile"
                    )

            if exposure != "tool":
                if "input_schema_ref" in profile or "output_schema_ref" in profile:
                    failures.append(
                        f"{name}.{profile_name}: non-tool exposure must not declare schema refs"
                    )
                if profile.get("error_transport") != "not-applicable":
                    failures.append(
                        f"{name}.{profile_name}: non-tool exposure must use "
                        "not-applicable error transport"
                    )
                continue

            if profile.get("error_transport") == "not-applicable":
                failures.append(
                    f"{name}.{profile_name}: tool exposure requires an applicable error transport"
                )
            if profile_name in exposed_public_names:
                exposed_public_names[profile_name].update((name, *aliases))
            for field in ("input_schema_ref", "output_schema_ref"):
                reference = profile.get(field)
                if not isinstance(reference, str):
                    failures.append(f"{name}.{profile_name}: missing {field}")
                    continue
                try:
                    schema_path = _safe_schema_path(registry_path, reference)
                    schema_document = _load_json(schema_path)
                except ValueError as exc:
                    failures.append(f"{name}.{profile_name}: {exc}")
                    continue
                if not isinstance(schema_document, Mapping):
                    failures.append(f"{name}.{profile_name}: {field} must resolve to an object")
                    continue
                schema_kind = "input" if field == "input_schema_ref" else "output"
                schema_claims[(name, profile_name, schema_kind)] = (
                    reference,
                    schema_document,
                )
                schema_usage.setdefault((name, schema_kind), {}).setdefault(
                    reference, set()
                ).add(profile_name)
                if field == "output_schema_ref":
                    effective_sensitive_paths = profile_sensitive_output_paths.get(
                        profile_name, sensitive_output_paths
                    )
                    effective_sensitive_paths = (
                        set(effective_sensitive_paths)
                        if isinstance(effective_sensitive_paths, (list, set, tuple))
                        else set()
                    )
                    failures.extend(
                        _closed_output_envelope_failures(
                            name,
                            profile_name,
                            schema_document,
                        )
                    )
                    output_properties = schema_document.get("properties", {})
                    output_properties = (
                        output_properties if isinstance(output_properties, Mapping) else {}
                    )
                    for pointer in effective_sensitive_paths:
                        if not isinstance(pointer, str) or not pointer.startswith("/"):
                            failures.append(
                                f"{name}.{profile_name}: sensitive output path "
                                f"{pointer!r} is not a JSON pointer"
                            )
                            continue
                        top_level = pointer[1:].split("/", 1)[0].replace("~1", "/").replace(
                            "~0", "~"
                        )
                        if top_level not in output_properties:
                            failures.append(
                                f"{name}.{profile_name}: sensitive output path {pointer!r} "
                                "does not resolve "
                                "to a declared top-level output property"
                            )
                    if (
                        "config_path" in output_properties
                        and "/config_path" not in effective_sensitive_paths
                    ):
                        failures.append(
                            f"{name}.{profile_name}: config_path output must be declared as sensitive"
                        )
                    if (
                        "loopback-listener" in effect_names
                        and "url" in output_properties
                        and "/url" not in effective_sensitive_paths
                    ):
                        failures.append(
                            f"{name}.{profile_name}: loopback URL output must be declared as sensitive"
                        )
        ids = tool.get("smoke_call_ids", [])
        if isinstance(ids, list):
            smoke_ids.extend(item for item in ids if isinstance(item, str))

    for (name, profile_name, schema_kind), (reference, schema_document) in schema_claims.items():
        references = schema_usage.get((name, schema_kind), {})
        exposed_profiles = {
            profile
            for profile in RUNTIME_PROFILES
            if (name, profile, schema_kind) in schema_claims
        }
        shared_reference = (
            len(exposed_profiles) > 1
            and len(references) == 1
            and set(next(iter(references.values()))) == exposed_profiles
        )
        expected_stem = (
            f"{name}.{schema_kind}"
            if shared_reference
            else f"{name}.{profile_name}.{schema_kind}"
        )
        expected_schema_id = f"https://qiongli.dev/schemas/tools/{expected_stem}.v2.json"
        schema_id = schema_document.get("$id")
        if schema_id != expected_schema_id:
            failures.append(
                f"{name}.{profile_name}: {schema_kind}_schema_ref must declare $id "
                f"{expected_schema_id!r}"
            )
        elif isinstance(schema_id, str):
            prior_reference = schema_ids.get(schema_id)
            if prior_reference is not None and prior_reference != reference:
                failures.append(
                    f"{name}: schema $id {schema_id!r} is reused by "
                    f"{prior_reference!r} and {reference!r}"
                )
            schema_ids[schema_id] = reference

    if len(canonical_names) != len(set(canonical_names)):
        failures.append("registry canonical tool names must be unique")
    if len(public_names) != len(set(public_names)):
        failures.append("registry aliases must not collide with canonical or alias names")
    if len(smoke_ids) != len(set(smoke_ids)):
        failures.append("registry smoke_call_ids must be unique")

    coverage = registry.get("coverage", {})
    coverage_mode = coverage.get("mode") if isinstance(coverage, Mapping) else None
    registry_status = registry.get("status")
    complete_mode = coverage_mode == "complete"
    if complete_mode and registry_status != "preview":
        failures.append("coverage mode complete requires registry status preview")
    if registry_status == "preview" and not complete_mode:
        failures.append("registry status preview requires coverage mode complete")
    if require_complete and not complete_mode:
        failures.append("Capability Contract v2 complete coverage is required")

    canonical_set = set(canonical_names)
    public_set = set(public_names)
    if complete_mode:
        if tuple(canonical_names) != oracle["canonical"]:
            failures.append("registry canonical tool order differs from frozen CTR-201 inventory")
        if canonical_set != expected_canonical:
            failures.append(
                _name_set_failure("registry canonical names", canonical_set, expected_canonical)
            )
        if public_set != expected_public:
            failures.append(_name_set_failure("registry public names", public_set, expected_public))
    else:
        if not canonical_set.issubset(expected_canonical):
            failures.append("pilot registry contains canonical names outside frozen CTR-201 target")
        if not public_set.issubset(expected_public):
            failures.append("pilot registry contains public names outside frozen CTR-201 target")

    registry_aliases = {
        alias: canonical
        for canonical, aliases in tool_aliases.items()
        for alias in aliases
    }
    expected_registry_aliases = {
        alias: canonical
        for alias, canonical in EXPECTED_ALIASES.items()
        if alias in public_set or complete_mode
    }
    if registry_aliases != expected_registry_aliases:
        failures.append(
            "registry compatibility alias mapping differs from frozen CTR-201 inventory"
        )

    if isinstance(coverage, Mapping):
        if coverage.get("canonical_tool_count") != len(canonical_names):
            failures.append("coverage.canonical_tool_count does not match registry tools")
        if coverage.get("public_name_count") != len(public_names):
            failures.append(
                "coverage.public_name_count does not match canonical names plus aliases"
            )
        if coverage.get("target_canonical_tool_count") != len(expected_canonical):
            failures.append(
                "coverage.target_canonical_tool_count does not match frozen "
                f"CTR-201 total ({len(expected_canonical)})"
            )
        if coverage.get("target_public_name_count") != len(expected_public):
            failures.append(
                "coverage.target_public_name_count does not match frozen "
                f"CTR-201 total ({len(expected_public)})"
            )
        if complete_mode:
            if coverage.get("target_canonical_tool_count") != len(canonical_names):
                failures.append("complete registry must reach target_canonical_tool_count")
            if coverage.get("target_public_name_count") != len(public_names):
                failures.append("complete registry must reach target_public_name_count")

    try:
        lite_contract = _load_json(root / LITE_TOOLS_RELATIVE)
        if lite_tool_definitions is None:
            lite_tools = _tool_map(lite_contract, label="Marketplace Lite contract")
        else:
            lite_tools = dict(lite_tool_definitions)
        if full_tool_definitions is None:
            full_tools = _load_full_tool_definitions(root)
            for name, definition in lite_tools.items():
                if name in expected_runtime_public["full"] and name not in full_tools:
                    full_tools[name] = definition
        else:
            full_tools = dict(full_tool_definitions)
        complete_smoke_path = root / COMPLETE_SMOKE_CALLS_RELATIVE
        if complete_smoke_path.is_file():
            smoke_fixture = _load_json(complete_smoke_path)
        elif complete_mode:
            raise ValueError(
                f"missing complete Contract v2 smoke fixture: {complete_smoke_path}"
            )
        else:
            smoke_fixture = _load_json(root / LEGACY_SMOKE_CALLS_RELATIVE)
        if mcpb_tool_definitions is None:
            mcpb_manifest = _load_json(root / MCPB_MANIFEST_RELATIVE)
            mcpb_tools = _tool_map(mcpb_manifest, label="MCPB manifest")
        else:
            mcpb_tools = dict(mcpb_tool_definitions)
    except (AttributeError, ValueError) as exc:
        failures.append(str(exc))
        return failures

    for profile_name, runtime_tools in (
        ("marketplace-lite", lite_tools),
        ("full", full_tools),
    ):
        actual_names = set(runtime_tools)
        expected_names = expected_runtime_public[profile_name]
        if tuple(runtime_tools) != oracle[profile_name]:
            failures.append(
                f"{profile_name} runtime declaration order differs from frozen CTR-201 inventory"
            )
        if actual_names != expected_names:
            failures.append(
                _name_set_failure(
                    f"{profile_name} runtime public names", actual_names, expected_names
                )
            )

    target_canonical_count, target_public_count, coverage_failures = (
        _derive_runtime_coverage(full_tools, lite_tools)
    )
    failures.extend(coverage_failures)
    if (target_canonical_count, target_public_count) != (
        len(expected_canonical),
        len(expected_public),
    ):
        failures.append(
            "live Full/Lite runtime union does not preserve the frozen CTR-201 "
            "canonical/public totals"
        )
    ordered_runtime_union = tuple(
        dict.fromkeys((*full_tools.keys(), *lite_tools.keys()))
    )
    if ordered_runtime_union != oracle["public"]:
        failures.append(
            "ordered Full-to-Lite runtime union differs from frozen CTR-201 target public names"
        )

    lite_names = set(lite_tools)
    if tuple(mcpb_tools) != EXPECTED_FROZEN_MCPB_PUBLIC_NAMES:
        failures.append("MCPB manifest public names differ from the frozen MCPB surface")

    for profile_name in RUNTIME_PROFILES:
        actual_exposed = exposed_public_names[profile_name]
        expected_names = set(
            name
            for name in expected_runtime_public[profile_name]
            if name in public_set
        )
        if complete_mode:
            expected_names = expected_runtime_public[profile_name]
        if actual_exposed != expected_names:
            failures.append(
                _name_set_failure(
                    f"registry {profile_name} tool exposure",
                    actual_exposed,
                    expected_names,
                )
            )

    for tool in tools:
        if not isinstance(tool, dict) or not isinstance(tool.get("name"), str):
            continue
        name = tool["name"]
        profiles = tool.get("profiles", {})
        aliases = [
            alias["name"]
            for alias in tool.get("aliases", [])
            if isinstance(alias, Mapping) and isinstance(alias.get("name"), str)
        ]
        for profile_name, runtime_tools in (
            ("marketplace-lite", lite_tools),
            ("full", full_tools),
        ):
            profile = profiles.get(profile_name, {}) if isinstance(profiles, Mapping) else {}
            if not isinstance(profile, Mapping) or profile.get("exposure") != "tool":
                continue
            runtime_definition = runtime_tools.get(name)
            if not isinstance(runtime_definition, Mapping):
                failures.append(f"{name}: missing from {profile_name} runtime definitions")
                continue
            if runtime_definition.get("description") != tool.get("description"):
                failures.append(f"{name}: {profile_name} description drifts from v2 registry")
            schema_claim = schema_claims.get((name, profile_name, "input"))
            canonical_input = schema_claim[1] if schema_claim is not None else None
            if canonical_input is not None and runtime_definition.get(
                "inputSchema"
            ) != runtime_schema_projection(canonical_input):
                failures.append(f"{name}: {profile_name} input schema drifts from v2 registry")
            for alias_name in aliases:
                alias_definition = runtime_tools.get(alias_name)
                if not isinstance(alias_definition, Mapping):
                    failures.append(
                        f"{name}: alias {alias_name} is missing from "
                        f"{profile_name} runtime definitions"
                    )
                    continue
                expected_description = f"Compatibility alias for {name}."
                if alias_definition.get("description") != expected_description:
                    failures.append(
                        f"{alias_name}: {profile_name} alias description drifts from v2 registry"
                    )
                if canonical_input is not None and alias_definition.get(
                    "inputSchema"
                ) != runtime_schema_projection(canonical_input):
                    failures.append(
                        f"{alias_name}: {profile_name} alias input schema drifts from {name}"
                    )

    smoke_calls = smoke_fixture.get("calls") if isinstance(smoke_fixture, Mapping) else None
    if not isinstance(smoke_calls, list):
        failures.append("Contract v2 smoke fixture calls must be a list")
        smoke_calls = []
    smoke_by_id: dict[str, Mapping[str, Any]] = {}
    for smoke in smoke_calls:
        if not isinstance(smoke, Mapping) or not isinstance(smoke.get("id"), str):
            failures.append("Contract v2 smoke fixture contains a case without an id")
            continue
        smoke_id = smoke["id"]
        if smoke_id in smoke_by_id:
            failures.append(f"Contract v2 smoke fixture duplicates id {smoke_id!r}")
            continue
        if complete_mode and smoke.get("profile") not in RUNTIME_PROFILES:
            failures.append(
                f"Contract v2 complete smoke fixture {smoke_id!r} must declare a runtime profile"
            )
        if smoke.get("expected_response_class") not in EXPECTED_SMOKE_RESPONSE_CLASSES:
            failures.append(
                f"Contract v2 smoke fixture {smoke_id!r} has an unsupported response class"
            )
        smoke_by_id[smoke_id] = smoke

    referenced_smoke_ids = set(smoke_ids)
    fixture_smoke_ids = set(smoke_by_id)
    if complete_mode and referenced_smoke_ids != fixture_smoke_ids:
        failures.append(
            _name_set_failure(
                "complete Contract v2 registry/fixture smoke IDs",
                referenced_smoke_ids,
                fixture_smoke_ids,
            )
        )

    all_observed_pairs: set[tuple[str, str]] = set()
    for tool in tools:
        if not isinstance(tool, Mapping) or not isinstance(tool.get("name"), str):
            continue
        name = tool["name"]
        aliases = tool_aliases.get(name, ())
        profiles = tool.get("profiles", {})
        profiles = profiles if isinstance(profiles, Mapping) else {}
        observed_pairs: set[tuple[str, str]] = set()
        observed_names: set[str] = set()
        for smoke_id in tool.get("smoke_call_ids", []):
            smoke = smoke_by_id.get(smoke_id)
            if smoke is None:
                failures.append(f"{name}: missing smoke fixture {smoke_id!r}")
                continue
            smoke_name = smoke.get("name")
            if smoke_name not in {name, *aliases}:
                failures.append(
                    f"{name}: smoke fixture {smoke_id!r} targets unrelated tool "
                    f"{smoke_name!r}"
                )
                continue
            observed_names.add(str(smoke_name))
            smoke_profile = smoke.get("profile")
            if complete_mode:
                if smoke_profile not in RUNTIME_PROFILES:
                    continue
                profile = profiles.get(smoke_profile, {})
                if not isinstance(profile, Mapping) or profile.get("exposure") != "tool":
                    failures.append(
                        f"{name}: smoke fixture {smoke_id!r} targets an unavailable profile"
                    )
                    continue
                observed_pairs.add((str(smoke_name), str(smoke_profile)))
                all_observed_pairs.add((str(smoke_name), str(smoke_profile)))
                schema_claim = schema_claims.get((name, str(smoke_profile), "input"))
            else:
                schema_claim = next(
                    (
                        schema_claims[(name, profile_name, "input")]
                        for profile_name in RUNTIME_PROFILES
                        if (name, profile_name, "input") in schema_claims
                    ),
                    None,
                )
            if schema_claim is not None:
                canonical_input = schema_claim[1]
                arguments = smoke.get("arguments")
                input_failures = validate_instance(arguments, canonical_input)
                expects_input_error = smoke.get("expected_response_class") == "input_error"
                if expects_input_error and not input_failures:
                    failures.append(
                        f"{name}: input-error smoke fixture {smoke_id!r} is schema-valid"
                    )
                elif not expects_input_error and input_failures:
                    failures.append(
                        f"{name}: smoke fixture {smoke_id!r} violates its input schema: "
                        + "; ".join(input_failures)
                    )

        if complete_mode:
            required_pairs = {
                (public_name, profile_name)
                for public_name in (name, *aliases)
                for profile_name in RUNTIME_PROFILES
                if isinstance(profiles.get(profile_name), Mapping)
                and profiles[profile_name].get("exposure") == "tool"
            }
            for public_name, profile_name in sorted(required_pairs - observed_pairs):
                failures.append(
                    f"{name}: no referenced smoke fixture covers {public_name} "
                    f"in {profile_name}"
                )
        else:
            for public_name in (name, *aliases):
                if public_name not in observed_names:
                    failures.append(f"{name}: no referenced smoke fixture covers {public_name}")

        lite_profile = profiles.get("marketplace-lite", {})
        if isinstance(lite_profile, Mapping) and lite_profile.get("exposure") == "tool":
            for public_name in (name, *aliases):
                manifest_definition = mcpb_tools.get(public_name)
                if not isinstance(manifest_definition, Mapping):
                    continue
                expected_description = (
                    tool.get("description")
                    if public_name == name
                    else f"Compatibility alias for {name}."
                )
                if manifest_definition.get("description") != expected_description:
                    failures.append(
                        f"{public_name}: MCPB manifest description drifts from v2 registry"
                    )

    if complete_mode:
        actual_input_error_pairs = {
            (str(smoke.get("profile")), str(smoke.get("name")))
            for smoke in smoke_by_id.values()
            if smoke.get("expected_response_class") == "input_error"
        }
        if actual_input_error_pairs != EXPECTED_INPUT_ERROR_SMOKE_PAIRS:
            missing = sorted(EXPECTED_INPUT_ERROR_SMOKE_PAIRS - actual_input_error_pairs)
            extra = sorted(actual_input_error_pairs - EXPECTED_INPUT_ERROR_SMOKE_PAIRS)
            failures.append(
                "complete Contract v2 input-error smoke boundary differs from the "
                f"approved safe-call disposition (missing={missing!r}, extra={extra!r})"
            )
        expected_smoke_pairs = {
            (public_name, profile_name)
            for profile_name in RUNTIME_PROFILES
            for public_name in expected_runtime_public[profile_name]
        }
        if len(expected_smoke_pairs) != 40:
            failures.append("frozen CTR-201 smoke-pair oracle must contain exactly 40 pairs")
        if all_observed_pairs != expected_smoke_pairs:
            missing = sorted(expected_smoke_pairs - all_observed_pairs)
            extra = sorted(all_observed_pairs - expected_smoke_pairs)
            failures.append(
                "complete Contract v2 smoke profile/public coverage differs from frozen "
                f"CTR-201 inventory (missing={missing!r}, extra={extra!r})"
            )
        if len(smoke_by_id) != len(expected_smoke_pairs):
            failures.append("complete Contract v2 smoke fixture must contain exactly 40 cases")

    return failures


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate Qiongli Capability Contract v2.")
    parser.add_argument(
        "--root",
        type=Path,
        default=REPO_ROOT,
        help="Repository or materialized distribution root.",
    )
    parser.add_argument("--json", action="store_true", help="Emit a machine-readable report.")
    parser.add_argument(
        "--require-complete",
        action="store_true",
        help="Fail unless Contract v2 has preview status and complete frozen-target coverage.",
    )
    args = parser.parse_args(argv)

    failures = validate_capability_contract(
        args.root,
        require_complete=args.require_complete,
    )
    if args.json:
        print(
            json.dumps(
                {
                    "status": "failed" if failures else "passed",
                    "contract": str(REGISTRY_RELATIVE),
                    "failures": failures,
                },
                indent=2,
                sort_keys=True,
            )
        )
    elif failures:
        for failure in failures:
            print(f"[FAIL] {failure}")
    else:
        print("[OK] Capability Contract v2 is valid")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
