# v1.13.0 Release Polish Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the v1.13.0 post-release audit issues around PyPI messaging, release verification ergonomics, acceptance receipt evidence, and CLI version reporting.

**Architecture:** Keep the source checkout output-free and continue using staged materialization for installable payload validation. Add narrow regression tests before each behavior change, then make small updates to docs, shell release helpers, and CLI entrypoints without changing package layout or release artifact generation semantics.

**Tech Stack:** Python 3.12+ `unittest`/`argparse`, Node 18+ `node:test`, Bash release scripts, existing Qiongli distribution materializer.

---

## File Structure

- Modify `README_PYPI.md`: align PyPI long description with the current full-runtime boundary, Antigravity naming, project guidance, and npm/Python split.
- Modify `tests/test_package_readmes.py`: add a PyPI README contract test that blocks stale Gemini-only/runtime wording and requires current 1.13.0 concepts.
- Modify `tooling/scripts/verify_release_tag_version.sh`: add a configurable Python executable, a dependency preflight, and safe default-root behavior that materializes to a temporary staging tree when generated payloads are absent.
- Modify `tests/test_release_automation.py`: add static contract tests for the release verifier and acceptance template.
- Modify `tooling/release/templates/beta-acceptance-template.md`: replace unchecked boxes with generated evidence statements.
- Modify `packages/npm-qiongli/lib/cli.mjs`: support `--version` and `version` without invoking install/check paths.
- Modify `packages/npm-qiongli/test/cli.test.mjs`: test npm version output.
- Modify `packages/python-qiongli/src/qiongli/cli.py`: add top-level `--version` through `argparse`.
- Modify `tests/test_cli.py`: test Python CLI version output.

## Task 1: Refresh PyPI Long Description

**Files:**
- Modify: `README_PYPI.md`
- Modify: `tests/test_package_readmes.py`

- [ ] **Step 1: Add the failing PyPI README contract test**

Add this method to `PackageReadmeTests` in `tests/test_package_readmes.py`:

```python
    def test_pypi_readme_documents_current_runtime_boundaries(self) -> None:
        text = (REPO_ROOT / "README_PYPI.md").read_text(encoding="utf-8")

        self.assertIn("full local runtime", text)
        self.assertIn("Codex, Claude Code, Antigravity, and Hermes", text)
        self.assertIn("Project subject guidance", text)
        self.assertIn("qiongli project init", text)
        self.assertIn("qiongli project set-subject finance", text)
        self.assertIn("npm/npx is the Python-free asset manager", text)
        self.assertIn("qiongli doctor", text)
        self.assertNotIn("and Gemini.", text)
        self.assertNotIn("Support `codex`, `claude`, `gemini`, or `all` targets", text)
        self.assertNotIn("lightweight updater CLI", text)
```

- [ ] **Step 2: Run the targeted failing test**

Run:

```bash
uv run python -m unittest tests.test_package_readmes.PackageReadmeTests.test_pypi_readme_documents_current_runtime_boundaries -v
```

Expected: FAIL because `README_PYPI.md` still says "lightweight updater CLI", mentions "Gemini" as a current target, and does not describe the 1.13.0 npm/Python boundary.

- [ ] **Step 3: Replace the stale opening and capability sections**

In `README_PYPI.md`, replace lines 1-61 with this content:

