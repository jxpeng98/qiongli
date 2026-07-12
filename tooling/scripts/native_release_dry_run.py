#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import sys
import tomllib
from typing import Any, Mapping, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
for _import_root in (Path(__file__).resolve().parent, REPO_ROOT):
    if str(_import_root) not in sys.path:
        sys.path.insert(0, str(_import_root))

from release_version import ReleaseIdentity, parse_release_version  # noqa: E402
from validate_capability_contract import validate_instance  # noqa: E402


SCHEMA_RELATIVE = "tooling/release/native-release-plan.schema.json"
SCHEMA_REFERENCE = "https://qiongli.dev/schemas/native-release-plan-v1.json"
NATIVE_MANIFEST_RELATIVE = "packages/qiongli-native/Cargo.toml"
NATIVE_LOCK_RELATIVE = "packages/qiongli-native/Cargo.lock"
VERSION_SOURCE = f"{NATIVE_MANIFEST_RELATIVE}#workspace.package.version"
CHANNEL_SOURCE = f"{NATIVE_MANIFEST_RELATIVE}#workspace.metadata.qiongli.channel"
SOURCE_BRANCH = "2.x"
SCHEMA_VERSION = "1.0"
RECORD_TYPE = "qiongli-native-release-dry-run-plan"
CANONICALIZATION = "utf-8-json-sorted-keys-compact-excluding-integrity"
SOURCE_COMMIT_PATTERN = re.compile(r"^(?:[0-9a-f]{40}|[0-9a-f]{64})$")
SOURCE_REF_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._/-]{0,254}$")

_OS_ALIASES = {
    "darwin": "macos",
    "mac": "macos",
    "macos": "macos",
    "linux": "linux",
    "win32": "windows",
    "windows": "windows",
}
_ARCH_ALIASES = {
    "aarch64": "aarch64",
    "arm64": "aarch64",
    "amd64": "x86_64",
    "x64": "x86_64",
    "x86-64": "x86_64",
    "x86_64": "x86_64",
}
_FORBIDDEN_TARGET_TOKENS = frozenset(
    {"*", "all", "any", "auto", "current", "current-host", "host", "unknown"}
)
_FUTURE_BLOCKERS = (
    "native target artifact builder is not implemented",
    "target-native install and startup receipt is not available",
    "checksum, signature, SBOM, and provenance evidence is not available",
    "signed channel metadata and updater rollback evidence is not available",
    "marketplace target selection is not proven truthful for native payloads",
)


class DryRunError(RuntimeError):
    """The requested native dry run is invalid or unavailable."""


class SourceMismatch(DryRunError):
    """The native release source does not match the requested identity."""


class CliUsageError(DryRunError):
    """The public CLI arguments are invalid and must be reported without echoing them."""


class _RedactedArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:  # pragma: no cover - exercised through main
        del message
        raise CliUsageError("invalid command usage")


def _identity_field(identity: ReleaseIdentity, field: str) -> Any:
    if not hasattr(identity, field):
        raise DryRunError("release identity contract is unavailable")
    return getattr(identity, field)


def _canonical_json_bytes(value: Any) -> bytes:
    try:
        rendered = json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
            allow_nan=False,
        )
    except (TypeError, ValueError) as error:
        raise DryRunError("release plan is not canonically serializable") from error
    return rendered.encode("utf-8")


def canonical_payload_sha256(record: Mapping[str, Any]) -> str:
    payload = {key: value for key, value in record.items() if key != "integrity"}
    return hashlib.sha256(_canonical_json_bytes(payload)).hexdigest()


def _read_toml(path: Path, label: str) -> Mapping[str, Any]:
    try:
        with path.open("rb") as handle:
            payload = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise DryRunError(f"{label} is unavailable") from error
    if not isinstance(payload, Mapping):
        raise DryRunError(f"{label} must contain a TOML table")
    return payload


