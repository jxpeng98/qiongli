# CLI Full Plugin Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a CLI-installed plugin surface so `qiongli install --profile full --target all --surface plugin` installs Qiongli as a local plugin bundle backed by the unified Python full MCP server.

**Architecture:** Keep marketplace plugin artifacts as the no-Python lite path with bundled Node literature MCP. Add a separate local-plugin install path in the Python CLI that materializes plugin manifests, commands, and `skills/qiongli-workflow`, then writes MCP config pointing to `qiongli mcp serve --transport stdio`. Preserve legacy global skill installation through `--surface skills` and let users request both surfaces explicitly.

**Tech Stack:** Python 3.12, `unittest`, existing `qiongli.universal_installer`, existing `qiongli.subject_materializer`, existing `qiongli.distribution_metadata`, Codex personal marketplace JSON, generated `.codex-plugin` and `.claude-plugin` manifests.

---

## Scope Decisions

- `--surface skills` keeps the current behavior: install `qiongli-workflow` into global client skill directories and optionally register full MCP at the client level.
- `--surface plugin` installs a local plugin bundle. For Codex, it writes a personal marketplace entry and a plugin root under the marketplace root. For Claude Code, it generates the same plugin payload with `.claude-plugin/plugin.json` and routes full MCP through that manifest; the registration adapter owns the client-specific install command.
- `--surface both` installs both the legacy global skill directory and the local plugin bundle.
- Marketplace release artifacts keep their existing lite Node literature MCP. The release generator in `tooling/scripts/build_plugin_artifacts.py` should not be changed to point at the Python full MCP.
- Provider secrets remain outside plugin manifests, `.mcp.json`, release ZIPs, and research artifacts.
- The first stable implementation keeps the CLI default as `--surface skills` to avoid surprising existing users. Documentation recommends `--surface plugin` for full local Qiongli.

## File Structure

- Modify `packages/python-qiongli/src/qiongli/universal_installer.py`
  - Add `SURFACE_CHOICES`.
  - Extend `InstallOptions` and `RemoveOptions` with `surface`.
  - Add `plugin` to managed install parts.
  - Wire plugin installation and removal into the existing install/remove flow.
  - Avoid duplicate full MCP registrations for targets whose plugin manifest already owns MCP.

- Create `packages/python-qiongli/src/qiongli/local_plugin_installer.py`
  - Own local plugin paths, manifest writing, MCP manifest writing, command wrapper generation, personal marketplace entry updates, and managed plugin removal.
  - Keep Codex local marketplace behavior in this module instead of writing Codex cache internals.
  - Generate `.codex-plugin/plugin.json`, optional `.claude-plugin/plugin.json`, `skills/qiongli-workflow/`, `commands/*.md`, and `.mcp.json`.

- Modify `packages/python-qiongli/src/qiongli/cli.py`
  - Add `--surface skills|plugin|both` to `install`, `upgrade`, `init` where relevant, and `remove`.
  - Pass `surface` through to installer options.

- Modify `tests/test_cli.py`
  - Assert CLI parser passes `surface`.

- Modify `tests/test_universal_installer.py`
  - Assert plugin surface skips legacy global skill install unless `both` is selected.
  - Assert full profile with plugin surface installs plugin-backed MCP for Codex without writing duplicate client-level Codex MCP.
  - Assert Antigravity and Hermes still receive client-level full MCP config under `--target all --surface plugin`.

- Create `tests/test_local_plugin_installer.py`
  - Unit-test local plugin manifest content, full MCP manifest content, marketplace path resolution, command wrapper generation, and managed removal.

- Modify docs:
  - `docs/reference/cli.md`
  - `docs/guide/install.md`
  - `docs/zh/guide/install.md`
  - `README_CN.md`
  - `docs/advanced/plugin-first-architecture.md`
  - `docs/advanced/qiongli-cli-plugin-structure.html`

## Task 1: Add Surface Parsing And Data Model

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/universal_installer.py`
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `tests/test_cli.py`

- [ ] **Step 1: Write failing CLI parser tests**

Append these tests to `tests/test_cli.py` near the existing install/upgrade parser tests:

```python
    def test_install_command_passes_surface_to_installer(self) -> None:
        captured = {}

        def fake_install(options):
            captured["options"] = options
            return 0

        with mock.patch("qiongli.cli.install", fake_install):
            rc = cli.main(
                [
                    "qiongli",
                    "install",
                    "--profile",
                    "full",
                    "--target",
                    "codex",
                    "--surface",
                    "plugin",
                    "--dry-run",
                ]
            )

        self.assertEqual(rc, 0)
        self.assertEqual(captured["options"].surface, "plugin")

    def test_upgrade_command_passes_surface_to_installer(self) -> None:
        captured = {}

        def fake_run_installer(options):
            captured["options"] = options
            return 0

        with tempfile.TemporaryDirectory() as tmp_dir:
            archive = Path(tmp_dir) / "repo.tar.gz"
            source = Path(tmp_dir) / "source"
            source.mkdir()
            (source / "scripts").mkdir()
            (source / "scripts" / "bootstrap_qiongli.py").write_text("print('ok')\n", encoding="utf-8")

            with tarfile.open(archive, "w:gz") as tar:
                tar.add(source, arcname="qiongli-source")

            with mock.patch("qiongli.cli._resolve_upstream_repo", return_value=("jxpeng98/qiongli", "arg")):
                with mock.patch("qiongli.cli._download", side_effect=lambda _url, dest: shutil.copy2(archive, dest)):
                    with mock.patch("qiongli.cli._run_installer", fake_run_installer):
                        rc = cli.main(
                            [
                                "qiongli",
                                "upgrade",
                                "--ref",
                                "v1.7.0",
                                "--target",
                                "codex",
                                "--surface",
                                "plugin",
                                "--dry-run",
                            ]
                        )

        self.assertEqual(rc, 0)
        self.assertEqual(captured["options"].surface, "plugin")
