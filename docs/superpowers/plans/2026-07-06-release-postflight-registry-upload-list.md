# Release Postflight Registry Upload List Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make release postflight upload artifacts derive from the registry-backed release download index instead of maintaining a separate hardcoded shell list.

**Architecture:** Add a small Python helper that builds the same release download index as `generate_release_downloads.py`, flattens `assets_by_target` from `content/distribution/platform-targets.yaml`, appends release companion assets, verifies files exist in `dist`, and prints one upload path per line. Change `release_postflight.sh` to build artifacts, generate downloads, then read `PLUGIN_ARTIFACTS` from the helper.

**Tech Stack:** Python stdlib, existing `qiongli.platform_targets`, existing release download generator, Bash `mapfile`.

---

## Files

- Create: `tooling/scripts/release_upload_assets.py`
  - Build ordered upload asset names from `generate_release_downloads.build_index`.
  - Flatten target registry `assets_by_target`.
  - Append non-target companion assets: MCPB, Zotero companion, download guide, download index.
  - Verify asset files under `dist` by default.
- Create: `scripts/release_upload_assets.py`
  - Thin wrapper following the existing `scripts/*.py` tooling-wrapper pattern.
- Create: `tests/test_release_upload_assets.py`
  - Test stable and prerelease upload lists are registry-derived and de-duplicated.
  - Test missing files fail with useful diagnostics.
- Modify: `tooling/scripts/release_postflight.sh`
  - Remove the stable/prerelease hardcoded `PLUGIN_ARTIFACTS=(...)` blocks.
  - Run build helpers, generate downloads, then populate `PLUGIN_ARTIFACTS` from `scripts/release_upload_assets.py`.
- Modify: `tests/test_release_automation.py`
  - Replace shell hardcoded artifact assertions with contract checks that postflight calls the helper and does not maintain the old static arrays.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark release postflight as connected to registry metadata, leaving local installers and npm-lite installation as Stage 12 follow-up.

## Task 1: Add Failing Upload Helper Tests

- [x] **Step 1: Add stable/prerelease list tests**

Create `tests/test_release_upload_assets.py`:

```python
from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = REPO_ROOT / "tooling" / "scripts" / "release_upload_assets.py"


def load_release_upload_assets():
    spec = importlib.util.spec_from_file_location("release_upload_assets", SCRIPT_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Unable to load {SCRIPT_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class ReleaseUploadAssetsTests(unittest.TestCase):
    def test_stable_upload_assets_are_derived_from_registry_targets(self) -> None:
        module = load_release_upload_assets()
        names = module.release_upload_asset_names(
            "v1.6.0",
            root=REPO_ROOT,
            require_existing=False,
        )

        self.assertIn("qiongli-codex-plugin-v1.6.0.tar.gz", names)
        self.assertIn("qiongli-core-codex-plugin-v1.6.0.tar.gz", names)
        self.assertIn("qiongli-claude-plugin-v1.6.0.tar.gz", names)
        self.assertIn("qiongli-claude-plugin-v1.6.0.zip", names)
        self.assertIn("qiongli-claude-desktop-plugin-v1.6.0.zip", names)
        self.assertIn("qiongli-claude-desktop-skill-core-v1.6.0.zip", names)
        self.assertIn("qiongli-downloads-v1.6.0.md", names)
        self.assertIn("qiongli-downloads-v1.6.0.json", names)
        self.assertEqual(len(names), len(set(names)))
        self.assertLess(
            names.index("qiongli-codex-plugin-v1.6.0.tar.gz"),
            names.index("qiongli-claude-plugin-v1.6.0.tar.gz"),
        )

    def test_prerelease_upload_assets_are_next_channel_only(self) -> None:
        module = load_release_upload_assets()

        names = module.release_upload_asset_names(
            "v1.6.0-beta.1",
            root=REPO_ROOT,
            require_existing=False,
        )

        self.assertIn("qiongli-next-codex-plugin-v1.6.0-beta.1.tar.gz", names)
        self.assertIn("qiongli-next-claude-plugin-v1.6.0-beta.1.tar.gz", names)
        self.assertIn("qiongli-next-claude-plugin-v1.6.0-beta.1.zip", names)
        self.assertIn("qiongli-next-claude-desktop-plugin-v1.6.0-beta.1.zip", names)
        self.assertIn("qiongli-next-claude-desktop-skill-core-v1.6.0-beta.1.zip", names)
        self.assertNotIn("qiongli-finance-codex-plugin-v1.6.0-beta.1.tar.gz", names)

    def test_missing_upload_asset_paths_fail_when_required(self) -> None:
        module = load_release_upload_assets()
        with tempfile.TemporaryDirectory() as tmp_dir:
            dist_dir = Path(tmp_dir)
            with self.assertRaisesRegex(module.ReleaseUploadAssetError, "missing upload assets"):
                module.release_upload_asset_paths(
                    "v1.6.0-beta.1",
                    dist_dir=dist_dir,
                    root=REPO_ROOT,
                    require_existing=True,
                )


if __name__ == "__main__":
    unittest.main()
```

