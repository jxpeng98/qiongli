from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
import uuid
from collections.abc import Mapping
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SRC = REPO_ROOT / "packages" / "python-qiongli" / "src"
if str(PYTHON_SRC) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC))

from bridges.mcp_tool_handlers import call_qiongli_tool  # noqa: E402


FIXTURE_DIR = REPO_ROOT / "tests" / "fixtures" / "subject_runtime_smoke"
MANIFEST_REL = Path(".qiongli") / "guidance_manifest.yaml"
LOCAL_AGENT_ENV = "QIONGLI_SMOKE_RUN_AGENTS"
REPORT_SCHEMA_VERSION = "1.1"
LOCAL_AGENT_DEFAULT_CASES = ("confirmed_finance_guidance_loaded",)
SUBJECT_GUIDANCE_SOURCE = ".qiongli/guidance.d/subject-runtime.md"
LOCAL_AGENT_TASK_OVERRIDES: dict[str, Any] = {
    "run_agents": True,
    "max_revision_rounds": 0,
    "output_budget": 1,
    "skip_validation": True,
    "execution_mode": "solo",
    "controller": "codex",
    "primary": "codex",
    "reviewer": "codex",
    "solo_role_gates": "standard",
}


@dataclass(frozen=True)
class SmokeCase:
    name: str
    manifest: dict[str, Any] | None
    args: dict[str, Any]
    expected: dict[str, Any]
    source: Path
    setup_subject_action: dict[str, Any] | None = None