```

- [ ] **Step 2: Run parser tests and verify failure**

Run:

```bash
python3 -m unittest tests.test_cli -v
```

Expected: FAIL because `InstallOptions` has no `surface` field and the parsers do not define `--surface`.

- [ ] **Step 3: Add installer surface constants and option fields**

In `packages/python-qiongli/src/qiongli/universal_installer.py`, update constants and dataclasses:

```python
TARGET_CHOICES = ("codex", "claude", "antigravity", "hermes", "all")
PROFILE_CHOICES = ("partial", "full")
SURFACE_CHOICES = ("skills", "plugin", "both")
PART_CHOICES = ("globals", "plugin", "project", "cli", "mcp", "doctor")
```

Add `surface` to `InstallOptions`:

```python
@dataclass
class InstallOptions:
    repo_root: Path
    project_dir: Path
    subject: str = "core"
    coverage: str = "complete"
    target: str = "all"
    mode: str = "copy"
    overwrite: bool = False
    install_cli: bool | None = None
    cli_dir: Path | None = None
    install_mcp: bool | None = None
    doctor: bool | None = None
    dry_run: bool = False
    profile: str | None = None
    parts: tuple[str, ...] | None = None
    surface: str = "skills"
```

Add `surface` to `RemoveOptions`:

```python
@dataclass
class RemoveOptions:
    project_dir: Path
    target: str = "all"
    dry_run: bool = False
    parts: tuple[str, ...] | None = None
    cli_dir: Path | None = None
    surface: str = "skills"
```

When `apply_profile()` creates a new `InstallOptions`, carry `surface=options.surface`.

- [ ] **Step 4: Add `--surface` to CLI parsers**

In `packages/python-qiongli/src/qiongli/cli.py`, import `SURFACE_CHOICES` from `qiongli.universal_installer`.

Add this argument to `upgrade`, `install`, and `remove` parsers:

```python
parser.add_argument(
    "--surface",
    default="skills",
    choices=SURFACE_CHOICES,
    help="Install or remove surface: skills, plugin, or both.",
)
```

Use the variable name that matches each parser object: `upgrade.add_argument`, `install_parser.add_argument`, and `remove_parser.add_argument`.

Pass it into `InstallOptions` in `cmd_upgrade()`, `cmd_install()`, and `cmd_init()`:

```python
surface=getattr(args, "surface", "skills"),
```

Pass it into `RemoveOptions` in `cmd_remove()`:

```python
surface=getattr(args, "surface", "skills"),
```

- [ ] **Step 5: Run parser tests and verify pass**

Run:

```bash
python3 -m unittest tests.test_cli -v
```

Expected: PASS for parser tests.

- [ ] **Step 6: Commit**

```bash
git add packages/python-qiongli/src/qiongli/universal_installer.py packages/python-qiongli/src/qiongli/cli.py tests/test_cli.py
git commit -m "feat(installer): add install surface option"
```

## Task 2: Implement Local Plugin Payload Builder

**Files:**
- Create: `packages/python-qiongli/src/qiongli/local_plugin_installer.py`
- Create: `tests/test_local_plugin_installer.py`

- [ ] **Step 1: Write failing local plugin tests**

Create `tests/test_local_plugin_installer.py`:

```python
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from qiongli.local_plugin_installer import (
    LocalPluginOptions,
    install_local_plugin,
    remove_local_plugin,
    resolve_codex_plugin_paths,
)


REPO_ROOT = Path(__file__).resolve().parents[1]


