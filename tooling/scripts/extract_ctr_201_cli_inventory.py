#!/usr/bin/env python3
from __future__ import annotations

import argparse
import copy
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tempfile
import tomllib
from types import ModuleType
from typing import Any, Mapping, Sequence


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_RELATIVE = "tooling/migration/baselines/v1.19.0-beta.1/manifest.json"
MANIFEST_SHA256 = "77bb7628d43a496c995e4b0a8daf6a624847b62e96948c0461affe89002da131"
DEFAULT_OUTPUT_RELATIVE = "tooling/migration/ctr-201-cli.json"
ACCEPTED_TAG = "v1.19.0-beta.1"
ACCEPTED_COMMIT = "8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f"
PYTHON_PACKAGE_ROOT = "packages/python-qiongli/"
PYTHON_SOURCE_ROOT = "packages/python-qiongli/src"
EXPECTED_PACKAGE_FILE_COUNT = 76
EXPECTED_PACKAGE_TREE_SHA256 = (
    "3a91a6dde9a78116fed73358275b2797c3ce7bf3d9a54894e7dbd11d2f0f9781"
)
PYPROJECT_BINDING: Mapping[str, Any] = {
    "path": "pyproject.toml",
    "git_blob_oid": "4fc00c6a21b5c7e8a9ffc1ac58698b9d2bd087a5",
    "sha256": "3001a8c7e6002e6fca928a7748dbb556e95c3cb6b9cc864a0a8d4294638f5ebf",
    "size_bytes": 1315,
    "mode": "100644",
}
CLI_BLOB_BINDING: Mapping[str, Any] = {
    "role": "cli-parser",
    "binding_class": "frozen-a8-domain-file",
    "path": "packages/python-qiongli/src/qiongli/cli.py",
    "git_blob_oid": "4802fddd7a019eb24ea6c69cc5f065e40ee3c61e",
    "sha256": "bab1a374aece96fbf802771774203536830709c3579541987ea46cf650d357bb",
    "size_bytes": 71311,
}
MCP_CLI_BLOB_BINDING: Mapping[str, Any] = {
    "role": "mcp-cli-parser",
    "binding_class": "frozen-a8-domain-file",
    "path": "packages/python-qiongli/src/qiongli/bridges/mcp_cli.py",
    "git_blob_oid": "c0ab03e7e1e07bfd8f059c94eadc0b69cefb5eae",
    "sha256": "5409f37d0eb1ac769fa7988f18d9a431a75c51947819e0e8087bf238559ba33b",
    "size_bytes": 11850,
}
PYPROJECT_BLOB_ANCHOR: Mapping[str, Any] = {
    "role": "console-entrypoints",
    "binding_class": "accepted-tag-additional-blob",
    "path": "pyproject.toml",
    "git_blob_oid": PYPROJECT_BINDING["git_blob_oid"],
    "sha256": PYPROJECT_BINDING["sha256"],
    "size_bytes": PYPROJECT_BINDING["size_bytes"],
}
PYTHON_ORACLE_BINDING: Mapping[str, Any] = {
    "oracle_id": "python-full",
    "path": "tooling/migration/baselines/v1.19.0-beta.1/oracles/python-full.json",
    "sha256": "26d247c9268c3166c98080aef420acfdb8248f62b11cc69420250f6e493a23e3",
    "case_count": 5,
}
EXPECTED_ENTRYPOINTS = (
    ("qiongli", "qiongli.cli:main"),
    ("ql", "qiongli.cli:main"),
    ("research-skills", "qiongli.cli:main"),
    ("rsk", "qiongli.cli:main"),
    ("rsw", "qiongli.cli:main"),
)
EXPECTED_COUNTS = {
    "main_canonical_commands": 39,
    "main_public_commands": 42,
    "mounted_mcp_canonical_commands": 7,
    "mounted_mcp_public_commands": 7,
    "total_canonical_commands": 46,
    "total_public_commands": 49,
    "non_help_actions": 164,
    "cwd_defaults": 27,
    "entrypoints": 5,
}
EXPECTED_ROOT_ALIASES = {
    "self-update": ["update"],
    "remove": ["uninstall", "delete"],
}
EXPECTED_ZERO_ARGUMENT_COMMANDS = {
    ("qiongli", "provider"),
    ("qiongli", "guidance"),
    ("qiongli", "subject"),
    ("qiongli", "project"),
    ("qiongli", "mcp", "config"),
}
EXPECTED_PARSER_ROOTS = [
    {
        "root_id": "qiongli-cli",
        "path": ["qiongli"],
        "builder": "qiongli.cli:build_parser",
        "source_anchor_role": "cli-parser",
        "description": (
            "Install/upgrade qiongli client skills without requiring a git fork."
        ),
        "subcommand_metadata": {"destination": "cmd", "required": True},
        "declaration_ordinal": 0,
    },
    {
        "root_id": "qiongli-mcp-cli",
        "path": ["qiongli", "mcp"],
        "builder": "bridges.mcp_cli:build_parser",
        "source_anchor_role": "mcp-cli-parser",
        "description": "Run and configure the Qiongli cross-platform MCP server.",
        "subcommand_metadata": {"destination": "cmd", "required": True},
        "declaration_ordinal": 1,
    },
]
WORKER_DIRECTORY_ENV_KEYS = (
    "HOME",
    "USERPROFILE",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_DATA_HOME",
    "CODEX_HOME",
    "CLAUDE_CODE_HOME",
    "ANTIGRAVITY_HOME",
    "HERMES_HOME",
    "TMP",
    "TEMP",
    "TMPDIR",
)
HEX_40 = re.compile(r"^[0-9a-f]{40}$")
HEX_64 = re.compile(r"^[0-9a-f]{64}$")
ARTIFACT_SCHEMA = "./ctr-201-cli.schema.json"
ARTIFACT_SCHEMA_VERSION = "1.0"
ARTIFACT_RECORD_TYPE = "qiongli-ctr-201-cli-static-semantics"
CANONICALIZATION = "utf-8-json-sorted-keys-compact-excluding-integrity"
STDLIB_FILELESS_MODULE_ALLOWLIST = frozenset(
    {"pyexpat.errors", "pyexpat.model", "typing.io", "typing.re"}
)


class ExtractorError(RuntimeError):
    """The accepted source or extraction environment cannot be evaluated safely."""


class InventoryMismatch(RuntimeError):
    """The extracted inventory differs from its accepted facts or checked output."""


