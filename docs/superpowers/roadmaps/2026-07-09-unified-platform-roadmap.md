# Qiongli Unified Platform Roadmap

> **Transition status (updated July 12, 2026): superseded for 2.x execution.**
> Gate 0, Stage 1, and canonical-source evidence remain historical 1.x input.
> AC1 and RC1 enforcement is cancelled as a migration, CI, and release
> requirement; references below are optional design history only. The former
> Python-Full stable path in
> Stages 2-5 is replaced by the Rust-native 2.x program in
> `docs/superpowers/roadmaps/2026-07-10-qiongli-2-rust-native-platform-roadmap.md`.
> The immediate handoff is defined in
> `docs/superpowers/plans/2026-07-10-qiongli-1x-closeout-and-2x-native-bootstrap.md`.
> Retained stage text below is historical design input, not authority to ship
> the previously planned `v1.19.0-beta.2`, RC, or stable release train.

## Purpose

Qiongli now has a self-contained Rust Marketplace Lite MCP runtime and a
Python-backed Full CLI runtime. This roadmap established the contracts, target
model, installation vocabulary, diagnostics, and release evidence needed to
make them profiles of one platform. The July 10 transition decision changes
the implementation destination: the shared control plane remains, but active
product development moves to one Rust-native core after the final 1.x beta.

This roadmap covers the work after `v1.18.0-beta.2`. It starts with a mandatory
Lite MCP functional-closure gate, then builds the shared control plane required
for platform convergence.

Execution plans:

- `docs/superpowers/plans/2026-07-09-lite-mcp-functional-closure.md`
- `docs/superpowers/plans/2026-07-10-capability-contract-v2-pilot.md`
- `docs/superpowers/plans/2026-07-10-capability-contract-v2-configuration.md`
- `docs/superpowers/plans/2026-07-10-capability-contract-v2-literature-planning.md`
- `docs/superpowers/plans/2026-07-10-qiongli-1x-closeout-and-2x-native-bootstrap.md`

Successor roadmap:

- `docs/superpowers/roadmaps/2026-07-10-qiongli-2-rust-native-platform-roadmap.md`

## Execution Update — July 10, 2026

`v1.18.0-beta.3` completed the Gate 0 release. Stage 1 is active on `dev`. The
initial `qiongli_literature_export_evidence` pilot established the versioned
registry, canonical schemas, semantic errors, runtime-declaration drift checks,
Lite/Full golden calls, and required CI validation. The configuration batch
adds `qiongli_config_status`, `qiongli_save_provider_config`, and
`qiongli_configure_provider`, with `qiongli_open_config_wizard` represented as a
compatibility alias. The literature-planning batch then adds
`qiongli_literature_status` and `qiongli_search_plan`, converging Lite on the
rich hybrid plan already exposed by Full. Coverage is now explicitly `6 / 23`
canonical capability records and `7 / 24` public names; the registry remains
`pilot`, not complete.

The configuration batch also closes contract-relevant security differences:
Full provider configuration now fails closed on malformed persisted JSON and
uses atomic replacement with Unix mode `0600`; Rust Lite Windows writes use a
protected current-user-only DACL, while the legacy Node MCPB fails closed and
remains read-only on Windows. Redacted status output allowlists public search
settings, and the Full loopback wizard enforces a bounded lifetime, request
limits, single-use submission, and no-store response headers. Runtime Windows
CI is the remaining acceptance gate for DACL and reparse-point behavior.

Provider configuration path resolution is also shared: an explicit
`QIONGLI_CONFIG_HOME` must be a fully qualified absolute path or use portable
`~` / `~/...` home notation; rooted or drive-prefixed home suffixes and other
relative values fail closed, and the default is always derived from the
platform user home rather than the server working directory.

The literature-planning batch separates configured from active providers,
strictly normalizes legacy search-plan arguments, preserves provider/native
provenance, propagates corrupted config as a redacted tool error, and brings
the capability contract validator into staged release preflight. Provider
opt-out now propagates through command environments, diagnostics, and Full
search execution without a legacy network fallback; deprecated camel-case year
filters remain present beside canonical snake-case keys during the compatibility
window.

A subsequent final-beta audit found unresolved provider-disable, legacy Node
fail-closed/atomic-write, disabled-credential export, active-status, redaction,
and compatibility-alias cases. The preceding paragraphs describe the intended
Stage 1 behavior, not accepted release evidence. These findings are blocking
items in Phase A1 of the 1.x closeout plan and must be closed before
`v1.19.0-beta.1`.

The AC1 academic-code and RC1 repository-policy designs are retained only as
optional guidance. Neither governs the active Rust migration or blocks CI,
pull requests, or release promotion.

Stage 1 freeze also records the machine-readable 1.x-to-2.x baseline plan at
`tooling/migration/qiongli-1x-baseline-plan.json`. It references the existing
Capability Contract v2 registry and golden-call fixture instead of duplicating
their tool records, and makes `qiongli-testkit` the read-only 2.x consumer.

## Execution Update — July 9, 2026

Gate 0 implementation and local repository-wide acceptance are complete on
`dev`. The implemented closure includes all twelve dispatcher
handlers and safe calls, real bounded five-provider search, deterministic
deduplication and diagnostics, a tokenized single-flight loopback wizard,
bounded Zotero Connector/Companion probes, preview-only routing, Content-Length
stdio support, Lite/Full semantic projections, locked Rust CI gates, and
machine-readable current-host identity.

Rust Lite and the MCPB component are now `0.2.0-beta.2`; the repository product
version remains `1.18.0b2` until the separate `v1.18.0-beta.3` release task.
Release index, artifact manifest, and acceptance receipt generation record both
version lines, the contract version, and the native target triple explicitly.

Local acceptance passed the Rust gates, Lite/Full conformance, legacy Node
reference suite, packaging and launch smoke, the complete Python regression
suite, materialized-distribution audit, maintainer beta smoke, documentation
build, and strict validation (`6203 passed, 0 failed, 0 warnings`). No commit,
tag, release, or external marketplace mutation was performed as part of Gate 0
execution.

The current-host beta policy is intentionally narrower than Stage 4: staged
MCPB compatibility, binary sidecars, artifact records, and human release notes
identify the actual build target. Release postflight uploads matching beta
assets but does not advance generic Codex or Claude marketplace dist refs while
the native policy remains `current-host-only`. Native matrix selection,
signing, provenance, and install-time variant enforcement remain Stage 4 work.

## Decision Summary

Qiongli will use a contract-first modular monorepo architecture:

- `v1.19.0-beta.1` is the final planned Python-led 1.x beta and the migration
  oracle; Python Full receives critical fixes only after that accepted tag.
- Rust Marketplace Lite remains the self-contained, no-user-runtime,
  marketplace-safe subset during the transition, then becomes a profile of the
  shared Rust-native core.
- Qiongli 2 migrates Full CLI, MCP, project writes, local agents, orchestration,
  guidance, validation, installation, update, and desktop management to Rust.
- Shared contracts, platform targets, generated install plans, and conformance
  tests remain the migration control plane and compatibility oracle.
- `v2.0.0-alpha.1`, not beta, begins the native release train; beta requires
  full parity, data migration, zero-runtime, integration, and native-matrix
  evidence.
- A hosted control plane is not part of this roadmap. If a future team or web
  product requires one, its service implementation belongs in a separate
  repository; this repository should retain open contracts, local runtimes,
  SDKs, and platform adapters.

## Current Baseline

As of July 9, 2026, on `dev`:

- `v1.18.0-beta.2` includes a Rust Lite MCP executable under
  `packages/qiongli-lite-mcp/`.
- Codex, Claude Code, Claude Desktop direct-plugin, and MCPB packages can bundle
  the executable at `bin/qiongli-literature-provider`.
- Canonical Lite tool declarations and initial schemas exist under
  `content/mcp-contracts/`.
- Python Full already exposes literature, provider configuration,
  orchestration, task planning, task execution, subject lifecycle, and
  experience tools through one MCP server.
- Platform packaging metadata exists in
  `content/distribution/platform-targets.yaml`.
- Release validators can launch plugin-declared MCP commands and verify
  `initialize` plus `tools/list`.

The baseline is installable but not yet a trustworthy unified platform:

- `content/mcp-contracts/lite-tools.json` declares twelve tools, while the Rust
  dispatcher handles only ten. The configuration wizard and its compatibility
  alias are advertised but not dispatchable.
- `qiongli_literature_search` builds a plan and then returns an empty
  `not_run` search result instead of calling the five implemented provider
  clients.
- Provider configuration fields, environment aliases, status semantics, and
  default result limits have drifted between Rust, Python, MCPB, and docs.
- Zotero Companion support validates a loopback URL but does not probe the
  companion; status is permanently reported as fallback-only.
- Lite/Full parity tests compare tool names, not input, output, error,
  redaction, or side-effect behavior.
- Plugin and MCPB builders package `build_current_platform()` binaries into
  generic artifact names while manifests claim multiple operating systems.
- The current target model mixes client, install surface, runtime profile, and
  artifact identity. A Full local plugin can therefore inherit metadata from a
  Marketplace Lite target.
- Platform rendering, installation, marker generation, and validation rules
  are duplicated across large scripts and two installer implementations.
- Existing Lite roadmap status labels no longer reflect the implementation
  already present on `dev`.

## Product Model

Qiongli presents one product with three public capability profiles:

| Profile | Runtime | Primary entry | Capability boundary |
|---|---|---|---|
| Skill-only | Host client only | Skill ZIP or portable workflow package | Routing, instructions, templates, and host-native capabilities only |
| Marketplace Lite | Rust local MCP | Marketplace/direct plugin or MCPB | Provider configuration, literature discovery, evidence export, Zotero import files, Companion status, and preview-only planning |
| Full | Python local runtime | `qiongli install --profile full` | All Lite-compatible tools plus orchestration, project state, local agents, validation, subject lifecycle, experience, and task execution |

The user-facing distinction is capability and install profile, not a separate
product brand. `qiongli` and `qiongli-next` remain the stable product IDs.

## Target Architecture

```text
Canonical academic content
  content/workflow + content/skills + content/standards
                         |
                         v
Unified control plane
  content/mcp-contracts        tool, result, error, side-effect contracts
  content/distribution         products, profiles, targets, variants
                         |
                         v
Platform compiler
  tooling/platform/model.py
  tooling/platform/compiler.py
  tooling/platform/adapters/*
  tooling/platform/validators/*
                  /                         \
                 v                           v
Rust Marketplace Lite                  Python Full runtime
packages/qiongli-lite-mcp           packages/python-qiongli
                 \                           /
                  v                         v
Codex | Claude Code | Claude Desktop | Antigravity | Hermes | npm | PyPI
                         |
                         v
Unified setup, check, doctor, upgrade, rollback, and release evidence
```

## Canonical Source Boundaries

| Boundary | Canonical source | Responsibility |
|---|---|---|
| Academic behavior | `content/workflow/`, `content/skills/`, `content/standards/` | Task IDs, skills, workflows, artifacts, and academic quality gates |
| MCP capability contracts | `content/mcp-contracts/` | Tool definitions, input/output schemas, errors, aliases, profiles, side effects, fixtures |
| Product and platform model | `content/distribution/` | Product IDs, runtime profiles, clients, surfaces, variants, artifact policy |
| Rust Lite implementation | `packages/qiongli-lite-mcp/` | Marketplace-safe local MCP subset |
| Python Full implementation | `packages/python-qiongli/` | Complete CLI, MCP, orchestrator, installer, and project runtime |
| Package shells | `packages/npm-qiongli/`, `packages/qiongli-literature-mcpb/`, `packages/qiongli-zotero-companion/` | Publishable wrapper or companion packages, not canonical behavior definitions |
| Platform compilation | `tooling/platform/` and compatibility wrappers in `tooling/scripts/` | Rendering, staging, installation plans, validation, and artifact metadata |
| Quality evidence | `tests/`, `evals/`, release receipts | Contract, conformance, integration, acceptance, and release evidence |

Generated plugin directories, portable payloads, marketplace catalogs, and
release archives must not become hidden sources of truth. Marketplace catalog
state remains in the external marketplace repository.

## Roadmap Principles

- Truth before expansion: every advertised capability must be dispatchable,
  tested, and documented accurately before new tools are added.
- One contract, migration-safe implementations: shared behavior is defined
  outside Rust and Python, verified against the frozen 1.x oracle during alpha,
  and implemented by the Rust-native core for 2.x.
- One product model, platform-specific artifacts: canonical inputs are shared,
  but native binaries and manifests remain explicit per target variant.
- Profiles are capability boundaries: Lite must not launch agents, run arbitrary
  shell commands, or write project guidance.
- Python Full freezes after the final planned 1.x beta. The approved migration
  design, rollback policy, and release train live in the successor 2.x roadmap.
