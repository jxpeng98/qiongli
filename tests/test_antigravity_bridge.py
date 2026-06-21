from __future__ import annotations

import unittest
from pathlib import Path

from bridges.antigravity_bridge import AntigravityBridge


class AntigravityBridgeTests(unittest.TestCase):
    def test_build_command_uses_noninteractive_print_mode(self) -> None:
        bridge = AntigravityBridge(
            sandbox=True,
            model="ag-model",
            dangerously_skip_permissions=True,
            print_timeout="30s",
        )

        command = bridge.build_command(
            "Review this draft.",
            Path("/tmp/project"),
            session_id="conversation-123",
            add_dirs=["/tmp/project/notes"],
        )

        self.assertEqual(command[0], "antigravity")
        self.assertIn("--print", command)
        self.assertIn("--sandbox", command)
        self.assertIn("--dangerously-skip-permissions", command)
        self.assertIn("--model", command)
        self.assertIn("ag-model", command)
        self.assertIn("--conversation", command)
        self.assertIn("conversation-123", command)
        self.assertIn("--print-timeout", command)
        self.assertIn("30s", command)
        self.assertEqual(command[-1], "Review this draft.")

    def test_build_command_allows_runtime_options_to_override_defaults(self) -> None:
        bridge = AntigravityBridge(sandbox=False)

        command = bridge.build_command(
            "Audit this result.",
            Path("/tmp/project"),
            model="profile-model",
            sandbox=True,
            print_timeout="45s",
            add_dirs=["/tmp/project/data"],
        )

        self.assertIn("--sandbox", command)
        self.assertIn("--model", command)
        self.assertIn("profile-model", command)
        self.assertIn("--print-timeout", command)
        self.assertIn("45s", command)
        self.assertIn("--add-dir", command)
        self.assertIn("/tmp/project/data", command)

    def test_parse_output_accepts_plain_print_response_without_session_id(self) -> None:
        bridge = AntigravityBridge()

        response = bridge.parse_output(["Line one", "Line two"])

        self.assertTrue(response.success)
        self.assertEqual(response.model, "antigravity")
        self.assertIsNone(response.session_id)
        self.assertEqual(response.content, "Line one\nLine two")

    def test_parse_output_extracts_json_assistant_content_and_conversation_id(self) -> None:
        bridge = AntigravityBridge()

        response = bridge.parse_output(
            [
                '{"type":"assistant","conversation_id":"abc","content":[{"text":"Hello"}]}',
                '{"type":"status","message":"done"}',
            ]
        )

        self.assertTrue(response.success)
        self.assertEqual(response.session_id, "abc")
        self.assertEqual(response.content, "Hello")


if __name__ == "__main__":
    unittest.main()
