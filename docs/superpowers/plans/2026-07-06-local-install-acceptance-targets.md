# Local Install Acceptance Targets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release local-install acceptance consume the platform target registry's `client_activation_check` policy and verify installed marker target metadata against the canonical registry.

**Architecture:** Keep `content/distribution/platform-targets.yaml` as the source of truth for which platform targets need local client activation. `release_local_install_check.py` should load registry targets whose `smoke.client_activation_check` is `local_install_acceptance`, validate only supported local installer targets there, and compare installed `.qiongli-managed.json` marker metadata to the registry-derived target metadata.

**Tech Stack:** Python standard library, existing `qiongli.platform_targets`, `unittest`.

---

## Files

- Modify: `tooling/scripts/release_local_install_check.py`
  - Import platform target loading helpers.
  - Add local acceptance target selection from registry smoke policy.
  - Validate installed plugin markers carry matching `platform_target` metadata.
- Modify: `tests/test_release_local_install_check.py`
  - Add a failing test proving marker target metadata is required and registry-derived.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record local-install acceptance target selection as registry-backed.

### Task 1: Add RED Marker Metadata Test

- [x] **Step 1: Add installed marker target metadata fixture**

Add this test to `tests/test_release_local_install_check.py`:

```python
    def test_validate_install_tree_requires_registry_target_metadata_in_markers(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            repo_root = root / "repo"
            sandbox = module.build_sandbox(root / "sandbox")
            self._write_repo_version(repo_root, "v9.9.9")
            self._write_platform_target_registry(repo_root)
            self._write_codex_plugin_tree(
                sandbox,
                manifest_category=False,
                include_target_metadata=False,
            )
            self._write_claude_plugin_tree(sandbox)
            self._write_antigravity_plugin_tree(sandbox)
            self._write_json(
                sandbox.antigravity_config_path,
                {"mcpServers": {}},
            )
            self._write_json(
                sandbox.hermes_config_path,
                {"mcpServers": {"qiongli": {"command": "qiongli", "args": module.QIONGLI_MCP_ARGS}}},
            )
            payload = {
                "installed": {
                    "codex": {"installed": True, "surface": "plugin"},
                    "claude": {"installed": True, "surface": "plugin"},
                    "antigravity": {
                        "installed": True,
                        "surface": "plugin",
                        "mcp": {
                            "path": str(sandbox.antigravity_plugin_root / "mcp_config.json"),
                            "source": "plugin",
                        },
                    },
                    "hermes": {"installed": True, "surface": "mcp"},
                }
            }

            with self.assertRaisesRegex(
                module.LocalInstallCheckError,
                "Codex plugin marker platform_target.target_id expected codex-marketplace-plugin",
            ):
                module.validate_install_tree(repo_root, sandbox, payload)
```

- [x] **Step 2: Add fixture registry writer**

Add this helper to `ReleaseLocalInstallCheckTests`:

```python
    def _write_platform_target_registry(self, repo_root: Path) -> None:
        registry = repo_root / "content" / "distribution" / "platform-targets.yaml"
        registry.parent.mkdir(parents=True, exist_ok=True)
        registry.write_text(
            """
schema_version: "1.0"
targets:
  codex-marketplace-plugin:
    display_name: Codex Marketplace Plugin
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: codex
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.claude-plugin/]
    bundled_mcp_mode: codex-plugin-local-node
    command_surface: slash-commands
    validator: codex-marketplace-plugin
    release_download:
      guide_label: Codex
      recommended_key: codex
      asset_groups: []
  claude-code-marketplace-plugin:
    display_name: Claude Code Marketplace Plugin
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: claude
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: claude-plugin-local-node
    command_surface: slash-commands
    validator: claude-code-marketplace-plugin
    release_download:
      guide_label: Claude Code
      recommended_key: claude_code
      asset_groups: []
  antigravity-local-plugin:
    display_name: Antigravity Local Plugin
    artifact_kind: local-plugin
    archive_format: directory
    adapter:
      kind: local-plugin
      plugin_manifest_platform: none
      materializer: local_plugin_installer
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: antigravity-python-runtime
    command_surface: slash-commands
    validator: antigravity-local-plugin
    release_download:
      guide_label: Antigravity
      recommended_key: antigravity
      asset_groups: []
  claude-desktop-skill-zip:
    display_name: Claude Desktop Skill ZIP
    artifact_kind: skill-zip
    archive_format: zip
    adapter:
      kind: skill-zip
      plugin_manifest_platform: none
      materializer: desktop_skill_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: not_applicable
    source_inputs: [content/workflow/**]
    required_paths: [SKILL.md]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: none
    command_surface: skill-workflows
    validator: claude-desktop-skill-zip
    release_download:
      guide_label: Claude Desktop/Web skills
      recommended_key: claude_desktop_skill
      asset_groups: []
""".lstrip(),
            encoding="utf-8",
        )
```

