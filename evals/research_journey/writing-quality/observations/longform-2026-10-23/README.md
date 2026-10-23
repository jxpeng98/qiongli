# Cross-discipline long-form writing observation — 2026-10-23

This bounded local observation tests sustained argument, Chinese/English prose
and claim calibration on public ecology and social-policy papers. It continues
the existing plugin-quality plan; it is not a release or acceptance decision.

## Inputs and method

- Hallmann et al. (2017), *More than 75 percent decline over 27 years in total
  flying insect biomass in protected areas*, DOI
  [10.1371/journal.pone.0185809](https://doi.org/10.1371/journal.pone.0185809).
  The publisher PDF has 21 pages; the packet includes its complete extracted text
  with page anchors and original page renders for Figures 2 and 5.
- Dwyer et al. (2023), *Unconditional cash transfers reduce homelessness*, DOI
  [10.1073/pnas.2222103120](https://doi.org/10.1073/pnas.2222103120).
  The published nine-page article is obtained from USC HPRI's public deposit;
  original page renders include preregistered Table 2 and exploratory Figure 1.
  This is a closed source packet, not a review of later literature or uninspected
  supplementary appendices.

Each writer receives two independent substantive drafting tasks. Each task asks
for 1100–1500 Chinese non-whitespace code points and 700–900 English whitespace
words, excluding headings and standalone source anchors. The versions must share
the supported answer and consequential qualifications, with natural syntax in
each language. Paragraph counts and hedge counts are not quality scores.

The baseline guidance is frozen from 4d295445. Source bytes, tasks and criteria
are frozen before the first run. Fresh read-only profiles retain the configured
gpt-6-astra/xhigh model and Codex 0.162.1. Writers receive only selected canonical
Qiongli guidance and public files. They do not receive review criteria, probes,
other guidance versions or prior answers. Effective configuration is read back
before execution; no MCP server is configured, and web search and tool network
access are disabled. No research project is opened for mutation. Raw captures
and temporary profiles remain in the isolated local
observation directory, outside Git.

Seven dimensions distinguish request/length, connected argument, explanatory
reasoning, idiomatic language, source fidelity, bilingual equivalence and claim
calibration. Six separately frozen meaning-review probes include faithful direct
prose, excessive-caution denial, unwarranted causal inference, translation drift
and treating experimental messaging as an observed result. Full prompt, hashes,
criteria, receipts and results are recorded in `observation.json` at integration.

## Narrow rule changes

The baseline already distinguishes exploratory housing analyses from preregistered
cognitive/wellbeing tests, reads Figure 1's markers after multiple-test adjustment, and
separates evidence of biomass decline from causal attribution. Its measured
lengths are 1250 Chinese characters / 732 English words for ecology and
1364 / 775 for social policy. These are successful capabilities to retain.

The coordinator's improvement targets are sustained prose and measurement
precision: successive paragraphs repeatedly return to what cannot be inferred;
Chinese wording such as “无家可归暴露” is indirect for a duration measure; and a
comparison of stable-housing days shifts to wording about “sustained stable
housing.” The last is an avoidable ambiguity, not a claim that the baseline
asserted a demonstrated durable benefit. These judgments motivate a small
revision and must be evaluated separately from factual errors.

Source e406ce8c updates the existing Stage F and scholarly-voice owners. It carries
established limits through precise populations/measures, keeps necessary limits
when the inference changes, and distinguishes aggregate days from a durable or
uninterrupted state. The bilingual example makes a duration measure idiomatic
without adding durability. There is no banned phrase, paragraph quota, required
extra review pass or authority to change research meaning during J2. The owner
contract is updated and the supported pack generator produces eaa4e47d, retaining
446 resources and content version 2.5.3.

## Execution evidence

Two completed writer runs produce four disciplinary task outputs, each with two
language versions: eight sections in total. All eight meet their frozen lengths.

| Guidance | Ecology Chinese / English | Social-policy Chinese / English | Seconds | Tool calls / failed calls | Outer truncations / script failures |
| --- | --- | --- | ---: | --- | --- |
| Baseline | 1250 / 732 | 1364 / 775 | 891.7 | 30 / 4 | 0 / 1 |
| Candidate | 1195 / 722 | 1385 / 824 | 954.9 | 21 / 0 | 0 / 0 |

Both writers receive all four supplied original images, verified from returned
image bytes against the PNG hashes. Source/guidance snapshots remain unchanged.
Baseline attempts to render an additional PDF page encounter a shell here-document
temporary-file failure, an unavailable `python` command, missing `fitz` and missing
`pdftoppm`. Its final prose does not claim to have inspected that extra image.
These are retained execution failures; inner-call and outer-script failure counts
overlap and must not be added. The candidate uses the supplied images successfully.
The different reading choices and one run per version do not establish a speedup.

Review preparation creates and freezes the anonymous inputs, then fails while
trying to archive its script onto the same file. No model review has started at
that point. The original driver/log are retained; a same-file guard fixes the
archival step, and fingerprint checks confirm that the existing anonymous inputs
and mapping are unchanged before launching the sole comparison review. Neither
writer is rerun, and the original preparation error is not counted as a second
model review or hidden as a successful model attempt.

## Anonymous review and source issue

After the 598.1-second review completed, the preserved key revealed P as baseline
and Q as candidate. The seven-dimensional assessment gives 27 passes, one local
unknown and no definite failure across the four bilingual task outputs. All
reviewer length counts independently match the coordinator's counts. The reviewer
receives the same four original images, preserves its inputs, and has four
unsuccessful command invocations but no outer truncation or script failure. These
include a here-document temporary-file failure, missing `python` and `fitz`, and
an optional-renderer availability probe that returns nonzero while finding
`python3`. These attempts are retained; they do not invalidate the subsequently
verified counts or supplied-image inspection.

| Dimension | Ecology | Social policy |
| --- | --- | --- |
| Flow | Tied overall; candidate ends more directly, baseline explains ecological relevance more fully | Slight candidate preference for separating the final synthesis from the public-attitude experiment |
| Explanation | Mixed: baseline explains warming as an offsetting covariate; candidate better explains unequal sampling intervals and regional scope | Slight candidate preference for cash/workshop comparability and unadjusted time-specific analyses |
| Fidelity | Mostly tied, with the baseline source ambiguity below | Tied; the two versions retain different useful supported details |
| Language and bilingual meaning | Both pass; candidate's “群落属性” is somewhat abstract, a style preference | Both pass; no universal language-quality gain established |

The reviewer does **not** confirm a general defensive-writing failure: most
qualifications constrain different claims and serve necessary purposes. Both
versions state the supported findings directly. The coordinator's initial concern
about recurring defensive endings therefore remains a selective editing target,
not a reason to delete qualifications automatically. The modest observed benefit
is clearer organization and explanation in one task.

The local unknown concerns baseline's “下降速率却相近 / showed similar rates of
decline.” Hallmann p.12 does describe similar annual rates, but the S5 description
printed on p.17 reports 7.5% (6.6–8.4) and 5.2% (4.8–5.5), from independently
fitted models. This is a source-level tension, not established misquotation or a
resolved statistical contradiction. The supplementary S5 image/model data were
not inspected. “两类均出现下降 / both groups declined” is a supported narrowing,
already used by candidate. No new causal explanation or equivalence judgment is
invented; the original baseline stays intact for audit.

All six meaning-review probes receive the source-supported handling: accept A/F
unchanged; identify and locally repair B's excessive-caution denial, C's causal
protection claim, D's exploratory/population drift and E's conflation of stimulus,
spending result and housing mechanism. These are intentionally constructed probes,
not four failures generated by either writer. Faithful direct language does not
trigger added caveats or another stylistic rewrite.

## Validation and limits

The canonical Skill validator and 38 distinct affected checks pass: 14 Python
routing/saved-document tests, 16 native content tests, 1 CLI embedded-pack test
and 7 program-ledger/index tests. Native commands complete within their bounds,
are reaped and leave no process group. Frozen source/guidance/review fingerprints,
actual returned-image hashes and removal of all three temporary auth links are
verified. The cache check passes at about 1.75 GB listed caches / 72.1 GB free;
no deletion is needed. Two metadata-assembly quoting errors are fixed and the
final driver is syntax-checked.
A follow-up audit catches a missing program-test field in the earlier report; the
final rebuild verifies it. No model process starts and no evaluation input or answer
changes during these repairs. Failed and corrected driver versions are archived.

The observation directory is `/home/hermes/qiongli-longform-20261023-01aez0w_`.
It retains original answers, review, raw captures, frozen inputs, sources,
preparation logs, drivers and copied native receipts outside Git. `observation.json`
contains the reproducible task, source URLs/digests, criteria, probes, hashes,
verdicts, cleanup receipts and pack identity, without copying raw papers or
model reasoning into the repository.

This is one diagnostic before/after pair and a same-model anonymous review. The
candidate responds to phrasing seen in this source packet; it is not a held-out
generalization test, subject-expert judgment, installed-Plugin/cross-Host
qualification or project-recovery check. Next use unseen questions/sources,
including qualitative or theoretical arguments, and author/disciplinary-expert
review. Initial discovery and optional PDF-renderer availability remain separate
work. Local integration preserves program states and accepted evidence, with no
push or publication.
