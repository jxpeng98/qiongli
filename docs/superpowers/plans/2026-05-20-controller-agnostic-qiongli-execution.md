# Controller-Agnostic Qiongli Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Qiongli support both Codex-Claude fusion and strict single-platform execution through shared contracts, role gates, handoffs, and validators.

**Architecture:** Keep the existing workflow contract as the source of truth, then add a runtime execution layer that is independent of which platform is controller. Fusion mode uses cross-agent handoff/review/adjudication; solo mode uses the same artifact contract plus role-specific self-review and validator gates.

**Tech Stack:** Python 3.12, unittest, YAML/JSON/Markdown templates, existing `bridges/` orchestrator and validator scripts.

---

## Execution Principles

- Preserve existing `task-run`, `parallel`, and `team-run` behavior unless a task explicitly extends them.
- Add contracts before orchestration behavior, so validators can enforce the new modes.
- Keep Codex-primary, Claude-primary, Codex-only, and Claude-only as first-class modes.
- Use offline tests and fake bridge outputs. Do not require real Codex or Claude CLI calls in tests.
- Commit each task independently after tests pass and review is complete.

## Planned File Structure

- `docs/audits/controller-agnostic-orchestration-audit.md`: current-state audit of Codex/Claude/solo integration.
- `standards/agent-run-contract.yaml`: canonical schema and allowed enums for agent run/review/handoff packets.
- `standards/solo-role-policy.yaml`: role gates for `solo_codex`, `solo_claude`, and `solo_gemini`.
- `standards/agent-routing-policy.yaml`: stage defaults for solo/duo/triad routing.
- `templates/agent-run-packet.json`: machine-readable run packet scaffold.
- `templates/agent-review-packet.md`: reviewer report scaffold.
- `templates/agent-handoff.md`: cross-agent and solo stage handoff scaffold.
- `templates/disagreement-matrix.md`: disagreement/adjudication scaffold.
- `templates/duo-review-report.md`: duo review summary scaffold.
- `templates/solo-task-packet.md`: solo execution task packet scaffold.
- `templates/solo-self-review.md`: structured solo self-review scaffold.
- `templates/implementation-intent.md`: code-task intent and write-set scaffold.
- `templates/writing-claim-map.md`: writing-task claim/evidence scaffold.
- `templates/quality-gate-report.md`: generic gate result scaffold.
- `bridges/context_package.py`: context package builder for Codex/Claude/Gemini views.
- `bridges/orchestrator.py`: CLI and task-run integration for controller/mode metadata.
- `scripts/audit_solo_role_gates.py`: offline solo gate audit.
- `scripts/audit_agent_handoffs.py`: offline handoff/review/disagreement audit.
- `scripts/validate_research_standard.py`: strict-mode integration for new contracts.
- `tests/test_agent_run_contract.py`: agent run contract tests.
- `tests/test_solo_role_policy.py`: solo role policy tests.
- `tests/test_context_package_builder.py`: context package tests.
- `tests/test_controller_agnostic_orchestration.py`: orchestrator CLI/task packet tests.
- `tests/test_duo_orchestration.py`: duo handoff/review tests.
- `tests/test_disagreement_adjudication.py`: disagreement matrix tests.
- `tests/test_agent_routing_policy.py`: routing policy tests.
- `tests/test_solo_role_gate_audit.py`: solo audit tests.
- `tests/test_agent_handoff_audit.py`: handoff audit tests.
- `evals/controller_modes/`: offline fixtures for controller-mode regressions.
- `scripts/run_controller_mode_evals.py`: offline eval runner.

## Phase 0: Baseline Audit

### Task 0.1: Controller-Agnostic Integration Audit

**Files:**
- Create: `docs/audits/controller-agnostic-orchestration-audit.md`

- [ ] **Step 1: Inspect current integration points**

Read:
```bash
sed -n '1,220p' bridges/base_bridge.py
sed -n '1,220p' bridges/codex_bridge.py
sed -n '1,220p' bridges/claude_bridge.py
sed -n '1,260p' bridges/orchestrator.py
sed -n '1,260p' standards/mcp-agent-capability-map.yaml
sed -n '1,220p' standards/agent-profiles.example.json
```

- [ ] **Step 2: Write the audit**

