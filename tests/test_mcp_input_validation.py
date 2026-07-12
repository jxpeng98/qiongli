from __future__ import annotations

import unittest

from bridges.mcp_input_validation import first_input_error


class MCPInputValidationTests(unittest.TestCase):
    def test_accepts_supported_local_refs_and_closed_objects(self) -> None:
        schema = {
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": {"$ref": "#/$defs/nonblank"},
                "tags": {
                    "type": "array",
                    "items": {"$ref": "#/$defs/nonblank"},
                    "maxItems": 2,
                    "uniqueItems": True,
                },
            },
            "additionalProperties": False,
            "$defs": {
                "nonblank": {
                    "type": "string",
                    "minLength": 1,
                    "pattern": ".*\\S.*",
                }
            },
        }

        self.assertIsNone(first_input_error({"query": "contract", "tags": ["a"]}, schema))
        self.assertIn("is required", first_input_error({}, schema) or "")
        self.assertIn(
            "unsupported fields",
            first_input_error({"query": "x", "raw": True}, schema) or "",
        )
        self.assertIn("unique", first_input_error({"query": "x", "tags": ["a", "a"]}, schema) or "")

    def test_closed_object_error_does_not_echo_unknown_key(self) -> None:
        canary = "QIONGLI_UNKNOWN_KEY_CANARY_DO_NOT_ECHO"
        error = first_input_error(
            {canary: True},
            {"type": "object", "properties": {}, "additionalProperties": False},
        )

        self.assertEqual(error, "input contains unsupported fields")
        self.assertNotIn(canary, error or "")

    def test_rejects_bool_as_number_and_integer(self) -> None:
        for expected in ("integer", "number"):
            with self.subTest(expected=expected):
                error = first_input_error(True, {"type": expected})
                self.assertIn("wrong type", error or "")

        for value in (float("nan"), float("inf"), float("-inf")):
            with self.subTest(value=value):
                error = first_input_error(value, {"type": "number"})
                self.assertIn("wrong type", error or "")

    def test_one_of_checks_shape_without_echoing_values(self) -> None:
        canary = "QIONGLI_INPUT_VALIDATION_CANARY_DO_NOT_ECHO"
        schema = {
            "type": "object",
            "required": ["provider"],
            "properties": {"provider": {"type": "string"}},
            "oneOf": [
                {"properties": {"provider": {"const": "openalex"}}},
                {"properties": {"provider": {"const": "crossref"}}},
            ],
            "additionalProperties": False,
        }

        error = first_input_error({"provider": canary}, schema)

        self.assertIn("exactly one", error or "")
        self.assertNotIn(canary, error or "")

    def test_rejects_invalid_internal_reference_fail_closed(self) -> None:
        error = first_input_error("x", {"$ref": "#/$defs/missing"})

        self.assertIn("invalid input schema reference", error or "")


if __name__ == "__main__":
    unittest.main()
