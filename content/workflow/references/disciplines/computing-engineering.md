# Computing, AI and engineering

Use for algorithms, machine learning, HCI, systems, simulation, hardware and
experimental engineering. Start from the research contribution and the evidence
needed to assess it, not a preferred programming language or benchmark count.

## Frame and inspect prior work — A/B

- Separate algorithmic/theoretical, empirical, dataset, systems and user-experience
  contributions. Describe the operating conditions and the claim's limits.
- Record paper, code, dataset, model and benchmark versions separately. A repository
  may implement a later variant; a leaderboard entry is not a reproduced result.
- Compare baseline assumptions, tuning budget, hardware, data access and metric
  definitions. An unavailable implementation limits comparison, not permission to
  invent its output or substitute a favorable number from another setting.

## Common methods — C/I

| Work | Design and validation | Failure to guard against |
|---|---|---|
| Supervised ML | Intended prediction use, independent unit, group/time split, train-only preprocessing, tuning separation | Duplicate people/documents/devices across splits; fitting preprocessing to held-out data; tuning on the final test set |
| LLM / generative evaluation | Exact model/version, prompts, decoding, retrieval/tool setup, test provenance, scoring and judge validation | Assuming hidden training data are uncontaminated; judge bias; changing prompts after inspecting test answers |
| Ablation / benchmark | Hypothesis for each component, comparable budget, appropriate baseline, uncertainty source | Best-seed selection, unfair tuning, treating correlated CV folds as independent replicates |
| HCI / user study | Task, participant selection, accessibility, counterbalancing/learning effects, outcome and qualitative method | Generalizing convenience participants to all users; turning preference or task time into broad usability/safety claims |
| Systems / performance | Workload, hardware/software, warmup, measurement window, repeats, throughput/latency distribution | Comparing different workloads; hiding failure rates or tail latency; interpreting simulation as production measurement |
| Numerical / simulation method | Equations, units, boundary/initial conditions, solver/tolerance, convergence and validation target | Code verification mistaken for validation against reality; calibration and validation using the same observations |
| Hardware / experiments | Calibration, uncertainty budget, sampling rate, synchronization, operating conditions, independent repetitions | Pseudoreplication from sensor samples; assumed nominal frequency; drift and unrecorded calibration changes |
| Theory / formal analysis | Complete assumptions, definitions, proof/derivation, complexity model and edge cases | Treating examples or test success as proof; extrapolating beyond the theorem's assumptions |

Choose evaluation metrics from the task: classification, regression, ranking,
generation and survival outcomes need different quantities. Explain whether error
bars represent runs, test items, datasets or another sampling unit. Correlated
folds need a justified comparison, not an automatic paired t-test.

## Common content to organize

In `code/code_specification.md` and `analysis_plan.md`, connect each claim to inputs,
reference behavior, evaluation unit and expected table/figure. Keep split manifests,
data transformations, configs, seeds, environment, compute budget and failures with
executed outputs. Where a published formula is ambiguous, isolate the assumption
and compare a small known case before claiming reproduction.

For AI-assisted labels or extraction, retain model/prompt versions and validate
against an authorized human/reference sample appropriate to the task. An LLM
response is not ground truth. For closed models, distinguish observed controls
from undisclosed training/evaluation details.

## Ethics, writing and delivery — D/F/G/H

Check data/model licenses and participant permissions for the intended use. Report
privacy, subgroup performance and misuse issues when they affect the claim.
Preserve the user's configured toolchain; a profile library list is not an install
instruction. Distinguish a smoke test, reproduced experiment and external validation.
Explain reproducibility limits of hardware, private data and hosted APIs.

The [NeurIPS checklist](https://neurips.cc/public/guides/PaperChecklist) supports
claim, experiment and reproducibility reporting for the applicable conference
edition. [NIST measurement-process guidance](https://www.nist.gov/publications/nistsematech-engineering-statistics-handbook-chapter-2-measurement-process)
supports calibration, stability and uncertainty planning. Neither establishes
that the user's artifact passed an evaluation or is safe for deployment.
