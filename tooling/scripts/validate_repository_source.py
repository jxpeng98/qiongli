#!/usr/bin/env python3
from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
from datetime import date
import hashlib
import json
import os
from pathlib import Path, PurePosixPath, PureWindowsPath
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable, Mapping, Sequence

import yaml

try:
    from tooling.scripts.release_version import parse_release_version
except ModuleNotFoundError:  # Direct execution from tooling/scripts.
    from release_version import parse_release_version


REPO_ROOT = Path(__file__).resolve().parents[2]
CONTRACT_RELATIVE = "tooling/quality/repository-source-code-contract.yaml"
BASELINE_RELATIVE = "tooling/quality/repository-source-code-baseline.json"
NATIVE_ROOT_RELATIVE = "packages/qiongli-native"
NATIVE_MANIFEST_RELATIVE = f"{NATIVE_ROOT_RELATIVE}/Cargo.toml"
NATIVE_LOCK_RELATIVE = f"{NATIVE_ROOT_RELATIVE}/Cargo.lock"
NATIVE_TOOLCHAIN_RELATIVE = f"{NATIVE_ROOT_RELATIVE}/rust-toolchain.toml"
NATIVE_CLIPPY_RELATIVE = f"{NATIVE_ROOT_RELATIVE}/clippy.toml"
PRODUCT_MEMBER = "apps/qiongli"
PRODUCT_MANIFEST_RELATIVE = f"{NATIVE_ROOT_RELATIVE}/{PRODUCT_MEMBER}/Cargo.toml"
PRODUCT_MAIN_RELATIVE = f"{NATIVE_ROOT_RELATIVE}/{PRODUCT_MEMBER}/src/main.rs"
REQUIRED_PROCESS_LINT_ATTRIBUTE = "#![forbid(clippy::disallowed_methods)]"

EXPECTED_RULES = {
    "RSC-BOUNDARY-001": "full-tree",
    "RSC-TOPOLOGY-001": "full-tree",
    "RSC-RUST-001": "changed-file",
    "RSC-UNSAFE-001": "full-tree",
    "RSC-DEPENDENCY-001": "changed-file",
    "RSC-SECURITY-001": "full-tree",
    "RSC-RUNTIME-001": "full-tree",
    "RSC-GENERATED-001": "delegated",
    "RSC-EXCEPTION-001": "full-tree",
}
FULL_TREE_RULES = tuple(
    rule_id for rule_id, enforcement in EXPECTED_RULES.items() if enforcement == "full-tree"
)
NATIVE_CHANGED_RULES = ("RSC-RUST-001", "RSC-DEPENDENCY-001")
DELEGATED_RULES = ("RSC-GENERATED-001",)
EXPECTED_RUST_COMMANDS = (
    "cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check",
    "cargo clippy --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked -- -D warnings",
    "cargo test --manifest-path packages/qiongli-native/Cargo.toml --workspace --all-targets --all-features --locked",
)
EXPECTED_PLANNED_PROFILES = {
    ("javascript-typescript", "planned-before-beta"),
    ("powershell", "planned-before-beta"),
    ("python", "planned-before-beta"),
    ("shell", "planned-before-beta"),
}
EXPECTED_TEXT_EXTENSIONS = {".json", ".md", ".rs", ".toml", ".txt", ".yaml", ".yml"}
BASELINE_ENTRY_KEYS = {
    "rule_id",
    "path",
    "fingerprint",
    "owner",
    "rationale",
    "compensating_check",
    "expires_on",
}
DISALLOWED_NATIVE_COMPONENTS = {"build", "dist", "out", "target"}
DISALLOWED_NATIVE_SUFFIXES = {
    ".a",
    ".dll",
    ".dylib",
    ".exe",
    ".o",
    ".pdb",
    ".rlib",
    ".rmeta",
    ".so",
}
LOCAL_PATH_PATTERN = re.compile(
    r"(?:/Users/[^/\s]+/|/home/[^/\s]+/|/root(?:/|\b)|"
    r"/Volumes/[^/\s]+/|/(?:tmp|var/tmp|var/folders|opt|usr/local|workspace|mnt)/|"
    r"/private/(?:tmp|var)/|(?<![A-Za-z0-9+.-])[A-Za-z]:[\\/]+|"
    r"\\\\[^\\/\s]+[\\/][^\\/\s]+|file://)",
    re.IGNORECASE,
)
SECRET_PATTERNS = (
    re.compile(r"-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY-----"),
    re.compile(r"\bgithub_pat_[A-Za-z0-9_]{24,}\b"),
    re.compile(r"\bgh[pousr]_[A-Za-z0-9]{30,}\b"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"\bxox[baprs]-[A-Za-z0-9-]{20,}\b"),
)
ALLOWED_PROCESS_REFERENCE_PATTERN = re.compile(
    r"\bstd\s*::\s*process\s*::\s*ExitCode\b"
)
PROCESS_LAUNCH_PATTERN = re.compile(
    r"(?:\b(?:std|tokio|async_process)\s*::\s*process\b|"
    r"\b(?:std|tokio|async_process)\s*::\s*\{[^}\n]*\bprocess\b|"
    r"\bprocess\s*::\s*(?:Command\b|\{[^}\n]*\bCommand\b)|"
    r"\bCommand\s*::\s*new\s*\()"
)
FINGERPRINT_PATTERN = re.compile(r"^[0-9a-f]{64}$")
MAX_NATIVE_TEXT_BYTES = 2 * 1024 * 1024
FORBIDDEN_CARGO_CONFIG_PATHS = {
    ".cargo/config",
    ".cargo/config.toml",
    f"{NATIVE_ROOT_RELATIVE}/.cargo/config",
    f"{NATIVE_ROOT_RELATIVE}/.cargo/config.toml",
}
FORBIDDEN_RUST_ENVIRONMENT = {
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTC_WRAPPER",
    "RUSTFLAGS",
}


