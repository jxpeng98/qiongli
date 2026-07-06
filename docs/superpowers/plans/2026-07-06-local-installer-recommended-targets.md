# Local Installer Recommended Targets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Python local plugin installation select platform target records by registry `release_download.recommended_key` metadata instead of fixed target IDs.

**Architecture:** Keep target selection inside `qiongli.local_plugin_installer` because marker and marketplace metadata are written there. Reuse the existing `PlatformTarget.release_download` schema rather than adding a second mapping layer. Preserve current CLI target names (`codex`, `claude`, `antigravity`) and only change how those names resolve to registry records.

**Tech Stack:** Python `unittest`, Qiongli platform target registry, local plugin installer.

---

### Task 1: Add Recommended-Key Regression Test

**Files:**
- Modify: `tests/test_local_plugin_installer.py`

- [ ] **Step 1: Write the failing test**

Add this test to `LocalPluginInstallerTests` after `test_local_plugin_marker_uses_loaded_platform_target_metadata`:

```python
    def test_local_plugin_target_uses_registry_recommended_key(self) -> None:
        fake_target = mock.Mock(
            target_id="fixture-codex-target",
            artifact_kind="fake-artifact",
            archive_format="fake-archive",
            bundled_mcp_mode="fake-mcp-mode",
            command_surface="fake-command-surface",
            validator="fake-validator",
            release_download={"recommended_key": "codex"},
        )
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            marketplace = root / "agents" / "marketplace.json"

            with mock.patch(
                "qiongli.local_plugin_installer.load_platform_targets",
                return_value={"fixture-codex-target": fake_target},
            ):
                install_local_plugin(
                    LocalPluginOptions(
                        repo_root=REPO_ROOT,
                        target="codex",
                        codex_marketplace_path=marketplace,
                    )
                )

            marker = self._read_json(
                marketplace.parent / "plugins" / "qiongli" / ".qiongli-managed.json"
            )

        self.assertEqual(marker["platform_target"]["target_id"], "fixture-codex-target")
        self.assertEqual(marker["platform_target"]["validator"], "fake-validator")
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
.venv/bin/python -m unittest tests.test_local_plugin_installer.LocalPluginInstallerTests.test_local_plugin_target_uses_registry_recommended_key -q
```

Expected: FAIL with `platform target registry missing target: codex-marketplace-plugin`, proving the installer still uses the old hard-coded target ID.

### Task 2: Resolve Local Installer Targets By Recommended Key

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/local_plugin_installer.py`

- [ ] **Step 1: Write minimal implementation**

Replace the old `LOCAL_PLUGIN_PLATFORM_TARGETS` constant and `_local_plugin_target` lookup with:

```python
LOCAL_PLUGIN_PLATFORM_RECOMMENDED_KEYS = {
    "codex": "codex",
    "claude": "claude_code",
    "antigravity": "antigravity",
}
```

```python
def _local_plugin_target(repo_root: Path, platform: str) -> PlatformTarget:
    recommended_key = LOCAL_PLUGIN_PLATFORM_RECOMMENDED_KEYS.get(platform)
    if recommended_key is None:
        raise ValueError(f"unsupported local plugin platform: {platform}")
    return _platform_target_by_recommended_key(load_platform_targets(repo_root), recommended_key)


def _platform_target_by_recommended_key(
    targets: dict[str, PlatformTarget],
    recommended_key: str,
) -> PlatformTarget:
    matches = sorted(
        (
            target
            for target in targets.values()
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

Remove the now-unused `require_platform_target` import.

- [ ] **Step 2: Run test to verify it passes**

Run:

```bash
.venv/bin/python -m unittest tests.test_local_plugin_installer.LocalPluginInstallerTests.test_local_plugin_target_uses_registry_recommended_key -q
```

Expected: PASS.

### Task 3: Update Roadmap Status

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record the completed slice**

Update the Stage 12 status and remaining product gap text to state that Python local plugin installer target selection now follows registry `release_download.recommended_key` metadata. Keep the remaining opt-in local-agent smoke and future adapter-extension backlog unchanged.

### Task 4: Verify And Commit

**Files:**
- Test: `tests/test_local_plugin_installer.py`
- Test: `tests/test_release_local_install_check.py`
- Test: `tests/test_universal_installer.py`
- Test: `tooling/scripts/validate_platform_targets.py`

- [ ] **Step 1: Run focused installer regression**

```bash
.venv/bin/python -m unittest tests.test_local_plugin_installer tests.test_universal_installer tests.test_release_local_install_check -q
```

Expected: all tests pass.

- [ ] **Step 2: Run registry validator**

```bash
.venv/bin/python tooling/scripts/validate_platform_targets.py
```

Expected: no validation errors.

- [ ] **Step 3: Run diff hygiene check**

```bash
git diff --check
```

Expected: no whitespace errors.

- [ ] **Step 4: Run boundary scan**

```bash
rg -n "(/[U]sers/|/[p]rivate/|BEGI[N] (RSA|OPENSSH|EC|DSA) PRIVATE KEY|secre[t]:|toke[n]:|passwor[d]:)" docs/superpowers/plans/2026-07-06-local-installer-recommended-targets.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md packages/python-qiongli/src/qiongli/local_plugin_installer.py tests/test_local_plugin_installer.py
```

Expected: no matches.

- [ ] **Step 5: Commit implementation**

```bash
git add packages/python-qiongli/src/qiongli/local_plugin_installer.py tests/test_local_plugin_installer.py docs/superpowers/plans/2026-07-06-local-installer-recommended-targets.md
git commit -m "feat(installer): select local plugin targets by recommended key"
```

- [ ] **Step 6: Commit roadmap update**

```bash
git add docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record local installer target lookup"
```