def _require_native_identity(raw_tag: str) -> ReleaseIdentity:
    try:
        identity = parse_release_version(raw_tag)
    except (TypeError, ValueError) as error:
        raise DryRunError("release tag is invalid") from error
    if _identity_field(identity, "release_line") != "native-2x":
        raise DryRunError("dry run accepts only the native 2.x release line")
    if _identity_field(identity, "product") != "qiongli":
        raise DryRunError("release product is invalid")
    if _identity_field(identity, "source_branch") != SOURCE_BRANCH:
        raise DryRunError("native release branch contract is invalid")
    if _identity_field(identity, "version_source") != NATIVE_MANIFEST_RELATIVE:
        raise DryRunError("native version source contract is invalid")
    if _identity_field(identity, "repo_tag") != raw_tag.strip():
        raise DryRunError("release tag must use canonical v-prefixed form")
    return identity


def _source_identity(repo_root: Path) -> tuple[str, str, str]:
    manifest = _read_toml(repo_root / NATIVE_MANIFEST_RELATIVE, "native manifest")
    workspace = manifest.get("workspace")
    package = workspace.get("package") if isinstance(workspace, Mapping) else None
    metadata = workspace.get("metadata") if isinstance(workspace, Mapping) else None
    qiongli = metadata.get("qiongli") if isinstance(metadata, Mapping) else None
    version = package.get("version") if isinstance(package, Mapping) else None
    channel = qiongli.get("channel") if isinstance(qiongli, Mapping) else None
    product = qiongli.get("product") if isinstance(qiongli, Mapping) else None
    if not isinstance(version, str) or not version:
        raise DryRunError("native workspace package version is unavailable")
    if channel not in {"alpha", "beta", "stable"}:
        raise DryRunError("native workspace channel is invalid")
    if product != "qiongli":
        raise DryRunError("native workspace product is invalid")
    return product, version, channel


def _locked_product_version(repo_root: Path) -> str:
    lock = _read_toml(repo_root / NATIVE_LOCK_RELATIVE, "native Cargo lock")
    packages = lock.get("package")
    if not isinstance(packages, list):
        raise DryRunError("native Cargo lock package list is unavailable")
    matches = [
        item.get("version")
        for item in packages
        if isinstance(item, Mapping) and item.get("name") == "qiongli"
    ]
    if len(matches) != 1 or not isinstance(matches[0], str) or not matches[0]:
        raise DryRunError("native Cargo lock must contain exactly one qiongli package")
    return matches[0]


def verify_native_source(repo_root: Path, identity: ReleaseIdentity) -> None:
    product, version, channel = _source_identity(repo_root)
    if product != _identity_field(identity, "product"):
        raise SourceMismatch("native product does not match the release tag")
    if version != _identity_field(identity, "version"):
        raise SourceMismatch("native workspace version does not match the release tag")
    if channel != _identity_field(identity, "channel"):
        raise SourceMismatch("native workspace channel does not match the release tag")
    if _locked_product_version(repo_root) != version:
        raise SourceMismatch("native Cargo lock version does not match the workspace")


def _normalise_target_token(raw: str, aliases: Mapping[str, str], label: str) -> str:
    token = raw.strip().lower()
    if not token or token in _FORBIDDEN_TARGET_TOKENS:
        raise DryRunError(f"{label} target is not concrete")
    value = aliases.get(token)
    if value is None:
        raise DryRunError(f"{label} target is unsupported")
    return value


def normalise_os(raw: str) -> str:
    return _normalise_target_token(raw, _OS_ALIASES, "operating system")


def normalise_arch(raw: str) -> str:
    return _normalise_target_token(raw, _ARCH_ALIASES, "architecture")


def resolve_target(raw_os: str | None, raw_arch: str | None) -> tuple[str, str, str]:
    if (raw_os is None) != (raw_arch is None):
        raise DryRunError("operating system and architecture must be provided together")
    target_source = "explicit" if raw_os is not None and raw_arch is not None else "host-detected"
    os_value = normalise_os(raw_os if raw_os is not None else platform.system())
    arch_value = normalise_arch(raw_arch if raw_arch is not None else platform.machine())
    return os_value, arch_value, target_source


