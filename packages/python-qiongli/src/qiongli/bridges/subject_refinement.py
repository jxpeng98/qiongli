from __future__ import annotations

from dataclasses import dataclass
from importlib import resources as importlib_resources
import math
from pathlib import Path
import re
from typing import Any, Mapping

import yaml

from .project_manifest import ProjectManifest, ProjectManifestState
from .subject_contracts import (
    RuntimeSubjectContract,
    load_runtime_subject_contracts,
    subject_activation_status,
)
from .subject_resources import build_resource_activation_plan


CONTRACT_FILE = "subject-refinement-contract.yaml"

SUBJECT_TO_DOMAIN = {
    "auto": "auto",
    "core": "auto",
    "economics": "economics",
    "accounting": "accounting",
    "business": "business-management",
    "finance": "finance",
    "political-economy": "political-economy",
    "geoeconomics": "geoeconomics",
    "economics-accounting": "economics",
}

DEFAULT_OVERLAYS = {
    "economics": "overlays/economics.yaml",
    "finance": "overlays/finance.yaml",
}
DEFAULT_SUBJECT_SKILLS = {
    "economics": "skills/economics/SKILL.md",
    "finance": "skills/finance/SKILL.md",
}
DEFAULT_METHOD_PACKS = {
    "economics": {
        "did": "method-packs/economics/did.yaml",
        "causal-identification": "method-packs/economics/causal-identification.yaml",
    },
    "finance": {
        "event-study": "method-packs/finance/event-study.yaml",
        "asset-pricing": "method-packs/finance/asset-pricing.yaml",
    },
}

FINANCE_METHOD_PATTERNS = {
    "event-study": re.compile(
        r"\b(event[- ]study|event windows?|announcement windows?)\b",
        re.I,
    ),
    "asset-pricing": re.compile(
        r"\b(asset pricing|factor models?|factor exposures?|portfolio sorts?|"
        r"factor regressions?|Fama[- ]MacBeth)\b",
        re.I,
    ),
}
FINANCE_DATA_OUTCOME_PATTERNS = (
    re.compile(r"\b(CRSP|Compustat)\b", re.I),
    re.compile(
        r"\b(abnormal returns?|stock returns?|market returns?|portfolio returns?|"
        r"asset returns?|factor returns?|return predictability|corporate bond spreads?|"
        r"bond spread reactions?)\b",
        re.I,
    ),
)
FINANCE_VENUE_PATTERNS = (
    re.compile(r"\b(Journal of Finance|JF)\b", re.I),
    re.compile(r"\b(Journal of Financial Economics|JFE)\b", re.I),
    re.compile(r"\b(Review of Financial Studies|RFS)\b", re.I),
)
FINANCE_DATA_OUTCOME_SIGNAL_PATTERNS = {
    "crsp": re.compile(r"\bCRSP\b", re.I),
    "compustat": re.compile(r"\bCompustat\b", re.I),
    "abnormal-returns": re.compile(r"\babnormal returns?\b", re.I),
    "stock-returns": re.compile(r"\bstock returns?\b", re.I),
    "market-returns": re.compile(r"\bmarket returns?\b", re.I),
    "portfolio-returns": re.compile(r"\bportfolio returns?\b", re.I),
    "asset-returns": re.compile(r"\basset returns?\b", re.I),
    "factor-returns": re.compile(r"\bfactor returns?\b", re.I),
    "return-predictability": re.compile(r"\breturn predictability\b", re.I),
    "corporate-bond-spreads": re.compile(r"\bcorporate bond spreads?\b", re.I),
    "bond-spread-reactions": re.compile(r"\bbond spread reactions?\b", re.I),
}
FINANCE_VENUE_SIGNAL_PATTERNS = {
    "journal-of-finance": FINANCE_VENUE_PATTERNS[0],
    "journal-of-financial-economics": FINANCE_VENUE_PATTERNS[1],
    "review-of-financial-studies": FINANCE_VENUE_PATTERNS[2],
}

ECONOMICS_METHOD_PATTERNS = {
    "did": re.compile(
        r"\bDID\b|(?i:\bdifference[- ]in[- ]differences\b|"
        r"\bparallel trends?\b|\bpre[- ]trends?\b)"
    ),
    "causal-identification": re.compile(
        r"\b(causal identification|instrumental variables?|"
        r"regression discontinuity|identification strategy|"
        r"quasi[- ]experimental identification|local projections?|"
        r"policy[- ]shock identification|policy shocks?)\b",
        re.I,
    ),
}
ECONOMICS_VENUE_PATTERNS = (
    re.compile(r"\b(American Economic Review|AER)\b", re.I),
    re.compile(r"\b(Quarterly Journal of Economics|QJE)\b", re.I),
    re.compile(r"\b(Journal of Political Economy|JPE)\b", re.I),
)
ECONOMICS_VENUE_SIGNAL_PATTERNS = {
    "american-economic-review": ECONOMICS_VENUE_PATTERNS[0],
    "quarterly-journal-of-economics": ECONOMICS_VENUE_PATTERNS[1],
    "journal-of-political-economy": ECONOMICS_VENUE_PATTERNS[2],
}
SIGNAL_WEIGHTS = {
    "finance": {
        "method": 0.35,
        "data_or_outcome": 0.30,
        "venue": 0.20,
    },
    "economics": {
        "method": 0.40,
        "venue": 0.20,
    },
}


