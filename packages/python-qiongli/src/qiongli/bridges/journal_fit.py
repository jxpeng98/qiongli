from __future__ import annotations

import csv
import re
from pathlib import Path
from typing import Any

import yaml


REQUIRED_INPUTS = [
    "manuscript/manuscript.md",
    "framing/contribution_statement.md",
    "study_design.md",
    "evidence/claim-evidence-ledger.csv",
]

PROFILE_SCORE_FIELDS = (
    "community",
    "article_types",
    "contribution_expectations",
    "methods_expectations",
    "evidence_standards",
    "writing_style",
)

_COMPLETE_EVIDENCE_STATUSES = {"complete", "ok", "ready", "supported", "verified"}
_FATAL_FLAW_MARKERS = (
    "block_submission",
    "decision: block",
    "decision: reopen_stage",
    "decision: revise",
    "fatal flaw:",
    "fatal issue",
    "major unresolved",
    "unresolved blocker",
    "unresolved concern",
    "unresolved fatal",
)
_FATAL_PASS_MARKERS = (
    "decision: pass",
    "no fatal flaw",
    "no unresolved fatal",
)
_TOKEN_RE = re.compile(r"[a-z0-9]+")


def recommend_journals(
    project_root: Path | str,
    *,
    venue_roots: list[Path | str],
    limit: int = 5,
) -> dict[str, Any]:
    """Rank local venue profiles against an existing manuscript."""

    root = Path(project_root)
    missing_inputs = [path for path in REQUIRED_INPUTS if not _readable_file(root / path)]
    blocking_reasons = [f"missing {path}" for path in missing_inputs]

    if "manuscript/manuscript.md" in missing_inputs:
        return {
            "schema_version": "1.0",
            "status": "blocked",
            "blocking_reasons": blocking_reasons,
            "ranked_venues": [],
        }

    evidence_status = _claim_evidence_status(root / "evidence" / "claim-evidence-ledger.csv")
    if evidence_status["coverage"] != "complete":
        blocking_reasons.append("incomplete claim evidence")

    manuscript_text = _project_text(root)
    fatal_flaw = _has_unresolved_fatal_flaw(root)
    venues = _load_venues([Path(path) for path in venue_roots])

    ranked = [
        _score_venue(
            venue,
            manuscript_text,
            fatal_flaw=fatal_flaw,
            evidence_coverage=str(evidence_status["coverage"]),
        )
        for venue in venues
    ]
    ranked.sort(key=lambda item: (-item["score"], item["venue_id"]))

    capped_limit = max(0, int(limit))
    return {
        "schema_version": "1.0",
        "status": "ok",
        "blocking_reasons": blocking_reasons,
        "ranked_venues": ranked[:capped_limit],
    }


def _project_text(root: Path) -> str:
    parts: list[str] = []
    for rel_path in REQUIRED_INPUTS:
        text = _read_text(root / rel_path)
        if text:
            parts.append(text)
    return _normalize_text("\n".join(parts))


def _load_venues(roots: list[Path]) -> list[dict[str, Any]]:
    venues: list[dict[str, Any]] = []
    for root in roots:
        paths = [root] if root.is_file() else [*sorted(root.glob("*.yaml")), *sorted(root.glob("*.yml"))]
        for path in paths:
            venue = _read_venue(path)
            if venue is not None:
                venues.append(venue)
    return venues


def _read_venue(path: Path) -> dict[str, Any] | None:
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    except (OSError, UnicodeError, yaml.YAMLError):
        return None
    if not isinstance(payload, dict):
        return None

    venue_id = payload.get("venue_id") or payload.get("id") or path.stem
    payload["venue_id"] = str(venue_id).strip() or path.stem
    payload["_source"] = str(path)
    return payload


