#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout


REQUIRED_SECTIONS = [
    "Purpose",
    "Inputs",
    "Process",
    "Output Contract",
    "Quality Bar",
    "Common Pitfalls",
]

CORE_BATCH_STAGES = {"A_framing", "B_literature", "F_writing", "J_proofread", "I_code"}

PLATFORM_SPECIFIC_PATTERNS = [
    re.compile(r"\bclaude\s+code\b", re.IGNORECASE),
    re.compile(r"\bclaude\s+-p\b", re.IGNORECASE),
    re.compile(r"\bcodex\s+exec\b", re.IGNORECASE),
    re.compile(r"\bgemini\s+-p\b", re.IGNORECASE),
]


@dataclass
class CoverageCount:
    present: int = 0
    total: int = 0

    @property
    def percent(self) -> float:
        if self.total == 0:
            return 0.0
        return self.present / self.total * 100


@dataclass
class SkillAuditResult:
    path: Path
    stage: str
    sections: dict[str, bool]
    constraints: dict[str, bool]

    @property
    def missing_sections(self) -> list[str]:
        return [name for name in REQUIRED_SECTIONS if not self.sections.get(name, False)]

    @property
    def missing_constraints(self) -> list[str]:
        return [name for name, present in self.constraints.items() if not present]

    @property
    def issue_count(self) -> int:
        return len(self.missing_sections) + len(self.missing_constraints)

    @property
    def is_complete(self) -> bool:
        return self.issue_count == 0


@dataclass
class AuditResult:
    root: Path
    skill_results: list[SkillAuditResult] = field(default_factory=list)

    @property
    def total_skills(self) -> int:
        return len(self.skill_results)

    @property
    def complete_skills(self) -> int:
        return sum(1 for item in self.skill_results if item.is_complete)

    @property
    def section_coverage(self) -> dict[str, CoverageCount]:
        coverage = {name: CoverageCount(total=self.total_skills) for name in REQUIRED_SECTIONS}
        for item in self.skill_results:
            for name in REQUIRED_SECTIONS:
                if item.sections.get(name, False):
                    coverage[name].present += 1
        return coverage

    @property
    def stage_coverage(self) -> dict[str, CoverageCount]:
        coverage: dict[str, CoverageCount] = {}
        for item in self.skill_results:
            count = coverage.setdefault(item.stage, CoverageCount())
            count.total += 1
            if item.is_complete:
                count.present += 1
        return dict(sorted(coverage.items()))

    @property
    def first_batch_priority(self) -> list[SkillAuditResult]:
        return [
            item
            for item in sorted(
                self.skill_results,
                key=lambda result: (
                    result.stage not in CORE_BATCH_STAGES,
                    -result.issue_count,
                    str(result.path),
                ),
            )
            if item.issue_count > 0
        ]

    @property
    def has_gaps(self) -> bool:
        return any(not item.is_complete for item in self.skill_results)


def _read_skill_files(root: Path) -> list[Path]:
    skills_dir = RepoLayout(root).skills
    if not skills_dir.is_dir():
        return []
    return sorted(path for path in skills_dir.rglob("*.md") if path.is_file())


def _extract_sections(text: str) -> set[str]:
    sections = set()
    for match in re.finditer(r"^##\s+(.+?)\s*$", text, flags=re.MULTILINE):
        name = re.sub(r"\s+\(.+?\)\s*$", "", match.group(1).strip())
        sections.add(name)
    return sections


def _stage_for(path: Path, root: Path, text: str) -> str:
    match = re.search(r"^stage:\s*([A-Za-z0-9_ -]+)\s*$", text, flags=re.MULTILINE)
    if match:
        return match.group(1).strip()
    try:
        rel = path.relative_to(RepoLayout(root).skills)
    except ValueError:
        return "unknown"
    return rel.parts[0] if rel.parts else "unknown"


def _has_platform_neutral_wording(text: str) -> bool:
    for pattern in PLATFORM_SPECIFIC_PATTERNS:
        if pattern.search(text):
            return False
    return True