@dataclass(frozen=True)
class SubjectSignals:
    finance_method_lenses: list[str]
    finance_data_outcomes: list[str]
    finance_venues: list[str]
    economics_method_lenses: list[str]
    economics_venues: list[str]
    evidence: list[str]
    signals: list[dict[str, Any]]
    runtime_subject_matches: dict[str, RuntimeSubjectMatch]
    contract_warnings: list[str]

    @property
    def has_any(self) -> bool:
        return bool(
            self.finance_method_lenses
            or self.finance_data_outcomes
            or self.finance_venues
            or self.economics_method_lenses
            or self.economics_venues
            or self.runtime_subject_matches
        )

    @property
    def has_strong_finance(self) -> bool:
        return bool(
            self.finance_method_lenses
            and self.finance_data_outcomes
        )

    @property
    def has_economics_subject_signal(self) -> bool:
        return bool(self.economics_method_lenses)


@dataclass(frozen=True)
class RuntimeSubjectMatch:
    subject: str
    dimensions: tuple[str, ...]
    method_lenses: tuple[str, ...]
    evidence: tuple[str, ...]
    signal_ids: tuple[str, ...]

    @property
    def has_subject_strength(self) -> bool:
        return len(self.dimensions) >= 2


@dataclass(frozen=True)
class ContractLoadResult:
    contract: Mapping[str, Any]
    warnings: list[str]


@dataclass(frozen=True)
class SubjectRefinementPacket:
    decision: str
    mode: str
    active_subject: str
    primary_subject: str
    secondary_subjects: list[str]
    candidate_subjects: list[dict[str, Any]]
    method_lenses: list[str]
    borrowed_lenses: list[dict[str, Any]]
    loaded_resources: dict[str, Any]
    persistence: dict[str, Any]
    summary: str
    domain: str
    confidence: float = 0.0
    evidence: list[str] | None = None
    signals: list[dict[str, Any]] | None = None
    resource_activation_plan: dict[str, Any] | None = None

    def to_packet(self) -> dict[str, Any]:
        return {
            "decision": self.decision,
            "mode": self.mode,
            "active_subject": self.active_subject,
            "primary_subject": self.primary_subject,
            "secondary_subjects": list(self.secondary_subjects),
            "candidate_subjects": [
                _copy_record(candidate)
                for candidate in self.candidate_subjects
            ],
            "method_lenses": list(self.method_lenses),
            "borrowed_lenses": [_copy_record(lens) for lens in self.borrowed_lenses],
            "loaded_resources": {
                key: list(value) if isinstance(value, list) else value
                for key, value in self.loaded_resources.items()
            },
            "persistence": dict(self.persistence),
            "summary": self.summary,
            "domain": self.domain,
            "confidence": self.confidence,
            "evidence": list(self.evidence or []),
            "signals": [_copy_record(signal) for signal in self.signals or []],
            "resource_activation_plan": _copy_value(self.resource_activation_plan or {}),
        }


def _packet(**kwargs: Any) -> SubjectRefinementPacket:
    resource_activation_plan = build_resource_activation_plan(
        decision=str(kwargs["decision"]),
        active_subject=str(kwargs["active_subject"]),
        primary_subject=str(kwargs["primary_subject"]),
        loaded_resources=dict(kwargs["loaded_resources"]),
        method_lenses=list(kwargs["method_lenses"]),
        borrowed_lenses=[
            _copy_record(lens)
            for lens in list(kwargs.get("borrowed_lenses", []) or [])
            if isinstance(lens, Mapping)
        ],
        persistence=dict(kwargs["persistence"]),
    )
    return SubjectRefinementPacket(
        **kwargs,
        resource_activation_plan=resource_activation_plan,
    )


