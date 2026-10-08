# ADR 0238: Resilient Plugin Installation

- Status: Accepted
- Date: 2026-10-08
- Task: CLI-402
- Authority: the maintainer requests installation progress, actionable DSH errors
  and completion of remaining clients after one client fails.
- Supersedes only the batch stop-on-failure/cancellation policy in ADRs 0235–0237.
  Source ownership, official managers, receipt/CAS and separate approvals remain.

Run every selected Host sequentially with its own approval. A Host error is
recorded as failed and a complete negative confirmation as skipped; continue to
the next Host. Closed/incomplete input or failed output stops the batch, and
unattempted Hosts are reported separately. Global language/menu cancellation
remains a cancellation before installation. A partial failure returns nonzero;
single-Host errors retain their original reason code. No automatic retry or
rollback, duplicated install, receipt bypass or implicit approval is added.

Show the Host ordinal, manager command step and elapsed time every five seconds
while its direct command runs. Counts describe processed Hosts, not download
bytes or scientific/Host-session readiness. Final output reports installed,
failed, skipped and not-run Hosts, their reason codes/remediation, and safely
quoted targeted Qiongli retry commands. Failed commands show the actual official
manager argv, so DSH profile and exact npm version remain available for diagnosis.

Reuse the bounded command owner and its isolated environment, per-stream output
cap and timeouts. Preserve exit status and captured bytes for installation-only
diagnostics while the existing string wrapper keeps its reason codes and UTF-8
contract. Emit only recognized static error categories/hints; never echo or save
raw manager output, which can include credentials, configuration or terminal
escapes. Unknown errors remain unknown with a command for direct diagnosis.
No requested package version, registry, user model or authentication setting is
changed to make an installation appear successful. Pi retains its private umask.

Output-pipe EOF shares the command deadline, so inherited pipes cannot block
the batch indefinitely. Keep normal terminal process-group membership for Ctrl-C
and existing direct-child termination. This is not process-tree termination:
descendants may outlive a timeout; inspect manager state before retrying. No
partial filesystem changes are claimed to be rolled back.

Checks cover first-failure/later-success, decline/EOF, summary and retry quoting,
known/unknown errors, exit-code capture, output caps/UTF-8 compatibility, timeout
with inherited pipes, progress and approval/receipt negatives. Synthetic manager
fixtures qualify the control flow, not the user's still-unidentified DSH failure
or a completed five-Host installation. Rollback restores the previous batch and
terminal reporting behavior without deleting sources or changing profiles.
