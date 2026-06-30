# Qiongli Update Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `qiongli update` / `qiongli self-update` an interactive two-step package update flow, keep `install` behavior unchanged, and document `upgrade` as content-only refresh.

**Architecture:** Keep the package-manager update logic in `qiongli.self_update`, because that module already owns install-channel detection and package update commands. Add a small testable package-version check and confirmation layer around the existing command runner. Keep `upgrade` implementation mostly unchanged and clarify its role through help text and docs: it refreshes local content/assets and does not update the installed npm/pipx/pip CLI package.

**Tech Stack:** Python 3.12 stdlib (`argparse`, `dataclasses`, `subprocess`, `urllib`, `importlib.metadata`), existing `unittest` tests, npm launcher tests with `node:test`.

---

## Scope Decisions

- `qiongli install` remains the low-level content install/refresh command and keeps its existing defaults.
- `qiongli update` and `qiongli self-update` become the normal user-facing CLI/package update path.
- `qiongli update` without `--yes` becomes interactive.
- `qiongli update --yes` remains non-interactive for CI/scripts and answers both update prompts as yes.
- `qiongli update --dry-run` prints the package update and refresh plan and runs nothing.
- `qiongli update --no-refresh` skips the second prompt and does not refresh local assets after package update.
- `qiongli self-update --target`, `--surface`, and `--profile` are removed from public help/docs but remain accepted as hidden compatibility options for one release. If a user passes non-default values, print a deprecation warning telling them to run `qiongli install ...` separately.
- `qiongli upgrade` does not update the CLI package. It remains a content-only refresh path. In Python CLI it may download a GitHub ref and install it; in the npm launcher it remains an install alias that refreshes current bundled content.

## File Structure

- Modify `packages/python-qiongli/src/qiongli/self_update.py`
  - Add `PackageUpdateStatus`.
  - Add default package-version checking helpers.
  - Add injectable confirmation and update-check functions to `execute_self_update`.
  - Change `execute_self_update` from “requires `--yes` before running” to “interactive unless `--yes`”.
  - Keep `build_self_update_plan` producing default refresh command.

- Modify `packages/python-qiongli/src/qiongli/cli.py`
  - Hide `self-update` `--target`, `--surface`, and `--profile` from public help with `argparse.SUPPRESS`.
  - Keep compatibility parsing for those options.
  - Update `self-update` help text to describe interactive package update and optional refresh.
  - Update `upgrade` help text to say content-only refresh and no CLI package update.

- Modify `tests/test_self_update.py`
  - Replace the “requires `--yes`” expectation with interactive prompt tests.
  - Add decline-package, accept-package-decline-refresh, accept-both, no-update, and unknown-check tests.

- Modify `tests/test_cli.py`
  - Update parser dispatch tests for hidden compatibility options.
  - Add a help-output assertion that `self-update` no longer advertises `--target`, `--surface`, or `--profile`.

- Modify `packages/npm-qiongli/test/args.test.mjs` and `packages/npm-qiongli/test/cli.test.mjs`
  - Keep delegation to Python CLI for `self-update` / `update`.
  - Add a small test that npm `upgrade` remains content-only install alias and does not delegate to Python package update.

- Modify docs:
  - `README.md`
  - `README_CN.md`
  - `packages/npm-qiongli/README.md`
  - `docs/reference/cli.md`
  - `docs/zh/reference/cli.md`
  - `docs/guide/install.md`
  - `docs/zh/guide/install.md`
  - `docs/guide/upgrade.md`
  - `docs/zh/guide/upgrade.md`

## Task 1: Add Interactive Self-Update Tests

**Files:**
- Modify: `tests/test_self_update.py`
- Test: `tests/test_self_update.py`

- [ ] **Step 1: Replace the old `--yes` gate test with an interactive acceptance test**

Replace `test_execute_requires_yes_before_running_real_update` with:

```python
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
```

- [ ] **Step 2: Add the import for the new status dataclass**

Change:

```python
from qiongli.self_update import SelfUpdateOptions, build_self_update_plan, execute_self_update
```

to:

```python
from qiongli.self_update import (
    PackageUpdateStatus,
    SelfUpdateOptions,
    build_self_update_plan,
    execute_self_update,
)
```

- [ ] **Step 3: Add a decline-package test**

