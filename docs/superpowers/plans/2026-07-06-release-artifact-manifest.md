# Release Artifact Manifest Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Emit a machine-readable release artifact manifest that maps release assets to platform target metadata, subject, archive format, install method, and forbidden-path policy.

**Architecture:** Extend the existing registry-backed release download generator instead of adding a second source of truth. `generate_release_downloads.py` already knows assets, target metadata, recommended install entries, and asset grouping; the new manifest will flatten those structures into per-asset records and write `qiongli-artifacts-<tag>.json` next to the human guide and download index. `release_upload_assets.py` will include the new manifest so postflight uploads it with the rest of the release metadata.

**Tech Stack:** Python stdlib, existing `qiongli.platform_targets`, existing release download generator, existing `unittest` release tests.

---

## Files

- Modify: `tooling/scripts/generate_release_downloads.py`
  - Add `artifact_manifest` to release asset metadata.
  - Serialize a third output file `qiongli-artifacts-<tag>.json`.
  - Add per-asset manifest records with `asset`, `asset_key`, `target_id`, `subject`, `archive_format`, `expected_install_method`, `artifact_kind`, `validator`, `required_paths`, and `forbidden_paths`.
- Modify: `tooling/scripts/release_upload_assets.py`
  - Include `artifact_manifest` in extra release metadata uploads.
- Modify: `tooling/scripts/generate_release_notes.sh`
  - Mention `qiongli-artifacts-<tag>.json` in prerelease release notes.
- Modify: `tooling/scripts/generate_stable_release_notes.py`
  - Link `qiongli-artifacts-<tag>.json` in stable release notes.
- Modify: `tests/test_release_downloads.py`
  - Assert the new manifest file is generated, release notes mention it, and
    manifest records contain registry-backed asset metadata.
- Modify: `tests/test_release_upload_assets.py`
  - Assert the new artifact manifest asset is included in stable and prerelease upload lists.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that Stage 12 now emits the machine-readable artifact manifest.

## Task 1: Add Failing Manifest Tests

- [x] **Step 1: Extend release download generation test**

In `tests/test_release_downloads.py`, update
`test_generates_human_and_machine_download_guides` to expect:

```python
manifest_path = out_dir / "qiongli-artifacts-v1.1.0-beta.2.json"
self.assertIn(str(manifest_path), result.stdout)
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
self.assertEqual(index["assets"]["artifact_manifest"], "qiongli-artifacts-v1.1.0-beta.2.json")
self.assertEqual(manifest["tag"], "v1.1.0-beta.2")
self.assertEqual(manifest["schema_version"], "1.0")
codex_record = next(
    item
    for item in manifest["artifacts"]
    if item["asset"] == "qiongli-next-codex-plugin-v1.1.0-beta.2.tar.gz"
)
self.assertEqual(codex_record["target_id"], "codex-marketplace-plugin")
self.assertEqual(codex_record["archive_format"], "tar.gz")
self.assertEqual(codex_record["expected_install_method"], "marketplace")
self.assertIn(".claude-plugin/", codex_record["forbidden_paths"])
desktop_record = next(
    item
    for item in manifest["artifacts"]
    if item["asset"] == "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip"
)
self.assertEqual(desktop_record["target_id"], "claude-desktop-direct-plugin")
self.assertEqual(desktop_record["subject"], "core")
```

- [x] **Step 2: Extend release upload asset tests**

In `tests/test_release_upload_assets.py`, assert stable and prerelease names
include `qiongli-artifacts-<tag>.json`.

- [x] **Step 3: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides tests.test_release_upload_assets.ReleaseUploadAssetsTests.test_stable_upload_assets_are_derived_from_registry_targets tests.test_release_upload_assets.ReleaseUploadAssetsTests.test_prerelease_upload_assets_are_next_channel_only -q
```

Expected: FAIL because `artifact_manifest` and `qiongli-artifacts-*.json` do not exist yet.

## Task 2: Implement Artifact Manifest

- [x] **Step 1: Add manifest asset key**

Add `artifact_manifest: f"qiongli-artifacts-{tag}.json"` to stable and
prerelease `_release_assets(...)` results.

- [x] **Step 2: Add target policy fields to target index**

Extend `_target_index(...)` so each target includes `required_paths` and
`forbidden_paths`.

- [x] **Step 3: Add artifact manifest builder**

Add `build_artifact_manifest(index)` to flatten `assets_by_target` plus
companion metadata assets into records. Target-backed records should use target
registry metadata. Companion records should use `target_id: "release-companion"`
and empty path policies.

- [x] **Step 4: Write third output**

Update `write_outputs(...)` and `main(...)` so the generator writes and prints
`qiongli-artifacts-<tag>.json`.

- [x] **Step 5: Upload the manifest**

Add `"artifact_manifest"` to `EXTRA_UPLOAD_ASSET_KEYS` in
`tooling/scripts/release_upload_assets.py`.

- [x] **Step 6: Expose the manifest from release notes**

Update prerelease and stable release notes so users can find
`qiongli-artifacts-<tag>.json` from the release body, not only from uploaded
asset names.

- [x] **Step 7: Run GREEN**

Run the focused tests from Task 1. Expected: PASS.

## Task 3: Verify, Document, Commit

- [x] **Step 1: Run related release tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_upload_assets -q
```

- [x] **Step 2: Run generator smoke**

Run:

```bash
.venv/bin/python scripts/generate_release_downloads.py --tag v1.1.0-beta.2 --out-dir /private/tmp/qiongli-artifact-manifest-smoke
```

Expected: exits 0 and writes markdown, download index JSON, and artifact
manifest JSON.

- [x] **Step 3: Update roadmap**

Update Stage 12 status/backlog wording so the machine-readable artifact
manifest backlog item is no longer listed as missing.

- [x] **Step 4: Commit by content**

Implementation and tests:

```bash
git add tooling/scripts/generate_release_downloads.py tooling/scripts/release_upload_assets.py tooling/scripts/generate_release_notes.sh tooling/scripts/generate_stable_release_notes.py tests/test_release_downloads.py tests/test_release_upload_assets.py
git commit -m "feat(release): emit artifact manifest"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-release-artifact-manifest.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record release artifact manifest"
```

## Self-Review

- Spec coverage: Covers Stage 12 machine-readable artifact manifest backlog using the existing registry-backed release download source.
- Placeholder scan: No open placeholders remain.
- Type consistency: Manifest fields are derived from current `PlatformTarget`, `recommended`, `assets`, and `assets_by_target` structures.
