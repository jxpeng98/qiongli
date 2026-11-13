#!/usr/bin/env python3
"""Check current installation pages against a published release; never write or publish."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from urllib.error import URLError
from urllib.request import Request, urlopen

try:
    from .native_registry_packages import TARGETS
    from .release_version import parse_release_version
except ImportError:
    from native_registry_packages import TARGETS
    from release_version import parse_release_version


ROOT = Path(__file__).resolve().parents[2]
REPOSITORY = "https://github.com/jxpeng98/qiongli"
LATEST = "https://api.github.com/repos/jxpeng98/qiongli/releases/latest"
PAGES = (
    "README.md", "README_CN.md", "docs/index.md", "docs/zh/index.md",
    "docs/guide/install.md", "docs/zh/guide/install.md",
)
PLATFORM_ROWS = {
    "aarch64-apple-darwin": "| macOS Apple Silicon",
    "x86_64-pc-windows-msvc": "| Windows x64",
    "x86_64-unknown-linux-gnu": "| Linux x64",
    "aarch64-unknown-linux-gnu": "| Linux ARM64",
}
# These six current-entry pages describe native 2.x, not historical releases.
VERSION = re.compile(r"(?<![\w.])v?(2\.\d+\.\d+(?:-[a-z]+\.\d+)?)(?![\w.])")
MAX_METADATA_BYTES = 2 * 1024 * 1024


def fetch(url: str) -> bytes:
    request = Request(url, headers={"User-Agent": "qiongli-install-docs-check"})
    with urlopen(request, timeout=20) as response:
        data = response.read(MAX_METADATA_BYTES + 1)
    if len(data) > MAX_METADATA_BYTES:
        raise ValueError("published release metadata exceeds the read limit")
    return data


def release_assets(release: dict) -> tuple[str, dict]:
    if not isinstance(release, dict) or not isinstance(release.get("assets"), list):
        raise ValueError("expected GitHub release metadata with an asset list")
    identity = parse_release_version(release["tag_name"])
    if (identity.release_line != "native-2x" or identity.is_prerelease
            or release["tag_name"] != identity.repo_tag
            or release.get("draft") is not False or release.get("prerelease") is not False
            or not release.get("published_at")
            or release.get("html_url") != f"{REPOSITORY}/releases/tag/{identity.repo_tag}"):
        raise ValueError("installation pages require a published native stable release")
    assets = {}
    for asset in release["assets"]:
        if not isinstance(asset, dict) or not isinstance(asset.get("name"), str):
            raise ValueError("invalid published asset metadata")
        name = asset["name"]
        if name in assets:
            raise ValueError(f"duplicate published asset: {name}")
        assets[name] = asset
    names = {"SHA256SUMS"} | {
        f"qiongli-{identity.version}-{target}.{'zip' if target.endswith('msvc') else 'tar.gz'}"
        for target in TARGETS
    }
    for name in names:
        asset = assets.get(name, {})
        if (type(asset.get("size")) is not int or asset["size"] <= 0
                or asset.get("browser_download_url") !=
                f"{REPOSITORY}/releases/download/{identity.repo_tag}/{name}"):
            raise ValueError(f"missing or invalid published asset: {name}")
        if not re.fullmatch(r"sha256:[0-9a-f]{64}", asset.get("digest", "")):
            raise ValueError(f"missing published SHA-256 identity: {name}")
    return identity.version, {name: assets[name] for name in names}


def check_docs(root: Path, release: dict, checksums: bytes) -> str:
    version, assets = release_assets(release)
    if assets["SHA256SUMS"]["digest"] != "sha256:" + hashlib.sha256(checksums).hexdigest():
        raise ValueError("SHA256SUMS does not match its published digest")
    listed = {}
    for line in checksums.decode("utf-8").splitlines():
        match = re.fullmatch(r"([0-9a-f]{64})  ([^/\\]+)", line)
        if not match or match[2] in listed:
            raise ValueError("malformed or duplicate SHA256SUMS entry")
        listed[match[2]] = match[1]
    for name, asset in assets.items():
        if name != "SHA256SUMS" and asset["digest"] != "sha256:" + listed.get(name, ""):
            raise ValueError(f"archive is missing or mismatched in SHA256SUMS: {name}")

    notes = f"tooling/release/v{version}.md"
    if not (root / notes).is_file():
        raise ValueError(f"missing release notes: {notes}")
    expected_urls = {asset["browser_download_url"] for asset in assets.values()}
    errors = []
    for name in PAGES:
        text = (root / name).read_text(encoding="utf-8")
        versions = set(VERSION.findall(text))
        if versions != {version}:
            errors.append(f"{name}: expected published {version}, found {sorted(versions)}")
        if name.startswith("README"):
            for expected in (
                f'href="{notes}"', f"{REPOSITORY}/releases/tag/v{version}",
                f"npm install --global qiongli@{version}",
            ):
                if expected not in text:
                    errors.append(f"{name}: missing {expected}")
        if name.endswith("/install.md"):
            urls = set(re.findall(r"https://github\.com/[^\s)\"<>]+/releases/download/[^\s)\"<>]+", text))
            if urls != expected_urls:
                errors.append(f"{name}: downloads must cover exactly the published CLI targets and SHA256SUMS")
            for target in TARGETS:
                prefix = PLATFORM_ROWS[target]
                rows = [line for line in text.splitlines() if line.startswith(prefix)]
                filename = f"qiongli-{version}-{target}.{'zip' if target.endswith('msvc') else 'tar.gz'}"
                if len(rows) != 1 or assets[filename]["browser_download_url"] not in rows[0]:
                    errors.append(f"{name}: incorrect download for {target}")
            for command in (
                f"npm install --global qiongli@{version}",
                f'python -m pip install --upgrade "qiongli=={version}"',
                f"cargo install qiongli --version {version} --locked",
            ):
                if f"`{command}`" not in text:
                    errors.append(f"{name}: missing pinned install command: {command}")
    if errors:
        raise ValueError("\n".join(errors))
    return version


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--release-json", type=Path, help="Previously downloaded GitHub release API JSON")
    parser.add_argument("--checksums", type=Path, help="That release's original SHA256SUMS bytes; requires --release-json")
    args = parser.parse_args(argv)
    if args.checksums and not args.release_json:
        parser.error("--checksums requires --release-json for offline checking")
    try:
        release = json.loads(args.release_json.read_bytes() if args.release_json else fetch(LATEST))
        _, assets = release_assets(release)
        checksums = (args.checksums.read_bytes() if args.checksums else
                     fetch(assets["SHA256SUMS"]["browser_download_url"]))
        version = check_docs(args.root, release, checksums)
    except (OSError, URLError, ValueError, KeyError, TypeError) as error:
        print(f"[install-docs] {error}", file=sys.stderr)
        return 1
    scope = "supplied release snapshot" if args.release_json else "latest published release"
    print(f"[install-docs] {len(PAGES)} pages match {scope} v{version}; "
          "asset metadata and checksum coverage checked, binaries and registry installs not tested")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