def _normalise_source_commit(raw: str | None) -> str | None:
    if raw is None:
        return None
    if not isinstance(raw, str):
        raise DryRunError("source commit must be a string")
    value = raw.strip()
    if SOURCE_COMMIT_PATTERN.fullmatch(value) is None:
        raise DryRunError("source commit must be a lowercase hexadecimal object id")
    return value


def _normalise_source_ref(raw: str | None) -> str | None:
    if raw is None:
        return None
    if not isinstance(raw, str):
        raise DryRunError("observed source ref must be a string")
    value = raw.strip()
    if (
        SOURCE_REF_PATTERN.fullmatch(value) is None
        or ".." in value
        or "//" in value
        or "@{" in value
        or value.endswith(("/", ".", ".lock"))
    ):
        raise DryRunError("observed source ref is invalid")
    return value


def _source_record(
    *,
    source_ref: str | None,
    source_ref_type: str,
    worktree_state: str,
    source_commit: str | None,
) -> dict[str, Any]:
    observed_ref = _normalise_source_ref(source_ref)
    commit = _normalise_source_commit(source_commit)
    if worktree_state not in {"clean", "dirty", "unknown"}:
        raise DryRunError("worktree state is invalid")
    if source_ref_type not in {"branch", "tag", "detached", "unknown"}:
        raise DryRunError("observed source ref type is invalid")
    if (observed_ref is None) != (source_ref_type == "unknown"):
        raise DryRunError("observed source ref and ref type must be assessed together")
    if commit is not None and worktree_state != "clean":
        raise DryRunError("source commit may be bound only to a clean worktree")
    if worktree_state in {"clean", "dirty"} and observed_ref is None:
        raise DryRunError("an assessed worktree requires an observed source ref")
    if worktree_state in {"clean", "dirty"} and source_ref_type == "unknown":
        raise DryRunError("an assessed worktree requires an observed source ref type")
    if worktree_state == "clean" and commit is None:
        raise DryRunError("a clean worktree assessment requires a source commit")

    return {
        "required_branch": SOURCE_BRANCH,
        "required_ref_type": "branch",
        "observed_ref": observed_ref,
        "observed_ref_type": source_ref_type,
        "worktree_state": worktree_state,
        "source_commit": commit,
        "release_source_eligible": (
            observed_ref == SOURCE_BRANCH
            and source_ref_type == "branch"
            and worktree_state == "clean"
            and commit is not None
        ),
        "version_source": VERSION_SOURCE,
        "channel_source": CHANNEL_SOURCE,
        "lock_source": NATIVE_LOCK_RELATIVE,
    }


def _identity_record(identity: ReleaseIdentity) -> dict[str, str]:
    return {
        "product": str(_identity_field(identity, "product")),
        "version": str(_identity_field(identity, "version")),
        "repo_tag": str(_identity_field(identity, "repo_tag")),
        "channel": str(_identity_field(identity, "channel")),
        "release_line": str(_identity_field(identity, "release_line")),
    }


def _artifact_id(identity: Mapping[str, str], os_value: str, arch_value: str) -> str:
    return "-".join(
        (
            identity["product"],
            identity["version"],
            identity["channel"],
            "bootstrap",
            os_value,
            arch_value,
            "portable-archive",
        )
    )


