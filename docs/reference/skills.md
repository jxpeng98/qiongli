# Skills guide

> Auto-generated from `content/skills/registry.yaml` by `python3 scripts/generate_skill_docs.py`.
> Edit the registry or the generator, then regenerate this page.

Describe your research task in the Host, or select a visible Qiongli Skill.
In Codex, use `$qiongli` or a shortcut such as `$qiongli-paper-read`.
The shared guidance helps the model choose the relevant work; it does not require
every stage to run. Task IDs identify research activities, not native CLI commands.

The tables below list the internal research Skills. They are not all separate
Host entries. Supplemental cards and subject profiles support these Skills;
they do not create additional agents or MCP tools.

The descriptions and use cases below come from the same registry in both languages.

## Stage Overview

| Stage | Focus | Skill count | Typical user intent |
|---|---|---:|---|
| `A_framing` | topic framing, questions, theory, gap, venue | 6 | "What exactly is my contribution?" |
| `B_literature` | search, screen, extract, cite, map | 9 | "What does the literature say, and how do I build a corpus?" |
| `C_design` | design, variables, robustness, datasets | 9 | "How should this study be designed and operationalized?" |
| `D_ethics` | IRB, privacy, governance | 3 | "What ethics and data-protection materials do I need?" |
| `E_synthesis` | evidence synthesis, quality, bias | 5 | "How do I combine and rate evidence?" |
| `F_writing` | manuscript building, tables, figures, results writing | 8 | "How do I turn analysis into publishable text?" |
| `G_compliance` | reporting checklists, tone, PRISMA | 3 | "Is this compliant and submission-ready?" |
| `J_proofread` | AI detection, humanization, similarity, final polish | 4 | "How do I de-AI and finalize the manuscript?" |
| `H_submission` | submission package, rebuttal, review simulation | 8 | "How do I package, defend, and stress-test the paper?" |
| `I_code` | academic code, stats, reproducibility | 10 | "How do I implement and verify research code?" |
| `K_presentation` | academic talks, slide planning, Slidev, Beamer | 4 | "How do I turn the paper into a defensible talk?" |
| `L_coursework` | assignment briefs, rubrics, learning outcomes, coursework revision | 4 | "How do I satisfy this assignment honestly and clearly?" |
| `M_dissertation` | dissertation planning, chapter maps, supervisor feedback, defense prep | 4 | "How do I plan, draft, revise, and defend this dissertation?" |
| `Z_cross_cutting` | metadata, model collaboration, self-critique | 5 | "How do I improve quality across stages?" |

## Canonical Skills By Stage

### A. Framing

Use Stage A when you are still defining the research question, contribution, theory anchor, or venue positioning.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `question-refiner` | Question Refiner | Use when you need to transform vague topics into structured rqs via pico/peo + finer evaluation. | `RQSet` |
| `contribution-crafter` | Contribution Crafter | When framing the pitch for a manuscript introduction | `ContributionStatement` |
| `hypothesis-generator` | Hypothesis Generator | Use when you need to translate rqs into testable hypotheses with mechanisms and boundary conditions. | `HypothesisSet` |
| `theory-mapper` | Theory Mapper | Use when you need to map theoretical concepts, relationships, and frameworks with mermaid diagrams. | `TheoreticalFramework` |
| `gap-analyzer` | Gap Analyzer | Use when you need to identify and categorize research gaps using 5-type taxonomy with finer prioritization. | `GapAnalysis` |
| `venue-analyzer` | Venue Analyzer | Explore venues before a draft or adapt a manuscript to a chosen journal or conference. | `VenueAnalysis` |

### B. Literature

Use Stage B when you are building or maintaining the literature base for a topic, especially systematic or reproducible reviews.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `academic-searcher` | Academic Searcher | Use when a B1 search needs reproducible provider execution, search_diagnostics.md, dedup logs, or review-grade coverage checks. | `SearchQueryPlan`, `SearchResults`, `SearchLog`, `DedupLog`, `SearchDiagnostics` |
| `paper-screener` | Paper Screener | Use when candidate records need include/exclude decisions, diagnostic flags preserved, or PRISMA flow counts. | `ScreeningDecisionLog`, `FullTextScreening`, `PRISMAFlowData` |
| `paper-extractor` | Paper Extractor | Use when included papers need notes or extraction_table.md without overclaiming metadata-only or abstract-only evidence. | `ExtractionTable`, `PaperNotes` |
| `citation-snowballer` | Citation Snowballer | Use when B3 needs forward/backward citation expansion, known-item follow-up, or coverage repair after search diagnostics. | `SnowballLog`, `SearchResults`, `DedupLog` |
| `fulltext-fetcher` | Full-text Fetcher | Use when reports need retrieval_manifest.csv, controlled full-text statuses, or Zotero/OA resolver handoff. | `FullTextStatus`, `RetrievalManifest` |
| `citation-formatter` | Citation Formatter | Use when B5 needs citekey cleanup, DOI normalization, duplicate handling, or export-ready bibliography.bib. | `Bibliography` |
| `concept-extractor` | Concept Extractor | Use after Stage A when B1 search terms, controlled vocabulary, excluded ambiguous terms, or seed recall are unsettled. | `ConceptMap` |
| `literature-mapper` | Literature Mapper | Use when extracted papers need a defensible non-chronological map for related work or novelty positioning. | `LiteratureMap` |
| `reference-manager-bridge` | Reference Manager Bridge | Use when B5 needs Zotero local sync, import-file fallback, RIS/CSL-JSON export, or bibliography exchange. | `Bibliography`, `RISExport`, `CSLJSONExport`, `ZoteroImportReport` |

