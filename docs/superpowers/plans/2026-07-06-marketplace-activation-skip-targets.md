# Marketplace Activation Skip Targets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make marketplace structural validation's client activation skip evidence list the platform target IDs selected from registry smoke policy.

**Architecture:** `validate_marketplace_install.py` already loads `content/distribution/platform-targets.yaml` for artifact validation. Reuse that loaded target map to derive every target whose `smoke.client_activation_check` is `local_install_acceptance`, and print those target IDs in the final skip line that delegates activation to `scripts/release_local_install_check.py`.

**Tech Stack:** Python standard library, existing platform target registry, `unittest` integration test.

---

## Files

- Modify: `tooling/scripts/validate_marketplace_install.py`
  - Add helper for registry-selected client activation target IDs.
  - Include those IDs in the final `[SKIP]` line.
- Modify: `tests/test_plugin_distribution_contract.py`
  - Strengthen the marketplace validator stdout assertion to require the registry-selected target IDs.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record that marketplace validation's skipped client activation evidence is target-specific and registry-backed.

### Task 1: Add RED Output Assertion

- [x] **Step 1: Strengthen marketplace validator test**

In `test_marketplace_validator_builds_platform_artifacts_and_checks_invocation`, replace the generic skip assertion with:

```python
self.assertIn(
    "[SKIP] client CLI activation checks skipped for targets: "
    "antigravity-local-plugin, claude-code-marketplace-plugin, codex-marketplace-plugin; "
    "run scripts/release_local_install_check.py",
    result.stdout,
)
```

- [x] **Step 2: Run RED**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_builds_platform_artifacts_and_checks_invocation -q
```

Expected: FAIL because the current output does not list target IDs.

### Task 2: Implement Registry-Backed Skip Line

- [x] **Step 1: Add helper**

In `tooling/scripts/validate_marketplace_install.py`, add:

```python
def _client_activation_target_ids(targets: dict[str, PlatformTarget]) -> list[str]:
    return sorted(
        target_id
        for target_id, target in targets.items()
        if target.smoke.get("client_activation_check") == "local_install_acceptance"
    )
```

- [x] **Step 2: Return activation target IDs from validation**

Keep artifact validation output as the existing list of messages, but make the CLI compute activation target IDs from the same loaded registry before printing the final skip line.

- [x] **Step 3: Print target-specific skip evidence**

In `main()`, replace:

```python
print("[SKIP] client CLI activation checks skipped; run scripts/release_local_install_check.py")
```

with:

```python
targets = ", ".join(_client_activation_target_ids(load_platform_targets(args.root)))
print(
    "[SKIP] client CLI activation checks skipped for targets: "
    f"{targets}; run scripts/release_local_install_check.py"
)
```

- [x] **Step 4: Run GREEN**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_builds_platform_artifacts_and_checks_invocation -q
```

Expected: OK.

### Task 3: Update Roadmap And Verify

- [x] **Step 1: Update Stage 12 wording**

Record that marketplace validation's structural/client split now names the registry-selected client activation targets.

- [x] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/validate_marketplace_install.py tests/test_plugin_distribution_contract.py
git commit -m "feat(release): report activation skip target ids"
git add docs/superpowers/plans/2026-07-06-marketplace-activation-skip-targets.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record marketplace activation skip targets"
```
