# Design: editable content and connected Research Graph

## Boundary

This task changes no public graph schema and adds no new product subsystem. It
reuses two existing owners:

- `WorkflowVariantStore` plus existing Skills/Plugin reconciliation plans own
  customized Workflow/Skill activation; and
- `AcademicGraphService` plus the existing Cytoscape route own research graph
  projection and presentation.

## Root causes

### Editable content evidence

The product implementation already stores and materializes a receipt-bound
variant. The missing proof is vertical: packaged acceptance removes canonical
targets before exercising a confirmed edit, and the real-Host tests install
only canonical bundles. A second implementation would duplicate authority.

### Graph continuity

`AcademicGraphService` already creates:

```text
project --contains--> canonical artifact
```

and extractors already create stable semantic nodes with `artifact_path` and
`source_anchor`. The missing deterministic edge is:

```text
canonical artifact --contains--> semantic record
```

Without it, a valid research question or decision can be visually isolated
until a stronger scholarly relationship happens to exist. The source binding
is already authoritative, so adding this structural edge requires no language
model, similarity heuristic, or new durable state.

## Packaged lifecycle flow

Extend `native_packaged_product_acceptance.rs` in place:

1. install canonical standalone Skills and Codex/Claude integrations;
2. save one bounded marker edit through `WorkflowVariantStore`;
3. assert existing App snapshots mark selected targets stale/update-required;
4. apply the existing `skills-update` and `integrations-reconcile` plans;
5. assert exact variant receipts, cache bytes, fresh Customized Ready, and
   unchanged manifest/MCP/binary bytes;
6. reset the resource through the same store;
7. require update/reconcile again and assert Canonical Ready;
8. perform the existing cleanup only after the cycle completes.

The real Codex/Claude ignored tests will compose a customized bundle through
`WorkflowOverrides`, install it into isolated homes, and inspect the official
Host cache. They remain non-network model-free activation tests.

## Graph projection flow

During `AcademicGraphService::rebuild`:

1. retain the artifact node ID for every present graph source;
2. run the existing canonical extractors and explicit semantic-record merge;
3. for every non-project, non-artifact semantic node, find its already-present
   source artifact by exact `artifact_path`;
4. create one `contains` edge from that artifact node to the semantic node with
   the existing `AcademicGraphEdgeV1::new` constructor;
5. insert it through the same deterministic edge map and validate the unchanged
   graph bounds.

The edge is structural only:

- inference strength: `direct_evidence`;
- confidence: `high`;
- status: `observed`;
- evidence limit: source containment does not imply academic support,
  contradiction, causality, or validity.

Existing extractor-produced relations such as `belongs-to-cluster`,
`supports`, `contradicts`, and `appears-in-section` remain the only scholarly
meaning. Rebuilding the same inputs yields the same edge IDs because the
existing identity function already binds source, relation, target, path, and
anchor.

## Plugin/Skill contract

Make the smallest content correction in the canonical Workflow and academic
context maintainer:

- preserve exact graph-bearing template headers and stable IDs;
- include `contribution_claim` in the maintained research state;
- run the existing academic context maintainer at major stage transitions;
- after graph-bearing writes, state that the registered project needs App/CLI
  revision refresh before the graph can be treated as current;
- when Full MCP graph tools are visible, inspect the refreshed graph rather
  than claiming continuity from file writes alone.

No new Skill or reference document is needed.

## UI

Keep Research Library as the index. Reuse the existing selected-project route
to `/academic-graph`; change only the user-facing action wording if needed so it
names the knowledge graph. The Academic Graph route already supplies pan/zoom,
search, focus history, component counts, source inspection, path finding,
keyboard controls, reduced-motion-safe behavior, and an accessible table/list
alternative. A second mini graph would duplicate state and worsen usability.

## Compatibility and rollback

- The graph schema and serialized node/edge structures are unchanged; only the
  deterministic projection contains additional observed `contains` edges.
- Existing consumers already accept `contains` and bounded edge counts.
- Rollback removes the structural-edge insertion and content-contract wording;
  canonical project artifacts remain unchanged.
- Customized content rollback continues to use the existing variant reset and
  explicit reconcile operations.
