#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout

try:
    from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package, validate_subject_catalog
except ModuleNotFoundError as exc:
    if exc.name != "yaml":
        raise
    MaterializeOptions = None
    materialize_subject_package = None
    validate_subject_catalog = None


PLUGIN_NAME = "qiongli"
PLUGIN_ROOT = Path("plugins") / PLUGIN_NAME
SKILL_DIR_NAME = "qiongli-workflow"
DESKTOP_SKILL_FILE_BUDGET = 180
FALLBACK_SUBJECT_LAYERS = {
    "core": ["core"],
    "business": ["core", "business"],
    "economics": ["core", "economics"],
    "accounting": ["core", "accounting"],
    "economics-accounting": ["core", "economics", "accounting", "economics-accounting"],
    "finance": ["core", "finance"],
}
BUSINESS_SKILL_REFS = (
    "question-refiner",
    "contribution-crafter",
    "gap-analyzer",
    "theory-mapper",
    "hypothesis-generator",
    "venue-analyzer",
    "academic-searcher",
    "citation-snowballer",
    "literature-mapper",
    "paper-extractor",
    "paper-screener",
    "reference-manager-bridge",
    "study-designer",
    "rival-hypothesis-designer",
    "robustness-planner",
    "dataset-finder",
    "variable-constructor",
    "data-dictionary-builder",
    "qualitative-coding",
    "stats-engine",
    "business-journal-positioning-auditor",
    "effect-size-calculator",
    "evidence-synthesizer",
    "analysis-interpreter",
    "effect-size-interpreter",
    "table-generator",
    "figure-specifier",
    "manuscript-architect",
    "discussion-writer",
    "meta-optimizer",
    "reporting-checker",
    "tone-normalizer",
    "submission-packager",
    "fatal-flaw-detector",
    "code-builder",
    "code-review",
    "reproducibility-auditor",
    "final-proofreader",
)
ECONOMICS_SKILL_REFS = (
    "question-refiner",
    "contribution-crafter",
    "gap-analyzer",
    "theory-mapper",
    "hypothesis-generator",
    "venue-analyzer",
    "academic-searcher",
    "citation-snowballer",
    "literature-mapper",
    "paper-extractor",
    "paper-screener",
    "reference-manager-bridge",
    "study-designer",
    "rival-hypothesis-designer",
    "robustness-planner",
    "dataset-finder",
    "variable-constructor",
    "data-dictionary-builder",
    "prereg-writer",
    "stats-engine",
    "econ-identification-auditor",
    "effect-size-calculator",
    "evidence-synthesizer",
    "analysis-interpreter",
    "effect-size-interpreter",
    "table-generator",
    "figure-specifier",
    "manuscript-architect",
    "discussion-writer",
    "meta-optimizer",
    "reporting-checker",
    "tone-normalizer",
    "submission-packager",
    "fatal-flaw-detector",
    "code-builder",
    "code-review",
    "reproducibility-auditor",
    "final-proofreader",
)
ECONOMICS_ACCOUNTING_SKILL_REFS = (
    *ECONOMICS_SKILL_REFS,
    "accounting-measurement-auditor",
)
ACCOUNTING_SKILL_REFS = (
    "question-refiner",
    "contribution-crafter",
    "gap-analyzer",
    "theory-mapper",
    "hypothesis-generator",
    "venue-analyzer",
    "academic-searcher",
    "citation-snowballer",
    "literature-mapper",
    "paper-extractor",
    "paper-screener",
    "reference-manager-bridge",
    "study-designer",
    "rival-hypothesis-designer",
    "robustness-planner",
    "dataset-finder",
    "variable-constructor",
    "data-dictionary-builder",
    "prereg-writer",
    "stats-engine",
    "accounting-measurement-auditor",
    "effect-size-calculator",
    "evidence-synthesizer",
    "analysis-interpreter",
    "effect-size-interpreter",
    "table-generator",
    "figure-specifier",
    "manuscript-architect",
    "discussion-writer",
    "meta-optimizer",
    "reporting-checker",
    "tone-normalizer",
    "submission-packager",
    "fatal-flaw-detector",
    "code-builder",
    "code-review",
    "reproducibility-auditor",
    "final-proofreader",
)
FINANCE_SKILL_REFS = (
    "question-refiner",
    "contribution-crafter",
    "gap-analyzer",
    "theory-mapper",
    "hypothesis-generator",
    "venue-analyzer",
    "academic-searcher",
    "citation-snowballer",
    "literature-mapper",
    "paper-extractor",
    "paper-screener",
    "reference-manager-bridge",
    "study-designer",
    "rival-hypothesis-designer",
    "robustness-planner",
    "dataset-finder",
    "variable-constructor",
    "data-dictionary-builder",
    "prereg-writer",
    "stats-engine",
    "finance-identification-risk-auditor",
    "effect-size-calculator",
    "evidence-synthesizer",
    "analysis-interpreter",
    "effect-size-interpreter",
    "table-generator",
    "figure-specifier",
    "manuscript-architect",
    "discussion-writer",
    "meta-optimizer",
    "reporting-checker",
    "tone-normalizer",
    "submission-packager",
    "fatal-flaw-detector",
    "code-builder",
    "code-review",
    "reproducibility-auditor",
    "final-proofreader",
)
ECONOMICS_TEMPLATES = (
    "analysis-plan.md",
    "claim-evidence-ledger.csv",
    "data-availability.md",
    "data-management-plan.md",
    "figures-tables-plan.md",
    "manuscript-outline.md",
    "method-diagnostic-report.md",
    "quality-gate-report.md",
    "research-state.md",
    "search-log.md",
    "stage-handoff.md",
    "study-design.md",
    "validity-threat-matrix.md",
    "writing-claim-map.md",
    "code/economics/causal_did.py",
    "code/statistics/meta_analysis_random_effects.py",
)
ACCOUNTING_TEMPLATES = (
    "analysis-plan.md",
    "claim-evidence-ledger.csv",
    "data-availability.md",
    "data-management-plan.md",
    "figures-tables-plan.md",
    "manuscript-outline.md",
    "method-diagnostic-report.md",
    "quality-gate-report.md",
    "research-state.md",
    "search-log.md",
    "stage-handoff.md",
    "study-design.md",
    "validity-threat-matrix.md",
    "writing-claim-map.md",
)
BUSINESS_TEMPLATES = ACCOUNTING_TEMPLATES
FINANCE_TEMPLATES = (*ACCOUNTING_TEMPLATES, "code/statistics/meta_analysis_random_effects.py")


