# Agent Handoff

## Handoff Metadata

- from_agent:
- to_agent:
- task_id:
- requested_outcome:
- collaboration_mode: independent-review / draft-review / candidate-edit
- coordinator:
- observed_source_revision_or_digests:
- candidate_identity: exact reviewed version, or not applicable
- allowed_actions_and_file_scope:
- return_destination:
- dispatch_tool_and_returned_task_identity:
- execution_status: prepared / dispatched / awaiting-result / awaiting-external-review / returned / failed / cancelled / reconciled
- status_observation_or_failure_reason:

This portable packet grants no tool or write permission. Carry only authorized
sources, not private chat history or native approval/handoff tokens. Use actual
observed identities; do not fill unknown values with invented hashes or session IDs.
`dispatched` requires a real tool receipt. If dispatch timed out, check that receipt
before retrying; unknown delivery is not failed execution or permission to duplicate
the task. A manual packet remains `awaiting-external-review` until a result returns.

## Completed Artifacts

- None.

## Decision Summary

- decision_summary:
- None.

## Unresolved Questions

- None.

## Evidence Dependencies

- evidence_dependencies:
- stable_claim_decision_ids_and_citekeys:
- prior_stage_summary:
- None.

## Assumptions

- None.

## Risks

- None.

## Next Actions

- None.
