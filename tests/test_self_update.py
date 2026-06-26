from __future__ import annotations

import io
import unittest

from qiongli.self_update import SelfUpdateOptions, build_self_update_plan, execute_self_update


class SelfUpdateTests(unittest.TestCase):
    def test_npm_channel_uses_npm_install_and_refreshes_full_plugin_surface(self) -> None:
        options = SelfUpdateOptions(channel="stable")

        plan = build_self_update_plan(
            options,
            env={"QIONGLI_INSTALL_CHANNEL": "npm"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        self.assertEqual(plan.install_channel, "npm")
        self.assertEqual(plan.package_command, ("npm", "install", "-g", "qiongli@latest"))
        self.assertEqual(
            plan.refresh_command,
            (
                "qiongli-bin",
                "install",
                "--target",
                "all",
                "--surface",
                "plugin",
                "--profile",
                "full",
                "--overwrite",
            ),
        )
        self.assertEqual(plan.check_command, ("qiongli-bin", "check", "--offline"))

    def test_next_channel_allows_prerelease_for_python_package_managers(self) -> None:
        options = SelfUpdateOptions(channel="next")

        pip_plan = build_self_update_plan(
            options,
            env={"QIONGLI_INSTALL_CHANNEL": "pip"},
            executable="qiongli-bin",
            python_executable="/python",
        )
        pipx_plan = build_self_update_plan(
            options,
            env={"QIONGLI_INSTALL_CHANNEL": "pipx"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        self.assertEqual(
            pip_plan.package_command,
            ("/python", "-m", "pip", "install", "--upgrade", "--pre", "qiongli"),
        )
        self.assertEqual(pipx_plan.package_command, ("pipx", "upgrade", "qiongli", "--pip-args", "--pre"))

    def test_execute_requires_yes_before_running_real_update(self) -> None:
        calls: list[tuple[str, ...]] = []
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(),
            env={"QIONGLI_INSTALL_CHANNEL": "npm"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [])
        self.assertIn("--yes", output.getvalue())

    def test_execute_runs_package_refresh_and_check_when_confirmed(self) -> None:
        calls: list[tuple[str, ...]] = []

        exit_code = execute_self_update(
            SelfUpdateOptions(yes=True, target="claude", surface="plugin", profile="full"),
            env={"QIONGLI_INSTALL_CHANNEL": "pipx"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            output=io.StringIO(),
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(
            calls,
            [
                ("pipx", "upgrade", "qiongli"),
                (
                    "qiongli-bin",
                    "install",
                    "--target",
                    "claude",
                    "--surface",
                    "plugin",
                    "--profile",
                    "full",
                    "--overwrite",
                ),
                ("qiongli-bin", "check", "--offline"),
            ],
        )

    def test_dry_run_prints_plan_without_running_commands(self) -> None:
        calls: list[tuple[str, ...]] = []
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(dry_run=True, channel="next"),
            env={"QIONGLI_INSTALL_CHANNEL": "npm"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [])
        self.assertIn("npm install -g qiongli@next", output.getvalue())

    def test_source_checkout_reports_manual_update_guidance(self) -> None:
        plan = build_self_update_plan(
            SelfUpdateOptions(),
            env={"QIONGLI_INSTALL_CHANNEL": "source"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        self.assertEqual(plan.install_channel, "source")
        self.assertEqual(plan.package_command, ())
        self.assertIn("git pull", plan.guidance)


if __name__ == "__main__":
    unittest.main()
