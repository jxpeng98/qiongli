# Adaptive Subject Runtime Roadmap

## Purpose

Qiongli now installs as an adaptive core package and can infer economics or
finance refinements at runtime. The next product goal is to make this adaptive
subject layer measurable, user-confirmable, and safe to expand across more
disciplines.

This roadmap covers the work after the initial runtime subject refinement and
real preview smoke harness. It is organized as staged increments so each stage
can ship with tests, local verification, and clear rollback behavior.

## Current Baseline

Completed as of July 5, 2026:

- Runtime subject refinement packet with `decision`, `signals`,
  `resource_activation_plan`, `borrowed_lenses`, and backward-compatible fields.
- Cross-run `.qiongli/trace/subject_evidence.json` memory for repeated subject
  suggestions.
- Confirmation recommendations in guidance proposals without automatic manifest
  writes.
- Preview-first real smoke harness for subject runtime behavior.
- Isolated smoke environment including `HOME`, `CODEX_HOME`,
  `QIONGLI_CONFIG_HOME`, `QIONGLI_GUIDANCE_HOME`, `XDG_CONFIG_HOME`, and
  `RESEARCH_CLI_LANG=en`.
- Curated subject router evaluation fixtures and a thresholded evaluation
  runner for clear, mixed, borrowed-lens, near-miss, and core-only cases.
- Project-local subject lifecycle controls through CLI and MCP: `status`,
  `confirm`, `dismiss`, `reset`, `lock`, and `unlock`.
- Managed subject guidance materialization at
  `.qiongli/guidance.d/subject-runtime.md` after explicit confirmation or lock.
- Runtime preview packets that expose loaded local guidance, including the
  managed subject guidance fragment.
- Opt-in local-agent smoke foundations, including `--mode local-agent`,
  `QIONGLI_SMOKE_RUN_AGENTS=1`, bounded task-run overrides, trace assertions,
  and write-boundary reporting.
- Runtime subject contract and subject-gate foundations that keep candidate
  subjects inactive until their evaluation gates pass.
- MCP provider status parity, broader literature-search defaults, full-text
  candidate planning, and Zotero attachment verification summaries released in
  `v1.15.0-beta.1`.
- Preview-first full-cycle workflow harness for topic framing, evidence search,
  data/methods, writing, compliance, judging, journal fit, and feedback loops.
- Manuscript-first reverse journal-fit recommendation backed by local venue
  profiles.
- Accounting eval-ready fixture pack with manifest-backed method, venue,
  data/outcome, theory/construct signals and gate-specific fixture
  expectations.
- Accounting runtime-enabled routing completed by the accounting runtime
  promotion change, with a method-only auto-mode guard and passing
  runtime-enabled gate.
- Business runtime activation completed after the promotion-ready review, with
  business-owned runtime resources, expanded near-miss coverage, and a passing
  runtime-enabled gate.
- Political economy runtime activation completed after the promotion-ready
  review, with subject-owned runtime signals, a six-case fixture pack,
  method-only borrowing, mixed-adjacent coverage, and near-miss guards.
- Geoeconomics runtime activation completed after the promotion-ready review,
  with subject-owned statecraft and supply-chain exposure signals, a six-case
  fixture pack, method-only borrowing, mixed-adjacent coverage, and near-miss
  guards.
- Economics-accounting bridge runtime activation completed after the
  promotion-ready review, with subject-owned identification/measurement bridge
  signals, a six-case fixture pack, method-only borrowing, mixed-adjacent
  coverage, near-miss guards, and a dedicated bridge auditor skill.
- `v1.16.1` separated the Claude Desktop direct plugin from the hybrid plugin
  bundle: direct Desktop ZIPs now exclude Codex metadata and Codex workflow
  wrapper skills while retaining Claude commands, the main workflow skill, and
  the bundled lightweight literature MCP runtime.

Current residual constraints and convergence notes:

- Real local-agent smoke remains opt-in and should stay outside the default
  release gate until maintainer environments are stable; its machine-readable
  diagnostics now report runtime requests, routing notes, checked Qiongli-visible
  paths, rerun commands, and trace paths when failures occur.
