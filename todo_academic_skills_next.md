# Academic Skills Next Enhancement TODO

> **For future agents:** This is a phased master TODO for the next generation of the `research-skills` plugin and `research-paper-workflow` skill package. Before implementation, select one phase or one coherent milestone and convert it into a detailed implementation plan.

**Goal:** Make the plugin-first academic skills package more evidence-grounded, stateful, venue-aware, methodologically rigorous, and measurable.

**Architecture:** Keep one main plugin, `research-skills`, and one portable skill package, `research-paper-workflow`. Add durable research-state and evidence artifacts first, then upgrade skills and workflows to consume those artifacts. Use tests and eval cases as the main guardrail against vague, generic, or fabricated academic output.

**Tech Stack:** Markdown skill specs, YAML contracts, Python validators/eval scripts, shell release/install scripts, Codex / Claude Code / Gemini plugin surfaces.

---

## Operating Principles

- Keep `skills/` as the source of truth for canonical internal skill specs.
- Keep `research-paper-workflow/` as the portable package for installed client skills.
- Keep `plugins/research-skills/skills/research-paper-workflow/` as a generated distribution copy.
- Run `bash scripts/sync_skill_package.sh --target all` after source skill or portable package changes.
- Do not add new skills unless the capability cannot be cleanly added to an existing skill or workflow.
- Every new scholarly claim feature must encode evidence handling, insufficient-input behavior, and non-fabrication rules.
- Every implementation phase must add or update tests before release.

---

## Phase 0: Baseline And Scope Lock

**Purpose:** Establish a measurable baseline before expanding capabilities.

**Primary files:**

- `docs/maintainer/skill-quality-gap-report.md`
- `docs/maintainer/skill-quality-contract.md`
- `scripts/audit_skill_sections.py`
- `scripts/validate_research_standard.py`
- `tests/test_audit_skill_sections.py`
- `tests/test_skill_structure_lint.py`

**Tasks:**

- [ ] Re-run the current baseline:
  - `python3 scripts/validate_research_standard.py --strict`
  - `python3 scripts/audit_skill_sections.py --strict`
  - `python3 -m unittest discover -s tests -v`
- [ ] Record current pass counts and known limitations in `docs/maintainer/skill-quality-gap-report.md`.
- [ ] Add a "Next Quality Targets" section to `docs/maintainer/skill-quality-contract.md`.
- [ ] Define the next quality dimensions:
  - evidence traceability
  - research-state continuity
  - venue fit
  - method validity
  - paragraph-level scholarly quality
  - eval coverage
- [ ] Add validator warnings for missing next-generation artifacts without failing strict mode yet.

**Acceptance:**

- [ ] Baseline report distinguishes current section completeness from next-generation scholarly capability.
- [ ] Strict validator remains green.
- [ ] No existing install, release, or marketplace test regresses.

---

## Phase 1: Evidence Ledger And Claim Traceability

**Purpose:** Make every important claim traceable to a source, data artifact, analysis result, or explicit gap note.

**Primary files to create:**

- `research-paper-workflow/templates/evidence-ledger.md`
- `research-paper-workflow/templates/claim-evidence-ledger.csv`
- `research-paper-workflow/references/evidence-ledger-contract.md`
- `scripts/audit_evidence_contract.py`
- `tests/test_evidence_ledger_contract.py`

**Primary files to modify:**

- `research-paper-workflow/SKILL.md`
- `research-paper-workflow/references/workflow-contract.md`
- `research-paper-workflow/templates/claim-evidence-map.md`
- `skills/B_literature/*.md`
- `skills/E_synthesis/*.md`
- `skills/F_writing/*.md`
- `skills/H_submission/*.md`
- `skills/J_proofread/*.md`
- `scripts/validate_research_standard.py`

**Tasks:**

- [ ] Define a canonical evidence ledger schema with fields:
  - `claim_id`
  - `claim_text`
  - `claim_type`
  - `evidence_type`
  - `source_id`
  - `source_location`
  - `artifact_path`
  - `confidence`
  - `limitations`
  - `status`
- [ ] Define allowed `claim_type` values:
  - `finding`
  - `interpretation`
  - `implication`
  - `method_assumption`
  - `limitation`
  - `speculation`
