#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path


RESOURCE_LINK_PATTERN = re.compile(
    r"`((?:references|templates|workflows|skills|standards|roles|venue-profiles)/[^`\n]+?)`"
)
IGNORED_MARKERS = ("<", ">", "*", "[", "]", "...")


@dataclass(frozen=True)
class MissingResourceLink:
    source: str
    target: str


def _is_literal_resource_path(path: str) -> bool:
    if path.endswith("/"):
        return False
    return not any(marker in path for marker in IGNORED_MARKERS)


def audit_package_resource_links(package_dir: Path) -> list[MissingResourceLink]:
    package_dir = package_dir.resolve()
    missing: list[MissingResourceLink] = []
    for markdown_path in sorted(package_dir.rglob("*.md")):
        if any(part in {".git", "node_modules", "__pycache__"} for part in markdown_path.parts):
            continue
        content = markdown_path.read_text(encoding="utf-8")
        for match in RESOURCE_LINK_PATTERN.finditer(content):
            target = match.group(1).strip()
            if not _is_literal_resource_path(target):
                continue
            if not (package_dir / target).exists():
                missing.append(
                    MissingResourceLink(
                        source=str(markdown_path.relative_to(package_dir)),
                        target=target,
                    )
                )
    return missing


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit qiongli skill package markdown resource links."
    )
    parser.add_argument("package_dir", type=Path)
    args = parser.parse_args()

    missing = audit_package_resource_links(args.package_dir)
    for item in missing:
        print(f"[FAIL] {item.source} -> {item.target}")
    if missing:
        print(f"Missing resource links: {len(missing)}")
        return 1
    print("[PASS] All resource links resolve")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