def validate_plan_semantics(plan: Mapping[str, Any]) -> None:
    try:
        identity_record = plan["identity"]
        if not isinstance(identity_record, Mapping):
            raise DryRunError("release plan identity is invalid")
        identity = _require_native_identity(str(identity_record["repo_tag"]))
        if dict(identity_record) != _identity_record(identity):
            raise DryRunError("release plan identity fields disagree")

        source_record = plan["source"]
        if not isinstance(source_record, Mapping):
            raise DryRunError("release plan source is invalid")
        expected_source = _source_record(
            source_ref=source_record["observed_ref"],
            source_ref_type=str(source_record["observed_ref_type"]),
            worktree_state=str(source_record["worktree_state"]),
            source_commit=source_record["source_commit"],
        )
        if dict(source_record) != expected_source:
            raise DryRunError("release plan source fields disagree")

        isolation = plan["channel_isolation"]
        if (
            not isinstance(isolation, Mapping)
            or isolation.get("selected_channel") != identity_record["channel"]
        ):
            raise DryRunError("release plan channel fields disagree")

        artifacts = plan["planned_artifacts"]
        if not isinstance(artifacts, list) or len(artifacts) != 1:
            raise DryRunError("release plan must contain one planned artifact")
        artifact = artifacts[0]
        artifact_identity = artifact["identity"] if isinstance(artifact, Mapping) else None
        if not isinstance(artifact_identity, Mapping):
            raise DryRunError("planned artifact identity is invalid")
        if {
            "product": artifact_identity.get("product"),
            "version": artifact_identity.get("version"),
            "channel": artifact_identity.get("channel"),
        } != {
            "product": identity_record["product"],
            "version": identity_record["version"],
            "channel": identity_record["channel"],
        }:
            raise DryRunError("planned artifact release identity disagrees")
        os_value = normalise_os(str(artifact_identity["os"]))
        arch_value = normalise_arch(str(artifact_identity["arch"]))
        if artifact_identity.get("profile") != "bootstrap" or artifact_identity.get(
            "installer_kind"
        ) != "portable-archive":
            raise DryRunError("planned artifact shape is invalid")
        if artifact.get("artifact_id") != _artifact_id(
            identity_record, os_value, arch_value
        ):
            raise DryRunError("planned artifact id disagrees")

        publication = plan["publication"]
        if (
            not isinstance(publication, Mapping)
            or publication.get("mode") != "dry-run"
            or publication.get("publication_performed") is not False
            or publication.get("publication_allowed") is not False
        ):
            raise DryRunError("release plan publication boundary is invalid")

        integrity = plan["integrity"]
        if (
            not isinstance(integrity, Mapping)
            or integrity.get("canonicalization") != CANONICALIZATION
            or integrity.get("payload_sha256") != canonical_payload_sha256(plan)
        ):
            raise DryRunError("release plan integrity is invalid")
    except (AttributeError, KeyError, TypeError, ValueError) as error:
        raise DryRunError("release plan semantic contract is invalid") from error


def validate_plan_schema(repo_root: Path, plan: Mapping[str, Any]) -> None:
    schema_path = repo_root / SCHEMA_RELATIVE
    try:
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise DryRunError("native release plan schema is unavailable") from error
    if not isinstance(schema, Mapping):
        raise DryRunError("native release plan schema is invalid")
    try:
        failures = validate_instance(plan, schema)
    except (TypeError, ValueError) as error:
        raise DryRunError("native release plan schema could not be evaluated") from error
    if failures:
        raise DryRunError("native release plan does not satisfy its closed schema")


