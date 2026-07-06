# Release Recommended Target IDs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release download `recommended.*.target_id` values derive from platform target registry `release_download.recommended_key` metadata instead of hard-coded target IDs.

**Architecture:** `build_index()` already loads `content/distribution/platform-targets.yaml`. Add a small lookup layer that maps each required recommended install key (`qiongli_cli`, `codex`, `claude_code`, `claude_desktop_skill`, `claude_desktop_plugin`) to exactly one target whose `release_download.recommended_key` matches that key, then use that lookup when building the recommended install block.

**Tech Stack:** Python standard library, existing `qiongli.platform_targets`, existing `unittest` release download tests.

---

## Files

- Modify: `tests/test_release_downloads.py`
  - Add a fixture registry with fake target IDs but stable `recommended_key` values.
  - Assert `build_index()` emits those fake target IDs in `recommended`.
- Modify: `tooling/scripts/generate_release_downloads.py`
  - Add registry lookup helpers for recommended target IDs.
  - Use lookup results in `build_index()` recommended install entries.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that recommended install target IDs are now keyed by registry metadata.

### Task 1: Add RED Recommended-Key Test

- [x] **Step 1: Add fake platform registry helper**

Add helper methods to `ReleaseDownloadsTests`:

```python
    def _write_valid_companion_registry(self, root: Path) -> None:
        registry = root / "content" / "distribution" / "release-companion-targets.yaml"
        registry.parent.mkdir(parents=True, exist_ok=True)
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
                        "artifact_manifest": {
                            "target_id": "release-artifact-manifest",
                            "subject": "not-applicable",
                            "artifact_kind": "release-metadata",
                            "expected_install_method": "download_json",
                        },
                    },
                },
                indent=2,
            ),
            encoding="utf-8",
        )

    def _platform_target_fixture(self, *, recommended_key: str, kind: str = "plugin") -> dict[str, object]:
        adapters = {
            "plugin": {"kind": "plugin", "plugin_manifest_platform": "codex", "materializer": "plugin_artifacts"},
            "claude-plugin": {"kind": "plugin", "plugin_manifest_platform": "claude", "materializer": "plugin_artifacts"},
            "skill-zip": {"kind": "skill-zip", "plugin_manifest_platform": "none", "materializer": "desktop_skill_artifacts"},
            "package": {"kind": "package", "plugin_manifest_platform": "none", "materializer": "npm_package"},
        }
        return {
            "display_name": f"Fixture {recommended_key}",
            "artifact_kind": kind,
            "archive_format": "zip" if kind == "skill-zip" else "tar.gz",
            "adapter": adapters[kind],
            "smoke": {
                "structural_archive_check": "marketplace_validation",
                "client_activation_check": "not_applicable",
            },
            "source_inputs": ["content/workflow/**"],
            "required_paths": ["plugin.json"],
            "allowed_wrapper_dirs": [],
            "forbidden_paths": [".codex-plugin/"],
            "bundled_mcp_mode": "none",
            "command_surface": "fixture-cli",
            "validator": f"fixture-{recommended_key}",
            "release_download": {
                "guide_label": f"Fixture {recommended_key}",
                "recommended_key": recommended_key,
                "asset_groups": [],
            },
        }
```

- [x] **Step 2: Add failing behavior test**

Add:

```python
    def test_recommended_target_ids_follow_registry_recommended_keys(self) -> None:
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

        self.assertEqual(index["recommended"]["qiongli_cli"]["target_id"], "fixture-npm-target")
        self.assertEqual(index["recommended"]["codex"]["target_id"], "fixture-codex-target")
        self.assertEqual(index["recommended"]["claude_code"]["target_id"], "fixture-claude-target")
        self.assertEqual(
            index["recommended"]["claude_desktop_skill"]["target_id"],
            "fixture-desktop-skill-target",
        )
        self.assertEqual(
            index["recommended"]["claude_desktop_plugin"]["target_id"],
            "fixture-desktop-plugin-target",
        )
```

- [x] **Step 3: Run RED**

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_recommended_target_ids_follow_registry_recommended_keys -q
```

Expected: FAIL because current `build_index()` uses hard-coded target IDs.

### Task 2: Implement Recommended Target Lookup

- [x] **Step 1: Add required key constant and helper**

Add to `tooling/scripts/generate_release_downloads.py`:

```python
REQUIRED_RECOMMENDED_TARGET_KEYS = (
    "qiongli_cli",
    "codex",
    "claude_code",
    "claude_desktop_skill",
    "claude_desktop_plugin",
)


def _recommended_target_ids(targets: dict[str, PlatformTarget]) -> dict[str, str]:
    resolved: dict[str, str] = {}
    for key in REQUIRED_RECOMMENDED_TARGET_KEYS:
        matches = sorted(
            target_id
            for target_id, target in targets.items()
            if target.release_download.get("recommended_key") == key
        )
        if len(matches) != 1:
            raise ValueError(
                "platform target registry must define exactly one "
                f"release_download.recommended_key={key!r}; found {len(matches)}"
            )
        resolved[key] = matches[0]
    return resolved
```

- [x] **Step 2: Use lookup in `build_index()`**

After loading targets:

```python
recommended_target_ids = _recommended_target_ids(targets)
```

Use `recommended_target_ids["..."]` for each recommended `target_id`.

- [x] **Step 3: Run GREEN**

```bash
.venv/bin/python -m unittest tests.test_release_downloads.ReleaseDownloadsTests.test_recommended_target_ids_follow_registry_recommended_keys -q
```

Expected: OK.

### Task 3: Update Roadmap And Verify

- [x] **Step 1: Update Stage 12 wording**

Record that release recommended install target IDs now derive from registry `recommended_key` metadata.

- [x] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_upload_assets -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "feat(release): derive recommended target ids from registry"
git add docs/superpowers/plans/2026-07-06-release-recommended-target-ids.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record recommended target id registry lookup"
```
