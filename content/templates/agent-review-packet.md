# Agent Review Packet

## Review Metadata

- reviewer_agent:
- reviewed_run_id:
- review_status:
- task_id:
- source_revision_or_digests_reviewed:
- candidate_identity_reviewed:
- execution_identity: actual Host/session/task, if available
- dispatch_identity_returned: match the original task receipt when available
- independence_or_prior_context:
- scope_read_and_unavailable_material:
- execution_outcome: completed / partial / failed / cancelled

Match these bindings to the handoff. A missing or stale identity remains a gap;
this report does not grant approval or prove that suggested changes were applied.
The coordinator checks the returned task and candidate against the current source,
records stale or duplicate results, and leaves unavailable verification explicit.
A copied identity is a binding to verify, not authentication or proof of independence.

## Optional Delegated Result

When supported by live `tools/list`, the coordinator captures this envelope from
actual execution; it is not an instruction for the reviewer to invent a receipt.
Native subagents and configured external Agent tools use the same fields:

- adapter: native-subagent / external-agent
- executionId: actual nonempty dispatch-returned task/session ID
- dispatchTool: actual dispatch tool
- scope: bounded assignment
- handoffSha256: unchanged originating Full handoff digest
- status: queued / running / completed / failed / cancelled
- resultText: exact returned text
- resultSha256: SHA-256 of exact UTF-8 resultText bytes

Submit only completed, reconciled entries, at most eight, with no duplicate
`(adapter, executionId)` pairs. Native entries require `NativeSubagents` capability.
Combined delegated text shares `maxCandidateBytes` with the candidate. Keep the
exact tool-returned text separate from the coordinator's synthesis; do not trim it
after hashing. Failed, cancelled, late or duplicate results remain in the existing
trace. Older servers receive no unsupported `delegationResults` field.

These bindings do not prove sender identity, replace authenticated source reads,
grant approval or launch an Agent. A manual external packet without an observed
execution ID remains trace-only.

## Coordinator Reconciliation

- observed_original_execution_state_and_cancellation:
- current_source_and_candidate_match: unchecked / matched / stale / mismatched
- result_disposition: pending / accepted / changes-requested / duplicate / rejected
- accepted_findings_and_unresolved_disagreements:
- applied_changes_and_authority: none unless separately verified

## Findings

- None.

## Blocking Issues

- None.

## Required Revisions

- None.

## Verification Evidence

- verification_evidence:
- Not run.