def infer_subject_refinement(
    task_packet: Mapping[str, Any],
    *,
    manifest_state: ProjectManifestState | ProjectManifest | Mapping[str, Any],
    draft_content: str = "",
    review_content: str = "",
    merged_analysis: str = "",
    standards_dir: str | Path | None = None,
    evaluation_subjects: set[str] | None = None,
) -> SubjectRefinementPacket:
    manifest = _coerce_manifest(manifest_state)
    text = _collect_text(task_packet, draft_content, review_content, merged_analysis)
    signals = _detect_signals(text)
    contract_result = _load_contract(standards_dir)
    contract = contract_result.contract
    contract_warnings = [*contract_result.warnings, *signals.contract_warnings]
    evaluation_subjects = set(evaluation_subjects or set())
    finance_runtime_enabled = _subject_can_be_suggested(
        "finance",
        evaluation_subjects=evaluation_subjects,
    )
    economics_runtime_enabled = _subject_can_be_suggested(
        "economics",
        evaluation_subjects=evaluation_subjects,
    )

    if manifest.subject_mode == "locked":
        borrowed_lenses = _borrowed_lenses(manifest.active_subject, signals)
        method_lenses = list(manifest.method_lenses or [])
        return _packet(
            decision="lock_subject",
            mode="locked",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(signals),
            method_lenses=method_lenses,
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["subject_overlay", "subject_skill"]
                + (["method_pack_only"] if borrowed_lenses else []),
                primary_subject=manifest.active_subject,
                method_lenses=method_lenses,
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "locked"},
            summary=_summary(
                "Locked project subject remains active.",
                manifest.active_subject,
                borrowed_lenses,
            ),
            domain=_domain_for_subject(manifest.active_subject),
            confidence=1.0,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    if manifest.subject_mode == "confirmed":
        borrowed_lenses = _borrowed_manifest_lenses(manifest.active_subject, signals)
        method_lenses = _unique(list(manifest.method_lenses or []))
        return _packet(
            decision="confirm_subject",
            mode="confirmed",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=method_lenses,
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["subject_overlay", "subject_skill"]
                + (["method_pack_only"] if borrowed_lenses else []),
                primary_subject=manifest.active_subject,
                method_lenses=method_lenses,
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "applied"},
            summary=f"Confirmed project subject '{manifest.active_subject}' controls this task.",
            domain=_domain_for_subject(manifest.active_subject),
            confidence=1.0,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    if signals.has_strong_finance and finance_runtime_enabled:
        method_lenses = _unique(signals.finance_method_lenses)
        borrowed_lenses = _borrowed_lenses("finance", signals)
        return _packet(
            decision="suggest_subject",
            mode="suggested",
            active_subject=manifest.active_subject,
            primary_subject="finance",
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="finance",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=method_lenses,
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["subject_overlay", "subject_skill", "method_pack"]
                + (["method_pack_only"] if borrowed_lenses else []),
                primary_subject="finance",
                method_lenses=method_lenses,
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "proposed"},
            summary="Finance subject suggested from method, data/outcome, and venue signals.",
            domain="finance",
            confidence=0.85,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    if signals.has_economics_subject_signal and economics_runtime_enabled:
        method_lenses = _unique(signals.economics_method_lenses)
        borrowed_lenses = _borrowed_lenses("economics", signals)
        return _packet(
            decision="suggest_subject",
            mode="suggested",
            active_subject=manifest.active_subject,
            primary_subject="economics",
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="economics",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=method_lenses,
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["subject_overlay", "subject_skill", "method_pack"]
                + (["method_pack_only"] if borrowed_lenses else []),
                primary_subject="economics",
                method_lenses=method_lenses,
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "proposed"},
            summary="Economics subject suggested from causal-method signals.",
            domain="economics",
            confidence=0.7,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    accounting_match = signals.runtime_subject_matches.get("accounting")
    accounting_runtime_enabled = _subject_can_be_suggested(
        "accounting",
        evaluation_subjects=evaluation_subjects,
    )
    if (
        accounting_match is not None
        and accounting_match.has_subject_strength
        and accounting_runtime_enabled
    ):
        method_lenses = _unique(list(accounting_match.method_lenses))
        borrowed_lenses = _borrowed_lenses("accounting", signals)
        return _packet(
            decision="suggest_subject",
            mode="suggested",
            active_subject=manifest.active_subject,
            primary_subject="accounting",
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="accounting",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=method_lenses,
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["subject_overlay", "subject_skill", "method_pack"]
                + (["method_pack_only"] if borrowed_lenses else []),
                primary_subject="accounting",
                method_lenses=method_lenses,
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "proposed"},
            summary=(
                "Accounting subject measured from archival method, construct, "
                "data, and venue signals."
            ),
            domain="accounting",
            confidence=0.75,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    if (
        accounting_match is not None
        and accounting_match.method_lenses
        and manifest.active_subject != "accounting"
    ):
        borrowed_lenses = _borrowed_lenses(manifest.active_subject, signals)
        return _packet(
            decision="borrow_lens",
            mode="auto",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="accounting",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=_unique(list(manifest.method_lenses or [])),
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["method_pack_only"],
                primary_subject=manifest.active_subject,
                method_lenses=[],
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "temporary"},
            summary=_summary(
                "Borrowing accounting method lens without changing the project subject.",
                manifest.active_subject,
                borrowed_lenses,
            ),
            domain=_domain_for_subject(manifest.active_subject),
            confidence=0.45,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    if signals.finance_method_lenses and manifest.active_subject != "finance":
        borrowed_lenses = _borrowed_lenses(manifest.active_subject, signals)
        return _packet(
            decision="borrow_lens",
            mode="auto",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="finance",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=_unique(list(manifest.method_lenses or [])),
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["method_pack_only"],
                primary_subject=manifest.active_subject,
                method_lenses=[],
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "temporary"},
            summary=_summary(
                "Borrowing finance method lens without changing the project subject.",
                manifest.active_subject,
                borrowed_lenses,
            ),
            domain=_domain_for_subject(manifest.active_subject),
            confidence=0.45,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    if signals.economics_method_lenses and manifest.active_subject != "economics":
        borrowed_lenses = _borrowed_lenses(manifest.active_subject, signals)
        return _packet(
            decision="borrow_lens",
            mode="auto",
            active_subject=manifest.active_subject,
            primary_subject=manifest.active_subject,
            secondary_subjects=list(manifest.secondary_subjects or []),
            candidate_subjects=_candidate_subjects(
                signals,
                preferred="economics",
                evaluation_subjects=evaluation_subjects,
            ),
            method_lenses=_unique(list(manifest.method_lenses or [])),
            borrowed_lenses=borrowed_lenses,
            loaded_resources=_loaded_resources(
                ["method_pack_only"],
                primary_subject=manifest.active_subject,
                method_lenses=[],
                borrowed_lenses=borrowed_lenses,
                contract=contract,
                contract_warnings=contract_warnings,
            ),
            persistence={"status": "temporary"},
            summary=_summary(
                "Borrowing economics method lens without changing the project subject.",
                manifest.active_subject,
                borrowed_lenses,
            ),
            domain=_domain_for_subject(manifest.active_subject),
            confidence=0.45,
            evidence=signals.evidence,
            signals=signals.signals,
        )

    return _packet(
        decision="no_subject",
        mode="auto",
        active_subject="auto",
        primary_subject="auto",
        secondary_subjects=[],
        candidate_subjects=[],
        method_lenses=[],
        borrowed_lenses=[],
        loaded_resources=_loaded_resources(
            ["core_only"],
            primary_subject="auto",
            method_lenses=[],
            borrowed_lenses=[],
            contract=contract,
            contract_warnings=contract_warnings,
        ),
        persistence={"status": "none"},
        summary="No subject-specific signal detected; using core guidance only.",
        domain="auto",
        confidence=0.0,
        evidence=[],
        signals=signals.signals,
    )


