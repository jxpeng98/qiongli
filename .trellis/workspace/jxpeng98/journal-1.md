# Journal - jxpeng98 (Part 1)

> AI development session journal
> Started: 2026-08-10

---


## Session 1: Close Alpha 3 internal candidate

**Date**: 2026-08-11
**Task**: Close Alpha 3 internal candidate
**Package**: product
**Branch**: `docs/alpha3-internal-candidate-closeout`

### Summary

Recorded the cced6082 exact internal candidate, rejected publication, exposed the CLI size blocker, reconciled M0 governance, and archived the completed first-usable task.

### Git Commits

| Hash | Message |
|------|---------|
| `b7fb44d7` | (see git log) |

### Status

[OK] **Completed**


## Session 2: Close Alpha 3 Host qualification and repair Full MCP routing

**Date**: 2026-08-12
**Task**: Close Alpha 3 Host qualification and repair Full MCP routing
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Recorded partial exact-candidate qualification, fixed Full MCP route fallback to Marketplace Lite, aligned canonical Skills and roadmap, and hardened the role-gate audit without authorizing publication.

### Git Commits

| Hash | Message |
|------|---------|
| `f1260f44` | (see git log) |
| `ce9afbb5` | (see git log) |

### Status

[OK] **Completed**


## Session 3: Close evaluation false-green cases

**Date**: 2026-08-12
**Task**: Close evaluation false-green cases
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Closed EVAL-401 through EVAL-405 with fail-closed typed assertions and restored the repository test baseline.

### Main Changes

- Migrated four golden cases to explicit requiredness and V1 typed assertions.
- Made the shared eval runner reject missing, malformed, blocked, unknown, and zero-execution evidence.
- Reconciled stale distribution and Alpha 3 packaging contracts.

### Git Commits

| Hash | Message |
|------|---------|
| `80837676` | (see git log) |
| `65b9eaa2` | (see git log) |
| `be7422be` | (see git log) |

### Testing

- [OK] python -m unittest discover -s tests -v (1737 tests, 18 skipped, OK)
- [OK] python -m unittest tests.test_eval_cases -v (7 tests, OK)
- [OK] python3 scripts/validate_capability_contract.py (OK)
- [OK] git diff --check (clean)

### Status

[OK] **Completed**

### Next Steps

- Begin EVAL-406 only in a new Trellis task; keep EVAL-407 and later work deferred.


## Session 4: Implement EVAL-406 scientific validators

**Date**: 2026-08-12
**Task**: Implement EVAL-406 scientific validators
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Added seven fail-closed scientific validators to the shared eval runner, synchronized the Evaluation Truth V1 contract, and closed roadmap item EVAL-406.

### Main Changes

- Implemented schema, field, count, cross-artifact, locator, citation-identity, and exact-byte digest assertions.
- Added strict path, configuration, parsing, non-vacuity, and required-empty-artifact handling with focused regression coverage.
- Updated the systematic-review PRISMA case, Evaluation Truth spec, and master roadmap.

### Git Commits

| Hash | Message |
|------|---------|
| `ae4dfaef585e04b944964046782cf8a6c690f854` | (see git log) |

### Testing

- [OK] python -m unittest tests.test_eval_cases -v (10 tests passed)
- [OK] python -m unittest discover -s tests (1740 tests passed, 18 skipped)
- [OK] git diff --check and py_compile passed

### Status

[OK] **Completed**

### Next Steps

- Start EVAL-407 deterministic JSON and JUnit evaluation receipts in a new Trellis task.


## Session 5: Implement EVAL-407 deterministic receipts

**Date**: 2026-08-12
**Task**: Implement EVAL-407 deterministic receipts
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Added deterministic, redacted JSON and JUnit eval receipts with opt-in atomic CLI writes; verified 1743 tests, documented the contract, and closed only EVAL-407.

### Git Commits

| Hash | Message |
|------|---------|
| `33f9d6a04cb4c99c5f24015299ef25ac2d52134c` | (see git log) |

### Status

[OK] **Completed**


## Session 6: Audit Qiongli 2 roadmap executability and credibility

**Date**: 2026-08-12
**Task**: Audit Qiongli 2 roadmap executability and credibility
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Audited roadmap structure, evidence, GitHub mapping, milestone executability, and live-state credibility without changing roadmap or remote state.

### Main Changes