- [x] **Step 2: Add postflight static contract test**

Update `tests/test_release_automation.py` in the postflight test:

```python
self.assertIn('UPLOAD_ASSETS_FILE=""', content)
self.assertIn('python3 scripts/release_upload_assets.py --tag "$TAG" --dist-dir dist >"$UPLOAD_ASSETS_FILE"', content)
self.assertIn('mapfile -t PLUGIN_ARTIFACTS <"$UPLOAD_ASSETS_FILE"', content)
self.assertNotIn('if [[ "${TAG#v}" == *-* ]]; then\n  PLUGIN_ARTIFACTS=(', content)
self.assertNotIn('"dist/qiongli-core-codex-plugin-${TAG}.tar.gz"', content)
```

- [x] **Step 3: Run tests and verify RED**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_upload_assets tests.test_release_automation.ReleaseAutomationTests.test_release_postflight_waits_for_branch_and_tag_workflows -q
```

Expected: FAIL because `release_upload_assets.py` does not exist and
postflight still maintains hardcoded arrays.

## Task 2: Implement Registry-Derived Upload Helper

- [x] **Step 1: Add helper implementation**

Create `tooling/scripts/release_upload_assets.py`:

```python
#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
PYTHON_SOURCE_ROOT = REPO_ROOT / "packages" / "python-qiongli" / "src"
for import_root in (Path(__file__).resolve().parent, PYTHON_SOURCE_ROOT, REPO_ROOT):
    if str(import_root) not in sys.path:
        sys.path.insert(0, str(import_root))

from generate_release_downloads import build_index  # noqa: E402


EXTRA_UPLOAD_ASSET_KEYS = (
    "claude_desktop_literature_mcpb",
    "zotero_desktop_companion",
    "download_guide",
    "download_index",
)


class ReleaseUploadAssetError(RuntimeError):
    pass


def release_upload_asset_names(
    tag: str,
    *,
    root: Path = REPO_ROOT,
    require_existing: bool = True,
    dist_dir: Path | None = None,
) -> list[str]:
    index = build_index(tag, root=root)
    names: list[str] = []
    for target_assets in index.get("assets_by_target", {}).values():
        if isinstance(target_assets, dict):
            for value in target_assets.values():
                _append_asset_value(names, value)
    assets = index.get("assets", {})
    if isinstance(assets, dict):
        for key in EXTRA_UPLOAD_ASSET_KEYS:
            _append_asset_value(names, assets.get(key))
    unique_names = _dedupe(names)
    if require_existing:
        if dist_dir is None:
            raise ReleaseUploadAssetError("dist_dir is required when require_existing is true")
        _require_existing(unique_names, dist_dir)
    return unique_names


def release_upload_asset_paths(
    tag: str,
    *,
    dist_dir: Path,
    root: Path = REPO_ROOT,
    require_existing: bool = True,
) -> list[str]:
    names = release_upload_asset_names(
        tag,
        root=root,
        require_existing=require_existing,
        dist_dir=dist_dir,
    )
    return [str(Path(dist_dir) / name) for name in names]


def _append_asset_value(names: list[str], value: Any) -> None:
    if isinstance(value, str) and value:
        names.append(value)
    elif isinstance(value, list):
        names.extend(item for item in value if isinstance(item, str) and item)


