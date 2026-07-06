from __future__ import annotations

import csv
import re
import stat
from pathlib import Path
from typing import Any


STAGE_REQUIRED_ARTIFACTS: dict[str, list[str]] = {
    "A": [
        "context/research_state.md",
        "context/decision_log.md",
        "context/boundary_review.md",
        "context/stage_handoff.md",
        "framing/research_question.md",
        "framing/contribution_statement.md",
    ],
    "B": [
        "search_strategy.md",
        "search_log.md",
        "search_results.csv",
        "dedup_log.csv",
        "retrieval_manifest.csv",
    ],
    "C": [
        "study_design.md",
        "analysis_plan.md",
    ],
    "F": [
        "manuscript/manuscript.md",
        "evidence/claim-evidence-ledger.csv",
    ],
    "GJ": [
        "reporting_checklist.md",
        "proofread/proofread_checklist.md",
    ],
    "H": [
        "revision/peer_review_simulation.md",
        "revision/fatal_flaw_analysis.md",
    ],
}

_STAGE_ORDER = ("A", "B", "C", "F", "GJ", "H")
_SUPPORTED_EVIDENCE_STATUSES = {"supported", "ready", "verified", "complete"}
_QUESTION_STOPWORDS = {
    "a",
    "an",
    "and",
    "are",
    "can",
    "affect",
    "affected",
    "affects",
    "among",
    "association",
    "associations",
    "between",
    "by",
    "do",
    "does",
    "effect",
    "effects",
    "for",
    "from",
    "how",
    "impact",
    "impacts",
    "influence",
    "influenced",
    "influences",
    "in",
    "is",
    "of",
    "on",
    "or",
    "paper",
    "research",
    "relationship",
    "relationships",
    "studied",
    "studies",
    "study",
    "the",
    "to",
    "using",
    "what",
    "whether",
    "with",
}


def evaluate_stage_gate(project_root: Path | str, stage: str) -> dict[str, Any]:
    root = Path(project_root)
    required = STAGE_REQUIRED_ARTIFACTS.get(stage, [])
    missing = [path for path in required if not _artifact_readable(root / path)]
    status = "passed" if not missing else "blocked_missing_artifact"
    return {
        "stage": stage,
        "status": status,
        "required_artifacts": list(required),
        "missing_artifacts": missing,
        "warnings": [],
    }


def build_lifecycle_report(
    project_root: Path | str,
    *,
    topic: str,
    paper_type: str,
    mode: str = "preview",
) -> dict[str, Any]:
    root = Path(project_root)
    gates = [evaluate_stage_gate(root, stage) for stage in _STAGE_ORDER]

    blocking_reasons: list[str] = []
    for gate in gates:
        if gate["status"] != "passed":
            blocking_reasons.append(f"{gate['stage']}:missing_artifact")

    drift_checks = _drift_checks(root)
    if not drift_checks["locked_question_preserved"]:
        blocking_reasons.append("research_question_drift")
    if drift_checks["claim_evidence_coverage"] != "complete":
        blocking_reasons.append("missing_claim_evidence")
    if drift_checks["unresolved_judge_blocks"] > 0:
        blocking_reasons.append("unresolved_judge_blocks")

    if not blocking_reasons:
        lifecycle_status = "ready_for_h5"
    elif any(gate["status"] != "passed" for gate in gates):
        lifecycle_status = "blocked_missing_artifact"
    elif not drift_checks["locked_question_preserved"]:
        lifecycle_status = "blocked_research_question_drift"
    elif drift_checks["claim_evidence_coverage"] != "complete":
        lifecycle_status = "blocked_missing_claim_evidence"
    else:
        lifecycle_status = "blocked_unresolved_judge"

    return {
        "schema_version": "1.0",
        "mode": mode,
        "topic": topic,
        "paper_type": paper_type,
        "lifecycle_status": lifecycle_status,
        "stage_gates": gates,
        "drift_checks": drift_checks,
        "journal_fit": {
            "status": "not_run",
            "primary": None,
            "blocking_reasons": [],
        },
        "blocking_reasons": blocking_reasons,
        "recommended_next_tasks": _recommended_next_tasks(gates, drift_checks),
    }