def _normalize_tag(raw: str) -> tuple[str, str]:
    tag = raw.strip()
    if not tag:
        raise ValueError("tag is required")
    repo_tag = tag if tag.startswith("v") else f"v{tag}"
    skill_version = repo_tag.removeprefix("v")
    return repo_tag, skill_version


def _read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def _collect_versions(value: object) -> list[str]:
    versions: list[str] = []
    if isinstance(value, dict):
        for key, item in value.items():
            if key == "version" and isinstance(item, str):
                versions.append(item)
            else:
                versions.extend(_collect_versions(item))
    elif isinstance(value, list):
        for item in value:
            versions.extend(_collect_versions(item))
    return versions


def _assert_json_versions(path: Path, expected_version: str) -> None:
    data = _read_json(path)
    versions = _collect_versions(data)
    if not versions:
        raise ValueError(f"missing version in {path}")
    for version in versions:
        if version != expected_version:
            raise ValueError(f"version mismatch in {path}: expected {expected_version}, found {version}")


def _copy_path(src: Path, dest: Path) -> None:
    if src.is_dir():
        shutil.copytree(src, dest)
    else:
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)


def _copy_path_excluding(src: Path, dest: Path, excluded_names: set[str]) -> None:
    if dest.exists():
        if dest.is_dir():
            shutil.rmtree(dest)
        else:
            dest.unlink()
    if src.is_dir():
        shutil.copytree(src, dest, ignore=lambda _copy_src, names: {name for name in names if name in excluded_names})
    else:
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)


