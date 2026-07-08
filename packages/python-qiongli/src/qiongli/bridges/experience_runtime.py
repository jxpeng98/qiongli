from __future__ import annotations

import json
from collections.abc import Mapping
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCHEMA_VERSION = "1.0"
EXPERIENCE_INDEX_REL = Path(".qiongli") / "trace" / "experience.jsonl"
EXPERIENCE_RECORD_NAME = "experience_record.json"
REQUIRED_EXPERIENCE_OBJECT_FIELDS = (
    "task",
    "execution",
    "inputs",
    "outputs",
    "quality",
    "experience",
    "privacy",
)


def build_experience_record(
    *,
    project_root: Path,
    run_dir: Path,
    guidance_trace: dict[str, Any],
    task_packet: dict[str, Any],
    validator_gate: dict[str, Any],
) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    worker_state = _mapping(task_packet.get("worker_orchestration"))
    controller = _mapping(task_packet.get("controller_metadata"))
    subject_refinement = _mapping(
        guidance_trace.get("subject_refinement", task_packet.get("subject_refinement", {}))
    )
    required_outputs = _string_list(task_packet.get("required_outputs"))
    found_outputs = _string_list(validator_gate.get("found"))
    missing_outputs = _string_list(validator_gate.get("missing"))

    return {
        "schema_version": SCHEMA_VERSION,
        "run_id": str(guidance_trace.get("run_id", "")),
        "created_at": str(guidance_trace.get("created_at") or _utc_now()),
        "project_root": str(root),
        "task": {
            "task_id": str(task_packet.get("task_id", guidance_trace.get("task_id", ""))),
            "paper_type": str(
                task_packet.get("paper_type", guidance_trace.get("paper_type", ""))
            ),
            "topic": str(task_packet.get("topic", guidance_trace.get("topic", ""))),
            "workflow": str(task_packet.get("workflow", "")),
            "stage": str(task_packet.get("stage", "")),
        },
        "execution": {
            "run_agents": bool(task_packet.get("run_agents", False)),
            "execution_mode": str(
                controller.get("execution_mode") or _execution_mode_from_worker(worker_state)
            ),
            "controller": str(controller.get("controller", "")),
            "primary_agent": str(controller.get("primary_agent", "")),
            "review_agent": str(controller.get("review_agent", "")),
            "verifier_agent": str(controller.get("verifier_agent", "")),
            "worker_mode": str(
                worker_state.get("mode", worker_state.get("orchestration_mode", "none"))
            ),
            "worker_status": str(worker_state.get("status", "")),
            "worker_barrier_status": str(worker_state.get("barrier_status", "")),
            "worker_count": _worker_count(worker_state),
            "worker_merge_status": str(worker_state.get("merge_status", "")),
            "worker_final_review_status": str(worker_state.get("merge_review_status", "")),
        },
        "inputs": {
            "guidance_files_read": _string_list(guidance_trace.get("guidance_files_read")),
            "guidance_sources": _dict_list(guidance_trace.get("guidance_sources")),
            "project_manifest": _mapping(guidance_trace.get("project_manifest")),
            "subject_refinement": subject_refinement,
            "provider_status": _mapping(task_packet.get("provider_status")),
            "mcp_evidence": _dict_list(task_packet.get("mcp_evidence")),
            "required_skills": _string_list(task_packet.get("required_skills")),
        },
        "outputs": {
            "required_outputs": required_outputs,
            "found_outputs": found_outputs,
            "missing_outputs": missing_outputs,
            "artifacts_written": _string_list(task_packet.get("artifacts_written")),
            "trace_files": _trace_files(root, run_dir),
        },
        "quality": {
            "validator_status": _validator_status(validator_gate),
            "review_status": _review_status(worker_state),
            "blocking_issues": _blocking_issues(validator_gate, worker_state),
            "warnings": _string_list(task_packet.get("warnings")),
            "confidence": _safe_float(task_packet.get("confidence", 0.0)),
        },
        "experience": {
            "lessons": [],
            "failure_modes": _failure_modes(missing_outputs, validator_gate, worker_state),
            "reusable_guidance": [],
            "promotion_candidates": [],
            "guidance_update": {
                "proposal_path": str(guidance_trace.get("guidance_proposal", "")),
                "applied": bool(guidance_trace.get("applied_guidance_update", False)),
                "mode": str(guidance_trace.get("guidance_mode", "")),
            },
        },
        "privacy": {
            "redaction_status": "not_needed",
            "contains_user_corpus": False,
            "contains_provider_metadata": bool(task_packet.get("mcp_evidence")),
        },
    }


