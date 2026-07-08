#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path


TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
)
PACKAGE_RELATIVE = Path("packages") / "qiongli-lite-mcp"
BIN_BASENAME = "qiongli-literature-provider"


def _manifest_path(root: Path) -> Path:
    return root / PACKAGE_RELATIVE / "Cargo.toml"


def _target_dir(root: Path) -> Path:
    return root / PACKAGE_RELATIVE / "target"


def _binary_name(*, windows: bool = False) -> str:
    return f"{BIN_BASENAME}.exe" if windows else BIN_BASENAME


def _run_cargo(root: Path, args: list[str]) -> None:
    subprocess.run(args, cwd=root, check=True)


def _stage_binary(source: Path, out_dir: Path, *, windows: bool = False) -> Path:
    if not source.is_file():
        raise FileNotFoundError(f"Lite MCP binary was not built: {source}")
    out_dir.mkdir(parents=True, exist_ok=True)
    staged = out_dir / _binary_name(windows=windows)
    shutil.copy2(source, staged)
    if not windows:
        staged.chmod(staged.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return staged


def build_current_platform(root: Path, out_dir: Path) -> Path:
    root = root.resolve()
    manifest = _manifest_path(root)
    _run_cargo(
        root,
        [
            "cargo",
            "build",
            "--release",
            "--manifest-path",
            str(manifest),
        ],
    )
    windows = os.name == "nt"
    built = _target_dir(root) / "release" / _binary_name(windows=windows)
    return _stage_binary(built, out_dir.resolve(), windows=windows)


def build_target(root: Path, out_dir: Path, target: str) -> Path:
    if target not in TARGETS:
        raise ValueError(f"unsupported Lite MCP target: {target}")
    root = root.resolve()
    manifest = _manifest_path(root)
    _run_cargo(
        root,
        [
            "cargo",
            "build",
            "--release",
            "--target",
            target,
            "--manifest-path",
            str(manifest),
        ],
    )
    windows = target.endswith("windows-msvc")
    built = _target_dir(root) / target / "release" / _binary_name(windows=windows)
    return _stage_binary(built, out_dir.resolve() / target, windows=windows)


def build_all_platforms(root: Path, out_dir: Path) -> list[Path]:
    return [build_target(root, out_dir, target) for target in TARGETS]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build and stage Qiongli Lite MCP binaries.")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--out-dir", type=Path, default=Path("dist") / "lite-mcp")
    parser.add_argument("--target", choices=(*TARGETS, "current", "all"), default="current")
    args = parser.parse_args(argv)

    if args.target == "current":
        artifacts = [build_current_platform(args.root, args.out_dir)]
    elif args.target == "all":
        artifacts = build_all_platforms(args.root, args.out_dir)
    else:
        artifacts = [build_target(args.root, args.out_dir, args.target)]

    for artifact in artifacts:
        print(artifact)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
