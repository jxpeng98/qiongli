# Connected Chinese prose — November 6, 2026

This observation uses Daniele Fanelli's [2009 PLOS ONE paper](https://doi.org/10.1371/journal.pone.0005738),
*How Many Scientists Fabricate and Falsify Research? A Systematic Review and
Meta-Analysis of Survey Data* (CC BY). The source packet contains the first ten
**consecutive** Discussion paragraphs, 1,281 English words, with no intervening
paragraph or other content omitted. Publisher JATS locators `s4/p[1]` through
`s4/p[10]` are section/paragraph identifiers, not PDF pages. Complete publisher
HTML/XML/PDF are retained; models receive the selected text inline. Supplementary
data, figures and separately cited studies are outside this review.

A first-use search at `53de04a6` finds no author/title/DOI match in canonical
content, research-journey evaluations or the current plan. This establishes local
first use within those paths, not absence from model training data. The task
explains the 2009 paper; it does not estimate misconduct prevalence today.

## Frozen task and execution

Both writers produce one connected four-paragraph Chinese section of 1,200–1,800
non-whitespace Unicode characters, excluding the heading and standalone source
markers. The common task incorporates the user's preference for fluent expression
with accurate terminology and sufficient explanation. It permits omission of
optional examples, statistics and trends; earlier A/B prose is not supplied.

The Stage F and scholarly-voice prompts differ only by the preceding 16-line
Chinese clarification/example (`04af70af` versus `53de04a6`). The source, task
and configured `gpt-6-astra` / `xhigh` are identical. The writers complete in
338.1 and 289.4 seconds; a fresh identity-blinded reviewer completes in 262.7
seconds. These are runtime observations, not a speed comparison or a causal
estimate of the guidance's effect.

All three complete prompts are verified in their rollouts. Inputs and user
configuration remain unchanged, and temporary authentication links are removed.
Policy readback confirms disabled tool network/web search and empty MCP; actual
turns use read-only permissions and make no tool calls. An initial sandbox DNS
failure is resolved by an authorized public-source download. Optional PATH alias
warnings and the isolated policy probe's untrusted-project notice remain recorded;
no global trust setting is changed or requested operation left unfinished.

## Anonymous comparison and judgments

The [anonymous comparison](anonymous-comparison.md) preserves both original
sections. Whole-section labels are randomized once after generation, independently
of the preceding trial. The human response arrives before the coordinator displays
the identity key or automatic judgments. The supplied comparison, source and task
remain bound to exact hashes. The [raw observation](observation.json) now retains
the key, outputs, source excerpts, frozen criteria and full model review.
The first staged whitespace check rejects four Markdown hard-break space pairs
in the comparison. Its repository display trims only those trailing spaces;
the exact packet issued to the human and its original digest remain separately
embedded in the observation and preserved externally. No wording or input changes.

| Original section | Characters | Content dimensions | Paragraph transitions |
|---|---:|---|---|
| A | 1,512 | All six pass | All three pass |
| B | 1,400 | All six pass | All three pass |

The model ties argument, language, terminology and qualifications, returning no
required or optional issues. It explicitly accepts omission of optional examples
when the existing argument remains complete. The coordinator agrees after source
comparison; no score override or mandatory prose repair is needed. This passage
examines interpretation limits, so qualifications about denominators, truthful
reporting and calibration perform substantive work rather than empty caution.

The [human response](human-feedback.json) prefers B's overall effect and language,
while valuing A's handling of details. It supplies no exact preferred spans,
separate dimension scores or editing time. Preserve this scoped preference
separately from the model's ties; neither is a general language-quality verdict.

## Selection and integration

The [combined text](selected.md) keeps B's structure and adds two source-supported
details from A: the distinction between historical reasoning about repeated
misconduct and direct measurement, and an example distinguishing willingness
from reported past behavior. Both additions retain their evidential limits.
Paragraphs 2 and 4 and all citation anchors remain unchanged apart from trailing
whitespace. The selection has 1,506 characters. Exact choices belong to the
coordinator; this combined wording has not received another human or model review.

Existing G4/J2 and scholarly-voice rules already cover these decisions. Preserve
them, the content pack, native code and evaluation helpers. The current execution
plan records the focused integration checks once. Raw local captures remain at
`/tmp/qiongli-zh-connected-20261106-2m1clugm`.

One source and one generation per arm cannot separate guidance effects from
sampling variation. This is Chinese connected-section evidence, not English,
whole-manuscript, expert, installed-Plugin or Antigravity acceptance. The passages
now become regression material; later held-out evaluation needs new material.