def _score_venue(
    venue: dict[str, Any],
    manuscript_text: str,
    *,
    fatal_flaw: bool,
    evidence_coverage: str,
) -> dict[str, Any]:
    community_score, community_terms = _field_fit(venue.get("community"), manuscript_text)
    article_score, article_terms = _field_fit(venue.get("article_types"), manuscript_text)
    scope_score = _weighted_average(
        (
            (community_score, 0.80),
            (article_score, 0.20),
        )
    )
    contribution_score, contribution_terms = _field_fit(
        venue.get("contribution_expectations"), manuscript_text
    )
    methods_score, methods_terms = _field_fit(venue.get("methods_expectations"), manuscript_text)
    evidence_score, evidence_terms = _field_fit(venue.get("evidence_standards"), manuscript_text)
    method_evidence_score = _weighted_average(((methods_score, 0.50), (evidence_score, 0.50)))
    writing_score, writing_terms = _field_fit(venue.get("writing_style"), manuscript_text)
    community_terms = _field_fit(venue.get("community"), manuscript_text)[1]
    article_terms = _field_fit(venue.get("article_types"), manuscript_text)[1]

    score = round(
        (scope_score * 0.25)
        + (contribution_score * 0.30)
        + (method_evidence_score * 0.35)
        + (writing_score * 0.10),
        3,
    )
    matched_terms = _dedupe_terms(
        [
            *community_terms,
            *article_terms,
            *contribution_terms,
            *methods_terms,
            *evidence_terms,
            *writing_terms,
        ]
    )
    matched_dimensions = sum(
        1
        for dimension_terms in (
            [*community_terms, *article_terms],
            contribution_terms,
            methods_terms,
            evidence_terms,
            writing_terms,
        )
        if dimension_terms
    )

    return {
        "venue_id": str(venue["venue_id"]),
        "class": _classify(
            score,
            fatal_flaw=fatal_flaw,
            evidence_coverage=evidence_coverage,
            unique_matched_terms=len(matched_terms),
            matched_dimensions=matched_dimensions,
        ),
        "score": score,
        "scope_fit": _fit_label(scope_score),
        "contribution_fit": _fit_label(contribution_score),
        "method_evidence_fit": _method_evidence_label(method_evidence_score, evidence_coverage),
        "reviewer_risk": _reviewer_risk(
            score,
            fatal_flaw=fatal_flaw,
            evidence_coverage=evidence_coverage,
        ),
        "desk_reject_risk": _desk_reject_risk(score, fatal_flaw=fatal_flaw),
        "matched_terms": matched_terms,
        "required_revision": _required_revision(
            score,
            fatal_flaw=fatal_flaw,
            evidence_coverage=evidence_coverage,
        ),
        "source": str(venue.get("_source", "")),
    }


def _field_fit(raw: Any, manuscript_text: str) -> tuple[float, list[str]]:
    terms = _profile_terms(raw)
    if not terms:
        return 0.0, []
    matched = [term for term in terms if _term_in_text(term, manuscript_text)]
    return len(matched) / len(terms), matched


def _profile_terms(raw: Any) -> list[str]:
    values = _flatten_values(raw)
    terms: list[str] = []
    for value in values:
        normalized = _normalize_text(value)
        if normalized:
            terms.append(normalized)
    return _dedupe_terms(terms)


def _flatten_values(raw: Any) -> list[str]:
    if raw is None:
        return []
    if isinstance(raw, str):
        return [raw]
    if isinstance(raw, dict):
        values: list[str] = []
        for value in raw.values():
            values.extend(_flatten_values(value))
        return values
    if isinstance(raw, (list, tuple, set)):
        values = []
        for item in raw:
            values.extend(_flatten_values(item))
        return values
    return [str(raw)]


def _term_in_text(term: str, manuscript_text: str) -> bool:
    if not term:
        return False
    if " " in term:
        return f" {term} " in f" {manuscript_text} "
    return term in set(_TOKEN_RE.findall(manuscript_text))


def _classify(
    score: float,
    *,
    fatal_flaw: bool,
    evidence_coverage: str,
    unique_matched_terms: int,
    matched_dimensions: int,
) -> str:
    if score < 0.25:
        return "do_not_submit"
    if fatal_flaw and score >= 0.60:
        return "stretch"
    if evidence_coverage != "complete" and score >= 0.70:
        return "stretch"
    if score >= 0.75 and unique_matched_terms >= 3 and matched_dimensions >= 3:
        return "primary"
    if score >= 0.55:
        return "safe"
    return "fallback"


