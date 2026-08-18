# Repair Academic Graph continuity

## Goal

Make the existing Academic Graph v1 represent a connected scholarly network
rather than a file inventory. A project with only project/artifact nodes and
`contains` edges must not appear graph-ready, while Qiongli's bundled workflow
must give new and legacy projects a safe path to produce stable ideas, gaps,
literature clusters, papers, claims, evidence, and decisions with explicit
source-bound relations.

## Background

- The Desktop already has the required Obsidian-like interaction surface:
  Cytoscape rendering, search, focus history, path finding, clustering,
  backlinks, risk overlays, and revision-bound artifact inspection.
- Native Graph v1 already extracts semantic records from the canonical
  research-state, decision-log, boundary-review, idea-funnel, literature-map,
  claim-evidence-ledger, and manuscript claim-map formats.
- One observed registered project produced seven nodes and six edges, but every
  non-project node was an artifact and every edge was `contains`. Five source
  diagnostics showed legacy/missing graph-bearing structures. The readiness
  response still counted artifact nodes as semantic nodes and containment as
  topology.
- Existing workflow wording asks agents to preserve templates and refresh the
  graph, but it does not define a concrete legacy-normalization sequence or a
  semantic-connectivity acceptance check.
- Graph v2, the Typed Research Kernel, automatic prose inference, and editable
  graph facts remain deferred by the current M1 product boundary.

## Requirements

### R1 — Make Graph v1 readiness truthful

- Count only scholarly node types as semantic; `project` and `artifact` are
  structural and must not satisfy semantic readiness.
- Classify topology using non-`contains` relations. Structural containment may
  remain in the projection and public `relationCount`, but it cannot turn a
  project into `nodes-without-edges`, `sparse`, or `visualizable` incorrectly.
- A structural-only project must resolve to `no-recognized-artifacts`; a project
  with scholarly nodes but no scholarly relation must resolve to
  `nodes-without-edges`.
- Preserve the existing public Graph v1 schema and projection contents.

### R2 — Show semantic topology honestly in Desktop

- The readiness panel must label and display non-containment semantic relations,
  rather than presenting all structural edges as scholarly relations.
- Recovery copy must point the user to the existing Run in client workflow for
  stable-ID normalization/relationship enrichment and then the existing graph
  rebuild action.
- Keep the existing graph canvas and project workspace navigation; do not add a
  second mini-graph, editor, or data store.

### R3 — Make the bundled Plugin/Skill produce connected records

- Add one progressively disclosed Academic Graph continuity reference to the
  canonical Qiongli workflow package.
- Route graph-building/repair requests and major stage closes through that
  reference and `academic-context-maintainer`.
- Require exact bundled headings/columns and stable IDs for graph-bearing
  artifacts, plus a post-refresh snapshot/doctor check that reports semantic
  nodes, non-`contains` relations, diagnostics, and readiness.
- Prefer relations already derived from canonical artifacts. Do not require an
  agent to hand-author hashed node/edge IDs in `graph/semantic_links.jsonl`.

### R4 — Repair legacy projects without damaging research content

- Inspect Graph doctor/snapshot diagnostics before editing.
- Preserve narrative prose and already-valid stable IDs. When an old file lacks
  a supported structure, append or update the minimum canonical section rather
  than replacing the document.
- Preview the exact files, records, IDs, and source anchors before applying a
  project write. Create missing canonical files from bundled templates.
- Never invent a citation, evidence source, support direction, confidence,
  decision, or relationship. Unsupported material remains an explicit gap.
- Rebuild and re-read the exact project revision after apply; do not claim graph
  continuity from file presence alone.

### R5 — Keep roadmap and packaged outputs aligned

- Record this as an immediate bounded Plugin-quality/Graph v1 regression repair
  before the wider remaining M1 queue, without activating Graph v2 or Kernel
  work.
- Edit only canonical content sources; verify staged portable, Codex, and Claude
  payloads contain the new reference.
- Because native/Desktop and embedded Skill inputs change, finish with focused
  checks, exact-head CI, and a local non-publishing macOS package for manual
  inspection.

## Acceptance Criteria

- [x] A native regression proves that artifact nodes connected only by
      `contains` yield zero semantic nodes and `no-recognized-artifacts`.
- [x] A native regression proves scholarly nodes need at least one
      non-`contains` relation before topology is considered connected.
- [x] Existing Graph v1 snapshot/query/schema compatibility remains unchanged.
- [x] Desktop displays a semantic-relation count that excludes `contains`, with
      English and Chinese recovery copy that points to Run in client.
- [x] The canonical workflow and context-maintainer define the safe legacy
      normalization and post-refresh verification flow.
- [x] A focused content test fails if the graph continuity reference, stable-ID
      rules, semantic relation check, or non-destructive legacy rule disappears.
- [x] A staged Plugin/Skill package contains the reference and passes Skill and
      distribution validation without editing generated source-tree payloads.
- [x] The immediate product-control priority names this bounded Graph v1 repair
      and keeps Graph v2/Kernel deferred.
- [x] Focused native/Desktop/content tests, full affected checks, `git diff
      --check`, and the local macOS package build pass.

## Out of Scope

- Graph v2, a Typed Research Kernel, new graph storage, or new public schema.
- Automatic extraction of facts or relations from arbitrary prose, PDFs, or
  BibTeX without reviewed canonical records.
- Editable graph edges, manual canvas layout persistence, collaboration, or
  sync.
- Silently rewriting or deleting a user's existing research documents.
- Treating package/manual evidence as release publication authority.

## Authorization

The user explicitly approved continued implementation, commits, pushes, PRs,
CI repair, and local package work without repeated routine authorization. Only
dangerous or potentially data-destructive actions require another decision.
