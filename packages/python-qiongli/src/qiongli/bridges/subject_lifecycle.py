from __future__ import annotations

import json
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Mapping

from .project_manifest import (
    OFFICIAL_SUBJECTS,
    load_project_manifest,
    update_project_manifest,
)
from .subject_guidance import (
    END_MARKER,
    START_MARKER,
    SubjectGuidanceError,
    disable_subject_guidance,
    inspect_subject_guidance,
    write_subject_guidance,
)


ACTIONS = {"confirm", "dismiss", "reset", "lock", "unlock"}
STATE_REL = Path(".qiongli") / "trace" / "subject_evidence.json"
GUIDANCE_WRITE_ACTIONS = {"confirm", "lock", "unlock", "reset"}
ACTIVE_GUIDANCE_WRITE_ACTIONS = {"confirm", "lock"}
LIFECYCLE_WRITE_RELS = (
    Path(".qiongli"),
    Path(".qiongli") / "guidance_manifest.yaml",
    Path(".qiongli") / "trace",
    STATE_REL,
    Path(".qiongli") / "guidance.d",
    Path(".qiongli") / "guidance.d" / "subject-runtime.md",
)
CONCRETE_OFFICIAL_SUBJECTS = set(OFFICIAL_SUBJECTS) - {"auto", "core"}


class SubjectLifecycleError(ValueError):
    pass


def subject_status(project_root: Path) -> dict[str, Any]:
    root = _normalize_project_root(project_root)
    manifest_state = load_project_manifest(root)
    return _status_packet(root, manifest_state=manifest_state, state=_load_state(root))


def apply_subject_action(
    project_root: Path,
    action: str,
    subject: str | None = None,
    *,
    source: str = "user",
    run_id: str | None = None,
) -> dict[str, Any]:
    root = _normalize_project_root(project_root)
    normalized_action = _validate_action(action)
    normalized_subject = _validate_subject_for_action(normalized_action, subject)
    _preflight_lifecycle_symlinks(root)
    state = _load_state(root)
    manifest_state = load_project_manifest(root)
    _preflight_subject_guidance_write(root, action=normalized_action)
    _preflight_active_subject_guidance_render(
        manifest_state,
        action=normalized_action,
    )

    if normalized_action == "confirm":
        manifest_state = update_project_manifest(
            root,
            active_subject=normalized_subject,
            subject_mode="confirmed",
        )
        _update_subject_guidance(
            root,
            manifest_state=manifest_state,
            action=normalized_action,
            source=source,
            run_id=run_id,
        )
    elif normalized_action == "lock":
        manifest_state = update_project_manifest(
            root,
            active_subject=normalized_subject,
            subject_mode="locked",
        )
        _update_subject_guidance(
            root,
            manifest_state=manifest_state,
            action=normalized_action,
            source=source,
            run_id=run_id,
        )
    elif normalized_action == "unlock":
        active_subject = manifest_state.manifest.active_subject
        if active_subject in {"auto", "core"}:
            manifest_state = update_project_manifest(
                root,
                active_subject="auto",
                subject_mode="auto",
            )
        else:
            manifest_state = update_project_manifest(root, subject_mode="confirmed")
        _update_subject_guidance(
            root,
            manifest_state=manifest_state,
            action=normalized_action,
            source=source,
            run_id=run_id,
        )
    elif normalized_action == "dismiss":
        _dismiss_subject(
            state,
            subject=normalized_subject,
            source=source,
            run_id=run_id,
            created_at=_timestamp(),
        )
    elif normalized_action == "reset":
        manifest_state = update_project_manifest(
            root,
            active_subject="auto",
            subject_mode="auto",
            secondary_subjects=[],
            venue_profiles=[],
            method_lenses=[],
            strictness="standard",
        )
        state["subjects"] = {}
        state["dismissed_subjects"] = {}
        _disable_subject_guidance(
            root,
            action=normalized_action,
            source=source,
            run_id=run_id,
        )

    _append_event(
        state,
        action=normalized_action,
        subject=normalized_subject,
        source=source,
        run_id=run_id,
    )
    _write_state(root, state)
    return _status_packet(root, manifest_state=manifest_state, state=state)


def _preflight_lifecycle_symlinks(project_root: Path) -> None:
    for relative_path in LIFECYCLE_WRITE_RELS:
        if (project_root / relative_path).is_symlink():
            raise SubjectLifecycleError(
                f"Refusing lifecycle write through symlink at {relative_path.as_posix()}"
            )


