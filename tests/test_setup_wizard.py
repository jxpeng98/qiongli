from __future__ import annotations

import io
from pathlib import Path
from types import SimpleNamespace

from qiongli.setup_wizard import (
    ProviderValue,
    SetupAnswers,
    build_setup_plan,
    collect_setup_answers,
    execute_setup_plan,
    run_setup_wizard,
)


def test_defaults_create_core_complete_all_plan_and_doctor_enabled(tmp_path: Path) -> None:
    answers = SetupAnswers(project_dir=tmp_path)

    plan = build_setup_plan(answers)

    assert plan.install_command == (
        "qiongli",
        "install",
        "--mode",
        "copy",
        "--subject",
        "core",
        "--coverage",
        "complete",
        "--target",
        "all",
        "--project-dir",
        str(tmp_path),
        "--parts",
        "globals,project,cli,doctor",
    )
    assert plan.operation == "install"
    assert plan.doctor is True
    assert plan.mode == "copy"
    assert plan.overwrite is False
    assert plan.client_target == "all"
    assert plan.scope == "all"
    assert plan.parts == ("globals", "project", "cli", "doctor")
    assert plan.install_options.mode == "copy"
    assert plan.install_options.overwrite is False
    assert plan.install_options.target == "all"
    assert plan.install_options.parts == ("globals", "project", "cli", "doctor")
    assert plan.provider_actions == ()


def test_install_plan_includes_mode_overwrite_and_cli_dir(tmp_path: Path) -> None:
    cli_dir = tmp_path / "bin"
    answers = SetupAnswers(
        runtime="claude-code",
        subject="business",
        coverage="focused",
        scope="all",
        mode="link",
        overwrite=True,
        cli_dir=cli_dir,
        project_dir=tmp_path,
    )

    plan = build_setup_plan(answers)

    assert plan.install_command == (
        "qiongli",
        "install",
        "--mode",
        "link",
        "--subject",
        "business",
        "--coverage",
        "focused",
        "--target",
        "claude",
        "--project-dir",
        str(tmp_path),
        "--parts",
        "globals,project,cli,doctor",
        "--overwrite",
        "--cli-dir",
        str(cli_dir),
    )
    assert plan.install_options.mode == "link"
    assert plan.install_options.overwrite is True
    assert plan.install_options.cli_dir == cli_dir


def test_upgrade_plan_includes_repo_ref_beta_and_no_overwrite(tmp_path: Path) -> None:
    answers = SetupAnswers(
        operation="upgrade",
        runtime="codex",
        subject="finance",
        coverage="focused",
        scope="project",
        mode="link",
        overwrite=False,
        repo="owner/repo",
        ref="main",
        ref_type="branch",
        beta=True,
        project_dir=tmp_path,
    )

    plan = build_setup_plan(answers)

    assert plan.operation == "upgrade"
    assert plan.install_command == (
        "qiongli",
        "upgrade",
        "--repo",
        "owner/repo",
        "--ref",
        "main",
        "--ref-type",
        "branch",
        "--beta",
        "--mode",
        "link",
        "--subject",
        "finance",
        "--coverage",
        "focused",
        "--target",
        "codex",
        "--project-dir",
        str(tmp_path),
        "--parts",
        "project,doctor",
        "--no-overwrite",
        "--doctor",
    )
    assert plan.upgrade_options.repo == "owner/repo"
    assert plan.upgrade_options.ref == "main"
    assert plan.upgrade_options.ref_type == "branch"
    assert plan.upgrade_options.beta is True
    assert plan.upgrade_options.overwrite is False
    assert plan.upgrade_options.parts == "project,doctor"


def test_numbered_choice_collection_uses_empty_defaults(tmp_path: Path) -> None:
    inputs = iter(["", "", "", "", "", "", "", "", "", "", ""])
    output = io.StringIO()

    answers = collect_setup_answers(
        input_fn=lambda _prompt: next(inputs),
        output=output,
        default_project_dir=tmp_path,
    )

    assert answers.operation == "install"
    assert answers.runtime == "multi-platform"
    assert answers.subject == "core"
    assert answers.coverage == "complete"
    assert answers.mode == "copy"
    assert answers.scope == "all"
    assert answers.project_dir == tmp_path
    assert answers.overwrite is False
    assert answers.cli_dir is None
    assert answers.configure_providers is False
    assert answers.run_doctor is True
    rendered = output.getvalue()
    assert "Tip:" in rendered
    assert "Choose whether setup should install bundled assets or upgrade from upstream." in rendered