def _build_materialize_source(root: Path, work_dir: Path) -> Path:
    layout = RepoLayout(root)
    source = work_dir / "materialize-source"
    _copy_path(layout.workflow, source / "qiongli-workflow")
    _copy_path(layout.skills, source / "skills")
    _copy_path(layout.subjects, source / "subjects")
    source_dirs = {
        "skills": layout.skills,
        "templates": layout.templates,
        "standards": layout.standards,
        "roles": layout.roles,
        "venue-profiles": layout.venue_profiles,
    }
    for item, src in source_dirs.items():
        if src.exists():
            excluded = {"CLAUDE.project.md"} if item == "templates" else set()
            _copy_path_excluding(src, source / "qiongli-workflow" / item, excluded)
    source_files = {
        "skills-core.md": layout.skills_core,
        "skills-summary.md": layout.skills_summary,
    }
    for item, src in source_files.items():
        if src.exists():
            _copy_path(src, source / "qiongli-workflow" / item)
    return source


def _copy_common_skill(root: Path, dest_plugin_root: Path) -> None:
    generated_skill = root / PLUGIN_ROOT / "skills"
    if generated_skill.is_dir():
        _copy_path(generated_skill, dest_plugin_root / "skills")
        return

    _copy_subject_skill(root, dest_plugin_root, "core")


def _copy_subject_skill(root: Path, dest_plugin_root: Path, subject: str) -> None:
    if materialize_subject_package is None or MaterializeOptions is None:
        raise ValueError("PyYAML is required to build subject-specific marketplace plugin artifacts")

    with tempfile.TemporaryDirectory(prefix=f"qiongli-marketplace-{subject}-source-") as tmp:
        materialize_root = _build_materialize_source(root, Path(tmp))
        materialize_subject_package(
            MaterializeOptions(
                source=materialize_root,
                out=dest_plugin_root / "skills" / SKILL_DIR_NAME,
                subject=subject,
                flavor="full",
                coverage="complete",
            )
        )


def _copy_commands(root: Path, dest_plugin_root: Path) -> None:
    commands = root / PLUGIN_ROOT / "commands"
    if commands.is_dir():
        _copy_path(commands, dest_plugin_root / "commands")


def _make_tarball(source_dir: Path, tar_path: Path) -> None:
    tar_path.parent.mkdir(parents=True, exist_ok=True)
    if tar_path.exists():
        tar_path.unlink()
    with tarfile.open(tar_path, "w:gz") as tar:
        tar.add(source_dir, arcname=source_dir.name)


def _make_zip(source_dir: Path, zip_path: Path) -> None:
    zip_path.parent.mkdir(parents=True, exist_ok=True)
    if zip_path.exists():
        zip_path.unlink()
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for item in sorted(source_dir.rglob("*")):
            if item.is_file():
                archive.write(item, item.relative_to(source_dir.parent).as_posix())


def _copy_claude_desktop_skill(root: Path, skill_dest: Path, subject: str) -> None:
    with tempfile.TemporaryDirectory(prefix="qiongli-desktop-source-") as tmp:
        materialize_root = _build_materialize_source(root, Path(tmp))
        if materialize_subject_package is not None and MaterializeOptions is not None:
            materialize_subject_package(
                MaterializeOptions(
                    source=materialize_root,
                    out=skill_dest,
                    subject=subject,
                    flavor="desktop",
                    coverage="focused",
                )
            )
        else:
            _copy_claude_desktop_skill_without_pyyaml(materialize_root, skill_dest, subject)
    file_count = sum(1 for item in skill_dest.rglob("*") if item.is_file())
    if file_count > DESKTOP_SKILL_FILE_BUDGET:
        raise ValueError(
            f"Claude Desktop {subject} skill package has {file_count} files; "
            f"limit is {DESKTOP_SKILL_FILE_BUDGET}"
        )