Add after the interactive acceptance test:

```python
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
```

- [ ] **Step 4: Add a second-step refresh decline test**

Add:

```python
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
```

- [ ] **Step 5: Update the `--yes` test to use default refresh semantics**

Replace `test_execute_runs_package_refresh_and_check_when_confirmed` with:

```python
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
```

- [ ] **Step 6: Add no-update and unknown-check tests**

Add:

```python
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
```

In this test, pass `python_executable="/python"` through `build_self_update_plan` by adding a new optional `python_executable` parameter to `execute_self_update` in Task 2.

- [ ] **Step 7: Run the tests and verify RED**

Run:

```bash
python3 -m unittest tests.test_self_update
```

Expected: failures mentioning `PackageUpdateStatus` import and unsupported `confirmer`, `update_checker`, and `python_executable` arguments.

## Task 2: Implement Interactive Package Update Flow

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/self_update.py`
- Test: `tests/test_self_update.py`

- [ ] **Step 1: Add package status and injectable callables**

In `self_update.py`, add below `SelfUpdatePlan`:

```python
@dataclass(frozen=True)
class PackageUpdateStatus:
    installed_version: str
    latest_version: str
    update_available: bool | None
    detail: str = ""


ConfirmFn = Callable[[str, bool], bool]
UpdateChecker = Callable[[SelfUpdatePlan], PackageUpdateStatus]
```

- [ ] **Step 2: Extend `execute_self_update` signature**

Change the signature to:

```python
def execute_self_update(
    options: SelfUpdateOptions,
    *,
    env: Mapping[str, str] | None = None,
    executable: str | None = None,
    python_executable: str | None = None,
    runner: CommandRunner | None = None,
    confirmer: ConfirmFn | None = None,
    update_checker: UpdateChecker | None = None,
    output: TextIO | None = None,
) -> int:
```

Change the plan build line to:

```python
    plan = build_self_update_plan(options, env=env, executable=executable, python_executable=python_executable)
```

- [ ] **Step 3: Replace the old `--yes` gate in `execute_self_update`**

Replace the block:

```python
    if not options.yes:
        print("Rerun with --yes to execute these update commands.", file=out)
        return 0

    for command in _commands_to_run(plan):
        print(f"[run] {_format_command(command)}", file=out)
        exit_code = _runner_exit_code(command_runner(command))
        if exit_code != 0:
            print(f"[error] command failed with exit code {exit_code}: {_format_command(command)}", file=out)
            return exit_code
```

with:

```python
    status = (update_checker or _default_update_checker)(plan)
    _print_update_status(status, out)

    if status.update_available is False:
        print("qiongli CLI/package is already up to date.", file=out)
        return 0

    if not options.yes:
        confirm = confirmer or _confirm
        if status.update_available is True:
            package_prompt = "Upgrade qiongli CLI/package now?"
        else:
            package_prompt = "Unable to confirm the latest qiongli package version. Run package update anyway?"
        if not confirm(package_prompt, False):
            print("Package update skipped.", file=out)
            return 0

    print(f"[run] {_format_command(plan.package_command)}", file=out)
    exit_code = _runner_exit_code(command_runner(plan.package_command))
    if exit_code != 0:
        print(f"[error] command failed with exit code {exit_code}: {_format_command(plan.package_command)}", file=out)
        return exit_code

    if plan.refresh_command:
        refresh = options.yes or (confirmer or _confirm)("Refresh installed local plugins/assets from the new package?", True)
        if refresh:
            print(f"[run] {_format_command(plan.refresh_command)}", file=out)
            exit_code = _runner_exit_code(command_runner(plan.refresh_command))
            if exit_code != 0:
                print(f"[error] command failed with exit code {exit_code}: {_format_command(plan.refresh_command)}", file=out)
                return exit_code
        else:
            print("Installed local plugins/assets refresh skipped.", file=out)

    if plan.check_command and (options.yes or refresh):
        print(f"[run] {_format_command(plan.check_command)}", file=out)
        exit_code = _runner_exit_code(command_runner(plan.check_command))
        if exit_code != 0:
            print(f"[error] command failed with exit code {exit_code}: {_format_command(plan.check_command)}", file=out)
            return exit_code