```markdown
# qiongli

`qiongli` is the full local runtime CLI for **Qiongli** (`穷理`), a contract-driven academic workflow system for Codex, Claude Code, Antigravity, and Hermes.

The full system name is **Qiongli Zhengche** (`穷理证澈`): Qiongli names the public research workflow, while Zhengche names the evidence-governance method that keeps claims, citations, assumptions, and output paths auditable.

## What it does

- Install or refresh Qiongli skill and plugin assets for Codex, Claude Code, Antigravity, and Hermes
- Run full-runtime commands such as `qiongli doctor`, `qiongli mcp serve`, provider setup, `task-plan`, `task-run`, and `customize`
- Manage Project subject guidance with `qiongli project init`, `qiongli project status`, and `qiongli project set-subject <subject>`
- Select subject packages with `--subject core|economics|accounting|business|finance|political-economy|geoeconomics|economics-accounting`
- Choose package coverage with `--coverage complete|focused` (`complete` is the default)
- Keep local workflow assets separate from CLI package upgrades

## Installation

For a global full-runtime CLI command, `pipx` is recommended:

```bash
pipx install qiongli
```

Upgrade an existing PyPI install with:

```bash
pipx upgrade qiongli
```

You can also install with `pip`:

```bash
pip install qiongli
```

If you use `pip` inside an active virtual environment, the CLI is installed into that environment. That only affects where the Python command lives; it does not make the workflow assets project-local.

## npm and Python boundaries

npm/npx is the Python-free asset manager. Use it for scripted skill/plugin asset installs, current-package asset refreshes, and basic project subject guidance:

```bash
npm install -g qiongli
qiongli install --target all --surface skills
qiongli project init --project-dir "$PWD"
qiongli project set-subject finance --project-dir "$PWD"
```

Use this PyPI package when you need the full local runtime: `qiongli doctor`, `qiongli mcp serve`, provider configuration, orchestration previews, `task-run`, `team-run`, `parallel`, and local customization.

## Global-first update model

Current releases separate the package install from workflow asset installation:

- `pipx install qiongli` / `pipx upgrade qiongli` updates the full-runtime Python CLI.
- `qiongli install --target all` installs the default `core/complete` package.
- `qiongli install --subject economics --target all` installs the full framework plus economics specialization.
- `qiongli install --subject accounting --target all` installs the full framework plus accounting specialization.
- `qiongli install --subject business --target all` installs the full framework plus business/management specialization.
- `qiongli install --subject finance --target all` installs the full framework plus finance specialization.
- `qiongli install --subject political-economy --target all` installs the full framework plus political economy specialization.
- `qiongli install --subject geoeconomics --target all` installs the full framework plus geoeconomics specialization.
- `qiongli install --subject economics --coverage focused --target all` installs the slimmer economics-focused package.
- `qiongli install --subject economics-accounting --target all` installs the official economics/accounting composite.
- `qiongli upgrade --subject accounting --target all` refreshes the active global assets without upgrading the Python package.
- `--project-dir` selects a project when you run `doctor`, read project-level config, or explicitly write project files.

Global assets are written under client home directories such as:

```text
~/.codex/skills/qiongli-workflow
~/.claude/skills/qiongli-workflow
~/.gemini/antigravity/skills/qiongli-workflow
~/.hermes/skills/qiongli-workflow
```
```

- [ ] **Step 4: Keep the rest of `README_PYPI.md` coherent**

In the remaining sections of `README_PYPI.md`, make these exact text replacements:

```text
Replace: qiongli init --project-dir /path/to/project
With:    qiongli project init --project-dir /path/to/project

Replace: qiongli customize plus --custom-dir materialization is for the Python/source checkout workflow; npm runtime installs use pre-generated payloads in this phase.
With:    qiongli customize plus --custom-dir materialization is for the Python/source checkout workflow; npm asset-manager installs use pre-generated payloads in this phase.
```

- [ ] **Step 5: Verify the PyPI README tests pass**

Run:

```bash
uv run python -m unittest tests.test_package_readmes -v
```

Expected: PASS.

- [ ] **Step 6: Commit Task 1**

```bash
git add README_PYPI.md tests/test_package_readmes.py
git commit -m "docs: refresh PyPI runtime boundaries"
```

## Task 2: Make Release Version Verification Safe From Clean Checkout

**Files:**
- Modify: `tooling/scripts/verify_release_tag_version.sh`
- Modify: `tests/test_release_automation.py`
- Check: `docs/advanced/publish-pypi.md`
- Check: `docs/zh/advanced/publish-pypi.md`

- [ ] **Step 1: Add the failing release verifier contract test**

Add this method to `ReleaseAutomationTests` in `tests/test_release_automation.py`:

```python
    def test_verify_release_tag_script_materializes_clean_checkout_root(self) -> None:
        content = VERIFY_RELEASE_TAG.read_text(encoding="utf-8")

        self.assertIn('PYTHON_BIN="${PYTHON:-python3}"', content)
        self.assertIn('require_python_module yaml PyYAML', content)
        self.assertIn("materialize_distribution_payloads.py", content)
        self.assertIn("AUTO_MATERIALIZED_ROOT", content)
        self.assertIn("packages/npm-qiongli/payload/qiongli-workflow/VERSION", content)
```

- [ ] **Step 2: Run the targeted failing test**

Run:

```bash
uv run python -m unittest tests.test_release_automation.ReleaseAutomationTests.test_verify_release_tag_script_materializes_clean_checkout_root -v
```

Expected: FAIL because the script uses hard-coded `python3`, has no dependency preflight, and assumes generated payload directories already exist.

- [ ] **Step 3: Add Python selection, dependency preflight, and cleanup**

Insert this block after `TAG=""` in `tooling/scripts/verify_release_tag_version.sh`:

```bash
PYTHON_BIN="${PYTHON:-python3}"
AUTO_MATERIALIZED_ROOT=""

cleanup() {
  if [[ -n "$AUTO_MATERIALIZED_ROOT" && -d "$AUTO_MATERIALIZED_ROOT" ]]; then
    rm -rf "$AUTO_MATERIALIZED_ROOT"
  fi
}

trap cleanup EXIT

require_python_module() {
  local module="$1"
  local package="$2"

  if "$PYTHON_BIN" -c "import ${module}" >/dev/null 2>&1; then
    return 0
  fi

  echo "[verify-release-tag] missing Python dependency: ${package} (module: ${module})" >&2
  echo "[verify-release-tag] install release dependencies first, for example:" >&2
  echo "  python3 -m pip install -e ." >&2
  echo "  uv run bash scripts/verify_release_tag_version.sh --root /tmp/qiongli-dist --tag ${TAG:-<tag>}" >&2
  exit 1
}

ensure_materialized_root() {
  if [[ -f "packages/npm-qiongli/payload/qiongli-workflow/VERSION" && -d "plugins/qiongli" ]]; then
    return 0
  fi

  if [[ ! -f "scripts/materialize_distribution_payloads.py" ]]; then
    echo "[verify-release-tag] root is not a materialized distribution tree and cannot materialize itself: $ROOT_DIR" >&2
    exit 1
  fi

  AUTO_MATERIALIZED_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-verify-release.XXXXXX")"
  echo "[verify-release-tag] materializing clean checkout into $AUTO_MATERIALIZED_ROOT"
  "$PYTHON_BIN" scripts/materialize_distribution_payloads.py --target all --out "$AUTO_MATERIALIZED_ROOT" --force
  ROOT_DIR="$AUTO_MATERIALIZED_ROOT"
  cd "$ROOT_DIR"
}
```

- [ ] **Step 4: Call the new helpers before reading version files**

Replace:

```bash
cd "$ROOT_DIR"

expected_repo_tag="$(python3 scripts/sync_versions.py "$TAG" --print-field repo_version)"
expected_package_version="$(python3 scripts/sync_versions.py "$TAG" --print-field package_version)"
expected_skill_version="$(python3 scripts/sync_versions.py "$TAG" --print-field skill_version)"
expected_npm_version="$(python3 scripts/sync_versions.py "$TAG" --print-field npm_version)"
```

With:

```bash
cd "$ROOT_DIR"
require_python_module yaml PyYAML
ensure_materialized_root

expected_repo_tag="$("$PYTHON_BIN" scripts/sync_versions.py "$TAG" --print-field repo_version)"
expected_package_version="$("$PYTHON_BIN" scripts/sync_versions.py "$TAG" --print-field package_version)"
expected_skill_version="$("$PYTHON_BIN" scripts/sync_versions.py "$TAG" --print-field skill_version)"
expected_npm_version="$("$PYTHON_BIN" scripts/sync_versions.py "$TAG" --print-field npm_version)"
```

- [ ] **Step 5: Replace remaining hard-coded Python calls in the script**

In `tooling/scripts/verify_release_tag_version.sh`, replace every remaining command prefix `python3` with `"$PYTHON_BIN"` when it runs repository scripts or heredoc snippets. Examples:

```bash
actual_package_version="$("$PYTHON_BIN" - <<'PY'
```

```bash
"$PYTHON_BIN" scripts/audit_distribution_payloads.py --root "$ROOT_DIR"
```

Do not change documentation text inside `usage()`.

- [ ] **Step 6: Verify direct source-tree use now works in the managed environment**

Run:

```bash
uv run bash scripts/verify_release_tag_version.sh --tag v1.13.0
```

Expected output includes:

```text
[verify-release-tag] materializing clean checkout into
[verify-release-tag] tag and repo versions are aligned: v1.13.0
```

- [ ] **Step 7: Verify explicit staging root still works**

Run:

```bash
uv run python scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
uv run bash scripts/verify_release_tag_version.sh --root /tmp/qiongli-dist --tag v1.13.0
```

Expected output includes:

```text
[verify-release-tag] tag and repo versions are aligned: v1.13.0
```

Expected output does not include:

```text
[verify-release-tag] materializing clean checkout into
```

- [ ] **Step 8: Run release automation tests**

Run:

```bash
uv run python -m unittest tests.test_release_automation -v
```

Expected: PASS.

- [ ] **Step 9: Commit Task 2**

```bash
git add tooling/scripts/verify_release_tag_version.sh tests/test_release_automation.py
git commit -m "fix: make release tag verification stage-aware"
```