def test_codex_runtime_targets_codex_client(tmp_path: Path) -> None:
    answers = SetupAnswers(runtime="codex", project_dir=tmp_path)

    plan = build_setup_plan(answers)

    assert plan.client_target == "codex"
    assert "--target" in plan.install_command
    assert plan.install_command[plan.install_command.index("--target") + 1] == "codex"
    assert plan.install_options.target == "codex"


def test_hermes_runtime_targets_hermes_client(tmp_path: Path) -> None:
    answers = SetupAnswers(runtime="hermes", project_dir=tmp_path)

    plan = build_setup_plan(answers)

    assert plan.client_target == "hermes"
    assert "--target" in plan.install_command
    assert plan.install_command[plan.install_command.index("--target") + 1] == "hermes"
    assert plan.install_options.target == "hermes"


def test_antigravity_runtime_targets_antigravity_client(tmp_path: Path) -> None:
    answers = SetupAnswers(runtime="antigravity", project_dir=tmp_path)

    plan = build_setup_plan(answers)

    assert plan.client_target == "antigravity"
    assert "--target" in plan.install_command
    assert plan.install_command[plan.install_command.index("--target") + 1] == "antigravity"
    assert plan.install_options.target == "antigravity"


def test_project_scope_uses_parts_without_invalid_project_target(tmp_path: Path) -> None:
    answers = SetupAnswers(runtime="multi-platform", scope="project", project_dir=tmp_path)

    plan = build_setup_plan(answers)

    assert plan.client_target == "all"
    assert plan.scope == "project"
    assert "--target" in plan.install_command
    assert plan.install_command[plan.install_command.index("--target") + 1] == "all"
    assert "--parts" in plan.install_command
    assert plan.install_command[plan.install_command.index("--parts") + 1] == "project,doctor"
    assert "project" not in (plan.install_command[plan.install_command.index("--target") + 1],)
    assert plan.install_options.target == "all"
    assert plan.install_options.parts == ("project", "doctor")
    assert plan.install_options.install_cli is False


def test_provider_values_are_accepted_but_redacted_in_summary_output(tmp_path: Path) -> None:
    inputs = iter(
        [
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "1",
            "openalex-secret-key",
            "openalex@example.com",
            "secret-s2-key",
            "",
            "secret-pubmed-key",
            "",
        ]
    )
    output = io.StringIO()

    answers = collect_setup_answers(
        input_fn=lambda _prompt: next(inputs),
        output=output,
        default_project_dir=tmp_path,
    )
    plan = build_setup_plan(answers)
    execute_setup_plan(plan, dry_run=True, output=output)

    rendered = output.getvalue()
    assert ProviderValue("openalex", "api_key", "openalex-secret-key") in answers.provider_values
    assert ProviderValue("openalex", "email", "openalex@example.com") in answers.provider_values
    assert ProviderValue("semantic_scholar", "api_key", "secret-s2-key") in answers.provider_values
    assert ProviderValue("pubmed", "api_key", "secret-pubmed-key") in answers.provider_values
    assert "openalex-secret-key" not in rendered
    assert "openalex@example.com" not in rendered
    assert "secret-s2-key" not in rendered
    assert "secret-pubmed-key" not in rendered
    assert "secret-s2-key" not in repr(answers)
    assert "secret-s2-key" not in repr(execute_setup_plan(plan, dry_run=True, output=io.StringIO()))
    assert "openalex api_key: configured" in rendered
    assert "openalex email: configured" in rendered
    assert "semantic_scholar api_key: configured" in rendered
    assert "crossref email: skipped" in rendered


def test_upgrade_collection_prints_notes_for_each_major_step(tmp_path: Path) -> None:
    inputs = iter(
        [
            "2",
            "",
            "",
            "",
            "",
            "2",
            "",
            "",
            "",
            "3",
            "v0.15.0",
            "1",
            "",
            "",
            "",
            "",
            "",
            "2",
        ]
    )
    output = io.StringIO()

    answers = collect_setup_answers(
        input_fn=lambda _prompt: next(inputs),
        output=output,
        default_project_dir=tmp_path,
    )

    rendered = output.getvalue()
    assert answers.operation == "upgrade"
    assert answers.ref == "v0.15.0"
    for token in (
        "Choose whether setup should install bundled assets or upgrade from upstream.",
        "This decides which AI client skill target receives the workflow assets.",
        "Copy is stable for most users",
        "Upgrade normally overwrites managed Qiongli installs",
        "Leave empty to use QIONGLI_REPO",
        "Enter a tag such as v0.14.0",
        "Provider keys enable configured scholarly search clients",
        "OpenAlex requires a free API key",
        "OpenAlex email is optional",
        "Semantic Scholar works without a key",
        "Crossref uses an email address",
        "PubMed API keys are optional",
        "Doctor checks the local environment",
    ):
        assert token in rendered