```

- [ ] **Step 4: Initialize `refresh` before use**

Immediately before the `if plan.refresh_command:` block add:

```python
    refresh = False
```

- [ ] **Step 5: Add confirmation helper**

Add below `_print_plan`:

```python
def _confirm(prompt: str, default: bool) -> bool:
    suffix = "[Y/n]" if default else "[y/N]"
    try:
        answer = input(f"{prompt} {suffix} ").strip().lower()
    except EOFError:
        return default
    if not answer:
        return default
    return answer in {"y", "yes"}
```

- [ ] **Step 6: Add update status printing**

Add:

```python
def _print_update_status(status: PackageUpdateStatus, output: TextIO) -> None:
    if status.latest_version:
        print(f"- package latest: {status.latest_version}", file=output)
    if status.installed_version:
        print(f"- package installed: {status.installed_version}", file=output)
    if status.detail:
        print(f"- package status: {status.detail}", file=output)
```

- [ ] **Step 7: Add default update checker skeleton**

Add:

```python
def _default_update_checker(plan: SelfUpdatePlan) -> PackageUpdateStatus:
    installed = _installed_package_version()
    if plan.install_channel in {"source", "unknown"}:
        return PackageUpdateStatus(installed, "", None, plan.guidance or "package update status unavailable")
    if plan.install_channel == "npm":
        latest = _npm_latest_version(plan.channel)
    else:
        latest = _pypi_latest_version(plan.channel)
    installed_parsed = _parse_version_tuple(installed)
    latest_parsed = _parse_version_tuple(latest)
    if installed_parsed is None or latest_parsed is None:
        return PackageUpdateStatus(installed, latest, None, "unable to compare package versions")
    if latest_parsed > installed_parsed:
        return PackageUpdateStatus(installed, latest, True, f"{installed} -> {latest}")
    return PackageUpdateStatus(installed, latest, False, "up-to-date")
```

- [ ] **Step 8: Add version helper functions**

Add below `_default_update_checker`:

```python
def _installed_package_version() -> str:
    try:
        return metadata.version("qiongli")
    except metadata.PackageNotFoundError:
        return ""


