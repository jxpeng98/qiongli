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

Remaining product gaps:

- Real local-agent smoke remains opt-in and should stay outside the default
  release gate until maintainer environments are stable.
- The full-cycle workflow harness is deterministic and preview-first; later
  runtime-enabled multi-agent execution still needs separate opt-in hardening.
- Expansion to additional subjects still needs subject-specific fixture packs,
  near-miss guards, and gate thresholds before activation.
- Feedback from lifecycle actions is recorded, but router explainability does
  not yet clearly separate task-text, manifest, trace-memory, and user-action
  evidence in every output path.
- Marketplace, plugin, and read-only client behavior needs a release-ready
  fallback contract for proposed actions when `.qiongli` writes are unavailable.

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

## Priority Update: Business Promotion-Readiness Review

Status: the subject expansion onboarding contract is complete, and business is
the current Stage 4 promotion-readiness slice.

The accounting runtime-enabled gate has been reviewed and remains green.
Business now has a dedicated eval-ready pack plus a promotion-ready review path.
This keeps business below runtime activation while proving, under a harness-only
runtime simulation, that business-owned fixtures, method-lens borrowing, mixed
cases, confirmed-subject behavior, and practitioner near-miss guards remain
precise enough for a later activation PR.

Business must remain `eval_ready`, not `runtime_enabled`, until a separate
activation PR changes the manifest. A green `promotion-ready` gate is evidence
for that later PR; it is not activation by itself.

Formal design and execution plan:

- `docs/superpowers/specs/2026-07-05-accounting-runtime-promotion-design.md`
- `docs/superpowers/plans/2026-07-05-accounting-runtime-promotion.md`
- `docs/superpowers/specs/2026-07-05-subject-expansion-onboarding-contract-design.md`
- `docs/superpowers/plans/2026-07-05-subject-expansion-onboarding-contract.md`
- `docs/superpowers/specs/2026-07-05-business-subject-eval-ready-design.md`
- `docs/superpowers/plans/2026-07-05-business-subject-eval-ready.md`
- `docs/superpowers/specs/2026-07-05-business-runtime-promotion-readiness-design.md`
- `docs/superpowers/plans/2026-07-05-business-runtime-promotion-readiness.md`

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

Status: partially implemented on `dev`; remains opt-in for maintainers.

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
July 5, 2026. The subject expansion onboarding contract is complete. Business
is eval-ready with promotion-ready gate coverage; political economy,
geoeconomics, and the economics-accounting bridge remain deferred candidates.

Primary outcome:

- New subjects can be added without weakening router precision.

Runtime-enabled subjects:

- Accounting.
- Economics.
- Finance.

Eval-ready subjects:

- Business and management, with promotion-ready gate coverage.

Deferred candidate subjects:

- Political economy.
- Geoeconomics.
- Economics-accounting bridge.

Scope:

- Require an evaluation fixture pack before enabling each new subject.
- Add subject-specific method, venue, data, and outcome signal groups.
- Add near-miss cases for adjacent disciplines to prevent broad over-activation.
- Extend subject resource activation plans only after the evaluation pack passes.
- Business runtime activation must stay separate from promotion-ready review:
  `promotion-ready` may pass while the checked-in manifest remains
  `eval_ready`; only a later activation PR may change business to
  `runtime_enabled`.

Success criteria:

- Each new subject has clear positive, borrowed-lens, mixed, and near-miss
  fixtures.
- Existing economics and finance metrics do not regress.
- Subject expansion does not increase false positives in core-only cases.

## Stage 5: Feedback-Aware Subject Refinement

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

## Recommended Immediate Plan

1. Finish and review the business `promotion-ready` gate while keeping the
   checked-in manifest at `activation_status: eval_ready`.
2. If `promotion-ready` remains green after review, prepare a separate business
   runtime activation PR that changes `activation_status` and reruns the
   runtime-enabled gate.
3. If activation is deferred, continue Stage 5 feedback-aware explainability
   work so router outputs separate task-text, manifest, trace-memory, and
   user-action evidence more clearly.
4. Keep political economy, geoeconomics, and economics-accounting as separate
   follow-up subject specs with their own fixture packs and gate criteria.

Current Stage 4 execution sequence:

- Complete business promotion-ready review.
- Prepare a separate business runtime activation PR only after promotion-ready
  review passes.
- Keep political economy, geoeconomics, and economics-accounting as separate
  follow-up specs.
