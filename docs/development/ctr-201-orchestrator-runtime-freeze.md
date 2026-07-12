# CTR-201F accepted-source orchestrator runtime inventory closure

Status: **source-oracle inventory closure; protected integration requires
exact-head CI**

CTR-201F is an engineering child of the Qiongli 2 migration task `CTR-201`.
It freezes the observable accepted-source orchestrator control flow that was
left open by CTR-201C and explicitly dispositions runtime dimensions that
cannot be executed safely or do not exist in the accepted product. Together
with CTR-201A-E, it closes the CTR-201 source-oracle inventory. It does not add
a canonical academic research Task ID.

## Dependencies and program order

CTR-201F depends on:

- the accepted `v1.19.0-beta.1` tag, peeled commit, and immutable A8 migration
  baseline;
- CTR-201A's master semantic ledger and frozen-source bindings; and
- CTR-201C's declared/static orchestrator contract.

CTR-201F must not declare its parent CTR-201 as a dependency because the parent
exit gate depends on its required children. After CTR-201F closes that gate,
CTR-202 Contract v2 completion and FND-202 resource-pack implementation become
separate, unblocked successors. Neither is implemented by CTR-201F.

CTR-201D does not establish archive-member, published ZIP/TAR, or plugin-wrapper
parity. That remains a downstream governance boundary rather than a CTR-201F
deliverable or CTR-201 exit dependency.

## Capture contract

The runtime inventory is derived from the accepted 1.x source rather than the
mutable 2.x checkout. It must bind the accepted tag, commit, source digests,
CTR-201C schema and payload digests, Python version, fixture matrix, normalized
outcomes, and ordered case-manifest root.

The closed matrix must classify the accepted orchestrator's observable control
flow for:

- task planning and missing/satisfied prerequisite behavior;
- built-in and custom profile selection, including invalid profiles;
- solo, duo, and triad routing as the accepted handler actually implements
  those modes;
- primary, reviewer, verifier, revision, and fallback control flow;
- `code_build` focus routing and its topic-less standard/advanced execution
  branches, with the strict topic route explicitly dispositioned at its
  state-writing Stage-I boundary;
- team and worker configuration, concurrency, generic fallback, thresholds,
  merge/review failures, degradation, and block behavior;
- bridge session-command passthrough, experience replay advice, and the
  accepted absence of durable task/team resume or cancel APIs; and
- declared quality gates and their actual prompt/artifact-level enforcement
  boundary.

Every required matrix cell must be captured, marked not applicable with a
machine-checkable reason, or linked to an approved inventory-only disposition.
Missing cells and free-form exceptions fail closed. An inventory disposition
closes classification only; it is not evidence that the behavior works.

## Isolation and normalization

Canonical extraction runs in a temporary accepted-source tree under Python
3.12. It uses deterministic bridge and MCP fakes, temporary HOME and project
roots, fixed locale and timezone, an environment allowlist, and before/after
manifests for mutable fixture roots. The worker denies child processes,
network access, and SUT writes; the parent launches only Git and the isolated
Python capture workers. It must not read or write real user configuration,
host plugin directories, Marketplace caches, or generated distribution
payloads. The declared injection boundary also includes CLI availability,
fixed UUID identity, and the MCP handler's orchestrator constructor; it is not
limited to bridge objects alone.

The checked artifact must not contain machine-local absolute paths,
secret-shaped data, raw temporary roots, nondeterministic timestamps, callable
representations, or platform-specific separators. Normalization may remove
approved environmental noise but must retain semantically meaningful streams,
status, ordering, fallback, and filesystem effects. Sequential calls retain
their accepted happens-before order; only calls observed on executor threads
inside one concurrent cohort are canonically ordered.

The checked matrix contains **44 cases**: one accepted A8 oracle case and 43
bounded accepted-source runtime cases. Those cases close six required behavior
dimensions and bind six explicit disposition decisions. The counts are
inventory-closure evidence, not a parity score.

Windows and macOS run closed-schema and semantic validation against the checked
portable artifact. Canonical runtime re-extraction runs only in the Ubuntu full
tier. Passing that matrix proves portable validation, not cross-platform
orchestrator runtime parity.

## Explicit behavior boundaries

CTR-201F does not launch a real Codex, Claude, or Antigravity agent, call a live
model/provider API, or establish production process isolation. Deterministic
bridge and MCP fakes prove accepted handler branching only.

The inventory must preserve these accepted-product boundaries rather than
silently upgrading them into capabilities:

- `doctor` is advisory and is not called or enforced by
  `_tool_task_run(run_agents=true)`;
- the accepted solo route must be recorded as observed rather than described as
  strict single-agent enforcement;
- recognized worker adapter names may still fall back to `generic_prompt`;
- topic-less `code_build` routes are captured, while the strict topic route is
  dispositioned because it forces project guidance/trace writes and Stage-I
  task-run integration outside the no-write fixture boundary;
- public cancellation and real external-session resume are unavailable;
- timeout or interruption behavior that cannot be safely reproduced requires
  an explicit disposition; and
- quality gates may be declarations, prompt propagation, or artifact-existence
  checks rather than semantic evaluators of academic quality.

## Source-controlled evidence

The CTR-201F slice owns:

- `tooling/migration/ctr-201-orchestrator-runtime.json`;
- `tooling/migration/ctr-201-orchestrator-runtime.schema.json`;
- `tooling/scripts/extract_ctr_201_orchestrator_runtime_inventory.py` and its
  stable root wrapper under `scripts/`;
- `tests/test_ctr_201_orchestrator_runtime_inventory.py`; and
- the digest, status, count, and completion binding in the mutable CTR-201
  master ledger.

The immutable A8 baseline, CTR-201C static artifact, canonical academic
content, generated plugins/packages, marketplace catalogs, and native Rust
product source are outside this slice.

## Exit gate

CTR-201F closes the source-oracle inventory only when all of the following
pass:

1. every required orchestrator runtime category is captured or explicitly
   dispositioned, with no unclassified required gap;
2. the artifact and schema are recursively closed, deterministic,
   digest-bound, path-safe, and free of secrets or unstable representations;
3. source, CTR-201C, case order, per-case digests, aggregate roots, and master
   bindings fail closed under independent mutation tests;
4. two isolated canonical extractions produce byte-identical normalized output;
5. the CTR-201 master ledger binds CTR-201F, reports the source-oracle inventory
   complete, and retains CTR-202 and FND-202 as unimplemented successors;
6. Ubuntu full CI re-extracts the accepted runtime with Python 3.12; Windows and
   macOS validate the checked artifact without executing the runtime extractor;
   and
7. exact-head protected-branch CI passes without changing release, plugin, or
   Marketplace outputs.

## Claim boundary

Allowed completion language:

> CTR-201F captures and explicitly dispositions the accepted-source
> orchestrator control-flow inventory under deterministic fixtures. It closes
> the CTR-201 source-oracle gate, not real agent/provider parity, a Rust
> orchestrator, or cross-platform runtime parity. CTR-202 and FND-202 are
> unblocked but remain unimplemented.

Do not describe CTR-201F as:

- production agent execution, model/provider integration, or native worker
  dispatch;
- strict solo semantics, public cancellation, real session resume, or semantic
  academic quality evaluation;
- a Rust orchestrator, Capability Contract v2 completion, or an embedded
  resource pack;
- Tier 1 runtime parity, clean-machine zero-runtime acceptance, or a complete
  Qiongli 2 vertical slice; or
- an installable or published alpha, plugin, Marketplace package, release
  artifact, tag, registry update, or authorization to publish.