def _dedupe(values: list[str]) -> list[str]:
    seen: set[str] = set()
    deduped: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        deduped.append(value)
    return deduped


def _require_existing(names: list[str], dist_dir: Path) -> None:
    missing = [name for name in names if not (dist_dir / name).is_file()]
    if missing:
        preview = ", ".join(missing[:8])
        if len(missing) > 8:
            preview += f", ... (+{len(missing) - 8} more)"
        raise ReleaseUploadAssetError(f"missing upload assets: {preview}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="List Qiongli release upload asset paths.")
    parser.add_argument("--tag", required=True)
    parser.add_argument("--dist-dir", type=Path, default=Path("dist"))
    parser.add_argument("--root", type=Path, default=REPO_ROOT)
    parser.add_argument("--no-require-existing", action="store_true")
    args = parser.parse_args(argv)
    try:
        paths = release_upload_asset_paths(
            args.tag,
            dist_dir=args.dist_dir,
            root=args.root,
            require_existing=not args.no_require_existing,
        )
    except ReleaseUploadAssetError as exc:
        print(f"[release-upload-assets] {exc}", file=sys.stderr)
        return 1
    for path in paths:
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

- [x] **Step 2: Add root wrapper**

Create `scripts/release_upload_assets.py` using the same wrapper pattern as
`scripts/generate_release_downloads.py`.

- [x] **Step 3: Run upload helper tests and verify GREEN**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_upload_assets -q
```

Expected: PASS.

## Task 3: Wire Postflight To Helper

- [x] **Step 1: Replace hardcoded arrays**

In `tooling/scripts/release_postflight.sh`, replace both `PLUGIN_ARTIFACTS`
branches with:

```bash
python3 scripts/build_plugin_artifacts.py --root "$POSTFLIGHT_STAGING_DIR" --tag "$TAG" --dist-dir dist
python3 scripts/build_literature_mcpb.py --dist-dir dist >/dev/null
python3 scripts/build_zotero_companion.py --dist-dir dist >/dev/null
python3 scripts/generate_release_downloads.py --tag "$TAG" --out-dir dist
UPLOAD_ASSETS_FILE="$(mktemp -t qiongli-upload-assets.XXXXXX.txt)"
python3 scripts/release_upload_assets.py --tag "$TAG" --dist-dir dist >"$UPLOAD_ASSETS_FILE"
mapfile -t PLUGIN_ARTIFACTS <"$UPLOAD_ASSETS_FILE"
```

- [x] **Step 2: Update release automation tests**

Remove assertions that require explicit shell asset arrays and assert the helper
call instead.

- [x] **Step 3: Run focused tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_release_upload_assets tests.test_release_automation -q
```

Expected: PASS.

## Task 4: Verify And Document

- [x] **Step 1: Run helper smoke with generated dummy files**

Run:

```bash
python3 scripts/release_upload_assets.py --tag v1.1.0-beta.2 --dist-dir /private/tmp/qiongli-upload-assets-smoke --no-require-existing
```

Expected: prints one upload path per line, including next plugin, MCPB, Zotero,
download guide, and download index assets.

- [x] **Step 2: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [x] **Step 3: Update roadmap**

Update Stage 12 status to say release postflight upload assets now come from
the registry-backed release download index, leaving local plugin installers and
npm plugin-lite installation as follow-up.

- [x] **Step 4: Commit by content**

Implementation:

```bash
git add tooling/scripts/release_upload_assets.py scripts/release_upload_assets.py tests/test_release_upload_assets.py tests/test_release_automation.py tooling/scripts/release_postflight.sh
git commit -m "feat(release): derive postflight uploads from target registry"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-release-postflight-registry-upload-list.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record registry-backed postflight uploads"
```

## Self-Review

- Spec coverage: Covers the Stage 12 release postflight alignment item without
  changing local installers or npm-lite installation.
- Placeholder scan: No placeholders remain.
- Type consistency: Tests and implementation use `release_upload_asset_names`,
  `release_upload_asset_paths`, and `ReleaseUploadAssetError` consistently.
