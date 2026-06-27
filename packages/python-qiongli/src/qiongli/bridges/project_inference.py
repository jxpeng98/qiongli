from __future__ import annotations

import re
from typing import Any


FINANCE_PATTERNS = {
    "asset-pricing": re.compile(
        r"\b(asset pricing|factor model|factor exposure|portfolio|"
        r"stock returns?|asset returns?|portfolio returns?|market returns?|"
        r"abnormal returns?|expected returns?|return predictability)\b",
        re.I,
    ),
    "event-study": re.compile(r"\b(event study|event window|abnormal returns?|leakage)\b", re.I),
}
ECONOMICS_PATTERNS = {
    "did": re.compile(
        r"\b(DID|(?i:difference[- ]in[- ]differences|parallel trends?|pre[- ]trends?))\b"
    ),
    "causal-identification": re.compile(
        r"\b(causal identification|instrumental variable|regression discontinuity|identification)\b",
        re.I,
    ),
}


def infer_project_manifest_suggestion(
    task_packet: dict[str, Any],
    *,
    draft_content: str,
    review_content: str,
    merged_analysis: str,
) -> dict[str, Any]:
    text = " ".join(
        [
            str(task_packet.get("topic", "")),
            str(task_packet.get("context", "")),
            draft_content or "",
            review_content or "",
            merged_analysis or "",
        ]
    )
    finance_hits = _hits(FINANCE_PATTERNS, text)
    economics_hits = _hits(ECONOMICS_PATTERNS, text)
    if len(finance_hits) > len(economics_hits) and finance_hits:
        return {
            "active_subject": "finance",
            "method_lenses": finance_hits,
            "confidence": min(0.95, 0.55 + 0.15 * len(finance_hits)),
            "evidence": _evidence(text, FINANCE_PATTERNS),
        }
    if len(economics_hits) >= len(finance_hits) and economics_hits:
        return {
            "active_subject": "economics",
            "method_lenses": economics_hits,
            "confidence": min(0.95, 0.55 + 0.15 * len(economics_hits)),
            "evidence": _evidence(text, ECONOMICS_PATTERNS),
        }
    return {"active_subject": "auto", "method_lenses": [], "confidence": 0.0, "evidence": []}


def _hits(patterns: dict[str, re.Pattern[str]], text: str) -> list[str]:
    return [name for name, pattern in patterns.items() if pattern.search(text)]


def _evidence(text: str, patterns: dict[str, re.Pattern[str]]) -> list[str]:
    snippets: list[str] = []
    for pattern in patterns.values():
        match = pattern.search(text)
        if match:
            start = max(0, match.start() - 40)
            end = min(len(text), match.end() + 40)
            snippet = " ".join(text[start:end].split())
            if snippet not in snippets:
                snippets.append(snippet)
    return snippets[:3]
