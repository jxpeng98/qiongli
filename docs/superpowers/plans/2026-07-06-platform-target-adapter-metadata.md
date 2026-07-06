# Platform Target Adapter Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Move plugin manifest platform selection into the platform target registry so validators no longer hard-code target IDs for Codex or Claude manifest behavior.

**Architecture:** Extend each `PlatformTarget` with an `adapter` mapping containing `kind` and `plugin_manifest_platform`. Registry validation will require both strings. Marketplace install validation will call a small helper on the target metadata instead of branching on `target_id`, while npm payload sync/audit will preserve the adapter metadata in generated registry JSON.

**Tech Stack:** Python stdlib, PyYAML-backed `qiongli.platform_targets`, existing release and distribution unittest coverage.

---

## Files

- Modify: `content/distribution/platform-targets.yaml`
  - Add `adapter.kind` and `adapter.plugin_manifest_platform` to every target.
- Modify: `packages/python-qiongli/src/qiongli/platform_targets.py`
  - Add `adapter` to `PlatformTarget`.
  - Validate `adapter.kind` and `adapter.plugin_manifest_platform`.
  - Add `plugin_manifest_platform(target)` helper.
- Modify: `tooling/scripts/validate_marketplace_install.py`
  - Replace target-ID manifest platform branching with the registry helper.
- Modify: `tooling/scripts/sync_npm_package_payload.py`
  - Include `adapter` in `platform-targets.json`.
- Modify: `tooling/scripts/audit_distribution_payloads.py`
  - Include `adapter` in the expected npm payload registry.
- Modify: `tests/test_plugin_distribution_contract.py`
  - Assert registry adapter metadata is present.
  - Assert missing adapter metadata fails validation.
  - Assert marketplace validation reads a fake target's adapter instead of requiring known target IDs.
- Modify: `tooling/scripts/generate_release_downloads.py`
  - Include `adapter` in the machine-readable platform target index.
- Modify: `tests/test_release_downloads.py`
  - Assert release download index target metadata includes adapter metadata.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that Stage 12 now has a small target adapter metadata interface.

## Task 1: Write Failing Adapter Tests

- [x] **Step 1: Extend registry declaration test**

In `tests/test_plugin_distribution_contract.py`, update
`test_platform_target_registry_declares_boundary_rules` to assert:

```python
self.assertEqual(
    targets["codex-marketplace-plugin"].adapter["plugin_manifest_platform"],
    "codex",
)
self.assertEqual(
    targets["claude-desktop-direct-plugin"].adapter["plugin_manifest_platform"],
    "claude",
)
self.assertEqual(
    targets["claude-desktop-skill-zip"].adapter["plugin_manifest_platform"],
    "none",
)
```

- [x] **Step 2: Add adapter schema failure test**

Add `test_platform_target_registry_rejects_missing_adapter_metadata` that writes
a fixture registry without `adapter` and expects a validation failure containing
`adapter`.

- [x] **Step 3: Add fake-target adapter behavior test**

Add `test_marketplace_validator_uses_target_adapter_for_manifest_platform` that
loads the real Codex target, replaces `target_id` with
`fixture-codex-like-plugin`, and verifies `validator._platform_for_target(...)`
returns `codex`.

- [x] **Step 4: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_declares_boundary_rules tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_platform_target_registry_rejects_missing_adapter_metadata tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_uses_target_adapter_for_manifest_platform -q
```

Expected: FAIL because `PlatformTarget.adapter` and adapter schema validation do
not exist yet, and `_platform_for_target` still rejects unknown target IDs.

## Task 2: Implement Registry Adapter Metadata

- [x] **Step 1: Add registry adapter fields**

Add an `adapter` mapping to every target in
`content/distribution/platform-targets.yaml`:

- Codex marketplace: `kind: plugin`, `plugin_manifest_platform: codex`.
- Claude Code marketplace: `kind: plugin`, `plugin_manifest_platform: claude`.
- Claude Desktop direct plugin: `kind: plugin`, `plugin_manifest_platform: claude`.
- Claude Desktop/Web skill ZIP: `kind: skill-zip`, `plugin_manifest_platform: none`.
- Antigravity local plugin: `kind: local-plugin`, `plugin_manifest_platform: none`.
- npm plugin-lite: `kind: package`, `plugin_manifest_platform: none`.
- PyPI full runtime: `kind: package`, `plugin_manifest_platform: none`.

- [x] **Step 2: Parse and validate adapter metadata**

Update `PlatformTarget` and `_parse_target(...)` to include an adapter dict.
Reject missing or malformed `adapter.kind` and
`adapter.plugin_manifest_platform`.

- [x] **Step 3: Add helper**

Add:

```python
def plugin_manifest_platform(target: PlatformTarget) -> str:
    value = target.adapter.get("plugin_manifest_platform")
    if not isinstance(value, str) or not value:
        raise ValueError(f"platform target {target.target_id} adapter.plugin_manifest_platform must be a non-empty string")
    return value
```

- [x] **Step 4: Use helper in marketplace validation**

Update `_platform_for_target(...)` in
`tooling/scripts/validate_marketplace_install.py` to return the helper value
when it is `codex` or `claude`, and raise a clear error otherwise.

- [x] **Step 5: Preserve adapter in generated target metadata**

Add `"adapter": dict(target.adapter)` to the payload builders in
`tooling/scripts/sync_npm_package_payload.py` and
`tooling/scripts/audit_distribution_payloads.py`, and to
`tooling/scripts/generate_release_downloads.py` target-index records.

- [x] **Step 6: Run GREEN**

Run the focused command from Task 1. Expected: PASS.

## Task 3: Verify, Document, Commit

- [x] **Step 1: Run related tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract tests.test_release_downloads tests.test_release_upload_assets -q
node --test packages/npm-qiongli/test/installer.test.mjs packages/npm-qiongli/test/cli.test.mjs
```

- [x] **Step 2: Run registry validator**

Run:

```bash
.venv/bin/python scripts/validate_platform_targets.py --root .
```

- [x] **Step 3: Update roadmap**

Update Stage 12 status/backlog to record the small adapter metadata interface
and narrow the remaining adapter-interface backlog.

- [x] **Step 4: Commit by content**

Implementation and tests:

```bash
git add content/distribution/platform-targets.yaml packages/python-qiongli/src/qiongli/platform_targets.py tooling/scripts/validate_marketplace_install.py tooling/scripts/sync_npm_package_payload.py tooling/scripts/audit_distribution_payloads.py tooling/scripts/generate_release_downloads.py tests/test_plugin_distribution_contract.py tests/test_release_downloads.py
git commit -m "feat(distribution): add platform target adapter metadata"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-platform-target-adapter-metadata.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record platform target adapter metadata"
```

## Self-Review

- Spec coverage: Covers the Stage 12 backlog item for a small `PlatformTarget`
  adapter interface without attempting to add new platforms.
- Placeholder scan: No placeholders remain.
- Type consistency: `adapter` is parsed as `dict[str, str]`, exported as JSON,
  and consumed through `plugin_manifest_platform(target)`.
