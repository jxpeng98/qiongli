# Rigorous literature search

Use this workflow when you need a defensible account of how papers were found,
selected and used. Begin with the research question and agreed review protocol;
choose sources, date ranges and checks to suit that design.

## What Qiongli provides

The native MCP supports bounded searches through configured OpenAlex, Semantic
Scholar, Crossref, PubMed and arXiv providers. `qiongli_search_plan` proposes the
search route and queries; `qiongli_literature_search` executes provider requests
and reports normalized results and diagnostics. See [provider setup](mcp-providers-setup.md).

The Host can supplement this with its available native search and authorized
local materials. Citation expansion, full-text retrieval and screening still need
actual sources and suitable tools. A query plan, URL or metadata record is not a
retrieved paper. Do not treat the old Python adapter slots as installed native tools.

## Keep the search trace

For a formal review, retain the applicable outputs: `search_strategy.md`,
`search_log.md`, `search_results.csv`, `dedup_log.csv`, `snowball_log.md`,
`bibliography.bib`, `screening/full_text.md` and `retrieval_manifest.csv`.
Record the service, query, date, filters, exclusions and failures. Carry stable
citekeys and source locations into extraction and synthesis. Saving follows the
project's existing preview and approval process.

## Assess coverage without overstating it

No provider can prove absolute completeness. Use checks that fit the protocol:

| Check | What it can tell you |
|---|---|
| known-item recall | Whether the strategy finds papers that should be included |
| Concept and source coverage | Whether each required query block and database was searched, with zero-hit or unavailable paths explained |
| duplicate saturation | Whether additional queries mainly return records already seen; this alone does not prove completeness |
| Citation follow-up | Whether required backward and forward checks were performed and logged |
| full-text access coverage | Which sought reports were actually retrieved and readable |

Suggestions in `native_fulltext_queries` still need to be executed by the Host.
Keep candidate URLs and access dates. Zotero attachment verification also requires
checking the actual attachment: a matching library item does not prove a readable
PDF is available, and a readable PDF does not by itself support a particular claim.

Record `evidence_limit` for each extracted claim: `full_text`, `abstract_only`,
`metadata_only` or `unavailable`, as supported by the material actually read.
Retain version details when a preprint differs from the published article.
For a claim grounded in a supplied body excerpt, the reading templates retain
`full_text` with an explicit `partial body excerpt` qualifier and inspected/missing
locations in the anchor or limitations. This describes that claim's evidence;
it does not mark the whole paper retrieved or read. Keep retrieval status and
corpus coverage separate, and preserve the packet's original access description.

## Connect findings to the Graph