- The full-cycle workflow harness is deterministic and preview-first; later
  runtime-enabled multi-agent execution now has a maintainer smoke hardening
  slice for the parallel Codex/Claude/Antigravity path, which requires both
  `--run-parallel` and `QIONGLI_SMOKE_RUN_AGENTS=1`.
- Feedback from lifecycle actions is recorded, and subject refinement traces now
  separate task-text, manifest, trace-memory, and user-action evidence sources
  for maintainer inspection.
- Marketplace, plugin, and read-only client behavior now has CLI/MCP
  proposed-action export for subject lifecycle updates when `.qiongli` writes
  are unavailable; release receipts include subject evidence summaries; isolated
  local install acceptance starts the full MCP server and checks lifecycle
  tools in `tools/list`.
- Local guidance, trace bundles, subject evidence memory, worker orchestration,
  and validator results are now normalized into queryable experience records
  with metrics, replay, promotion candidates, and release schema checks.
- Platform packaging is registry-converged for the current supported surfaces:
  the canonical source model is sound and release download guide/index
  generation now carries registry target metadata, and release postflight upload
  lists now derive from the same registry-backed index; upload asset selection
  now consumes the same `assets_by_target` mapping for both platform artifacts
  and companion release metadata assets; Python local plugin installer markers
  and Codex marketplace entries now carry the same registry metadata; npm
  plugin-lite installation now selects its target record by the `qiongli_cli`
  recommended key and records the registry-derived metadata in npm-managed
  plugin markers and status output; release preflight now validates that each
  platform target declares positive path checks, negative forbidden path checks,
  and release-download metadata; release downloads now include a
  machine-readable artifact manifest that flattens assets into per-target
  policy records with adapter/materializer metadata; platform target adapters
  now use schema-enforced kind, manifest-platform, materializer values, and
  compatibility rules, release recommended install target IDs now derive from
  registry `release_download.recommended_key` metadata, release guide and
  release notes labels use those recommended target IDs, marketplace artifact
  validation and Python local plugin installation select their target records by
  the same recommended keys, and beta release notes consume the same
  registry-backed download summary instead of maintaining a separate asset
  table; direct Desktop plugin artifact building applies forbidden-path policy
  through registry recommended-key lookup; companion metadata assets now load
  specialized manifest target IDs from a release companion target registry with
  required current asset-key validation instead of one catch-all release
  companion target; local-install acceptance now selects client activation
  targets from registry smoke policy, maps clients by registry recommended
  keys, and validates installed marker target metadata against the registry;
  marketplace validation now explicitly reports its structural-only scope and
  the registry-selected target IDs whose client activation checks are skipped
  there.

## Roadmap Principles

- Install remains core-first. Users should not choose a subject during normal
  installation.
- Runtime inference proposes and explains. User action confirms or locks.
- Method-only evidence borrows lenses. It does not switch the project subject.
- Subject expansion requires evaluation fixtures before new rules are trusted.
- Local guidance writes must be explicit, reversible, and project-scoped.
- Every stage must preserve preview-first safety: local agents do not launch
  unless explicitly requested.
- End-to-end workflow claims require a lifecycle harness: stage handoffs,
  claim-evidence coverage, strong judge results, and journal-fit decisions must
  be checked together before later expansion work is considered release-ready.
- Self-improvement must be local-first and promotion-gated: task runs may write
  experience records and candidates, but canonical skills, standards, and
  generated packages change only through reviewed source edits and tests.
- Cross-platform development stays single-source, but artifacts stay
  platform-specific: Codex, Claude Code, Claude Desktop direct plugin,
  Desktop/Web skill ZIPs, Antigravity, npm, and PyPI may share canonical inputs
  but must not share incompatible manifests or wrapper shapes in final bundles.

## Priority Update: Business Runtime Activation

Status: the subject expansion onboarding contract, business promotion-ready
review, and business runtime activation are complete.