class PolicyError(RuntimeError):
    pass


class PolicyUsageError(RuntimeError):
    pass


class PolicyArgumentParser(argparse.ArgumentParser):
    def error(self, message: str) -> None:
        raise PolicyUsageError(message)


@dataclass(frozen=True)
class Finding:
    rule_id: str
    severity: str
    path: str
    message: str
    remediation: str
    fingerprint: str


@dataclass(frozen=True)
class ValidationResult:
    mode: str
    base_ref: str | None
    evaluated_paths: tuple[str, ...]
    applicable_rule_ids: tuple[str, ...]
    findings: tuple[Finding, ...]

    @property
    def status(self) -> str:
        return "failed" if self.findings else "passed"

    def as_report(self) -> dict[str, Any]:
        return {
            "schema_version": "1.0",
            "status": self.status,
            "mode": self.mode,
            "base_ref": self.base_ref,
            "evaluated_paths": list(self.evaluated_paths),
            "applicable_rule_ids": list(self.applicable_rule_ids),
            "findings": [asdict(finding) for finding in self.findings],
        }


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


def _require_git_success(result: subprocess.CompletedProcess[bytes], operation: str) -> bytes:
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        raise PolicyError(f"{operation} failed: {detail}")
    return result.stdout


def is_canonical_repository_path(path: str) -> bool:
    if not path or "\\" in path or "\x00" in path:
        return False
    posix = PurePosixPath(path)
    windows = PureWindowsPath(path)
    if posix.is_absolute() or windows.is_absolute():
        return False
    if ".." in posix.parts or ".." in windows.parts:
        return False
    return str(posix) == path


def _require_canonical_paths(paths: Iterable[str]) -> list[str]:
    values = sorted(set(paths))
    invalid = [path for path in values if not is_canonical_repository_path(path)]
    if invalid:
        raise PolicyError(
            "changed paths must be canonical repository-relative POSIX paths: "
            + ", ".join(repr(path) for path in invalid)
        )
    return values


def _decode_git_paths(output: bytes, operation: str) -> list[str]:
    try:
        values = [value.decode("utf-8") for value in output.split(b"\0") if value]
    except UnicodeDecodeError as error:
        raise PolicyError(f"{operation} returned a non-UTF-8 repository path") from error
    return _require_canonical_paths(values)


def changed_paths_from_git(repo_root: Path, base_ref: str) -> list[str]:
    verify = _git(repo_root, ["rev-parse", "--verify", f"{base_ref}^{{commit}}"])
    _require_git_success(verify, f"verify comparison base {base_ref}")
    result = _git(
        repo_root,
        ["diff", "--name-only", "--no-renames", "-z", f"{base_ref}...HEAD"],
    )
    output = _require_git_success(result, f"compare {base_ref} with HEAD")
    return _decode_git_paths(output, "compare changed paths")


def repository_paths(repo_root: Path) -> list[str]:
    result = _git(
        repo_root,
        ["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
    )
    output = _require_git_success(result, "enumerate repository files")
    return _decode_git_paths(output, "enumerate repository files")


def tracked_modes(repo_root: Path) -> dict[str, str]:
    result = _git(repo_root, ["ls-files", "-s", "-z"])
    output = _require_git_success(result, "enumerate tracked file modes")
    modes: dict[str, str] = {}
    for record in output.split(b"\0"):
        if not record:
            continue
        try:
            metadata, raw_path = record.split(b"\t", 1)
            mode = metadata.split(b" ", 1)[0].decode("ascii")
            path = raw_path.decode("utf-8")
        except (UnicodeDecodeError, ValueError) as error:
            raise PolicyError("git returned an invalid tracked-file record") from error
        modes[path] = mode
    return modes


def _load_yaml(path: Path) -> Mapping[str, Any]:
    try:
        value = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, yaml.YAMLError) as error:
        raise PolicyError(f"cannot load policy contract {path}: {error}") from error
    if not isinstance(value, Mapping):
        raise PolicyError(f"policy contract {path} must contain an object")
    return value


