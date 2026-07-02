from __future__ import annotations

from collections.abc import Mapping, Sequence
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


SUBJECT_GUIDANCE_REL = Path(".qiongli") / "guidance.d" / "subject-runtime.md"
START_MARKER = "<!-- qiongli:subject-runtime:start -->"
END_MARKER = "<!-- qiongli:subject-runtime:end -->"


class SubjectGuidanceError(ValueError):
    pass


def inspect_subject_guidance(project_root: Path) -> dict[str, Any]:
    root = _normalize_project_root(project_root)
    path = root / SUBJECT_GUIDANCE_REL
    if not path.is_file():
        return _status_packet(
            root,
            exists=False,
            managed_block="missing",
            active_subject=None,
            subject_mode=None,
            updated_at=None,
            warnings=[],
        )

    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        return _status_packet(
            root,
            exists=True,
            managed_block="invalid",
            active_subject=None,
            subject_mode=None,
            updated_at=None,
            warnings=[f"Failed to read {_rel(root, path)}: {exc}"],
        )

    block = _find_managed_block(text)
    if block.status == "absent":
        return _status_packet(
            root,
            exists=True,
            managed_block="absent",
            active_subject=None,
            subject_mode=None,
            updated_at=None,
            warnings=[],
        )
    if block.status == "invalid":
        return _status_packet(
            root,
            exists=True,
            managed_block="invalid",
            active_subject=None,
            subject_mode=None,
            updated_at=None,
            warnings=[block.reason],
        )

    metadata = _parse_metadata(text[block.start : block.end])
    metadata_warning = _metadata_warning(metadata)
    if metadata_warning:
        return _status_packet(
            root,
            exists=True,
            managed_block="invalid",
            active_subject=None,
            subject_mode=None,
            updated_at=None,
            warnings=[metadata_warning],
        )
    managed_block = "disabled" if metadata.get("status") == "disabled" else "active"
    return _status_packet(
        root,
        exists=True,
        managed_block=managed_block,
        active_subject=metadata.get("active_subject"),
        subject_mode=metadata.get("subject_mode"),
        updated_at=metadata.get("updated_at"),
        warnings=[],
    )


