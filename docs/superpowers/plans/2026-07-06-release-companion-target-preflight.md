# Release Companion Target Preflight Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release preflight validate both the platform target registry and the release companion target registry.

**Architecture:** Keep the existing `scripts/validate_platform_targets.py --root "$PREFLIGHT_ROOT"` gate, but make the underlying tooling script validate both registries. This preserves the existing preflight wiring while adding coverage for the new `content/distribution/release-companion-targets.yaml` source of truth.

**Tech Stack:** Python standard library, PyYAML, Bash release preflight, `unittest`.

---

### Task 1: Add RED Preflight Tests

**Files:**
- Modify: `tests/test_release_downloads.py`
- Modify: `tests/test_release_automation.py`

- [ ] **Step 1: Add a validator loader helper**

```python
def _load_validate_platform_targets_module():
    spec = importlib.util.spec_from_file_location(
        "validate_platform_targets",
        REPO_ROOT / "tooling" / "scripts" / "validate_platform_targets.py",
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module
```

- [ ] **Step 2: Assert companion registry failures are included**

```python
def test_release_target_registry_validator_reports_companion_failures(self) -> None:
    module = _load_validate_platform_targets_module()

    with tempfile.TemporaryDirectory() as tmp_dir:
        root = Path(tmp_dir)
        self._write_valid_platform_registry(root)
        companion = root / "content" / "distribution" / "release-companion-targets.yaml"
        companion.write_text(
            json.dumps(
                {
                    "schema_version": "1.0",
                    "targets": {
                        "fixture_asset": {
                            "target_id": "fixture-target",
                            "subject": "fixture",
                            "artifact_kind": "fixture",
                        }
                    },
                }
            ),
            encoding="utf-8",
        )

        failures = module.validate_release_target_registries(root)

    self.assertTrue(any("release companion target registry" in failure for failure in failures), failures)
    self.assertTrue(any("expected_install_method" in failure for failure in failures), failures)
```

- [ ] **Step 3: Assert release preflight label covers both registries**

```python
self.assertIn('echo "[preflight] release target registries schema"', content)
self.assertNotIn('echo "[preflight] platform target registry schema"', content)
```

- [ ] **Step 4: Run RED**

```bash
.venv/bin/python -m unittest \
  tests.test_release_downloads.ReleaseDownloadsTests.test_release_target_registry_validator_reports_companion_failures \
  tests.test_release_automation.ReleaseAutomationTests.test_release_preflight_validates_platform_target_registry_before_standard_validator \
  -q
```

Expected: FAIL because the combined validator helper does not exist and preflight still prints the old platform-only label.

### Task 2: Implement Combined Registry Validation

**Files:**
- Modify: `tooling/scripts/validate_platform_targets.py`
- Modify: `tooling/scripts/release_preflight.sh`

- [ ] **Step 1: Import the companion loader**

```python
SCRIPT_DIR = Path(__file__).resolve().parent
for import_root in (SCRIPT_DIR, PYTHON_SOURCE_ROOT, REPO_ROOT):
    ...
from generate_release_downloads import load_companion_targets
```

- [ ] **Step 2: Add combined validation helper**

```python
def validate_release_target_registries(root: Path) -> list[str]:
    failures = [
        f"platform target registry: {failure}"
        for failure in validate_platform_target_registry(root)
    ]
    try:
        load_companion_targets(root)
    except ValueError as exc:
        failures.append(f"release companion target registry: {exc}")
    return failures
```

- [ ] **Step 3: Update CLI output**

```python
failures = validate_release_target_registries(args.root)
...
print("[OK] release target registries schema valid")
```

- [ ] **Step 4: Update preflight label**

```bash
echo "[preflight] release target registries schema"
python3 scripts/validate_platform_targets.py --root "$PREFLIGHT_ROOT"
```

### Task 3: Update Docs And Verify

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record companion preflight validation**

Update Stage 12 status so release preflight validates both platform target and release companion target registries.

- [ ] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_release_downloads tests.test_release_automation -q
.venv/bin/python tooling/scripts/validate_platform_targets.py
git diff --check
```

- [ ] **Step 3: Commit by category**

```bash
git add tooling/scripts/validate_platform_targets.py tooling/scripts/release_preflight.sh tests/test_release_downloads.py tests/test_release_automation.py
git commit -m "feat(release): validate companion target registry in preflight"
git add docs/superpowers/plans/2026-07-06-release-companion-target-preflight.md \
  docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record companion target preflight"
```
