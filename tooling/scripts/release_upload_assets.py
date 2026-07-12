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

from generate_release_downloads import build_index  # noqa: E402
from tooling.scripts.release_version import parse_release_version  # noqa: E402


class ReleaseUploadAssetError(RuntimeError):
    pass


def release_upload_asset_names(
    tag: str,
    *,
    root: Path = REPO_ROOT,
    require_existing: bool = True,
    dist_dir: Path | None = None,
) -> list[str]:
    try:
        release_identity = parse_release_version(tag)
    except ValueError as exc:
        raise ReleaseUploadAssetError(str(exc)) from exc
    if release_identity.release_line == "native-2x":
        raise ReleaseUploadAssetError(
            "native 2.x assets must come from a target-identified native manifest; "
            "legacy upload assets are disabled"
        )
    index = build_index(tag, root=root)
    names: list[str] = []
    for target_assets in index.get("assets_by_target", {}).values():
        if isinstance(target_assets, dict):
            for value in target_assets.values():
                _append_asset_value(names, value)
    unique_names = _dedupe(names)
    if require_existing:
        if dist_dir is None:
            raise ReleaseUploadAssetError("dist_dir is required when require_existing is true")
        _require_existing(unique_names, dist_dir)
    return unique_names


def release_upload_asset_paths(
    tag: str,
    *,
    dist_dir: Path,
    root: Path = REPO_ROOT,
    require_existing: bool = True,
) -> list[str]:
    names = release_upload_asset_names(
        tag,
        root=root,
        require_existing=require_existing,
        dist_dir=dist_dir,
    )
    return [str(Path(dist_dir) / name) for name in names]


def _append_asset_value(names: list[str], value: Any) -> None:
    if isinstance(value, str) and value:
        names.append(value)
    elif isinstance(value, list):
        names.extend(item for item in value if isinstance(item, str) and item)


def _dedupe(values: list[str]) -> list[str]:
    seen: set[str] = set()
    deduped: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        deduped.append(value)
    return deduped


def _require_existing(names: list[str], dist_dir: Path) -> None:
    missing = [name for name in names if not (dist_dir / name).is_file()]
    if missing:
        preview = ", ".join(missing[:8])
        if len(missing) > 8:
            preview += f", ... (+{len(missing) - 8} more)"
        raise ReleaseUploadAssetError(f"missing upload assets: {preview}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="List Qiongli release upload asset paths.")
    parser.add_argument("--tag", required=True)
    parser.add_argument("--dist-dir", type=Path, default=Path("dist"))
    parser.add_argument("--root", type=Path, default=REPO_ROOT)
    parser.add_argument("--no-require-existing", action="store_true")
    args = parser.parse_args(argv)
    try:
        paths = release_upload_asset_paths(
            args.tag,
            dist_dir=args.dist_dir,
            root=args.root,
            require_existing=not args.no_require_existing,
        )
    except ReleaseUploadAssetError as exc:
        print(f"[release-upload-assets] {exc}", file=sys.stderr)
        return 1
    for path in paths:
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
