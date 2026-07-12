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

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from qiongli.source_layout import RepoLayout
from qiongli.distribution_metadata import PluginDefinition, load_plugin_distribution
from qiongli.platform_targets import load_platform_targets, remove_path_pattern
from qiongli.workflow_wrapper_skills import write_codex_workflow_wrapper_skills
from tooling.scripts.release_version import parse_release_version
from tooling.scripts.build_lite_mcp import (
    build_current_platform,
    read_target_identity,
    write_target_identity,
)

try:
    from qiongli.subject_materializer import MaterializeOptions, materialize_subject_package, validate_subject_catalog
except ModuleNotFoundError as exc:
    if exc.name != "yaml":
        raise
    MaterializeOptions = None
    materialize_subject_package = None
    validate_subject_catalog = None


PLUGIN_NAME = "qiongli"
NEXT_PLUGIN_NAME = "qiongli-next"
SKILL_DIR_NAME = "qiongli-workflow"
DEFAULT_SKILL_NAME = "qiongli"
NEXT_SKILL_NAME = "qiongli-next"
DEFAULT_MCP_SERVER_NAME = "qiongli"
NEXT_MCP_SERVER_NAME = "qiongli-next"
DEFAULT_CATEGORY = "Education"
NEXT_PLUGIN_DESCRIPTION = (
    "Qiongli Next prerelease academic research workflow plugin for testing the upcoming core workflow "
    "with bundled literature MCP tools."
)
LITE_MCP_BIN_NAME = "qiongli-literature-provider"
_LITE_MCP_BINARY_CACHE: dict[str, Path] = {}
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
AGENT_PACKET_TEMPLATES = (
    "agent-handoff.md",
    "agent-review-packet.md",
    "agent-run-packet.json",
)


def _normalize_tag(raw: str) -> tuple[str, str]:
    identity = parse_release_version(raw)
    return identity.repo_tag, identity.version


def _is_prerelease_tag(repo_tag: str) -> bool:
    return parse_release_version(repo_tag).is_prerelease


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


def _skill_version(root: Path) -> str:
    return (RepoLayout(root).workflow / "VERSION").read_text(encoding="utf-8").strip().lstrip("v")


def _plugin_definition(root: Path, plugin_name: str) -> PluginDefinition:
    return load_plugin_distribution(root).plugins[plugin_name]


def _keywords(plugin: PluginDefinition, platform_keyword: str) -> list[str]:
    return [*plugin.keywords, *[item for item in (platform_keyword,) if item not in plugin.keywords]]


def _write_codex_manifest(path: Path, plugin: PluginDefinition, version: str) -> None:
    manifest = {
        "name": plugin.id,
        "version": version,
        "description": plugin.description,
        "author": plugin.author,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "codex-skills"),
        "skills": "./skills/",
        "mcpServers": "./.mcp.json",
        "interface": {
            "displayName": plugin.display_name,
            "shortDescription": plugin.codex_short_description,
            "longDescription": plugin.description,
            "developerName": plugin.author["name"],
            "category": plugin.category,
            "capabilities": ["Write"],
            "websiteURL": plugin.repository,
            "defaultPrompt": list(plugin.default_prompts),
            "brandColor": plugin.brand_color,
        },
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _write_claude_manifest(path: Path, plugin: PluginDefinition, version: str) -> None:
    manifest = {
        "name": plugin.id,
        "description": plugin.description,
        "version": version,
        "author": plugin.author,
        "category": plugin.category,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "claude-code-plugins"),
        "skills": "./skills/",
        "commands": "./commands/",
        "mcpServers": {
            plugin.mcp_server_name: {
                "command": "${CLAUDE_PLUGIN_ROOT}/bin/qiongli-literature-provider",
                "args": ["--transport", "stdio"],
                "cwd": "${CLAUDE_PLUGIN_ROOT}",
            }
        },
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _write_platform_manifest(root: Path, platform: str, plugin_name: str, manifest_path: Path) -> None:
    plugin = _plugin_definition(root, plugin_name)
    version = _skill_version(root)
    if platform == "codex":
        _write_codex_manifest(manifest_path, plugin, version)
        return
    if platform == "claude":
        _write_claude_manifest(manifest_path, plugin, version)
        return
    raise ValueError(f"unsupported plugin manifest platform: {platform}")


