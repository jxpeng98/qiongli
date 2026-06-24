#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (Path(__file__).resolve().parent, PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from changelog_section import extract_section
from generate_release_downloads import DEFAULT_REPO_SLUG, build_index


def _normalize_tag(raw: str) -> str:
    tag = raw.strip()
    if not tag:
        raise ValueError("tag is required")
    return tag if tag.startswith("v") else f"v{tag}"


def _stable_version(tag: str) -> str:
    version = tag.removeprefix("v")
    if "-" in version or "beta" in version:
        raise ValueError(f"stable release notes require a stable tag, got {tag}")
    return version


def _asset_link(label: str, url: str) -> str:
    return f"[`{label}`]({url})"


def _required_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"download index missing {label}")
    return value


def _required_list(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or not value:
        raise ValueError(f"download index missing {label}")
    if not all(isinstance(item, str) and item for item in value):
        raise ValueError(f"download index has invalid {label}")
    return value


def _demote_changelog_heading(section: str) -> str:
    lines = [
        f"#{line}" if line.startswith("#") else line
        for line in section.splitlines()
    ]
    return "\n".join(lines).strip() + "\n"


def render_notes(tag: str, changelog_path: Path, repo_slug: str = DEFAULT_REPO_SLUG) -> str:
    tag = _normalize_tag(tag)
    version = _stable_version(tag)
    content = changelog_path.read_text(encoding="utf-8")
    changelog_section = extract_section(content, version)
    if changelog_section is None:
        raise ValueError(f"missing changelog section: {version}")

    index = build_index(tag, repo_slug=repo_slug)
    release_url = _required_string(index.get("release_url"), "release_url")
    assets = index["assets"]
    asset_urls = index["asset_urls"]

    desktop_skills = _required_list(assets.get("claude_desktop_skills"), "desktop skill assets")
    desktop_urls = _required_list(asset_urls.get("claude_desktop_skills"), "desktop skill URLs")
    desktop_url_by_asset = dict(zip(desktop_skills, desktop_urls))
    desktop_core_asset = f"qiongli-claude-desktop-skill-core-{tag}.zip"
    if desktop_core_asset not in desktop_url_by_asset:
        desktop_core_asset = desktop_skills[0]

    mcpb_asset = _required_string(assets.get("claude_desktop_literature_mcpb"), "MCPB asset")
    zotero_asset = _required_string(assets.get("zotero_desktop_companion"), "Zotero companion asset")
    guide_asset = _required_string(assets.get("download_guide"), "download guide asset")
    index_asset = _required_string(assets.get("download_index"), "download index asset")

    mcpb_url = _required_string(asset_urls.get("claude_desktop_literature_mcpb"), "MCPB URL")
    zotero_url = _required_string(asset_urls.get("zotero_desktop_companion"), "Zotero companion URL")
    guide_url = _required_string(asset_urls.get("download_guide"), "download guide URL")
    index_url = _required_string(asset_urls.get("download_index"), "download index URL")

    changelog = _demote_changelog_heading(changelog_section)

    lines = [
        "## Release Category",
        "",
        f"`{tag}` is the stable release for normal installs and upgrades. Use it for npm `latest`, PyPI stable, the `qiongli` marketplace entry, Claude Desktop/Web skill ZIPs, the literature MCPB, and the Zotero companion XPI.",
        "",
        "| Category | Use it for | Notes |",
        "|---|---|---|",
        "| Stable packages | npm `latest`, PyPI stable, and normal CLI upgrades | This is the default path for most users. |",
        "| Client workflow assets | Codex, Claude Code, Claude Desktop/Web, Antigravity, and Hermes workflow installs | Marketplace installs are preferred where supported; Desktop/Web users should download a skill ZIP. |",
        "| Local literature tools | Claude Desktop MCPB and bundled plugin MCP runtimes | Install the MCPB only when local provider/search tools are required. |",
        "| Zotero companion | Local Zotero Desktop search/write workflows | Install the XPI only when you want Qiongli to talk to Zotero Desktop. |",
        "| Maintainer artifacts | Codex/Claude plugin tarballs and plugin ZIPs | These are for release checks and direct upload tests, not the normal user path. |",
        "| Prerelease channel | `qiongli-next`, npm `next`, and PyPI beta | This channel remains for validation builds and does not need to be newer than the stable tag. |",
        "",
        "## Download Guide",
        "",
        "Start here instead of scanning GitHub's flat asset list.",
        "",
        "| Need | Link or command |",
        "|---|---|",
        f"| Release page and all assets | [Qiongli {tag}]({release_url}) |",
        f"| npm CLI | `npm install -g qiongli@latest` or `npx qiongli@latest install --target all --project-dir \"$PWD\"` |",
        f"| PyPI CLI | `pipx install qiongli` or `pipx upgrade qiongli` |",
        f"| Default Claude Desktop/Web skill ZIP | {_asset_link(desktop_core_asset, desktop_url_by_asset[desktop_core_asset])} |",
        f"| All Desktop/Web subject ZIPs | {_asset_link(guide_asset, guide_url)} |",
        f"| Claude Desktop literature MCPB | {_asset_link(mcpb_asset, mcpb_url)} |",
        f"| Zotero Desktop companion XPI | {_asset_link(zotero_asset, zotero_url)} |",
        f"| Machine-readable download index | {_asset_link(index_asset, index_url)} |",
        "",
        "## Changelog",
        "",
        changelog.rstrip(),
        "",
    ]
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Generate stable Qiongli release notes.")
    parser.add_argument("--tag", required=True, help="Stable release tag, for example v1.5.0")
    parser.add_argument("--repo", default=DEFAULT_REPO_SLUG, help="GitHub repo slug used in release links")
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"), help="Changelog path")
    parser.add_argument("--output", type=Path, required=True, help="Output Markdown path")
    args = parser.parse_args(argv)

    try:
        notes = render_notes(args.tag, args.changelog, repo_slug=args.repo)
    except (OSError, ValueError) as exc:
        print(f"[stable-notes] {exc}", file=sys.stderr)
        return 1

    args.output.write_text(notes, encoding="utf-8")
    print(f"[stable-notes] generated: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
