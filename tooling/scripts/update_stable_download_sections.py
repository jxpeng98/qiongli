#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (Path(__file__).resolve().parent, PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from generate_release_downloads import DEFAULT_REPO_SLUG, build_index


ENGLISH_TARGETS = (
    Path("README.md"),
    Path("docs/index.md"),
    Path("docs/guide/install.md"),
)
CHINESE_TARGETS = (
    Path("README_CN.md"),
    Path("docs/zh/index.md"),
    Path("docs/zh/guide/install.md"),
)
ENGLISH_HEADING = "## Latest Stable Downloads"
CHINESE_HEADING = "## 最新稳定版下载"


def _required_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"missing {label}")
    return value


def _required_url(index: dict[str, Any], key: str) -> str:
    asset_urls = index.get("asset_urls")
    if not isinstance(asset_urls, dict):
        raise ValueError("release index is missing asset URLs")
    return _required_string(asset_urls.get(key), f"{key} URL")


def _required_asset(index: dict[str, Any], key: str) -> str:
    assets = index.get("assets")
    if not isinstance(assets, dict):
        raise ValueError("release index is missing assets")
    return _required_string(assets.get(key), f"{key} asset")


def _desktop_core_asset(index: dict[str, Any]) -> tuple[str, str]:
    tag = _required_string(index.get("tag"), "tag")
    expected = f"qiongli-claude-desktop-skill-core-{tag}.zip"
    assets = index.get("assets")
    asset_urls = index.get("asset_urls")
    if not isinstance(assets, dict) or not isinstance(asset_urls, dict):
        raise ValueError("release index is missing desktop skill assets")
    desktop_assets = assets.get("claude_desktop_skills")
    desktop_urls = asset_urls.get("claude_desktop_skills")
    if not isinstance(desktop_assets, list) or not isinstance(desktop_urls, list):
        raise ValueError("release index is missing desktop skill URLs")
    if expected not in desktop_assets:
        raise ValueError(f"release index is missing {expected}")
    return expected, _required_string(desktop_urls[desktop_assets.index(expected)], f"{expected} URL")


def _render_block(index: dict[str, Any], *, language: str) -> str:
    tag = _required_string(index.get("tag"), "tag")
    version = tag.removeprefix("v")
    release_url = _required_string(index.get("release_url"), "release URL")
    desktop_asset, desktop_url = _desktop_core_asset(index)
    desktop_plugin_asset = _required_asset(index, "claude_desktop_plugin")
    desktop_plugin_url = _required_url(index, "claude_desktop_plugin")
    mcpb_asset = _required_asset(index, "claude_desktop_literature_mcpb")
    mcpb_url = _required_url(index, "claude_desktop_literature_mcpb")
    zotero_asset = _required_asset(index, "zotero_desktop_companion")
    zotero_url = _required_url(index, "zotero_desktop_companion")
    guide_asset = _required_asset(index, "download_guide")
    guide_url = _required_url(index, "download_guide")
    npm_url = f"https://www.npmjs.com/package/qiongli/v/{version}"
    pypi_url = f"https://pypi.org/project/qiongli/{version}/"

    if language == "zh":
        return "\n".join(
            [
                CHINESE_HEADING,
                "",
                f"当前稳定版是 [{tag}]({release_url})。下面这些直达链接覆盖常见安装路径；需要 subject 专精 Desktop ZIP 或维护者 artifacts 时，再打开下载指南。",
                "",
                "| 需求 | 链接或命令 |",
                "|---|---|",
                f"| npm CLI | [`qiongli@{version}`]({npm_url})：`npm install -g qiongli@latest` |",
                f"| PyPI CLI | [`qiongli {version}`]({pypi_url})：`pipx install qiongli` |",
                f"| Claude Desktop direct plugin | [`{desktop_plugin_asset}`]({desktop_plugin_url}) |",
                f"| Claude Desktop/Web core skill | [`{desktop_asset}`]({desktop_url}) |",
                f"| Claude Desktop literature MCPB | [`{mcpb_asset}`]({mcpb_url}) |",
                f"| Zotero Desktop companion | [`{zotero_asset}`]({zotero_url}) |",
                f"| 全部 release assets | [下载指南]({guide_url}) 和 [GitHub Release]({release_url}) |",
            ]
        )

    return "\n".join(
        [
            ENGLISH_HEADING,
            "",
            f"Current stable release: [{tag}]({release_url}). These direct links cover the common install paths; use the download guide for subject-specific Desktop ZIPs and maintainer artifacts.",
            "",
            "| Need | Link or command |",
            "|---|---|",
            f"| npm CLI | [`qiongli@{version}`]({npm_url}): `npm install -g qiongli@latest` |",
            f"| PyPI CLI | [`qiongli {version}`]({pypi_url}): `pipx install qiongli` |",
            f"| Claude Desktop direct plugin | [`{desktop_plugin_asset}`]({desktop_plugin_url}) |",
            f"| Claude Desktop/Web core skill | [`{desktop_asset}`]({desktop_url}) |",
            f"| Claude Desktop literature MCPB | [`{mcpb_asset}`]({mcpb_url}) |",
            f"| Zotero Desktop companion | [`{zotero_asset}`]({zotero_url}) |",
            f"| All release assets | [Download guide]({guide_url}) and [GitHub Release]({release_url}) |",
        ]
    )


def _replace_section(content: str, heading: str, replacement: str, relative_path: Path) -> str:
    pattern = re.compile(rf"^{re.escape(heading)}\n\n.*?(?=^## |\Z)", re.MULTILINE | re.DOTALL)
    matches = list(pattern.finditer(content))
    if len(matches) != 1:
        raise ValueError(f"{relative_path} must contain exactly one {heading!r} section")
    match = matches[0]
    return content[: match.start()] + replacement.rstrip() + "\n\n" + content[match.end() :].lstrip("\n")


def update_stable_download_sections(
    *,
    tag: str,
    root: Path,
    asset_root: Path,
    repo_slug: str,
) -> list[Path]:
    index = build_index(tag, repo_slug=repo_slug, root=asset_root)
    if index.get("channel") != "stable":
        raise ValueError(f"stable download sections require a stable tag, got {index.get('tag')}")

    updated: list[Path] = []
    for language, heading, targets in (
        ("en", ENGLISH_HEADING, ENGLISH_TARGETS),
        ("zh", CHINESE_HEADING, CHINESE_TARGETS),
    ):
        replacement = _render_block(index, language=language)
        for relative_path in targets:
            path = root / relative_path
            content = path.read_text(encoding="utf-8")
            next_content = _replace_section(content, heading, replacement, relative_path)
            path.write_text(next_content, encoding="utf-8")
            updated.append(relative_path)
    return updated


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Update stable download sections in README and docs files.")
    parser.add_argument("--tag", required=True, help="Stable release tag, for example v1.6.0")
    parser.add_argument("--root", type=Path, default=REPO_ROOT, help="Repository/docs root to update")
    parser.add_argument("--asset-root", type=Path, default=REPO_ROOT, help="Repository root used for release asset metadata")
    parser.add_argument("--repo", default=DEFAULT_REPO_SLUG, help="GitHub repo slug used in generated URLs")
    args = parser.parse_args(argv)

    try:
        updated = update_stable_download_sections(
            tag=args.tag,
            root=args.root,
            asset_root=args.asset_root,
            repo_slug=args.repo,
        )
    except (OSError, ValueError) as exc:
        print(f"[stable-downloads] {exc}", file=sys.stderr)
        return 1

    print(f"[stable-downloads] updated stable download sections: {len(updated)} files")
    for relative_path in updated:
        print(f"[stable-downloads] updated: {relative_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
