# Product Control

Use this spec for work that changes the Qiongli 2 product spine, release scope,
or acceptance claims.

## Authority

Use the narrowest source that owns the decision:

1. the current task under `.trellis/tasks/` owns executable scope and next work;
2. the Alpha acceptance ledger owns accepted evidence and release authorization;
3. the master roadmap owns milestone order and long-term priorities;
4. accepted ADRs own architecture boundaries;
5. older plans and receipts are historical evidence only.

Do not copy the roadmap backlog into Trellis. Keep at most one implementation
task in progress and create the next task only when the current one is closed.

## Current Release Outcome

Alpha 3 must first produce one internally usable, self-contained product spine:

`App -> native CLI -> Plugin/Skills -> Lite/Full MCP -> Zotero`

The packaged product must not need a user-installed Python or Node runtime.
Public distribution is a later state: target-native, live-Host, update/rollback,
trust, authorization, and public-observation evidence remain owned by A6-A9.

M0 external/manual release evidence remains open, but it is no longer a blanket
development freeze after the internal first-usable spine closes. The M1
`EVAL-401`—`EVAL-406` evaluation-truth code lane may proceed in a reviewed Trellis task
without implying M0 exit, Alpha 3 qualification, or publication authority. M2
and later work remains deferred until the M1 entry/exit gates are satisfied.
Do not add Graph v2, a research kernel, more providers, more agents, or remote
collaboration to an Alpha 3 or M1 false-green task.

## Evidence Ladder

Run only the smallest evidence set that advances the change:

1. one focused local check while editing;
2. one exact-head CI run after the change set is frozen;
3. one packaged vertical acceptance run when package inputs changed.

Manual UI, real-profile Host, update, and publication checks are release-claim
gates, not substitutes for the development checks above. If a public claim is
not accepted, remove or narrow that claim rather than recording a false pass.

### Evidence closeout boundary

- A closeout records `product_source`, exact CI/promotion run IDs, candidate-set
  digest, package digests, and `publication_allowed`; it never substitutes the
  closeout commit's own SHA for the built product source.
- An evidence-only status commit does not require another package run when it
  changes no product or package input. Any product/package input change does.
- If protected publication requires the current branch head after an
  evidence-only commit has landed, do not authorize the older internal
  candidate. Freeze and qualify a new product candidate when release resumes.

## Pre-Development Checklist

- Read the current Trellis task and the Alpha acceptance ledger.
- Name the broken user outcome and its shared owner.
- Confirm the work is part of the Alpha 3 product spine.
- Identify one focused check before editing.

## Quality Check

- App, CLI, Plugin/Skills, MCP, and Zotero claims match the shipped contracts.
- Every accepted result is bound to the same source and package identity.
- No historical receipt is presented as evidence for a changed candidate.
- No extra umbrella test, duplicate backlog, or speculative abstraction was added.

Executable contracts:

- [Evaluation Truth V1](eval-truth-v1.md) — shared case schema, counters, and
  fail-closed success predicate.

Reference files:

- `docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md`
- `docs/superpowers/acceptance/2026-08-01-qiongli-alpha3-readiness.md`
- `docs/architecture/decisions/README.md`
