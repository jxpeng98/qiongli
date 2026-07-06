# Multi-Agent Parallel Smoke Opt-In Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Harden the maintainer multi-agent smoke harness so the parallel Codex/Claude/Antigravity runtime case requires both `--run-parallel` and `QIONGLI_SMOKE_RUN_AGENTS=1`.

**Architecture:** Keep existing per-runtime smoke cases unchanged. Gate only the optional `parallel_codex_claude_antigravity` case: when the flag is present but the environment opt-in is missing, emit a machine-readable WARN case instead of invoking the parallel runtime path.

**Tech Stack:** Python 3, unittest, Qiongli multi-agent smoke report JSON/Markdown.

---

## File Map

- Modify: `tests/test_multi_agent_smoke.py`
  - Add RED coverage that `--run-parallel` without the env var does not call the parallel runtime implementation.
  - Add coverage that the same flag with `QIONGLI_SMOKE_RUN_AGENTS=1` still runs the parallel case.
- Modify: `packages/python-qiongli/src/qiongli/multi_agent_smoke.py`
  - Add the shared `LOCAL_AGENT_ENV` constant.
  - Gate `_case_parallel_codex_claude_antigravity` behind the env opt-in.
- Modify: `docs/advanced/publish-pypi.md`
  - Document the double opt-in for heavier maintainer parallel smoke.
- Modify: `docs/zh/advanced/publish-pypi.md`
  - Keep the Chinese publishing guide aligned with the English guide.
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`
  - Record this opt-in hardening under the remaining runtime-enabled multi-agent gap.

## Task 1: Add Failing Tests

**Files:**
- Modify: `tests/test_multi_agent_smoke.py`

- [ ] **Step 1: Test missing env var warns and skips runtime**

Patch the regular smoke cases to pass without touching local runtimes, patch the parallel implementation to raise if called, run the runner with `run_parallel=True`, and assert the report includes:

```python
parallel = report.cases[-1]
self.assertEqual(parallel.name, "parallel_codex_claude_antigravity")
self.assertEqual(parallel.status, WARN)
self.assertIn("QIONGLI_SMOKE_RUN_AGENTS=1", parallel.detail)
```

- [ ] **Step 2: Test env var allows runtime case**

Patch `QIONGLI_SMOKE_RUN_AGENTS=1`, patch the parallel implementation to return `PASS`, and assert the final case status is `PASS`.

- [ ] **Step 3: Run RED**

Run:

```bash
.venv/bin/python -m unittest \
  tests.test_multi_agent_smoke.MultiAgentSmokeTests.test_parallel_smoke_requires_environment_opt_in \
  tests.test_multi_agent_smoke.MultiAgentSmokeTests.test_parallel_smoke_runs_when_environment_opt_in_is_present \
  -q
```

Expected before implementation: tests fail because `--run-parallel` always calls the parallel runtime case.

## Task 2: Implement The Gate

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/multi_agent_smoke.py`

- [ ] **Step 1: Add constant**

```python
LOCAL_AGENT_ENV = "QIONGLI_SMOKE_RUN_AGENTS"
```

- [ ] **Step 2: Add skip/WARN case**

When `self.args.run_parallel` is true and `os.environ.get(LOCAL_AGENT_ENV) != "1"`, record:

```text
parallel runtime smoke skipped; set QIONGLI_SMOKE_RUN_AGENTS=1 with --run-parallel to launch local agents
```

- [ ] **Step 3: Run GREEN**

Run the focused unittest command again. Expected: `OK`.

## Task 3: Docs, Roadmap, And Regression

**Files:**
- Modify: `docs/advanced/publish-pypi.md`
- Modify: `docs/zh/advanced/publish-pypi.md`
- Modify: `docs/superpowers/roadmaps/2026-07-01-adaptive-subject-runtime-roadmap.md`

- [ ] **Step 1: Document the double opt-in**

Add the env-var form for heavier maintainer parallel smoke:

```bash
QIONGLI_SMOKE_RUN_AGENTS=1 python3 tooling/scripts/smoke_multi_agent.py --run-parallel
```

- [ ] **Step 2: Run regression checks**

```bash
.venv/bin/python -m unittest tests.test_multi_agent_smoke -q
git diff --check
```

- [ ] **Step 3: Commit by category**

Feature commit:

```text
feat(smoke): gate parallel multi-agent smoke opt-in
```

Docs commit:

```text
docs(roadmap): record multi-agent smoke opt-in hardening
```
