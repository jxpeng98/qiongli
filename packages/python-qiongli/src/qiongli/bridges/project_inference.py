from __future__ import annotations

from typing import Any

from .project_manifest import ProjectManifest
from .subject_refinement import infer_subject_refinement


def infer_project_manifest_suggestion(
    task_packet: dict[str, Any],
    *,
    draft_content: str,
    review_content: str,
    merged_analysis: str,
) -> dict[str, Any]:
    refinement = infer_subject_refinement(
        task_packet,
        manifest_state=ProjectManifest(),
        draft_content=draft_content,
        review_content=review_content,
        merged_analysis=merged_analysis,
    )
    packet = refinement.to_packet()
    active_subject = packet["active_subject"]
    subject_mode = packet["mode"]
    if packet["decision"] == "suggest_subject":
        active_subject = packet["primary_subject"]
        subject_mode = "suggested"

    return {
        "active_subject": active_subject,
        "subject_mode": subject_mode,
        "method_lenses": _unique(
            packet["method_lenses"] + _borrowed_lens_names(packet["borrowed_lenses"])
        ),
        "confidence": refinement.confidence,
        "evidence": list(refinement.evidence or []),
        "subject_refinement": packet,
    }


def _unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    unique_values: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        unique_values.append(value)
    return unique_values


def _borrowed_lens_names(borrowed_lenses: list[dict[str, Any]]) -> list[str]:
    return [
        str(record["lens"])
        for record in borrowed_lenses
        if isinstance(record, dict) and isinstance(record.get("lens"), str)
    ]
