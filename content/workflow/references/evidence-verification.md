# Source-bound evidence verification

Use while drafting or revising manuscript text, checking whether its substantive
claims follow from sources, or incorporating new evidence. Begin with the actual
text being delivered, including claims that have not reached a map or ledger.
This is a bounded review of the requested text, not a new literature search.
For independent execution, use `skills/Z_cross_cutting/model-collaborator.md` and
an actual authorized Host reviewer. Changing roles within one conversation is
self-review. A reviewer may not see its dispatch receipt: leave unavailable
execution identity or independence unverified for the coordinator to reconcile,
rather than infer either independent execution or same-conversation self-review.
Text comparison performed by a model remains model review, not human review.

## Start from the manuscript

Identify the exact draft/version and selected section, paragraphs or sentences.
Read that unit in order, including relevant captions and footnotes/endnotes.
Split independently checkable assertions: one sentence can contain a background
fact, a reported result and an added inference with different evidence needs.
Inspect definitions, method descriptions, numbers, comparisons, generalizations,
causal transitions and claims of novelty or absence; a smooth transition can
introduce an unsupported claim. Citations already present do not establish support.
Do not limit this inventory to abstract/introduction/discussion or to existing IDs.

Use `references/citation-risk-policy.md` to distinguish source-dependent claims,
the project's own results, attributed interpretations and the writer's synthesis
or inference. A signpost or stated research aim needs no invented external source.
Original reasoning needs explicit premises and evidence, not a citation falsely
attributing the conclusion to another author. Each independently supported part
must be traceable even when prose combines it with other parts.

For project work, compare these assertions with
`manuscript/claims_evidence_map.md` and `evidence/claim-evidence-ledger.csv`.
Reuse an ID only for the same atomic claim and qualifiers. A note-local `C1` from
one paper is not a project-wide claim ID. If an existing ID names incompatible
claims, preserve both records and report the conflict before merging. Propose a
new stable ID for an unmapped substantive claim; do not silently omit it from review.
Keep the actual manuscript location and a short exact text span in the existing
map/coverage record, bound to the draft identity in the review packet. Paragraph
numbers alone can drift after edits; reconcile locations against the final text.
For a direct chat paragraph, use the same check without requiring project files
or exposing an internal claim table the user did not request.

## Follow each claim to its evidence

1. Inspect every matching ledger row and manuscript-map entry, retaining claim
   text/type, citekey or analysis identity, source location, artifact, limitations
   and recorded status. Inspect explicit gaps and conflicting evidence too.
   A Graph `supports` edge is a recorded relationship, not a fresh verdict or a
   complete inventory; missing edges cannot hide `needs_evidence` rows. Graph
   query/source opens registered records, not arbitrary manuscript prose or PDFs.
   Bounded/truncated artifact reads require further authorized inspection before
   claiming all rows were reviewed.
2. Follow the existing artifact/note references and exact citekeys. Keep each
   source/version separate, including different passages from the same paper.
   A reading-note paraphrase is a route to its original evidence, not a substitute
   for inspecting it. For one's own results, read the bound output/table and
   relevant procedure; never force those claims to cite an unrelated publication.
3. Recover an explicitly bound saved packet through `workflows/paper-read.md`.
   Use current receipt bindings or authorized exact file evidence; retain the
   expected project revision, packet path/hash and observed JSON pointer/range.
   When no exact location is recorded, search a phrase from the source in those
   selected saved materials, inspect each candidate and use its `readArguments`.
   A translated manuscript sentence may have no literal hit; inspect its cited
   source rather than fabricate an original quotation. Preserve the original
   document version and page/section/table independently from JSON offsets.
4. Compare the actual passage and necessary neighboring material with every part
   of the claim. Read table headings/notes, definitions and design limitations
   when they affect meaning. Separate source results, source-author interpretation,
   cross-source synthesis and the manuscript's inference. Name the supported
   wording and why the evidence supports it; a matching topic or citation is
   insufficient. Assess contrary evidence separately instead of voting by count.
5. Return the coverage judgments below with exact manuscript and source locations.
   Ambiguous citekeys/versions, missing or changed files, absent supplements and
   uninspected context stay unresolved. Never choose the newest packet, replace
   an expected hash or turn `no match` into proof of absence. Preserve prior review
   evidence and state what would resolve the particular gap.

This traversal is performed by the active Host using existing reads and records;
it is not a native semantic claim extractor or an automatic scholarly verdict.
Older Hosts lacking saved-passage tools use authorized source reads and disclose
the missing capability. No new search, project write or approval is implied.

## Select the review and its inputs

Agree the consequential claims or requested passage from the task; do not require
a second agent for every sentence. Include unmarked substantive claims in that
passage, not just entries already present in a ledger. Retain claim/decision IDs,
citekeys, exact candidate identity and current source revision/digests. Give the
reviewer the relevant original passages, tables/notes and source access limits;
an author's summary or another review cannot substitute for these materials.
For saved material, use the capability-gated binding, passage-read and literal
search route in `workflows/paper-read.md` to recover inspectable source locations.
A search hit is a candidate for this review; retain its packet/version binding,
read the needed context and distinguish metadata from original body evidence.

Choose the smallest useful assignment:

| Uncertainty | Work to request | Required return |
|---|---|---|
| Does this claim follow from the available source? | Evidence verification against original material | Claim and source locations, judgment, reason and smallest warranted correction |
| Can this design or analysis answer the question? | Methods review using the relevant C/E/I guidance | Estimand/design/assumption mismatch and evidence or diagnostic needed to resolve it |
| Might an omitted source change the conclusion? | Bounded literature search with available tools | Actual queries/results, candidate identities, relevance and access limits |

