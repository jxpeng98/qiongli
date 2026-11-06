# Unseen Chinese paragraphs — November 6, 2026

This increment tests the preceding Chinese-expression guidance on two passages
from Vincent A. Traag's [Complex Contagion of Campaign Donations](https://doi.org/10.1371/journal.pone.0153539)
(PLOS ONE, 2016; CC BY). The publisher's complete HTML, XML and PDF are retained.
Exact publisher JATS section/paragraph locators identify the excerpts; they are
not PDF page numbers. A first-use search at `486dd30c` finds no matching paper in
canonical content, research-journey evaluations or the current execution plan.
This is local first use, not a statement about model training data.

The two tasks ask for Chinese scholarly paragraphs explaining (1) independent
versus cohesive reinforcement and their conditions and (2) an observational
analysis of donation contagion, its controls and the conclusion it supports.
Each requires 250–450 non-whitespace Unicode characters, excluding its heading
and standalone source marker. Examples, statistics and reference lists may be
omitted when they are not needed for the requested argument. The study's scope
and consequential uncertainty must remain. This is source-based writing, not a
translation of the complete paper, data replication or campaign advice.

## Frozen comparison and actual execution

One writer receives the shared Stage F and scholarly-voice guidance from
`04af70af`; another receives the same files from `486dd30c`. The complete prompts
differ only by the 16-line Chinese clarification/example added in `81e2e5ad`.
The task, source excerpts and model configuration are identical. One generation
per arm cannot isolate a general rule effect from sampling variation.

Both writers and a fresh reviewer use the configured `gpt-6-astra` / `xhigh`,
with no model override. Complete source/guidance/task inputs arrive inline; all
three turns finish without tool calls, outer truncation or changed inputs.
Read-only tool permissions, disabled tool network and empty MCP configuration
are read back. Temporary authentication links are removed, and the user's config
digest remains unchanged. Codex's warning that it declined optional PATH helper
aliases under the temporary profile is retained; no requested operation failed.
Writer times are 260.5 and 201.1 seconds; review takes 192.7 seconds. These are
observed durations, not a speed comparison.

The completed outputs are randomly labelled A/B once, before review, and the
reviewer receives no producer mapping. The [anonymous comparison](anonymous-comparison.md)
retains the four exact raw passages. The key, task, excerpts, hashes, criteria,
runtime records and full review are in [observation.json](observation.json).
Full raw local evidence is at `/tmp/qiongli-zh-holdout-20261106-3vfczvmn`.

## Results and evaluation repair

| Raw passage | Characters | Argument | Natural Chinese | Necessary qualifications |
|---|---:|---|---|---|
| A, paragraph 1 | 363 | Partial in the original model review; pass after scope adjudication | Pass | Pass |
| B, paragraph 1 | 348 | Pass | Pass | Pass |
| A, paragraph 2 | 372 | Pass | Pass | Pass |
| B, paragraph 2 | 379 | Pass | Pass | Pass |

The model ties both language comparisons and both qualification comparisons.
It prefers B's first argument because B mentions an observation of cooperation
spreading in public-goods experiments, whereas A only states that the driving
reinforcement mechanism remains unresolved. That original judgment is preserved.

The coordinator finds a mismatch between the frozen checklist and the writing
request: A already explains independent signals, relational benefits,
coordination and uncertainty. Omitting the contextual experiment observation
removes no premise needed for those conditional explanations and does not deny
that cooperation was observed. The review itself calls the core argument
complete. Adding this observation can help, but it is optional for this task.
The separate scope adjudication therefore passes A's argument while retaining
the original partial score and preference. This is not a second blind review.
The existing review-criteria owner now requires a missing premise and its
consequence before treating omission as incompleteness. Correct prose is retained;
canonical writing guidance and its content pack need no further change here.

## Human feedback and combined wording

The user finds advantages in both versions, prefers B's overall fluency, and
wants to retain some of A's wording and completeness. The [exact feedback](human-feedback.json)
is bound to the anonymous file. No separate per-case scores, exact preferred
phrases or editing time were supplied. The main coordinator chooses the specific
edits; those choices are not attributed to the user.

The [combined wording](selected.md) uses B as its base. Paragraph 1 retains B's
order and adds A's network-externality definition and clearer comparison terms.
Paragraph 2 uses A's concrete explanation of selective fundraising, preserves
B's control limitations and makes source independence explicit. The source
comparison retains author attribution, hypotheses, observed associations,
unresolved confounding and exact citation anchors. The resulting paragraphs have
348 and 391 characters. This is an editorial selection after feedback, not
another trial or a human judgment of the combined wording.

The human-facing file excludes identities and automatic judgments, but the
coordinator printed the key and model review in tool output before the reply.
Whether the user saw that output is unknown; this feedback is therefore not
claimed as strictly blinded. The review criteria now keep such information out
of visible tool output until feedback arrives. The exposure and original feedback
remain; relabelling these same texts would not create a fresh blind trial.

Input bindings, complete prompt delivery, source locators, raw-output preservation,
counts, selected-text meanings and cleanup are checked. The existing execution
plan records integration checks once. No English, longer-manuscript, independent
expert, new Antigravity return or broad quality gain is established. These excerpts
become regression material after this review; a future held-out check needs new
material and should withhold the key and automatic judgments through human review.
