from __future__ import annotations

import sys
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Callable, TextIO

from .universal_installer import InstallOptions


RUNTIME_CHOICES = ("multi-platform", "cli", "codex", "claude-code")
CLIENT_TARGET_BY_RUNTIME = {
    "multi-platform": "all",
    "cli": "all",
    "codex": "codex",
    "claude-code": "claude",
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
SCOPE_CHOICES = ("all", "globals", "project", "cli")

PROVIDER_PROMPTS = (
    ("openalex", "email", "OpenAlex email"),
    ("semantic_scholar", "api_key", "Semantic Scholar API key"),
    ("crossref", "email", "Crossref email"),
    ("pubmed", "api_key", "PubMed API key"),
)


InputFn = Callable[[str], str]
InstallFn = Callable[[object], int | None]
ProviderSetFn = Callable[[str, str, str], object]
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
    runtime: str = "multi-platform"
    subject: str = "core"
    coverage: str = "complete"
    scope: str = "all"
    project_dir: Path = field(default_factory=Path.cwd)
    configure_providers: bool = False
    provider_values: tuple[ProviderValue, ...] = ()
    run_doctor: bool = True


@dataclass(frozen=True)
class SetupPlan:
    runtime: str
    client_target: str
    subject: str
    coverage: str
    scope: str
    parts: tuple[str, ...]
    project_dir: Path
    doctor: bool
    install_command: tuple[str, ...]
    install_options: object
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
) -> SetupAnswers:
    project_default = Path(default_project_dir).expanduser().resolve() if default_project_dir else Path.cwd()

    print("Qiongli setup", file=output)
    runtime = _choose(
        "Runtime/client surface",
        RUNTIME_CHOICES,
        default="multi-platform",
        input_fn=input_fn,
        output=output,
    )
    subject = _choose("Subject", SUBJECT_CHOICES, default="core", input_fn=input_fn, output=output)
    coverage = _choose("Coverage", COVERAGE_CHOICES, default="complete", input_fn=input_fn, output=output)
    scope = _choose("Install scope", SCOPE_CHOICES, default="all", input_fn=input_fn, output=output)
    project_dir = _ask_project_dir(project_default, input_fn=input_fn, output=output)
    configure_providers = _choose_yes_no(
        "Configure literature provider keys now?",
        default=False,
        input_fn=input_fn,
        output=output,
    )
    provider_values = _collect_provider_values(input_fn=input_fn, output=output) if configure_providers else ()
    run_doctor = False if force_no_doctor else _choose_yes_no(
        "Run doctor after setup?",
        default=True,
        input_fn=input_fn,
        output=output,
    )

    answers = SetupAnswers(
        runtime=runtime,
        subject=subject,
        coverage=coverage,
        scope=scope,
        project_dir=project_dir,
        configure_providers=configure_providers,
        provider_values=provider_values,
        run_doctor=run_doctor,
    )
    _print_answer_summary(answers, output=output)
    return answers


def build_setup_plan(answers: SetupAnswers) -> SetupPlan:
    project_dir = Path(answers.project_dir).expanduser().resolve()
    client_target = _client_target_for_runtime(answers.runtime)
    parts = _parts_for_scope(answers.scope, answers.run_doctor)
    install_command = (
        "qiongli",
        "install",
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

    provider_actions = tuple(value for value in answers.provider_values if value.value.strip())
    actions = [
        (
            f"install subject={answers.subject} coverage={answers.coverage} "
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
        install_cli="cli" in parts,
        doctor=answers.run_doctor,
        dry_run=False,
        profile="full",
        parts=parts,
    )
    return SetupPlan(
        runtime=answers.runtime,
        client_target=client_target,
        subject=answers.subject,
        coverage=answers.coverage,
        scope=answers.scope,
        parts=parts,
        project_dir=project_dir,
        doctor=answers.run_doctor,
        install_command=install_command,
        install_options=install_options,
        provider_actions=provider_actions,
        actions=tuple(actions),
    )


def execute_setup_plan(
    plan: SetupPlan,
    *,
    dry_run: bool = False,
    install_fn: InstallFn | None = None,
    provider_set_fn: ProviderSetFn | None = None,
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

    actual_install_fn = install_fn or _default_install
    actual_provider_set_fn = provider_set_fn or _default_provider_set
    install_options = replace(plan.install_options, dry_run=dry_run)
    install_code = actual_install_fn(install_options)
    if install_code not in (None, 0):
        raise RuntimeError(f"Install failed with exit code {install_code}")

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
    provider_set_fn: ProviderSetFn | None = None,
    doctor_fn: DoctorFn | None = None,
) -> SetupResult:
    dry_run = bool(getattr(args, "dry_run", False))
    project_dir = getattr(args, "project_dir", None)
    no_doctor = bool(getattr(args, "no_doctor", False))
    answers = collect_setup_answers(
        input_fn=input_fn,
        output=output,
        default_project_dir=project_dir,
        force_no_doctor=no_doctor,
    )
    plan = build_setup_plan(answers)
    return execute_setup_plan(
        plan,
        dry_run=dry_run,
        install_fn=install_fn,
        provider_set_fn=provider_set_fn,
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
) -> str:
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
) -> bool:
    choices = ("yes", "no")
    selected = _choose(label, choices, default="yes" if default else "no", input_fn=input_fn, output=output)
    return selected == "yes"


def _ask_project_dir(default: Path, *, input_fn: InputFn, output: TextIO) -> Path:
    raw = input_fn(f"Project directory [{default}]: ").strip()
    return Path(raw).expanduser().resolve() if raw else default


def _collect_provider_values(*, input_fn: InputFn, output: TextIO) -> tuple[ProviderValue, ...]:
    values: list[ProviderValue] = []
    print("Leave provider fields empty to skip them.", file=output)
    for provider, field, label in PROVIDER_PROMPTS:
        raw = input_fn(f"{label}: ").strip()
        if raw:
            values.append(ProviderValue(provider, field, raw))
    return tuple(values)


def _print_answer_summary(answers: SetupAnswers, *, output: TextIO) -> None:
    print("Setup summary:", file=output)
    print(f"  runtime: {answers.runtime}", file=output)
    print(f"  subject: {answers.subject}", file=output)
    print(f"  coverage: {answers.coverage}", file=output)
    print(f"  scope: {answers.scope}", file=output)
    print(f"  project_dir: {answers.project_dir}", file=output)
    statuses = _provider_status(answers.provider_values)
    for provider, field, _label in PROVIDER_PROMPTS:
        print(f"  {provider} {field}: {statuses[f'{provider}.{field}']}", file=output)
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


def _parts_for_scope(scope: str, run_doctor: bool) -> tuple[str, ...]:
    if scope == "all":
        parts = ["globals", "project", "cli"]
    elif scope in SCOPE_CHOICES:
        parts = [scope]
    else:
        available = ", ".join(SCOPE_CHOICES)
        raise ValueError(f"Unsupported install scope: {scope}. Available scopes: {available}")
    if run_doctor:
        parts.append("doctor")
    return tuple(parts)


def _packaged_payload_root() -> Path:
    package_payload = Path(__file__).resolve().parent / "payload"
    if (package_payload / "qiongli-workflow" / "SKILL.md").exists():
        return package_payload
    return Path(__file__).resolve().parents[1]


def _default_install(options: object) -> int | None:
    from .universal_installer import install

    return install(options)


def _default_provider_set(provider: str, field: str, value: str) -> object:
    from bridges.provider_config import set_provider_value

    return set_provider_value(provider, field, value)
