from __future__ import annotations

import sys
from argparse import Namespace
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Callable, TextIO

from .universal_installer import InstallOptions


OPERATION_CHOICES = ("install", "upgrade")
RUNTIME_CHOICES = (
    "multi-platform",
    "cli",
    "codex",
    "claude-code",
    "antigravity",
    "hermes",
)
CLIENT_TARGET_BY_RUNTIME = {
    "multi-platform": "all",
    "cli": "all",
    "codex": "codex",
    "claude-code": "claude",
    "antigravity": "antigravity",
    "hermes": "hermes",
}
SUBJECT_CHOICES = (
    "core",
    "economics",
    "accounting",
    "business",
    "finance",
    "political-economy",
    "geoeconomics",
    "economics-accounting",
)
COVERAGE_CHOICES = ("complete", "focused")
MODE_CHOICES = ("copy", "link")
SCOPE_CHOICES = ("all", "globals", "project", "cli")
REF_SOURCE_CHOICES = ("latest-stable", "latest-beta", "specific-tag", "branch")
PROVIDER_MODE_CHOICES = ("page", "prompt", "skip")

PROVIDER_PROMPTS = (
    ("openalex", "api_key", "OpenAlex API key"),
    ("openalex", "email", "OpenAlex email"),
    ("semantic_scholar", "api_key", "Semantic Scholar API key"),
    ("crossref", "email", "Crossref email"),
    ("pubmed", "api_key", "PubMed API key"),
)
PROVIDER_FIELD_NOTES = {
    ("openalex", "api_key"): "OpenAlex requires a free API key from openalex.org/settings/api for local provider calls.",
    ("openalex", "email"): "OpenAlex email is optional contact metadata included as mailto; leave empty to skip.",
    ("semantic_scholar", "api_key"): "Semantic Scholar works without a key, but a key improves reliability and rate limits.",
    ("crossref", "email"): "Crossref uses an email address for polite API usage; leave empty to skip.",
    ("pubmed", "api_key"): "PubMed API keys are optional and can be added later with qiongli provider setup.",
}


InputFn = Callable[[str], str]
InstallFn = Callable[[object], int | None]
UpgradeFn = Callable[[object], int | None]
ProviderSetFn = Callable[[str, str, str], object]
ProviderPageFn = Callable[[], object]
DoctorFn = Callable[[Path], int | None]


@dataclass(frozen=True)
class ProviderValue:
    provider: str
    field: str
    value: str

    def __repr__(self) -> str:
        status = "configured" if self.value.strip() else "skipped"
        return f"ProviderValue(provider={self.provider!r}, field={self.field!r}, status={status!r})"


@dataclass(frozen=True)
class SetupAnswers:
    operation: str = "install"
    runtime: str = "multi-platform"
    subject: str = "core"
    coverage: str = "complete"
    mode: str = "copy"
    scope: str = "all"
    project_dir: Path = field(default_factory=Path.cwd)
    overwrite: bool = False
    cli_dir: Path | None = None
    repo: str = ""
    ref: str = ""
    ref_type: str = "tag"
    beta: bool = False
    configure_providers: bool = False
    provider_mode: str = "page"
    provider_open_browser: bool = True
    provider_host: str = "127.0.0.1"
    provider_port: int = 0
    provider_values: tuple[ProviderValue, ...] = ()
    run_doctor: bool = True


@dataclass(frozen=True)
class SetupPlan:
    operation: str
    runtime: str
    client_target: str
    subject: str
    coverage: str
    mode: str
    scope: str
    parts: tuple[str, ...]
    project_dir: Path
    overwrite: bool
    cli_dir: Path | None
    repo: str
    ref: str
    ref_type: str
    beta: bool
    doctor: bool
    configure_providers: bool
    provider_mode: str
    provider_open_browser: bool
    provider_host: str
    provider_port: int
    install_command: tuple[str, ...]
    install_options: object
    upgrade_options: object
    provider_actions: tuple[ProviderValue, ...]
    actions: tuple[str, ...]


