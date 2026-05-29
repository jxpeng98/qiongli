# Political Economy and Geoeconomics Subjects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `political-economy` and `geoeconomics` as official Qiongli subjects with Claude Desktop focused ZIP support.

**Architecture:** Reuse the existing subject materializer and catalog model. Add two subject layers with canonical domain profiles, subject overlays, venue profiles, one expert audit skill per subject, tests, docs, and release artifact coverage.

**Tech Stack:** Python 3.12, unittest, YAML subject catalogs and profiles, Markdown skill cards, existing release artifact builder.

---

## File Structure

- Modify `subjects/catalog.yaml`: add two official subject definitions, selected assets, skill groups, and overrides.
- Create `subjects/political-economy/**`: overlays, venue profiles, registry, and `political-economy-mechanism-auditor.md`.
- Create `subjects/geoeconomics/**`: overlays, venue profiles, registry, and `geoeconomic-statecraft-auditor.md`.
- Create `skills/domain-profiles/political-economy.yaml` and `skills/domain-profiles/geoeconomics.yaml`.
- Modify `scripts/audit_subject_specialization.py`: add expected depth terms for both subjects.
- Modify `tests/test_subject_catalog.py`, `tests/test_subject_materializer.py`, `tests/test_subject_specialization_audit.py`, `tests/test_distribution_payloads.py`, and `tests/test_plugin_artifacts.py`.
- Modify docs in `README.md`, `README_CN.md`, `docs/advanced/subject-packaging-model.md`, and `docs/zh/advanced/subject-packaging-model.md`.
- Regenerate payloads with `scripts/sync_npm_package_payload.py` after source tests pass.

## Task 1: Failing Subject Catalog and Materializer Tests

**Files:**
- Modify: `tests/test_subject_catalog.py`
- Modify: `tests/test_subject_materializer.py`
- Modify: `tests/test_subject_specialization_audit.py`
- Modify: `tests/test_distribution_payloads.py`
- Modify: `tests/test_plugin_artifacts.py`

- [ ] **Step 1: Add catalog tests**

Add tests that assert the catalog now contains `political-economy` and `geoeconomics`, each extends `core`, has exactly five ordered groups, selects its own domain profile, and includes its subject-specific auditor.

- [ ] **Step 2: Add materializer tests**

Add focused materialization tests for both subjects. Political economy must include `political-economy-mechanism-auditor`, `political-economy.yaml`, `apsr.yaml`, and a manuscript overlay containing `political mechanism`. Geoeconomics must include `geoeconomic-statecraft-auditor`, `geoeconomics.yaml`, `international-security.yaml`, and a manuscript overlay containing `strategic economic statecraft`.

- [ ] **Step 3: Add audit and artifact tests**

Extend subject specialization expected terms and distribution/plugin artifact tests so both new subjects are required in generated payloads and Claude Desktop ZIP artifacts.

- [ ] **Step 4: Verify RED**

Run:

```bash
uv run python -m unittest \
  tests.test_subject_catalog \
  tests.test_subject_materializer \
  tests.test_subject_specialization_audit \
  tests.test_distribution_payloads \
  tests.test_plugin_artifacts -v
```

Expected: FAIL because subject source files and generated payloads do not exist yet.

## Task 2: Subject Source Assets

**Files:**
- Modify: `subjects/catalog.yaml`
- Create: `skills/domain-profiles/political-economy.yaml`
- Create: `skills/domain-profiles/geoeconomics.yaml`
- Create: `subjects/political-economy/**`
- Create: `subjects/geoeconomics/**`

- [ ] **Step 1: Add domain profiles**

Create domain profiles with method templates, assumptions, diagnostics, failure modes, and reporting expectations aligned to the approved design.

- [ ] **Step 2: Add subject registries and expert skills**

Create subject registry entries and expert skill markdown for `political-economy-mechanism-auditor` and `geoeconomic-statecraft-auditor`.

- [ ] **Step 3: Add overlays and venue profiles**

Create append overlays for the selected generic skills and venue profiles for each selected venue.

- [ ] **Step 4: Add catalog entries**

Add both subjects to `subjects/catalog.yaml` with compact five-group focused workflow maps and selected assets.

- [ ] **Step 5: Verify GREEN for source-level tests**

Run:

```bash
uv run python -m unittest \
  tests.test_subject_catalog \
  tests.test_subject_materializer \
  tests.test_subject_specialization_audit -v
```

Expected: PASS.

## Task 3: Distribution Payloads and Release Artifacts

**Files:**
- Generated: `qiongli/payload/subjects/**`
- Generated: `packages/npm-qiongli/payload/subjects/**`
- Generated: `packages/npm-qiongli/python-runtime/**`
- Modify: `scripts/build_plugin_artifacts.py` only if fallback subject metadata needs explicit coverage

- [ ] **Step 1: Regenerate package payloads**

Run:

```bash
uv run python scripts/sync_npm_package_payload.py --root .
```

Expected: generated Python and npm payloads include both subjects in `complete` and `focused` coverage.

- [ ] **Step 2: Verify distribution and artifact tests**

Run:

```bash
uv run python -m unittest tests.test_distribution_payloads tests.test_plugin_artifacts -v
```

Expected: PASS.

## Task 4: Documentation

**Files:**
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `docs/advanced/subject-packaging-model.md`
- Modify: `docs/zh/advanced/subject-packaging-model.md`

- [ ] **Step 1: Update subject lists and Desktop ZIP examples**

Add `political-economy` and `geoeconomics` to official subject lists, install examples, Desktop ZIP examples, npm payload descriptions, and subject packaging guidance.

- [ ] **Step 2: Verify docs references**

Run:

```bash
rg -n "political-economy|geoeconomics" README.md README_CN.md docs/advanced/subject-packaging-model.md docs/zh/advanced/subject-packaging-model.md
```

Expected: both subject IDs appear in English and Chinese docs.

## Task 5: Final Verification and Release Prep

**Files:**
- All changed source, generated payload, docs, and release metadata files.

- [ ] **Step 1: Run focused subject suite**

Run:

```bash
uv run python -m unittest \
  tests.test_subject_catalog \
  tests.test_subject_materializer \
  tests.test_subject_specialization_audit \
  tests.test_distribution_payloads \
  tests.test_plugin_artifacts \
  tests.test_npm_package_contract -v
```

Expected: PASS.

- [ ] **Step 2: Run release readiness**

Because this adds two installable subjects and release artifacts, use a minor prerelease from `dev`: `v0.14.0-beta.1`.

Run:

```bash
./scripts/release_ready.sh --version 0.14.0b1
```

Expected: synchronized version files, regenerated release readiness checks, and no failures.

- [ ] **Step 3: Commit implementation and release prep**

Use Conventional Commits:

```bash
git add <changed-files>
git commit -m "feat(subjects): add political economy and geoeconomics packages"
```

If `release_ready.sh` creates separate release-prep changes after the feature commit, commit them separately:

```bash
git add <release-files>
git commit -m "chore(release): prepare 0.14.0b1"
```

- [ ] **Step 4: Publish prerelease**

Run the standard release automation from `dev`:

```bash
./scripts/release_automation.sh publish --tag v0.14.0-beta.1 --skip-bump --from-tag v0.13.0
```

Expected: release branch and tag are pushed, prerelease artifacts are created, and acceptance receipt is generated.
