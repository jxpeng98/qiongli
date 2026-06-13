# Single-Source Plugin Distribution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove tracked plugin payload and wrapper source trees, then generate every installable plugin artifact from canonical Qiongli content, metadata, and runtime sources.

**Architecture:** `content/` remains the only workflow and academic-content source. A new canonical distribution metadata file drives manifest generation, while `build_plugin_artifacts.py` and `materialize_distribution_payloads.py` create plugin directories only in staging or ignored generated-output paths.

**Tech Stack:** Python 3, PyYAML, `unittest`, shell release scripts, existing Qiongli materializers.

---

## File Structure

- Create `content/distribution/plugins.yaml`: canonical plugin family metadata for stable, prerelease, platform, prompt, keyword, MCP, and invocation settings.
- Create `packages/python-qiongli/src/qiongli/distribution_metadata.py`: typed loader and validator for `content/distribution/plugins.yaml`.
- Modify `packages/python-qiongli/src/qiongli/source_layout.py`: remove plugin package source roots and classify generated plugin artifact roots.
- Modify `tooling/scripts/build_plugin_artifacts.py`: generate manifests, command wrappers, Gemini metadata, bundled MCP runtime, stable artifacts, and next artifacts from canonical sources.
- Modify `tooling/scripts/materialize_distribution_payloads.py`: materialize plugin outputs to generated/staged paths without reading checked-in plugin package mirrors.
- Modify `tooling/scripts/validate_marketplace_install.py`: validate artifacts emitted by the generator and stop relying on `packages/qiongli-*plugin`.
- Modify `tooling/scripts/verify_release_tag_version.sh`: verify generated or staged plugin manifests instead of deleted checked-in plugin manifests.
- Modify `tests/test_distribution_source_tree.py`: assert old plugin package mirrors are no longer tracked.
- Modify `tests/test_generated_payload_guard.py`: classify plugin package mirror paths as generated payloads.
- Modify `tests/test_plugin_manifests.py`: test canonical metadata and generated manifest output.
- Modify `tests/test_plugin_distribution_contract.py`: test generated staged artifacts only.
- Modify `tests/test_cross_platform_routing_grill_contract.py`: inspect staged plugin artifacts only.
- Modify docs under `docs/architecture.md` and `docs/development/distribution-materialization.md`: document the single-source plugin model.
- Delete tracked `packages/qiongli-plugin/` and `packages/qiongli-next-plugin/` trees.

## Task 1: Lock Source Boundary Tests

**Files:**
- Modify: `tests/test_distribution_source_tree.py`
- Modify: `tests/test_generated_payload_guard.py`
- Modify: `packages/python-qiongli/src/qiongli/source_layout.py`

- [ ] **Step 1: Write failing source-tree assertions**

Add this assertion to `DistributionSourceTreeTests.test_generated_distribution_outputs_are_not_tracked`:

```python
        self.assertNotIn("packages/qiongli-plugin/.codex-plugin/plugin.json", tracked)
        self.assertNotIn("packages/qiongli-next-plugin/.codex-plugin/plugin.json", tracked)
```

Add these generated root expectations near `GENERATED_OUTPUT_ROOTS` assertions:

```python
        self.assertIn("packages/qiongli-plugin", GENERATED_OUTPUT_ROOTS)
        self.assertIn("packages/qiongli-next-plugin", GENERATED_OUTPUT_ROOTS)
```

- [ ] **Step 2: Write failing generated guard expectations**

In `tests/test_generated_payload_guard.py`, add these paths to `generated_paths`:

```python
            "packages/qiongli-plugin/.codex-plugin/plugin.json",
            "packages/qiongli-plugin/commands/paper.md",
            "packages/qiongli-plugin/platforms/agent/workflows/paper.md",
            "packages/qiongli-next-plugin/.codex-plugin/plugin.json",
            "packages/qiongli-next-plugin/skills/qiongli-workflow/SKILL.md",
```

Remove these paths from `source_paths`:

```python
            "packages/qiongli-plugin/.codex-plugin/plugin.json",
            "packages/qiongli-plugin/commands/paper.md",
            "packages/qiongli-plugin/platforms/agent/workflows/paper.md",
            "packages/qiongli-plugin/platforms/gemini/qiongli.md",
            "packages/qiongli-next-plugin/.codex-plugin/plugin.json",
            "packages/qiongli-next-plugin/skills/qiongli-workflow/SKILL.md",
            "packages/qiongli-next-plugin/mcp/qiongli-literature-provider/index.mjs",
```

