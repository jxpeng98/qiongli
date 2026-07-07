from __future__ import annotations

import os
import shlex
import shutil
import subprocess
import sys
import json
import urllib.request
from dataclasses import dataclass
from importlib import metadata
from pathlib import Path
from typing import Callable, Mapping, Sequence, TextIO

from .universal_installer import PROFILE_CHOICES, SURFACE_CHOICES, TARGET_CHOICES


CHANNEL_CHOICES = ("stable", "next")
INSTALL_CHANNELS = ("npm", "pipx", "pip", "source", "unknown")
INSTALL_CHANNEL_ALIASES = {
    "node": "npm",
    "python": "pip",
    "venv": "pip",
    "editable": "source",
    "git": "source",
}


@dataclass(frozen=True)
class SelfUpdateOptions:
    channel: str = "stable"
    target: str = "all"
    surface: str = "plugin"
    profile: str = "full"
    refresh: bool = True
    check: bool = True
    dry_run: bool = False
    yes: bool = False


@dataclass(frozen=True)
class SelfUpdatePlan:
    channel: str
    install_channel: str
    package_command: tuple[str, ...]
    refresh_command: tuple[str, ...]
    check_command: tuple[str, ...]
    installed_version_hint: str = ""
    guidance: str = ""


@dataclass(frozen=True)
class PackageUpdateStatus:
    installed_version: str
    latest_version: str
    update_available: bool | None
    detail: str = ""


CommandRunner = Callable[[Sequence[str]], int | subprocess.CompletedProcess[object]]
ConfirmFn = Callable[[str, bool], bool]
UpdateChecker = Callable[[SelfUpdatePlan], PackageUpdateStatus]
SelfUpdateExecutor = Callable[[SelfUpdateOptions], int]


def build_self_update_plan(
    options: SelfUpdateOptions,
    *,
    env: Mapping[str, str] | None = None,
    executable: str | None = None,
    python_executable: str | None = None,
) -> SelfUpdatePlan:
    _validate_options(options)
    effective_env = os.environ if env is None else env
    channel = _detect_install_channel(effective_env)
    qiongli_executable = executable or shutil.which("qiongli") or "qiongli"
    python = python_executable or sys.executable

    package_command = _package_update_command(channel, options.channel, python)
    guidance = _manual_guidance(channel, options.channel)
    installed_version_hint = ""
    if channel == "npm":
        installed_version_hint = effective_env.get("QIONGLI_NPM_PACKAGE_VERSION", "").strip()
    refresh_command: tuple[str, ...] = ()
    check_command: tuple[str, ...] = ()

    if options.refresh:
        refresh_command = (
            qiongli_executable,
            "install",
            "--target",
            options.target,
            "--surface",
            options.surface,
            "--profile",
            options.profile,
            "--overwrite",
        )

    if options.check and options.refresh:
        check_command = (qiongli_executable, "check")

    return SelfUpdatePlan(
        channel=options.channel,
        install_channel=channel,
        package_command=package_command,
        refresh_command=refresh_command,
        check_command=check_command,
        installed_version_hint=installed_version_hint,
        guidance=guidance,
    )


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
    out = output or sys.stdout
    plan = build_self_update_plan(options, env=env, executable=executable, python_executable=python_executable)
    command_runner = runner or _default_runner
    confirm = confirmer or _confirm

    _print_plan(plan, out)

    if options.dry_run:
        print("[dry-run] no commands were executed.", file=out)
        return 0

    if not plan.package_command:
        print(f"[error] {plan.guidance}", file=out)
        return 2

    status = (update_checker or _default_update_checker)(plan)
    _print_update_status(status, out)

    if status.update_available is False:
        print("qiongli CLI/package is already up to date.", file=out)
        return 0

    if not options.yes:
        if status.update_available is True:
            prompt = "Upgrade qiongli CLI/package?"
        else:
            prompt = "Unable to confirm qiongli CLI/package update status. Upgrade anyway?"
        if not confirm(prompt, False):
            print("Package update skipped.", file=out)
            return 0

    exit_code = _run_command(plan.package_command, command_runner, out)
    if exit_code != 0:
        return exit_code

    refresh_accepted = False
    if plan.refresh_command:
        refresh_accepted = options.yes or confirm("Refresh installed local plugins/assets from the new package?", True)
        if refresh_accepted:
            exit_code = _run_command(plan.refresh_command, command_runner, out)
            if exit_code != 0:
                return exit_code
        else:
            print("Installed local plugins/assets refresh skipped.", file=out)

    should_check = bool(plan.check_command) and bool(plan.refresh_command) and (options.yes or refresh_accepted)
    if should_check:
        exit_code = _run_command(plan.check_command, command_runner, out)
        if exit_code != 0:
            return exit_code

    print("[ok] qiongli self-update completed.", file=out)
    return 0


