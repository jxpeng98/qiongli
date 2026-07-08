# Local Install Acceptance Recommended Clients Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release local-install acceptance map platform targets to client validators by registry `release_download.recommended_key` metadata instead of fixed target IDs.

**Architecture:** Keep smoke-policy filtering in `local_install_acceptance_targets()`. Change only the second step, `_local_acceptance_targets_by_client()`, so target records selected by `smoke.client_activation_check=local_install_acceptance` resolve their Codex, Claude Code, and Antigravity clients through `release_download.recommended_key`.

**Tech Stack:** Python `unittest`, Qiongli platform target registry, release local install acceptance script.

---

### Task 1: Add Recommended-Key Client Mapping Regression

**Files:**
- Modify: `tests/test_release_local_install_check.py`

- [ ] **Step 1: Write the failing test**

Add this test to `ReleaseLocalInstallCheckTests` after `test_validate_install_tree_requires_registry_target_metadata_in_markers`:

```python
    def test_local_acceptance_client_mapping_uses_registry_recommended_keys(self) -> None:
        module = load_release_local_install_check()
        with tempfile.TemporaryDirectory() as tmp_dir:
            repo_root = Path(tmp_dir) / "repo"
            self._write_renamed_local_acceptance_registry(repo_root)

            targets_by_client = module._local_acceptance_targets_by_client(repo_root)

        self.assertEqual(targets_by_client["codex"].target_id, "fixture-codex-target")
        self.assertEqual(targets_by_client["claude"].target_id, "fixture-claude-target")
        self.assertEqual(targets_by_client["antigravity"].target_id, "fixture-antigravity-target")
```

Add this helper before `_write_repo_version`:

```python
    def _write_renamed_local_acceptance_registry(self, repo_root: Path) -> None:
        registry = repo_root / "content" / "distribution" / "platform-targets.yaml"
        registry.parent.mkdir(parents=True, exist_ok=True)
        registry.write_text(
            """
schema_version: "1.0"
targets:
  fixture-codex-target:
    display_name: Codex Fixture
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: codex
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.claude-plugin/]
    bundled_mcp_mode: codex-plugin-local-node
    command_surface: slash-commands
    validator: fixture-codex-validator
    release_download:
      guide_label: Codex
      recommended_key: codex
      asset_groups: []
  fixture-claude-target:
    display_name: Claude Fixture
    artifact_kind: marketplace-plugin
    archive_format: tar.gz
    adapter:
      kind: plugin
      plugin_manifest_platform: claude
      materializer: plugin_artifacts
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: claude-plugin-local-node
    command_surface: slash-commands
    validator: fixture-claude-validator
    release_download:
      guide_label: Claude Code
      recommended_key: claude_code
      asset_groups: []
  fixture-antigravity-target:
    display_name: Antigravity Fixture
    artifact_kind: local-plugin
    archive_format: directory
    adapter:
      kind: local-plugin
      plugin_manifest_platform: none
      materializer: local_plugin_installer
    smoke:
      structural_archive_check: marketplace_validation
      client_activation_check: local_install_acceptance
    source_inputs: [content/workflow/**]
    required_paths: [plugin.json]
    allowed_wrapper_dirs: []
    forbidden_paths: [.codex-plugin/]
    bundled_mcp_mode: antigravity-python-runtime
    command_surface: slash-commands
    validator: fixture-antigravity-validator
    release_download:
      guide_label: Antigravity
      recommended_key: antigravity
      asset_groups: []
""".lstrip(),
            encoding="utf-8",
        )
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check.ReleaseLocalInstallCheckTests.test_local_acceptance_client_mapping_uses_registry_recommended_keys -q
```

Expected: FAIL with `local-install acceptance target fixture-codex-target has no client validation mapping`, proving the script still maps clients by fixed target ID.

### Task 2: Resolve Clients By Recommended Key

**Files:**
- Modify: `tooling/scripts/release_local_install_check.py`

- [ ] **Step 1: Write minimal implementation**

Replace the old `LOCAL_INSTALL_TARGET_CLIENTS` constant with:

```python
LOCAL_INSTALL_RECOMMENDED_KEY_CLIENTS = {
    "codex": ("codex", "Codex"),
    "claude_code": ("claude", "Claude"),
    "antigravity": ("antigravity", "Antigravity"),
}
```

Update `_local_acceptance_targets_by_client()` to read the target `release_download.recommended_key`:

```python
def _local_acceptance_targets_by_client(repo_root: Path) -> dict[str, PlatformTarget]:
    targets_by_client: dict[str, PlatformTarget] = {}
    for target_id, target in local_install_acceptance_targets(repo_root).items():
        recommended_key = target.release_download.get("recommended_key")
        if not isinstance(recommended_key, str) or not recommended_key:
            raise LocalInstallCheckError(
                f"local-install acceptance target {target_id} has no release_download.recommended_key"
            )
        client = LOCAL_INSTALL_RECOMMENDED_KEY_CLIENTS.get(recommended_key)
        if client is None:
            raise LocalInstallCheckError(
                "local-install acceptance target "
                f"{target_id} has no client validation mapping for "
                f"release_download.recommended_key={recommended_key!r}"
            )
        client_id, _label = client
        targets_by_client[client_id] = target
    return targets_by_client
```

- [ ] **Step 2: Run test to verify it passes**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check.ReleaseLocalInstallCheckTests.test_local_acceptance_client_mapping_uses_registry_recommended_keys -q
```

Expected: PASS.

### Task 3: Update Roadmap Status

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record the completed slice**

Update Stage 12 and the remaining product-gap summary to say local-install acceptance client mapping also follows registry `release_download.recommended_key` metadata. Do not change the opt-in local-agent smoke status.

### Task 4: Verify And Commit

**Files:**
- Test: `tests/test_release_local_install_check.py`
- Test: `tests/test_local_plugin_installer.py`
- Test: `tests/test_universal_installer.py`
- Test: `tooling/scripts/validate_platform_targets.py`

- [ ] **Step 1: Run focused regression**

```bash
.venv/bin/python -m unittest tests.test_release_local_install_check tests.test_local_plugin_installer tests.test_universal_installer -q
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
rg -n "(/[U]sers/|/[p]rivate/|BEGI[N] (RSA|OPENSSH|EC|DSA) PRIVATE KEY|secre[t]:|toke[n]:|passwor[d]:)" docs/superpowers/plans/2026-07-06-local-install-acceptance-recommended-clients.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md tooling/scripts/release_local_install_check.py tests/test_release_local_install_check.py
```

Expected: no matches.

- [ ] **Step 5: Commit implementation**

```bash
git add tooling/scripts/release_local_install_check.py tests/test_release_local_install_check.py
git commit -m "feat(release): map local install clients by recommended key"
```

- [ ] **Step 6: Commit roadmap update**

```bash
git add docs/superpowers/plans/2026-07-06-local-install-acceptance-recommended-clients.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record local install client target lookup"
```