- [ ] **Step 3: Run tests and verify they fail**

Run:

```bash
python3 -m unittest tests.test_distribution_source_tree tests.test_generated_payload_guard -v
```

Expected: failure showing plugin package paths are still treated as source or are still tracked.

- [ ] **Step 4: Update generated root classification**

In `packages/python-qiongli/src/qiongli/source_layout.py`, add:

```python
    Path("packages/qiongli-plugin"),
    Path("packages/qiongli-next-plugin"),
```

to `GENERATED_OUTPUT_ROOTS`.

- [ ] **Step 5: Run focused tests**

Run:

```bash
python3 -m unittest tests.test_generated_payload_guard -v
```

Expected: generated guard tests pass while source-tree tracking tests still fail until deletion happens.

## Task 2: Add Canonical Plugin Metadata

**Files:**
- Create: `content/distribution/plugins.yaml`
- Create: `packages/python-qiongli/src/qiongli/distribution_metadata.py`
- Modify: `tests/test_plugin_manifests.py`

- [ ] **Step 1: Write failing metadata loader tests**

Add tests to `tests/test_plugin_manifests.py`:

```python
from qiongli.distribution_metadata import load_plugin_distribution


class PluginDistributionMetadataTests(unittest.TestCase):
    def test_canonical_distribution_metadata_defines_stable_and_next_plugins(self) -> None:
        metadata = load_plugin_distribution(REPO_ROOT)

        self.assertEqual(set(metadata.plugins), {"qiongli", "qiongli-next"})
        self.assertEqual(metadata.plugins["qiongli"].skill_name, "qiongli")
        self.assertEqual(metadata.plugins["qiongli-next"].skill_name, "qiongli-next")
        self.assertEqual(metadata.plugins["qiongli"].mcp_server_name, "qiongli")
        self.assertEqual(metadata.plugins["qiongli-next"].mcp_server_name, "qiongli-next")

    def test_canonical_distribution_metadata_carries_discovery_terms(self) -> None:
        metadata = load_plugin_distribution(REPO_ROOT)
        stable = metadata.plugins["qiongli"]
        searchable = " ".join([stable.description, *stable.keywords, *stable.default_prompts]).lower()

        for term in EXPECTED_DISCOVERY_TERMS:
            with self.subTest(term=term):
                self.assertIn(term, searchable)
```

- [ ] **Step 2: Run metadata tests and verify they fail**

Run:

```bash
python3 -m unittest tests.test_plugin_manifests.PluginDistributionMetadataTests -v
```

Expected: import failure for `qiongli.distribution_metadata` or missing metadata file.

- [ ] **Step 3: Create metadata YAML**

Create `content/distribution/plugins.yaml` with:

```yaml
plugins:
  qiongli:
    display_name: Qiongli
    skill_name: qiongli
    mcp_server_name: qiongli
    description: Qiongli academic research workflow for literature, manuscripts, statistics, analysis code, reproducibility, rebuttal, submission, presentation, and stage-aware grill.
    author:
      name: Jiaxin Peng
      url: https://github.com/jxpeng98
    category: Education
    homepage: https://github.com/jxpeng98/qiongli
    repository: https://github.com/jxpeng98/qiongli
    license: MIT
    keywords:
      - research
      - academic-writing
      - literature-review
      - manuscript
      - analysis
      - statistics
      - reproducibility
      - rebuttal
      - academic-code
      - grill
    codex:
      short_description: Academic research workflows for Codex.
      default_prompts:
        - Use $qiongli to plan or narrow an academic research project.
        - Use $qiongli to revise a manuscript with claim-evidence checks.
        - Use $qiongli to review statistics, analysis code, or rebuttal risks.
      brand_color: "#2563EB"
    claude:
      enabled: true
    gemini:
      enabled: true
  qiongli-next:
    display_name: Qiongli Next
    skill_name: qiongli-next
    mcp_server_name: qiongli-next
    description: Qiongli Next prerelease academic research workflow plugin for testing the upcoming core workflow with bundled literature MCP tools.
    author:
      name: Jiaxin Peng
      url: https://github.com/jxpeng98
    category: Education
    homepage: https://github.com/jxpeng98/qiongli
    repository: https://github.com/jxpeng98/qiongli
    license: MIT
    keywords:
      - research
      - academic-writing
      - literature-review
      - manuscript
      - analysis
      - statistics
      - reproducibility
      - rebuttal
      - academic-code
      - grill
      - qiongli-next
      - prerelease
    codex:
      short_description: Prerelease academic research workflows for Codex.
      default_prompts:
        - Use $qiongli-next to test the next Qiongli paper workflow.
        - Use $qiongli-next to test a literature review workflow.
        - Use $qiongli-next to test submission checks with the bundled literature MCP.
      brand_color: "#2563EB"
    claude:
      enabled: true
    gemini:
      enabled: false
```

