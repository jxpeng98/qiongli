#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (Path(__file__).resolve().parent, PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from build_plugin_artifacts import _desktop_subjects, _is_prerelease_tag, _marketplace_subjects
from qiongli.platform_targets import PlatformTarget, load_platform_targets


DEFAULT_REPO_SLUG = "jxpeng98/qiongli"
PLUGIN_NAME = "qiongli"
NEXT_PLUGIN_NAME = "qiongli-next"
MCPB_MANIFEST = REPO_ROOT / "packages" / "qiongli-literature-mcpb" / "manifest.json"
ZOTERO_COMPANION_MANIFEST = REPO_ROOT / "packages" / "qiongli-zotero-companion" / "manifest.json"
ZOTERO_COMPANION_ASSET_SLUG = "qiongli-zotero-companion"
COMPANION_TARGET_REGISTRY_RELATIVE_PATH = Path("content") / "distribution" / "release-companion-targets.yaml"
COMPANION_TARGET_REQUIRED_FIELDS = (
    "target_id",
    "subject",
    "artifact_kind",
    "expected_install_method",
)
REQUIRED_COMPANION_TARGET_KEYS = frozenset(
    {
        "claude_desktop_literature_mcpb",
        "zotero_desktop_companion",
        "download_guide",
        "download_index",
        "artifact_manifest",
    }
)
REQUIRED_RECOMMENDED_TARGET_KEYS = (
    "qiongli_cli",
    "codex",
    "claude_code",
    "claude_desktop_skill",
    "claude_desktop_plugin",
)


def _normalize_tag(raw: str) -> str:
    tag = raw.strip()
    if not tag:
        raise ValueError("tag is required")
    return tag if tag.startswith("v") else f"v{tag}"


def _asset_url(repo_slug: str, tag: str, name: str) -> str:
    return f"https://github.com/{repo_slug}/releases/download/{tag}/{name}"


def _mcpb_asset_name() -> str:
    manifest = json.loads(MCPB_MANIFEST.read_text(encoding="utf-8"))
    name = manifest.get("name")
    version = manifest.get("version")
    if not isinstance(name, str) or not name:
        raise ValueError(f"{MCPB_MANIFEST} must define name")
    if not isinstance(version, str) or not version:
        raise ValueError(f"{MCPB_MANIFEST} must define version")
    return f"{name}-{version}.mcpb"


def _zotero_companion_asset_name() -> str:
    manifest = json.loads(ZOTERO_COMPANION_MANIFEST.read_text(encoding="utf-8"))
    version = manifest.get("version")
    if not isinstance(version, str) or not version:
        raise ValueError(f"{ZOTERO_COMPANION_MANIFEST} must define version")
    return f"{ZOTERO_COMPANION_ASSET_SLUG}-{version}.xpi"


def _claude_plugin_zip(plugin_name: str, tag: str) -> str:
    return f"{plugin_name}-claude-plugin-{tag}.zip"


def _claude_desktop_plugin_zip(plugin_name: str, tag: str) -> str:
    return f"{plugin_name}-claude-desktop-plugin-{tag}.zip"