The accounting runtime-enabled gate has been reviewed and remains green.
Business now has dedicated fixtures, a completed promotion-ready review path,
and checked-in `runtime_enabled` manifest status. The runtime-enabled gate now
validates business-owned fixtures, method-lens borrowing, mixed cases,
confirmed-subject behavior, and practitioner near-miss guards without activation
overrides.

Formal design and execution plan:

- `docs/superpowers/specs/2026-07-05-accounting-runtime-promotion-design.md`
- `docs/superpowers/plans/2026-07-05-accounting-runtime-promotion.md`
- `docs/superpowers/specs/2026-07-05-subject-expansion-onboarding-contract-design.md`
- `docs/superpowers/plans/2026-07-05-subject-expansion-onboarding-contract.md`
- `docs/superpowers/specs/2026-07-05-business-subject-eval-ready-design.md`
- `docs/superpowers/plans/2026-07-05-business-subject-eval-ready.md`
- `docs/superpowers/specs/2026-07-05-business-runtime-promotion-readiness-design.md`
- `docs/superpowers/plans/2026-07-05-business-runtime-promotion-readiness.md`
- `docs/superpowers/specs/2026-07-05-business-runtime-activation-design.md`
- `docs/superpowers/plans/2026-07-05-business-runtime-activation.md`
- `docs/superpowers/specs/2026-07-06-experience-promotion-loop-design.md`
- `docs/superpowers/plans/2026-07-06-political-economy-eval-ready.md`
- `docs/superpowers/plans/2026-07-06-political-economy-runtime-activation.md`
- `docs/superpowers/plans/2026-07-06-geoeconomics-eval-ready.md`
- `docs/superpowers/plans/2026-07-06-geoeconomics-runtime-activation.md`
- `docs/superpowers/plans/2026-07-06-economics-accounting-eval-ready.md`
- `docs/superpowers/plans/2026-07-06-economics-accounting-runtime-activation.md`

## Stage 1: Router Evaluation And Lifecycle Controls

Status: completed on `dev`.

Primary outcome:

- Qiongli can evaluate subject router quality on a curated fixture corpus.
- Users and clients can inspect and control subject state through stable CLI and
  MCP operations.

Scope:

- Add curated subject evaluation fixtures for clear finance, clear economics,
  method-only borrowed-lens, mixed subject, weak-signal core-only, and near-miss
  cases.
- Add a local evaluation runner that produces accuracy, false-positive,
  false-negative, and confusion summaries.
- Add lifecycle operations: `status`, `confirm`, `dismiss`, `reset`, `lock`,
  and `unlock`.
- Connect lifecycle operations to `.qiongli/guidance_manifest.yaml` and
  `.qiongli/trace/subject_evidence.json`.
- Keep `promotion_recommendation` as a recommendation until a user or client
  calls a lifecycle operation.

Success criteria:

- Evaluation report exits non-zero when thresholds fail.
- `suggest_subject` precision is measured separately from `borrow_lens`.
- Dismissed subject candidates stop producing confirmation prompts until new
  evidence appears or the user resets the dismissal.
- Confirm and lock operations update only project-local manifest state.
- Existing smoke harness and full test suite remain green.

First formal spec:

- `docs/superpowers/specs/2026-07-01-adaptive-subject-lifecycle-and-eval-design.md`

## Stage 2: Lightweight Local Guidance Materialization

Status: completed on `dev`.

Primary outcome:

- Confirmed subject state influences future agent runs through project-local
  guidance text, not only runtime packets.

Scope:

- Generate or update a concise `.qiongli/guidance.d/subject-runtime.md` file
  after explicit confirmation or lock.
- Include active subject, confirmation source, method lenses, resource
  activation summary, and date of last update.
- Keep generated guidance managed and clearly marked so user edits can be
  preserved or merged.
- Add lint checks that generated subject guidance cannot override core safety,
  evidence, or quality-gate contracts.

Success criteria:

- Confirming finance writes a stable local guidance fragment that future
  `effective_guidance()` calls include.
- Reset removes or disables managed subject guidance without touching unrelated
  user guidance fragments.
- Manual user edits are detected and not overwritten silently.