- [ ] Define allowed `evidence_type` values:
  - `paper`
  - `dataset`
  - `analysis_result`
  - `theory`
  - `artifact`
  - `gap_note`
- [ ] Add a rule that every central manuscript claim must map to at least one ledger row.
- [ ] Add a rule that unsupported claims become `gap_note` entries instead of invented citations.
- [ ] Update literature, synthesis, writing, submission, and proofreading skills to read and update the ledger.
- [ ] Add audit script checks for required columns, allowed enum values, duplicate claim IDs, and missing source pointers.
- [ ] Add tests with good and bad ledger fixtures.
- [ ] Sync the package with `bash scripts/sync_skill_package.sh --target all`.

**Acceptance:**

- [ ] `scripts/audit_evidence_contract.py` passes on bundled templates.
- [ ] Bad fixtures fail for unsupported central claims and invalid enum values.
- [ ] Writing skills explicitly require evidence ledger updates.
- [ ] `python3 scripts/validate_research_standard.py --strict` passes.

---

## Phase 2: Citation Risk And Source Integrity

**Purpose:** Reduce hallucinated citations, source-claim mismatch, and citation overreach.

**Primary files to create:**

- `scripts/audit_citation_risk.py`
- `tests/test_citation_risk_audit.py`
- `research-paper-workflow/references/citation-risk-policy.md`
- `research-paper-workflow/templates/citation-risk-report.md`

**Primary files to modify:**

- `skills/B_literature/citation-formatter.md`
- `skills/B_literature/reference-manager-bridge.md`
- `skills/F_writing/manuscript-architect.md`
- `skills/F_writing/discussion-writer.md`
- `skills/J_proofread/similarity-checker.md`
- `skills/J_proofread/final-proofreader.md`
- `scripts/validate_research_standard.py`

**Tasks:**

- [ ] Define citation risk categories:
  - missing source
  - source not in bibliography
  - weak support
  - wrong construct
  - outdated source for current claim
  - review cited as primary evidence
  - citation pile without synthesis
  - invented bibliographic metadata
- [ ] Add a citation risk report template.
- [ ] Add a source-integrity audit that checks bibliography keys against claim evidence entries.
- [ ] Add explicit rules for when to cite:
  - primary empirical source
  - review/meta-analysis
  - theory source
  - method source
  - dataset/code source
- [ ] Update writing and proofreading skills to produce citation risk notes before finalizing manuscripts.
- [ ] Add tests for valid, weak, missing, and fabricated citation cases.

**Acceptance:**

- [ ] Bad fixtures with fabricated or missing source IDs fail.
- [ ] Good fixtures with explicit source links pass.
- [ ] Final proofreading requires citation-risk review for submission-ready outputs.

---

## Phase 3: Research State And Long-Running Context

**Purpose:** Make the system preserve paper-level context across sessions, phases, and clients.

**Primary files to create:**

- `research-paper-workflow/references/research-state-contract.md`
- `scripts/audit_research_state.py`
- `tests/test_research_state_contract.py`

**Primary files to modify:**

- `research-paper-workflow/templates/research-state.md`
- `research-paper-workflow/templates/decision-log.md`
- `research-paper-workflow/SKILL.md`
- `skills/Z_cross_cutting/academic-context-maintainer.md`
- `skills/Z_cross_cutting/model-collaborator.md`
- `bridges/orchestrator.py`
- `tests/test_academic_context_continuity.py`
- `tests/test_orchestrator_workflows.py`

**Tasks:**

- [ ] Define required sections for `RESEARCH/[topic]/research-state.md`:
  - current paper type
  - active venue target
  - research question
  - current claim set
  - evidence ledger path
  - open decisions
  - unresolved gaps
  - latest artifact map
  - next recommended task
- [ ] Define required decision-log fields:
  - decision ID
  - date
  - decision
  - alternatives considered
  - evidence used
  - owner role
  - downstream impact
- [ ] Update `academic-context-maintainer` to refresh state before and after stage tasks.
- [ ] Update workflow prompts to read state before executing.
- [ ] Add orchestrator support for injecting a compact state summary into task-run prompts.
- [ ] Add tests for missing state, stale state, and valid state handoff.

**Acceptance:**

- [ ] New sessions can reconstruct the paper context from state artifacts.
- [ ] Task-run prompts include current state and unresolved gaps when present.
- [ ] Missing state results in a structured gap note, not guessed context.

