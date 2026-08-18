# Close editable Plugin acceptance and connected Research Graph

## Goal

Close the two remaining user-visible gaps in the current packaged product:

1. prove that an App-confirmed local Workflow/Skill edit becomes the exact
   active content in standalone Skills and official Codex/Claude Plugins, then
   returns to canonical content after reset; and
2. make canonical research records form one navigable, source-bound graph in
   the existing Academic Graph instead of appearing as disconnected records.

The graph should feel continuous in the way an Obsidian graph does, while
remaining academically safer: only deterministic structural containment and
explicit, source-backed scholarly relations are allowed.

## Confirmed baseline

- The packaged CLI, standalone Skills, Codex/Claude Plugin bundles, Lite MCP,
  Full MCP, App editor, variant receipts, and canonical Host activation already
  exist and pass their focused tests.
- Isolated real Codex `0.147.0` and Claude Code `2.1.231` canonical Plugin tests
  pass without touching normal Host profiles.
- The packaged acceptance does not yet exercise edit -> explicit reconcile ->
  Customized Ready -> reset -> explicit reconcile -> Canonical Ready.
- Academic Graph v1 already has a Cytoscape canvas, search, focus history,
  path finding, source inspection, deterministic layouts, accessible tables,
  15 node kinds, and 25 relation kinds.
- Graph v1 reads the project manifest, eight canonical structured artifacts,
  and reviewed `graph/semantic_links.jsonl`. It deliberately does not infer
  academic facts from arbitrary prose or BibTeX.
- The projection currently links project -> source artifact, but does not add a
  structural source artifact -> extracted semantic record edge. Therefore valid
  questions, decisions, claims, papers, and concepts can remain disconnected
  even though their exact source artifact is already known.
- The canonical Workflow/Skill content does not currently require a final
  graph-readiness check after graph-bearing project artifacts are updated.

## Requirements

### Packaged editable content lifecycle

- Preserve the existing receipt-owned local Workflow/Skill variant store,
  preview/confirm boundary, compare-and-swap revision checks, and explicit
  Skills/Plugin reconciliation.
- Extend the existing packaged acceptance rather than create a second harness.
- Prove one customized marker reaches every selected standalone Skills target,
  the Codex Plugin cache, and the Claude Plugin cache through existing plans.
- Prove fresh Host observations report Customized Ready only after explicit
  reconciliation, and report Canonical Ready only after reset plus explicit
  reconciliation.
- Keep Plugin manifests, MCP declarations, and bundled native binaries
  canonical and byte-identical throughout customization.
- Strengthen the existing ignored real-Host tests so they prove the actual Host
  cache contains customized Skill bytes, not only canonical bundle metadata.

### Connected canonical Research Graph

- Reuse Academic Graph v1 and its existing Cytoscape presentation; do not build
  a second graph, graph database, or layout engine.
- Add deterministic structural continuity from every present canonical source
  artifact to each semantic node extracted from that artifact.
- Reuse the existing `contains` relation and edge identity constructor. Mark
  these edges observed, high-confidence, direct structural evidence, with an
  explicit limit that containment does not imply scholarly support.
- Preserve all existing scholarly edges, exact source anchors, stable IDs,
  query bounds, projection determinism, and read-only derived-state boundary.
- Make every extracted semantic node reachable from the project node through
  project -> artifact -> semantic record, even when no stronger scholarly edge
  has been reviewed yet.
- Keep the Research Library as the project index and make its selected-project
  primary action explicitly open the existing connected knowledge graph.
- Update the canonical Workflow/Skill contract so graph-bearing artifacts keep
  exact templates and stable IDs, major transitions refresh academic context,
  and the agent reports when App/CLI refresh is still required.

### Integrated evidence

- Exercise one graph-bearing research project through App, CLI, and Full MCP
  and require the same revision, projection identity, node/edge counts, and
  connected result.
- Add exact-source packaged receipt checks for both the editable content cycle
  and connected graph outcome.
- Rebuild the local macOS App from the exact tested source and retain
  non-publishing/ad-hoc status.

## Constraints

- Do not read from or write to the user's normal Codex or Claude profiles in
  automated tests; use isolated temporary homes only.
- Do not invoke a live model or claim model-output quality.
- Do not add dependencies, a Graph v2 schema, arbitrary Markdown `[[wikilink]]`
  parsing, semantic similarity inference, editable edges, or hidden graph state.
- Do not weaken drift detection, receipt verification, path safety, bounded
  reads, revision binding, preview/approval, or rollback behavior.
- Do not make manifest, MCP, or binary files editable through the Workflow
  editor.
- Do not publish or authorize a release from local package evidence.

## Acceptance Criteria

- [ ] A packaged App-confirmed edit makes every installed Skills/Plugin target
  update-required until its existing explicit reconcile plan is applied.
- [ ] After reconciliation, standalone Skills plus real/fixture Codex and Claude
  Plugin caches contain the exact customized Skill bytes and variant digest;
  fresh observations report Customized Ready.
- [ ] Reset again makes all selected targets update-required; explicit
  reconciliation restores canonical bytes, receipts, and Canonical Ready.
- [ ] Plugin manifests, `.mcp.json`, Lite/Full MCP inventories, native binary,
  and unrelated canary files remain unchanged across the edit/reset cycle.
- [ ] Every semantic node produced from a present canonical artifact is
  reachable from the project node by deterministic structural and/or scholarly
  edges, with no unsupported scholarly relation invented.
- [ ] Rebuilding the same graph revision produces identical node, edge,
  projection, and connected-component identities.
- [ ] The App renders the connected result in the existing Cytoscape canvas,
  retains search/focus/source inspection, and preserves the synchronized table
  fallback and reduced-motion/keyboard behavior.
- [ ] The Research Library's selected-project action clearly opens the same
  project graph without losing project selection.
- [ ] Canonical Plugin/Skill guidance requires graph-readable stable artifacts
  and an explicit refresh/readiness handoff after relevant writes.
- [ ] App, CLI, and Full MCP return the same exact graph revision, projection,
  node count, edge count, and source anchors for the acceptance fixture.
- [ ] The exact-source macOS package receipt records both lifecycle checks as
  passing and retains `publication_allowed: false`.
- [ ] Focused tests, full required local checks, and exact-head required CI pass
  before merge.

## Out of scope

- Automatic relation extraction from arbitrary notes, PDFs, prose, or BibTeX.
- User-authored free-form graph nodes/edges or an Obsidian vault replacement.
- Graph v2 / Kernel authority, cloud sync, collaborative editing, or release.

## Notes

- This task closes a product evidence gap and one deterministic connectivity
  bug. Broader graph authoring remains on the existing long-term roadmap.
