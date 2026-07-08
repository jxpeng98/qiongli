# Registry-Backed Release Downloads Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Generate release download guide metadata from the platform target registry so public download docs cannot drift from artifact boundary targets.

**Architecture:** Extend the platform target registry with optional `release_download` metadata and expose it through `qiongli.platform_targets.PlatformTarget`. `tooling/scripts/generate_release_downloads.py` will load the registry, annotate recommended install surfaces with target IDs, group release assets by target, and render a registry-derived target table in the human guide.

**Tech Stack:** Python standard library, PyYAML-backed registry loader, `unittest`, existing release download scripts.

---

### Task 1: Add Failing Release Download Tests

**Files:**
- Modify: `tests/test_release_downloads.py`

- [x] **Step 1: Write the failing test expectations**

Add these assertions to `ReleaseDownloadsTests.test_generates_human_and_machine_download_guides` after the existing guide/index assertions:

```python
self.assertIn("## Platform target registry", guide)
self.assertIn("codex-marketplace-plugin", guide)
self.assertIn("claude-desktop-direct-plugin", guide)
self.assertIn("npm-plugin-lite", guide)
self.assertIn("pypi-full-runtime", guide)

self.assertEqual(index["recommended"]["codex"]["target_id"], "codex-marketplace-plugin")
self.assertEqual(index["recommended"]["claude_code"]["target_id"], "claude-code-marketplace-plugin")
self.assertEqual(index["recommended"]["claude_desktop_plugin"]["target_id"], "claude-desktop-direct-plugin")
self.assertEqual(index["recommended"]["claude_desktop_skill"]["target_id"], "claude-desktop-skill-zip")
self.assertEqual(index["recommended"]["qiongli_cli"]["target_id"], "npm-plugin-lite")

self.assertEqual(
    index["platform_targets"]["claude-desktop-direct-plugin"]["display_name"],
    "Claude Desktop Direct Plugin",
)
self.assertEqual(
    index["platform_targets"]["claude-desktop-direct-plugin"]["archive_format"],
    "zip",
)
self.assertEqual(
    index["assets_by_target"]["claude-desktop-direct-plugin"]["claude_desktop_plugin"],
    "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
)
self.assertIn(
    "qiongli-next-codex-plugin-v1.1.0-beta.2.tar.gz",
    index["assets_by_target"]["codex-marketplace-plugin"]["maintainer_plugin_tarballs"],
)
```

- [x] **Step 2: Run the targeted test to verify it fails**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides -q
```

Expected: FAIL because the generated index has no `target_id`, `platform_targets`, or `assets_by_target` fields yet.

### Task 2: Expose Release Download Metadata In The Registry Loader

**Files:**
- Modify: `content/distribution/platform-targets.yaml`
- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`

- [x] **Step 1: Add registry metadata**

Add `release_download` mappings to each target:

```yaml
release_download:
  guide_label: Codex
  recommended_key: codex
  asset_groups:
    - maintainer_plugin_tarballs
```

Use the matching labels and recommended keys for Claude Code, Claude Desktop direct plugin, Claude Desktop/Web skill ZIP, Antigravity, npm plugin-lite, and PyPI full runtime.

- [x] **Step 2: Extend the dataclass**

Add the optional field:

```python
release_download: dict[str, Any]
```

Populate it in `_parse_target` with:

```python
release_download=_optional_mapping(registry_path, target_id, raw_target, "release_download"),
```

Add `_optional_mapping`:

```python
def _optional_mapping(registry_path: Path, target_id: str, raw_target: dict[str, Any], field: str) -> dict[str, Any]:
    value = raw_target.get(field, {})
    if value is None:
        return {}
    if not isinstance(value, dict):
        raise ValueError(f"{registry_path} target {target_id}.{field} must be an object")
    return dict(value)
```

