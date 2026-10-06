# Literature Provider Routing

Read this reference when searching, configuring providers, or selecting search execution mode. Tool availability and permissions come from the active Host session.

The MCP/provider capability names `scholarly-search`, `metadata-registry` and
`fulltext-retrieval` identify routing owners; use the actual exposed tool schema
for execution. A capability name by itself does not establish an available reader.

For a supplied-excerpt explanation, no external search or search plan is needed.
When a search needs a plan but `qiongli_search_plan` is unavailable, record the
plan and capability limits in the response or proposed artifact. Execute only
actually available, authorized search tools; do not imitate an MCP call or
claim its result was saved. Honor a denial instead of switching search surfaces
to perform the denied operation.


- In native Qiongli 2, inspect configured provider metadata with visible `qiongli_config_status` and `qiongli_literature_status`. The current native `qiongli_configure_provider` and `qiongli_save_provider_config` handlers report unavailable writes; do not promise a setup browser or submit secrets to them. Use an actually available, authorized settings owner when configuration needs changing. `qiongli provider setup` and `qiongli provider doctor` are legacy 1.x commands, not native 2.x commands.
- Marketplace Lite plugin installs include a Rust-built self-contained local Literature Provider MCP runtime. They do not require user-installed Node, Python, npm, pip, Cargo, Rust, or the full Qiongli CLI for provider search tools.
- Full local orchestration is provided by the native Full MCP server. It returns bounded handoffs to Codex or Claude Code and never launches model processes itself; use `qiongli mcp serve --transport stdio --profile full` for doctor, checkpoint, evidence-read, and submission tools.
- Zotero local-library search and direct writes require Zotero Desktop plus the Qiongli Zotero Companion. Without the Companion, use the Lite MCP import-file export tools.
- In bundled MCP installs, do not expect the client MCP settings UI to inject provider keys into the Plugin server. Native status returns redacted metadata, not a usable config path or secret-store access proof. A configured key reference can still fail to load; do not treat that as a missing key or an API rejection.
- Keep provider secrets out of `.mcp.json`, plugin manifests, release ZIPs, research artifacts and chat. Native Qiongli 2 resolves them through its existing settings/secret-store owner; do not export keys or change system permissions to work around a failed read.
- In Codex plugin sessions, before declaring `strategy_only` for literature search, literature review, paper screening, citation snowballing, or evidence synthesis, attempt the visible `qiongli_literature_status` MCP tool. If it returns `capability_mode: provider_connected`, proceed with provider-backed literature workflow. If the tool is not visible in the session, state that Qiongli MCP tools are not visible and recommend reconnecting the installed Plugin or configuring the Host to launch the absolute installed CLI path with `mcp serve --profile full --transport stdio`. The legacy `qiongli install --target codex --parts mcp` command is not available in native 2.x. CLI installation does not automatically register or update a Plugin; do not claim the connection is ready until real MCP tools are visible. If provider preflight is unavailable or non-provider-connected but platform-native search is usable, write `qiongli_search_plan` with `search_execution_mode: native_only`.
- Literature workflows must create or update `qiongli_search_plan` after the `qiongli_literature_status` preflight and before search execution. The plan records `search_execution_mode` as exactly one of `hybrid_search`, `provider_connected`, `native_only`, or `strategy_only`; it separately records `provider_capability_mode` as `provider_connected` or `strategy_only` to show whether the MCP/provider layer has configured academic provider access.
- Do not use `qiongli_collect_evidence` to judge built-in literature provider configuration. That tool is a filesystem/builtin/external-command evidence adapter; direct provider names such as `openalex` require a separate `RESEARCH_MCP_OPENALEX_CMD`. Use visible `qiongli_literature_status`, `qiongli_config_status`, and `qiongli_literature_search` tools to judge OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv provider availability.
- Use `hybrid_search` when provider calls and platform-native search are both available and useful. Use `provider_connected` when the search run is provider-only. Use `native_only` when the active agent has platform native search but no provider-connected MCP. Use `strategy_only` only when neither provider MCP nor platform-native search is available and the workflow can only draft a search strategy or work from supplied corpus.
- MCP servers must not call Codex or Claude native search directly. The active agent executes `native_search_queries` from `qiongli_search_plan`; the MCP provider layer only performs provider calls and returns provider records. Do not hide native search behind a provider adapter.
- Preserve distinct provenance labels in `search_log.md`, `search_results.csv`, and diagnostics: provider records use labels such as `mcp:openalex`, `mcp:semantic_scholar`, `mcp:crossref`, `mcp:pubmed`, and `mcp:arxiv`; platform-native records use `native:codex_web_search` or `native:claude_web_search`; user-supplied files, notes, bibliographies, or pasted citations use `user_corpus`.
- `provider_capability_mode: provider_connected` describes configured provider metadata; `search_execution_mode` may still be `hybrid_search`. Neither value proves credential loading, live query success, coverage or metadata verification.
- Treat `strategy_only` as a constrained mode: draft the search strategy or use user-supplied corpus, record the limitation, and do not claim review-grade external provider or native-search coverage.
- Claude Desktop/Web focused ZIPs are skill-only packages kept within the 180-file upload budget. They contain workflows/prompts/templates, store no secrets, and cannot execute OpenAlex, Semantic Scholar, Crossref, PubMed, or arXiv API calls by themselves.
- For a manual Desktop install, upload the `qiongli-claude-desktop-skill-*.zip` first, then add a manual MCP install when provider calls or local orchestration are required. The skill ZIP supplies agent instructions, workflows/prompts/templates, and subject overlays; MCP supplies tool calls.
- Desktop/Web users need the Qiongli Literature Provider `.mcpb` (`qiongli-literature-provider.mcpb`) or another configured provider MCP before claiming `provider_connected` literature search. The MCPB is the separate local Claude Desktop provider for OpenAlex, Semantic Scholar, Crossref, PubMed, and arXiv configuration/search. Its primary package uses the Rust Lite MCP executable, not a user-installed Node or Python runtime. arXiv is enabled without credentials. Platform-native search alone is `native_only`, not `provider_connected`; if no provider MCP/MCPB and no platform-native search is available, record the run as `strategy_only`.
- The literature MCPB provides literature MCP tools only. It does not expose project orchestration. To add orchestration, install the native Full MCP server with `qiongli mcp serve --transport stdio --profile full`; the active Codex or Claude host executes each returned handoff and submits a bounded candidate back to Qiongli.

