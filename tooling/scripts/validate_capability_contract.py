#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib
import json
import re
import sys
from pathlib import Path
from typing import Any, Mapping


REPO_ROOT = Path(__file__).resolve().parents[2]
REGISTRY_RELATIVE = Path("content/mcp-contracts/v2/registry.json")
LITE_TOOLS_RELATIVE = Path("content/mcp-contracts/lite-tools.json")
SMOKE_CALLS_RELATIVE = Path("content/mcp-contracts/fixtures/lite-tool-smoke-calls.json")
FULL_SOURCE_RELATIVE = Path("packages/python-qiongli/src/qiongli")
EXPECTED_PROFILES = ("skill-only", "marketplace-lite", "full")
FORBIDDEN_LITE_SIDE_EFFECTS = {"project-write", "process-launch", "agent-launch"}


def _load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ValueError(f"missing JSON file: {path}") from exc
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid JSON in {path}: {exc}") from exc


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
        pattern = schema.get("pattern")
        if isinstance(pattern, str) and re.fullmatch(pattern, value) is None:
            failures.append(f"{path}: value does not match pattern {pattern!r}")

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        minimum = schema.get("minimum")
        maximum = schema.get("maximum")
        if isinstance(minimum, (int, float)) and value < minimum:
            failures.append(f"{path}: value is below minimum {minimum}")
        if isinstance(maximum, (int, float)) and value > maximum:
            failures.append(f"{path}: value exceeds maximum {maximum}")

    return failures


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
    return {
        str(item["name"]): item
        for item in definitions
        if isinstance(item, dict) and isinstance(item.get("name"), str)
    }


def runtime_schema_projection(schema: Mapping[str, Any]) -> dict[str, Any]:
    return {
        key: value
        for key, value in schema.items()
        if key not in {"$schema", "$id", "title"}
    }


def validate_capability_contract(
    root: Path,
    *,
    full_tool_definitions: Mapping[str, Mapping[str, Any]] | None = None,
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

    tools = registry.get("tools", [])
    tools = tools if isinstance(tools, list) else []
    canonical_names: list[str] = []
    public_names: list[str] = []
    smoke_ids: list[str] = []
    taxonomy = registry.get("error_taxonomy", {})
    taxonomy = taxonomy if isinstance(taxonomy, Mapping) else {}
    side_effect_classes = registry.get("side_effect_classes", [])
    side_effect_classes = set(side_effect_classes) if isinstance(side_effect_classes, list) else set()

    referenced_schemas: dict[str, Mapping[str, Any]] = {}
    for tool in tools:
        if not isinstance(tool, dict):
            continue
        name = tool.get("name")
        if not isinstance(name, str):
            continue
        canonical_names.append(name)
        public_names.append(name)
        aliases = tool.get("aliases", [])
        if isinstance(aliases, list):
            for alias in aliases:
                if isinstance(alias, dict) and isinstance(alias.get("name"), str):
                    public_names.append(alias["name"])
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
        profiles = tool.get("profiles", {})
        profiles = profiles if isinstance(profiles, Mapping) else {}
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
        for profile_name, profile in profiles.items():
            if not isinstance(profile, Mapping) or profile.get("exposure") != "tool":
                continue
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
                referenced_schemas[reference] = schema_document
        ids = tool.get("smoke_call_ids", [])
        if isinstance(ids, list):
            smoke_ids.extend(item for item in ids if isinstance(item, str))

    if len(canonical_names) != len(set(canonical_names)):
        failures.append("registry canonical tool names must be unique")
    if len(public_names) != len(set(public_names)):
        failures.append("registry aliases must not collide with canonical or alias names")
    if len(smoke_ids) != len(set(smoke_ids)):
        failures.append("registry smoke_call_ids must be unique")

    coverage = registry.get("coverage", {})
    if isinstance(coverage, Mapping):
        if coverage.get("canonical_tool_count") != len(canonical_names):
            failures.append("coverage.canonical_tool_count does not match registry tools")
        if coverage.get("public_name_count") != len(public_names):
            failures.append("coverage.public_name_count does not match canonical names plus aliases")
        if coverage.get("mode") == "complete":
            if coverage.get("target_canonical_tool_count") != len(canonical_names):
                failures.append("complete registry must reach target_canonical_tool_count")
            if coverage.get("target_public_name_count") != len(public_names):
                failures.append("complete registry must reach target_public_name_count")

    try:
        lite_contract = _load_json(root / LITE_TOOLS_RELATIVE)
        lite_tools = {
            str(item["name"]): item
            for item in lite_contract.get("tools", [])
            if isinstance(item, dict) and isinstance(item.get("name"), str)
        }
        full_tools = dict(full_tool_definitions or _load_full_tool_definitions(root))
        smoke_fixture = _load_json(root / SMOKE_CALLS_RELATIVE)
        smoke_by_id = {
            str(item["id"]): item
            for item in smoke_fixture.get("calls", [])
            if isinstance(item, dict) and isinstance(item.get("id"), str)
        }
    except (AttributeError, ValueError) as exc:
        failures.append(str(exc))
        return failures

    for tool in tools:
        if not isinstance(tool, dict) or not isinstance(tool.get("name"), str):
            continue
        name = tool["name"]
        profiles = tool.get("profiles", {})
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
            input_ref = profile.get("input_schema_ref")
            canonical_input = referenced_schemas.get(str(input_ref))
            if canonical_input is not None and runtime_definition.get(
                "inputSchema"
            ) != runtime_schema_projection(canonical_input):
                failures.append(f"{name}: {profile_name} input schema drifts from v2 registry")
        for smoke_id in tool.get("smoke_call_ids", []):
            if smoke_id not in smoke_by_id:
                failures.append(f"{name}: missing smoke fixture {smoke_id!r}")

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
    args = parser.parse_args(argv)

    failures = validate_capability_contract(args.root)
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
        print("[OK] Capability Contract v2 pilot is valid")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