def _coerce_manifest(
    manifest_state: ProjectManifestState | ProjectManifest | Mapping[str, Any],
) -> ProjectManifest:
    if isinstance(manifest_state, ProjectManifest):
        return manifest_state.normalized()
    if isinstance(manifest_state, ProjectManifestState):
        return manifest_state.manifest.normalized()
    if isinstance(manifest_state, Mapping):
        payload = manifest_state.get("manifest", manifest_state)
        if isinstance(payload, Mapping):
            active_subject = payload.get("active_subject", "auto")
            subject_mode = payload.get("subject_mode")
            if subject_mode is None:
                subject_mode = "confirmed" if active_subject != "auto" else "auto"
            return ProjectManifest(
                active_subject=active_subject,
                subject_mode=subject_mode,
                secondary_subjects=payload.get("secondary_subjects"),
                venue_profiles=payload.get("venue_profiles"),
                method_lenses=payload.get("method_lenses"),
                strictness=payload.get("strictness", "standard"),
            ).normalized()
    return ProjectManifest().normalized()


def _collect_text(
    task_packet: Mapping[str, Any],
    draft_content: str,
    review_content: str,
    merged_analysis: str,
) -> str:
    parts = [_stringify_task_value(value) for value in task_packet.values()]
    parts.extend([draft_content or "", review_content or "", merged_analysis or ""])
    return " ".join(part for part in parts if part)


def _stringify_task_value(value: Any) -> str:
    if isinstance(value, str):
        return value
    if isinstance(value, Mapping):
        return " ".join(_stringify_task_value(item) for item in value.values())
    if isinstance(value, list | tuple | set):
        return " ".join(_stringify_task_value(item) for item in value)
    return ""