First formal spec:

- `docs/superpowers/specs/2026-07-01-subject-guidance-materialization-design.md`

## Stage 3: Real Local-Agent Smoke And Runtime Hardening

Status: implemented on `dev` for the opt-in smoke path, isolated environment
roots, bounded local-agent task arguments, loaded-guidance and trace assertions,
write-boundary checks, and machine-readable failure diagnostics. It remains
opt-in for maintainers and outside the default release gate.

Primary outcome:

- Qiongli can verify a minimal end-to-end local-agent run in an isolated
  environment and prove that materialized subject guidance is consumed by the
  real runtime, not only the preview packet.

Scope:

- Add a separate opt-in smoke path that requires both a command flag and an
  environment variable.
- Use temporary `HOME`, client config roots, project root, and trace root.
- Run one small task through the local runtime with bounded output and no
  external literature or provider-network requirement beyond the selected local
  agent runtime's own model access.
- Verify trace bundle completeness, subject refinement packet persistence,
  local guidance loading, and no writes outside the isolated root.
- Add runtime hardening checks for local guidance load errors, write boundary
  reporting, and stable smoke diagnostics.

Success criteria:

- Preview smoke remains the default release gate.
- Local-agent smoke is available for maintainers and release candidates.
- Failed local-agent smoke reports the exact command, isolated root, and trace
  path for diagnosis.
- Confirmed subject guidance appears in the local-agent run trace as a loaded
  project guidance source.

First formal spec:

- `docs/superpowers/specs/2026-07-02-real-local-agent-smoke-runtime-hardening-design.md`

## Stage 4: Subject Expansion With Evaluation Gates

Status: accounting eval-ready and runtime promotion are completed as of
July 5, 2026. The subject expansion onboarding contract is complete. Business,
political economy, geoeconomics, and the economics-accounting bridge now have
runtime activation completed with passing runtime-enabled gate coverage.

Primary outcome:

- New subjects can be added without weakening router precision.

Runtime-enabled subjects:

- Accounting.
- Business and management.
- Economics.
- Economics-accounting bridge.
- Finance.
- Geoeconomics.
- Political economy.

Eval-ready subjects:

- None currently.

Deferred candidate subjects:

- None currently.

Scope:

- Require an evaluation fixture pack before enabling each new subject.
- Add subject-specific method, venue, data, and outcome signal groups.
- Add near-miss cases for adjacent disciplines to prevent broad over-activation.
- Extend subject resource activation plans only after the evaluation pack passes.
- Runtime activation must stay as a small follow-up to promotion-ready review
  for future eval-ready subjects: activation PRs may change
  `activation_status` only after the runtime-enabled gate passes without
  harness overrides.

Success criteria:

- Each new subject has clear positive, borrowed-lens, mixed, and near-miss
  fixtures.
- Existing economics and finance metrics do not regress.
- Subject expansion does not increase false positives in core-only cases.

## Stage 5: Feedback-Aware Subject Refinement

Status: implemented on `dev` for dismissed-subject cooldowns, lifecycle event
memory preservation, explicit subject evidence-source separation, and guidance
proposal rendering of task-text, manifest, trace-memory, and user-action
sources.

Primary outcome:

- User confirmations, dismissals, and resets improve later recommendations
  without turning the router into hidden automation.

Scope:

- Track lifecycle actions as explicit evidence events.
- Add per-project cooldowns for dismissed subjects.
- Use confirmation history as one signal dimension in routing.
- Add explainability output showing which evidence came from task text,
  manifest state, user action, or trace memory.

Success criteria:

- Users can see why a subject was suggested.
- Repeated user dismissals reduce repeated prompts.
- Explicit user confirmation remains stronger than inferred text evidence.

## Stage 6: Release And Marketplace Readiness

Status: completed on `dev` for read-only subject lifecycle proposed-action
export through CLI `--propose-only` and MCP `read_only: true`; npm-lite
full-runtime lifecycle messaging; generated release acceptance subject evidence
summaries; and isolated package/install-surface checks that start the full MCP
stdio server and require lifecycle tools in `tools/list`.