Create `docs/audits/controller-agnostic-orchestration-audit.md` with these sections:
```markdown
# Controller-Agnostic Orchestration Audit

## Current Runtime Surfaces

## Current Contract Coverage

## Codex Bridge Gaps

## Claude Bridge Gaps

## Orchestrator Mode Gaps

## Solo Mode Gaps

## Validator Coverage Gaps

## P0 Implementation Recommendations
```

- [ ] **Step 3: Verify documentation exists**

Run:
```bash
test -s docs/audits/controller-agnostic-orchestration-audit.md
```
Expected: exit code 0.

- [ ] **Step 4: Commit**

Run:
```bash
git add docs/audits/controller-agnostic-orchestration-audit.md
git commit -m "docs: audit controller agnostic orchestration"
```

## Phase 1: Shared Agent Run Contract

### Task 1.1: Agent Run Contract and Templates

**Files:**
- Create: `standards/agent-run-contract.yaml`
- Create: `templates/agent-run-packet.json`
- Create: `templates/agent-review-packet.md`
- Create: `templates/agent-handoff.md`
- Test: `tests/test_agent_run_contract.py`

- [ ] **Step 1: Write failing tests**

Add tests that require:
```python
REQUIRED_RUN_FIELDS = {
    "run_id",
    "execution_mode",
    "controller",
    "primary_agent",
    "task_id",
    "paper_type",
    "topic",
    "input_context_hash",
    "session_id",
    "artifacts_read",
    "artifacts_written",
    "warnings",
    "blocking_issues",
    "confidence",
    "verification_status",
}
```

Allowed enums:
```python
execution_mode = {"solo_codex", "solo_claude", "solo_gemini", "duo", "triad"}
agent = {"codex", "claude", "gemini"}
verification_status = {"passed", "failed", "blocked"}
```

Run:
```bash
python3 -m unittest tests.test_agent_run_contract -v
```
Expected: FAIL because files are missing.

- [ ] **Step 2: Add `standards/agent-run-contract.yaml`**

Define:
```yaml
contract_version: "1.0.0"
execution_modes:
  - solo_codex
  - solo_claude
  - solo_gemini
  - duo
  - triad
runtime_agents:
  - codex
  - claude
  - gemini
verification_statuses:
  - passed
  - failed
  - blocked
required_run_fields:
  - run_id
  - execution_mode
  - controller
  - primary_agent
  - task_id
  - paper_type
  - topic
  - input_context_hash
  - session_id
  - artifacts_read
  - artifacts_written
  - warnings
  - blocking_issues
  - confidence
  - verification_status
required_review_fields:
  - reviewer_agent
  - reviewed_run_id
  - review_status
  - findings
  - blocking_issues
  - required_revisions
required_handoff_fields:
  - from_agent
  - to_agent
  - task_id
  - completed_artifacts
  - unresolved_questions
  - assumptions
  - risks
  - next_actions
```

- [ ] **Step 3: Add templates**

`templates/agent-run-packet.json` must include every required run field with safe empty defaults.

`templates/agent-review-packet.md` must include:
```markdown
# Agent Review Packet

## Review Metadata
## Findings
## Blocking Issues
## Required Revisions
## Verification Evidence
```

`templates/agent-handoff.md` must include:
```markdown
# Agent Handoff

## Handoff Metadata
## Completed Artifacts
## Decision Summary
## Unresolved Questions
## Evidence Dependencies
## Assumptions
## Risks
## Next Actions
```

- [ ] **Step 4: Run tests**

Run:
```bash
python3 -m unittest tests.test_agent_run_contract -v
```
Expected: PASS.

- [ ] **Step 5: Commit**

Run:
```bash
git add standards/agent-run-contract.yaml templates/agent-run-packet.json templates/agent-review-packet.md templates/agent-handoff.md tests/test_agent_run_contract.py
git commit -m "Add agent run contract"
```

## Phase 2: Solo Role Policy

### Task 2.1: Solo Policy and Role Gate Templates

**Files:**
- Create: `standards/solo-role-policy.yaml`
- Create: `templates/solo-task-packet.md`
- Create: `templates/solo-self-review.md`
- Create: `templates/implementation-intent.md`
- Create: `templates/writing-claim-map.md`
- Create: `templates/quality-gate-report.md`
- Test: `tests/test_solo_role_policy.py`

- [ ] **Step 1: Write failing tests**