class CliUsageError(RuntimeError):
    """Public CLI usage is invalid and must be reported without echoing input."""


class _RedactedArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:  # pragma: no cover - exercised through main
        del message
        raise CliUsageError("invalid command usage")


def _require_python_312() -> None:
    if sys.version_info[:2] != (3, 12):
        raise ExtractorError("CTR-201B extraction requires Python 3.12")


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


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
        raise ExtractorError("artifact cannot be serialized canonically") from error
    return rendered.encode("utf-8")


def canonical_payload_sha256(record: Mapping[str, Any]) -> str:
    payload = {key: value for key, value in record.items() if key != "integrity"}
    return _sha256(_canonical_json_bytes(payload))


def _reject_duplicate_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ExtractorError("JSON document contains a duplicate key")
        result[key] = value
    return result


def _contains_surrogate(value: Any) -> bool:
    if isinstance(value, str):
        return any(0xD800 <= ord(character) <= 0xDFFF for character in value)
    if isinstance(value, Mapping):
        return any(_contains_surrogate(key) or _contains_surrogate(item) for key, item in value.items())
    if isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray)):
        return any(_contains_surrogate(item) for item in value)
    return False


def _reject_nonfinite_constant(value: str) -> None:
    del value
    raise ExtractorError("JSON document contains a non-finite number")


