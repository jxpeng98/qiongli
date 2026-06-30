# Cross-Stage Multiagent Collaboration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-stage multiagent collaboration layer so any Qiongli Task ID can use role-based worker delegation, merge adjudication, and independent final review.

**Architecture:** Keep the existing `task-run` and `worker_plan` contract as the execution surface. Add a cross-stage collaboration standard that maps Task ID stage families (`A` through `K`) to default worker roles, with task-specific legacy configs (`B1`, `H3`) still taking precedence. The orchestrator resolves the collaboration profile into worker packets, scoped artifact roots, controller metadata, merge prompts, and final review prompts.

**Tech Stack:** Python 3.12, `unittest`, YAML standards under `content/standards`, role YAML under `content/roles`, Markdown docs under `docs/advanced` and `docs/zh/advanced`.

---

## File Structure

- Create: `content/roles/senior-research-director.yaml`
  - Defines the experienced controller role responsible for task decomposition, worker assignment, conflict adjudication, and final gate discipline.
- Create: `content/standards/cross-stage-multiagent-collaboration.yaml`
  - Defines `cross_stage_research_team`, stage defaults for `A` through `K`, and a worker role library.
- Modify: `content/standards/worker-orchestration-contract.yaml`
  - Adds optional collaboration metadata fields without changing the existing required worker packet fields.
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
  - Loads the new collaboration standard, resolves fallback worker configs for any Task ID, enriches worker packets with role-aware goals, and includes controller/review metadata in prompts.
- Modify: `tests/test_worker_orchestration_contract.py`
  - Verifies the new collaboration standard, stage coverage, role references, and backward compatibility for legacy `B1` and `H3` config.
- Modify: `tests/test_worker_orchestration_runtime.py`
  - Adds runtime tests for non-legacy stages such as `A1`, `F3`, and `I5`.
- Create: `docs/advanced/cross-stage-multiagent-collaboration.md`
  - Explains the cross-stage controller/worker/reviewer model.
- Create: `docs/zh/advanced/cross-stage-multiagent-collaboration.md`
  - Chinese user-facing version of the same guide.
- Modify: `docs/advanced/index.md`
  - Adds the new advanced guide.
- Modify: `docs/zh/advanced/index.md`
  - Adds the new Chinese advanced guide.
- Modify: `docs/guide/multi-agent.md`
  - Adds CLI examples for arbitrary-stage worker orchestration.
- Modify: `docs/advanced/agent-skill-collaboration.md`
  - Updates the capability enhancement guide to point to the cross-stage collaboration profile.

---

### Task 1: Add The Controller Role And Cross-Stage Standard

**Files:**
- Create: `content/roles/senior-research-director.yaml`
- Create: `content/standards/cross-stage-multiagent-collaboration.yaml`
- Modify: `tests/test_worker_orchestration_contract.py`

- [ ] **Step 1: Write the failing contract tests**

Add these constants near the existing path constants in `tests/test_worker_orchestration_contract.py`:

```python
COLLABORATION_PATH = LAYOUT.standards / "cross-stage-multiagent-collaboration.yaml"
SENIOR_DIRECTOR_ROLE_PATH = LAYOUT.roles / "senior-research-director.yaml"
```

Add this test method inside `WorkerOrchestrationContractTests`:

```python
    def test_cross_stage_collaboration_standard_covers_all_stage_families(self) -> None:
        self.assertTrue(COLLABORATION_PATH.exists(), f"Missing {COLLABORATION_PATH}")
        self.assertTrue(SENIOR_DIRECTOR_ROLE_PATH.exists(), f"Missing {SENIOR_DIRECTOR_ROLE_PATH}")

        standard = yaml.safe_load(COLLABORATION_PATH.read_text(encoding="utf-8")) or {}
        self.assertEqual("1.0.0", standard.get("contract_version"))
        self.assertEqual("cross_stage_research_team", standard.get("collaboration_profile"))
        self.assertEqual("senior-research-director", standard.get("controller_role"))

        stage_defaults = standard.get("stage_defaults", {})
        self.assertEqual(set("ABCDEFGHIJK"), set(stage_defaults))

        worker_library = standard.get("worker_role_library", {})
        self.assertIsInstance(worker_library, dict)
        self.assertGreaterEqual(len(worker_library), 10)

        worker_id_pattern = re.compile(r"^[a-z][a-z0-9_]*$")
        for stage, block in stage_defaults.items():
            with self.subTest(stage=stage):
                self.assertIn(block["default_mode"], ORCHESTRATION_MODES)
                self.assertIn(block["merge_policy"], MERGE_POLICIES)
                self.assertIsInstance(block["partition_strategy"], str)
                self.assertTrue(block["partition_strategy"].strip())
                self.assertIsInstance(block["worker_pool"], list)
                self.assertGreater(len(block["worker_pool"]), 0)
                self.assertLessEqual(len(block["worker_pool"]), block["max_workers"])
                for worker_id in block["worker_pool"]:
                    self.assertIsNotNone(worker_id_pattern.fullmatch(worker_id))
                    self.assertIn(worker_id, worker_library)
                self.assertEqual(
                    {"min_success_ratio", "on_failure"},
                    set(block["barrier_rules"]),
                )

        director = yaml.safe_load(SENIOR_DIRECTOR_ROLE_PATH.read_text(encoding="utf-8")) or {}
        self.assertEqual("senior-research-director", director.get("id"))
        self.assertIn("model-collaborator", director.get("preferred_skills", []))
        self.assertTrue(director.get("quality_thresholds", {}).get("require_conflict_matrix"))
```

- [ ] **Step 2: Run the contract test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_cross_stage_collaboration_standard_covers_all_stage_families -v
```

Expected: FAIL because `content/standards/cross-stage-multiagent-collaboration.yaml` and `content/roles/senior-research-director.yaml` do not exist.

- [ ] **Step 3: Add `content/roles/senior-research-director.yaml`**

Create the file with this content:

```yaml
# -------------------------------------------------
# Academic Role: Senior Research Director
# -------------------------------------------------

id: senior-research-director
display_name: "Senior Research Director"
description: "Experienced cross-stage controller for task decomposition, worker assignment, merge adjudication, and independent review discipline across the full research lifecycle."

preferred_skills:
  - model-collaborator
  - self-critique
  - boundary-interviewer
  - question-refiner
  - academic-searcher
  - literature-mapper
  - study-designer
  - robustness-planner
  - manuscript-architect
  - reporting-checker
  - fatal-flaw-detector

