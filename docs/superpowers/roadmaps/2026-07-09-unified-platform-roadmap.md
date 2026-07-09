# Qiongli Unified Platform Roadmap

## Purpose

Qiongli now has a self-contained Rust Marketplace Lite MCP runtime and a
Python-backed Full CLI runtime. The next product goal is not to collapse both
runtimes into one language. It is to make them two capability profiles of one
platform with one set of contracts, one target model, one installation plan,
one diagnostic vocabulary, and one release policy.

This roadmap covers the work after `v1.18.0-beta.2`. It starts with a mandatory
Lite MCP functional-closure gate, then builds the shared control plane required
for platform convergence.

The first execution plan is:

- `docs/superpowers/plans/2026-07-09-lite-mcp-functional-closure.md`

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

- Rust Marketplace Lite remains the self-contained, no-user-runtime,
  marketplace-safe subset.
- Python Full remains the complete runtime for orchestration, project writes,
  local agents, guidance, validation, and task execution.
- Shared contracts, platform targets, generated install plans, and conformance
  tests form the unified control plane.
- A shared Rust provider kernel may be evaluated only after the control plane
  is stable and measured drift still justifies it.
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
- One contract, multiple implementations: shared behavior is defined outside
  Rust and Python and verified against both.
- One product model, platform-specific artifacts: canonical inputs are shared,
  but native binaries and manifests remain explicit per target variant.
- Profiles are capability boundaries: Lite must not launch agents, run arbitrary
  shell commands, or write project guidance.
- Full stays Python during this roadmap. A language migration requires a new
  design, benchmarks, rollback plan, and release train.
- Local-first security remains the default: loopback-only setup and Companion
  traffic, redacted secrets, owner-only config permissions where supported,
  and no remote telemetry by default.
- Install and upgrade operations must be previewable, reversible, and must not
  overwrite unmanaged client configuration.
- Compatibility is explicit: aliases and legacy target IDs have declared
  support windows instead of disappearing through incidental refactors.
- Release claims require native evidence. A binary built on one host cannot be
  presented as a generic multi-platform artifact.

## Dependency Sequence

```text
Gate 0: Lite functional closure
        |
        v
Stage 1: Capability Contract v2
        |
        v
Stage 2: Product and Platform Target v2
        |
        v
Stage 3: Platform compiler and shared install plan
        |
        v
Stage 4: Native release matrix and supply-chain evidence
        |
        v
Stage 5: Unified product experience and stable rollout
```

Stage 1 contract design and early Stage 2 schema design may be explored in
parallel after Gate 0 behavior is frozen. Stages 3 through 5 must consume the
new contracts rather than inventing additional sources of truth.

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

Status: planned after Gate 0.

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

Non-goals:

- Generating runtime handler implementations.
- Renaming existing MCP tools.
- Expanding the Lite profile merely to make lists equal.

## Stage 2: Product And Platform Target v2

Status: planned after the Stage 1 registry shape is stable.

Suggested release target: `v1.19.0-beta.1` or `v1.19.0-beta.2`.

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

## Stage 3: Platform Compiler And Shared Install Plan

Status: planned after Platform Target v2.

Suggested release target: `v1.19.0-beta.2`.

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

Status: planned after target variants and compiler inputs are stable.

Suggested release target: `v1.19.0-rc.1`.

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

Status: planned after native release evidence is reliable.

Suggested release target: `v1.19.0`.

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
| Native assets with explicit target identity | current release host only, honestly scoped | 100% of published native assets |
| Published native assets with startup evidence | current release host | 100% |
| Managed Lite-to-Full migrations that avoid duplicate server entries | measured after Target v2 | 100% of supported clients |
| Generated artifact drift from canonical inputs | 0 | 0 |

## Branch And Release Sequence

Recommended change sequence:

1. `fix/lite-mcp-functional-closure`
2. `test/mcp-behavioral-conformance`
3. `feat/capability-contract-v2`
4. `refactor/platform-target-v2`
5. `feat/platform-compiler`
6. `build/lite-mcp-release-matrix`
7. `feat/unified-platform-doctor`

Keep each branch independently reviewable. Gate 0 should land before contract
generation starts so the generated contract describes real behavior. Contract
and target schema changes should land before the compiler consumes them.

Suggested release train:

| Release | Main claim |
|---|---|
| `v1.18.0-beta.3` | Lite functional and truth closure |
| `v1.19.0-beta.1` | Capability Contract v2 and initial Target v2 |
| `v1.19.0-beta.2` | Platform compiler and shared install plan |
| `v1.19.0-rc.1` | Native release matrix and migration acceptance |
| `v1.19.0` | Unified setup, diagnostics, upgrade, and stable platform model |

Release numbers are planning labels, not permission to skip an unmet gate.

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
| Node fallback persists indefinitely | Maintenance never converges | Two-green-release retirement criterion and explicit owner/date in release plan |
| Full Rust rewrite distracts from control-plane convergence | High cost without product benefit | Separate decision gate based on measured distribution and maintenance evidence |
| Hosted scope enters the monorepo | Security and operational boundaries blur | Separate service repository and explicit future product design |

## Decision Gates After Stable

### Shared Rust Provider Kernel

Evaluate a shared Rust provider subprocess only if, after Contract v2:

- provider behavior drift remains a leading maintenance failure;
- native binary delivery is reliable on every supported Full platform;
- subprocess failure, version negotiation, and rollback behavior are designed;
- Python Full can preserve its public API and project runtime behavior.

Prefer a versioned JSON subprocess protocol before considering FFI.

### Full Rust Migration

Do not evaluate until:

- multiple stable Lite release trains have shipped;
- Python installation is a measured, persistent blocker;
- orchestration, project writes, agent execution, and doctor contracts are
  stable;
- the team can fund parallel compatibility and rollback paths.

### Hosted Control Plane

Evaluate only for an explicit team, account, policy, or remote coordination
product. Authentication, tenant isolation, billing, data residency, and service
operations are outside this repository's current boundary.

## Definition Of Done

This roadmap is complete when:

- Lite and Full are discoverable as capability profiles of one Qiongli product.
- Every overlapping public tool is defined by one versioned contract and passes
  schema, error, redaction, and behavior conformance.
- Product, target, surface, runtime profile, and native variant are modeled
  separately and reported accurately.
- Platform artifacts and install plans are compiled from canonical registries.
- Every published native binary has explicit identity and native startup
  evidence.
- Setup, check, doctor, upgrade, repair, removal, and rollback share one target
  and capability vocabulary.
- Lite-to-Full migration is reversible and does not duplicate managed client
  entries.
- Full remains Python-backed and retains complete orchestration behavior.
- Marketplace Lite remains self-contained and does not gain unsafe execution
  capabilities.
- Generated payloads, external marketplace catalogs, and release archives do
  not become canonical source.