def load_smoke_cases(fixture_dir: Path = FIXTURE_DIR) -> list[SmokeCase]:
    cases: list[SmokeCase] = []
    for path in sorted(Path(fixture_dir).glob("*.json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        cases.append(
            SmokeCase(
                name=str(payload["name"]),
                manifest=payload.get("manifest"),
                args=dict(payload["args"]),
                expected=dict(payload["expected"]),
                source=path,
                setup_subject_action=(
                    dict(payload["setup_subject_action"])
                    if isinstance(payload.get("setup_subject_action"), dict)
                    else None
                ),
            )
        )
    return sorted(cases, key=lambda case: case.name)


def _select_smoke_cases(
    cases: list[SmokeCase],
    *,
    mode: str,
    selected_cases: list[str] | None,
) -> list[SmokeCase]:
    selected = set(selected_cases or [])
    if mode == "local-agent" and not selected:
        selected = set(LOCAL_AGENT_DEFAULT_CASES)
    if selected:
        filtered = [case for case in cases if case.name in selected]
        found = {case.name for case in filtered}
        missing = sorted(selected - found)
        if missing:
            raise ValueError("unknown smoke case(s): " + ", ".join(missing))
        return filtered
    return list(cases)


def _write_manifest(project_root: Path, manifest: dict[str, Any] | None) -> None:
    if manifest is None:
        return
    manifest_path = project_root / MANIFEST_REL
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(
        yaml.safe_dump(manifest, sort_keys=False, allow_unicode=False),
        encoding="utf-8",
    )


def _isolated_env(project_root: Path) -> dict[str, str]:
    base = project_root / ".smoke-home"
    env = {
        "QIONGLI_GUIDANCE_HOME": str(base / "qiongli-guidance"),
        "QIONGLI_CONFIG_HOME": str(base / "qiongli-config"),
        "CODEX_HOME": str(base / "codex"),
        "XDG_CONFIG_HOME": str(base / "xdg-config"),
        "HOME": str(base / "home"),
        "RESEARCH_CLI_LANG": "en",
    }
    for key, value in env.items():
        if key != "RESEARCH_CLI_LANG":
            Path(value).mkdir(parents=True, exist_ok=True)
    return env


def _task_run_args_for_mode(case: SmokeCase, project_root: Path, mode: str) -> dict[str, Any]:
    args = dict(case.args)
    args["cwd"] = str(project_root)
    if mode == "local-agent":
        args.update(LOCAL_AGENT_TASK_OVERRIDES)
    else:
        args["run_agents"] = False
    return args


def run_smoke_case(case: SmokeCase, workspace_root: Path, mode: str) -> dict[str, Any]:
    if mode not in {"preview", "local-agent"}:
        raise ValueError("mode must be one of: preview, local-agent")

    root = Path(workspace_root).resolve()
    project_root = root / case.name
    project_root.mkdir(parents=True, exist_ok=True)
    _write_manifest(project_root, case.manifest)

    env_updates = _isolated_env(project_root)

    old_env = {key: os.environ.get(key) for key in env_updates}
    old_cwd = Path.cwd()
    try:
        os.environ.update(env_updates)
        os.chdir(project_root)
        if case.setup_subject_action:
            setup_args = dict(case.setup_subject_action)
            setup_args["cwd"] = str(project_root)
            setup_result = call_qiongli_tool("qiongli_subject_update", setup_args)
            if setup_result.get("isError"):
                report = {
                    "name": case.name,
                    "source": _repo_relative(case.source),
                    "project_root": str(project_root),
                    "status": "failed",
                    "failures": [f"setup_subject_action failed: {setup_result}"],
                    "environment": env_updates,
                    "result": setup_result,
                }
                if mode == "local-agent":
                    report["local_agent"] = _local_agent_metadata({})
                    report["trace_assertions"] = _trace_assertions({})
                    report["write_boundary"] = _write_boundary_report(
                        _payload_object(setup_result.get("structuredContent", setup_result)),
                        project_root,
                    )
                    report["rerun_command"] = _rerun_command(mode, case.name)
                return report

        args = _task_run_args_for_mode(case, project_root, mode)
        result = call_qiongli_tool("qiongli_task_run", args)
    finally:
        os.chdir(old_cwd)
        for key, value in old_env.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value

    payload = result.get("structuredContent", result)
    failures = _assert_case(case, result, mode=mode, project_root=project_root)
    report = {
        "name": case.name,
        "source": _repo_relative(case.source),
        "project_root": str(project_root),
        "status": "passed" if not failures else "failed",
        "failures": failures,
        "environment": env_updates,
        "result": payload,
    }
    if mode == "local-agent":
        diagnostic_payload = _payload_object(payload)
        report["local_agent"] = _local_agent_metadata(diagnostic_payload)
        report["trace_assertions"] = _trace_assertions(diagnostic_payload)
        write_boundary = _write_boundary_report(diagnostic_payload, project_root)
        report["write_boundary"] = write_boundary
        if not write_boundary["known_paths_inside_project"]:
            report["status"] = "failed"
            report["failures"].extend(
                f"write boundary violation: {item}"
                for item in write_boundary["violations"]
            )
        if report["status"] != "passed":
            report["rerun_command"] = _rerun_command(mode, case.name)
            report["diagnostics"] = _failure_diagnostics(
                case=case,
                workspace_root=root,
                project_root=project_root,
                payload=diagnostic_payload,
                rerun_command=report["rerun_command"],
            )
    return report


def _assert_case(
    case: SmokeCase,
    result: dict[str, Any],
    *,
    mode: str = "preview",
    project_root: Path | None = None,
) -> list[str]:
    failures: list[str] = []
    if result.get("isError"):
        payload = result.get("structuredContent", {})
        error = payload.get("error") if isinstance(payload, dict) else None
        failures.append(f"tool returned isError=true: {error or 'unknown error'}")
        return failures

    payload = result.get("structuredContent", result)
    if not isinstance(payload, dict):
        return ["tool returned non-object payload"]

    data = payload.get("data", {})
    if not isinstance(data, dict):
        return ["payload data is missing or not an object"]

    preview = data.get("task_run_preview", {})
    task_packet = data.get("task_packet", {})
    if not isinstance(preview, dict):
        preview = {}
    if not isinstance(task_packet, dict):
        task_packet = {}

    expected = case.expected
    guidance_source = expected.get("guidance_source")
    if guidance_source is not None:
        local_guidance = _payload_object(task_packet.get("local_guidance", {}))
        files_read = _payload_list(local_guidance.get("guidance_files_read", []))
        if guidance_source not in files_read:
            failures.append(f"missing guidance source {guidance_source!r}")

    refinement = preview.get("subject_refinement") or task_packet.get("subject_refinement") or {}
    if not isinstance(refinement, dict):
        refinement = {}

    expected_run_agents = True if mode == "local-agent" else expected.get("run_agents")
    _expect_equal(failures, "run_agents", payload.get("run_agents"), expected_run_agents)
    _expect_equal(failures, "decision", refinement.get("decision"), expected.get("decision"))
    _expect_equal(
        failures,
        "primary_subject",
        refinement.get("primary_subject"),
        expected.get("primary_subject"),
    )
    if refinement.get("decision") != "no_subject":
        if "signals" not in refinement:
            failures.append("missing signals ledger")
        if "resource_activation_plan" not in refinement:
            failures.append("missing resource_activation_plan")

    effective_domain = preview.get("effective_domain") or task_packet.get("domain")
    _expect_equal(
        failures,
        "effective_domain",
        effective_domain,
        expected.get("effective_domain"),
    )

    loaded_resources = refinement.get("loaded_resources", {})
    if not isinstance(loaded_resources, dict):
        loaded_resources = {}
    _expect_equal(
        failures,
        "resource_levels",
        list(loaded_resources.get("levels", []) or []),
        list(expected.get("resource_levels", []) or []),
    )

    method_lens = expected.get("method_lens")
    if method_lens is not None and method_lens not in list(refinement.get("method_lenses", []) or []):
        failures.append(f"missing method lens {method_lens!r}")

    borrowed_lens = expected.get("borrowed_lens")
    borrowed_subject = expected.get("borrowed_subject")
    if borrowed_lens is not None or borrowed_subject is not None:
        borrowed = refinement.get("borrowed_lenses", [])
        if not isinstance(borrowed, list):
            borrowed = []
        if not any(
            _borrowed_lens_matches(item, lens=borrowed_lens, subject=borrowed_subject)
            for item in borrowed
        ):
            failures.append(
                "missing borrowed lens "
                f"lens={borrowed_lens!r} subject={borrowed_subject!r}"
            )

    if mode == "local-agent":
        trace = _local_guidance_trace_from_payload(payload)
        trace_assertions = _trace_assertions(payload)
        if not trace:
            failures.append("missing local guidance trace")
        if expected.get("guidance_source") and not trace_assertions["subject_guidance_loaded"]:
            failures.append(
                f"missing local-agent guidance source {expected['guidance_source']!r}"
            )
        if not trace_assertions["subject_refinement_persisted"]:
            failures.append("missing local-agent subject refinement packet")

    return failures


def _borrowed_lens_matches(item: Any, *, lens: Any, subject: Any) -> bool:
    if not isinstance(item, dict):
        return False
    if lens is not None and item.get("lens") != lens:
        return False
    if subject is not None and item.get("source_subject") != subject:
        return False
    return True


def _expect_equal(failures: list[str], field: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        failures.append(f"{field}: expected {expected!r}, got {actual!r}")


def _payload_data(payload: dict[str, Any]) -> dict[str, Any]:
    return _payload_object(payload.get("data", {}))


def _task_packet_from_payload(payload: dict[str, Any]) -> dict[str, Any]:
    data = _payload_data(payload)
    return _payload_object(data.get("task_packet", {}))


def _local_guidance_trace_from_payload(payload: dict[str, Any]) -> dict[str, Any]:
    data = _payload_data(payload)
    return _payload_object(data.get("local_guidance_trace", {}))


def _payload_object(value: Any) -> dict[str, Any]:
    return dict(value) if isinstance(value, Mapping) else {}


def _payload_list(value: Any) -> list[Any]:
    return value if isinstance(value, list) else []


def _payload_string(value: Any) -> str:
    return value if isinstance(value, str) else ""


def _routing_notes_from_payload(payload: dict[str, Any]) -> list[str]:
    notes = _payload_data(payload).get("routing_notes", [])
    return [str(item) for item in notes] if isinstance(notes, list) else []


def _runtime_notes(notes: list[str]) -> list[str]:
    keywords = ("runtime", "preflight", "fallback", "agent")
    return [note for note in notes if any(keyword in note.lower() for keyword in keywords)]


def _local_agent_metadata(payload: dict[str, Any]) -> dict[str, Any]:
    packet = _task_packet_from_payload(payload)
    controller_metadata = _payload_object(packet.get("controller_metadata", {}))
    routing_notes = _routing_notes_from_payload(payload)
    return {
        "requested": True,
        "env_opt_in": os.environ.get(LOCAL_AGENT_ENV) == "1",
        "will_launch_agents": bool(payload.get("run_agents")),
        "requested_runtime": {
            "controller": _payload_string(
                packet.get("controller") or controller_metadata.get("controller")
            ),
            "primary_agent": _payload_string(
                packet.get("primary_agent") or controller_metadata.get("primary_agent")
            ),
            "review_agent": _payload_string(
                packet.get("review_agent") or controller_metadata.get("review_agent")
            ),
        },
        "runtime_plan": _payload_object(packet.get("runtime_plan", {})),
        "routing_notes": routing_notes,
        "runtime_notes": _runtime_notes(routing_notes),
    }


def _trace_assertions(payload: dict[str, Any]) -> dict[str, bool]:
    packet = _task_packet_from_payload(payload)
    guidance = _payload_object(packet.get("local_guidance", {}))
    trace = _local_guidance_trace_from_payload(payload)
    files_read = _payload_list(guidance.get("guidance_files_read", []))
    trace_files_read = _payload_list(trace.get("guidance_files_read", []))
    subject_guidance_loaded = (
        SUBJECT_GUIDANCE_SOURCE in files_read
        or SUBJECT_GUIDANCE_SOURCE in trace_files_read
    )
    return {
        "trace_written": bool(trace),
        "subject_guidance_loaded": subject_guidance_loaded,
        "subject_refinement_persisted": isinstance(packet.get("subject_refinement"), dict),
    }


def run_smoke_suite(
    fixture_dir: Path = FIXTURE_DIR,
    workspace_root: Path | None = None,
    mode: str = "preview",
    selected_cases: list[str] | None = None,
) -> dict[str, Any]:
    if mode not in {"preview", "local-agent"}:
        raise ValueError("mode must be one of: preview, local-agent")
    if mode == "local-agent" and os.environ.get(LOCAL_AGENT_ENV) != "1":
        raise RuntimeError(
            "local-agent smoke requires QIONGLI_SMOKE_RUN_AGENTS=1 and launches local runtime agents"
        )

    cases = _select_smoke_cases(
        load_smoke_cases(fixture_dir),
        mode=mode,
        selected_cases=selected_cases,
    )
    if not cases:
        raise ValueError("no subject runtime smoke cases selected or loaded")

    with tempfile.TemporaryDirectory(prefix="qiongli-smoke-") as tmp_dir:
        root = Path(workspace_root).resolve() if workspace_root else Path(tmp_dir).resolve()
        root.mkdir(parents=True, exist_ok=True)
        case_results = [run_smoke_case(case, root, mode) for case in cases]

    failed = sum(1 for case in case_results if case["status"] != "passed")
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "created_at": datetime.now(timezone.utc).isoformat(),
        "run_id": uuid.uuid4().hex,
        "mode": mode,
        "summary": {
            "total": len(case_results),
            "passed": len(case_results) - failed,
            "failed": failed,
        },
        "cases": case_results,
        "environment": {
            "repo_root": str(REPO_ROOT),
            "python": sys.executable,
            "workspace_root": str(root),
        },
    }


def _repo_relative(path: Path) -> str:
    resolved = path.resolve()
    try:
        return str(resolved.relative_to(REPO_ROOT))
    except ValueError:
        return str(resolved)


def _resolve_reported_path(project_root: Path, value: Any) -> Path | None:
    if not isinstance(value, str) or not value.strip():
        return None
    raw = Path(value)
    return raw.resolve() if raw.is_absolute() else (project_root / raw).resolve()


def _path_inside_project(project_root: Path, path: Path) -> bool:
    try:
        path.relative_to(project_root.resolve())
        return True
    except ValueError:
        return False


def _write_boundary_report(payload: dict[str, Any], project_root: Path) -> dict[str, Any]:
    violations: list[str] = []
    checked_paths: list[str] = []
    expected_paths = [
        ".qiongli/guidance_manifest.yaml",
        SUBJECT_GUIDANCE_SOURCE,
        ".qiongli/trace",
    ]
    for rel_path in expected_paths:
        resolved = (project_root / rel_path).resolve()
        checked_paths.append(str(resolved))
        if not _path_inside_project(project_root, resolved):
            violations.append(str(resolved))

    trace = _local_guidance_trace_from_payload(payload)
    for key in ("run_dir", "trace_index", "proposal_path", "guidance_proposal"):
        resolved = _resolve_reported_path(project_root, trace.get(key))
        if resolved is None:
            continue
        checked_paths.append(str(resolved))
        if not _path_inside_project(project_root, resolved):
            violations.append(str(resolved))

    return {
        "known_paths_inside_project": not violations,
        "checked_paths": sorted(set(checked_paths)),
        "violations": violations,
    }


def _trace_paths(payload: dict[str, Any]) -> dict[str, str]:
    trace = _local_guidance_trace_from_payload(payload)
    keys = ("run_dir", "trace_index", "proposal_path", "guidance_proposal")
    return {key: value for key in keys if isinstance((value := trace.get(key)), str) and value}


def _failure_diagnostics(
    *,
    case: SmokeCase,
    workspace_root: Path,
    project_root: Path,
    payload: dict[str, Any],
    rerun_command: str,
) -> dict[str, Any]:
    return {
        "case_name": case.name,
        "workspace_root": str(workspace_root.resolve()),
        "project_root": str(project_root.resolve()),
        "rerun_command": rerun_command,
        "trace_paths": _trace_paths(payload),
    }


def _rerun_command(mode: str, case_name: str | None = None) -> str:
    parts = [
        "uv",
        "run",
        "python",
        "tooling/scripts/run_subject_runtime_smoke.py",
        "--mode",
        mode,
    ]
    if case_name:
        parts.extend(["--case", case_name])
    parts.append("--json")
    command = " ".join(parts)
    if mode == "local-agent":
        command = f"{LOCAL_AGENT_ENV}=1 {command}"
    return command


def _error_report(mode: str, error: Exception) -> dict[str, Any]:
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "created_at": datetime.now(timezone.utc).isoformat(),
        "run_id": uuid.uuid4().hex,
        "mode": mode,
        "summary": {"total": 0, "passed": 0, "failed": 1},
        "cases": [],
        "environment": {
            "repo_root": str(REPO_ROOT),
            "python": sys.executable,
        },
        "error": str(error),
        "rerun_command": _rerun_command(mode),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run Qiongli subject runtime smoke cases.")
    parser.add_argument("--fixture-dir", type=Path, default=FIXTURE_DIR)
    parser.add_argument("--workspace-root", type=Path)
    parser.add_argument("--mode", choices=("preview", "local-agent"), default="preview")
    parser.add_argument("--case", action="append", default=[])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)

    try:
        report = run_smoke_suite(
            fixture_dir=args.fixture_dir,
            workspace_root=args.workspace_root,
            mode=args.mode,
            selected_cases=list(args.case),
        )
    except Exception as exc:  # noqa: BLE001 - CLI should still emit machine-readable JSON.
        report = _error_report(args.mode, exc)

    print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    return 0 if report["summary"]["failed"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