quality_thresholds:
  require_task_id_alignment: true
  require_worker_plan: true
  require_scoped_worker_artifacts: true
  require_conflict_matrix: true
  require_gap_summary: true
  require_independent_final_review: true
  prohibit_worker_writes_to_canonical_outputs: true
  require_controller_adjudication: true

tone: "senior, precise, adversarial when needed, evidence-first, and contract-driven"

escalation_rules:
  - condition: "task_scope_unclear"
    action: "pause worker dispatch and run boundary-interviewer before fanout"
  - condition: "missing_required_mcp_for_stage"
    action: "record provider limitation and downgrade to strategy_only or block when strict mode is active"
  - condition: "worker_attempts_canonical_write"
    action: "reject worker output and require rerun under scoped artifacts"
  - condition: "inter_worker_conflict_unresolved"
    action: "record conflict matrix and adjudicate before canonical output update"
  - condition: "final_review_blocks"
    action: "halt downstream execution until required revisions are addressed"

default_domain: null
```

- [ ] **Step 4: Add `content/standards/cross-stage-multiagent-collaboration.yaml`**

Create the file with this content:

```yaml
contract_version: "1.0.0"
collaboration_profile: cross_stage_research_team
controller_role: senior-research-director
description: "Stage-aware multiagent worker defaults for any Qiongli Task ID."

stage_defaults:
  A:
    default_mode: delegated_workers
    partition_strategy: by_idea_risk
    max_workers: 3
    worker_pool:
      - framing_theory_worker
      - contribution_risk_reviewer
      - venue_boundary_reviewer
    merge_policy: controller_adjudication
    review_focus: "answerability, contribution clarity, construct boundaries, venue fit"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  B:
    default_mode: delegated_workers
    partition_strategy: by_literature_workstream
    max_workers: 4
    worker_pool:
      - literature_search_worker
      - screening_worker
      - extraction_worker
      - citation_mapping_reviewer
    merge_policy: synthesize_with_conflict_matrix
    review_focus: "search coverage, screening consistency, extraction fidelity, citation risk"
    barrier_rules:
      min_success_ratio: 0.6
      on_failure: degrade

  C:
    default_mode: delegated_workers
    partition_strategy: by_design_risk
    max_workers: 4
    worker_pool:
      - methods_design_worker
      - data_strategy_worker
      - robustness_reviewer
      - measurement_reviewer
    merge_policy: controller_adjudication
    review_focus: "identification, measurement, data feasibility, robustness"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  D:
    default_mode: review_swarm
    partition_strategy: by_ethics_surface
    max_workers: 2
    worker_pool:
      - ethics_privacy_worker
      - disclosure_statement_reviewer
    merge_policy: controller_adjudication
    review_focus: "participant risk, privacy, consent, disclosure completeness"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  E:
    default_mode: delegated_workers
    partition_strategy: by_synthesis_risk
    max_workers: 3
    worker_pool:
      - evidence_synthesis_worker
      - quality_assessment_worker
      - statistics_review_worker
    merge_policy: synthesize_with_conflict_matrix
    review_focus: "effect size validity, heterogeneity, certainty, publication bias"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  F:
    default_mode: delegated_workers
    partition_strategy: by_manuscript_integrity
    max_workers: 3
    worker_pool:
      - manuscript_architect_worker
      - claim_evidence_worker
      - consistency_reviewer
    merge_policy: controller_adjudication
    review_focus: "story spine, claim evidence, cross-section consistency, overclaiming"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  G:
    default_mode: review_swarm
    partition_strategy: by_reporting_gate
    max_workers: 2
    worker_pool:
      - reporting_compliance_worker
      - tone_consistency_reviewer
    merge_policy: controller_adjudication
    review_focus: "reporting checklist, tone normalization, internal consistency"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  H:
    default_mode: review_swarm
    partition_strategy: by_submission_risk
    max_workers: 3
    worker_pool:
      - submission_package_worker
      - reviewer_risk_worker
      - fatal_flaw_reviewer
    merge_policy: controller_adjudication
    review_focus: "submission completeness, reviewer objections, fatal flaw risk"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  I:
    default_mode: delegated_workers
    partition_strategy: by_code_research_risk
    max_workers: 3
    worker_pool:
      - code_spec_worker
      - stats_validation_worker
      - reproducibility_reviewer
    merge_policy: controller_adjudication
    review_focus: "method fidelity, statistical validity, reproducibility evidence"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  J:
    default_mode: review_swarm
    partition_strategy: by_proofread_risk
    max_workers: 3
    worker_pool:
      - ai_fingerprint_worker
      - human_voice_editor
      - final_proofreader
    merge_policy: consensus_then_gaps
    review_focus: "AI trace, human voice, citation and similarity risk"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

  K:
    default_mode: delegated_workers
    partition_strategy: by_presentation_layer
    max_workers: 3
    worker_pool:
      - presentation_arc_worker
      - slide_architecture_worker
      - delivery_reviewer
    merge_policy: consensus_then_gaps
    review_focus: "talk arc, slide assertions, audience fit, delivery risk"
    barrier_rules:
      min_success_ratio: 1.0
      on_failure: block

adapter_preference:
  codex: codex_subagent
  claude: claude_cowork
  antigravity: generic_prompt

