# npm Lite Lifecycle Runtime Messaging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make npm-lite surfaces clearly explain that subject lifecycle controls and lifecycle MCP tools require the full Python runtime.

**Architecture:** Keep npm-lite as a Python-free asset manager and do not add subject lifecycle write behavior to Node. Add tests that enforce the help/README boundary language, then update npm CLI help and package README to point users to `pipx install qiongli` for `qiongli subject ...` and `qiongli_subject_update`.

**Tech Stack:** Node.js built-in `node:test`, npm package README, existing npm CLI help renderer.

---

### Task 1: Add Failing npm Help Test

**Files:**
- Modify: `packages/npm-qiongli/test/cli.test.mjs`

- [x] **Step 1: Extend help boundary assertions**

In `help describes the npm asset manager and full runtime boundary`, add:

```javascript
assert.match(stdout, /subject lifecycle controls require `pipx install qiongli`/);
assert.match(stdout, /qiongli_subject_update.*full runtime/);
```

- [x] **Step 2: Run the failing help test**

Run:

```bash
node --test packages/npm-qiongli/test/cli.test.mjs
```

Expected: FAIL because npm help does not yet mention subject lifecycle controls or `qiongli_subject_update`.

### Task 2: Add Failing README Test

**Files:**
- Modify: `packages/npm-qiongli/test/cli.test.mjs`

- [x] **Step 1: Add README boundary test**

Add:

```javascript
test('README documents subject lifecycle controls as full runtime only', () => {
  const readme = fs.readFileSync(new URL('../README.md', import.meta.url), 'utf-8');

  assert.match(readme, /qiongli subject confirm finance --cwd \./);
  assert.match(readme, /--propose-only --json/);
  assert.match(readme, /qiongli_subject_update.*read_only: true/);
  assert.match(readme, /full runtime.*pipx install qiongli/);
});
```

- [x] **Step 2: Run the failing README test**

Run:

```bash
node --test packages/npm-qiongli/test/cli.test.mjs
```

Expected: FAIL because the README does not yet document the full-runtime subject lifecycle boundary.

### Task 3: Update npm CLI Help

**Files:**
- Modify: `packages/npm-qiongli/lib/cli.mjs`

- [x] **Step 1: Add help text**

In `printHelp`, add lines explaining:

```text
Subject lifecycle controls require `pipx install qiongli`: use the full runtime for `qiongli subject ...` and MCP `qiongli_subject_update` / `qiongli_subject_status`.
```

- [x] **Step 2: Run help test**

Run:

```bash
node --test packages/npm-qiongli/test/cli.test.mjs
```

Expected: README test still FAILS until the README is updated.

### Task 4: Update npm README

**Files:**
- Modify: `packages/npm-qiongli/README.md`

- [x] **Step 1: Add subject lifecycle section**

Add a short section under the project guidance/update model explaining:

````markdown
## Subject lifecycle controls

The npm package can inspect and edit the lightweight `qiongli project ...`
manifest, but subject lifecycle controls live in the full Python runtime:

```bash
pipx install qiongli
qiongli subject confirm finance --cwd .
qiongli subject confirm finance --cwd . --propose-only --json
```

MCP clients that need `qiongli_subject_status` or `qiongli_subject_update`
should use the full runtime server. Read-only clients can call
`qiongli_subject_update` with `read_only: true` to export a proposed action
without writing `.qiongli` project files.
````

- [x] **Step 2: Run npm tests**

Run:

```bash
node --test packages/npm-qiongli/test/cli.test.mjs
```

Expected: PASS.

### Task 5: Update Roadmap And Commit

**Files:**
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-06-npm-lite-lifecycle-runtime-messaging.md`

- [x] **Step 1: Update Stage 6 status**

Extend Stage 6 status to include npm-lite lifecycle/full-runtime messaging.

- [x] **Step 2: Mark this plan complete**

Change checklist items in this plan to `[x]` after tests pass.

- [x] **Step 3: Verify**

Run:

```bash
node --test packages/npm-qiongli/test/cli.test.mjs
git diff --check
```

Expected: PASS and no whitespace errors.

- [x] **Step 4: Commit implementation/docs**

Run:

```bash
git add packages/npm-qiongli/lib/cli.mjs packages/npm-qiongli/README.md packages/npm-qiongli/test/cli.test.mjs
git commit -m "docs(npm): clarify lifecycle runtime boundary" -m "Explain that subject lifecycle controls and lifecycle MCP tools require the full Python runtime from npm-lite surfaces."
git add docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md docs/superpowers/plans/2026-07-06-npm-lite-lifecycle-runtime-messaging.md
git commit -m "docs(roadmap): record npm lifecycle runtime messaging" -m "Track the Stage 6 npm-lite lifecycle messaging update and remaining release-readiness checks."
```
