#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import json
import re
import sys
import tarfile
import tempfile
import warnings
import zipfile
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from build_plugin_artifacts import _desktop_subjects, _is_prerelease_tag, _marketplace_subjects, build_artifacts
from qiongli.source_layout import RepoLayout


PLUGIN_NAME = "qiongli"
NEXT_PLUGIN_NAME = "qiongli-next"
SKILL_DIR_NAME = "qiongli-workflow"
SKILL_NAME = "qiongli"
NEXT_SKILL_NAME = "qiongli-next"
MCP_SERVER_NAME = "qiongli"
NEXT_MCP_SERVER_NAME = "qiongli-next"
CLAUDE_DESKTOP_FILE_BUDGET = 180


@dataclass(frozen=True)
class ArtifactSpec:
    platform: str
    manifest: Path
    plugin_root: Path
    requires_commands: bool
    expects_bundled_mcp: bool = False


ARTIFACT_SPECS = {
    "codex": ArtifactSpec(
        platform="codex",
        manifest=Path("plugins") / PLUGIN_NAME / ".codex-plugin" / "plugin.json",
        plugin_root=Path("plugins") / PLUGIN_NAME,
        requires_commands=True,
        expects_bundled_mcp=True,
    ),
    "claude": ArtifactSpec(
        platform="claude",
        manifest=Path("plugins") / PLUGIN_NAME / ".claude-plugin" / "plugin.json",
        plugin_root=Path("plugins") / PLUGIN_NAME,
        requires_commands=True,
        expects_bundled_mcp=True,
    ),
}


def _read_json(path: Path) -> dict[str, object]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return data


def _mcp_server_name_for_plugin(plugin_name: str) -> str:
    return NEXT_MCP_SERVER_NAME if plugin_name == NEXT_PLUGIN_NAME else MCP_SERVER_NAME


def _extract_single_root(artifact: Path, dest: Path) -> Path:
    with tarfile.open(artifact, "r:gz") as tar:
        tar.extractall(dest, filter="data")
    roots = [item for item in dest.iterdir() if item.is_dir()]
    if len(roots) != 1:
        raise ValueError(f"{artifact} should extract to one top-level directory, found {len(roots)}")
    return roots[0]


def _extract_single_zip_root(artifact: Path, dest: Path) -> Path:
    with zipfile.ZipFile(artifact) as archive:
        archive.extractall(dest)
    roots = [item for item in dest.iterdir() if item.is_dir()]
    if len(roots) != 1:
        raise ValueError(f"{artifact} should extract to one top-level directory, found {len(roots)}")
    return roots[0]


def _extract_marketplace_root(artifact: Path, dest: Path) -> Path:
    if artifact.suffix == ".zip":
        return _extract_single_zip_root(artifact, dest)
    return _extract_single_root(artifact, dest)


def _assert_claude_desktop_zip_budget(artifact: Path, subject: str, *, skill_name: str = SKILL_NAME) -> int:
    with zipfile.ZipFile(artifact) as archive:
        file_names = [name for name in archive.namelist() if not name.endswith("/")]

    if len(file_names) > CLAUDE_DESKTOP_FILE_BUDGET:
        raise ValueError(
            f"{artifact} contains {len(file_names)} files; "
            f"Claude Desktop upload budget is {CLAUDE_DESKTOP_FILE_BUDGET}"
        )

    if subject == "core":
        detailed_skill_specs = [
            name
            for name in file_names
            if name.startswith(f"{skill_name}/skills/")
            and name != f"{skill_name}/skills/registry.yaml"
            and name.endswith(".md")
        ]
        if detailed_skill_specs:
            raise ValueError(
                f"{artifact} includes detailed skill specs that should be omitted from the core Desktop ZIP: "
                + ", ".join(detailed_skill_specs[:5])
            )

    return len(file_names)


