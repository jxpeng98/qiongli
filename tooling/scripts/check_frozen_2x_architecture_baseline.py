#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path
from typing import Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
FROZEN_ARCHITECTURE_ANCHOR = "tooling/architecture/arc-201-decisions.json"
FROZEN_PATHS = frozenset(
    {
        FROZEN_ARCHITECTURE_ANCHOR,
        "docs/architecture/decisions/0201-executable-topology.md",
        "docs/architecture/decisions/0202-rust-native-ui-and-accessibility.md",
        "docs/architecture/decisions/0203-agent-backend-and-tool-host.md",
        "docs/architecture/decisions/0204-versioned-state-and-secret-storage.md",
        "docs/architecture/decisions/0205-deterministic-resource-pack.md",
        "docs/architecture/decisions/0206-declarative-install-plan-and-client-trust.md",
        "docs/architecture/decisions/0207-release-channel-and-artifact-identity.md",
        "tooling/migration/2x-branch-point.json",
        "tooling/migration/2x-branch-point.schema.json",
    }
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


def base_contains_architecture_anchor(repo_root: Path, base_ref: str) -> bool:
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
            FROZEN_ARCHITECTURE_ANCHOR,
        ],
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise GuardError(f"cannot inspect comparison base {base_ref}: {detail}")
    paths = [value.decode("utf-8") for value in result.stdout.split(b"\0") if value]
    return paths == [FROZEN_ARCHITECTURE_ANCHOR]


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


def frozen_changes(
    changed_paths: Sequence[str], *, base_has_architecture_anchor: bool
) -> list[str]:
    if not base_has_architecture_anchor:
        return []
    return sorted({path for path in changed_paths if path in FROZEN_PATHS})


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Reject rewrites of the accepted Qiongli 2.x architecture baseline."
    )
    parser.add_argument(
        "--base-ref",
        required=True,
        help="Event-aware comparison base resolved by CI.",
    )
    args = parser.parse_args(argv)
    try:
        base_has_anchor = base_contains_architecture_anchor(REPO_ROOT, args.base_ref)
        changed_paths = changed_paths_from_git(REPO_ROOT, args.base_ref)
    except GuardError as error:
        print(f"[2x-architecture-guard] {error}", file=sys.stderr)
        return 2

    changes = frozen_changes(
        changed_paths, base_has_architecture_anchor=base_has_anchor
    )
    if not base_has_anchor:
        print(
            "[2x-architecture-guard] comparison base predates ARC-201; "
            "allowing the one-time B0 bootstrap"
        )
        return 0
    if not changes:
        print("[2x-architecture-guard] accepted B0 evidence is unchanged")
        return 0

    print(
        "The accepted Qiongli 2.x B0 architecture and handoff evidence is frozen.\n"
        "Create a superseding ADR instead of rewriting accepted evidence:",
        file=sys.stderr,
    )
    for path in changes:
        print(f"  - {path}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
