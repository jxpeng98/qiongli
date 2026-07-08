# Platform Target Adapter Enum Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Strengthen the platform target registry validator so adapter kind and manifest-platform values are explicit enums instead of arbitrary strings.

**Architecture:** Keep adapter metadata in `content/distribution/platform-targets.yaml`. `qiongli.platform_targets` validates `adapter.kind`, `adapter.plugin_manifest_platform`, and `adapter.materializer` before downstream scripts consume registry records.

**Tech Stack:** Python standard library, PyYAML, `unittest`.

---

### Task 1: Add RED Schema Tests

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`

- [ ] **Step 1: Assert real adapter kind values**

```python
self.assertEqual(targets["codex-marketplace-plugin"].adapter["kind"], "plugin")
self.assertEqual(targets["claude-desktop-skill-zip"].adapter["kind"], "skill-zip")
self.assertEqual(targets["antigravity-local-plugin"].adapter["kind"], "local-plugin")
self.assertEqual(targets["pypi-full-runtime"].adapter["kind"], "package")
```

- [ ] **Step 2: Reject unknown adapter kind**

```python
def test_platform_target_registry_rejects_unknown_adapter_kind(self) -> None:
    from qiongli.platform_targets import validate_platform_target_registry

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        self._write_platform_target_registry(
            root,
            required_paths=["plugin.json"],
            forbidden_paths=[".codex-plugin/"],
            adapter={
                "kind": "handcrafted-plugin",
                "plugin_manifest_platform": "none",
                "materializer": "plugin_artifacts",
            },
        )

        failures = validate_platform_target_registry(root)

    self.assertTrue(any("adapter.kind must be one of" in failure for failure in failures), failures)
```

- [ ] **Step 3: Reject unknown manifest platform**

```python
def test_platform_target_registry_rejects_unknown_manifest_platform(self) -> None:
    from qiongli.platform_targets import validate_platform_target_registry

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        self._write_platform_target_registry(
            root,
            required_paths=["plugin.json"],
            forbidden_paths=[".codex-plugin/"],
            adapter={
                "kind": "plugin",
                "plugin_manifest_platform": "gemini",
                "materializer": "plugin_artifacts",
            },
        )

        failures = validate_platform_target_registry(root)

    self.assertTrue(
        any("adapter.plugin_manifest_platform must be one of" in failure for failure in failures),
        failures,
    )
```

- [ ] **Step 4: Run RED**

```bash
.venv/bin/python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_declares_boundary_rules \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_unknown_adapter_kind \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_unknown_manifest_platform \
  -q
```

Expected: FAIL because validator currently accepts unknown adapter kind and manifest-platform strings.

### Task 2: Implement Enum Validation

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`

- [ ] **Step 1: Add allowed enum sets**

```python
ADAPTER_KINDS = frozenset({"plugin", "skill-zip", "local-plugin", "package"})
ADAPTER_MANIFEST_PLATFORMS = frozenset({"codex", "claude", "none"})
```

- [ ] **Step 2: Validate parsed adapter fields**

```python
kind = strings["kind"]
if kind not in ADAPTER_KINDS:
    allowed = ", ".join(sorted(ADAPTER_KINDS))
    raise ValueError(f"{registry_path} target {target_id}.{field}.kind must be one of: {allowed}")
manifest_platform = strings["plugin_manifest_platform"]
if manifest_platform not in ADAPTER_MANIFEST_PLATFORMS:
    allowed = ", ".join(sorted(ADAPTER_MANIFEST_PLATFORMS))
    raise ValueError(
        f"{registry_path} target {target_id}.{field}.plugin_manifest_platform must be one of: {allowed}"
    )
```

- [ ] **Step 3: Run GREEN**

Run the same focused unittest command from Task 1. Expected: OK.

### Task 3: Update Roadmap And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record adapter enum validation**

Update Stage 12 status and backlog wording so adapter schema coverage includes kind, manifest-platform, and materializer enums.

- [ ] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add packages/python-qiongli/src/qiongli/platform_targets.py tests/test_plugin_distribution_contract.py
git commit -m "feat(distribution): validate platform target adapter enums"
git add docs/superpowers/plans/2026-07-06-platform-target-adapter-enums.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record adapter enum validation"
```
