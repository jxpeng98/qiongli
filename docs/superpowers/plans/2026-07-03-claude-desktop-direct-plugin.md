# Claude Desktop Direct Plugin Artifact Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an additive Qiongli release artifact that can be installed through Claude Desktop/direct plugin installers, with a top-level `plugin.json`, without removing or renaming existing marketplace, skill ZIP, MCPB, or Zotero companion assets.

**Architecture:** Keep the existing Claude Code/Codex marketplace artifacts and Desktop/Web skill ZIPs intact. Add a separate direct plugin ZIP built from a materialized plugin directory whose top-level folder is `qiongli/` or `qiongli-next/` and contains `plugin.json`, platform manifests, commands, bundled MCP runtime, and the complete core skill. Release metadata and docs should expose this new direct plugin ZIP as the preferred Desktop direct plugin route while preserving skill ZIPs for manual skill upload.

**Tech Stack:** Python standard library packaging scripts, `unittest`, Bash release scripts, GitHub release assets.

---

## Review Findings From Supplied Draft

### High: prerelease direct plugin function does not exist

**File path:** `tooling/scripts/build_plugin_artifacts.py`

**Problem:** The draft says to call `materialize_next_plugin_package(...)`, but the current code has `materialize_next_codex_plugin(...)` only. That current function intentionally creates a git-backed `qiongli-next` Codex plugin source and the existing test asserts it does not include `.claude-plugin`.

**Why it violates the repository boundary:** Reusing `materialize_next_codex_plugin(...)` for a Claude Desktop direct plugin would blur the current `next-plugin` source contract. Changing it to include `.claude-plugin` would break the current installability test for the Codex-only next payload.

**Concrete fix:** Keep `materialize_next_codex_plugin(...)` Codex-only, but add a separate `materialize_next_plugin_package(...)` for the direct plugin ZIP. Both functions can write a root `plugin.json`; only the new direct materializer should include both `.codex-plugin/plugin.json` and `.claude-plugin/plugin.json`.

### Medium: release-facing download surfaces are incomplete

**File paths:** `tooling/scripts/generate_release_downloads.py`, `tooling/scripts/update_stable_download_sections.py`, `tooling/scripts/generate_stable_release_notes.py`, `tooling/scripts/generate_release_notes.sh`

**Problem:** The draft updates the download index and postflight upload list, but it does not update README/docs stable download sections or generated release notes that currently hard-code Desktop skill ZIP language.

**Why it violates the release boundary:** Publishing a new asset without updating the public download guide, stable README sections, and release notes leaves users on the old skill-ZIP-only path even though the direct plugin artifact exists.

**Concrete fix:** Add `claude_desktop_plugin` to the release index and propagate it to markdown guide rendering, stable README/docs section rendering, stable release notes, and prerelease release notes.

### Medium: validation should cover both marketplace compatibility and direct plugin shape

**File path:** `tooling/scripts/validate_marketplace_install.py`

**Problem:** The draft validates the new direct artifact, but it does not explicitly require root `plugin.json` in existing marketplace plugin payloads after adding the helper to `_build_marketplace_plugin(...)`.

**Why it violates the artifact contract:** If marketplace artifacts start carrying root `plugin.json`, the validator should assert that the additive field is correct and does not accidentally vary by subject, platform, or channel.

**Concrete fix:** Add `_assert_root_plugin_manifest(...)`, call it in `_validate_artifact(...)`, and add `_validate_direct_desktop_plugin_artifact(...)` for the new direct ZIP.

### Low: `skillsplace` follow-up belongs in a separate repo pass

**File paths:** `scripts/sync-qiongli-releases.mjs`, `marketplace.json`, `.antigravity/catalog.json` in `jxpeng98/skillsplace`

**Problem:** The draft includes the correct follow-up, but it should not be executed in the Qiongli implementation branch before a Qiongli release asset exists.

**Why it violates the repository boundary:** `qiongli` should publish the new asset first. `skillsplace` should continue syncing catalog and URLs only; Qiongli source or generated plugin payloads should not be copied into `skillsplace`.

**Concrete fix:** Treat the `skillsplace` work as a separate follow-up after the Qiongli release is published.

---

## File Structure

- Modify `tooling/scripts/build_plugin_artifacts.py`
  - Add root plugin manifest writer.
  - Add separate next-channel direct plugin materializer.
  - Add direct Desktop/plugin ZIP builder.
  - Include the new ZIP in stable and prerelease `build_artifacts(...)`.
