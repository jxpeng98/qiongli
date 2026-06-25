from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from qiongli import cli as cli_module


REPO_ROOT = Path(__file__).resolve().parents[1]


class InstallerCliTests(unittest.TestCase):
    def test_check_prints_latest_stable_and_prerelease(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            skill_dirs = {
                "codex": root / "codex",
                "claude": root / "claude",
                "antigravity": root / "antigravity",
                "hermes": root / "hermes",
            }
            args = argparse.Namespace(
                repo="owner/repo",
                json=False,
                strict_network=False,
                beta=False,
            )

            def fake_http_get_json(url: str) -> object:
                if url.endswith("/releases/latest"):
                    return {"tag_name": "v1.0.0"}
                if "/releases?per_page=" in url:
                    return [
                        {"tag_name": "v1.1.0-beta.1", "prerelease": True, "draft": False},
                        {"tag_name": "v1.0.0", "prerelease": False, "draft": False},
                    ]
                raise AssertionError(f"Unexpected URL: {url}")

            stdout = io.StringIO()
            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_pip_version", return_value=("1.0.0", "up-to-date")
            ), mock.patch.object(cli_module, "_check_system_env", return_value={}), mock.patch.object(
                cli_module, "_installed_skill_dirs", return_value=skill_dirs
            ), mock.patch.object(cli_module, "_http_get_json", side_effect=fake_http_get_json), contextlib.redirect_stdout(
                stdout
            ):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        output = stdout.getvalue()
        self.assertIn("   - Latest: v1.0.0", output)
        self.assertIn("   - Pre-release: v1.1.0-beta.1", output)

    def test_init_defaults_to_project_part(self) -> None:
        args = argparse.Namespace(
            project_dir=".",
            target="all",
            mode="copy",
            overwrite=False,
            doctor=False,
            dry_run=False,
            parts=None,
        )

        with mock.patch.object(cli_module, "install", return_value=0) as install_mock:
            exit_code = cli_module.cmd_init(args)

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.parts, ("project",))
        self.assertEqual(options.target, "all")

    def test_install_command_passes_subject_to_installer(self) -> None:
        with mock.patch.object(cli_module, "install", return_value=0) as install_mock:
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "install", "--subject", "economics", "--coverage", "focused", "--target", "codex"],
            ):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.subject, "economics")
        self.assertEqual(options.coverage, "focused")
        self.assertEqual(options.target, "codex")

    def test_install_command_passes_profile_to_installer(self) -> None:
        with mock.patch.object(cli_module, "install", return_value=0) as install_mock:
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "install", "--profile", "full", "--target", "codex", "--dry-run"],
            ):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.profile, "full")
        self.assertEqual(options.target, "codex")
        self.assertTrue(options.dry_run)

    def test_install_command_accepts_hermes_target(self) -> None:
        with mock.patch.object(cli_module, "install", return_value=0) as install_mock:
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "install", "--target", "hermes", "--dry-run"],
            ):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.target, "hermes")
        self.assertTrue(options.dry_run)

    def test_remove_command_passes_target_and_parts_to_remover(self) -> None:
        with mock.patch.object(cli_module, "remove", return_value=0) as remove_mock:
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "remove", "--target", "codex", "--parts", "globals,cli", "--dry-run"],
            ):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        options = remove_mock.call_args.args[0]
        self.assertEqual(options.target, "codex")
        self.assertEqual(options.parts, ("globals", "cli"))
        self.assertTrue(options.dry_run)

    def test_setup_command_dispatches_to_setup_wizard(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            with mock.patch("qiongli.setup_wizard.run_setup_wizard", return_value=object()) as setup_mock:
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "setup", "--dry-run", "--project-dir", tmp_dir, "--no-doctor"],
                ):
                    exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        setup_mock.assert_called_once()
        args = setup_mock.call_args.args[0]
        self.assertEqual(args.cmd, "setup")
        self.assertEqual(args.project_dir, tmp_dir)
        self.assertTrue(args.dry_run)
        self.assertTrue(args.no_doctor)

    def test_setup_command_is_listed_in_help(self) -> None:
        parser = cli_module.build_parser()

        help_text = parser.format_help()
        normalized_help = " ".join(help_text.split())

        self.assertIn("setup", help_text)
        self.assertIn(
            "Interactively configures Qiongli for CLI/Codex/Claude Code/Antigravity use",
            normalized_help,
        )

    def test_install_unknown_subject_reports_available_subjects(self) -> None:
        stderr = io.StringIO()
        with mock.patch.object(
            cli_module.sys,
            "argv",
            ["qiongli", "install", "--subject", "unknown", "--target", "codex", "--dry-run"],
        ), contextlib.redirect_stderr(stderr):
            exit_code = cli_module.main()

        self.assertEqual(exit_code, 2)
        self.assertIn(
            "Unknown subject 'unknown'. Available subjects: accounting, business, core, economics, economics-accounting, finance",
            stderr.getvalue(),
        )

    def test_packaged_payload_root_falls_back_to_checkout_cwd_when_payload_is_absent(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            fake_site_file = Path(tmp_dir) / "site-packages" / "qiongli" / "cli.py"
            fake_site_file.parent.mkdir(parents=True)
            fake_site_file.write_text("# installed package placeholder\n", encoding="utf-8")
            with mock.patch.object(cli_module, "__file__", str(fake_site_file)):
                payload_root = cli_module._packaged_payload_root()

        self.assertEqual(payload_root, REPO_ROOT)

    def test_align_describes_global_first_upgrade_and_project_init(self) -> None:
        args = argparse.Namespace(repo="owner/repo")

        with mock.patch("builtins.print") as print_mock:
            exit_code = cli_module.cmd_align(args)

        self.assertEqual(exit_code, 0)
        lines = [" ".join(str(part) for part in call.args) for call in print_mock.call_args_list]
        joined = "\n".join(lines)
        self.assertIn("What `", joined)
        self.assertIn("upgrade` modifies by default", joined)
        self.assertIn("Use `qiongli init --project-dir .` to create project config", joined)
        self.assertIn("qiongli init", joined)

    def test_upgrade_passes_parts_to_installer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            extracted_root = temp_root / "archive-root"
            scripts_dir = extracted_root / "scripts"
            scripts_dir.mkdir(parents=True)
            (scripts_dir / "bootstrap_qiongli.py").write_text("# stub\n", encoding="utf-8")

            args = argparse.Namespace(
                repo="owner/repo",
                ref="v0.4.0",
                ref_type="tag",
                target="all",
                beta=False,
                mode="copy",
                project_dir=str(temp_root / "project"),
                overwrite=True,
                doctor=False,
                dry_run=False,
                parts="project,cli",
                subject="economics",
                coverage="focused",
            )

            with mock.patch.object(cli_module, "_download") as download_mock, mock.patch.object(
                cli_module, "_extract_tarball", return_value=extracted_root
            ), mock.patch.object(cli_module, "install", return_value=0) as install_mock:
                exit_code = cli_module.cmd_upgrade(args)

        self.assertEqual(exit_code, 0)
        download_mock.assert_called_once()
        options = install_mock.call_args.args[0]
        self.assertEqual(options.parts, ("project", "cli"))
        self.assertEqual(options.subject, "economics")
        self.assertEqual(options.coverage, "focused")

    def test_check_json_reports_installed_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            skill_dir = root / "codex" / "skills" / "qiongli-workflow"
            skill_dir.mkdir(parents=True)
            (skill_dir / "VERSION").write_text("v9.9.9\n", encoding="utf-8")
            (skill_dir / "SUBJECT").write_text("economics\n", encoding="utf-8")
            (skill_dir / "SUBJECT_MANIFEST.json").write_text(
                json.dumps({"subject": "economics", "coverage": "focused", "flavor": "full", "layers": ["core", "economics"]}),
                encoding="utf-8",
            )
            args = argparse.Namespace(
                repo="",
                json=True,
                strict_network=False,
                beta=False,
            )
            skill_dirs = {
                "codex": skill_dir,
                "claude": root / "claude" / "skills" / "qiongli-workflow",
                "antigravity": root / "antigravity" / "skills" / "qiongli-workflow",
                "hermes": root / "hermes" / "skills" / "qiongli-workflow",
            }

            stdout = io.StringIO()
            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_pip_version", return_value=("9.9.9", "up-to-date")
            ), mock.patch.object(cli_module, "_check_system_env", return_value={}), mock.patch.object(
                cli_module, "_installed_skill_dirs", return_value=skill_dirs
            ), mock.patch.object(
                cli_module, "_resolve_upstream_repo", return_value=(None, "")
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["installed"]["codex"]["subject"], "economics")
        self.assertEqual(payload["installed"]["codex"]["coverage"], "focused")
        self.assertEqual(payload["installed"]["claude"]["subject"], None)
        self.assertEqual(payload["installed"]["claude"]["coverage"], None)

    def test_doctor_runs_orchestrator_subprocess(self) -> None:
        args = argparse.Namespace(cwd=".")
        completed = mock.Mock(returncode=0, stdout="doctor ok\n")

        with mock.patch.object(cli_module.subprocess, "run", return_value=completed) as run_mock:
            exit_code = cli_module.cmd_doctor(args)

        self.assertEqual(exit_code, 0)
        run_mock.assert_called_once()
        command = run_mock.call_args.args[0]
        self.assertEqual(command[:3], [cli_module.sys.executable, "-m", "bridges.orchestrator"])
        self.assertEqual(command[3:], ["doctor", "--cwd", str(Path(".").resolve())])

    def test_guidance_command_runs_orchestrator_subprocess(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            completed = mock.Mock(returncode=0, stdout="guidance ok\n")
            stdout = io.StringIO()

            with mock.patch.object(cli_module.subprocess, "run", return_value=completed) as run_mock, contextlib.redirect_stdout(
                stdout
            ):
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "guidance", "init", "--project-dir", str(project_dir)],
                ):
                    exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        run_mock.assert_called_once()
        command = run_mock.call_args.args[0]
        self.assertEqual(command[:3], [cli_module.sys.executable, "-m", "bridges.orchestrator"])
        self.assertEqual(
            command[3:],
            ["guidance", "init", "--project-dir", str(project_dir.resolve())],
        )

    def test_guidance_add_command_runs_orchestrator_subprocess(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir)
            completed = mock.Mock(returncode=0, stdout="guidance add ok\n")
            args = argparse.Namespace(guidance_cmd="add", project_dir=str(project_dir), name="writing-style")

            with mock.patch.object(cli_module.subprocess, "run", return_value=completed) as run_mock:
                exit_code = cli_module.cmd_guidance(args)

        self.assertEqual(exit_code, 0)
        command = run_mock.call_args.args[0]
        self.assertIn("guidance", command)
        self.assertIn("add", command)
        self.assertIn("--name", command)
        self.assertIn("writing-style", command)

    def test_provider_set_and_list_redacts_global_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_home = Path(tmp_dir) / "config"
            with mock.patch.dict(os.environ, {"QIONGLI_CONFIG_HOME": str(config_home)}, clear=False):
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "provider", "set", "semantic-scholar", "api-key", "cli-secret"],
                ):
                    self.assertEqual(cli_module.main(), 0)

                stdout = io.StringIO()
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "provider", "list", "--json"],
                ), contextlib.redirect_stdout(stdout):
                    self.assertEqual(cli_module.main(), 0)

        payload = json.loads(stdout.getvalue())
        rendered = json.dumps(payload, sort_keys=True)
        self.assertNotIn("cli-secret", rendered)
        self.assertEqual(payload["providers"]["semantic_scholar"]["fields"]["api_key"], "configured")

    def test_provider_doctor_json_reports_provider_connected_mode(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            config_home = Path(tmp_dir) / "config"
            with mock.patch.dict(os.environ, {"QIONGLI_CONFIG_HOME": str(config_home)}, clear=False):
                for provider, field, value in (
                    ("openalex", "api-key", "openalex-secret-key"),
                    ("semantic-scholar", "api-key", "cli-secret"),
                ):
                    with mock.patch.object(
                        cli_module.sys,
                        "argv",
                        ["qiongli", "provider", "set", provider, field, value],
                    ):
                        self.assertEqual(cli_module.main(), 0)

                stdout = io.StringIO()
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "provider", "doctor", "--json"],
                ), contextlib.redirect_stdout(stdout):
                    self.assertEqual(cli_module.main(), 0)

        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["capability_mode"], "provider_connected")
        self.assertEqual(payload["providers"]["openalex"], "configured")
        self.assertEqual(payload["providers"]["semantic_scholar"], "configured")

if __name__ == "__main__":
    unittest.main()