def _npm_latest_version(channel: str) -> str:
    package = "qiongli@next" if channel == "next" else "qiongli@latest"
    result = subprocess.run(
        ["npm", "view", package, "version"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError((result.stderr or result.stdout or "npm view failed").strip())
    return result.stdout.strip().splitlines()[-1].strip()


def _pypi_latest_version(channel: str) -> str:
    import json
    import urllib.request

    with urllib.request.urlopen("https://pypi.org/pypi/qiongli/json", timeout=8) as response:
        payload = json.loads(response.read().decode("utf-8"))
    info = payload.get("info", {})
    latest = str(info.get("version", "")).strip()
    if not latest:
        raise RuntimeError("PyPI response did not include a version")
    return latest
```

For the first implementation, `channel == "next"` on PyPI still uses the PyPI latest metadata endpoint. This is acceptable because `--channel next` package installation still uses `--pre`; the checker only decides whether to ask before running the package-manager command. If this behavior becomes confusing, add a PyPI prerelease release-list parser in a separate task.

- [ ] **Step 9: Add version tuple parser**

Add:

```python
def _parse_version_tuple(value: str) -> tuple[int, int, int, int] | None:
    raw = value.strip().removeprefix("v")
    if not raw:
        return None
    beta = 10**9
    if "b" in raw:
        raw, beta_raw = raw.split("b", 1)
        try:
            beta = int(beta_raw)
        except ValueError:
            return None
    elif "-beta." in raw:
        raw, beta_raw = raw.split("-beta.", 1)
        try:
            beta = int(beta_raw)
        except ValueError:
            return None
    parts = raw.split(".")
    if len(parts) != 3:
        return None
    try:
        major, minor, patch = (int(part) for part in parts)
    except ValueError:
        return None
    return (major, minor, patch, beta)
```

- [ ] **Step 10: Run the self-update tests**

Run:

```bash
python3 -m unittest tests.test_self_update
```

Expected: all tests in `tests.test_self_update` pass.

## Task 3: Simplify Python CLI Public Options While Preserving Compatibility

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `tests/test_cli.py`
- Test: `tests/test_cli.py`

- [ ] **Step 1: Update the parser dispatch test**

In `test_self_update_dispatches_to_update_runner`, keep passing legacy options, but add this assertion:

```python
        self.assertEqual(options.target, "claude")
        self.assertEqual(options.surface, "both")
        self.assertEqual(options.profile, "full")
```

Keep existing assertions for `--no-refresh`, `--skip-check`, `--dry-run`, and `--yes`. This proves hidden compatibility still works.

- [ ] **Step 2: Add a help test proving advanced install-shape options are not advertised**

Add to `tests/test_cli.py`:

```python
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
```

- [ ] **Step 3: Run the CLI tests and verify RED**

Run:

```bash
python3 -m unittest tests.test_cli.CliTests.test_self_update_help_hides_install_shape_options
```

Expected: FAIL because help currently includes `--target`, `--surface`, and `--profile`.

- [ ] **Step 4: Hide compatibility options in `cli.py`**

In the `self_update` parser block, change the three options:

```python
    self_update.add_argument(
        "--target",
        default="all",
        choices=TARGET_CHOICES,
        help="Installed client target to refresh after updating (default: all)",
    )
    self_update.add_argument(
        "--surface",
        choices=SURFACE_CHOICES,
        default=DEFAULT_CLI_SURFACE,
        help="Installed output surface to refresh after updating (default: plugin)",
    )
    self_update.add_argument(
        "--profile",
        choices=PROFILE_CHOICES,
        default=DEFAULT_CLI_PROFILE,
        help="Install profile to refresh after updating (default: full)",
    )
```

to:

```python
    self_update.add_argument("--target", default="all", choices=TARGET_CHOICES, help=argparse.SUPPRESS)
    self_update.add_argument("--surface", choices=SURFACE_CHOICES, default=DEFAULT_CLI_SURFACE, help=argparse.SUPPRESS)
    self_update.add_argument("--profile", choices=PROFILE_CHOICES, default=DEFAULT_CLI_PROFILE, help=argparse.SUPPRESS)
```

- [ ] **Step 5: Update public help strings**

Change the `self-update` parser help from:

```python
help="Update the qiongli CLI package, then refresh installed plugin/MCP surfaces",
```

to:

```python
help="Interactively update the qiongli CLI package, then optionally refresh installed local assets",
```

Change the `upgrade` parser help from:

```python
upgrade = subparsers.add_parser("upgrade", help="Download release archive and run installer with overwrite")
```

to:

```python
upgrade = subparsers.add_parser("upgrade", help="Refresh local assets from a release archive without updating the CLI package")
```

- [ ] **Step 6: Add deprecated option warning**

In `cmd_self_update` if there is a helper, or in the `main()` dispatch just before `return cmd_self_update(args)`, add:

```python
        _warn_deprecated_self_update_install_options(args)
```

Add this helper near `_effective_cli_surface` helpers:

```python
def _warn_deprecated_self_update_install_options(args: argparse.Namespace) -> None:
    if getattr(args, "cmd", "") not in {"self-update", "update"}:
        return
    changed = []
    if getattr(args, "target", "all") != "all":
        changed.append("--target")
    if getattr(args, "surface", DEFAULT_CLI_SURFACE) != DEFAULT_CLI_SURFACE:
        changed.append("--surface")
    if getattr(args, "profile", DEFAULT_CLI_PROFILE) != DEFAULT_CLI_PROFILE:
        changed.append("--profile")
    if changed:
        print(
            "[warn] "
            + ", ".join(changed)
            + " on qiongli update/self-update is deprecated; run `qiongli update --no-refresh`, then `qiongli install ...` for custom refresh options.",
            file=sys.stderr,
        )
```

- [ ] **Step 7: Add warning assertion**

In `test_self_update_dispatches_to_update_runner`, wrap stderr:

```python
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            ...
        self.assertIn("deprecated", stderr.getvalue())
```

- [ ] **Step 8: Run focused CLI tests**

Run:

```bash
python3 -m unittest tests.test_cli.CliTests.test_self_update_dispatches_to_update_runner tests.test_cli.CliTests.test_self_update_help_hides_install_shape_options tests.test_cli.CliTests.test_update_alias_dispatches_to_self_update_runner
```

Expected: all three tests pass.

## Task 4: Keep Npm Wrapper Semantics Explicit

**Files:**
- Modify: `packages/npm-qiongli/test/args.test.mjs`
- Modify: `packages/npm-qiongli/test/cli.test.mjs`
- Test: `packages/npm-qiongli/test/*.mjs`

- [ ] **Step 1: Rename npm upgrade parser test to content-only language**

In `packages/npm-qiongli/test/args.test.mjs`, change:

```javascript
test('parseArgv treats upgrade as install with overwrite', () => {
```

to:

```javascript
test('parseArgv treats upgrade as content-only install refresh with overwrite', () => {
```

- [ ] **Step 2: Add npm CLI test that `upgrade` does not delegate to Python self-update**

Add to `packages/npm-qiongli/test/cli.test.mjs`:

```javascript
test('main treats npm upgrade as content-only install refresh', async () => {
  const pythonCalls = [];
  const exitCode = await main(['upgrade', '--target', 'codex', '--dry-run'], {
    stdout: { write: () => {} },
    stderr: { write: () => {} },
    runPythonCliCommand: ({ args }) => {
      pythonCalls.push(args);
      return 9;
    },
  });

  assert.equal(exitCode, 0);
  assert.deepEqual(pythonCalls, []);
});
```

- [ ] **Step 3: Run npm tests**

Run:

```bash
npm --prefix packages/npm-qiongli test
```

Expected: npm package tests pass.

## Task 5: Update Documentation Semantics

**Files:**
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `packages/npm-qiongli/README.md`
- Modify: `docs/reference/cli.md`
- Modify: `docs/zh/reference/cli.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`
- Modify: `docs/guide/upgrade.md`
- Modify: `docs/zh/guide/upgrade.md`
- Test: `tests/test_cli_setup_docs.py` and text search

- [ ] **Step 1: Update English quick command examples**

Replace examples that show only:

```markdown
qiongli self-update --dry-run
qiongli self-update --yes
```

with:

```markdown
qiongli update
qiongli update --dry-run
qiongli update --yes
```

- [ ] **Step 2: Update English `self-update` prose**

Replace prose equivalent to:

```markdown
Use `qiongli self-update --dry-run` to preview the detected package-manager update command. `qiongli self-update --yes` updates npm/pipx/pip first, then runs `qiongli install --target all --surface plugin --profile full --overwrite` and `qiongli check --offline` from the refreshed package.
```

with:

```markdown
Use `qiongli update` for the normal interactive update flow. It checks whether the installed qiongli CLI/package has a newer release, asks before running the package-manager update, then asks whether to refresh installed local plugins/assets from the new package. Use `qiongli update --yes` for CI or scripts; it answers both prompts as yes. Use `qiongli update --no-refresh` when you only want the CLI/package update and plan to run `qiongli install ...` yourself.
```

- [ ] **Step 3: Update English upgrade prose**

Add this paragraph to `docs/reference/cli.md` under `qiongli upgrade` and mirror it in `docs/guide/upgrade.md`:

```markdown
`qiongli upgrade` is a content/assets refresh command. It does not update the installed npm, pipx, or pip qiongli CLI package. Use it when you intentionally want to refresh local installed assets from the current package or from a selected upstream release archive. For normal package updates, use `qiongli update`.
```

- [ ] **Step 4: Update Chinese self-update prose**

Use this Chinese text in `README_CN.md`, `docs/zh/reference/cli.md`, and `docs/zh/guide/install.md`:

```markdown
普通升级使用 `qiongli update`。它会先检查当前安装的 qiongli CLI/package 是否有新版本；如有，会询问是否升级。CLI/package 升级成功后，它会再询问是否用新 package 内的 payload 刷新本地 plugin/assets。脚本或 CI 使用 `qiongli update --yes`，它会把两个确认都视为 yes。只想升级 CLI/package、不刷新本地内容时使用 `qiongli update --no-refresh`。
```

- [ ] **Step 5: Update Chinese upgrade prose**

Use:

```markdown
`qiongli upgrade` 是内容/assets 刷新命令，不会升级 npm、pipx 或 pip 中安装的 qiongli CLI package。需要只刷新本地安装内容，或从指定上游 release archive 刷新内容时使用它。普通 package 升级使用 `qiongli update`。
```

- [ ] **Step 6: Update CLI usage blocks**

In `docs/reference/cli.md` and `docs/zh/reference/cli.md`, replace:

```markdown
qiongli self-update [--channel stable|next] [--target codex|claude|antigravity|hermes|all] [--surface skills|plugin|both] [--profile partial|full] [--dry-run] [--yes]
```

with:

```markdown
qiongli update [--channel stable|next] [--dry-run] [--yes] [--no-refresh] [--skip-check]
qiongli self-update [--channel stable|next] [--dry-run] [--yes] [--no-refresh] [--skip-check]
```

- [ ] **Step 7: Run docs-focused checks**

Run:

```bash
python3 -m unittest tests.test_cli_setup_docs
rg -n "self-update \\[--channel stable\\|next\\].*--target|qiongli self-update --yes updates npm/pipx/pip first|qiongli self-update --yes 会先更新" README.md README_CN.md docs packages/npm-qiongli/README.md
```

Expected: unittest passes; `rg` returns no matches.

## Task 6: Verify Upgrade Does Not Invoke Package Update

**Files:**
- Modify: `tests/test_cli.py`
- Test: `tests/test_cli.py`

- [ ] **Step 1: Add explicit Python CLI upgrade boundary test**

Add to `tests/test_cli.py`:

```python
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
```

- [ ] **Step 2: Run focused upgrade tests**

Run:

```bash
python3 -m unittest tests.test_cli.CliTests.test_upgrade_refreshes_assets_without_self_update_runner tests.test_cli.CliTests.test_upgrade_defaults_to_full_plugin_surface_and_migrates_old_surfaces tests.test_cli.CliTests.test_upgrade_failed_install_does_not_remove_old_surfaces
```

Expected: all focused upgrade tests pass.

## Task 7: Full Verification

**Files:**
- No code changes.
- Test: repository test commands.

- [ ] **Step 1: Run Python focused tests**

Run:

```bash
python3 -m unittest tests.test_self_update tests.test_cli tests.test_cli_setup_docs
```

Expected: all tests pass.

- [ ] **Step 2: Run npm package tests**

Run:

```bash
npm --prefix packages/npm-qiongli test
```

Expected: all npm tests pass.

- [ ] **Step 3: Run CLI help smoke checks**

Run:

```bash
python3 -m qiongli.cli self-update --help
python3 -m qiongli.cli upgrade --help
```

Expected:
- `self-update --help` shows `--channel`, `--no-refresh`, `--skip-check`, `--dry-run`, and `--yes`.
- `self-update --help` does not show `--target`, `--surface`, or `--profile`.
- `upgrade --help` says it refreshes local assets and does not update the CLI package.

- [ ] **Step 4: Run dry-run smoke checks**

Run:

```bash
QIONGLI_INSTALL_CHANNEL=pip python3 -m qiongli.cli update --dry-run
python3 -m qiongli.cli upgrade --ref v1.11.0 --target all --dry-run
```

Expected:
- `update --dry-run` prints package update, refresh, and check commands without executing them.
- `upgrade --dry-run` resolves the archive path and runs installer dry-run logic only; it does not call `pip`, `pipx`, or `npm install -g`.

- [ ] **Step 5: Inspect diff for boundary issues**

Run:

```bash
git diff --stat
git diff -- packages/python-qiongli/src/qiongli/self_update.py packages/python-qiongli/src/qiongli/cli.py tests/test_self_update.py tests/test_cli.py packages/npm-qiongli/test/args.test.mjs packages/npm-qiongli/test/cli.test.mjs
```

Expected:
- No generated payload directories are modified.
- No marketplace plugin files are changed.
- No secrets, local absolute paths, or machine-specific paths are introduced.

## Self-Review

- Spec coverage: The plan covers interactive `update`, non-interactive `--yes`, `--no-refresh`, unchanged `install`, content-only `upgrade`, npm wrapper behavior, docs, and verification.
- Placeholder scan: No placeholder marker or undefined implementation step remains.
- Type consistency: `PackageUpdateStatus`, `ConfirmFn`, and `UpdateChecker` are introduced before later tasks use them. `execute_self_update` signature matches the tests.
- Scope check: This is one bounded CLI semantics change. It does not include renaming `upgrade` or adding a new `refresh` command; that can be a separate compatibility project.