@dataclass(frozen=True)
class SetupResult:
    plan: SetupPlan
    dry_run: bool
    executed: bool
    planned_actions: tuple[str, ...]
    provider_status: dict[str, str]
    install_status: str = "planned"
    doctor_status: str = "skipped"


def collect_setup_answers(
    *,
    input_fn: InputFn = input,
    output: TextIO = sys.stdout,
    default_project_dir: Path | str | None = None,
    force_no_doctor: bool = False,
    provider_mode: str = "page",
    provider_open_browser: bool = True,
    provider_host: str = "127.0.0.1",
    provider_port: int = 0,
) -> SetupAnswers:
    project_default = Path(default_project_dir).expanduser().resolve() if default_project_dir else Path.cwd()
    provider_mode = _normalize_provider_mode(provider_mode)

    print("Qiongli setup", file=output)
    operation = _choose(
        "Setup path",
        OPERATION_CHOICES,
        default="install",
        input_fn=input_fn,
        output=output,
        note="Choose whether setup should install bundled assets or upgrade from upstream.",
    )
    runtime = _choose(
        "Runtime/client surface",
        RUNTIME_CHOICES,
        default="multi-platform",
        input_fn=input_fn,
        output=output,
        note="This decides which AI client skill target receives the workflow assets.",
    )
    subject = _choose(
        "Subject",
        SUBJECT_CHOICES,
        default="core",
        input_fn=input_fn,
        output=output,
        note="Subject packages add domain-specific guidance while keeping the core workflow.",
    )
    coverage = _choose(
        "Coverage",
        COVERAGE_CHOICES,
        default="complete",
        input_fn=input_fn,
        output=output,
        note="Complete is the full workflow; focused is the slimmer package shape.",
    )
    mode = _choose(
        "Install mode",
        MODE_CHOICES,
        default="copy",
        input_fn=input_fn,
        output=output,
        note="Copy is stable for most users; link is useful when developing from a checkout.",
    )
    scope = _choose(
        "Install scope",
        SCOPE_CHOICES,
        default="all",
        input_fn=input_fn,
        output=output,
        note="Scope controls whether setup writes global skills, project files, shell CLI wrappers, or all of them.",
    )
    project_dir = _ask_project_dir(
        project_default,
        input_fn=input_fn,
        output=output,
        note="Project directory is used when project-local assets or doctor checks are enabled.",
    )
    cli_dir = _ask_optional_path(
        "Shell CLI directory",
        input_fn=input_fn,
        output=output,
        note="Leave empty to use the default shell CLI directory.",
    ) if _scope_includes_cli(scope) else None
    overwrite = _choose_yes_no(
        "Overwrite existing installs?",
        default=operation == "upgrade",
        input_fn=input_fn,
        output=output,
        note="Upgrade normally overwrites managed Qiongli installs; fresh install leaves existing files alone by default.",
    )
    repo = ""
    ref = ""
    ref_type = "tag"
    beta = False
    if operation == "upgrade":
        repo = _ask_optional_text(
            "Upstream repo",
            input_fn=input_fn,
            output=output,
            note="Leave empty to use QIONGLI_REPO, project config, or the repository default.",
        )
        ref_source = _choose(
            "Upgrade source",
            REF_SOURCE_CHOICES,
            default="latest-stable",
            input_fn=input_fn,
            output=output,
            note="Pick latest stable for normal upgrades, latest beta for prerelease testing, or pin a tag/branch.",
        )
        if ref_source == "latest-beta":
            beta = True
        elif ref_source == "specific-tag":
            ref_type = "tag"
            ref = _ask_optional_text(
                "Release tag",
                input_fn=input_fn,
                output=output,
                note="Enter a tag such as v0.14.0; empty falls back to latest stable.",
            )
        elif ref_source == "branch":
            ref_type = "branch"
            ref = _ask_optional_text(
                "Branch name",
                input_fn=input_fn,
                output=output,
                note="Use this for testing a branch archive instead of a release tag.",
            )
    configure_providers = False
    provider_values: tuple[ProviderValue, ...] = ()
    if provider_mode != "skip":
        configure_providers = _choose_yes_no(
            "Configure literature provider keys now?",
            default=False,
            input_fn=input_fn,
            output=output,
            note="Provider keys enable configured scholarly search clients without storing secrets in research artifacts.",
        )
        if configure_providers and provider_mode == "prompt":
            provider_values = _collect_provider_values(input_fn=input_fn, output=output)
        elif configure_providers:
            _explain(
                "Provider setup opens one local browser page for all keys, with links for getting each provider key.",
                output=output,
            )
    run_doctor = False if force_no_doctor else _choose_yes_no(
        "Run doctor after setup?",
        default=True,
        input_fn=input_fn,
        output=output,
        note="Doctor checks the local environment after setup so missing CLIs or provider config are visible.",
    )

    answers = SetupAnswers(
        operation=operation,
        runtime=runtime,
        subject=subject,
        coverage=coverage,
        mode=mode,
        scope=scope,
        project_dir=project_dir,
        overwrite=overwrite,
        cli_dir=cli_dir,
        repo=repo,
        ref=ref,
        ref_type=ref_type,
        beta=beta,
        configure_providers=configure_providers,
        provider_mode=provider_mode,
        provider_open_browser=provider_open_browser,
        provider_host=provider_host,
        provider_port=provider_port,
        provider_values=provider_values,
        run_doctor=run_doctor,
    )
    _print_answer_summary(answers, output=output)
    return answers