def _copy_claude_desktop_skill_without_pyyaml(root: Path, skill_dest: Path, subject: str) -> None:
    if subject not in FALLBACK_SUBJECT_LAYERS:
        raise ValueError("PyYAML is required to materialize unknown subject packages")
    source = root / "qiongli-workflow"
    skill_dest.mkdir(parents=True)
    for filename in ("VERSION", "skills-core.md", "skills-summary.md"):
        _copy_path(source / filename, skill_dest / filename)
    for dirname in ("workflows", "references", "standards", "roles", "agents"):
        source_path = source / dirname
        if source_path.exists():
            _copy_path(source_path, skill_dest / dirname)

    (skill_dest / "SUBJECT").write_text(subject + "\n", encoding="utf-8")
    (skill_dest / "SUBJECT_MANIFEST.json").write_text(
        json.dumps(
            {
                "subject": subject,
                "coverage": "focused",
                "flavor": "desktop",
                "layers": FALLBACK_SUBJECT_LAYERS[subject],
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    if subject == "core":
        _write_fallback_skill_md(skill_dest, "Qiongli Core", "General-purpose Qiongli academic workflow.")
        _copy_path(source / "templates", skill_dest / "templates")
        _copy_path(source / "venue-profiles", skill_dest / "venue-profiles")
        _copy_path(source / "skills" / "registry.yaml", skill_dest / "skills" / "registry.yaml")
        _copy_path(source / "skills" / "domain-profiles", skill_dest / "skills" / "domain-profiles")
        return

    if subject == "business":
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Business",
            "Business-focused management, strategy, organization, marketing, and operations workflow for doctoral-level journal manuscripts.",
        )
    elif subject == "economics-accounting":
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Economics + Accounting",
            "Cross-disciplinary economics and accounting workflow for archival, causal, and reporting-setting research.",
        )
    elif subject == "finance":
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Finance",
            "Finance-focused corporate finance, asset pricing, market microstructure, and risk workflow for doctoral-level journal manuscripts.",
        )
    elif subject == "accounting":
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Accounting",
            "Accounting-focused archival, disclosure, audit, and measurement workflow.",
        )
    else:
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Economics",
            "Economics-focused empirical, theory, and reproducibility workflow.",
        )
    if subject == "accounting":
        template_refs = ACCOUNTING_TEMPLATES
    elif subject == "business":
        template_refs = BUSINESS_TEMPLATES
    elif subject == "finance":
        template_refs = FINANCE_TEMPLATES
    else:
        template_refs = ECONOMICS_TEMPLATES
    for rel in template_refs:
        _copy_path(source / "templates" / rel, skill_dest / "templates" / rel)
    if subject in {"economics", "economics-accounting"}:
        _copy_path(source / "skills" / "domain-profiles" / "economics.yaml", skill_dest / "skills" / "domain-profiles" / "economics.yaml")
    if subject == "business":
        _copy_path(root / "skills" / "domain-profiles" / "business-management.yaml", skill_dest / "skills" / "domain-profiles" / "business-management.yaml")
        for venue in ("academy-of-management-journal", "organization-science", "strategic-management-journal"):
            _copy_path(root / "subjects" / "business" / "venue-profiles" / f"{venue}.yaml", skill_dest / "venue-profiles" / f"{venue}.yaml")
    if subject == "finance":
        _copy_path(root / "skills" / "domain-profiles" / "finance.yaml", skill_dest / "skills" / "domain-profiles" / "finance.yaml")
        for venue in ("journal-of-finance", "review-of-financial-studies", "journal-of-financial-economics"):
            _copy_path(root / "subjects" / "finance" / "venue-profiles" / f"{venue}.yaml", skill_dest / "venue-profiles" / f"{venue}.yaml")
    if subject == "accounting":
        _copy_path(root / "skills" / "domain-profiles" / "accounting.yaml", skill_dest / "skills" / "domain-profiles" / "accounting.yaml")
        for venue in ("accounting-review", "journal-of-accounting-research", "review-of-accounting-studies"):
            _copy_path(
                root / "subjects" / "accounting" / "venue-profiles" / f"{venue}.yaml",
                skill_dest / "venue-profiles" / f"{venue}.yaml",
            )
    if subject == "economics-accounting":
        _copy_path(root / "skills" / "domain-profiles" / "accounting.yaml", skill_dest / "skills" / "domain-profiles" / "accounting.yaml")
        for venue in ("aer", "qje", "restud"):
            _copy_path(root / "subjects" / "economics" / "venue-profiles" / f"{venue}.yaml", skill_dest / "venue-profiles" / f"{venue}.yaml")
        for venue in ("accounting-review", "journal-of-accounting-research", "review-of-accounting-studies"):
            _copy_path(
                root / "subjects" / "economics-accounting" / "venue-profiles" / f"{venue}.yaml",
                skill_dest / "venue-profiles" / f"{venue}.yaml",
            )
    elif subject == "economics":
        for venue in ("aer", "qje", "restud"):
            _copy_path(root / "subjects" / "economics" / "venue-profiles" / f"{venue}.yaml", skill_dest / "venue-profiles" / f"{venue}.yaml")

    entries = _fallback_registry_entries(root)
    registry_lines = ["skills:"]
    if subject == "accounting":
        skill_refs = ACCOUNTING_SKILL_REFS
    elif subject == "business":
        skill_refs = BUSINESS_SKILL_REFS
    elif subject == "economics-accounting":
        skill_refs = ECONOMICS_ACCOUNTING_SKILL_REFS
    elif subject == "finance":
        skill_refs = FINANCE_SKILL_REFS
    else:
        skill_refs = ECONOMICS_SKILL_REFS
    for skill_id in skill_refs:
        rel = entries[skill_id]
        registry_lines.extend([f"  - id: {skill_id}", f"    file: {rel}"])
        if skill_id == "econ-identification-auditor":
            src = root / "subjects" / "economics" / "skills" / "econ-identification-auditor.md"
        elif skill_id == "accounting-measurement-auditor":
            src = root / "subjects" / "accounting" / "skills" / "accounting-measurement-auditor.md"
        elif skill_id == "business-journal-positioning-auditor":
            src = root / "subjects" / "business" / "skills" / "business-journal-positioning-auditor.md"
        elif skill_id == "finance-identification-risk-auditor":
            src = root / "subjects" / "finance" / "skills" / "finance-identification-risk-auditor.md"
        else:
            src = source / rel
        text = src.read_text(encoding="utf-8")
        if skill_id == "manuscript-architect":
            overlay_subject = subject if subject in {"accounting", "business", "economics-accounting", "finance"} else "economics"
            overlay = (root / "subjects" / overlay_subject / "overlays" / "skills" / "manuscript-architect.md").read_text(encoding="utf-8")
            text = text.rstrip() + "\n\n" + overlay.strip() + "\n"
        elif skill_id == "stats-engine":
            overlay_subject = subject if subject in {"accounting", "business", "economics-accounting", "finance"} else "economics"
            overlay = (root / "subjects" / overlay_subject / "overlays" / "skills" / "stats-engine.md").read_text(encoding="utf-8")
            for section in ("Quality Bar", "Common Pitfalls"):
                text = _replace_markdown_section(text, overlay, section)
        elif skill_id == "variable-constructor" and subject == "accounting":
            overlay = (root / "subjects" / "accounting" / "overlays" / "skills" / "variable-constructor.md").read_text(encoding="utf-8")
            text = text.rstrip() + "\n\n" + overlay.strip() + "\n"
        dest = skill_dest / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(text, encoding="utf-8")
    registry_path = skill_dest / "skills" / "registry.yaml"
    registry_path.parent.mkdir(parents=True, exist_ok=True)
    registry_path.write_text("\n".join(registry_lines) + "\n", encoding="utf-8")


