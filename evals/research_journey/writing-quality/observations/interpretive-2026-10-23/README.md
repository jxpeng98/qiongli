# First-use qualitative writing observation — 2026-10-23

This increment tests unchanged writing guidance on a qualitative paper and
question not used in the preceding local adjustments. It continues the current
plugin-quality plan. First-use refers to this evaluation history, not to the
model's training data. It is not a release or program acceptance decision.

## Source and frozen task

Mitter et al. (2019), *Exploring Farmers’ Climate Change Perceptions and Adaptation
Intentions: Empirical Evidence from Austria*, Environmental Management 63,
804–821, DOI [10.1007/s00267-019-01158-7](https://doi.org/10.1007/s00267-019-01158-7).
The complete 18-page publisher PDF is supplied with native-order extracted text
and original renders of Figure 1 (PDF p.4) and Table 1 (p.7). The article is
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), as stated on p.16;
text extraction and page rendering derive from the unchanged PDF. Supplements,
original interview transcripts and separately cited papers are not supplied.

The question asks how the study explains the divergence between climate concern
and willingness to adapt, and what this explanation contributes to research and
agricultural engagement. It requests connected Chinese and English discussion
sections, respectively 1400–1800 non-whitespace Unicode code points and 900–1100
whitespace-delimited words. Counts exclude headings and standalone source anchors.
The versions must preserve consequential meaning while using natural syntax.
Length, hedge frequency and paragraph count are not quality scores.

Canonical guidance from 997ea931, source bytes, task, seven review dimensions and
six editing cases are frozen before the first writing run. Repository searches
found no earlier use of this paper in the writing-quality observations or shared
writing references. The writer receives the task, copied canonical Qiongli
guidance and raw public sources; it receives neither the review criteria nor
editing cases. A separate fresh run receives the same guidance and the six cases,
without expected answers or the long-form response. This tests G4/J2 decisions
directly rather than attributing an unguided reviewer's capability to Qiongli.

The roles retain the configured gpt-6-astra/xhigh model and Codex 0.162.1. Each
uses a separate read-only profile with effective configuration readback, no MCP,
disabled web search/tool network and no private research access. Temporary auth
links are removed after execution; credentials are not copied into evidence.
The independent reviewer receives anonymous final responses, task, criteria and
original sources, without writer guidance, trace or prior judgments. It assesses
the whole bilingual passage separately from the six editing-case responses.

## Writing results and review-context defect

The unchanged rules produce 1672 Chinese code points and 925 English words,
meeting both frozen ranges. The first independent review passes all seven
dimensions, with no definite language, source-fidelity or reasoning defect. The
argument develops contrasting responses to risk, other farming purposes,
conditional intentions and planning horizons, then derives research questions
and engagement implications. It retains reported action while distinguishing
intention from independently measured effectiveness. Qualifications concerning
sampling, efficacy and temporal order each constrain a different inference;
removing them indiscriminately would lose meaning.

All six editing responses have source-supported content: A and D retain faithful
wording, C restores drift to the supplied original, and B/E/F correct an excessive
denial of qualitative explanation, a theoretical diagram presented as causal
proof, and perceived farm size presented as objective fact. The first reviewer
accepts A/C/D but challenges B/E/F's `repair` labels, requesting a separate
substantive-review status. Its original response is retained in full.

Inspection of the frozen Stage G and tone-normalizer contracts shows that G4
already permits evidence-backed substantive proposals with identified changes.
Stage J and scholarly voice distinguish that task from expression-only editing.
The case task explicitly requests G4 meaning review for B/E/F, and the response
provides the changed claim, source pages and suggested wording in chat. No
manuscript is applied or approved. The first reviewer lacked these role contracts;
its authority judgment therefore requires adjudication rather than automatic
conversion into a writing-rule failure.

The repair is in the existing `review-criteria.md`: authority judgments receive
the frozen task and applicable role contracts, preserve identity blinding, and
assess the described action rather than demand a literal escalation label. The
writing rules and content pack remain unchanged. A fresh, bounded scope review
receives the original six responses, task, public sources and exact frozen G4/J2
contracts. It receives no first-review judgment, writer identity or proposed
verdict. This is an adaptive adjudication of the same cases, not another held-out
writing trial. Historical manifests, first outputs and first judgments stay intact.
The contract-informed reviewer independently retains all six responses: the
source-backed G4 proposals are within scope, and the two J2 cases preserve or
restore meaning. The coordinator concurs after checking the source and contracts.

## Execution and checks

| Run | Seconds | Tool calls / failed calls | Outer truncations / script failures |
| --- | ---: | --- | --- |
| Long-form writer | 620.9 | 14 / 0 | 1 / 0 |
| G4/J2 cases | 162.5 | 17 / 0 | 2 / 0 |
| Initial anonymous review | 299.3 | 19 / 0 | 0 / 0 |
| Contract-informed scope adjudication | 114.0 | 10 / 0 | 0 / 0 |

All four runs finish, preserve their inputs, receive both original images
(verified against image-byte hashes), remove temporary auth links and reap their
process groups. The writer's truncation occurs in a combined guidance read; the
case run truncates initial guidance/catalog delivery and a combined reference
read. Relevant G4/J2 language is visible in that response, but complete guidance
delivery is not established. These warnings remain in the observation, without
being recast as successful full delivery or inner-command failures. Neither
elapsed times nor this single source establish a performance or general quality
improvement.

The seven `tests.test_program_roadmap` checks pass, the generated index is current
for all 249 tasks, and all four review-contract links resolve. The evidence audit
binds unchanged input/output hashes, the original role-contract bytes, actual
model/effort, source images, successful completion and cleanup. Canonical content,
native code, pack and dependencies are unchanged, so their preceding checks are
reused without a rebuild. Local integration follows the existing reviewed-commit,
frozen-source-guard and fast-forward workflow; no push or publication is requested.

## Source-access limitations

Ostrom's 2007 theoretical paper was considered but excluded before task freezing.
Three public PDF endpoints returned HTTP 403; another returned non-PDF content;
a guessed university path failed DNS resolution and was never evidence. The PMC
browser page also presented a browser check. No partial theory text is counted
as an evaluated paper. The successful qualitative source was selected before
model outcomes were available; no writing result was discarded to select it.

The original inputs, outputs, receipts and driver scripts remain outside Git at
`/home/hermes/qiongli-interpretive-20261023-nsww4ol8`. The adjacent
`observation.json` records prompts, source provenance, hashes and adjudication.
This is a bounded local diagnostic: one paper/question and four runs, including
the adaptive scope adjudication, all using the same model. Author/disciplinary-
expert judgment, a theoretical paper, installed Plugin behavior and other Hosts
remain outside this observation.
