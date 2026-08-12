# Initial Findings

Audit snapshot: `255baa3ec430efdc837748a6676b547163ba4416`.

## Confirmed drift candidates

- Roadmap lines 266, 308, 375—380 and 391 still speak about a current/active
  Alpha 3 qualification task and EVAL-401—405 as next work. That task and the
  EVAL-401—407 tasks are now archived.
- Section 3.3 still lists empty evals, metadata-only quality evals, and YAML
  validation together as current P0 gaps even though Section 10 records the
  false-green/YAML subset through EVAL-407 as complete. The metadata-only
  academic-quality gap remains open and must be separated from repaired items.
- Section 8.1 says detailed task state lives in a machine-readable ledger, while
  Section 10.2 leaves `GOV-401` (create that ledger) unchecked.
- The roadmap claims 32 Epic Issues cover all 232 IDs exactly once. This is not
  proven by local prose and requires live/machine verification.
- The M0 release lane remains truthfully open and permission-bound; completed M1
  eval work must not be treated as Alpha 3 qualification evidence.

These are audit leads, not final findings. Severity and corrections will be
assigned only after full evidence tracing.