- [ ] **Step 4: Implement metadata loader**

Create `packages/python-qiongli/src/qiongli/distribution_metadata.py` with dataclasses:

```python
@dataclass(frozen=True)
class PluginDefinition:
    id: str
    display_name: str
    skill_name: str
    mcp_server_name: str
    description: str
    author: dict[str, str]
    category: str
    homepage: str
    repository: str
    license: str
    keywords: tuple[str, ...]
    default_prompts: tuple[str, ...]
    codex_short_description: str
    brand_color: str
    claude_enabled: bool
    gemini_enabled: bool
```

Expose:

```python
def load_plugin_distribution(root: Path | str) -> PluginDistribution:
    ...
```

Use `yaml.safe_load`, require a top-level `plugins` object, and raise `PluginDistributionError` with a path-specific message for missing fields.

- [ ] **Step 5: Run metadata tests**

Run:

```bash
python3 -m unittest tests.test_plugin_manifests.PluginDistributionMetadataTests -v
```

Expected: all metadata tests pass.

## Task 3: Generate Manifests And Wrappers

**Files:**
- Modify: `tooling/scripts/build_plugin_artifacts.py`
- Modify: `tests/test_plugin_manifests.py`
- Modify: `tests/test_plugin_distribution_contract.py`

- [ ] **Step 1: Write failing generated manifest tests**

Add a helper in `tests/test_plugin_manifests.py`:

```python
    def materialize_plugin_root(self, tmp_dir: str, target: str = "plugin") -> Path:
        out = Path(tmp_dir) / "dist-source"
        result = subprocess.run(
            [
                sys.executable,
                "scripts/materialize_distribution_payloads.py",
                "--target",
                target,
                "--out",
                str(out),
                "--force",
            ],
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, msg=result.stderr + result.stdout)
        return out / "plugins" / "qiongli"
```

Add generated manifest assertions that read staged `plugins/qiongli/.codex-plugin/plugin.json`,
`plugins/qiongli/.claude-plugin/plugin.json`, and `gemini-extension.json` from generated artifacts.

- [ ] **Step 2: Verify generated tests fail for package-source dependency**

Run:

```bash
python3 -m unittest tests.test_plugin_manifests tests.test_plugin_distribution_contract -v
```

Expected: tests still pass before deletion, then fail after package source deletion unless generator stops copying from `packages/qiongli-plugin`.

- [ ] **Step 3: Implement manifest builders**

In `tooling/scripts/build_plugin_artifacts.py`, import:

```python
from qiongli.distribution_metadata import PluginDefinition, load_plugin_distribution
```

Add:

```python
def _plugin_definition(root: Path, plugin_name: str) -> PluginDefinition:
    return load_plugin_distribution(root).plugins[plugin_name]
```

Add manifest writers:

```python
def _write_codex_manifest(path: Path, plugin: PluginDefinition, version: str) -> None:
    manifest = {
        "name": plugin.id,
        "version": version,
        "description": plugin.description,
        "author": plugin.author,
        "category": plugin.category,
        "homepage": plugin.homepage,
        "repository": plugin.repository,
        "license": plugin.license,
        "keywords": [*plugin.keywords, "codex-skills"],
        "skills": "./skills/",
        "mcpServers": "./.mcp.json",
        "interface": {
            "displayName": plugin.display_name,
            "shortDescription": plugin.codex_short_description,
            "longDescription": plugin.description,
            "developerName": plugin.author["name"],
            "category": plugin.category,
            "capabilities": ["Write"],
            "websiteURL": plugin.repository,
            "defaultPrompt": list(plugin.default_prompts),
            "brandColor": plugin.brand_color,
        },
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
```

Add equivalent `_write_claude_manifest`, `_write_gemini_manifest`, and `_write_codex_mcp_manifest` helpers.

- [ ] **Step 4: Implement wrapper generation**

Add:

```python
def _workflow_description(workflow_path: Path) -> str:
    text = workflow_path.read_text(encoding="utf-8")
    match = re.search(r"(?ms)^---\n(.*?)\n---", text)
    if not match:
        return f"Run the {workflow_path.stem} research workflow."
    desc = re.search(r"(?m)^description:\s*(.+)$", match.group(1))
    return desc.group(1).strip() if desc else f"Run the {workflow_path.stem} research workflow."


def _generate_commands(root: Path, commands_root: Path, skill_name: str) -> None:
    workflow_root = RepoLayout(root).workflow / "workflows"
    commands_root.mkdir(parents=True, exist_ok=True)
    for workflow_path in sorted(workflow_root.glob("*.md")):
        text = "\n".join(
            [
                "---",
                f"description: {_workflow_description(workflow_path)}",
                "---",
                "",
                f"Load the `{skill_name}` skill from this plugin, then follow `skills/qiongli-workflow/workflows/{workflow_path.name}`.",
                "",
                "Use that workflow as the source of truth for task order, artifacts, and quality gates.",
                "",
            ]
        )
        (commands_root / workflow_path.name).write_text(text, encoding="utf-8")
```

- [ ] **Step 5: Replace package-source copy calls**

Replace `_copy_path(RepoLayout(root).plugin_package / manifest_dir, ...)`,
`_copy_commands`, and direct `gemini-extension.json` copy calls with generated manifest and wrapper helpers.

- [ ] **Step 6: Run manifest and distribution tests**

Run:

```bash
python3 -m unittest tests.test_plugin_manifests tests.test_plugin_distribution_contract -v
```

Expected: generated manifest and wrapper tests pass.

## Task 4: Bundle MCP Runtime From MCPB Source

**Files:**
- Modify: `tooling/scripts/build_plugin_artifacts.py`
- Modify: `tests/test_plugin_manifests.py`
- Modify: `tests/test_plugin_distribution_contract.py`

- [ ] **Step 1: Write failing MCP source assertion**

In generated plugin tests, assert:

```python
self.assertTrue((materialized_plugin / "mcp" / "qiongli-literature-provider" / "index.mjs").is_file())
self.assertTrue((materialized_plugin / "mcp" / "qiongli-literature-provider" / "providers" / "openalex.mjs").is_file())
```

- [ ] **Step 2: Change MCP runtime copy source**

In `build_plugin_artifacts.py`, replace `RepoLayout(root).plugin_package / "mcp"` with:

```python
source = RepoLayout(root).literature_mcpb_package / "server"
dest = dest_plugin_root / "mcp" / "qiongli-literature-provider"
```

Copy the entire server directory and preserve provider files.

- [ ] **Step 3: Run MCP-focused tests**

Run:

```bash
python3 -m unittest tests.test_plugin_manifests.PluginManifestTests.test_codex_plugin_bundles_qiongli_mcp_server tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_codex_plugin_materializes_bundled_mcp_manifest -v
```

Expected: both tests pass against generated artifacts.

## Task 5: Delete Tracked Plugin Source Trees

**Files:**
- Delete: `packages/qiongli-plugin/`
- Delete: `packages/qiongli-next-plugin/`
- Modify: `packages/python-qiongli/src/qiongli/source_layout.py`
- Modify: `tooling/scripts/materialize_distribution_payloads.py`

- [ ] **Step 1: Delete tracked plugin package trees**

Run:

```bash
git rm -r packages/qiongli-plugin packages/qiongli-next-plugin
```

Expected: git stages deletions for old plugin mirrors.

- [ ] **Step 2: Update layout properties**

In `RepoLayout`, make plugin artifact paths generated-only:

```python
    @property
    def plugin_artifact_package(self) -> Path:
        return self.root / "plugins" / "qiongli"

    @property
    def next_plugin_artifact_package(self) -> Path:
        return self.root / "plugins" / "qiongli-next"
```

Remove fallback behavior that treats `packages/qiongli-plugin` as a source root.

- [ ] **Step 3: Update materialize target destinations**

In `materialize_distribution_payloads.py`, ensure:

```python
plugin_root = layout.plugin_artifact_package
next_plugin_root = root / "plugins" / "qiongli-next"
```

Stable and next plugin artifacts should be generated under `plugins/`, not under deleted `packages/` paths.

- [ ] **Step 4: Run source boundary tests**

Run:

```bash
python3 -m unittest tests.test_distribution_source_tree tests.test_generated_payload_guard -v
```

Expected: tests pass and no plugin package source files are tracked.

