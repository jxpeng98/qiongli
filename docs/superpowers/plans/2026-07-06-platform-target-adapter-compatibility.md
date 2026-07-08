# Platform Target Adapter Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Validate that platform target adapter fields form coherent combinations, not only individually valid enum values.

**Architecture:** Keep `adapter.kind`, `adapter.plugin_manifest_platform`, and `adapter.materializer` in the platform target registry. `qiongli.platform_targets` enforces compatibility rules after parsing the individual enum fields so release tooling cannot consume contradictory target metadata.

**Tech Stack:** Python standard library, PyYAML, `unittest`.

---

### Task 1: Add RED Compatibility Tests

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`

- [ ] **Step 1: Reject manifest platform mismatch**

```python
def test_platform_target_registry_rejects_adapter_manifest_platform_mismatch(self) -> None:
    from qiongli.platform_targets import validate_platform_target_registry

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        self._write_platform_target_registry(
            root,
            required_paths=["package.json"],
            forbidden_paths=[".codex-plugin/"],
            adapter={
                "kind": "package",
                "plugin_manifest_platform": "codex",
                "materializer": "npm_package",
            },
        )

        failures = validate_platform_target_registry(root)

    self.assertTrue(any("adapter.plugin_manifest_platform=codex is not valid" in failure for failure in failures), failures)
```

- [ ] **Step 2: Reject materializer mismatch**

```python
def test_platform_target_registry_rejects_adapter_materializer_mismatch(self) -> None:
    from qiongli.platform_targets import validate_platform_target_registry

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        self._write_platform_target_registry(
            root,
            required_paths=["SKILL.md"],
            forbidden_paths=[".codex-plugin/"],
            adapter={
                "kind": "skill-zip",
                "plugin_manifest_platform": "none",
                "materializer": "plugin_artifacts",
            },
        )

        failures = validate_platform_target_registry(root)

    self.assertTrue(any("adapter.materializer=plugin_artifacts is not valid" in failure for failure in failures), failures)
```

- [ ] **Step 3: Run RED**

```bash
.venv/bin/python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_adapter_manifest_platform_mismatch \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_adapter_materializer_mismatch \
  -q
```

Expected: FAIL because validator currently accepts individually valid but incompatible adapter combinations.

### Task 2: Implement Compatibility Matrix

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`

- [ ] **Step 1: Add compatibility maps**

```python
ADAPTER_KIND_MANIFEST_PLATFORMS = {
    "plugin": frozenset({"codex", "claude"}),
    "skill-zip": frozenset({"none"}),
    "local-plugin": frozenset({"none"}),
    "package": frozenset({"none"}),
}
ADAPTER_KIND_MATERIALIZERS = {
    "plugin": frozenset({"plugin_artifacts"}),
    "skill-zip": frozenset({"desktop_skill_artifacts"}),
    "local-plugin": frozenset({"local_plugin_installer"}),
    "package": frozenset({"npm_package", "python_package"}),
}
```

- [ ] **Step 2: Validate combinations after enum checks**

```python
allowed_manifest_platforms = ADAPTER_KIND_MANIFEST_PLATFORMS[kind]
if manifest_platform not in allowed_manifest_platforms:
    allowed = ", ".join(sorted(allowed_manifest_platforms))
    raise ValueError(
        f"{registry_path} target {target_id}.{field}.plugin_manifest_platform={manifest_platform} "
        f"is not valid for adapter.kind={kind}; expected one of: {allowed}"
    )
allowed_materializers = ADAPTER_KIND_MATERIALIZERS[kind]
if materializer not in allowed_materializers:
    allowed = ", ".join(sorted(allowed_materializers))
    raise ValueError(
        f"{registry_path} target {target_id}.{field}.materializer={materializer} "
        f"is not valid for adapter.kind={kind}; expected one of: {allowed}"
    )
```

- [ ] **Step 3: Run GREEN**

Run the focused unittest command from Task 1. Expected: OK.

### Task 3: Update Roadmap And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record compatibility validation**

Update Stage 12 status and backlog wording so adapter schema coverage includes enum compatibility across kind, manifest platform, and materializer.

- [ ] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract tests.test_release_downloads -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add packages/python-qiongli/src/qiongli/platform_targets.py tests/test_plugin_distribution_contract.py
git commit -m "feat(distribution): validate platform target adapter compatibility"
git add docs/superpowers/plans/2026-07-06-platform-target-adapter-compatibility.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record adapter compatibility validation"
```