---

## Phase 4: Stage Handoff Protocol

**Purpose:** Make every stage transition explicit and auditable.

**Primary files to create:**

- `research-paper-workflow/templates/stage-handoff.md`
- `research-paper-workflow/references/stage-handoff-contract.md`
- `scripts/audit_stage_handoffs.py`
- `tests/test_stage_handoff_contract.py`

**Primary files to modify:**

- `research-paper-workflow/references/stage-A-framing.md`
- `research-paper-workflow/references/stage-B-literature.md`
- `research-paper-workflow/references/stage-C-design.md`
- `research-paper-workflow/references/stage-E-synthesis.md`
- `research-paper-workflow/references/stage-F-writing.md`
- `research-paper-workflow/references/stage-H-submission.md`
- `standards/research-workflow-contract.yaml`

**Tasks:**

- [ ] Define handoff sections:
  - completed artifacts
  - decision summary
  - unresolved questions
  - evidence dependencies
  - assumptions passed forward
  - risks for the next stage
  - recommended next tasks
- [ ] Add handoff artifact requirements to major stage references.
- [ ] Add contract entries for handoff artifacts where stage transitions are high-risk.
- [ ] Add audit tests that detect missing unresolved questions and missing artifact links.

**Acceptance:**

- [ ] Stage completion is not treated as complete unless handoff state is present.
- [ ] Downstream stages can identify what they inherited and what remains uncertain.

---

## Phase 5: Venue-Aware Research Workflows

**Purpose:** Make outputs fit specific academic communities, not generic paper templates.

**Primary files to create:**

- `research-paper-workflow/references/venue-profile-contract.md`
- `research-paper-workflow/venue-profiles/README.md`
- `research-paper-workflow/venue-profiles/chi.yaml`
- `research-paper-workflow/venue-profiles/neurips.yaml`
- `research-paper-workflow/venue-profiles/acl.yaml`
- `research-paper-workflow/venue-profiles/nature.yaml`
- `research-paper-workflow/venue-profiles/jama.yaml`
- `research-paper-workflow/venue-profiles/aom.yaml`
- `scripts/audit_venue_profiles.py`
- `tests/test_venue_profiles.py`

**Primary files to modify:**

- `skills/A_framing/venue-analyzer.md`
- `skills/A_framing/contribution-crafter.md`
- `skills/F_writing/manuscript-architect.md`
- `skills/H_submission/submission-packager.md`
- `skills/H_submission/peer-review-simulation.md`
- `skills/H_submission/rebuttal-assistant.md`
- `research-paper-workflow/SKILL.md`

**Tasks:**

- [ ] Define venue profile schema:
  - venue ID
  - community
  - article types
  - contribution expectations
  - methods expectations
  - evidence standards
  - writing style
  - common reviewer objections
  - formatting constraints
  - required reporting standards
- [ ] Add initial venue profiles for core target communities.
- [ ] Update venue analyzer to select or recommend a venue profile.
- [ ] Update writing skills to apply profile-specific contribution and style constraints.
- [ ] Update submission skills to produce venue-specific checklists.
- [ ] Add tests for valid profiles, missing required fields, and unsupported venue fallback.

**Acceptance:**

- [ ] A workflow can declare `venue_profile: chi` or another profile and receive profile-specific guidance.
- [ ] Unsupported venues produce a custom venue-gap note instead of generic assumptions.
- [ ] Venue profiles remain data files, not hard-coded logic in individual skills.

---

## Phase 6: Reviewer Model And Rebuttal Intelligence

**Purpose:** Anticipate reviewer objections and produce more precise revision and rebuttal artifacts.

**Primary files to create:**

- `research-paper-workflow/templates/reviewer-objection-map.md`
- `research-paper-workflow/templates/revision-plan.md`
- `research-paper-workflow/references/reviewer-model-contract.md`
- `tests/test_reviewer_model_contract.py`

**Primary files to modify:**

- `skills/H_submission/peer-review-simulation.md`
- `skills/H_submission/reviewer-empathy-checker.md`
- `skills/H_submission/rebuttal-assistant.md`
- `skills/H_submission/fatal-flaw-detector.md`
- `skills/H_submission/limitation-auditor.md`
- `research-paper-workflow/workflows/rebuttal.md`
- `research-paper-workflow/workflows/submission-prep.md`

