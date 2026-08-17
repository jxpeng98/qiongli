# Technical Design

## Boundary

The parent task is documentation-only. Its write set is limited to the master
roadmap, the product-control spec, and these Trellis planning artifacts. Product
code belongs to P0; canonical content and eval code belong to P1.

No long-term roadmap checklist ID is added. ADR 0213's `ARC-213` identifies the
new architecture decision and is not added to the master-roadmap backlog.
Trellis tasks are execution slices for existing outcomes, not a second program
ledger.

## Authority And Sequence

The authority order remains:

1. active Trellis task for executable scope;
2. Alpha acceptance ledger for accepted/release evidence;
3. master roadmap for cross-release order;
4. accepted ADRs for architecture and trust boundaries.

The immediate sequence becomes:

```text
roadmap realignment
  -> P0 App CLI + Plugin activation and Ready proof
  -> P1 executable Plugin-quality gate and bounded Skill repairs
  -> remaining M1 evaluation/governance/platform work
  -> M2-M7 unchanged
```

M0 external qualification remains an independent open evidence lane. Closing
P0 does not publish Alpha 3; closing P1 does not qualify model output quality or
Stable readiness.

## Roadmap Projection

The master roadmap update will change only the places that currently control
or contradict the immediate queue:

- document status and verified-baseline qualification;
- confirmed-gaps table for the App-managed Host activation gap and the still
  metadata-only academic-quality gate;
- dependency diagram and single-active-task rule;
- M0 execution-authority/current-task wording;
- M1 entry/current-task wording while retaining EVAL-409 unchecked;
- recommended first-90-days sequence;
- stale 232 references, corrected to the existing 233-ID inventory.

Historical receipts and completed checklist states remain unchanged. Local
EVAL-401 through EVAL-407 checks are labeled as working-head evidence until the
commits reach `2.x`.

## Child Boundaries

### P0: product activation

Reuses the existing App preview/confirm path, packaged-product control,
resolved client executables, bounded process runner, Host probes, receipts, and
isolated real-client tests. It may refine those owners but may not add a shell,
arbitrary command API, or direct Host-cache writer. It records the new
App-mediated official-CLI authority in ADR 0213, superseding only the relevant
part of frozen ADR 0206.

### P1: Plugin quality

Reuses Evaluation Truth V1 and the canonical Skill audit/materializers. It
removes the fake score authority and repairs only the eight currently exposed
content gaps. Paid/model-dependent ablation is deferred.

## Compatibility And Integration

The current working branch is ahead of `origin/2.x`. The parent roadmap can
describe working-head facts, but all target-branch acceptance language remains
conditional on integration. P1 requires an execution head containing
EVAL-401 through EVAL-407; if it starts from a different base, that prerequisite
must be integrated rather than reimplemented.

## Rollback

The parent has no runtime or data migration. Reverting its documentation commit
restores the prior queue. Child task directories may remain as planning history
but must not be marked active or accepted after a rollback.
