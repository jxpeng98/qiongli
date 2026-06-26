from __future__ import annotations

import os
import shlex
import shutil
import subprocess
import sys
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
    guidance: str = ""


CommandRunner = Callable[[Sequence[str]], int | subprocess.CompletedProcess[object]]


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

    if options.check:
        check_command = (qiongli_executable, "check", "--offline")

    return SelfUpdatePlan(
        channel=options.channel,
        install_channel=channel,
        package_command=package_command,
        refresh_command=refresh_command,
        check_command=check_command,
        guidance=guidance,
    )


def execute_self_update(
    options: SelfUpdateOptions,
    *,
    env: Mapping[str, str] | None = None,
    executable: str | None = None,
    runner: CommandRunner | None = None,
    output: TextIO | None = None,
) -> int:
    out = output or sys.stdout
    plan = build_self_update_plan(options, env=env, executable=executable)
    command_runner = runner or _default_runner

    _print_plan(plan, out)

    if options.dry_run:
        print("[dry-run] no commands were executed.", file=out)
        return 0

    if not plan.package_command:
        print(f"[error] {plan.guidance}", file=out)
        return 2

    if not options.yes:
        print("Rerun with --yes to execute these update commands.", file=out)
        return 0

    for command in _commands_to_run(plan):
        print(f"[run] {_format_command(command)}", file=out)
        exit_code = _runner_exit_code(command_runner(command))
        if exit_code != 0:
            print(f"[error] command failed with exit code {exit_code}: {_format_command(command)}", file=out)
            return exit_code

    print("[ok] qiongli self-update completed.", file=out)
    return 0


def _validate_options(options: SelfUpdateOptions) -> None:
    if options.channel not in CHANNEL_CHOICES:
        raise ValueError(f"Unsupported update channel: {options.channel}")
    if options.target not in TARGET_CHOICES:
        raise ValueError(f"Unsupported install target: {options.target}")
    if options.surface not in SURFACE_CHOICES:
        raise ValueError(f"Unsupported install surface: {options.surface}")
    if options.profile not in PROFILE_CHOICES:
        raise ValueError(f"Unsupported install profile: {options.profile}")


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
