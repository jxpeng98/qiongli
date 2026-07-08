# Platform Target Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement roadmap Stage 12 by moving platform artifact boundary rules into a canonical registry and using it to prevent cross-platform metadata leakage.

**Architecture:** Add `content/distribution/platform-targets.yaml` as the canonical platform target registry and a small Python loader under `qiongli.platform_targets`. Refactor release artifact validation to read target rules from the registry, and fix the Claude Desktop direct plugin builder so it emits a Desktop-specific bundle instead of a hybrid Codex/Claude Code plugin ZIP.

**Tech Stack:** Python 3.12, PyYAML, unittest, existing release artifact builders and marketplace validators.

---

## File Map

- Create: `content/distribution/platform-targets.yaml`
  - Declares target IDs, artifact shape, required/forbidden paths, command surface, MCP mode, archive format, and validator IDs.
- Create: `packages/python-qiongli/src/qiongli/platform_targets.py`
  - Loads and validates the registry, exposes lookup helpers, and converts registry entries into path checks.
- Modify: `tooling/scripts/build_plugin_artifacts.py`
  - Builds Claude Desktop direct plugin artifacts from a Desktop-only copy that removes Codex metadata, `.mcp.json`, and Codex workflow wrapper skills.
- Modify: `tooling/scripts/validate_marketplace_install.py`
  - Replaces hard-coded marketplace `ARTIFACT_SPECS` with registry-backed target lookup for Codex, Claude Code, Claude Desktop direct plugin, and Desktop skill ZIP validation.
- Modify: `tests/test_plugin_distribution_contract.py`
  - Adds registry contract tests and updates direct Desktop plugin expectations.
- Modify: `tests/test_plugin_artifacts.py`
  - Adds artifact-level negative checks for direct Desktop plugin ZIPs.

## Task 1: Add Registry Contract Tests

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`
- Create later: `content/distribution/platform-targets.yaml`
- Create later: `packages/python-qiongli/src/qiongli/platform_targets.py`

- [x] **Step 1: Write failing registry tests**

Add tests that import `load_platform_targets(REPO_ROOT)` and assert:

```python
expected_targets = {
    "codex-marketplace-plugin",
    "claude-code-marketplace-plugin",
    "claude-desktop-direct-plugin",
    "claude-desktop-skill-zip",
    "antigravity-local-plugin",
    "npm-plugin-lite",
    "pypi-full-runtime",
}
self.assertTrue(expected_targets.issubset(targets))
self.assertIn(".codex-plugin/plugin.json", targets["codex-marketplace-plugin"].required_paths)
self.assertIn(".codex-plugin/", targets["claude-desktop-direct-plugin"].forbidden_paths)
self.assertIn(".mcp.json", targets["claude-desktop-direct-plugin"].forbidden_paths)
self.assertEqual(targets["claude-desktop-direct-plugin"].archive_format, "zip")
```

- [x] **Step 2: Run the registry test and verify RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_declares_boundary_rules -q
```

Expected: fails because `qiongli.platform_targets` or the registry file does not exist.

## Task 2: Add Registry Loader And Registry File

**Files:**
- Create: `content/distribution/platform-targets.yaml`
- Create: `packages/python-qiongli/src/qiongli/platform_targets.py`

- [x] **Step 1: Implement registry data model**

Create a frozen dataclass:

```python
@dataclass(frozen=True)
class PlatformTarget:
    target_id: str
    display_name: str
    artifact_kind: str
    archive_format: str
    source_inputs: tuple[str, ...]
    required_paths: tuple[str, ...]
    allowed_wrapper_dirs: tuple[str, ...]
    forbidden_paths: tuple[str, ...]
    bundled_mcp_mode: str
    command_surface: str
    validator: str
```

Expose:

```python
def load_platform_targets(repo_root: Path | None = None) -> dict[str, PlatformTarget]
def require_platform_target(targets: Mapping[str, PlatformTarget], target_id: str) -> PlatformTarget
def validate_platform_target_registry(repo_root: Path | None = None) -> list[str]
```

- [x] **Step 2: Implement YAML registry**

Add targets for:

- `codex-marketplace-plugin`
- `claude-code-marketplace-plugin`
- `claude-desktop-direct-plugin`
- `claude-desktop-skill-zip`
- `antigravity-local-plugin`
- `npm-plugin-lite`
- `pypi-full-runtime`

The direct Desktop plugin target must require `.claude-plugin/plugin.json`, `commands/`, `skills/qiongli-workflow/`, and `mcp/qiongli-literature-provider/index.mjs`; it must forbid `.codex-plugin/`, `.mcp.json`, and Codex wrapper skill directories matching `skills/qiongli-[!w]*/` so the canonical `skills/qiongli-workflow/` directory is preserved.

- [x] **Step 3: Run registry tests and verify GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_declares_boundary_rules -q
```

Expected: `OK`.

## Task 3: Fix Direct Desktop Plugin Artifact Boundary

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`
- Modify: `tests/test_plugin_artifacts.py`
- Modify: `tooling/scripts/build_plugin_artifacts.py`
- Modify: `tooling/scripts/validate_marketplace_install.py`

- [x] **Step 1: Write failing artifact boundary tests**

Update direct Desktop plugin tests to assert:

```python
self.assertNotIn(f"{plugin_name}/.codex-plugin/plugin.json", names)
self.assertNotIn(f"{plugin_name}/.mcp.json", names)
self.assertNotIn(f"{plugin_name}/skills/{plugin_name}-lit-review/SKILL.md", names)
self.assertIn(f"{plugin_name}/.claude-plugin/plugin.json", names)
self.assertIn(f"{plugin_name}/commands/lit-review.md", names)
self.assertIn(f"{plugin_name}/mcp/qiongli-literature-provider/index.mjs", names)
self.assertIn(f"{plugin_name}/skills/qiongli-workflow/SKILL.md", names)
```

- [x] **Step 2: Run direct Desktop plugin tests and verify RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_build_artifacts_includes_direct_desktop_plugin -q
```

Expected: fails because the artifact still contains `.codex-plugin`.

- [x] **Step 3: Build Desktop-only plugin root**

In `_build_claude_desktop_plugin()`, materialize the normal Claude plugin package, then remove forbidden paths before zipping:

```python
_remove_desktop_forbidden_paths(plugin_dest)
```

`_remove_desktop_forbidden_paths()` must delete `.codex-plugin/`, `.mcp.json`, and direct Codex workflow wrapper skill directories under `skills/` while preserving `skills/qiongli-workflow/`.

- [x] **Step 4: Refactor validator to use registry**

Load target rules once in `validate()`. `_validate_direct_desktop_plugin_artifact()` must assert all registry `required_paths` exist and all `forbidden_paths` are absent before detailed manifest and invocation checks.

- [x] **Step 5: Run artifact boundary tests and verify GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_build_artifacts_includes_direct_desktop_plugin tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_builds_platform_artifacts_and_checks_invocation -q
.venv/bin/python -m unittest tests.test_plugin_artifacts.PluginArtifactsTests.test_release_builds_expected_channel_artifacts -q
```

Expected: all `OK`.

## Task 4: Mark Roadmap Status And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [x] **Step 1: Mark Stage 12 minimum implementation status**

Add under Stage 12:

```markdown
Status: implemented on `dev` for a canonical platform target registry, registry-backed artifact boundary checks, and Claude Desktop direct plugin negative checks. Remaining follow-up: full release download generation from the registry and optional adapter extraction.
```

- [x] **Step 2: Run targeted verification**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract tests.test_plugin_artifacts -q
git diff --check
```

Expected: tests pass and whitespace check exits 0.