def _release_assets(tag: str, root: Path) -> dict[str, list[str] | str]:
    if _is_prerelease_tag(tag):
        return {
            "download_guide": f"qiongli-downloads-{tag}.md",
            "download_index": f"qiongli-downloads-{tag}.json",
            "artifact_manifest": f"qiongli-artifacts-{tag}.json",
            "claude_desktop_plugin": _claude_desktop_plugin_zip(NEXT_PLUGIN_NAME, tag),
            "claude_desktop_skills": [
                f"{NEXT_PLUGIN_NAME}-claude-desktop-skill-core-{tag}.zip",
            ],
            "claude_desktop_legacy_core_skill": "",
            "claude_desktop_literature_mcpb": _mcpb_asset_name(),
            "zotero_desktop_companion": _zotero_companion_asset_name(),
            "maintainer_plugin_tarballs": [
                f"{NEXT_PLUGIN_NAME}-codex-plugin-{tag}.tar.gz",
                f"{NEXT_PLUGIN_NAME}-claude-plugin-{tag}.tar.gz",
            ],
            "maintainer_plugin_zips": [
                _claude_plugin_zip(NEXT_PLUGIN_NAME, tag),
            ],
        }

    marketplace_subjects = _marketplace_subjects(root)
    desktop_subjects = _desktop_subjects(root)
    plugin_tarballs = [
        f"{PLUGIN_NAME}-{platform}-plugin-{tag}.tar.gz"
        for platform in ("codex", "claude")
    ]
    plugin_zips = [_claude_plugin_zip(PLUGIN_NAME, tag)]
    for subject in marketplace_subjects:
        plugin_name = f"{PLUGIN_NAME}-{subject}"
        plugin_tarballs.extend(
            f"{plugin_name}-{platform}-plugin-{tag}.tar.gz"
            for platform in ("codex", "claude")
        )
        plugin_zips.append(_claude_plugin_zip(plugin_name, tag))

    return {
        "download_guide": f"qiongli-downloads-{tag}.md",
        "download_index": f"qiongli-downloads-{tag}.json",
        "artifact_manifest": f"qiongli-artifacts-{tag}.json",
        "claude_desktop_plugin": _claude_desktop_plugin_zip(PLUGIN_NAME, tag),
        "claude_desktop_skills": [
            f"{PLUGIN_NAME}-claude-desktop-skill-{subject}-{tag}.zip"
            for subject in desktop_subjects
        ],
        "claude_desktop_legacy_core_skill": f"{PLUGIN_NAME}-claude-desktop-skill-{tag}.zip",
        "claude_desktop_literature_mcpb": _mcpb_asset_name(),
        "zotero_desktop_companion": _zotero_companion_asset_name(),
        "maintainer_plugin_tarballs": plugin_tarballs,
        "maintainer_plugin_zips": plugin_zips,
    }


def load_companion_targets(root: Path = REPO_ROOT) -> dict[str, dict[str, str]]:
    registry_path = root / COMPANION_TARGET_REGISTRY_RELATIVE_PATH
    if not registry_path.is_file():
        raise ValueError(f"missing release companion target registry: {registry_path}")

    payload = yaml.safe_load(registry_path.read_text(encoding="utf-8")) or {}
    if not isinstance(payload, dict):
        raise ValueError(f"{registry_path} must contain a YAML object")
    if payload.get("schema_version") != "1.0":
        raise ValueError(f"{registry_path} schema_version must be 1.0")

    raw_targets = payload.get("targets")
    if not isinstance(raw_targets, dict) or not raw_targets:
        raise ValueError(f"{registry_path} must define targets")

    targets: dict[str, dict[str, str]] = {}
    for asset_key, raw_target in raw_targets.items():
        if not isinstance(asset_key, str) or not asset_key:
            raise ValueError(f"{registry_path} contains an invalid asset key: {asset_key!r}")
        if not isinstance(raw_target, dict):
            raise ValueError(f"{registry_path} target {asset_key} must be an object")
        target: dict[str, str] = {}
        for field in COMPANION_TARGET_REQUIRED_FIELDS:
            value = raw_target.get(field)
            if not isinstance(value, str) or not value:
                raise ValueError(f"{registry_path} target {asset_key}.{field} must be a non-empty string")
            target[field] = value
        targets[asset_key] = target
    missing = sorted(REQUIRED_COMPANION_TARGET_KEYS.difference(targets))
    if missing:
        raise ValueError(f"{registry_path} missing required companion target keys: {', '.join(missing)}")
    return targets


def _target_index(targets: dict[str, PlatformTarget]) -> dict[str, dict[str, Any]]:
    return {
        target_id: {
            "display_name": target.display_name,
            "artifact_kind": target.artifact_kind,
            "archive_format": target.archive_format,
            "adapter": dict(target.adapter),
            "smoke": dict(target.smoke),
            "bundled_mcp_mode": target.bundled_mcp_mode,
            "command_surface": target.command_surface,
            "validator": target.validator,
            "required_paths": list(target.required_paths),
            "forbidden_paths": list(target.forbidden_paths),
            "release_download": dict(target.release_download),
        }
        for target_id, target in targets.items()
    }