Primary outcome:

- Adaptive subject lifecycle works consistently across CLI, plugin, marketplace,
  and client-native installs.

Scope:

- Ensure lifecycle MCP tools are included in full runtime installs.
- Ensure npm-lite surfaces explain that lifecycle controls require the Python
  runtime when applicable.
- Document marketplace behavior for clients that cannot write `.qiongli`
  project files.
- Add release-readiness checks for lifecycle controls, evaluation reports, and
  smoke isolation.

Success criteria:

- CLI and MCP behavior match for status, confirm, dismiss, reset, lock, and
  unlock.
- Read-only clients can still show recommendations and export proposed actions.
- Release receipts include subject eval and smoke report summaries.

## Stage 7: Experience Record Contract

Status: implemented on `dev` for local experience record writing,
failure-mode extraction, worker/merge/final-review status capture, JSONL
indexing, and write-failure warnings that preserve existing trace artifacts.

Primary outcome:

- Every trace-writing task run also emits a compact, stable experience record
  that later tools can query without reading every run artifact by hand.

Scope:

- Add a versioned `experience_record` shape covering task metadata, execution
  mode, guidance sources, subject refinement, worker state, required/found/missing
  outputs, validator status, review blockers, reusable lessons, and privacy
  redaction state.
- Write `experience_record.json` under each `.qiongli/trace/runs/<run_id>/`
  directory.
- Append compact records to `.qiongli/trace/experience.jsonl`.
- Keep existing `.qiongli/trace/index.jsonl` behavior stable for backwards
  compatibility.

Success criteria:

- Preview and real task runs record `run_agents` and execution mode clearly.
- Failed validator gates produce structured failure modes.
- Worker orchestration runs record worker, merge, and final-review status.
- Experience write failures surface as warnings and do not delete formal
  research outputs.

Formal design:

- `docs/superpowers/specs/2026-07-06-experience-promotion-loop-design.md`

## Stage 8: Experience Query, Replay, And Planner Injection

Status: implemented on `dev` as local-project `experience`
list/show/search/lessons/replay-plan CLI commands, MCP query/show/lessons
tools, and bounded `prior_experience` injection for `task-plan` and
`task-run` packets.

Primary outcome:

- Qiongli can use local experience as bounded context for later planning
  instead of treating traces as passive audit files.

Scope:

- Add CLI commands such as `qiongli experience list/show/search/lessons` and
  `qiongli experience replay-plan`.
- Add MCP query/show/lessons tools for local clients.
- Support filters by task ID, stage, topic, subject, validator status, failure
  mode, guidance source, and worker mode.
- Add a bounded `prior_experience` block to `task-plan` and `task-run` packets.
- Record the query parameters and selected run IDs whenever prior experience is
  injected.

Success criteria:

- A maintainer can find prior failed `B1`, `F3`, or worker runs without manual
  trace-directory inspection.
- `task-plan` can show relevant prior lessons while preserving preview-first
  behavior.
- Prior experience is advisory and never overrides canonical contracts, required
  outputs, evidence gates, or safety constraints.
- Older trace bundles without experience records are skipped or summarized with
  warnings, not treated as fatal errors.

## Stage 9: Evidence-Backed Skill Reinforcement

Status: implemented on `dev` as reviewable skill reinforcement candidate
generation from repeated experience evidence. Candidate generation writes
under `.qiongli/trace/promotion/` and does not edit `content/skills/**`.

Primary outcome:

- Existing Qiongli skills are strengthened from repeated experience evidence
  before broader core promotion is attempted.

This is the stage where the current supporting skills should be reinforced.
It deliberately comes after query/replay and before canonical promotion:
experience records first identify the recurring weakness, then maintainers
update the relevant skill source and tests.

Scope:

- Generate skill reinforcement candidates from repeated experience patterns.
- Attribute candidates to existing skill IDs and Task IDs.
- Use supporting run IDs, validator results, review blockers, worker merge
  conflicts, local guidance proposals, and subject refinement evidence.