Tests must assert:
- `solo_codex`, `solo_claude`, and `solo_gemini` exist.
- `solo_codex.writing_required_gates` includes `evidence_ledger_check`, `citation_risk_check`, `claim_calibration_check`, `scholarly_voice_check`.
- `solo_claude.code_required_gates` includes `implementation_intent`, `declared_write_set`, `failing_test_first`, `command_evidence`, `rollback_notes`.
- Templates include the required headings used by those gates.

Run:
```bash
python3 -m unittest tests.test_solo_role_policy -v
```
Expected: FAIL because policy/templates are missing.

- [ ] **Step 2: Add solo policy**

Create `standards/solo-role-policy.yaml` with:
```yaml
policy_version: "1.0.0"
solo_modes:
  solo_codex:
    writing_required_gates:
      - evidence_ledger_check
      - citation_risk_check
      - claim_calibration_check
      - scholarly_voice_check
    code_required_gates:
      - tests
      - strict_validator
      - diff_review
  solo_claude:
    writing_required_gates:
      - evidence_ledger_check
      - citation_risk_check
      - reviewer_self_critique
    code_required_gates:
      - implementation_intent
      - declared_write_set
      - failing_test_first
      - command_evidence
      - rollback_notes
  solo_gemini:
    writing_required_gates:
      - evidence_ledger_check
      - claim_calibration_check
      - source_integrity_check
    code_required_gates:
      - implementation_intent
      - verification_blocked_when_commands_unavailable
      - artifact_contract_check
```

- [ ] **Step 3: Add templates**

Each template must have headings that correspond to policy gates:
- `templates/solo-task-packet.md`: Task Metadata, Required Artifacts, Role Gates, Verification Commands.
- `templates/solo-self-review.md`: Draft Pass, Self-Critique Pass, Revision Pass, Final Checklist.
- `templates/implementation-intent.md`: Declared Write Set, Rationale, Failing Test Plan, Rollback Notes.
- `templates/writing-claim-map.md`: Claim ID, Claim Text, Evidence IDs, Calibration, Unsupported Claims.
- `templates/quality-gate-report.md`: Gate Metadata, Passed Gates, Failed Gates, Blocked Verification, Next Actions.

- [ ] **Step 4: Run tests**

Run:
```bash
python3 -m unittest tests.test_solo_role_policy -v
```
Expected: PASS.

- [ ] **Step 5: Commit**

Run:
```bash
git add standards/solo-role-policy.yaml templates/solo-task-packet.md templates/solo-self-review.md templates/implementation-intent.md templates/writing-claim-map.md templates/quality-gate-report.md tests/test_solo_role_policy.py
git commit -m "Add solo role policy"
```

## Phase 3: Context Package Builder

### Task 3.1: Build Controller-Specific Context Packages

**Files:**
- Create: `bridges/context_package.py`
- Create: `templates/context-manifest.json`
- Test: `tests/test_context_package_builder.py`

- [ ] **Step 1: Write failing tests**

Tests must call:
```python
from bridges.context_package import build_context_package
```

Expected output keys:
```python
{
    "context_manifest",
    "agent_contexts",
}
```

Codex context must include `Declared Write Set`, `Verification Commands`, and `Artifact Paths`.
Claude context must include `Research State`, `Evidence Ledger`, and `Writing/Review Standards`.

Run:
```bash
python3 -m unittest tests.test_context_package_builder -v
```
Expected: FAIL because module is missing.

- [ ] **Step 2: Implement `build_context_package`**

Signature:
```python
def build_context_package(task_packet: dict[str, object], *, controller: str, agents: list[str]) -> dict[str, object]:
    ...
```

It must:
- normalize controller and agent names,
- build stable context strings for `codex`, `claude`, and `gemini`,
- compute `input_context_hash` from sorted JSON of the manifest,
- return a manifest with `task_id`, `paper_type`, `topic`, `controller`, `agents`, and `input_context_hash`.

- [ ] **Step 3: Run tests**

Run:
```bash
python3 -m unittest tests.test_context_package_builder -v
```
Expected: PASS.

- [ ] **Step 4: Commit**

Run:
```bash
git add bridges/context_package.py templates/context-manifest.json tests/test_context_package_builder.py
git commit -m "Add controller context package builder"
```

## Phase 4: Orchestrator Mode Surface