def _asset_matches_target(value: str, target: PlatformTarget) -> bool:
    contains = target.release_download.get("asset_name_contains", [])
    if not contains:
        return True
    if not isinstance(contains, list):
        return True
    tokens = [token for token in contains if isinstance(token, str) and token]
    return not tokens or any(token in value for token in tokens)


def _assets_by_target(
    assets: dict[str, list[str] | str],
    targets: dict[str, PlatformTarget],
) -> dict[str, dict[str, list[str] | str]]:
    grouped: dict[str, dict[str, list[str] | str]] = {}
    for target_id, target in targets.items():
        groups = target.release_download.get("asset_groups", [])
        if not isinstance(groups, list):
            continue
        target_assets: dict[str, list[str] | str] = {}
        for group in groups:
            if not isinstance(group, str):
                continue
            value = assets.get(group)
            if isinstance(value, list):
                filtered = [asset for asset in value if _asset_matches_target(asset, target)]
                if filtered:
                    target_assets[group] = filtered
            elif isinstance(value, str) and value and _asset_matches_target(value, target):
                target_assets[group] = value
        if target_assets:
            grouped[target_id] = target_assets
    return grouped


def _companion_assets_by_target(
    assets: dict[str, list[str] | str],
    companion_targets: dict[str, dict[str, str]],
) -> dict[str, dict[str, list[str] | str]]:
    grouped: dict[str, dict[str, list[str] | str]] = {}
    for asset_key, target in companion_targets.items():
        if not isinstance(target, dict):
            continue
        target_id = str(target.get("target_id") or asset_key)
        value = assets.get(asset_key)
        if _asset_names(value):
            grouped[target_id] = {asset_key: value}
    return grouped


def _recommended_target_ids(targets: dict[str, PlatformTarget]) -> dict[str, str]:
    resolved: dict[str, str] = {}
    for key in REQUIRED_RECOMMENDED_TARGET_KEYS:
        matches = sorted(
            target_id
            for target_id, target in targets.items()
            if target.release_download.get("recommended_key") == key
        )
        if len(matches) != 1:
            raise ValueError(
                "platform target registry must define exactly one "
                f"release_download.recommended_key={key!r}; found {len(matches)}"
            )
        resolved[key] = matches[0]
    return resolved