def _load_json(path: Path) -> Mapping[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise PolicyError(f"cannot load policy baseline {path}: {error}") from error
    if not isinstance(value, Mapping):
        raise PolicyError(f"policy baseline {path} must contain an object")
    return value


def load_contract(repo_root: Path) -> Mapping[str, Any]:
    contract = _load_yaml(repo_root / CONTRACT_RELATIVE)
    expected_top_level = {
        "schema_version",
        "contract_id",
        "title",
        "status",
        "scope",
        "language_gates",
        "delegated_gates",
        "rules",
        "planned_profiles",
    }
    if set(contract) != expected_top_level:
        raise PolicyError("RC1 contract top-level keys differ from the version 1.0 schema")
    if contract.get("schema_version") != "1.0" or contract.get("contract_id") != "RC1":
        raise PolicyError("RC1 contract identity must be schema 1.0 and contract_id RC1")
    if contract.get("status") != "native-foundation-enforcing":
        raise PolicyError("RC1 contract status must be native-foundation-enforcing")

    scope = contract.get("scope")
    if not isinstance(scope, Mapping):
        raise PolicyError("RC1 scope must be an object")
    if scope.get("native_root") != NATIVE_ROOT_RELATIVE:
        raise PolicyError(f"RC1 native_root must be {NATIVE_ROOT_RELATIVE}")
    if scope.get("baseline_path") != BASELINE_RELATIVE:
        raise PolicyError(f"RC1 baseline_path must be {BASELINE_RELATIVE}")
    extensions = scope.get("tracked_text_extensions")
    if not isinstance(extensions, list) or set(extensions) != EXPECTED_TEXT_EXTENSIONS:
        raise PolicyError("RC1 tracked_text_extensions differ from the native foundation set")

    rules = contract.get("rules")
    if not isinstance(rules, list):
        raise PolicyError("RC1 rules must be an array")
    actual_rules: dict[str, str] = {}
    required_rule_keys = {
        "id",
        "title",
        "enforcement",
        "severity",
        "description",
        "remediation",
    }
    for index, rule in enumerate(rules):
        if not isinstance(rule, Mapping) or set(rule) != required_rule_keys:
            raise PolicyError(f"RC1 rules[{index}] does not match the rule schema")
        rule_id = rule.get("id")
        enforcement = rule.get("enforcement")
        if not isinstance(rule_id, str) or rule_id in actual_rules:
            raise PolicyError(f"RC1 rules[{index}] has a missing or duplicate id")
        if rule.get("severity") != "error":
            raise PolicyError(f"{rule_id} severity must be error in Phase 1")
        for field in ("title", "description", "remediation"):
            if not isinstance(rule.get(field), str) or len(rule[field].strip()) < 12:
                raise PolicyError(f"{rule_id} {field} is missing or too short")
        actual_rules[rule_id] = str(enforcement)
    if actual_rules != EXPECTED_RULES:
        raise PolicyError("RC1 active rule ids or enforcement modes differ from Phase 1")

    language_gates = contract.get("language_gates")
    rust = language_gates.get("rust") if isinstance(language_gates, Mapping) else None
    if not isinstance(rust, Mapping):
        raise PolicyError("RC1 must define the Rust language gate")
    if rust.get("trigger_prefix") != f"{NATIVE_ROOT_RELATIVE}/":
        raise PolicyError("RC1 Rust trigger_prefix is invalid")
    commands = rust.get("commands")
    if not isinstance(commands, list) or tuple(commands) != EXPECTED_RUST_COMMANDS:
        raise PolicyError("RC1 Rust commands must match the accepted locked gate sequence")

    delegated = contract.get("delegated_gates")
    generated = delegated.get("generated_output") if isinstance(delegated, Mapping) else None
    if not isinstance(generated, Mapping):
        raise PolicyError("RC1 must declare the generated-output delegated gate")
    if generated.get("rule_id") != "RSC-GENERATED-001":
        raise PolicyError("RC1 generated-output delegated rule id is invalid")
    if generated.get("command") != (
        "python scripts/check_generated_payload_edits.py --base-ref <event-base>"
    ):
        raise PolicyError("RC1 generated-output command is invalid")
    if generated.get("canonical_roots_module") != (
        "packages/python-qiongli/src/qiongli/source_layout.py"
    ):
        raise PolicyError("RC1 generated-output canonical roots module is invalid")

    profiles = contract.get("planned_profiles")
    if not isinstance(profiles, list):
        raise PolicyError("RC1 planned_profiles must be an array")
    actual_profiles = {
        (item.get("language"), item.get("enforcement"))
        for item in profiles
        if isinstance(item, Mapping)
    }
    if actual_profiles != EXPECTED_PLANNED_PROFILES or len(profiles) != len(actual_profiles):
        raise PolicyError("RC1 planned language profiles differ from the Phase 2 handoff")
    return contract


def load_baseline(repo_root: Path) -> Mapping[str, Any]:
    return _load_json(repo_root / BASELINE_RELATIVE)


def _canonical_blob_bytes(repo_root: Path, relative: str) -> bytes:
    path = repo_root / PurePosixPath(relative)
    result = _git(repo_root, ["show", f"HEAD:{relative}"])
    if result.returncode == 0:
        staged = _git(repo_root, ["diff", "--cached", "--quiet", "HEAD", "--", relative])
        unstaged = _git(repo_root, ["diff", "--quiet", "--", relative])
        if staged.returncode == 0 and unstaged.returncode == 0:
            return result.stdout
        index = _git(repo_root, ["show", f":{relative}"])
        try:
            worktree = path.read_bytes()
        except OSError:
            worktree = b"<missing>"
        index_bytes = index.stdout if index.returncode == 0 else b"<missing>"
        return (
            b"<dirty-source>\0<head>\0"
            + result.stdout
            + b"\0<index>\0"
            + index_bytes
            + b"\0<worktree>\0"
            + worktree
        )
    try:
        return path.read_bytes()
    except OSError:
        return b"<missing>"


def finding_fingerprint(repo_root: Path, rule_id: str, relative: str) -> str:
    blob_digest = hashlib.sha256(_canonical_blob_bytes(repo_root, relative)).hexdigest()
    value = f"{rule_id}\0{relative}\0{blob_digest}".encode("utf-8")
    return hashlib.sha256(value).hexdigest()


def _finding(repo_root: Path, rule_id: str, path: str, message: str) -> Finding:
    remediations = {
        rule.get("id"): rule.get("remediation")
        for rule in load_contract(repo_root).get("rules", [])
        if isinstance(rule, Mapping)
    }
    return Finding(
        rule_id=rule_id,
        severity="error",
        path=path,
        message=message,
        remediation=str(remediations[rule_id]),
        fingerprint=finding_fingerprint(repo_root, rule_id, path),
    )


def _findings_for_baseline(
    repo_root: Path, baseline: Mapping[str, Any], *, today: date
) -> list[Finding]:
    findings: list[Finding] = []
    if set(baseline) != {"schema_version", "contract_id", "scope", "findings"}:
        return [
            _finding(
                repo_root,
                "RSC-EXCEPTION-001",
                BASELINE_RELATIVE,
                "baseline keys differ from the version 1.0 schema",
            )
        ]
    if (
        baseline.get("schema_version") != "1.0"
        or baseline.get("contract_id") != "RC1"
        or baseline.get("scope") != "native-foundation"
    ):
        findings.append(
            _finding(
                repo_root,
                "RSC-EXCEPTION-001",
                BASELINE_RELATIVE,
                "baseline identity does not match RC1 native-foundation",
            )
        )
    entries = baseline.get("findings")
    if not isinstance(entries, list):
        findings.append(
            _finding(
                repo_root,
                "RSC-EXCEPTION-001",
                BASELINE_RELATIVE,
                "baseline findings must be an array",
            )
        )
        return findings
    seen: set[tuple[str, str]] = set()
    for index, entry in enumerate(entries):
        label = f"{BASELINE_RELATIVE}#findings[{index}]"
        if not isinstance(entry, Mapping) or set(entry) != BASELINE_ENTRY_KEYS:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"baseline entry {index} does not match the exact exception schema",
                )
            )
            continue
        rule_id = entry.get("rule_id")
        path = entry.get("path")
        if rule_id not in EXPECTED_RULES or not isinstance(path, str):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} has an unknown rule or non-string path",
                )
            )
            continue
        if not is_canonical_repository_path(path):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} path is not canonical repository-relative POSIX syntax",
                )
            )
            continue
        key = (rule_id, path)
        if key in seen:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} duplicates an existing rule and path",
                )
            )
        seen.add(key)
        if path == NATIVE_ROOT_RELATIVE or path.startswith(f"{NATIVE_ROOT_RELATIVE}/"):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} attempts to suppress new native-foundation debt",
                )
            )
        referenced = repo_root / PurePosixPath(path)
        if any(character in path for character in "*?[]") or not referenced.is_file() or referenced.is_symlink():
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} must reference one existing regular file without glob syntax",
                )
            )
        for field in ("owner", "rationale", "compensating_check"):
            if not isinstance(entry.get(field), str) or len(entry[field].strip()) < 4:
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-EXCEPTION-001",
                        BASELINE_RELATIVE,
                        f"{label} {field} is missing or too short",
                    )
                )
        expiry = entry.get("expires_on")
        try:
            expiry_date = date.fromisoformat(expiry) if isinstance(expiry, str) else None
        except ValueError:
            expiry_date = None
        if expiry_date is None:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} expires_on is not a valid ISO date",
                )
            )
        elif expiry_date <= today:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} is expired",
                )
            )
        fingerprint = entry.get("fingerprint")
        if not isinstance(fingerprint, str) or not FINGERPRINT_PATTERN.fullmatch(fingerprint):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} fingerprint must be a lowercase SHA-256 digest",
                )
            )
        elif fingerprint != finding_fingerprint(repo_root, rule_id, path):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-EXCEPTION-001",
                    BASELINE_RELATIVE,
                    f"{label} fingerprint no longer matches the referenced source blob",
                )
            )
    return findings


