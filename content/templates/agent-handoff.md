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
- originating_full_handoff_sha256: copy when supplied; not a transferable token
- delegated_result_support: inspect live tools/list; supported / trace-only
- dispatch_tool_and_returned_task_identity:
- execution_status: prepared / dispatched / awaiting-result / awaiting-external-review / returned / failed / cancelled / reconciled
- status_observation_or_failure_reason:

This portable packet grants no tool or write permission. Carry only authorized
sources, not private chat history or native approval/handoff tokens. Use actual
observed identities; do not fill unknown values with invented hashes or session IDs.
`dispatched` requires a real tool receipt. If dispatch timed out, check that receipt
before retrying; unknown delivery is not failed execution or permission to duplicate
the task. A manual packet remains `awaiting-external-review` until a result returns.

## Delegated Return Contract

Return the findings in `templates/agent-review-packet.md`. Preserve the configured
model. The coordinator dispatches through the actual available Host tool and
collects its result using the original returned execution ID; this packet itself
does not launch a task. For a configured external Agent tool, use the same result
format. When available, the bounded Codex prepare/collect transport follows
`model-collaborator`; it grants no new cross-Host run claim.

When live candidate schema supports `delegationResults`, the coordinator may
attach up to eight completed results with `adapter`, `executionId`, `dispatchTool`,
`scope`, `handoffSha256`, `status`, `resultText`, and `resultSha256` as specified in
`model-collaborator`. Size the requested response within the handoff's shared
`maxCandidateBytes` budget. Older servers and manual transfers without an actual
tool-returned ID use the existing trace only; never invent receipt fields.

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