## Targeted references and bibliography delivery

For a few papers or a known item, use a bounded query and preserve its mode,
limits and sources; do not impose a systematic-review scaffold. `title`/`doi`
modes and `from_year`/`to_year`/`venue_filter` operate on bounded returned
candidates, not an exhaustive database filter. Check the visible tool schema;
an older installed Host may not expose them. Apply unsupported filters visibly
at the Host and report that limit instead of claiming the provider executed them.

Confirm title/DOI, authors, year and edition against the authoritative
publisher/registration-agency or repository record. Use `metadata-enricher`
for field checks and requested BibTeX, then `reference-manager-bridge` for
Zotero. Provider availability or a search hit is not a verified citation.
Preserve conflicts and absent fields. Native-only evidence is valid when its
source was actually inspected and labelled; do not relabel it MCP output.
Export `records` as objects with structured ordered authors, never JSON strings.
Use live capability/error responses for quotas and authentication requirements;
do not infer current service limits from an old static table.

If a combined native search cannot load some configured credentials, it can still
run the selected channels whose settings are usable. Retain its partial status,
`status_reason` and named warnings; an omitted, unsearched provider has unknown
hits, not zero. Do not repeat the same blocked selection or discard successful
public results. Actual Host search may fill a recorded gap; it does not establish
authenticated provider access or quota readiness.

## From discovery to source text

Select among the existing OpenAlex, Semantic Scholar, Crossref, PubMed and arXiv
providers for the question and source type; use actual capability/error responses
for access. Crossref can help confirm DOI metadata, PubMed biomedical records,
and arXiv repository versions. They complement discovery; none guarantees full
text. Do not apply an OA-only discovery filter unless the agreed protocol requires
it. Preserve source IDs, abstracts and `fulltext_candidates` through deduplication,
including each candidate's provider, format, version and license when supplied.
Absent fields remain unknown; a DOI or landing-page link is not a PDF.

Use actual Host search when it adds a missing repository, publisher page, known
item or full-text locator. Execute those queries in the declared `hybrid_search`
or `native_only` mode, record the Host tool/query/date/source URL, then inspect
the source. Search snippets and model recollection are not article body evidence.
The MCP plan can suggest `native_fulltext_queries`; only the active Host can
execute them. Respect denied access across both surfaces.

For source reading, load `skills/B_literature/fulltext-fetcher.md`. It owns the
native reader's actual schema, digest-bound continuation, source checks, access
limits and retrieval manifest. A Skill naming this capability does not add it to
an older installed Host or the retained Python planning adapter.