def build_setup_plan(answers: SetupAnswers) -> SetupPlan:
    project_dir = Path(answers.project_dir).expanduser().resolve()
    client_target = _client_target_for_runtime(answers.runtime)
    parts = _parts_for_scope(answers.scope, answers.run_doctor)
    cli_dir = Path(answers.cli_dir).expanduser().resolve() if answers.cli_dir else None
    command: tuple[str, ...] = (
        "qiongli",
        answers.operation,
    )
    if answers.operation == "upgrade":
        command = _append_optional(command, "--repo", answers.repo)
        command = _append_optional(command, "--ref", answers.ref)
        if answers.ref:
            command = (*command, "--ref-type", answers.ref_type)
        if answers.beta:
            command = (*command, "--beta")
    command = (
        *command,
        "--mode",
        answers.mode,
        "--subject",
        answers.subject,
        "--coverage",
        answers.coverage,
        "--target",
        client_target,
        "--project-dir",
        str(project_dir),
        "--parts",
        ",".join(parts),
    )
    if answers.operation == "install" and answers.overwrite:
        command = (*command, "--overwrite")
    if answers.operation == "upgrade" and not answers.overwrite:
        command = (*command, "--no-overwrite")
    if cli_dir is not None:
        command = (*command, "--cli-dir", str(cli_dir))
    if answers.operation == "upgrade" and answers.run_doctor:
        command = (*command, "--doctor")

    provider_actions = tuple(value for value in answers.provider_values if value.value.strip())
    actions = [
        (
            f"{answers.operation} subject={answers.subject} coverage={answers.coverage} "
            f"target={client_target} parts={','.join(parts)}"
        ),
        *(f"configure {value.provider} {value.field}" for value in provider_actions),
    ]
    if answers.run_doctor:
        actions.append("run doctor")
    install_options = InstallOptions(
        repo_root=_packaged_payload_root(),
        project_dir=project_dir,
        subject=answers.subject,
        coverage=answers.coverage,
        target=client_target,
        mode=answers.mode,
        overwrite=answers.overwrite,
        install_cli="cli" in parts,
        cli_dir=cli_dir,
        doctor=answers.run_doctor,
        dry_run=False,
        profile="full",
        parts=parts,
    )
    upgrade_options = Namespace(
        repo=answers.repo,
        ref=answers.ref or None,
        ref_type=answers.ref_type,
        target=client_target,
        beta=answers.beta,
        mode=answers.mode,
        project_dir=str(project_dir),
        overwrite=answers.overwrite,
        doctor=answers.run_doctor,
        dry_run=False,
        parts=",".join(parts),
        subject=answers.subject,
        coverage=answers.coverage,
        install_cli="cli" in parts,
        no_cli=False,
        cli_dir=str(cli_dir) if cli_dir is not None else None,
    )
    return SetupPlan(
        operation=answers.operation,
        runtime=answers.runtime,
        client_target=client_target,
        subject=answers.subject,
        coverage=answers.coverage,
        mode=answers.mode,
        scope=answers.scope,
        parts=parts,
        project_dir=project_dir,
        overwrite=answers.overwrite,
        cli_dir=cli_dir,
        repo=answers.repo,
        ref=answers.ref,
        ref_type=answers.ref_type,
        beta=answers.beta,
        doctor=answers.run_doctor,
        configure_providers=answers.configure_providers,
        provider_mode=answers.provider_mode,
        provider_open_browser=answers.provider_open_browser,
        provider_host=answers.provider_host,
        provider_port=answers.provider_port,
        install_command=command,
        install_options=install_options,
        upgrade_options=upgrade_options,
        provider_actions=provider_actions,
        actions=tuple(actions),
    )


