# Current Graph v1 continuity audit

Date: 2026-08-18

## Observed product path

- Research Library already preserves project selection into Academic Graph.
- Academic Graph already provides Cytoscape navigation, search, focus history,
  path queries, clustering, risk review, and source-bound inspection.
- The native projection recognizes ten graph source paths and has structured
  extractors for the canonical research-state, decision-log, boundary-review,
  idea-funnel, literature-map, evidence-ledger, and claim-map formats.

## Reproduced failure shape

One registered legacy project produced:

- 7 nodes;
- 6 edges;
- only artifact nodes beyond the project node;
- only `contains` relations;
- 5 diagnostics for missing stable fields/files or unsupported legacy tables.

The readiness implementation counted every non-project node as semantic and
used total edge count for topology. The result was a sparse-looking graph rather
than an explicit graph-structure repair state.

## Existing reusable mechanisms

- Canonical bundled templates already contain the supported headings, columns,
  stable IDs, evidence limits, and relation-bearing records.
- Legacy Markdown can retain its prose while gaining an appended supported
  section; the extractors scan for the canonical table/field shape.
- `qiongli project graph doctor`, graph snapshot/query, App rebuild, and Full
  MCP graph tools already provide post-write verification.

## Decision

Fix readiness classification and Skill output discipline. Do not build another
graph UI, parse arbitrary prose as truth, expose manual hashed sidecar authoring,
or start Graph v2/Kernel work.