def _read_toml(repo_root: Path, relative: str, rule_id: str) -> tuple[Mapping[str, Any] | None, list[Finding]]:
    path = repo_root / PurePosixPath(relative)
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError):
        return None, [
            _finding(repo_root, rule_id, relative, "required TOML file is missing or invalid")
        ]
    if not isinstance(value, Mapping):
        return None, [_finding(repo_root, rule_id, relative, "TOML root must be an object")]
    return value, []


def _native_repository_paths(paths: Iterable[str]) -> list[str]:
    return sorted(
        path
        for path in paths
        if path == NATIVE_ROOT_RELATIVE or path.startswith(f"{NATIVE_ROOT_RELATIVE}/")
    )


def _boundary_findings(
    repo_root: Path, paths: Sequence[str], modes: Mapping[str, str]
) -> list[Finding]:
    findings: list[Finding] = []
    native_root = (repo_root / NATIVE_ROOT_RELATIVE).resolve()
    for relative in sorted(set(paths) & FORBIDDEN_CARGO_CONFIG_PATHS):
        findings.append(
            _finding(
                repo_root,
                "RSC-RUNTIME-001",
                relative,
                "repository Cargo configuration is forbidden during B2a",
            )
        )
    for relative in _native_repository_paths(paths):
        rel_inside = PurePosixPath(relative).relative_to(NATIVE_ROOT_RELATIVE)
        path = repo_root / PurePosixPath(relative)
        tracked_mode = modes.get(relative)
        untracked_executable = False
        if tracked_mode is None and path.is_file():
            try:
                untracked_executable = bool(path.stat().st_mode & 0o111)
            except OSError:
                untracked_executable = True
        if (
            (tracked_mode is not None and tracked_mode != "100644")
            or untracked_executable
            or path.is_symlink()
            or any(component in DISALLOWED_NATIVE_COMPONENTS for component in rel_inside.parts)
            or path.suffix.lower() in DISALLOWED_NATIVE_SUFFIXES
        ):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-BOUNDARY-001",
                    relative,
                    "native source contains a symlink or generated/build artifact",
                )
            )
    for relative in sorted(path for path in paths if path.endswith("Cargo.toml")):
        if not relative.startswith(f"{NATIVE_ROOT_RELATIVE}/") and relative != NATIVE_MANIFEST_RELATIVE:
            continue
        manifest, parse_findings = _read_toml(repo_root, relative, "RSC-DEPENDENCY-001")
        findings.extend(parse_findings)
        if manifest is None:
            continue
        manifest_path = (repo_root / PurePosixPath(relative)).parent
        for dependency_path in _dependency_paths(manifest):
            posix_dependency = PurePosixPath(dependency_path)
            windows_dependency = PureWindowsPath(dependency_path)
            if (
                "\\" in dependency_path
                or posix_dependency.is_absolute()
                or windows_dependency.is_absolute()
            ):
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-BOUNDARY-001",
                        relative,
                        "Cargo path dependency must use portable workspace-relative POSIX syntax",
                    )
                )
                continue
            candidate = Path(dependency_path)
            resolved = (manifest_path / candidate).resolve()
            try:
                resolved.relative_to(native_root)
            except ValueError:
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-BOUNDARY-001",
                        relative,
                        "Cargo path dependency resolves outside packages/qiongli-native",
                    )
                )
    return findings


