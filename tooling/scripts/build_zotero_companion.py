from __future__ import annotations

import argparse
import json
import shutil
import sys
import zipfile
from pathlib import Path


PACKAGE_RELATIVE = Path("packages/qiongli-zotero-companion")
COMPANION_DISPLAY_NAME = "Qiongli Zotero Companion"
COMPANION_ARTIFACT_SLUG = "qiongli-zotero-companion"
ZOTERO_UPDATE_URL = (
    "https://github.com/jxpeng98/qiongli/releases/latest/download/"
    "qiongli-zotero-companion-updates.json"
)
REQUIRED_FILES = (
    Path("manifest.json"),
    Path("bootstrap.js"),
    Path("README.md"),
    Path("chrome/content/qiongli-bridge.js"),
)
REJECTED_TEXT = ("secret-key", "desktop-secret", "api-key-value", "/Users/", "/private/tmp")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build the Qiongli Zotero companion XPI.")
    parser.add_argument(
        "--dist-dir",
        type=Path,
        default=Path("dist"),
        help="Directory where the .xpi artifact should be written.",
    )
    return parser.parse_args()


def package_root() -> Path:
    return Path(__file__).resolve().parents[2] / PACKAGE_RELATIVE


def read_manifest(root: Path) -> dict[str, object]:
    manifest_path = root / "manifest.json"
    if not manifest_path.is_file():
        raise ValueError(f"Missing required file: {manifest_path}")
    with manifest_path.open(encoding="utf-8") as handle:
        return json.load(handle)


def validate_manifest(manifest: dict[str, object]) -> None:
    if manifest.get("name") != COMPANION_DISPLAY_NAME:
        raise ValueError("manifest.json must use the Qiongli Zotero Companion display name")
    description = manifest.get("description")
    if not isinstance(description, str) or "Zotero 9.0.4" not in description:
        raise ValueError("manifest.json description must mention Zotero 9.0.4 testing")
    if not isinstance(manifest.get("version"), str) or not manifest["version"]:
        raise ValueError("manifest.json must define a string version")

    applications = manifest.get("applications")
    zotero = applications.get("zotero") if isinstance(applications, dict) else None
    if not isinstance(zotero, dict) or not zotero.get("id"):
        raise ValueError("manifest.json must define applications.zotero.id")
    if zotero.get("update_url") != ZOTERO_UPDATE_URL:
        raise ValueError("manifest.json must define applications.zotero.update_url")
    if zotero.get("strict_min_version") != "8.0":
        raise ValueError("manifest.json must set applications.zotero.strict_min_version to 8.0")
    if zotero.get("strict_max_version") != "9.0.*":
        raise ValueError("manifest.json must set applications.zotero.strict_max_version to 9.0.*")


def validate_required_files(root: Path) -> None:
    missing = [path.as_posix() for path in REQUIRED_FILES if not (root / path).is_file()]
    if missing:
        raise ValueError("Missing required files: " + ", ".join(missing))


def iter_package_files(root: Path) -> list[Path]:
    files = [root / path for path in REQUIRED_FILES]
    chrome_dir = root / "chrome"
    if chrome_dir.exists():
        files.extend(path for path in chrome_dir.rglob("*") if path.is_file())
    return sorted(set(files), key=lambda path: path.relative_to(root).as_posix())


def validate_text_payloads(root: Path, files: list[Path]) -> None:
    for path in files:
        try:
            content = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue

        relative = path.relative_to(root).as_posix()
        if "not_implemented" in content:
            raise ValueError(f"Refusing to package unimplemented endpoint in {relative}")
        for rejected in REJECTED_TEXT:
            if rejected in content:
                raise ValueError(f"Refusing to package rejected text {rejected!r} in {relative}")


def artifact_path(dist_dir: Path, manifest: dict[str, object]) -> Path:
    version = manifest["version"]
    return dist_dir / f"{COMPANION_ARTIFACT_SLUG}-{version}.xpi"


def build(root: Path, dist_dir: Path) -> Path:
    manifest = read_manifest(root)
    validate_manifest(manifest)
    validate_required_files(root)

    files = iter_package_files(root)
    validate_text_payloads(root, files)

    dist_dir.mkdir(parents=True, exist_ok=True)
    path = artifact_path(dist_dir, manifest)
    temporary = path.with_suffix(path.suffix + ".tmp")
    if temporary.exists():
        temporary.unlink()

    with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for source in files:
            archive.write(source, source.relative_to(root).as_posix())

    shutil.move(str(temporary), path)
    return path


def main() -> int:
    args = parse_args()
    try:
        path = build(package_root(), args.dist_dir)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