def _assert_file(path: Path, label: str) -> None:
    if not path.is_file():
        raise ValueError(f"missing {label}: {path}")


def _assert_dir(path: Path, label: str) -> None:
    if not path.is_dir():
        raise ValueError(f"missing {label}: {path}")


def _assert_skill_invocation(
    skill_root: Path,
    expected_repo_tag: str,
    *,
    skill_name: str = SKILL_NAME,
) -> list[str]:
    _assert_file(skill_root / "SKILL.md", "skill entrypoint")
    _assert_file(skill_root / "VERSION", "skill version")
    _assert_file(skill_root / "skills" / "registry.yaml", "skill registry")
    _assert_dir(skill_root / "workflows", "workflow directory")

    skill_text = (skill_root / "SKILL.md").read_text(encoding="utf-8")
    if f"name: {skill_name}" not in skill_text:
        raise ValueError(f"{skill_root / 'SKILL.md'} must declare name: {skill_name}")

    actual_version = (skill_root / "VERSION").read_text(encoding="utf-8").strip()
    if actual_version != expected_repo_tag:
        raise ValueError(f"{skill_root / 'VERSION'} expected {expected_repo_tag}, found {actual_version}")

    workflow_names = sorted(path.name for path in (skill_root / "workflows").glob("*.md"))
    if not workflow_names:
        raise ValueError(f"{skill_root / 'workflows'} must contain invokable workflows")
    return workflow_names


def _assert_subject_marker(skill_root: Path, expected_subject: str) -> None:
    actual_subject = (skill_root / "SUBJECT").read_text(encoding="utf-8").strip()
    if actual_subject != expected_subject:
        raise ValueError(f"{skill_root / 'SUBJECT'} expected {expected_subject}, found {actual_subject}")


def _assert_subject_manifest(
    skill_root: Path,
    expected_subject: str,
    expected_coverage: str,
    expected_layers: list[str] | None = None,
) -> None:
    manifest_path = skill_root / "SUBJECT_MANIFEST.json"
    manifest = _read_json(manifest_path)
    if manifest.get("subject") != expected_subject:
        raise ValueError(f"{manifest_path} expected subject {expected_subject}, found {manifest.get('subject')}")
    if manifest.get("coverage") != expected_coverage:
        raise ValueError(f"{manifest_path} expected coverage {expected_coverage}, found {manifest.get('coverage')}")
    if expected_layers is not None and manifest.get("layers") != expected_layers:
        raise ValueError(f"{manifest_path} expected layers {expected_layers}, found {manifest.get('layers')}")


def _load_registry_ids(skill_root: Path) -> set[str]:
    registry_text = (skill_root / "skills" / "registry.yaml").read_text(encoding="utf-8")
    ids = {
        match.group(1).strip()
        for match in re.finditer(r"(?m)^\s*-?\s*id:\s*[\"']?([^\"'\n#]+)", registry_text)
    }
    if not ids:
        raise ValueError(f"{skill_root / 'skills' / 'registry.yaml'} must contain registry ids")
    return ids


def _has_pyyaml() -> bool:
    return importlib.util.find_spec("yaml") is not None


def _assert_core_desktop_package(skill_root: Path) -> None:
    _assert_subject_marker(skill_root, "core")
    _assert_subject_manifest(skill_root, "core", "focused")
    if (skill_root / "skills" / "A_framing" / "question-refiner.md").exists():
        raise ValueError("core Desktop ZIP must remain slim and omit detailed generic skill specs")


def _assert_focused_desktop_package(skill_root: Path, subject: str) -> None:
    _assert_subject_marker(skill_root, subject)
    _assert_subject_manifest(skill_root, subject, "focused")


