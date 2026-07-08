# Npm Installer Recommended Target Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the npm plugin-lite installer select its platform target record by registry `release_download.recommended_key=qiongli_cli` instead of the fixed `npm-plugin-lite` target key.

**Architecture:** Keep npm marker shape unchanged by continuing to write only the normalized platform target marker fields. Change the registry lookup inside `readNpmPluginPlatformTarget()` so future registry target ID renames do not break npm-managed plugin marker metadata.

**Tech Stack:** Node.js `node:test`, npm package installer, Qiongli platform target JSON payload.

---

### Task 1: Add Recommended-Key Regression Test

**Files:**
- Modify: `packages/npm-qiongli/test/installer.test.mjs`

- [ ] **Step 1: Write the failing test**

Update `writePlatformTargetRegistry()` so fixture target entries include release-download metadata and can be written under a custom registry key:

```javascript
function writePlatformTargetRegistry(root, overrides = {}, { targetKey = 'npm-plugin-lite' } = {}) {
  const registry = path.join(root, 'payload', 'content', 'distribution');
  fs.mkdirSync(registry, { recursive: true });
  const target = {
    ...EXPECTED_NPM_PLATFORM_TARGET,
    release_download: {
      guide_label: 'Qiongli npm/npx CLI',
      recommended_key: 'qiongli_cli',
      asset_groups: [],
    },
    ...overrides,
  };
  fs.writeFileSync(
    path.join(registry, 'platform-targets.json'),
    `${JSON.stringify({ schema_version: '1.0', targets: { [targetKey]: target } }, null, 2)}\n`,
  );
}
```

Add this test after `installSkills uses bundled npm platform target registry values`:

```javascript
test('installSkills selects npm platform target by registry recommended key', () => {
  const { root } = makeTempPackage();
  writePlatformTargetRegistry(
    root,
    {
      target_id: 'fixture-npm-target',
      artifact_kind: 'fixture-package',
      archive_format: 'fixture-tarball',
      bundled_mcp_mode: 'fixture-none',
      command_surface: 'fixture-cli',
      validator: 'fixture-validator',
    },
    { targetKey: 'fixture-npm-target' },
  );
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'qiongli-plugin-recommended-target-home-'));

  installSkills({
    packageRoot: root,
    target: 'codex',
    surface: 'plugin',
    env: { HOME: home },
    platform: 'linux',
  });

  const pluginDest = path.join(home, 'plugins', 'qiongli');
  const marker = JSON.parse(fs.readFileSync(npmPluginMarker(pluginDest), 'utf-8'));

  assert.equal(marker.platform_target.target_id, 'fixture-npm-target');
  assert.equal(marker.platform_target.validator, 'fixture-validator');
});
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
node --test packages/npm-qiongli/test/installer.test.mjs --test-name-pattern "recommended key"
```

Expected: FAIL with `Missing platform target metadata: npm-plugin-lite`, proving npm lookup still requires the fixed target key.

### Task 2: Resolve Npm Target By Recommended Key

**Files:**
- Modify: `packages/npm-qiongli/lib/installer.mjs`

- [ ] **Step 1: Write minimal implementation**

Replace the fixed target ID constant with:

```javascript
const NPM_PLUGIN_RECOMMENDED_KEY = 'qiongli_cli';
```

Update target loading:

```javascript
function readNpmPluginPlatformTarget(packageRoot) {
  const registryPath = path.join(packageRoot, 'payload', 'content', 'distribution', 'platform-targets.json');
  const registry = JSON.parse(fs.readFileSync(registryPath, 'utf-8'));
  return platformTargetMarker(platformTargetByRecommendedKey(registry, NPM_PLUGIN_RECOMMENDED_KEY));
}

function platformTargetByRecommendedKey(registry, recommendedKey) {
  const targets = registry?.targets;
  if (!targets || typeof targets !== 'object') {
    throw new Error('Missing platform target registry targets');
  }
  const matches = Object.values(targets).filter(
    (target) => target?.release_download?.recommended_key === recommendedKey,
  );
  if (matches.length !== 1) {
    throw new Error(
      `Platform target registry must define exactly one release_download.recommended_key=${JSON.stringify(recommendedKey)}; found ${matches.length}`,
    );
  }
  return matches[0];
}
```

Update invalid-field diagnostics to reference the actual selected target ID:

```javascript
function requiredPlatformTargetString(target, field) {
  if (typeof target[field] !== 'string' || !target[field]) {
    const targetId = typeof target.target_id === 'string' && target.target_id ? target.target_id : '<unknown>';
    throw new Error(`Invalid platform target metadata field: ${targetId}.${field}`);
  }
  return target[field];
}
```

- [ ] **Step 2: Run test to verify it passes**

Run:

```bash
node --test packages/npm-qiongli/test/installer.test.mjs --test-name-pattern "recommended key"
```

Expected: PASS.

### Task 3: Update Roadmap Status

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Record the completed slice**

Update Stage 12 and the remaining product-gap summary to say npm plugin-lite target selection follows registry `release_download.recommended_key=qiongli_cli`. Keep marker/status-output wording intact.

### Task 4: Verify And Commit

**Files:**
- Test: `packages/npm-qiongli/test/installer.test.mjs`
- Test: `packages/npm-qiongli/test/cli.test.mjs`
- Test: `tooling/scripts/validate_platform_targets.py`

- [ ] **Step 1: Run npm tests**

```bash
node --test packages/npm-qiongli/test/installer.test.mjs packages/npm-qiongli/test/cli.test.mjs
```

Expected: all tests pass.

- [ ] **Step 2: Run registry validator**

```bash
.venv/bin/python tooling/scripts/validate_platform_targets.py
```

Expected: no validation errors.

- [ ] **Step 3: Run diff hygiene check**

```bash
git diff --check
```

Expected: no whitespace errors.

- [ ] **Step 4: Run boundary scan**

```bash
rg -n "(/[U]sers/|/[p]rivate/|BEGI[N] (RSA|OPENSSH|EC|DSA) PRIVATE KEY|secre[t]:|toke[n]:|passwor[d]:)" docs/superpowers/plans/2026-07-06-npm-installer-recommended-target.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md packages/npm-qiongli/lib/installer.mjs packages/npm-qiongli/test/installer.test.mjs
```

Expected: no matches.

- [ ] **Step 5: Commit implementation**

```bash
git add packages/npm-qiongli/lib/installer.mjs packages/npm-qiongli/test/installer.test.mjs
git commit -m "feat(npm): select plugin target by recommended key"
```

- [ ] **Step 6: Commit roadmap update**

```bash
git add docs/superpowers/plans/2026-07-06-npm-installer-recommended-target.md docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md
git commit -m "docs(roadmap): record npm installer target lookup"
```