- [x] **Step 3: Run RED**

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check.ReleaseLocalInstallCheckTests.test_validate_install_tree_requires_registry_target_metadata_in_markers -q
```

Expected: FAIL because `validate_install_tree()` accepts markers without registry `platform_target` metadata.

### Task 2: Implement Registry-Backed Marker Validation

- [x] **Step 1: Import target helpers**

In `tooling/scripts/release_local_install_check.py`, add the repo Python source root to `sys.path`, then import:

```python
from qiongli.platform_targets import PlatformTarget, load_platform_targets
```

- [x] **Step 2: Add acceptance target mapping**

Add:

```python
LOCAL_INSTALL_TARGET_CLIENTS = {
    "codex-marketplace-plugin": ("codex", "Codex"),
    "claude-code-marketplace-plugin": ("claude", "Claude"),
    "antigravity-local-plugin": ("antigravity", "Antigravity"),
}


def local_install_acceptance_targets(repo_root: Path) -> dict[str, PlatformTarget]:
    return {
        target_id: target
        for target_id, target in load_platform_targets(repo_root).items()
        if target.smoke.get("client_activation_check") == "local_install_acceptance"
    }
```

- [x] **Step 3: Validate marker target metadata**

Add:

```python
def _validate_marker_platform_target(marker: dict[str, Any], target: PlatformTarget, label: str) -> None:
    payload = marker.get("platform_target")
    if not isinstance(payload, dict):
        payload = {}
    expected = {
        "target_id": target.target_id,
        "artifact_kind": target.artifact_kind,
        "archive_format": target.archive_format,
        "bundled_mcp_mode": target.bundled_mcp_mode,
        "command_surface": target.command_surface,
        "validator": target.validator,
    }
    for key, value in expected.items():
        _expect(
            payload.get(key) == value,
            f"{label} plugin marker platform_target.{key} expected {value}",
        )
```

In the existing marker loop, call `_validate_marker_platform_target(...)` for each target in `local_install_acceptance_targets(repo_root)` that is present in `LOCAL_INSTALL_TARGET_CLIENTS`.

- [x] **Step 4: Run GREEN**

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check.ReleaseLocalInstallCheckTests.test_validate_install_tree_requires_registry_target_metadata_in_markers -q
```

Expected: OK after fixture markers include target metadata in the existing helper writers, or fail until the test fixture helper writers are updated for the happy path.

### Task 3: Update Existing Happy Fixtures And Roadmap

- [x] **Step 1: Add marker metadata to test fixture writers**

Update `_write_codex_plugin_tree`, `_write_claude_plugin_tree`, and `_write_antigravity_plugin_tree` to include the expected `platform_target` object in `.qiongli-managed.json` for existing happy-path tests.

- [x] **Step 2: Update roadmap**

Update Stage 12 status and remaining product gap wording to say local-install acceptance now selects registry targets by `client_activation_check` and verifies installed marker metadata against the registry.

- [x] **Step 3: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check tests.test_local_plugin_installer tests.test_plugin_distribution_contract -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 4: Commit by category**

```bash
git add tooling/scripts/release_local_install_check.py tests/test_release_local_install_check.py
git commit -m "feat(release): validate local install target metadata"
git add docs/superpowers/plans/2026-07-06-local-install-acceptance-targets.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record local install acceptance target validation"
```