def _detect_signals(text: str) -> SubjectSignals:
    finance_method_lenses = _hits(FINANCE_METHOD_PATTERNS, text)
    economics_method_lenses = _hits(ECONOMICS_METHOD_PATTERNS, text)
    (
        manifest_records,
        runtime_subject_matches,
        contract_warnings,
    ) = _detect_manifest_signal_records(text)
    finance_data_outcomes = _pattern_labels(
        {
            "finance-data": FINANCE_DATA_OUTCOME_PATTERNS[0],
            "finance-outcome": FINANCE_DATA_OUTCOME_PATTERNS[1],
        },
        text,
    )
    finance_venues = _pattern_labels(
        {
            "journal-of-finance": FINANCE_VENUE_PATTERNS[0],
            "journal-of-financial-economics": FINANCE_VENUE_PATTERNS[1],
            "review-of-financial-studies": FINANCE_VENUE_PATTERNS[2],
        },
        text,
    )
    economics_venues = _pattern_labels(
        {
            "american-economic-review": ECONOMICS_VENUE_PATTERNS[0],
            "quarterly-journal-of-economics": ECONOMICS_VENUE_PATTERNS[1],
            "journal-of-political-economy": ECONOMICS_VENUE_PATTERNS[2],
        },
        text,
    )
    return SubjectSignals(
        finance_method_lenses=finance_method_lenses,
        finance_data_outcomes=finance_data_outcomes,
        finance_venues=finance_venues,
        economics_method_lenses=economics_method_lenses,
        economics_venues=economics_venues,
        evidence=_evidence(text, extra_records=manifest_records),
        signals=_unique_records([*_detect_signal_records(text), *manifest_records], key="id"),
        runtime_subject_matches=runtime_subject_matches,
        contract_warnings=contract_warnings,
    )


def _detect_signal_records(text: str) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    records.extend(_signal_records_for_patterns("finance", "method", FINANCE_METHOD_PATTERNS, text))
    records.extend(
        _signal_records_for_patterns(
            "finance",
            "data_or_outcome",
            FINANCE_DATA_OUTCOME_SIGNAL_PATTERNS,
            text,
        )
    )
    records.extend(_signal_records_for_patterns("finance", "venue", FINANCE_VENUE_SIGNAL_PATTERNS, text))
    records.extend(_signal_records_for_patterns("economics", "method", ECONOMICS_METHOD_PATTERNS, text))
    records.extend(_signal_records_for_patterns("economics", "venue", ECONOMICS_VENUE_SIGNAL_PATTERNS, text))
    return _unique_records(records, key="id")


def _detect_manifest_signal_records(
    text: str,
) -> tuple[list[dict[str, Any]], dict[str, RuntimeSubjectMatch], list[str]]:
    records: list[dict[str, Any]] = []
    matches: dict[str, RuntimeSubjectMatch] = {}
    contracts, warnings = _safe_load_runtime_subject_contracts()
    for subject, contract in contracts.items():
        if subject in {"economics", "finance"}:
            continue
        subject_records = _manifest_records_for_contract(contract, text)
        if not subject_records:
            continue
        records.extend(subject_records)
        dimensions = _unique([str(record["dimension"]) for record in subject_records])
        method_lenses = _unique(
            [
                str(record["value"])
                for record in subject_records
                if record["dimension"] == "method"
                and str(record["value"]) in contract.method_lenses
            ]
        )
        matches[subject] = RuntimeSubjectMatch(
            subject=subject,
            dimensions=tuple(dimensions),
            method_lenses=tuple(method_lenses),
            evidence=tuple(_unique([str(record["snippet"]) for record in subject_records])),
            signal_ids=tuple(_unique([str(record["id"]) for record in subject_records])),
        )
    return _unique_records(records, key="id"), matches, warnings


def _safe_load_runtime_subject_contracts() -> tuple[dict[str, RuntimeSubjectContract], list[str]]:
    try:
        return load_runtime_subject_contracts(), []
    except Exception as exc:
        return {}, [f"Runtime subject contracts unavailable: {exc}"]


