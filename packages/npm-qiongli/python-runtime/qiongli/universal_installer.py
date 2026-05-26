from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys
import tempfile
from functools import lru_cache
from importlib import resources
from dataclasses import dataclass
from pathlib import Path

from .subject_materializer import MaterializeOptions, SubjectMaterializationError, materialize_subject_package, validate_subject_catalog


TARGET_CHOICES = ("codex", "claude", "gemini", "antigravity", "all")
PROFILE_CHOICES = ("partial", "full")
PART_CHOICES = ("globals", "project", "cli", "doctor")
LEGACY_MANIFEST_PATH = Path(__file__).resolve().parents[1] / "install" / "install_manifest.tsv"


@dataclass
class InstallOptions:
    repo_root: Path
    project_dir: Path
    subject: str = "core"
    target: str = "all"
    mode: str = "copy"
    overwrite: bool = False
    install_cli: bool | None = None
    cli_dir: Path | None = None
    doctor: bool | None = None
    dry_run: bool = False
    profile: str | None = None
    parts: tuple[str, ...] | None = None


def profile_defaults(profile: str) -> dict[str, bool]:
    if profile == "partial":
        return {"install_cli": False, "doctor": False}
    if profile == "full":
        return {"install_cli": True, "doctor": True}
    raise ValueError(f"Unsupported profile: {profile}")


def apply_profile(options: InstallOptions) -> InstallOptions:
    if not options.profile:
        return options
    defaults = profile_defaults(options.profile)
    return InstallOptions(
        repo_root=options.repo_root,
        project_dir=options.project_dir,
        subject=options.subject,
        target=options.target,
        mode=options.mode,
        overwrite=options.overwrite,
        install_cli=defaults["install_cli"] if options.install_cli is None else options.install_cli,
        cli_dir=options.cli_dir,
        doctor=defaults["doctor"] if options.doctor is None else options.doctor,
        dry_run=options.dry_run,
        profile=options.profile,
        parts=options.parts,
    )


def normalize_parts(parts: tuple[str, ...] | list[str] | str | None) -> tuple[str, ...] | None:
    if parts is None:
        return None
    raw_items: list[str]
    if isinstance(parts, str):
        raw_items = [item.strip() for item in parts.split(",")]
    else:
        raw_items = [str(item).strip() for item in parts]

    cleaned = [item for item in raw_items if item]
    if not cleaned:
        return None

    normalized: list[str] = []
    for item in cleaned:
        token = item.lower()
        if token in {"all", "*"}:
            return PART_CHOICES
        if token == "global":
            token = "globals"
        if token == "shell-cli":
            token = "cli"
        if token not in PART_CHOICES:
            raise ValueError(f"Unsupported install part: {item}")
        if token not in normalized:
            normalized.append(token)
    return tuple(normalized)


def cli_name_for_target(target: str) -> str:
    mapping = {
        "codex": "codex",
        "claude": "claude",
        "gemini": "gemini",
        "antigravity": "antigravity",
    }
    return mapping[target]


def cli_install_hint(target: str) -> str:
    hints = {
        "codex": "Install the Codex CLI from the official OpenAI distribution and ensure `codex` is on PATH.",
        "claude": "Install Claude Code: npm install -g @anthropic-ai/claude-code",
        "gemini": "Install Gemini CLI: npm install -g @google/gemini-cli",
        "antigravity": "Install Antigravity and ensure `antigravity` is on PATH before relying on the global skill directory.",
    }
    return hints[target]


def _resolve(path: Path | str) -> Path:
    return Path(path).expanduser().resolve()


def _ensure_dir(path: Path, dry_run: bool) -> None:
    if dry_run:
        return
    path.mkdir(parents=True, exist_ok=True)