- Modify `tooling/scripts/validate_marketplace_install.py`
  - Validate `plugin.json` on marketplace plugin roots.
  - Validate the new direct plugin ZIP shape.
  - Require the direct plugin artifact in stable and prerelease validation.
- Modify `tooling/scripts/generate_release_downloads.py`
  - Add `claude_desktop_plugin` to release assets, URLs, recommendations, and markdown.
- Modify `tooling/scripts/release_postflight.sh`
  - Upload `dist/qiongli-claude-desktop-plugin-${TAG}.zip` for stable releases.
  - Upload `dist/qiongli-next-claude-desktop-plugin-${TAG}.zip` for prereleases.
- Modify `tooling/scripts/update_stable_download_sections.py`
  - Add direct plugin rows to generated README/docs stable download sections.
- Modify `tooling/scripts/generate_stable_release_notes.py`
  - Add the direct plugin to stable release notes.
- Modify `tooling/scripts/generate_release_notes.sh`
  - Add the direct plugin to prerelease and stable release notes generated by the legacy shell script.
- Modify `tests/test_plugin_distribution_contract.py`
  - Test root manifest materialization.
  - Test direct plugin artifact inclusion and ZIP layout.
  - Test validator output mentions the direct plugin artifact.
- Modify `tests/test_release_downloads.py`
  - Test direct plugin asset names in JSON index, markdown guide, stable README/docs sections, and release notes.
- Do not modify `scripts/*.py` wrappers unless a wrapper is missing. The current wrappers delegate to `tooling/scripts/...`.
- Do not modify `skillsplace` in this branch.

---