def execute_setup_plan(
    plan: SetupPlan,
    *,
    dry_run: bool = False,
    install_fn: InstallFn | None = None,
    upgrade_fn: UpgradeFn | None = None,
    provider_set_fn: ProviderSetFn | None = None,
    provider_page_fn: ProviderPageFn | None = None,
    doctor_fn: DoctorFn | None = None,
    output: TextIO = sys.stdout,
) -> SetupResult:
    provider_status = _provider_status(plan.provider_actions)
    _print_plan_summary(plan, provider_status=provider_status, dry_run=dry_run, output=output)

    if dry_run:
        return SetupResult(
            plan=plan,
            dry_run=True,
            executed=False,
            planned_actions=plan.actions,
            provider_status=provider_status,
            install_status="planned",
            doctor_status="planned" if plan.doctor else "skipped",
        )

    actual_provider_set_fn = provider_set_fn or _default_provider_set
    actual_provider_page_fn = provider_page_fn or (
        lambda: _default_provider_page(
            host=plan.provider_host,
            port=plan.provider_port,
            open_browser=plan.provider_open_browser,
            output=output,
        )
    )
    if plan.operation == "upgrade":
        actual_upgrade_fn = upgrade_fn or _default_upgrade
        command_options = Namespace(**vars(plan.upgrade_options))
        command_options.dry_run = dry_run
        install_code = actual_upgrade_fn(command_options)
    else:
        actual_install_fn = install_fn or _default_install
        command_options = replace(plan.install_options, dry_run=dry_run)
        install_code = actual_install_fn(command_options)
    if install_code not in (None, 0):
        raise RuntimeError(f"{plan.operation.capitalize()} failed with exit code {install_code}")

    if plan.configure_providers and plan.provider_mode == "page":
        actual_provider_page_fn()
    else:
        for value in plan.provider_actions:
            actual_provider_set_fn(value.provider, value.field, value.value)

    doctor_status = "skipped"
    if plan.doctor:
        if doctor_fn is not None:
            doctor_code = doctor_fn(plan.project_dir)
            if doctor_code not in (None, 0):
                raise RuntimeError(f"Doctor failed with exit code {doctor_code}")
        doctor_status = "completed"

    return SetupResult(
        plan=plan,
        dry_run=False,
        executed=True,
        planned_actions=plan.actions,
        provider_status=provider_status,
        install_status="completed",
        doctor_status=doctor_status,
    )