- Update canonical sources only through normal reviewed edits under:
  - `content/skills/**`
  - `content/skills-core.md`
  - `content/skills/registry.yaml`
  - `content/workflow/references/**`
  - `content/standards/mcp-agent-capability-map.yaml`
- Add or update tests/evals before accepting skill behavior changes.

Candidate triggers:

- Repeated missing required artifacts for the same skill or Task ID.
- Repeated reviewer blocks with the same cause.
- Repeated local guidance proposals that compensate for weak skill language.
- Worker merge conflicts tied to a specific skill boundary.
- Subject overlays repeatedly patching behavior that belongs in core guidance.

Success criteria:

- Candidate output names affected skill IDs, supporting experience records,
  proposed source changes, expected behavior change, required tests, and rollback
  path.
- Candidate generation does not edit `content/skills/**` automatically.
- Accepted skill updates also update `content/skills-core.md` when the behavior
  must be visible in token-efficient execution.
- Generated workflow payloads remain derived outputs and are not edited by hand.

## Stage 10: Local-To-Global/Core Promotion Gates

Status: implemented on `dev` for explicit promotion scopes and candidate
artifact generation. Canonical candidates require a test plan and never edit
canonical source automatically; user-global promotion requires explicit
approval.

Primary outcome:

- Repeated local evidence can become a project-local rule, user-global
  preference, skill reinforcement candidate, or canonical candidate through
  explicit and reviewable promotion scopes.

Scope:

- Add promotion scopes: `local`, `user-global`, `skill-candidate`, and
  `canonical-candidate`.
- Require explicit approval for user-global promotion.
- Require repeated support and proposed tests for skill and canonical candidates.
- Generate promotion artifacts that preserve privacy boundaries and avoid raw
  provider credentials, private corpora, or project-specific secrets.
- Document that canonical candidates require normal repository review and tests.

Success criteria:

- Local promotion can apply an accepted guidance proposal.
- User-global promotion refuses project-specific private evidence.
- Skill-candidate promotion requires experience support and names affected skill
  source.
- Canonical-candidate promotion creates a review artifact but does not edit
  canonical source automatically.

## Stage 11: Experience Metrics And Release Readiness

Status: implemented on `dev` for local `experience metrics` summaries covering
validator pass rate, missing artifact rate, failure modes, guidance acceptance,
subject routing confirmation/dismissal/correction, review blockers, worker
merge/final review blockers, literature diagnostic failures, and
release-readiness experience schema compatibility checks.

Primary outcome:

- Qiongli can measure whether experience-driven changes improve outcomes before
  claiming self-improvement.

Scope:

- Add experience-derived metrics for validator pass rate, missing artifact rate,
  review blocker count, guidance acceptance rate, subject routing correction
  rate, worker merge failure rate, final-review block rate, and literature
  diagnostic failure rate.
- Add release-readiness checks for experience schema compatibility.
- Document local learning, skill reinforcement, and canonical promotion
  boundaries in CLI and maintainer docs.

Success criteria:

- `qiongli experience metrics --project-dir .` produces compact JSON and a
  human-readable table.
- Release checks can detect malformed new experience records.
- Docs clearly separate local guidance, experience records, skill reinforcement,
  user-global preferences, and canonical promotion.

## Stage 12: Platform Target Registry And Artifact Boundary Governance

