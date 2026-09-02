# Current delivery authority audit

## Confirmed owners

- `.trellis/workflow.md` ends every task with Slice validation even when no PR
  or integration checkpoint is being produced.
- `.trellis/config.yaml` currently uses `codex.dispatch_mode: inline`; the
  existing Codex Implement and Check Agent definitions need no new role.
- Native CI already limits package assembly, packaged-product acceptance,
  candidate lifecycle, and promotion dispatch to explicit `workflow_dispatch`
  on `2.x`.
- `scripts/check_2x_native_change_boundary.sh` preserves required context names
  but skips their Rust work only for a narrow evidence allowlist; general docs
  and local process files currently trigger the matrix.
- `.github/delivery-checklists.md` already separates Focused, Slice, and
  Acceptance semantics, but repeats authority detail across four stages.
- The master roadmap contains all 237 task IDs. The program ledger and generated
  index provide state/evidence; several roadmap passages incorrectly call the
  generated index the immediate planning owner.
- `CONTRIBUTING.md` spends most of its 126 lines on historical CTR inventory
  details instead of the current contribution path.

## Minimal change

Reuse every existing owner. Add no workflow or Agent. Change only the default
trigger boundary, the shared native-matrix classifier, and the human/AI routing
documents. Preserve fail-closed behavior for runtime, mixed, unknown, workflow,
fixture, security, authorization, schema, path, and data-loss changes.