def build_index(tag: str, repo_slug: str = DEFAULT_REPO_SLUG, root: Path = REPO_ROOT) -> dict[str, Any]:
    tag = _normalize_tag(tag)
    is_next = _is_prerelease_tag(tag)
    assets = _release_assets(tag, root)
    targets = load_platform_targets(root)
    recommended_target_ids = _recommended_target_ids(targets)
    assets_by_target = _assets_by_target(assets, targets)
    companion_targets = load_companion_targets(root)
    assets_by_target.update(_companion_assets_by_target(assets, companion_targets))
    recommended: dict[str, Any] = {
        "qiongli_cli": {
            "target_id": recommended_target_ids["qiongli_cli"],
            "install": "npm_next" if is_next else "npm_latest",
            "command": (
                'npx qiongli@next install --target all --project-dir "$PWD"'
                if is_next
                else 'npx qiongli@latest install --target all --project-dir "$PWD"'
            ),
        },
        "codex": {
            "target_id": recommended_target_ids["codex"],
            "install": "marketplace",
            "command": "codex plugin marketplace add jxpeng98/skillsplace --ref main",
            "plugin": NEXT_PLUGIN_NAME if is_next else PLUGIN_NAME,
            "manual_asset": None,
        },
        "claude_code": {
            "target_id": recommended_target_ids["claude_code"],
            "install": "marketplace",
            "command": "claude plugin marketplace add jxpeng98/skillsplace@main",
            "plugin": NEXT_PLUGIN_NAME if is_next else PLUGIN_NAME,
            "manual_asset": None,
        },
        "claude_desktop_skill": {
            "target_id": recommended_target_ids["claude_desktop_skill"],
            "install": "download_zip",
            "asset_pattern": (
                f"{NEXT_PLUGIN_NAME}-claude-desktop-skill-core-{tag}.zip"
                if is_next
                else f"{PLUGIN_NAME}-claude-desktop-skill-<subject>-{tag}.zip"
            ),
        },
        "claude_desktop_plugin": {
            "target_id": recommended_target_ids["claude_desktop_plugin"],
            "install": "download_plugin_zip",
            "asset": assets["claude_desktop_plugin"],
            "note": "Use this for direct Claude Desktop/plugin install; use skill ZIP only for manual skill upload.",
        },
        "claude_desktop_literature_mcpb": {
            "install": "download_mcpb",
            "asset": assets["claude_desktop_literature_mcpb"],
        },
        "zotero_desktop_companion": {
            "install": "download_xpi",
            "asset": assets["zotero_desktop_companion"],
        },
    }
    return {
        "tag": tag,
        "channel": "next" if is_next else "stable",
        "release_url": f"https://github.com/{repo_slug}/releases/tag/{tag}",
        "recommended": recommended,
        "assets": {
            key: value for key, value in assets.items()
        },
        "asset_urls": {
            key: (
                [_asset_url(repo_slug, tag, item) for item in value]
                if isinstance(value, list)
                else _asset_url(repo_slug, tag, value)
            )
            for key, value in assets.items()
            if value
        },
        "platform_targets": _target_index(targets),
        "companion_target_registry": {
            "path": COMPANION_TARGET_REGISTRY_RELATIVE_PATH.as_posix(),
            "schema_version": "1.0",
        },
        "companion_targets": {
            key: dict(value)
            for key, value in companion_targets.items()
        },
        "assets_by_target": assets_by_target,
    }


def build_artifact_manifest(index: dict[str, Any]) -> dict[str, Any]:
    tag = str(index["tag"])
    channel = str(index["channel"])
    targets = index.get("platform_targets", {})
    if not isinstance(targets, dict):
        targets = {}
    recommended = index.get("recommended", {})
    if not isinstance(recommended, dict):
        recommended = {}
    companion_targets = index.get("companion_targets", {})
    if not isinstance(companion_targets, dict):
        companion_targets = {}
    companion_targets_by_id = {
        str(target.get("target_id") or asset_key): dict(target)
        for asset_key, target in companion_targets.items()
        if isinstance(target, dict)
    }
    records: list[dict[str, Any]] = []
    seen: set[tuple[str, str]] = set()

    assets_by_target = index.get("assets_by_target", {})
    if isinstance(assets_by_target, dict):
        for target_id, target_assets in assets_by_target.items():
            if not isinstance(target_id, str) or not isinstance(target_assets, dict):
                continue
            target = targets.get(target_id, {})
            if not isinstance(target, dict):
                target = {}
            registry_target = bool(target)
            if not target:
                target = companion_targets_by_id.get(target_id, {})
            install_method = str(
                target.get("expected_install_method")
                or _expected_install_method(target_id, target, recommended)
            )
            for asset_key, value in target_assets.items():
                for asset in _asset_names(value):
                    key = (asset_key, asset)
                    if key in seen:
                        continue
                    seen.add(key)
                    records.append(
                        {
                            "asset": asset,
                            "asset_key": asset_key,
                            "target_id": target_id,
                            "registry_target": registry_target,
                            "subject": str(
                                target.get("subject") or _subject_from_asset(asset, tag)
                            ),
                            "archive_format": _archive_format(asset),
                            "expected_install_method": install_method,
                            "artifact_kind": str(target.get("artifact_kind") or ""),
                            "adapter": dict(target.get("adapter") or {}),
                            "validator": str(target.get("validator") or ""),
                            "smoke": dict(target.get("smoke") or {}),
                            "required_paths": list(target.get("required_paths") or []),
                            "forbidden_paths": list(target.get("forbidden_paths") or []),
                        }
                    )

    assets = index.get("assets", {})
    if isinstance(assets, dict):
        for asset_key, target in companion_targets.items():
            if not isinstance(target, dict):
                continue
            for asset in _asset_names(assets.get(asset_key)):
                key = (asset_key, asset)
                if key in seen:
                    continue
                seen.add(key)
                records.append(
                    {
                        "asset": asset,
                        "asset_key": asset_key,
                        "target_id": str(target.get("target_id") or asset_key),
                        "registry_target": False,
                        "subject": str(target.get("subject") or "not-applicable"),
                        "archive_format": _archive_format(asset),
                        "expected_install_method": str(target.get("expected_install_method") or asset_key),
                        "artifact_kind": str(target.get("artifact_kind") or "release-metadata"),
                        "adapter": {},
                        "validator": "",
                        "required_paths": [],
                        "forbidden_paths": [],
                    }
                )

    return {
        "schema_version": "1.0",
        "tag": tag,
        "channel": channel,
        "release_url": str(index["release_url"]),
        "artifacts": records,
    }