### C. Design

Use Stage C when the question is already clear and the next problem is design validity, data feasibility, and operationalization.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `study-designer` | Study Designer | After `/find-gap` and `/build-framework`, when you want to run an empirical, qualitative, or mixed-methods study. | `DesignSpec`, `AnalysisPlan`, `DataManagementPlan`, `Instruments`, `Preregistration` |
| `rival-hypothesis-designer` | Rival Hypothesis Designer | After hypotheses (A1_5) and study design (C1) are drafted | `RivalHypotheses` |
| `robustness-planner` | Robustness Planner | After analysis plan (C3) is drafted | `RobustnessPlan` |
| `dataset-finder` | Dataset Finder | After study design specifies what data is needed | `DatasetPlan` |
| `variable-constructor` | Variable Constructor | After study design is finalized | `VariableSpec` |
| `data-dictionary-builder` | Data Dictionary Builder | After study design is finalized and instruments are selected | `DataDictionary` |
| `data-management-plan` | Data Management Plan | Use when you need to generate fair-compliant data management plans specifying storage, backup, retention, sharing, and archival. | `DataManagementPlan` |
| `prereg-writer` | Pre-registration Writer | After study design and analysis plan are finalized | `Preregistration` |
| `variable-operationalizer` | Variable Operationalizer | After theoretical framework is established | `OperationalizationMap` |

### D. Ethics

Use Stage D when the study touches human participants, sensitive data, governance, or data-release constraints.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `ethics-irb-helper` | Ethics & IRB Helper | Before data collection involving human participants or sensitive data | `EthicsPackage` |
| `statement-generator` | Statement Generator | When finalizing a manuscript for submission and ensuring compliance with journal statement requirements (e.g., PLOS, Nature, Elsevier). | `Manuscript` |
| `deidentification-planner` | Deidentification Planner | When handling any dataset containing identifiable information | `DeidentificationPlan` |

### E. Synthesis

Use Stage E when the evidence base already exists and the task is to combine, rate, or stress-test that evidence.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `effect-size-calculator` | Effect Size Calculator | When preparing data for a quantitative meta-analysis | `EffectSizeTable`, `AnalysisCode` |
| `evidence-synthesizer` | Evidence Synthesizer | Use after you have: | `EvidenceTable`, `SynthesisMatrix` |
| `quality-assessor` | Quality Assessor | Use when you need to assess risk of bias and certainty via rob 2, robins-i, grade. | `QualityTable`, `GRADESummary` |
| `publication-bias-checker` | Publication Bias Checker | After completing meta-analysis (E3) with ≥5 studies | `PublicationBiasReport` |
| `qualitative-coding` | Qualitative Coding | Use after raw qualitative data has been collected or extracted, but before the final synthesis narrative is drafted. Ideal for Grounded Theory, Thematic Analysis, or Interpretative Phenomenological Analysis (IPA). | `DataDictionary`, `ThematicCodebook` |

### F. Writing

Use Stage F when the main question is turning evidence and analysis into sections, tables, figures, and readable claims.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `manuscript-architect` | Manuscript Architect | You want to draft a paper from an existing `RESEARCH/[topic]/` project folder (empirical study or systematic review). | `ManuscriptOutline`, `Manuscript`, `ClaimGraph`, `FiguresTablesPlan` |
| `proposal-writer` | Proposal Writer | When preparing a research proposal, opening report, prospectus, study plan, or 开题报告 before results-focused manuscript drafting. | `ResearchProposal` |
| `analysis-interpreter` | Analysis Interpreter | After primary analysis is complete | `ResultInterpretation` |
| `effect-size-interpreter` | Effect Size Interpreter | After primary analysis is complete | `EffectInterpretation` |
| `table-generator` | Table Generator | After primary analysis is complete | `FormattedTables` |
| `figure-specifier` | Figure Specifier | After primary analysis is complete | `FigureSpecs` |
| `meta-optimizer` | Meta Optimizer | After manuscript draft is complete (F2/F3/F4) | `MetaOptimization` |
| `discussion-writer` | Discussion Writer | Use after the Results section has been drafted and the primary findings are finalized. Provide the core contribution statement and literature framing to ensure alignment. | `DiscussionDraft`, `StorySpine` |