def _dependency_paths(value: Mapping[str, Any]) -> list[str]:
    paths: list[str] = []
    for key, child in value.items():
        if key in {"dependencies", "dev-dependencies", "build-dependencies"} and isinstance(
            child, Mapping
        ):
            for dependency in child.values():
                if isinstance(dependency, Mapping) and isinstance(dependency.get("path"), str):
                    paths.append(dependency["path"])
        if isinstance(child, Mapping):
            paths.extend(_dependency_paths(child))
    return paths


def _topology_and_dependency_findings(
    repo_root: Path, paths: Sequence[str]
) -> list[Finding]:
    findings: list[Finding] = []
    workspace, workspace_errors = _read_toml(
        repo_root, NATIVE_MANIFEST_RELATIVE, "RSC-TOPOLOGY-001"
    )
    findings.extend(workspace_errors)
    if workspace is None:
        return findings
    workspace_table = workspace.get("workspace")
    if not isinstance(workspace_table, Mapping):
        return [
            *findings,
            _finding(
                repo_root,
                "RSC-TOPOLOGY-001",
                NATIVE_MANIFEST_RELATIVE,
                "native manifest must define a workspace",
            ),
        ]

    expected_workspace = {
        "resolver": "3",
        "members": [PRODUCT_MEMBER],
        "default-members": [PRODUCT_MEMBER],
    }
    for field, expected in expected_workspace.items():
        if workspace_table.get(field) != expected:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-TOPOLOGY-001",
                    NATIVE_MANIFEST_RELATIVE,
                    f"workspace {field} must be {expected!r} during B2a",
                )
            )

    native_version: str | None = None
    native_channel: str | None = None
    package = workspace_table.get("package")
    expected_package = {
        "edition": "2024",
        "rust-version": "1.97",
        "license": "MIT",
        "publish": False,
    }
    if not isinstance(package, Mapping):
        findings.append(
            _finding(
                repo_root,
                "RSC-DEPENDENCY-001",
                NATIVE_MANIFEST_RELATIVE,
                "workspace package metadata is missing",
            )
        )
    else:
        raw_version = package.get("version")
        if not isinstance(raw_version, str):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-DEPENDENCY-001",
                    NATIVE_MANIFEST_RELATIVE,
                    "workspace package version must be a supported native 2.x version",
                )
            )
        else:
            try:
                identity = parse_release_version(raw_version)
            except ValueError as error:
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-DEPENDENCY-001",
                        NATIVE_MANIFEST_RELATIVE,
                        f"workspace package version is invalid: {error}",
                    )
                )
            else:
                if identity.release_line != "native-2x":
                    findings.append(
                        _finding(
                            repo_root,
                            "RSC-DEPENDENCY-001",
                            NATIVE_MANIFEST_RELATIVE,
                            "workspace package version must belong to the native 2.x release line",
                        )
                    )
                else:
                    native_version = identity.version
                    native_channel = identity.channel
        for field, expected in expected_package.items():
            if package.get(field) != expected:
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-DEPENDENCY-001",
                        NATIVE_MANIFEST_RELATIVE,
                        f"workspace package {field} must be {expected!r}",
                    )
                )

    metadata = workspace_table.get("metadata")
    qiongli = metadata.get("qiongli") if isinstance(metadata, Mapping) else None
    if not isinstance(qiongli, Mapping) or qiongli.get("product") != "qiongli":
        findings.append(
            _finding(
                repo_root,
                "RSC-TOPOLOGY-001",
                NATIVE_MANIFEST_RELATIVE,
                "workspace metadata must identify product qiongli",
            )
        )
    else:
        metadata_channel = qiongli.get("channel")
        if native_channel is not None and metadata_channel != native_channel:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-TOPOLOGY-001",
                    NATIVE_MANIFEST_RELATIVE,
                    "workspace metadata channel must match the channel implied by "
                    f"version {native_version!r}: expected {native_channel!r}",
                )
            )
        elif native_channel is None and metadata_channel not in {"alpha", "beta", "stable"}:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-TOPOLOGY-001",
                    NATIVE_MANIFEST_RELATIVE,
                    "workspace metadata channel must be alpha, beta, or stable",
                )
            )

    lints = workspace_table.get("lints")
    rust_lints = lints.get("rust") if isinstance(lints, Mapping) else None
    if not isinstance(rust_lints, Mapping) or rust_lints.get("unsafe_code") != "forbid":
        findings.append(
            _finding(
                repo_root,
                "RSC-UNSAFE-001",
                NATIVE_MANIFEST_RELATIVE,
                "workspace rust lint unsafe_code must be forbid",
            )
        )
    clippy_lints = lints.get("clippy") if isinstance(lints, Mapping) else None
    if (
        not isinstance(clippy_lints, Mapping)
        or clippy_lints.get("disallowed_methods") != "deny"
    ):
        findings.append(
            _finding(
                repo_root,
                "RSC-RUNTIME-001",
                NATIVE_MANIFEST_RELATIVE,
                "workspace clippy lint disallowed_methods must be deny",
            )
        )
    workspace_dependencies = workspace_table.get("dependencies")
    if isinstance(workspace_dependencies, Mapping) and workspace_dependencies:
        findings.append(
            _finding(
                repo_root,
                "RSC-DEPENDENCY-001",
                NATIVE_MANIFEST_RELATIVE,
                "B2a workspace dependencies must remain empty",
            )
        )

    manifest_paths = sorted(
        path
        for path in paths
        if path != NATIVE_MANIFEST_RELATIVE
        and path.startswith(f"{NATIVE_ROOT_RELATIVE}/")
        and path.endswith("/Cargo.toml")
    )
    expected_manifest_paths = [PRODUCT_MANIFEST_RELATIVE]
    if manifest_paths != expected_manifest_paths:
        findings.append(
            _finding(
                repo_root,
                "RSC-TOPOLOGY-001",
                NATIVE_MANIFEST_RELATIVE,
                "B2a must contain exactly the apps/qiongli package manifest",
            )
        )

    product, product_errors = _read_toml(
        repo_root, PRODUCT_MANIFEST_RELATIVE, "RSC-TOPOLOGY-001"
    )
    findings.extend(product_errors)
    if product is not None:
        product_package = product.get("package")
        inherited_fields = (
            "version",
            "edition",
            "rust-version",
            "authors",
            "license",
            "repository",
            "publish",
        )
        if not isinstance(product_package, Mapping) or product_package.get("name") != "qiongli":
            findings.append(
                _finding(
                    repo_root,
                    "RSC-TOPOLOGY-001",
                    PRODUCT_MANIFEST_RELATIVE,
                    "product package name must be qiongli",
                )
            )
        else:
            if any(
                product_package.get(field) != {"workspace": True}
                for field in inherited_fields
            ):
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-DEPENDENCY-001",
                        PRODUCT_MANIFEST_RELATIVE,
                        "product identity fields must inherit workspace package metadata",
                    )
                )
            if product_package.get("autobins") is False:
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-TOPOLOGY-001",
                        PRODUCT_MANIFEST_RELATIVE,
                        "product package cannot disable Cargo automatic binary discovery",
                    )
                )
            if "build" in product_package:
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-RUNTIME-001",
                        PRODUCT_MANIFEST_RELATIVE,
                        "B2a product must not declare a Cargo build script",
                    )
                )
        member_lints = product.get("lints")
        if not isinstance(member_lints, Mapping) or member_lints.get("workspace") is not True:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-UNSAFE-001",
                    PRODUCT_MANIFEST_RELATIVE,
                    "product member must inherit workspace lints",
                )
            )
        for dependency_table in ("dependencies", "dev-dependencies", "build-dependencies"):
            dependencies = product.get(dependency_table)
            if isinstance(dependencies, Mapping) and dependencies:
                findings.append(
                    _finding(
                        repo_root,
                        "RSC-DEPENDENCY-001",
                        PRODUCT_MANIFEST_RELATIVE,
                        f"B2a product {dependency_table} must remain empty",
                    )
                )
        if product.get("bin") not in (None, []):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-TOPOLOGY-001",
                    PRODUCT_MANIFEST_RELATIVE,
                    "B2a product must use the single default qiongli binary target",
                )
            )

    main_paths = sorted(
        path
        for path in paths
        if path.startswith(f"{NATIVE_ROOT_RELATIVE}/") and path.endswith("/src/main.rs")
    )
    if main_paths != [PRODUCT_MAIN_RELATIVE]:
        findings.append(
            _finding(
                repo_root,
                "RSC-TOPOLOGY-001",
                NATIVE_MANIFEST_RELATIVE,
                "B2a must contain exactly one product main.rs at apps/qiongli",
            )
        )
    auto_bin_paths = sorted(
        path
        for path in paths
        if path.startswith(f"{NATIVE_ROOT_RELATIVE}/") and "/src/bin/" in path
    )
    if auto_bin_paths:
        findings.append(
            _finding(
                repo_root,
                "RSC-TOPOLOGY-001",
                auto_bin_paths[0],
                "B2a forbids additional Cargo auto-discovered binary targets under src/bin",
            )
        )
    build_scripts = sorted(
        path
        for path in paths
        if path.startswith(f"{NATIVE_ROOT_RELATIVE}/") and path.endswith("/build.rs")
    )
    if build_scripts:
        findings.append(
            _finding(
                repo_root,
                "RSC-RUNTIME-001",
                build_scripts[0],
                "B2a forbids Cargo build scripts",
            )
        )

    toolchain, toolchain_errors = _read_toml(
        repo_root, NATIVE_TOOLCHAIN_RELATIVE, "RSC-DEPENDENCY-001"
    )
    findings.extend(toolchain_errors)
    toolchain_table = toolchain.get("toolchain") if isinstance(toolchain, Mapping) else None
    if not isinstance(toolchain_table, Mapping) or toolchain_table.get("channel") != "1.97.0":
        findings.append(
            _finding(
                repo_root,
                "RSC-DEPENDENCY-001",
                NATIVE_TOOLCHAIN_RELATIVE,
                "native toolchain channel must be pinned to 1.97.0",
            )
        )

    clippy_config, clippy_errors = _read_toml(
        repo_root, NATIVE_CLIPPY_RELATIVE, "RSC-RUNTIME-001"
    )
    findings.extend(clippy_errors)
    disallowed_methods = (
        clippy_config.get("disallowed-methods")
        if isinstance(clippy_config, Mapping)
        else None
    )
    if not isinstance(disallowed_methods, list) or not any(
        isinstance(item, Mapping)
        and item.get("path") == "std::process::Command::new"
        for item in disallowed_methods
    ):
        findings.append(
            _finding(
                repo_root,
                "RSC-RUNTIME-001",
                NATIVE_CLIPPY_RELATIVE,
                "Clippy must disallow the resolved std::process::Command::new method",
            )
        )

    lock, lock_errors = _read_toml(repo_root, NATIVE_LOCK_RELATIVE, "RSC-DEPENDENCY-001")
    findings.extend(lock_errors)
    packages = lock.get("package") if isinstance(lock, Mapping) else None
    product_packages = (
        [
            item
            for item in packages
            if isinstance(item, Mapping) and item.get("name") == "qiongli"
        ]
        if isinstance(packages, list)
        else []
    )
    if native_version is not None and (
        len(product_packages) != 1 or product_packages[0].get("version") != native_version
    ):
        findings.append(
            _finding(
                repo_root,
                "RSC-DEPENDENCY-001",
                NATIVE_LOCK_RELATIVE,
                f"Cargo.lock must bind exactly one qiongli package at {native_version}",
            )
        )
    elif native_version is None and len(product_packages) != 1:
        findings.append(
            _finding(
                repo_root,
                "RSC-DEPENDENCY-001",
                NATIVE_LOCK_RELATIVE,
                "Cargo.lock must bind exactly one qiongli package",
            )
        )
    return findings