def run_setup_wizard(
    args: object,
    *,
    input_fn: InputFn = input,
    output: TextIO = sys.stdout,
    install_fn: InstallFn | None = None,
    upgrade_fn: UpgradeFn | None = None,
    provider_set_fn: ProviderSetFn | None = None,
    doctor_fn: DoctorFn | None = None,
) -> SetupResult:
    dry_run = bool(getattr(args, "dry_run", False))
    project_dir = getattr(args, "project_dir", None)
    no_doctor = bool(getattr(args, "no_doctor", False))
    provider_mode = str(getattr(args, "provider_mode", "page") or "page")
    provider_open_browser = not bool(getattr(args, "no_browser", False))
    provider_host = str(getattr(args, "provider_host", "127.0.0.1") or "127.0.0.1")
    provider_port = int(getattr(args, "provider_port", 0) or 0)
    answers = collect_setup_answers(
        input_fn=input_fn,
        output=output,
        default_project_dir=project_dir,
        force_no_doctor=no_doctor,
        provider_mode=provider_mode,
        provider_open_browser=provider_open_browser,
        provider_host=provider_host,
        provider_port=provider_port,
    )
    plan = build_setup_plan(answers)
    return execute_setup_plan(
        plan,
        dry_run=dry_run,
        install_fn=install_fn,
        upgrade_fn=upgrade_fn,
        provider_set_fn=provider_set_fn,
        provider_page_fn=None,
        doctor_fn=doctor_fn,
        output=output,
    )


def _choose(
    label: str,
    choices: tuple[str, ...],
    *,
    default: str,
    input_fn: InputFn,
    output: TextIO,
    note: str = "",
) -> str:
    if note:
        _explain(note, output=output)
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
        if raw in choices:
            return raw
        print(f"Please enter a number from 1 to {len(choices)}.", file=output)


def _choose_yes_no(
    label: str,
    *,
    default: bool,
    input_fn: InputFn,
    output: TextIO,
    note: str = "",
) -> bool:
    choices = ("yes", "no")
    selected = _choose(label, choices, default="yes" if default else "no", input_fn=input_fn, output=output, note=note)
    return selected == "yes"


def _ask_project_dir(default: Path, *, input_fn: InputFn, output: TextIO, note: str = "") -> Path:
    if note:
        _explain(note, output=output)
    raw = input_fn(f"Project directory [{default}]: ").strip()
    return Path(raw).expanduser().resolve() if raw else default


def _ask_optional_path(label: str, *, input_fn: InputFn, output: TextIO, note: str = "") -> Path | None:
    if note:
        _explain(note, output=output)
    raw = input_fn(f"{label} [default]: ").strip()
    return Path(raw).expanduser().resolve() if raw else None


def _ask_optional_text(label: str, *, input_fn: InputFn, output: TextIO, note: str = "") -> str:
    if note:
        _explain(note, output=output)
    return input_fn(f"{label} [default]: ").strip()


def _collect_provider_values(*, input_fn: InputFn, output: TextIO) -> tuple[ProviderValue, ...]:
    values: list[ProviderValue] = []
    _explain("Leave provider fields empty to skip them.", output=output)
    for provider, field, label in PROVIDER_PROMPTS:
        _explain(PROVIDER_FIELD_NOTES[(provider, field)], output=output)
        raw = input_fn(f"{label}: ").strip()
        if raw:
            values.append(ProviderValue(provider, field, raw))
    return tuple(values)


def _print_answer_summary(answers: SetupAnswers, *, output: TextIO) -> None:
    print("Setup summary:", file=output)
    print(f"  operation: {answers.operation}", file=output)
    print(f"  runtime: {answers.runtime}", file=output)
    print(f"  subject: {answers.subject}", file=output)
    print(f"  coverage: {answers.coverage}", file=output)
    print(f"  mode: {answers.mode}", file=output)
    print(f"  scope: {answers.scope}", file=output)
    print(f"  project_dir: {answers.project_dir}", file=output)
    print(f"  overwrite: {'yes' if answers.overwrite else 'no'}", file=output)
    print(f"  cli_dir: {answers.cli_dir or 'default'}", file=output)
    if answers.operation == "upgrade":
        print(f"  repo: {answers.repo or 'configured/default'}", file=output)
        print(f"  ref: {answers.ref or ('latest beta' if answers.beta else 'latest stable')}", file=output)
    statuses = _provider_status(answers.provider_values)
    for provider, field, _label in PROVIDER_PROMPTS:
        print(f"  {provider} {field}: {statuses[f'{provider}.{field}']}", file=output)
    if answers.configure_providers and answers.provider_mode == "page":
        print("  provider setup: local page", file=output)
    print(f"  doctor: {'run' if answers.run_doctor else 'skip'}", file=output)


