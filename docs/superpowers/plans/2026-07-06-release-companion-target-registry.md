# Release Companion Target Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move release companion target metadata out of `generate_release_downloads.py` and into a versioned content registry.

**Architecture:** Add `content/distribution/release-companion-targets.yaml` as the source of truth for non-platform release assets such as MCPB, Zotero XPI, download guide, download index, and artifact manifest. `tooling/scripts/generate_release_downloads.py` loads and validates that registry before generating download indexes and artifact manifests.

**Tech Stack:** Python standard library, PyYAML, `unittest`, JSON release metadata generation.

---

### Task 1: Add RED Registry Tests

**Files:**
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Add a module loader helper**

```python
def _load_release_download_module():
    spec = importlib.util.spec_from_file_location(
        "generate_release_downloads",
        REPO_ROOT / "tooling" / "scripts" / "generate_release_downloads.py",
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
```

- [ ] **Step 2: Assert generated indexes expose companion registry metadata**

```python
self.assertEqual(
    index["companion_target_registry"]["path"],
    "content/distribution/release-companion-targets.yaml",
)
self.assertEqual(index["companion_target_registry"]["schema_version"], "1.0")
```

- [ ] **Step 3: Assert malformed companion registry fails**

```python
def test_release_companion_target_registry_rejects_missing_metadata(self) -> None:
    module = _load_release_download_module()

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        registry = root / "content" / "distribution" / "release-companion-targets.yaml"
        registry.parent.mkdir(parents=True)
        registry.write_text(
            json.dumps(
                {
                    "schema_version": "1.0",
                    "targets": {
                        "fixture_asset": {
                            "target_id": "fixture-target",
                            "subject": "fixture",
                            "artifact_kind": "fixture",
                        }
                    },
                }
            ),
            encoding="utf-8",
        )

        with self.assertRaisesRegex(ValueError, "expected_install_method"):
            module.load_companion_targets(root)
```

- [ ] **Step 4: Run RED**

```bash
.venv/bin/python -m unittest \
  tests.test_release_downloads.ReleaseDownloadsTests.test_generates_human_and_machine_download_guides \
  tests.test_release_downloads.ReleaseDownloadsTests.test_release_companion_target_registry_rejects_missing_metadata \
  -q
```

Expected: FAIL because there is no companion target registry metadata in the generated index, and no `load_companion_targets()` function yet.

### Task 2: Implement The Companion Registry

**Files:**
- Create: `content/distribution/release-companion-targets.yaml`
- Modify: `tooling/scripts/generate_release_downloads.py`

- [ ] **Step 1: Add registry YAML**

```yaml
schema_version: "1.0"
targets:
  claude_desktop_literature_mcpb:
    target_id: claude-desktop-literature-mcpb
    subject: literature
    artifact_kind: mcpb
    expected_install_method: download_mcpb
```

Include entries for `zotero_desktop_companion`, `download_guide`, `download_index`, and `artifact_manifest`.

- [ ] **Step 2: Add `load_companion_targets(root)`**

Validate:

```text
schema_version == "1.0"
targets is a non-empty object
each target has non-empty string target_id, subject, artifact_kind, expected_install_method
```

- [ ] **Step 3: Use the loader in generated index**

```python
companion_targets = load_companion_targets(root)
assets_by_target.update(_companion_assets_by_target(assets, companion_targets))
...
"companion_target_registry": {
    "path": COMPANION_TARGET_REGISTRY_RELATIVE_PATH.as_posix(),
    "schema_version": "1.0",
},
"companion_targets": {
    key: dict(value) for key, value in companion_targets.items()
},
```

- [ ] **Step 4: Run GREEN**

Run the focused unittest command from Task 1. Expected: OK.

### Task 3: Update Roadmap And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record companion registry extraction**

Update Stage 12 status so companion metadata assets are described as coming from a release companion target registry rather than a Python constant.

- [ ] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_plugin_distribution_contract -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add content/distribution/release-companion-targets.yaml tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "feat(release): load companion targets from registry"
git add docs/superpowers/plans/2026-07-06-release-companion-target-registry.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record companion target registry"
```