def write_subject_guidance(
    project_root: Path,
    *,
    active_subject: str,
    subject_mode: str,
    lifecycle_action: str,
    source: str,
    run_id: str | None = None,
    method_lenses: Sequence[str] | None = None,
    resource_activation_plan: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    root = _normalize_project_root(project_root)
    updated_at = _timestamp()
    block = _render_active_block(
        active_subject=active_subject,
        subject_mode=subject_mode,
        lifecycle_action=lifecycle_action,
        source=source,
        run_id=run_id,
        updated_at=updated_at,
        method_lenses=method_lenses,
        resource_activation_plan=resource_activation_plan,
    )
    managed_block = _write_managed_block(root, block)
    return _status_packet(
        root,
        exists=True,
        managed_block=managed_block if managed_block == "appended" else "active",
        active_subject=active_subject,
        subject_mode=subject_mode,
        updated_at=updated_at,
        warnings=[],
    )


def disable_subject_guidance(
    project_root: Path,
    *,
    lifecycle_action: str,
    source: str,
    run_id: str | None = None,
) -> dict[str, Any]:
    root = _normalize_project_root(project_root)
    updated_at = _timestamp()
    block = _render_disabled_block(
        lifecycle_action=lifecycle_action,
        source=source,
        run_id=run_id,
        updated_at=updated_at,
    )
    _write_managed_block(root, block)
    return _status_packet(
        root,
        exists=True,
        managed_block="disabled",
        active_subject="auto",
        subject_mode="auto",
        updated_at=updated_at,
        warnings=[],
    )


class _ManagedBlock:
    def __init__(
        self,
        status: str,
        *,
        start: int | None = None,
        end: int | None = None,
        reason: str = "",
    ) -> None:
        self.status = status
        self.start = start
        self.end = end
        self.reason = reason


def _write_managed_block(project_root: Path, block: str) -> str:
    path = project_root / SUBJECT_GUIDANCE_REL
    try:
        _reject_symlinked_guidance_path(project_root)
        if not path.exists():
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(_default_document(block), encoding="utf-8")
            return "active"

        text = path.read_text(encoding="utf-8")
        existing = _find_managed_block(text)
        if existing.status == "invalid":
            raise SubjectGuidanceError(existing.reason)
        if existing.status == "absent":
            path.write_text(_append_document(text, block), encoding="utf-8")
            return "appended"

        assert existing.start is not None
        assert existing.end is not None
        metadata_warning = _metadata_warning(_parse_metadata(text[existing.start : existing.end]))
        if metadata_warning:
            raise SubjectGuidanceError(metadata_warning)
        path.write_text(text[: existing.start] + block + text[existing.end :], encoding="utf-8")
        return "active"
    except SubjectGuidanceError:
        raise
    except OSError as exc:
        raise SubjectGuidanceError(
            f"Failed to update {_rel(project_root, path)}: {exc}"
        ) from exc


def _find_managed_block(text: str) -> _ManagedBlock:
    start_count = text.count(START_MARKER)
    end_count = text.count(END_MARKER)
    if start_count == 0 and end_count == 0:
        return _ManagedBlock("absent")
    if start_count > 1 or end_count > 1:
        return _ManagedBlock("invalid", reason="multiple managed blocks found")
    if start_count != 1 or end_count != 1:
        return _ManagedBlock("invalid", reason="invalid marker order")

    start = text.find(START_MARKER)
    end_marker_start = text.find(END_MARKER)
    if end_marker_start < start:
        return _ManagedBlock("invalid", reason="invalid marker order")
    return _ManagedBlock("active", start=start, end=end_marker_start + len(END_MARKER))


def _render_active_block(
    *,
    active_subject: str,
    subject_mode: str,
    lifecycle_action: str,
    source: str,
    run_id: str | None,
    updated_at: str,
    method_lenses: Sequence[str] | None,
    resource_activation_plan: Mapping[str, Any] | None,
) -> str:
    clean_subject_mode = _clean_scalar(subject_mode)
    lines = [
        START_MARKER,
        "schema_version: 1.0",
        "managed_by: qiongli",
        f"active_subject: {_clean_scalar(active_subject)}",
        f"subject_mode: {clean_subject_mode}",
        f"updated_at: {updated_at}",
        f"updated_by: {_clean_scalar(source)}",
        f"lifecycle_action: {_clean_scalar(lifecycle_action)}",
    ]
    if run_id:
        lines.append(f"run_id: {_clean_scalar(run_id)}")
    lines.extend(
        [
            "",
            "## Active Subject",
            "",
            "- Use the canonical Qiongli workflow as the base.",
            (
                f"- Add the {_clean_scalar(active_subject)} subject layer when interpreting "
                "project-specific methods, evidence standards, venue norms, and quality checks."
            ),
            (
                "- Treat this guidance as project-local. It does not change global user "
                "preferences or installed canonical skills."
            ),
        ]
    )
    if clean_subject_mode == "locked":
        lines.append("- Do not automatically replace the active subject.")
    lines.extend(
        [
            "",
            "## Method Lenses",
            "",
            *_render_method_lenses(method_lenses),
            "",
            "## Resource Activation",
            "",
            *_render_resource_activation(
                resource_activation_plan,
                subject_mode=clean_subject_mode,
            ),
            "",
            "## Evidence And Trace Anchors",
            "",
            "- manifest: `.qiongli/guidance_manifest.yaml`",
            "- subject evidence: `.qiongli/trace/subject_evidence.json`",
            f"- latest action: `{_clean_scalar(lifecycle_action)}`",
            END_MARKER,
        ]
    )
    return "\n".join(lines)


def _render_disabled_block(
    *,
    lifecycle_action: str,
    source: str,
    run_id: str | None,
    updated_at: str,
) -> str:
    lines = [
        START_MARKER,
        "schema_version: 1.0",
        "managed_by: qiongli",
        "active_subject: auto",
        "subject_mode: auto",
        "status: disabled",
        f"updated_at: {updated_at}",
        f"updated_by: {_clean_scalar(source)}",
        f"lifecycle_action: {_clean_scalar(lifecycle_action)}",
    ]
    if run_id:
        lines.append(f"run_id: {_clean_scalar(run_id)}")
    lines.extend(
        [
            "",
            "## Active Subject",
            "",
            "- No project-specific subject is confirmed or locked.",
            "- Use adaptive core inference for future runs.",
            END_MARKER,
        ]
    )
    return "\n".join(lines)


def _render_method_lenses(method_lenses: Sequence[str] | None) -> list[str]:
    lenses = [
        _clean_scalar(lens)
        for lens in (method_lenses or [])
        if isinstance(lens, str) and _clean_scalar(lens)
    ]
    if not lenses:
        return ["- none"]
    return [f"- {lens}" for lens in lenses]


def _render_resource_activation(
    resource_activation_plan: Mapping[str, Any] | None,
    *,
    subject_mode: str,
) -> list[str]:
    levels = resource_activation_plan.get("levels") if resource_activation_plan else None
    if isinstance(levels, Mapping):
        rendered = [
            f"- {_clean_scalar(str(level))}: {_clean_scalar(str(state))}"
            for level, state in levels.items()
            if _clean_scalar(str(level)) and _clean_scalar(str(state))
        ]
        return rendered or ["- core: active"]
    if isinstance(levels, Sequence) and not isinstance(levels, (str, bytes)):
        rendered = []
        for level in levels:
            if not isinstance(level, str):
                continue
            normalized = _clean_scalar(level)
            if not normalized:
                continue
            state = "active" if normalized == "core" else _clean_scalar(subject_mode)
            rendered.append(f"- {normalized}: {state}")
        return rendered or ["- core: active"]
    return ["- core: active"]


def _parse_metadata(block: str) -> dict[str, str]:
    metadata: dict[str, str] = {}
    for raw_line in block.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or line.startswith("<!--") or line.startswith("-"):
            continue
        if ":" not in line:
            continue
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if key:
            metadata[key] = value
    return metadata


def _metadata_warning(metadata: Mapping[str, str]) -> str:
    required = ("active_subject", "subject_mode", "updated_at")
    missing = [key for key in required if not metadata.get(key)]
    if not missing:
        return ""
    return "invalid managed metadata: missing " + ", ".join(missing)


def _reject_symlinked_guidance_path(project_root: Path) -> None:
    for relative_path in (
        Path(".qiongli"),
        Path(".qiongli") / "guidance.d",
        SUBJECT_GUIDANCE_REL,
    ):
        path = project_root / relative_path
        if path.is_symlink():
            raise SubjectGuidanceError(
                f"Unsafe subject guidance path: symlink at {relative_path.as_posix()}"
            )


def _status_packet(
    project_root: Path,
    *,
    exists: bool,
    managed_block: str,
    active_subject: str | None,
    subject_mode: str | None,
    updated_at: str | None,
    warnings: list[str],
) -> dict[str, Any]:
    return {
        "path": SUBJECT_GUIDANCE_REL.as_posix(),
        "exists": exists,
        "managed_block": managed_block,
        "active_subject": active_subject,
        "subject_mode": subject_mode,
        "updated_at": updated_at,
        "warnings": warnings,
    }


def _default_document(block: str) -> str:
    return f"# Qiongli Subject Runtime Guidance\n\n{block}\n"


def _append_document(text: str, block: str) -> str:
    prefix = text
    if prefix and not prefix.endswith("\n"):
        prefix += "\n"
    separator = "\n" if prefix else ""
    return f"{prefix}{separator}# Qiongli Subject Runtime Guidance\n\n{block}\n"


def _clean_scalar(value: str) -> str:
    cleaned = value.strip().replace("\n", " ")
    if START_MARKER in cleaned or END_MARKER in cleaned:
        raise SubjectGuidanceError("managed marker is not allowed in subject guidance values")
    return cleaned


def _timestamp() -> str:
    return datetime.now(UTC).isoformat()


def _normalize_project_root(project_root: Path) -> Path:
    return Path(project_root).expanduser().resolve()


def _rel(project_root: Path, path: Path) -> str:
    try:
        return path.relative_to(project_root).as_posix()
    except ValueError:
        return str(path)