def _set_executable(path: Path, dry_run: bool) -> None:
    if dry_run or not path.exists() or path.is_symlink():
        return
    current = path.stat().st_mode
    path.chmod(current | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def _remove_path(path: Path, dry_run: bool) -> None:
    if dry_run or not path.exists() and not path.is_symlink():
        return
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink()


def _same_path(src: Path, dest: Path) -> bool:
    try:
        return src.resolve() == dest.resolve()
    except OSError:
        return False


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def _read_bytes(path: Path) -> bytes:
    try:
        return path.read_bytes()
    except OSError:
        return b""


def _read_version_file(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8").strip()
    except OSError:
        return ""


def _is_qiongli_package_dir(path: Path) -> bool:
    if not path.is_dir():
        return False
    skill_text = _read_text(path / "SKILL.md")
    return "name: qiongli\n" in skill_text or "name: qiongli-workflow" in skill_text


def _skill_package_version(path: Path) -> str:
    if not _is_qiongli_package_dir(path):
        return ""
    return _read_version_file(path / "VERSION")


def _skill_package_subject(path: Path) -> str:
    if not _is_qiongli_package_dir(path):
        return ""
    subject = _read_version_file(path / "SUBJECT")
    return subject or "core"


def _skill_package_state(path: Path) -> str:
    if not _is_qiongli_package_dir(path):
        return "not installed"
    return _skill_package_version(path) or "unknown"


_LEGACY_SKILL_PACKAGE_NAMES = ("research-paper-workflow",)
_WORKFLOW_LINK_PACKAGE_MARKERS = ("qiongli-workflow", *_LEGACY_SKILL_PACKAGE_NAMES)


def _selected_target_names(target: str) -> tuple[str, ...]:
    return TARGET_CHOICES[:-1] if target == "all" else (target,)


def _legacy_global_skill_residues(
    target: str,
    target_paths: dict[str, Path],
) -> list[tuple[str, str, Path]]:
    residues: list[tuple[str, str, Path]] = []
    for target_name in _selected_target_names(target):
        skill_root = target_paths[target_name].parent
        for legacy_name in _LEGACY_SKILL_PACKAGE_NAMES:
            candidate = skill_root / legacy_name
            if candidate.exists() or candidate.is_symlink():
                residues.append((target_name, legacy_name, candidate))
    return residues


def _print_legacy_install_residues(target: str, target_paths: dict[str, Path]) -> None:
    residues = _legacy_global_skill_residues(target, target_paths)
    if not residues:
        return
    _print_section("Legacy Install Residues")
    for target_name, legacy_name, path in residues:
        _print_result("Legacy Skill", f"{target_name}: {legacy_name} -> {path}", "skip")
    print("          These legacy skill directories are left in place.")
    print("          Remove them manually after confirming you no longer use them.")
    print("          `qiongli clean --globals` removes legacy workflow discovery symlinks only.")


def _skill_copy_detail(dest: Path, src_version: str, dest_version: str = "", action: str = "") -> str:
    if action == "skip":
        return f"{dest} (current {src_version}; source {src_version}; already installed)"
    if action == "update" and dest_version:
        return f"{dest} (current {dest_version}; source {src_version}; updated {dest_version} -> {src_version})"
    if action == "install":
        return f"{dest} (installed {src_version})"
    return str(dest)


def _skill_subject_copy_detail(
    dest: Path,
    src_version: str,
    src_subject: str,
    dest_version: str,
    dest_subject: str,
) -> str:
    return (
        f"{dest} (current {dest_version}/{dest_subject}; source {src_version}/{src_subject}; "
        f"updated {dest_subject} -> {src_subject})"
    )


@lru_cache(maxsize=None)
def _managed_copy_markers(src_name: str) -> tuple[str, ...]:
    markers = {
        "qiongli_cli.sh": ('CLI_FLAVOR="shell-bootstrap"', 'qiongli <command>'),
        "bootstrap_qiongli.sh": ('DEFAULT_REPO="jxpeng98/qiongli"', "--profile <partial|full>"),
    }
    return markers.get(src_name, ())


def _is_managed_copy(src: Path, dest: Path) -> bool:
    if not dest.is_file():
        return False
    markers = _managed_copy_markers(src.name)
    if not markers:
        return False
    content = _read_text(dest)
    return all(marker in content for marker in markers)


def _copy_path(src: Path, dest: Path, mode: str, overwrite: bool, dry_run: bool) -> tuple[str, str]:
    src_version = _skill_package_version(src)
    if _same_path(src, dest):
        return "skip", f"{dest} (same path)"
    if dest.exists() or dest.is_symlink():
        auto_detail = ""
        if not overwrite:
            dest_version = _skill_package_version(dest)
            src_subject = _skill_package_subject(src)
            dest_subject = _skill_package_subject(dest)
            if src_version and dest_version:
                if src_version == dest_version and src_subject == dest_subject:
                    return "skip", _skill_copy_detail(dest, src_version, action="skip")
                if src_subject and dest_subject and src_subject != dest_subject:
                    auto_detail = _skill_subject_copy_detail(dest, src_version, src_subject, dest_version, dest_subject)
                else:
                    auto_detail = _skill_copy_detail(dest, src_version, dest_version, action="update")
            elif src_version and _is_qiongli_package_dir(dest):
                auto_detail = _skill_copy_detail(dest, src_version, "unknown", action="update")
            elif src.is_file() and dest.is_file():
                if _read_bytes(src) == _read_bytes(dest):
                    return "skip", f"{dest} (already current)"
                if _is_managed_copy(src, dest):
                    auto_detail = "updated"
                else:
                    return "skip", f"{dest} (use --overwrite)"
            elif mode == "link" and dest.is_symlink() and _same_path(src, dest):
                return "skip", f"{dest} (already linked)"
            else:
                return "skip", f"{dest} (use --overwrite)"
        else:
            auto_detail = ""
        _remove_path(dest, dry_run)
        detail_suffix = auto_detail
    else:
        detail_suffix = ""
    _ensure_dir(dest.parent, dry_run)
    if dry_run:
        return "ok", str(dest)
    if mode == "link":
        os.symlink(str(src), str(dest), target_is_directory=src.is_dir())
    elif src.is_dir():
        shutil.copytree(src, dest)
    else:
        shutil.copy2(src, dest)
    if isinstance(detail_suffix, str) and detail_suffix.startswith(str(dest)):
        return "ok", detail_suffix
    if src_version and not detail_suffix:
        return "ok", _skill_copy_detail(dest, src_version, action="install")
    if detail_suffix:
        return "ok", f"{dest} ({detail_suffix})"
    return "ok", str(dest)


def _parse_manifest() -> list[dict[str, str]]:
    entries: list[dict[str, str]] = []
    try:
        manifest_text = resources.files("qiongli").joinpath("install_manifest.tsv").read_text(encoding="utf-8")
    except (FileNotFoundError, ModuleNotFoundError):
        manifest_text = LEGACY_MANIFEST_PATH.read_text(encoding="utf-8")
    for raw_line in manifest_text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        target, op, label, source, destination = raw_line.split("\t")
        entries.append(
            {
                "target": target,
                "op": op,
                "label": label,
                "source": source,
                "destination": destination,
            }
        )
    return entries


def _manifest_entry_part(entry: dict[str, str]) -> str:
    return "project" if "${PROJECT_DIR}" in entry["destination"] else "globals"


def _expand_path(template: str, values: dict[str, str]) -> Path:
    result = template
    for key, value in values.items():
        result = result.replace("${" + key + "}", value)
    return Path(result)


def _on_path(directory: Path) -> bool:
    target = str(directory)
    for entry in os.environ.get("PATH", "").split(os.pathsep):
        if not entry:
            continue
        try:
            if str(Path(entry).expanduser().resolve()) == target:
                return True
        except OSError:
            continue
    return False


def _print_result(label: str, dest: str, status: str) -> None:
    if status == "ok":
        print(f"  [ok]   {label:<12} -> {dest}")
    else:
        print(f"  [skip] {label:<12} -> {dest}")


def _print_section(title: str) -> None:
    print(f"\n== {title} ==")


def _print_detected_versions(target: str, source_version: str, target_paths: dict[str, Path]) -> None:
    _print_section("Detected Versions")
    print(f"  source:      {source_version or 'unknown'}")
    section_targets = TARGET_CHOICES[:-1] if target == "all" else (target,)
    for item in section_targets:
        state = _skill_package_state(target_paths[item])
        print(f"  {item:<11} {state}")


def _copy_display(src: Path, dest: Path, label: str, options: InstallOptions) -> None:
    status, detail = _copy_path(src, dest, options.mode, options.overwrite, options.dry_run)
    _print_result(label, detail, status)


def _install_alias_copy(src: Path, dest: Path, options: InstallOptions) -> None:
    status, detail = _copy_path(src, dest, "copy", options.overwrite, options.dry_run)
    _print_result("Alias", detail, status)
    if status == "ok":
        _set_executable(dest, options.dry_run)


def _windows_shell_cli_available() -> bool:
    return shutil.which("bash") is not None


def _install_shell_cli(options: InstallOptions) -> None:
    assert options.cli_dir is not None
    repo_root = options.repo_root
    cli_src = repo_root / "scripts" / "qiongli_cli.sh"
    bootstrap_src = repo_root / "scripts" / "bootstrap_qiongli.sh"
    cli_dir = options.cli_dir
    cli_dest = cli_dir / "qiongli"
    bootstrap_dest = cli_dir / "qiongli-bootstrap"

    _print_section("Shell CLI")

    if os.name == "nt" and not _windows_shell_cli_available():
        print("  [skip] Shell CLI    -> Git Bash / `bash` not found on Windows")
        print("          Hint: winget install -e --id Git.Git --source winget")
        return

    _copy_display(cli_src, cli_dest, "CLI", options)
    _set_executable(cli_dest, options.dry_run)
    _copy_display(bootstrap_src, bootstrap_dest, "Bootstrap", options)
    _set_executable(bootstrap_dest, options.dry_run)

    aliases = ("ql", "research-skills", "rsk", "rsw")
    if os.name == "nt":
        for name in aliases:
            alias_dest = cli_dir / name
            _install_alias_copy(cli_src, alias_dest, options)
    else:
        for name in aliases:
            alias_dest = cli_dir / name
            if alias_dest.exists() or alias_dest.is_symlink():
                if alias_dest.is_symlink() and _same_path(cli_dest, alias_dest):
                    _print_result("Alias", f"{alias_dest} (already linked)", "skip")
                    continue
                if not options.overwrite:
                    _print_result("Alias", f"{alias_dest} (use --overwrite)", "skip")
                    continue
                _remove_path(alias_dest, options.dry_run)
            _ensure_dir(alias_dest.parent, options.dry_run)
            if options.dry_run:
                _print_result("Alias", str(alias_dest), "ok")
                continue
            try:
                os.symlink(str(cli_dest), str(alias_dest))
            except OSError:
                shutil.copy2(cli_dest, alias_dest)
            _set_executable(alias_dest, options.dry_run)
            _print_result("Alias", str(alias_dest), "ok")

    if _on_path(cli_dir):
        print(f"  [info] cli dir on PATH: {cli_dir}")
    else:
        print(f"  [warn] CLI installed to {cli_dir} but this directory is not on PATH")


# Legacy project-local copy helpers removed — workflows are now bundled
# inside the skill directory and installed globally with each dir-copy.


# ── Sync skill package ───────────────────────────────────────────────────────

_SYNC_DIRS = ("skills", "templates", "standards", "roles", "venue-profiles")
_SYNC_FILES = ("skills-core.md", "skills-summary.md")
_SYNC_EXCLUDE = {"CLAUDE.project.md"}


def _sync_skill_package(repo_root: Path, *, dry_run: bool = False) -> None:
    """Populate qiongli-workflow/ with bundled copies of repo assets.

    The canonical source of truth remains the repo-root directories.
    These copies are .gitignore'd and regenerated on every install/upgrade.
    """
    pkg_dir = repo_root / "qiongli-workflow"
    if not pkg_dir.is_dir():
        return

    _print_section("Sync Skill Package")
    for dir_name in _SYNC_DIRS:
        src = repo_root / dir_name
        dest = pkg_dir / dir_name
        if not src.is_dir():
            _print_result("Sync", f"{dir_name}/ (source not found)", "skip")
            continue
        if dry_run:
            _print_result("Sync", f"{dir_name}/ (dry-run)", "skip")
            continue
        # Remove stale destination and copy fresh
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(
            src, dest,
            ignore=shutil.ignore_patterns(
                ".DS_Store", "__pycache__", *_SYNC_EXCLUDE,
            ),
        )
        file_count = sum(1 for _ in dest.rglob("*") if _.is_file())
        _print_result("Sync", f"{dir_name}/ ({file_count} files)", "ok")

    for file_name in _SYNC_FILES:
        src = repo_root / file_name
        dest = pkg_dir / file_name
        if not src.is_file():
            _print_result("Sync", f"{file_name} (source not found)", "skip")
            continue
        if dry_run:
            _print_result("Sync", f"{file_name} (dry-run)", "skip")
            continue
        shutil.copy2(src, dest)
        _print_result("Sync", file_name, "ok")


# ── Workflow symlink shims ───────────────────────────────────────────────────

# Maps target → (discovery_dir_name, skill_dest_env_var_key)
# Claude: ~/.claude/commands/<name>.md  (slash command discovery)
# Gemini: ~/.gemini/workflows/<name>.md (workflow discovery)
_SYMLINK_TARGETS: dict[str, tuple[str, str]] = {
    "claude": ("commands", "CLAUDE_CODE_HOME"),
    "gemini": ("workflows", "GEMINI_HOME"),
}


def _create_workflow_symlinks(
    target: str,
    skill_dest: Path,
    *,
    dry_run: bool = False,
) -> None:
    """Create symlinks from canonical workflow discovery paths to bundled workflows.

    For Claude Code:  ~/.claude/commands/<name>.md → ~/.claude/skills/.../workflows/<name>.md
    For Gemini CLI:   ~/.gemini/workflows/<name>.md → ~/.gemini/skills/.../workflows/<name>.md

    This enables direct /slash-command invocation (e.g. /paper, /lit-review).
    """
    if target not in _SYMLINK_TARGETS:
        return

    dir_name, _env_key = _SYMLINK_TARGETS[target]
    workflows_src = skill_dest / "workflows"
    if not workflows_src.is_dir():
        return

    # Discovery dir is sibling to skills/ under the client home
    # skill_dest = ~/.claude/skills/qiongli-workflow
    # discovery_dir = ~/.claude/commands/
    client_home = skill_dest.parent.parent  # ~/.claude or ~/.gemini
    discovery_dir = client_home / dir_name
    discovery_dir.mkdir(parents=True, exist_ok=True)

    workflow_files = sorted(workflows_src.glob("*.md"))
    created = 0
    for wf in workflow_files:
        link_path = discovery_dir / wf.name
        target_path = wf  # absolute path to the bundled workflow

        if dry_run:
            _print_result("Symlink", f"{wf.name} (dry-run)", "skip")
            continue

        # Remove stale link or file if it exists
        if link_path.is_symlink() or link_path.exists():
            link_path.unlink()

        link_path.symlink_to(target_path)
        created += 1

    if not dry_run and created > 0:
        _print_result("Symlinks", f"{created} workflows -> {discovery_dir}", "ok")


def _print_cli_checks(target: str) -> bool:
    found_antigravity = False
    _print_section("CLI Checks")
    targets = TARGET_CHOICES[:-1] if target == "all" else (target,)
    for item in targets:
        cli_name = cli_name_for_target(item)
        resolved = shutil.which(cli_name)
        if resolved:
            _print_result("CLI", f"{item} -> {resolved}", "ok")
            if item == "antigravity":
                found_antigravity = True
            continue
        _print_result("CLI", f"{item} -> missing", "skip")
        print(f"          Hint: {cli_install_hint(item)}")
    return found_antigravity


def _print_full_readiness(options: InstallOptions) -> None:
    if options.profile != "full":
        return
    _print_section("Full Profile Readiness")
    version = sys.version_info
    python_status = "ok" if (version.major, version.minor) >= (3, 12) else "skip"
    _print_result("Python", f"{sys.executable} ({version.major}.{version.minor}.{version.micro})", python_status)
    if python_status != "ok":
        print(
            "          Hint: install Python >= 3.12 using python.org/downloads, "
            "your OS package manager, pyenv, mise, winget install -e --id Python.Python.3.12, "
            "or another method you prefer"
        )
    for env_var in ("OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY"):
        value = os.environ.get(env_var, "").strip()
        _print_result(env_var, "configured" if value else "missing", "ok" if value else "skip")
    if os.name == "nt":
        has_bash = shutil.which("bash") is not None
        _print_result("Windows Bash", "available" if has_bash else "missing", "ok" if has_bash else "skip")
        if not has_bash:
            print("          Hint: winget install -e --id Git.Git --source winget")


def _run_doctor(project_dir: Path, dry_run: bool) -> None:
    _print_section("Doctor")
    if dry_run:
        print(f"  [ok]   Doctor       -> dry-run ({project_dir})")
        return
    repo_root = Path(__file__).resolve().parents[1]
    env = os.environ.copy()
    existing_pythonpath = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = str(repo_root) if not existing_pythonpath else f"{repo_root}{os.pathsep}{existing_pythonpath}"
    result = subprocess.run(
        [sys.executable, "-m", "bridges.orchestrator", "doctor", "--cwd", str(project_dir)],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
        env=env,
    )
    print(result.stdout.strip() if result.stdout.strip() else "  [warn] doctor produced no output")
    if result.returncode != 0:
        print(f"  [warn] doctor exited with code {result.returncode}")


def install(options: InstallOptions) -> int:
    options = apply_profile(
        InstallOptions(
            repo_root=_resolve(options.repo_root),
            project_dir=_resolve(options.project_dir),
            subject=options.subject,
            target=options.target,
            mode=options.mode,
            overwrite=options.overwrite,
            install_cli=options.install_cli,
            cli_dir=_resolve(options.cli_dir or Path.home() / ".local" / "bin"),
            doctor=options.doctor,
            dry_run=options.dry_run,
            profile=options.profile,
            parts=options.parts,
        )
    )

    if options.target not in TARGET_CHOICES:
        raise ValueError(f"Unsupported target: {options.target}")
    if options.mode not in {"copy", "link"}:
        raise ValueError(f"Unsupported mode: {options.mode}")
    selected_parts = normalize_parts(options.parts)
    install_globals = True if selected_parts is None else "globals" in selected_parts
    install_project = False if selected_parts is None else "project" in selected_parts
    install_cli = bool(options.install_cli) if selected_parts is None else "cli" in selected_parts
    doctor = bool(options.doctor) if selected_parts is None else "doctor" in selected_parts

    repo_root = options.repo_root
    catalog = validate_subject_catalog(repo_root)
    if options.subject not in catalog.subjects:
        available = ", ".join(sorted(catalog.subjects))
        raise SubjectMaterializationError(f"Unknown subject '{options.subject}'. Available subjects: {available}")
    base_skill_src = repo_root / "qiongli-workflow"
    skill_src = base_skill_src
    if not (skill_src / "SKILL.md").exists():
        raise FileNotFoundError(f"Missing skill source: {skill_src / 'SKILL.md'}")

    codex_dest = Path(os.environ.get("CODEX_HOME", str(Path.home() / ".codex"))) / "skills" / "qiongli-workflow"
    claude_dest = Path(os.environ.get("CLAUDE_CODE_HOME", str(Path.home() / ".claude"))) / "skills" / "qiongli-workflow"
    gemini_dest = Path(os.environ.get("GEMINI_HOME", str(Path.home() / ".gemini"))) / "skills" / "qiongli-workflow"
    antigravity_dest = Path(os.environ.get("ANTIGRAVITY_HOME", str(Path.home() / ".gemini" / "antigravity"))) / "skills" / "qiongli-workflow"
    source_version = _skill_package_version(skill_src)
    manifest_values = {
        "PROJECT_DIR": str(options.project_dir),
        "CODEX_HOME": str(codex_dest.parent.parent),
        "CLAUDE_CODE_HOME": str(claude_dest.parent.parent),
        "GEMINI_HOME": str(gemini_dest.parent.parent),
        "ANTIGRAVITY_HOME": str(antigravity_dest.parent.parent),
    }
    manifest_entries = _parse_manifest()

    print("\nQiongli Universal Installer")
    print(f"  source:  {repo_root}")
    print(f"  project: {options.project_dir}")
    print(f"  target:  {options.target} | mode: {options.mode}")
    print(f"  subject: {options.subject}")
    if options.profile:
        print(f"  profile: {options.profile}")
    if selected_parts is not None:
        print(f"  parts:   {', '.join(selected_parts)}")
    if install_cli:
        print(f"  cli:     install -> {options.cli_dir}")

    target_paths = {
        "codex": codex_dest,
        "claude": claude_dest,
        "gemini": gemini_dest,
        "antigravity": antigravity_dest,
    }
    _print_detected_versions(options.target, source_version, target_paths)
    if install_globals:
        _print_legacy_install_residues(options.target, target_paths)
    _print_full_readiness(options)
    _print_cli_checks(options.target)

    # Sync bundled assets into the skill package before dir-copy
    if install_globals and not options.dry_run:
        _sync_skill_package(repo_root, dry_run=options.dry_run)

    materialized_tmp: tempfile.TemporaryDirectory[str] | None = None
    if install_globals:
        materialized_tmp = tempfile.TemporaryDirectory(prefix="qiongli-subject-")
        skill_src = Path(materialized_tmp.name) / "qiongli-workflow"
        if options.dry_run:
            _print_section("Subject Package")
            _print_result("Subject", f"{options.subject} (dry-run)", "skip")
            skill_src = base_skill_src
        else:
            materialize_subject_package(
                MaterializeOptions(
                    source=repo_root,
                    out=skill_src,
                    subject=options.subject,
                    flavor="full",
                )
            )
            _print_section("Subject Package")
            _print_result("Subject", f"{options.subject} -> {skill_src}", "ok")

    section_targets = ("codex", "claude", "gemini", "antigravity")
    try:
        for section_target in section_targets:
            if options.target not in {section_target, "all"}:
                continue
            entries_for_target = [
                entry
                for entry in manifest_entries
                if entry["target"] == section_target
                and (
                    (install_globals and _manifest_entry_part(entry) == "globals")
                    or (install_project and _manifest_entry_part(entry) == "project")
                )
            ]
            if not entries_for_target:
                continue

            _print_section(section_target.capitalize() if section_target != "antigravity" else "Antigravity")
            for entry in entries_for_target:
                op = entry["op"]
                label = entry["label"]
                src = skill_src if entry["source"] == "qiongli-workflow" else repo_root / entry["source"]
                dest = _expand_path(entry["destination"], manifest_values)

                if op in {"dir-copy", "file-copy"}:
                    _copy_display(src, dest, label, options)
                    continue

                raise ValueError(f"Unsupported manifest operation: {op}")
    finally:
        if materialized_tmp is not None:
            materialized_tmp.cleanup()

    # Create workflow discovery symlinks (Claude: commands/, Gemini: workflows/)
    if install_globals and not options.dry_run:
        _print_section("Workflow Discovery")
        target_dest_map = {
            "claude": claude_dest,
            "gemini": gemini_dest,
        }
        for sym_target, sym_dest in target_dest_map.items():
            if options.target in {sym_target, "all"}:
                _create_workflow_symlinks(sym_target, sym_dest, dry_run=options.dry_run)

    if install_cli:
        _install_shell_cli(options)

    if install_project:
        _print_section("Project Env")
        for entry in manifest_entries:
            if entry["target"] != "project":
                continue
            _copy_display(repo_root / entry["source"], _expand_path(entry["destination"], manifest_values), entry["label"], options)

    if doctor:
        _run_doctor(options.project_dir, options.dry_run)

    print("\n[done] Installation complete")
    if install_cli and options.cli_dir and not _on_path(options.cli_dir):
        print(f"       Add {options.cli_dir} to PATH to use qiongli / ql / research-skills / rsk / rsw")
    print("       Restart Codex / Claude Code / Gemini CLI to activate changes")
    return 0


# ── Clean stale project-local assets ─────────────────────────────────────────

_CLEANABLE_GLOBS = (
    ".agent/workflows/*.md",
    ".agent/skills/qiongli-workflow",
    ".agent/skills/research-paper-workflow",
    ".agents/skills/qiongli-workflow",
    ".agents/skills/research-paper-workflow",
    "CLAUDE.qiongli.md",
    ".gemini/qiongli.md",
    ".gemini/agent-profiles.example.json",
)

_CLEANABLE_CONDITIONAL = (
    # Only remove these if their content matches the template we used to install
    "CLAUDE.md",
)


def _is_qiongli_claude_md(path: Path, repo_root: Path | None) -> bool:
    """Return True if `path` looks like a qiongli template CLAUDE.md."""
    if not path.is_file():
        return False
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return False
    legacy_marker = "Academic Deep Qiongli" in text
    current_marker = "Qiongli Zhengche" in text or "穷理证澈" in text
    return (current_marker or legacy_marker) and "qiongli-workflow" in text


def clean(project_dir: Path, *, dry_run: bool = False, repo_root: Path | None = None) -> int:
    """Remove stale project-local qiongli assets."""
    project_dir = _resolve(project_dir)
    removed = 0

    _print_section("Clean stale project-local assets")
    for pattern in _CLEANABLE_GLOBS:
        candidates = sorted(project_dir.glob(pattern))
        for path in candidates:
            _remove_path(path, dry_run)
            _print_result("Removed", str(path), "ok")
            removed += 1
    # If no wildcard matches were found for the parent dir, it might be empty now
    for parent in {"agent/workflows", "agent/skills", "agents/skills"}:
        parent_path = project_dir / f".{parent}"
        if parent_path.is_dir() and not any(parent_path.iterdir()):
            _remove_path(parent_path, dry_run)
            _print_result("Removed", f"{parent_path}/ (empty)", "ok")
            removed += 1

    # Conditional: CLAUDE.md only if it matches our template
    claude_md = project_dir / "CLAUDE.md"
    if _is_qiongli_claude_md(claude_md, repo_root):
        _remove_path(claude_md, dry_run)
        _print_result("Removed", str(claude_md), "ok")
        removed += 1
    elif claude_md.exists():
        _print_result("Kept", f"{claude_md} (user-customized)", "skip")

    if removed:
        print(f"\n[done] Cleaned {removed} stale asset(s) from {project_dir}")
    else:
        print(f"\n[done] No stale assets found in {project_dir}")
    return 0


def clean_workflow_symlinks(*, dry_run: bool = False) -> int:
    """Remove workflow discovery symlinks created by the installer.

    Cleans: ~/.claude/commands/<name>.md and ~/.gemini/workflows/<name>.md
    Only removes symlinks that point into a qiongli-workflow directory.
    """
    removed = 0
    _print_section("Clean workflow discovery symlinks")

    for target, (dir_name, env_key) in _SYMLINK_TARGETS.items():
        home = Path(os.environ.get(env_key, str(Path.home() / f".{target}")))
        discovery_dir = home / dir_name
        if not discovery_dir.is_dir():
            continue
        for link in sorted(discovery_dir.iterdir()):
            if not link.is_symlink():
                continue
            target_path = str(link.resolve())
            if any(marker in target_path for marker in _WORKFLOW_LINK_PACKAGE_MARKERS):
                _remove_path(link, dry_run)
                _print_result("Removed", f"{link.name} -> {target}", "ok")
                removed += 1

    if removed:
        print(f"\n[done] Removed {removed} workflow symlink(s)")
    else:
        print(f"\n[done] No workflow symlinks found")
    return 0