def _asset_names(value: Any) -> list[str]:
    if isinstance(value, str) and value:
        return [value]
    if isinstance(value, list):
        return [item for item in value if isinstance(item, str) and item]
    return []


def _expected_install_method(target_id: str, target: dict[str, Any], recommended: dict[str, Any]) -> str:
    release_download = target.get("release_download", {})
    if not isinstance(release_download, dict):
        release_download = {}
    recommended_key = release_download.get("recommended_key")
    if isinstance(recommended_key, str):
        entry = recommended.get(recommended_key, {})
        if isinstance(entry, dict) and isinstance(entry.get("install"), str):
            return entry["install"]
        return recommended_key
    return target_id


def _archive_format(asset: str) -> str:
    if asset.endswith(".tar.gz"):
        return "tar.gz"
    suffix = Path(asset).suffix
    return suffix.removeprefix(".") if suffix else "unknown"


def _subject_from_asset(asset: str, tag: str) -> str:
    if asset.startswith(f"{NEXT_PLUGIN_NAME}-"):
        return "core"
    if asset == f"{PLUGIN_NAME}-claude-desktop-skill-{tag}.zip":
        return "core"

    desktop_prefix = f"{PLUGIN_NAME}-claude-desktop-skill-"
    desktop_suffix = f"-{tag}.zip"
    if asset.startswith(desktop_prefix) and asset.endswith(desktop_suffix):
        subject = asset.removeprefix(desktop_prefix).removesuffix(desktop_suffix)
        return subject or "core"

    for platform, suffixes in (
        ("codex", (f"-codex-plugin-{tag}.tar.gz",)),
        ("claude", (f"-claude-plugin-{tag}.tar.gz", f"-claude-plugin-{tag}.zip")),
    ):
        base_names = {f"{PLUGIN_NAME}-{platform}-plugin-{tag}.tar.gz"}
        if platform == "claude":
            base_names.add(f"{PLUGIN_NAME}-claude-plugin-{tag}.zip")
        if asset in base_names:
            return "core"
        for suffix in suffixes:
            prefix = f"{PLUGIN_NAME}-"
            if asset.startswith(prefix) and asset.endswith(suffix):
                subject = asset.removeprefix(prefix).removesuffix(suffix)
                return subject or "core"

    if asset == _claude_desktop_plugin_zip(PLUGIN_NAME, tag):
        return "core"
    return "not-applicable"


def _desktop_skill_label(asset: str, tag: str) -> str:
    for prefix in (
        f"{NEXT_PLUGIN_NAME}-claude-desktop-skill-",
        f"{PLUGIN_NAME}-claude-desktop-skill-",
    ):
        if asset.startswith(prefix):
            return asset.removeprefix(prefix).removesuffix(f"-{tag}.zip")
    return asset


def _markdown_link(label: str, url: str) -> str:
    return f"[`{label}`]({url})"