def _assert_economics_desktop_package(skill_root: Path) -> None:
    _assert_subject_marker(skill_root, "economics")
    _assert_subject_manifest(skill_root, "economics", "focused")
    registry_ids = _load_registry_ids(skill_root)
    for expected in ("econ-identification-auditor", "stats-engine", "manuscript-architect"):
        if expected not in registry_ids:
            raise ValueError(f"economics registry missing selected skill: {expected}")
    for excluded in ("citation-formatter", "prisma-checker", "beamer-builder", "ai-fingerprint-scanner"):
        if excluded in registry_ids:
            raise ValueError(f"economics registry includes unselected generic skill: {excluded}")

    domain_root = skill_root / "skills" / "domain-profiles"
    domain_profiles = sorted(path.name for path in domain_root.glob("*.yaml"))
    if domain_profiles != ["economics.yaml"]:
        raise ValueError(f"economics package must include only economics domain profile, found {domain_profiles}")

    venue_profiles = sorted(path.stem for path in (skill_root / "venue-profiles").glob("*.yaml"))
    expected_venues = ["aer", "econometrica", "jpe", "qje", "restud"] if _has_pyyaml() else ["aer", "qje", "restud"]
    if venue_profiles != expected_venues:
        raise ValueError(f"economics package venue profiles mismatch: {venue_profiles}")

    manuscript = (skill_root / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
    if "## Economics Overlay" not in manuscript:
        raise ValueError("economics manuscript-architect effective skill missing append overlay")

    stats = (skill_root / "skills" / "I_code" / "stats-engine.md").read_text(encoding="utf-8")
    for expected in ("The estimand, identifying variation", "Naive TWFE under staggered adoption"):
        if expected not in stats:
            raise ValueError("economics stats-engine effective skill missing replace_sections overlay")
    if not (skill_root / "skills" / "C_design" / "econ-identification-auditor.md").is_file():
        raise ValueError("economics package missing subject-specific skill")


def _assert_business_desktop_package(skill_root: Path) -> None:
    _assert_subject_marker(skill_root, "business")
    _assert_subject_manifest(skill_root, "business", "focused")
    registry_ids = _load_registry_ids(skill_root)
    for expected in ("business-journal-positioning-auditor", "stats-engine", "manuscript-architect"):
        if expected not in registry_ids:
            raise ValueError(f"business registry missing selected skill: {expected}")

    domain_profiles = sorted(path.name for path in (skill_root / "skills" / "domain-profiles").glob("*.yaml"))
    if domain_profiles != ["business-management.yaml"]:
        raise ValueError(f"business package domain profiles mismatch: {domain_profiles}")

    venue_profiles = sorted(path.stem for path in (skill_root / "venue-profiles").glob("*.yaml"))
    expected_venues = [
        "academy-of-management-journal",
        "journal-of-management",
        "journal-of-marketing",
        "organization-science",
        "strategic-management-journal",
    ]
    fallback_venues = [
        "academy-of-management-journal",
        "organization-science",
        "strategic-management-journal",
    ]
    if venue_profiles not in (expected_venues, fallback_venues):
        raise ValueError(f"business package venue profiles mismatch: {venue_profiles}")

    manuscript = (skill_root / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
    if "## Business Overlay" not in manuscript:
        raise ValueError("business manuscript-architect missing append overlay")
    if not (skill_root / "skills" / "C_design" / "business-journal-positioning-auditor.md").is_file():
        raise ValueError("business package missing subject-specific skill")


def _assert_economics_accounting_desktop_package(skill_root: Path) -> None:
    _assert_subject_marker(skill_root, "economics-accounting")
    _assert_subject_manifest(
        skill_root,
        "economics-accounting",
        "focused",
        ["core", "economics", "accounting", "economics-accounting"],
    )
    registry_ids = _load_registry_ids(skill_root)
    for expected in ("econ-identification-auditor", "accounting-measurement-auditor", "stats-engine"):
        if expected not in registry_ids:
            raise ValueError(f"economics-accounting registry missing selected skill: {expected}")
    for excluded in ("biomedical", "beamer-builder", "ai-fingerprint-scanner"):
        if excluded in registry_ids:
            raise ValueError(f"economics-accounting registry includes unselected skill: {excluded}")

    domain_profiles = sorted(path.name for path in (skill_root / "skills" / "domain-profiles").glob("*.yaml"))
    if domain_profiles != ["accounting.yaml", "economics.yaml"]:
        raise ValueError(f"economics-accounting package domain profiles mismatch: {domain_profiles}")

    venue_profiles = sorted(path.stem for path in (skill_root / "venue-profiles").glob("*.yaml"))
    expected_venues = [
        "accounting-review",
        "aer",
        "journal-of-accounting-research",
        "qje",
        "restud",
        "review-of-accounting-studies",
    ]
    if venue_profiles != expected_venues:
        raise ValueError(f"economics-accounting package venue profiles mismatch: {venue_profiles}")

    manuscript = (skill_root / "skills" / "F_writing" / "manuscript-architect.md").read_text(encoding="utf-8")
    if "## Economics and Accounting Overlay" not in manuscript:
        raise ValueError("economics-accounting manuscript-architect missing composite overlay")

    stats = (skill_root / "skills" / "I_code" / "stats-engine.md").read_text(encoding="utf-8")
    if "archival accounting" not in stats:
        raise ValueError("economics-accounting stats-engine missing composite overlay")

    if not (skill_root / "skills" / "C_design" / "accounting-measurement-auditor.md").is_file():
        raise ValueError("economics-accounting package missing accounting subject-specific skill")


def _assert_finance_desktop_package(skill_root: Path) -> None:
    _assert_subject_marker(skill_root, "finance")
    _assert_subject_manifest(skill_root, "finance", "focused")
    registry_ids = _load_registry_ids(skill_root)
    for expected in ("finance-identification-risk-auditor", "stats-engine", "manuscript-architect"):
        if expected not in registry_ids:
            raise ValueError(f"finance registry missing selected skill: {expected}")

    domain_profiles = sorted(path.name for path in (skill_root / "skills" / "domain-profiles").glob("*.yaml"))
    if domain_profiles != ["finance.yaml"]:
        raise ValueError(f"finance package domain profiles mismatch: {domain_profiles}")

    venue_profiles = sorted(path.stem for path in (skill_root / "venue-profiles").glob("*.yaml"))
    expected_venues = [
        "financial-management",
        "journal-of-corporate-finance",
        "journal-of-finance",
        "journal-of-financial-economics",
        "review-of-financial-studies",
    ]
    fallback_venues = [
        "journal-of-finance",
        "journal-of-financial-economics",
        "review-of-financial-studies",
    ]
    if venue_profiles not in (expected_venues, fallback_venues):
        raise ValueError(f"finance package venue profiles mismatch: {venue_profiles}")

    stats = (skill_root / "skills" / "I_code" / "stats-engine.md").read_text(encoding="utf-8")
    for expected in ("asset pricing", "look-ahead bias"):
        if expected not in stats:
            raise ValueError("finance stats-engine effective skill missing replace_sections overlay")
    if not (skill_root / "skills" / "C_design" / "finance-identification-risk-auditor.md").is_file():
        raise ValueError("finance package missing subject-specific skill")


def _assert_command_invocation(
    plugin_root: Path,
    workflow_names: list[str],
    *,
    skill_name: str = SKILL_NAME,
) -> None:
    commands_dir = plugin_root / "commands"
    _assert_dir(commands_dir, "slash command directory")

    command_names = sorted(path.name for path in commands_dir.glob("*.md"))
    if command_names != workflow_names:
        missing = sorted(set(workflow_names) - set(command_names))
        extra = sorted(set(command_names) - set(workflow_names))
        raise ValueError(f"command/workflow mismatch; missing={missing}, extra={extra}")

    for command_name in command_names:
        command_path = commands_dir / command_name
        command_text = command_path.read_text(encoding="utf-8")
        expected_reference = f"skills/{SKILL_DIR_NAME}/workflows/{command_name}"
        if skill_name not in command_text or expected_reference not in command_text:
            raise ValueError(f"{command_path} must load {skill_name} and reference {expected_reference}")


def _assert_bundled_literature_mcp(
    plugin_root: Path,
    platform: str,
    *,
    mcp_server_name: str = MCP_SERVER_NAME,
) -> None:
    provider_entrypoint = plugin_root / "mcp" / "qiongli-literature-provider" / "index.mjs"
    _assert_file(provider_entrypoint, "bundled literature MCP entrypoint")

    if platform == "codex":
        codex_manifest_path = plugin_root / ".codex-plugin" / "plugin.json"
        codex_manifest = _read_json(codex_manifest_path)
        if codex_manifest.get("mcpServers") != "./.mcp.json":
            raise ValueError(f"{codex_manifest_path} mcpServers must point to ./.mcp.json")
        config_path = plugin_root / ".mcp.json"
        expected_args = ["./mcp/qiongli-literature-provider/index.mjs"]
    elif platform == "claude":
        config_path = plugin_root / ".claude-plugin" / "plugin.json"
        expected_args = ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"]
    else:
        raise ValueError(f"unsupported bundled literature MCP platform: {platform}")

    config_text = config_path.read_text(encoding="utf-8")
    for forbidden in ("QIONGLI_OPENALEX_EMAIL", "SEMANTIC_SCHOLAR_API_KEY", "qiongli mcp"):
        if forbidden in config_text:
            raise ValueError(f"{config_path} must not contain forbidden bundled MCP config string: {forbidden}")

    config = _read_json(config_path)
    mcp_servers = config.get("mcpServers")
    if not isinstance(mcp_servers, dict):
        raise ValueError(f"{config_path} missing mcpServers")
    if mcp_server_name != MCP_SERVER_NAME and MCP_SERVER_NAME in mcp_servers:
        raise ValueError(f"{config_path} must not expose stable mcpServers.{MCP_SERVER_NAME}")
    server = mcp_servers.get(mcp_server_name)
    if not isinstance(server, dict):
        raise ValueError(f"{config_path} missing mcpServers.{mcp_server_name}")
    if server.get("command") != "node":
        raise ValueError(f"{config_path} mcpServers.{mcp_server_name}.command must be node")
    if server.get("args") != expected_args:
        raise ValueError(
            f"{config_path} mcpServers.{mcp_server_name}.args expected {expected_args}, found {server.get('args')}"
        )


def _assert_manifest(
    platform: str,
    manifest_path: Path,
    expected_version: str,
    *,
    expected_plugin_name: str = PLUGIN_NAME,
    expected_skill_name: str = SKILL_NAME,
) -> None:
    manifest = _read_json(manifest_path)
    for key in ("name", "version", "description"):
        if not manifest.get(key):
            raise ValueError(f"{manifest_path} missing required field: {key}")
    if manifest["name"] != expected_plugin_name:
        raise ValueError(f"{manifest_path} expected name {expected_plugin_name}, found {manifest['name']}")
    if manifest["version"] != expected_version:
        raise ValueError(f"{manifest_path} expected version {expected_version}, found {manifest['version']}")

    if platform == "codex":
        if manifest.get("skills") != "./skills/":
            raise ValueError(f"{manifest_path} must expose skills via ./skills/")
        interface = manifest.get("interface")
        if not isinstance(interface, dict):
            raise ValueError(f"{manifest_path} missing Codex interface metadata")
        prompts = interface.get("defaultPrompt")
        if not isinstance(prompts, list) or not any(f"${expected_skill_name}" in str(item) for item in prompts):
            raise ValueError(f"{manifest_path} defaultPrompt must include ${expected_skill_name}")


def _validate_artifact(
    artifact: Path,
    spec: ArtifactSpec,
    expected_repo_tag: str,
    expected_version: str,
    *,
    plugin_name: str = PLUGIN_NAME,
    subject: str | None = None,
    coverage: str | None = None,
    skill_name: str = SKILL_NAME,
    subject_label: str | None = None,
) -> str:
    with tempfile.TemporaryDirectory(prefix=f"qiongli-{spec.platform}-artifact-") as tmp:
        bundle_root = _extract_marketplace_root(artifact, Path(tmp))
        plugin_root = (bundle_root / "plugins" / plugin_name).resolve()
        manifest_path = plugin_root / (".codex-plugin" if spec.platform == "codex" else ".claude-plugin") / "plugin.json"
        skill_root = plugin_root / "skills" / SKILL_DIR_NAME

        _assert_manifest(
            spec.platform,
            manifest_path,
            expected_version,
            expected_plugin_name=plugin_name,
            expected_skill_name=skill_name,
        )
        workflow_names = _assert_skill_invocation(skill_root, expected_repo_tag, skill_name=skill_name)
        if subject is not None:
            _assert_subject_marker(skill_root, subject)
            _assert_subject_manifest(skill_root, subject, coverage or "complete")
        if spec.requires_commands:
            _assert_command_invocation(plugin_root, workflow_names, skill_name=skill_name)
        if spec.expects_bundled_mcp:
            _assert_bundled_literature_mcp(
                plugin_root,
                spec.platform,
                mcp_server_name=_mcp_server_name_for_plugin(plugin_name),
            )

    label = subject_label or subject
    subject_suffix = f" ({label})" if label else ""
    checked = f"{skill_name} invocation checked"
    if spec.expects_bundled_mcp:
        checked += "; bundled literature MCP checked"
    archive_label = " ZIP" if artifact.suffix == ".zip" else ""
    return f"[OK] {spec.platform} marketplace{archive_label} artifact{subject_suffix}: {checked}"


def _validate_claude_desktop_artifact(
    artifact: Path,
    expected_repo_tag: str,
    subject: str,
    *,
    skill_name: str = SKILL_NAME,
    subject_label: str | None = None,
) -> str:
    file_count = _assert_claude_desktop_zip_budget(artifact, subject, skill_name=skill_name)
    with tempfile.TemporaryDirectory(prefix=f"qiongli-claude-desktop-{subject}-artifact-") as tmp:
        skill_root = _extract_single_zip_root(artifact, Path(tmp))
        if skill_root.name != skill_name:
            raise ValueError(f"{artifact} must contain top-level {skill_name}/ directory")
        _assert_skill_invocation(skill_root, expected_repo_tag, skill_name=skill_name)
        if subject == "economics":
            _assert_economics_desktop_package(skill_root)
        elif subject == "business":
            _assert_business_desktop_package(skill_root)
        elif subject == "economics-accounting":
            _assert_economics_accounting_desktop_package(skill_root)
        elif subject == "finance":
            _assert_finance_desktop_package(skill_root)
        elif subject == "core":
            _assert_core_desktop_package(skill_root)
        else:
            _assert_focused_desktop_package(skill_root, subject)
        _assert_file(skill_root / "skills-core.md", "consolidated skill core")
        _assert_file(skill_root / "skills-summary.md", "consolidated skill summary")
        if (skill_root / ".claude-plugin").exists():
            raise ValueError(f"{artifact} must not include Claude Code plugin metadata")
        if (skill_root / "commands").exists():
            raise ValueError(f"{artifact} must not include Claude Code slash command wrappers")

    label = subject_label or subject
    return (
        f"[OK] claude-desktop skill artifact ({label}): {skill_name} invocation checked; "
        f"{file_count}/{CLAUDE_DESKTOP_FILE_BUDGET} files under desktop file budget"
    )


def _validate_subject_specialization(root: Path) -> None:
    try:
        try:
            from scripts.audit_subject_specialization import audit_subject_specialization
        except ModuleNotFoundError as exc:
            if exc.name == "yaml":
                raise
            from audit_subject_specialization import audit_subject_specialization
    except ModuleNotFoundError as exc:
        if exc.name != "yaml":
            raise
        warnings.warn(
            "Skipping subject specialization audit because optional dependency PyYAML is unavailable.",
            RuntimeWarning,
            stacklevel=2,
        )
        return

    findings = audit_subject_specialization(root)
    if not findings:
        return
    details = "\n".join(f"{finding.subject}: {finding.code}: {finding.message}" for finding in findings)
    raise ValueError(f"Subject specialization audit failed:\n{details}")


def _validate_subject_eval_cases(root: Path) -> None:
    try:
        try:
            from scripts.audit_subject_eval_cases import audit_subject_eval_cases
        except ModuleNotFoundError as exc:
            if exc.name == "yaml":
                raise
            from audit_subject_eval_cases import audit_subject_eval_cases
    except ModuleNotFoundError as exc:
        if exc.name != "yaml":
            raise
        warnings.warn(
            "Skipping subject eval case audit because optional dependency PyYAML is unavailable.",
            RuntimeWarning,
            stacklevel=2,
        )
        return

    findings = audit_subject_eval_cases(root)
    if not findings:
        return
    details = "\n".join(f"{finding.case_id}: {finding.code}: {finding.message}" for finding in findings)
    raise ValueError(f"Subject eval case audit failed:\n{details}")


def validate(root: Path, dist_dir: Path) -> list[str]:
    root = root.resolve()
    dist_dir = dist_dir.resolve()
    expected_repo_tag = (RepoLayout(root).workflow / "VERSION").read_text(encoding="utf-8").strip()
    expected_version = expected_repo_tag.removeprefix("v")

    artifacts = build_artifacts(root, expected_repo_tag, dist_dir)
    by_platform = {artifact.name: artifact for artifact in artifacts}
    messages: list[str] = []

    if _is_prerelease_tag(expected_repo_tag):
        for platform in ("codex", "claude"):
            spec = ARTIFACT_SPECS[platform]
            artifact_name = f"{NEXT_PLUGIN_NAME}-{platform}-plugin-{expected_repo_tag}.tar.gz"
            artifact = by_platform.get(artifact_name)
            if artifact is None:
                raise ValueError(f"expected {platform} next marketplace artifact: {artifact_name}")
            messages.append(
                _validate_artifact(
                    artifact,
                    spec,
                    expected_repo_tag,
                    expected_version,
                    plugin_name=NEXT_PLUGIN_NAME,
                    subject="core",
                    coverage="complete",
                    skill_name=NEXT_SKILL_NAME,
                    subject_label="core-next",
                )
            )
            if platform == "claude":
                zip_name = f"{NEXT_PLUGIN_NAME}-claude-plugin-{expected_repo_tag}.zip"
                zip_artifact = by_platform.get(zip_name)
                if zip_artifact is None:
                    raise ValueError(f"expected claude next marketplace ZIP artifact: {zip_name}")
                messages.append(
                    _validate_artifact(
                        zip_artifact,
                        spec,
                        expected_repo_tag,
                        expected_version,
                        plugin_name=NEXT_PLUGIN_NAME,
                        subject="core",
                        coverage="complete",
                        skill_name=NEXT_SKILL_NAME,
                        subject_label="core-next",
                    )
                )

        desktop_name = f"{NEXT_PLUGIN_NAME}-claude-desktop-skill-core-{expected_repo_tag}.zip"
        desktop_artifact = by_platform.get(desktop_name)
        if desktop_artifact is None:
            raise ValueError(f"expected claude-desktop next core artifact: {desktop_name}")
        messages.append(
            _validate_claude_desktop_artifact(
                desktop_artifact,
                expected_repo_tag,
                "core",
                skill_name=NEXT_SKILL_NAME,
                subject_label="core-next",
            )
        )

        _validate_subject_specialization(root)
        _validate_subject_eval_cases(root)
        return messages

    for platform, spec in ARTIFACT_SPECS.items():
        artifact_name = f"{PLUGIN_NAME}-{platform}-plugin-{expected_repo_tag}.tar.gz"
        artifact = by_platform.get(artifact_name)
        if artifact is None:
            raise ValueError(f"expected {platform} artifact: {artifact_name}")
        messages.append(_validate_artifact(artifact, spec, expected_repo_tag, expected_version))
        if platform == "claude":
            zip_name = f"{PLUGIN_NAME}-claude-plugin-{expected_repo_tag}.zip"
            zip_artifact = by_platform.get(zip_name)
            if zip_artifact is None:
                raise ValueError(f"expected claude ZIP artifact: {zip_name}")
            messages.append(_validate_artifact(zip_artifact, spec, expected_repo_tag, expected_version))

    for subject in _marketplace_subjects(root):
        plugin_name = f"{PLUGIN_NAME}-{subject}"
        for platform in ("codex", "claude"):
            spec = ARTIFACT_SPECS[platform]
            artifact_name = f"{plugin_name}-{platform}-plugin-{expected_repo_tag}.tar.gz"
            artifact = by_platform.get(artifact_name)
            if artifact is None:
                raise ValueError(f"expected {platform} marketplace {subject} artifact: {artifact_name}")
            messages.append(
                _validate_artifact(
                    artifact,
                    spec,
                    expected_repo_tag,
                    expected_version,
                    plugin_name=plugin_name,
                    subject=subject,
                    coverage="complete",
                )
            )
            if platform == "claude":
                zip_name = f"{plugin_name}-claude-plugin-{expected_repo_tag}.zip"
                zip_artifact = by_platform.get(zip_name)
                if zip_artifact is None:
                    raise ValueError(f"expected claude marketplace {subject} ZIP artifact: {zip_name}")
                messages.append(
                    _validate_artifact(
                        zip_artifact,
                        spec,
                        expected_repo_tag,
                        expected_version,
                        plugin_name=plugin_name,
                        subject=subject,
                        coverage="complete",
                    )
                )

    for subject in _desktop_subjects(root):
        desktop_name = f"{PLUGIN_NAME}-claude-desktop-skill-{subject}-{expected_repo_tag}.zip"
        desktop_artifact = by_platform.get(desktop_name)
        if desktop_artifact is None:
            raise ValueError(f"expected claude-desktop {subject} artifact: {desktop_name}")
        messages.append(_validate_claude_desktop_artifact(desktop_artifact, expected_repo_tag, subject))

    legacy_desktop_name = f"{PLUGIN_NAME}-claude-desktop-skill-{expected_repo_tag}.zip"
    legacy_desktop_artifact = by_platform.get(legacy_desktop_name)
    if legacy_desktop_artifact is None:
        raise ValueError(f"expected legacy claude-desktop artifact: {legacy_desktop_name}")
    messages.append(_validate_claude_desktop_artifact(legacy_desktop_artifact, expected_repo_tag, "core"))

    _validate_subject_specialization(root)
    _validate_subject_eval_cases(root)

    return messages


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate marketplace artifacts install Qiongli and expose platform invocation surfaces."
    )
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--dist-dir", type=Path, help="Directory for temporary artifact builds. Defaults to a temp dir.")
    args = parser.parse_args(argv)

    try:
        if args.dist_dir is None:
            with tempfile.TemporaryDirectory(prefix="qiongli-marketplace-validate-") as tmp:
                messages = validate(args.root, Path(tmp))
        else:
            messages = validate(args.root, args.dist_dir)
    except ValueError as exc:
        print(f"[FAIL] marketplace validation: {exc}")
        return 1

    for message in messages:
        print(message)
    print("[OK] marketplace validation completed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
