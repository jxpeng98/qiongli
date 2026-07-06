# Release Upload Assets By Target Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the remaining hand-maintained release upload companion asset list by deriving all upload assets from the release download index target mapping.

**Architecture:** Keep `generate_release_downloads.py` as the single release download model. Add companion release targets to `assets_by_target` alongside platform targets, then simplify `release_upload_assets.py` so it reads only `assets_by_target`.

**Tech Stack:** Python stdlib, existing release download index builder, existing `unittest` release tests.

---

## Files

- Modify: `tests/test_release_downloads.py`
  - Assert companion release assets are present under specialized target IDs in `assets_by_target`.
- Modify: `tests/test_release_upload_assets.py`
  - Assert upload asset names still include MCPB, Zotero, download guide, download index, and artifact manifest without an `EXTRA_UPLOAD_ASSET_KEYS` fallback.
- Modify: `tooling/scripts/generate_release_downloads.py`
  - Add companion assets to the index `assets_by_target` mapping.
  - Preserve artifact manifest metadata by resolving companion target metadata when a target ID is not a platform target.
- Modify: `tooling/scripts/release_upload_assets.py`
  - Remove the duplicate `EXTRA_UPLOAD_ASSET_KEYS` list.
  - Build upload names only from `index["assets_by_target"]`.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that release upload assets now consume companion targets from the same target-index mapping.

## Task 1: Add Failing Target-Derivation Tests

- [x] **Step 1: Assert companion targets in `assets_by_target`**

In `tests/test_release_downloads.py`, extend
`test_generates_human_and_machine_download_guides`:

```python
self.assertEqual(
    index["assets_by_target"]["claude-desktop-literature-mcpb"]["claude_desktop_literature_mcpb"],
    "qiongli-literature-provider-0.1.5.mcpb",
)
self.assertEqual(
    index["assets_by_target"]["zotero-desktop-companion-xpi"]["zotero_desktop_companion"],
    "qiongli-zotero-companion-0.2.2.xpi",
)
self.assertEqual(
    index["assets_by_target"]["release-download-guide"]["download_guide"],
    "qiongli-downloads-v1.1.0-beta.2.md",
)
self.assertEqual(
    index["assets_by_target"]["release-download-index"]["download_index"],
    "qiongli-downloads-v1.1.0-beta.2.json",
)
self.assertEqual(
    index["assets_by_target"]["release-artifact-manifest"]["artifact_manifest"],
    "qiongli-artifacts-v1.1.0-beta.2.json",
)
```

- [x] **Step 2: Assert upload helper has no fallback list**

In `tests/test_release_upload_assets.py`, add to the stable upload test:

```python
self.assertFalse(hasattr(module, "EXTRA_UPLOAD_ASSET_KEYS"))
self.assertIn("qiongli-literature-provider-0.1.5.mcpb", names)
self.assertIn("qiongli-zotero-companion-0.2.2.xpi", names)
self.assertIn("qiongli-downloads-v1.6.0.md", names)
self.assertIn("qiongli-downloads-v1.6.0.json", names)
self.assertIn("qiongli-artifacts-v1.6.0.json", names)
```

- [x] **Step 3: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides tests.test_release_upload_assets.ReleaseUploadAssetsTests.test_stable_upload_assets_are_derived_from_registry_targets -q
```

Expected: FAIL because companion target IDs are not yet in `assets_by_target`
and `release_upload_assets.py` still defines `EXTRA_UPLOAD_ASSET_KEYS`.

## Task 2: Derive Upload Assets From Target Mapping

- [x] **Step 1: Add companion assets to `assets_by_target`**

In `tooling/scripts/generate_release_downloads.py`, add a helper that maps each
`COMPANION_TARGETS` entry to its specialized `target_id`:

```python
def _companion_assets_by_target(assets: dict[str, list[str] | str]) -> dict[str, dict[str, list[str] | str]]:
    grouped: dict[str, dict[str, list[str] | str]] = {}
    for asset_key, target in COMPANION_TARGETS.items():
        if not isinstance(target, dict):
            continue
        target_id = str(target.get("target_id") or asset_key)
        value = assets.get(asset_key)
        if _asset_names(value):
            grouped[target_id] = {asset_key: value}
    return grouped
```

Then have `build_index()` set `assets_by_target` to the merge of platform and
companion mappings.

- [x] **Step 2: Preserve companion manifest metadata**

In `build_artifact_manifest()`, when `target_id` is not a platform target,
look up companion metadata by target ID before computing artifact records.

- [x] **Step 3: Simplify upload asset collection**

In `tooling/scripts/release_upload_assets.py`, delete
`EXTRA_UPLOAD_ASSET_KEYS` and the secondary `assets` loop. Keep only the
`assets_by_target` traversal.

- [x] **Step 4: Run GREEN**

Run the focused tests from Task 1. Expected: PASS.

## Task 3: Verify, Document, Commit

- [x] **Step 1: Run related tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_upload_assets tests.test_release_automation -q
```

- [x] **Step 2: Run release download smoke**

Run:

```bash
.venv/bin/python scripts/generate_release_downloads.py --tag v1.1.0-beta.2 --out-dir /private/tmp/qiongli-upload-assets-by-target-smoke
```

- [x] **Step 3: Run upload list smoke**

Run:

```bash
.venv/bin/python scripts/release_upload_assets.py --tag v1.1.0-beta.2 --dist-dir /private/tmp/qiongli-upload-assets-by-target-smoke --no-require-existing
```

- [x] **Step 4: Update roadmap**

Update Stage 12 status/backlog to record that upload assets now consume the
same target-index mapping for platform and companion assets.

- [x] **Step 5: Commit by content**

Implementation and tests:

```bash
git add tooling/scripts/generate_release_downloads.py tooling/scripts/release_upload_assets.py tests/test_release_downloads.py tests/test_release_upload_assets.py
git commit -m "feat(release): derive upload companions by target"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-release-upload-assets-by-target.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record upload target derivation"
```

## Self-Review

- Spec coverage: Covers the Stage 12 backlog item about moving remaining repeated release-asset lists into the registry-backed target index.
- Placeholder scan: No placeholders remain.
- Type consistency: Companion target IDs match the existing `COMPANION_TARGETS` mapping.