def _target_label(platform_targets: dict[str, Any], target_id: str, fallback: str) -> str:
    target = platform_targets.get(target_id, {})
    if not isinstance(target, dict):
        return fallback
    release_download = target.get("release_download", {})
    if not isinstance(release_download, dict):
        release_download = {}
    label = release_download.get("guide_label") or target.get("display_name") or fallback
    return f"{label} (`{target_id}`)"


def _recommended_target_label(
    platform_targets: dict[str, Any],
    recommended: dict[str, Any],
    key: str,
    fallback: str,
) -> str:
    entry = recommended.get(key, {})
    if not isinstance(entry, dict):
        return fallback
    target_id = entry.get("target_id")
    if not isinstance(target_id, str) or not target_id:
        return fallback
    return _target_label(platform_targets, target_id, fallback)


def _platform_target_rows(platform_targets: dict[str, Any]) -> str:
    rows: list[str] = []
    for target_id, target in platform_targets.items():
        if not isinstance(target, dict):
            continue
        rows.append(
            "| "
            f"`{target_id}` | "
            f"{target.get('display_name', target_id)} | "
            f"`{target.get('archive_format', 'unknown')}` | "
            f"`{target.get('validator', 'unknown')}` |"
        )
    return "\n".join(rows) or "| None | None | None | None |"


