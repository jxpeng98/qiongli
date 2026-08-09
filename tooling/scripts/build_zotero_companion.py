from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import sys
import zipfile
from pathlib import Path

from release_version import parse_release_version


PACKAGE_RELATIVE = Path("packages/qiongli-zotero-companion")
COMPANION_DISPLAY_NAME = "Qiongli Zotero Companion"
COMPANION_ARTIFACT_SLUG = "qiongli-zotero-companion"
COMPANION_ID = "qiongli-zotero-companion@qiongli.local"
COMPANION_ENDPOINT_VERSION = "2"
COMPANION_MANIFEST_FILE = "qiongli-zotero-companion.manifest.json"
COMPANION_UPDATE_MANIFEST_FILE = "qiongli-zotero-companion-updates.json"
COMPANION_ARTIFACT_SCHEMA_VERSION = 1
DEFAULT_REPO_SLUG = "jxpeng98/qiongli"
COMPANION_VERSION_PATTERN = re.compile(
    r"^[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z]+(?:[.-][0-9A-Za-z]+)*)?$"
)
ZOTERO_MIN_VERSION = "8.0"
ZOTERO_MAX_VERSION = "9.0.*"
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
CONTENT_ROOT_DOMAIN = b"qiongli-zotero-companion-content-root-v1\0"
ZIP_TIMESTAMP = (1980, 1, 1, 0, 0, 0)
ZIP_FLAGS = 0
ZIP_VERSION = 20
ZIP_REGULAR_MODE = 0o100644


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build the Qiongli Zotero companion XPI.")
    parser.add_argument(
        "--dist-dir",
        type=Path,
        default=Path("dist"),
        help="Directory where the .xpi artifact should be written.",
    )
    parser.add_argument(
        "--release-tag",
        help=(
            "Optional Qiongli release tag. When present, also emit the Zotero "
            "automatic-update manifest bound to this immutable release."
        ),
    )
    parser.add_argument(
        "--repo",
        default=DEFAULT_REPO_SLUG,
        help="GitHub owner/repository used for the versioned XPI update link.",
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
    version = manifest.get("version")
    if not isinstance(version, str) or not COMPANION_VERSION_PATTERN.fullmatch(version):
        raise ValueError("manifest.json must define a safe semantic version")

    applications = manifest.get("applications")
    zotero = applications.get("zotero") if isinstance(applications, dict) else None
    if not isinstance(zotero, dict) or zotero.get("id") != COMPANION_ID:
        raise ValueError("manifest.json must define applications.zotero.id")
    if zotero.get("update_url") != ZOTERO_UPDATE_URL:
        raise ValueError("manifest.json must define applications.zotero.update_url")
    if zotero.get("strict_min_version") != ZOTERO_MIN_VERSION:
        raise ValueError("manifest.json must set applications.zotero.strict_min_version to 8.0")
    if zotero.get("strict_max_version") != ZOTERO_MAX_VERSION:
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
            content = path.read_bytes().decode("utf-8")
        except UnicodeDecodeError:
            continue

        relative = path.relative_to(root).as_posix()
        if "\r" in content:
            raise ValueError(
                f"Refusing to package non-LF line endings in {relative}"
            )
        if "not_implemented" in content:
            raise ValueError(f"Refusing to package unimplemented endpoint in {relative}")
        for rejected in REJECTED_TEXT:
            if rejected in content:
                raise ValueError(f"Refusing to package rejected text {rejected!r} in {relative}")


def validate_source_identity(root: Path, manifest: dict[str, object]) -> None:
    version = manifest["version"]
    version_declaration = f'version: "{version}"'
    endpoint_declaration = f'endpoint_version: "{COMPANION_ENDPOINT_VERSION}"'
    for relative in (Path("bootstrap.js"), Path("chrome/content/qiongli-bridge.js")):
        content = (root / relative).read_text(encoding="utf-8")
        if version_declaration not in content:
            raise ValueError(f"{relative.as_posix()} companion version does not match manifest.json")
        if endpoint_declaration not in content:
            raise ValueError(
                f"{relative.as_posix()} endpoint version does not match "
                f"{COMPANION_ENDPOINT_VERSION}"
            )


def artifact_path(dist_dir: Path, manifest: dict[str, object]) -> Path:
    version = manifest["version"]
    return dist_dir / f"{COMPANION_ARTIFACT_SLUG}-{version}.xpi"


def artifact_manifest_path(dist_dir: Path) -> Path:
    return dist_dir / COMPANION_MANIFEST_FILE


def update_manifest_path(dist_dir: Path) -> Path:
    return dist_dir / COMPANION_UPDATE_MANIFEST_FILE


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def source_entry(root: Path, path: Path) -> dict[str, object]:
    payload = (root / path).read_bytes()
    return {
        "path": path.as_posix(),
        "size_bytes": len(payload),
        "sha256": sha256_bytes(payload),
    }


def entry_content_root(entries: list[dict[str, object]]) -> str:
    digest = hashlib.sha256()
    digest.update(CONTENT_ROOT_DOMAIN)
    for entry in entries:
        path = entry["path"]
        size = entry["size_bytes"]
        sha256 = entry["sha256"]
        if not isinstance(path, str) or not isinstance(size, int) or not isinstance(sha256, str):
            raise ValueError("invalid Zotero Companion source entry")
        encoded_path = path.encode("ascii")
        digest.update(len(encoded_path).to_bytes(8, "big"))
        digest.update(encoded_path)
        digest.update(size.to_bytes(8, "big"))
        digest.update(sha256.encode("ascii"))
    return digest.hexdigest()


def canonical_json(value: dict[str, object]) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def validate_repo_slug(value: str) -> str:
    parts = value.split("/")
    if len(parts) != 2 or any(
        not part
        or part in {".", ".."}
        or not all(
            character.isascii()
            and (character.isalnum() or character in "._-")
            for character in part
        )
        for part in parts
    ):
        raise ValueError("GitHub repo must use an owner/repository slug")
    return value


def build_update_manifest(
    artifact_manifest: dict[str, object],
    *,
    release_tag: str,
    repo_slug: str,
) -> dict[str, object]:
    tag = parse_release_version(release_tag).repo_tag
    repo = validate_repo_slug(repo_slug)
    version = artifact_manifest.get("companion_version")
    artifact_file = artifact_manifest.get("artifact_file")
    artifact_sha256 = artifact_manifest.get("artifact_sha256")
    if (
        not isinstance(version, str)
        or not version
        or not isinstance(artifact_file, str)
        or artifact_file != f"{COMPANION_ARTIFACT_SLUG}-{version}.xpi"
        or not isinstance(artifact_sha256, str)
        or len(artifact_sha256) != 64
        or any(character not in "0123456789abcdef" for character in artifact_sha256)
    ):
        raise ValueError("invalid Zotero Companion artifact manifest")
    update_link = f"https://github.com/{repo}/releases/download/{tag}/{artifact_file}"
    return {
        "addons": {
            COMPANION_ID: {
                "updates": [
                    {
                        "version": version,
                        "update_link": update_link,
                        "update_hash": f"sha256:{artifact_sha256}",
                        "applications": {
                            "zotero": {
                                "strict_min_version": ZOTERO_MIN_VERSION,
                                "strict_max_version": ZOTERO_MAX_VERSION,
                            }
                        },
                    }
                ]
            }
        }
    }


def write_deterministic_xpi(root: Path, files: list[Path], path: Path) -> None:
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        archive.comment = b""
        for source in files:
            info = zipfile.ZipInfo(source.as_posix(), ZIP_TIMESTAMP)
            info.compress_type = zipfile.ZIP_STORED
            info.create_system = 3
            info.create_version = ZIP_VERSION
            info.extract_version = ZIP_VERSION
            info.flag_bits = ZIP_FLAGS
            info.external_attr = ZIP_REGULAR_MODE << 16
            info.internal_attr = 0
            info.extra = b""
            info.comment = b""
            archive.writestr(info, (root / source).read_bytes())


def build(
    root: Path,
    dist_dir: Path,
    *,
    release_tag: str | None = None,
    repo_slug: str = DEFAULT_REPO_SLUG,
) -> Path:
    manifest = read_manifest(root)
    validate_manifest(manifest)
    validate_required_files(root)

    absolute_files = iter_package_files(root)
    validate_text_payloads(root, absolute_files)
    validate_source_identity(root, manifest)
    files = [path.relative_to(root) for path in absolute_files]

    dist_dir.mkdir(parents=True, exist_ok=True)
    path = artifact_path(dist_dir, manifest)
    temporary = path.with_suffix(path.suffix + ".tmp")
    if temporary.exists():
        temporary.unlink()

    write_deterministic_xpi(root, files, temporary)

    shutil.move(str(temporary), path)
    xpi_bytes = path.read_bytes()
    entries = [source_entry(root, source) for source in files]
    artifact_manifest = {
        "schema_version": COMPANION_ARTIFACT_SCHEMA_VERSION,
        "record_type": "qiongli-zotero-companion-artifact",
        "status": "assembled-unpublished",
        "companion_id": COMPANION_ID,
        "display_name": COMPANION_DISPLAY_NAME,
        "companion_version": manifest["version"],
        "zotero_min_version": ZOTERO_MIN_VERSION,
        "zotero_max_version": ZOTERO_MAX_VERSION,
        "endpoint_version": COMPANION_ENDPOINT_VERSION,
        "artifact_file": path.name,
        "artifact_size_bytes": len(xpi_bytes),
        "artifact_sha256": sha256_bytes(xpi_bytes),
        "entry_content_root_sha256": entry_content_root(entries),
        "entries": entries,
    }
    artifact_manifest_path(dist_dir).write_bytes(canonical_json(artifact_manifest))
    if release_tag is not None:
        updates = build_update_manifest(
            artifact_manifest,
            release_tag=release_tag,
            repo_slug=repo_slug,
        )
        update_manifest_path(dist_dir).write_bytes(canonical_json(updates))
    return path


def main() -> int:
    args = parse_args()
    try:
        path = build(
            package_root(),
            args.dist_dir,
            release_tag=args.release_tag,
            repo_slug=args.repo,
        )
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    print(path)
    print(artifact_manifest_path(args.dist_dir))
    if args.release_tag is not None:
        print(update_manifest_path(args.dist_dir))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
