from __future__ import annotations

from pathlib import Path


WORKFLOW_TRIGGER_DESCRIPTIONS = {
    "academic-present": (
        "Use when the user asks Qiongli for academic slides, scholarly presentation planning, "
        "Slidev, Beamer, PowerPoint, talk structure, or /academic-present routing."
    ),
    "academic-write": (
        "Use when the user asks Qiongli for manuscript writing, academic writing, related work, "
        "abstract, introduction, discussion, section drafting, or /academic-write routing."
    ),
    "build-framework": (
        "Use when the user asks Qiongli for conceptual framework, theory mapping, contribution "
        "framing, construct relationships, model building, or /build-framework routing."
    ),
    "code-build": (
        "Use when the user asks Qiongli for academic analysis code, notebooks, reproducibility "
        "scripts, statistical pipelines, replication packages, or /code-build routing."
    ),
    "compliance-check": (
        "Use when the user asks Qiongli for reporting compliance, PRISMA, CONSORT, STROBE, "
        "checklist review, citation risk, or /compliance-check routing."
    ),
    "ethics-check": (
        "Use when the user asks Qiongli for ethics, IRB text, consent, deidentification, "
        "data governance, disclosure, or /ethics-check routing."
    ),
    "find-gap": (
        "Use when the user asks Qiongli to find research gaps, narrow a topic, assess novelty, "
        "choose a direction, map contributions, or /find-gap routing."
    ),
    "lit-review": (
        "Use when the user asks Qiongli for literature review, systematic review, PRISMA search, "
        "screening, extraction, synthesis, evidence mapping, related work, or /lit-review routing."
    ),
    "paper": (
        "Use when the user asks Qiongli for end-to-end paper planning, research workflow routing, "
        "academic project setup, stage selection, or /paper routing."
    ),
    "paper-read": (
        "Use when the user asks Qiongli to read a paper, PDF, DOI, article, notes, claims, "
        "methods, contribution, evidence, or /paper-read routing."
    ),
    "paper-write": (
        "Use when the user asks Qiongli to assemble a paper, draft manuscript sections, "
        "integrate upstream research artifacts, or /paper-write routing."
    ),
    "proofread": (
        "Use when the user asks Qiongli to proofread, polish academic prose, reduce AI-like "
        "wording, check tone, final copyedit, or /proofread routing."
    ),
    "rebuttal": (
        "Use when the user asks Qiongli for reviewer response, rebuttal letter, response matrix, "
        "peer review comments, revision strategy, or /rebuttal routing."
    ),
    "study-design": (
        "Use when the user asks Qiongli for study design, variables, robustness, instruments, "
        "data management, preregistration, methods, or /study-design routing."
    ),
    "submission-prep": (
        "Use when the user asks Qiongli for submission package, cover letter, author contributions, "
        "reporting checks, journal readiness, or /submission-prep routing."
    ),
    "synthesize": (
        "Use when the user asks Qiongli for evidence synthesis, meta-analysis, qualitative "
        "synthesis, effect sizes, quality assessment, or /synthesize routing."
    ),
}


def write_codex_workflow_wrapper_skills(
    workflow_root: Path,
    skills_root: Path,
    *,
    skill_name: str,
    canonical_skill_dir: str,
) -> list[Path]:
    """Write thin Codex skill adapters for each canonical workflow entrypoint."""

    skills_root.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    for workflow_path in sorted(workflow_root.glob("*.md")):
        wrapper_name = f"{skill_name}-{workflow_path.stem}"
        wrapper_dir = skills_root / wrapper_name
        wrapper_dir.mkdir(parents=True, exist_ok=True)
        skill_path = wrapper_dir / "SKILL.md"
        skill_path.write_text(
            render_codex_workflow_wrapper_skill(
                workflow_path.stem,
                wrapper_name=wrapper_name,
                skill_name=skill_name,
                canonical_skill_dir=canonical_skill_dir,
            ),
            encoding="utf-8",
        )
        written.append(skill_path)
    return written


def render_codex_workflow_wrapper_skill(
    workflow_slug: str,
    *,
    wrapper_name: str,
    skill_name: str,
    canonical_skill_dir: str,
) -> str:
    description = WORKFLOW_TRIGGER_DESCRIPTIONS.get(
        workflow_slug,
        (
            "Use when the user asks Qiongli for the "
            f"{workflow_slug.replace('-', ' ')} workflow, /{workflow_slug} routing, "
            "or the matching academic research stage."
        ),
    )
    title = " ".join(part.capitalize() for part in wrapper_name.split("-"))
    canonical_workflow = f"../{canonical_skill_dir}/workflows/{workflow_slug}.md"
    return "\n".join(
        [
            "---",
            f"name: {wrapper_name}",
            f"description: {description}",
            "---",
            "",
            f"# {title}",
            "",
            f"This Codex wrapper mirrors the cross-platform `/{workflow_slug}` workflow entrypoint.",
            "",
            "## Canonical Route",
            "",
            (
                f"- Use `${skill_name}` as the main Qiongli skill for trigger rules, "
                "project guidance, and subject overlays."
            ),
            f"- Follow `{canonical_workflow}` as the source of truth for task order, artifacts, and quality gates.",
            f"- Keep behavior aligned with Claude Code `/{workflow_slug}` and Antigravity workflow routing.",
            "",
            (
                "Do not duplicate or reinterpret workflow logic in this wrapper. If this wrapper "
                "and the canonical workflow disagree, the canonical workflow wins."
            ),
            "",
        ]
    )