def run_self_update_wizard(
    *,
    input_fn: Callable[[str], str] = input,
    output: TextIO | None = None,
    executor: SelfUpdateExecutor | None = None,
) -> int:
    out = output or sys.stdout
    print("Qiongli self-update wizard", file=out)
    channel_choice = _choose(
        "Update channel",
        ("stable", "beta"),
        default="stable",
        input_fn=input_fn,
        output=out,
        note="Beta uses the next/prerelease package channel.",
    )
    target = _choose(
        "Refresh target",
        TARGET_CHOICES,
        default="auto",
        input_fn=input_fn,
        output=out,
        note="Auto refreshes only client CLIs detected on PATH.",
    )
    surface = _choose(
        "Install surface",
        SURFACE_CHOICES,
        default="plugin",
        input_fn=input_fn,
        output=out,
        note="Plugin is the recommended local runtime surface; skills keeps legacy skill-directory installs.",
    )
    profile = _choose(
        "Install profile",
        PROFILE_CHOICES,
        default="full",
        input_fn=input_fn,
        output=out,
        note="Full refreshes plugin/MCP-capable local assets; partial limits the local asset refresh.",
    )
    refresh = _choose_yes_no(
        "Refresh installed local plugins/assets after package update?",
        default=True,
        input_fn=input_fn,
        output=out,
    )
    check = False
    if refresh:
        check = _choose_yes_no(
            "Run qiongli check after refresh?",
            default=True,
            input_fn=input_fn,
            output=out,
        )
    yes = _choose_yes_no(
        "Run package update without additional confirmation prompts?",
        default=False,
        input_fn=input_fn,
        output=out,
    )
    options = SelfUpdateOptions(
        channel="next" if channel_choice == "beta" else "stable",
        target=target,
        surface=surface,
        profile=profile,
        refresh=refresh,
        check=check,
        yes=yes,
    )
    if executor is not None:
        return executor(options)
    return execute_self_update(options, output=out)


def _validate_options(options: SelfUpdateOptions) -> None:
    if options.channel not in CHANNEL_CHOICES:
        raise ValueError(f"Unsupported update channel: {options.channel}")
    if options.target not in TARGET_CHOICES:
        raise ValueError(f"Unsupported install target: {options.target}")
    if options.surface not in SURFACE_CHOICES:
        raise ValueError(f"Unsupported install surface: {options.surface}")
    if options.profile not in PROFILE_CHOICES:
        raise ValueError(f"Unsupported install profile: {options.profile}")


def _choose(
    label: str,
    choices: tuple[str, ...],
    *,
    default: str,
    input_fn: Callable[[str], str],
    output: TextIO,
    note: str = "",
) -> str:
    if note:
        print(f"Tip: {note}", file=output)
    default_index = choices.index(default) + 1
    while True:
        print(f"{label}:", file=output)
        for index, choice in enumerate(choices, start=1):
            suffix = " (default)" if choice == default else ""
            print(f"  {index}. {choice}{suffix}", file=output)
        raw = input_fn(f"Choose {label} [{default_index}]: ").strip()
        if not raw:
            return default
        if raw.isdigit():
            index = int(raw)
            if 1 <= index <= len(choices):
                return choices[index - 1]
        normalized = raw.lower()
        if normalized in choices:
            return normalized
        print(f"Please choose one of: {', '.join(choices)}", file=output)


def _choose_yes_no(
    label: str,
    *,
    default: bool,
    input_fn: Callable[[str], str],
    output: TextIO,
) -> bool:
    suffix = "Y/n" if default else "y/N"
    while True:
        raw = input_fn(f"{label} [{suffix}]: ").strip().lower()
        if not raw:
            return default
        if raw in {"y", "yes"}:
            return True
        if raw in {"n", "no"}:
            return False
        print("Please answer yes or no.", file=output)


