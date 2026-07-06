# Structural Vs Client Activation Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release artifact validation explicitly separate cheap structural archive checks from client CLI activation checks.

**Architecture:** Keep `validate_marketplace_install.py` as the structural artifact validator: it builds archives, extracts them, checks required/forbidden paths, manifests, skill invocation text, bundled MCP files, and target IDs. Add explicit stdout evidence that client CLI activation is intentionally skipped there and delegated to `scripts/release_local_install_check.py`, which release readiness already runs later against the staged release root.

**Tech Stack:** Python stdlib, existing marketplace validator, existing `unittest` distribution contract coverage.

---

## Files

- Modify: `tooling/scripts/validate_marketplace_install.py`
  - Print a structural-check completion line.
  - Print an explicit client CLI activation skip line pointing to `scripts/release_local_install_check.py`.
- Modify: `tests/test_plugin_distribution_contract.py`
  - Assert marketplace validator output includes both lines.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that Stage 12 structural artifact validation now makes skipped client activation explicit.

## Task 1: Add Failing Output Contract Test

- [x] **Step 1: Extend marketplace validator test**

In `tests/test_plugin_distribution_contract.py`,
`test_marketplace_validator_builds_platform_artifacts_and_checks_invocation`,
assert:

```python
self.assertIn("[OK] structural archive checks completed", result.stdout)
self.assertIn(
    "[SKIP] client CLI activation checks skipped; run scripts/release_local_install_check.py",
    result.stdout,
)
```

- [x] **Step 2: Run RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_builds_platform_artifacts_and_checks_invocation -q
```

Expected: FAIL because the validator currently ends with a generic marketplace
validation completion line and does not mention skipped client activation.

## Task 2: Implement Explicit Validation Layers

- [x] **Step 1: Update validator output**

In `tooling/scripts/validate_marketplace_install.py`, replace:

```python
print("[OK] marketplace validation completed")
```

with:

```python
print("[OK] structural archive checks completed")
print("[SKIP] client CLI activation checks skipped; run scripts/release_local_install_check.py")
```

- [x] **Step 2: Run GREEN**

Run the focused test from Task 1. Expected: PASS.

## Task 3: Verify, Document, Commit

- [x] **Step 1: Run related tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract tests.test_release_automation -q
```

- [x] **Step 2: Run validator smoke**

Run:

```bash
.venv/bin/python scripts/validate_marketplace_install.py --dist-dir /private/tmp/qiongli-structural-validator-smoke
```

- [x] **Step 3: Update roadmap**

Update Stage 12 status/backlog to record explicit structural-vs-client activation
validation output.

- [x] **Step 4: Commit by content**

Implementation and tests:

```bash
git add tooling/scripts/validate_marketplace_install.py tests/test_plugin_distribution_contract.py
git commit -m "feat(release): mark structural validator scope"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-structural-vs-client-activation-validation.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record structural validation scope"
```

## Self-Review

- Spec coverage: Covers the Stage 12 backlog item about keeping platform smoke
  tests cheap by separating structural archive checks from optional client CLI
  activation checks.
- Placeholder scan: No placeholders remain.
- Type consistency: Output strings are plain stdout contract lines asserted by
  the existing distribution contract test.