def _drift_checks(root: Path) -> dict[str, Any]:
    locked_question = _locked_question(root)
    manuscript = _read(root / "manuscript" / "manuscript.md")
    locked_question_preserved = True
    if locked_question and manuscript:
        locked_question_preserved = _question_preserved(locked_question, manuscript)

    return {
        "locked_question_preserved": locked_question_preserved,
        "claim_evidence_coverage": _claim_evidence_coverage(
            root / "evidence" / "claim-evidence-ledger.csv"
        ),
        "unresolved_judge_blocks": _unresolved_judge_blocks(root),
    }


def _claim_evidence_coverage(path: Path) -> str:
    if not path.is_file():
        return "missing"

    try:
        with path.open(newline="", encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle))
    except OSError:
        return "missing"

    if not rows:
        return "missing"

    statuses = [_claim_evidence_status(row) for row in rows]
    if statuses and all(status in _SUPPORTED_EVIDENCE_STATUSES for status in statuses):
        return "complete"
    return "partial"


def _claim_evidence_status(row: dict[str, Any]) -> str:
    for field in ("status", "evidence_status"):
        value = row.get(field)
        if value is not None:
            return str(value).strip().lower()
    return ""


def _unresolved_judge_blocks(root: Path) -> int:
    judge_text = "\n".join(
        (
            _read(root / "revision" / "peer_review_simulation.md"),
            _read(root / "revision" / "fatal_flaw_analysis.md"),
        )
    ).lower()
    if not judge_text:
        return 0
    blockers = (
        "block_submission",
        "decision: block",
        "decision: reopen_stage",
        "decision: revise",
        "reopen_stage",
        "major issue",
        "major concern",
        "major unresolved",
        "unresolved blocker",
        "unresolved concern",
        "unresolved fatal",
        "unresolved issue",
        "fatal issue",
    )
    if any(blocker in judge_text for blocker in blockers):
        return 1
    if "decision: pass" in judge_text:
        return 0
    return 0


def _recommended_next_tasks(gates: list[dict[str, Any]], drift_checks: dict[str, Any]) -> list[str]:
    recommended: list[str] = []
    if not drift_checks["locked_question_preserved"]:
        recommended.extend(["A1", "A2"])
    if drift_checks["claim_evidence_coverage"] != "complete":
        recommended.append("F4")
    if drift_checks["unresolved_judge_blocks"] > 0:
        recommended.append("H4")
    for gate in gates:
        if gate["status"] != "passed":
            recommended.append(_first_task_for_stage(str(gate["stage"])))

    if not recommended:
        return ["H5"]

    deduped: list[str] = []
    for task in recommended:
        if task not in deduped:
            deduped.append(task)
    return deduped


def _first_task_for_stage(stage: str) -> str:
    return {
        "A": "A1",
        "B": "B1",
        "C": "C1",
        "F": "F3",
        "GJ": "G3",
        "H": "H3",
    }.get(stage, "A1")


def _read(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def _artifact_readable(path: Path) -> bool:
    try:
        mode = path.lstat().st_mode
    except OSError:
        return False
    if path.is_symlink() or not stat.S_ISREG(mode):
        return False
    try:
        with path.open("r", encoding="utf-8", errors="replace") as handle:
            handle.read(0)
    except OSError:
        return False
    return True


def _locked_question(root: Path) -> str:
    framed_question = _read(root / "framing" / "research_question.md").strip()
    if framed_question:
        return framed_question.splitlines()[0].strip()

    research_state = _read(root / "context" / "research_state.md")
    for line in research_state.splitlines():
        if "RQ:" in line:
            return line.split("RQ:", 1)[1].strip()
    return ""


def _question_preserved(question: str, manuscript: str) -> bool:
    normalized_question = _normalize_text(question)
    normalized_manuscript = _normalize_text(manuscript)
    if normalized_question and normalized_question in normalized_manuscript:
        return True

    question_tokens = [
        token
        for token in re.findall(r"[a-z0-9]+", question.lower())
        if token not in _QUESTION_STOPWORDS
    ]
    if not question_tokens:
        return False

    manuscript_tokens = set(re.findall(r"[a-z0-9]+", manuscript.lower()))
    matched = sum(1 for token in question_tokens if token in manuscript_tokens)
    required_matches = (len(question_tokens) // 2) + 1
    return matched >= required_matches


def _normalize_text(text: str) -> str:
    return " ".join(re.findall(r"[a-z0-9]+", text.lower()))