def _detect_install_channel(env: Mapping[str, str]) -> str:
    raw = (
        env.get("QIONGLI_INSTALL_CHANNEL", "")
        or env.get("QIONGLI_INSTALL_METHOD", "")
        or env.get("QIONGLI_PACKAGE_MANAGER", "")
    ).strip().lower()
    if raw:
        normalized = INSTALL_CHANNEL_ALIASES.get(raw, raw)
        if normalized in INSTALL_CHANNELS:
            return normalized

    module_path = Path(__file__).resolve()
    parts = {part.lower() for part in module_path.parts}
    path_text = str(module_path).lower()
    if "pipx" in parts or "/pipx/venvs/qiongli/" in path_text:
        return "pipx"
    if _looks_like_source_checkout(module_path):
        return "source"
    if sys.prefix != getattr(sys, "base_prefix", sys.prefix):
        return "pip"
    try:
        metadata.version("qiongli")
    except metadata.PackageNotFoundError:
        return "unknown"
    return "pip"


def _looks_like_source_checkout(path: Path) -> bool:
    for parent in (path.parent, *path.parents):
        if (parent / ".git").exists() and (parent / "standards" / "research-workflow-contract.yaml").exists():
            return True
    return False


def _package_update_command(install_channel: str, channel: str, python_executable: str) -> tuple[str, ...]:
    if install_channel == "npm":
        package = "qiongli@next" if channel == "next" else "qiongli@latest"
        return ("npm", "install", "-g", package)
    if install_channel == "pipx":
        command = ["pipx", "upgrade", "qiongli"]
        if channel == "next":
            command.extend(["--pip-args", "--pre"])
        return tuple(command)
    if install_channel == "pip":
        command = [python_executable, "-m", "pip", "install", "--upgrade"]
        if channel == "next":
            command.append("--pre")
        command.append("qiongli")
        return tuple(command)
    return ()


def _manual_guidance(install_channel: str, channel: str) -> str:
    if install_channel == "source":
        branch_hint = "main" if channel == "stable" else "your prerelease branch"
        return f"Source checkout detected; update with `git pull` on {branch_hint}, then run `qiongli install --overwrite`."
    if install_channel == "unknown":
        return "Install channel could not be detected; run your package manager update command manually."
    return ""


def _commands_to_run(plan: SelfUpdatePlan) -> tuple[tuple[str, ...], ...]:
    return tuple(command for command in (plan.package_command, plan.refresh_command, plan.check_command) if command)


def _run_command(command: Sequence[str], command_runner: CommandRunner, output: TextIO) -> int:
    print(f"[run] {_format_command(command)}", file=output)
    exit_code = _runner_exit_code(command_runner(command))
    if exit_code != 0:
        print(f"[error] command failed with exit code {exit_code}: {_format_command(command)}", file=output)
    return exit_code


def _print_plan(plan: SelfUpdatePlan, output: TextIO) -> None:
    print("Qiongli self-update", file=output)
    print(f"- channel: {plan.channel}", file=output)
    print(f"- install channel: {plan.install_channel}", file=output)
    if plan.package_command:
        print(f"- package update: {_format_command(plan.package_command)}", file=output)
    elif plan.guidance:
        print(f"- package update: {plan.guidance}", file=output)
    else:
        print("- package update: unavailable", file=output)
    if plan.refresh_command:
        print(f"- refresh installed surfaces: {_format_command(plan.refresh_command)}", file=output)
    else:
        print("- refresh installed surfaces: skipped", file=output)
    if plan.check_command:
        print(f"- post-update check: {_format_command(plan.check_command)}", file=output)
    else:
        print("- post-update check: skipped", file=output)


def _format_command(command: Sequence[str]) -> str:
    return shlex.join(tuple(command))


def _default_runner(command: Sequence[str]) -> subprocess.CompletedProcess[object]:
    return subprocess.run(list(command), check=False)


def _runner_exit_code(result: int | subprocess.CompletedProcess[object]) -> int:
    if isinstance(result, int):
        return result
    return int(result.returncode)


def _confirm(prompt: str, default: bool) -> bool:
    suffix = "[Y/n]" if default else "[y/N]"
    try:
        answer = input(f"{prompt} {suffix} ").strip().lower()
    except EOFError:
        return default
    if not answer:
        return default
    return answer in {"y", "yes"}


def _print_update_status(status: PackageUpdateStatus, output: TextIO) -> None:
    if status.installed_version:
        print(f"- installed package version: {status.installed_version}", file=output)
    if status.latest_version:
        print(f"- latest package version: {status.latest_version}", file=output)
    if status.update_available is True:
        print("- package update status: update available", file=output)
    elif status.update_available is False:
        print("- package update status: current", file=output)
    else:
        print("- package update status: unknown", file=output)
    if status.detail:
        print(f"- package update detail: {status.detail}", file=output)