- Local-first security remains the default: loopback-only setup and Companion
  traffic, redacted secrets, owner-only config permissions where supported,
  and no remote telemetry by default.
- Install and upgrade operations must be previewable, reversible, and must not
  overwrite unmanaged client configuration.
- Compatibility is explicit: aliases and legacy target IDs have declared
  support windows instead of disappearing through incidental refactors.
- Release claims require native evidence. A binary built on one host cannot be
  presented as a generic multi-platform artifact.
- Academic analysis-code rules must be enforceable: code used to produce a
  paper's estimates, tables, figures, or other claim-bearing evidence follows
  versioned academic policy, machine-checkable gates, independent review, and
  explicit time-bounded exceptions instead of prose-only conventions.

## Dependency Sequence

```text
Gate 0: Lite functional closure
        |
        +-- Platform: Stage 1 -> Stage 2 -> Stage 3 -> Stage 4 --+
        |                                                        |
        +-- Academic code: Task AC1 -----------------------------+--> Stage 5
        |                                                        |
        +-- Repository source: Task RC1 -------------------------+
```

Stage 1 contract design, early Stage 2 schema design, Task AC1, and Task RC1 may
be explored in parallel after Gate 0 behavior is frozen. Task AC1 does not
govern the Qiongli platform implementation; it blocks stable maturity for
code-generating academic capabilities and the Stage 5 stable rollout. Task RC1
does not judge academic validity; its changed-file gate is required before the
Stage 3 compiler cutover, and its release gate is required for Stage 4. Stages
3 through 5 must consume the new contracts rather than inventing additional
sources of truth.

## Gate 0: Lite MCP Functional And Truth Closure

Status: implementation and local acceptance complete on `dev`; review and the
separate `v1.18.0-beta.3` release procedure remain pending.

Suggested release target: `v1.18.0-beta.3`.

Primary outcome:

- Marketplace Lite moves from “starts and lists tools” to “every declared
  capability is real, bounded, and behavior-tested.”

Scope:

- Enforce `declared tools == dispatchable tools == behavior-tested tools`.
- Align provider fields, aliases, required-versus-optional semantics, default
  limits, and secret redaction across Rust, Python, MCPB, and contracts.
- Wire OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv clients into a
  deterministic search orchestrator.
- Add provider fan-out, partial-failure diagnostics, DOI/title-year
  deduplication, provider provenance merging, per-provider limits, and total
  result limits.
- Implement the Rust loopback configuration wizard and compatibility alias, or
  remove them from all public declarations before release. The default plan is
  to implement them.
- Probe the Qiongli Zotero Companion through loopback and preserve the existing
  public status vocabulary: `ok`, `companion_missing`, `fallback_only`, or
  `disabled`. Direct writes remain outside Lite unless a separately approved
  tool contract exists.
- Keep route and task-plan tools preview-only and make their response explicitly
  recommend Full for execution.
- Add Rust tests to CI and expand black-box and conformance tests from names to
  behavior, schemas, redaction, and errors.
- Correct docs and MCPB claims that exceed the implemented Lite subset.
- Add a minimum artifact-truth guard: a current-host binary must not be labeled
  as a generic multi-platform artifact. The complete native build matrix remains
  Stage 4.

Success criteria:

- Every tool returned by `tools/list` has a dispatcher entry and at least one
  successful or intentionally bounded behavior test.
- A mocked multi-provider search returns non-empty normalized results, merged
  provenance, deterministic deduplication, and per-provider diagnostics.
- A failed provider produces a partial result rather than hiding successful
  providers or leaking credentials.
- Configuration wizard URLs bind only to loopback and contain an unguessable
  session token; saved secrets never appear in MCP output or test logs.
- Lite and Full agree on overlapping provider-config semantics and normalized
  literature-record fields. Gate 0 may adapt each runtime's existing envelope
  in conformance tests; common envelope design belongs to Stage 1.
- Zotero status reflects the observed local state and keeps import-file fallback
  available without the Companion.
- `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test` run in CI.
- Release validation fails for advertised-but-undispatchable tools.
- Docs describe only the behavior verified by the Gate 0 test suite.
- Full CLI behavior remains unchanged except for shared-contract compatibility
  fixes.

Non-goals:

- Full Rust migration.
- Full task execution or local-agent launch from Lite.
- Remote MCP hosting.
- Complete OS/architecture release automation.
- A new tool naming scheme.

Formal execution plan:

- `docs/superpowers/plans/2026-07-09-lite-mcp-functional-closure.md`

## Stage 1: Capability Contract v2

Status: implementation active on `dev`; six canonical capabilities are
contract-backed, plus one compatibility alias, and the remaining 17 canonical
records are pending.

Suggested release target: `v1.19.0-beta.1`.

Primary outcome:

- One versioned capability registry defines the public MCP behavior shared by
  Lite, Full, MCPB manifests, validators, and documentation.

Scope:

- Add a versioned registry under `content/mcp-contracts/` with one record per
  tool.
- Define for every tool:
  - stable name and compatibility aliases;
  - input and structured-output schema references;
  - error taxonomy;
  - side-effect classes such as read, local-config-write, project-write,
    network, process-launch, or agent-launch;
  - profile membership: skill-only metadata, Lite, Full;
  - capability maturity: stable, preview, experimental, deprecated;
  - introduced, deprecated, and removal versions;
  - security and secret-handling notes.
- Define a common result envelope containing runtime profile, contract version,
  status, diagnostics, provenance, and next action without breaking existing
  structured payloads.
- Generate or strictly validate Rust tool definitions, Python definitions,
  MCPB tool lists, documentation tables, and safe smoke-call fixtures.
- Add schema and semantic conformance tests for both runtimes.

Success criteria:

- A tool change made only in one runtime fails CI.
- Input schemas for every overlapping tool are structurally compatible.
- Golden calls produce schema-valid results in Lite and Full.
- Error codes and redaction semantics match the contract.
- Side-effectful tools cannot be added to Lite without an explicit profile and
  security review.
- MCPB manifests and docs no longer maintain independent hand-written tool
  inventories.

Compatibility follow-up:

- Keep `--legacy-node` outside the Capability Contract v2 production profiles
  and normal beta/postflight artifacts. Before that reference runtime can be
  published again, either retire it after the documented compatibility window
  or bring provider-config reads/writes, `enabled: false`, atomic replacement,
  and fixed redacted errors up to the Rust Lite/Full contract.

Non-goals:

- Generating runtime handler implementations.
- Renaming existing MCP tools.
- Expanding the Lite profile merely to make lists equal.

Formal pilot execution plan:

- `docs/superpowers/plans/2026-07-10-capability-contract-v2-pilot.md`
- `docs/superpowers/plans/2026-07-10-capability-contract-v2-configuration.md`
- `docs/superpowers/plans/2026-07-10-capability-contract-v2-literature-planning.md`

## Stage 2: Product And Platform Target v2

Status: archived as a 1.x implementation stage. Its target-model requirements
move to the Rust-native successor roadmap.

Historical target: `v1.19.0-beta.1` or `v1.19.0-beta.2`; cancelled as a
standalone 1.x stage.

Primary outcome:

- Platform metadata represents what is actually installed instead of mixing
  client, surface, runtime, and artifact identity in one target label.

Scope:

- Introduce a product/profile registry under `content/distribution/`.
- Evolve platform targets so every record separates:
  - `client`: Codex, Claude Code, Claude Desktop, Antigravity, Hermes, or none;
  - `surface`: marketplace plugin, direct plugin, local plugin, MCPB, skill ZIP,
    npm package, PyPI package, or companion;
  - `runtime_profile`: skill-only, marketplace-lite, or full;
  - `variant`: operating system, architecture, executable name, and archive
    format;
  - `capability_profile`: reference to the Stage 1 registry;
  - `adapter`, `materializer`, `validator`, smoke policy, and activation policy;
  - component versions and legacy target IDs.
- Add explicit Full local-plugin targets for Codex, Claude Code, Antigravity,
  and managed Hermes MCP configuration.
- Bring MCPB and Zotero XPI companion artifact targets into the same product
  model while preserving specialized release metadata.
- Preserve v1 target IDs as aliases for at least two stable release trains.
- Update managed markers so they record the true runtime profile and variant.

Success criteria:

- A Full local plugin never records `marketplace-lite-binary` metadata.
- Every released asset maps to exactly one target plus one variant.
- Target resolution does not depend on ambiguous `recommended_key` lookup.
- Install discovery can distinguish marketplace Lite, local Full, skill-only,
  legacy skill, MCPB, and companion surfaces.
- Old managed markers remain readable and produce a clear migration path.

Non-goals:

- Replacing all builders in the same change.
- Removing legacy target IDs immediately.

## Cross-Cutting Task AC1: Academic Analysis Code Standard And Enforcement

`AC1` is a roadmap work-item label, not a new canonical research Task ID. It
strengthens the existing Stage I tasks `I1` through `I9`.

Status: archived as optional workflow guidance on July 12, 2026. AC1 has no
active implementation or release target and does not block exploratory code,
the Rust migration, or Qiongli 2 promotion. The detailed design below is kept
for users or journals that explicitly request stricter academic-code evidence.

Primary outcome:

- Code used to produce or validate a paper's estimates, models, tables,
  figures, simulations, qualitative computations, or other claim-bearing
  evidence is method-faithful, statistically defensible, auditable,
  reproducible, and traceable to the manuscript.

Canonical contract and workflow integration:

- Add `content/standards/academic-analysis-code-contract.yaml` as the source of
  truth, with stable `AAC-*` rule IDs, applicability conditions, evidence
  requirements, failure severity, and enforcement mode.
- Extend rather than replace the canonical Stage I flow:
  `I5 specification -> I6 planning -> I7 execution -> I8 independent review`,
  with `I4 reproducibility audit` and `I9 release packaging` where applicable.
- Connect the standard to `Q1` research-question/method alignment, `Q2`
  claim/evidence traceability, and the primary Stage I gate, `Q4`
  reproducibility baseline.
- Reuse `code/code_specification.md`, `code/plan.md`,
  `code/performance_profile.md`, `code/code_review.md`,
  `code/reproducibility_audit.md`, `quality-gate-report.md`, and the existing
  claim-evidence ledger instead of creating a parallel artifact system.
- Allow method- and domain-specific overlays for econometrics, statistics,
  experiments, simulation, machine learning, qualitative computation, and
  other scholarly methods without weakening the common minimum standard.
- Permit an exception only when it records an owner, scientific rationale,
  affected claim or output, compensating validation, residual limitation, and
  expiry date or version. Silent waivers are invalid.

Progressive enforcement modes:

- `explore`: permit reversible scripts, notebooks, method spikes, and
  data-shape discovery before the full I5/I6 chain. Require an exploratory
  label, a minimal source/assumption note, disclosure-safe inputs, and no
  fabricated or confirmatory claim. Do not require I8, a lockfile, container,
  synthetic generator, profiling, or a final output tree by default;
- `candidate`: trigger when code feeds a draft table, figure, estimate, model
  choice, or claim candidate. Require a decision-bounded specification/plan,
  changed-analysis lineage, relevant diagnostics/tests, environment version,
  stable output mapping, and recorded deviations. High-risk method or leakage
  changes require targeted independent review;
- `release`: trigger for final manuscript evidence, submission, archive,
  replication, or public package. Require the full applicable I5-I9 chain,
  independent I8 review, Q4 rerun or recorded tolerance/limitation, exact data
  identity/environment/command/output provenance, and no open blocking
  scientific finding.

Promotion strengthens evidence. An exploratory artifact may be promoted after
the missing candidate/release evidence is attached, but it may not be cited as
claim support merely because it runs. Supplying `--topic` chooses project
storage; it must not implicitly select strict release mode. Workflow routing
must use explicit or inferred evidence intent (`explore`, `candidate`, or
`release`).

Academic analysis-code constraints:

- Start from the research question, hypothesis, estimand, identification
  strategy, analysis plan, or manuscript-facing output. Code must not silently
  change the method, population, outcome, specification, or evidential claim.
- Lock the scientific decisions that affect estimand, population, outcome,
  identification, data lineage, specification, or manuscript output before
  candidate promotion. Implementation details may iterate when assumptions
  and deviations remain visible and affected evidence is revalidated.
- Preserve data lineage for every analysis input: source and vintage, raw-data
  immutability, cleaning rules, exclusions, missingness, joins, derived
  variables, sample construction, and leakage checks. This is computational
  evidence governance, not a general platform-data policy.
- Make statistical assumptions and inferential choices explicit. Record model
  diagnostics, uncertainty estimates, effect sizes where applicable,
  multiplicity handling, sensitivity analyses, robustness checks, and reasons
  for analytic exclusions.
- Prohibit undisclosed specification search, outcome switching, selective
  reporting, test-set leakage, post-treatment leakage, fabricated observations,
  and presentation of simulated, imputed, or exploratory results as observed
  confirmatory evidence.