def build_plan(
    repo_root: Path,
    raw_tag: str,
    *,
    target_os: str | None = None,
    target_arch: str | None = None,
    source_ref: str | None = None,
    source_ref_type: str = "unknown",
    worktree_state: str = "unknown",
    source_commit: str | None = None,
) -> dict[str, Any]:
    root = repo_root.resolve(strict=True)
    if not root.is_dir():
        raise DryRunError("repository root is unavailable")
    identity = _require_native_identity(raw_tag)
    verify_native_source(root, identity)
    os_value, arch_value, target_source = resolve_target(target_os, target_arch)
    source = _source_record(
        source_ref=source_ref,
        source_ref_type=source_ref_type,
        worktree_state=worktree_state,
        source_commit=source_commit,
    )
    identity_record = _identity_record(identity)
    artifact_id = _artifact_id(identity_record, os_value, arch_value)
    record: dict[str, Any] = {
        "$schema": SCHEMA_REFERENCE,
        "schema_version": SCHEMA_VERSION,
        "record_type": RECORD_TYPE,
        "task_id": "REL-201",
        "status": "planned-only",
        "identity": identity_record,
        "source": source,
        "channel_isolation": {
            "canonical_channels": ["alpha", "beta", "stable"],
            "selected_channel": identity_record["channel"],
            "mutable_alias_is_canonical": False,
            "cross_channel_fallback": False,
            "legacy_1x_feed_included": False,
            "pypi_publication": "not-applicable",
            "npm_publication": "not-applicable",
        },
        "publication": {
            "mode": "dry-run",
            "publication_performed": False,
            "publication_allowed": False,
            "publication_network_access": "forbidden",
            "git_mutation": "forbidden",
            "registry_publication": "not-applicable",
            "future_blockers": list(_FUTURE_BLOCKERS),
        },
        "planned_artifacts": [
            {
                "status": "planned-only",
                "artifact_id": artifact_id,
                "identity": {
                    "product": identity_record["product"],
                    "version": identity_record["version"],
                    "channel": identity_record["channel"],
                    "profile": "bootstrap",
                    "os": os_value,
                    "arch": arch_value,
                    "installer_kind": "portable-archive",
                },
                "target_source": target_source,
                "artifact_created": False,
                "target_native_startup_verified": False,
                "signed": False,
            }
        ],
        "rollback": {
            "strategy": "discard-unpublished-dry-run-bundle",
            "publication_withdrawal_required": False,
            "last_known_good_affected": False,
            "cross_channel_fallback_allowed": False,
            "actions": [
                "delete only the three qiongli-native-release dry-run bundle files if no longer needed; preserve the containing directory and unrelated files",
                "do not mutate Git refs, registries, release pages, channels, or installations",
                "if a future published identity fails, issue signed revocation metadata and retain the verified last-known-good identity",
            ],
        },
        "promotion": {
            "strategy": "new-version-new-identity",
            "relabel_existing_artifact": False,
            "move_mutable_channel_alias": False,
            "requirements": [
                "create a new SemVer release in the destination channel",
                "rerun destination-channel gates and target-native acceptance",
                "issue new manifests, evidence, signatures, and release receipt",
            ],
        },
        "non_goals": [
            "building or publishing a native executable or installer",
            "publishing Python or npm packages",
            "creating or moving Git refs",
            "publishing plugin or marketplace records",
            "claiming target-native startup, signing, SBOM, or provenance evidence",
        ],
    }
    record["integrity"] = {
        "canonicalization": CANONICALIZATION,
        "payload_sha256": canonical_payload_sha256(record),
    }
    return record


