from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from qiongli import cli as cli_module


REPO_ROOT = Path(__file__).resolve().parents[1]


def _isolated_qiongli_env(root: Path, **overrides: str) -> dict[str, str]:
    env = {
        "HOME": str(root / "home"),
        "USERPROFILE": str(root / "home"),
        "CODEX_HOME": str(root / "codex-home"),
        "CLAUDE_CODE_HOME": str(root / "claude-home"),
        "ANTIGRAVITY_HOME": str(root / "antigravity-home"),
        "HERMES_HOME": str(root / "hermes-home"),
        "QIONGLI_CODEX_MARKETPLACE_PATH": str(root / ".agents" / "plugins" / "marketplace.json"),
        "QIONGLI_CLAUDE_PLUGIN_PARENT": str(root / ".qiongli" / "plugins" / "claude-code"),
        "QIONGLI_ANTIGRAVITY_PLUGIN_PARENT": str(root / ".qiongli" / "plugins" / "antigravity"),
        "CLAUDE_CODE_CONFIG_PATH": str(root / ".claude.json"),
        "ANTIGRAVITY_CONFIG_PATH": str(root / ".gemini" / "config" / "mcp_config.json"),
        "HERMES_CONFIG_PATH": str(root / "hermes-home" / "settings.json"),
        "QIONGLI_CONFIG_HOME": str(root / ".qiongli-config"),
        "PATH": "",
    }
    env.update(overrides)
    return env