- Record deviations from preregistration or the approved analysis plan and
  state how each deviation changes interpretation or claim strength.
- Use researcher-readable scripts, notebooks, Quarto files, or small modules.
  Names should reflect scholarly constructs; constants and thresholds require
  a source or rationale; functions should have explicit inputs, outputs, units,
  missing-value behavior, and failure conditions.
- Prefer validation and method clarity over application scaffolding. Service
  layers, controllers, generic framework abstractions, and unnecessary classes
  are out of scope unless the research method genuinely requires a reusable
  library.
- Choose a trusted library, high-level wrapper, or custom implementation based
  on method fidelity and auditability. First-principles code is required only
  when available abstractions cannot represent the specified method; it is not
  a blanket requirement for advanced work.
- Require performance profiling only when scale, runtime, memory, numerical
  stability, or a release acceptance criterion creates a material performance
  risk. Ordinary exploration and candidate analysis prioritize correctness and
  scientific diagnostics.
- Comments and documentation explain the method, assumption, equation,
  transformation, or non-obvious research decision. They must not merely
  restate syntax or imply evidence that the code did not establish.
- Keep notebooks runnable from a clean kernel without hidden execution order.
  Claim-bearing transformations and estimators must be testable outside opaque
  interactive state.
- Use synthetic or disclosure-safe fixtures with known expected properties
  when they are needed to test cleaning, merges, estimators, edge cases, and
  failure paths. Exploration still must not commit protected data or call live
  services without explicit authorization, but it need not manufacture a
  synthetic generator when a safe local stub or structural check is sufficient.
- Record runtime and dependency versions, exact commands, random seeds, known
  nondeterminism, environment requirements, output paths, and rerun limits.
- Write tables, figures, model outputs, and machine-readable results to
  predictable paths. Every manuscript-facing output must resolve to the script
  or notebook, input-data identity, model/specification, sample, and command
  that produced it.
- Label exploratory code and outputs explicitly. Exploratory results cannot
  enter the claim-evidence ledger as confirmatory support until the strict
  Stage I specification, execution, review, and audit path is complete.
- Keep credentials, unapproved personal data, and disclosive row-level values
  out of source code, prompts, fixtures, logs, review artifacts, and replication
  packages.

Review and enforcement model:

- Machine checks validate required artifacts, contract blocks, paths, seeds,
  environment evidence, output provenance, exception expiry, and forbidden
  leakage patterns.
- `I8` supplies the semantic review that automation cannot: method fidelity,
  inferential validity, data leakage, robustness adequacy, claim inflation, and
  reproducibility limits.
- Blocking academic findings take priority over style, refactoring, performance,
  or packaging suggestions. The implementer must not self-approve unresolved
  method-validity findings.
- Paper-facing results cannot pass `Q4`, enter release packaging, or be cited as
  final manuscript evidence while blocking `I8` findings remain open.
- Language-specific formatters and linters remain useful supporting checks, but
  they are not evidence of academic correctness.

Deliverables:

- The canonical academic analysis-code contract and a concise Stage I reference
  for researchers and reviewers.
- `tooling/scripts/validate_academic_analysis_code.py` for structural checks and
  machine-readable findings, with semantic review status consumed from `I8` and
  Q4 artifacts.
- Positive and negative fixtures plus eval cases covering lineage, sample
  construction, missingness, leakage, seed control, estimator correctness,
  robustness evidence, manuscript-output traceability, and exploratory-result
  labeling.
- Updated `/code-build` routing and Stage C/Stage I skills/templates that cite
  `AAC-*` rules without duplicating the canonical policy.
- A staged rollout: exploration labels and report-only audit, candidate
  promotion checks for changed claim-supporting code, then the full release
  gate after legacy academic-code debt has owners and bounded remediation
  plans.

Success criteria:

- A full `/code-build --focus full` run preserves the strict
  `I5 -> I6 -> I7 -> I8` chain and produces sufficient Q4 evidence for an
  independent reviewer to rerun and audit the work.
- Every final table, figure, coefficient, metric, or computational claim in a
  manuscript resolves to code entrypoint, input-data identity, specification,
  sample, environment, command, and output artifact.
- A clean rerun reproduces declared outputs exactly or within a justified,
  recorded numerical tolerance; nondeterministic limits weaken the release
  claim rather than being hidden.
- `I8` verifies method fidelity, inferential validity, leakage risks,
  robustness, and reproducibility. Unresolved blocking findings prevent Q4 and
  release-packaging completion.
- Exploratory outputs are distinguishable and cannot silently become
  confirmatory claim support.
- No code or supporting artifact invents data, results, sample sizes,
  statistical significance, citations, or reviewer conclusions.
- Every active exception has an owner, affected claim, compensating evidence,
  residual limitation, and unexpired deadline.

Non-goals:

- Governing the Rust, Python, or JavaScript source code used to implement the
  Qiongli product itself.
- Imposing one universal formatting style or application architecture on R,
  Python, Stata, Julia, MATLAB, notebooks, or other research environments.
- Treating formatter, linter, type-check, or unit-test success as proof of
  statistical or scientific validity.
- Replacing method expertise, ethics review, statistical review, or independent
  academic code review with automated validation.
- Requiring protected raw data to be committed or publicly released.

## Cross-Cutting Task RC1: Repository Source Code Standard And Enforcement

`RC1` means repository code in this roadmap; it is not a release-candidate
version label and it is not a canonical academic Task ID. It governs Qiongli's
own product source and maintainer tooling. It never substitutes for AC1's
method, statistics, claim-traceability, or reproducibility review.

Status: archived as optional diagnostic guidance on July 12, 2026. The
Phase-1 validator can evaluate whole-native-tree rules for a selected file and
can report context-free path text as a cybersecurity finding. It is therefore
removed from required CI and has no active implementation or release target.
GOV-201A-C is cancelled.

Primary outcome:

- Qiongli's Rust, Python, JavaScript/TypeScript, Shell, PowerShell, tests,
  generators, and maintainer scripts follow one repository-only engineering
  policy with stable rule IDs, deterministic validation, bounded legacy debt,
  and narrow expiring exceptions.

Canonical boundary and policy:

- Add `tooling/quality/repository-source-code-contract.yaml` as the canonical
  repo-only policy. Do not place RC1 under `content/standards/`, because content
  materialization would leak internal engineering policy into portable skills,
  plugins, npm/Python payloads, and release artifacts.
- Add `tooling/quality/repository-source-code-baseline.json` only for existing,
  fingerprinted debt. Every record requires rule ID, exact paths, owner,
  rationale, compensating check, and expiry. New findings cannot be hidden by
  broad path or rule suppression.
