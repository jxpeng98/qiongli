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

## Coordinator Reconciliation

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
