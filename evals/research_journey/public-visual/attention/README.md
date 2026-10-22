# Public attention diagrams, source versions and recovery

October 22, 2026. This follow-up adds computer science to the preceding education
pilot: one public study in two versions, an attention-link diagram and comparison /
ablation tables. The [observation](observation.json) binds original captures,
frozen inputs, source hashes, exact image delivery, reviews and save receipts.
It is coordinator model review, not subject-expert, installed-Plugin, release or
program acceptance. No private manuscript or publication is involved.

## Sources and question

The question is how multi-head attention supports translation quality and what
its visualizations establish about linguistic relations. The sources are the
[2017 proceedings PDF](https://papers.nips.cc/paper_files/paper/2017/file/3f5ee243547dee91fbd053c1c4a845aa-Paper.pdf)
and [arXiv 1706.03762v7](https://arxiv.org/pdf/1706.03762v7). Their original bytes,
11/15-page text extractions and rendered pages stay outside Git. Four page renders
are supplied for extraction: proceedings p.8 and v7 pp.8, 9, 14. Figure 4 is an
attention-link diagram, not a heatmap. Other rendered pages are not counted as
visually inspected merely because their files exist.

Source review checks the following distinctions:

- Proceedings English–French big-model BLEU is 41.0; v7's abstract/Table 2 say
  41.8 while its §6.1 still says 41.0. The discrepancy remains unresolved.
- Table 2 test results and Table 3 development-set ablations are different
  comparisons. Estimated training FLOPs are not elapsed time or energy.
- Head-count ablations adjust per-head dimensions at fixed computation. They do
  not isolate encoder self-attention or establish monotonic gains with head count.
- Figure 4 shows selected layer-5 attention links involving “its”, “Law” and
  “application”; these examples do not establish a causal translation mechanism.
- The PDFs are versions of one study, not independent replications.

## Frozen task and observations

All four processes use Codex 0.162.1 with the user's `gpt-6-astra` / `xhigh`.
The isolated profile enables and explicitly approves only three read tools; strict
app-server config readback verifies that policy before model calls. No model or
installed-profile setting is changed. The runtime binary and its build snapshot
are retained; the model runs use direct Full MCP and frozen canonical guidance.

Extraction produces source-bound Chinese notes. The coordinator reviews and saves
two exact source packets, one note and a coordinator-authored `partial` bilingual
seed through existing preview/academic-approval/filesystem-approval/CAS owners.
The seed is not counted as another model result. At revision 5, baseline,
candidate and balanced follow-up each start a fresh session with only the project
ID, guidance paths and the same continuation request. Each discovers four current
bindings, writes Chinese/English prose in chat and preserves claim IDs and source
conflicts. All three leave project/config bytes unchanged.

Baseline and candidate inputs/criteria are frozen before either recovery call.
Only `paper-read.md` differs between their guidance trees; the guidance-root path
is the only prompt substitution. The balanced follow-up is frozen after observing
the candidate's additional calls, with the same fixture, task, model and criteria.
It is iterative evidence, not a blinded confirmatory sample. Criteria remain
outside the model's permitted input tree and are not included in prompts.

| Attempt | Seconds | Recorded calls / failed | Document reads | Read content bytes | MCP response bytes | Input tokens |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| extraction | 188.3 | 15 / 2 | 0 | 0 | 0 | 283,185 |
| baseline | 201.1 | 14 / 0 | 8 | 87,632 | 97,369 | 472,894 |
| candidate | 281.3 | 30 / 2 | 21 | 44,917 | 68,887 | 757,298 |
| balanced | 270.4 | 17 / 1 | 9 | 87,632 | 97,834 | 655,115 |

Document reads include searches; `Read content bytes` counts decoded `content`
fields and excludes search-hit text. `MCP response bytes` counts each structured
response once using compact UTF-8 JSON, including search results. Recorded calls
exclude nested image calls missing from simplified exec JSON. Input tokens include
repeated/cached context, not unique source size or a price estimate. Command output
bytes and cache-token counts are also retained in the observation.

The first candidate reduces document content but adds searches and round trips.
The follow-up returns to coherent whole-packet reads and reduces calls relative
to that candidate. **Neither candidate demonstrates an improvement over baseline
latency or input tokens.** The baseline has no tool errors; the later runs have
invalid list calls, so timings also include those failures. One case per variant,
a familiar paper, service variation, context truncation and adaptive follow-up
preclude a general performance or model-quality claim.

Source-fidelity review passes within the requested scope for all completed drafts.
Public local rollouts prove actual image delivery by matching decoded returned
image bytes to saved renders: four original pages for extraction/baseline/balanced,
three for candidate. Original answers remain unchanged. The balanced English text
retains Chinese-style citation punctuation; this is not publication-ready language
editing or an independent assessment of prose quality.

## Repairs and retained failures

`paper-read.md` now selects source scope from current notes/summaries and the
question, permits passage selection without mandatory whole-packet paging, and
reuses context only under the same version/revision/hash/scope. It retains source
support, neighboring context, visual limits and fresh-session binding checks.
Observed extra calls led to an explicit compact-packet / dispersed-claim tradeoff:
choose useful windows and weigh returned text against round trips. No fixed call,
byte or coverage quota is introduced.

The candidate submits `limit: 100` twice and the balanced follow-up once; all
refuse, then recover with a valid page size. The schema already specified default
32 / maximum 64. The final tool description now repeats these bounds so the text
help exposes them too. Actual final native inventory differs only in that
35-tool inventory description. The model captures use the preceding binary;
preventing this model error with the new description remains unproven.

Extraction retains two shell failures (read-only heredoc temporary-file creation
and missing `python`), followed by successful text reads. Its project/config
fingerprint changes because the coordinator prepares pending captures during that
turn; extraction itself makes no MCP calls. This run is not counted as proof of
unchanged fixture bytes. Later recovery fixtures are frozen without concurrent
coordinator writes. An initially prepared second packet preview becomes stale
after the first save; it is retained unused and replaced by a current-revision
capture, preserving the existing write sequence.

Three live refusal cases cover stale revision, a non-string JSON selector and a
changed source packet. The changed listing withholds affected read arguments and
keeps other bindings current; all original fixture bytes are restored. A driver
mistakenly carries read-only offset/size controls into metadata search and receives
a JSON-RPC refusal; its original error remains, and a corrected search omits those
controls as existing guidance requires. Metadata hits are locators, not body
support. Missing-image behavior is not tested in a fresh model session here.

## Checks, replay and next increment

The final source passes 57 affected Python tests (including the 14 content/routing
checks rerun after guidance changes), 9 MCP fixture tests, 16 native content tests,
1 CLI embedded-pack test and 7 ledger/index tests: **90 distinct tests**. Repeated
runs are not added to this count. Live negative cases and qualitative review are
separate. Final native receipts are under `/tmp/qiongli-attention-final-native-checks`.
The cache budget passes at about 1.75 GB listed caches / 73.3 GB free; no deletion
is needed. An initial sandbox background build launch produces no receipt; actual
checks come from the later bounded capture. No full release or remote CI is implied.

Replay uses the [parent pilot setup](../tasks.md), existing capture/parser helpers
and the frozen prompt/criteria/project maps named in `observation.json`. Create a
new private artifact root and new capture directories; do not overwrite originals.
Adapt the retained helper paths, preserve the configured model, check effective
read-tool policy and use actual image-return proof. Keep source PDFs, rendered
pages, local rollouts and authentication outside Git; remove temporary auth links.
Saved packet, image and original PDF hashes have different roles.

Local integration updates CLI-405 progress only, not task state, accepted evidence
or publication authority. The existing execution plan records source/pack identity.
Next are multi-paper measurements of source-window selection and Host guidance
truncation, an unavailable-image recovery case, and author/subject-expert review.