- Verified 233 unique roadmap IDs versus 232 Epic-mapped IDs and identified REL-300 as the unmapped item.
- Separated supported release evidence from stale current-task, candidate, EVAL, and program-ledger claims.

### Git Commits

| Hash | Message |
|------|---------|
| `0a50b023e35df06d32a25c64cd6f46fbf8032c1b` | (see git log) |

### Testing

- [OK] Ran 13 focused eval tests, deterministic M1 coverage checks, task validation, JSON validation, and staged diff checks.

### Status

[OK] **Completed**

### Next Steps

- Resolve current branch delivery identity, then implement the bounded GOV-401 through GOV-403 program-ledger foundation before EVAL-408.


## Session 7: Realign App CLI and Plugin roadmap priorities

**Date**: 2026-08-13
**Task**: Realign App CLI and Plugin roadmap priorities
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Prioritized App-managed official Host CLI activation as P0, queued executable Plugin quality as P1, synchronized product-control guidance, and preserved later milestone gates.

### Git Commits

| Hash | Message |
|------|---------|
| `252248854951a7b132e3da02ad98b49f37a59f1b` | (see git log) |

### Status

[OK] **Completed**


## Session 8: Close App-mediated Host Plugin activation

**Date**: 2026-08-14
**Task**: Close App-mediated Host Plugin activation
**Package**: content
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Bound one App approval to fixed official Codex and Claude CLI plans, required fresh Plugin/cache/Skill/MCP evidence for Ready, passed full workspace and packaged macOS acceptance, archived P0, and activated the P1 executable Plugin-quality task.

### Git Commits

| Hash | Message |
|------|---------|
| `fdfd5323` | (see git log) |

### Status

[OK] **Completed**


## Session 9: Make Plugin quality executable

**Date**: 2026-08-15
**Task**: Make Plugin quality executable
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Restored the frozen legacy literature MCPB contract, converted 12 academic-quality fixtures to executable V1 assertions, repaired the eight bounded Coursework and Dissertation Skills, and verified canonical plus staged Plugin quality gates.

### Git Commits

| Hash | Message |
|------|---------|
| `65c6e4f4` | (see git log) |
| `4b9a8fa6` | (see git log) |

### Status

[OK] **Completed**


## Session 10: Make Evaluation Truth own 2.x CI

**Date**: 2026-08-15
**Task**: Make Evaluation Truth own 2.x CI
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Added six adversarial Evaluation Truth fixtures, moved the 12-case suite to a canonical runner, preserved legacy shims, and verified direct 2.x CI ownership.

### Git Commits

| Hash | Message |
|------|---------|
| `d16d0fc5` | (see git log) |

### Status

[OK] **Completed**


## Session 11: Close EVAL-411 mutation evidence

**Date**: 2026-08-15
**Task**: Close EVAL-411 mutation evidence
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Proved freshly passing Evaluation Truth V1 cases fail after required-evidence deletion or count mutation, passed local and exact-head gates, and closed only EVAL-411 while preserving integration and release non-claims.

### Git Commits

| Hash | Message |
|------|---------|
| `93a84945` | (see git log) |
| `356d6582` | (see git log) |

### Status

[OK] **Completed**


## Session 12: Integrate PR 124 into 2.x

**Date**: 2026-08-17
**Task**: Integrate PR 124 into 2.x
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Updated evidence-gated roadmap wording, merged the exact PR head into protected 2.x, verified merge-SHA Evaluation Truth and Native CI, and observed non-publishing promotion without an authorization action.

### Git Commits

| Hash | Message |
|------|---------|
| `f34a3a16` | (see git log) |

### Status

[OK] **Completed**


## Session 13: Validate latest 2.x local macOS package

**Date**: 2026-08-17
**Task**: Validate latest 2.x local macOS package
**Package**: product
**Branch**: `fix/alpha3-codex-claude-host-qualification`

### Summary

Built and opened a local ad-hoc signed Qiongli.app from exact origin/2.x source 237de9ba for manual inspection.

### Main Changes

- Copied the ignored artifact to dist/local-2x-237de9ba/Qiongli.app without changing product source.

### Git Commits

(No commits - planning session)

### Testing

- [OK] codesign deep strict verification passed
- [OK] ui startup check reported ready
- [OK] bundled CLI reported qiongli 2.0.0-alpha.3

### Status

[OK] **Completed**