def render_markdown(index: dict[str, Any]) -> str:
    tag = str(index["tag"])
    channel = str(index["channel"])
    release_url = str(index["release_url"])
    recommended = index["recommended"]
    assets = index["assets"]
    asset_urls = index["asset_urls"]
    platform_targets = index.get("platform_targets", {})
    if not isinstance(platform_targets, dict):
        platform_targets = {}
    desktop_skills = list(assets["claude_desktop_skills"])
    desktop_plugin_asset = str(assets["claude_desktop_plugin"])
    mcpb_asset = str(assets["claude_desktop_literature_mcpb"])
    zotero_asset = str(assets["zotero_desktop_companion"])
    guide_asset = str(assets["download_guide"])
    index_asset = str(assets["download_index"])
    manifest_asset = str(assets["artifact_manifest"])
    plugin_zips = list(assets.get("maintainer_plugin_zips", []))
    desktop_core_asset = (
        f"{NEXT_PLUGIN_NAME}-claude-desktop-skill-core-{tag}.zip"
        if channel == "next"
        else f"{PLUGIN_NAME}-claude-desktop-skill-core-{tag}.zip"
    )
    if desktop_core_asset not in desktop_skills and desktop_skills:
        desktop_core_asset = desktop_skills[0]

    desktop_urls = asset_urls.get("claude_desktop_skills")
    desktop_url_by_asset = (
        dict(zip(desktop_skills, desktop_urls))
        if isinstance(desktop_urls, list)
        else {}
    )
    desktop_core_url = desktop_url_by_asset.get(desktop_core_asset, release_url)
    if isinstance(desktop_urls, list) and desktop_core_asset in desktop_skills:
        desktop_core_url = desktop_urls[desktop_skills.index(desktop_core_asset)]
    desktop_plugin_url = str(asset_urls["claude_desktop_plugin"])
    mcpb_url = str(asset_urls["claude_desktop_literature_mcpb"])
    zotero_url = str(asset_urls["zotero_desktop_companion"])
    guide_url = str(asset_urls["download_guide"])
    index_url = str(asset_urls["download_index"])
    manifest_url = str(asset_urls["artifact_manifest"])

    desktop_rows = "\n".join(
        f"| `{_desktop_skill_label(asset, tag)}` | {_markdown_link(asset, desktop_url_by_asset.get(asset, release_url))} |"
        for asset in desktop_skills
    )
    plugin_zip_rows = "\n".join(f"- `{asset}`" for asset in plugin_zips)
    cli_channel_label = "npm `next`" if recommended["qiongli_cli"]["install"] == "npm_next" else "npm `latest`"
    channel_description = (
        "Stable releases are for everyday installs and upgrades. npm `latest`, PyPI stable, "
        "the `qiongli` marketplace entry, and the Desktop/MCP assets should all point at this tag."
        if channel == "stable"
        else "Next releases are prerelease validation builds. Use them to test `qiongli-next`, "
        "npm `next`, and the beta Desktop/MCP assets before the next stable release."
    )

    lines = [
        f"# Qiongli {tag} Download Guide",
        "",
        "Start here before using GitHub's asset list. Most users should not download plugin tarballs manually.",
        f"Channel: {channel}",
        "",
        channel_description,
        "",
        f"Release page: {release_url}",
        "",
        "## Direct downloads",
        "",
        "| Need | Link |",
        "|---|---|",
        f"| Release page and all assets | [Qiongli {tag}]({release_url}) |",
        f"| Claude Desktop direct plugin ZIP | {_markdown_link(desktop_plugin_asset, desktop_plugin_url)} |",
        f"| Default Claude Desktop/Web skill ZIP | {_markdown_link(desktop_core_asset, desktop_core_url)} |",
        f"| Claude Desktop literature MCPB | {_markdown_link(mcpb_asset, mcpb_url)} |",
        f"| Zotero Desktop companion XPI | {_markdown_link(zotero_asset, zotero_url)} |",
        f"| Human download guide | {_markdown_link(guide_asset, guide_url)} |",
        f"| Machine-readable download index | {_markdown_link(index_asset, index_url)} |",
        f"| Machine-readable artifact manifest | {_markdown_link(manifest_asset, manifest_url)} |",
        "",
        "## Start here",
        "",
        "| You use | Download or install | Why |",
        "|---|---|---|",
        f"| {_recommended_target_label(platform_targets, recommended, 'qiongli_cli', 'Qiongli CLI')} | `{recommended['qiongli_cli']['command']}` | Uses the {cli_channel_label} channel and bundled CLI payload. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'codex', 'Codex')} | Use the marketplace command; do not download a plugin tarball. | Marketplace install keeps skills and bundled literature MCP registration together. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'claude_code', 'Claude Code')} | Use the marketplace command; do not download a plugin tarball. | Marketplace install keeps slash commands, skills, and bundled literature MCP together. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'claude_desktop_plugin', 'Claude Desktop direct plugin')} | Download `{desktop_plugin_asset}`. | Use this recommended direct plugin for Claude Desktop/plugin install. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'claude_desktop_skill', 'Claude Desktop/Web skills')} | Download exactly one fallback skill ZIP from the table below. | Use skill ZIPs only for manual skill upload. |",
        f"| Claude Desktop literature tools | Download `{mcpb_asset}`. | MCPB adds local literature/provider tools and provider key configuration. |",
        f"| Zotero Desktop local writes | Download `{zotero_asset}` and install it from Zotero's add-on manager. | The companion enables Qiongli to search and write the local Zotero database through Zotero Desktop. |",
        "| Maintainers | Use plugin tarballs and Claude plugin ZIPs only for manual artifact checks or direct Claude plugin upload tests. | They are not the normal end-user install path. |",
        "",
        "## Claude Desktop/Web skill ZIPs",
        "",
        "| Subject | Asset |",
        "|---|---|",
        desktop_rows,
        "",
        "## Maintainer Claude plugin ZIPs",
        "",
        plugin_zip_rows or "- None",
        "",
        "## Platform target registry",
        "",
        "These rows come from `content/distribution/platform-targets.yaml`.",
        "",
        "| Target ID | Surface | Archive | Validator |",
        "|---|---|---|---|",
        _platform_target_rows(platform_targets),
        "",
        "## Machine-readable index",
        "",
        f"- Human guide asset: `{guide_asset}`",
        f"- JSON index asset: `{index_asset}`",
        f"- Artifact manifest asset: `{manifest_asset}`",
        "",
        "The JSON index groups assets by install surface; the artifact manifest flattens assets into per-target policy records.",
        "",
    ]
    return "\n".join(lines)


