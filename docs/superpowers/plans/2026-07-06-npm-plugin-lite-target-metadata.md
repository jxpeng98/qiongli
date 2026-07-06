# npm Plugin Lite Target Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect npm plugin-lite installation to the canonical platform target registry metadata used by the rest of Stage 12.

**Architecture:** Keep the npm package dependency-free by generating a structured JSON target registry into `payload/content/distribution/platform-targets.json` from canonical `content/distribution/platform-targets.yaml` during distribution materialization. The npm installer reads that JSON, records the `npm-plugin-lite` target metadata in `.qiongli-npm-lite.json`, and exposes it through `buildCheck()`.

**Tech Stack:** Node.js stdlib, Python materializer scripts, existing `node:test` and `unittest` package contract tests.

---

## Files

- Modify: `packages/npm-qiongli/lib/installer.mjs`
  - Load `payload/content/distribution/platform-targets.json`.
  - Require the `npm-plugin-lite` target.
  - Write compact platform target metadata into npm plugin-lite markers.
  - Report marker platform target metadata from `buildCheck()`.
- Modify: `packages/npm-qiongli/test/installer.test.mjs`
  - Add failing tests for registry-derived marker metadata.
  - Add failing tests for `buildCheck()` plugin metadata reporting.
  - Update existing exact marker assertions.
- Modify: `packages/npm-qiongli/test/cli.test.mjs`
  - Add the minimal target registry JSON to temporary CLI package roots.
- Modify: `tooling/scripts/sync_npm_package_payload.py`
  - Generate `payload/content/distribution/platform-targets.json` from canonical YAML using `qiongli.platform_targets`.
- Modify: `tooling/scripts/audit_distribution_payloads.py`
  - Verify generated npm platform target JSON matches canonical registry metadata.
- Modify: `tests/test_npm_package_contract.py`
  - Assert staged npm payload contains the generated platform target registry and `npm-plugin-lite` target.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Mark Stage 12 npm plugin-lite metadata alignment complete.

## Task 1: Add Failing npm Installer Tests

- [x] **Step 1: Add test helper registry writer**

In `packages/npm-qiongli/test/installer.test.mjs`, add:

```js
function writePlatformTargetRegistry(root, overrides = {}) {
  const registry = path.join(root, 'payload', 'content', 'distribution');
  fs.mkdirSync(registry, { recursive: true });
  const target = {
    target_id: 'npm-plugin-lite',
    artifact_kind: 'npm-package',
    archive_format: 'npm-tarball',
    bundled_mcp_mode: 'none',
    command_surface: 'npx-cli',
    validator: 'npm-plugin-lite',
    ...overrides,
  };
  fs.writeFileSync(
    path.join(registry, 'platform-targets.json'),
    `${JSON.stringify({ schema_version: '1.0', targets: { 'npm-plugin-lite': target } }, null, 2)}\n`,
  );
}
```

Call `writePlatformTargetRegistry(root)` inside `makeTempPackage()`.

- [x] **Step 2: Add marker metadata test**

Add:

```js
test('installSkills records npm plugin-lite platform target metadata in marker', () => {
  const { root } = makeTempPackage();
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-target-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const marker = JSON.parse(fs.readFileSync(npmPluginMarker(pluginDest), 'utf-8'));

  assert.deepEqual(marker.platform_target, {
    target_id: 'npm-plugin-lite',
    artifact_kind: 'npm-package',
    archive_format: 'npm-tarball',
    bundled_mcp_mode: 'none',
    command_surface: 'npx-cli',
    validator: 'npm-plugin-lite',
  });
});
```

- [x] **Step 3: Add fake registry proof test**

Add:

```js
test('installSkills uses bundled npm platform target registry values', () => {
  const { root } = makeTempPackage();
  writePlatformTargetRegistry(root, {
    target_id: 'fake-npm-plugin-lite',
    artifact_kind: 'fake-package',
    archive_format: 'fake-tarball',
    bundled_mcp_mode: 'fake-none',
    command_surface: 'fake-cli',
    validator: 'fake-validator',
  });
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-fake-target-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const marker = JSON.parse(fs.readFileSync(npmPluginMarker(pluginDest), 'utf-8'));

  assert.equal(marker.platform_target.target_id, 'fake-npm-plugin-lite');
  assert.equal(marker.platform_target.validator, 'fake-validator');
});
```

- [x] **Step 4: Add `buildCheck()` metadata assertion**

In `buildCheck reports plugin-only installs`, add:

```js
  assert.deepEqual(result.installed.codex.plugin.platform_target, {
    target_id: 'npm-plugin-lite',
    artifact_kind: 'npm-package',
    archive_format: 'npm-tarball',
    bundled_mcp_mode: 'none',
    command_surface: 'npx-cli',
    validator: 'npm-plugin-lite',
  });
```

- [x] **Step 5: Run RED**

Run:

```bash
node --test packages/npm-qiongli/test/installer.test.mjs
```

Expected: FAIL because marker and `buildCheck()` do not include `platform_target`.

## Task 2: Implement npm Installer Metadata

- [x] **Step 1: Add target constants and metadata helpers**

In `packages/npm-qiongli/lib/installer.mjs`, add:

```js
const NPM_PLUGIN_TARGET_ID = 'npm-plugin-lite';

function readNpmPluginPlatformTarget(packageRoot) {
  const registryPath = path.join(packageRoot, 'payload', 'content', 'distribution', 'platform-targets.json');
  const registry = JSON.parse(fs.readFileSync(registryPath, 'utf-8'));
  const target = registry?.targets?.[NPM_PLUGIN_TARGET_ID];
  if (!target || typeof target !== 'object') {
    throw new Error(`Missing platform target metadata: ${NPM_PLUGIN_TARGET_ID}`);
  }
  return platformTargetMarker(target);
}

function platformTargetMarker(target) {
  return {
    target_id: requiredPlatformTargetString(target, 'target_id'),
    artifact_kind: requiredPlatformTargetString(target, 'artifact_kind'),
    archive_format: requiredPlatformTargetString(target, 'archive_format'),
    bundled_mcp_mode: requiredPlatformTargetString(target, 'bundled_mcp_mode'),
    command_surface: requiredPlatformTargetString(target, 'command_surface'),
    validator: requiredPlatformTargetString(target, 'validator'),
  };
}

function requiredPlatformTargetString(target, field) {
  if (typeof target[field] !== 'string' || !target[field]) {
    throw new Error(`Invalid platform target metadata field: ${NPM_PLUGIN_TARGET_ID}.${field}`);
  }
  return target[field];
}
```

- [x] **Step 2: Pass metadata through plugin copy**

In `installSkills()`, before `copyPlugin(...)`:

```js
const pluginPlatformTarget = readNpmPluginPlatformTarget(packageRoot);
```

Pass `platformTarget: pluginPlatformTarget` into `copyPlugin()`.

- [x] **Step 3: Persist metadata in markers**

Change `copyPlugin()` and `writeNpmPluginMarker()` to accept `platformTarget` and write:

```js
platform_target: platformTarget,
```

- [x] **Step 4: Report metadata in `buildCheck()`**

In the `plugin` object returned by `buildCheck()`, add:

```js
platform_target: pluginMarker?.platform_target || null,
```

- [x] **Step 5: Update exact marker assertion**

Update the existing plugin-only marker `deepEqual` assertion to include the
expected `platform_target` object.

- [x] **Step 6: Run GREEN**

Run:

```bash
node --test packages/npm-qiongli/test/installer.test.mjs
```

Expected: PASS.

## Task 3: Generate And Audit npm Target Registry Payload

- [x] **Step 1: Add materializer helper**

In `tooling/scripts/sync_npm_package_payload.py`, add a helper that writes
`payload/content/distribution/platform-targets.json`:

```python
def sync_npm_platform_target_registry(root: Path, payload_root: Path, *, dry_run: bool) -> None:
    from qiongli.platform_targets import load_platform_targets

    dest = payload_root / "content" / "distribution" / "platform-targets.json"
    if dry_run:
        print(f"[npm-sync] would sync platform target registry -> {dest}")
        return
    targets = load_platform_targets(root)
    payload = {
        "schema_version": "1.0",
        "targets": {
            target_id: {
                "target_id": target.target_id,
                "display_name": target.display_name,
                "artifact_kind": target.artifact_kind,
                "archive_format": target.archive_format,
                "source_inputs": list(target.source_inputs),
                "required_paths": list(target.required_paths),
                "allowed_wrapper_dirs": list(target.allowed_wrapper_dirs),
                "forbidden_paths": list(target.forbidden_paths),
                "bundled_mcp_mode": target.bundled_mcp_mode,
                "command_surface": target.command_surface,
                "validator": target.validator,
                "release_download": target.release_download,
            }
            for target_id, target in sorted(targets.items())
        },
    }
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
```

Call it from `sync_npm_payload(...)` after `payload_root` is initialized.

- [x] **Step 2: Add package contract assertion**

In `tests/test_npm_package_contract.py`, assert:

```python
target_registry = json.loads(
    (package_root / "payload" / "content" / "distribution" / "platform-targets.json").read_text(encoding="utf-8")
)
self.assertEqual(target_registry["targets"]["npm-plugin-lite"]["target_id"], "npm-plugin-lite")
self.assertEqual(target_registry["targets"]["npm-plugin-lite"]["command_surface"], "npx-cli")
```

- [x] **Step 3: Add audit comparison**

In `tooling/scripts/audit_distribution_payloads.py`, compare the generated npm
target registry JSON with `load_platform_targets(root)` and report a failure if
`npm-plugin-lite` or any core field drifts.

- [x] **Step 4: Run Python package contract RED/GREEN**

Run after adding tests before implementation to see RED, then after
implementation:

```bash
.venv/bin/python -m unittest tests.test_npm_package_contract -q
```

Expected final result: PASS.

## Task 4: Verify And Document

- [x] **Step 1: Run full npm tests**

Run:

```bash
node --test packages/npm-qiongli/test/*.test.mjs
```

Expected: PASS.

- [x] **Step 2: Run package contract and distribution audit tests**

Run:

```bash
.venv/bin/python -m unittest tests.test_npm_package_contract -q
```

Expected: PASS.

- [x] **Step 3: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: no output.

- [x] **Step 4: Update roadmap**

Update Stage 12 to say npm plugin-lite installation now records
registry-derived target metadata in npm-managed plugin markers and status
output, leaving Stage 12 optimization backlog items as non-blocking follow-ups.

- [x] **Step 5: Commit by content**

Implementation:

```bash
git add packages/npm-qiongli/lib/installer.mjs packages/npm-qiongli/test/installer.test.mjs packages/npm-qiongli/test/cli.test.mjs tooling/scripts/sync_npm_package_payload.py tooling/scripts/audit_distribution_payloads.py tests/test_npm_package_contract.py
git commit -m "feat(npm): record plugin lite target metadata"
```

Docs:

```bash
git add docs/superpowers/plans/2026-07-06-npm-plugin-lite-target-metadata.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record npm plugin lite metadata"
```

## Self-Review

- Spec coverage: Covers the remaining Stage 12 npm plugin-lite metadata follow-up.
- Placeholder scan: No TBD/TODO placeholders remain.
- Type consistency: Marker field is `platform_target`, matching Python local plugin markers; npm status output exposes the same object under `installed.<target>.plugin.platform_target`.