def _load_json_bytes(data: bytes) -> Mapping[str, Any]:
    try:
        text = data.decode("utf-8")
        value = json.loads(
            text,
            object_pairs_hook=_reject_duplicate_object,
            parse_constant=_reject_nonfinite_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ExtractorError("JSON document is invalid") from error
    if not isinstance(value, Mapping):
        raise ExtractorError("JSON document must contain an object")
    if _contains_surrogate(value):
        raise ExtractorError("JSON document contains invalid Unicode scalar data")
    return value


def _canonical_repository_path(raw: Any, *, prefix: str | None = None) -> str:
    if not isinstance(raw, str) or not raw or "\\" in raw or ":" in raw or "\x00" in raw:
        raise ExtractorError("accepted source contains a non-canonical path")
    path = PurePosixPath(raw)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        raise ExtractorError("accepted source contains a non-canonical path")
    normalized = path.as_posix()
    if normalized != raw:
        raise ExtractorError("accepted source contains a non-canonical path")
    if prefix is not None and not normalized.startswith(prefix):
        raise ExtractorError("accepted source escapes its package boundary")
    return normalized


def _read_manifest(repo_root: Path) -> tuple[Mapping[str, Any], list[Mapping[str, Any]]]:
    manifest_path = repo_root / MANIFEST_RELATIVE
    try:
        manifest_bytes = manifest_path.read_bytes()
    except OSError as error:
        raise ExtractorError("accepted A8 manifest is unavailable") from error
    if _sha256(manifest_bytes) != MANIFEST_SHA256:
        raise InventoryMismatch("accepted A8 manifest digest drifted")
    manifest = _load_json_bytes(manifest_bytes)
    source = manifest.get("source")
    if not isinstance(source, Mapping):
        raise ExtractorError("accepted A8 source identity is missing")
    if source.get("tag") != ACCEPTED_TAG or source.get("peeled_commit") != ACCEPTED_COMMIT:
        raise InventoryMismatch("accepted A8 source identity drifted")
    package_trees = manifest.get("package_trees")
    if not isinstance(package_trees, list):
        raise ExtractorError("accepted A8 package trees are unavailable")
    matches = [
        item
        for item in package_trees
        if isinstance(item, Mapping) and item.get("root") == PYTHON_PACKAGE_ROOT
    ]
    if len(matches) != 1:
        raise ExtractorError("accepted Python package tree is not unique")
    tree = matches[0]
    if (
        tree.get("file_count") != EXPECTED_PACKAGE_FILE_COUNT
        or tree.get("tree_sha256") != EXPECTED_PACKAGE_TREE_SHA256
    ):
        raise InventoryMismatch("accepted Python package tree identity drifted")
    files = tree.get("files")
    if not isinstance(files, list) or len(files) != EXPECTED_PACKAGE_FILE_COUNT:
        raise ExtractorError("accepted Python package file inventory is incomplete")
    normalized: list[Mapping[str, Any]] = []
    seen_paths: set[str] = set()
    seen_oids: set[str] = set()
    for item in files:
        if not isinstance(item, Mapping):
            raise ExtractorError("accepted Python package file entry is invalid")
        path = _canonical_repository_path(item.get("path"), prefix=PYTHON_PACKAGE_ROOT)
        oid = item.get("git_blob_oid")
        digest = item.get("sha256")
        size = item.get("size_bytes")
        mode = item.get("mode")
        if path in seen_paths:
            raise ExtractorError("accepted Python package contains a duplicate path")
        if (
            not isinstance(oid, str)
            or not HEX_40.fullmatch(oid)
            or not isinstance(digest, str)
            or not HEX_64.fullmatch(digest)
            or not isinstance(size, int)
            or isinstance(size, bool)
            or size < 0
            or mode != "100644"
        ):
            raise ExtractorError("accepted Python package file metadata is invalid")
        seen_paths.add(path)
        seen_oids.add(oid)
        normalized.append(
            {
                "path": path,
                "git_blob_oid": oid,
                "sha256": digest,
                "size_bytes": size,
                "mode": mode,
            }
        )
    if len(seen_oids) == 0:
        raise ExtractorError("accepted Python package has no readable blobs")
    by_path = {str(item["path"]): item for item in normalized}
    for anchor in (CLI_BLOB_BINDING, MCP_CLI_BLOB_BINDING):
        actual = by_path.get(str(anchor["path"]))
        expected = {
            "path": anchor["path"],
            "git_blob_oid": anchor["git_blob_oid"],
            "sha256": anchor["sha256"],
            "size_bytes": anchor["size_bytes"],
            "mode": "100644",
        }
        if actual != expected:
            raise InventoryMismatch("accepted parser blob anchor drifted")
    oracle_fixtures = manifest.get("oracle_fixtures")
    if not isinstance(oracle_fixtures, list):
        raise ExtractorError("accepted A8 oracle fixtures are unavailable")
    oracle_matches = [
        item
        for item in oracle_fixtures
        if isinstance(item, Mapping) and item.get("oracle_id") == PYTHON_ORACLE_BINDING["oracle_id"]
    ]
    if len(oracle_matches) != 1:
        raise ExtractorError("accepted Python oracle fixture is not unique")
    oracle = oracle_matches[0]
    if (
        oracle.get("path") != "oracles/python-full.json"
        or oracle.get("sha256") != PYTHON_ORACLE_BINDING["sha256"]
        or oracle.get("case_count") != PYTHON_ORACLE_BINDING["case_count"]
    ):
        raise InventoryMismatch("accepted Python oracle binding drifted")
    return manifest, sorted(normalized, key=lambda item: str(item["path"]))


def _verify_pyproject_commit_binding(repo_root: Path) -> None:
    try:
        completed = subprocess.run(
            ["git", "ls-tree", "-z", ACCEPTED_COMMIT, "--", PYPROJECT_BINDING["path"]],
            cwd=repo_root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise ExtractorError("accepted commit tree reader is unavailable") from error
    expected = (
        f"{PYPROJECT_BINDING['mode']} blob {PYPROJECT_BINDING['git_blob_oid']}"
        f"\t{PYPROJECT_BINDING['path']}\0"
    ).encode("ascii")
    if completed.returncode != 0 or completed.stderr:
        raise ExtractorError("accepted commit tree could not be inspected")
    if completed.stdout != expected:
        raise InventoryMismatch("accepted pyproject path-to-blob binding drifted")


def _cat_file_blobs(repo_root: Path, entries: Sequence[Mapping[str, Any]]) -> dict[str, bytes]:
    request = b"".join(f"{entry['git_blob_oid']}\n".encode("ascii") for entry in entries)
    try:
        completed = subprocess.run(
            ["git", "cat-file", "--batch"],
            cwd=repo_root,
            input=request,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise ExtractorError("local Git object reader is unavailable") from error
    if completed.returncode != 0:
        raise ExtractorError("local Git object reader failed")
    stream = io.BytesIO(completed.stdout)
    result: dict[str, bytes] = {}
    for expected in entries:
        header = stream.readline()
        if not header.endswith(b"\n"):
            raise ExtractorError("local Git object response is truncated")
        tokens = header[:-1].split()
        if len(tokens) == 2 and tokens[1] == b"missing":
            raise ExtractorError("accepted Git blob is unavailable locally")
        if len(tokens) != 3:
            raise ExtractorError("local Git object response is invalid")
        try:
            oid = tokens[0].decode("ascii")
            object_type = tokens[1].decode("ascii")
            size = int(tokens[2].decode("ascii"))
        except (UnicodeDecodeError, ValueError) as error:
            raise ExtractorError("local Git object response is invalid") from error
        if oid != expected["git_blob_oid"] or object_type != "blob" or size != expected["size_bytes"]:
            raise InventoryMismatch("accepted Git blob identity does not match")
        payload = stream.read(size)
        if len(payload) != size or stream.read(1) != b"\n":
            raise ExtractorError("local Git blob payload is truncated")
        if _sha256(payload) != expected["sha256"]:
            raise InventoryMismatch("accepted Git blob digest does not match")
        result[str(expected["path"])] = payload
    if stream.read(1):
        raise ExtractorError("local Git object response contains trailing data")
    return result


def _write_materialized_tree(destination: Path, blobs: Mapping[str, bytes]) -> Path:
    source_root = destination / PYTHON_SOURCE_ROOT
    for relative, payload in blobs.items():
        path = destination.joinpath(*PurePosixPath(relative).parts)
        try:
            path.relative_to(destination)
        except ValueError as error:  # pragma: no cover - protected by canonical path validation
            raise ExtractorError("accepted source path escapes the temporary tree") from error
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(payload)
    return source_root


def _worker_environment(temp_root: Path, variant: str) -> dict[str, str]:
    state_root = temp_root / f"state-{variant}"
    values = {
        "HOME": str(state_root / "home"),
        "USERPROFILE": str(state_root / "home"),
        "XDG_CONFIG_HOME": str(state_root / "xdg-config"),
        "XDG_CACHE_HOME": str(state_root / "xdg-cache"),
        "XDG_DATA_HOME": str(state_root / "xdg-data"),
        "CODEX_HOME": str(state_root / "codex"),
        "CLAUDE_CODE_HOME": str(state_root / "claude"),
        "ANTIGRAVITY_HOME": str(state_root / "antigravity"),
        "HERMES_HOME": str(state_root / "hermes"),
        "TMP": str(state_root / "tmp"),
        "TEMP": str(state_root / "tmp"),
        "TMPDIR": str(state_root / "tmp"),
        "PATH": "" if variant == "a" else str(state_root / "unused-bin"),
        "PYTHONNOUSERSITE": "1",
        "PYTHONDONTWRITEBYTECODE": "1",
        "PYTHONUTF8": "1",
    }
    if os.name == "nt" and os.environ.get("SystemRoot"):
        values["SystemRoot"] = os.environ["SystemRoot"]
    return values


def _capture_once(repo_root: Path, package_entries: Sequence[Mapping[str, Any]], variant: str) -> Mapping[str, Any]:
    pyproject_entry = dict(PYPROJECT_BINDING)
    all_entries = [*package_entries, pyproject_entry]
    blobs = _cat_file_blobs(repo_root, all_entries)
    with tempfile.TemporaryDirectory(prefix=f"qiongli-ctr201b-{variant}-") as raw_temp:
        temp_root = Path(raw_temp)
        accepted_root = temp_root / "accepted"
        accepted_root.mkdir()
        source_root = _write_materialized_tree(accepted_root, blobs)
        cwd = temp_root / f"cwd-{variant}"
        cwd.mkdir()
        environment = _worker_environment(temp_root, variant)
        for key in WORKER_DIRECTORY_ENV_KEYS:
            Path(environment[key]).mkdir(parents=True, exist_ok=True)
        control = {
            "accepted_root": str(accepted_root),
            "source_root": str(source_root),
            "cwd": str(cwd),
            "write_root": str(temp_root),
            "pyproject": str(accepted_root / "pyproject.toml"),
        }
        control_path = temp_root / "control.json"
        control_path.write_bytes(_canonical_json_bytes(control))
        command = [
            sys.executable,
            "-I",
            "-S",
            "-B",
            str(Path(__file__).resolve()),
            "--_worker",
            str(control_path),
        ]
        try:
            completed = subprocess.run(
                command,
                cwd=cwd,
                env=environment,
                text=True,
                encoding="utf-8",
                errors="strict",
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
        except OSError as error:
            raise ExtractorError("isolated parser worker could not start") from error
        if completed.returncode != 0 or completed.stderr:
            raise ExtractorError("isolated parser worker failed")
        try:
            payload = json.loads(
                completed.stdout,
                object_pairs_hook=_reject_duplicate_object,
                parse_constant=_reject_nonfinite_constant,
            )
        except (json.JSONDecodeError, ExtractorError) as error:
            raise ExtractorError("isolated parser worker returned invalid JSON") from error
        if not isinstance(payload, Mapping) or payload.get("status") != "pass":
            raise ExtractorError("isolated parser worker did not return an inventory")
        artifact = payload.get("artifact")
        if not isinstance(artifact, Mapping) or _contains_surrogate(artifact):
            raise ExtractorError("isolated parser worker artifact is invalid")
        return artifact


def _parse_entrypoints(pyproject_bytes: bytes) -> list[Mapping[str, Any]]:
    try:
        document = tomllib.loads(pyproject_bytes.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise ExtractorError("accepted pyproject is invalid") from error
    project = document.get("project")
    scripts = project.get("scripts") if isinstance(project, Mapping) else None
    if not isinstance(scripts, Mapping) or list(scripts.items()) != list(EXPECTED_ENTRYPOINTS):
        raise InventoryMismatch("accepted console entrypoints differ from CTR-201B")
    result: list[Mapping[str, Any]] = []
    for ordinal, (name, target) in enumerate(scripts.items()):
        if not isinstance(target, str) or target.count(":") != 1:
            raise ExtractorError("accepted console entrypoint target is invalid")
        result.append(
            {
                "name": name,
                "target": target,
                "declaration_ordinal": ordinal,
            }
        )
    return result


def _validate_expected_inventory(artifact: Mapping[str, Any]) -> None:
    expected_top_level = {
        "$schema",
        "schema_version",
        "record_type",
        "task_id",
        "status",
        "source",
        "capture_contract",
        "console_entrypoints",
        "parser_roots",
        "commands",
        "coverage",
        "integrity",
    }
    if set(artifact) != expected_top_level:
        raise InventoryMismatch("extracted CLI artifact shape differs from CTR-201B")
    entrypoints = artifact.get("console_entrypoints")
    expected_entrypoints = [
        {"name": name, "target": target, "declaration_ordinal": ordinal}
        for ordinal, (name, target) in enumerate(EXPECTED_ENTRYPOINTS)
    ]
    if entrypoints != expected_entrypoints:
        raise InventoryMismatch("extracted console entrypoints differ from CTR-201B")
    if artifact.get("parser_roots") != EXPECTED_PARSER_ROOTS:
        raise InventoryMismatch("extracted parser-root metadata differs from CTR-201B")
    expected_capture_contract = {
        "source_mode": "accepted-tag-git-blobs",
        "python_version": "python3.12",
        "help_mode": "authored-help-only",
        "environment_mode": "dual-environment",
        "environment_count": 2,
        "dynamic_default_policy": "symbolic-context-values",
        "callable_policy": "allowlisted-symbols-no-repr",
        "ambient_dependency_policy": "disabled-with-deny-use-stubs",
        "side_effect_policy": "read-only-no-network-no-process",
    }
    if artifact.get("capture_contract") != expected_capture_contract:
        raise InventoryMismatch("extracted capture contract differs from CTR-201B")
    commands = artifact.get("commands")
    if not isinstance(commands, list) or len(commands) != EXPECTED_COUNTS["total_canonical_commands"]:
        raise InventoryMismatch("extracted canonical commands differ from CTR-201B")
    if any(not isinstance(command, Mapping) for command in commands):
        raise InventoryMismatch("extracted CLI command is invalid")
    aliases = {
        tuple(command.get("path", [])): command.get("aliases", [])
        for command in commands
    }
    for command_name, expected_aliases in EXPECTED_ROOT_ALIASES.items():
        if aliases.get(("qiongli", command_name)) != expected_aliases:
            raise InventoryMismatch("extracted root command aliases differ from CTR-201B")
    public_count = sum(1 + len(command.get("aliases", [])) for command in commands)
    argument_count = sum(len(command.get("arguments", [])) for command in commands)
    cwd_default_count = sum(
        argument.get("default") == {"kind": "context", "source": "cwd"}
        for command in commands
        for argument in command.get("arguments", [])
        if isinstance(argument, Mapping)
    )
    zero_argument_commands = {
        tuple(command.get("path", []))
        for command in commands
        if command.get("arguments") == []
    }
    if zero_argument_commands != EXPECTED_ZERO_ARGUMENT_COMMANDS:
        raise InventoryMismatch("extracted zero-argument commands differ from CTR-201B")
    mcp_delegate = {
        "kind": "parser-root",
        "parser_root_id": "qiongli-mcp-cli",
        "argument_destination": "mcp_args",
    }
    for command in commands:
        expected_delegate = mcp_delegate if command.get("path") == ["qiongli", "mcp"] else None
        if command.get("delegate") != expected_delegate:
            raise InventoryMismatch("extracted parser delegation differs from CTR-201B")
    coverage = artifact.get("coverage")
    expected_coverage = {
        "canonical_command_count": EXPECTED_COUNTS["total_canonical_commands"],
        "public_command_count": EXPECTED_COUNTS["total_public_commands"],
        "console_entrypoint_count": EXPECTED_COUNTS["entrypoints"],
        "argument_action_count": EXPECTED_COUNTS["non_help_actions"],
        "cwd_default_count": EXPECTED_COUNTS["cwd_defaults"],
        "static_semantics": "captured",
        "runtime_behavior_matrix": "incomplete",
        "exit_code_matrix": "incomplete",
        "error_matrix": "incomplete",
        "side_effect_matrix": "incomplete",
        "formatted_help_output": "incomplete",
        "json_output": "incomplete",
        "dry_run_semantics": "incomplete",
        "legacy_npm_compatibility_surface": "incomplete",
        "ctr_201": "in-progress",
        "fnd_202": "not-implemented",
        "completion_ready": False,
    }
    if (
        not isinstance(coverage, Mapping)
        or dict(coverage) != expected_coverage
        or public_count != expected_coverage["public_command_count"]
        or argument_count != expected_coverage["argument_action_count"]
        or cwd_default_count != expected_coverage["cwd_default_count"]
    ):
        raise InventoryMismatch("CTR-201B completion boundary was weakened")
    integrity = artifact.get("integrity")
    if (
        not isinstance(integrity, Mapping)
        or integrity.get("algorithm") != "sha256"
        or integrity.get("canonicalization") != CANONICALIZATION
        or integrity.get("payload_sha256") != canonical_payload_sha256(artifact)
    ):
        raise InventoryMismatch("CTR-201B artifact integrity is invalid")


def extract_cli_inventory(repo_root: Path = REPO_ROOT) -> Mapping[str, Any]:
    _require_python_312()
    root = repo_root.resolve()
    _, package_entries = _read_manifest(root)
    _verify_pyproject_commit_binding(root)
    captures = [_capture_once(root, package_entries, variant) for variant in ("a", "b")]
    if _canonical_json_bytes(captures[0]) != _canonical_json_bytes(captures[1]):
        raise InventoryMismatch("isolated CLI captures are not deterministic")
    artifact = copy.deepcopy(captures[0])
    pyproject_bytes = _cat_file_blobs(root, [PYPROJECT_BINDING])["pyproject.toml"]
    entrypoints = _parse_entrypoints(pyproject_bytes)
    if artifact.get("console_entrypoints") != entrypoints:
        raise InventoryMismatch("worker and parent entrypoint inventories differ")
    _validate_expected_inventory(artifact)
    return artifact


def _normalize_typed_value(value: Any, *, cwd: str) -> Mapping[str, Any]:
    if value == argparse.SUPPRESS:
        return {"kind": "suppressed"}
    if isinstance(value, str) and value == cwd:
        return {"kind": "context", "source": "cwd"}
    if value is None:
        return {"kind": "none"}
    if isinstance(value, bool):
        return {"kind": "boolean", "value": value}
    if isinstance(value, int):
        return {"kind": "integer", "value": value}
    if isinstance(value, str):
        return {"kind": "string", "value": value}
    raise ExtractorError("argparse value is outside the CTR-201B scalar allowlist")


def _normalize_nargs(value: Any) -> str:
    if value is None or value == 1:
        return "one"
    if isinstance(value, int) and not isinstance(value, bool) and value == 0:
        return "zero"
    symbolic = {
        argparse.OPTIONAL: "optional",
        argparse.ONE_OR_MORE: "one-or-more",
        argparse.REMAINDER: "remainder",
    }
    if value in symbolic:
        return symbolic[value]
    raise ExtractorError("argparse nargs is outside the CTR-201B allowlist")


def _normalize_type(value: Any) -> str:
    if value is None:
        return "none"
    if value is int:
        return "integer"
    raise ExtractorError("argparse type is outside the CTR-201B callable allowlist")


def _normalize_help(value: Any) -> Mapping[str, Any]:
    if value == argparse.SUPPRESS:
        return {"kind": "suppressed"}
    if value is None:
        return {"kind": "none"}
    if not isinstance(value, str) or not value:
        raise ExtractorError("argparse help text is invalid")
    return {"kind": "text", "value": value}


def _action_kind(action: argparse.Action) -> str:
    mapping = {
        argparse._StoreAction: "store",
        argparse._StoreConstAction: "store-const",
        argparse._StoreTrueAction: "store-true",
        argparse._StoreFalseAction: "store-false",
    }
    kind = mapping.get(type(action))
    if kind is None:
        raise ExtractorError("argparse action is outside the CTR-201B allowlist")
    return kind


def _normalize_metavar(value: Any) -> Mapping[str, Any]:
    if value is None:
        return {"kind": "none"}
    if isinstance(value, str) and value:
        return {"kind": "text", "value": value}
    if (
        isinstance(value, tuple)
        and value
        and all(isinstance(item, str) and item for item in value)
    ):
        return {"kind": "tuple", "values": list(value)}
    raise ExtractorError("argparse metavar is outside the CTR-201B allowlist")


def _normalize_choices(value: Any) -> Mapping[str, Any]:
    if value is None:
        return {"kind": "none"}
    if isinstance(value, (set, frozenset)):
        raise ExtractorError("argparse choices do not preserve declaration order")
    try:
        items = list(value)
    except TypeError as error:
        raise ExtractorError("argparse choices are not iterable") from error
    if not items or len(items) != len({(type(item), item) for item in items}):
        raise ExtractorError("argparse choices are empty or duplicated")
    if any(isinstance(item, bool) or not isinstance(item, (str, int)) for item in items):
        raise ExtractorError("argparse choice is outside the CTR-201B scalar allowlist")
    return {"kind": "ordered", "values": items}


def _normalize_destination(value: Any) -> str | None:
    if value == argparse.SUPPRESS:
        return None
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z][A-Za-z0-9._-]*", value):
        raise ExtractorError("argparse destination is invalid")
    return value


def _normalize_action(action: argparse.Action, ordinal: int, *, cwd: str) -> Mapping[str, Any]:
    return {
        "destination": _normalize_destination(action.dest),
        "option_strings": list(action.option_strings),
        "positional": not bool(action.option_strings),
        "declaration_ordinal": ordinal,
        "action": _action_kind(action),
        "nargs": _normalize_nargs(action.nargs),
        "type": _normalize_type(action.type),
        "required": bool(action.required),
        "choices": _normalize_choices(action.choices),
        "default": _normalize_typed_value(action.default, cwd=cwd),
        "const": _normalize_typed_value(action.const, cwd=cwd),
        "help": _normalize_help(action.help),
        "metavar": _normalize_metavar(action.metavar),
    }


def _require_allowlisted_parser(parser: argparse.ArgumentParser) -> None:
    if (
        type(parser) is not argparse.ArgumentParser
        or parser.formatter_class is not argparse.HelpFormatter
    ):
        raise ExtractorError("argparse parser is outside the CTR-201B allowlist")


def _subparser_groups(action: argparse._SubParsersAction[Any]) -> list[tuple[argparse.ArgumentParser, list[str], str | None]]:
    if type(action) is not argparse._SubParsersAction:
        raise ExtractorError("argparse subparser action is outside the allowlist")
    groups: list[tuple[argparse.ArgumentParser, list[str]]] = []
    by_identity: dict[int, int] = {}
    for name, parser in action.choices.items():
        _require_allowlisted_parser(parser)
        identity = id(parser)
        if identity not in by_identity:
            by_identity[identity] = len(groups)
            groups.append((parser, [name]))
        else:
            groups[by_identity[identity]][1].append(name)
    pseudo_actions = list(getattr(action, "_choices_actions", []))
    if len(pseudo_actions) != len(groups):
        raise ExtractorError("argparse subcommand help mapping is ambiguous")
    result: list[tuple[argparse.ArgumentParser, list[str], str | None]] = []
    for (parser, names), pseudo in zip(groups, pseudo_actions, strict=True):
        if type(pseudo) is not argparse._SubParsersAction._ChoicesPseudoAction:
            raise ExtractorError("argparse command help action is outside the allowlist")
        help_value = getattr(pseudo, "help", None)
        if help_value is not None and not isinstance(help_value, str):
            raise ExtractorError("argparse subcommand help is invalid")
        result.append((parser, names, help_value))
    return result


def _parser_subcommand_metadata(parser: argparse.ArgumentParser) -> Mapping[str, Any] | None:
    _require_allowlisted_parser(parser)
    actions = [
        action
        for action in parser._actions
        if type(action) is argparse._SubParsersAction
    ]
    if not actions:
        return None
    if len(actions) != 1:
        raise ExtractorError("argparse parser has multiple subparser actions")
    action = actions[0]
    destination = _normalize_destination(action.dest)
    if destination is None:
        raise ExtractorError("argparse subcommand destination cannot be suppressed")
    return {"destination": destination, "required": bool(action.required)}


def _normalize_nullable_text(value: Any) -> str | None:
    if value is None or value == argparse.SUPPRESS:
        return None
    if not isinstance(value, str) or not value:
        raise ExtractorError("argparse command text is invalid")
    return value


def _extract_commands(
    parser: argparse.ArgumentParser,
    *,
    root_path: Sequence[str],
    cwd: str,
) -> Mapping[str, Any]:
    _require_allowlisted_parser(parser)
    root_semantic_actions = [
        action
        for action in parser._actions
        if type(action) not in (argparse._HelpAction, argparse._SubParsersAction)
    ]
    if root_semantic_actions:
        raise ExtractorError("argparse root actions cannot be represented by CTR-201B")
    queue: list[tuple[argparse.ArgumentParser, tuple[str, ...]]] = [(parser, tuple(root_path))]
    seen: set[int] = set()
    commands: list[Mapping[str, Any]] = []
    while queue:
        current, parent_path = queue.pop(0)
        _require_allowlisted_parser(current)
        if id(current) in seen:
            continue
        seen.add(id(current))
        for action in current._actions:
            if type(action) is argparse._SubParsersAction:
                for child, names, command_help in _subparser_groups(action):
                    canonical = names[0]
                    child_path = (*parent_path, canonical)
                    semantic_actions = [
                        child_action
                        for child_action in child._actions
                        if type(child_action)
                        not in (argparse._HelpAction, argparse._SubParsersAction)
                    ]
                    commands.append(
                        {
                            "segment": canonical,
                            "path": list(child_path),
                            "aliases": names[1:],
                            "help": _normalize_nullable_text(command_help),
                            "description": _normalize_nullable_text(child.description),
                            "declaration_ordinal": len(commands),
                            "subcommand_metadata": _parser_subcommand_metadata(child),
                            "delegate": (
                                {
                                    "kind": "parser-root",
                                    "parser_root_id": "qiongli-mcp-cli",
                                    "argument_destination": "mcp_args",
                                }
                                if child_path == ("qiongli", "mcp")
                                else None
                            ),
                            "arguments": [
                                _normalize_action(child_action, ordinal, cwd=cwd)
                                for ordinal, child_action in enumerate(semantic_actions)
                            ],
                        }
                    )
                    queue.append((child, child_path))
    canonical_count = len(commands)
    public_count = sum(1 + len(command["aliases"]) for command in commands)
    argument_count = sum(len(command["arguments"]) for command in commands)
    cwd_defaults = sum(
        argument["default"] == {"kind": "context", "source": "cwd"}
        for command in commands
        for argument in command["arguments"]
    )
    return {
        "commands": commands,
        "counts": {
            "canonical_commands": canonical_count,
            "public_commands": public_count,
            "non_help_actions": argument_count,
            "cwd_defaults": cwd_defaults,
        },
    }


def _path_is_within(path: Path, root: Path) -> bool:
    try:
        path.resolve(strict=False).relative_to(root.resolve(strict=False))
        return True
    except (OSError, ValueError):
        return False


_FORBIDDEN_PROCESS_EVENTS = {
    "os.chdir",
    "os.exec",
    "os.fchdir",
    "os.fork",
    "os.forkpty",
    "os.posix_spawn",
    "os.spawn",
    "os.startfile",
    "os.startfile/2",
    "os.system",
    "pty.spawn",
    "subprocess.Popen",
}
_MUTATION_EVENTS = frozenset(
    {
        "os.chflags",
        "os.chmod",
        "os.chown",
        "os.link",
        "os.mkdir",
        "os.mkfifo",
        "os.mknod",
        "os.remove",
        "os.removexattr",
        "os.rename",
        "os.replace",
        "os.rmdir",
        "os.setxattr",
        "os.symlink",
        "os.truncate",
        "os.unlink",
        "os.utime",
        "shutil.copyfile",
        "shutil.copymode",
        "shutil.copystat",
        "shutil.copytree",
        "shutil.make_archive",
        "shutil.move",
        "shutil.rmtree",
        "shutil.unpack_archive",
    }
)


def _guard_worker_audit_event(event: str, args: tuple[Any, ...], write_root: Path) -> None:
    del write_root
    if (
        event in _FORBIDDEN_PROCESS_EVENTS
        or event.startswith(("os.exec", "os.spawn", "pty.spawn", "socket."))
    ):
        raise PermissionError("accepted parser attempted a forbidden capability")
    if event == "open" and args:
        mode = args[1] if len(args) > 1 else "r"
        flags = args[2] if len(args) > 2 else 0
        write_mode = isinstance(mode, str) and any(
            token in mode for token in ("w", "a", "x", "+")
        )
        write_flags = isinstance(flags, int) and bool(
            flags
            & (
                getattr(os, "O_WRONLY", 0)
                | getattr(os, "O_RDWR", 0)
                | getattr(os, "O_CREAT", 0)
                | getattr(os, "O_TRUNC", 0)
                | getattr(os, "O_APPEND", 0)
            )
        )
        if write_mode or write_flags:
            raise PermissionError("accepted parser attempted a write")
    if event in _MUTATION_EVENTS:
        raise PermissionError("accepted parser attempted a mutation")


def _install_worker_audit_hook(write_root: Path) -> None:
    def audit(event: str, args: tuple[Any, ...]) -> None:
        _guard_worker_audit_event(event, args, write_root)

    sys.addaudithook(audit)


class _DeniedYamlUseError(Exception):
    """Sentinel exception type exposed by the inert parser-capture YAML stub."""


def _reject_yaml_attribute(name: str) -> Any:
    del name
    raise ExtractorError("accepted parser attempted to use ambient YAML")


def _make_yaml_deny_use_stub() -> ModuleType:
    stub = ModuleType("yaml")
    stub.__dict__["YAMLError"] = _DeniedYamlUseError
    stub.__dict__["__getattr__"] = _reject_yaml_attribute
    return stub


def _verify_dependency_isolation(
    source_root: Path,
    base_sys_path: Sequence[str],
    modules_before: set[str],
    yaml_stub: ModuleType,
) -> None:
    if sys.flags.no_site != 1:
        raise ExtractorError("isolated parser worker did not disable site imports")
    if any(
        not isinstance(entry, str)
        or any(
            part.lower() in {"site-packages", "dist-packages"}
            for part in Path(entry).parts
        )
        for entry in base_sys_path
    ):
        raise ExtractorError("isolated parser worker exposes ambient dependencies")
    if sys.path != [str(source_root), *base_sys_path]:
        raise ExtractorError("accepted parser changed the isolated import path")
    if sys.modules.get("yaml") is not yaml_stub:
        raise ExtractorError("accepted parser replaced the YAML deny-use stub")
    if (
        set(vars(yaml_stub))
        != {
            "__name__",
            "__doc__",
            "__package__",
            "__loader__",
            "__spec__",
            "YAMLError",
            "__getattr__",
        }
        or vars(yaml_stub).get("YAMLError") is not _DeniedYamlUseError
        or vars(yaml_stub).get("__getattr__") is not _reject_yaml_attribute
    ):
        raise ExtractorError("accepted parser changed the YAML deny-use stub")

    allowed_import_roots = [Path(entry) for entry in base_sys_path if entry]
    for name in sorted(set(sys.modules) - modules_before):
        module = sys.modules.get(name)
        if module is None:
            continue
        if name == "yaml":
            if module is not yaml_stub:
                raise ExtractorError("accepted parser replaced the YAML deny-use stub")
            continue
        if (
            name == "qiongli"
            or name == "bridges"
            or name.startswith("qiongli.")
            or name.startswith("bridges.")
        ):
            continue
        namespace = vars(module)
        spec = namespace.get("__spec__")
        origin = getattr(spec, "origin", None)
        if origin in {"built-in", "frozen"}:
            continue
        file_value = namespace.get("__file__")
        if file_value is None and name in STDLIB_FILELESS_MODULE_ALLOWLIST:
            continue
        if (
            not isinstance(file_value, str)
            or any(
                part.lower() in {"site-packages", "dist-packages"}
                for part in Path(file_value).parts
            )
            or not any(
                _path_is_within(Path(file_value), root)
                for root in allowed_import_roots
            )
        ):
            raise ExtractorError("accepted parser loaded an ambient dependency")


def _verify_module_provenance(source_root: Path) -> None:
    checked = 0
    for name, module in sorted(sys.modules.items()):
        if name != "qiongli" and name != "bridges" and not name.startswith("qiongli.") and not name.startswith("bridges."):
            continue
        file_value = getattr(module, "__file__", None)
        if isinstance(file_value, str):
            if not _path_is_within(Path(file_value), source_root):
                raise ExtractorError("accepted module escaped the materialized source tree")
            checked += 1
            continue
        path_value = getattr(module, "__path__", None)
        if path_value is not None:
            paths = [Path(value) for value in path_value]
            if not paths or any(not _path_is_within(path, source_root) for path in paths):
                raise ExtractorError("accepted package escaped the materialized source tree")
            checked += 1
            continue
        raise ExtractorError("accepted module has no verifiable source provenance")
    if checked == 0:
        raise ExtractorError("accepted module provenance could not be verified")


def _worker_artifact(control: Mapping[str, Any]) -> Mapping[str, Any]:
    _require_python_312()
    required = {"accepted_root", "source_root", "cwd", "write_root", "pyproject"}
    if set(control) != required or any(not isinstance(control[key], str) for key in required):
        raise ExtractorError("worker control is invalid")
    accepted_root = Path(control["accepted_root"]).resolve()
    source_root = Path(control["source_root"]).resolve()
    cwd = Path(control["cwd"]).resolve()
    write_root = Path(control["write_root"]).resolve()
    pyproject_path = Path(control["pyproject"]).resolve()
    if (
        not _path_is_within(accepted_root, write_root)
        or not _path_is_within(source_root, accepted_root)
        or not _path_is_within(cwd, write_root)
        or not _path_is_within(pyproject_path, accepted_root)
    ):
        raise ExtractorError("worker paths escape the temporary boundary")
    os.chdir(cwd)
    sys.dont_write_bytecode = True
    _install_worker_audit_hook(write_root)
    base_sys_path = tuple(sys.path)
    modules_before = set(sys.modules)
    if "yaml" in sys.modules:
        raise ExtractorError("isolated parser worker preloaded an ambient YAML module")
    yaml_stub = _make_yaml_deny_use_stub()
    sys.modules["yaml"] = yaml_stub
    sys.path.insert(0, str(source_root))
    sys.argv[:] = ["qiongli"]
    try:
        from qiongli import cli as qiongli_cli
        from bridges import mcp_cli
    except Exception as error:
        raise ExtractorError("accepted parser modules could not be imported") from error
    try:
        main_parser = qiongli_cli.build_parser()
        mcp_parser = mcp_cli.build_parser()
    except Exception as error:
        raise ExtractorError("accepted parser builders failed") from error
    _verify_module_provenance(source_root)
    _verify_dependency_isolation(
        source_root,
        base_sys_path,
        modules_before,
        yaml_stub,
    )
    main_surface = _extract_commands(
        main_parser,
        root_path=["qiongli"],
        cwd=str(cwd),
    )
    mounted_surface = _extract_commands(
        mcp_parser,
        root_path=["qiongli", "mcp"],
        cwd=str(cwd),
    )
    pyproject_bytes = pyproject_path.read_bytes()
    entrypoints = _parse_entrypoints(pyproject_bytes)
    coverage = {
        "canonical_command_count": (
            main_surface["counts"]["canonical_commands"]
            + mounted_surface["counts"]["canonical_commands"]
        ),
        "public_command_count": (
            main_surface["counts"]["public_commands"] + mounted_surface["counts"]["public_commands"]
        ),
        "console_entrypoint_count": len(entrypoints),
        "argument_action_count": (
            main_surface["counts"]["non_help_actions"]
            + mounted_surface["counts"]["non_help_actions"]
        ),
        "cwd_default_count": (
            main_surface["counts"]["cwd_defaults"] + mounted_surface["counts"]["cwd_defaults"]
        ),
        "static_semantics": "captured",
        "runtime_behavior_matrix": "incomplete",
        "exit_code_matrix": "incomplete",
        "error_matrix": "incomplete",
        "side_effect_matrix": "incomplete",
        "formatted_help_output": "incomplete",
        "json_output": "incomplete",
        "dry_run_semantics": "incomplete",
        "legacy_npm_compatibility_surface": "incomplete",
        "ctr_201": "in-progress",
        "fnd_202": "not-implemented",
        "completion_ready": False,
    }
    artifact: dict[str, Any] = {
        "$schema": ARTIFACT_SCHEMA,
        "schema_version": ARTIFACT_SCHEMA_VERSION,
        "record_type": ARTIFACT_RECORD_TYPE,
        "task_id": "CTR-201B",
        "status": "static-semantics-captured",
        "source": {
            "accepted_tag": ACCEPTED_TAG,
            "accepted_commit": ACCEPTED_COMMIT,
            "a8_manifest": {
                "path": MANIFEST_RELATIVE,
                "sha256": MANIFEST_SHA256,
            },
            "python_full_oracle": dict(PYTHON_ORACLE_BINDING),
            "package_tree": {
                "root": PYTHON_PACKAGE_ROOT,
                "file_count": EXPECTED_PACKAGE_FILE_COUNT,
                "tree_sha256": EXPECTED_PACKAGE_TREE_SHA256,
            },
            "blob_anchors": [
                dict(CLI_BLOB_BINDING),
                dict(MCP_CLI_BLOB_BINDING),
                dict(PYPROJECT_BLOB_ANCHOR),
            ],
        },
        "capture_contract": {
            "source_mode": "accepted-tag-git-blobs",
            "python_version": "python3.12",
            "help_mode": "authored-help-only",
            "environment_mode": "dual-environment",
            "environment_count": 2,
            "dynamic_default_policy": "symbolic-context-values",
            "callable_policy": "allowlisted-symbols-no-repr",
            "ambient_dependency_policy": "disabled-with-deny-use-stubs",
            "side_effect_policy": "read-only-no-network-no-process",
        },
        "console_entrypoints": entrypoints,
        "parser_roots": [
            {
                **EXPECTED_PARSER_ROOTS[0],
                "description": _normalize_nullable_text(main_parser.description),
                "subcommand_metadata": _parser_subcommand_metadata(main_parser),
            },
            {
                **EXPECTED_PARSER_ROOTS[1],
                "description": _normalize_nullable_text(mcp_parser.description),
                "subcommand_metadata": _parser_subcommand_metadata(mcp_parser),
            },
        ],
        "commands": [*main_surface["commands"], *mounted_surface["commands"]],
        "coverage": coverage,
    }
    artifact["integrity"] = {
        "algorithm": "sha256",
        "canonicalization": CANONICALIZATION,
        "payload_sha256": canonical_payload_sha256(artifact),
    }
    return artifact


def _worker_main(control_path: str) -> int:
    try:
        control_bytes = Path(control_path).read_bytes()
        control = _load_json_bytes(control_bytes)
        artifact = _worker_artifact(control)
        response = {"status": "pass", "artifact": artifact}
        sys.stdout.buffer.write(_canonical_json_bytes(response) + b"\n")
        return 0
    except Exception:
        response = {"status": "error", "code": "worker-failed"}
        sys.stdout.buffer.write(_canonical_json_bytes(response) + b"\n")
        return 2


def _build_parser() -> argparse.ArgumentParser:
    parser = _RedactedArgumentParser(
        description="Extract the accepted CTR-201B parser-declared CLI inventory."
    )
    parser.add_argument("--root", default=str(REPO_ROOT))
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--output")
    return parser


def _write_output(path: Path, artifact: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    rendered = json.dumps(artifact, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False) + "\n"
    path.write_text(rendered, encoding="utf-8", newline="\n")


def _load_checked_output(path: Path) -> Mapping[str, Any]:
    try:
        return _load_json_bytes(path.read_bytes())
    except OSError as error:
        raise InventoryMismatch("checked CLI inventory output is unavailable") from error


def _emit_result(*, json_mode: bool, status: str, exit_code: int, code: str, artifact: Mapping[str, Any] | None = None) -> None:
    if json_mode:
        payload: dict[str, Any] = {
            "status": status,
            "exit_code": exit_code,
            "code": code,
            "ctr_201": "in-progress",
            "fnd_202": "not-implemented",
        }
        if artifact is not None:
            payload["payload_sha256"] = artifact["integrity"]["payload_sha256"]
            payload["coverage"] = artifact["coverage"]
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
        return
    label = "PASS" if exit_code == 0 else "FAIL" if exit_code == 1 else "ERROR"
    print(f"[ctr-201b-cli] {label}: {code}")


def main(argv: Sequence[str] | None = None) -> int:
    arguments = list(sys.argv[1:] if argv is None else argv)
    json_mode = "--json" in arguments
    try:
        args = _build_parser().parse_args(arguments)
        root = Path(args.root)
        if not args.check and not args.output:
            raise CliUsageError("generation requires an explicit output path")
        artifact = extract_cli_inventory(root)
        output = Path(args.output) if args.output else root / DEFAULT_OUTPUT_RELATIVE
        if args.check:
            expected = _load_checked_output(output)
            if _canonical_json_bytes(expected) != _canonical_json_bytes(artifact):
                raise InventoryMismatch("checked CLI inventory output drifted")
            success_code = "accepted-cli-inventory-matches"
        else:
            _write_output(output, artifact)
            success_code = "accepted-cli-inventory-written"
        _emit_result(
            json_mode=bool(args.json),
            status="pass",
            exit_code=0,
            code=success_code,
            artifact=artifact,
        )
        return 0
    except InventoryMismatch:
        _emit_result(
            json_mode=json_mode,
            status="fail",
            exit_code=1,
            code="accepted-cli-inventory-mismatch",
        )
        return 1
    except (CliUsageError, ExtractorError, OSError, ValueError):
        _emit_result(
            json_mode=json_mode,
            status="error",
            exit_code=2,
            code="accepted-cli-inventory-unavailable",
        )
        return 2


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--_worker":
        raise SystemExit(_worker_main(sys.argv[2]))
    raise SystemExit(main())