def _preflight_subject_guidance_write(project_root: Path, *, action: str) -> None:
    if action not in GUIDANCE_WRITE_ACTIONS:
        return
    guidance_status = inspect_subject_guidance(project_root)
    if guidance_status.get("managed_block") != "invalid":
        return
    warnings = guidance_status.get("warnings")
    if isinstance(warnings, list):
        reason = "; ".join(
            warning.strip() for warning in warnings if isinstance(warning, str) and warning.strip()
        )
    else:
        reason = ""
    if not reason:
        reason = "invalid subject guidance"
    raise SubjectLifecycleError(f"Failed to update subject guidance: {reason}")


def _preflight_active_subject_guidance_render(manifest_state: Any, *, action: str) -> None:
    if action not in ACTIVE_GUIDANCE_WRITE_ACTIONS and not (
        action == "unlock" and _has_concrete_active_subject(manifest_state)
    ):
        return
    for lens in manifest_state.manifest.method_lenses or []:
        if START_MARKER in lens or END_MARKER in lens:
            raise SubjectLifecycleError(
                "Failed to update subject guidance: managed marker is not allowed "
                "in subject guidance values"
            )


def _has_concrete_active_subject(manifest_state: Any) -> bool:
    manifest = manifest_state.manifest
    return manifest.active_subject not in {"auto", "core"} and manifest.subject_mode != "auto"


def _update_subject_guidance(
    project_root: Path,
    *,
    manifest_state: Any,
    action: str,
    source: str,
    run_id: str | None,
) -> None:
    manifest = manifest_state.manifest
    if manifest.active_subject in {"auto", "core"} or manifest.subject_mode == "auto":
        _disable_subject_guidance(
            project_root,
            action=action,
            source=source,
            run_id=run_id,
        )
        return
    try:
        write_subject_guidance(
            project_root,
            active_subject=manifest.active_subject,
            subject_mode=manifest.subject_mode,
            lifecycle_action=action,
            source=source,
            run_id=run_id,
            method_lenses=manifest.method_lenses or [],
        )
    except SubjectGuidanceError as exc:
        raise SubjectLifecycleError(f"Failed to update subject guidance: {exc}") from exc


def _disable_subject_guidance(
    project_root: Path,
    *,
    action: str,
    source: str,
    run_id: str | None,
) -> None:
    try:
        disable_subject_guidance(
            project_root,
            lifecycle_action=action,
            source=source,
            run_id=run_id,
        )
    except SubjectGuidanceError as exc:
        raise SubjectLifecycleError(f"Failed to update subject guidance: {exc}") from exc


def _status_packet(
    project_root: Path,
    *,
    manifest_state: Any,
    state: dict[str, Any],
) -> dict[str, Any]:
    packet = manifest_state.to_packet()
    return {
        "project_root": str(project_root.resolve()),
        "manifest": packet["manifest"],
        "manifest_exists": manifest_state.exists,
        "subject_guidance": inspect_subject_guidance(project_root),
        "state": state,
    }


def _load_state(project_root: Path) -> dict[str, Any]:
    path = project_root / STATE_REL
    empty = _empty_state()
    if not path.is_file():
        return empty
    try:
        loaded = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return {
            **empty,
            "warnings": [
                f"Invalid subject evidence memory at {_rel(project_root, path)}: {exc}"
            ],
        }
    if not isinstance(loaded, Mapping):
        return {
            **empty,
            "warnings": [
                f"Invalid subject evidence memory at {_rel(project_root, path)}: expected object"
            ],
        }
    return _normalize_state(loaded)


def _normalize_state(loaded: Mapping[Any, Any]) -> dict[str, Any]:
    state: dict[str, Any] = dict(loaded)
    state["schema_version"] = "1.0"

    subjects = loaded.get("subjects")
    state["subjects"] = dict(subjects) if isinstance(subjects, Mapping) else {}

    dismissed_subjects = loaded.get("dismissed_subjects")
    state["dismissed_subjects"] = (
        dict(dismissed_subjects) if isinstance(dismissed_subjects, Mapping) else {}
    )

    events = loaded.get("lifecycle_events")
    state["lifecycle_events"] = list(events) if isinstance(events, list) else []

    warnings = _string_list(loaded.get("warnings"))
    if warnings:
        state["warnings"] = warnings
    else:
        state.pop("warnings", None)
    if subjects is not None and not isinstance(subjects, Mapping):
        _state_warnings(state).append(
            "Invalid subject evidence memory: expected subjects object"
        )
    if dismissed_subjects is not None and not isinstance(dismissed_subjects, Mapping):
        _state_warnings(state).append(
            "Invalid subject evidence memory: expected dismissed_subjects object"
        )
    if events is not None and not isinstance(events, list):
        _state_warnings(state).append(
            "Invalid subject evidence memory: expected lifecycle_events list"
        )
    if "warnings" in state:
        state["warnings"] = _unique_strings(state["warnings"])
    return state