def _fit_label(score: float) -> str:
    if score >= 0.75:
        return "strong"
    if score >= 0.55:
        return "moderate"
    if score >= 0.25:
        return "weak"
    return "poor"


def _method_evidence_label(score: float, evidence_coverage: str) -> str:
    label = _fit_label(score)
    if evidence_coverage == "complete":
        return label
    return f"{label}_with_evidence_gaps"


def _reviewer_risk(score: float, *, fatal_flaw: bool, evidence_coverage: str) -> str:
    risks: list[str] = []
    if fatal_flaw:
        risks.append("unresolved fatal flaw")
    if evidence_coverage != "complete":
        risks.append("claim evidence is incomplete")

    if score >= 0.75:
        risks.append("fit risk is low; contribution strength still needs venue calibration")
    elif score >= 0.55:
        risks.append("fit is plausible but positioning revision is needed")
    elif score >= 0.25:
        risks.append("scope or method fit is weak")
    else:
        risks.append("venue scope does not match the manuscript")
    return "; ".join(risks)


def _desk_reject_risk(score: float, *, fatal_flaw: bool) -> str:
    if fatal_flaw:
        return "high"
    if score >= 0.75:
        return "low"
    if score >= 0.55:
        return "medium"
    return "high"


def _required_revision(score: float, *, fatal_flaw: bool, evidence_coverage: str) -> str:
    revisions: list[str] = []
    if fatal_flaw:
        revisions.append("Resolve fatal flaw before submission.")
    if evidence_coverage != "complete":
        revisions.append("Complete claim-evidence ledger support before submission.")

    if score >= 0.75:
        revisions.append("Tighten venue-facing contribution and formatting.")
    elif score >= 0.55:
        revisions.append("Revise framing to match venue scope and methods expectations.")
    elif score >= 0.25:
        revisions.append("Substantially re-scope positioning before treating this as a fallback.")
    else:
        revisions.append("Choose a different venue.")
    return " ".join(revisions)


def _claim_evidence_status(path: Path) -> dict[str, Any]:
    if not _readable_file(path):
        return {"coverage": "missing", "statuses": []}

    try:
        with path.open(newline="", encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle))
    except (OSError, UnicodeError, csv.Error):
        return {"coverage": "missing", "statuses": []}

    if not rows:
        return {"coverage": "partial", "statuses": []}

    statuses = [_row_status(row) for row in rows]
    if statuses and all(status in _COMPLETE_EVIDENCE_STATUSES for status in statuses):
        coverage = "complete"
    else:
        coverage = "partial"
    return {"coverage": coverage, "statuses": statuses}


def _row_status(row: dict[str, Any]) -> str:
    for field in ("status", "evidence_status"):
        value = row.get(field)
        if value is not None:
            return str(value).strip().lower()
    return ""


def _has_unresolved_fatal_flaw(root: Path) -> bool:
    text = _read_text(root / "revision" / "fatal_flaw_analysis.md").lower()
    if not text:
        return False
    if any(marker in text for marker in _FATAL_FLAW_MARKERS):
        return True
    if any(marker in text for marker in _FATAL_PASS_MARKERS):
        return False
    return False


def _weighted_average(weighted_scores: tuple[tuple[float, float], ...]) -> float:
    weight_total = sum(weight for _, weight in weighted_scores)
    if weight_total == 0:
        return 0.0
    return sum(score * weight for score, weight in weighted_scores) / weight_total


def _dedupe_terms(terms: list[str]) -> list[str]:
    deduped: list[str] = []
    for term in terms:
        if term and term not in deduped:
            deduped.append(term)
    return deduped


def _normalize_text(text: str) -> str:
    tokens = _TOKEN_RE.findall(text.lower())
    return " ".join(tokens)


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def _readable_file(path: Path) -> bool:
    return path.is_file() and not path.is_symlink()