## Task 3: Replace Empty Acceptance Checklists With Evidence Statements

**Files:**
- Modify: `tooling/release/templates/beta-acceptance-template.md`
- Modify: `tests/test_release_automation.py`
- Check: `tooling/scripts/release_postflight.sh`

- [ ] **Step 1: Add the failing acceptance template contract test**

Add this method to `ReleaseAutomationTests` in `tests/test_release_automation.py`:

```python
    def test_acceptance_receipt_template_records_evidence_not_empty_checkboxes(self) -> None:
        template = (REPO_ROOT / "tooling" / "release" / "templates" / "beta-acceptance-template.md").read_text(
            encoding="utf-8"
        )

        self.assertNotIn("- [ ]", template)
        self.assertIn("## Generated Evidence", template)
        self.assertIn("CI Status: {{CI_STATUS}}", template)
        self.assertIn("Release automation completed postflight before generating this receipt.", template)
        self.assertIn("Owner:", template)
        self.assertIn("Reviewer:", template)
```

- [ ] **Step 2: Run the targeted failing test**

Run:

```bash
uv run python -m unittest tests.test_release_automation.ReleaseAutomationTests.test_acceptance_receipt_template_records_evidence_not_empty_checkboxes -v
```

Expected: FAIL because the current template uses unchecked checklist rows.

- [ ] **Step 3: Replace the acceptance template**

Replace the entire content of `tooling/release/templates/beta-acceptance-template.md` with:

```markdown
# Release Acceptance Receipt — {{TAG}}

- Date: {{DATE}}
- Release Tag: {{TAG}}
- Commit: {{COMMIT}}
- CI Status: {{CI_STATUS}}

## Generated Evidence

- Release automation completed preflight before creating or verifying the release tag.
- Release automation completed postflight before generating this receipt.
- Remote branch/tag consistency was verified by postflight.
- GitHub Actions branch checks and tag publish workflow status were queried by postflight.
- GitHub Release page state and artifact upload path were verified by postflight.
- Rollback documentation was present at `tooling/release/rollback.md`.

## Validation Commands

- `python3 scripts/validate_research_standard.py --strict`
- `python3 -m unittest discover -s tests -v`
- `./scripts/run_beta_smoke.sh`
- `bash scripts/verify_release_tag_version.sh --root <staged-dist> --tag {{TAG}}`
- `bash scripts/pypi_preflight.sh --root <staged-dist>`
- `bash scripts/npm_preflight.sh --root <staged-dist>`

## Collaboration Coverage

- Release smoke covers literature smoke and orchestrator doctor paths.
- Maintainer smoke covers `parallel` and `task-run` profile paths when release automation is run with `--maintainer-smoke`.
- Any unavailable external model workers must be recorded in the release notes or this receipt's notes.

## Sign-off

- Owner:
- Reviewer:
- Notes:
```

- [ ] **Step 4: Verify postflight still fills all template placeholders**

Run:

```bash
uv run python -m unittest tests.test_release_automation.ReleaseAutomationTests.test_acceptance_receipt_template_records_evidence_not_empty_checkboxes -v
```

Expected: PASS.

- [ ] **Step 5: Run release automation tests**

Run:

```bash
uv run python -m unittest tests.test_release_automation -v
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

```bash
git add tooling/release/templates/beta-acceptance-template.md tests/test_release_automation.py
git commit -m "docs: record release acceptance evidence"
```

## Task 4: Add Top-Level Version Commands

**Files:**
- Modify: `packages/npm-qiongli/lib/cli.mjs`
- Modify: `packages/npm-qiongli/test/cli.test.mjs`
- Modify: `packages/python-qiongli/src/qiongli/cli.py`
- Modify: `tests/test_cli.py`

- [ ] **Step 1: Add failing npm CLI tests**

Add these tests to `packages/npm-qiongli/test/cli.test.mjs` after the existing help test:

```javascript
test('top-level --version prints npm package version', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const { exitCode, stdout, stderr } = await runMain(['--version'], { packageRoot });

  assert.equal(exitCode, 0);
  assert.equal(stderr, '');
  assert.equal(stdout, '0.0.0-test\n');
});