def render_release_notes_download_summary(index: dict[str, Any]) -> str:
    tag = str(index["tag"])
    recommended = index["recommended"]
    assets = index["assets"]
    platform_targets = index.get("platform_targets", {})
    if not isinstance(platform_targets, dict):
        platform_targets = {}

    desktop_skills = list(assets["claude_desktop_skills"])
    desktop_plugin_asset = str(assets["claude_desktop_plugin"])
    mcpb_asset = str(assets["claude_desktop_literature_mcpb"])
    zotero_asset = str(assets["zotero_desktop_companion"])
    guide_asset = str(assets["download_guide"])
    index_asset = str(assets["download_index"])
    manifest_asset = str(assets["artifact_manifest"])
    desktop_core_asset = (
        f"{NEXT_PLUGIN_NAME}-claude-desktop-skill-core-{tag}.zip"
        if str(index["channel"]) == "next"
        else f"{PLUGIN_NAME}-claude-desktop-skill-core-{tag}.zip"
    )
    if desktop_core_asset not in desktop_skills and desktop_skills:
        desktop_core_asset = desktop_skills[0]

    lines = [
        "Most users should not download plugin tarballs manually from GitHub's flat asset list.",
        "",
        "| You use | Recommended path |",
        "|---|---|",
        f"| {_recommended_target_label(platform_targets, recommended, 'qiongli_cli', 'Qiongli CLI')} | Use `{recommended['qiongli_cli']['command']}`. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'codex', 'Codex')} | Use `{recommended['codex']['command']}` and install `{recommended['codex']['plugin']}`; no manual tarball download required. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'claude_code', 'Claude Code')} | Use `{recommended['claude_code']['command']}` and install `{recommended['claude_code']['plugin']}`; no manual tarball download required. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'claude_desktop_plugin', 'Claude Desktop direct plugin')} | Download `{desktop_plugin_asset}` as the recommended direct plugin. |",
        f"| {_recommended_target_label(platform_targets, recommended, 'claude_desktop_skill', 'Claude Desktop/Web skills')} | Download `{desktop_core_asset}` as the fallback skill ZIP unless you need a different subject ZIP. |",
        f"| Claude Desktop literature tools | Download `{mcpb_asset}` and pair it with a Desktop skill ZIP when provider calls are required. |",
        f"| Zotero Desktop local writes | Download `{zotero_asset}` and install it from Zotero's add-on manager when local Zotero search/write support is required. |",
        "| Maintainers | Use Codex/Claude plugin tarballs and Claude plugin ZIPs only for manual marketplace artifact checks or direct Claude plugin upload tests. |",
        "",
        f"The release also includes `{guide_asset}`, `{index_asset}`, and `{manifest_asset}` to group the asset list by install surface and expose per-target artifact policy.",
    ]
    return "\n".join(lines)


def write_outputs(tag: str, out_dir: Path, repo_slug: str, root: Path = REPO_ROOT) -> tuple[Path, Path, Path]:
    tag = _normalize_tag(tag)
    out_dir.mkdir(parents=True, exist_ok=True)
    index = build_index(tag, repo_slug=repo_slug, root=root)
    manifest = build_artifact_manifest(index)
    guide_path = out_dir / f"qiongli-downloads-{tag}.md"
    index_path = out_dir / f"qiongli-downloads-{tag}.json"
    manifest_path = out_dir / f"qiongli-artifacts-{tag}.json"
    guide_path.write_text(render_markdown(index), encoding="utf-8")
    index_path.write_text(json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return guide_path, index_path, manifest_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Generate Qiongli GitHub release download guide assets.")
    parser.add_argument("--tag", required=True, help="Release tag, for example v1.1.0-beta.2")
    parser.add_argument("--out-dir", type=Path, default=Path("dist"), help="Directory for generated guide assets")
    parser.add_argument("--repo", default=DEFAULT_REPO_SLUG, help="GitHub repo slug used in generated URLs")
    args = parser.parse_args(argv)

    try:
        guide_path, index_path, manifest_path = write_outputs(args.tag, args.out_dir, args.repo)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"[downloads] {exc}", file=sys.stderr)
        return 1

    print(f"[downloads] generated: {guide_path}")
    print(f"[downloads] generated: {index_path}")
    print(f"[downloads] generated: {manifest_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