def _default_update_checker(plan: SelfUpdatePlan) -> PackageUpdateStatus:
    if not plan.package_command:
        return PackageUpdateStatus(
            installed_version="",
            latest_version="",
            update_available=None,
            detail=plan.guidance or "Package update command is unavailable.",
        )

    try:
        installed_version = plan.installed_version_hint or _installed_package_version()
        if plan.install_channel == "npm":
            latest_version = _npm_latest_version(plan.channel)
        elif plan.install_channel in {"pip", "pipx"}:
            latest_version = _pypi_latest_version(plan.channel)
        else:
            return PackageUpdateStatus(
                installed_version=installed_version,
                latest_version="",
                update_available=None,
                detail=plan.guidance or f"Cannot check latest version for install channel {plan.install_channel}.",
            )
    except Exception as exc:
        return PackageUpdateStatus(
            installed_version="",
            latest_version="",
            update_available=None,
            detail=f"Unable to check latest package version: {exc}",
        )

    installed_tuple = _parse_version_tuple(installed_version)
    latest_tuple = _parse_version_tuple(latest_version)
    if installed_tuple and latest_tuple:
        update_available: bool | None = installed_tuple < latest_tuple
    else:
        update_available = None

    return PackageUpdateStatus(
        installed_version=installed_version,
        latest_version=latest_version,
        update_available=update_available,
        detail=f"{installed_version} -> {latest_version}" if update_available else "",
    )


def _installed_package_version() -> str:
    return metadata.version("qiongli")


def _npm_latest_version(channel: str) -> str:
    tag = "next" if channel == "next" else "latest"
    result = subprocess.run(
        ["npm", "view", "qiongli", f"dist-tags.{tag}", "--json"],
        check=True,
        capture_output=True,
        text=True,
    )
    raw = result.stdout.strip()
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError:
        parsed = raw
    return str(parsed).strip()


def _pypi_latest_version(channel: str) -> str:
    with urllib.request.urlopen("https://pypi.org/pypi/qiongli/json", timeout=10) as response:
        payload = json.load(response)

    info_version = str(payload["info"]["version"])
    if channel != "next":
        return info_version

    candidate_versions = [
        str(version)
        for version in (info_version, *payload.get("releases", {}))
        if _parse_version_tuple(str(version))
        and (str(version) == info_version or _is_prerelease_version(str(version)))
    ]
    if not candidate_versions:
        return info_version
    return max(candidate_versions, key=_parse_version_tuple)


def _is_prerelease_version(version: str) -> bool:
    raw = version.strip().lower()
    if raw.startswith("v"):
        raw = raw[1:]

    _, separator, suffix = raw.partition("-")
    if not separator:
        suffix = ""
        for part in raw.split("."):
            digits = ""
            for char in part:
                if not char.isdigit():
                    suffix = part[len(digits) :]
                    break
                digits += char
            if suffix:
                break

    suffix = suffix.lstrip(".-_")
    return suffix.startswith(("a", "alpha", "rc", "b", "beta"))


def _parse_version_tuple(version: str) -> tuple[int, ...]:
    raw = version.strip().lower()
    if raw.startswith("v"):
        raw = raw[1:]
    raw = raw.split("+", 1)[0]
    if not raw:
        return ()

    numeric_text, separator, prerelease_text = raw.partition("-")
    numeric_parts: list[int] = []
    inline_suffix = ""

    for part in numeric_text.split("."):
        digits = ""
        for char in part:
            if not char.isdigit():
                inline_suffix = part[len(digits) :]
                break
            digits += char
        if not digits:
            return ()
        numeric_parts.append(int(digits))
        if inline_suffix:
            break

    if len(numeric_parts) > 3:
        return ()

    major, minor, patch = (*numeric_parts, 0, 0)[:3]
    suffix = prerelease_text if separator else inline_suffix
    if not suffix:
        return (major, minor, patch, 3, 0)

    suffix = suffix.lstrip(".-_")
    prerelease_rank = {
        "a": 0,
        "alpha": 0,
        "b": 1,
        "beta": 1,
        "rc": 2,
    }
    for label, rank in prerelease_rank.items():
        if suffix == label:
            return (major, minor, patch, rank, 0)
        for separator in ("", ".", "-", "_"):
            prefix = f"{label}{separator}"
            if suffix.startswith(prefix):
                number_text = suffix[len(prefix) :]
                if number_text.isdigit():
                    return (major, minor, patch, rank, int(number_text))
    return ()