class LocalPluginInstallerTests(unittest.TestCase):
    def test_resolve_codex_paths_match_marketplace_relative_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            marketplace_path = Path(tmp_dir) / ".agents" / "plugins" / "marketplace.json"
            paths = resolve_codex_plugin_paths("qiongli", marketplace_path=marketplace_path)

        self.assertEqual(paths.marketplace_path, marketplace_path)
        self.assertEqual(paths.plugin_root, marketplace_path.parent / "plugins" / "qiongli")
        self.assertEqual(paths.marketplace_source_path, "./plugins/qiongli")

    def test_install_codex_plugin_writes_full_mcp_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            marketplace_path = root / ".agents" / "plugins" / "marketplace.json"
            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    subject="core",
                    coverage="complete",
                    target="codex",
                    mode="copy",
                    overwrite=False,
                    dry_run=False,
                    codex_marketplace_path=marketplace_path,
                )
            )

            plugin_root = result.installed_roots["codex"]
            codex_manifest = json.loads((plugin_root / ".codex-plugin" / "plugin.json").read_text(encoding="utf-8"))
            mcp_manifest = json.loads((plugin_root / ".mcp.json").read_text(encoding="utf-8"))
            marketplace = json.loads(marketplace_path.read_text(encoding="utf-8"))

        self.assertEqual(codex_manifest["name"], "qiongli")
        self.assertEqual(codex_manifest["skills"], "./skills/")
        self.assertEqual(codex_manifest["mcpServers"], "./.mcp.json")
        self.assertTrue((plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file())
        self.assertTrue((plugin_root / "commands" / "paper.md").is_file())
        server = mcp_manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["command"], "qiongli")
        self.assertEqual(server["args"], ["mcp", "serve", "--transport", "stdio"])
        self.assertNotIn("env", server)
        self.assertNotIn("qiongli-literature-provider", json.dumps(mcp_manifest))
        self.assertEqual(marketplace["name"], "personal")
        self.assertEqual(marketplace["plugins"][0]["source"]["path"], "./plugins/qiongli")

    def test_install_claude_plugin_writes_full_mcp_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            claude_root = root / ".qiongli" / "plugins" / "claude-code"
            result = install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    subject="core",
                    coverage="complete",
                    target="claude",
                    mode="copy",
                    overwrite=False,
                    dry_run=False,
                    claude_plugin_parent=claude_root,
                )
            )

            plugin_root = result.installed_roots["claude"]
            manifest = json.loads((plugin_root / ".claude-plugin" / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest["name"], "qiongli")
        server = manifest["mcpServers"]["qiongli"]
        self.assertEqual(server["type"], "stdio")
        self.assertEqual(server["command"], "qiongli")
        self.assertEqual(server["args"], ["mcp", "serve", "--transport", "stdio"])
        self.assertTrue((plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file())

    def test_remove_local_plugin_removes_managed_codex_payload_and_entry(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            marketplace_path = Path(tmp_dir) / ".agents" / "plugins" / "marketplace.json"
            install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    subject="core",
                    coverage="complete",
                    target="codex",
                    mode="copy",
                    overwrite=False,
                    dry_run=False,
                    codex_marketplace_path=marketplace_path,
                )
            )

            removed = remove_local_plugin(
                target="codex",
                dry_run=False,
                codex_marketplace_path=marketplace_path,
            )
            marketplace = json.loads(marketplace_path.read_text(encoding="utf-8"))

        self.assertEqual(removed, 1)
        self.assertEqual(marketplace["plugins"], [])
        self.assertFalse((marketplace_path.parent / "plugins" / "qiongli").exists())


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```bash
python3 -m unittest tests.test_local_plugin_installer -v
```

Expected: FAIL because `qiongli.local_plugin_installer` does not exist.

- [ ] **Step 3: Create local plugin installer module**

Create `packages/python-qiongli/src/qiongli/local_plugin_installer.py` with these public types and helpers:

```python
from __future__ import annotations

import json
import os
import re
import shutil
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .distribution_metadata import PluginDefinition, load_plugin_distribution
from .source_layout import RepoLayout
from .subject_materializer import MaterializeOptions, materialize_subject_package


PLUGIN_ID = "qiongli"
SKILL_DIR_NAME = "qiongli-workflow"
QIONGLI_FULL_MCP_SERVER = {
    "command": "qiongli",
    "args": ["mcp", "serve", "--transport", "stdio"],
}


@dataclass(frozen=True)
class CodexPluginPaths:
    marketplace_path: Path
    plugin_root: Path
    marketplace_source_path: str


@dataclass(frozen=True)
class LocalPluginOptions:
    repo_root: Path
    subject: str = "core"
    coverage: str = "complete"
    target: str = "all"
    mode: str = "copy"
    overwrite: bool = False
    dry_run: bool = False
    codex_marketplace_path: Path | None = None
    claude_plugin_parent: Path | None = None


@dataclass(frozen=True)
class LocalPluginResult:
    installed_roots: dict[str, Path] = field(default_factory=dict)
    changed: bool = False
```

Add path helpers:

```python
def resolve_codex_plugin_paths(
    plugin_id: str = PLUGIN_ID,
    *,
    marketplace_path: Path | str | None = None,
) -> CodexPluginPaths:
    raw_path = marketplace_path or os.environ.get("QIONGLI_CODEX_MARKETPLACE_PATH", "").strip()
    path = Path(raw_path).expanduser() if raw_path else Path.home() / ".agents" / "plugins" / "marketplace.json"
    path = path.resolve()
    source_path = f"./plugins/{plugin_id}"
    plugin_root = path.parent / "plugins" / plugin_id
    return CodexPluginPaths(
        marketplace_path=path,
        plugin_root=plugin_root,
        marketplace_source_path=source_path,
    )


def _claude_plugin_root(plugin_id: str, parent: Path | str | None = None) -> Path:
    raw_parent = parent or os.environ.get("QIONGLI_CLAUDE_PLUGIN_PARENT", "").strip()
    base = Path(raw_parent).expanduser() if raw_parent else Path.home() / ".qiongli" / "plugins" / "claude-code"
    return base.resolve() / plugin_id
```

Add manifest writers:

```python
def _version(root: Path) -> str:
    return (RepoLayout(root).workflow / "VERSION").read_text(encoding="utf-8").strip().lstrip("v")


def _keywords(plugin: PluginDefinition, platform_keyword: str) -> list[str]:
    return [*plugin.keywords, *[item for item in (platform_keyword,) if item not in plugin.keywords]]


def _write_json(path: Path, payload: dict[str, Any], *, dry_run: bool) -> None:
    if dry_run:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _write_codex_manifest(dest: Path, plugin: PluginDefinition, version: str, *, dry_run: bool) -> None:
    payload = {
        "name": plugin.id,
        "version": version,
        "description": plugin.description,
        "author": plugin.author,
        "category": plugin.category,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "codex-skills"),
        "skills": "./skills/",
        "mcpServers": "./.mcp.json",
        "interface": {
            "displayName": plugin.display_name,
            "shortDescription": plugin.codex_short_description,
            "longDescription": plugin.description,
            "developerName": plugin.author["name"],
            "category": plugin.category,
            "capabilities": ["Write"],
            "websiteURL": plugin.repository,
            "defaultPrompt": list(plugin.default_prompts),
            "brandColor": plugin.brand_color,
        },
    }
    _write_json(dest, payload, dry_run=dry_run)


def _write_claude_manifest(dest: Path, plugin: PluginDefinition, version: str, *, dry_run: bool) -> None:
    payload = {
        "name": plugin.id,
        "description": plugin.description,
        "version": version,
        "author": plugin.author,
        "category": plugin.category,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "claude-code-plugins"),
        "mcpServers": {
            plugin.mcp_server_name: {
                **QIONGLI_FULL_MCP_SERVER,
                "type": "stdio",
            }
        },
    }
    _write_json(dest, payload, dry_run=dry_run)


def _write_codex_mcp_manifest(dest: Path, server_name: str, *, dry_run: bool) -> None:
    payload = {
        "mcpServers": {
            server_name: {
                **QIONGLI_FULL_MCP_SERVER,
                "startup_timeout_sec": 20,
                "tool_timeout_sec": 120,
            }
        }
    }
    _write_json(dest, payload, dry_run=dry_run)
```

Add skill and command materialization:

```python
def _remove_existing(path: Path, *, overwrite: bool, dry_run: bool) -> None:
    if not (path.exists() or path.is_symlink()):
        return
    if not overwrite and not _is_managed_plugin_root(path):
        raise FileExistsError(f"{path} exists and is not a managed Qiongli plugin; use --overwrite")
    if dry_run:
        return
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink()


def _is_managed_plugin_root(path: Path) -> bool:
    codex_manifest = path / ".codex-plugin" / "plugin.json"
    claude_manifest = path / ".claude-plugin" / "plugin.json"
    for manifest in (codex_manifest, claude_manifest):
        if not manifest.is_file():
            continue
        try:
            payload = json.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if payload.get("name") == PLUGIN_ID:
            return True
    return False


def _workflow_description(workflow_path: Path) -> str:
    text = workflow_path.read_text(encoding="utf-8")
    match = re.search(r"(?ms)^---\n(.*?)\n---", text)
    if match:
        desc = re.search(r"(?m)^description:\s*(.+)$", match.group(1))
        if desc:
            return desc.group(1).strip()
    return f"Run the {workflow_path.stem} research workflow."


def _write_commands(repo_root: Path, commands_root: Path, *, dry_run: bool) -> None:
    if dry_run:
        return
    workflow_root = RepoLayout(repo_root).workflow / "workflows"
    commands_root.mkdir(parents=True, exist_ok=True)
    for workflow_path in sorted(workflow_root.glob("*.md")):
        text = "\n".join(
            [
                "---",
                f"description: {_workflow_description(workflow_path)}",
                "---",
                "",
                (
                    "Load the `qiongli` skill from this plugin, then follow "
                    f"`skills/{SKILL_DIR_NAME}/workflows/{workflow_path.name}`."
                ),
                "",
                "Use that workflow as the source of truth for task order, artifacts, and quality gates.",
                "",
            ]
        )
        (commands_root / workflow_path.name).write_text(text, encoding="utf-8")


def _materialize_skill(repo_root: Path, dest: Path, options: LocalPluginOptions) -> None:
    if options.dry_run:
        return
    materialize_subject_package(
        MaterializeOptions(
            source=repo_root,
            out=dest,
            subject=options.subject,
            flavor="full",
            coverage=options.coverage,
        )
    )
```

Add marketplace update:

```python
def _load_marketplace(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return {"name": "personal", "interface": {"displayName": "Personal"}, "plugins": []}
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"{path} must contain a JSON object")
    payload.setdefault("name", "personal")
    payload.setdefault("interface", {"displayName": "Personal"})
    payload.setdefault("plugins", [])
    if not isinstance(payload["plugins"], list):
        raise ValueError(f"{path} plugins field must be a list")
    return payload


def _upsert_marketplace_entry(paths: CodexPluginPaths, *, dry_run: bool) -> None:
    payload = _load_marketplace(paths.marketplace_path)
    entry = {
        "name": PLUGIN_ID,
        "source": {"source": "local", "path": paths.marketplace_source_path},
        "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
        "category": "Education",
    }
    plugins = payload["plugins"]
    for index, current in enumerate(plugins):
        if isinstance(current, dict) and current.get("name") == PLUGIN_ID:
            plugins[index] = entry
            break
    else:
        plugins.append(entry)
    _write_json(paths.marketplace_path, payload, dry_run=dry_run)
```

Add install/remove entrypoints:

```python
def _selected_plugin_targets(target: str) -> tuple[str, ...]:
    if target == "all":
        return ("codex", "claude")
    if target in {"codex", "claude"}:
        return (target,)
    return ()


def install_local_plugin(options: LocalPluginOptions) -> LocalPluginResult:
    repo_root = Path(options.repo_root).expanduser().resolve()
    plugin = load_plugin_distribution(repo_root).plugins[PLUGIN_ID]
    version = _version(repo_root)
    installed: dict[str, Path] = {}
    changed = False

    for target in _selected_plugin_targets(options.target):
        if target == "codex":
            paths = resolve_codex_plugin_paths(PLUGIN_ID, marketplace_path=options.codex_marketplace_path)
            plugin_root = paths.plugin_root
        else:
            plugin_root = _claude_plugin_root(PLUGIN_ID, options.claude_plugin_parent)

        _remove_existing(plugin_root, overwrite=options.overwrite, dry_run=options.dry_run)
        if not options.dry_run:
            plugin_root.mkdir(parents=True, exist_ok=True)

        if target == "codex":
            _write_codex_manifest(plugin_root / ".codex-plugin" / "plugin.json", plugin, version, dry_run=options.dry_run)
            _write_codex_mcp_manifest(plugin_root / ".mcp.json", plugin.mcp_server_name, dry_run=options.dry_run)
            _upsert_marketplace_entry(paths, dry_run=options.dry_run)
        else:
            _write_claude_manifest(plugin_root / ".claude-plugin" / "plugin.json", plugin, version, dry_run=options.dry_run)

        _materialize_skill(repo_root, plugin_root / "skills" / SKILL_DIR_NAME, options)
        _write_commands(repo_root, plugin_root / "commands", dry_run=options.dry_run)
        installed[target] = plugin_root
        changed = True

    return LocalPluginResult(installed_roots=installed, changed=changed)
```

Implement `remove_local_plugin()`:

```python
def remove_local_plugin(
    *,
    target: str = "all",
    dry_run: bool = False,
    codex_marketplace_path: Path | str | None = None,
    claude_plugin_parent: Path | str | None = None,
) -> int:
    removed = 0
    for selected in _selected_plugin_targets(target):
        if selected == "codex":
            paths = resolve_codex_plugin_paths(PLUGIN_ID, marketplace_path=codex_marketplace_path)
            plugin_root = paths.plugin_root
            if plugin_root.exists() or plugin_root.is_symlink():
                if not _is_managed_plugin_root(plugin_root):
                    continue
                if not dry_run:
                    shutil.rmtree(plugin_root)
                removed += 1
            if paths.marketplace_path.is_file() and not dry_run:
                payload = _load_marketplace(paths.marketplace_path)
                payload["plugins"] = [
                    entry
                    for entry in payload["plugins"]
                    if not (isinstance(entry, dict) and entry.get("name") == PLUGIN_ID)
                ]
                _write_json(paths.marketplace_path, payload, dry_run=False)
            continue

        plugin_root = _claude_plugin_root(PLUGIN_ID, claude_plugin_parent)
        if plugin_root.exists() or plugin_root.is_symlink():
            if not _is_managed_plugin_root(plugin_root):
                continue
            if not dry_run:
                shutil.rmtree(plugin_root)
            removed += 1
    return removed
```

- [ ] **Step 4: Run local plugin tests and verify pass**

Run:

```bash
python3 -m unittest tests.test_local_plugin_installer -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add packages/python-qiongli/src/qiongli/local_plugin_installer.py tests/test_local_plugin_installer.py
git commit -m "feat(plugin): generate local full qiongli plugin"
```

## Task 3: Wire Plugin Surface Into Universal Installer

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/universal_installer.py`
- Modify: `tests/test_universal_installer.py`

- [ ] **Step 1: Write failing installer behavior tests**

Append these tests to `tests/test_universal_installer.py` inside `UniversalInstallerTests`:

```python
    def test_plugin_surface_installs_codex_plugin_without_global_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            marketplace_path = temp_root / ".agents" / "plugins" / "marketplace.json"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["QIONGLI_CODEX_MARKETPLACE_PATH"] = str(marketplace_path)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="full",
                        install_cli=False,
                        doctor=False,
                        surface="plugin",
                    )
                )

            plugin_root = marketplace_path.parent / "plugins" / "qiongli"
            self.assertEqual(result, 0)
            self.assertTrue((plugin_root / ".codex-plugin" / "plugin.json").is_file())
            self.assertTrue((plugin_root / "skills" / "qiongli-workflow" / "SKILL.md").is_file())
            self.assertFalse((codex_home / "skills" / "qiongli-workflow" / "SKILL.md").exists())
            self.assertFalse((codex_home / "config.toml").exists())

    def test_both_surface_installs_codex_plugin_and_global_skill(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            marketplace_path = temp_root / ".agents" / "plugins" / "marketplace.json"
            env = os.environ.copy()
            env["CODEX_HOME"] = str(codex_home)
            env["QIONGLI_CODEX_MARKETPLACE_PATH"] = str(marketplace_path)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="codex",
                        profile="full",
                        install_cli=False,
                        doctor=False,
                        surface="both",
                    )
                )

            self.assertEqual(result, 0)
            self.assertTrue((marketplace_path.parent / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").is_file())
            self.assertTrue((codex_home / "skills" / "qiongli-workflow" / "SKILL.md").is_file())

    def test_plugin_surface_all_keeps_full_mcp_for_antigravity_and_hermes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            temp_root = Path(tmp_dir)
            project_dir = temp_root / "project"
            project_dir.mkdir(parents=True)
            codex_home = temp_root / "codex-home"
            claude_home = temp_root / ".claude"
            antigravity_home = temp_root / "antigravity-home"
            hermes_home = temp_root / "hermes-home"
            marketplace_path = temp_root / ".agents" / "plugins" / "marketplace.json"
            env = os.environ.copy()
            env["HOME"] = str(temp_root)
            env["CODEX_HOME"] = str(codex_home)
            env["CLAUDE_CODE_HOME"] = str(claude_home)
            env["ANTIGRAVITY_HOME"] = str(antigravity_home)
            env["HERMES_HOME"] = str(hermes_home)
            env["QIONGLI_CODEX_MARKETPLACE_PATH"] = str(marketplace_path)
            env["PATH"] = ""

            with mock.patch.dict(os.environ, env, clear=True):
                result = install(
                    InstallOptions(
                        repo_root=REPO_ROOT,
                        project_dir=project_dir,
                        target="all",
                        profile="full",
                        install_cli=False,
                        doctor=False,
                        surface="plugin",
                    )
                )

            antigravity_config = json.loads((antigravity_home / "settings.json").read_text(encoding="utf-8"))
            hermes_config = json.loads((hermes_home / "settings.json").read_text(encoding="utf-8"))

            self.assertEqual(result, 0)
            self.assertTrue((marketplace_path.parent / "plugins" / "qiongli" / ".codex-plugin" / "plugin.json").is_file())
            self.assertFalse((codex_home / "config.toml").exists())
            self.assertEqual(antigravity_config["mcpServers"]["qiongli"]["command"], "qiongli")
            self.assertEqual(hermes_config["mcpServers"]["qiongli"]["command"], "qiongli")
```

- [ ] **Step 2: Run installer tests and verify failure**

Run:

```bash
python3 -m unittest tests.test_universal_installer -v
```

Expected: FAIL because `surface` is not used by the installer flow.

- [ ] **Step 3: Add container selection helpers**

In `packages/python-qiongli/src/qiongli/universal_installer.py`, import local plugin helpers:

```python
from .local_plugin_installer import LocalPluginOptions, install_local_plugin, remove_local_plugin
```

Add helper functions near `normalize_parts()`:

```python
def _default_container_parts(surface: str) -> tuple[str, ...]:
    if surface == "skills":
        return ("globals",)
    if surface == "plugin":
        return ("plugin",)
    if surface == "both":
        return ("globals", "plugin")
    raise ValueError(f"Unsupported surface: {surface}")


def _container_parts(options: InstallOptions, selected_parts: tuple[str, ...] | None) -> tuple[bool, bool]:
    if selected_parts is not None:
        return "globals" in selected_parts, "plugin" in selected_parts
    defaults = _default_container_parts(options.surface)
    return "globals" in defaults, "plugin" in defaults


def _plugin_managed_mcp_targets(target: str, install_plugin: bool) -> set[str]:
    if not install_plugin:
        return set()
    if target == "all":
        return {"codex", "claude"}
    if target in {"codex", "claude"}:
        return {target}
    return set()
```

- [ ] **Step 4: Validate surface in `install()` and `remove()`**

In `install()`, after target validation:

```python
    if options.surface not in SURFACE_CHOICES:
        raise ValueError(f"Unsupported surface: {options.surface}")
```

Carry `surface=options.surface` when recreating `InstallOptions` inside `apply_profile()`.

In `remove()`, validate `options.surface` the same way after target validation.

- [ ] **Step 5: Use surface to select globals and plugin install**

Replace the existing install booleans in `install()`:

```python
    selected_parts = normalize_parts(options.parts)
    install_globals, install_plugin = _container_parts(options, selected_parts)
    install_project = False if selected_parts is None else "project" in selected_parts
    install_cli = bool(options.install_cli) if selected_parts is None else "cli" in selected_parts
    install_mcp = bool(options.install_mcp) if selected_parts is None else "mcp" in selected_parts
    doctor = bool(options.doctor) if selected_parts is None else "doctor" in selected_parts
```

Print surface in the install header:

```python
    print(f"  surface: {options.surface}")
```

- [ ] **Step 6: Install local plugin before MCP client config**

After the legacy global skill copy block and before shell CLI installation, add:

```python
    if install_plugin:
        _print_section("Local Plugin")
        plugin_result = install_local_plugin(
            LocalPluginOptions(
                repo_root=repo_root,
                subject=options.subject,
                coverage=options.coverage,
                target=options.target,
                mode=options.mode,
                overwrite=options.overwrite,
                dry_run=options.dry_run,
            )
        )
        if plugin_result.installed_roots:
            for plugin_target, plugin_root in plugin_result.installed_roots.items():
                _print_result("Plugin", f"{plugin_target}: {plugin_root}", "ok")
        else:
            _print_result("Plugin", f"target {options.target} has no plugin surface", "skip")
```

- [ ] **Step 7: Avoid duplicate MCP registration for plugin-managed targets**

Change `_install_mcp_client_config()` signature:

```python
def _install_mcp_client_config(options: InstallOptions, *, skip_targets: set[str] | None = None) -> None:
```

Inside it:

```python
    skip_targets = skip_targets or set()
    for target, label in targets:
        normalized = "claude" if target == "claude-code" else target
        if normalized in skip_targets:
            _print_mcp_config_result(label, "skipped", Path("<plugin manifest>"), "MCP is managed by local plugin manifest")
            continue
        result = install_mcp_config(target=target, dry_run=options.dry_run)
        _print_mcp_config_result(label, result.status, result.path, result.detail)
```

In `install()`, replace:

```python
    if install_mcp:
        _install_mcp_client_config(options)
```

with:

```python
    if install_mcp:
        _install_mcp_client_config(
            options,
            skip_targets=_plugin_managed_mcp_targets(options.target, install_plugin),
        )
```

- [ ] **Step 8: Wire plugin removal**

In `remove()`, update selected booleans:

```python
    remove_plugin = "plugin" in selected_parts or options.surface in {"plugin", "both"}
```

After global skill removal:

```python
    if remove_plugin:
        _print_section("Local Plugin")
        removed += remove_local_plugin(target=options.target, dry_run=options.dry_run)
```

- [ ] **Step 9: Run installer tests and verify pass**

Run:

```bash
python3 -m unittest tests.test_universal_installer tests.test_local_plugin_installer -v
```

Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add packages/python-qiongli/src/qiongli/universal_installer.py tests/test_universal_installer.py
git commit -m "feat(installer): wire local plugin surface"
```

## Task 4: Preserve Marketplace Lite MCP Contract

**Files:**
- Modify: `tests/test_plugin_manifests.py`
- No production code should change in this task unless the test exposes a regression.

- [ ] **Step 1: Strengthen marketplace plugin tests**

In `tests/test_plugin_manifests.py`, update `test_codex_plugin_bundles_qiongli_mcp_server` to assert the marketplace artifact remains lite:

```python
        self.assertEqual(server["command"], "node")
        self.assertEqual(server["args"], ["./mcp/qiongli-literature-provider/index.mjs"])
        self.assertNotEqual(server["command"], "qiongli")
        self.assertNotIn("mcp serve", json.dumps(server))
```

Update `test_claude_plugin_bundles_qiongli_mcp_server` similarly:

```python
        self.assertEqual(server["command"], "node")
        self.assertEqual(
            server["args"],
            ["${CLAUDE_PLUGIN_ROOT}/mcp/qiongli-literature-provider/index.mjs"],
        )
        self.assertNotEqual(server["command"], "qiongli")
        self.assertNotIn("mcp serve", json.dumps(server))
```

- [ ] **Step 2: Run marketplace tests**

Run:

```bash
python3 -m unittest tests.test_plugin_manifests -v
```

Expected: PASS. If this fails because marketplace artifacts switched to full MCP, restore `tooling/scripts/build_plugin_artifacts.py` to emit the existing Node literature provider for marketplace builds.

- [ ] **Step 3: Commit**

```bash
git add tests/test_plugin_manifests.py
git commit -m "test(plugin): preserve marketplace lite mcp contract"
```

## Task 5: Document The New Install Matrix

**Files:**
- Modify: `docs/reference/cli.md`
- Modify: `docs/guide/install.md`
- Modify: `docs/zh/guide/install.md`
- Modify: `README_CN.md`
- Modify: `docs/advanced/plugin-first-architecture.md`
- Modify: `docs/advanced/qiongli-cli-plugin-structure.html`

- [ ] **Step 1: Update CLI reference**

In `docs/reference/cli.md`, update the `qiongli install` synopsis:

```markdown
qiongli install \
  [--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting] \
  [--coverage complete|focused] \
  [--target codex|claude|antigravity|hermes|all] \
  [--surface skills|plugin|both] \
  [--mode copy|link] \
  [--project-dir <path>] \
  [--overwrite] \
  [--doctor] \
  [--dry-run]
```

Add examples:

```markdown
qiongli install --profile full --target codex --surface plugin
qiongli install --profile full --target all --surface plugin
qiongli install --profile full --target all --surface both
qiongli remove --target codex --surface plugin
```

Add this behavior note:

```markdown
`--surface plugin` installs a local plugin bundle backed by the full Python MCP server.
For Codex, the CLI writes a personal marketplace entry and a plugin payload whose `.mcp.json`
launches `qiongli mcp serve --transport stdio`. Marketplace plugins remain the lite no-Python path
and continue to bundle the Node literature provider.
```

- [ ] **Step 2: Update install guides**

In `docs/guide/install.md` and `docs/zh/guide/install.md`, add a matrix row:

```markdown
| Full local plugin | `qiongli install --profile full --target all --surface plugin` | Full local Qiongli inside a client-native plugin container | Python 3.12+ |
```

In the Chinese guide use:

```markdown
| 本地 full plugin | `qiongli install --profile full --target all --surface plugin` | 让 CLI 生成客户端 plugin，并接入统一 full MCP | Python 3.12+ |
```

- [ ] **Step 3: Update README_CN**

In `README_CN.md`, update the “不知道选什么” path:

````markdown
如果你要完整本地 Qiongli，优先使用：

```bash
qiongli install --profile full --target all --surface plugin
```

这会生成本地 plugin，并把 MCP 接到 `qiongli mcp serve --transport stdio`。
Marketplace plugin 仍然是无 CLI / 无 Python 环境下的轻量入口。
```
````

- [ ] **Step 4: Update architecture docs**

In `docs/advanced/plugin-first-architecture.md`, replace the current compatibility paragraph with:

````markdown
For full local Qiongli, the CLI can now install a client-native local plugin bundle:

```bash
qiongli install --profile full --target all --surface plugin
```

That local plugin uses the same skill/workflow payload shape as marketplace artifacts, but its MCP
configuration points at the unified Python-backed full MCP server. Marketplace artifacts keep the
bundled Node literature MCP so they remain usable without Python or the Qiongli CLI.
```
````

- [ ] **Step 5: Update the HTML explanation**

In `docs/advanced/qiongli-cli-plugin-structure.html`, replace the final note:

```html
这张图描述的是建议改造后的目标结构，不是当前 v1.7.0 的完整现状。
```

with:

```html
这张图描述的是 CLI full plugin surface 的目标结构：CLI local plugin 使用 full MCP；
marketplace plugin 保留 lite literature MCP。
```

- [ ] **Step 6: Run documentation checks**

Run:

```bash
rg -n "surface plugin|full plugin|lite literature|qiongli install --profile full --target all --surface plugin" README_CN.md docs
```

Expected: output includes the README, install guides, CLI reference, architecture doc, and HTML page.

- [ ] **Step 7: Commit**

```bash
git add README_CN.md docs/reference/cli.md docs/guide/install.md docs/zh/guide/install.md docs/advanced/plugin-first-architecture.md docs/advanced/qiongli-cli-plugin-structure.html
git commit -m "docs(installer): document full plugin surface"
```

## Task 6: End-To-End Verification

**Files:**
- No new source files.
- Update tests only if verification exposes a gap.

- [ ] **Step 1: Run targeted unit tests**

Run:

```bash
python3 -m unittest \
  tests.test_cli \
  tests.test_universal_installer \
  tests.test_local_plugin_installer \
  tests.test_plugin_manifests \
  -v
```

Expected: PASS.

- [ ] **Step 2: Run full installer dry-run smoke**

Run:

```bash
python3 -m qiongli.cli install --profile full --target all --surface plugin --dry-run
```

Expected output includes:

```text
surface: plugin
== Local Plugin ==
Plugin
== MCP Client Config ==
```

Expected output does not include provider secrets.

- [ ] **Step 3: Run Codex temp-home install smoke**

Run:

```bash
tmp="$(mktemp -d)"
HOME="$tmp" \
CODEX_HOME="$tmp/.codex" \
QIONGLI_CODEX_MARKETPLACE_PATH="$tmp/.agents/plugins/marketplace.json" \
PATH="$PATH" \
python3 -m qiongli.cli install --profile full --target codex --surface plugin --no-cli
test -f "$tmp/.agents/plugins/plugins/qiongli/.codex-plugin/plugin.json"
test -f "$tmp/.agents/plugins/plugins/qiongli/.mcp.json"
test -f "$tmp/.agents/plugins/plugins/qiongli/skills/qiongli-workflow/SKILL.md"
python3 -m json.tool "$tmp/.agents/plugins/plugins/qiongli/.mcp.json" >/dev/null
python3 -m json.tool "$tmp/.agents/plugins/marketplace.json" >/dev/null
```

Expected: command exits 0.

- [ ] **Step 4: Inspect Codex full MCP manifest**

Run:

```bash
tmp="$(mktemp -d)"
HOME="$tmp" \
CODEX_HOME="$tmp/.codex" \
QIONGLI_CODEX_MARKETPLACE_PATH="$tmp/.agents/plugins/marketplace.json" \
PATH="$PATH" \
python3 -m qiongli.cli install --profile full --target codex --surface plugin --no-cli
rg -n '"command": "qiongli"|"mcp"|"serve"|"stdio"|qiongli-literature-provider' "$tmp/.agents/plugins/plugins/qiongli/.mcp.json"
```

Expected:

```text
"command": "qiongli"
"mcp"
"serve"
"stdio"
```

Expected: no `qiongli-literature-provider` line in the CLI-generated plugin `.mcp.json`.

- [ ] **Step 5: Inspect marketplace lite MCP still uses Node**

Run:

```bash
python3 scripts/materialize_distribution_payloads.py --target plugin --out /tmp/qiongli-marketplace-check --force
rg -n '"command": "node"|qiongli-literature-provider|"command": "qiongli"' /tmp/qiongli-marketplace-check/plugins/qiongli/.mcp.json
```

Expected:

```text
"command": "node"
qiongli-literature-provider
```

Expected: no `"command": "qiongli"` in the marketplace artifact `.mcp.json`.

- [ ] **Step 6: Run MCP server tool-list smoke**

Run:

```bash
printf '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}\n' | python3 -m qiongli.bridges.mcp_server_stdio
```

Expected output includes:

```text
qiongli_literature_search
qiongli_orchestrator_route
qiongli_task_plan
qiongli_task_run
```

- [ ] **Step 7: Final boundary review**

Run:

```bash
git diff --name-only
```

Expected changed files are limited to:

```text
packages/python-qiongli/src/qiongli/cli.py
packages/python-qiongli/src/qiongli/local_plugin_installer.py
packages/python-qiongli/src/qiongli/universal_installer.py
tests/test_cli.py
tests/test_local_plugin_installer.py
tests/test_plugin_manifests.py
tests/test_universal_installer.py
README_CN.md
docs/reference/cli.md
docs/guide/install.md
docs/zh/guide/install.md
docs/advanced/plugin-first-architecture.md
docs/advanced/qiongli-cli-plugin-structure.html
```

Also verify:

```bash
rg -n "OPENAI_API_KEY|ANTHROPIC_API_KEY|SEMANTIC_SCHOLAR_API_KEY|QIONGLI_OPENALEX_EMAIL" \
  packages/python-qiongli/src/qiongli tests README_CN.md docs
```

Expected: only documentation or config field names appear; no real secret values appear.

- [ ] **Step 8: Commit verification fixes**

If verification required small fixes, commit them:

```bash
git add packages/python-qiongli/src/qiongli tests README_CN.md docs
git commit -m "fix(installer): harden full plugin surface"
```

If no fixes are needed, do not create an empty commit.

## Rollout Notes

- First release keeps `--surface skills` as default.
- Recommended full local command becomes:

```bash
qiongli install --profile full --target all --surface plugin
```

- Marketplace install remains:

```text
skill workflow + bundled lite Node literature MCP
```

- CLI local plugin install becomes:

```text
skill workflow + full Python MCP + orchestrator + doctor + task plan/run
```

## Self-Review

- Spec coverage: The plan covers CLI parser, installer model, local plugin payload generation, full MCP routing, marketplace-lite preservation, docs, tests, and verification.
- Placeholder scan: No incomplete implementation markers are present.
- Type consistency: `surface`, `LocalPluginOptions`, `LocalPluginResult`, `resolve_codex_plugin_paths`, `install_local_plugin`, and `remove_local_plugin` are named consistently across tasks.
- Repository boundary: Marketplace release generator remains separate. Local plugin payload generation lives in the Python CLI package because it is runtime installer behavior, not published marketplace catalog state.
