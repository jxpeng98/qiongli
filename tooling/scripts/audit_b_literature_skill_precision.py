#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout


PROVIDER_FACING_SKILLS = {"academic-searcher", "citation-snowballer", "fulltext-fetcher"}


@dataclass
class SkillPrecisionResult:
    path: Path
    checks: dict[str, bool]

    @property
    def missing_checks(self) -> list[str]:
        return [name for name, passed in self.checks.items() if not passed]

    @property
    def issue_count(self) -> int:
        return len(self.missing_checks)


@dataclass
class GlobalPrecisionResult:
    name: str
    checks: dict[str, bool]

    @property
    def missing_checks(self) -> list[str]:
        return [name for name, passed in self.checks.items() if not passed]

    @property
    def issue_count(self) -> int:
        return len(self.missing_checks)


@dataclass
class BLiteraturePrecisionAuditResult:
    root: Path
    skill_results: list[SkillPrecisionResult] = field(default_factory=list)
    global_results: list[GlobalPrecisionResult] = field(default_factory=list)

    @property
    def has_gaps(self) -> bool:
        return any(item.issue_count for item in self.skill_results) or any(
            item.issue_count for item in self.global_results
        )


def audit_b_literature_skill_precision(root: Path) -> BLiteraturePrecisionAuditResult:
    root = root.resolve()
    result = BLiteraturePrecisionAuditResult(root=root)
    for path in _read_b_skill_files(root):
        text = path.read_text(encoding="utf-8")
        skill_id = _skill_id(path)
        result.skill_results.append(
            SkillPrecisionResult(
                path=path.relative_to(root),
                checks=_skill_checks(skill_id, text),
            )
        )
    result.global_results.append(
        GlobalPrecisionResult(
            name="content/skills-core.md",
            checks=_skills_core_checks(root),
        )
    )
    return result


def render_markdown_report(result: BLiteraturePrecisionAuditResult) -> str:
    lines = [
        "# B Literature Skill Precision Audit",
        "",
        "## Summary",
        "",
        f"- Skills scanned: {len(result.skill_results)}",
        f"- Files with gaps: {sum(1 for item in result.skill_results if item.issue_count)}",
        f"- Global gaps: {sum(item.issue_count for item in result.global_results)}",
        "",
        "## Skill Findings",
        "",
        "| File | Missing Checks |",
        "|------|----------------|",
    ]
    for item in result.skill_results:
        missing = ", ".join(item.missing_checks) or "-"
        lines.append(f"| `{item.path}` | {missing} |")

    lines.extend(["", "## Global Findings", "", "| Scope | Missing Checks |", "|-------|----------------|"])
    for item in result.global_results:
        missing = ", ".join(item.missing_checks) or "-"
        lines.append(f"| `{item.name}` | {missing} |")
    return "\n".join(lines).rstrip() + "\n"


def _read_b_skill_files(root: Path) -> list[Path]:
    stage_dir = RepoLayout(root).skills / "B_literature"
    if not stage_dir.is_dir():
        return []
    return sorted(path for path in stage_dir.glob("*.md") if path.is_file())


def _skill_id(path: Path) -> str:
    return path.stem


def _skill_checks(skill_id: str, text: str) -> dict[str, bool]:
    checks = {
        "task-stage relation": _has_task_stage_relation(text),
        "canonical artifact paths": bool(re.search(r"RESEARCH/\[topic\]/[A-Za-z0-9_./-]+", text)),
    }
    if skill_id in PROVIDER_FACING_SKILLS:
        checks["compact provider references"] = _has_compact_provider_references(text)

    checks.update(_specific_skill_checks(skill_id, text))
    return checks