### G. Compliance

Use Stage G when the paper exists and now needs formal checklist coverage, tone cleanup, or reporting verification.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `prisma-checker` | PRISMA Checker | Use when you need to verify prisma 2020 flow diagram and all 27 checklist items. | `PRISMAChecklist` |
| `reporting-checker` | Reporting Checker | Before submission (final quality assurance pass) | `ReportingChecklist` |
| `tone-normalizer` | Tone Normalizer | After manuscript draft is complete (but before final submission) | `ToneNormalization` |

### J. Proofread

Use Stage J when the draft is substantively complete and needs AI-fingerprint review, human-voice rewriting, similarity screening, or final proofreading before submission.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `ai-fingerprint-scanner` | AI Fingerprint Scanner | Use for a requested language-pattern diagnosis; grammar-only correction can enter final-proofreader directly. | `AIDetectionReport` |
| `human-voice-rewriter` | Human-Voice Rewriter | Use for requested expression or voice revision; reuse supplied text and any existing findings. | `HumanizedManuscript` |
| `similarity-checker` | Similarity Checker | Use for requested source-overlap or attribution checks; prior rewriting is not an automatic prerequisite. | `SimilarityReport` |
| `final-proofreader` | Final Proofreader | Use for requested scholarly grammar/consistency corrections or a formal final proofreading pass. | `ProofreadChecklist` |

### H. Submission

Use Stage H when the manuscript is near submission or already under review.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `submission-packager` | Submission Packager | You have a near-final manuscript draft ready for submission | `SubmissionPackage` |
| `rebuttal-assistant` | Rebuttal Assistant | After receiving a Revise & Resubmit (R&R) decision | `ResponseToReviewers`, `ResponseLetter` |
| `peer-review-simulation` | Peer Review Simulation | Use for a requested referee-style manuscript review or formal H3 task; a focused check needs only relevant lenses. | `PeerReviewSimulation` |
| `fatal-flaw-detector` | Fatal Flaw Detector | Before final submission (last quality gate) | `FatalFlawAnalysis` |
| `journal-fit-recommender` | Journal Fit Recommender | Use when an existing manuscript needs manuscript-first reverse journal fit before submission. | `JournalFitRecommendation` |
| `reviewer-empathy-checker` | Reviewer Empathy Checker | After drafting the response letter (from `rebuttal-assistant` H2) but BEFORE submitting | `EmpathyCheck` |
| `credit-taxonomy-helper` | CRediT Taxonomy Helper | During submission packaging (H1) | `CRediTStatement` |
| `limitation-auditor` | Limitation Auditor | Use during the late drafting or pre-submission phase when the Methods and Discussion sections are complete, to ensure the manuscript demonstrates scholarly humility and self-awareness before peer reviewers point out the flaws. | `LimitationSection`, `MitigationStrategy` |

### I. Code

Use Stage I for academic code, data workflows, statistical execution, and reproducibility. This lane is stricter than general engineering prompts.

The core strict sequence is:

1. `code-specification`
2. `code-planning`
3. `code-execution`
4. `code-review`
5. `reproducibility-auditor`

That sequence is what `code-build --focus full` is designed to reinforce.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `code-builder` | Code Builder | Use when you need to convert academic methods into executable code with domain-profile library selection. | `AnalysisCode` |
| `data-cleaning-planner` | Data Cleaning Planner | After dataset is obtained, before any analysis | `CleaningPlan` |
| `data-merge-planner` | Data Merge Planner | When analysis requires combining multiple data sources | `MergePlan` |
| `code-specification` | Code Specification | Use when you need to generate strict opsx-style constraint sets before coding. | `CodeSpec` |
| `code-planning` | Code Planning | Use when you need to transform specs into parallelizable, zero-decision execution plans. | `CodePlan` |
| `code-execution` | Code Execution | Use when you need to execute code plans with cprofile performance profiling and optimization. | `PerformanceProfile` |
| `code-review` | Code Review | Use when you need to secondary model reviews code logic, security, and statistical validity. | `CodeReview` |
| `reproducibility-auditor` | Reproducibility Auditor | Use when you need to verify seeds, containerization, and fail-graceful contingencies. | `ReproducibilityReport` |
| `release-packager` | Release Packager | When preparing for manuscript submission (data/code availability statement) | `ReleasePackage` |
| `stats-engine` | Stats Engine | You need to execute modeling/testing for synthesis (`E3/E3_5`) or empirical results (`F` stage) with a clear report of assumptions and uncertainty. | `StatsReport` |

### K. Presentation