### Task 4.1: Add Controller-Agnostic CLI Metadata

**Files:**
- Modify: `bridges/orchestrator.py`
- Modify: `standards/agent-profiles.example.json`
- Test: `tests/test_controller_agnostic_orchestration.py`
- Test: `tests/test_orchestrator_workflows.py`

- [ ] **Step 1: Write failing tests**

Tests must verify parser/task packet support for:
```bash
--execution-mode solo|duo|triad
--controller codex|claude|gemini
--primary codex|claude|gemini
--reviewer codex|claude|gemini
--verifier codex|claude|gemini
--solo-role-gates strict|standard|off
```

Run:
```bash
python3 -m unittest tests.test_controller_agnostic_orchestration tests.test_orchestrator_workflows -v
```
Expected: FAIL until CLI metadata is wired.

- [ ] **Step 2: Implement CLI metadata only**

Add argument parsing and include fields in task packet/run metadata. Do not change existing runtime execution order in this task.

- [ ] **Step 3: Run tests**

Run:
```bash
python3 -m unittest tests.test_controller_agnostic_orchestration tests.test_orchestrator_workflows -v
```
Expected: PASS.

- [ ] **Step 4: Commit**

Run:
```bash
git add bridges/orchestrator.py standards/agent-profiles.example.json tests/test_controller_agnostic_orchestration.py tests/test_orchestrator_workflows.py
git commit -m "Add controller agnostic orchestration options"
```

## Phase 5: Duo Handoff and Disagreement Protocol

### Task 5.1: Duo Review and Disagreement Templates

**Files:**
- Create: `templates/disagreement-matrix.md`
- Create: `templates/duo-review-report.md`
- Create: `scripts/audit_agent_handoffs.py`
- Test: `tests/test_duo_orchestration.py`
- Test: `tests/test_disagreement_adjudication.py`
- Test: `tests/test_agent_handoff_audit.py`

- [ ] **Step 1: Write failing tests**

Tests must require:
- duo review report has Findings, Blocking Issues, Required Revisions, Adjudication.
- disagreement matrix has issue_id, codex_position, claude_position, evidence_refs, risk_level, final_decision.
- audit fails when duo mode lacks handoff or disagreement artifact after conflicting positions.

- [ ] **Step 2: Add templates and audit script**

Implement an offline audit function:
```python
def audit_agent_handoffs(root: Path) -> list[str]:
    ...
```

It returns a list of error strings and performs no network calls.

- [ ] **Step 3: Run tests**

Run:
```bash
python3 -m unittest tests.test_duo_orchestration tests.test_disagreement_adjudication tests.test_agent_handoff_audit -v
```
Expected: PASS.

- [ ] **Step 4: Commit**

Run:
```bash
git add templates/disagreement-matrix.md templates/duo-review-report.md scripts/audit_agent_handoffs.py tests/test_duo_orchestration.py tests/test_disagreement_adjudication.py tests/test_agent_handoff_audit.py
git commit -m "Add duo handoff protocol"
```

## Phase 6: Stage Routing Policy

### Task 6.1: Agent Routing Policy

**Files:**
- Create: `standards/agent-routing-policy.yaml`
- Test: `tests/test_agent_routing_policy.py`

- [ ] **Step 1: Write failing tests**

Tests must assert defaults for:
- `B_literature`: primary `claude`, reviewer `codex`, verifier `codex`.
- `F_writing`: primary `claude`, reviewer `codex`, verifier `codex`.
- `I_code`: primary `codex`, reviewer `claude`, verifier `codex`.

- [ ] **Step 2: Add routing policy**

Create the YAML policy and include solo gate mappings for `solo_codex` and `solo_claude`.

- [ ] **Step 3: Run tests**

Run:
```bash
python3 -m unittest tests.test_agent_routing_policy -v
```
Expected: PASS.

- [ ] **Step 4: Commit**

Run:
```bash
git add standards/agent-routing-policy.yaml tests/test_agent_routing_policy.py
git commit -m "Add stage agent routing policy"
```

## Phase 7: Strict Validator Integration

### Task 7.1: Solo and Handoff Validator Gates

**Files:**
- Create: `scripts/audit_solo_role_gates.py`
- Modify: `scripts/validate_research_standard.py`
- Test: `tests/test_solo_role_gate_audit.py`
- Test: `tests/test_research_standard_validator.py`