test('version command prints npm package version', async (t) => {
  const packageRoot = createMinimalPackageRoot(t);
  const { exitCode, stdout, stderr } = await runMain(['version'], { packageRoot });

  assert.equal(exitCode, 0);
  assert.equal(stderr, '');
  assert.equal(stdout, '0.0.0-test\n');
});
```

- [ ] **Step 2: Run the targeted npm failing tests**

Run:

```bash
npm --prefix packages/npm-qiongli test -- --test-name-pattern "version"
```

Expected: FAIL because npm CLI currently treats `--version` and `version` as unknown commands.

- [ ] **Step 3: Implement npm version handling**

In `packages/npm-qiongli/lib/cli.mjs`, insert this helper near `helpText()`:

```javascript
function packageVersion(root) {
  const packageJsonPath = path.join(root, 'package.json');
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
  return packageJson.version || '<unknown>';
}
```

Because `cli.mjs` currently does not import `fs` or `path`, add these imports at the top:

```javascript
import fs from 'node:fs';
import path from 'node:path';
```

Then insert this early return after `const [rawCommand = 'help'] = argv;`:

```javascript
  if (rawCommand === '--version' || rawCommand === '-v' || rawCommand === 'version') {
    stdout.write(`${packageVersion(root)}\n`);
    return 0;
  }
```

- [ ] **Step 4: Run npm CLI tests**

Run:

```bash
npm --prefix packages/npm-qiongli test
```

Expected: PASS.

- [ ] **Step 5: Add failing Python CLI version test**

Add this method to `InstallerCliTests` in `tests/test_cli.py`:

```python
    def test_top_level_version_prints_package_version(self) -> None:
        stdout = io.StringIO()
        with mock.patch.object(cli_module.sys, "argv", ["qiongli", "--version"]), contextlib.redirect_stdout(stdout):
            with self.assertRaises(SystemExit) as raised:
                cli_module.main()

        self.assertEqual(raised.exception.code, 0)
        self.assertEqual(stdout.getvalue().strip(), cli_module.__version__)
```

- [ ] **Step 6: Run the targeted Python failing test**

Run:

```bash
uv run python -m unittest tests.test_cli.InstallerCliTests.test_top_level_version_prints_package_version -v
```

Expected: FAIL because `build_parser()` does not define a top-level version action.

- [ ] **Step 7: Implement Python version handling**

In `packages/python-qiongli/src/qiongli/cli.py`, add this line in `build_parser()` immediately after the line that closes the top-level `parser = argparse.ArgumentParser` call and before the existing `subparsers = parser.add_subparsers(dest="cmd", required=True)` line:

```python
    parser.add_argument("--version", action="version", version=__version__)
```

- [ ] **Step 8: Run Python CLI tests**

Run:

```bash
uv run python -m unittest tests.test_cli.InstallerCliTests.test_top_level_version_prints_package_version -v
uv run python -m unittest tests.test_cli -v
```

Expected: PASS.

- [ ] **Step 9: Commit Task 4**

```bash
git add packages/npm-qiongli/lib/cli.mjs packages/npm-qiongli/test/cli.test.mjs packages/python-qiongli/src/qiongli/cli.py tests/test_cli.py
git commit -m "feat: add qiongli version commands"
```

## Final Verification

- [ ] **Step 1: Run Python source validation**

Run:

```bash
uv run python scripts/validate_research_standard.py --strict
```

Expected:

```text
Summary: 5540 passed, 0 failed, 0 warnings
```

- [ ] **Step 2: Run npm tests**

Run:

```bash
npm --prefix packages/npm-qiongli test
```

Expected: all npm tests pass.

- [ ] **Step 3: Run release automation tests**

Run:

```bash
uv run python -m unittest tests.test_release_automation tests.test_package_readmes tests.test_cli -v
```

Expected: PASS.

- [ ] **Step 4: Run full Python test suite**

Run:

```bash
uv run python -m unittest discover -s tests -v
```

Expected: PASS.

- [ ] **Step 5: Verify clean checkout release tag command**

Run:

```bash
uv run bash scripts/verify_release_tag_version.sh --tag v1.13.0
```

Expected:

```text
[verify-release-tag] tag and repo versions are aligned: v1.13.0
```

- [ ] **Step 6: Check worktree**

Run:

```bash
git status --short --branch
```

Expected: only intended commits are present and the worktree is clean.

## Self-Review

- Spec coverage: The plan covers all four audit findings: stale PyPI messaging, clean-checkout release verification failure, unchecked acceptance receipt template, and missing top-level version commands.
- Placeholder scan: The plan uses concrete files, exact test functions, exact command lines, exact expected results, and exact replacement content.
- Type consistency: Python tests target `InstallerCliTests` and `PackageReadmeTests`; Node tests use the existing `runMain()` and `createMinimalPackageRoot()` helpers; shell changes stay inside the existing release verifier script.
