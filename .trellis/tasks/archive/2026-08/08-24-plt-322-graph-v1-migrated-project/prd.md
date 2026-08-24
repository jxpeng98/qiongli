# Accept Graph v1 on a real asset-pricing project

## Goal

Close `PLT-322` with a repository-owned empirical asset-pricing project built
through the Qiongli workflow. Migrate its 1.19-compatible source layout through
the supported 2.x project-migration path and prove that Graph v1 preserves
source-bound scholarly meaning, useful query and visualization, deterministic
rebuild, and truthful empty/sparse states.

## Background and Confirmed Facts

- Product control makes `PLT-322` the next replacement-critical slice after
  accepted `PLT-320` and `PLT-321`.
- Graph v1 already owns projection, source diagnostics, bounded query, path
  finding, readiness, revision comparison, App API events, and the Desktop
  Cytoscape presentation.
- Project migration is already copy-based, retains the source, excludes
  recognized private runtime state, records a migration receipt, and qualifies
  two deterministic graph rebuilds.
- The August 18 continuity repair already excludes structural `project` /
  `artifact` nodes and `contains` edges from semantic readiness.
- Existing migration and Graph checks use synthetic fixtures. The roadmap
  explicitly requires a representative migrated project and says fixture/code
  presence alone cannot close the replacement row.
- The user explicitly chose a new repository-owned asset-pricing project as the
  representative source. It may contain public scholarly artifacts and derived
  results, but not Host conversations, credentials, private data, or absolute
  local paths.

## Requirements

### R1 — Build a genuine Qiongli empirical project

- Create `RESEARCH/asset-pricing-capm-ff3/` from Qiongli's empirical workflow,
  beginning with an Academic Idea Funnel and boundary review.
- Study how much the Fama-French three-factor model attenuates CAPM pricing
  errors for the 25 U.S. size/book-to-market portfolios using the official
  Kenneth French Data Library monthly files.
- Use real public data and verified source metadata. Do not fabricate data,
  citations, model results, or supportive claims.
- Complete the project through analysis, diagnostics, claim-evidence mapping,
  and reproducibility audit. A full manuscript is not required for this slice.
- Keep the committed source 1.19-compatible by omitting the 2.x project
  manifest and private runtime state; 2.x state is created only by migration.

### R2 — Preserve data and source boundaries

- Commit the analysis code, dependency lock, provenance manifest, canonical
  research artifacts, and bounded derived result tables.
- Do not commit the copyrighted raw ZIP/CSV inputs, dependency caches, Host
  conversations, credentials, private runtime state, or absolute paths.
- Pin each downloaded input by official URL and SHA-256. A changed upstream
  digest must fail closed until it is explicitly reviewed and refreshed.
- Run migration against an isolated destination and prove the committed source
  inventory is byte-for-byte unchanged before and after migration.

### R3 — Prove source-bound scholarly semantics and useful query

- Require at least one scholarly node type other than `project` / `artifact`
  and at least one non-`contains` relation derived from reviewed canonical
  artifacts; a generated `graph/semantic_links.jsonl` sidecar alone cannot be
  the authority for this claim.
- Verify every accepted semantic node and relation has a bounded artifact path
  and source anchor that resolves through the existing artifact-read contract.
- Exercise the existing bounded Graph query with stable IDs and relations from
  the project, and require non-empty, internally consistent results without
  widening public schemas.

### R4 — Prove deterministic migration, rebuild, and restart continuity

- Use the existing migration preview/apply contract and its digest-bound
  approval rather than copying files in a new implementation.
- Rebuild the migrated graph twice and require the same projection identity,
  projection digest, node/edge identities, diagnostics, and query result.
- Reopen the migrated project through a fresh native/App state and require the
  same graph and readiness while the original 1.19 source remains unchanged.

### R5 — Prove useful presentation on the same migrated project

- Feed the same migrated project's native Graph/App API payload through the
  existing Desktop Graph v1 adapters and layout/readiness contracts.
- Require a renderable non-empty semantic topology, useful search/focus
  state, and source inspection metadata; do not create a second renderer or a
  parallel Graph data model.
- Keep synthetic empty, structural-only, nodes-without-relations, and sparse
  cases as negative controls. They may prove truthful diagnostics but cannot
  substitute for the selected representative project.

### R6 — Produce truthful, reproducible acceptance evidence

- Add one bounded acceptance entrypoint that fails closed when the real-project
  input, semantic assertions, UI assertions, or deterministic checks are
  skipped.
- Bind the receipt to the exact product commit, migration plan/project
  identity, repository-relative source identity, redacted input digest, Graph
  projection digest, and executed check set.
- Mark `PLT-322` accepted and publish an acceptance note only after the named
  real-project run passes. Otherwise leave the ledger state `proposed` and
  record the blocker.

## Acceptance Criteria

- [x] The repository contains a Qiongli empirical project with an Idea Funnel,
      boundary review, framing, literature map, study/analysis design, real
      analysis outputs, evidence ledger, and reproducibility audit.
- [x] The official data inputs are digest-pinned, raw inputs remain uncommitted,
      and rerunning the analysis from matching inputs reproduces the committed
      machine-readable outputs.
- [x] The 1.19-compatible source is unchanged by migration and no credential,
      Host conversation, private runtime state, or absolute path appears in
      committed evidence.
- [x] A supported copy-based migration completes with a valid project migration
      receipt and excludes recognized private runtime state.
- [x] The migrated graph contains source-resolvable scholarly nodes and at
      least one non-`contains` relation from canonical artifacts.
- [x] Bounded stable-ID and relation queries return useful, consistent results
      for the selected project.
- [x] Two rebuilds plus a fresh-state reopen produce the same projection and
      query evidence.
- [x] The same project payload passes Desktop readiness, layout, interaction,
      and source-inspection acceptance.
- [x] Empty, structural-only, relationless, and sparse negative controls report
      their truthful states and cannot pass the representative-project gate.
- [x] The acceptance entrypoint fails closed for missing input or any skipped
      required check and emits a redacted exact-source receipt.
- [x] Focused checks and the affected native/App API/Desktop Slice checks pass.
- [x] Only after the real-project run succeeds, the acceptance note and program
      ledger record `PLT-322` with exact commit and evidence identity.

## Out of Scope

- Graph v2, a Typed Research Kernel, new graph storage, or public schema changes.
- Automatic prose-to-fact inference or acceptance-time enrichment of the
  project's research meaning.
- Editable graph facts, collaboration, sync, or Host conversation retention.
- Candidate packaging, release publication, 1.19 retirement, or the later
  `PILOT-903` independent real-project pilot.
- A full paper, novel factor discovery, causal asset-pricing claim, trading
  strategy, or investment recommendation.

## Resolved Decisions

- The project is public and repository-owned rather than copied from a private
  user directory.
- “Random asset pricing project” means Qiongli selects one bounded project idea
  from several candidates; the empirical data and results themselves are not
  randomized or synthetic.
- The selected question compares CAPM with Fama-French three-factor pricing
  errors on the 25 size/book-to-market portfolios.
- The project stops after verified analysis and evidence mapping; the later
  independent real-project pilot remains separate.