def _specific_skill_checks(skill_id: str, text: str) -> dict[str, bool]:
    lower = text.lower()
    if skill_id == "academic-searcher":
        return {
            "provider ownership": _has_all(lower, ["scholarly-search", "mcp/provider"]),
            "review-grade blockers": _has_all(
                lower,
                [
                    "at least two productive providers",
                    "known-item",
                    "zero-hit",
                    "weak screening readiness",
                ],
            ),
            "supplemental manual search boundary": "google scholar" in lower
            and "supplemental" in lower
            and "not the default" in lower,
        }
    if skill_id == "concept-extractor":
        return {
            "concept bucket contract": _has_all(lower, ["concept bucket", "controlled vocab", "near misses"]),
            "seed recall test": _has_all(lower, ["seed", "recall", "query gap"]),
        }
    if skill_id == "paper-screener":
        return {
            "review-grade blockers": _has_all(lower, ["search_diagnostics.md", "triage", "review-grade"]),
            "screening provenance": _has_all(
                lower,
                ["record_id", "query_id", "source", "relevance_reason", "diagnostic_flags"],
            ),
            "controlled fulltext statuses": _has_all(lower, ["retrieved_oa", "retrieved_preprint", "abstract_only"]),
        }
    if skill_id == "fulltext-fetcher":
        return {
            "provider ownership": _has_all(lower, ["fulltext-retrieval", "retrieval_manifest.csv"]),
            "resolver boundary": _has_all(lower, ["built-in", "stub", "external resolver", "zotero"]),
            "controlled retrieval statuses": _has_all(lower, ["retrieved_oa", "retrieved_preprint", "not_retrieved"]),
            "legal access boundary": "illegal" in lower or "paywall-bypassing" in lower,
        }
    if skill_id == "citation-snowballer":
        return {
            "provider ownership": _has_all(lower, ["citation-graph", "search_results.csv", "dedup_log.csv"]),
            "seed rationale and saturation": _has_all(lower, ["seed_selection_reason", "saturation_status"]),
            "append contract": "append" in lower and "dedup" in lower,
        }
    if skill_id == "paper-extractor":
        return {
            "evidence limits": _has_all(
                lower,
                ["source_anchor", "evidence_limit", "unsupported_gap", "metadata_only", "abstract_only"],
            ),
        }
    if skill_id == "literature-mapper":
        return {
            "evidence limits": "evidence limit" in lower or "evidence_limit" in lower,
            "non-chronological taxonomy": _has_all(lower, ["clustering basis", "representative papers"])
            and "chronological" in lower,
        }
    if skill_id == "citation-formatter":
        return {
            "metadata integrity": _has_all(
                lower,
                ["bibliography.bib", "doi", "duplicate", "missing required metadata"],
            )
            and ("normalize" in lower or "normalise" in lower),
        }
    if skill_id == "reference-manager-bridge":
        return {
            "local Zotero boundary": _has_all(
                lower,
                ["local reference database", "do not route scholarly discovery through zotero by default"],
            ),
            "Zotero write safety": _has_all(
                lower,
                [
                    "qiongli_zotero_status",
                    "dry-run",
                    "dry_run: false",
                    "fill blank",
                    "user-curated",
                    "qiongli_zotero_export_import_files",
                    "zotero-import-report.md",
                ],
            ),
            "no third-party Zotero plugin dependency": "better bibtex" not in lower
            and "no third-party zotero plugin is required" in lower,
        }
    return {}


def _skills_core_checks(root: Path) -> dict[str, bool]:
    path = RepoLayout(root).skills_core
    if not path.is_file():
        return {"skills-core direct API defaults": False}
    text = path.read_text(encoding="utf-8").lower()
    b_core = _extract_b_core(text)
    direct_default_patterns = (
        "semantic scholar api",
        "arxiv api",
        "openalex api",
        "web search for google scholar",
        "google scholar as fallback",
        "fallback: s2",
    )
    return {
        "skills-core direct API defaults": not any(pattern in b_core for pattern in direct_default_patterns),
    }


def _extract_b_core(text: str) -> str:
    start = text.find("## academic-searcher")
    if start == -1:
        return text
    end = text.find("## gap-analyzer", start)
    return text[start:] if end == -1 else text[start:end]


def _has_task_stage_relation(text: str) -> bool:
    return bool(re.search(r"\bB[0-9](?:_[0-9])?\b", text) or re.search(r"^stage:\s*B_literature", text, re.M))


def _has_compact_provider_references(text: str) -> bool:
    headings = re.findall(r"^#{2,4}\s+.*(?:API|Endpoint|Using Semantic Scholar|Using OpenAlex)", text, re.M)
    return "## API Reference" not in text and len(headings) <= 1


def _has_all(text: str, terms: list[str]) -> bool:
    return all(term in text for term in terms)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Audit Stage B literature skill semantic precision.")
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="Repository root")
    parser.add_argument("--strict", action="store_true", help="Return non-zero when gaps exist")
    args = parser.parse_args(argv)

    result = audit_b_literature_skill_precision(args.root)
    print(render_markdown_report(result))
    if args.strict and result.has_gaps:
        print("[b-literature-skill-audit] precision gaps found", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
