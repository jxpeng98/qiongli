# Qiongli Literature Provider MCPB

This package is the Claude Desktop adapter for Qiongli Marketplace Lite. The
primary MCPB contains the Rust Lite executable, so it does not require Node,
Python, the Qiongli CLI, npm, or pip at runtime.

Pair it with a manual Desktop skill ZIP when installing outside the Codex or
Claude Code plugin marketplaces. The MCPB supplies literature-provider tools;
the skill ZIP supplies the research workflow.

Upload a `qiongli-claude-desktop-skill-*.zip` first, then install this
Rust Lite MCP executable when the same workspace needs literature MCP tools.
Rust Lite does not launch orchestrator agents. Use the full CLI MCP for
executable tools such as `qiongli_task_run`.

## Runtime Boundary

Rust Lite provides:

- redacted provider configuration status and local configuration writes;
- a tokenized loopback setup page whose URL is returned to the MCP caller;
- bounded OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv search;
- deterministic normalization, DOI-first deduplication, title/year fallback,
  limits, partial-failure diagnostics, and evidence export;
- Zotero Connector/Companion status probes and import-file generation;
- preview-only route and task-plan responses.

Rust Lite does not launch agents, run shell commands, write project guidance,
search or modify a Zotero library, expand citation graphs, or perform the Full
runtime's domain-specific deep-search workflow. Install the Python Full runtime
for those capabilities:

```bash
qiongli mcp serve --transport stdio
```

## Provider Configuration

The Lite MCP and Full runtime read the same local `providers.json` contract.
Claude Desktop may also inject these MCPB settings:

- OpenAlex API key and optional contact email
- Semantic Scholar API key
- Crossref contact email
- NCBI/PubMed API key
- Default non-review result limit
- Zotero local-enabled flag and loopback Connector URL

OpenAlex, Semantic Scholar, Crossref, and PubMed are called only when their
activation field is configured. arXiv is available without credentials.

Call `qiongli_configure_provider` to start the local setup page. The result
contains a `127.0.0.1` URL with a one-time token; open that URL yourself. The
server does not promise to launch a system browser. `qiongli_open_config_wizard`
is the compatibility alias. Provider values are never returned in MCP output.

## Literature Search

The supported Lite arguments are deliberately small:

```json
{ "query": "platform governance" }
{ "query": "social media mental health", "search_mode": "review" }
{ "query": "climate governance", "providers": ["openalex", "arxiv"] }
{ "query": "climate governance", "per_provider_limit": 50, "total_limit": 75 }
```

Supported `search_mode` values are `auto`, `topic`, `review`, and
`systematic_review`. General searches default to 25 results per provider;
review modes default to 50. `limit` is the compatibility alias for
`per_provider_limit`; explicit per-provider values are bounded to `1..200`.
`total_limit` is applied after deduplication.

Provider calls are bounded and may complete independently. A partial provider
failure returns successful records with top-level `status: "warning"` and
`diagnostics.status: "partial"`. If every attempted provider fails, the result
uses `status: "error"`. Diagnostics expose stable error kinds, never
credential-bearing request URLs.

PubMed uses ESearch followed by ESummary. Records are deduplicated by normalized
DOI, then by normalized title and year when no DOI is present. Merged records
retain deterministic provider provenance.

## Zotero Boundary

Lite exposes two Zotero tools:

- `qiongli_zotero_status` probes only loopback Connector and Companion
  endpoints and returns `ok`, `companion_missing`, `fallback_only`, or
  `disabled`.
- `qiongli_zotero_export_import_files` produces `references.json`,
  `references.ris`, `bibliography.bib`, and `zotero-import-report.md` without
  writing to Zotero.

Zotero search, collections, tags, notes, and reference writes belong to the
Full runtime plus the separately installed Qiongli Zotero Companion. Import
files remain available when Desktop or the Companion is unavailable.

## Native Artifact Identity

The Rust Lite beta is built for the current host. Builders stage a machine-
readable target identity beside the executable, and the resulting artifact must
be named or scoped for that target. A current-host binary must not be presented
as a generic Darwin/Linux/Windows package.

## Development

Build the primary Rust Lite MCPB:

```bash
python3 scripts/build_literature_mcpb.py --dist-dir dist
```

The legacy Node reference remains an explicit compatibility artifact for one
release train. It keeps its own manifest overlay and advanced controls; those
controls are not claims about Rust Lite:

```bash
python3 scripts/build_literature_mcpb.py --dist-dir dist --legacy-node
npm --prefix packages/qiongli-literature-mcpb test
```