worker_role_library:
  framing_theory_worker:
    functional_role: theory-agent
    goal_template: "Stress-test and refine the Task {task_id} framing for {topic}, focusing on constructs, mechanisms, contribution, and non-claims."
    required_skills_extra:
      - question-refiner
      - theory-mapper
      - gap-analyzer
    stop_conditions:
      - "Stop if the research question is not answerable within one paper."
      - "Stop if contribution type or non-claims are unclear."

  contribution_risk_reviewer:
    functional_role: pi
    goal_template: "Review Task {task_id} for contribution risk, novelty weakness, and reviewer-visible framing problems."
    required_skills_extra:
      - gap-analyzer
      - fatal-flaw-detector
    stop_conditions:
      - "Stop if the contribution cannot be distinguished from prior work."
      - "Stop if the weakest assumption is not explicit."

  venue_boundary_reviewer:
    functional_role: theory-agent
    goal_template: "Check Task {task_id} against venue expectations, boundary conditions, and claim strength."
    required_skills_extra:
      - venue-analyzer
      - boundary-interviewer
    stop_conditions:
      - "Stop if the target audience or venue standard changes the task scope."

  literature_search_worker:
    functional_role: literature-ra
    goal_template: "Execute the search-facing part of Task {task_id} for {topic}, recording sources, queries, and provider limitations."
    required_skills_extra:
      - academic-searcher
      - concept-extractor
      - fulltext-fetcher
    stop_conditions:
      - "Stop if provider coverage is strategy_only and review-grade claims would be unsafe."

  screening_worker:
    functional_role: literature-ra
    goal_template: "Screen and classify literature candidates for Task {task_id}, preserving inclusion and exclusion rationale."
    required_skills_extra:
      - paper-screener
    stop_conditions:
      - "Stop if inclusion criteria are underspecified."

  extraction_worker:
    functional_role: literature-ra
    goal_template: "Extract structured study details for Task {task_id}, focusing on methods, findings, limitations, and evidence quality."
    required_skills_extra:
      - paper-extractor
      - quality-assessor
    stop_conditions:
      - "Stop if extraction would require unsupported full-text access."

  citation_mapping_reviewer:
    functional_role: literature-ra
    goal_template: "Audit citation coverage, snowballing opportunities, and missing schools of thought for Task {task_id}."
    required_skills_extra:
      - citation-snowballer
      - literature-mapper
    stop_conditions:
      - "Stop if citation expansion would materially change the corpus."

  methods_design_worker:
    functional_role: methods-lead
    goal_template: "Design or audit the method structure for Task {task_id}, focusing on identification, validity, and analysis alignment."
    required_skills_extra:
      - study-designer
      - rival-hypothesis-designer
    stop_conditions:
      - "Stop if identification or design logic is not defensible."

  data_strategy_worker:
    functional_role: data-agent
    goal_template: "Assess data availability, provenance, privacy, and pipeline readiness for Task {task_id}."
    required_skills_extra:
      - dataset-finder
      - data-management-plan
      - variable-constructor
    stop_conditions:
      - "Stop if no data source or data generation path is specified."

  robustness_reviewer:
    functional_role: methods-lead
    goal_template: "Review robustness and rival explanations for Task {task_id}."
    required_skills_extra:
      - robustness-planner
      - fatal-flaw-detector
    stop_conditions:
      - "Stop if a plausible rival explanation is not addressed."

  measurement_reviewer:
    functional_role: methods-lead
    goal_template: "Review construct measurement, variable operationalization, and instrument validity for Task {task_id}."
    required_skills_extra:
      - variable-operationalizer
      - variable-constructor
    stop_conditions:
      - "Stop if constructs and measures are not aligned."

  ethics_privacy_worker:
    functional_role: compliance-officer
    goal_template: "Review ethics, consent, privacy, and participant protection surfaces for Task {task_id}."
    required_skills_extra:
      - ethics-irb-helper
      - deidentification-planner
    stop_conditions:
      - "Stop if participant or privacy risk is unresolved."

  disclosure_statement_reviewer:
    functional_role: compliance-officer
    goal_template: "Review disclosure, data availability, funding, conflict, and AI statement needs for Task {task_id}."
    required_skills_extra:
      - statement-generator
      - reporting-checker
    stop_conditions:
      - "Stop if required disclosure status is unknown."

  evidence_synthesis_worker:
    functional_role: statistician
    goal_template: "Synthesize evidence for Task {task_id}, preserving uncertainty, heterogeneity, and outcome boundaries."
    required_skills_extra:
      - evidence-synthesizer
      - effect-size-calculator
    stop_conditions:
      - "Stop if effect sizes or synthesis units are not comparable."

  quality_assessment_worker:
    functional_role: literature-ra
    goal_template: "Assess study quality and certainty for Task {task_id}."
    required_skills_extra:
      - quality-assessor
      - publication-bias-checker
    stop_conditions:
      - "Stop if quality criteria are missing."

  statistics_review_worker:
    functional_role: statistician
    goal_template: "Review statistical synthesis, assumptions, diagnostics, and uncertainty for Task {task_id}."
    required_skills_extra:
      - stats-engine
      - publication-bias-checker
    stop_conditions:
      - "Stop if model assumptions are violated without sensitivity analysis."

  manuscript_architect_worker:
    functional_role: academic-writer
    goal_template: "Architect the manuscript-facing output for Task {task_id}, preserving story spine and section jobs."
    required_skills_extra:
      - manuscript-architect
    stop_conditions:
      - "Stop if story spine or evidence threshold is unclear."

  claim_evidence_worker:
    functional_role: academic-writer
    goal_template: "Audit claims, evidence support, citation risk, and overclaiming for Task {task_id}."
    required_skills_extra:
      - manuscript-architect
      - self-critique
    stop_conditions:
      - "Stop if key claims lack evidence anchors."

  consistency_reviewer:
    functional_role: research-orchestrator
    goal_template: "Review cross-section consistency, artifact paths, handoff readiness, and contract alignment for Task {task_id}."
    required_skills_extra:
      - reporting-checker
      - self-critique
    stop_conditions:
      - "Stop if canonical outputs would contradict upstream artifacts."

  reporting_compliance_worker:
    functional_role: compliance-officer
    goal_template: "Check reporting guideline completeness for Task {task_id}."
    required_skills_extra:
      - reporting-checker
      - prisma-checker
    stop_conditions:
      - "Stop if required checklist family is unknown."

  tone_consistency_reviewer:
    functional_role: academic-writer
    goal_template: "Review tone, style, academic specificity, and consistency for Task {task_id}."
    required_skills_extra:
      - tone-normalizer
      - final-proofreader
    stop_conditions:
      - "Stop if tone changes would alter scientific meaning."

  submission_package_worker:
    functional_role: compliance-officer
    goal_template: "Assemble or audit submission package needs for Task {task_id}."
    required_skills_extra:
      - submission-packager
      - credit-taxonomy-helper
    stop_conditions:
      - "Stop if venue-specific submission requirements are unavailable."

  reviewer_risk_worker:
    functional_role: pi
    goal_template: "Simulate reviewer objections and revision risks for Task {task_id}."
    required_skills_extra:
      - peer-review-simulation
      - reviewer-empathy-checker
    stop_conditions:
      - "Stop if a high-probability reviewer objection changes the required artifact."

  fatal_flaw_reviewer:
    functional_role: pi
    goal_template: "Run a desk-reject and fatal flaw scan for Task {task_id}."
    required_skills_extra:
      - fatal-flaw-detector
      - limitation-auditor
    stop_conditions:
      - "Stop if fatal flaw risk is high and unresolved."

  code_spec_worker:
    functional_role: academic-code-reviewer
    goal_template: "Specify or audit code constraints for Task {task_id}, focusing on method fidelity and decision boundaries."
    required_skills_extra:
      - code-specification
      - code-planning
    stop_conditions:
      - "Stop if the code task lacks a testable specification."

  stats_validation_worker:
    functional_role: statistician
    goal_template: "Validate statistical implementation assumptions and diagnostics for Task {task_id}."
    required_skills_extra:
      - stats-engine
      - code-review
    stop_conditions:
      - "Stop if inference risks cannot be tested."

  reproducibility_reviewer:
    functional_role: academic-code-reviewer
    goal_template: "Review reproducibility, rerun evidence, seeds, dependencies, and package readiness for Task {task_id}."
    required_skills_extra:
      - reproducibility-auditor
      - release-packager
    stop_conditions:
      - "Stop if reproducibility evidence is unverifiable."

  ai_fingerprint_worker:
    functional_role: academic-writer
    goal_template: "Review Task {task_id} text for AI trace, generic phrasing, and voice risk without changing scientific meaning."
    required_skills_extra:
      - ai-fingerprint-scanner
    stop_conditions:
      - "Stop if source text is unavailable."

  human_voice_editor:
    functional_role: academic-writer
    goal_template: "Rewrite flagged Task {task_id} passages for human academic voice while preserving claims."
    required_skills_extra:
      - human-voice-rewriter
      - tone-normalizer
    stop_conditions:
      - "Stop if rewriting would require unsupported substantive changes."

  final_proofreader:
    functional_role: compliance-officer
    goal_template: "Proofread Task {task_id} for grammar, formatting, references, and final consistency."
    required_skills_extra:
      - final-proofreader
      - similarity-checker
    stop_conditions:
      - "Stop if citation or similarity risk requires source verification."

  presentation_arc_worker:
    functional_role: academic-writer
    goal_template: "Design the talk-level argument arc for Task {task_id}."
    required_skills_extra:
      - presentation-planner
    stop_conditions:
      - "Stop if audience or duration is unknown."

  slide_architecture_worker:
    functional_role: academic-writer
    goal_template: "Design slide-level assertions, sequence, and evidence placement for Task {task_id}."
    required_skills_extra:
      - slide-architect
    stop_conditions:
      - "Stop if claims cannot be mapped to slides."

  delivery_reviewer:
    functional_role: pi
    goal_template: "Review presentation risks, audience fit, and defensibility for Task {task_id}."
    required_skills_extra:
      - presentation-planner
      - peer-review-simulation
    stop_conditions:
      - "Stop if the presentation overstates results or hides limitations."
