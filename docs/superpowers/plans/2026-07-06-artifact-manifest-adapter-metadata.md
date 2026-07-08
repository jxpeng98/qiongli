# Artifact Manifest Adapter Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Carry platform target adapter metadata into each registry-backed release artifact manifest record.

**Architecture:** `build_index()` already emits platform target adapter metadata. `build_artifact_manifest()` should copy that adapter mapping into each registry-backed artifact record so per-asset policy records expose the same materializer source as the target index.

**Tech Stack:** Python standard library, `unittest`, JSON release metadata generation.

---

### Task 1: Add RED Manifest Coverage

**Files:**
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Assert registry-backed artifact adapter metadata**

```python
self.assertEqual(
    codex_record["adapter"]["materializer"],
    "plugin_artifacts",
)
self.assertEqual(
    desktop_record["adapter"]["materializer"],
    "plugin_artifacts",
)
```

- [ ] **Step 2: Run RED**

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides -q
```

Expected: FAIL because artifact manifest records do not include `adapter`.

### Task 2: Emit Adapter Metadata

**Files:**
- Modify: `tooling/scripts/generate_release_downloads.py`

- [ ] **Step 1: Copy adapter metadata into registry-backed records**

```python
"adapter": dict(target.get("adapter") or {}),
```

- [ ] **Step 2: Keep companion records explicit**

```python
"adapter": {},
```

- [ ] **Step 3: Run GREEN**

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides -q
```

Expected: OK.

### Task 3: Update Roadmap And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record manifest adapter metadata**

Update Stage 12 status text so artifact manifests are described as carrying adapter/materializer metadata alongside smoke and path policy.

- [ ] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_release_downloads -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "feat(release): include adapter metadata in artifact manifests"
git add docs/superpowers/plans/2026-07-06-artifact-manifest-adapter-metadata.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record artifact manifest adapter metadata"
```