def _constraint_coverage(text: str) -> dict[str, bool]:
    lowered = text.lower()
    return {
        "canonical artifact path": bool(
            re.search(r"RESEARCH/\[topic\]/[A-Za-z0-9_./-]+", text)
            or re.search(r"artifact:\s*[\"'][A-Za-z0-9_./-]+\.(md|csv|json|bib|yaml|yml|txt|R|py)[\"']", text)
            or re.search(r"`[A-Za-z0-9_./-]+\.(md|csv|json|bib|yaml|yml|txt|R|py)`", text)
        ),
        "task-stage relation": bool(
            re.search(r"\b[A-K][0-9](?:_[0-9])?\b", text)
            or re.search(r"^stage:\s*[A-Za-z0-9_ -]+$", text, flags=re.MULTILINE)
        ),
        "evidence handling": any(term in lowered for term in ("evidence", "citation", "literature", "source")),
        "insufficient-input behavior": any(
            term in lowered for term in ("missing input", "missing inputs", "insufficient", "gap note", "if inputs are missing")
        ),
        "claim strength rules": all(term in lowered for term in ("finding", "interpretation", "implication")),
        "no hallucinated citations/data": any(
            term in lowered
            for term in (
                "do not invent",
                "do not fabricate",
                "no hallucinated",
                "never invent",
                "not invent",
                "不编造",
            )
        ),
        "platform-neutral wording": _has_platform_neutral_wording(text),
    }


def audit_skills(root: Path) -> AuditResult:
    root = root.resolve()
    result = AuditResult(root=root)
    for path in _read_skill_files(root):
        text = path.read_text(encoding="utf-8")
        section_names = _extract_sections(text)
        sections = {name: name in section_names for name in REQUIRED_SECTIONS}
        result.skill_results.append(
            SkillAuditResult(
                path=path.relative_to(root),
                stage=_stage_for(path, root, text),
                sections=sections,
                constraints=_constraint_coverage(text),
            )
        )
    return result


def _format_percent(count: CoverageCount) -> str:
    return f"{count.present}/{count.total} ({count.percent:.1f}%)"


def render_markdown_report(result: AuditResult) -> str:
    lines = [
        "# Skill Quality Gap Report",
        "",
        "## Summary",
        "",
        f"- Total skills scanned: {result.total_skills}",
        f"- Complete skills: {result.complete_skills}/{result.total_skills}",
        "",
        "## Required Section Coverage",
        "",
        "| Section | Coverage |",
        "|---------|----------|",
    ]
    for section, count in result.section_coverage.items():
        lines.append(f"| {section} | {_format_percent(count)} |")

    lines.extend(
        [
            "",
            "## Stage Coverage",
            "",
            "| Stage | Complete Coverage |",
            "|-------|-------------------|",
        ]
    )
    for stage, count in result.stage_coverage.items():
        lines.append(f"| {stage} | {_format_percent(count)} |")

    lines.extend(
        [
            "",
            "## First Batch Priority",
            "",
            "Prioritize A/B/F/J/I stages, then sort by missing coverage count.",
            "",
            "| Skill | Stage | Missing Sections | Missing Constraints |",
            "|-------|-------|------------------|---------------------|",
        ]
    )
    for item in result.first_batch_priority[:30]:
        missing_sections = ", ".join(item.missing_sections) or "-"
        missing_constraints = ", ".join(item.missing_constraints) or "-"
        lines.append(f"| `{item.path}` | {item.stage} | {missing_sections} | {missing_constraints} |")

    lines.extend(["", "## Full Matrix", ""])
    for item in sorted(result.skill_results, key=lambda entry: str(entry.path)):
        lines.append(f"### `{item.path}`")
        lines.append("")
        lines.append(f"- Stage: {item.stage}")
        lines.append(f"- Missing sections: {', '.join(item.missing_sections) or '-'}")
        lines.append(f"- Missing constraints: {', '.join(item.missing_constraints) or '-'}")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Audit academic skill quality section and content coverage.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1], help="Repository root")
    parser.add_argument("--output", type=Path, help="Optional markdown report path")
    parser.add_argument("--strict", action="store_true", help="Return non-zero when any skill has missing coverage")
    args = parser.parse_args(argv)

    result = audit_skills(args.root)
    report = render_markdown_report(result)
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(report, encoding="utf-8")
    else:
        print(report)

    if args.strict and result.has_gaps:
        print("[skill-audit] missing required skill quality coverage", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