```

- [ ] **Step 5: Run the contract test and verify it passes**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_cross_stage_collaboration_standard_covers_all_stage_families -v
```

Expected: PASS.

- [ ] **Step 6: Commit Task 1**

Run:

```bash
git add content/roles/senior-research-director.yaml content/standards/cross-stage-multiagent-collaboration.yaml tests/test_worker_orchestration_contract.py
git commit -m "feat: add cross-stage multiagent collaboration standard"
```

---

### Task 2: Resolve Stage Defaults For Any Task ID

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `tests/test_worker_orchestration_runtime.py`

- [ ] **Step 1: Write the failing runtime fallback test**

Add this test method to `WorkerOrchestrationRuntimeTests` in `tests/test_worker_orchestration_runtime.py`:

```python
    def test_task_run_uses_stage_default_worker_plan_for_f3(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="cross-stage-writing",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=2,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("cross_stage_research_team", worker_state["collaboration_profile"])
        self.assertEqual("senior-research-director", worker_state["controller_role"])
        self.assertEqual("F", worker_state["stage_family"])
        self.assertEqual("by_manuscript_integrity", worker_state["partition_strategy"])
        self.assertEqual("controller_adjudication", worker_state["merge_policy"])
        self.assertEqual(2, len(worker_state["workers"]))
        self.assertEqual(
            ["manuscript_architect_worker", "claim_evidence_worker"],
            [worker["id"] for worker in worker_state["workers"]],
        )
        self.assertIn("story spine", worker_state["review_focus"])
        self.assertIn("Worker Orchestration", result.merged_analysis)
```

