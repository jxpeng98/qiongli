from __future__ import annotations

import json
import math
import re
from collections.abc import Mapping
from typing import Any


def first_input_error(value: Any, schema: Mapping[str, Any]) -> str | None:
    """Return a bounded error for the supported MCP input-schema vocabulary."""

    return _first_error(value, schema, path="arguments", root=schema)


def _first_error(
    value: Any,
    schema: Mapping[str, Any],
    *,
    path: str,
    root: Mapping[str, Any],
) -> str | None:
    reference = schema.get("$ref")
    if isinstance(reference, str):
        target = _resolve_local_reference(root, reference)
        if target is None:
            return f"{path} uses an invalid input schema reference"
        return _first_error(value, target, path=path, root=root)

    one_of = schema.get("oneOf")
    if isinstance(one_of, list):
        matches = sum(
            1
            for option in one_of
            if isinstance(option, Mapping)
            and _first_error(value, option, path=path, root=root) is None
        )
        if matches != 1:
            return f"{path} must match exactly one allowed input shape"

    if "const" in schema and value != schema["const"]:
        return f"{path} has an unsupported value"
    allowed = schema.get("enum")
    if isinstance(allowed, list) and value not in allowed:
        return f"{path} has an unsupported value"

    expected = schema.get("type")
    if expected is not None:
        expected_types = [expected] if isinstance(expected, str) else expected
        if not isinstance(expected_types, list) or not any(
            isinstance(item, str) and _type_matches(value, item)
            for item in expected_types
        ):
            return f"{path} has the wrong type"

    if isinstance(value, dict):
        required = schema.get("required", [])
        if isinstance(required, list):
            for key in required:
                if isinstance(key, str) and key not in value:
                    return f"{path}.{key} is required"
        properties = schema.get("properties", {})
        properties = properties if isinstance(properties, Mapping) else {}
        additional = schema.get("additionalProperties", True)
        for key, item in value.items():
            child_path = f"{path}.{key}"
            child_schema = properties.get(key)
            if isinstance(child_schema, Mapping):
                error = _first_error(item, child_schema, path=child_path, root=root)
                if error is not None:
                    return error
            elif additional is False:
                return "input contains unsupported fields"
            elif isinstance(additional, Mapping):
                error = _first_error(item, additional, path=child_path, root=root)
                if error is not None:
                    return error

    if isinstance(value, list):
        minimum_items = schema.get("minItems")
        maximum_items = schema.get("maxItems")
        if isinstance(minimum_items, int) and len(value) < minimum_items:
            return f"{path} has too few items"
        if isinstance(maximum_items, int) and len(value) > maximum_items:
            return f"{path} has too many items"
        if schema.get("uniqueItems") is True:
            rendered = [
                json.dumps(item, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
                for item in value
            ]
            if len(rendered) != len(set(rendered)):
                return f"{path} must contain unique items"
        item_schema = schema.get("items")
        if isinstance(item_schema, Mapping):
            for index, item in enumerate(value):
                error = _first_error(
                    item,
                    item_schema,
                    path=f"{path}[{index}]",
                    root=root,
                )
                if error is not None:
                    return error

    if isinstance(value, str):
        minimum_length = schema.get("minLength")
        maximum_length = schema.get("maxLength")
        if isinstance(minimum_length, int) and len(value) < minimum_length:
            return f"{path} is too short"
        if isinstance(maximum_length, int) and len(value) > maximum_length:
            return f"{path} is too long"
        pattern = schema.get("pattern")
        if isinstance(pattern, str):
            try:
                matches = re.search(pattern, value) is not None
            except re.error:
                return f"{path} uses an invalid input schema pattern"
            if not matches:
                return f"{path} has an invalid format"

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        minimum = schema.get("minimum")
        maximum = schema.get("maximum")
        if isinstance(minimum, (int, float)) and value < minimum:
            return f"{path} is below the allowed minimum"
        if isinstance(maximum, (int, float)) and value > maximum:
            return f"{path} exceeds the allowed maximum"

    return None


def _resolve_local_reference(
    root: Mapping[str, Any],
    reference: str,
) -> Mapping[str, Any] | None:
    if not reference.startswith("#/"):
        return None
    current: Any = root
    for raw_token in reference[2:].split("/"):
        token = raw_token.replace("~1", "/").replace("~0", "~")
        if not isinstance(current, Mapping) or token not in current:
            return None
        current = current[token]
    return current if isinstance(current, Mapping) else None


def _type_matches(value: Any, expected: str) -> bool:
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "string":
        return isinstance(value, str)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return (
            isinstance(value, (int, float))
            and not isinstance(value, bool)
            and math.isfinite(value)
        )
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "null":
        return value is None
    return False