**Tasks:**

- [ ] Define reviewer personas by concern type:
  - theory contribution
  - method validity
  - measurement validity
  - novelty
  - external validity
  - ethics/compliance
  - writing clarity
- [ ] Add an objection map template connecting objection, evidence, response strategy, and required manuscript changes.
- [ ] Update peer-review simulation to produce actionable objections, not generic criticism.
- [ ] Update rebuttal workflow to require manuscript change tracking before response drafting.
- [ ] Add tests for objection map completeness and evidence-backed response requirements.

**Acceptance:**

- [ ] Rebuttal outputs separate "response to reviewer" from "actual manuscript change".
- [ ] Review simulation names specific evidence or missing evidence for each objection.

---

## Phase 7: Methods And Research Design Diagnostics

**Purpose:** Help users identify design threats before writing or analysis locks them in.

**Primary files to create:**

- `research-paper-workflow/templates/method-diagnostic-report.md`
- `research-paper-workflow/templates/validity-threat-matrix.md`
- `research-paper-workflow/references/method-diagnostic-contract.md`
- `skills/C_design/method-diagnostician.md`
- `tests/test_method_diagnostics.py`

**Primary files to modify:**

- `skills/C_design/study-designer.md`
- `skills/C_design/robustness-planner.md`
- `skills/C_design/rival-hypothesis-designer.md`
- `skills/C_design/variable-operationalizer.md`
- `skills/I_code/stats-engine.md`
- `standards/mcp-agent-capability-map.yaml`
- `standards/research-workflow-contract.yaml`
- `skills/registry.yaml`

**Tasks:**

- [ ] Add a method diagnostic skill or extend existing design skills if a new skill is not justified.
- [ ] Define validity threat categories:
  - construct validity
  - internal validity
  - external validity
  - statistical conclusion validity
  - measurement validity
  - data leakage
  - missingness
  - confounding
  - selection bias
- [ ] Add diagnostic outputs to Stage C and Stage I tasks.
- [ ] Add method-specific checks for empirical, qualitative, systematic review, methods, theory, and code-first papers.
- [ ] Add tests for diagnostic routing and artifact path alignment.

**Acceptance:**

- [ ] Study-design outputs include explicit threats and mitigation plans.
- [ ] Code and stats planning consumes the diagnostic report when present.
- [ ] Diagnostics produce insufficient-input notes when design information is missing.

---

## Phase 8: Analysis Plan And Reproducibility Pack

**Purpose:** Make analysis work executable, inspectable, and reproducible.

**Primary files to create:**

- `research-paper-workflow/templates/reproducible-analysis-pack.md`
- `research-paper-workflow/templates/analysis-script-order.md`
- `research-paper-workflow/templates/computational-environment.md`
- `scripts/audit_reproducibility_pack.py`
- `tests/test_reproducibility_pack.py`

**Primary files to modify:**

- `skills/I_code/code-specification.md`
- `skills/I_code/code-planning.md`
- `skills/I_code/code-builder.md`
- `skills/I_code/code-execution.md`
- `skills/I_code/reproducibility-auditor.md`
- `skills/I_code/release-packager.md`
- `research-paper-workflow/workflows/code-build.md`

**Tasks:**

- [ ] Define required reproducibility pack sections:
  - environment
  - data inputs
  - data exclusion rules
  - script execution order
  - random seeds
  - expected outputs
  - validation checks
  - known non-reproducible dependencies
- [ ] Update Stage I skills to produce or consume the pack.
- [ ] Add audit script for missing script order, missing environment, or untracked inputs.
- [ ] Add tests for a complete and incomplete reproducibility pack.

**Acceptance:**

- [ ] Code-build full flow produces a reproducibility plan before execution guidance.
- [ ] Reproducibility auditor can flag missing environment, script order, and expected outputs.

---

## Phase 9: Systematic Review And Qualitative Research Depth

**Purpose:** Improve non-empirical and qualitative workflows beyond generic synthesis.

**Primary files to create:**