def _write_root_plugin_manifest(plugin_root: Path, plugin_name: str) -> None:
    plugin_root.mkdir(parents=True, exist_ok=True)
    (plugin_root / "plugin.json").write_text(
        json.dumps({"name": plugin_name}, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def _write_codex_mcp_manifest(root: Path, dest_plugin_root: Path, *, server_name: str) -> None:
    manifest = {
        "mcpServers": {
            server_name: {
                "command": "./bin/qiongli-literature-provider",
                "args": ["--transport", "stdio"],
                "cwd": ".",
                "startup_timeout_sec": 20,
                "tool_timeout_sec": 60,
            }
        }
    }
    dest = dest_plugin_root / ".mcp.json"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def _workflow_description(workflow_path: Path) -> str:
    text = workflow_path.read_text(encoding="utf-8")
    match = re.search(r"(?ms)^---\n(.*?)\n---", text)
    if not match:
        return f"Run the {workflow_path.stem} research workflow."
    desc = re.search(r"(?m)^description:\s*(.+)$", match.group(1))
    return desc.group(1).strip() if desc else f"Run the {workflow_path.stem} research workflow."


def _generate_commands(root: Path, commands_root: Path, skill_name: str) -> None:
    workflow_root = RepoLayout(root).workflow / "workflows"
    commands_root.mkdir(parents=True, exist_ok=True)
    for workflow_path in sorted(workflow_root.glob("*.md")):
        text = "\n".join(
            [
                "---",
                f"description: {_workflow_description(workflow_path)}",
                "---",
                "",
                (
                    f"Load the `{skill_name}` skill from this plugin, then follow "
                    f"`skills/{SKILL_DIR_NAME}/workflows/{workflow_path.name}`."
                ),
                "",
                "Use that workflow as the source of truth for task order, artifacts, and quality gates.",
                "",
            ]
        )
        (commands_root / workflow_path.name).write_text(text, encoding="utf-8")


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


def _rewrite_skill_entrypoint(skill_root: Path, skill_name: str) -> None:
    if skill_name == DEFAULT_SKILL_NAME:
        return
    skill_path = skill_root / "SKILL.md"
    text = skill_path.read_text(encoding="utf-8")
    text = re.sub(r"(?m)^name:\s*qiongli\s*$", f"name: {skill_name}", text)
    if skill_name == NEXT_SKILL_NAME:
        text = text.replace("Qiongli version:", "Qiongli Next version:", 1)
    text = text.replace("$qiongli", f"${skill_name}")
    if f"${skill_name}" not in text:
        text = (
            text.rstrip()
            + "\n\n## Prerelease Invocation\n\n"
            + f"Invoke this beta package as `${skill_name}` when testing the next Qiongli core workflow.\n"
        )
    skill_path.write_text(text, encoding="utf-8")


def _rewrite_command_invocation(commands_root: Path, skill_name: str) -> None:
    if skill_name == DEFAULT_SKILL_NAME or not commands_root.is_dir():
        return
    for command_path in sorted(commands_root.glob("*.md")):
        text = command_path.read_text(encoding="utf-8")
        text = text.replace("Load the `qiongli` skill", f"Load the `{skill_name}` skill")
        command_path.write_text(text, encoding="utf-8")


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
    generated_skill = RepoLayout(root).plugin_artifact_package / "skills"
    if generated_skill.is_dir():
        _copy_path(generated_skill, dest_plugin_root / "skills")
        return

    _copy_subject_skill(root, dest_plugin_root, "core")


def _copy_subject_skill(root: Path, dest_plugin_root: Path, subject: str, *, skill_name: str = DEFAULT_SKILL_NAME) -> None:
    if materialize_subject_package is None or MaterializeOptions is None:
        raise ValueError("PyYAML is required to build subject-specific marketplace plugin artifacts")

    with tempfile.TemporaryDirectory(prefix=f"qiongli-marketplace-{subject}-source-") as tmp:
        materialize_root = _build_materialize_source(root, Path(tmp))
        skill_dest = dest_plugin_root / "skills" / SKILL_DIR_NAME
        materialize_subject_package(
            MaterializeOptions(
                source=materialize_root,
                out=skill_dest,
                subject=subject,
                flavor="full",
                coverage="complete",
            )
        )
        _rewrite_skill_entrypoint(skill_dest, skill_name)


def _copy_commands(root: Path, dest_plugin_root: Path, *, skill_name: str = DEFAULT_SKILL_NAME) -> None:
    _generate_commands(root, dest_plugin_root / "commands", skill_name)


def _copy_codex_workflow_wrapper_skills(
    root: Path,
    dest_plugin_root: Path,
    *,
    skill_name: str = DEFAULT_SKILL_NAME,
) -> None:
    write_codex_workflow_wrapper_skills(
        RepoLayout(root).workflow / "workflows",
        dest_plugin_root / "skills",
        skill_name=skill_name,
        canonical_skill_dir=SKILL_DIR_NAME,
    )


def _mcp_server_name_for_plugin(plugin_name: str) -> str:
    return NEXT_MCP_SERVER_NAME if plugin_name == NEXT_PLUGIN_NAME else DEFAULT_MCP_SERVER_NAME


def _rewrite_mcp_server_name(container: dict[str, object], server_name: str) -> None:
    mcp_servers = container.get("mcpServers")
    if not isinstance(mcp_servers, dict):
        return
    if server_name == DEFAULT_MCP_SERVER_NAME:
        return
    if server_name not in mcp_servers:
        server = mcp_servers.pop(DEFAULT_MCP_SERVER_NAME, None)
        if server is None:
            raise ValueError(f"bundled MCP manifest missing {DEFAULT_MCP_SERVER_NAME} server")
        mcp_servers[server_name] = server
    else:
        mcp_servers.pop(DEFAULT_MCP_SERVER_NAME, None)


def _copy_codex_mcp_manifest(
    root: Path,
    dest_plugin_root: Path,
    *,
    server_name: str = DEFAULT_MCP_SERVER_NAME,
) -> None:
    _write_codex_mcp_manifest(root, dest_plugin_root, server_name=server_name)


def _copy_literature_mcp_runtime(root: Path, dest_plugin_root: Path) -> None:
    mcp_runtime = RepoLayout(root).literature_mcpb_package / "server"
    if mcp_runtime.is_dir():
        _copy_path(mcp_runtime, dest_plugin_root / "mcp" / "qiongli-literature-provider")


def _cached_lite_mcp_binary(root: Path) -> Path:
    root = root.resolve()
    cache_key = str(root)
    cached = _LITE_MCP_BINARY_CACHE.get(cache_key)
    if cached is not None and cached.is_file():
        return cached
    cache_dir = Path(tempfile.mkdtemp(prefix="qiongli-lite-mcp-binary-"))
    binary = build_current_platform(root, cache_dir)
    _LITE_MCP_BINARY_CACHE[cache_key] = binary
    return binary


def _copy_lite_mcp_runtime(root: Path, dest_plugin_root: Path) -> None:
    binary = _cached_lite_mcp_binary(root)
    dest = dest_plugin_root / "bin" / LITE_MCP_BIN_NAME
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(binary, dest)
    identity = read_target_identity(binary)
    target = identity.get("target_triple")
    if not isinstance(target, str) or not target:
        raise ValueError(f"Lite MCP target identity missing target_triple: {binary}")
    version = identity.get("component_version")
    if not isinstance(version, str) or not version:
        raise ValueError(f"Lite MCP target identity missing component_version: {binary}")
    write_target_identity(dest, target, version)


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


def _platform_target_by_recommended_key(root: Path, recommended_key: str):
    matches = sorted(
        (
            target
            for target in load_platform_targets(root).values()
            if target.release_download.get("recommended_key") == recommended_key
        ),
        key=lambda target: target.target_id,
    )
    if len(matches) != 1:
        raise ValueError(
            "platform target registry must define exactly one "
            f"release_download.recommended_key={recommended_key!r}; found {len(matches)}"
        )
    return matches[0]


def _apply_recommended_platform_forbidden_paths(root: Path, plugin_root: Path, recommended_key: str) -> None:
    target = _platform_target_by_recommended_key(root, recommended_key)
    for pattern in target.forbidden_paths:
        remove_path_pattern(plugin_root, pattern)


def _copy_claude_desktop_skill(
    root: Path,
    skill_dest: Path,
    subject: str,
    *,
    skill_name: str = DEFAULT_SKILL_NAME,
) -> None:
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
            _copy_claude_desktop_skill_without_pyyaml(materialize_root, skill_dest, subject, skill_name=skill_name)
    _rewrite_skill_entrypoint(skill_dest, skill_name)
    file_count = sum(1 for item in skill_dest.rglob("*") if item.is_file())
    if file_count > DESKTOP_SKILL_FILE_BUDGET:
        raise ValueError(
            f"Claude Desktop {subject} skill package has {file_count} files; "
            f"limit is {DESKTOP_SKILL_FILE_BUDGET}"
        )


def _copy_claude_desktop_skill_without_pyyaml(
    root: Path,
    skill_dest: Path,
    subject: str,
    *,
    skill_name: str = DEFAULT_SKILL_NAME,
) -> None:
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
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Core",
            "General-purpose Qiongli academic workflow.",
            skill_name=skill_name,
        )
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
            skill_name=skill_name,
        )
    elif subject == "economics-accounting":
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Economics + Accounting",
            "Cross-disciplinary economics and accounting workflow for archival, causal, and reporting-setting research.",
            skill_name=skill_name,
        )
    elif subject == "finance":
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Finance",
            "Finance-focused corporate finance, asset pricing, market microstructure, and risk workflow for doctoral-level journal manuscripts.",
            skill_name=skill_name,
        )
    elif subject == "accounting":
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Accounting",
            "Accounting-focused archival, disclosure, audit, and measurement workflow.",
            skill_name=skill_name,
        )
    else:
        _write_fallback_skill_md(
            skill_dest,
            "Qiongli Economics",
            "Economics-focused empirical, theory, and reproducibility workflow.",
            skill_name=skill_name,
        )
    if subject == "accounting":
        template_refs = ACCOUNTING_TEMPLATES
    elif subject == "business":
        template_refs = BUSINESS_TEMPLATES
    elif subject == "finance":
        template_refs = FINANCE_TEMPLATES
    else:
        template_refs = ECONOMICS_TEMPLATES
    for rel in (*template_refs, *AGENT_PACKET_TEMPLATES):
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


