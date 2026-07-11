#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path
from typing import Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
FROZEN_BASELINE_ANCHOR = (
    "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
)
FROZEN_PATHS = frozenset(
    {
        "tooling/migration/baseline-manifest.schema.json",
        "tooling/migration/baseline-plan.schema.json",
        "tooling/migration/oracle-fixture.schema.json",
        "tooling/migration/qiongli-1x-baseline-plan.json",
    }
)
FROZEN_PREFIXES = (
    "tooling/migration/baselines/v1.19.0-beta.1/",
)


class GuardError(RuntimeError):
    pass


def _git_environment() -> dict[str, str]:
    environment = os.environ.copy()
    for name in (
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_COMMON_DIR",
        "GIT_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_WORK_TREE",
    ):
        environment.pop(name, None)
    return environment


def _git(repo_root: Path, arguments: Sequence[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["git", "-C", str(repo_root), *arguments],
        check=False,
        env=_git_environment(),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def _require_base_commit(repo_root: Path, base_ref: str) -> None:
    result = _git(repo_root, ["rev-parse", "--verify", f"{base_ref}^{{commit}}"])
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise GuardError(f"comparison base is not a commit: {base_ref}: {detail}")


def base_contains_frozen_anchor(repo_root: Path, base_ref: str) -> bool:
    _require_base_commit(repo_root, base_ref)
    result = _git(
        repo_root,
        [
            "ls-tree",
            "--name-only",
            "--full-tree",
            "-z",
            base_ref,
            "--",
            FROZEN_BASELINE_ANCHOR,
        ],
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise GuardError(f"cannot inspect comparison base {base_ref}: {detail}")
    paths = [value.decode("utf-8") for value in result.stdout.split(b"\0") if value]
    return paths == [FROZEN_BASELINE_ANCHOR]


def changed_paths_from_git(repo_root: Path, base_ref: str) -> list[str]:
    _require_base_commit(repo_root, base_ref)
    result = _git(
        repo_root,
        ["diff", "--name-only", "--no-renames", "-z", f"{base_ref}...HEAD"],
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise GuardError(f"cannot compare {base_ref} with HEAD: {detail}")
    return sorted(
        value.decode("utf-8") for value in result.stdout.split(b"\0") if value
    )


def is_frozen_path(path: str) -> bool:
    return path in FROZEN_PATHS or any(
        path.startswith(prefix) for prefix in FROZEN_PREFIXES
    )


def frozen_changes(
    changed_paths: Sequence[str], *, base_has_frozen_anchor: bool
) -> list[str]:
    if not base_has_frozen_anchor:
        return []
    return sorted({path for path in changed_paths if is_frozen_path(path)})


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Reject changes to the frozen Qiongli 1.x migration baseline."
    )
    parser.add_argument(
        "--base-ref",
        required=True,
        help="Event-aware comparison base resolved by CI.",
    )
    args = parser.parse_args(argv)

    try:
        base_has_anchor = base_contains_frozen_anchor(REPO_ROOT, args.base_ref)
        changed_paths = changed_paths_from_git(REPO_ROOT, args.base_ref)
    except GuardError as error:
        print(f"[frozen-baseline-guard] {error}", file=sys.stderr)
        return 2

    changes = frozen_changes(
        changed_paths, base_has_frozen_anchor=base_has_anchor
    )
    if not base_has_anchor:
        print(
            "[frozen-baseline-guard] comparison base predates the frozen manifest; "
            "allowing the one-time A8/2.x bootstrap"
        )
        return 0
    if not changes:
        print("[frozen-baseline-guard] frozen 1.x migration baseline is unchanged")
        return 0

    print(
        "The Qiongli 1.x migration baseline is frozen at the A8 branch point.\n"
        "Create new 2.x conformance evidence instead of rewriting this oracle:",
        file=sys.stderr,
    )
    for path in changes:
        print(f"  - {path}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
