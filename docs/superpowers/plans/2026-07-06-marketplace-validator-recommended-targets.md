# Marketplace Validator Recommended Targets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make marketplace artifact validation select platform targets by registry `release_download.recommended_key` metadata instead of fixed target IDs.

**Architecture:** `validate_marketplace_install.py` already loads the platform target registry before validating generated artifacts. Add a small lookup helper that requires exactly one target per recommended key and use it for Codex, Claude Code, Claude Desktop direct plugin, and Desktop/Web skill ZIP validation surfaces.

**Tech Stack:** Python standard library, existing `qiongli.platform_targets.PlatformTarget`, `unittest`.

---

## Files

- Modify: `tests/test_plugin_distribution_contract.py`
  - Add a helper-level test proving marketplace validator target selection follows `release_download.recommended_key`.
- Modify: `tooling/scripts/validate_marketplace_install.py`
  - Add `_target_by_recommended_key()`.
  - Use it in `validate()` for the platform target records used by artifact validation.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record marketplace validator target selection as registry-keyed.

### Task 1: Add RED Recommended-Key Test

- [x] **Step 1: Add test**

Add to `PluginDistributionContractTests`:

```python
    def test_marketplace_validator_selects_target_by_recommended_key(self) -> None:
        from qiongli.platform_targets import load_platform_targets

        codex_target = load_platform_targets(REPO_ROOT)["codex-marketplace-plugin"]
        fake_target = replace(codex_target, target_id="fixture-codex-target")

        selected = validator._target_by_recommended_key(
            {"fixture-codex-target": fake_target},
            "codex",
        )

        self.assertEqual(selected.target_id, "fixture-codex-target")
```

- [x] **Step 2: Run RED**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_selects_target_by_recommended_key -q
```

Expected: FAIL because `_target_by_recommended_key` does not exist.

### Task 2: Implement Registry-Keyed Selection

- [x] **Step 1: Add helper**

Add to `tooling/scripts/validate_marketplace_install.py`:

```python
def _target_by_recommended_key(targets: dict[str, PlatformTarget], recommended_key: str) -> PlatformTarget:
    matches = sorted(
        target
        for target in targets.values()
        if target.release_download.get("recommended_key") == recommended_key
    )
    if len(matches) != 1:
        raise ValueError(
            "platform target registry must define exactly one "
            f"release_download.recommended_key={recommended_key!r}; found {len(matches)}"
        )
    return matches[0]
```

- [x] **Step 2: Use helper in `validate()`**

Replace fixed-ID calls:

```python
marketplace_specs = {
    "codex": _target_by_recommended_key(targets, "codex"),
    "claude": _target_by_recommended_key(targets, "claude_code"),
}
direct_desktop_target = _target_by_recommended_key(targets, "claude_desktop_plugin")
desktop_skill_target = _target_by_recommended_key(targets, "claude_desktop_skill")
```

- [x] **Step 3: Run GREEN**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract.PluginDistributionContractTests.test_marketplace_validator_selects_target_by_recommended_key -q
```

Expected: OK.

### Task 3: Update Roadmap And Verify

- [x] **Step 1: Update roadmap**

Record that marketplace artifact validation target selection now uses registry `recommended_key` metadata.

- [x] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_plugin_distribution_contract -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/validate_marketplace_install.py tests/test_plugin_distribution_contract.py
git commit -m "feat(release): select marketplace targets by recommended key"
git add docs/superpowers/plans/2026-07-06-marketplace-validator-recommended-targets.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record marketplace validator target lookup"
```