def _write_fallback_skill_md(skill_dest: Path, display_name: str, description: str) -> None:
    text = "\n".join(
        [
            "---",
            "name: qiongli",
            f"description: {description}",
            "---",
            "",
            f"# {display_name}",
            "",
            description,
            "",
            f"## {display_name.removeprefix('Qiongli ')} Workflow Map",
            "",
            "Use `workflows/`, `references/`, `templates/`, and `skills/registry.yaml` as the active subject contract.",
            "",
            "## Literature Provider Configuration",
            "",
            "- CLI, Codex, and Claude Code installs can configure external literature providers with `qiongli provider setup` and audit them with `qiongli provider doctor`.",
            "- Treat `provider_connected` as the only mode where configured external academic provider credentials are available to the local runtime.",
            "- Treat `strategy_only` as a constrained mode: use platform search or user-supplied corpus, record the limitation, and do not claim review-grade external provider coverage.",
            "- Claude Desktop/Web focused ZIPs are skill-only packages kept within the 180-file upload budget. They contain workflows/prompts/templates, store no secrets, and cannot execute OpenAlex, Semantic Scholar, Crossref, or PubMed API calls by themselves.",
            "- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or platform-native search capability before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex and Semantic Scholar configuration. If no MCPB or platform-native search is available, record the run as `strategy_only`.",
            "",
        ]
    )
    (skill_dest / "SKILL.md").write_text(text, encoding="utf-8")