def write_experience_record(
    *,
    project_root: Path,
    run_dir: Path,
    guidance_trace: dict[str, Any],
    task_packet: dict[str, Any],
    validator_gate: dict[str, Any],
) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    resolved_run_dir = Path(run_dir).expanduser().resolve()
    record = build_experience_record(
        project_root=root,
        run_dir=resolved_run_dir,
        guidance_trace=guidance_trace,
        task_packet=task_packet,
        validator_gate=validator_gate,
    )
    record_path = resolved_run_dir / EXPERIENCE_RECORD_NAME
    index_path = root / EXPERIENCE_INDEX_REL
    record_path.parent.mkdir(parents=True, exist_ok=True)
    index_path.parent.mkdir(parents=True, exist_ok=True)
    record_path.write_text(
        json.dumps(record, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    with index_path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(_index_row(record), ensure_ascii=False, sort_keys=True) + "\n")
    return {
        "experience_status": "written",
        "experience_record": _rel(root, record_path),
        "experience_index": _rel(root, index_path),
    }


def experience_summary(project_root: Path, *, limit: int = 20) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    rows, malformed_count = _load_experience_rows(root)
    if not rows and malformed_count == 0:
        return {"project_dir": str(root), "run_count": 0, "malformed_count": 0, "runs": []}

    return {
        "project_dir": str(root),
        "run_count": len(rows),
        "malformed_count": malformed_count,
        "runs": rows[-max(0, limit):],
    }


def query_experience(
    project_root: Path,
    *,
    task_id: str | None = None,
    stage: str | None = None,
    topic: str | None = None,
    subject: str | None = None,
    validator_status: str | None = None,
    failure_mode: str | None = None,
    guidance_source: str | None = None,
    worker_mode: str | None = None,
    limit: int = 20,
) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    rows, malformed_count = _load_experience_rows(root)
    filters = _query_filters(
        task_id=task_id,
        stage=stage,
        topic=topic,
        subject=subject,
        validator_status=validator_status,
        failure_mode=failure_mode,
        guidance_source=guidance_source,
        worker_mode=worker_mode,
        limit=limit,
    )
    matched = [row for row in rows if _record_matches(row, filters)]
    return {
        "project_dir": str(root),
        "run_count": len(matched),
        "malformed_count": malformed_count,
        "filters": filters,
        "records": matched[-max(0, int(limit)):],
    }


def show_experience(project_root: Path, run_id: str) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    normalized_run_id = str(run_id).strip()
    if not normalized_run_id:
        raise ValueError("run_id is required")
    record_path = _record_path_for_run(root, normalized_run_id)
    if record_path.is_file():
        loaded = json.loads(record_path.read_text(encoding="utf-8"))
        if not isinstance(loaded, dict):
            raise ValueError(f"Experience record is not an object: {_rel(root, record_path)}")
        return {
            "project_dir": str(root),
            "run_id": normalized_run_id,
            "record_path": _rel(root, record_path),
            "record": loaded,
        }

    rows, malformed_count = _load_experience_rows(root)
    for row in rows:
        if str(row.get("run_id", "")).strip() == normalized_run_id:
            return {
                "project_dir": str(root),
                "run_id": normalized_run_id,
                "record_path": "",
                "malformed_count": malformed_count,
                "record": row,
            }
    raise FileNotFoundError(f"Experience record not found for run_id: {normalized_run_id}")


def experience_lessons(
    project_root: Path,
    *,
    task_id: str | None = None,
    topic: str | None = None,
    failure_mode: str | None = None,
    limit: int = 20,
) -> dict[str, Any]:
    result = query_experience(
        project_root,
        task_id=task_id,
        topic=topic,
        failure_mode=failure_mode,
        limit=limit,
    )
    records = [_lesson_record(row) for row in result["records"]]
    return {
        "project_dir": result["project_dir"],
        "run_count": len(records),
        "malformed_count": result["malformed_count"],
        "filters": result["filters"],
        "records": records,
    }


def replay_experience_plan(project_root: Path, run_id: str) -> dict[str, Any]:
    shown = show_experience(project_root, run_id)
    record = shown["record"]
    status = _validator_status_from_record(record)
    failure_modes = _failure_modes_from_record(record)
    missing_outputs = _outputs_from_record(record).get("missing_outputs", [])
    next_action = (
        "rerun_after_addressing_failures"
        if status in {"failed", "blocked"} or failure_modes or missing_outputs
        else "no_rerun_needed"
    )
    return {
        "project_dir": shown["project_dir"],
        "run_id": shown["run_id"],
        "record_path": shown["record_path"],
        "task": _task_from_record(record),
        "guidance_sources": _guidance_sources_from_record(record),
        "validator_status": status,
        "failure_modes": failure_modes,
        "missing_outputs": missing_outputs,
        "next_action": next_action,
        "recommendation": _replay_recommendation(status, failure_modes, missing_outputs),
    }


def select_prior_experience(
    project_root: Path,
    *,
    task_id: str,
    topic: str = "",
    limit: int = 5,
) -> dict[str, Any]:
    result = query_experience(project_root, task_id=task_id, topic=topic, limit=limit)
    return {
        "source": "local-project-experience",
        "query": {
            "task_id": task_id,
            "topic": topic,
            "limit": int(limit),
        },
        "records": [_prior_record(row) for row in result["records"]],
        "malformed_count": result["malformed_count"],
    }


def generate_skill_reinforcement_candidate(
    project_root: Path,
    *,
    task_id: str | None = None,
    min_support: int = 3,
    scope: str = "skill-candidate",
    test_plan: str = "",
) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    rows, malformed_count = _load_experience_rows(root)
    patterns = _supported_failure_patterns(rows, task_id=task_id, min_support=min_support)
    if not patterns:
        return {
            "status": "insufficient_support",
            "scope": scope,
            "project_dir": str(root),
            "min_support": int(min_support),
            "malformed_count": malformed_count,
            "affected_skill_ids": [],
            "supporting_run_ids": [],
        }

    selected = patterns[0]
    supporting_records = selected["records"]
    affected_skill_ids = _affected_skill_ids(supporting_records)
    supporting_run_ids = [
        str(record.get("run_id", ""))
        for record in supporting_records
        if str(record.get("run_id", "")).strip()
    ]
    candidate_dir = root / ".qiongli" / "trace" / "promotion"
    candidate_dir.mkdir(parents=True, exist_ok=True)
    filename = (
        "canonical-candidate-"
        if scope == "canonical-candidate"
        else "skill-reinforcement-candidate-"
    ) + _utc_date() + ".md"
    candidate_path = candidate_dir / filename
    candidate_path.write_text(
        _skill_reinforcement_candidate_text(
            scope=scope,
            task_id=str(selected["task_id"]),
            failure_mode=str(selected["failure_mode"]),
            affected_skill_ids=affected_skill_ids,
            supporting_run_ids=supporting_run_ids,
            support_count=len(supporting_records),
            test_plan=test_plan,
        ),
        encoding="utf-8",
    )
    return {
        "status": "candidate_written",
        "scope": scope,
        "project_dir": str(root),
        "candidate_path": _rel(root, candidate_path),
        "affected_skill_ids": affected_skill_ids,
        "support_count": len(supporting_records),
        "supporting_run_ids": supporting_run_ids,
        "failure_mode": selected["failure_mode"],
        "malformed_count": malformed_count,
    }


def promote_experience(
    project_root: Path,
    *,
    scope: str,
    task_id: str | None = None,
    min_support: int = 3,
    test_plan: str = "",
    approved: bool = False,
) -> dict[str, Any]:
    normalized_scope = str(scope or "").strip().lower()
    if normalized_scope in {"skill-candidate", "canonical-candidate"}:
        if normalized_scope == "canonical-candidate" and not str(test_plan or "").strip():
            raise ValueError("canonical-candidate promotion requires test_plan")
        return generate_skill_reinforcement_candidate(
            project_root,
            task_id=task_id,
            min_support=min_support,
            scope=normalized_scope,
            test_plan=test_plan,
        )
    if normalized_scope == "user-global":
        if not approved:
            raise ValueError("user-global promotion requires explicit approval")
        return {
            "status": "manual_global_review_required",
            "scope": normalized_scope,
            "project_dir": str(Path(project_root).expanduser().resolve()),
        }
    if normalized_scope == "local":
        return {
            "status": "manual_guidance_review_required",
            "scope": normalized_scope,
            "project_dir": str(Path(project_root).expanduser().resolve()),
        }
    raise ValueError(
        "scope must be one of: local, user-global, skill-candidate, canonical-candidate"
    )


def experience_metrics(project_root: Path) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    rows, malformed_count = _load_experience_rows(root)
    by_task: dict[str, dict[str, Any]] = {}
    failure_modes: dict[str, int] = {}
    worker = {
        "merge_failure_count": 0,
        "final_review_block_count": 0,
    }
    guidance = {
        "proposal_runs": 0,
        "accepted_runs": 0,
        "acceptance_rate": 0.0,
    }
    subject_routing = {
        "action_count": 0,
        "confirmation_count": 0,
        "dismissal_count": 0,
        "correction_count": 0,
        "confirmation_rate": 0.0,
        "dismissal_rate": 0.0,
        "correction_rate": 0.0,
    }
    review = {
        "blocker_count": 0,
        "blocked_review_runs": 0,
    }
    literature_diagnostics = {
        "checked_runs": 0,
        "failure_count": 0,
        "failure_rate": 0.0,
    }
    for record in rows:
        task_id = str(_task_from_record(record).get("task_id", "") or "unknown")
        current = by_task.setdefault(
            task_id,
            {
                "total_runs": 0,
                "passed": 0,
                "failed": 0,
                "blocked": 0,
                "missing_artifact_runs": 0,
                "pass_rate": 0.0,
                "missing_artifact_rate": 0.0,
            },
        )
        current["total_runs"] += 1
        status = _validator_status_from_record(record).lower()
        if status == "passed":
            current["passed"] += 1
        elif status == "blocked":
            current["blocked"] += 1
        elif status == "failed":
            current["failed"] += 1
        if _string_list(_outputs_from_record(record).get("missing_outputs")):
            current["missing_artifact_runs"] += 1
        for failure_mode in _failure_modes_from_record(record):
            failure_modes[failure_mode] = failure_modes.get(failure_mode, 0) + 1
        execution = _execution_from_record(record)
        if str(execution.get("worker_merge_status", "")).lower() in {"failed", "blocked"}:
            worker["merge_failure_count"] += 1
        if str(execution.get("worker_final_review_status", "")).lower() == "blocked":
            worker["final_review_block_count"] += 1

        guidance_update = _guidance_update_from_record(record)
        if _guidance_update_was_proposed(guidance_update):
            guidance["proposal_runs"] += 1
            if _truthy(guidance_update.get("applied")):
                guidance["accepted_runs"] += 1

        subject_action = _subject_lifecycle_action(record)
        if subject_action:
            subject_routing["action_count"] += 1
            if subject_action in {"confirm", "lock"}:
                subject_routing["confirmation_count"] += 1
            if subject_action == "dismiss":
                subject_routing["dismissal_count"] += 1
            if subject_action in {"dismiss", "reset", "unlock"}:
                subject_routing["correction_count"] += 1

        blocking_issues = _review_blocking_issues(record)
        review["blocker_count"] += len(blocking_issues)
        if str(_quality_from_record(record).get("review_status", "")).lower() == "blocked":
            review["blocked_review_runs"] += 1

        if _is_literature_diagnostic_checked(record):
            literature_diagnostics["checked_runs"] += 1
            if _is_literature_diagnostic_failure(record):
                literature_diagnostics["failure_count"] += 1

    for current in by_task.values():
        total = max(1, int(current["total_runs"]))
        current["pass_rate"] = round(float(current["passed"]) / total, 4)
        current["missing_artifact_rate"] = round(
            float(current["missing_artifact_runs"]) / total,
            4,
        )
    guidance_total = max(1, guidance["proposal_runs"])
    guidance["acceptance_rate"] = round(float(guidance["accepted_runs"]) / guidance_total, 4)
    subject_total = max(1, subject_routing["action_count"])
    subject_routing["confirmation_rate"] = round(
        float(subject_routing["confirmation_count"]) / subject_total,
        4,
    )
    subject_routing["dismissal_rate"] = round(
        float(subject_routing["dismissal_count"]) / subject_total,
        4,
    )
    subject_routing["correction_rate"] = round(
        float(subject_routing["correction_count"]) / subject_total,
        4,
    )
    diagnostic_total = max(1, literature_diagnostics["checked_runs"])
    literature_diagnostics["failure_rate"] = round(
        float(literature_diagnostics["failure_count"]) / diagnostic_total,
        4,
    )

    return {
        "project_dir": str(root),
        "run_count": len(rows),
        "malformed_count": malformed_count,
        "validator": {"by_task": by_task},
        "failure_modes": failure_modes,
        "worker": worker,
        "guidance": guidance,
        "subject_routing": subject_routing,
        "review": review,
        "literature_diagnostics": literature_diagnostics,
    }


def experience_schema_compatibility(project_root: Path) -> dict[str, Any]:
    root = Path(project_root).expanduser().resolve()
    index_path = root / EXPERIENCE_INDEX_REL
    errors: list[str] = []
    checked_records = 0
    malformed_count = 0

    if not index_path.is_file():
        return {
            "project_dir": str(root),
            "ok": True,
            "checked_records": 0,
            "malformed_count": 0,
            "errors": [],
        }

    for line_number, line in enumerate(index_path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        source = f"{_rel(root, index_path)}:{line_number}"
        try:
            parsed = json.loads(line)
        except json.JSONDecodeError as exc:
            malformed_count += 1
            errors.append(f"{source}: malformed JSON: {exc.msg}")
            continue
        if not isinstance(parsed, dict):
            malformed_count += 1
            errors.append(f"{source}: experience record must be an object")
            continue
        checked_records += 1
        errors.extend(_experience_schema_errors(parsed, source=source))

        run_id = str(parsed.get("run_id", "") or "").strip()
        if not run_id:
            continue
        record_path = _record_path_for_run(root, run_id)
        if not record_path.is_file():
            continue
        record_source = _rel(root, record_path)
        try:
            record = json.loads(record_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            malformed_count += 1
            errors.append(f"{record_source}: malformed JSON: {exc.msg}")
            continue
        if not isinstance(record, dict):
            malformed_count += 1
            errors.append(f"{record_source}: experience record must be an object")
            continue
        checked_records += 1
        errors.extend(_experience_schema_errors(record, source=record_source, expected_run_id=run_id))

    return {
        "project_dir": str(root),
        "ok": not errors,
        "checked_records": checked_records,
        "malformed_count": malformed_count,
        "errors": errors,
    }


def _experience_schema_errors(
    record: Mapping[str, Any],
    *,
    source: str,
    expected_run_id: str | None = None,
) -> list[str]:
    errors: list[str] = []
    schema_version = record.get("schema_version")
    if schema_version != SCHEMA_VERSION:
        errors.append(f"{source}: schema_version must be {SCHEMA_VERSION}")
    run_id = str(record.get("run_id", "") or "").strip()
    if not run_id:
        errors.append(f"{source}: run_id must be a non-empty string")
    elif expected_run_id is not None and run_id != expected_run_id:
        errors.append(f"{source}: run_id {run_id!r} does not match index run_id {expected_run_id!r}")
    for field in REQUIRED_EXPERIENCE_OBJECT_FIELDS:
        if not isinstance(record.get(field), Mapping):
            errors.append(f"{source}: missing required object: {field}")
    return errors


def _index_row(record: dict[str, Any]) -> dict[str, Any]:
    return {
        "schema_version": record.get("schema_version", SCHEMA_VERSION),
        "run_id": record.get("run_id", ""),
        "created_at": record.get("created_at", ""),
        "task": record.get("task", {}),
        "execution": record.get("execution", {}),
        "quality": record.get("quality", {}),
        "outputs": record.get("outputs", {}),
        "experience": record.get("experience", {}),
        "privacy": record.get("privacy", {}),
    }


def _supported_failure_patterns(
    rows: list[dict[str, Any]],
    *,
    task_id: str | None,
    min_support: int,
) -> list[dict[str, Any]]:
    normalized_task = str(task_id or "").strip().upper()
    groups: dict[tuple[str, str], list[dict[str, Any]]] = {}
    for record in rows:
        record_task = str(_task_from_record(record).get("task_id", "")).strip().upper()
        if normalized_task and record_task != normalized_task:
            continue
        for failure_mode in _failure_modes_from_record(record):
            groups.setdefault((record_task, failure_mode), []).append(record)
    patterns = [
        {
            "task_id": key[0],
            "failure_mode": key[1],
            "records": records,
        }
        for key, records in groups.items()
        if len(records) >= max(1, int(min_support))
    ]
    return sorted(
        patterns,
        key=lambda item: (-len(item["records"]), str(item["task_id"]), str(item["failure_mode"])),
    )


def _affected_skill_ids(records: list[dict[str, Any]]) -> list[str]:
    skills: list[str] = []
    for record in records:
        inputs = _inputs_from_record(record)
        skills.extend(_string_list(inputs.get("required_skills")))
        task = _task_from_record(record)
        skills.extend(_string_list(task.get("required_skills")))
        skills.extend(_string_list(record.get("required_skills")))
    return _unique_strings(skills) or ["unknown"]


def _skill_reinforcement_candidate_text(
    *,
    scope: str,
    task_id: str,
    failure_mode: str,
    affected_skill_ids: list[str],
    supporting_run_ids: list[str],
    support_count: int,
    test_plan: str,
) -> str:
    test_text = str(test_plan or "").strip() or (
        "Add or update a regression test that reproduces the repeated failure "
        "before changing canonical skill source."
    )
    return "\n".join(
        [
            "# Skill Reinforcement Candidate",
            "",
            f"- scope: {scope}",
            f"- task_id: {task_id}",
            f"- support_count: {support_count}",
            f"- failure_mode: {failure_mode}",
            "",
            "## Affected Skill IDs",
            "",
            *[f"- {skill_id}" for skill_id in affected_skill_ids],
            "",
            "## Supporting Experience Records",
            "",
            *[f"- {run_id}" for run_id in supporting_run_ids],
            "",
            "## Repeated Failure Or Improvement Pattern",
            "",
            (
                f"- Task {task_id} repeatedly produced `{failure_mode}` across "
                f"{support_count} local experience records."
            ),
            "",
            "## Proposed Canonical Source Change",
            "",
            "- Strengthen the affected skill source only through normal repository edits.",
            "- Update `content/skills-core.md` if the behavior must be visible in compact runtime guidance.",
            "- Do not edit generated workflow payloads by hand.",
            "",
            "## Expected Behavior Change",
            "",
            "- Future runs should satisfy the repeated missing or blocked requirement without project-local patches.",
            "",
            "## Required Eval Or Regression Test",
            "",
            f"- {test_text}",
            "",
            "## Rollback Path",
            "",
            "- Revert the reviewed source and test changes if the eval fails or causes regressions.",
            "- Keep the local experience records as audit evidence.",
            "",
        ]
    )


def _load_experience_rows(root: Path) -> tuple[list[dict[str, Any]], int]:
    index_path = root / EXPERIENCE_INDEX_REL
    if not index_path.is_file():
        return [], 0
    rows: list[dict[str, Any]] = []
    malformed_count = 0
    for line in index_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            parsed = json.loads(line)
        except json.JSONDecodeError:
            malformed_count += 1
            continue
        if isinstance(parsed, dict):
            rows.append(parsed)
        else:
            malformed_count += 1
    return rows, malformed_count


def _query_filters(**values: Any) -> dict[str, Any]:
    filters: dict[str, Any] = {}
    for key, value in values.items():
        if key == "limit":
            filters[key] = int(value or 20)
            continue
        text = str(value or "").strip()
        if text:
            filters[key] = text
    return filters


def _record_matches(record: dict[str, Any], filters: dict[str, Any]) -> bool:
    checks = {
        "task_id": _task_from_record(record).get("task_id", ""),
        "stage": _task_from_record(record).get("stage", ""),
        "topic": _task_from_record(record).get("topic", ""),
        "validator_status": _validator_status_from_record(record),
        "worker_mode": _execution_from_record(record).get("worker_mode", ""),
    }
    for key, value in checks.items():
        expected = str(filters.get(key, "")).strip()
        if expected and str(value).strip().lower() != expected.lower():
            return False

    subject = str(filters.get("subject", "")).strip()
    if subject and subject.lower() not in _subject_text(record).lower():
        return False

    failure_mode = str(filters.get("failure_mode", "")).strip()
    if failure_mode and failure_mode not in _failure_modes_from_record(record):
        return False

    guidance_source = str(filters.get("guidance_source", "")).strip()
    if guidance_source and guidance_source.lower() not in _guidance_source_text(record).lower():
        return False

    return True


def _record_path_for_run(root: Path, run_id: str) -> Path:
    return root / ".qiongli" / "trace" / "runs" / run_id / EXPERIENCE_RECORD_NAME


def _task_from_record(record: dict[str, Any]) -> dict[str, Any]:
    task = record.get("task")
    if isinstance(task, Mapping):
        return dict(task)
    return {
        "task_id": record.get("task_id", ""),
        "paper_type": record.get("paper_type", ""),
        "topic": record.get("topic", ""),
        "workflow": record.get("workflow", ""),
        "stage": record.get("stage", ""),
    }


def _execution_from_record(record: dict[str, Any]) -> dict[str, Any]:
    return _mapping(record.get("execution"))


def _outputs_from_record(record: dict[str, Any]) -> dict[str, Any]:
    outputs = _mapping(record.get("outputs"))
    if outputs:
        return outputs
    return {
        "required_outputs": _string_list(record.get("required_outputs")),
        "found_outputs": _string_list(record.get("found_outputs")),
        "missing_outputs": _string_list(record.get("missing_outputs")),
        "trace_files": [],
    }


def _experience_from_record(record: dict[str, Any]) -> dict[str, Any]:
    return _mapping(record.get("experience"))


def _inputs_from_record(record: dict[str, Any]) -> dict[str, Any]:
    return _mapping(record.get("inputs"))


def _quality_from_record(record: dict[str, Any]) -> dict[str, Any]:
    return _mapping(record.get("quality"))


def _validator_status_from_record(record: dict[str, Any]) -> str:
    quality = _quality_from_record(record)
    return str(quality.get("validator_status", record.get("validator_status", "unknown")))


def _failure_modes_from_record(record: dict[str, Any]) -> list[str]:
    return _string_list(_experience_from_record(record).get("failure_modes"))


def _guidance_update_from_record(record: dict[str, Any]) -> dict[str, Any]:
    experience = _experience_from_record(record)
    guidance_update = experience.get("guidance_update")
    if isinstance(guidance_update, Mapping):
        return dict(guidance_update)
    return {
        "proposal_path": record.get("guidance_proposal", ""),
        "applied": record.get("applied_guidance_update", False),
        "mode": record.get("guidance_mode", ""),
    }


def _guidance_update_was_proposed(guidance_update: Mapping[str, Any]) -> bool:
    proposal_path = str(guidance_update.get("proposal_path", "") or "").strip()
    if proposal_path:
        return True
    return _truthy(guidance_update.get("proposed"))


def _guidance_sources_from_record(record: dict[str, Any]) -> list[dict[str, Any]]:
    inputs = _inputs_from_record(record)
    sources = inputs.get("guidance_sources", record.get("guidance_sources", []))
    return _dict_list(sources)


def _guidance_source_text(record: dict[str, Any]) -> str:
    return json.dumps(_guidance_sources_from_record(record), ensure_ascii=False, sort_keys=True)


def _subject_text(record: dict[str, Any]) -> str:
    inputs = _inputs_from_record(record)
    subject_refinement = inputs.get("subject_refinement", {})
    return json.dumps(subject_refinement, ensure_ascii=False, sort_keys=True)


def _subject_lifecycle_action(record: dict[str, Any]) -> str:
    inputs = _inputs_from_record(record)
    subject_refinement = inputs.get("subject_refinement")
    if not isinstance(subject_refinement, Mapping):
        subject_refinement = record.get("subject_refinement")
    if not isinstance(subject_refinement, Mapping):
        return ""
    evidence_sources = subject_refinement.get("evidence_sources")
    if not isinstance(evidence_sources, Mapping):
        return ""
    user_action = evidence_sources.get("user_action")
    if not isinstance(user_action, Mapping):
        return ""
    latest_action = user_action.get("latest_action")
    if not isinstance(latest_action, Mapping):
        return ""
    return str(latest_action.get("action", "") or "").strip().lower()


def _review_blocking_issues(record: dict[str, Any]) -> list[str]:
    return _string_list(_quality_from_record(record).get("blocking_issues"))


def _is_literature_diagnostic_checked(record: dict[str, Any]) -> bool:
    task_id = str(_task_from_record(record).get("task_id", "") or "").strip().upper()
    if task_id.startswith("B"):
        return True
    searchable = " ".join(
        [
            " ".join(_string_list(_outputs_from_record(record).get("required_outputs"))),
            " ".join(_string_list(_outputs_from_record(record).get("found_outputs"))),
            " ".join(_string_list(_outputs_from_record(record).get("missing_outputs"))),
            " ".join(_failure_modes_from_record(record)),
        ]
    ).lower()
    return "search_diagnostics" in searchable or "literature_diagnostic" in searchable


def _is_literature_diagnostic_failure(record: dict[str, Any]) -> bool:
    searchable = " ".join(
        [
            " ".join(_string_list(_outputs_from_record(record).get("missing_outputs"))),
            " ".join(_failure_modes_from_record(record)),
        ]
    ).lower()
    if (
        "search_diagnostics" in searchable
        or "literature_diagnostic" in searchable
        or "provider_diagnostic" in searchable
    ):
        return True
    provider_status = _inputs_from_record(record).get("provider_status")
    return _has_unhealthy_provider_status(provider_status)


def _has_unhealthy_provider_status(value: Any) -> bool:
    if isinstance(value, Mapping):
        status = str(value.get("status", "") or "").strip().lower()
        if status in {"failed", "failure", "error", "unavailable", "blocked"}:
            return True
        ok = value.get("ok")
        if ok is False:
            return True
        return any(_has_unhealthy_provider_status(item) for item in value.values())
    if isinstance(value, list):
        return any(_has_unhealthy_provider_status(item) for item in value)
    return False


def _lesson_record(record: dict[str, Any]) -> dict[str, Any]:
    experience = _experience_from_record(record)
    return {
        "run_id": str(record.get("run_id", "")),
        "task": _task_from_record(record),
        "validator_status": _validator_status_from_record(record),
        "failure_modes": _string_list(experience.get("failure_modes")),
        "lessons": _string_list(experience.get("lessons")),
        "reusable_guidance": _string_list(experience.get("reusable_guidance")),
        "trace_path": _trace_path_from_record(record),
    }


def _prior_record(record: dict[str, Any]) -> dict[str, Any]:
    lesson = _lesson_record(record)
    return {
        "run_id": lesson["run_id"],
        "status": lesson["validator_status"],
        "failure_modes": lesson["failure_modes"],
        "reusable_guidance": lesson["reusable_guidance"],
        "trace_path": lesson["trace_path"],
    }


def _trace_path_from_record(record: dict[str, Any]) -> str:
    outputs = _outputs_from_record(record)
    trace_files = _string_list(outputs.get("trace_files"))
    if trace_files:
        first = trace_files[0]
        marker = "/runs/"
        if marker in first:
            prefix, _, rest = first.partition(marker)
            run_id = rest.split("/", 1)[0]
            return f"{prefix}{marker}{run_id}/{EXPERIENCE_RECORD_NAME}"
    run_id = str(record.get("run_id", "")).strip()
    return f".qiongli/trace/runs/{run_id}/{EXPERIENCE_RECORD_NAME}" if run_id else ""


def _replay_recommendation(
    status: str,
    failure_modes: list[str],
    missing_outputs: Any,
) -> str:
    missing = _string_list(missing_outputs)
    if missing:
        return "Address missing required outputs before rerun: " + ", ".join(missing)
    if failure_modes:
        return "Review failure modes before rerun: " + ", ".join(failure_modes)
    if status in {"failed", "blocked"}:
        return f"Review validator status before rerun: {status}"
    return "Prior run did not record blocking failures."


def _trace_files(root: Path, run_dir: Path) -> list[str]:
    resolved_run_dir = Path(run_dir).expanduser().resolve()
    if not resolved_run_dir.is_dir():
        return []
    return sorted(
        _rel(root, path)
        for path in resolved_run_dir.iterdir()
        if path.is_file() and path.name != EXPERIENCE_RECORD_NAME
    )


def _validator_status(validator_gate: dict[str, Any]) -> str:
    if not validator_gate:
        return "unknown"
    if bool(validator_gate.get("passed")):
        return "passed"
    return "failed"


def _review_status(worker_state: Mapping[str, Any]) -> str:
    merge_review_status = worker_state.get("merge_review_status")
    if merge_review_status:
        return str(merge_review_status)
    review = worker_state.get("review")
    if isinstance(review, Mapping):
        status = review.get("status")
        if status:
            return str(status)
    status = worker_state.get("review_status")
    return str(status or "unknown")


def _blocking_issues(
    validator_gate: dict[str, Any],
    worker_state: Mapping[str, Any],
) -> list[str]:
    issues = [f"missing_required_output:{item}" for item in _string_list(validator_gate.get("missing"))]
    worker_issues = worker_state.get("blocking_issues")
    if isinstance(worker_issues, list):
        issues.extend(str(item) for item in worker_issues if str(item).strip())
    return _unique_strings(issues)


def _failure_modes(
    missing_outputs: list[str],
    validator_gate: dict[str, Any],
    worker_state: Mapping[str, Any],
) -> list[str]:
    modes = [f"missing_required_output:{item}" for item in missing_outputs]
    if validator_gate and not bool(validator_gate.get("passed")):
        modes.append("validator_gate_failed")
    if str(worker_state.get("status", "")).lower() in {"failed", "error"}:
        modes.append("worker_orchestration_failed")
    if str(worker_state.get("barrier_status", "")).lower() == "blocked":
        modes.append("worker_barrier_blocked")
    merge_status = str(worker_state.get("merge_status", "")).lower()
    if merge_status in {"failed", "blocked"}:
        modes.append(f"worker_merge_{merge_status}")
    final_review_status = str(worker_state.get("merge_review_status", "")).lower()
    if final_review_status in {"failed", "blocked"}:
        modes.append(f"worker_final_review_{final_review_status}")
    return _unique_strings(modes)


def _execution_mode_from_worker(worker_state: Mapping[str, Any]) -> str:
    mode = str(worker_state.get("mode", worker_state.get("orchestration_mode", ""))).strip()
    if mode and mode != "none":
        return "worker"
    return "solo"


def _worker_count(worker_state: Mapping[str, Any]) -> int:
    workers = worker_state.get("workers")
    return len(workers) if isinstance(workers, list) else 0


def _mapping(value: Any) -> dict[str, Any]:
    return dict(value) if isinstance(value, Mapping) else {}


def _string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if str(item).strip()]


def _dict_list(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        return []
    return [dict(item) for item in value if isinstance(item, Mapping)]


def _safe_float(value: Any) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return 0.0


def _truthy(value: Any) -> bool:
    if isinstance(value, bool):
        return value
    return str(value or "").strip().lower() in {"1", "true", "yes", "y", "applied", "accepted"}


def _unique_strings(values: list[str]) -> list[str]:
    seen: set[str] = set()
    unique: list[str] = []
    for value in values:
        text = str(value).strip()
        if not text or text in seen:
            continue
        seen.add(text)
        unique.append(text)
    return unique


def _rel(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return str(path)


def _utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _utc_date() -> str:
    return datetime.now(timezone.utc).date().isoformat()
