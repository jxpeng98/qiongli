# Design: truthful Graph v1 continuity

## Boundary

Reuse the existing Graph v1 projection and Desktop canvas. The defect is at two
shared inputs: readiness classification treats structural records as semantic,
and the bundled workflow does not give legacy projects an executable
normalization contract. Fix those owners only.

## Native readiness

`AcademicGraphReadinessV1::from_graph_and_last_successful` keeps the public
response shape unchanged.

- `semantic_node_count` excludes `Project` and `Artifact` nodes.
- A local `semantic_edge_count` excludes `Contains` relations.
- `classify` receives semantic counts and uses them for no-record, no-relation,
  and sparse/visualizable decisions.
- `relation_count`, `relation_counts`, graph nodes, and graph edges remain total
  projection counts so Graph v1 serialization and App API invariants do not
  change.

One focused Rust fixture covers structural-only containment. Existing fixtures
use a scholarly relation when testing sparse/visualizable topology.

## Desktop presentation

`AcademicGraphReadinessPanel` derives the displayed semantic relation total from
the existing `relationCounts` array by excluding `contains`. No API field or
frontend store is added. Copy changes direct repairable states to the existing
Run in client and rebuild journey.

## Plugin/Skill continuity

Add `content/workflow/references/academic-graph-continuity.md` and link it from
the root Qiongli Skill. `academic-context-maintainer` remains the single stage-
close owner.

The reference defines:

1. inspect snapshot/doctor diagnostics;
2. select only reviewed canonical records;
3. preserve prose and IDs, append/update minimum template sections;
4. preview exact writes;
5. refresh/rebuild;
6. verify semantic nodes, non-containment relations, diagnostics, and revision.

Canonical artifact relations remain the preferred authoring surface. The
hashed semantic sidecar is not exposed as a manual authoring requirement.

## Compatibility and safety

- No public schema version changes.
- No migration writes are added to the native runtime.
- Legacy repair is host-authored and preview-before-apply; existing prose is
  retained and unsupported evidence stays explicit.
- Generated Plugin/Skill trees remain outputs and are checked only in a staging
  directory.

## Roadmap and rollout

Update only the immediate product-control lane to insert this user-reported
Graph v1/Plugin-quality repair after the in-flight GOV-413 closeout and before
the remaining M1 queue. The master Kernel/Graph v2 task inventory stays
deferred and unchanged.

Rollback is a normal code revert: readiness returns to the previous classifier
and the Skill reference is removed. No user project data migration is involved.
