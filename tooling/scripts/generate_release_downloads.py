#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (Path(__file__).resolve().parent, PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from build_plugin_artifacts import _desktop_subjects, _is_prerelease_tag, _marketplace_subjects


DEFAULT_REPO_SLUG = "jxpeng98/qiongli"
PLUGIN_NAME = "qiongli"
NEXT_PLUGIN_NAME = "qiongli-next"
MCPB_MANIFEST = REPO_ROOT / "packages" / "qiongli-literature-mcpb" / "manifest.json"
ZOTERO_COMPANION_MANIFEST = REPO_ROOT / "packages" / "qiongli-zotero-companion" / "manifest.json"
ZOTERO_COMPANION_ASSET_SLUG = "qiongli-zotero-companion"


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


def build_index(tag: str, repo_slug: str = DEFAULT_REPO_SLUG, root: Path = REPO_ROOT) -> dict[str, Any]:
    tag = _normalize_tag(tag)
    is_next = _is_prerelease_tag(tag)
    assets = _release_assets(tag, root)
    recommended: dict[str, Any] = {
        "qiongli_cli": {
            "install": "npm_next" if is_next else "npm_latest",
            "command": (
                'npx qiongli@next install --target all --project-dir "$PWD"'
                if is_next
                else 'npx qiongli@latest install --target all --project-dir "$PWD"'
            ),
        },
        "codex": {
            "install": "marketplace",
            "command": "codex plugin marketplace add jxpeng98/skillsplace --ref main",
            "plugin": NEXT_PLUGIN_NAME if is_next else PLUGIN_NAME,
            "manual_asset": None,
        },
        "claude_code": {
            "install": "marketplace",
            "command": "claude plugin marketplace add jxpeng98/skillsplace@main",
            "plugin": NEXT_PLUGIN_NAME if is_next else PLUGIN_NAME,
            "manual_asset": None,
        },
        "claude_desktop_skill": {
            "install": "download_zip",
            "asset_pattern": (
                f"{NEXT_PLUGIN_NAME}-claude-desktop-skill-core-{tag}.zip"
                if is_next
                else f"{PLUGIN_NAME}-claude-desktop-skill-<subject>-{tag}.zip"
            ),
        },
        "claude_desktop_plugin": {
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
    }


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


def render_markdown(index: dict[str, Any]) -> str:
    tag = str(index["tag"])
    channel = str(index["channel"])
    release_url = str(index["release_url"])
    recommended = index["recommended"]
    assets = index["assets"]
    asset_urls = index["asset_urls"]
    desktop_skills = list(assets["claude_desktop_skills"])
    desktop_plugin_asset = str(assets["claude_desktop_plugin"])
    mcpb_asset = str(assets["claude_desktop_literature_mcpb"])
    zotero_asset = str(assets["zotero_desktop_companion"])
    guide_asset = str(assets["download_guide"])
    index_asset = str(assets["download_index"])
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
        f"| Claude Desktop recommended direct plugin ZIP | {_markdown_link(desktop_plugin_asset, desktop_plugin_url)} |",
        f"| Claude Desktop/Web fallback skill ZIP | {_markdown_link(desktop_core_asset, desktop_core_url)} |",
        f"| Claude Desktop literature MCPB | {_markdown_link(mcpb_asset, mcpb_url)} |",
        f"| Zotero Desktop companion XPI | {_markdown_link(zotero_asset, zotero_url)} |",
        f"| Human download guide | {_markdown_link(guide_asset, guide_url)} |",
        f"| Machine-readable download index | {_markdown_link(index_asset, index_url)} |",
        "",
        "## Start here",
        "",
        "| You use | Download or install | Why |",
        "|---|---|---|",
        f"| Qiongli CLI | `{recommended['qiongli_cli']['command']}` | Uses the {cli_channel_label} channel and bundled CLI payload. |",
        "| Codex | Use the marketplace command; do not download a plugin tarball. | Marketplace install keeps skills and bundled literature MCP registration together. |",
        "| Claude Code | Use the marketplace command; do not download a plugin tarball. | Marketplace install keeps slash commands, skills, and bundled literature MCP together. |",
        f"| Claude Desktop | Download `{desktop_plugin_asset}`. | This is the recommended direct plugin package for the unified `qiongli` Desktop entry. |",
        "| Claude Desktop/Web fallback skills | Download exactly one Desktop skill ZIP from the table below. | Use skill ZIPs only when direct plugin install is unavailable or for manual skill upload. |",
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
        "## Machine-readable index",
        "",
        f"- Human guide asset: `{guide_asset}`",
        f"- JSON index asset: `{index_asset}`",
        "",
        "The JSON index groups assets by install surface so scripts do not need to parse GitHub's flat asset list.",
        "",
    ]
    return "\n".join(lines)


def write_outputs(tag: str, out_dir: Path, repo_slug: str, root: Path = REPO_ROOT) -> tuple[Path, Path]:
    tag = _normalize_tag(tag)
    out_dir.mkdir(parents=True, exist_ok=True)
    index = build_index(tag, repo_slug=repo_slug, root=root)
    guide_path = out_dir / f"qiongli-downloads-{tag}.md"
    index_path = out_dir / f"qiongli-downloads-{tag}.json"
    guide_path.write_text(render_markdown(index), encoding="utf-8")
    index_path.write_text(json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return guide_path, index_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Generate Qiongli GitHub release download guide assets.")
    parser.add_argument("--tag", required=True, help="Release tag, for example v1.1.0-beta.2")
    parser.add_argument("--out-dir", type=Path, default=Path("dist"), help="Directory for generated guide assets")
    parser.add_argument("--repo", default=DEFAULT_REPO_SLUG, help="GitHub repo slug used in generated URLs")
    args = parser.parse_args(argv)

    try:
        guide_path, index_path = write_outputs(args.tag, args.out_dir, args.repo)
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"[downloads] {exc}", file=sys.stderr)
        return 1

    print(f"[downloads] generated: {guide_path}")
    print(f"[downloads] generated: {index_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