def _notes_text(plan: Mapping[str, Any]) -> str:
    identity = plan["identity"]
    artifact = plan["planned_artifacts"][0]
    target = artifact["identity"]
    blockers = "\n".join(f"- {item}" for item in plan["publication"]["future_blockers"])
    return (
        f"# Qiongli {identity['repo_tag']} Native Release Notes Dry Run\n\n"
        "Status: **planned only; publication is not allowed**.\n\n"
        f"Stage: {str(identity['channel']).title()}\n\n"
        f"- Product: `{identity['product']}`\n"
        f"- Version: `{identity['version']}`\n"
        f"- Canonical channel: `{identity['channel']}`\n"
        f"- Release line: `{identity['release_line']}`\n"
        f"- Required source branch: `{plan['source']['required_branch']}`\n"
        f"- Observed source ref: `{plan['source']['observed_ref_type']}:{plan['source']['observed_ref'] or 'not-assessed'}`\n"
        f"- Worktree state: `{plan['source']['worktree_state']}`\n"
        f"- Release-source eligible: `{str(plan['source']['release_source_eligible']).lower()}`\n"
        f"- Planned target: `{target['os']}/{target['arch']}`\n"
        f"- Planned profile/installer: `{target['profile']}/{target['installer_kind']}`\n"
        "- Artifact produced: `false`\n"
        "- PyPI publication: `not-applicable`\n"
        "- npm publication: `not-applicable`\n"
        "- Publication network access and Git/ref, registry, release-page, and marketplace mutation: `forbidden`\n\n"
        "## Future publication blockers\n\n"
        f"{blockers}\n\n"
        "This bundle is planning evidence only. It is not a native build, startup receipt, "
        "signature, SBOM, provenance record, release receipt, or authorization to publish.\n\n"
        "## Validation Evidence\n\n"
        "- Native version, channel, Cargo.lock, target identity, and closed plan schema are checked by the REL-201 dry-run.\n"
        "- Artifact build, target-native startup, signing, SBOM, provenance, updater, and publication acceptance remain unavailable.\n\n"
        "CI may download the pinned toolchain or locked dependencies and upload this planning evidence; those diagnostic transfers are not native publication.\n\n"
        "## Publish Steps\n\n"
        "None are authorized by this plan. Native `publish` and production `post` remain fail-closed.\n\n"
        "For the non-publishing recovery rules, see `tooling/release/rollback.md` and the generated rollback companion.\n"
    )


def _rollback_text(plan: Mapping[str, Any]) -> str:
    identity = plan["identity"]
    return (
        f"# Rollback Plan for {identity['repo_tag']} Dry Run\n\n"
        "No release was published, no Git ref was created or moved, no registry was changed, "
        "and no installed last-known-good identity was affected.\n\n"
        "## Current dry-run rollback\n\n"
        "1. Remove only the three generated `qiongli-native-release-*` bundle files if the planning bundle is no longer needed; preserve the containing directory and every unrelated file.\n"
        "2. Do not yank PyPI, change npm tags, edit marketplace records, or modify Git refs; those systems were not touched.\n"
        "3. Preserve the separate frozen 1.x feeds and every existing alpha, beta, or stable channel record.\n\n"
        "## Future publication and promotion semantics\n\n"
        "A failed future publication must be withdrawn through signed revocation or replacement metadata while retaining a verified last-known-good installation. "
        "Promotion never relabels this identity or moves it into another channel; it creates a new SemVer version, new immutable identity, new evidence, new signatures, and a new release receipt after destination-channel gates pass.\n"
    )


def _safe_output_directory(repo_root: Path, out_dir: Path) -> Path:
    root = repo_root.resolve(strict=True)
    if out_dir.is_symlink():
        raise DryRunError("output directory must not be a symbolic link")
    output = out_dir.resolve(strict=False)
    if output == root or output.is_relative_to(root):
        raise DryRunError("output directory must be outside the repository")
    if output.exists():
        if not output.is_dir():
            raise DryRunError("output directory path must be a directory")
        try:
            next(output.iterdir())
        except StopIteration:
            pass
        else:
            raise DryRunError("output directory must be empty")
    return output


def _assert_safe_output_path(path: Path, output: Path) -> None:
    if path.parent.resolve() != output:
        raise DryRunError("dry-run output path escapes its directory")
    if path.exists() and (path.is_symlink() or not path.is_file()):
        raise DryRunError("dry-run output path is unsafe")
    temporary = path.with_name(f".{path.name}.tmp")
    if temporary.exists() or temporary.is_symlink():
        raise DryRunError("dry-run temporary output path is unsafe")


def _write_bundle_texts(paths: Sequence[Path], contents: Sequence[str]) -> None:
    temporaries = [path.with_name(f".{path.name}.tmp") for path in paths]
    committed: list[Path] = []
    try:
        for temporary, content in zip(temporaries, contents, strict=True):
            with temporary.open("x", encoding="utf-8", newline="\n") as handle:
                handle.write(content)
        for temporary, path in zip(temporaries, paths, strict=True):
            temporary.replace(path)
            committed.append(path)
    except OSError as error:
        for candidate in [*temporaries, *committed]:
            try:
                if candidate.is_file() and not candidate.is_symlink():
                    candidate.unlink()
            except OSError:
                pass
        raise DryRunError("dry-run output could not be written safely") from error


