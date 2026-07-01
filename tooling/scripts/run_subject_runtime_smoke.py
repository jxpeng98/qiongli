from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
import uuid
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


@dataclass(frozen=True)
class SmokeCase:
    name: str
    manifest: dict[str, Any] | None
    args: dict[str, Any]
    expected: dict[str, Any]
    source: Path


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
            )
        )
    return sorted(cases, key=lambda case: case.name)


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
    return {
        "QIONGLI_GUIDANCE_HOME": str(base / "qiongli-guidance"),
        "QIONGLI_CONFIG_HOME": str(base / "qiongli-config"),
        "CODEX_HOME": str(base / "codex"),
        "XDG_CONFIG_HOME": str(base / "xdg-config"),
    }


def run_smoke_case(case: SmokeCase, workspace_root: Path, mode: str) -> dict[str, Any]:
    if mode not in {"preview", "local-agent"}:
        raise ValueError("mode must be one of: preview, local-agent")

    root = Path(workspace_root).resolve()
    project_root = root / case.name
    project_root.mkdir(parents=True, exist_ok=True)
    _write_manifest(project_root, case.manifest)

    args = dict(case.args)
    args["cwd"] = str(project_root)
    args["run_agents"] = mode == "local-agent"

    env_updates = _isolated_env(project_root)
    for value in env_updates.values():
        Path(value).mkdir(parents=True, exist_ok=True)

    old_env = {key: os.environ.get(key) for key in env_updates}
    old_cwd = Path.cwd()
    try:
        os.environ.update(env_updates)
        os.chdir(project_root)
        result = call_qiongli_tool("qiongli_task_run", args)
    finally:
        os.chdir(old_cwd)
        for key, value in old_env.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value

    failures = _assert_case(case, result)
    payload = result.get("structuredContent", result)
    return {
        "name": case.name,
        "source": _repo_relative(case.source),
        "project_root": str(project_root),
        "status": "passed" if not failures else "failed",
        "failures": failures,
        "environment": env_updates,
        "result": payload,
    }


def _assert_case(case: SmokeCase, result: dict[str, Any]) -> list[str]:
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

    refinement = preview.get("subject_refinement") or task_packet.get("subject_refinement") or {}
    if not isinstance(refinement, dict):
        refinement = {}

    expected = case.expected
    _expect_equal(failures, "run_agents", payload.get("run_agents"), expected.get("run_agents"))
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

    cases = load_smoke_cases(fixture_dir)
    selected = set(selected_cases or [])
    if selected:
        cases = [case for case in cases if case.name in selected]
        found = {case.name for case in cases}
        missing = sorted(selected - found)
        if missing:
            raise ValueError("unknown smoke case(s): " + ", ".join(missing))
    if not cases:
        raise ValueError("no subject runtime smoke cases selected or loaded")

    with tempfile.TemporaryDirectory(prefix="qiongli-smoke-") as tmp_dir:
        root = Path(workspace_root).resolve() if workspace_root else Path(tmp_dir).resolve()
        root.mkdir(parents=True, exist_ok=True)
        case_results = [run_smoke_case(case, root, mode) for case in cases]

    failed = sum(1 for case in case_results if case["status"] != "passed")
    return {
        "schema_version": "1.0",
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


def _error_report(mode: str, error: Exception) -> dict[str, Any]:
    return {
        "schema_version": "1.0",
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
