# Platform Target Materializer Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move one remaining Stage 12 packaging assumption into the platform target registry by requiring every target adapter to declare its materializer surface.

**Architecture:** Keep the registry as the source of truth. `qiongli.platform_targets` validates the new `adapter.materializer` field, and release download generation keeps emitting the full adapter mapping without a separate code path.

**Tech Stack:** Python standard library, PyYAML, `unittest`, YAML registry metadata.

---

### Task 1: Add RED Coverage For Materializer Metadata

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Write the failing registry contract tests**

```python
def test_platform_target_registry_declares_boundary_rules(self) -> None:
    from qiongli.platform_targets import load_platform_targets

    targets = load_platform_targets(REPO_ROOT)
    self.assertEqual(
        targets["codex-marketplace-plugin"].adapter["materializer"],
        "plugin_artifacts",
    )
    self.assertEqual(
        targets["claude-desktop-skill-zip"].adapter["materializer"],
        "desktop_skill_artifacts",
    )
    self.assertEqual(
        targets["npm-plugin-lite"].adapter["materializer"],
        "npm_package",
    )
```

```python
def test_platform_target_registry_rejects_missing_adapter_materializer(self) -> None:
    from qiongli.platform_targets import validate_platform_target_registry

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        self._write_platform_target_registry(
            root,
            required_paths=["plugin.json"],
            forbidden_paths=[".codex-plugin/"],
            adapter={
                "kind": "fixture",
                "plugin_manifest_platform": "none",
            },
        )

        failures = validate_platform_target_registry(root)

    self.assertTrue(any("adapter.materializer" in failure for failure in failures), failures)
```

```python
def test_platform_target_registry_rejects_unknown_adapter_materializer(self) -> None:
    from qiongli.platform_targets import validate_platform_target_registry

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        self._write_platform_target_registry(
            root,
            required_paths=["plugin.json"],
            forbidden_paths=[".codex-plugin/"],
            adapter={
                "kind": "fixture",
                "plugin_manifest_platform": "none",
                "materializer": "ad_hoc_script",
            },
        )

        failures = validate_platform_target_registry(root)

    self.assertTrue(any("adapter.materializer must be one of" in failure for failure in failures), failures)
```

- [ ] **Step 2: Write the failing release index assertion**

```python
self.assertEqual(
    index["platform_targets"]["codex-marketplace-plugin"]["adapter"]["materializer"],
    "plugin_artifacts",
)
```

- [ ] **Step 3: Run tests to verify RED**

Run:

```bash
.venv/bin/python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_declares_boundary_rules \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_missing_adapter_materializer \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_unknown_adapter_materializer \
  tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides \
  -q
```

Expected: FAIL because `adapter.materializer` is absent from the current registry and validator.

### Task 2: Implement The Registry Schema Field

**Files:**
- Modify: `content/distribution/platform-targets.yaml`
- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`

- [ ] **Step 1: Add materializer values to registry adapters**

```yaml
adapter:
  kind: plugin
  plugin_manifest_platform: codex
  materializer: plugin_artifacts
```

Target values:

```text
codex-marketplace-plugin -> plugin_artifacts
claude-code-marketplace-plugin -> plugin_artifacts
claude-desktop-direct-plugin -> plugin_artifacts
claude-desktop-skill-zip -> desktop_skill_artifacts
antigravity-local-plugin -> local_plugin_installer
npm-plugin-lite -> npm_package
pypi-full-runtime -> python_package
```

- [ ] **Step 2: Validate allowed materializers**

```python
ADAPTER_MATERIALIZERS = frozenset(
    {
        "plugin_artifacts",
        "desktop_skill_artifacts",
        "local_plugin_installer",
        "npm_package",
        "python_package",
    }
)
```

```python
for required_string in ("kind", "plugin_manifest_platform", "materializer"):
    if not isinstance(value.get(required_string), str) or not value[required_string]:
        raise ValueError(
            f"{registry_path} target {target_id}.{field}.{required_string} must be a non-empty string"
        )
materializer = strings["materializer"]
if materializer not in ADAPTER_MATERIALIZERS:
    allowed = ", ".join(sorted(ADAPTER_MATERIALIZERS))
    raise ValueError(
        f"{registry_path} target {target_id}.{field}.materializer must be one of: {allowed}"
    )
```

- [ ] **Step 3: Run tests to verify GREEN**

Run the same focused unittest command from Task 1. Expected: OK.

### Task 3: Update Roadmap Evidence

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record materializer metadata in Stage 12 status**

Add that platform target adapters now declare the materializer surface for plugin artifacts, Desktop skill artifacts, local plugin installers, npm packages, and Python packages.

- [ ] **Step 2: Narrow the adapter backlog**

Replace the generic adapter backlog item with a future-only item for fields beyond current manifest-platform and materializer metadata.

### Task 4: Verify And Commit By Category

**Files:**
- Feature commit: `content/distribution/platform-targets.yaml`, `packages/python-qiongli/src/qiongli/platform_targets.py`, `tests/test_plugin_distribution_contract.py`, `tests/test_release_downloads.py`
- Docs commit: `docs/superpowers/plans/2026-07-06-platform-target-materializer-metadata.md`, `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Run focused and regression checks**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract tests.test_release_downloads -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 2: Review repository boundary**

Check that no marketplace catalog files, local absolute paths, secrets, generated archives, or derived plugin payloads were added.

- [ ] **Step 3: Commit feature files**

```bash
git add content/distribution/platform-targets.yaml \
  packages/python-qiongli/src/qiongli/platform_targets.py \
  tests/test_plugin_distribution_contract.py \
  tests/test_release_downloads.py
git commit -m "feat(distribution): declare platform target materializers"
```

- [ ] **Step 4: Commit docs files**

```bash
git add docs/superpowers/plans/2026-07-06-platform-target-materializer-metadata.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record platform materializer metadata"
```
