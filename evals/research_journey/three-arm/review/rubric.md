# Common outcome rubric — reviewer-only

These definitions apply equally to A/B/C. Native tooling, file names, claim IDs
and compliance with Qiongli guidance are process observations, never prerequisites
for a common outcome pass. Preserve structural checks separately from semantic
judgments. No weighted overall quality score is defined.

| Dimension | Record | Boundary |
| --- | --- | --- |
| Factual correctness | Erroneous source-dependent atomic claims / all assessable source-dependent claims, with absolute counts and error categories | Include numeric, population, timeframe, comparator and inference errors; missing answer is not zero error |
| Necessary coverage | Correctly expressed predeclared necessary facts/limits / all applicable required facts/limits | Missing necessary content cannot improve the error rate by shortening the answer |
| Citation support | Supported adjacent citations / citations assessed; separately supported claims / all claims requiring a source | Multiple citations or valid file identity alone do not establish support |
| Synthesis and reasoning | Present/partial/absent for each required relation, with answer spans and source rationale | Distinguish necessary premises from optional background; no fixed number of objections |
| Editing meaning | Original errors, newly introduced errors, preserved correct clauses, acceptable style changes and unresolved judgments | A correct unchanged original can meet the task fully; text change percentage is not effort |
| Expression and bilingual agreement | Anonymous reader judgments and examples; separate checks of matched propositions/qualifiers across languages | Naturalness does not compensate for source drift; technical terms can be appropriate |
| Recovery | Current task, source/version, saved original, preference/override and unresolved scope recovered, then used correctly | Finding a file or emitting a tool name is insufficient |
| Human effort | Timed active minutes, necessary repair units/types and coordinator interventions | Unknown stays unknown; separately report setup versus recurring effort |
| Execution and failure cost | Actual wait/calls/retries/token/cache/billing evidence; time and extra work to restore a correct task | Token-price multiplication is not actual billing; retain noncompletion |

Source-dependent atomic claims receive `supported`, `contradicted`, `insufficient`
or `not-assessable` with exact answer/source spans. Unsupported additions count even
without a citation or predefined claim ID. Predeclare the required fact checklist
from sources before viewing outputs; it limits coverage scoring, not error search.
For noncompleted output, report assessable partial claims and missing coverage
alongside the failed completion status. Do not silently exclude it from totals.

Model review may flag candidate issues. A domain reviewer evaluates substantive
source support/methods; an ordinary reader evaluates expression. Preserve who did
each review, whether independent, sources actually inspected and uncertainties.
Calibration's author/coordinator review can validate a synthetic key but cannot
be reported as independent academic evaluation. If a reviewer lacks the required
source or expertise, mark that judgment not assessed.

Every serious error must quote the affected claim and identify its evidence gap.
Hard failures: fabricated sources/numbers; unsupported causal or population/time
strengthening; claiming to read missing evidence; silently accepting stale source
bindings; or writing a project without authority. Keep these separate from minor
style preferences and the total count. Their introduction blocks the affected
candidate's quality-improvement claim until a separately recorded repair is checked.

Review common outcomes before process logs reveal conditions. Anonymization may
remove surrounding product labels but must retain citations, caveats and content
necessary to judge the answer. If that content itself discloses the arm, record
the leak instead of calling the review strictly blind. Keep the identity key,
other reviewers' answers and prior model scores outside the reviewer workspace.
