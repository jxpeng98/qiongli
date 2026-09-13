# Close editable packaged content and connect Research Library

## Goal

Close the only material gap found in the packaged
`App -> native CLI -> Plugin/Skills -> MCP` chain: let a user edit the
behavioral Markdown instructions that the managed Plugin and standalone Skills
actually install, without changing canonical embedded content, executable/MCP
authority, or Host-owned caches directly.

The user expanded this task to include the next approved usability phase:
packaged typography must be consistent, and Research Library must expose the
existing source-bound Academic Graph as a connected, Obsidian-like project
topology. `GOV-401` through `GOV-404` remains the following task.

## Confirmed Baseline

- The exact `origin/2.x` merge source is
  `237de9ba9e235f2b5067cc9704aef49eee3ce9c6` (PR #124).
- Post-merge Evaluation Truth run `31984053266` and Native CI run
  `31984053292` passed. Native CI included the macOS packaged-product control
  acceptance job.
- The packaged App already installs the exact bundled CLI, configures PATH,
  drives fixed official Codex/Claude CLI plans after one confirmation, clears
  prior observations, and reports Ready only from fresh Plugin/cache/MCP
  evidence. Claude additionally exposes exactly one `qiongli-workflow` Skill;
  Codex proves the exact Skill bytes through bundle identity, not live use.
- Canonical Plugin/Skill composition, standalone Skills materialization,
  receipt-bound update/remove, Full MCP launch, and isolated real-client tests
  already exist and passed for the activation source recorded in the archived
  task.
- The current App's “Preview and customize” surface is incomplete for the
  user's requested meaning of editability. It previews verified Plugin/Skill
  resources, but the only editable write is project-local
  `<project>/.qiongli/local_guidance.md`.
- Directly editing a current managed Skill or Plugin tree is classified as
  drift. It cannot be the customization mechanism because it breaks exact
  removal, repair, cache comparison, and Ready evidence.
- The current local accepted package receipt is for older source `fdfd5323`;
  the ordinary manually inspected package is from `237de9ba` but intentionally
  lacks packaged-product install authority. A new exact-source acceptance App
  is therefore required after this product change.

Detailed evidence is in `research/packaged-content-chain-audit.md`.

## Requirements

### R1 — Keep one immutable canonical parent

- Canonical embedded content, Plugin IDs/manifests, MCP declarations, schemas,
  binaries, product version, and signed launch grants remain immutable.
- A local workflow variant is derived only from an exact verified canonical
  pack and is never written back to `content/`, embedded bytes, generated
  package mirrors, or Host caches.
- The first version permits only UTF-8 Markdown instruction resources already
  projected as the top-level workflow Skill or canonical Skill cards. It does
  not permit editing Plugin identity, executable arguments, MCP configuration,
  tool schemas, standards, roles, templates, or arbitrary paths.

### R2 — Make the local variant receipt-owned

- Store one Qiongli-managed local workflow variant under the product's private
  v2 configuration root.
- Bind every override to canonical path and base digest, bounded total/file
  limits, a monotonic revision, and a deterministic variant digest.
- Preview/update/reset use compare-and-swap state plus the existing
  digest-bound confirmation operation. Stale drafts, unsupported paths,
  invalid controls/encoding, oversize content, links, drift, and unmanaged
  state fail closed.
- Removal or reset restores canonical derivation; it does not delete unrelated
  files or user project guidance.

### R3 — Reuse the current materialization and activation path

- Standalone managed Skills and Codex/Claude Plugin composition apply the same
  selected local variant at materialization time and record canonical parent
  identity plus exact derived identity in their receipts.
- A variant change makes affected managed destinations update/repair-required;
  it does not silently rewrite them.
- Reconciliation still uses the existing preview/approval transaction and the
  fixed official Host CLI plan. The App never writes a Host cache directly.
- Ready means the exact selected canonical-or-local-variant managed receipt
  matches the freshly observed Host cache and MCP state. A customized Ready
  state is labeled as local/customized and is never described as canonical.

### R4 — Complete the App editing journey

- Extend the existing Workflow Content panel rather than adding a new route or
  editor framework.
- List the bounded editable instruction resources, show canonical/current
  status, edit one resource at a time, preview its exact managed effect, confirm
  or reset it, and expose which managed destinations now need reconciliation.
- Preserve project-local guidance as a separate per-project advisory layer.
- Keep read-only Plugin manifests visible for audit, but label why they are not
  editable.
- Preserve keyboard labels, focus, loading/disabled/error states, and one
  vertical scroll owner.

### R5 — Prove the packaged chain after customization

- Focused tests cover allowed-path validation, size/control bounds, stale
  revision rejection, deterministic variant identity, canonical reset, receipt
  drift, and unrelated-file preservation.
- App API/native/browser fixtures express the same versioned contract.
- Codex and Claude bundle tests prove that the customized Skill bytes are in
  the managed and cached exact bundle while MCP remains the packaged binary.
- Isolated fake-client and installed real-client tests use temporary homes only;
  no normal Codex or Claude profile is mutated.
- Freeze one exact product commit, run the existing macOS packaged-product
  acceptance, and preserve `publication_allowed=false`.

### R6 — Unify typography and expose connected research topology

- Native text controls inherit the App's existing Geist/system sans stack; no
  additional font or editor dependency is added.
- Research Library loads the existing revision-bound Academic Graph portfolio
  and shows its deterministic project/shared-identity topology before the
  project index.
- Selecting a topology project preserves the shared project workspace state;
  the existing Explore workspace action opens the full Academic Graph with the
  selected project.
- The graph remains derived, source-bound, read-only, bounded, and backed by
  the existing accessible identity/relation records. No second graph store,
  inferred-link editor, or Graph v2 is introduced.

## Acceptance Criteria

- [ ] A user can edit an allowed workflow/Skill Markdown resource in the App,
      preview the exact change, confirm it, and later reset it to canonical.
- [ ] Canonical embedded content, Plugin manifests, MCP declarations, binaries,
      tool schemas, and unrelated project/user files remain byte-identical.
- [ ] Standalone Skills, Codex Plugin, and Claude Plugin derive the same selected
      local variant and retain exact verify/update/remove behavior.
- [ ] A changed variant never becomes active silently; affected destinations
      require explicit reconcile/install confirmation.
- [ ] After reconciliation, fresh Codex Plugin/cache/MCP evidence and fresh
      Claude Plugin/cache/Skill/MCP evidence match the exact selected variant
      before Ready is shown.
- [ ] Stale drafts, invalid paths/content, missing authority, receipt drift,
      command failure, or Host observation mismatch remain non-Ready and leave
      unrelated state untouched.
- [ ] Focused App API/Desktop/native tests, full native workspace checks, both
      isolated real-client tests, and one exact-source macOS packaged acceptance
      pass.
- [ ] No public release, publication, live model-quality, or live Codex Skill
      invocation claim is added.
- [ ] Buttons and native form controls use the same intended font family in the
      packaged WebView.
- [ ] Research Library shows a connected portfolio topology and continues into
      the full revision-bound Academic Graph without losing project selection.

## Out of Scope

- Editing Plugin manifests, IDs, versions, MCP configuration, executable
  arguments, tool schemas, binaries, or canonical repository sources from the
  App.
- Arbitrary files, a general code editor, sync/collaboration, variant sharing,
  multiple named variants, or a custom marketplace.
- Mutating the developer's real Host profiles, authenticated model prompts,
  notarization, tagging, release publication, or Alpha qualification.
- `GOV-401` through `GOV-404`; that machine-readable roadmap task remains next
  in the approved train.

## Open Questions

None. The recommended first version is one local, receipt-bound behavioral
instruction variant. Broader Plugin metadata or MCP editing remains excluded
because it would expand execution authority rather than customize research
behavior.