- `research-paper-workflow/templates/screening-conflict-log.md`
- `research-paper-workflow/templates/coding-book.md`
- `research-paper-workflow/templates/negative-case-log.md`
- `research-paper-workflow/references/systematic-review-advanced-contract.md`
- `research-paper-workflow/references/qualitative-research-contract.md`
- `tests/test_review_and_qualitative_contracts.py`

**Primary files to modify:**

- `skills/B_literature/paper-screener.md`
- `skills/E_synthesis/quality-assessor.md`
- `skills/E_synthesis/evidence-synthesizer.md`
- `skills/E_synthesis/qualitative-coding.md`
- `skills/G_compliance/prisma-checker.md`
- `research-paper-workflow/templates/prisma-flowchart.md`
- `research-paper-workflow/templates/grade-summary-of-findings.md`

**Tasks:**

- [ ] Add screening conflict logging for systematic review workflows.
- [ ] Add explicit PRISMA 2020 alignment checks.
- [ ] Add ROB2 / GRADE artifact expectations where relevant.
- [ ] Add coding book and memo expectations for qualitative research.
- [ ] Add negative case analysis and intercoder agreement guidance.
- [ ] Add tests for systematic review and qualitative artifact completeness.

**Acceptance:**

- [ ] Systematic review workflow distinguishes search, screening, extraction, quality, and synthesis risks.
- [ ] Qualitative workflow produces coding, memoing, and negative-case artifacts instead of generic theme lists.

---

## Phase 10: Paragraph-Level Scholarly Writing Quality

**Purpose:** Move from complete outputs to outputs that read like careful academic writing.

**Primary files to create:**

- `research-paper-workflow/templates/paragraph-diagnostic-report.md`
- `research-paper-workflow/references/paragraph-quality-contract.md`
- `scripts/audit_paragraph_quality.py`
- `tests/test_paragraph_quality_contract.py`

**Primary files to modify:**

- `skills/F_writing/analysis-interpreter.md`
- `skills/F_writing/discussion-writer.md`
- `skills/F_writing/manuscript-architect.md`
- `skills/F_writing/meta-optimizer.md`
- `skills/J_proofread/human-voice-rewriter.md`
- `skills/J_proofread/final-proofreader.md`
- `research-paper-workflow/references/academic-output-rubric.md`

**Tasks:**

- [ ] Define paragraph diagnostic dimensions:
  - topic sentence clarity
  - evidence anchoring
  - mechanism
  - boundary condition
  - limitation
  - transition
  - overclaim risk
  - generic phrasing risk
- [ ] Update writing and proofreading skills to apply paragraph diagnostics.
- [ ] Add examples of weak and improved academic paragraphs.
- [ ] Add an audit script for missing evidence anchors and overclaim phrases in fixture paragraphs.
- [ ] Add tests for good, weak, and overclaiming paragraphs.

**Acceptance:**

- [ ] Writing skills can produce paragraph-level revision notes.
- [ ] Proofreading distinguishes grammar cleanup from scholarly argument repair.

---

## Phase 11: Contribution And Theory Calibration

**Purpose:** Make contribution statements precise enough for venue reviewers and domain scholars.

**Primary files to create:**

- `research-paper-workflow/templates/contribution-calibration.md`
- `research-paper-workflow/templates/theory-fit-matrix.md`
- `research-paper-workflow/references/contribution-taxonomy.md`
- `tests/test_contribution_calibration.py`

**Primary files to modify:**

- `skills/A_framing/contribution-crafter.md`
- `skills/A_framing/gap-analyzer.md`
- `skills/A_framing/theory-mapper.md`
- `skills/A_framing/hypothesis-generator.md`
- `skills/F_writing/manuscript-architect.md`

**Tasks:**

- [ ] Define contribution types:
  - theoretical
  - empirical
  - methodological
  - measurement
  - design artifact
  - practical
  - computational
- [ ] Add contribution calibration template with evidence, novelty, scope, and reviewer risk columns.
- [ ] Update theory mapper to separate theory use, theory extension, and theory testing.
- [ ] Update manuscript architect to require calibrated contribution statements before drafting.
- [ ] Add tests for contribution taxonomy coverage and artifact path alignment.

**Acceptance:**

- [ ] Contribution outputs state what changes in knowledge, for whom, and under what boundary conditions.
- [ ] Theory outputs distinguish borrowed framing from genuine theory contribution.

---

## Phase 12: Evaluation Corpus And Regression Scoring

