# Release Guide Target Labels Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release guide and release notes target labels use `recommended.*.target_id` from the generated index instead of fixed platform target IDs.

**Architecture:** `build_index()` now derives recommended target IDs from platform registry `release_download.recommended_key`. `render_markdown()` and `render_release_notes_download_summary()` should use those recommended target IDs when calling `_target_label()`, so public docs follow registry target ID changes without another hard-coded lookup table.

**Tech Stack:** Python standard library, existing release download generator, `unittest`.

---

## Files

- Modify: `tests/test_release_downloads.py`
  - Add a fake registry test that renders markdown and release notes summary from registry-derived recommended target IDs.
- Modify: `tooling/scripts/generate_release_downloads.py`
  - Add a helper for recommended target labels.
  - Replace fixed target ID label lookups in markdown and release notes rendering.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that release guide labels now follow recommended target IDs from the generated index.

### Task 1: Add RED Guide Label Test

- [x] **Step 1: Add failing test**

Add to `ReleaseDownloadsTests`:

```python
    def test_release_guides_label_recommended_targets_from_index(self) -> None:
        module = _load_release_download_module()

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            registry = root / "content" / "distribution" / "platform-targets.yaml"
            registry.parent.mkdir(parents=True, exist_ok=True)
            registry.write_text(
                json.dumps(
                    {
                        "schema_version": "1.0",
                        "targets": {
                            "fixture-npm-target": self._platform_target_fixture(
                                recommended_key="qiongli_cli",
                                kind="package",
                            ),
                            "fixture-codex-target": self._platform_target_fixture(recommended_key="codex"),
                            "fixture-claude-target": self._platform_target_fixture(
                                recommended_key="claude_code",
                                kind="claude-plugin",
                            ),
                            "fixture-desktop-skill-target": self._platform_target_fixture(
                                recommended_key="claude_desktop_skill",
                                kind="skill-zip",
                            ),
                            "fixture-desktop-plugin-target": self._platform_target_fixture(
                                recommended_key="claude_desktop_plugin",
                                kind="claude-plugin",
                            ),
                        },
                    },
                    indent=2,
                ),
                encoding="utf-8",
            )
            self._write_valid_companion_registry(root)
            index = module.build_index("v1.1.0-beta.2", root=root)

        guide = module.render_markdown(index)
        notes = module.render_release_notes_download_summary(index)

        self.assertIn("Fixture qiongli_cli (`fixture-npm-target`)", guide)
        self.assertIn("Fixture codex (`fixture-codex-target`)", guide)
        self.assertIn("Fixture claude_code (`fixture-claude-target`)", guide)
        self.assertIn("Fixture claude_desktop_plugin (`fixture-desktop-plugin-target`)", guide)
        self.assertIn("Fixture claude_desktop_skill (`fixture-desktop-skill-target`)", guide)
        self.assertIn("Fixture qiongli_cli (`fixture-npm-target`)", notes)
        self.assertIn("Fixture codex (`fixture-codex-target`)", notes)
```

- [x] **Step 2: Run RED**

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_release_guides_label_recommended_targets_from_index -q
```

Expected: FAIL because renderers still call `_target_label()` with fixed target IDs.

### Task 2: Implement Recommended Label Helper

- [x] **Step 1: Add helper**

Add to `tooling/scripts/generate_release_downloads.py`:

```python
def _recommended_target_label(
    platform_targets: dict[str, Any],
    recommended: dict[str, Any],
    key: str,
    fallback: str,
) -> str:
    entry = recommended.get(key, {})
    if not isinstance(entry, dict):
        return fallback
    target_id = entry.get("target_id")
    if not isinstance(target_id, str) or not target_id:
        return fallback
    return _target_label(platform_targets, target_id, fallback)
```

- [x] **Step 2: Replace markdown label calls**

In `render_markdown()`, replace the five fixed `_target_label(platform_targets, "...", ...)` calls for `qiongli_cli`, `codex`, `claude_code`, `claude_desktop_plugin`, and `claude_desktop_skill` with `_recommended_target_label(...)`.

- [x] **Step 3: Replace release note label calls**

Apply the same replacement in `render_release_notes_download_summary()`.

- [x] **Step 4: Run GREEN**

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_release_guides_label_recommended_targets_from_index -q
```

Expected: OK.

### Task 3: Update Roadmap And Verify

- [x] **Step 1: Update roadmap**

Record that release guide and release notes summary labels use recommended target IDs from the index.

- [x] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_upload_assets -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "feat(release): label guides from recommended targets"
git add docs/superpowers/plans/2026-07-06-release-guide-target-labels.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record release guide target labels"
```