After reviewing the extraction, propose structured claims and source links.
Preserve uncertainty and unresolved conflicts when saving them. The
[Research Graph](../guide/cli-2x.md#research-graph) helps inspect those saved
relationships; it does not replace screening decisions or establish search recall.

## Read back a saved passage — 2.4.0 candidate

The candidate reader can select an observed string field in a saved source
packet. First read the project's current revision and recover its saved bindings
with `qiongli project document list`. Only a `current` binding supplies usable
`readArguments`; preserve its path and whole-file SHA-256. Inspect the packet's
actual structure with the raw document reader before selecting a field.

For example, if the inspected passage is at `/segments/2/text`:

```bash
qiongli project document read \
  --project-id <prj_id> --expected-project-revision <revision> \
  --relative-path sources/<citekey>/<sha256>.json --expected-sha256 <sha256> \
  --json-pointer /segments/2/text --max-bytes 4096 --json
```

Full MCP exposes the same optional `json_pointer` on
`qiongli_project_document_read`. Check CLI help or the live tool schema first;
older installations retain raw reads. Array packets have different paths, such
as `/0/segments/2/text`; these examples must not replace inspecting real data.

The response contains decoded text, `jsonPointer` and `selectedTextSizeBytes`.
Offsets and `nextOffsetBytes` address that string; `sha256` and `sourceSizeBytes`
still describe the entire raw packet. Continue with the same pointer, revision
and hash. Invalid locations or changed files refuse instead of choosing a
replacement passage. The operation reads only the selected known file.

Retain this locator with the existing claim/reading-note record and the observed
source version, section, table or page. Read relevant surrounding passages and
table notes before judging support. Byte offsets are not paper page numbers,
and exact text readback is not academic verification.

## Find a phrase in saved material — 2.4.0 candidate

The candidate can search one explicitly bound saved file. After recovering its
current binding as above, use a phrase from the source language:

```bash
qiongli project document search \
  --project-id <prj_id> --expected-project-revision <revision> \
  --relative-path sources/<citekey>/<sha256>.json --expected-sha256 <sha256> \
  --search-text 'reported phrase' --max-matches 8 --context-bytes 128 --json
```

Full MCP selects the same mode by adding `search_text` to
`qiongli_project_document_read`; omit read-only `offset_bytes`/`max_bytes`.
Check live help/schema for availability. An optional `json_pointer` searches one
observed string. Without it, a packet searches decoded string values, including
metadata; notes and retrieval history search raw text. Matches are literal,
case-sensitive and non-overlapping, without normalization or crossing fields.

Each match returns context, UTF-8 match offsets and `readArguments` that reproduce
the context through the reader. `context_bytes` is the maximum on each side;
raise the read window if the claim needs more surrounding text. Repeated text at
different pointers remains separate. Check original source metadata and content
before treating a match as evidence. Keep reviewed locations in existing notes
and evidence records; neither byte offsets nor metadata matches are paper pages.

For another page, pass `nextMatchOffset` as `--match-offset` and `searchSha256`
as `--expected-search-sha256`, retaining the same query, pointer, context, page
size, revision and file hash. Changed bindings refuse. `searchScope`,
`scannedTextFields` and `totalMatches` describe only that saved scope. No match
does not establish absence from the full paper. Search other selected sources
through their own known bindings. Automatic cross-document traversal and
native claim extraction remain outside this reader's scope.

## Trace the manuscript being written — 2.4.0 candidate

Traceability also applies during drafting and revision. Start with the actual
body text, captions and notes: a sentence may combine a finding, explanation and
inference that need different support. Include assertions absent from the current
claim map. Retain the final text span and location, reuse an existing claim ID only
for the same meaning, and keep note-local IDs distinct from project-wide IDs.

The writing Host follows each claim through the current map and all relevant
ledger rows, then inspects the original source or analysis output. Saved packets
use the bound read/search route above. A note summary or Graph support edge is
not a substitute for reading the evidence. Record supporting and contrary material
separately; ambiguous versions, unavailable context and changed files remain gaps.
Do not choose a replacement hash or source merely because it matches the wording.

Review the prose separately: explain what the evidence means for the paragraph's
point, make the connection to the next paragraph explicit, and define concepts
needed by the reader. Put essential reasoning and limits in the body; use a
sourced explanatory note for useful secondary detail when the format allows it.
A citation-complete paragraph can still be shallow or disconnected. After a
meaning-bearing edit, recheck both the source match and the revised flow.

This is a Host-guided writing/review process using existing records and tools,
not native semantic extraction or automatic academic approval. A direct paragraph
request needs no new project or unsolicited claim table. The 2.4 plan retains
installed-workflow and final-candidate qualification separately.

## Keep synthesis and later revisions traceable — 2.4.0 candidate

Carry claim IDs and each inspected source/version/locator from the reading matrix
into the summary, synthesis and manuscript map. Give a cross-paper inference its
own identity and retain its contributing claims. Compare designs, outcomes, time
points and uncertainty in relation to the paper's question. Several notes or
reports from one study are not independent corroboration; paper counts alone do
not establish consensus. Unknown access is different from a conflicting finding.

When a source changes, the Host compares its old and new content and follows the
affected claims through actual prose, including paraphrases, background, abstract
and notes. Return revised passages with the impact record, retaining unchanged
conclusions and reasons. A correction, newly accessible section, reformatted file
and failed retrieval need different checks; a changed digest alone says nothing
about the academic conclusion. Preserve old source bindings, reviews and failures,
and state the missing evidence needed for any unresolved use.

Use the existing decision log and handoff for this comparison; saved stage
summaries receive a new revision linked to the predecessor. Native Graph and
artifact-change reads do not automatically find every semantic dependency.
Proposed edits still use the existing project preview, approval and revision
checks; neither re-reading a source nor updating a matrix approves the manuscript.