### Task 1: Cover Root Plugin Manifest Contract

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`
- Modify: `tooling/scripts/build_plugin_artifacts.py`

- [ ] **Step 1: Add failing tests for root `plugin.json` in materialized plugin payloads**

Insert these tests after `materialize_next_plugin_payload(...)` in `tests/test_plugin_distribution_contract.py`:

```python
    def test_materialized_plugin_has_root_plugin_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_plugin_payload(tmp_dir)
            manifest = json.loads((materialized_plugin / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest, {"name": "qiongli"})

    def test_materialized_next_plugin_has_root_plugin_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            materialized_plugin = self.materialize_next_plugin_payload(tmp_dir)
            manifest = json.loads((materialized_plugin / "plugin.json").read_text(encoding="utf-8"))

        self.assertEqual(manifest, {"name": "qiongli-next"})
```

- [ ] **Step 2: Run the new tests and verify they fail**

Run:

```bash
python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_materialized_plugin_has_root_plugin_manifest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_materialized_next_plugin_has_root_plugin_manifest
```

Expected: FAIL with `FileNotFoundError` for `plugin.json`.

- [ ] **Step 3: Add root plugin manifest writer**

Insert this helper near `_write_platform_manifest(...)` in `tooling/scripts/build_plugin_artifacts.py`:

```python
def _write_root_plugin_manifest(plugin_root: Path, plugin_name: str) -> None:
    plugin_root.mkdir(parents=True, exist_ok=True)
    (plugin_root / "plugin.json").write_text(
        json.dumps({"name": plugin_name}, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
```

- [ ] **Step 4: Call the helper from marketplace and materialized plugin builders**

In `_build_marketplace_plugin(...)`, immediately after `_write_subject_manifest(...)`, add:

```python
    _write_root_plugin_manifest(plugin_dest, plugin_name)
```

In `materialize_next_codex_plugin(...)`, immediately after `_write_subject_manifest(...)`, add:

```python
    _write_root_plugin_manifest(dest_plugin_root, NEXT_PLUGIN_NAME)
```

In `materialize_plugin_package(...)`, immediately after the optional Claude manifest is written, add:

```python
    _write_root_plugin_manifest(dest_plugin_root, PLUGIN_NAME)
```

- [ ] **Step 5: Run the root manifest tests and verify they pass**

Run:

```bash
python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_materialized_plugin_has_root_plugin_manifest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_materialized_next_plugin_has_root_plugin_manifest
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add tests/test_plugin_distribution_contract.py tooling/scripts/build_plugin_artifacts.py
git commit -m "test: cover desktop direct plugin manifest"
```

---

### Task 2: Build Direct Claude Desktop Plugin ZIP

**Files:**
- Modify: `tests/test_plugin_distribution_contract.py`
- Modify: `tooling/scripts/build_plugin_artifacts.py`

- [ ] **Step 1: Import `zipfile` in the distribution contract tests**

Add this import in `tests/test_plugin_distribution_contract.py`:

```python
import zipfile
```

- [ ] **Step 2: Add a failing test for the new direct plugin ZIP**

Insert this test near `test_marketplace_validator_builds_platform_artifacts_and_checks_invocation(...)`:

```python
    def test_build_artifacts_includes_direct_desktop_plugin(self) -> None:
        from scripts.build_plugin_artifacts import build_artifacts

        current_tag = (RepoLayout(REPO_ROOT).workflow / "VERSION").read_text(encoding="utf-8").strip()
        is_next = "-" in current_tag.removeprefix("v")
        plugin_name = "qiongli-next" if is_next else "qiongli"
        skill_name = "qiongli-next" if is_next else "qiongli"
        expected_name = f"{plugin_name}-claude-desktop-plugin-{current_tag}.zip"

        with tempfile.TemporaryDirectory() as tmp_dir:
            artifacts = build_artifacts(REPO_ROOT, current_tag, Path(tmp_dir))
            artifact_by_name = {artifact.name: artifact for artifact in artifacts}
            self.assertIn(expected_name, artifact_by_name)

            with zipfile.ZipFile(artifact_by_name[expected_name]) as archive:
                names = set(archive.namelist())
                manifest = json.loads(archive.read(f"{plugin_name}/plugin.json").decode("utf-8"))
                skill_text = archive.read(f"{plugin_name}/skills/qiongli-workflow/SKILL.md").decode("utf-8")

        self.assertEqual(manifest, {"name": plugin_name})
        self.assertIn(f"{plugin_name}/.codex-plugin/plugin.json", names)
        self.assertIn(f"{plugin_name}/.claude-plugin/plugin.json", names)
        self.assertIn(f"{plugin_name}/commands/qiongli.md", names)
        self.assertIn(f"{plugin_name}/mcp/qiongli-literature-provider/index.mjs", names)
        self.assertIn(f"{plugin_name}/skills/qiongli-workflow/SKILL.md", names)
        self.assertIn(f"name: {skill_name}", skill_text)
```

- [ ] **Step 3: Run the direct artifact test and verify it fails**

Run:

```bash
python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_build_artifacts_includes_direct_desktop_plugin
```

Expected: FAIL because `qiongli-claude-desktop-plugin-<tag>.zip` or `qiongli-next-claude-desktop-plugin-<tag>.zip` is not built yet.

- [ ] **Step 4: Add a separate next-channel direct plugin materializer**

Add this function after `materialize_next_codex_plugin(...)` in `tooling/scripts/build_plugin_artifacts.py`:

```python
def materialize_next_plugin_package(root: Path, dest_plugin_root: Path, *, force: bool = False) -> Path:
    """Materialize the generated qiongli-next plugin payload for direct plugin ZIP installs."""

    root = root.resolve()
    dest_plugin_root = dest_plugin_root.resolve()
    if dest_plugin_root.exists():
        if not force:
            raise ValueError(f"{dest_plugin_root} already exists; pass force=True to replace it")
        if dest_plugin_root.is_dir():
            shutil.rmtree(dest_plugin_root)
        else:
            dest_plugin_root.unlink()

    _display_name, package_goal = _subject_definitions(root)["core"]
    for platform, manifest_dir in (("codex", ".codex-plugin"), ("claude", ".claude-plugin")):
        _write_platform_manifest(
            root,
            platform,
            NEXT_PLUGIN_NAME,
            dest_plugin_root / manifest_dir / "plugin.json",
        )
        _write_subject_manifest(
            dest_plugin_root / manifest_dir / "plugin.json",
            platform=platform,
            plugin_name=NEXT_PLUGIN_NAME,
            subject="core",
            display_name="Qiongli Next",
            package_goal=package_goal,
            skill_name=NEXT_SKILL_NAME,
            mcp_server_name=NEXT_MCP_SERVER_NAME,
        )

    _write_root_plugin_manifest(dest_plugin_root, NEXT_PLUGIN_NAME)
    _copy_codex_mcp_manifest(root, dest_plugin_root, server_name=NEXT_MCP_SERVER_NAME)
    _copy_literature_mcp_runtime(root, dest_plugin_root)
    _copy_commands(root, dest_plugin_root, skill_name=NEXT_SKILL_NAME)
    _copy_subject_skill(root, dest_plugin_root, "core", skill_name=NEXT_SKILL_NAME)
    _copy_codex_workflow_wrapper_skills(root, dest_plugin_root, skill_name=NEXT_SKILL_NAME)
    return dest_plugin_root
```

This function is intentionally separate from `materialize_next_codex_plugin(...)`.

- [ ] **Step 5: Add direct Desktop/plugin ZIP builder**

Add this helper before `_build_claude_desktop_skill(...)`:

```python
def _build_claude_desktop_plugin(
    root: Path,
    tag: str,
    dist_dir: Path,
    work_dir: Path,
    *,
    plugin_name: str = PLUGIN_NAME,
    artifact_prefix: str = PLUGIN_NAME,
    next_channel: bool = False,
) -> Path:
    plugin_dest = work_dir / f"desktop-plugin-{artifact_prefix}" / plugin_name
    if next_channel:
        materialize_next_plugin_package(root, plugin_dest, force=True)
    else:
        materialize_plugin_package(root, plugin_dest, force=True)

    artifact = dist_dir / f"{artifact_prefix}-claude-desktop-plugin-{tag}.zip"
    _make_zip(plugin_dest, artifact)
    return artifact
```

- [ ] **Step 6: Include the direct plugin artifact in `build_artifacts(...)`**

In the prerelease branch, change the returned list to include the direct plugin before the skill ZIP:

```python
            return [
                *_build_next_marketplace_plugins(root, repo_tag, dist_dir, work_dir),
                _build_claude_desktop_plugin(
                    root,
                    repo_tag,
                    dist_dir,
                    work_dir,
                    plugin_name=NEXT_PLUGIN_NAME,
                    artifact_prefix=NEXT_PLUGIN_NAME,
                    next_channel=True,
                ),
                _build_claude_desktop_skill(
                    root,
                    repo_tag,
                    dist_dir,
                    work_dir,
                    "core",
                    artifact_prefix=NEXT_PLUGIN_NAME,
                    skill_name=NEXT_SKILL_NAME,
                ),
            ]
```

In the stable branch, add the new artifact to the `artifacts` list before Desktop/Web skill ZIPs:

```python
            _build_claude_desktop_plugin(root, repo_tag, dist_dir, work_dir),
```

- [ ] **Step 7: Run the direct artifact test and verify it passes**

Run:

```bash
python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_build_artifacts_includes_direct_desktop_plugin
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add tests/test_plugin_distribution_contract.py tooling/scripts/build_plugin_artifacts.py
git commit -m "build: add claude desktop plugin artifact"
```

---

### Task 3: Validate Marketplace Root Manifests And Direct Plugin Shape

**Files:**
- Modify: `tooling/scripts/validate_marketplace_install.py`
- Modify: `tests/test_plugin_distribution_contract.py`

- [ ] **Step 1: Add validator output assertions to the existing validator test**

In `test_marketplace_validator_builds_platform_artifacts_and_checks_invocation(...)`, add this assertion in the prerelease branch:

```python
            self.assertIn(
                "[OK] claude-desktop direct plugin artifact (core-next): "
                "qiongli-next invocation checked; bundled literature MCP checked",
                result.stdout,
            )
```

Add this assertion in the stable branch:

```python
            self.assertIn(
                "[OK] claude-desktop direct plugin artifact: "
                "qiongli invocation checked; bundled literature MCP checked",
                result.stdout,
            )
```

- [ ] **Step 2: Run the validator test and verify it fails**

Run:

```bash
python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_builds_platform_artifacts_and_checks_invocation
```

Expected: FAIL because validator output does not mention the direct plugin artifact yet.

- [ ] **Step 3: Add root manifest assertion helper**

Add this helper after `_read_json(...)` in `tooling/scripts/validate_marketplace_install.py`:

```python
def _assert_root_plugin_manifest(plugin_root: Path, expected_plugin_name: str) -> None:
    manifest_path = plugin_root / "plugin.json"
    manifest = _read_json(manifest_path)
    if manifest != {"name": expected_plugin_name}:
        raise ValueError(f"{manifest_path} expected {{'name': {expected_plugin_name!r}}}, found {manifest}")
```

In `_validate_artifact(...)`, after `plugin_root` is computed, add:

```python
        _assert_root_plugin_manifest(plugin_root, plugin_name)
```

- [ ] **Step 4: Add direct plugin ZIP validator**

Add this helper after `_validate_claude_desktop_artifact(...)`:

```python
def _validate_direct_desktop_plugin_artifact(
    artifact: Path,
    expected_repo_tag: str,
    expected_version: str,
    *,
    plugin_name: str = PLUGIN_NAME,
    skill_name: str = SKILL_NAME,
    subject_label: str | None = None,
) -> str:
    with tempfile.TemporaryDirectory(prefix="qiongli-direct-desktop-plugin-") as tmp:
        plugin_root = _extract_single_zip_root(artifact, Path(tmp))
        if plugin_root.name != plugin_name:
            raise ValueError(f"{artifact} must contain top-level {plugin_name}/ directory")

        _assert_root_plugin_manifest(plugin_root, plugin_name)
        _assert_manifest(
            "codex",
            plugin_root / ".codex-plugin" / "plugin.json",
            expected_version,
            expected_plugin_name=plugin_name,
            expected_skill_name=skill_name,
        )
        _assert_manifest(
            "claude",
            plugin_root / ".claude-plugin" / "plugin.json",
            expected_version,
            expected_plugin_name=plugin_name,
            expected_skill_name=skill_name,
        )

        skill_root = plugin_root / "skills" / SKILL_DIR_NAME
        workflow_names = _assert_skill_invocation(skill_root, expected_repo_tag, skill_name=skill_name)
        _assert_subject_marker(skill_root, "core")
        _assert_subject_manifest(skill_root, "core", "complete")
        _assert_command_invocation(plugin_root, workflow_names, skill_name=skill_name)
        _assert_bundled_literature_mcp(
            plugin_root,
            "codex",
            mcp_server_name=_mcp_server_name_for_plugin(plugin_name),
        )
        _assert_bundled_literature_mcp(
            plugin_root,
            "claude",
            mcp_server_name=_mcp_server_name_for_plugin(plugin_name),
        )

    subject_suffix = f" ({subject_label})" if subject_label else ""
    return (
        f"[OK] claude-desktop direct plugin artifact{subject_suffix}: "
        f"{skill_name} invocation checked; bundled literature MCP checked"
    )
```

- [ ] **Step 5: Require the direct plugin artifact in `validate(...)`**

In the prerelease branch, after validating `desktop_artifact`, add:

```python
        direct_plugin_name = f"{NEXT_PLUGIN_NAME}-claude-desktop-plugin-{expected_repo_tag}.zip"
        direct_plugin_artifact = by_platform.get(direct_plugin_name)
        if direct_plugin_artifact is None:
            raise ValueError(f"expected claude-desktop next direct plugin artifact: {direct_plugin_name}")
        messages.append(
            _validate_direct_desktop_plugin_artifact(
                direct_plugin_artifact,
                expected_repo_tag,
                expected_version,
                plugin_name=NEXT_PLUGIN_NAME,
                skill_name=NEXT_SKILL_NAME,
                subject_label="core-next",
            )
        )
```

In the stable branch, after validating the base Claude ZIP artifact, add:

```python
    direct_plugin_name = f"{PLUGIN_NAME}-claude-desktop-plugin-{expected_repo_tag}.zip"
    direct_plugin_artifact = by_platform.get(direct_plugin_name)
    if direct_plugin_artifact is None:
        raise ValueError(f"expected claude-desktop direct plugin artifact: {direct_plugin_name}")
    messages.append(
        _validate_direct_desktop_plugin_artifact(
            direct_plugin_artifact,
            expected_repo_tag,
            expected_version,
        )
    )
```

- [ ] **Step 6: Run the validator test and verify it passes**

Run:

```bash
python -m unittest \
  tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_builds_platform_artifacts_and_checks_invocation
```

Expected: PASS.

- [ ] **Step 7: Run the full distribution contract tests**

Run:

```bash
python -m unittest tests.test_plugin_distribution_contract
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add tooling/scripts/validate_marketplace_install.py tests/test_plugin_distribution_contract.py
git commit -m "test: validate desktop direct plugin artifact"
```

---

### Task 4: Add Direct Plugin To Release Download Metadata

**Files:**
- Modify: `tooling/scripts/generate_release_downloads.py`
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Add failing release download assertions**

In `test_generates_human_and_machine_download_guides(...)`, add:

```python
        self.assertIn("qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip", guide)
        self.assertEqual(
            index["recommended"]["claude_desktop_plugin"]["asset"],
            "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
        )
        self.assertEqual(
            index["assets"]["claude_desktop_plugin"],
            "qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
        )
        self.assertEqual(
            index["asset_urls"]["claude_desktop_plugin"],
            "https://github.com/jxpeng98/qiongli/releases/download/v1.1.0-beta.2/qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip",
        )
```

In `test_stable_download_section_updater_rewrites_docs(...)`, add:

```python
                self.assertIn("qiongli-claude-desktop-plugin-v1.6.0.zip", content)
```

- [ ] **Step 2: Run release download tests and verify they fail**

Run:

```bash
python -m unittest tests.test_release_downloads
```

Expected: FAIL because `claude_desktop_plugin` is missing.

- [ ] **Step 3: Add direct plugin asset to release index**

In `tooling/scripts/generate_release_downloads.py`, add:

```python
def _claude_desktop_plugin_zip(plugin_name: str, tag: str) -> str:
    return f"{plugin_name}-claude-desktop-plugin-{tag}.zip"
```

In `_release_assets(...)`, add this key to the prerelease return value:

```python
            "claude_desktop_plugin": _claude_desktop_plugin_zip(NEXT_PLUGIN_NAME, tag),
```

Add this key to the stable return value:

```python
        "claude_desktop_plugin": _claude_desktop_plugin_zip(PLUGIN_NAME, tag),
```

In `build_index(...)`, add this recommendation:

```python
        "claude_desktop_plugin": {
            "install": "download_plugin_zip",
            "asset": assets["claude_desktop_plugin"],
            "note": "Use this for direct Claude Desktop/plugin install; use skill ZIP only for manual skill upload.",
        },
```

- [ ] **Step 4: Render direct plugin in the download guide markdown**

In `render_markdown(...)`, add:

```python
    desktop_plugin_asset = str(assets["claude_desktop_plugin"])
    desktop_plugin_url = str(asset_urls["claude_desktop_plugin"])
```

In the `Direct downloads` table, add this row after the release page row:

```python
        f"| Claude Desktop direct plugin ZIP | {_markdown_link(desktop_plugin_asset, desktop_plugin_url)} |",
```

In the `Start here` table, replace the Desktop skills row with two separate rows:

```python
        "| Claude Desktop direct plugin | "
        f"Download `{desktop_plugin_asset}`. | "
        "Use this for direct Claude Desktop/plugin install. |",
        "| Claude Desktop/Web skills | Download exactly one Desktop skill ZIP from the table below. | Use skill ZIPs only for manual skill upload. |",
```

- [ ] **Step 5: Run release download tests and verify the direct metadata assertions pass**

Run:

```bash
python -m unittest tests.test_release_downloads
```

Expected: FAIL only if the README/docs stable updater and release notes have not yet been updated. The new JSON and guide assertions should pass.

- [ ] **Step 6: Commit**

```bash
git add tooling/scripts/generate_release_downloads.py tests/test_release_downloads.py
git commit -m "docs: include desktop direct plugin download"
```

---

### Task 5: Propagate Direct Plugin To Release Uploads And Public Docs

**Files:**
- Modify: `tooling/scripts/release_postflight.sh`
- Modify: `tooling/scripts/update_stable_download_sections.py`
- Modify: `tooling/scripts/generate_stable_release_notes.py`
- Modify: `tooling/scripts/generate_release_notes.sh`
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Add release note assertions for the direct plugin**

In `test_release_notes_include_download_guide_section(...)`, add:

```python
        self.assertIn("qiongli-next-claude-desktop-plugin-v1.1.0-beta.2.zip", notes)
```

In `test_stable_release_notes_include_category_downloads_and_changelog(...)`, add:

```python
        self.assertIn("qiongli-claude-desktop-plugin-v1.5.0.zip", notes)
```

- [ ] **Step 2: Run release download tests and verify remaining failures**

Run:

```bash
python -m unittest tests.test_release_downloads
```

Expected: FAIL in stable section or release note assertions until the scripts below are updated.

- [ ] **Step 3: Upload the new asset in postflight**

In `tooling/scripts/release_postflight.sh`, add this entry to the prerelease `PLUGIN_ARTIFACTS` array after `qiongli-next-claude-plugin-${TAG}.zip`:

```bash
    "dist/qiongli-next-claude-desktop-plugin-${TAG}.zip"
```

Add this entry to the stable `PLUGIN_ARTIFACTS` array after `qiongli-claude-plugin-${TAG}.zip`:

```bash
    "dist/qiongli-claude-desktop-plugin-${TAG}.zip"
```

- [ ] **Step 4: Add direct plugin to stable README/docs download sections**

In `tooling/scripts/update_stable_download_sections.py`, inside `_render_block(...)`, add:

```python
    desktop_plugin_asset = _required_asset(index, "claude_desktop_plugin")
    desktop_plugin_url = _required_url(index, "claude_desktop_plugin")
```

In the Chinese table, add this row before the Desktop/Web core skill row:

```python
                f"| Claude Desktop direct plugin | [`{desktop_plugin_asset}`]({desktop_plugin_url}) |",
```

In the English table, add this row before the Desktop/Web core skill row:

```python
            f"| Claude Desktop direct plugin | [`{desktop_plugin_asset}`]({desktop_plugin_url}) |",
```

- [ ] **Step 5: Add direct plugin to stable release notes**

In `tooling/scripts/generate_stable_release_notes.py`, after `desktop_core_asset` is selected, add:

```python
    desktop_plugin_asset = _required_string(assets.get("claude_desktop_plugin"), "Desktop direct plugin asset")
    desktop_plugin_url = _required_string(asset_urls.get("claude_desktop_plugin"), "Desktop direct plugin URL")
```

In the release category text, change:

```python
        f"`{tag}` is the stable release for normal installs and upgrades. Use it for npm `latest`, PyPI stable, the `qiongli` marketplace entry, Claude Desktop/Web skill ZIPs, the literature MCPB, and the Zotero companion XPI.",
```

to:

```python
        f"`{tag}` is the stable release for normal installs and upgrades. Use it for npm `latest`, PyPI stable, the `qiongli` marketplace entry, the Claude Desktop direct plugin ZIP, Claude Desktop/Web skill ZIPs, the literature MCPB, and the Zotero companion XPI.",
```

In the download table, add this row before the default Desktop/Web skill ZIP row:

```python
        f"| Claude Desktop direct plugin ZIP | {_asset_link(desktop_plugin_asset, desktop_plugin_url)} |",
```

- [ ] **Step 6: Add direct plugin to legacy shell release notes**

In `tooling/scripts/generate_release_notes.sh`, update the prerelease Desktop row from:

```bash
    echo "| Claude Desktop/Web skills | Download \`qiongli-next-claude-desktop-skill-core-${TAG}.zip\` for the prerelease core skill. |"
```

to:

```bash
    echo "| Claude Desktop direct plugin | Download \`qiongli-next-claude-desktop-plugin-${TAG}.zip\` for direct plugin install. |"
    echo "| Claude Desktop/Web skills | Download \`qiongli-next-claude-desktop-skill-core-${TAG}.zip\` only for manual skill upload. |"
```

Update the stable Desktop row from:

```bash
    echo "| Claude Desktop/Web skills | Download one \`qiongli-claude-desktop-skill-<subject>-${TAG}.zip\` asset. Start with \`qiongli-claude-desktop-skill-core-${TAG}.zip\` unless you need a subject package. |"
```

to:

```bash
    echo "| Claude Desktop direct plugin | Download \`qiongli-claude-desktop-plugin-${TAG}.zip\` for direct plugin install. |"
    echo "| Claude Desktop/Web skills | Download one \`qiongli-claude-desktop-skill-<subject>-${TAG}.zip\` asset only for manual skill upload. Start with \`qiongli-claude-desktop-skill-core-${TAG}.zip\` unless you need a subject package. |"
```

- [ ] **Step 7: Run release download tests and verify they pass**

Run:

```bash
python -m unittest tests.test_release_downloads
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add \
  tooling/scripts/release_postflight.sh \
  tooling/scripts/update_stable_download_sections.py \
  tooling/scripts/generate_stable_release_notes.py \
  tooling/scripts/generate_release_notes.sh \
  tests/test_release_downloads.py
git commit -m "docs: surface desktop direct plugin release asset"
```

---

### Task 6: Full Verification

**Files:**
- No code changes expected.

- [ ] **Step 1: Run focused unit tests**

Run:

```bash
python -m unittest tests.test_plugin_distribution_contract tests.test_release_downloads
```

Expected: PASS.

- [ ] **Step 2: Run marketplace/direct artifact validator**

Run:

```bash
python scripts/validate_marketplace_install.py
```

Expected output includes the existing marketplace, Claude Desktop skill, and the new direct plugin message:

```text
[OK] claude-desktop direct plugin artifact
```

- [ ] **Step 3: Build release artifacts into a temp dist directory**

Run:

```bash
python scripts/build_plugin_artifacts.py --tag "$(cat qiongli-workflow/VERSION)" --dist-dir /tmp/qiongli-direct-plugin-dist
```

Expected output includes one of:

```text
qiongli-claude-desktop-plugin-<tag>.zip
qiongli-next-claude-desktop-plugin-<tag>.zip
```

- [ ] **Step 4: Inspect the ZIP layout manually**

For a stable tag, run:

```bash
python - <<'PY'
import json
import zipfile
from pathlib import Path

tag = Path("qiongli-workflow/VERSION").read_text(encoding="utf-8").strip()
artifact = Path("/tmp/qiongli-direct-plugin-dist") / f"qiongli-claude-desktop-plugin-{tag}.zip"
if not artifact.exists():
    artifact = Path("/tmp/qiongli-direct-plugin-dist") / f"qiongli-next-claude-desktop-plugin-{tag}.zip"
with zipfile.ZipFile(artifact) as archive:
    names = set(archive.namelist())
    root = sorted({name.split("/", 1)[0] for name in names if "/" in name})[0]
    manifest = json.loads(archive.read(f"{root}/plugin.json").decode("utf-8"))
    required = [
        f"{root}/plugin.json",
        f"{root}/.codex-plugin/plugin.json",
        f"{root}/.claude-plugin/plugin.json",
        f"{root}/skills/qiongli-workflow/SKILL.md",
        f"{root}/mcp/qiongli-literature-provider/index.mjs",
    ]
    missing = [name for name in required if name not in names]
    if missing:
        raise SystemExit(f"missing: {missing}")
    print(artifact.name)
    print(manifest)
PY
```

Expected:

```text
qiongli-...-claude-desktop-plugin-<tag>.zip
{'name': 'qiongli'} or {'name': 'qiongli-next'}
```

- [ ] **Step 5: Check git status**

Run:

```bash
git status --short
```

Expected: clean if all task commits were made.

---

### Task 7: Separate `skillsplace` Follow-Up After Qiongli Release

**Files in `jxpeng98/skillsplace`:**
- Modify: `scripts/sync-qiongli-releases.mjs`
- Modify: `marketplace.json`
- Modify: `.antigravity/catalog.json` if that catalog remains the direct-plugin adapter
- Modify tests that currently expect Qiongli native plugin status to be pending

- [ ] **Step 1: Confirm the Qiongli release contains the new asset**

Run in `skillsplace` or any clean shell:

```bash
gh release view <tag> --repo jxpeng98/qiongli --json assets
```

Expected asset:

```text
qiongli-claude-desktop-plugin-<tag>.zip
```

For prerelease-only validation, expect:

```text
qiongli-next-claude-desktop-plugin-<tag>.zip
```

- [ ] **Step 2: Update catalog status and URL routing**

Change Qiongli native/direct status from:

```json
"status": "pending-native-manifest"
```

to:

```json
"status": "ready"
```

Point the Desktop/direct plugin route to:

```text
https://github.com/jxpeng98/qiongli/releases/download/<tag>/qiongli-claude-desktop-plugin-<tag>.zip
```

- [ ] **Step 3: Validate `skillsplace`**

Run in `skillsplace`:

```bash
npm run validate
```

Expected: PASS.

---

## Self-Review Checklist

- Spec coverage: root `plugin.json`, direct plugin artifact, stable/prerelease naming, release upload, download metadata, validation, and `skillsplace` follow-up are covered.
- Boundary check: this plan keeps Qiongli implementation in `jxpeng98/qiongli` and keeps `skillsplace` as a later catalog/URL sync only.
- Backward compatibility: existing artifacts are not renamed or removed:
  - `qiongli-claude-plugin-<tag>.zip`
  - `qiongli-claude-plugin-<tag>.tar.gz`
  - `qiongli-claude-desktop-skill-core-<tag>.zip`
  - `qiongli-literature-provider-*.mcpb`
- Placeholder scan: all task steps name concrete files, commands, and expected outcomes.