- [ ] **Step 2: Run the fallback test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_task_run_uses_stage_default_worker_plan_for_f3 -v
```

Expected: FAIL because `_load_worker_orchestration_config("F3")` returns no config and the worker state is skipped.

- [ ] **Step 3: Add cross-stage loading helpers**

In `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`, add these methods near `_load_worker_orchestration_config`:

```python
    @staticmethod
    def _stage_family_for_task(task_id: str) -> str:
        normalized = str(task_id).strip().upper()
        return normalized[:1] if normalized else ""

    def _load_cross_stage_collaboration_standard(self) -> dict[str, Any]:
        try:
            standard = self._load_yaml("cross-stage-multiagent-collaboration.yaml")
        except FileNotFoundError:
            return {}
        return standard if isinstance(standard, dict) else {}

    def _normalize_worker_config_block(
        self,
        block: dict[str, Any],
        *,
        task_id: str,
        stage_family: str,
        collaboration_profile: str = "",
        controller_role: str = "",
        adapter_preference: dict[str, Any] | None = None,
        worker_role_library: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        worker_pool = [
            str(worker).strip()
            for worker in block.get("worker_pool", [])
            if str(worker).strip()
        ]
        normalized_adapter_preference: dict[str, str] = {}
        raw_adapter_preference = adapter_preference
        if raw_adapter_preference is None:
            raw_adapter_preference = block.get("adapter_preference", {})
        if isinstance(raw_adapter_preference, dict):
            for runtime_agent in RUNTIME_AGENT_CHOICES:
                normalized_adapter_preference[runtime_agent] = self._normalize_worker_adapter(
                    str(raw_adapter_preference.get(runtime_agent, "generic_prompt"))
                )

        raw_barrier_rules = block.get("barrier_rules", {})
        barrier_rules = raw_barrier_rules if isinstance(raw_barrier_rules, dict) else {}
        raw_max_workers = block.get("max_workers", len(worker_pool) or 1)
        config_max_workers = (
            raw_max_workers
            if type(raw_max_workers) is int and raw_max_workers > 0
            else len(worker_pool) or 1
        )
        normalized_role_library = worker_role_library if isinstance(worker_role_library, dict) else {}
        worker_roles: dict[str, dict[str, Any]] = {}
        for worker_id in worker_pool:
            role_block = normalized_role_library.get(worker_id, {})
            worker_roles[worker_id] = role_block if isinstance(role_block, dict) else {}

        return {
            "default_mode": self._normalize_worker_mode(
                str(block.get("default_mode", "none"))
            ),
            "adapter_preference": normalized_adapter_preference,
            "partition_strategy": str(block.get("partition_strategy", "")).strip(),
            "max_workers": config_max_workers,
            "worker_pool": worker_pool,
            "worker_roles": worker_roles,
            "merge_policy": str(block.get("merge_policy", "")).strip(),
            "barrier_rules": {
                "min_success_ratio": float(barrier_rules.get("min_success_ratio", 1.0)),
                "on_failure": str(barrier_rules.get("on_failure", "block")).strip() or "block",
            },
            "collaboration_profile": collaboration_profile,
            "controller_role": controller_role,
            "stage_family": stage_family,
            "review_focus": str(block.get("review_focus", "")).strip(),
            "source": str(block.get("source", "")).strip(),
            "task_id": task_id,
        }
```

- [ ] **Step 4: Replace `_load_worker_orchestration_config` with merged legacy and stage fallback logic**

Replace the existing `_load_worker_orchestration_config` method with:

```python
    def _load_worker_orchestration_config(self, task_id: str) -> dict[str, Any]:
        """Load worker orchestration config for a task.

        Precedence:
        1. Legacy task-specific config in mcp-agent-capability-map.yaml.
        2. Task override in cross-stage-multiagent-collaboration.yaml.
        3. Stage default in cross-stage-multiagent-collaboration.yaml.
        """
        normalized_task = str(task_id).strip().upper()
        stage_family = self._stage_family_for_task(normalized_task)
        collaboration_standard = self._load_cross_stage_collaboration_standard()
        collaboration_profile = str(
            collaboration_standard.get("collaboration_profile", "")
        ).strip()
        controller_role = str(collaboration_standard.get("controller_role", "")).strip()
        standard_adapter_preference = collaboration_standard.get("adapter_preference", {})
        worker_role_library = collaboration_standard.get("worker_role_library", {})

        capability_map = self._load_yaml("mcp-agent-capability-map.yaml")
        worker_config = capability_map.get("worker_orchestration_config", {})
        if isinstance(worker_config, dict):
            legacy_block = worker_config.get(normalized_task)
            if isinstance(legacy_block, dict):
                config = self._normalize_worker_config_block(
                    legacy_block,
                    task_id=normalized_task,
                    stage_family=stage_family,
                    collaboration_profile=collaboration_profile,
                    controller_role=controller_role,
                    adapter_preference=legacy_block.get(
                        "adapter_preference",
                        standard_adapter_preference,
                    ),
                    worker_role_library=worker_role_library,
                )
                config["source"] = "task_config"
                return config

        if isinstance(collaboration_standard, dict):
            task_overrides = collaboration_standard.get("task_overrides", {})
            if isinstance(task_overrides, dict):
                task_block = task_overrides.get(normalized_task)
                if isinstance(task_block, dict):
                    config = self._normalize_worker_config_block(
                        task_block,
                        task_id=normalized_task,
                        stage_family=stage_family,
                        collaboration_profile=collaboration_profile,
                        controller_role=controller_role,
                        adapter_preference=standard_adapter_preference,
                        worker_role_library=worker_role_library,
                    )
                    config["source"] = "task_override"
                    return config

            stage_defaults = collaboration_standard.get("stage_defaults", {})
            stage_block = stage_defaults.get(stage_family) if isinstance(stage_defaults, dict) else None
            if isinstance(stage_block, dict):
                config = self._normalize_worker_config_block(
                    stage_block,
                    task_id=normalized_task,
                    stage_family=stage_family,
                    collaboration_profile=collaboration_profile,
                    controller_role=controller_role,
                    adapter_preference=standard_adapter_preference,
                    worker_role_library=worker_role_library,
                )
                config["source"] = "stage_default"
                return config

        return {}
```

- [ ] **Step 5: Run the fallback test and verify it now reaches a different failure**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_task_run_uses_stage_default_worker_plan_for_f3 -v
```

Expected: FAIL on missing `collaboration_profile`, `controller_role`, `stage_family`, or `review_focus` in `worker_state`, because the plan builder has not copied these fields yet.

- [ ] **Step 6: Commit Task 2**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_worker_orchestration_runtime.py
git commit -m "feat: resolve cross-stage worker orchestration defaults"
```

---

### Task 3: Add Role-Aware Worker Plan Metadata

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `tests/test_worker_orchestration_runtime.py`

- [ ] **Step 1: Write the role-aware worker metadata test**

Add this test method to `WorkerOrchestrationRuntimeTests`:

```python
    def test_stage_default_workers_get_role_specific_goals_and_skills(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="A1",
            paper_type="theory",
            topic="platform-governance",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=1,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("A", worker_state["stage_family"])
        self.assertEqual("senior-research-director", worker_state["controller_role"])
        worker = worker_state["workers"][0]
        self.assertEqual("framing_theory_worker", worker["id"])
        self.assertEqual("theory-agent", worker["functional_role"])
        self.assertIn("platform-governance", worker["goal"])
        self.assertIn("A1", worker["goal"])
        self.assertIn("theory-mapper", worker["required_skills"])
        self.assertIn("Stop if the research question is not answerable within one paper.", worker["stop_conditions"])
```

- [ ] **Step 2: Run the role-aware test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_stage_default_workers_get_role_specific_goals_and_skills -v
```

Expected: FAIL because workers still use generic goals and `functional_role` equal to worker ID.

- [ ] **Step 3: Add helper methods for role rendering**

Add these methods near `_unique_strings`:

```python
    @staticmethod
    def _format_worker_goal_template(
        template: str,
        *,
        task_id: str,
        paper_type: str,
        topic: str,
    ) -> str:
        if not template:
            return f"Execute scoped {task_id} worker analysis for {topic}."
        return template.format(
            task_id=task_id,
            paper_type=paper_type,
            topic=topic,
        )

    def _worker_required_skills(
        self,
        task_packet: dict[str, Any],
        role_config: dict[str, Any],
    ) -> list[str]:
        base_skills = list(task_packet.get("required_skills", []))
        extra_skills = role_config.get("required_skills_extra", [])
        if not isinstance(extra_skills, list):
            extra_skills = []
        return self._unique_strings(base_skills + list(extra_skills))
```

- [ ] **Step 4: Update `_build_worker_orchestration_plan` metadata and worker creation**

Inside `_build_worker_orchestration_plan`, after `merge_policy = str(worker_config.get("merge_policy", ""))`, add:

```python
        collaboration_profile = str(worker_config.get("collaboration_profile", "")).strip()
        controller_role = str(worker_config.get("controller_role", "")).strip()
        stage_family = str(worker_config.get("stage_family", "")).strip()
        review_focus = str(worker_config.get("review_focus", "")).strip()
        worker_roles = worker_config.get("worker_roles", {})
        if not isinstance(worker_roles, dict):
            worker_roles = {}
```

Replace the existing `for worker_id in worker_pool:` loop with:

```python
        for worker_id in worker_pool:
            role_config = worker_roles.get(worker_id, {})
            if not isinstance(role_config, dict):
                role_config = {}
            worker_root = self._join_artifact_path(
                run_root,
                "workers",
                worker_id,
                trailing_slash=True,
            )
            functional_role = str(role_config.get("functional_role", worker_id)).strip() or worker_id
            goal = self._format_worker_goal_template(
                str(role_config.get("goal_template", "")).strip(),
                task_id=task_id,
                paper_type=paper_type,
                topic=topic,
            )
            role_stop_conditions = role_config.get("stop_conditions", [])
            if not isinstance(role_stop_conditions, list):
                role_stop_conditions = []
            stop_conditions = self._unique_strings(
                list(role_stop_conditions)
                + [
                    "Stop before writing any forbidden artifact directly.",
                    "Stop if required MCP evidence is unavailable for the assigned scope.",
                    "Stop if the worker scope cannot be completed without changing canonical outputs.",
                ]
            )
            workers.append(
                {
                    "id": worker_id,
                    "goal": goal,
                    "functional_role": functional_role,
                    "required_skills": self._worker_required_skills(
                        task_packet,
                        role_config,
                    ),
                    "required_mcp": list(task_packet.get("required_mcp", [])),
                    "allowed_artifacts": [worker_root],
                    "forbidden_artifacts": forbidden_artifacts,
                    "review_required": True,
                    "stop_conditions": stop_conditions,
                    "worker_root": worker_root,
                    "status": "planned",
                }
            )
```

In the returned worker state dictionary, add these keys after `"topic": topic,`:

```python
            "collaboration_profile": collaboration_profile,
            "controller_role": controller_role,
            "stage_family": stage_family,
            "review_focus": review_focus,
            "config_source": str(worker_config.get("source", "")).strip(),
```

- [ ] **Step 5: Run role-aware and F3 fallback tests**

Run:

```bash
python3 -m unittest \
  tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_stage_default_workers_get_role_specific_goals_and_skills \
  tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_task_run_uses_stage_default_worker_plan_for_f3 \
  -v
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_worker_orchestration_runtime.py
git commit -m "feat: enrich worker plans with role-aware metadata"
```

---

### Task 4: Include Controller Role And Review Focus In Prompts

**Files:**
- Modify: `packages/python-qiongli/src/qiongli/bridges/orchestrator.py`
- Modify: `tests/test_worker_orchestration_runtime.py`

- [ ] **Step 1: Write the prompt metadata test**

Add this test method to `WorkerOrchestrationRuntimeTests`:

```python
    def test_cross_stage_worker_merge_and_review_prompts_include_controller_metadata(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        orchestrator.task_run(
            task_id="F3",
            paper_type="empirical",
            topic="prompt-metadata",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=1,
        )

        prompts = "\n\n".join(call["prompt"] for call in orchestrator.runtime_calls)
        self.assertIn("Controller role: senior-research-director", prompts)
        self.assertIn("Collaboration profile: cross_stage_research_team", prompts)
        self.assertIn("Stage family: F", prompts)
        self.assertIn("Review focus: story spine, claim evidence, cross-section consistency, overclaiming", prompts)
```

- [ ] **Step 2: Run the prompt test and verify it fails**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_cross_stage_worker_merge_and_review_prompts_include_controller_metadata -v
```

Expected: FAIL because prompts do not yet include controller role and review focus lines.

- [ ] **Step 3: Add metadata lines to worker prompt**

In `_build_worker_orchestration_prompt`, change the first part of the returned prompt to:

```python
        return f"""You are a scoped worker running under the task controller.
Complete only the assigned worker packet and return findings to the controller.

Controller role: {worker_state.get("controller_role", "")}
Collaboration profile: {worker_state.get("collaboration_profile", "")}
Stage family: {worker_state.get("stage_family", "")}
Review focus: {worker_state.get("review_focus", "")}

Worker packet (JSON):
{json.dumps(worker_packet, ensure_ascii=False, indent=2)}
```

Keep the rest of the existing prompt body unchanged after `Worker packet (JSON):`.

- [ ] **Step 4: Add metadata lines to merge prompt**

In `_build_worker_merge_prompt`, change the first line of the returned prompt to:

```python
        return f"""Merge worker results for this Qiongli task.

Controller role: {worker_plan.get("controller_role", "")}
Collaboration profile: {worker_plan.get("collaboration_profile", "")}
Stage family: {worker_plan.get("stage_family", "")}
Review focus: {worker_plan.get("review_focus", "")}

Worker plan (JSON):
{json.dumps(worker_plan_view, ensure_ascii=False, indent=2)}
```

Keep the rest of the existing prompt body unchanged after `Worker plan (JSON):`.

- [ ] **Step 5: Add metadata lines to final review prompt**

In `_build_worker_final_review_prompt`, change the first line of the returned prompt to:

```python
        return f"""Final-review the merged worker output.

Controller role: {worker_plan.get("controller_role", "")}
Collaboration profile: {worker_plan.get("collaboration_profile", "")}
Stage family: {worker_plan.get("stage_family", "")}
Review focus: {worker_plan.get("review_focus", "")}

Worker plan (JSON):
{json.dumps(worker_plan_view, ensure_ascii=False, indent=2)}
```

Keep the rest of the existing prompt body unchanged after `Worker plan (JSON):`.

- [ ] **Step 6: Include metadata in compact worker views and summary output**

In `_compact_worker_plan_view`, add these fields after `"topic": worker_plan.get("topic", ""),`:

```python
            "collaboration_profile": worker_plan.get("collaboration_profile", ""),
            "controller_role": worker_plan.get("controller_role", ""),
            "stage_family": worker_plan.get("stage_family", ""),
            "review_focus": worker_plan.get("review_focus", ""),
            "config_source": worker_plan.get("config_source", ""),
```

In `_format_worker_orchestration_section`, add these lines after worker adapter:

```python
            f"- Collaboration profile: {worker_state.get('collaboration_profile', '')}",
            f"- Controller role: {worker_state.get('controller_role', '')}",
            f"- Stage family: {worker_state.get('stage_family', '')}",
            f"- Worker config source: {worker_state.get('config_source', '')}",
```

- [ ] **Step 7: Run the prompt metadata test**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_cross_stage_worker_merge_and_review_prompts_include_controller_metadata -v
```

Expected: PASS.

- [ ] **Step 8: Commit Task 4**

Run:

```bash
git add packages/python-qiongli/src/qiongli/bridges/orchestrator.py tests/test_worker_orchestration_runtime.py
git commit -m "feat: include controller metadata in worker prompts"
```

---

### Task 5: Preserve Legacy B1 And H3 Behavior

**Files:**
- Modify: `tests/test_worker_orchestration_runtime.py`
- Modify: `tests/test_worker_orchestration_contract.py`

- [ ] **Step 1: Add regression tests for legacy config precedence**

Add this method to `WorkerOrchestrationRuntimeTests`:

```python
    def test_legacy_task_worker_configs_take_precedence_over_stage_defaults(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="B1",
            paper_type="systematic-review",
            topic="legacy-b1",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="solo",
            controller="codex",
            primary_agent="codex",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
            max_workers=4,
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("task_config", worker_state["config_source"])
        self.assertEqual("by_search_facet", worker_state["partition_strategy"])
        self.assertEqual(
            ["literature_search_worker", "screening_worker", "extraction_worker"],
            [worker["id"] for worker in worker_state["workers"]],
        )
        self.assertEqual("degrade", worker_state["barrier_rules"]["on_failure"])
```

- [ ] **Step 2: Add contract test that legacy config remains exactly scoped**

Keep `test_worker_orchestration_config_references_valid_contract_values` asserting:

```python
        self.assertEqual({"B1", "H3"}, set(config))
```

If an implementation step changed this assertion, restore it. The new cross-stage file should carry arbitrary-stage defaults; the legacy capability map should remain the explicit override surface for `B1` and `H3`.

- [ ] **Step 3: Run legacy regression tests**

Run:

```bash
python3 -m unittest \
  tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_legacy_task_worker_configs_take_precedence_over_stage_defaults \
  tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_orchestration_config_references_valid_contract_values \
  -v
```

Expected: PASS.

- [ ] **Step 4: Commit Task 5**

Run:

```bash
git add tests/test_worker_orchestration_runtime.py tests/test_worker_orchestration_contract.py
git commit -m "test: preserve legacy worker orchestration overrides"
```

---

### Task 6: Cover Review-Swarm And Code-Stage Use Cases

**Files:**
- Modify: `tests/test_worker_orchestration_runtime.py`

- [ ] **Step 1: Add review-swarm stage test**

Add this method to `WorkerOrchestrationRuntimeTests`:

```python
    def test_review_swarm_stage_default_works_for_g1(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="G1",
            paper_type="empirical",
            topic="reporting-check",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="review-swarm",
            worker_adapter="generic-prompt",
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("review_swarm", worker_state["mode"])
        self.assertEqual("G", worker_state["stage_family"])
        self.assertEqual("by_reporting_gate", worker_state["partition_strategy"])
        self.assertEqual(
            ["reporting_compliance_worker", "tone_consistency_reviewer"],
            [worker["id"] for worker in worker_state["workers"]],
        )
        self.assertEqual("block", worker_state["barrier_rules"]["on_failure"])
```

- [ ] **Step 2: Add code-stage worker test**

Add this method to `WorkerOrchestrationRuntimeTests`:

```python
    def test_code_stage_default_works_for_i5(self) -> None:
        orchestrator = WorkerCaptureOrchestrator()

        result = orchestrator.task_run(
            task_id="I5",
            paper_type="methods",
            topic="causal-did",
            cwd=REPO_ROOT,
            skip_validation=True,
            execution_mode="duo",
            controller="codex",
            primary_agent="codex",
            review_agent="claude",
            worker_mode="delegated-workers",
            worker_adapter="generic-prompt",
        )

        worker_state = result.data["task_packet"]["worker_orchestration"]
        self.assertEqual("I", worker_state["stage_family"])
        self.assertEqual("by_code_research_risk", worker_state["partition_strategy"])
        self.assertEqual(
            ["code_spec_worker", "stats_validation_worker", "reproducibility_reviewer"],
            [worker["id"] for worker in worker_state["workers"]],
        )
        self.assertIn("method fidelity", worker_state["review_focus"])
        self.assertIn("code-specification", worker_state["workers"][0]["required_skills"])
```

- [ ] **Step 3: Run the new stage tests**

Run:

```bash
python3 -m unittest \
  tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_review_swarm_stage_default_works_for_g1 \
  tests.test_worker_orchestration_runtime.WorkerOrchestrationRuntimeTests.test_code_stage_default_works_for_i5 \
  -v
```

Expected: PASS.

- [ ] **Step 4: Commit Task 6**

Run:

```bash
git add tests/test_worker_orchestration_runtime.py
git commit -m "test: cover cross-stage review and code worker defaults"
```

---

### Task 7: Update Documentation

**Files:**
- Create: `docs/advanced/cross-stage-multiagent-collaboration.md`
- Create: `docs/zh/advanced/cross-stage-multiagent-collaboration.md`
- Modify: `docs/advanced/index.md`
- Modify: `docs/zh/advanced/index.md`
- Modify: `docs/guide/multi-agent.md`
- Modify: `docs/advanced/agent-skill-collaboration.md`

- [ ] **Step 1: Add English advanced guide**

Create `docs/advanced/cross-stage-multiagent-collaboration.md`:

```markdown
# Cross-Stage Multiagent Collaboration

Use cross-stage multiagent collaboration when a Qiongli task benefits from role-based decomposition, adversarial review, or independent merge validation.

The controller role is `senior-research-director`. It does not replace Task IDs, skills, MCP evidence, or artifact contracts. It decides worker scope, enforces scoped worker artifacts, merges worker findings, records conflicts and gaps, and sends the merged result to an independent final review.

## Execution Shape

```text
Task ID
-> senior-research-director planning
-> worker_plan
-> scoped workers
-> merge with conflict matrix
-> independent final review
-> canonical artifact update plan
```

## CLI Examples

Brainstorming and framing:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id A1 \
  --paper-type theory \
  --topic platform-governance \
  --cwd . \
  --worker-mode delegated-workers \
  --worker-adapter generic-prompt
```

Literature search and review:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id B1 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd . \
  --worker-mode delegated-workers \
  --worker-adapter auto
```

Manuscript writing:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --execution-mode duo \
  --controller codex \
  --primary codex \
  --reviewer claude \
  --worker-mode delegated-workers
```

Reporting review swarm:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id G1 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --worker-mode review-swarm
```

## Safety Rules

- Worker outputs are candidate outputs until merge and final review pass.
- Workers write only under `RESEARCH/[topic]/runs/<run_id>/workers/<worker_id>/`.
- Canonical outputs remain controller-owned.
- Conflicts must be preserved in the merge report.
- Missing provider coverage must be recorded before review-grade claims are made.
```

- [ ] **Step 2: Add Chinese advanced guide**

Create `docs/zh/advanced/cross-stage-multiagent-collaboration.md`:

```markdown
# 跨阶段 Multi-Agent 协作

当任意 Qiongli 任务需要角色化分工、对抗式审查或独立合并验证时，可以启用跨阶段 multi-agent 协作。

总控角色是 `senior-research-director`。它不替代 Task ID、skill、MCP evidence 或 artifact contract。它负责拆分 worker 范围、限制 worker 写入目录、合并发现、记录冲突与缺口，并把合并结果交给独立终审。

## 执行形态

```text
Task ID
-> senior-research-director planning
-> worker_plan
-> scoped workers
-> merge with conflict matrix
-> independent final review
-> canonical artifact update plan
```

## CLI 示例

头脑风暴与选题 framing:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id A1 \
  --paper-type theory \
  --topic platform-governance \
  --cwd . \
  --worker-mode delegated-workers \
  --worker-adapter generic-prompt
```

文献搜索与综述:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id B1 \
  --paper-type systematic-review \
  --topic ai-in-education \
  --cwd . \
  --worker-mode delegated-workers \
  --worker-adapter auto
```

论文写作:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --execution-mode duo \
  --controller codex \
  --primary codex \
  --reviewer claude \
  --worker-mode delegated-workers
```

报告规范审查:

```bash
python3 -m bridges.orchestrator task-run \
  --task-id G1 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --worker-mode review-swarm
```

## 安全规则

- worker 输出在 merge 和 final review 通过前只是候选输出。
- worker 只能写入 `RESEARCH/[topic]/runs/<run_id>/workers/<worker_id>/`。
- canonical outputs 仍由 controller 拥有。
- merge report 必须保留冲突和缺口。
- provider coverage 不足时，必须记录限制，不能声称 review-grade 覆盖。
```

- [ ] **Step 3: Link guides from indexes**

In `docs/advanced/index.md`, add this item under Topics after Agent + Skill Collaboration:

```markdown
- [Cross-Stage Multiagent Collaboration](/advanced/cross-stage-multiagent-collaboration)
```

In `docs/zh/advanced/index.md`, add this item under Topics after Agent + Skill Collaboration:

```markdown
- [跨阶段 Multi-Agent 协作](/zh/advanced/cross-stage-multiagent-collaboration)
```

- [ ] **Step 4: Update existing multi-agent docs**

In `docs/guide/multi-agent.md`, add this section after "Parallel And Team Runs":

```markdown
## Cross-Stage Worker Plans

`task-run` can use worker orchestration for any Task ID when `--worker-mode` is enabled. Task-specific entries such as `B1` and `H3` still use explicit overrides. Other Task IDs fall back to `cross_stage_research_team`, where `senior-research-director` controls worker scope, merge adjudication, and final review.

```bash
python3 -m bridges.orchestrator task-run \
  --task-id A1 \
  --paper-type theory \
  --topic platform-governance \
  --cwd . \
  --worker-mode delegated-workers
```

Use `--worker-mode review-swarm` when the task is primarily an audit or review gate, such as `G1`, `H3`, or `J4`.
```

In `docs/advanced/agent-skill-collaboration.md`, add this paragraph after the worker-enabled chain:

```markdown
For arbitrary-stage role delegation, use the `cross_stage_research_team` collaboration profile. It maps Task ID stage families (`A` through `K`) to default worker pools and keeps task-specific overrides for high-value workflows such as `B1` and `H3`.
```

- [ ] **Step 5: Run documentation contract test**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract.WorkerOrchestrationContractTests.test_worker_orchestration_docs_describe_contract_and_fallbacks -v
```

Expected: PASS.

- [ ] **Step 6: Commit Task 7**

Run:

```bash
git add docs/advanced/cross-stage-multiagent-collaboration.md docs/zh/advanced/cross-stage-multiagent-collaboration.md docs/advanced/index.md docs/zh/advanced/index.md docs/guide/multi-agent.md docs/advanced/agent-skill-collaboration.md
git commit -m "docs: explain cross-stage multiagent collaboration"
```

---

### Task 8: Run Full Verification

**Files:**
- No file edits.

- [ ] **Step 1: Run focused unit tests**

Run:

```bash
python3 -m unittest tests.test_worker_orchestration_contract tests.test_worker_orchestration_runtime -v
```

Expected: PASS.

- [ ] **Step 2: Run orchestrator workflow regression tests**

Run:

```bash
python3 -m unittest tests.test_orchestrator_workflows -v
```

Expected: PASS.

- [ ] **Step 3: Run strict standard validation**

Run:

```bash
python3 scripts/validate_research_standard.py --strict
```

Expected: command exits 0 and reports no strict validation errors.

- [ ] **Step 4: Run release-readiness subset for changed surfaces**

Run:

```bash
python3 -m unittest \
  tests.test_skill_resource_links \
  tests.test_agent_routing_policy \
  tests.test_worker_orchestration_contract \
  tests.test_worker_orchestration_runtime \
  -v
```

Expected: PASS.

- [ ] **Step 5: Check working tree**

Run:

```bash
git status --short
```

Expected: only intentional files remain modified if commits were not made during execution; otherwise working tree is clean.

---

## Plan Self-Review

- Spec coverage: The plan covers the requested arbitrary-stage multiagent layer, including brainstorming/framing, literature search, data/study design, writing, compliance, submission, code, proofreading, and presentation through stage defaults.
- Backward compatibility: Legacy `B1` and `H3` worker configs remain in `mcp-agent-capability-map.yaml` and keep precedence over stage defaults.
- Artifact discipline: Worker outputs remain run-scoped and forbidden canonical outputs stay controller-owned.
- Runtime scope: No new runtime agent is introduced. Existing `codex`, `claude`, `antigravity`, `generic_prompt`, `codex_subagent`, and `claude_cowork` semantics remain intact.
- Test coverage: The plan adds contract tests, runtime fallback tests, role metadata tests, prompt metadata tests, legacy precedence tests, and focused verification commands.