Use Stage K when the paper already exists and the next task is to turn it into a talk, seminar deck, or conference presentation.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `presentation-planner` | Presentation Planner | After manuscript is substantially complete (or results are available) | `PresentationPlan` |
| `slide-architect` | Slide Architect | After `presentation-planner` has produced a slide blueprint | `SlideDeckSpec` |
| `slidev-scholarly-builder` | Slidev Scholarly Builder | When Slidev + scholarly is chosen as the output backend | `SlidevDeck`, `BibTeXFile` |
| `beamer-builder` | Beamer Builder | When LaTeX Beamer is the chosen backend | `BeamerDeck`, `BibTeXFile` |

### L. Coursework

Use Stage L when the task is coursework, an assignment brief, marking rubric, learning outcomes, a case analysis, reflective writing, portfolio, or capstone coursework.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `assignment-brief-analyzer` | Assignment Brief Analyzer | Use when a coursework, capstone, or dissertation brief needs parsing before planning or drafting. | `AssignmentBrief`, `AcademicIntegrityNotes` |
| `rubric-mapper` | Rubric Mapper | Use when coursework must be planned or revised against a rubric or learning outcomes. | `RubricMap`, `LearningOutcomeMap` |
| `coursework-architect` | Coursework Architect | Use before drafting coursework essays, reports, cases, reflections, portfolios, or capstone components. | `CourseworkOutline`, `CourseworkClaimEvidencePlan` |
| `coursework-reviser` | Coursework Reviser | Use when a coursework draft needs rubric-facing revision or final submission readiness checks. | `CourseworkRevisionPlan`, `CourseworkSubmissionChecklist` |

### M. Dissertation

Use Stage M when the task is a dissertation, thesis, capstone, major project, supervisor feedback integration, or viva/defense preparation.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `dissertation-planner` | Dissertation Planner | Use when starting or restructuring a dissertation, thesis, capstone, or major project. | `DissertationPlan` |
| `chapter-architect` | Chapter Architect | Use when a dissertation needs chapter architecture, chapter status, or word-count allocation. | `DissertationChapterMap` |
| `supervisor-feedback-integrator` | Supervisor Feedback Integrator | Use when supervisor or committee feedback must become a traceable dissertation revision plan. | `DissertationFeedbackLog` |
| `dissertation-readiness-checker` | Dissertation Readiness Checker | Use before dissertation submission, viva, or defense preparation. | `DissertationReadinessReport` |

### Z. Cross-Cutting

Use Stage Z when the need cuts across stages rather than belonging to one paper section.

| Skill | Display Name | When to use | Produces |
|---|---|---|---|
| `metadata-enricher` | Metadata Enricher | Use when you need to normalize and enrich doi, venue, year, author metadata across artifacts. | `Bibliography` |
| `academic-context-maintainer` | Academic Context Maintainer | Use for research-state continuity, detailed stage consolidation, progress tracking, or a file-by-file review for the user's own manual cleanup. | `ResearchStateSnapshot`, `ResearchDecisionLog`, `StageSummary` |
| `boundary-interviewer` | Boundary Interviewer | Use during brainstorms, Stage A framing, gap finding, or before high-risk study design, writing, submission, research code, and handoff work when research scope, claim strength, evidence threshold, validity risk, generalizability, or reviewer expectations remain unclear. | `AcademicIdeaFunnel`, `BoundaryReview` |
| `model-collaborator` | Model Collaborator | Use when literature screening, peer review simulation, rebuttal drafting, qualitative coding, or code/statistics validation benefits from independent multi-model passes before synthesis. | `CollaborationTrace` |
| `self-critique` | Self-Critique | Use for a requested research review or a required quality check; repeat only for affected fixes or the formal review contract. | `CritiqueLog` |

## Subject guidance

Describe the field, method and protocol in your request. The Host can read relevant
bundled profiles; the 1.x `--domain` switch is not a native 2.x CLI option.
Profiles guide the work, but do not establish evidence or current journal policy.

The bundled `references/discipline-guidance.md` selects focused guides for
economics/finance/accounting, business/society/policy, education/psychology, health/biomedicine,
computing/engineering, environment/spatial research and humanities/language/law.
They organize common questions, methods, evidence and delivery checks across A–M,
and load only when relevant. These guides do not add runtime subject IDs or Host entries.

Available profiles:

- `accounting`
- `biomedical`
- `business-management`
- `cs-ai`
- `ecology-environmental`
- `economics`
- `education`
- `epidemiology`
- `finance`
- `geoeconomics`
- `political-economy`
- `political-science`
- `psychology`

## Continue

Use [task examples](/guide/task-recipes), [the CLI guide](/guide/cli-2x) or
[collaboration guidance](/advanced/agent-skill-collaboration).
For changes to the framework, see [Extend Qiongli](/advanced/extend-qiongli).
