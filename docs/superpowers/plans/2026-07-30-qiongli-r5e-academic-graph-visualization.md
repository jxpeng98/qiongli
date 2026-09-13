# Qiongli R5E Academic Graph Visualization Plan

Status: in progress — G0 through G5 and G6 automated gates are implemented locally;
packaged acceptance remains

Date: July 30, 2026

Target branch: `feat/r4b-ui-localization-polish`

Baseline: R5D local implementation commit `6dd5ac65`

Roadmap:
`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

## Goal

Turn Academic Graph from a dense inventory of extracted entities into a
visualization-first research workspace that answers three questions without
requiring the user to read tables:

1. what is this project trying to establish;
2. how are sources, ideas, decisions, claims, evidence, and manuscript sections
   connected; and
3. where are the unsupported, contradictory, weak, or missing links?

The graph remains a deterministic projection of explicit project artifacts.
R5E does not ask a model to invent relationships, infer authorship, or rewrite
the canonical academic state.

## Current diagnosis

The current implementation is functional but does not yet satisfy the product
goal:

- the visualization appears after metrics, filters, risk analysis, and revision
  comparison, so the primary canvas is visually secondary;
- the renderer uses a `340px` viewport and a preset `qiongli-layered-v1`
  geometry;
- that geometry assigns every node to one primary layer, places each layer in
  one vertical column, and sorts by type and ID rather than graph topology;
- dense imported projects therefore produce long columns, overlapping edges,
  and weak visual hierarchy even when the underlying query is correct;
- extraction currently has seven graph extractors across eight registered
  artifact paths, while one registered artifact intentionally contributes no
  graph records; the readiness view must distinguish presence from graph
  contribution;
- diagnostics, tables, edge details, risk overlay, revision comparison, path
  finder, and inspector are all expanded in one page, adding text before the
  user has established visual context; and
- selection exists, but there is no neighbourhood isolation, semantic zoom,
  cluster collapse, minimap, navigation history, or edge-first inspection.

The first R5E batch must distinguish a sparse projection from a poor layout.
Changing colours or adding animation before that evidence exists would hide the
actual failure mode.

## Product contract

Academic Graph has three progressive modes:

| Mode | Default content | Purpose |
|---|---|---|
| Overview | project spine, major clusters, coverage and risk summary | understand the project in seconds |
| Explore | selected entity, bounded neighbours, relation controls, minimap | follow academic relationships |
| Inspect | exact source anchor, rationale, confidence, diagnostics and history | verify why a relationship exists |

Overview is the default. Tables, raw IDs, diagnostic codes, edge ledgers, and
revision details remain available as accessible disclosure panels rather than
competing with the canvas.

## G0 — Visualization readiness and representative fixtures

Status: locally implemented; exact packaged-App viewport capture remains

Add a read-only readiness projection before changing the layout:

- report recognized, present, missing, unsupported, and invalid graph source
  artifacts;
- report node and edge counts by academic layer and entity/relation type;
- distinguish `empty-project`, `no-recognized-artifacts`, `nodes-without-edges`,
  `sparse`, `visualizable`, and `bounded/truncated`;
- expose path-redacted source anchors and remediation, never artifact content or
  home-directory paths;
- add three deterministic fixtures:
  - a migrated 1.x project with incomplete canonical artifacts;
  - a medium connected paper project with claims and evidence; and
  - a bounded large project with dense cross-layer relations;
- capture current screenshots and layout metrics at `375`, `768`, `1024`, and
  `1440` CSS pixels; and
- record whether each reported defect is caused by extraction, query bounds,
  layout, or presentation order.

Primary files:

- `packages/qiongli-native/crates/qiongli-project/src/academic_graph.rs`
- `packages/qiongli-native/crates/qiongli-project/src/academic_graph_extract.rs`
- `packages/qiongli-native/apps/qiongli/src/desktop_api.rs`
- `packages/qiongli-app-api/src/schema.ts`
- `packages/qiongli-desktop/src/routes/academic-graph/+page.svelte`

G0 acceptance:

- an imported project never presents a blank or misleadingly healthy graph
  without a readiness explanation;
- a zero-edge graph identifies whether relationships were absent, rejected, or
  truncated; and
- fixture results are deterministic across rebuild and restart.

### G0 implementation record — July 30, 2026

- `AcademicGraphReadinessV1` is a separate read-only native projection. It does
  not enter the canonical graph digest or change graph entity identity.
- App API schema 10 binds readiness to the exact project and projection and
  validates source, node, relation, layer, entity-type, and relation-type
  counts.
- Native classification covers empty, present-but-unrecognized, zero-edge,
  sparse, visualizable, and source-repair cases. The bounded query result adds
  the explicit `bounded-truncated` presentation state.
- Desktop replaces four duplicated metric cards with one compact readiness
  summary, keeps exact source readiness behind a disclosure, hides the
  misleading project-only canvas, and moves the real canvas ahead of risk and
  revision analysis.
- Deterministic tests cover incomplete, unconnected, sparse, visualizable,
  unsupported-source, and bounded-query states. The development fixture is the
  connected medium project; existing bounded index fixtures remain the large
  query authority.
- Local responsive inspection passed at actual CSS content widths 374, 689,
  893, and 1226 pixels with no horizontal overflow, a `26px` non-wrapping
  readiness badge, source rows contained within the viewport, the canvas before
  risk analysis, and no browser warnings or errors.
- Exact `375`, `768`, `1024`, and `1440` packaged-App screenshots remain a G0
  manual acceptance item because the in-app browser chrome constrained the
  available content viewport.

## G1 — Projection completeness for migrated projects

Use G0 evidence to close only proven extraction gaps:

- map supported 1.x artifact names and schemas into the existing 2.x canonical
  graph sources during migration or graph rebuild;
- preserve stable project, node, edge, and source-anchor identity;
- add adapters for supported research-state, decisions, idea funnel, boundary
  review, literature map, evidence ledger, and manuscript claim map variants;
- retain explicit diagnostics for malformed or unsupported legacy structures;
- never derive an academic claim or relation from prose similarity alone; and
- keep rebuild idempotent and rollback-safe.

G1 acceptance:

- the migrated fixture produces the same supported semantic graph as its
  equivalent native 2.x fixture;
- unsupported data remains visible as a diagnostic instead of disappearing;
- rebuilding twice produces the same projection ID and entity identities; and
- migration does not modify the source 1.x project.

### G1 implementation record — July 30, 2026

- The existing bounded legacy adapters already cover the supported `RQ:`
  research-state form, the three-column decision log, and the canonical
  evidence ledger without deriving relationships from prose similarity.
- A two-authority parity fixture migrates one 1.x directory and registers an
  equivalent native 2.x directory under the same explicit project identity.
  Their complete graph snapshots and readiness projections are byte-equivalent.
- The parity fixture contains nodes and reviewed relations, so equality cannot
  pass by comparing two empty projections.
- Existing migration acceptance retains the original 1.x source and excludes
  private or unsupported files. Existing extractor acceptance keeps malformed
  legacy structures visible as sorted diagnostics.
- G1 found no additional supported-schema extraction gap. Exact packaged
  viewport captures remain a separate G0 manual gate.

## G2 — Visualization-first workspace

Reorder and simplify the route:

- place a compact project/view selector and the graph canvas immediately after
  the page heading;
- move metrics into one compact canvas toolbar;
- move risk, revision comparison, path finder, raw node/edge tables, and
  diagnostics into an inspector rail or collapsed lower panels;
- make the canvas height responsive with a useful desktop minimum and a bounded
  mobile fallback;
- keep search and the most-used relationship/layer controls in the toolbar;
- place advanced query controls behind one disclosure;
- preserve the table as the accessible equivalent of the canvas; and
- show a clear empty/readiness state inside the canvas rather than above it.

G2 acceptance:

- the graph is visible without vertical scrolling at a `1024 × 768` content
  viewport;
- the route contains one primary heading and one primary visualization;
- no status pill or toolbar action wraps internally at supported widths;
- keyboard users can move from toolbar to canvas alternative and inspector; and
- reduced-motion mode does not depend on animation to communicate progress.

### G2 implementation record — July 30, 2026

- The route now keeps search, relation, and layer in the primary toolbar. Node
  type and focus direction remain available in one closed-by-default advanced
  filter disclosure.
- The interactive canvas follows the toolbar directly. Its compact header owns
  readiness, result count, fit, and accessible-table actions; the canvas height
  uses a responsive desktop minimum and a bounded mobile fallback.
- Readiness evidence remains visible in compact form after the canvas. Risk and
  revision analysis, path finding, exact node/edge inventories, and diagnostics
  are closed-by-default lower disclosures. The inspector is not rendered until
  an entity is selected.
- Local browser acceptance at a `1024`-pixel window placed the canvas at
  `338px`, already visible inside the browser-constrained `576px` content
  height, with one `h1`, no horizontal overflow, closed secondary panels, and
  non-wrapping readiness and toolbar pills. The focus order reaches the filters,
  canvas actions, accessible table target, and lower disclosures.
- Svelte diagnostics, all 122 Desktop tests, and the static production build
  pass. Exact packaged `375`/`768`/`1024`/`1440` captures remain the manual G0
  and G6 evidence gate.

## G3 — Deterministic topology-aware layout v2

Replace the primary-layer column index with a stable hybrid layout:

- use the research question, contribution, or project node as the project
  spine;
- build deterministic connected components and relation-aware communities;
- apply a seeded, layer-constrained topology layout within components;
- preserve the academic reading direction from literature and ideas through
  argument to manuscript;
- route dense edges with separation and visually de-emphasize non-neighbour
  edges;
- retain stable positions where possible when a revision adds a small number
  of entities;
- collapse disconnected or repeated entities into labelled clusters at
  overview scale; and
- keep `qiongli-layered-v1` as a temporary deterministic fallback until v2
  fixture and performance gates pass.

The layout must consume validated graph data only. Cytoscape is a renderer and
interaction engine, not the authority for semantic identity.

G3 acceptance:

- layout output is deterministic for one projection and viewport class;
- connected entities are visually closer than unrelated entities;
- no accepted fixture degenerates into one unbounded vertical column;
- incremental revisions do not reposition the entire graph without cause; and
- the bounded large fixture reaches first usable render within the agreed
  performance budget.

G3 entry gates and budgets:

- retain the connected six-node development fixture, add a deterministic
  medium fixture at the Desktop query boundary (`100` nodes / `200` edges), and
  add a hard-bound stress fixture (`256` nodes / `512` edges);
- compare canonical geometry, component membership, cluster identity, and
  layout key across repeated runs and shuffled input order;
- when a revision adds no more than five percent new entities, keep at least
  ninety percent of unchanged nodes within one node width of their prior
  position unless their connected component changes;
- budget pure layout computation at `100 ms` for the Desktop boundary and
  `250 ms` for the hard bound on the recorded CI performance runner; and
- budget first usable packaged render at `1 s` for the Desktop boundary while
  retaining `qiongli-layered-v1` whenever a v2 validation or budget gate fails.

### G3 implementation record — July 30, 2026

- `qiongli-topology-v2` now consumes only the validated bounded query result.
  It derives sorted undirected connected components, stable component and
  community identities, and one research-question/contribution/project-first
  spine per component without mutating or inferring semantic graph records.
- Four deterministic barycentric sweeps order nodes inside the fixed academic
  reading direction. Compact, standard, and wide viewport classes use bounded
  multi-lane bands, so a dense layer no longer degenerates into one vertical
  column. Stable hashes replace stochastic physics and make shuffled input
  byte-equivalent.
- When a new result retains at least ninety percent of the previous result in
  the same project and viewport class, existing lane/row slots are preserved
  with bounded growth slack. The 100-to-105 node revision fixture keeps at least
  ninety percent of unchanged nodes within one node width.
- Every community has a deterministic non-semantic overview cluster and
  aggregated cluster edges. Cytoscape switches to these labelled clusters below
  the overview zoom threshold and returns to exact semantic records above it;
  the synchronized table remains the accessible authority at every zoom level.
- Stable route offsets separate dense parallel edges. Focusing a semantic node
  retains its closed neighbourhood and de-emphasizes unrelated semantic nodes
  and edges without changing their source identity.
- `qiongli-layered-v1` is now an explicit truthful fallback only. A topology
  computation that exceeds the 100/200 desktop or 256/512 hard-bound budget, or
  fails internally after input validation, returns a labelled fallback layout;
  dangling semantic endpoints still fail closed before either visualizer runs.
- Tests cover shuffled equality, component/community/cluster identity,
  connected-versus-unrelated distance, multi-lane density, incremental
  stability, explicit fallback, dangling endpoints, responsive viewport
  classes, overview zoom collapse, and the `100 ms` / `250 ms` pure-layout
  budgets. All 131 Desktop tests, Svelte diagnostics, and the static production
  build pass.
- Local browser acceptance at a `1024`-pixel window shows `Topology layout v2`,
  one connected-group summary, a first-view canvas at `338px`, no horizontal
  overflow or wrapping pills, and no warning/error logs. Selecting a node
  through the accessible table synchronizes focus, the canvas announcement,
  and the exact evidence inspector. Exact packaged multi-viewport captures
  remain the G0/G6 manual gate.

## G4 — Exploration and inspection

Add graph-native interaction:

- search-to-focus with highlighted matches;
- one-click neighbourhood isolation with inbound, outbound, and both
  directions;
- bounded depth control;
- expand/collapse for communities and repeated source/evidence clusters;
- node and edge selection with an inspector rail;
- back/forward focus history and reset-to-overview;
- fit selection, fit graph, zoom controls, and minimap;
- relation and layer visibility toggles; and
- keyboard-operable equivalents for every canvas-only action.

Selection, focus, filters, and expansion state are ephemeral UI state. They do
not mutate project artifacts or create graph relations.

### G4 implementation record — July 30, 2026

- Academic Graph query v1 now carries a bounded `maxDepth` of one through three
  only when an exact focus node is present. Rust, the CLI `--max-depth` option,
  the full-project MCP contract, App API, Desktop, and the development
  transport share the same deterministic directional breadth-first traversal.
  Relation and layer filters constrain traversal rather than hiding
  disallowed edges after expansion.
- Search uses deterministic, bounded label and canonical-ID matches from the
  exact loaded projection. Suggestions lead directly to one stable node,
  preserve focus for consecutive searches, clear the text constraint before
  neighbourhood isolation, and highlight matching nodes without creating or
  changing graph records.
- The compact exploration toolbar owns previous/next focus history, inbound,
  outbound, or bidirectional traversal, depth, explicit community
  expand/collapse, and reset-to-overview. Forward history is discarded after a
  branch, history is capped, and every state is reset on project or view
  changes.
- Community collapse is renderer-only. Collapsed semantic nodes and incident
  edges are replaced by the existing deterministic overview clusters and
  aggregate edges; expanded communities retain their exact semantic records.
  The table remains authoritative at every expansion and zoom state.
- Cytoscape semantic edges are now selectable and synchronize with the exact
  inspector. Fit-selection, fit-graph, zoom-in, zoom-out, a live percentage,
  and a projection-derived minimap share one viewport callback. The buttons,
  node table, relation list, and inspector are keyboard-operable equivalents
  for visual canvas actions.
- Component and pure-state tests cover depth normalization, two-hop native
  traversal, search ranking, branching history, edge selection, explicit
  community collapse, zoom controls, viewport feedback, and minimap
  accessibility. All `135` Desktop tests, `22` App API tests, Svelte
  diagnostics, and the static production build pass. The full `qiongli`,
  `qiongli-project`, and `qiongli-runtime` Rust suites pass after regenerating
  the verified embedded resource-pack lock for the updated MCP contract.
- Local browser acceptance at a `1024`-pixel window has no horizontal overflow
  or internally wrapping exploration controls. It verifies consecutive
  search-to-focus, back/forward history, one-hop outgoing isolation, two-hop
  expansion, `0/2` through `2/2` community controls, reset-to-overview, and
  synchronized zoom/minimap feedback from `89%` to `108%`. Exact packaged
  `375`/`768`/`1024`/`1440` captures remain the G0/G6 manual gate.

## G5 — Academic visual language

Make meaning readable without relying on colour alone:

- give papers, questions, decisions, claims, evidence, contributions, and
  manuscript sections distinct shapes or icons;
- encode relation families with line style and arrow treatment;
- encode confidence, contradiction, missing evidence, and stale source state
  with shape, stroke, label, and accessible text;
- introduce semantic zoom so labels and secondary detail appear only when
  useful;
- keep the legend compact, interactive, and synchronized with visible layers;
  and
- use tooltips only for supplementary information, never required controls.

### G5 implementation record — July 30, 2026

- One deterministic visual-language module maps all fifteen supported node
  types to a visible type mark and a Cytoscape-supported shape. Research
  questions and decisions use diamonds, claims and contributions use
  hexagons, evidence uses a barrel, gaps use a triangle, methods use a
  pentagon, and document-like records remain rectangular. Colour is only a
  secondary layer signal.
- All twenty-five relations map exhaustively to evidence, challenge,
  provenance, structure, or development families. Each family has an explicit
  solid, dashed, or dotted line and triangle, tee, diamond, or square arrow
  treatment. Exact relation text remains available in the synchronized
  relation inventory.
- Validated edge confidence, inference strength, status, and explicit risk
  kinds now survive the presentation layout boundary. Low or unknown
  confidence receives a non-colour line treatment; contradictions, rejected
  relations, evidence gaps, and low-confidence records retain the exact risk
  overlay and accessible text.
- Semantic zoom has three bounded presentation levels: overview clusters below
  the existing threshold, type marks at reduced semantic zoom, concise labels
  at normal zoom, and canonical IDs only at detail zoom. It does not change
  entity identity or query results.
- A closed-by-default visual key reports only node types and relation families
  present in the bounded result. Every entry is a keyboard-operable pressed
  control; hiding a node also hides incident canvas edges without deleting the
  semantic record. The deterministic fallback map and accessible table reuse
  the same type marks and relation-family semantics.
- The deterministic fallback now preserves the relation family's arrow
  treatment as well as its line style: evidence/development use a triangle,
  challenge uses a tee, provenance uses a diamond, and structure uses a
  square. Renderer failure therefore does not collapse distinct academic
  relationships into one generic arrow.
- The current native source contract exposes present, missing, invalid, and
  unsupported sources but no explicit `stale` state for a graph entity. G5
  intentionally does not infer staleness from timestamps or visual proximity.
  A stale-source visual remains gated on an authoritative native source-state
  field and is recorded for G6 contract review.
- Svelte diagnostics report zero errors and warnings; all `140` Desktop tests
  and the static production build pass. Browser acceptance at the available
  `1024`-pixel window verifies the compact closed legend, expanded shape and
  line keys, keyboard visibility toggles, synchronized node/edge hiding,
  distinct canvas shapes, semantic labels, and minimap. The global operation
  notice also appears as a right-aligned non-layout banner and automatically
  expires. Exact packaged multi-viewport evidence remains the G6 manual gate.

## G6 — Performance, accessibility, and release acceptance

Qualify the complete visualization:

- deterministic unit tests for readiness, clustering, geometry, and stable
  revision placement;
- Rust extraction and migration parity tests for all supported source variants;
- component tests for toolbar, disclosure, selection, empty, sparse, and
  truncated states;
- screenshot and interaction acceptance at the four supported viewport widths;
- keyboard, focus order, screen-reader table parity, contrast, and
  reduced-motion checks;
- cancellation and stale-result rejection during rebuild or rapid project
  switching;
- bounded node/edge limits with an explicit summary when the overview is
  clustered or truncated; and
- restart acceptance proving the project graph rebuilds to the same semantic
  and layout identities.

R5E is complete when a user can open each accepted fixture, identify the
research spine, follow a claim to its evidence and manuscript location, find
one risk or missing link, and open the exact source anchor without reading the
raw node and edge inventories.

### G6 automated acceptance record — July 30, 2026

- Read-only Academic Graph work now carries a monotonic request generation in
  addition to project, revision, and projection identity. The generation gate
  rejects late A results across an A-to-B-to-A switch before `AppState` applies
  the event. A guarded event also cannot clear or replace a newer accepted
  result.
- `AppState.loading` is backed by an active-operation count, so one completed
  concurrent read cannot re-enable controls while another read remains in
  flight. Existing callers retain the same default event-application path.
- The graph canvas is a real keyboard focus target. Plus/equal and
  minus/underscore zoom, zero fits the graph, and F fits the current node or
  relation selection. The existing visible buttons and exact table remain
  equivalent controls; a focus outline and screen-reader help describe the
  shortcuts.
- The deterministic renderer fallback retains relation selection as well as
  node selection. Each visible relation has a non-colour accessible label,
  keyboard activation, selected state, and synchronization with the exact
  artifact/source-anchor inspector. A project that is not graph-ready exposes
  a direct Research Library recovery action instead of ending at explanatory
  copy.
- A component gate proves a truncated query is labelled `Bounded` with the
  exact omission and narrowing explanation instead of being presented as a
  complete graph. The global reduced-motion rule collapses animation and
  transition duration while Cytoscape layout and viewport changes remain
  non-animated.
- A new Rust restart gate recreates `ProjectStateService` from the same private
  configuration root and proves projection ID, index ID, project revision,
  node/edge counts, and the complete bounded query result are unchanged.
  Existing shuffled-input, portable round-trip, migration parity, layout
  identity, incremental placement, and hard-bound tests remain green.
- Final automated results: Svelte diagnostics `0` errors and `0` warnings;
  Desktop `179/179`; App API `29/29`; `qiongli-project` `161/161`; Rust format
  check; and the static production build all pass. The App API runner was
  invoked directly with the repository's locked Node script after the package
  manager wrapper attempted an unavailable network signature/version check;
  no security verification was disabled.
- Browser acceptance at the available `1024`-pixel window verifies focusable
  canvas keyboard zoom from `89%` to `108%`, zero-to-fit back to `89%`, and
  F-to-selected-node at `250%`, with no application warnings. Browser tabs and
  the local server were closed after inspection.
- R5E is not marked complete. The signed packaged App must still capture and
  inspect `375`, `768`, `1024`, and `1440` widths, keyboard focus order,
  contrast, reduced motion, restart, and representative migrated/medium/large
  fixtures. Entity-level stale-source visuals also remain gated on an
  authoritative native stale field rather than inference.

## Execution order

```text
G0 readiness + fixtures
  -> G1 migrated projection parity
  -> G2 visualization-first page shell
  -> G3 topology-aware layout v2
  -> G4 exploration
  -> G5 academic visual language
  -> G6 packaged acceptance
```

G0 and G1 establish whether content exists. G2 through G5 must not compensate
for missing semantic data with inferred relationships.

## Explicit non-goals

R5E does not:

- invoke a model backend to generate graph entities or edges;
- make Qiongli a graph database server;
- mutate source artifacts from a filter, selection, or layout action;
- treat visual proximity as academic evidence;
- claim unsupported legacy content was migrated;
- remove the accessible table representation; or
- block existing R5C/R5D manual host acceptance.