**Purpose:** Make quality improvements measurable and prevent regression toward vague AI prose.

**Primary files to create:**

- `evals/academic_quality/README.md`
- `evals/academic_quality/cases/*.yaml`
- `evals/academic_quality/fixtures/`
- `scripts/run_academic_quality_evals.py`
- `scripts/score_academic_output.py`
- `tests/test_academic_quality_evals.py`

**Primary files to modify:**

- `docs/maintainer/skill-quality-contract.md`
- `.github/workflows/ci.yml`
- `scripts/release_preflight.sh`

**Tasks:**

- [ ] Define eval dimensions:
  - artifact completeness
  - evidence traceability
  - no fabricated sources
  - claim strength calibration
  - venue fit
  - method validity awareness
  - scholarly voice
- [ ] Add fixture cases for:
  - empirical causal design
  - systematic review
  - qualitative coding
  - theory contribution
  - code-first methods paper
  - reviewer rebuttal
- [ ] Add expected outputs or scoring rubrics for each case.
- [ ] Add a scoring script that returns JSON and markdown summaries.
- [ ] Add unit tests for scoring functions.
- [ ] Add release preflight warning for eval regression.
- [ ] Decide after one beta cycle whether eval regression should become a hard release gate.

**Acceptance:**

- [ ] Evals can run locally without network.
- [ ] Scores identify vague output, unsupported claims, and missing artifacts.
- [ ] Release preflight surfaces eval results in a clear summary.

---

## Phase 13: Plugin UX, Migration, And Runtime Doctor

**Purpose:** Reduce confusion between official plugin install, global skill install, and full runtime install.

**Primary files to create:**

- `docs/guide/plugin-migration.md`
- `docs/zh/guide/plugin-migration.md`
- `scripts/plugin_runtime_doctor.py`
- `tests/test_plugin_runtime_doctor.py`

**Primary files to modify:**

- `README.md`
- `docs/guide/install.md`
- `docs/zh/guide/install.md`
- `docs/reference/cli.md`
- `docs/zh/reference/cli.md`
- `scripts/research_skills_cli.sh`
- `scripts/install_research_skill.sh`

**Tasks:**

- [ ] Add a migration guide with paths:
  - new plugin-only user
  - existing partial user moving to plugin
  - existing full user keeping CLI/orchestrator
  - maintainer validating all surfaces
- [ ] Add `rsk doctor plugin` or equivalent CLI path if it fits the existing command model.
- [ ] Detect version mismatch across:
  - plugin manifest version
  - portable package version
  - global skill version
  - CLI source version
- [ ] Print direct remediation commands:
  - `rsk upgrade --target all --doctor`
  - `rsk clean --globals --dry-run`
  - official plugin reinstall path
- [ ] Add tests for plugin-only, global-only, mixed aligned, and mixed mismatched states.

**Acceptance:**

- [ ] Users can determine whether they need plugin-only, partial, or full.
- [ ] Mixed installs produce clear warnings instead of silent ambiguity.
- [ ] Migration docs are linked from README and install guides.

---

## Phase 14: Marketplace Artifact And Release Hardening

**Purpose:** Ensure the enhanced plugin and skill package ship cleanly across Codex, Claude Code, and Gemini.

**Primary files to modify:**

- `scripts/build_marketplace_artifacts.py`
- `scripts/verify_release_tag_version.sh`
- `scripts/release_preflight.sh`
- `scripts/release_postflight.sh`
- `tests/test_marketplace_artifacts.py`
- `tests/test_release_automation.py`

**Tasks:**

- [ ] Ensure new templates, references, venue profiles, eval metadata, and contracts are included in all marketplace artifacts.
- [ ] Add artifact tests for evidence ledger, research state, venue profiles, and eval docs.
- [ ] Add release preflight checks for:
  - plugin version alignment
  - package sync freshness
  - evidence contract validity
  - research-state contract validity
  - eval script health
- [ ] Keep postflight focused on release availability and remote verification, not long-running quality analysis.
- [ ] Update release notes generation to mention plugin/runtime compatibility guidance.

**Acceptance:**

- [ ] Marketplace artifacts contain all new runtime references.
- [ ] Release automation fails fast on missing package sync or invalid contracts.
- [ ] Release notes clearly state plugin-only vs full-runtime behavior.

