# Release Companion Artifact Targets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the catch-all `release-companion` artifact manifest target with specialized target IDs for release metadata, MCPB, and Zotero companion assets.

**Architecture:** Keep platform install surfaces in `content/distribution/platform-targets.yaml`; companion release assets are not install platforms, so model them as release-download metadata inside `generate_release_downloads.py`. `build_index()` will expose a `companion_targets` mapping, and `build_artifact_manifest()` will use that mapping to flatten companion records with stable target IDs and install methods.

**Tech Stack:** Python stdlib, existing release download generator, existing `unittest` release tests.

---

## Files

- Modify: `tooling/scripts/generate_release_downloads.py`
  - Add a `COMPANION_TARGETS` mapping for:
    - `claude_desktop_literature_mcpb` -> `claude-desktop-literature-mcpb`
    - `zotero_desktop_companion` -> `zotero-desktop-companion-xpi`
    - `download_guide` -> `release-download-guide`
    - `download_index` -> `release-download-index`
    - `artifact_manifest` -> `release-artifact-manifest`
  - Add `companion_targets` to the machine-readable download index.
  - Make `build_artifact_manifest()` use specialized companion target metadata.
- Modify: `tests/test_release_downloads.py`
  - Assert companion manifest records use specialized target IDs.
  - Assert no manifest record uses the old catch-all `release-companion`.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that the artifact manifest now distinguishes companion target IDs.

## Task 1: Add Failing Manifest Tests

- [x] **Step 1: Extend release download manifest test**

In `tests/test_release_downloads.py`, update
`test_generates_human_and_machine_download_guides` to assert:

```python
mcpb_record = next(
    item
    for item in manifest["artifacts"]
    if item["asset"] == "qiongli-literature-provider-0.1.5.mcpb"
)
self.assertEqual(mcpb_record["target_id"], "claude-desktop-literature-mcpb")
self.assertEqual(mcpb_record["expected_install_method"], "download_mcpb")
self.assertEqual(mcpb_record["artifact_kind"], "mcpb")
self.assertFalse(mcpb_record["registry_target"])

zotero_record = next(
    item
    for item in manifest["artifacts"]
    if item["asset"] == "qiongli-zotero-companion-0.2.2.xpi"
)
self.assertEqual(zotero_record["target_id"], "zotero-desktop-companion-xpi")
self.assertEqual(zotero_record["expected_install_method"], "download_xpi")

manifest_record = next(
    item
    for item in manifest["artifacts"]
    if item["asset"] == "qiongli-artifacts-v1.1.0-beta.2.json"
)
self.assertEqual(manifest_record["target_id"], "release-artifact-manifest")
self.assertNotIn("release-companion", {item["target_id"] for item in manifest["artifacts"]})
self.assertEqual(
    index["companion_targets"]["artifact_manifest"]["target_id"],
    "release-artifact-manifest",
)
```

- [x] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides -q
```

Expected: FAIL because companion records still use `release-companion`, and the
index does not expose `companion_targets`.

## Task 2: Implement Companion Target Mapping

- [x] **Step 1: Add `COMPANION_TARGETS`**

Add a module-level mapping in `tooling/scripts/generate_release_downloads.py`
with `target_id`, `subject`, `artifact_kind`, and
`expected_install_method` for each companion asset key.

- [x] **Step 2: Include companion targets in the index**

Add:

```python
"companion_targets": {
    key: dict(value)
    for key, value in COMPANION_TARGETS.items()
},
```

to `build_index(...)`.

- [x] **Step 3: Use companion targets in manifest records**

Replace the `companion_install_methods` map in `build_artifact_manifest(...)`
with `companion_targets`. Each companion record should use:

- `target_id` from the mapping.
- `subject` from the mapping.
- `artifact_kind` from the mapping.
- `expected_install_method` from the mapping.
- `registry_target: False`.
- empty path policies.

- [x] **Step 4: Run GREEN**

Run the focused test from Task 1. Expected: PASS.

## Task 3: Verify, Document, Commit

- [x] **Step 1: Run related release tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_upload_assets -q
```

- [x] **Step 2: Run generator smoke**

Run:

```bash
.venv/bin/python scripts/generate_release_downloads.py --tag v1.1.0-beta.2 --out-dir /private/tmp/qiongli-companion-targets-smoke
```

- [x] **Step 3: Update roadmap**

Update Stage 12 status/backlog to record specialized companion artifact target
IDs in the release artifact manifest.

- [x] **Step 4: Commit by content**

Implementation and tests:

```bash
git add tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "feat(release): specialize companion artifact targets"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-release-companion-artifact-targets.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record companion artifact targets"
```

## Self-Review

- Spec coverage: Covers the Stage 12 backlog item about companion asset target
  IDs beyond catch-all `release-companion` records.
- Placeholder scan: No placeholders remain.
- Type consistency: Companion target metadata uses the same target-facing field
  names already present in artifact manifest records.