- Use stable `RSC-*` rule families for repository boundaries, Python, Rust,
  JavaScript/TypeScript, Shell, PowerShell, security, tests, dependencies,
  generated outputs, and exceptions.
- Keep language-native configuration authoritative for formatting, linting,
  types, tests, and dependencies. The RC1 contract declares which checks are
  required and how evidence is collected; it does not duplicate every tool's
  configuration.
- Add a concise contributor entrypoint in `CONTRIBUTING.md` and the full policy
  guide in `docs/development/repository-source-code-standard.md`.

Repository-source constraints:

- Preserve canonical-source boundaries among `content/`, runtime packages,
  plugin shells, generated payloads, release artifacts, and the external
  marketplace catalog. Generated or mirrored files cannot become an
  undocumented source of truth.
- Require Rust formatting, clippy with warnings denied, locked tests, explicit
  target/platform behavior, and a justification plus safety test for any
  `unsafe` code.
- Require Python formatting/lint checks, import hygiene, explicit error
  boundaries, types at public or contract-facing interfaces, deterministic
  tests, and no catch-all exception handling that erases actionable failure
  context.
- Require JavaScript/TypeScript formatting/lint checks, `node:test` or an
  approved test runner, consistent package-engine declarations, explicit async
  error handling, and safe process/argument construction.
- Require Shell syntax and ShellCheck policy, quoted variables and paths,
  failure-safe handling in release or mutating scripts, secure temporary-file
  use, and no secret-bearing command traces. Apply equivalent report-only then
  blocking analysis to PowerShell.
- Prohibit embedded credentials, private data, machine-specific absolute paths,
  unsafe command construction, unvalidated archive paths, and sensitive values
  in logs, fixtures, manifests, caches, or generated outputs.
- Require behavior-changing code to include focused tests and relevant docs or
  contract updates. Live external services cannot be mandatory test
  dependencies.
- Keep generated-output drift at zero: generators identify their canonical
  inputs, generated files carry provenance where appropriate, and CI rejects
  hand-edited payloads or stale projections.
- Pin or record toolchains and dependencies where reproducibility or release
  identity depends on them. Dependency and lockfile changes require scoped
  review and rollback awareness.
- Prefer clear, cohesive modules and explicit interfaces over speculative
  abstractions. Comments explain non-obvious product, compatibility, safety, or
  architectural decisions instead of restating syntax.

Archived enforcement proposal (not active for the 2.x migration):

- `development/changed-component`: always block real credentials/private data,
  traversal or archive escape, command injection, unauthorized writes, and
  canonical-boundary violations; run language-native formatting and focused
  tests only for the touched component;
- `candidate/PR`: add dependency/lock review, relevant integration and contract
  tests, and platform checks for any touched platform surface;
- `release`: add full-tree security and generated parity, all-target/all-feature
  workspace tests, dependency/license/SBOM/provenance, signing, and clean-machine
  no-runtime-dependency evidence.

Portable documentation, fixture paths, and sanitized sentinels require
context-aware classification; they are not equivalent to a live credential or
runtime path escape. A real secret, unapproved private data, command injection,
or traversal finding is never waivable. Other bounded debt requires exact
rule/path/component, owner, rationale, compensating check, and expiry within 30
days or one milestone.

The B2a single-member, zero-dependency, no-build-script, blanket process-launch,
and workspace-wide no-exception rules no longer constrain FND-202 or later
native work. Reviewed locked dependencies, service crates, build scripts,
crate-scoped FFI, and typed process adapters may be introduced through normal
implementation, tests, and review. Core startup must still have no hidden
Python, Node, Cargo, shell, or optional CLI dependency.

Archived deliverables (optional reference only):

- The repository-source contract, debt baseline, contributor guide, and short
  `CONTRIBUTING.md` entrypoint.
- `tooling/scripts/validate_repository_source.py`, a stable wrapper at
  `scripts/validate_repository_source.py`, and machine-readable JSON findings.
- Positive and negative fixtures plus
  `tests/test_repository_source_validator.py` covering rule selection,
  changed-file enforcement, baseline fingerprints, exception expiry,
  boundaries, secrets, paths, generated drift, and language profiles.
- CI and release-preflight integration were proposed but are cancelled. The
  validator remains available for manual diagnostics.

Rollout: cancelled. No GOV-201 phase, RC1 CI gate, or RC1 beta/stable release
gate remains in the active plan.

Success criteria:

- All changed first-party source passes applicable blocking `RSC-*` rules; new
  baseline suppressions and expired exceptions are zero.
- Rust fmt/clippy/test evidence remains 100%; touched Python, Node, Shell, and
  PowerShell files run their declared language checks before merge.
- High-severity secret, command-injection, unsafe-path, and private-data
  findings are zero in source and release artifacts.
- Public API, contract, installer, or release behavior changes include focused
  tests and documentation or an explicit non-applicability reason.
- Generated payload drift and hand-edited generated files remain zero.
- Every active exception is path- and rule-scoped, has complete ownership and
  compensating-check metadata, and has not expired.

Non-goals:

- Judging the scientific validity of paper analysis code; that belongs to AC1,
  Stage I, I8, and Q4.
- Forcing one implementation tool across every language or rewriting the whole
  repository to satisfy a new formatter in one change.
- Treating style compliance as proof of correctness, security, compatibility,
  or test adequacy.
- Copying marketplace catalogs, generated plugin payloads, or installable skill
  sources into the wrong repository boundary.

## Stage 3: Platform Compiler And Shared Install Plan

Status: archived as a Python-led 1.x implementation stage. Its declarative
install-plan requirements move to the native installer workstream.

Historical target: `v1.19.0-beta.2`; cancelled.

Primary outcome:

- Platform artifacts and installers are compiled from the same product, target,
  profile, and contract model.

Scope:

- Add a platform compiler boundary such as:

  ```text
  tooling/platform/
    model.py
    compiler.py
    install_plan.py
    adapters/
    renderers/
    validators/
  ```

- Keep platform adapters narrow:
  - client manifest rendering;
  - client path and activation behavior;
  - client-specific validation.
- Keep runtime staging separate:
  - `none` for skill-only;
  - `rust-lite` for Marketplace Lite;
  - `python-full` for Full local plugins and managed MCP entries.
- Generate a versioned `install-plan.json` that both Python and npm installers
  consume.
- Define adapter operations: detect, plan, materialize, install, activate,
  verify, remove, and rollback.