def _security_findings(repo_root: Path, paths: Sequence[str]) -> list[Finding]:
    findings: list[Finding] = []
    for relative in _native_repository_paths(paths):
        path = repo_root / PurePosixPath(relative)
        if not path.is_file() or path.is_symlink():
            continue
        try:
            size = path.stat().st_size
        except OSError:
            size = MAX_NATIVE_TEXT_BYTES + 1
        if size > MAX_NATIVE_TEXT_BYTES:
            findings.append(
                _finding(
                    repo_root,
                    "RSC-BOUNDARY-001",
                    relative,
                    "native source exceeds the bootstrap text-file size limit",
                )
            )
            continue
        try:
            content = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-BOUNDARY-001",
                    relative,
                    "tracked native text is not readable UTF-8",
                )
            )
            continue
        if LOCAL_PATH_PATTERN.search(content):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-SECURITY-001",
                    relative,
                    "machine-specific absolute path detected in native source",
                )
            )
        if any(pattern.search(content) for pattern in SECRET_PATTERNS):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-SECURITY-001",
                    relative,
                    "high-confidence secret signature detected in native source",
                )
            )
        parts = PurePosixPath(relative).parts
        is_production_rust = (
            path.suffix.lower() == ".rs"
            and "src" in parts
            and not any(part in {"benches", "examples", "tests"} for part in parts)
        ) or path.name == "build.rs"
        runtime_content = ALLOWED_PROCESS_REFERENCE_PATTERN.sub("", content)
        if is_production_rust and PROCESS_LAUNCH_PATTERN.search(runtime_content):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-RUNTIME-001",
                    relative,
                    "B2a production source may not launch external processes",
                )
            )
        if relative == PRODUCT_MAIN_RELATIVE and not content.startswith(
            f"{REQUIRED_PROCESS_LINT_ATTRIBUTE}\n"
        ):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-RUNTIME-001",
                    relative,
                    "product crate root must forbid the resolved process-launch lint",
                )
            )
    return findings


