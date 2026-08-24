# Design: real migrated-project Graph v1 acceptance

## System boundary

The slice has two deliverables that meet at one stable project path:

1. a real, repository-owned Qiongli empirical project at
   `RESEARCH/asset-pricing-capm-ff3/`;
2. a fail-closed acceptance entrypoint that migrates that project and passes
   its native Graph/App payload through existing Desktop Graph adapters.

No production Graph schema, projector, store, renderer, or migration path is
added.

## End-to-end data flow

```text
Kenneth French official ZIPs
  -> project-local Python analysis (digest check, parse, join, regressions)
  -> deterministic derived tables + provenance manifest
  -> Qiongli canonical research artifacts and evidence ledger
  -> 1.19-compatible repository source (no 2.x manifest/runtime)
  -> existing project migrate preview/apply into an isolated destination
  -> existing Graph snapshot/doctor/query and App artifact read
  -> existing Desktop readiness/layout/search/focus/inspection/Cytoscape adapters
  -> redacted PLT-322 receipt
```

## Research project design

### Workflow stages

Use the installed Qiongli workflow in this order:

- Idea Funnel and boundary review;
- `A1`, `A1_5`, and `A2` for question, hypotheses, and contribution;
- targeted reading plus `B6` for a small source-anchored literature map,
  including notes, retrieval limits, and an extraction table;
- `C1`, `C3`, `C3_5`, and `C4` for design, variables, robustness, diagnostics,
  and data management;
- `I5`, `I6`, `I7`, and `I4` for code specification, plan, execution, profile,
  and reproducibility audit;
- the evidence-ledger portion of `F4` after results exist.

The project stops at the analysis stage. It does not claim systematic-review
coverage or manuscript completeness.

### Analysis contract

The single analysis entrypoint owns download, validation, parsing, common-sample
construction, estimation, diagnostics, and output writing. It must:

- accept no hard-coded absolute paths;
- reject ZIP path traversal, missing/extra expected members, duplicate months,
  malformed dates, missing sentinels in the analysis sample, and digest drift;
- use July 1963 through the latest common month from the pinned inputs;
- estimate CAPM and FF3 for all 25 value-weighted portfolios, then repeat as an
  equal-weighted sensitivity;
- use HAC standard errors with six lags and record the exact model settings;
- write deterministically ordered CSV/JSON/Markdown outputs with no timestamps
  inside result files;
- keep retrieval time and byte/digest metadata in a separate provenance file;
- include a small self-check for parsing/model invariants and rerun the pipeline
  twice to compare output digests.

Dependency isolation uses PEP 723 metadata plus a script lock. NumPy and
statsmodels are project-only; Qiongli's product dependency graph is unchanged.

### Source project compatibility

The committed project is intentionally a bounded legacy-shaped source:

- canonical research artifacts and derived outputs are present;
- the repository's 1.19 Python CLI initializes project-local guidance and locks
  the `finance` subject before artifact work begins;
- `context/project_manifest.json` is absent;
- the resulting `.qiongli/` guidance/runtime state, plus `.codex/`, `.claude/`,
  caches, raw data, and conversations, is ignored and never committed;
- no file contains an absolute developer path.

The acceptance runner rejects a source that already has a 2.x manifest or has
uncommitted source drift.

## Acceptance coordinator

Add one root command backed by a small Node coordinator and one Desktop Vitest
acceptance test. Reuse the existing native CLI as the only product authority.

### Coordinator responsibilities

- require explicit `--source` and `--receipt` arguments;
- require a clean exact Git product commit;
- run the existing native readiness negative controls and Desktop empty/sparse
  controls before the representative-project run;
- build/use the native binary without a shell command surface;
- create a private temporary config home and migration destination;
- pass only stable flags and bounded environment variables to subprocesses;
- delete temporary state after completion;
- write the receipt only after every required subprocess succeeds.

### Representative-project assertions

The Desktop acceptance test drives the native CLI with a fresh process for each
operation and decodes native JSON with the existing App API Zod schemas. It
asserts:

1. migration preview/apply is digest-bound, source-retaining, and excludes
   private runtime state;
2. a pre/post source inventory digest is unchanged;
3. two snapshots and a restart/reopen snapshot have identical projection IDs,
   projection digests, node/edge identities, and diagnostics;
4. readiness is `visualizable`, with semantic nodes and reviewed non-`contains`
   relations from canonical artifacts;
5. relation and stable-ID queries are non-empty, bounded, and internally
   consistent;
6. every accepted semantic entity has a supported artifact path and non-empty
   source anchor;
7. `app read-project-artifact` resolves at least one accepted node and one edge
   at the exact revision/projection;
8. the same decoded query builds a non-empty existing Desktop layout and
   Cytoscape element set, supports search/focus, and produces matching source
   inspection metadata.

The test does not render a second UI or duplicate native semantic logic.

## Evidence and privacy

The machine receipt may contain only:

- schema/document kind and pass/fail status;
- exact Git product commit;
- repository-relative source identifier and a canonical inventory digest;
- project ID, migration plan digest, projection ID/digest, and bounded counts;
- relation/node-type names, readiness/reason code, and required check IDs;
- hashes of the analysis results and the receipt itself.

It must not contain raw research rows, prose labels, citations, absolute paths,
environment values, Host conversations, credentials, or temporary locations.

## Rollout and closeout

1. Land the project and acceptance implementation as the product-source commit.
2. Run the real analysis and acceptance against that exact clean commit.
3. If any required check fails, retain `PLT-322=proposed` and report the stable
   blocker; do not create a passing receipt.
4. After a pass, add the redacted receipt/acceptance note, update the Program
   Ledger to `accepted`, regenerate the current program index, and record the
   exact Slice CI run.
5. The evidence-only closeout commit does not change product/package inputs and
   does not authorize release publication.

Rollback is ordinary Git reversion of the research project, acceptance harness,
and evidence-only ledger update. All acceptance destinations are temporary and
the source is never removed.
