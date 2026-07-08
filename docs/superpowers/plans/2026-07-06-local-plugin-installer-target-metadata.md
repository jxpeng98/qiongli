# Local Plugin Installer Target Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect Python local plugin installers to the platform target registry by recording registry-derived target metadata in installed plugin markers and marketplace entries.

**Architecture:** Map local installer platforms (`codex`, `claude`, `antigravity`) to platform target IDs, load `content/distribution/platform-targets.yaml`, and require the target entry during install. The installed `.qiongli-managed.json` marker and Codex marketplace metadata will include target ID, artifact kind, bundled MCP mode, command surface, archive format, and validator from the registry.

**Tech Stack:** Python stdlib, existing `qiongli.platform_targets`, existing `unittest` local plugin installer tests.

---

## Files

- Modify: `packages/python-qiongli/src/qiongli/local_plugin_installer.py`
  - Import `load_platform_targets`, `require_platform_target`, and `PlatformTarget`.
  - Add local platform-to-target mapping.
  - Resolve target metadata during each local plugin materialization.
  - Write registry-derived metadata to `.qiongli-managed.json`.
  - Write Codex marketplace metadata with the same target ID.
- Modify: `tests/test_local_plugin_installer.py`
  - Add tests proving metadata is registry-derived and present in installed markers.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark Python local plugin installers as connected to registry target metadata, leaving npm-lite installation as follow-up.

## Task 1: Add Failing Tests

- [x] **Step 1: Add registry-derived marker test**

Add a test to `tests/test_local_plugin_installer.py`:

```python
    def test_install_codex_plugin_records_registry_target_metadata_in_marker(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"

            install_local_plugin(
                LocalPluginOptions(
                    repo_root=REPO_ROOT,
                    target="codex",
                    codex_marketplace_path=marketplace,
                )
            )

            plugin_root = marketplace.parent / "plugins" / "qiongli"
            marker = self._read_json(plugin_root / ".qiongli-managed.json")
            marketplace_entry = self._marketplace_entry(self._read_json(marketplace))

        self.assertEqual(marker["platform_target"]["target_id"], "codex-marketplace-plugin")
        self.assertEqual(marker["platform_target"]["artifact_kind"], "marketplace-plugin")
        self.assertEqual(marker["platform_target"]["bundled_mcp_mode"], "codex-plugin-local-node")
        self.assertEqual(marker["platform_target"]["command_surface"], "slash-commands")
        self.assertEqual(marker["platform_target"]["validator"], "codex-marketplace-plugin")
        self.assertEqual(marketplace_entry["metadata"]["targetId"], "codex-marketplace-plugin")
```

- [x] **Step 2: Add fake registry proof test**

Add:

```python
    def test_local_plugin_marker_uses_loaded_platform_target_metadata(self) -> None:
        fake_target = mock.Mock(
            target_id="fake-codex-target",
            artifact_kind="fake-artifact",
            archive_format="fake-archive",
            bundled_mcp_mode="fake-mcp-mode",
            command_surface="fake-command-surface",
            validator="fake-validator",
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"

            with mock.patch(
                "qiongli.local_plugin_installer.load_platform_targets",
                return_value={"codex-marketplace-plugin": fake_target},
            ):
                install_local_plugin(
                    LocalPluginOptions(
                        repo_root=REPO_ROOT,
                        target="codex",
                        codex_marketplace_path=marketplace,
                    )
                )

            marker = self._read_json(
                marketplace.parent / "plugins" / "qiongli" / ".qiongli-managed.json"
            )

        self.assertEqual(marker["platform_target"]["target_id"], "fake-codex-target")
        self.assertEqual(marker["platform_target"]["validator"], "fake-validator")
```

- [x] **Step 3: Run tests and verify RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_local_plugin_installer.LocalPluginInstallerTests.test_install_codex_plugin_records_registry_target_metadata_in_marker tests.test_local_plugin_installer.LocalPluginInstallerTests.test_local_plugin_marker_uses_loaded_platform_target_metadata -q
```

Expected: FAIL because markers and marketplace entries do not include registry
target metadata.

## Task 2: Implement Registry Metadata

- [x] **Step 1: Import platform target helpers**

In `local_plugin_installer.py`, add:

```python
from qiongli.platform_targets import PlatformTarget, load_platform_targets, require_platform_target
```

- [x] **Step 2: Add platform target mapping and resolver**

Add:

```python
LOCAL_PLUGIN_PLATFORM_TARGETS = {
    "codex": "codex-marketplace-plugin",
    "claude": "claude-code-marketplace-plugin",
    "antigravity": "antigravity-local-plugin",
}


def _local_plugin_target(repo_root: Path, platform: str) -> PlatformTarget:
    target_id = LOCAL_PLUGIN_PLATFORM_TARGETS.get(platform)
    if target_id is None:
        raise ValueError(f"unsupported local plugin platform: {platform}")
    return require_platform_target(load_platform_targets(repo_root), target_id)
```

- [x] **Step 3: Pass target metadata into materialization and markers**

Resolve the platform target in `install_local_plugin(...)` before each
platform materialization:

```python
    codex_target = _local_plugin_target(repo_root, "codex")
```

Pass the resolved target into `_materialize_plugin_root(...)`, then into
`_managed_marker(platform=..., version=..., platform_target=platform_target)`.

Update `_managed_marker`:

```python
def _managed_marker(*, platform: str, version: str, platform_target: PlatformTarget) -> dict[str, Any]:
    return {
        ...
        "platform_target": _platform_target_marker(platform_target),
        ...
    }
```

Add:

```python
def _platform_target_marker(target: PlatformTarget) -> dict[str, str]:
    return {
        "target_id": target.target_id,
        "artifact_kind": target.artifact_kind,
        "archive_format": target.archive_format,
        "bundled_mcp_mode": target.bundled_mcp_mode,
        "command_surface": target.command_surface,
        "validator": target.validator,
    }
```

- [x] **Step 4: Add Codex marketplace target metadata**

In `_write_codex_marketplace_entry`, accept the already resolved Codex target
and include:

```python
"targetId": target.target_id,
"artifactKind": target.artifact_kind,
"validator": target.validator,
```

inside `metadata`.

- [x] **Step 5: Run focused tests and verify GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_local_plugin_installer -q
```

Expected: PASS.

## Task 3: Verify And Document

- [x] **Step 1: Run wider installer tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_local_plugin_installer tests.test_universal_installer tests.test_release_local_install_check -q
```

Expected: PASS.

- [x] **Step 2: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [x] **Step 3: Update Stage 12 roadmap**

Update Stage 12 status to say Python local plugin installers now record
registry-derived target metadata in managed markers and Codex marketplace
entries, leaving npm-lite installation as the remaining packaging metadata
follow-up.

- [x] **Step 4: Commit by content**

Implementation:

```bash
git add packages/python-qiongli/src/qiongli/local_plugin_installer.py tests/test_local_plugin_installer.py
git commit -m "feat(installer): record platform target metadata"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-local-plugin-installer-target-metadata.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record local installer target metadata"
```

## Self-Review

- Spec coverage: Covers local plugin installers only. npm-lite installation
  remains an explicit Stage 12 follow-up.
- Placeholder scan: No placeholders remain.
- Type consistency: Tests and implementation use `platform_target` marker fields
  and Codex marketplace `targetId` consistently.