- [ ] **Step 1: Write failing tests**

Tests must assert strict failures for:
- solo Codex writing without claim map,
- solo Claude code without implementation intent,
- duo run without handoff,
- reviewer blocker with final status passed,
- missing `verification_status`.

- [ ] **Step 2: Add audit script and validator hooks**

Implement:
```python
def audit_solo_role_gates(root: Path) -> list[str]:
    ...
```

Wire a lightweight contract-file existence check into `validate_research_standard.py --strict`.

- [ ] **Step 3: Run tests**

Run:
```bash
python3 -m unittest tests.test_solo_role_gate_audit tests.test_research_standard_validator -v
python3 scripts/validate_research_standard.py --strict
```
Expected: PASS.

- [ ] **Step 4: Commit**

Run:
```bash
git add scripts/audit_solo_role_gates.py scripts/validate_research_standard.py tests/test_solo_role_gate_audit.py tests/test_research_standard_validator.py
git commit -m "Add solo role validator gates"
```

## Phase 8: Documentation and Package Sync

### Task 8.1: Document Controller Modes

**Files:**
- Create: `guides/advanced/controller-modes.md`
- Create: `guides/advanced/solo-mode.md`
- Create: `guides/advanced/codex-claude-duo.md`
- Modify: `README.md`
- Modify: `README_CN.md`
- Modify: `CLAUDE.md`
- Modify: `qiongli-workflow/SKILL.md`

- [ ] **Step 1: Add docs**

Document:
- Codex-primary mode.
- Claude-primary mode.
- Codex-only writing gates.
- Claude-only engineering gates.
- Duo review and disagreement matrix.

- [ ] **Step 2: Sync package**

Run:
```bash
bash scripts/sync_skill_package.sh --target all --dry-run
```
Expected: self-contained.

- [ ] **Step 3: Commit**

Run:
```bash
git add guides/advanced/controller-modes.md guides/advanced/solo-mode.md guides/advanced/codex-claude-duo.md README.md README_CN.md CLAUDE.md qiongli-workflow/SKILL.md
git commit -m "Document controller execution modes"
```

## Phase 9: Offline Controller Mode Evals

### Task 9.1: Controller Mode Eval Fixtures

**Files:**
- Create: `evals/controller_modes/solo_codex_writing.json`
- Create: `evals/controller_modes/solo_claude_code.json`
- Create: `evals/controller_modes/claude_primary_codex_review.json`
- Create: `evals/controller_modes/codex_primary_claude_review.json`
- Create: `evals/controller_modes/duo_disagreement.json`
- Create: `evals/controller_modes/verification_blocked.json`
- Create: `scripts/run_controller_mode_evals.py`
- Test: `tests/test_controller_mode_evals.py`

- [ ] **Step 1: Write failing tests**

Require the eval runner to score:
- artifact completeness,
- role gate compliance,
- evidence traceability,
- command verification honesty,
- handoff quality,
- disagreement resolution.

- [ ] **Step 2: Add fixtures and runner**

Implement a deterministic offline runner that returns JSON summary with `status`, `scores`, and `failures`.

- [ ] **Step 3: Run tests**

Run:
```bash
python3 -m unittest tests.test_controller_mode_evals -v
```
Expected: PASS.

- [ ] **Step 4: Commit**

Run:
```bash
git add evals/controller_modes scripts/run_controller_mode_evals.py tests/test_controller_mode_evals.py
git commit -m "Add controller mode eval fixtures"
```

## Final Verification

Run:
```bash
python3 -m unittest discover -s tests -v
python3 scripts/validate_research_standard.py --strict
python3 scripts/audit_skill_sections.py --strict
bash scripts/sync_skill_package.sh --target all --dry-run
```

Expected:
- all tests pass,
- strict validator reports 0 failures,
- skill section audit reports 71/71 complete or the updated expected total,
- sync dry-run reports self-contained package.

## Self-Review

- Spec coverage: The plan covers fusion mode, Claude-primary, Codex-primary, Codex-only, Claude-only, handoff, solo gates, routing, validator integration, docs, and offline evals.
- Placeholder scan: No task contains TBD/TODO placeholders.
- Type consistency: The same enum names are used across contract, policy, tests, and CLI arguments.