def _rust_environment_findings(repo_root: Path) -> list[Finding]:
    findings: list[Finding] = []
    names = set(FORBIDDEN_RUST_ENVIRONMENT)
    names.update(
        name
        for name in os.environ
        if name.startswith("CARGO_TARGET_") and name.endswith("_RUSTFLAGS")
    )
    for name in sorted(names):
        if os.environ.get(name):
            findings.append(
                _finding(
                    repo_root,
                    "RSC-RUNTIME-001",
                    NATIVE_MANIFEST_RELATIVE,
                    f"Rust compiler override environment variable {name} must be unset",
                )
            )
    return findings


def applicable_rule_ids(changed_paths: Sequence[str]) -> tuple[str, ...]:
    selected = set(FULL_TREE_RULES) | set(DELEGATED_RULES)
    if any(
        path == NATIVE_ROOT_RELATIVE or path.startswith(f"{NATIVE_ROOT_RELATIVE}/")
        for path in changed_paths
    ):
        selected.update(NATIVE_CHANGED_RULES)
    return tuple(sorted(selected))


def validate_repository(
    repo_root: Path,
    changed_paths: Sequence[str],
    *,
    mode: str,
    base_ref: str | None = None,
    today: date | None = None,
) -> ValidationResult:
    repo_root = repo_root.resolve()
    changed = _require_canonical_paths(changed_paths)
    load_contract(repo_root)
    baseline = load_baseline(repo_root)
    paths = repository_paths(repo_root)
    modes = tracked_modes(repo_root)
    findings = [
        *_findings_for_baseline(repo_root, baseline, today=today or date.today()),
        *_boundary_findings(repo_root, paths, modes),
        *_topology_and_dependency_findings(repo_root, paths),
        *_security_findings(repo_root, paths),
        *_rust_environment_findings(repo_root),
    ]
    unique = {
        (finding.rule_id, finding.path, finding.message, finding.fingerprint): finding
        for finding in findings
    }
    ordered = tuple(
        sorted(
            unique.values(),
            key=lambda item: (item.rule_id, item.path, item.message, item.fingerprint),
        )
    )
    evaluated = (
        sorted(
            {
                *_native_repository_paths(paths),
                CONTRACT_RELATIVE,
                BASELINE_RELATIVE,
            }
        )
        if mode == "full-tree"
        else changed
    )
    return ValidationResult(
        mode=mode,
        base_ref=base_ref,
        evaluated_paths=tuple(evaluated),
        applicable_rule_ids=applicable_rule_ids(changed),
        findings=ordered,
    )


