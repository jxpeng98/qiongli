# Builder Recommended Target Forbidden Paths Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the plugin artifact builder apply Claude Desktop direct plugin forbidden-path cleanup by registry `release_download.recommended_key` instead of a fixed platform target ID.

**Architecture:** `build_plugin_artifacts.py` already loads platform target metadata before removing forbidden paths. Add a recommended-key lookup helper and use it when building the Claude Desktop direct plugin artifact, so target ID changes stay isolated to `content/distribution/platform-targets.yaml`.

**Tech Stack:** Python standard library, existing platform target registry helpers, `unittest`.

---

## Files

- Modify: `tests/test_plugin_artifacts.py`
  - Add a helper-level RED test proving forbidden-path cleanup can select a fake target by `recommended_key`.
- Modify: `tooling/scripts/build_plugin_artifacts.py`
  - Add `_platform_target_by_recommended_key()`.
  - Add `_apply_recommended_platform_forbidden_paths()`.
  - Use it for `_build_claude_desktop_plugin()`.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that direct Desktop plugin builder forbidden-path policy now follows registry recommended keys.

### Task 1: Add RED Helper Test

- [x] **Step 1: Add test**

Add to `PluginArtifactsTests`:

```python
    def test_recommended_forbidden_paths_use_registry_recommended_key(self) -> None:
        from dataclasses import replace
        from qiongli.platform_targets import load_platform_targets

        target = replace(
            load_platform_targets(REPO_ROOT)["claude-desktop-direct-plugin"],
            target_id="fixture-desktop-plugin-target",
            forbidden_paths=("remove-me/",),
        )

        with tempfile.TemporaryDirectory() as tmp_dir:
            root = Path(tmp_dir)
            plugin_root = root / "plugin"
            (plugin_root / "remove-me").mkdir(parents=True)
            (plugin_root / "remove-me" / "file.txt").write_text("remove\n", encoding="utf-8")
            (plugin_root / "keep-me").mkdir()

            original_load = module.load_platform_targets
            module.load_platform_targets = lambda _root: {"fixture-desktop-plugin-target": target}
            try:
                module._apply_recommended_platform_forbidden_paths(
                    root,
                    plugin_root,
                    "claude_desktop_plugin",
                )
            finally:
                module.load_platform_targets = original_load

            self.assertFalse((plugin_root / "remove-me").exists())
            self.assertTrue((plugin_root / "keep-me").is_dir())
```

- [x] **Step 2: Run RED**

```bash
.venv/bin/python -m unittest tests.test_plugin_artifacts.PluginArtifactsTests.test_recommended_forbidden_paths_use_registry_recommended_key -q
```

Expected: FAIL because `_apply_recommended_platform_forbidden_paths` does not exist.

### Task 2: Implement Recommended-Key Forbidden Path Lookup

- [x] **Step 1: Add helper**

Add to `tooling/scripts/build_plugin_artifacts.py`:

```python
def _platform_target_by_recommended_key(root: Path, recommended_key: str):
    matches = sorted(
        (
            target
            for target in load_platform_targets(root).values()
            if target.release_download.get("recommended_key") == recommended_key
        ),
        key=lambda target: target.target_id,
    )
    if len(matches) != 1:
        raise ValueError(
            "platform target registry must define exactly one "
            f"release_download.recommended_key={recommended_key!r}; found {len(matches)}"
        )
    return matches[0]
```

- [x] **Step 2: Add apply helper**

```python
def _apply_recommended_platform_forbidden_paths(root: Path, plugin_root: Path, recommended_key: str) -> None:
    target = _platform_target_by_recommended_key(root, recommended_key)
    for pattern in target.forbidden_paths:
        remove_path_pattern(plugin_root, pattern)
```

- [x] **Step 3: Use helper in direct Desktop plugin build**

Replace:

```python
_apply_platform_forbidden_paths(root, plugin_dest, "claude-desktop-direct-plugin")
```

with:

```python
_apply_recommended_platform_forbidden_paths(root, plugin_dest, "claude_desktop_plugin")
```

- [x] **Step 4: Run GREEN**

```bash
.venv/bin/python -m unittest tests.test_plugin_artifacts.PluginArtifactsTests.test_recommended_forbidden_paths_use_registry_recommended_key -q
```

Expected: OK.

### Task 3: Update Roadmap And Verify

- [x] **Step 1: Update roadmap**

Record that direct Desktop plugin artifact builder forbidden-path cleanup now uses registry recommended-key target selection.

- [x] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_plugin_artifacts tests.test_plugin_distribution_contract -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/build_plugin_artifacts.py tests/test_plugin_artifacts.py
git commit -m "feat(distribution): apply builder forbidden paths by recommended key"
git add docs/superpowers/plans/2026-07-06-builder-recommended-target-forbidden-paths.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record builder forbidden path target lookup"
```
