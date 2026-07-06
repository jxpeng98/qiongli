# Plugin-Aware Check And Doctor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make CLI-installed Qiongli plugins discoverable by `qiongli check` and visible/readable in Codex plugin details.

**Architecture:** Add a small install-surface discovery layer that reports plugin, MCP, and legacy skill state without changing installer ownership rules. Fix generated Codex plugin manifests and materialized skill frontmatter so local plugins pass Codex plugin validation.

**Tech Stack:** Python 3.12, unittest, PyYAML, Codex local plugin manifests, Qiongli universal installer.

---

### Task 1: Fix Codex Plugin Manifest And Skill Frontmatter

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/local_plugin_installer.py`
- Modify: `tooling/scripts/build_plugin_artifacts.py`
- Modify: `packages/python-qiongli/src/qiongli/subject_materializer.py`
- Modify: `content/workflow/SKILL.md`
- Modify: `tests/test_local_plugin_installer.py`
- Modify: `tests/test_plugin_manifests.py`
- Modify: `tests/test_skill_doc_generation.py`
- Modify: `tests/test_sync_versions.py`

- [ ] **Step 1: Write failing manifest tests**

```python
self.assertNotIn("category", codex_manifest)
self.assertEqual(codex_manifest["interface"]["category"], "Education")
```

- [ ] **Step 2: Write failing YAML frontmatter tests**

```python
frontmatter = skill_text.split("---", 2)[1]
payload = yaml.safe_load(frontmatter)
self.assertEqual(payload["name"], "qiongli")
self.assertIn("Qiongli version:", payload["description"])
```

- [ ] **Step 3: Run targeted tests to verify red**

Run:

```bash
python3 -m unittest tests.test_local_plugin_installer tests.test_plugin_manifests tests.test_skill_doc_generation tests.test_sync_versions
```

Expected: FAIL on Codex manifest top-level `category` and invalid YAML frontmatter.

- [ ] **Step 4: Remove unsupported Codex manifest field**

Implementation:

```python
def _codex_manifest(plugin: PluginDefinition, version: str) -> dict[str, Any]:
    return {
        "name": plugin.id,
        "version": version,
        "description": plugin.description,
        "author": plugin.author,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": _keywords(plugin, "codex-skills"),
        "skills": "./skills/",
        "mcpServers": "./.mcp.json",
        "interface": {
            "displayName": plugin.display_name,
            "category": plugin.category,
        },
    }
```

Keep `category` in marketplace entries and Claude plugin manifests.

- [ ] **Step 5: Quote generated skill descriptions**

Implementation:

```python
def _yaml_quote(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)
```

Use it for `description` in materialized `SKILL.md` frontmatter and quote the canonical `content/workflow/SKILL.md` description.

- [ ] **Step 6: Verify local plugin validation**

Run:

```bash
python3 /Users/pengjiaxin/.codex/skills/.system/plugin-creator/scripts/validate_plugin.py <generated-plugin-root>
```

Expected: validation succeeds.

### Task 2: Add Install Surface Discovery

**Files:**
- Create: `packages/python-qiongli/src/qiongli/install_discovery.py`
- Modify: `tests/test_cli.py`
- Modify: `packages/python-qiongli/src/qiongli/cli.py`

- [ ] **Step 1: Write failing check JSON test for Codex plugin**

```python
self.assertEqual(payload["installed"]["codex"]["surface"], "plugin")
self.assertTrue(payload["installed"]["codex"]["plugin"]["installed"])
self.assertEqual(payload["installed"]["codex"]["version"], "v9.9.9")
```

- [ ] **Step 2: Write failing check JSON test for legacy fallback**

```python
self.assertEqual(payload["installed"]["codex"]["surface"], "legacy_skill")
self.assertTrue(payload["installed"]["codex"]["skill"]["installed"])
```

- [ ] **Step 3: Implement discovery dataclasses**

```python
@dataclass(frozen=True)
class SurfaceState:
    client: str
    surface: str
    installed: bool
    version: str | None
    subject: str | None
    coverage: str | None
    path: Path
    plugin: dict[str, object]
    skill: dict[str, object]
    mcp: dict[str, object]
```

- [ ] **Step 4: Discover Codex and Claude plugins first**

Use `resolve_codex_plugin_paths()` and a public Claude plugin path helper. A plugin is installed when its plugin manifest exists and its managed marker or skill payload is present.

- [ ] **Step 5: Discover Antigravity and Hermes MCP configs**

Use existing `default_antigravity_config_path()` and `default_hermes_config_path()` paths. Report whether a managed `qiongli` MCP server is configured, then fallback to legacy skill paths for version/subject when present.

- [ ] **Step 6: Keep backward-compatible `installed` payload**

`payload["installed"][client]` should still expose `path`, `installed`, `version`, `subject`, and `coverage`, with new nested `plugin`, `skill`, `mcp`, and `surface` fields.

### Task 3: Update `qiongli check` And `doctor`

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `tests/test_cli.py`

- [ ] **Step 1: Replace direct `_installed_skill_dirs()` use**

Use:

```python
installed = discover_install_surfaces()
```

- [ ] **Step 2: Rename text output section**

Change:

```text
3) Installed Workflow Skills (Payload)
```

To:

```text
3) Installed Client Surfaces
```

- [ ] **Step 3: Include install-surface summary in `doctor`**

After orchestrator doctor output, print a short `Client Integration` section based on discovery state. Do not fail doctor only because optional clients are absent.

- [ ] **Step 4: Run targeted CLI tests**

Run:

```bash
python3 -m unittest tests.test_cli
```

Expected: PASS.

### Task 4: Documentation And Validation

**Files:**
- Modify: `README.md`
- Modify: `docs/reference/cli.md`
- Modify: `docs/zh/reference/cli.md`
- Modify: `docs/advanced/plugin-first-architecture.md`
- Modify: `docs/zh/guide/install.md`

- [ ] **Step 1: Document `check` vs `doctor` split**

Add that `check` reports CLI package, host CLI readiness, and client install surfaces; `doctor` validates runtime/orchestrator health plus a non-fatal client integration summary.

- [ ] **Step 2: Explain Codex TUI failure mode**

Document that Codex may list a personal plugin as available while details fail if the local plugin manifest or skill frontmatter is invalid.

- [ ] **Step 3: Run validation**

Run:

```bash
python3 -m unittest tests.test_cli tests.test_local_plugin_installer tests.test_plugin_manifests tests.test_skill_doc_generation tests.test_sync_versions
git diff --check
```

Expected: PASS and no whitespace errors.

### Task 5: Refresh Local Plugin For Manual Verification

**Files:**
- No repository files required beyond implementation.

- [ ] **Step 1: Reinstall local Codex plugin**

Run:

```bash
python3 -m qiongli.cli install --target codex --surface plugin --overwrite
```

- [ ] **Step 2: Validate installed plugin**

Run:

```bash
python3 /Users/pengjiaxin/.codex/skills/.system/plugin-creator/scripts/validate_plugin.py /Users/pengjiaxin/.agents/plugins/plugins/qiongli
```

Expected: validation succeeds, and Codex can open qiongli plugin details after restart/reload.