def _error_report(message: str, *, mode: str, base_ref: str | None) -> dict[str, Any]:
    return {
        "schema_version": "1.0",
        "status": "error",
        "mode": mode,
        "base_ref": base_ref,
        "evaluated_paths": [],
        "applicable_rule_ids": [],
        "findings": [],
        "errors": [message],
    }


def main(argv: Sequence[str] | None = None) -> int:
    raw_argv = list(argv) if argv is not None else sys.argv[1:]
    parser = PolicyArgumentParser(
        description="Validate the RC1 repository source policy for the native foundation."
    )
    parser.add_argument("--root", type=Path, default=REPO_ROOT)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--base-ref", help="Compare the merge base with HEAD.")
    mode.add_argument(
        "--changed-file",
        action="append",
        help="Explicit changed repository path. May be repeated.",
    )
    mode.add_argument("--full-tree", action="store_true", help="Evaluate the repository tree.")
    parser.add_argument("--json", action="store_true", help="Emit one machine-readable report.")
    json_requested = "--json" in raw_argv
    try:
        args = parser.parse_args(raw_argv)
    except PolicyUsageError as error:
        if "--full-tree" in raw_argv:
            mode_hint = "full-tree"
        elif "--base-ref" in raw_argv or "--changed-file" in raw_argv:
            mode_hint = "changed-files"
        else:
            mode_hint = "unspecified"
        if json_requested:
            print(
                json.dumps(
                    _error_report(str(error), mode=mode_hint, base_ref=None),
                    indent=2,
                    sort_keys=True,
                )
            )
        else:
            print(parser.format_usage().rstrip(), file=sys.stderr)
            print(f"[repository-source] ERROR: {error}", file=sys.stderr)
        return 2

    selected_mode = "full-tree" if args.full_tree else "changed-files"
    try:
        if args.base_ref:
            changed = changed_paths_from_git(args.root.resolve(), args.base_ref)
        elif args.changed_file:
            changed = _require_canonical_paths(args.changed_file)
        else:
            changed = repository_paths(args.root.resolve())
        result = validate_repository(
            args.root,
            changed,
            mode=selected_mode,
            base_ref=args.base_ref,
        )
    except PolicyError as error:
        if args.json:
            print(
                json.dumps(
                    _error_report(str(error), mode=selected_mode, base_ref=args.base_ref),
                    indent=2,
                    sort_keys=True,
                )
            )
        else:
            print(f"[repository-source] ERROR: {error}", file=sys.stderr)
        return 2

    if args.json:
        print(json.dumps(result.as_report(), indent=2, sort_keys=True))
    elif result.findings:
        for finding in result.findings:
            print(
                f"[repository-source] FAIL {finding.rule_id} {finding.path}: "
                f"{finding.message}",
                file=sys.stderr,
            )
        print(
            f"[repository-source] {len(result.findings)} blocking finding(s)",
            file=sys.stderr,
        )
    else:
        print(
            "[repository-source] PASS: RC1 native-foundation policy is satisfied "
            f"({len(result.evaluated_paths)} evaluated path(s))"
        )
    return 1 if result.findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