class InstallerCliTests(unittest.TestCase):
    def test_check_prints_latest_stable_and_prerelease(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
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
                cli_module.os,
                "environ",
                _isolated_qiongli_env(
                    root,
                    QIONGLI_CODEX_MARKETPLACE_PATH=str(root / "agents" / "plugins" / "marketplace.json"),
                    QIONGLI_CLAUDE_PLUGIN_PARENT=str(root / "claude-plugins"),
                    CLAUDE_CODE_CONFIG_PATH=str(root / "claude.json"),
                    ANTIGRAVITY_CONFIG_PATH=str(root / "antigravity-settings.json"),
                    HERMES_CONFIG_PATH=str(root / "hermes-settings.json"),
                ),
            ), mock.patch.object(cli_module, "_http_get_json", side_effect=fake_http_get_json), contextlib.redirect_stdout(
                stdout
            ):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        output = stdout.getvalue()
        self.assertIn("   - Latest: v1.0.0", output)
        self.assertIn("   - Pre-release: v1.1.0-beta.1", output)
        self.assertIn("3) Installed Client Surfaces", output)

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

    def test_install_defaults_to_full_plugin_surface(self) -> None:
        with mock.patch.object(cli_module, "install", return_value=0) as install_mock:
            with mock.patch.object(cli_module.sys, "argv", ["qiongli", "install", "--dry-run"]):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.profile, "full")
        self.assertEqual(options.surface, "plugin")
        self.assertTrue(options.dry_run)

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

    def test_install_command_passes_surface_to_installer(self) -> None:
        stderr = io.StringIO()
        with mock.patch.object(cli_module, "install", return_value=0) as install_mock:
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "install", "--profile", "full", "--target", "codex", "--surface", "plugin", "--dry-run"],
            ), contextlib.redirect_stderr(stderr):
                try:
                    exit_code = cli_module.main()
                except SystemExit as exc:
                    self.fail(f"--surface should be accepted by install parser: {exc}")

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.surface, "plugin")

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

    def test_self_update_dispatches_to_update_runner(self) -> None:
        stderr = io.StringIO()
        with mock.patch.object(cli_module, "execute_self_update", return_value=0) as update_mock:
            with mock.patch.object(
                cli_module.sys,
                "argv",
                [
                    "qiongli",
                    "self-update",
                    "--channel",
                    "next",
                    "--target",
                    "claude",
                    "--surface",
                    "both",
                    "--profile",
                    "full",
                    "--no-refresh",
                    "--skip-check",
                    "--dry-run",
                    "--yes",
                ],
            ), contextlib.redirect_stderr(stderr):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        update_mock.assert_called_once()
        options = update_mock.call_args.args[0]
        self.assertEqual(options.channel, "next")
        self.assertEqual(options.target, "claude")
        self.assertEqual(options.surface, "both")
        self.assertEqual(options.profile, "full")
        self.assertTrue(options.dry_run)
        self.assertTrue(options.yes)
        self.assertFalse(options.refresh)
        self.assertFalse(options.check)
        self.assertIn("deprecated", stderr.getvalue())

    def test_self_update_help_hides_install_shape_options(self) -> None:
        stdout = io.StringIO()
        with mock.patch.object(cli_module.sys, "argv", ["qiongli", "self-update", "--help"]):
            with contextlib.redirect_stdout(stdout):
                with self.assertRaises(SystemExit) as cm:
                    cli_module.main()

        self.assertEqual(cm.exception.code, 0)
        help_text = stdout.getvalue()
        self.assertIn("--yes", help_text)
        self.assertIn("--no-refresh", help_text)
        self.assertNotIn("--target", help_text)
        self.assertNotIn("--surface", help_text)
        self.assertNotIn("--profile", help_text)

    def test_install_help_describes_adaptive_core_subject_semantics(self) -> None:
        stdout = io.StringIO()
        with mock.patch.object(cli_module.sys, "argv", ["qiongli", "install", "--help"]):
            with contextlib.redirect_stdout(stdout):
                with self.assertRaises(SystemExit) as cm:
                    cli_module.main()

        self.assertEqual(cm.exception.code, 0)
        help_text = stdout.getvalue()
        normalized_help = " ".join(help_text.split())
        self.assertIn("Advanced override for pre-materialized subject packages", normalized_help)
        self.assertIn("Default core installs adaptive runtime subject refinement", normalized_help)

    def test_upgrade_help_describes_content_only_refresh(self) -> None:
        stdout = io.StringIO()
        with mock.patch.object(cli_module.sys, "argv", ["qiongli", "upgrade", "--help"]):
            with contextlib.redirect_stdout(stdout):
                with self.assertRaises(SystemExit) as cm:
                    cli_module.main()

        self.assertEqual(cm.exception.code, 0)
        help_text = stdout.getvalue()
        normalized_help = " ".join(help_text.split())
        self.assertIn("Refresh local assets", help_text)
        self.assertIn("without updating the CLI package", help_text)
        self.assertIn("Advanced override for pre-materialized subject packages", normalized_help)
        self.assertIn("Default core keeps runtime subject refinement adaptive", normalized_help)

    def test_update_alias_dispatches_to_self_update_runner(self) -> None:
        with mock.patch.object(cli_module, "execute_self_update", return_value=0) as update_mock:
            with mock.patch.object(cli_module.sys, "argv", ["qiongli", "update", "--dry-run"]):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        options = update_mock.call_args.args[0]
        self.assertEqual(options.channel, "stable")
        self.assertTrue(options.dry_run)

    def test_provider_setup_opens_config_page_without_prompting_for_keys(self) -> None:
        args = argparse.Namespace(
            provider_cmd="setup",
            global_config=True,
            project=False,
            provider=None,
            host="127.0.0.1",
            port=0,
            no_browser=True,
        )
        page_result = SimpleNamespace(status="saved", url="http://127.0.0.1:8765/?token=abc")

        with mock.patch.object(cli_module, "run_config_wizard", return_value=page_result) as wizard_mock:
            with mock.patch("builtins.input", side_effect=AssertionError("provider setup should use the page")):
                exit_code = cli_module.cmd_provider(args)

        self.assertEqual(exit_code, 0)
        wizard_mock.assert_called_once()
        self.assertEqual(wizard_mock.call_args.kwargs["provider"], None)
        self.assertFalse(wizard_mock.call_args.kwargs["open_browser"])

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

    def test_align_describes_plugin_first_upgrade_migration_and_project_init(self) -> None:
        args = argparse.Namespace(repo="owner/repo")

        with mock.patch("builtins.print") as print_mock:
            exit_code = cli_module.cmd_align(args)

        self.assertEqual(exit_code, 0)
        lines = [" ".join(str(part) for part in call.args) for call in print_mock.call_args_list]
        joined = "\n".join(lines)
        self.assertIn("What `", joined)
        self.assertIn("upgrade` modifies by default", joined)
        self.assertIn("Full local plugin surface", joined)
        self.assertIn("migrates old skills/MCP surfaces", joined)
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

    def test_upgrade_command_passes_surface_to_installer(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            extracted_root = temp_root / "archive-root"
            scripts_dir = extracted_root / "scripts"
            scripts_dir.mkdir(parents=True)
            (scripts_dir / "bootstrap_qiongli.py").write_text("# stub\n", encoding="utf-8")

            stderr = io.StringIO()
            with mock.patch.object(cli_module, "_resolve_upstream_repo", return_value=("owner/repo", "test")), mock.patch.object(
                cli_module, "_download"
            ), mock.patch.object(cli_module, "_extract_tarball", return_value=extracted_root), mock.patch.object(
                cli_module, "install", return_value=0
            ) as install_mock, mock.patch.object(
                cli_module, "cleanup_legacy_surfaces_after_plugin_upgrade", return_value=0
            ):
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "upgrade", "--ref", "v1.7.0", "--target", "codex", "--surface", "plugin", "--dry-run"],
                ), contextlib.redirect_stderr(stderr):
                    try:
                        exit_code = cli_module.main()
                    except SystemExit as exc:
                        self.fail(f"--surface should be accepted by upgrade parser: {exc}")

        self.assertEqual(exit_code, 0)
        options = install_mock.call_args.args[0]
        self.assertEqual(options.surface, "plugin")

    def test_upgrade_defaults_to_full_plugin_surface_and_migrates_old_surfaces(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            extracted_root = temp_root / "archive-root"
            scripts_dir = extracted_root / "scripts"
            scripts_dir.mkdir(parents=True)
            (scripts_dir / "bootstrap_qiongli.py").write_text("# stub\n", encoding="utf-8")

            with mock.patch.object(cli_module, "_resolve_upstream_repo", return_value=("owner/repo", "test")), mock.patch.object(
                cli_module, "_download"
            ), mock.patch.object(cli_module, "_extract_tarball", return_value=extracted_root), mock.patch.object(
                cli_module, "install", return_value=0
            ) as install_mock, mock.patch.object(
                cli_module, "cleanup_legacy_surfaces_after_plugin_upgrade", return_value=0
            ) as cleanup_mock:
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "upgrade", "--ref", "v1.8.0", "--target", "all", "--dry-run"],
                ):
                    exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        install_options = install_mock.call_args.args[0]
        self.assertEqual(install_options.profile, "full")
        self.assertEqual(install_options.surface, "plugin")
        self.assertEqual(install_options.target, "all")
        self.assertTrue(install_options.dry_run)
        cleanup_mock.assert_called_once()
        cleanup_options = cleanup_mock.call_args.args[0]
        self.assertEqual(cleanup_options.target, "all")
        self.assertTrue(cleanup_options.dry_run)

    def test_upgrade_refreshes_assets_without_self_update_runner(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            extracted_root = temp_root / "archive-root"
            scripts_dir = extracted_root / "scripts"
            scripts_dir.mkdir(parents=True)
            (scripts_dir / "bootstrap_qiongli.py").write_text("# stub\n", encoding="utf-8")

            with mock.patch.object(cli_module, "_resolve_upstream_repo", return_value=("owner/repo", "test")), mock.patch.object(
                cli_module, "_download"
            ), mock.patch.object(cli_module, "_extract_tarball", return_value=extracted_root), mock.patch.object(
                cli_module, "install", return_value=0
            ) as install_mock, mock.patch.object(
                cli_module, "execute_self_update", side_effect=AssertionError("upgrade must not update CLI package")
            ):
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "upgrade", "--ref", "v1.11.0", "--target", "all", "--dry-run"],
                ):
                    exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        self.assertEqual(install_mock.call_count, 1)
        install_options = install_mock.call_args.args[0]
        self.assertEqual(install_options.target, "all")
        self.assertTrue(install_options.dry_run)

    def test_upgrade_explicit_skills_surface_skips_plugin_migration(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            extracted_root = temp_root / "archive-root"
            scripts_dir = extracted_root / "scripts"
            scripts_dir.mkdir(parents=True)
            (scripts_dir / "bootstrap_qiongli.py").write_text("# stub\n", encoding="utf-8")

            with mock.patch.object(cli_module, "_resolve_upstream_repo", return_value=("owner/repo", "test")), mock.patch.object(
                cli_module, "_download"
            ), mock.patch.object(cli_module, "_extract_tarball", return_value=extracted_root), mock.patch.object(
                cli_module, "install", return_value=0
            ) as install_mock, mock.patch.object(
                cli_module, "cleanup_legacy_surfaces_after_plugin_upgrade", return_value=0
            ) as cleanup_mock:
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    [
                        "qiongli",
                        "upgrade",
                        "--ref",
                        "v1.8.0",
                        "--target",
                        "all",
                        "--profile",
                        "partial",
                        "--surface",
                        "skills",
                        "--dry-run",
                    ],
                ):
                    exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        install_options = install_mock.call_args.args[0]
        self.assertEqual(install_options.profile, "partial")
        self.assertEqual(install_options.surface, "skills")
        cleanup_mock.assert_not_called()

    def test_upgrade_failed_install_does_not_remove_old_surfaces(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            extracted_root = temp_root / "archive-root"
            scripts_dir = extracted_root / "scripts"
            scripts_dir.mkdir(parents=True)
            (scripts_dir / "bootstrap_qiongli.py").write_text("# stub\n", encoding="utf-8")

            with mock.patch.object(cli_module, "_resolve_upstream_repo", return_value=("owner/repo", "test")), mock.patch.object(
                cli_module, "_download"
            ), mock.patch.object(cli_module, "_extract_tarball", return_value=extracted_root), mock.patch.object(
                cli_module, "install", return_value=1
            ), mock.patch.object(
                cli_module, "cleanup_legacy_surfaces_after_plugin_upgrade", return_value=0
            ) as cleanup_mock:
                with mock.patch.object(
                    cli_module.sys,
                    "argv",
                    ["qiongli", "upgrade", "--ref", "v1.8.0", "--target", "all", "--dry-run"],
                ):
                    exit_code = cli_module.main()

        self.assertEqual(exit_code, 1)
        cleanup_mock.assert_not_called()

    def test_check_json_reports_installed_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            skill_dir = root / "codex-home" / "skills" / "qiongli-workflow"
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
            stdout = io.StringIO()
            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_pip_version", return_value=("9.9.9", "up-to-date")
            ), mock.patch.object(cli_module, "_check_system_env", return_value={}), mock.patch.object(
                cli_module.os,
                "environ",
                _isolated_qiongli_env(
                    root,
                    QIONGLI_CODEX_MARKETPLACE_PATH=str(root / "agents" / "plugins" / "marketplace.json"),
                    QIONGLI_CLAUDE_PLUGIN_PARENT=str(root / "claude-plugins"),
                    CLAUDE_CODE_CONFIG_PATH=str(root / "claude.json"),
                    ANTIGRAVITY_CONFIG_PATH=str(root / "antigravity-settings.json"),
                    HERMES_CONFIG_PATH=str(root / "hermes-settings.json"),
                ),
            ), mock.patch.object(
                cli_module, "_resolve_upstream_repo", return_value=(None, "")
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["installed"]["codex"]["surface"], "legacy_skill")
        self.assertTrue(payload["installed"]["codex"]["skill"]["installed"])
        self.assertEqual(payload["installed"]["codex"]["subject"], "economics")
        self.assertEqual(payload["installed"]["codex"]["coverage"], "focused")
        self.assertEqual(payload["installed"]["claude"]["subject"], None)
        self.assertEqual(payload["installed"]["claude"]["coverage"], None)

    def test_check_offline_skips_network_version_queries(self) -> None:
        args = argparse.Namespace(
            repo="owner/repo",
            json=True,
            strict_network=True,
            beta=False,
            offline=True,
        )
        stdout = io.StringIO()
        installed = {
            client: {"installed": False, "surface": "none"}
            for client in ("codex", "claude", "antigravity", "hermes")
        }

        with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
            cli_module, "_check_pip_version"
        ) as pip_mock, mock.patch.object(cli_module, "_resolve_upstream_repo") as upstream_mock, mock.patch.object(
            cli_module, "_check_system_env", return_value={}
        ), mock.patch.object(
            cli_module, "discover_install_surfaces", return_value=installed
        ), contextlib.redirect_stdout(stdout):
            exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        pip_mock.assert_not_called()
        upstream_mock.assert_not_called()
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["cli_package"]["status"], "skipped (offline)")
        self.assertEqual(payload["repo_source"], "offline")
        self.assertEqual(payload["latest_release"], "")

    def test_check_json_reports_codex_plugin_surface(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            marketplace = root / "agents" / "plugins" / "marketplace.json"
            plugin_root = marketplace.parent / "plugins" / "qiongli"
            skill_dir = plugin_root / "skills" / "qiongli-workflow"
            (plugin_root / ".codex-plugin").mkdir(parents=True)
            skill_dir.mkdir(parents=True)
            (plugin_root / ".codex-plugin" / "plugin.json").write_text(
                json.dumps({"name": "qiongli", "version": "9.9.9", "mcpServers": "./.mcp.json"}),
                encoding="utf-8",
            )
            (plugin_root / ".mcp.json").write_text(
                json.dumps(
                    {
                        "mcpServers": {
                            "qiongli": {
                                "command": "qiongli",
                                "args": ["mcp", "serve", "--transport", "stdio"],
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )
            (plugin_root / ".qiongli-managed.json").write_text(
                json.dumps(
                    {
                        "managed_by": "qiongli-cli",
                        "plugin": "qiongli",
                        "surface": "plugin",
                        "version": "9.9.9",
                    }
                ),
                encoding="utf-8",
            )
            (skill_dir / "VERSION").write_text("v9.9.9\n", encoding="utf-8")
            (skill_dir / "SUBJECT_MANIFEST.json").write_text(
                json.dumps({"subject": "economics", "coverage": "focused", "flavor": "full"}),
                encoding="utf-8",
            )
            args = argparse.Namespace(repo="", json=True, strict_network=False, beta=False)

            stdout = io.StringIO()
            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_pip_version", return_value=("9.9.9", "up-to-date")
            ), mock.patch.object(cli_module, "_check_system_env", return_value={}), mock.patch.object(
                cli_module, "_resolve_upstream_repo", return_value=(None, "")
            ), mock.patch.object(
                cli_module.os,
                "environ",
                _isolated_qiongli_env(
                    root,
                    CODEX_HOME=str(root / "codex-home"),
                    QIONGLI_CODEX_MARKETPLACE_PATH=str(marketplace),
                    QIONGLI_CLAUDE_PLUGIN_PARENT=str(root / "claude-plugins"),
                    CLAUDE_CODE_CONFIG_PATH=str(root / "claude.json"),
                    ANTIGRAVITY_CONFIG_PATH=str(root / "antigravity-settings.json"),
                    HERMES_CONFIG_PATH=str(root / "hermes-settings.json"),
                ),
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        codex = payload["installed"]["codex"]
        self.assertEqual(codex["surface"], "plugin")
        self.assertTrue(codex["installed"])
        self.assertEqual(codex["version"], "v9.9.9")
        self.assertEqual(codex["subject"], "economics")
        self.assertEqual(codex["coverage"], "focused")
        self.assertTrue(codex["plugin"]["installed"])
        self.assertEqual(codex["plugin"]["path"], str(plugin_root))
        self.assertTrue(codex["mcp"]["installed"])
        self.assertEqual(codex["mcp"]["source"], "plugin")
        self.assertEqual(codex["mcp"]["path"], str(plugin_root / ".mcp.json"))
        self.assertEqual(codex["mcp"]["server"], "qiongli")
        self.assertFalse(codex["standalone_mcp"]["installed"])
        self.assertEqual(codex["standalone_mcp"]["source"], "standalone")
        self.assertEqual(codex["standalone_mcp"]["path"], str(root / "codex-home" / "config.toml"))
        self.assertTrue(codex["plugin_mcp"]["installed"])
        self.assertEqual(codex["plugin_mcp"]["source"], "plugin")

    def test_check_json_reports_codex_standalone_mcp_when_no_plugin_mcp(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            codex_home = root / "codex-home"
            config_path = codex_home / "config.toml"
            config_path.parent.mkdir(parents=True)
            config_path.write_text(
                "\n".join(
                    [
                        "# >>> qiongli managed mcp >>>",
                        "[mcp_servers.qiongli]",
                        'command = "qiongli"',
                        'args = ["mcp", "serve", "--transport", "stdio"]',
                        "# <<< qiongli managed mcp <<<",
                        "",
                    ]
                ),
                encoding="utf-8",
            )
            args = argparse.Namespace(repo="", json=True, strict_network=False, beta=False, offline=True)
            stdout = io.StringIO()

            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_system_env", return_value={}
            ), mock.patch.object(
                cli_module.os,
                "environ",
                _isolated_qiongli_env(root, CODEX_HOME=str(codex_home)),
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        codex = payload["installed"]["codex"]
        self.assertEqual(codex["surface"], "mcp")
        self.assertTrue(codex["mcp"]["installed"])
        self.assertEqual(codex["mcp"]["source"], "standalone")
        self.assertFalse(codex["plugin_mcp"]["installed"])
        self.assertTrue(codex["standalone_mcp"]["installed"])

    def test_check_json_reports_codex_plugin_activation_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            marketplace = root / ".agents" / "plugins" / "marketplace.json"
            plugin_root = root / "plugins" / "qiongli"
            skill_dir = plugin_root / "skills" / "qiongli-workflow"
            (plugin_root / ".codex-plugin").mkdir(parents=True)
            skill_dir.mkdir(parents=True)
            marketplace.parent.mkdir(parents=True)
            marketplace.write_text(
                json.dumps({"name": "personal", "plugins": [{"name": "qiongli"}]}),
                encoding="utf-8",
            )
            (plugin_root / ".codex-plugin" / "plugin.json").write_text(
                json.dumps({"name": "qiongli", "version": "9.9.9", "mcpServers": "./.mcp.json"}),
                encoding="utf-8",
            )
            (plugin_root / ".qiongli-managed.json").write_text(
                json.dumps(
                    {
                        "managed_by": "qiongli-cli",
                        "plugin": "qiongli",
                        "surface": "plugin",
                        "version": "9.9.9",
                    }
                ),
                encoding="utf-8",
            )
            (skill_dir / "VERSION").write_text("v9.9.9\n", encoding="utf-8")
            args = argparse.Namespace(repo="", json=True, strict_network=False, beta=False, offline=True)
            codex_list_output = (
                "WARNING: proceeding, even though aliases could not be created\n"
                + json.dumps(
                    {
                        "installed": [
                            {
                                "pluginId": "qiongli@personal",
                                "name": "qiongli",
                                "marketplaceName": "personal",
                                "installed": True,
                                "enabled": True,
                            }
                        ]
                    }
                )
            )
            completed = mock.Mock(returncode=0, stdout=codex_list_output)

            stdout = io.StringIO()
            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_system_env", return_value={}
            ), mock.patch.object(
                cli_module.os,
                "environ",
                _isolated_qiongli_env(root, HOME=str(root), USERPROFILE=str(root)),
            ), mock.patch(
                "qiongli.install_discovery.shutil.which", return_value="/usr/local/bin/codex"
            ), mock.patch(
                "qiongli.install_discovery.subprocess.run", return_value=completed
            ) as run_mock, contextlib.redirect_stdout(
                stdout
            ):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        codex = payload["installed"]["codex"]
        self.assertEqual(codex["surface"], "plugin")
        self.assertTrue(codex["plugin"]["active"])
        self.assertTrue(codex["plugin"]["enabled"])
        self.assertEqual(codex["plugin"]["plugin_id"], "qiongli@personal")
        run_mock.assert_called_once()

    def test_check_json_reports_claude_plugin_activation_status(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            plugin_root = root / ".qiongli" / "plugins" / "claude-code" / "plugins" / "qiongli"
            skill_dir = plugin_root / "skills" / "qiongli-workflow"
            (plugin_root / ".claude-plugin").mkdir(parents=True)
            skill_dir.mkdir(parents=True)
            (plugin_root / ".claude-plugin" / "plugin.json").write_text(
                json.dumps({"name": "qiongli", "version": "9.9.9"}),
                encoding="utf-8",
            )
            (plugin_root / ".qiongli-managed.json").write_text(
                json.dumps(
                    {
                        "managed_by": "qiongli-cli",
                        "plugin": "qiongli",
                        "surface": "plugin",
                        "version": "9.9.9",
                    }
                ),
                encoding="utf-8",
            )
            (skill_dir / "VERSION").write_text("v9.9.9\n", encoding="utf-8")
            args = argparse.Namespace(repo="", json=True, strict_network=False, beta=False, offline=True)
            completed = mock.Mock(
                returncode=0,
                stdout=json.dumps(
                    [
                        {
                            "id": "qiongli@qiongli-local",
                            "version": "9.9.9",
                            "enabled": True,
                            "installPath": str(plugin_root),
                        }
                    ]
                ),
            )

            stdout = io.StringIO()
            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_system_env", return_value={}
            ), mock.patch.object(
                cli_module.os,
                "environ",
                _isolated_qiongli_env(root, CLAUDE_CODE_HOME=str(root / ".claude")),
            ), mock.patch(
                "qiongli.install_discovery.shutil.which", return_value="/usr/local/bin/claude"
            ), mock.patch(
                "qiongli.install_discovery.subprocess.run", return_value=completed
            ), contextlib.redirect_stdout(
                stdout
            ):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        claude = payload["installed"]["claude"]
        self.assertEqual(claude["surface"], "plugin")
        self.assertTrue(claude["plugin"]["active"])
        self.assertTrue(claude["plugin"]["enabled"])
        self.assertEqual(claude["plugin"]["plugin_id"], "qiongli@qiongli-local")

    def test_check_json_reports_antigravity_plugin_and_real_mcp_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            plugin_root = root / ".qiongli" / "plugins" / "antigravity" / "qiongli"
            skill_dir = plugin_root / "skills" / "qiongli-workflow"
            skill_dir.mkdir(parents=True)
            (plugin_root / "plugin.json").write_text(
                json.dumps({"name": "qiongli", "version": "9.9.9"}),
                encoding="utf-8",
            )
            (plugin_root / "mcp_config.json").write_text(
                json.dumps({"mcpServers": {"qiongli": {"command": "qiongli", "args": ["mcp", "serve", "--transport", "stdio"]}}}),
                encoding="utf-8",
            )
            (plugin_root / ".qiongli-managed.json").write_text(
                json.dumps(
                    {
                        "managed_by": "qiongli-cli",
                        "plugin": "qiongli",
                        "surface": "plugin",
                        "version": "9.9.9",
                    }
                ),
                encoding="utf-8",
            )
            (skill_dir / "VERSION").write_text("v9.9.9\n", encoding="utf-8")
            mcp_config = root / ".gemini" / "config" / "mcp_config.json"
            mcp_config.parent.mkdir(parents=True)
            mcp_config.write_text(
                json.dumps({"mcpServers": {"qiongli": {"command": "qiongli", "args": ["mcp", "serve", "--transport", "stdio"]}}}),
                encoding="utf-8",
            )
            args = argparse.Namespace(repo="", json=True, strict_network=False, beta=False, offline=True)
            completed = mock.Mock(returncode=0, stdout="qiongli enabled\n")

            stdout = io.StringIO()
            with mock.patch.object(cli_module, "_find_repo_root", return_value=None), mock.patch.object(
                cli_module, "_check_system_env", return_value={}
            ), mock.patch.object(
                cli_module.os,
                "environ",
                _isolated_qiongli_env(root, CLAUDE_CODE_HOME=str(root / ".claude")),
            ), mock.patch(
                "qiongli.install_discovery.shutil.which", return_value="/usr/local/bin/antigravity"
            ), mock.patch(
                "qiongli.install_discovery.subprocess.run", return_value=completed
            ), contextlib.redirect_stdout(
                stdout
            ):
                exit_code = cli_module.cmd_check(args)

        self.assertEqual(exit_code, 0)
        payload = json.loads(stdout.getvalue())
        antigravity = payload["installed"]["antigravity"]
        self.assertEqual(antigravity["surface"], "plugin")
        self.assertTrue(antigravity["plugin"]["active"])
        self.assertTrue(antigravity["mcp"]["installed"])
        self.assertEqual(antigravity["mcp"]["path"], str(plugin_root / "mcp_config.json"))
        self.assertEqual(antigravity["mcp"]["source"], "plugin")

    def test_doctor_summary_reports_codex_plugin_mcp_source(self) -> None:
        args = argparse.Namespace(cwd=".")
        completed = mock.Mock(returncode=0, stdout="doctor ok\n")
        installed = {
            client: {
                "installed": False,
                "surface": "none",
                "version": None,
                "path": f"/tmp/{client}/qiongli-workflow",
                "mcp": {"installed": False, "source": "standalone", "path": "", "server": ""},
            }
            for client in ("codex", "claude", "antigravity", "hermes")
        }
        installed["codex"] = {
            "installed": True,
            "surface": "plugin",
            "version": "v9.9.9",
            "path": "/tmp/plugins/qiongli",
            "mcp": {
                "installed": True,
                "source": "plugin",
                "path": "/tmp/plugins/qiongli/.mcp.json",
                "server": "qiongli",
            },
        }
        stdout = io.StringIO()

        with mock.patch.object(cli_module.subprocess, "run", return_value=completed), mock.patch.object(
            cli_module, "discover_install_surfaces", return_value=installed
        ), contextlib.redirect_stdout(stdout):
            exit_code = cli_module.cmd_doctor(args)

        self.assertEqual(exit_code, 0)
        output = stdout.getvalue()
        self.assertIn("- codex: installed, surface=plugin", output)
        self.assertIn("mcp=plugin:qiongli", output)

    def test_doctor_runs_orchestrator_subprocess(self) -> None:
        args = argparse.Namespace(cwd=".")
        completed = mock.Mock(returncode=0, stdout="doctor ok\n")
        installed = {
            client: {
                "installed": False,
                "surface": "none",
                "version": None,
                "path": f"/tmp/{client}/qiongli-workflow",
            }
            for client in ("codex", "claude", "antigravity", "hermes")
        }
        stdout = io.StringIO()

        with mock.patch.object(cli_module.subprocess, "run", return_value=completed) as run_mock, mock.patch.object(
            cli_module, "discover_install_surfaces", return_value=installed
        ), contextlib.redirect_stdout(stdout):
            exit_code = cli_module.cmd_doctor(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("Client Integration", stdout.getvalue())
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

    def test_project_set_subject_delegates_to_orchestrator(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            project_dir = Path(tmp_dir) / "project"
            completed = mock.Mock(returncode=0, stdout="project set-subject ok\n")
            args = argparse.Namespace(project_cmd="set-subject", project_dir=str(project_dir), subject="finance")

            with mock.patch("subprocess.run", return_value=completed) as run:
                exit_code = cli_module.cmd_project(args)

        self.assertEqual(exit_code, 0)
        command = run.call_args.args[0]
        self.assertEqual(
            command[-5:],
            ["set-subject", "--project-dir", str(project_dir.resolve()), "--subject", "finance"],
        )

    def test_project_status_delegates_to_orchestrator(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            completed = mock.Mock(returncode=0, stdout="project status ok\n")
            args = argparse.Namespace(project_cmd="status", project_dir=tmp_dir)

            with mock.patch("subprocess.run", return_value=completed) as run:
                exit_code = cli_module.cmd_project(args)

        self.assertEqual(exit_code, 0)
        self.assertIn("project", run.call_args.args[0])
        self.assertIn("status", run.call_args.args[0])

    def test_subject_status_json_reports_auto_without_creating_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            stdout = io.StringIO()

            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "status", "--cwd", str(root), "--json"],
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.main()

            payload = json.loads(stdout.getvalue())
            manifest_exists = (root / ".qiongli" / "guidance_manifest.yaml").exists()

        self.assertEqual(exit_code, 0)
        self.assertFalse(manifest_exists)
        self.assertFalse(payload["manifest_exists"])
        self.assertEqual(payload["manifest"]["active_subject"], "auto")
        self.assertEqual(payload["manifest"]["subject_mode"], "auto")

    def test_subject_confirm_json_updates_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            stdout = io.StringIO()

            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "confirm", "finance", "--cwd", str(root), "--json"],
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.main()

            payload = json.loads(stdout.getvalue())
            manifest_exists = (root / ".qiongli" / "guidance_manifest.yaml").is_file()

        self.assertEqual(exit_code, 0)
        self.assertTrue(manifest_exists)
        self.assertEqual(payload["manifest"]["active_subject"], "finance")
        self.assertEqual(payload["manifest"]["subject_mode"], "confirmed")

    def test_subject_dismiss_json_records_cli_source_without_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            stdout = io.StringIO()

            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "dismiss", "economics", "--cwd", str(root), "--json"],
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.main()

            payload = json.loads(stdout.getvalue())
            manifest_exists = (root / ".qiongli" / "guidance_manifest.yaml").exists()

        self.assertEqual(exit_code, 0)
        self.assertFalse(manifest_exists)
        self.assertFalse(payload["manifest_exists"])
        self.assertEqual(payload["state"]["dismissed_subjects"]["economics"]["source"], "cli")

    def test_subject_reset_json_returns_confirmed_subject_to_auto(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "confirm", "finance", "--cwd", str(root), "--json"],
            ), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(cli_module.main(), 0)

            stdout = io.StringIO()
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "reset", "--cwd", str(root), "--json"],
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.main()

            payload = json.loads(stdout.getvalue())

        self.assertEqual(exit_code, 0)
        self.assertEqual(payload["manifest"]["active_subject"], "auto")
        self.assertEqual(payload["manifest"]["subject_mode"], "auto")

    def test_subject_lock_then_unlock_json_returns_confirmed_subject(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "lock", "economics", "--cwd", str(root), "--json"],
            ), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(cli_module.main(), 0)

            stdout = io.StringIO()
            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "unlock", "--cwd", str(root), "--json"],
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.main()

            payload = json.loads(stdout.getvalue())

        self.assertEqual(exit_code, 0)
        self.assertEqual(payload["manifest"]["active_subject"], "economics")
        self.assertEqual(payload["manifest"]["subject_mode"], "confirmed")

    def test_subject_status_human_output_includes_subject_fields(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            stdout = io.StringIO()

            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "status", "--cwd", tmp_dir],
            ), contextlib.redirect_stdout(stdout):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 0)
        self.assertIn("active_subject:", stdout.getvalue())
        self.assertIn("subject_mode:", stdout.getvalue())

    def test_subject_status_reports_invalid_manifest_as_cli_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            manifest = root / ".qiongli" / "guidance_manifest.yaml"
            manifest.parent.mkdir(parents=True)
            manifest.write_text(
                "active_subject: unknown\nsubject_mode: confirmed\n",
                encoding="utf-8",
            )
            stderr = io.StringIO()

            with mock.patch.object(
                cli_module.sys,
                "argv",
                ["qiongli", "subject", "status", "--cwd", str(root)],
            ), contextlib.redirect_stderr(stderr):
                exit_code = cli_module.main()

        self.assertEqual(exit_code, 2)
        self.assertIn("qiongli subject:", stderr.getvalue())
        self.assertIn("Unsupported active_subject: unknown", stderr.getvalue())

    def test_subject_confirm_requires_subject_argument(self) -> None:
        stderr = io.StringIO()
        with mock.patch.object(
            cli_module.sys,
            "argv",
            ["qiongli", "subject", "confirm"],
        ), contextlib.redirect_stderr(stderr):
            with self.assertRaises(SystemExit) as raised:
                cli_module.main()

        self.assertEqual(raised.exception.code, 2)

    def test_provider_set_and_list_redacts_global_config(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            config_home = root / "config"
            env = _isolated_qiongli_env(root, QIONGLI_CONFIG_HOME=str(config_home))
            with mock.patch.dict(os.environ, env, clear=True):
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
            root = Path(tmp_dir)
            config_home = root / "config"
            env = _isolated_qiongli_env(root, QIONGLI_CONFIG_HOME=str(config_home))
            with mock.patch.dict(os.environ, env, clear=True):
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
