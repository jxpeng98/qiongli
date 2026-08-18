# Approved execution train after packaged content editability

Only one Trellis implementation task is active at a time. These two follow-on
outcomes start automatically after the preceding task is accepted; routine
implementation, commits, push, PR, CI repair, and local package work require no
new authorization.

## Phase B — Typography and graph-backed project continuation (absorbed here)

The user expanded the active task before PR creation, so this phase is now part
of the same exact-source implementation and package gate rather than a separate
PR.

### User outcome

Research Library becomes the place to choose a project and continue it. The
continuation surface is the existing source-bound Academic Graph v1, giving the
project an Obsidian-like connected view without inventing a Graph v2 or editable
edge model.

### Minimum change

- Define one Geist/system sans token in `app.css`, bridge shadcn to that token,
  and make `button`, `input`, `select`, and `textarea` inherit the same
  family and compatible font metrics. Add no font dependency.
- Keep list-row selection, but replace the ambiguous no-op affordance with one
  explicit “Continue research” primary action for the selected project.
- Route that action through `ProjectWorkspaceState` to
  `/academic-graph?project=...`; preserve project selection across Overview,
  Artifacts, Captures, Graph, Timeline, and Run in client.
- Reuse Academic Graph's current Cytoscape map, search, focus history, path
  finder, risk overlay, entity inspector, and revision-bound artifact
  preview/open. Do not build a second mini graph in Research Library.
- Turn existing readiness text into a real safe next action:
  - empty project -> Run in client to create canonical research artifacts;
  - unrecognized artifacts -> inspect Artifacts;
  - nodes without edges or sparse topology -> Run in client to enrich explicit
    source-bound relations;
  - stale projection -> use the existing refresh/rebuild path.
- Keep graph nodes/edges derived and read-only. Arbitrary notes, hand-drawn
  links, inferred-edge promotion, and Graph v2 remain deferred until Kernel
  authority exists.

### Acceptance

- Equivalent page, control, and button labels render with one intended font in
  the packaged WebView.
- A single-project Library row is no longer the end of the journey; the primary
  action reaches its connected graph with the same selected project.
- Selecting a supported node/edge reaches its exact revision/projection-bound
  artifact anchor; workspace navigation returns without losing selection.
- Empty/sparse states explain the missing canonical input and expose one working
  existing next action.
- Loading, empty, error, keyboard focus, narrow, and tall states remain usable
  with no competing vertical scroll owners.
- Focused Desktop tests, App checks/build, native unchanged-contract checks, and
  a new exact-source macOS acceptance/manual package pass.

## Phase C — GOV-401 through GOV-404

### User outcome

Roadmap state has one machine-readable authority. Historical checkboxes stop
acting like status evidence, and a generated current index makes the program
state reviewable without manually reconciling 233 task lines.

### Minimum change

- Add one versioned program ledger containing all 233 roadmap task IDs exactly
  once with required fields:
  `id`, `state`, `owner`, `dependencies`, `evidence`, `commit`,
  `run`, `updated_at`, and `blocker`.
- Restrict state to `proposed`, `active`, `accepted`, `blocked`,
  `deferred`, or `superseded`.
- Require non-empty repository evidence plus exact commit/run identity before
  `accepted`; a checked Markdown box alone never qualifies.
- Add one Python-stdlib validator/generator with `--check` mode. It validates
  schema, unique IDs, dependency closure, accepted evidence, deterministic
  ordering, and generated output freshness.
- Generate one compact current roadmap index grouped by milestone/workstream
  from the ledger. Keep the long-term roadmap's descriptions and ordering; the
  generated index owns live status presentation.
- Add the one validator/check to the closest existing CI owner; do not create an
  umbrella governance framework or duplicate every roadmap description in JSON.

### Acceptance

- The ledger and roadmap contain the same 233 unique task IDs.
- Invalid state, duplicate/missing ID, unknown dependency, accepted-without-
  evidence, or stale generated index fails locally and in CI.
- The generated index is byte-deterministic and clearly distinguishes accepted,
  active, blocked, deferred, proposed, and superseded work.
- Existing accepted EVAL items cite the protected merge/post-merge evidence;
  unresolved items are not upgraded from checkboxes.
- `GOV-401`–`GOV-404` alone are marked accepted after their exact-head PR and
  CI evidence exists; later GOV/PLT/SEC items remain unchanged.

## Integration order

1. Current PR: receipt-bound editable Plugin/Skills variant plus typography and
   Research Library -> Academic Graph continuity.
2. Next PR: machine-readable program ledger and generated index.

Each PR starts from the latest protected `2.x`, freezes its own exact source,
waits for required CI, and does not claim release/publication authority.