def _print_plan_summary(
    plan: SetupPlan,
    *,
    provider_status: dict[str, str],
    dry_run: bool,
    output: TextIO,
) -> None:
    print("Planned actions:", file=output)
    print(f"  command: {' '.join(plan.install_command)}", file=output)
    for provider, field, _label in PROVIDER_PROMPTS:
        print(f"  {provider} {field}: {provider_status[f'{provider}.{field}']}", file=output)
    if plan.configure_providers and plan.provider_mode == "page":
        print("  provider setup: local page", file=output)
    print(f"  doctor: {'run' if plan.doctor else 'skip'}", file=output)
    if dry_run:
        print("  mode: dry-run", file=output)


def _provider_status(values: tuple[ProviderValue, ...]) -> dict[str, str]:
    configured = {(value.provider, value.field) for value in values if value.value.strip()}
    return {
        f"{provider}.{field}": "configured" if (provider, field) in configured else "skipped"
        for provider, field, _label in PROVIDER_PROMPTS
    }


def _client_target_for_runtime(runtime: str) -> str:
    try:
        return CLIENT_TARGET_BY_RUNTIME[runtime]
    except KeyError as exc:
        available = ", ".join(RUNTIME_CHOICES)
        raise ValueError(f"Unsupported runtime: {runtime}. Available runtimes: {available}") from exc


def _append_optional(command: tuple[str, ...], flag: str, value: str) -> tuple[str, ...]:
    return (*command, flag, value) if value else command


def _scope_includes_cli(scope: str) -> bool:
    return scope in {"all", "cli"}


def _normalize_provider_mode(mode: str) -> str:
    normalized = str(mode or "page").strip().lower()
    if normalized not in PROVIDER_MODE_CHOICES:
        available = ", ".join(PROVIDER_MODE_CHOICES)
        raise ValueError(f"Unsupported provider mode: {mode}. Available modes: {available}")
    return normalized


def _parts_for_scope(scope: str, run_doctor: bool) -> tuple[str, ...]:
    if scope == "all":
        parts = ["globals", "project", "cli", "mcp"]
    elif scope in SCOPE_CHOICES:
        parts = [scope]
    else:
        available = ", ".join(SCOPE_CHOICES)
        raise ValueError(f"Unsupported install scope: {scope}. Available scopes: {available}")
    if run_doctor:
        parts.append("doctor")
    return tuple(parts)


def _explain(note: str, *, output: TextIO) -> None:
    print(f"Tip: {note}", file=output)


def _packaged_payload_root() -> Path:
    package_payload = Path(__file__).resolve().parent / "payload"
    if (package_payload / "qiongli-workflow" / "SKILL.md").exists():
        return package_payload
    npm_payload = Path(__file__).resolve().parents[2] / "payload"
    if (npm_payload / "qiongli-workflow" / "SKILL.md").exists():
        return npm_payload
    return Path(__file__).resolve().parents[1]


def _default_install(options: object) -> int | None:
    from .universal_installer import install

    return install(options)


def _default_upgrade(options: object) -> int | None:
    from .cli import cmd_upgrade

    return cmd_upgrade(options)


def _default_provider_set(provider: str, field: str, value: str) -> object:
    from bridges.provider_config import set_provider_value

    return set_provider_value(provider, field, value)


def _default_provider_page(*, host: str, port: int, open_browser: bool, output: TextIO) -> object:
    from bridges.mcp_config_wizard import run_config_wizard

    return run_config_wizard(host=host, port=port, open_browser=open_browser, output=output)