- [x] **Step 3: Run the targeted test**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides -q
```

Expected: still FAIL until the generator consumes the new metadata.

### Task 3: Generate Download Index Fields From Platform Targets

**Files:**
- Modify: `tooling/scripts/generate_release_downloads.py`

- [x] **Step 1: Import the registry loader**

Add:

```python
from qiongli.platform_targets import PlatformTarget, load_platform_targets
```

- [x] **Step 2: Add target serialization helpers**

Add helpers that produce `platform_targets` and `assets_by_target`:

```python
def _target_index(targets: dict[str, PlatformTarget]) -> dict[str, dict[str, Any]]:
    return {
        target_id: {
            "display_name": target.display_name,
            "artifact_kind": target.artifact_kind,
            "archive_format": target.archive_format,
            "bundled_mcp_mode": target.bundled_mcp_mode,
            "command_surface": target.command_surface,
            "validator": target.validator,
            "release_download": dict(target.release_download),
        }
        for target_id, target in targets.items()
    }


def _assets_by_target(
    assets: dict[str, list[str] | str],
    targets: dict[str, PlatformTarget],
) -> dict[str, dict[str, list[str] | str]]:
    grouped: dict[str, dict[str, list[str] | str]] = {}
    for target_id, target in targets.items():
        groups = target.release_download.get("asset_groups", [])
        if not isinstance(groups, list):
            continue
        target_assets: dict[str, list[str] | str] = {}
        for group in groups:
            if not isinstance(group, str):
                continue
            value = assets.get(group)
            if value:
                target_assets[group] = value
        if target_assets:
            grouped[target_id] = target_assets
    return grouped
```

- [x] **Step 3: Annotate recommended install entries**

Load targets in `build_index`, add `target_id` to the existing recommended entries, then add `platform_targets` and `assets_by_target` to the returned index.

- [x] **Step 4: Render a registry-derived table**

Add a `## Platform target registry` section in `render_markdown` with target ID, display name, archive format, and validator rows from `index["platform_targets"]`.

- [x] **Step 5: Run the targeted test**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides -q
```

Expected: PASS.

### Task 4: Verify Release Download Integration

**Files:**
- Test: `tests/test_release_downloads.py`
- Test: `tests/test_plugin_distribution_contract.py`
- Test: `tests/test_release_automation.py`

- [x] **Step 1: Run release download tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads -q
```

Expected: PASS.

- [x] **Step 2: Run related registry and automation tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_declares_boundary_rules tests.test_release_automation.ReleaseAutomationTests.test_release_postflight_generates_download_guide_assets -q
```

Expected: PASS.

- [x] **Step 3: Run generated output smoke**

Run:

```bash
python3 scripts/generate_release_downloads.py --tag v1.1.0-beta.2 --out-dir /private/tmp/qiongli-downloads-smoke
```

Expected: exits 0 and writes both markdown and JSON assets.

### Task 5: Update Roadmap And Commit

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-06-registry-backed-release-downloads.md`

- [x] **Step 1: Mark this plan complete**

Change each checkbox in this plan to `[x]` after verification passes.

- [x] **Step 2: Update Stage 12 status**

Change the Stage 12 status to say release download docs and machine-readable download index now include registry-derived target IDs, target metadata, and asset grouping. Leave installer/postflight metadata integration as the remaining follow-up.

- [x] **Step 3: Run diff checks**

Run:

```bash
git diff --check
git status --short
```

Expected: no whitespace errors; only intended files changed.

- [x] **Step 4: Commit implementation**

Run:

```bash
git add content/distribution/platform-targets.yaml packages/python-qiongli/src/qiongli/platform_targets.py tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "feat(distribution): generate downloads from target registry" -m "Annotate release download indexes with platform target metadata and group assets by registry target."
```

- [x] **Step 5: Commit roadmap docs**

Run:

```bash
git add docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md docs/superpowers/plans/2026-07-06-registry-backed-release-downloads.md
git commit -m "docs(roadmap): record registry-backed release downloads" -m "Track the Stage 12 release download follow-up and remaining installer/postflight metadata work."
```