def test_dry_run_does_not_call_injected_functions(tmp_path: Path) -> None:
    calls: list[object] = []
    answers = SetupAnswers(
        project_dir=tmp_path,
        provider_values=(ProviderValue("crossref", "email", "crossref@example.com"),),
    )
    plan = build_setup_plan(answers)

    result = execute_setup_plan(
        plan,
        dry_run=True,
        install_fn=lambda options: calls.append(("install", options)),
        provider_set_fn=lambda provider, field, value: calls.append((provider, field, value)),
        doctor_fn=lambda project_dir: calls.append(("doctor", project_dir)),
        output=io.StringIO(),
    )

    assert result.executed is False
    assert calls == []
    assert result.planned_actions == plan.actions


def test_non_dry_run_calls_injected_functions_with_expected_values(tmp_path: Path) -> None:
    calls: list[object] = []
    answers = SetupAnswers(
        subject="finance",
        coverage="focused",
        runtime="codex",
        scope="project",
        project_dir=tmp_path,
        provider_values=(ProviderValue("semantic_scholar", "api_key", "secret-s2-key"),),
        run_doctor=True,
    )
    plan = build_setup_plan(answers)

    result = execute_setup_plan(
        plan,
        dry_run=False,
        install_fn=lambda options: calls.append(("install", options)) or 0,
        provider_set_fn=lambda provider, field, value: calls.append((provider, field, value)),
        upgrade_fn=lambda options: calls.append(("upgrade", options)) or 0,
        doctor_fn=lambda project_dir: calls.append(("doctor", project_dir)) or 0,
        output=io.StringIO(),
    )

    install_call = calls[0]
    assert result.executed is True
    assert install_call[0] == "install"
    assert install_call[1].subject == "finance"
    assert install_call[1].coverage == "focused"
    assert install_call[1].target == "codex"
    assert install_call[1].parts == ("project", "doctor")
    assert install_call[1].doctor is True
    assert install_call[1].dry_run is False
    assert install_call[1].project_dir == tmp_path
    assert calls[1] == ("semantic_scholar", "api_key", "secret-s2-key")
    assert calls[2] == ("doctor", tmp_path)


def test_non_dry_run_upgrade_calls_injected_upgrade_function(tmp_path: Path) -> None:
    calls: list[object] = []
    answers = SetupAnswers(
        operation="upgrade",
        runtime="codex",
        scope="globals",
        overwrite=True,
        project_dir=tmp_path,
        run_doctor=False,
    )
    plan = build_setup_plan(answers)

    result = execute_setup_plan(
        plan,
        dry_run=False,
        install_fn=lambda options: calls.append(("install", options)) or 0,
        upgrade_fn=lambda options: calls.append(("upgrade", options)) or 0,
        output=io.StringIO(),
    )

    assert result.executed is True
    assert calls[0][0] == "upgrade"
    assert calls[0][1].target == "codex"
    assert calls[0][1].parts == "globals"
    assert all(call[0] != "install" for call in calls)


def test_invalid_numbered_choice_reprompts(tmp_path: Path) -> None:
    inputs = iter(["", "99", "2", "", "", "", "", "", "", "", "", ""])
    output = io.StringIO()

    answers = collect_setup_answers(
        input_fn=lambda _prompt: next(inputs),
        output=output,
        default_project_dir=tmp_path,
    )

    assert answers.runtime == "cli"
    assert "Please enter a number from 1 to 5." in output.getvalue()


def test_run_setup_wizard_honors_args_dry_run_project_dir_and_no_doctor(tmp_path: Path) -> None:
    output = io.StringIO()

    result = run_setup_wizard(
        SimpleNamespace(dry_run=True, project_dir=tmp_path, no_doctor=True),
        input_fn=lambda _prompt: "",
        output=output,
    )

    assert result.executed is False
    assert result.plan.project_dir == tmp_path
    assert result.plan.doctor is False
    assert result.plan.parts == ("globals", "project", "cli")


def test_run_setup_wizard_preserves_execution_injection_seams(tmp_path: Path) -> None:
    calls: list[object] = []

    result = run_setup_wizard(
        SimpleNamespace(dry_run=False, project_dir=tmp_path, no_doctor=True),
        input_fn=lambda _prompt: "",
        output=io.StringIO(),
        install_fn=lambda options: calls.append(("install", options)) or 0,
        provider_set_fn=lambda provider, field, value: calls.append((provider, field, value)),
        doctor_fn=lambda project_dir: calls.append(("doctor", project_dir)),
    )

    assert result.executed is True
    assert calls[0][0] == "install"
    assert len(calls) == 1