- Run the new compiler beside existing builders for one or two beta trains and
  compare normalized artifact trees before switching the default.
- Retain root `scripts/` and existing CLI commands as compatibility wrappers.

Success criteria:

- Python and npm installers no longer independently infer target metadata.
- Adding a platform adapter does not require copying core product or capability
  logic.
- Old and new builders produce equivalent artifacts for unchanged targets.
- Dry-run output is derived from the same install plan used for writes.
- Unmanaged client configuration remains untouched.
- Rollback can restore the previous managed marker and MCP/plugin entry.

Non-goals:

- Changing academic workflow content.
- Replacing client-native plugin formats with a universal archive.

## Stage 4: Native Release Matrix And Supply-Chain Evidence

Status: migrated to the Qiongli 2 native release-matrix workstream.

Historical target: `v1.19.0-rc.1`; cancelled.

Primary outcome:

- Every native Lite artifact is built, identified, checked, and launched on the
  platform it claims to support.

Scope:

- Build and publish the current supported target set:
  - `aarch64-apple-darwin`;
  - `x86_64-apple-darwin`;
  - `x86_64-unknown-linux-gnu`;
  - `aarch64-unknown-linux-gnu`;
  - `x86_64-pc-windows-msvc`.
- Include OS and architecture in native artifact identity or use a host format
  that performs explicit variant selection.
- Run initialize, tools/list, safe tool calls, and config-redaction smoke on
  native runners.
- Produce checksums, an artifact manifest, build provenance, and an SBOM per
  native binary.
- Add policy fields for macOS signing/notarization and Windows signing, then
  enforce them when release credentials and process are available.
- Test fresh install, Lite upgrade, Lite-to-Full migration, Full reinstall,
  remove, and rollback for each supported local client.
- Keep Node fallback explicitly labeled legacy until native coverage and
  behavioral parity have remained green for two release trains.

Success criteria:

- No generic archive contains an unidentified host-specific executable.
- Every published native asset has checksum, target identity, and startup
  evidence.
- Windows and macOS Lite tests are no longer represented by Python-only smoke.
- Release notes list exact supported targets and any unsupported variants.
- Node fallback retirement is a deliberate release decision with rollback, not
  incidental deletion.

Non-goals:

- Supporting every Rust target.
- Remote distribution services beyond existing package and release channels.

## Stage 5: Unified Product Experience And Stable Rollout

Status: migrated to the Qiongli 2 desktop, installer, doctor, updater, and
stable-promotion workstreams.

Historical target: `v1.19.0`; cancelled.

Primary outcome:

- Users see one Qiongli product that reports what is installed, what it can do,
  and how to upgrade without needing to understand internal runtime history.

Scope:

- Add a common runtime/capability status response available from Lite and Full.
- Make `qiongli setup`, `qiongli check`, and `qiongli doctor` report:
  - client and install surface;
  - runtime profile;
  - component and contract versions;
  - available, preview-only, and unavailable capabilities;
  - provider status without secrets;
  - artifact target and variant;
  - exact upgrade, repair, or rollback action.
- Ensure Lite requests for Full-only behavior return a structured Full upgrade
  recommendation instead of pretending to execute.
- Ensure Full installation migrates or disables duplicate managed Lite entries
  for the same product while preserving unmanaged entries.
- Make Lite task plans export a Full-compatible handoff packet so a user can
  upgrade and resume without re-entering project context.
- Publish migration, troubleshooting, and rollback documentation for each
  supported client.
- Move `qiongli-next` through canary and release-candidate gates before stable
  defaults change.

Success criteria:

- A single diagnostic command can explain all detected Qiongli surfaces.
- Fresh install, repair, upgrade, downgrade, and removal have deterministic
  dry-run and applied results.
- Lite-to-Full migration does not create duplicate managed MCP servers.
- A Lite plan can be consumed by Full without changing Task ID, topic, paper
  type, required outputs, or capability provenance.
- Stable documentation uses the same profile and target vocabulary as runtime
  diagnostics.

## Compatibility Policy

The roadmap preserves:

- product and skill IDs: `qiongli`, `qiongli-next`, `qiongli-workflow`;
- current CLI aliases and root script entrypoints;
- current MCP tool names and documented compatibility aliases;
- `QIONGLI_CONFIG_HOME/providers.json` and existing environment aliases;
- existing managed marker discovery, with migrations for older marker shapes;
- existing Task IDs, artifact paths, and workflow contracts;
- legacy target IDs for at least two stable release trains after Target v2;
- legacy Node provider packaging until Rust behavioral and native distribution
  gates are satisfied.

Breaking changes require:

- a versioned contract change;
- migration behavior and release notes;
- a compatibility or deprecation window;
- rollback instructions;
- tests against the last supported format.

## Testing Strategy

| Layer | Required evidence |
|---|---|
| Contract | JSON/YAML schema validation, fixtures, aliases, version and side-effect policy |
| Rust unit | Config, request construction, normalization, deduplication, diagnostics, wizard and Companion loopback rules |
| Rust integration | Mock HTTP providers, partial failures, MCP dispatcher coverage, stdio lifecycle |
| Python Full | Shared schema conformance, Full-only behavior regression, installer and doctor behavior |
| Cross-runtime | Golden calls for overlapping tools, error and redaction parity, handoff compatibility |
| Academic analysis code | I5/I6 contract checks, method and inferential validity, lineage/leakage tests, robustness evidence, manuscript-output traceability, I8 review, and clean Q4 rerun |
| Repository source | `RSC-*` changed-file rules, language-native gates, security/boundary scans, generated drift, baseline fingerprints, and exception expiry |
| Artifact | Manifest, binary identity, permissions, forbidden paths, checksums, SBOM |
| Client activation | Native launch and safe tool calls from the client-declared command |
| Migration | Fresh, upgrade, repair, downgrade, remove, rollback, unmanaged-entry preservation |
| Release | Strict validator, full unit suite, beta smoke, target matrix, acceptance receipt |

No release test should depend on live scholarly-provider availability. Provider
behavior uses local mock servers and committed fixtures; live probes remain
optional maintainer diagnostics.

## Platform Scorecard

The following metrics are release gates, not aspirational dashboards:

| Metric | Gate 0 target | Stable target |
|---|---:|---:|
| Advertised Lite tools with dispatcher coverage | 100% | 100% |
| Advertised Lite tools with behavior tests | 100% | 100% |
| Overlapping Lite/Full golden calls schema-valid | 100% | 100% |
| Raw secret occurrences in tool output or artifacts | 0 | 0 |
| Generated repository payload drift | 0 | 0 |
| High-severity repository secret, injection, unsafe-path, or private-data findings | 0 | 0 |
| Native assets with explicit target identity | current release host only, honestly scoped | 100% of published native assets |
| Published native assets with startup evidence | current release host | 100% |
| Managed Lite-to-Full migrations that avoid duplicate server entries | measured after Target v2 | 100% of supported clients |
| Generated artifact drift from canonical inputs | 0 | 0 |

## Branch And Release Sequence

Historical 1.x change sequence:

1. `fix/lite-mcp-functional-closure`
2. `test/mcp-behavioral-conformance`
3. `feat/capability-contract-v2`
4. `feat/academic-analysis-code-governance`
5. `chore/repository-source-governance`
6. `refactor/platform-target-v2`
7. `feat/platform-compiler`
8. `build/lite-mcp-release-matrix`
9. `feat/unified-platform-doctor`

Keep each branch independently reviewable. Gate 0 should land before contract
generation starts so the generated contract describes real behavior. Contract
and target schema changes should land before the compiler consumes them. The
AC1 and RC1 branches are historical planning records; neither is required for
the active Rust-native migration or stable rollout.

Revised release transition:

| Release | Main claim |
|---|---|
| `v1.18.0-beta.3` | Lite functional and truth closure |
| `v1.19.0-beta.1` | Final planned 1.x beta: implemented Contract v2 pilot batches, AC1/RC1 direction, and frozen Python migration baseline |
| `v2.0.0-alpha.1` | First usable Rust-native, zero-language-runtime vertical slice |
| `v2.0.0-beta.1` | Full migration qualification after all successor-roadmap beta gates |
| `v2.0.0` | Stable Rust-native platform |

Release numbers are planning labels, not permission to skip an unmet gate.
The previously listed `v1.19.0-beta.2`, `v1.19.0-rc.1`, and `v1.19.0` platform
releases are cancelled unless a future explicit decision restores them.

## Risk Register

| Risk | Impact | Mitigation |
|---|---|---|
| Platform work starts before Lite behavior is real | Contracts and generators preserve false claims | Make Gate 0 blocking and add dispatcher/behavior coverage tests first |
| Rust and Python continue to drift | Users receive profile-dependent semantics | Capability Contract v2, golden calls, schema and error parity in CI |
| Target v2 becomes another registry beside v1 | More sources of truth | Versioned loader, legacy aliases, one migration path, remove v1 writes after compatibility window |
| Platform compiler rewrite destabilizes releases | Artifact regressions across clients | Dual-run old/new builders, normalized tree comparison, per-target cutover |
| Native artifact mismatch | Plugin installs but executable cannot start | Explicit variants, native runners, checksums, startup evidence |
| Config wizard leaks secrets or binds publicly | Credential exposure | Loopback allowlist, random tokens, redaction tests, no secrets in URL or response |
| Provider mocks diverge from APIs | Tests pass while live requests fail | Preserve raw fixtures, request-shape tests, optional non-gating live diagnostics |
| Lite scope expands into unsafe execution | Marketplace binary becomes a shell/agent launcher | Side-effect classes and profile policy in Capability Contract v2 |
| Academic code rules remain stylistic prose or blanket waivers | Method errors, leakage, selective reporting, or irreproducible results can reach a manuscript despite clean formatting | `AAC-*` rules, strict I5-I8 flow, semantic I8 review, Q4 rerun evidence, and claim-scoped expiring exceptions |
| Repository engineering rules become a mass-reformat project or permanent blanket baseline | Review noise grows while real boundary, security, test, and compatibility risks remain hidden | Separate `RSC-*` policy, report-first inventory, changed-file ratchet, fingerprinted debt, narrow expiring exceptions, and language-native tools |
| Node fallback persists indefinitely | Maintenance never converges | Two-green-release retirement criterion and explicit owner/date in release plan |
| Rust migration starts without a frozen oracle | Skills, state, agents, or installer behavior is lost | Accept the final 1.x beta, freeze normalized fixtures, then migrate by contract |
| Hosted scope enters the monorepo | Security and operational boundaries blur | Separate service repository and explicit future product design |

## Transition Decisions

### Shared Rust Provider Kernel

Decision taken as part of the native program. Extract the existing Rust Lite
provider behavior into shared native crates while retaining the old binary as a
compatibility entry during alpha. The 2.x core should use those crates directly;
a versioned JSON subprocess remains preferable to FFI only when process
isolation is intentionally required.

### Full Rust Migration

Decision taken on July 10, 2026. Full Rust migration is now the active 2.x
program. Its architecture, task catalog, parity gates, target matrix, local and
cloud boundaries, rollback plan, alpha/beta promotion criteria, and definition
of done are authoritative in
`docs/superpowers/roadmaps/2026-07-10-qiongli-2-rust-native-platform-roadmap.md`.
The contracts and evidence created by this roadmap remain required migration
inputs; they are not discarded by the language change.

### Hosted Control Plane

Evaluate only for an explicit team, account, policy, or remote coordination
product. Authentication, tenant isolation, billing, data residency, and service
operations are outside this repository's current boundary.

## Definition Of Done

This definition applies only to the retained 1.x foundation. The end-to-end
product definition of done is owned by the Rust-native successor roadmap.

This roadmap is complete when:

- Gate 0 Lite functional and truth closure is published and accepted.
- Every Stage 1 capability actually included in the final 1.x beta has a valid
  versioned record, schema, semantic error, redaction, side-effect, declaration,
  and golden-call evidence for its advertised profiles.
- Contract v2 is described honestly as a pilot with generated exact coverage;
  completion of all Full tools is a 2.x migration gate, not a false 1.x claim.
- `v1.19.0-beta.1` is published and accepted as the immutable Python-led
  behavior oracle, with explicit product/component versions, target identity,
  startup evidence, rollback, and known limitations.
- Python Full behavior, Rust Lite behavior, Node-only compatibility behavior,
  CLI semantics, mutable state, installation behavior, skills, agents, and
  orchestrator scenarios are inventoried for the successor migration.
- Canonical content, contract, distribution, implementation, generated-output,
  and external marketplace boundaries remain explicit.
- Marketplace Lite remains self-contained and does not gain unsafe execution
  capabilities in the final 1.x release.
- AC1 academic-code and RC1 repository-code documents remain optional guidance
  and are not carried into the active 2.x dependency graph or release gates.
- Generated payloads, external marketplace catalogs, and release archives do
  not become canonical source.