def _fallback_registry_entries(root: Path) -> dict[str, str]:
    entries: dict[str, str] = {}
    registries = [
        root / "skills" / "registry.yaml",
        *sorted((root / "subjects").glob("*/skills/registry.yaml")),
    ]
    for registry in registries:
        current_id: str | None = None
        for line in registry.read_text(encoding="utf-8").splitlines():
            id_match = re.match(r"\s*-\s*id:\s*[\"']?([^\"'\n#]+)", line)
            if id_match:
                current_id = id_match.group(1).strip()
                continue
            file_match = re.match(r"\s*file:\s*[\"']?([^\"'\n#]+)", line)
            if current_id and file_match:
                entries[current_id] = file_match.group(1).strip()
                current_id = None
    return entries


def _replace_markdown_section(base_text: str, overlay_text: str, section: str) -> str:
    base_range = _find_section_range(base_text, section)
    overlay_range = _find_section_range(overlay_text, section)
    if base_range is None or overlay_range is None:
        raise ValueError(f"fallback overlay missing section: {section}")
    base_start, base_end = base_range
    overlay_start, overlay_end = overlay_range
    replacement = overlay_text[overlay_start:overlay_end].strip() + "\n"
    return base_text[:base_start].rstrip() + "\n\n" + replacement + base_text[base_end:].lstrip()


def _find_section_range(text: str, section: str) -> tuple[int, int] | None:
    heading_re = re.compile(r"(?m)^(?P<marker>#{2})\s+(?P<title>.+?)\s*$")
    wanted = re.sub(r"\s+", " ", section.strip().lower())
    for match in heading_re.finditer(text):
        title = re.sub(r"\s+", " ", match.group("title").strip().lower())
        if title != wanted and not title.startswith(wanted + " "):
            continue
        next_match = heading_re.search(text, match.end())
        return match.start(), next_match.start() if next_match else len(text)
    return None


def _subject_definitions(root: Path) -> dict[str, tuple[str, str]]:
    if validate_subject_catalog is None:
        return {
            "core": ("Qiongli Core", "General-purpose Qiongli academic workflow."),
            "business": (
                "Qiongli Business",
                "Business-focused management, strategy, organization, marketing, and operations workflow for doctoral-level journal manuscripts.",
            ),
            "economics": ("Qiongli Economics", "Economics-focused empirical, theory, and reproducibility workflow."),
            "accounting": ("Qiongli Accounting", "Accounting-focused archival, disclosure, audit, and measurement workflow."),
            "economics-accounting": (
                "Qiongli Economics + Accounting",
                "Cross-disciplinary economics and accounting workflow for archival, causal, and reporting-setting research.",
            ),
            "finance": (
                "Qiongli Finance",
                "Finance-focused corporate finance, asset pricing, market microstructure, and risk workflow for doctoral-level journal manuscripts.",
            ),
        }
    catalog = validate_subject_catalog(root)
    return {
        subject_id: (subject.display_name, subject.package_goal)
        for subject_id, subject in catalog.subjects.items()
    }


def _marketplace_subjects(root: Path) -> list[str]:
    return sorted(_subject_definitions(root))


def _subject_plugin_name(subject: str) -> str:
    return f"{PLUGIN_NAME}-{subject}"


def _subject_modifier(display_name: str) -> str:
    return display_name.removeprefix("Qiongli ").strip() or "Core"


