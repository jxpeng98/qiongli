# Release Receipt Subject Evidence Plan

## Goal

Make release acceptance receipts include generated subject-router evaluation and
preview subject-runtime smoke summaries, so Stage 6 release readiness records
adaptive subject evidence instead of relying only on manual checkboxes.

## Scope

- Add a small release evidence helper that can render markdown from subject
  router eval reports and subject runtime smoke reports.
- Run the helper from `release_postflight.sh` before writing the acceptance
  receipt.
- Add a template placeholder for the generated evidence block.
- Update the adaptive subject runtime roadmap after verification.

## Out Of Scope

- Do not make opt-in local-agent smoke part of the default release gate.
- Do not change the existing subject router thresholds or smoke fixtures.
- Do not change GitHub release upload behavior.

## Steps

1. Add failing tests for the evidence renderer and postflight/template wiring.
2. Implement the evidence helper and thin root script wrapper.
3. Inject generated evidence into the acceptance template from postflight.
4. Run focused release automation tests plus a helper smoke command.
5. Update the Stage 6 roadmap status and commit implementation/docs by topic.

## Verification

```bash
.venv/bin/python -m unittest tests.test_release_acceptance_evidence tests.test_release_automation -q
python3 scripts/release_acceptance_evidence.py --root .
git diff --check
```
