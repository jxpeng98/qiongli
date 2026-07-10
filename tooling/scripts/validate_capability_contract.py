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
MCPB_MANIFEST_RELATIVE = Path("packages/qiongli-literature-mcpb/manifest.json")
FULL_SOURCE_RELATIVE = Path("packages/python-qiongli/src/qiongli")
EXPECTED_PROFILES = ("skill-only", "marketplace-lite", "full")
FORBIDDEN_LITE_SIDE_EFFECTS = {"project-write", "process-launch", "agent-launch"}
COMPATIBILITY_ALIAS_PATTERN = re.compile(
    r"^Compatibility alias for (?P<canonical>[A-Za-z0-9_]+)\.$"
)


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


def validate_capability_contract(
    root: Path,
    *,
    full_tool_definitions: Mapping[str, Mapping[str, Any]] | None = None,
    lite_tool_definitions: Mapping[str, Mapping[str, Any]] | None = None,
    mcpb_tool_definitions: Mapping[str, Mapping[str, Any]] | None = None,
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
    schema_ids: dict[str, str] = {}
    canonical_inputs: dict[str, Mapping[str, Any]] = {}
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
            if effect.get("mode") in {"conditional", "deferred"} and not effect.get("trigger"):
                failures.append(
                    f"{name}: {effect.get('mode')} side effect {effect_name!r} requires a trigger"
                )
        lifecycle = tool.get("lifecycle", {})
        if isinstance(lifecycle, Mapping):
            if lifecycle.get("remove_after") is not None and lifecycle.get("deprecated_in") is None:
                failures.append(f"{name}: remove_after requires deprecated_in")
        aliases = tool.get("aliases", [])
        if isinstance(aliases, list):
            for alias in aliases:
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
                schema_id = schema_document.get("$id")
                schema_kind = "input" if field == "input_schema_ref" else "output"
                expected_schema_id = (
                    f"https://qiongli.dev/schemas/tools/{name}.{schema_kind}.v2.json"
                )
                if schema_id != expected_schema_id:
                    failures.append(
                        f"{name}.{profile_name}: {field} must declare $id "
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
                if field == "input_schema_ref":
                    canonical_inputs[name] = schema_document
                    if schema_document.get("additionalProperties") is not False:
                        failures.append(
                            f"{name}: migrated input schema must reject additional properties"
                        )
                if field == "output_schema_ref":
                    output_properties = schema_document.get("properties", {})
                    output_properties = (
                        output_properties if isinstance(output_properties, Mapping) else {}
                    )
                    for pointer in sensitive_output_paths:
                        if not isinstance(pointer, str) or not pointer.startswith("/"):
                            failures.append(
                                f"{name}: sensitive output path {pointer!r} is not a JSON pointer"
                            )
                            continue
                        top_level = pointer[1:].split("/", 1)[0].replace("~1", "/").replace(
                            "~0", "~"
                        )
                        if top_level not in output_properties:
                            failures.append(
                                f"{name}: sensitive output path {pointer!r} does not resolve "
                                "to a declared top-level output property"
                            )
                    if (
                        "config_path" in output_properties
                        and "/config_path" not in sensitive_output_paths
                    ):
                        failures.append(
                            f"{name}: config_path output must be declared as sensitive"
                        )
                    if (
                        "loopback-listener" in effect_names
                        and "url" in output_properties
                        and "/url" not in sensitive_output_paths
                    ):
                        failures.append(f"{name}: loopback URL output must be declared as sensitive")
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
        if lite_tool_definitions is None:
            lite_tools = {
                str(item["name"]): item
                for item in lite_contract.get("tools", [])
                if isinstance(item, dict) and isinstance(item.get("name"), str)
            }
        else:
            lite_tools = dict(lite_tool_definitions)
        full_tools = (
            _load_full_tool_definitions(root)
            if full_tool_definitions is None
            else dict(full_tool_definitions)
        )
        smoke_fixture = _load_json(root / SMOKE_CALLS_RELATIVE)
        if mcpb_tool_definitions is None:
            mcpb_manifest = _load_json(root / MCPB_MANIFEST_RELATIVE)
            mcpb_tools = {
                str(item["name"]): item
                for item in mcpb_manifest.get("tools", [])
                if isinstance(item, dict) and isinstance(item.get("name"), str)
            }
        else:
            mcpb_tools = dict(mcpb_tool_definitions)
        smoke_by_id = {
            str(item["id"]): item
            for item in smoke_fixture.get("calls", [])
            if isinstance(item, dict) and isinstance(item.get("id"), str)
        }
    except (AttributeError, ValueError) as exc:
        failures.append(str(exc))
        return failures

    target_canonical_count, target_public_count, coverage_failures = (
        _derive_runtime_coverage(full_tools, lite_tools)
    )
    failures.extend(coverage_failures)
    if isinstance(coverage, Mapping):
        if coverage.get("target_canonical_tool_count") != target_canonical_count:
            failures.append(
                "coverage.target_canonical_tool_count does not match the derived "
                f"Full/Lite runtime total ({target_canonical_count})"
            )
        if coverage.get("target_public_name_count") != target_public_count:
            failures.append(
                "coverage.target_public_name_count does not match the derived "
                f"Full/Lite runtime total ({target_public_count})"
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
            input_ref = profile.get("input_schema_ref")
            canonical_input = referenced_schemas.get(str(input_ref))
            if canonical_input is not None and runtime_definition.get(
                "inputSchema"
            ) != runtime_schema_projection(canonical_input):
                failures.append(f"{name}: {profile_name} input schema drifts from v2 registry")
            for alias_name in aliases:
                alias_definition = runtime_tools.get(alias_name)
                if not isinstance(alias_definition, Mapping):
                    failures.append(
                        f"{name}: alias {alias_name} is missing from {profile_name} runtime definitions"
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
        for smoke_id in tool.get("smoke_call_ids", []):
            smoke = smoke_by_id.get(smoke_id)
            if smoke is None:
                failures.append(f"{name}: missing smoke fixture {smoke_id!r}")
            elif smoke.get("name") not in {name, *aliases}:
                failures.append(
                    f"{name}: smoke fixture {smoke_id!r} targets unrelated tool "
                    f"{smoke.get('name')!r}"
                )
            elif isinstance(canonical_inputs.get(name), Mapping):
                arguments = smoke.get("arguments")
                input_failures = validate_instance(arguments, canonical_inputs[name])
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
        smoke_names = {
            smoke_by_id[smoke_id].get("name")
            for smoke_id in tool.get("smoke_call_ids", [])
            if smoke_id in smoke_by_id
        }
        for public_name in (name, *aliases):
            if public_name not in smoke_names:
                failures.append(f"{name}: no referenced smoke fixture covers {public_name}")
            manifest_definition = mcpb_tools.get(public_name)
            if not isinstance(manifest_definition, Mapping):
                failures.append(f"{name}: {public_name} is missing from the MCPB manifest")
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
