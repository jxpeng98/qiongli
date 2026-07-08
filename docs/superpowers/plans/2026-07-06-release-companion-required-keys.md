# Release Companion Required Keys Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the release companion target registry fail preflight when a current required release metadata asset key is missing.

**Architecture:** Keep `content/distribution/release-companion-targets.yaml` as the source of truth for non-platform release assets. `load_companion_targets()` should validate both per-target fields and the required current asset key set before release download generation or preflight can pass.

**Tech Stack:** Python standard library, PyYAML, `unittest`.

---

### Task 1: Add RED Required-Key Test

**Files:**
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Add missing-key fixture**

```python
def test_release_companion_target_registry_rejects_missing_required_key(self) -> None:
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
                        "claude_desktop_literature_mcpb": {
                            "target_id": "claude-desktop-literature-mcpb",
                            "subject": "literature",
                            "artifact_kind": "mcpb",
                            "expected_install_method": "download_mcpb",
                        },
                        "zotero_desktop_companion": {
                            "target_id": "zotero-desktop-companion-xpi",
                            "subject": "zotero",
                            "artifact_kind": "xpi",
                            "expected_install_method": "download_xpi",
                        },
                        "download_guide": {
                            "target_id": "release-download-guide",
                            "subject": "not-applicable",
                            "artifact_kind": "release-metadata",
                            "expected_install_method": "download_markdown",
                        },
                        "download_index": {
                            "target_id": "release-download-index",
                            "subject": "not-applicable",
                            "artifact_kind": "release-metadata",
                            "expected_install_method": "download_json",
                        },
                    },
                }
            ),
            encoding="utf-8",
        )

        with self.assertRaisesRegex(ValueError, "missing required companion target keys: artifact_manifest"):
            module.load_companion_targets(root)
```

- [ ] **Step 2: Run RED**

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_release_companion_target_registry_rejects_missing_required_key -q
```

Expected: FAIL because the loader currently accepts a registry that omits `artifact_manifest`.

### Task 2: Implement Required-Key Validation

**Files:**
- Modify: `tooling/scripts/generate_release_downloads.py`

- [ ] **Step 1: Add required key set**

```python
REQUIRED_COMPANION_TARGET_KEYS = frozenset(
    {
        "claude_desktop_literature_mcpb",
        "zotero_desktop_companion",
        "download_guide",
        "download_index",
        "artifact_manifest",
    }
)
```

- [ ] **Step 2: Validate after parsing targets**

```python
missing = sorted(REQUIRED_COMPANION_TARGET_KEYS.difference(targets))
if missing:
    raise ValueError(f"{registry_path} missing required companion target keys: {', '.join(missing)}")
```

- [ ] **Step 3: Run GREEN**

Run the focused unittest command from Task 1. Expected: OK.

### Task 3: Update Roadmap And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record required-key validation**

Update Stage 12 status/backlog wording so companion registry coverage includes required current asset keys.

- [ ] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_automation -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "feat(release): require companion target registry keys"
git add docs/superpowers/plans/2026-07-06-release-companion-required-keys.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record companion required key validation"
```
