# Solo Mode

Solo mode is for running one controller while preserving explicit quality gates. It is useful when only one runtime is available, or when a task needs a single accountable operator before later review.

## Command Shape

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --execution-mode solo \
  --controller codex \
  --primary codex \
  --solo-role-gates strict \
  --mcp-strict \
  --skills-strict
```

Set `--controller` and `--primary` to the same runtime in normal solo use. If you declare a separate `--reviewer` or `--verifier`, record why the run is still considered solo.

## Codex-Only Writing Gates

When Codex is the only operator for writing-facing tasks, use these gates as the intended review checklist:

| Gate | Required check |
| --- | --- |
| `evidence_ledger_check` | Central claims are traceable to `claim-evidence-ledger.csv` or marked as gaps. |
| `citation_risk_check` | Unsupported, stale, or venue-sensitive citations are flagged before final prose. |
| `claim_calibration_check` | Claims are narrowed to the actual method, data, and evidence. |
| `scholarly_voice_check` | Drafts follow the academic output rubric and avoid unsupported certainty. |

Current offline hard gate: `scripts/audit_solo_role_gates.py` requires an existing claim-map artifact for Codex-only writing packets unless `solo_role_gates` is `off`.

## Claude-Only Engineering Gates

When Claude is the only operator for engineering or Stage-I code tasks, use these gates as the intended review checklist:

| Gate | Required check |
| --- | --- |
| `implementation_intent` | State the intended code behavior and artifact paths before editing. |
| `declared_write_set` | List the files expected to change and avoid unrelated edits. |
| `failing_test_first` | Create or identify the failing test before implementation when behavior changes. |
| `command_evidence` | Record the verification commands and results. |
| `rollback_notes` | Explain how to revert or isolate the change if validation fails. |

Current offline hard gate: `scripts/audit_solo_role_gates.py` requires an existing implementation-intent artifact for Claude-only code packets unless `solo_role_gates` is `off`.

For Stage-I code, prefer `code-build` or `task-run` on `I5 -> I6 -> I7 -> I8`; do not collapse spec, plan, execution, and review into one prompt.

## Gate Levels

| Level | Use when | Behavior |
| --- | --- | --- |
| `strict` | External-facing outputs, Stage-I code, submission material | Record stricter intent; current offline audits use the same artifact checks as `standard`. |
| `standard` | Internal drafts and exploratory synthesis | Apply current offline artifact checks. |
| `off` | Local experiments only | Record controller metadata without treating solo gates as release blockers. |

`--solo-role-gates` is declaration metadata for the run packet. Pair it with `python3 scripts/validate_research_standard.py --strict` and `scripts/audit_solo_role_gates.py` when the run needs auditable enforcement.