Status: implemented on `dev` for a canonical platform target registry,
registry-backed artifact boundary checks, direct Claude Desktop plugin
negative checks, validator target ID reporting, and release download guide/index
generation with registry-derived target metadata and asset grouping. Release
postflight upload assets now derive from the same registry-backed download
index. Python local plugin installers now record registry-derived target
metadata in managed markers and Codex marketplace entries. npm plugin-lite
installation now selects its target by `release_download.recommended_key` and
records the registry-derived metadata in npm-managed plugin markers and `check`
output. Release preflight now runs an
explicit release target registry schema gate for platform and companion target
registries before the standard validator. Release download generation now emits
a machine-readable artifact manifest mapping each asset to target metadata,
adapter/materializer metadata, subject, archive format, install method, smoke
policy, required paths, and forbidden-path policy.
Platform targets now include schema-enforced adapter metadata for kind,
manifest-platform selection, materializer-surface declaration, and adapter
compatibility so marketplace validation and release metadata can consume
registry records instead of branching on target IDs or process-only assumptions.
Local-install acceptance now reads registry targets whose smoke policy requires
client activation, maps Codex, Claude Code, and Antigravity client validators
through `release_download.recommended_key` metadata, and verifies installed
marker `platform_target` metadata against those canonical records.
Beta release notes now render their download summary from the same
registry-backed release download index instead of maintaining a separate Bash
asset table. The release artifact manifest now uses a release companion target
registry to assign specialized target IDs for MCPB, Zotero XPI, download guide,
download index, and manifest metadata assets instead of the old catch-all
`release-companion` target.
Marketplace artifact validation now prints an explicit structural-archive
completion line and a client-CLI activation skip line that lists the
registry-selected activation target IDs before pointing maintainers to
`scripts/release_local_install_check.py`.
Release upload selection now reads companion MCPB, Zotero, download guide,
download index, and artifact manifest records from the same `assets_by_target`
mapping as platform artifacts instead of maintaining a second companion key
list. Release recommended install entries now derive their target IDs from
platform target `release_download.recommended_key` values rather than fixed
target strings, and guide/release-note labels read those target IDs from the
generated index. Marketplace artifact validation selects Codex, Claude Code,
Desktop plugin, and Desktop/Web skill targets from the same recommended-key
metadata instead of fixed target IDs. Direct Desktop plugin artifact building
now applies registry forbidden-path policy through the `claude_desktop_plugin`
recommended key rather than a fixed target ID. Python local plugin installation
now selects Codex, Claude Code, and Antigravity local plugin target metadata
from registry `release_download.recommended_key` values instead of fixed target
IDs. Platform target registry entries now carry smoke policy metadata that keeps
structural archive validation in marketplace validation and client CLI
activation in local-install acceptance where applicable.

Primary outcome:

- Qiongli keeps one canonical development path while generating strict,
  platform-specific plugin and skill artifacts from an explicit target registry.

This stage protects the `v1.16.1` packaging fix from regression. It does not
reintroduce a universal hybrid ZIP. The shared layer is the canonical source and
materializer contract; the final artifacts remain platform-specific.

Scope:

- Add a versioned platform target registry, for example
  `content/distribution/platform-targets.yaml` or
  `tooling/distribution/platform_targets.yaml`.
- Model each install surface as a target entry:
  - Codex marketplace plugin.
  - Claude Code marketplace plugin.
  - Claude Desktop direct plugin.
  - Claude Desktop/Web skill ZIP.
  - Antigravity local plugin.
  - npm/npx plugin-lite payload.
  - PyPI/full runtime payload.
- For each target, declare source inputs, required manifests, allowed wrapper
  directories, bundled MCP mode, command surface, archive format, validator
  command, adapter metadata, and forbidden paths.
- Refactor `tooling/scripts/build_plugin_artifacts.py`,
  `scripts/validate_marketplace_install.py`, local plugin installers, npm
  plugin-lite installation, and release postflight to consume the same target
  metadata incrementally.
- Generate positive and negative artifact tests from the target registry.
- Document that canonical source edits belong under `content/**`,
  `content/distribution/plugins.yaml`, and runtime/package source, while root
  `qiongli-workflow/`, `plugins/qiongli/`, and release archives remain derived
  outputs.

Boundary invariants:

- Codex artifacts may include `.codex-plugin/`, `.mcp.json`, and
  `skills/qiongli-*` workflow wrapper skills.
- Claude Code marketplace artifacts may include Claude plugin metadata and
  Claude-compatible commands/MCP configuration.
- Claude Desktop direct plugin artifacts must include `.claude-plugin/`,
  commands, the main `skills/qiongli-workflow` skill, and the bundled
  lightweight literature MCP runtime.
