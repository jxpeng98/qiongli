# Audit Design

## Boundary

This is a read-only governance audit. The task writes only its Trellis planning
and research report. It does not edit the roadmap, product, acceptance state, or
remote GitHub objects.

## Evidence Order

Use the narrowest authority for each claim:

1. active/archived Trellis task for current execution scope;
2. acceptance ledger and immutable receipts for accepted release claims;
3. work commit plus executable tests/specs for implemented behavior;
4. master roadmap for ordering and intended milestone gates;
5. GitHub Project/Issues/PRs/runs as collaboration or external evidence, never
   as substitutes for the owning local authority.

When sources disagree, report the conflict. Do not silently pick the newest
prose or promote a weaker source.

## Audit Matrices

### Executability

For every milestone, and every remaining M1 task, record:

- deliverable and user/program outcome;
- prerequisite and current entry state;
- owning authority and likely implementation owner;
- runnable validation or missing validation design;
- exit evidence and known blocker;
- classification: `ready-now`, `planning-needed`, `dependency-blocked`, or
  `aspirational`.

### Credibility

For status and evidence claims, record:

- exact claim and file/line anchor;
- owning source and current evidence;
- freshness/identity binding;
- verdict: `supported`, `partially-supported`, `stale`, `contradictory`, or
  `unverified`;
- impact and smallest correction.

## Rating Rubric

Rate executability and credibility separately from 0 to 5:

- `5`: exact owner, dependency, command/evidence, and exit gate are current;
- `4`: actionable now with only bounded Trellis decomposition needed;
- `3`: coherent but missing one material contract, baseline, or evidence owner;
- `2`: multiple dependencies or authority decisions are unresolved;
- `1`: aspirational direction without a credible execution boundary;
- `0`: internally contradictory or disproven.

The overall rating is a reasoned range/weighted judgment, not an arithmetic
quality score. Confidence is reported separately.

## Structural Checks

Use deterministic repository inspection to count and de-duplicate task IDs,
checkbox states, headings, links, and status vocabulary. Inspect GitHub through
`gh` only for live claims explicitly made by the roadmap. Record inaccessible
evidence as a limitation.

## Output

Write `research/roadmap-audit.md` with:

1. executive verdict;
2. high-severity contradictions and stale claims;
3. milestone executability table;
4. current M1 task-readiness table;
5. evidence/credibility assessment;
6. prioritized repair sequence;
7. recommended next task;
8. methods, snapshot identity, and verification limits.

## Rollback

Delete/archive the audit task. No product or roadmap state is changed.
