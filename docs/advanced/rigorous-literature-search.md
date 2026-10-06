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
and exact text readback is not academic verification. Automatic cross-document
passage search and claim-to-passage traversal remain subsequent work in the
2.4 plan; the Graph currently opens the structured evidence record.