- Claude Desktop direct plugin artifacts must not include `.codex-plugin/`,
  `.mcp.json`, or expanded `skills/qiongli-*` Codex workflow wrapper skills.
- Desktop/Web skill ZIPs remain skill-only fallback packages and must not claim
  full Python orchestrator capability.
- Antigravity plugin artifacts keep Antigravity's root plugin shape and may let
  the Antigravity CLI convert commands to skills during validation.

Optimization backlog:

- Extend the current `PlatformTarget.adapter` metadata only when future
  platform adapters need behavior beyond schema-enforced kind,
  manifest-platform selection, materializer-surface declaration, and adapter
  compatibility rules.
- Extend the schema validator when new platform adapters add target-specific
  fields beyond required positive checks, negative checks, adapter enums,
  smoke policies, and release-download metadata.
- Extend companion target metadata only when future non-platform release assets
  need fields beyond the current release companion target registry records for
  MCPB, Zotero XPI, download guide, download index, artifact manifest, and
  required current asset-key validation.
- Keep future release asset selection attached to `assets_by_target` so docs,
  release notes, postflight upload, and install validation cannot drift.
- Extend platform smoke policies only when a future target needs a new
  structural archive check type or client activation gate beyond the current
  registry-backed `smoke` metadata.

Success criteria:

- A maintainer can add or change a platform target without changing canonical
  academic workflow source.
- Release preflight fails if any artifact contains a manifest, wrapper skill,
  MCP config, or generated path that the target registry forbids.
- The Claude Desktop direct plugin regression from `1.16.0` is covered by both
  unit tests and release artifact validation.
- Marketplace install validation reports each target by target ID and makes
  skipped client-CLI activation explicit.
- Documentation and release downloads are generated from the same target list
  used by artifact builders and validators.

## Cross-Stage Risks

- Over-activation risk: mitigated by near-miss fixtures and precision
  thresholds.
- User trust risk: mitigated by explicit confirm, dismiss, reset, lock, and
  unlock controls.
- Client parity risk: mitigated by CLI and MCP contracts that share one runtime
  implementation.
- Local file safety risk: mitigated by project-local writes, managed fragments,
  and isolated smoke tests.
- Scope creep risk: mitigated by activating subjects one at a time, merging the
  subject expansion onboarding contract before expansion work, and keeping
  runtime activation behind separate reviewed promotion specs.
- Self-improvement overreach risk: mitigated by making experience records
  queryable first, placing skill reinforcement before canonical promotion, and
  requiring tests before any framework-level behavior change.
- Hybrid packaging regression risk: mitigated by a platform target registry,
  negative artifact checks, and release gates that reject cross-platform
  manifest or wrapper leakage.

## Recommended Immediate Plan

1. Keep local-agent runtime execution opt-in until maintainer smoke
   environments are stable. The default release gate should continue to use
   deterministic preview-first checks plus explicit maintainer smoke commands.
2. Maintain Stage 12 as a registry-extension backlog rather than an active
   target-lookup migration. New platforms should extend registry schemas,
   adapter metadata, companion metadata, or smoke policies only when the current
   required fields are insufficient.
3. Use Stage 8 replay and Stage 11 metrics to decide when to reinforce existing
   supporting skills. Skill updates should remain evidence-driven: recurring
   validator failures, review blockers, routing corrections, or worker merge
   failures should point to the skill that needs reinforcement before canonical
   workflow changes are proposed.
4. Treat future subject additions as separate follow-up specs with explicit
   eval-ready, promotion-ready, and runtime-enabled gate criteria.
5. Before release, run the registry and packaging gates together: platform
   target validation, release download/index generation, marketplace structural
   validation, local-install acceptance, npm installer tests, and the relevant
   subject runtime gates.

Current Stage 4 execution sequence:

- Business runtime activation is completed.
- Political economy runtime activation is completed.
- Geoeconomics eval-ready activation is completed.
- Geoeconomics runtime activation is completed.
- Economics-accounting eval-ready activation is completed.
- Economics-accounting runtime activation is completed.
- Keep future subject runtime promotions as separate reviewed follow-up specs.