## Task 6: Update Release And Marketplace Validation

**Files:**
- Modify: `tooling/scripts/verify_release_tag_version.sh`
- Modify: `tooling/scripts/release_ready.sh`
- Modify: `tooling/scripts/release_automation.sh`
- Modify: `tooling/scripts/validate_marketplace_install.py`
- Modify: `tests/test_release_automation.py`
- Modify: `tests/test_release_downloads.py`

- [ ] **Step 1: Write failing release-script assertions**

Update tests that grep or assert release paths so `packages/qiongli-plugin` and
`packages/qiongli-next-plugin` are not required release sources.

Use this expected source allowlist phrase:

```python
"content/distribution/plugins.yaml"
```

and this forbidden path assertion:

```python
self.assertNotIn("packages/qiongli-next-plugin", script_text)
```

- [ ] **Step 2: Update tag verifier**

Change `verify_release_tag_version.sh` to materialize into a temp directory:

```bash
release_verify_tmp="$(mktemp -d)"
trap 'rm -rf "$release_verify_tmp"' EXIT
python3 scripts/materialize_distribution_payloads.py --target all --out "$release_verify_tmp/dist" --force
```

Read plugin manifest and payload versions from `$release_verify_tmp/dist/plugins/qiongli` and `$release_verify_tmp/dist/plugins/qiongli-next`.

- [ ] **Step 3: Update release source allowlists**

In release scripts, replace package plugin paths with:

```bash
content/distribution/plugins.yaml
tooling/scripts/build_plugin_artifacts.py
tooling/scripts/materialize_distribution_payloads.py
```

- [ ] **Step 4: Run release-focused tests**

Run:

```bash
python3 -m unittest tests.test_release_automation tests.test_release_downloads tests.test_plugin_distribution_contract -v
```

Expected: tests pass without checked-in plugin source directories.

## Task 7: Update Docs

**Files:**
- Modify: `docs/architecture.md`
- Modify: `docs/development/distribution-materialization.md`
- Modify: `docs/development/repository-structure.md`

- [ ] **Step 1: Update architecture source boundary docs**

Change package-shell wording from tracked plugin source to generated plugin artifacts. Keep `packages/qiongli-literature-mcpb/` as an editable package shell.

- [ ] **Step 2: Update materialization docs**

State that `packages/qiongli-plugin/` and `packages/qiongli-next-plugin/` are no longer source paths and that plugin artifacts are generated from `content/distribution/plugins.yaml`.

- [ ] **Step 3: Run doc grep checks**

Run:

```bash
rg -n "packages/qiongli-plugin|packages/qiongli-next-plugin" docs tooling tests packages/python-qiongli/src/qiongli
```

Expected: remaining references describe deleted generated paths, release migration notes, or guard tests only.

## Task 8: Final Verification And Commit

**Files:**
- All changed implementation, test, and doc files.

- [ ] **Step 1: Run focused regression suite**

Run:

```bash
python3 -m unittest \
  tests.test_distribution_source_tree \
  tests.test_generated_payload_guard \
  tests.test_plugin_manifests \
  tests.test_plugin_distribution_contract \
  tests.test_cross_platform_routing_grill_contract \
  tests.test_release_automation \
  tests.test_release_downloads -v
```

Expected: all listed tests pass.

- [ ] **Step 2: Run staged plugin materialization smoke**

Run:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-single-source-dist --force
python3 scripts/validate_marketplace_install.py --dist-dir /tmp/qiongli-single-source-marketplace
```

Expected: materialization and marketplace validation commands exit 0.

- [ ] **Step 3: Inspect git status**

Run:

```bash
git status --short
```

Expected: no generated plugin payload files under ignored staging paths; deleted tracked package mirrors are staged only as intentional deletions.

- [ ] **Step 4: Commit implementation**

Run:

```bash
git add content/distribution packages/python-qiongli/src/qiongli tooling/scripts tests docs
git add -u packages/qiongli-plugin packages/qiongli-next-plugin
git commit -m "refactor(plugin): generate plugin distributions from canonical sources"
```

Expected: commit succeeds with tests already passing.

## Self-Review

- Spec coverage: every acceptance criterion maps to tasks for metadata, generator, source deletion, staged validation, release updates, docs, and guards.
- Placeholder scan: the plan contains no deferred implementation markers.
- Type consistency: metadata types use `PluginDefinition`, `PluginDistribution`, and `load_plugin_distribution` consistently across tests and generator steps.
