from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from bridges.mcp_client_config import install_mcp_config, remove_mcp_config


class MCPClientConfigTests(unittest.TestCase):
    def test_install_codex_mcp_config_dry_run_does_not_write(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_path = Path(tmp_dir) / "config.toml"

            result = install_mcp_config(config_path=config_path, dry_run=True)

        self.assertEqual(result.status, "dry-run")
        self.assertTrue(result.changed)
        self.assertFalse(config_path.exists())
        self.assertIn("[mcp_servers.qiongli]", result.preview)
        self.assertIn('args = ["mcp", "serve", "--transport", "stdio"]', result.preview)

    def test_install_codex_mcp_config_writes_managed_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_path = Path(tmp_dir) / "config.toml"
            config_path.write_text('model = "gpt-5"\n', encoding="utf-8")

            result = install_mcp_config(config_path=config_path)

            rendered = config_path.read_text(encoding="utf-8")

        self.assertEqual(result.status, "installed")
        self.assertTrue(result.changed)
        self.assertIn('model = "gpt-5"', rendered)
        self.assertIn("# BEGIN QIONGLI MANAGED MCP", rendered)
        self.assertIn("[mcp_servers.qiongli]", rendered)
        self.assertIn("# END QIONGLI MANAGED MCP", rendered)

    def test_install_codex_mcp_config_skips_unmanaged_existing_server(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_path = Path(tmp_dir) / "config.toml"
            original = '[mcp_servers.qiongli]\ncommand = "custom-qiongli"\n'
            config_path.write_text(original, encoding="utf-8")

            result = install_mcp_config(config_path=config_path)

            rendered = config_path.read_text(encoding="utf-8")

        self.assertEqual(result.status, "skipped")
        self.assertFalse(result.changed)
        self.assertEqual(rendered, original)
        self.assertIn("unmanaged", result.detail)

    def test_remove_codex_mcp_config_removes_only_managed_block(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_path = Path(tmp_dir) / "config.toml"
            config_path.write_text(
                'model = "gpt-5"\n\n'
                "# BEGIN QIONGLI MANAGED MCP\n"
                "[mcp_servers.qiongli]\n"
                'command = "qiongli"\n'
                'args = ["mcp", "serve", "--transport", "stdio"]\n'
                "# END QIONGLI MANAGED MCP\n\n"
                'approval_policy = "on-request"\n',
                encoding="utf-8",
            )

            result = remove_mcp_config(config_path=config_path)

            rendered = config_path.read_text(encoding="utf-8")

        self.assertEqual(result.status, "removed")
        self.assertTrue(result.changed)
        self.assertIn('model = "gpt-5"', rendered)
        self.assertIn('approval_policy = "on-request"', rendered)
        self.assertNotIn("QIONGLI MANAGED MCP", rendered)
        self.assertNotIn("[mcp_servers.qiongli]", rendered)


if __name__ == "__main__":
    unittest.main()
