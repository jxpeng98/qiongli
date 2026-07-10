from __future__ import annotations

import contextlib
import io
import json
import os
import queue
import subprocess
import sys
import tempfile
import threading
import unittest
import urllib.parse
import urllib.request
from pathlib import Path
from unittest import mock

from bridges import mcp_cli


class MCPCLITests(unittest.TestCase):
    def test_mcp_cli_doctor_json_reports_shared_provider_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            env = self._env(root)
            subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "bridges.mcp_cli",
                    "configure",
                    "--provider",
                    "openalex",
                    "--field",
                    "api-key",
                    "--value",
                    "openalex-secret-key",
                ],
                capture_output=True,
                text=True,
                check=True,
                env=env,
            )

            result = subprocess.run(
                [sys.executable, "-m", "bridges.mcp_cli", "doctor", "--json", "--cwd", str(root)],
                capture_output=True,
                text=True,
                check=False,
                env=env,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        rendered = json.dumps(payload, sort_keys=True)
        self.assertEqual(payload["providers"]["openalex"], "configured")
        self.assertEqual(payload["capability_mode"], "provider_connected")
        self.assertIn("literature_tools_available", payload)
        self.assertTrue(payload["literature_tools_available"])
        self.assertIn("qiongli_literature_search", payload["literature_tools"])
        self.assertIn("qiongli_search_plan", payload["literature_tools"])
        self.assertIn("qiongli_configure_provider", payload["next_action"]["tool"])
        self.assertNotIn("openalex-secret-key", rendered)

    def test_mcp_doctor_reports_strategy_only_for_configured_but_disabled_providers(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            config_home.mkdir()
            (config_home / "providers.json").write_text(
                json.dumps(
                    {
                        "version": 1,
                        "providers": {
                            "semantic_scholar": {
                                "enabled": False,
                                "api_key": "disabled-canary",
                            },
                            "arxiv": {"enabled": False},
                        },
                    }
                ),
                encoding="utf-8",
            )
            env = self._env(root)
            with mock.patch.dict(os.environ, env, clear=True):
                payload = mcp_cli._doctor_payload(root)

        self.assertEqual(payload["providers"]["semantic_scholar"], "configured")
        self.assertEqual(payload["capability_mode"], "strategy_only")

    def test_mcp_cli_config_example_for_codex_json(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_cli", "config", "example", "--target", "codex", "--json"],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["target"], "codex")
        self.assertEqual(payload["server"]["command"], "qiongli")
        self.assertEqual(payload["server"]["args"], ["mcp", "serve", "--transport", "stdio"])
        self.assertIn("qiongli_literature_search", payload["literature_tools"])
        self.assertIn("qiongli_search_plan", payload["literature_tools"])
        self.assertIn("qiongli_orchestrator_route", payload["orchestrator_tools"])
        self.assertIn("qiongli_task_run", payload["orchestrator_tools"])
        self.assertIn("qiongli_task_plan", payload["orchestrator_tools"])
        self.assertIn("qiongli_configure_provider", payload["configuration_tools"])
        self.assertIn("qiongli_open_config_wizard", payload["configuration_tools"])
        self.assertEqual(payload["safety"]["task_run_default"], "preview")

    def test_mcp_cli_config_example_accepts_claude_code_alias(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_cli", "config", "example", "--target", "claude-code", "--json"],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["target"], "claude-code")
        self.assertEqual(payload["server"]["args"], ["mcp", "serve", "--transport", "stdio"])
        self.assertIn("qiongli_orchestrator_route", payload["orchestration_tools"])
        self.assertIn("qiongli_task_run", payload["orchestration_tools"])

    def test_mcp_cli_config_example_for_hermes_json(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_cli", "config", "example", "--target", "hermes", "--json"],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["target"], "hermes")
        self.assertEqual(payload["server"]["command"], "qiongli")
        self.assertEqual(payload["server"]["args"], ["mcp", "serve", "--transport", "stdio"])
        self.assertIn("qiongli_configure_provider", payload["configuration_tools"])
        self.assertIn("qiongli_orchestrator_route", payload["orchestration_tools"])
        self.assertIn("qiongli_task_run", payload["orchestration_tools"])

    def test_mcp_cli_config_example_for_antigravity_json(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "bridges.mcp_cli", "config", "example", "--target", "antigravity", "--json"],
            capture_output=True,
            text=True,
            check=False,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["target"], "antigravity")
        self.assertEqual(payload["server"]["command"], "qiongli")
        self.assertEqual(payload["server"]["args"], ["mcp", "serve", "--transport", "stdio"])
        self.assertIn("qiongli_literature_search", payload["literature_tools"])
        self.assertIn("qiongli_search_plan", payload["literature_tools"])
        self.assertIn("qiongli_orchestrator_route", payload["orchestration_tools"])
        self.assertIn("qiongli_task_run", payload["orchestration_tools"])

    def test_mcp_cli_upgrade_delegates_to_qiongli_upgrade(self) -> None:
        calls = []

        with mock.patch("qiongli.cli.cmd_upgrade", side_effect=lambda args: calls.append(args) or 7):
            exit_code = mcp_cli.main(
                [
                    "upgrade",
                    "--repo",
                    "owner/repo",
                    "--ref",
                    "v1.2.0",
                    "--target",
                    "hermes",
                    "--project-dir",
                    "/tmp/project",
                    "--dry-run",
                ]
            )

        self.assertEqual(exit_code, 7)
        self.assertEqual(len(calls), 1)
        args = calls[0]
        self.assertEqual(args.repo, "owner/repo")
        self.assertEqual(args.ref, "v1.2.0")
        self.assertEqual(args.ref_type, "tag")
        self.assertEqual(args.target, "hermes")
        self.assertEqual(args.project_dir, "/tmp/project")
        self.assertTrue(args.overwrite)
        self.assertTrue(args.dry_run)

    def test_mcp_cli_upgrade_help_describes_adaptive_core_subject_semantics(self) -> None:
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            with self.assertRaises(SystemExit) as cm:
                mcp_cli.main(["upgrade", "--help"])

        self.assertEqual(cm.exception.code, 0)
        help_text = stdout.getvalue()
        normalized_help = " ".join(help_text.split())
        self.assertIn("Advanced override for pre-materialized subject packages", normalized_help)
        self.assertIn("Default core keeps runtime subject refinement adaptive", normalized_help)

    def test_mcp_cli_wizard_exits_after_provider_values_are_saved(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            proc = subprocess.Popen(
                [
                    sys.executable,
                    "-u",
                    "-m",
                    "bridges.mcp_cli",
                    "wizard",
                    "--json",
                ],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                env=self._env(root),
            )
            try:
                payload = self._read_wizard_payload(proc)
                body = urllib.parse.urlencode({"openalex.api_key": "openalex-secret-key"}).encode(
                    "utf-8"
                )
                request = urllib.request.Request(
                    payload["url"].replace("/?token=", "/save?token="),
                    data=body,
                    method="POST",
                )
                response = urllib.request.urlopen(request, timeout=5)
                html = response.read().decode("utf-8")
                self.assertIn("You can close this page", html)

                returncode = proc.wait(timeout=10)
            finally:
                if proc.poll() is None:
                    proc.terminate()
                    proc.wait(timeout=5)

                stderr = proc.stderr.read() if proc.stderr is not None else ""
                if proc.stdout is not None:
                    proc.stdout.close()
                if proc.stderr is not None:
                    proc.stderr.close()

        self.assertEqual(returncode, 0, stderr)

    def test_mcp_cli_wizard_payload_reader_fails_when_child_exits_without_json(self) -> None:
        proc = subprocess.Popen(
            [
                sys.executable,
                "-c",
                "import sys; sys.stderr.write('wizard failed before JSON\\n'); sys.exit(7)",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            with self.assertRaisesRegex(AssertionError, "exited before emitting JSON"):
                self._read_wizard_payload(proc, timeout=2)
        finally:
            proc.wait(timeout=5)
            if proc.stdout is not None:
                proc.stdout.close()
            if proc.stderr is not None:
                proc.stderr.close()

    def test_qiongli_cli_delegates_mcp_subcommand(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            result = subprocess.run(
                [sys.executable, "-m", "qiongli.cli", "mcp", "doctor", "--json", "--cwd", str(root)],
                capture_output=True,
                text=True,
                check=False,
                env=self._env(root),
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["server"]["name"], "qiongli-mcp")
        self.assertEqual(payload["providers"]["semantic_scholar"], "missing")

    def _env(self, root: Path) -> dict[str, str]:
        env = dict(os.environ)
        env["QIONGLI_CONFIG_HOME"] = str(root / "config")
        return env

    def _read_wizard_payload(self, proc: subprocess.Popen[str], *, timeout: float = 5.0) -> dict[str, object]:
        self.assertIsNotNone(proc.stdout)
        payloads: queue.Queue[dict[str, object] | None] = queue.Queue(maxsize=1)

        def read_payload() -> None:
            assert proc.stdout is not None
            lines = []
            while True:
                line = proc.stdout.readline()
                if not line:
                    payloads.put(None)
                    return
                lines.append(line)
                try:
                    payloads.put(json.loads("".join(lines)))
                    return
                except json.JSONDecodeError:
                    continue

        reader = threading.Thread(target=read_payload, name="qiongli-wizard-json-reader", daemon=True)
        reader.start()
        try:
            payload = payloads.get(timeout=timeout)
        except queue.Empty:
            if proc.poll() is None:
                proc.terminate()
                proc.wait(timeout=5)
            self.fail(f"timed out waiting for wizard JSON: {self._read_stderr(proc)}")
        if payload is None:
            stderr = self._read_stderr(proc)
            if "PermissionError" in stderr and "Operation not permitted" in stderr:
                self.skipTest("local environment does not allow binding the wizard HTTP server")
            self.fail(f"wizard exited before emitting JSON with code {proc.poll()}: {stderr}")
        return payload

    def _read_stderr(self, proc: subprocess.Popen[str]) -> str:
        if proc.stderr is None:
            return ""
        return proc.stderr.read()


if __name__ == "__main__":
    unittest.main()
