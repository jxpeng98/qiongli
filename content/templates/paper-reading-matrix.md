# Paper Reading Matrix Template

<!--
Usage: Maintain this compact matrix alongside the narrative paper reading summary.
Save to: RESEARCH/[topic]/literature/paper_reading_matrix.md
Source notes: RESEARCH/[topic]/notes/[citekey].md
-->

# Paper Reading Matrix

## Evidence Boundary

Do not invent citations, page numbers, sample sizes, methods, results, effect sizes, datasets, author claims, or implications. Do not upgrade an inference into a fact. Each row must include a `source_anchor`, an `evidence_limit`, and an inference-strength label. Unknown fields stay blank or become `unsupported_gap`.

Controlled labels:

- `evidence_limit`: `full_text`, `abstract_only`, `metadata_only`, `unavailable`
- `inference_strength`: `direct_evidence`, `reasonable_inference`, `unsupported_gap`
- `source_anchor`: citekey plus section, page, table, quote ID, abstract, or metadata field

Keep the inspected source version and exact passage binding in `source_anchor`;
use method/dataset/limitations cells to distinguish reports from independent
studies and preserve overlap or ambiguity. Reuse project claim IDs in the Claim
Index; qualify note-local IDs by their note. Carry the same IDs into the summary
and manuscript map. For a cross-source inference, give its own claim ID and name
its contributing claims/anchors. Unknown access or missing text is not contrary
evidence or proof that a paper did not address the question.
For body excerpts, follow the claim-level scope in `workflows/paper-read.md`:
`full_text` must be accompanied by `partial body excerpt`, inspected locations
and missing sections in `source_anchor`/limitations. It is not a whole-paper
access claim or a new retrieval status. Preserve free-form packet provenance
outside the controlled `evidence_limit` field.

## Matrix

| citekey | evidence_limit | theory/framework | method/identification | dataset/source | main finding | limitations | project relevance | source_anchor | inference_strength | gap note |
|---|---|---|---|---|---|---|---|---|---|---|
| | full_text / abstract_only / metadata_only / unavailable | | | | | | | citekey:section/table/page/abstract/metadata | direct_evidence / reasonable_inference / unsupported_gap | |

## Claim Index

| claim_id | claim | supporting_citekeys | source_anchor | evidence_limit | inference_strength | safe_to_use_in_writing |
|---|---|---|---|---|---|---|
| C1 | | | citekey:anchor | full_text / abstract_only / metadata_only | direct_evidence / reasonable_inference / unsupported_gap | yes / no / needs verification |

## Unsupported Or Under-Specified Items

| unsupported_gap | citekey | missing evidence | next action | blocked claim |
|---|---|---|---|---|
| | | full text / supplement / dataset documentation / additional papers | retrieve / screen / snowball / ask user | |

---
*Matrix created: [Date]*
*Last updated: [Date]*