def _subject_description(display_name: str, package_goal: str, subject: str) -> str:
    if subject == "core":
        return f"General-purpose Qiongli marketplace plugin. {package_goal}"
    modifier = _subject_modifier(display_name)
    return (
        f"{modifier}-specialized Qiongli marketplace plugin. "
        f"Installs the {subject}/complete subject package with the full workflow plus subject overlays, "
        f"selected profiles, and subject-specific skills. {package_goal}"
    )


def _write_subject_manifest(
    manifest_path: Path,
    *,
    platform: str,
    plugin_name: str,
    subject: str,
    display_name: str,
    package_goal: str,
) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["name"] = plugin_name
    manifest["description"] = _subject_description(display_name, package_goal, subject)
    keywords = manifest.get("keywords")
    if isinstance(keywords, list):
        additions = ["qiongli-subject", subject]
        manifest["keywords"] = [*keywords, *[item for item in additions if item not in keywords]]

    if platform == "codex":
        interface = manifest.get("interface")
        if not isinstance(interface, dict):
            interface = {}
            manifest["interface"] = interface
        interface["displayName"] = display_name
        if subject == "core":
            interface["shortDescription"] = "General academic paper workflows for Codex."
            interface["defaultPrompt"] = [
                "Use $qiongli to plan my paper.",
                "Use $qiongli to run a literature review for my research topic.",
                "Use $qiongli to prepare a submission package for my manuscript.",
            ]
        else:
            modifier = _subject_modifier(display_name)
            interface["shortDescription"] = f"{modifier}-specialized academic workflows for Codex."
            interface["defaultPrompt"] = [
                f"Use $qiongli to plan my {modifier.lower()} paper.",
                f"Use $qiongli to run a {modifier.lower()} literature review.",
                f"Use $qiongli to prepare {modifier.lower()} methods, diagnostics, and reporting checks.",
            ]
        interface["longDescription"] = _subject_description(display_name, package_goal, subject)

    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _build_marketplace_plugin(
    root: Path,
    tag: str,
    dist_dir: Path,
    work_dir: Path,
    *,
    platform: str,
    subject: str,
    plugin_name: str,
    artifact_name: str,
    display_name: str,
    package_goal: str,
) -> Path:
    bundle_name = f"{artifact_name}-{platform}-plugin-{tag}"
    bundle = work_dir / bundle_name
    plugin_dest = bundle / "plugins" / plugin_name
    manifest_dir = ".codex-plugin" if platform == "codex" else ".claude-plugin"
    _copy_path(root / PLUGIN_ROOT / manifest_dir, plugin_dest / manifest_dir)
    _write_subject_manifest(
        plugin_dest / manifest_dir / "plugin.json",
        platform=platform,
        plugin_name=plugin_name,
        subject=subject,
        display_name=display_name,
        package_goal=package_goal,
    )
    _copy_commands(root, plugin_dest)
    _copy_subject_skill(root, plugin_dest, subject)
    artifact = dist_dir / f"{bundle_name}.tar.gz"
    _make_tarball(bundle, artifact)
    return artifact


