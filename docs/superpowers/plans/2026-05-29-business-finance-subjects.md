# Business And Finance Subjects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class `business` and `finance` Qiongli subjects with Codex, Claude Code, and Claude Desktop release artifacts, then publish stable `v0.13.0`.

**Architecture:** The subjects extend the existing `core` subject and follow the current `economics` and `accounting` patterns: catalog metadata drives materialization, overlays specialize shared skills, and release scripts discover marketplace subjects from the catalog. Skillsplace will expose independent install entries that point to `v0.13.0` release assets.

**Tech Stack:** Python subject materializer and release scripts, Node npm wrapper tests, YAML subject catalogs, JSON plugin marketplace metadata, GitHub release automation.

---

### Task 1: Subject Contract Tests

**Files:**
- Modify: `tests/test_subject_catalog.py`
- Modify: `tests/test_cli.py`
- Modify: `packages/npm-qiongli/test/installer.test.mjs`
- Modify: `tests/test_plugin_artifacts.py`
- Modify: `tests/test_release_automation.py`

- [x] Add failing assertions that `business` and `finance` are valid catalog subjects, extend `core`, expose doctoral/journal-grade package goals, and have focused subject-specific skill refs.
- [x] Add failing CLI/npm assertions that available subjects include `business` and `finance`.
- [x] Add failing artifact assertions for `qiongli-business-*`, `qiongli-finance-*`, and Claude Desktop ZIP outputs.
- [x] Run focused tests and confirm failures are caused by missing subject support.

### Task 2: Subject Material

**Files:**
- Modify: `subjects/catalog.yaml`
- Create: `subjects/business/skills/business-journal-positioning-auditor.md`
- Create: `subjects/business/skills/registry.yaml`
- Create: `subjects/business/overlays/skills/manuscript-architect.md`
- Create: `subjects/business/overlays/skills/study-designer.md`
- Create: `subjects/business/overlays/skills/stats-engine.md`
- Create: `subjects/business/venue-profiles/*.yaml`
- Create: `subjects/finance/skills/finance-identification-risk-auditor.md`
- Create: `subjects/finance/skills/registry.yaml`
- Create: `subjects/finance/overlays/skills/manuscript-architect.md`
- Create: `subjects/finance/overlays/skills/study-designer.md`
- Create: `subjects/finance/overlays/skills/stats-engine.md`
- Create: `subjects/finance/venue-profiles/*.yaml`

- [x] Define `business` and `finance` in the catalog with `extends: core`, domain profiles, venue profiles, templates, overlays, and ordered skill groups.
- [x] Add subject-specific skills with YAML frontmatter and journal-publication quality bars.
- [x] Add overlays that make shared skills target undergraduate-plus usage while enforcing doctoral/journal-grade research standards.
- [x] Materialize payloads through the existing sync/audit flow.

### Task 3: Distribution And Release

**Files:**
- Modify: `scripts/build_plugin_artifacts.py`
- Modify: `scripts/release_postflight.sh`
- Modify: `scripts/validate_marketplace_install.py`
- Modify: `release/automation.md`
- Modify: install docs and npm help text.

- [x] Extend fallback artifact generation for `business` and `finance`.
- [x] Include both subjects in Codex and Claude marketplace artifacts.
- [x] Include both subjects in Claude Desktop ZIP artifacts.
- [x] Update validation scripts to require and inspect the new artifacts.
- [x] Sync versions to `0.13.0` and run release readiness checks.

### Task 4: Skillsplace Metadata

**Files:**
- Modify: `/Users/pengjiaxin/Work/utility/cli-tools/skillsplace/marketplace.json`
- Modify: `/Users/pengjiaxin/Work/utility/cli-tools/skillsplace/.agents/plugins/marketplace.json`
- Modify: `/Users/pengjiaxin/Work/utility/cli-tools/skillsplace/.claude-plugin/marketplace.json`
- Modify: `/Users/pengjiaxin/Work/utility/cli-tools/skillsplace/.antigravity/catalog.json`
- Modify: `/Users/pengjiaxin/Work/utility/cli-tools/skillsplace/tests/*.test.mjs`

- [x] Update existing Qiongli entries from `0.12.1` to `0.13.0`.
- [x] Add `qiongli-business` and `qiongli-finance` Codex and Claude entries pointing to `v0.13.0` release assets.
- [x] Keep Antigravity to the base `qiongli` entry unless native subject entries are added later.
- [x] Run `npm run validate` in Skillsplace.

### Task 5: Verification And Stable Release

**Files:**
- Repository state in `research-skills` and `skillsplace`.

- [x] Run focused Python unit tests for subject catalog, materialization, artifacts, and release automation.
- [x] Run npm package tests.
- [x] Run repository validation.
- [ ] Stage and commit both repository changes.
- [ ] Publish stable `v0.13.0` with `scripts/release_automation.sh publish --tag v0.13.0 --from-tag v0.12.1`.
- [ ] Confirm release artifacts include business and finance Codex, Claude, and Claude Desktop ZIP files.
