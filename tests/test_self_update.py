from __future__ import annotations

import io
import unittest
from unittest import mock

from qiongli.self_update import (
    PackageUpdateStatus,
    SelfUpdateOptions,
    _default_update_checker,
    _pypi_latest_version,
    build_self_update_plan,
    execute_self_update,
)


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

    def test_execute_prompts_and_runs_package_update_when_confirmed(self) -> None:
        calls: list[tuple[str, ...]] = []
        prompts: list[tuple[str, bool]] = []
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(refresh=False),
            env={"QIONGLI_INSTALL_CHANNEL": "npm"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            confirmer=lambda prompt, default: prompts.append((prompt, default)) or True,
            update_checker=lambda plan: PackageUpdateStatus(
                installed_version="1.10.0",
                latest_version="1.11.0",
                update_available=True,
                detail="1.10.0 -> 1.11.0",
            ),
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [("npm", "install", "-g", "qiongli@latest")])
        self.assertEqual(len(prompts), 1)
        self.assertIn("Upgrade qiongli CLI/package", prompts[0][0])
        self.assertFalse(prompts[0][1])
        self.assertIn("[ok] qiongli self-update completed.", output.getvalue())

    def test_execute_exits_without_commands_when_package_update_declined(self) -> None:
        calls: list[tuple[str, ...]] = []
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(refresh=True),
            env={"QIONGLI_INSTALL_CHANNEL": "pipx"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            confirmer=lambda _prompt, _default: False,
            update_checker=lambda plan: PackageUpdateStatus(
                installed_version="1.10.0",
                latest_version="1.11.0",
                update_available=True,
                detail="1.10.0 -> 1.11.0",
            ),
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [])
        self.assertIn("Package update skipped.", output.getvalue())

    def test_execute_prompts_for_refresh_after_package_update(self) -> None:
        calls: list[tuple[str, ...]] = []
        prompts: list[tuple[str, bool]] = []

        def confirm(prompt: str, default: bool) -> bool:
            prompts.append((prompt, default))
            return len(prompts) == 1

        exit_code = execute_self_update(
            SelfUpdateOptions(refresh=True),
            env={"QIONGLI_INSTALL_CHANNEL": "pipx"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            confirmer=confirm,
            update_checker=lambda plan: PackageUpdateStatus(
                installed_version="1.10.0",
                latest_version="1.11.0",
                update_available=True,
                detail="1.10.0 -> 1.11.0",
            ),
            output=io.StringIO(),
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [("pipx", "upgrade", "qiongli")])
        self.assertEqual(len(prompts), 2)
        self.assertIn("Upgrade qiongli CLI/package", prompts[0][0])
        self.assertFalse(prompts[0][1])
        self.assertIn("Refresh installed local plugins/assets", prompts[1][0])
        self.assertTrue(prompts[1][1])

    def test_yes_runs_package_refresh_and_check_without_prompts(self) -> None:
        calls: list[tuple[str, ...]] = []
        prompts: list[str] = []

        exit_code = execute_self_update(
            SelfUpdateOptions(yes=True),
            env={"QIONGLI_INSTALL_CHANNEL": "pipx"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            confirmer=lambda prompt, _default: prompts.append(prompt) or False,
            update_checker=lambda plan: PackageUpdateStatus(
                installed_version="1.10.0",
                latest_version="1.11.0",
                update_available=True,
                detail="1.10.0 -> 1.11.0",
            ),
            output=io.StringIO(),
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(prompts, [])
        self.assertEqual(
            calls,
            [
                ("pipx", "upgrade", "qiongli"),
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
                ("qiongli-bin", "check", "--offline"),
            ],
        )

    def test_yes_no_refresh_runs_package_update_only(self) -> None:
        calls: list[tuple[str, ...]] = []

        exit_code = execute_self_update(
            SelfUpdateOptions(yes=True, refresh=False),
            env={"QIONGLI_INSTALL_CHANNEL": "pipx"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            update_checker=lambda plan: PackageUpdateStatus(
                installed_version="1.10.0",
                latest_version="1.11.0",
                update_available=True,
                detail="1.10.0 -> 1.11.0",
            ),
            output=io.StringIO(),
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [("pipx", "upgrade", "qiongli")])

    def test_execute_skips_when_package_is_already_current(self) -> None:
        calls: list[tuple[str, ...]] = []
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(yes=True),
            env={"QIONGLI_INSTALL_CHANNEL": "pip"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            update_checker=lambda plan: PackageUpdateStatus(
                installed_version="1.11.0",
                latest_version="1.11.0",
                update_available=False,
                detail="up-to-date",
            ),
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [])
        self.assertIn("qiongli CLI/package is already up to date.", output.getvalue())

    def test_execute_prompts_when_update_status_is_unknown(self) -> None:
        calls: list[tuple[str, ...]] = []
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(refresh=False),
            env={"QIONGLI_INSTALL_CHANNEL": "pip"},
            executable="qiongli-bin",
            python_executable="/python",
            runner=lambda command: calls.append(tuple(command)) or 0,
            confirmer=lambda prompt, default: "Unable to confirm" in prompt and default is False,
            update_checker=lambda plan: PackageUpdateStatus(
                installed_version="1.10.0",
                latest_version="",
                update_available=None,
                detail="network unavailable",
            ),
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [("/python", "-m", "pip", "install", "--upgrade", "qiongli")])
        self.assertIn("network unavailable", output.getvalue())

    def test_dry_run_prints_plan_without_running_commands(self) -> None:
        calls: list[tuple[str, ...]] = []
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(dry_run=True, channel="next"),
            env={"QIONGLI_INSTALL_CHANNEL": "npm"},
            executable="qiongli-bin",
            runner=lambda command: calls.append(tuple(command)) or 0,
            update_checker=lambda _plan: self.fail("dry-run should not check package versions"),
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [])
        self.assertIn("npm install -g qiongli@next", output.getvalue())

    def test_no_refresh_dry_run_prints_check_skipped(self) -> None:
        output = io.StringIO()

        exit_code = execute_self_update(
            SelfUpdateOptions(dry_run=True, refresh=False),
            env={"QIONGLI_INSTALL_CHANNEL": "npm"},
            executable="qiongli-bin",
            update_checker=lambda _plan: self.fail("dry-run should not check package versions"),
            output=output,
        )

        self.assertEqual(exit_code, 0)
        self.assertIn("- refresh installed surfaces: skipped", output.getvalue())
        self.assertIn("- post-update check: skipped", output.getvalue())
        self.assertNotIn("qiongli-bin check --offline", output.getvalue())

    def test_default_update_checker_detects_prerelease_to_final_update(self) -> None:
        plan = build_self_update_plan(
            SelfUpdateOptions(),
            env={"QIONGLI_INSTALL_CHANNEL": "pip"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        with mock.patch("qiongli.self_update._installed_package_version", return_value="1.2.0rc1"), mock.patch(
            "qiongli.self_update._pypi_latest_version", return_value="1.2.0"
        ):
            status = _default_update_checker(plan)

        self.assertTrue(status.update_available)
        self.assertEqual(status.installed_version, "1.2.0rc1")
        self.assertEqual(status.latest_version, "1.2.0")

    def test_default_update_checker_uses_npm_package_version_hint(self) -> None:
        plan = build_self_update_plan(
            SelfUpdateOptions(),
            env={"QIONGLI_INSTALL_CHANNEL": "npm", "QIONGLI_NPM_PACKAGE_VERSION": "1.10.0"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        with mock.patch(
            "qiongli.self_update._installed_package_version",
            side_effect=AssertionError("npm package version hint should avoid Python metadata lookup"),
        ), mock.patch("qiongli.self_update._npm_latest_version", return_value="1.11.0"):
            status = _default_update_checker(plan)

        self.assertEqual(status.installed_version, "1.10.0")
        self.assertEqual(status.latest_version, "1.11.0")
        self.assertTrue(status.update_available)

    def test_default_update_checker_accepts_npm_build_metadata_hint(self) -> None:
        plan = build_self_update_plan(
            SelfUpdateOptions(),
            env={"QIONGLI_INSTALL_CHANNEL": "npm", "QIONGLI_NPM_PACKAGE_VERSION": "1.10.0+build.1"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        with mock.patch(
            "qiongli.self_update._installed_package_version",
            side_effect=AssertionError("npm package version hint should avoid Python metadata lookup"),
        ), mock.patch("qiongli.self_update._npm_latest_version", return_value="1.11.0"):
            status = _default_update_checker(plan)

        self.assertEqual(status.installed_version, "1.10.0+build.1")
        self.assertEqual(status.latest_version, "1.11.0")
        self.assertTrue(status.update_available)

    def test_default_update_checker_parses_hyphenated_npm_prerelease_hint(self) -> None:
        plan = build_self_update_plan(
            SelfUpdateOptions(),
            env={"QIONGLI_INSTALL_CHANNEL": "npm", "QIONGLI_NPM_PACKAGE_VERSION": "1.10.0-beta.1+build.1"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        with mock.patch(
            "qiongli.self_update._installed_package_version",
            side_effect=AssertionError("npm package version hint should avoid Python metadata lookup"),
        ), mock.patch("qiongli.self_update._npm_latest_version", return_value="1.9.0"):
            status = _default_update_checker(plan)

        self.assertEqual(status.installed_version, "1.10.0-beta.1+build.1")
        self.assertEqual(status.latest_version, "1.9.0")
        self.assertIs(status.update_available, False)

    def test_default_update_checker_reports_unknown_for_unrecognized_npm_prerelease_hint(self) -> None:
        plan = build_self_update_plan(
            SelfUpdateOptions(),
            env={"QIONGLI_INSTALL_CHANNEL": "npm", "QIONGLI_NPM_PACKAGE_VERSION": "1.10.0-next.1"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        with mock.patch(
            "qiongli.self_update._installed_package_version",
            side_effect=AssertionError("npm package version hint should avoid Python metadata lookup"),
        ), mock.patch("qiongli.self_update._npm_latest_version", return_value="1.9.0"):
            status = _default_update_checker(plan)

        self.assertEqual(status.installed_version, "1.10.0-next.1")
        self.assertEqual(status.latest_version, "1.9.0")
        self.assertIsNone(status.update_available)

    def test_pypi_next_version_uses_newest_prerelease(self) -> None:
        payload = {
            "info": {"version": "1.2.0"},
            "releases": {
                "1.2.0": [],
                "1.3.0rc1": [],
                "1.3.0b2": [],
                "1.1.9": [],
            },
        }

        class FakeResponse:
            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

            def read(self):
                import json

                return json.dumps(payload).encode("utf-8")

        with mock.patch("qiongli.self_update.urllib.request.urlopen", return_value=FakeResponse()):
            latest = _pypi_latest_version("next")

        self.assertEqual(latest, "1.3.0rc1")

    def test_pypi_next_version_prefers_stable_over_stale_prerelease(self) -> None:
        payload = {
            "info": {"version": "2.0.0"},
            "releases": {
                "1.9.0rc1": [],
                "2.0.0": [],
            },
        }

        class FakeResponse:
            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

            def read(self):
                import json

                return json.dumps(payload).encode("utf-8")

        with mock.patch("qiongli.self_update.urllib.request.urlopen", return_value=FakeResponse()):
            latest = _pypi_latest_version("next")

        self.assertEqual(latest, "2.0.0")

    def test_pypi_next_version_includes_alpha_prereleases(self) -> None:
        payload = {
            "info": {"version": "1.2.0"},
            "releases": {
                "1.2.0": [],
                "1.3.0rc1": [],
                "1.4.0a1": [],
            },
        }

        class FakeResponse:
            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

            def read(self):
                import json

                return json.dumps(payload).encode("utf-8")

        with mock.patch("qiongli.self_update.urllib.request.urlopen", return_value=FakeResponse()):
            latest = _pypi_latest_version("next")

        self.assertEqual(latest, "1.4.0a1")

    def test_default_update_checker_next_detects_prerelease_update(self) -> None:
        plan = build_self_update_plan(
            SelfUpdateOptions(channel="next"),
            env={"QIONGLI_INSTALL_CHANNEL": "pip"},
            executable="qiongli-bin",
            python_executable="/python",
        )

        with mock.patch("qiongli.self_update._installed_package_version", return_value="1.2.0"), mock.patch(
            "qiongli.self_update._pypi_latest_version", return_value="1.3.0rc1"
        ):
            status = _default_update_checker(plan)

        self.assertTrue(status.update_available)
        self.assertEqual(status.latest_version, "1.3.0rc1")

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