These are assignments for existing capabilities, not new Host types or mandatory
parallel roles. A source-only reviewer cannot certify search completeness, rerun
unavailable data or establish professional expertise from an Agent's name.

## Check support, not just citation presence

- Verify paper/version and the specific passage actually read. Distinguish a
  candidate URL, abstract, body excerpt, complete document and inspected figure.
  A digest binds bytes; it does not establish that those bytes support a claim.
- Compare each material part of a claim: population/setting, design, comparison,
  outcome, direction, magnitude, uncertainty and time point where relevant.
  Separate reported findings, the authors' interpretation and the draft's added
  inference. Qualify only the unsupported part; do not discard supported evidence.
  Preserve that uncertainty in the review itself as well as in proposed revisions.
- For numbers, check units, denominators, assignment versus analysis units,
  repeated observations and study/report overlap. Preserve table headings,
  footnotes and model/adjustment labels. Recompute a derived value only when its
  required inputs are available; distinguish reported from independently computed.
- For qualitative evidence, retain speaker/context and distinguish quotation,
  source-author theme and reviewer interpretation. Published quotations are not
  the complete transcript corpus; they cannot establish prevalence or saturation.
- Read the surrounding methods, limitations and contrary results when they can
  change the judgment. If a needed table, image, supplement or text is unavailable,
  name the affected claim and missing check. Do not infer its contents from a
  caption, search snippet or the fact that the document downloaded successfully.

## Return and reconcile

Use the coverage table in `templates/agent-review-packet.md`. For each reviewed
claim, distinguish supported within stated limits, contradicted, insufficient
evidence, and not checked. These are review judgments, not new ledger statuses
or automatic project approval. State the inspected scope and unreviewed remainder;
“no findings” alone does not establish coverage. Unsupported added claims need
findings even if they have no existing claim ID; propose an ID without renumbering
the existing claims. No fixed issue quota is required.

Return source-bound findings or a separate candidate, leaving canonical writes
to one coordinator. The coordinator checks the cited passages, resolves conflicts
by evidence and uses the existing ledger statuses and preview/approval/CAS owner.
Different wording or reviewer agreement is not additional source evidence.

## Re-review after a source change

Start from the observed change and the requested project scope. Preserve the old
source, source binding, candidate and review before preparing a new candidate.
Record the old and new version/path/digest, inspected locations and access limits
in the existing note or review packet. A date, filename or changed digest alone
does not establish a correction, supersession or changed academic conclusion.

| Observed change | What to check before reuse |
|---|---|
| Correction to a result, definition, design or appraisal | Inspect the correction's identity and scope, original context and revised passage. Recheck the dependent extraction, synthesis and wording, including derived quantities and interpretations |
| Newly accessible body section or supplement | Compare the newly inspected content with the earlier access-limited claim. New access need not change the source's result, but may resolve a gap or reveal a limitation |
| Different bytes with apparently unchanged content | Inspect relevant text, values, metadata and locators. A new binding still needs verification; semantic conclusions can survive an explained comparison, never an automatic hash replacement |
| Missing or unreadable material, failed retrieval | Retain the previous evidence and failed attempt. Distinguish unavailable fresh verification from actual contradiction; name the dependent claim and specific recovery needed |
| Another report or version of an existing study | Reconcile report/cohort and version identity before treating it as new evidence. Preserve ambiguous alternatives; neither a newer date nor an extra report establishes independent replication |

Trace the impact through existing records and actual text:

1. Follow the changed source locations to note claims, reading-matrix/summary
   claims, synthesis rows and decisions, and manuscript-map/ledger entries.
   Carry each contributing source separately. A synthesis claim has its own ID
   and explicit contributing claim IDs/anchors; it must not impersonate one
   paper's finding or inherit support solely from that paper's citation.
2. Read the affected draft unit as well as its records. Look for paraphrases and
   reused estimates in background, LR, results, discussion, abstract, captions,
   recommendations and notes as applicable. Include newly found, unmapped uses
   and indirect inferences. ID/phrase search finds candidates, not complete
   semantic coverage; disclose sections or dependencies that were not inspected.
3. Compare each dependent claim with old and new evidence. State which wording
   changes, which survives and why, which remains unresolved, and what further
   check would resolve it. Preserve unchanged source-supported clauses. Changing
   a premise does not automatically negate every conclusion that cites the paper;
   absence of a declared dependency does not prove independence either.
4. Prepare the revised summary/synthesis and actual affected prose together when
   in scope, preserving argument, explanation, citations and note links. A list
   of stale IDs alone is not a completed writing revision. If the proposition or
   qualifiers change, retain the old claim and propose a linked successor ID;
   never reuse an ID for an incompatible claim. Record continuity for unchanged
   claims and the exact new manuscript spans. Use existing ledger statuses and
   map/review fields, not a new invalidation vocabulary or evidence database.
5. Recheck source support and then language/logic for the revised text. Retain
   the earlier review as evidence about its original bytes, with the new review
   explicitly bound to the new candidate and inputs. Record old-to-new rationale,
   affected claims, unchecked remainder and resumption conditions in the existing
   decision log/handoff. For saved stage summaries, follow
   `references/stage-consolidation.md`: append a new revision with its predecessor,
   never overwrite earlier summaries or erase failed retrieval history.

This is Host-directed comparison, not native automatic dependency invalidation.
Graph and artifact-change reads cover registered records, not all source packets,
summaries or prose. Persist only through the existing preview/approval/CAS owner;
a stale-source refusal requires fresh evidence and a new preview, not a retry with
an unverified new hash. Unaffected work can continue within the inspected scope.