def _build_codex(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    display_name, package_goal = _subject_definitions(root)["core"]
    return _build_marketplace_plugin(
        root,
        tag,
        dist_dir,
        work_dir,
        platform="codex",
        subject="core",
        plugin_name=PLUGIN_NAME,
        artifact_name=PLUGIN_NAME,
        display_name="Qiongli",
        package_goal=package_goal,
    )


def _build_claude(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    display_name, package_goal = _subject_definitions(root)["core"]
    return _build_marketplace_plugin(
        root,
        tag,
        dist_dir,
        work_dir,
        platform="claude",
        subject="core",
        plugin_name=PLUGIN_NAME,
        artifact_name=PLUGIN_NAME,
        display_name="Qiongli",
        package_goal=package_goal,
    )


def _build_subject_marketplace_plugins(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> list[Path]:
    subject_defs = _subject_definitions(root)
    artifacts: list[Path] = []
    for subject in _marketplace_subjects(root):
        display_name, package_goal = subject_defs[subject]
        plugin_name = _subject_plugin_name(subject)
        artifact_name = plugin_name
        for platform in ("codex", "claude"):
            artifacts.append(
                _build_marketplace_plugin(
                    root,
                    tag,
                    dist_dir,
                    work_dir,
                    platform=platform,
                    subject=subject,
                    plugin_name=plugin_name,
                    artifact_name=artifact_name,
                    display_name=display_name,
                    package_goal=package_goal,
                )
            )
    return artifacts


def _build_gemini(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> Path:
    bundle_name = f"{PLUGIN_NAME}-gemini-extension-{tag}"
    bundle = work_dir / bundle_name
    _copy_path(root / PLUGIN_ROOT / "gemini-extension.json", bundle / "gemini-extension.json")
    _copy_common_skill(root, bundle)
    artifact = dist_dir / f"{bundle_name}.tar.gz"
    _make_tarball(bundle, artifact)
    return artifact


def _build_claude_desktop_skill(root: Path, tag: str, dist_dir: Path, work_dir: Path, subject: str) -> Path:
    bundle_name = f"{PLUGIN_NAME}-claude-desktop-skill-{subject}-{tag}"
    skill_dest = work_dir / f"desktop-{subject}" / "qiongli"
    _copy_claude_desktop_skill(root, skill_dest, subject)
    artifact = dist_dir / f"{bundle_name}.zip"
    _make_zip(skill_dest, artifact)
    return artifact


def _desktop_subjects(root: Path) -> list[str]:
    return [
        subject
        for subject in _marketplace_subjects(root)
        if subject != "accounting"
    ]


def build_artifacts(root: Path, raw_tag: str, dist_dir: Path) -> list[Path]:
    root = root.resolve()
    layout = RepoLayout(root)
    dist_dir = dist_dir.resolve()
    repo_tag, skill_version = _normalize_tag(raw_tag)

    workflow_version = (layout.workflow / "VERSION").read_text(encoding="utf-8").strip()
    if workflow_version != repo_tag:
        raise ValueError(f"version mismatch in qiongli-workflow/VERSION: expected {repo_tag}, found {workflow_version}")

    versioned_json = [
        root / PLUGIN_ROOT / ".codex-plugin" / "plugin.json",
        root / PLUGIN_ROOT / ".claude-plugin" / "plugin.json",
        root / PLUGIN_ROOT / "gemini-extension.json",
    ]
    for path in versioned_json:
        _assert_json_versions(path, skill_version)

    with tempfile.TemporaryDirectory(prefix="qiongli-plugin-") as tmp:
        work_dir = Path(tmp)
        desktop_artifacts = [
            _build_claude_desktop_skill(root, repo_tag, dist_dir, work_dir, subject)
            for subject in _desktop_subjects(root)
        ]
        legacy_desktop_artifact = dist_dir / f"{PLUGIN_NAME}-claude-desktop-skill-{repo_tag}.zip"
        shutil.copy2(dist_dir / f"{PLUGIN_NAME}-claude-desktop-skill-core-{repo_tag}.zip", legacy_desktop_artifact)
        subject_marketplace_artifacts = _build_subject_marketplace_plugins(root, repo_tag, dist_dir, work_dir)
        artifacts = [
            _build_codex(root, repo_tag, dist_dir, work_dir),
            _build_claude(root, repo_tag, dist_dir, work_dir),
            *subject_marketplace_artifacts,
            _build_gemini(root, repo_tag, dist_dir, work_dir),
            *desktop_artifacts,
            legacy_desktop_artifact,
        ]
    return artifacts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build Codex, Claude Code, and Gemini plugin/extension artifacts.")
    parser.add_argument("--tag", required=True, help="Release tag, for example v0.5.0-beta.3")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1], help="Repository root")
    parser.add_argument("--dist-dir", type=Path, default=Path("dist"), help="Output directory")
    args = parser.parse_args(argv)

    artifacts = build_artifacts(args.root, args.tag, args.dist_dir)
    print("[plugin-artifacts] built")
    for artifact in artifacts:
        print(f"  - {artifact}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as exc:
        print(f"[plugin-artifacts] {exc}")
        raise SystemExit(2)