def _manifest_records_for_contract(
    contract: RuntimeSubjectContract,
    text: str,
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for dimension, entries in contract.signal_groups.items():
        for entry in entries:
            if not isinstance(entry, Mapping):
                continue
            entry_id = entry.get("id")
            value = entry.get("value")
            patterns = entry.get("patterns", [])
            if not isinstance(entry_id, str) or not isinstance(value, str):
                continue
            if not isinstance(patterns, list):
                continue
            for pattern_text in patterns:
                if not isinstance(pattern_text, str):
                    continue
                try:
                    pattern = re.compile(pattern_text, re.I)
                except re.error:
                    continue
                match = pattern.search(text)
                if not match:
                    continue
                records.append(
                    {
                        "id": entry_id,
                        "subject": contract.subject,
                        "dimension": str(dimension),
                        "value": value,
                        "weight": _coerce_signal_weight(entry.get("weight", 0.0)),
                        "source": "task_text",
                        "snippet": _snippet_for_match(text, match),
                    }
                )
                break
    return _unique_records(records, key="id")


def _signal_records_for_patterns(
    subject: str,
    dimension: str,
    patterns: Mapping[str, re.Pattern[str]],
    text: str,
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    for value, pattern in patterns.items():
        match = pattern.search(text)
        if not match:
            continue
        records.append(
            {
                "id": f"{subject}.{dimension}.{value}",
                "subject": subject,
                "dimension": dimension,
                "value": value,
                "weight": SIGNAL_WEIGHTS.get(subject, {}).get(dimension, 0.0),
                "source": "task_text",
                "snippet": _snippet_for_match(text, match),
            }
        )
    return records


def _coerce_signal_weight(value: Any) -> float:
    try:
        weight = float(value or 0.0)
    except (TypeError, ValueError):
        return 0.0
    if not math.isfinite(weight):
        return 0.0
    return weight


def _hits(patterns: Mapping[str, re.Pattern[str]], text: str) -> list[str]:
    return [name for name, pattern in patterns.items() if pattern.search(text)]


def _pattern_labels(patterns: Mapping[str, re.Pattern[str]], text: str) -> list[str]:
    return [name for name, pattern in patterns.items() if pattern.search(text)]


def _evidence(text: str, *, extra_records: list[dict[str, Any]] | None = None) -> list[str]:
    patterns = [
        *FINANCE_METHOD_PATTERNS.values(),
        *FINANCE_DATA_OUTCOME_PATTERNS,
        *FINANCE_VENUE_PATTERNS,
        *ECONOMICS_METHOD_PATTERNS.values(),
        *ECONOMICS_VENUE_PATTERNS,
    ]
    snippets: list[str] = []
    for pattern in patterns:
        match = pattern.search(text)
        if not match:
            continue
        snippet = _snippet_for_match(text, match)
        if snippet not in snippets:
            snippets.append(snippet)
    for record in extra_records or []:
        snippet = record.get("snippet")
        if isinstance(snippet, str) and snippet not in snippets:
            snippets.append(snippet)
    return snippets[:5]


def _snippet_for_match(text: str, match: re.Match[str]) -> str:
    start = max(0, match.start() - 40)
    end = min(len(text), match.end() + 40)
    return " ".join(text[start:end].split())


def _candidate_subjects(
    signals: SubjectSignals,
    *,
    preferred: str | None = None,
    evaluation_subjects: set[str] | None = None,
) -> list[dict[str, Any]]:
    subjects: list[str] = []
    if preferred:
        subjects.append(preferred)
    if (
        signals.finance_method_lenses
        or signals.finance_data_outcomes
        or signals.finance_venues
    ):
        subjects.append("finance")
    if signals.economics_method_lenses or signals.economics_venues:
        subjects.append("economics")
    subjects.extend(signals.runtime_subject_matches)
    return [
        _candidate_subject_record(subject, signals)
        for subject in _unique(subjects)
        if _subject_can_be_suggested(subject, evaluation_subjects=evaluation_subjects)
    ]


def _candidate_subject_record(subject: str, signals: SubjectSignals) -> dict[str, Any]:
    if subject == "finance":
        matched_dimensions: list[str] = []
        if signals.finance_method_lenses:
            matched_dimensions.append("method")
        if signals.finance_data_outcomes:
            matched_dimensions.append("data_or_outcome")
        if signals.finance_venues:
            matched_dimensions.append("venue")
        return {
            "subject": "finance",
            "confidence": _candidate_confidence("finance", matched_dimensions),
            "evidence": list(signals.evidence),
            "matched_dimensions": matched_dimensions,
            "method_lenses": list(signals.finance_method_lenses),
        }
    if subject == "economics":
        matched_dimensions = []
        if signals.economics_method_lenses:
            matched_dimensions.append("method")
        if signals.economics_venues:
            matched_dimensions.append("venue")
        return {
            "subject": "economics",
            "confidence": _candidate_confidence("economics", matched_dimensions),
            "evidence": list(signals.evidence),
            "matched_dimensions": matched_dimensions,
            "method_lenses": list(signals.economics_method_lenses),
        }
    runtime_match = signals.runtime_subject_matches.get(subject)
    if runtime_match is not None:
        return {
            "subject": subject,
            "confidence": min(0.85, 0.35 + 0.15 * len(runtime_match.dimensions)),
            "evidence": list(runtime_match.evidence),
            "matched_dimensions": list(runtime_match.dimensions),
            "method_lenses": list(runtime_match.method_lenses),
            "signal_ids": list(runtime_match.signal_ids),
        }
    return {
        "subject": subject,
        "confidence": 0.0,
        "evidence": [],
        "matched_dimensions": [],
        "method_lenses": [],
    }


def _candidate_confidence(subject: str, matched_dimensions: list[str]) -> float:
    if subject == "finance":
        if matched_dimensions == ["method", "data_or_outcome", "venue"]:
            return 0.85
        if "method" in matched_dimensions:
            return 0.45 + 0.1 * (len(matched_dimensions) - 1)
        return 0.25
    if subject == "economics":
        if "method" in matched_dimensions:
            return 0.7
        return 0.35
    return 0.0


def _borrowed_lenses(active_subject: str, signals: SubjectSignals) -> list[dict[str, Any]]:
    lenses: list[dict[str, Any]] = []
    if active_subject != "finance":
        lenses.extend(
            _borrowed_lens_record(
                "finance",
                lens,
                reason=(
                    "finance method-only signal; keep active subject"
                    if not signals.has_strong_finance
                    else "locked subject; borrow neighboring finance lens without replacing active subject"
                ),
            )
            for lens in signals.finance_method_lenses
        )
    if active_subject != "economics":
        lenses.extend(
            _borrowed_lens_record(
                "economics",
                lens,
                reason="borrow neighboring economics method lens without replacing active subject",
            )
            for lens in signals.economics_method_lenses
        )
    lenses.extend(_borrowed_manifest_lenses(active_subject, signals))
    return _unique_records(lenses, key="lens")


def _borrowed_manifest_lenses(
    active_subject: str,
    signals: SubjectSignals,
) -> list[dict[str, Any]]:
    lenses: list[dict[str, Any]] = []
    for subject, match in signals.runtime_subject_matches.items():
        if active_subject == subject:
            continue
        lenses.extend(
            _borrowed_lens_record(
                subject,
                lens,
                reason=f"{subject} method-only signal; keep active subject",
            )
            for lens in match.method_lenses
        )
    return _unique_records(lenses, key="lens")


def _borrowed_lens_record(source_subject: str, lens: str, *, reason: str) -> dict[str, Any]:
    return {
        "source_subject": source_subject,
        "lens": lens,
        "resource_level": "method_pack_only",
        "reason": reason,
    }


def _loaded_resources(
    levels: list[str],
    *,
    primary_subject: str,
    method_lenses: list[str],
    borrowed_lenses: list[dict[str, Any]],
    contract: Mapping[str, Any],
    contract_warnings: list[str],
) -> dict[str, Any]:
    overlays = _subject_resource_map(contract, "overlays", DEFAULT_OVERLAYS)
    subject_skills = _subject_resource_map(
        contract,
        "subject_skills",
        DEFAULT_SUBJECT_SKILLS,
    )
    method_packs = _method_pack_map(contract)
    method_pack_resources = _method_pack_resources(
        method_lenses + _borrowed_lens_names(borrowed_lenses),
        method_packs,
    )
    activation_enabled, activation_warnings = _subject_level_resources_enabled(primary_subject)
    warnings = list(contract_warnings) + activation_warnings
    loaded_levels = list(levels)
    if not activation_enabled:
        loaded_levels = [
            level
            for level in loaded_levels
            if level not in {"subject_overlay", "subject_skill"}
        ]
    if method_lenses and method_pack_resources and "method_pack" not in loaded_levels:
        loaded_levels.append("method_pack")

    loaded_overlays: list[str] = []
    loaded_subject_skills: list[str] = []
    if activation_enabled and primary_subject not in {"auto", "core"} and (
        "subject_overlay" in levels or "subject_skill" in levels
    ):
        overlay = overlays.get(primary_subject)
        skill = subject_skills.get(primary_subject)
        if overlay:
            loaded_overlays.append(overlay)
        if skill:
            loaded_subject_skills.append(skill)

    return {
        "levels": loaded_levels,
        "overlays": loaded_overlays,
        "subject_skills": loaded_subject_skills,
        "method_packs": method_pack_resources,
        "standards": [CONTRACT_FILE] if contract else [],
        "contract_warnings": warnings,
    }


def _subject_level_resources_enabled(subject: str) -> tuple[bool, list[str]]:
    if subject in {"auto", "core"}:
        return True, []
    status = _runtime_activation_status(subject)
    if status == "runtime_enabled":
        return True, []
    return False, [
        f"Subject {subject} activation_status={status}; subject resources withheld",
    ]


def _subject_can_be_suggested(
    subject: str,
    *,
    evaluation_subjects: set[str] | None = None,
) -> bool:
    if evaluation_subjects and subject in evaluation_subjects:
        return True
    return _runtime_activation_status(subject) == "runtime_enabled"


def _runtime_activation_status(subject: str) -> str:
    try:
        return subject_activation_status(subject)
    except Exception:
        if subject in {"economics", "finance"}:
            return "runtime_enabled"
        return "candidate"


def _load_contract(standards_dir: str | Path | None) -> ContractLoadResult:
    warnings: list[str] = []
    candidates = _contract_candidates(standards_dir)
    explicit_candidate = candidates[0] if standards_dir is not None and candidates else None

    for candidate in candidates:
        if not candidate.is_file():
            if explicit_candidate is not None and candidate == explicit_candidate:
                warnings.append(f"Missing subject refinement contract: {candidate}")
            continue
        try:
            loaded = yaml.safe_load(candidate.read_text(encoding="utf-8")) or {}
        except yaml.YAMLError as exc:
            warnings.append(f"Malformed subject refinement contract at {candidate}: {exc}")
            continue
        if not isinstance(loaded, Mapping):
            warnings.append(
                f"Malformed subject refinement contract at {candidate}: expected YAML object"
            )
            continue
        return ContractLoadResult(contract=loaded, warnings=warnings)

    checked = ", ".join(str(candidate) for candidate in candidates)
    if checked:
        warnings.append(f"Missing subject refinement contract; checked: {checked}")
    else:
        warnings.append("Missing subject refinement contract; no lookup candidates were available")
    return ContractLoadResult(contract={}, warnings=warnings)


def _contract_candidates(standards_dir: str | Path | None) -> list[Any]:
    candidates: list[Any] = []
    if standards_dir is not None:
        explicit = Path(standards_dir).expanduser()
        candidates.append(explicit if explicit.name == CONTRACT_FILE else explicit / CONTRACT_FILE)

    runtime_file = Path(__file__).resolve()
    cwd = Path.cwd().resolve()
    for parent in (cwd, *cwd.parents):
        candidates.append(parent / "content" / "standards" / CONTRACT_FILE)

    for parent in runtime_file.parents:
        candidates.append(parent / "content" / "standards" / CONTRACT_FILE)

    try:
        package_root = importlib_resources.files("qiongli")
    except ModuleNotFoundError:
        package_root = None
    if package_root is not None:
        candidates.append(package_root / "payload" / "qiongli-workflow" / "standards" / CONTRACT_FILE)

    for parent in runtime_file.parents:
        candidates.append(parent / "payload" / "qiongli-workflow" / "standards" / CONTRACT_FILE)

    return _unique_candidates(candidates)


def _subject_resource_map(
    contract: Mapping[str, Any],
    key: str,
    fallback: Mapping[str, str],
) -> dict[str, str]:
    values = contract.get(key)
    if not isinstance(values, Mapping):
        return dict(fallback)
    resources = dict(fallback)
    for subject, resource in values.items():
        if isinstance(subject, str) and isinstance(resource, str):
            resources[subject] = resource
    return resources


def _method_pack_map(contract: Mapping[str, Any]) -> dict[str, dict[str, str]]:
    resources = {subject: dict(lenses) for subject, lenses in DEFAULT_METHOD_PACKS.items()}
    configured = contract.get("method_lenses")
    if isinstance(configured, Mapping):
        for subject, lenses in configured.items():
            if not isinstance(subject, str) or not isinstance(lenses, Mapping):
                continue
            subject_resources = resources.setdefault(subject, {})
            for lens, details in lenses.items():
                if not isinstance(lens, str) or not isinstance(details, Mapping):
                    continue
                resource = details.get("resource")
                if isinstance(resource, str):
                    subject_resources[lens] = resource
    runtime_contracts, _ = _safe_load_runtime_subject_contracts()
    for subject, runtime_contract in runtime_contracts.items():
        subject_resources = resources.setdefault(subject, {})
        for lens, details in runtime_contract.method_lenses.items():
            if not isinstance(lens, str) or not isinstance(details, Mapping):
                continue
            resource = details.get("resource")
            if isinstance(resource, str) and resource.strip():
                subject_resources[lens] = resource
    return resources


def _method_pack_resources(
    lenses: list[str],
    method_packs: Mapping[str, Mapping[str, str]],
) -> list[str]:
    resources: list[str] = []
    for lens in lenses:
        for subject_packs in method_packs.values():
            resource = subject_packs.get(lens)
            if resource:
                resources.append(resource)
                break
    return _unique(resources)


def _summary(prefix: str, active_subject: str, borrowed_lenses: list[dict[str, Any]]) -> str:
    lens_names = _borrowed_lens_names(borrowed_lenses)
    if lens_names:
        return (
            f"{prefix} active_subject={active_subject}; "
            f"borrowed_lenses={', '.join(lens_names)}."
        )
    return f"{prefix} active_subject={active_subject}."


def _domain_for_subject(subject: str) -> str:
    return SUBJECT_TO_DOMAIN.get(subject, "auto")


def _unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    unique_values: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        unique_values.append(value)
    return unique_values


def _unique_records(records: list[dict[str, Any]], *, key: str) -> list[dict[str, Any]]:
    seen: set[Any] = set()
    unique_values: list[dict[str, Any]] = []
    for record in records:
        value = record.get(key)
        if value in seen:
            continue
        seen.add(value)
        unique_values.append(record)
    return unique_values


def _borrowed_lens_names(borrowed_lenses: list[dict[str, Any]]) -> list[str]:
    return [
        str(record["lens"])
        for record in borrowed_lenses
        if isinstance(record, Mapping) and isinstance(record.get("lens"), str)
    ]


def _copy_record(record: Mapping[str, Any]) -> dict[str, Any]:
    copied: dict[str, Any] = {}
    for key, value in record.items():
        copied[key] = _copy_value(value)
    return copied


def _copy_value(value: Any) -> Any:
    if isinstance(value, list):
        return [_copy_value(item) for item in value]
    if isinstance(value, Mapping):
        return {key: _copy_value(item) for key, item in value.items()}
    return value


def _unique_candidates(candidates: list[Any]) -> list[Any]:
    seen: set[str] = set()
    unique_values: list[Any] = []
    for candidate in candidates:
        marker = str(candidate)
        if marker in seen:
            continue
        seen.add(marker)
        unique_values.append(candidate)
    return unique_values