def write_bundle(repo_root: Path, out_dir: Path, plan: Mapping[str, Any]) -> list[Path]:
    validate_plan_schema(repo_root, plan)
    validate_plan_semantics(plan)
    output = _safe_output_directory(repo_root, out_dir)
    output_created = not output.exists()
    repo_tag = str(plan["identity"]["repo_tag"])
    _require_native_identity(repo_tag)
    try:
        output.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        raise DryRunError("dry-run output directory could not be created") from error
    stem = f"qiongli-native-release-{repo_tag}"
    paths = [
        output / f"{stem}.json",
        output / f"{stem}-notes.md",
        output / f"{stem}-rollback.md",
    ]
    for path in paths:
        _assert_safe_output_path(path, output)
    try:
        _write_bundle_texts(
            paths,
            (
                json.dumps(plan, indent=2, ensure_ascii=False, sort_keys=True) + "\n",
                _notes_text(plan),
                _rollback_text(plan),
            ),
        )
    except DryRunError:
        if output_created:
            try:
                output.rmdir()
            except OSError:
                pass
        raise
    return paths


def _build_parser() -> argparse.ArgumentParser:
    parser = _RedactedArgumentParser(description="Create a non-publishing Qiongli native release dry-run bundle.")
    parser.add_argument("--tag", required=True)
    parser.add_argument("--root", type=Path, default=REPO_ROOT)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--os", dest="target_os")
    parser.add_argument("--arch", dest="target_arch")
    parser.add_argument("--source-ref")
    parser.add_argument(
        "--source-ref-type",
        choices=("branch", "tag", "detached", "unknown"),
        default="unknown",
    )
    parser.add_argument(
        "--worktree-state",
        choices=("clean", "dirty", "unknown"),
        default="unknown",
    )
    parser.add_argument("--source-commit")
    parser.add_argument("--json", action="store_true")
    return parser


def _emit(*, json_mode: bool, status: str, exit_code: int, code: str, plan: Mapping[str, Any] | None = None) -> None:
    if json_mode:
        payload: dict[str, Any] = {"status": status, "exit_code": exit_code, "code": code}
        if plan is not None:
            payload.update(
                {
                    "repo_tag": plan["identity"]["repo_tag"],
                    "channel": plan["identity"]["channel"],
                    "payload_sha256": plan["integrity"]["payload_sha256"],
                    "publication_performed": False,
                    "publication_allowed": False,
                }
            )
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
        return
    label = "PASS" if exit_code == 0 else "FAIL" if exit_code == 1 else "ERROR"
    print(f"[native-release-dry-run] {label}: {code}")


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    json_mode = "--json" in arguments
    try:
        args = _build_parser().parse_args(arguments)
        plan = build_plan(
            args.root,
            args.tag,
            target_os=args.target_os,
            target_arch=args.target_arch,
            source_ref=args.source_ref,
            source_ref_type=args.source_ref_type,
            worktree_state=args.worktree_state,
            source_commit=args.source_commit,
        )
        write_bundle(args.root, args.out_dir, plan)
        _emit(
            json_mode=bool(args.json),
            status="pass",
            exit_code=0,
            code="native-release-dry-run-written",
            plan=plan,
        )
        return 0
    except SourceMismatch:
        _emit(
            json_mode=json_mode,
            status="fail",
            exit_code=1,
            code="native-release-source-mismatch",
        )
        return 1
    except (CliUsageError, DryRunError, OSError, ValueError):
        _emit(
            json_mode=json_mode,
            status="error",
            exit_code=2,
            code="native-release-dry-run-unavailable",
        )
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