---

## Phase 15: Documentation And User Education

**Purpose:** Make the system understandable for ordinary users, advanced users, and maintainers.

**Primary files to modify or create:**

- `README.md`
- `docs/quickstart.md`
- `docs/guide/install.md`
- `docs/zh/guide/install.md`
- `docs/guide/plugin-migration.md`
- `docs/zh/guide/plugin-migration.md`
- `docs/reference/skills.md`
- `docs/zh/reference/skills.md`
- `docs/maintainer/skill-quality-contract.md`
- `docs/advanced/plugin-first-architecture.md`

**Tasks:**

- [ ] Add three user paths:
  - ordinary plugin user
  - full runtime user
  - maintainer/release operator
- [ ] Add examples for common workflows:
  - `/paper`
  - `/lit-review`
  - `/study-design`
  - `/code-build`
  - `/submission-prep`
  - `/rebuttal`
- [ ] Add examples showing how evidence ledger and research state evolve across a paper.
- [ ] Add a table explaining plugin vs skill vs CLI vs orchestrator.
- [ ] Add a troubleshooting section for mixed plugin/global installs.
- [ ] Add Chinese equivalents for migration and install changes.

**Acceptance:**

- [ ] A new user can choose plugin-only or full runtime without reading implementation docs.
- [ ] A maintainer can find all quality gates and release checks from docs.
- [ ] Chinese and English docs are conceptually aligned.

---

## Phase 16: Final Verification And Release Candidate

**Purpose:** Validate the full enhancement set before tagging a new beta.

**Required verification commands:**

```bash
python3 scripts/validate_research_standard.py --strict
python3 scripts/audit_skill_sections.py --strict
bash scripts/sync_skill_package.sh --target all --dry-run
python3 -m unittest discover -s tests -v
python3 -m unittest tests.test_plugin_distribution_contract tests.test_marketplace_artifacts tests.test_release_automation -v
```

**Release candidate tasks:**

- [ ] Run all required verification commands.
- [ ] Run any new academic quality eval command added in Phase 12.
- [ ] Build marketplace artifacts locally.
- [ ] Inspect artifact contents for new templates and references.
- [ ] Run `./scripts/release_ready.sh --version <next-version> --skip-bump` after version files are aligned.
- [ ] Publish only after branch merge and explicit version/tag decision.

**Acceptance:**

- [ ] All tests pass.
- [ ] All quality contracts pass.
- [ ] Marketplace artifacts include new capabilities.
- [ ] Release docs explain plugin-only, partial, and full runtime compatibility.

---

## Recommended Implementation Order

1. Phase 0: Baseline and scope lock.
2. Phase 1: Evidence ledger and claim traceability.
3. Phase 3: Research state and long-running context.
4. Phase 12: Evaluation corpus and regression scoring.
5. Phase 13: Plugin UX, migration, and runtime doctor.
6. Phase 5: Venue-aware research workflows.
7. Phase 7: Methods and research design diagnostics.
8. Phase 10: Paragraph-level scholarly writing quality.
9. Phase 2: Citation risk and source integrity.
10. Phase 4: Stage handoff protocol.
11. Phase 6: Reviewer model and rebuttal intelligence.
12. Phase 8: Analysis plan and reproducibility pack.
13. Phase 9: Systematic review and qualitative research depth.
14. Phase 11: Contribution and theory calibration.
15. Phase 14: Marketplace artifact and release hardening.
16. Phase 15: Documentation and user education.
17. Phase 16: Final verification and release candidate.

This order builds the quality substrate first, then domain-specific depth, then release hardening.

---

## First Milestone Cut

Use this milestone if the next work needs to stay small enough for one focused branch:

- [ ] Phase 0 baseline updates.
- [ ] Phase 1 evidence ledger templates, contract, and tests.
- [ ] Phase 3 research-state contract updates without orchestrator changes.
- [ ] Phase 12 minimal offline eval harness with two fixture cases.
- [ ] Phase 13 documentation-only migration guide.

**Milestone acceptance:**

- [ ] Evidence ledger and research state are documented and bundled.
- [ ] At least two eval cases run offline.
- [ ] Plugin/global/full compatibility guidance is discoverable.
- [ ] No orchestrator behavior changes are required for this milestone.

