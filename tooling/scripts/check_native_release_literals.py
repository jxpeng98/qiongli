#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ALLOWLIST = ROOT / "tooling/release/native-alpha1-historical-allowlist.txt"
SCAN_ROOTS = (
    ROOT / ".github/workflows",
    ROOT / "packages/qiongli-native/apps/qiongli/examples",
    ROOT / "tooling/scripts",
)
STALE = re.compile(r"2\.0\.0-alpha\.1|alpha1|Alpha\.1")


def read_allowlist() -> set[Path]:
    entries: set[Path] = set()
    for raw in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        value = raw.strip()
        if not value or value.startswith("#"):
            continue
        path = ROOT / value
        if path in entries:
            raise ValueError(f"duplicate allowlist entry: {value}")
        if not path.is_file():
            raise ValueError(f"missing allowlisted historical fixture: {value}")
        if "native_alpha1_" not in path.name:
            raise ValueError(f"non-historical file may not be allowlisted: {value}")
        entries.add(path)
    return entries


def main() -> int:
    try:
        allowlist = read_allowlist()
    except (OSError, ValueError) as exc:
        print(f"[native-release-literals] {exc}", file=sys.stderr)
        return 2

    found_allowlisted: set[Path] = set()
    violations: list[str] = []
    for scan_root in SCAN_ROOTS:
        for path in sorted(candidate for candidate in scan_root.rglob("*") if candidate.is_file()):
            if path == Path(__file__).resolve():
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            matches = list(STALE.finditer(text))
            if not matches:
                continue
            if path in allowlist:
                found_allowlisted.add(path)
                continue
            for match in matches:
                line = text.count("\n", 0, match.start()) + 1
                violations.append(f"{path.relative_to(ROOT)}:{line}:{match.group(0)}")

    missing_evidence = allowlist - found_allowlisted
    for path in sorted(missing_evidence):
        violations.append(
            f"{path.relative_to(ROOT)}: allowlist entry no longer contains an Alpha.1 literal"
        )
    if violations:
        print("[native-release-literals] stale active release literals found:", file=sys.stderr)
        for violation in violations:
            print(f"  - {violation}", file=sys.stderr)
        return 1

    print(
        "[native-release-literals] active release paths are version-generic; "
        f"historical fixtures={len(allowlist)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
