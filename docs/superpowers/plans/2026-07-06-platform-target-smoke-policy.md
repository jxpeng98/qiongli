# Platform Target Smoke Policy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the Stage 12 platform-specific smoke split from roadmap prose into the platform target registry so structural archive checks and client CLI activation checks cannot drift.

**Architecture:** Add a required `smoke` mapping to each platform target. Registry validation should require explicit `structural_archive_check` and `client_activation_check` values, release download metadata should expose the policy, and artifact manifests should carry the policy for every registry-backed artifact.

**Tech Stack:** Python 3, unittest, YAML platform registry, release download/index JSON generation.

---

## File Map

- Modify: `content/distribution/platform-targets.yaml`
  - Add `smoke` metadata to every platform target.
- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`
  - Add `smoke` to `PlatformTarget`.
  - Validate allowed smoke policy values.
- Modify: `tooling/scripts/generate_release_downloads.py`
  - Include `smoke` in platform target index records.
  - Include `smoke` in registry-backed artifact manifest records.
- Modify: `tests/test_plugin_distribution_contract.py`
  - Assert every target declares the smoke split.
  - Assert schema rejects missing smoke metadata.
- Modify: `tests/test_release_downloads.py`
  - Assert generated index and artifact manifest carry smoke metadata.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record this Stage 12 backlog item as implemented.

## Task 1: Add Failing Smoke Policy Tests

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Assert registry smoke policies**

Add assertions that:

```python
self.assertEqual(
    targets["codex-marketplace-plugin"].smoke["structural_archive_check"],
    "marketplace_validation",
)
self.assertEqual(
    targets["codex-marketplace-plugin"].smoke["client_activation_check"],
    "local_install_acceptance",
)
self.assertEqual(
    targets["claude-desktop-skill-zip"].smoke["client_activation_check"],
    "not_applicable",
)
```

- [ ] **Step 2: Assert schema rejection for missing smoke metadata**

Create a fixture target without `smoke` and expect:

```text
target fixture-target.smoke must be an object
```

- [ ] **Step 3: Assert generated outputs include smoke metadata**

In release download tests, assert:

```python
self.assertEqual(
    index["platform_targets"]["codex-marketplace-plugin"]["smoke"]["client_activation_check"],
    "local_install_acceptance",
)
self.assertEqual(
    codex_record["smoke"]["structural_archive_check"],
    "marketplace_validation",
)
```

- [ ] **Step 4: Run RED**

Run:

```bash
.venv/bin/python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_declares_boundary_rules \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_missing_smoke_metadata \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_unknown_smoke_policy \
  tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides \
  -q
```

Expected before implementation: `PlatformTarget` has no `smoke` attribute or generated metadata lacks the `smoke` key.

## Task 2: Implement Registry Smoke Policy

**Files:**
- Modify: `content/distribution/platform-targets.yaml`
- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`
- Modify: `tooling/scripts/generate_release_downloads.py`

- [ ] **Step 1: Add YAML metadata**

For plugin and package targets that require real client checks, use:

```yaml
smoke:
  structural_archive_check: marketplace_validation
  client_activation_check: local_install_acceptance
```

For skill-only or package-only targets without client activation, use:

```yaml
smoke:
  structural_archive_check: marketplace_validation
  client_activation_check: not_applicable
```

- [ ] **Step 2: Validate schema**

Allow exactly:

```python
STRUCTURAL_ARCHIVE_CHECKS = {"marketplace_validation", "package_build_validation"}
CLIENT_ACTIVATION_CHECKS = {"local_install_acceptance", "not_applicable"}
```

- [ ] **Step 3: Emit generated metadata**

Add `smoke` to `_target_index()` and to each registry-backed artifact manifest record.

- [ ] **Step 4: Run GREEN**

Run the same focused unittest command. Expected: `OK`.

## Task 3: Update Roadmap And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Update Stage 12 status/backlog**

Record that platform-specific smoke policy is now registry-backed and no longer only prose in the optimization backlog.

- [ ] **Step 2: Run regression checks**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract tests.test_release_downloads -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

Feature commit:

```text
feat(distribution): add platform target smoke policy
```

Docs commit:

```text
docs(roadmap): record platform smoke policy hardening
```
