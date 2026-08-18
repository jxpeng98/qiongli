# Completeness and graph audit

Audit baseline: latest protected `2.x` at
`82b10c1ee9541785dabdc77bd56b0e1c0894f93e`, 2026-08-18.

## Product chain

| Surface | Evidence | Status |
| --- | --- | --- |
| Packaged CLI | Existing exact-source macOS receipt and copied-binary control-plane checks | Implemented and canonically proven |
| Standalone Skills | Preview/apply/verify/update/remove plus receipt-owned variants | Implemented; customized packaged round trip not yet proven |
| Codex Plugin | Real Codex `0.147.0` isolated install/cache/Full MCP launch passes | Canonical complete; customized cache bytes not yet proven |
| Claude Plugin | Real Claude Code `2.1.231` strict isolated install/cache/Full MCP launch passes | Canonical complete; customized cache bytes not yet proven |
| Lite/Full MCP | Canonical binaries, declarations, empty-PATH launch and 32-tool Full inventory pass | Complete; must remain immutable under customization |
| App editor | Native textarea, preview/confirm/reset, CAS revision/digest, canonical/customized identity | Implemented; package-level end-to-end cycle missing |

Focused local evidence rerun during planning:

- `qiongli-config` WorkflowVariantStore tests: 3 passed.
- `qiongli-content` workflow override test: 1 passed.
- Codex bundle tests: 3 passed, 1 real-Host test ignored by default.
- Claude bundle tests: 4 passed, 1 real-Host test ignored by default.
- client activation tests: 3 passed.
- App API: 32 passed.
- Desktop: 247 passed.
- capability contract v2: valid.
- isolated real Codex customized-capable install path: canonical test passed.
- isolated real Claude strict install path: canonical test passed.

## Graph chain

| Layer | Existing capability | Gap |
| --- | --- | --- |
| Canonical sources | Manifest, eight structured artifacts, reviewed semantic links | New projects begin empty; workflow must produce exact structured artifacts |
| Projection | Stable node/edge IDs, bounds, diagnostics, deterministic rebuild | Semantic nodes are not structurally linked to their known source artifact |
| Scholarly relations | Literature, claim/evidence, manuscript, and explicit-link extractors | Intentionally absent when no reviewed source relation exists |
| App | Cytoscape, search, focus, path, inspector, risk/revision overlays, table fallback | A valid but weakly related project can look fragmented |
| Research Library | Project selection, portfolio topology, route to Academic Graph | One-project portfolio is not the local content graph; action wording is indirect |
| Plugin/Skills | Templates mostly match extractors and stable IDs are documented locally | No shared post-write graph refresh/readiness contract |

## Root-cause conclusion

The missing Obsidian-like continuity is not a missing graph renderer. It is the
absence of a deterministic structural link between each canonical artifact and
the stable semantic records already extracted from it, combined with a missing
Workflow handoff that keeps graph-bearing artifacts current. Adding source
containment once in the shared projection is smaller and safer than teaching
every extractor or UI caller to invent relations.

Arbitrary Markdown wikilinks, full-text inference, editable edges, and Graph v2
would expand the trust boundary and are not required for the reported outcome.
