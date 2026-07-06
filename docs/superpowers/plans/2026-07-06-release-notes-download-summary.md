# Release Notes Download Summary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make beta release notes reuse the registry-backed release download index for their download table instead of duplicating asset names in Bash.

**Architecture:** Add a small renderer to `tooling/scripts/generate_release_downloads.py` that formats the release-notes download summary from `build_index()`. Update `tooling/scripts/generate_release_notes.sh` to call that renderer and print the resulting Markdown section, so target labels, asset names, companion assets, and metadata assets all come from the same source as the release download guide/index.

**Tech Stack:** Python stdlib, existing Bash release notes generator, existing `unittest` release download coverage.

---

## Files

- Modify: `tooling/scripts/generate_release_downloads.py`
  - Add `render_release_notes_download_summary(index)`.
  - Use existing `build_index`, `_target_label`, assets, and recommended commands.
- Modify: `tooling/scripts/generate_release_notes.sh`
  - Remove duplicated Desktop/plugin/MCPB/Zotero asset-name rendering.
  - Invoke the Python renderer and print its Markdown output.
- Modify: `tests/test_release_downloads.py`
  - Assert generated release notes include registry target labels with target IDs.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that beta release notes now consume the registry-backed download summary.

## Task 1: Add Failing Release Notes Test

- [x] **Step 1: Extend release notes test**

In `tests/test_release_downloads.py`,
`test_release_notes_include_download_guide_section`, assert:

```python
self.assertIn("Qiongli npm/npx CLI (`npm-plugin-lite`)", notes)
self.assertIn("Claude Desktop direct plugin (`claude-desktop-direct-plugin`)", notes)
```

- [x] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_release_notes_include_download_guide_section -q
```

Expected: FAIL because the Bash release-notes table does not include registry
target labels or target IDs.

## Task 2: Implement Registry-Backed Summary

- [x] **Step 1: Add Python renderer**

Add `render_release_notes_download_summary(index)` to
`tooling/scripts/generate_release_downloads.py`. It should return the same
Markdown summary currently used by release notes, but derive values from:

- `index["recommended"]`
- `index["assets"]`
- `index["platform_targets"]`
- existing `_target_label(...)`

- [x] **Step 2: Replace Bash asset-name rendering**

In `tooling/scripts/generate_release_notes.sh`, remove the inline MCPB/Zotero
asset version lookup and the hard-coded download table. Instead assign:

```bash
DOWNLOAD_GUIDE_SUMMARY="$(python3 - "$TAG" <<'PY'
import sys

from scripts.generate_release_downloads import build_index, render_release_notes_download_summary

print(render_release_notes_download_summary(build_index(sys.argv[1])))
PY
)"
```

Then print `DOWNLOAD_GUIDE_SUMMARY` under `## Download Guide`.

- [x] **Step 3: Run GREEN**

Run the focused test from Task 1. Expected: PASS.

## Task 3: Verify, Document, Commit

- [x] **Step 1: Run related release tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_upload_assets -q
```

- [x] **Step 2: Run release notes smoke**

Run:

```bash
bash scripts/generate_release_notes.sh --tag v1.1.0-beta.2 --from-tag v1.1.0-beta.1 --output /private/tmp/qiongli-release-notes-download-summary.md --overwrite
```

- [x] **Step 3: Update roadmap**

Update Stage 12 status/backlog to record that beta release notes now consume the
registry-backed download summary.

- [x] **Step 4: Commit by content**

Implementation and tests:

```bash
git add tooling/scripts/generate_release_downloads.py tooling/scripts/generate_release_notes.sh tests/test_release_downloads.py
git commit -m "feat(release): reuse download index in beta notes"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-release-notes-download-summary.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record release notes download summary"
```

## Self-Review

- Spec coverage: Covers the Stage 12 backlog item about moving repeated
  release-asset lists to the target-registry-backed download source.
- Placeholder scan: No placeholders remain.
- Type consistency: The new renderer takes the same `index` shape returned by
  `build_index()`.
