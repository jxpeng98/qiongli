# Source-bound evidence verification

Use when checking whether a draft's substantive claims follow from its sources,
or when new source material may change an earlier conclusion. This is a bounded
review of the selected claims, not a new literature review or reporting checklist.
For independent execution, use `skills/Z_cross_cutting/model-collaborator.md` and
an actual authorized Host reviewer. Otherwise label the check self-review.

## Select the review and its inputs

Agree the consequential claims or requested passage from the task; do not require
a second agent for every sentence. Include unmarked substantive claims in that
passage, not just entries already present in a ledger. Retain claim/decision IDs,
citekeys, exact candidate identity and current source revision/digests. Give the
reviewer the relevant original passages, tables/notes and source access limits;
an author's summary or another review cannot substitute for these materials.

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

When evidence or candidate bytes change, preserve the old review and identify
affected claims, decisions and downstream passages. Verify those dependencies
against the new source before reusing their conclusions; unrelated settled work
can continue. A later-supplied section is new access to evidence, not necessarily
a correction to the paper. Record the revision, reason and resumption condition
in the existing decision log, handoff and collaboration trace.