def _empty_state() -> dict[str, Any]:
    return {
        "schema_version": "1.0",
        "subjects": {},
        "dismissed_subjects": {},
        "lifecycle_events": [],
    }


def _write_state(project_root: Path, state: Mapping[str, Any]) -> None:
    path = project_root / STATE_REL
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(dict(state), ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def _dismiss_subject(
    state: dict[str, Any],
    *,
    subject: str,
    source: str,
    run_id: str | None,
    created_at: str,
) -> None:
    dismissed_subjects = state.setdefault("dismissed_subjects", {})
    if not isinstance(dismissed_subjects, dict):
        dismissed_subjects = {}
        state["dismissed_subjects"] = dismissed_subjects
    dismissed_subjects[subject] = {
        "source": source,
        "run_id": run_id,
        "created_at": created_at,
        "last_suggestion_count": _last_suggestion_count(state, subject),
    }


def _last_suggestion_count(state: Mapping[str, Any], subject: str) -> int:
    subjects = state.get("subjects", {})
    record = subjects.get(subject, {}) if isinstance(subjects, Mapping) else {}
    value = record.get("suggestion_count", 0) if isinstance(record, Mapping) else 0
    try:
        if isinstance(value, bool):
            raise TypeError("boolean is not a valid count")
        count = int(value)
    except (TypeError, ValueError, OverflowError):
        return 0
    return max(count, 0)


def _append_event(
    state: dict[str, Any],
    *,
    action: str,
    subject: str | None,
    source: str,
    run_id: str | None,
) -> None:
    events = state.setdefault("lifecycle_events", [])
    if not isinstance(events, list):
        events = []
        state["lifecycle_events"] = events
    events.append(
        {
            "action": action,
            "subject": subject,
            "source": source,
            "run_id": run_id,
            "created_at": _timestamp(),
        }
    )


def _validate_action(action: str) -> str:
    if not isinstance(action, str) or action not in ACTIONS:
        raise SubjectLifecycleError(f"Unsupported subject lifecycle action: {action!r}")
    return action


def _validate_subject_for_action(action: str, subject: str | None) -> str | None:
    if action in {"reset", "unlock"}:
        if subject is None:
            return None
        if not isinstance(subject, str):
            raise SubjectLifecycleError(f"{action} does not accept a subject")
        if not subject.strip():
            return None
        raise SubjectLifecycleError(f"{action} does not accept a subject")
    if action not in {"confirm", "dismiss", "lock"}:
        return subject
    if subject is None:
        raise SubjectLifecycleError(f"{action} requires a subject")
    if not isinstance(subject, str):
        raise SubjectLifecycleError(f"{action} requires a subject")
    if not subject.strip():
        raise SubjectLifecycleError(f"{action} requires a subject")
    normalized = subject.strip()
    if normalized not in CONCRETE_OFFICIAL_SUBJECTS:
        raise SubjectLifecycleError(
            f"{action} requires a concrete official subject; got {normalized!r}"
        )
    return normalized


def _string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item.strip() for item in value if isinstance(item, str) and item.strip()]


def _state_warnings(state: dict[str, Any]) -> list[str]:
    warnings = state.setdefault("warnings", [])
    if not isinstance(warnings, list):
        warnings = []
        state["warnings"] = warnings
    return warnings


def _unique_strings(values: list[str]) -> list[str]:
    unique: list[str] = []
    for value in values:
        if value not in unique:
            unique.append(value)
    return unique


def _timestamp() -> str:
    return datetime.now(UTC).isoformat()


def _normalize_project_root(project_root: Path) -> Path:
    return Path(project_root).expanduser().resolve()


def _rel(project_root: Path, path: Path) -> str:
    try:
        return path.relative_to(project_root).as_posix()
    except ValueError:
        return str(path)