def _write_fallback_skill_md(
    skill_dest: Path,
    display_name: str,
    description: str,
    *,
    skill_name: str = DEFAULT_SKILL_NAME,
) -> None:
    text = "\n".join(
        [
            "---",
            f"name: {skill_name}",
            f"description: {description}",
            "---",
            "",
            f"# {display_name}",
            "",
            description,
            "",
            f"## {display_name.removeprefix('Qiongli ')} Workflow Map",
            "",
            f"Use `${skill_name}`, `workflows/`, `references/`, `templates/`, and `skills/registry.yaml` as the active subject contract.",
            "",
            "## Literature Provider Configuration",
            "",
            "- CLI, Codex, Claude Code, Antigravity, and Hermes installs can configure external literature providers with `qiongli provider setup` and audit them with `qiongli provider doctor`.",
            "- In bundled MCP installs, do not expect the client MCP settings UI to inject provider keys into the plugin-bundled MCP server. Use `qiongli_config_status` to find the shared provider config path, then use `qiongli_configure_provider` to open a local browser setup page. Use `qiongli_save_provider_config` only for explicit scripted writes.",
            "- Keep provider secrets out of `.mcp.json`, plugin manifests, release ZIPs, and research artifacts. The bundled provider server reads the shared provider config or explicit provider environment variables at runtime.",
            "- Do not use `qiongli_collect_evidence` to judge built-in literature provider configuration. That tool is a filesystem/builtin/external-command evidence adapter; direct provider names such as `openalex` require a separate `RESEARCH_MCP_OPENALEX_CMD`. Use `qiongli_literature_status`, `qiongli_config_status`, `qiongli_test_provider`, and `qiongli_literature_search` to judge OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv provider availability.",
            "- Treat `provider_connected` as the only mode where configured external academic provider credentials are available to the local runtime.",
            "- Treat `strategy_only` as a constrained mode: draft the search strategy or use user-supplied corpus, record the limitation, and do not claim review-grade external provider or native-search coverage.",
            "- Claude Desktop/Web focused ZIPs are skill-only packages kept within the 180-file upload budget. They contain workflows/prompts/templates, store no secrets, and cannot execute OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv API calls by themselves.",
            "- For a manual Desktop install, upload the `qiongli-claude-desktop-skill-*.zip` first, then add a manual MCP install when provider calls or local orchestration are required. The skill ZIP supplies agent instructions, workflows/prompts/templates, and subject overlays; MCP supplies tool calls.",
            "- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or another configured provider MCP before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv configuration/search. Its primary package uses the Rust Lite MCP executable, not a user-installed Node or Python runtime. arXiv is enabled without credentials. Platform-native search alone is `native_only`, not `provider_connected`; if no provider MCP/MCPB and no platform-native search is available, record the run as `strategy_only`.",
            "- The literature MCPB provides literature MCP tools only. It does not launch orchestrator agents. To expose the full agent runtime through MCP, manually install the full CLI MCP server with `qiongli mcp serve --transport stdio`; clients can then call `qiongli_orchestrator_route`, `qiongli_task_plan`, and `qiongli_task_run` after the local CLI runtime and model CLIs are configured. `qiongli_task_run` remains preview-first unless the caller sends JSON boolean `run_agents: true`.",
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
        return f"Qiongli academic research workflow plugin. {package_goal}"
    modifier = _subject_modifier(display_name)
    return (
        f"Qiongli {modifier} academic research workflow plugin. "
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
    skill_name: str = DEFAULT_SKILL_NAME,
    mcp_server_name: str = DEFAULT_MCP_SERVER_NAME,
) -> None:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["name"] = plugin_name
    _rewrite_mcp_server_name(manifest, mcp_server_name)
    if plugin_name == NEXT_PLUGIN_NAME:
        manifest["description"] = NEXT_PLUGIN_DESCRIPTION
    else:
        manifest["description"] = _subject_description(display_name, package_goal, subject)
    if platform == "codex":
        manifest.pop("category", None)
    else:
        manifest["category"] = DEFAULT_CATEGORY
    keywords = manifest.get("keywords")
    if isinstance(keywords, list):
        additions = ["qiongli-next", "prerelease"] if plugin_name == NEXT_PLUGIN_NAME else ["qiongli-subject", subject]
        manifest["keywords"] = [*keywords, *[item for item in additions if item not in keywords]]

    if platform == "codex":
        interface = manifest.get("interface")
        if not isinstance(interface, dict):
            interface = {}
            manifest["interface"] = interface
        interface["displayName"] = display_name
        interface["category"] = DEFAULT_CATEGORY
        if plugin_name == NEXT_PLUGIN_NAME:
            interface["shortDescription"] = "Prerelease academic research workflows for Codex."
            interface["defaultPrompt"] = [
                f"Use ${skill_name} to test the next Qiongli paper workflow.",
                f"Use ${skill_name} to test a literature review workflow.",
                f"Use ${skill_name} to test submission checks with the bundled literature MCP.",
            ]
            interface["longDescription"] = manifest["description"]
        elif subject == "core":
            interface["shortDescription"] = "General academic paper workflows for Codex."
            interface["defaultPrompt"] = [
                "Use $qiongli to plan my paper.",
                "Use $qiongli to run a literature review for my research topic.",
                "Use $qiongli to prepare a submission package for my manuscript.",
            ]
            interface["longDescription"] = _subject_description(display_name, package_goal, subject)
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
    skill_name: str = DEFAULT_SKILL_NAME,
) -> list[Path]:
    bundle_name = f"{artifact_name}-{platform}-plugin-{tag}"
    bundle = work_dir / bundle_name
    plugin_dest = bundle / "plugins" / plugin_name
    manifest_dir = ".codex-plugin" if platform == "codex" else ".claude-plugin"
    base_plugin_name = NEXT_PLUGIN_NAME if plugin_name == NEXT_PLUGIN_NAME else PLUGIN_NAME
    _write_platform_manifest(
        root,
        platform,
        base_plugin_name,
        plugin_dest / manifest_dir / "plugin.json",
    )
    _write_subject_manifest(
        plugin_dest / manifest_dir / "plugin.json",
        platform=platform,
        plugin_name=plugin_name,
        subject=subject,
        display_name=display_name,
        package_goal=package_goal,
        skill_name=skill_name,
        mcp_server_name=_mcp_server_name_for_plugin(plugin_name),
    )
    _write_root_plugin_manifest(plugin_dest, plugin_name)
    if platform == "codex":
        _copy_codex_mcp_manifest(
            root,
            plugin_dest,
            server_name=_mcp_server_name_for_plugin(plugin_name),
        )
    _copy_lite_mcp_runtime(root, plugin_dest)
    _copy_commands(root, plugin_dest, skill_name=skill_name)
    _copy_subject_skill(root, plugin_dest, subject, skill_name=skill_name)
    if platform == "codex":
        _copy_codex_workflow_wrapper_skills(root, plugin_dest, skill_name=skill_name)
    artifacts = [dist_dir / f"{bundle_name}.tar.gz"]
    _make_tarball(bundle, artifacts[0])
    if platform == "claude":
        zip_artifact = dist_dir / f"{bundle_name}.zip"
        _make_zip(bundle, zip_artifact)
        artifacts.append(zip_artifact)
    return artifacts


def materialize_next_codex_plugin(root: Path, dest_plugin_root: Path, *, force: bool = False) -> Path:
    """Materialize the generated qiongli-next plugin payload."""

    root = root.resolve()
    dest_plugin_root = dest_plugin_root.resolve()
    if dest_plugin_root.exists():
        if not force:
            raise ValueError(f"{dest_plugin_root} already exists; pass force=True to replace it")
        if dest_plugin_root.is_dir():
            shutil.rmtree(dest_plugin_root)
        else:
            dest_plugin_root.unlink()

    _display_name, package_goal = _subject_definitions(root)["core"]
    manifest_dir = ".codex-plugin"
    _write_platform_manifest(
        root,
        "codex",
        NEXT_PLUGIN_NAME,
        dest_plugin_root / manifest_dir / "plugin.json",
    )
    _write_subject_manifest(
        dest_plugin_root / manifest_dir / "plugin.json",
        platform="codex",
        plugin_name=NEXT_PLUGIN_NAME,
        subject="core",
        display_name="Qiongli Next",
        package_goal=package_goal,
        skill_name=NEXT_SKILL_NAME,
        mcp_server_name=NEXT_MCP_SERVER_NAME,
    )
    _write_root_plugin_manifest(dest_plugin_root, NEXT_PLUGIN_NAME)
    _copy_codex_mcp_manifest(root, dest_plugin_root, server_name=NEXT_MCP_SERVER_NAME)
    _copy_lite_mcp_runtime(root, dest_plugin_root)
    _copy_commands(root, dest_plugin_root, skill_name=NEXT_SKILL_NAME)
    _copy_subject_skill(root, dest_plugin_root, "core", skill_name=NEXT_SKILL_NAME)
    _copy_codex_workflow_wrapper_skills(root, dest_plugin_root, skill_name=NEXT_SKILL_NAME)
    return dest_plugin_root


def materialize_next_plugin_package(root: Path, dest_plugin_root: Path, *, force: bool = False) -> Path:
    """Materialize the generated qiongli-next plugin payload for direct plugin ZIP installs."""

    root = root.resolve()
    dest_plugin_root = dest_plugin_root.resolve()
    if dest_plugin_root.exists():
        if not force:
            raise ValueError(f"{dest_plugin_root} already exists; pass force=True to replace it")
        if dest_plugin_root.is_dir():
            shutil.rmtree(dest_plugin_root)
        else:
            dest_plugin_root.unlink()

    _display_name, package_goal = _subject_definitions(root)["core"]
    for platform, manifest_dir in (("codex", ".codex-plugin"), ("claude", ".claude-plugin")):
        _write_platform_manifest(
            root,
            platform,
            NEXT_PLUGIN_NAME,
            dest_plugin_root / manifest_dir / "plugin.json",
        )
        _write_subject_manifest(
            dest_plugin_root / manifest_dir / "plugin.json",
            platform=platform,
            plugin_name=NEXT_PLUGIN_NAME,
            subject="core",
            display_name="Qiongli Next",
            package_goal=package_goal,
            skill_name=NEXT_SKILL_NAME,
            mcp_server_name=NEXT_MCP_SERVER_NAME,
        )

    _write_root_plugin_manifest(dest_plugin_root, NEXT_PLUGIN_NAME)
    _copy_codex_mcp_manifest(root, dest_plugin_root, server_name=NEXT_MCP_SERVER_NAME)
    _copy_lite_mcp_runtime(root, dest_plugin_root)
    _copy_commands(root, dest_plugin_root, skill_name=NEXT_SKILL_NAME)
    _copy_subject_skill(root, dest_plugin_root, "core", skill_name=NEXT_SKILL_NAME)
    _copy_codex_workflow_wrapper_skills(root, dest_plugin_root, skill_name=NEXT_SKILL_NAME)
    return dest_plugin_root


def materialize_plugin_package(root: Path, dest_plugin_root: Path, *, force: bool = False) -> Path:
    """Materialize the stable Qiongli plugin package from canonical sources."""

    root = root.resolve()
    dest_plugin_root = dest_plugin_root.resolve()
    if dest_plugin_root.exists():
        if not force:
            raise ValueError(f"{dest_plugin_root} already exists; pass force=True to replace it")
        if dest_plugin_root.is_dir():
            shutil.rmtree(dest_plugin_root)
        else:
            dest_plugin_root.unlink()

    plugin = _plugin_definition(root, PLUGIN_NAME)
    _write_platform_manifest(
        root,
        "codex",
        PLUGIN_NAME,
        dest_plugin_root / ".codex-plugin" / "plugin.json",
    )
    if plugin.claude_enabled:
        _write_platform_manifest(
            root,
            "claude",
            PLUGIN_NAME,
            dest_plugin_root / ".claude-plugin" / "plugin.json",
        )
    _write_root_plugin_manifest(dest_plugin_root, PLUGIN_NAME)
    _copy_codex_mcp_manifest(root, dest_plugin_root, server_name=DEFAULT_MCP_SERVER_NAME)
    _copy_lite_mcp_runtime(root, dest_plugin_root)
    _copy_commands(root, dest_plugin_root, skill_name=DEFAULT_SKILL_NAME)
    _copy_subject_skill(root, dest_plugin_root, "core", skill_name=DEFAULT_SKILL_NAME)
    _copy_codex_workflow_wrapper_skills(root, dest_plugin_root, skill_name=DEFAULT_SKILL_NAME)
    return dest_plugin_root


def _remove_path(path: Path) -> None:
    if path.is_dir():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()


def _strip_codex_only_plugin_content(dest_plugin_root: Path, *, skill_name: str) -> None:
    _remove_path(dest_plugin_root / ".codex-plugin")
    _remove_path(dest_plugin_root / ".mcp.json")

    skills_dir = dest_plugin_root / "skills"
    if not skills_dir.is_dir():
        return
    for child in skills_dir.iterdir():
        if child.name != SKILL_DIR_NAME and child.name.startswith(f"{skill_name}-"):
            _remove_path(child)


def materialize_next_claude_plugin_package(root: Path, dest_plugin_root: Path, *, force: bool = False) -> Path:
    """Materialize the generated qiongli-next plugin payload for Claude Desktop direct installs."""

    materialize_next_plugin_package(root, dest_plugin_root, force=force)
    _strip_codex_only_plugin_content(dest_plugin_root, skill_name=NEXT_SKILL_NAME)
    return dest_plugin_root


def materialize_claude_plugin_package(root: Path, dest_plugin_root: Path, *, force: bool = False) -> Path:
    """Materialize the stable Qiongli plugin payload for Claude Desktop direct installs."""

    materialize_plugin_package(root, dest_plugin_root, force=force)
    _strip_codex_only_plugin_content(dest_plugin_root, skill_name=DEFAULT_SKILL_NAME)
    return dest_plugin_root


def materialize_agent_platform(root: Path, dest_platform_root: Path, *, force: bool = False) -> Path:
    """Materialize agent workflow entrypoints from canonical workflows."""

    root = root.resolve()
    dest_platform_root = dest_platform_root.resolve()
    if dest_platform_root.exists():
        if not force:
            raise ValueError(f"{dest_platform_root} already exists; pass force=True to replace it")
        if dest_platform_root.is_dir():
            shutil.rmtree(dest_platform_root)
        else:
            dest_platform_root.unlink()
    _copy_path(RepoLayout(root).workflow / "workflows", dest_platform_root / "workflows")
    return dest_platform_root


def _build_codex(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> list[Path]:
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


def _build_claude(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> list[Path]:
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
            artifacts.extend(
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


def _build_next_marketplace_plugins(root: Path, tag: str, dist_dir: Path, work_dir: Path) -> list[Path]:
    _display_name, package_goal = _subject_definitions(root)["core"]
    artifacts: list[Path] = []
    for platform in ("codex", "claude"):
        artifacts.extend(
            _build_marketplace_plugin(
                root,
                tag,
                dist_dir,
                work_dir,
                platform=platform,
                subject="core",
                plugin_name=NEXT_PLUGIN_NAME,
                artifact_name=NEXT_PLUGIN_NAME,
                display_name="Qiongli Next",
                package_goal=package_goal,
                skill_name=NEXT_SKILL_NAME,
            )
        )
    return artifacts


def _build_claude_desktop_plugin(
    root: Path,
    tag: str,
    dist_dir: Path,
    work_dir: Path,
    *,
    plugin_name: str = PLUGIN_NAME,
    artifact_prefix: str = PLUGIN_NAME,
    next_channel: bool = False,
) -> Path:
    plugin_dest = work_dir / f"desktop-plugin-{artifact_prefix}" / plugin_name
    if next_channel:
        materialize_next_claude_plugin_package(root, plugin_dest, force=True)
    else:
        materialize_claude_plugin_package(root, plugin_dest, force=True)
    _apply_recommended_platform_forbidden_paths(root, plugin_dest, "claude_desktop_plugin")

    artifact = dist_dir / f"{artifact_prefix}-claude-desktop-plugin-{tag}.zip"
    _make_zip(plugin_dest, artifact)
    return artifact


def _build_claude_desktop_skill(
    root: Path,
    tag: str,
    dist_dir: Path,
    work_dir: Path,
    subject: str,
    *,
    artifact_prefix: str = PLUGIN_NAME,
    skill_name: str = DEFAULT_SKILL_NAME,
) -> Path:
    bundle_name = f"{artifact_prefix}-claude-desktop-skill-{subject}-{tag}"
    skill_dest = work_dir / f"desktop-{artifact_prefix}-{subject}" / skill_name
    _copy_claude_desktop_skill(root, skill_dest, subject, skill_name=skill_name)
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

    release_identity = parse_release_version(repo_tag)
    plugin_name = NEXT_PLUGIN_NAME if release_identity.is_prerelease else PLUGIN_NAME
    plugin = _plugin_definition(root, plugin_name)
    if release_identity.release_line not in plugin.release_lines:
        raise ValueError(
            f"{plugin_name} does not support release line {release_identity.release_line}"
        )
    if release_identity.channel not in plugin.release_channels:
        raise ValueError(
            f"{plugin_name} does not support release channel {release_identity.channel}"
        )

    with tempfile.TemporaryDirectory(prefix="qiongli-plugin-") as tmp:
        work_dir = Path(tmp)
        if _is_prerelease_tag(repo_tag):
            return [
                *_build_next_marketplace_plugins(root, repo_tag, dist_dir, work_dir),
                _build_claude_desktop_plugin(
                    root,
                    repo_tag,
                    dist_dir,
                    work_dir,
                    plugin_name=NEXT_PLUGIN_NAME,
                    artifact_prefix=NEXT_PLUGIN_NAME,
                    next_channel=True,
                ),
                _build_claude_desktop_skill(
                    root,
                    repo_tag,
                    dist_dir,
                    work_dir,
                    "core",
                    artifact_prefix=NEXT_PLUGIN_NAME,
                    skill_name=NEXT_SKILL_NAME,
                ),
            ]

        desktop_artifacts = [
            _build_claude_desktop_skill(root, repo_tag, dist_dir, work_dir, subject)
            for subject in _desktop_subjects(root)
        ]
        legacy_desktop_artifact = dist_dir / f"{PLUGIN_NAME}-claude-desktop-skill-{repo_tag}.zip"
        shutil.copy2(dist_dir / f"{PLUGIN_NAME}-claude-desktop-skill-core-{repo_tag}.zip", legacy_desktop_artifact)
        subject_marketplace_artifacts = _build_subject_marketplace_plugins(root, repo_tag, dist_dir, work_dir)
        artifacts = [
            *_build_codex(root, repo_tag, dist_dir, work_dir),
            *_build_claude(root, repo_tag, dist_dir, work_dir),
            *subject_marketplace_artifacts,
            _build_claude_desktop_plugin(root, repo_tag, dist_dir, work_dir),
            *desktop_artifacts,
            legacy_desktop_artifact,
        ]
    return artifacts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build Codex and Claude Code plugin artifacts.")
    parser.add_argument("--tag", required=True, help="Release tag, for example v0.5.0-beta.3")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2], help="Repository root")
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
