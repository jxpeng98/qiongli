# Zotero Source And Crossref Verification Design

## Goal

Extend the local Zotero bridge so Zotero can optionally participate as a
reference source during Qiongli literature search, and make Zotero writes safer
by adding DOI-based Crossref verification, conservative metadata enrichment, and
explicit review-state tagging.

The default search behavior must remain unchanged. Zotero is included in the
main literature search only when the caller explicitly passes
`include_zotero: true`.

## Context

The current local Zotero bridge already provides:

- `qiongli_zotero_status` for connector and companion readiness.
- `qiongli_zotero_search` for local Zotero library search.
- `qiongli_zotero_upsert_references` for dry-run or explicit local writes.
- `qiongli_zotero_export_import_files` for CSL-JSON, RIS, BibTeX, and report
  fallback.
- A Qiongli Zotero companion extension with `/qiongli/search` and
  `/qiongli/upsertItems` endpoints.
- A Crossref provider that supports DOI singleton lookup through
  `https://api.crossref.org/works/{doi}`.

This design builds on those pieces. It does not replace the local companion or
introduce a dependency on third-party Zotero MCP servers.

## Non-Goals

- Do not enable Zotero as a default search source.
- Do not treat Zotero as a public scholarly discovery index.
- Do not overwrite user-curated Zotero fields by default.
- Do not require Zotero Web API, Zotero cloud sync, or a third-party Zotero
  plugin.
- Do not use Crossref title search as an automatic verifier for DOI-less
  records in the first implementation, because title search has a higher false
  match risk.
- Do not claim Crossref verification means human verification. It is metadata
  verification from DOI registration data.

## User-Facing Behavior

### Optional Zotero Source In Main Search

`qiongli_literature_search` gains explicit Zotero options:

```json
{
  "query": "platform governance",
  "include_zotero": true,
  "zotero_limit": 25
}
```

Defaults:

- `include_zotero`: `false`
- `zotero_limit`: bounded by the same search-depth limits, but should not exceed
  the per-provider limit unless explicitly supplied.
- Zotero failures are non-fatal. A missing Zotero companion adds a warning and
  the external providers continue.

When `include_zotero` is false, `qiongli_literature_search` must not call the
local Zotero companion.

### Search Result Shape

Zotero source records are normalized into the same reference result shape as
external provider records:

```json
{
  "title": "Local Paper",
  "authors": [],
  "year": 2024,
  "doi": "10.1000/local",
  "url": "",
  "abstract": "",
  "venue": "",
  "document_type": "journalArticle",
  "provider": "zotero",
  "source_id": "ABC123",
  "source_type": "local_reference_database",
  "zotero": {
    "item_key": "ABC123",
    "select_uri": "zotero://select/library/items/ABC123",
    "tags": ["qiongli:verified"],
    "collections": ["Qiongli/platform-governance"]
  }
}
```

If an external provider result matches a local Zotero item by DOI or normalized
title/year, the external result is still returned as the canonical search result
unless ranking/deduplication already prefers the local item. The result should
carry a local match marker:

```json
{
  "local_zotero_match": {
    "item_key": "ABC123",
    "match_basis": "doi",
    "select_uri": "zotero://select/library/items/ABC123"
  }
}
```

This lets Qiongli tell the user that a discovered reference already exists in
their local library without losing provider provenance.

## Main Search Integration

### Provider Attempt Model

When `include_zotero: true`, the search response should include Zotero in the
provider accounting:

```json
{
  "providers": {
    "attempted": ["openalex", "semantic_scholar", "crossref", "pubmed", "zotero"],
    "successful": ["openalex", "zotero"],
    "failed": ["semantic_scholar"]
  }
}
```

Zotero diagnostics should identify it as a local source:

```json
{
  "provider": "zotero",
  "source_type": "local_reference_database",
  "result_count": 4,
  "status": "success"
}
```

### Query Mapping

For main search integration, Qiongli maps the literature search intent to a
Zotero query:

- DOI intent: send `{ "doi": "<doi>" }`.
- Exact title intent: send `{ "title": "<title>" }`.
- Topic/review intent: send `{ "title": "<query>" }` in the first slice.
- Optional filters:
  - `fromYear` and `toYear` become local post-filters when available.
  - `zotero_tag` maps to companion `tag`.
  - `zotero_collection_path` maps to companion `collection_path`.

The first implementation should avoid broad full-library scans. The companion
may still perform local filtering over Zotero items internally, but the MCPB
contract should bound the returned result count.

### Ranking And Deduplication

Deduplication order stays DOI-first, then normalized title/year. Local Zotero
records should not erase richer external provider records during search. The
preferred behavior is:

1. Merge exact DOI duplicates.
2. Preserve provider provenance in a `sources` or `duplicates` trace when the
   existing normalizer supports it.
3. Attach `local_zotero_match` when a local item matches an external result.
4. If a record exists only in Zotero, return it as `provider: "zotero"`.

This distinguishes "already in my library" from "new discovery result".

## Zotero Write Safety

### Default Review Tags

All writes through `qiongli_zotero_upsert_references` should add Qiongli review
state tags unless the caller opts out:

- `qiongli:imported`
- `qiongli:needs-review`
- `qiongli:source:<provider>` when a provider is known

Crossref verification adds one of:

- `qiongli:crossref-verified`
- `qiongli:metadata-conflict`
- `qiongli:metadata-unverified`
- `qiongli:verification-unavailable`

The default collection path should route new imports to a review area when a
topic is known:

```text
Qiongli/[topic]/To Review
```

If no topic is supplied, use either the configured default collection path or no
collection. Do not invent a topic from a low-confidence query string.

### Review Status Field

Every upsert result should include `review_status`:

- `needs_review`: created or updated records that require human confirmation.
- `unchanged`: no write was needed.
- `skipped`: the record was not written.

Crossref results are reported separately under `verification.crossref.status`.
The default for newly imported records is `needs_review`, even when Crossref
verification succeeds, because registry metadata is not the same as human
confirmation.

### Field Update Policy

The write policy remains conservative:

1. Zotero existing non-empty user fields win by default.
2. Original Qiongli/provider non-empty fields win over Crossref fields.
3. Crossref DOI metadata fills blank fields.
4. Empty values never overwrite non-empty values.
5. `update_policy: "prefer_enriched"` is required before enriched metadata can
   overwrite non-empty Zotero fields.

Fields considered user-curated and protected by default:

- `title`
- `creators`
- `date`
- `publicationTitle`
- `conferenceName`
- `abstractNote`
- `url`
- `extra`, except appending Qiongli provenance lines

Tags and collection membership may be appended, not removed.

## Crossref Verification And Enrichment

### When It Runs

`qiongli_zotero_upsert_references` gains Crossref options:

```json
{
  "verify_crossref": true,
  "crossref_enrichment": "fill_blank"
}
```

Defaults:

- `verify_crossref`: `true` when a record has a DOI.
- `verify_crossref`: effectively skipped when no DOI is present.
- `crossref_enrichment`: `fill_blank`.

Callers can set `verify_crossref: false` for fully offline/local writes.

### Data Source

Crossref enrichment uses DOI registration metadata from:

```text
https://api.crossref.org/works/{doi}
```

This is the same DOI lookup path already implemented by the MCPB Crossref
provider. It should reuse the existing provider code path first, rather than
adding a new dependency or a separate plugin.

Crossref can provide or help normalize:

- title
- authors
- publication year
- DOI
- DOI landing URL
- venue/container title
- document type
- abstract when Crossref has one
- reference count
- references when available
- publisher, volume, issue, pages, ISSN, ISBN, license, and funder in a later
  mapper expansion

Crossref does not provide a guarantee that the metadata is complete or
human-reviewed for the user's project. It is registry metadata.

### Verification Outcomes

For each record with DOI:

- `verified`: Crossref returns a DOI record and material fields are compatible.
- `conflict`: Crossref returns a DOI record but title/year/venue materially
  conflict with the incoming metadata.
- `not_found`: Crossref returns no usable DOI record.
- `unavailable`: Crossref lookup fails due to network, rate limit, provider
  error, or configuration/runtime failure.
- `skipped`: no DOI or `verify_crossref: false`.

The upsert response should include a verification trace:

```json
{
  "verification": {
    "crossref": {
      "status": "verified",
      "doi": "10.1000/example",
      "filled_fields": ["venue", "year"],
      "conflicts": []
    }
  }
}
```

For conflicts:

```json
{
  "verification": {
    "crossref": {
      "status": "conflict",
      "doi": "10.1000/example",
      "filled_fields": [],
      "conflicts": [
        {
          "field": "title",
          "incoming": "Working Paper Title",
          "crossref": "Published Article Title"
        }
      ]
    }
  }
}
```

### Conflict Rules

Use conservative, explainable conflict rules:

- DOI equality is strong identity evidence.
- Title conflict only when normalized titles are materially different, not just
  punctuation/case differences.
- Year conflict only when both years exist and differ.
- Venue conflict is advisory, because abbreviations and proceedings names vary.
- Missing Crossref fields are not conflicts.

Conflict records should not be blocked automatically. They should be written
only if the caller requested a write, and they should be tagged
`qiongli:metadata-conflict` plus `qiongli:needs-review`.

## External Plugin Coordination

The first implementation should reuse the internal Crossref provider because it
already exists and has tests. The design should leave an extension point for
future verifier plugins:

```json
{
  "metadata_verifiers": ["crossref"]
}
```

Future verifier adapters may include OpenAlex, Semantic Scholar, PubMed, Zotero
Web API, or a separate MCP/plugin. They must conform to the same verifier result
shape:

- `name`
- `status`
- `identity_basis`
- `candidate_metadata`
- `filled_fields`
- `conflicts`
- `warnings`

Do not require another plugin for the Crossref path in this slice.

## Import File Fallback

Import-file generation should include the same review-state metadata where the
target format supports it:

- CSL-JSON `keyword` includes Qiongli tags.
- RIS includes `KW` tags.
- BibTeX includes Qiongli tags in `keywords` or `note`.
- `zotero-import-report.md` includes verification counts:
  - crossref verified
  - metadata conflicts
  - unverified/no DOI
  - verification unavailable

Generated files should not claim that Crossref-verified records are
human-verified.

## Configuration

Add MCPB user config / environment mappings:

- `zotero_default_review_tags`
  - Default: `qiongli:imported,qiongli:needs-review`.

- `zotero_default_review_collection_path`
  - Default: blank, or `Qiongli/[topic]/To Review` when topic context is
    supplied.

- `zotero_crossref_verification_enabled`
  - Default: `true`.

The project should keep loopback-only validation for Zotero connector URLs.
Crossref polite email remains configured through the existing Crossref provider
configuration.

Do not add a configuration field that silently enables Zotero in
`qiongli_literature_search`. Zotero participates in main search only when the
tool call explicitly passes `include_zotero: true`.

## Error Handling

- Missing Zotero companion during `include_zotero: true` search:
  - Return external provider results.
  - Add warning `zotero_companion_missing`.
  - Add diagnostic provider row with status `failed`.

- Zotero connector unreachable:
  - Same as missing companion, but warning `zotero_not_running`.

- Crossref DOI lookup fails:
  - Do not fail the whole upsert by default.
  - Mark affected records with `verification.crossref.status: "unavailable"`
    and tag `qiongli:verification-unavailable`.
  - Continue dry-run/write unless caller sets a future strict mode.

- Crossref returns metadata conflict:
  - Preserve incoming/provider metadata.
  - Add conflict trace and review tags.
  - Continue dry-run/write.

- Crossref rate limit or network error:
  - Return sanitized provider error.
  - Do not expose stack traces or secrets.

## Security And Privacy

- Zotero connector URL remains loopback-only.
- Search responses must not expose local filesystem paths, attachment paths, or
  note bodies by default.
- Local Zotero item keys and `zotero://select/...` URIs are acceptable because
  they are needed for user navigation.
- Crossref lookup sends DOI and polite email if configured. The response should
  disclose that external verification was attempted.
- Do not send Zotero notes, local paths, or collection names to Crossref.

## Test Plan

### MCPB Tool Schema Tests

- `qiongli_literature_search` exposes `include_zotero`, `zotero_limit`,
  `zotero_tag`, and `zotero_collection_path`.
- `qiongli_zotero_upsert_references` exposes `verify_crossref`,
  `crossref_enrichment`, and review tag controls.

### Search Integration Tests

- Default `qiongli_literature_search` does not call Zotero.
- `include_zotero: true` calls the companion search endpoint.
- Zotero-only local results appear with `provider: "zotero"` and
  `source_type: "local_reference_database"`.
- External results get `local_zotero_match` when DOI matches a Zotero item.
- Missing Zotero companion adds a warning but does not fail external search.

### Crossref Verification Tests

- Upsert with DOI calls Crossref DOI lookup by default.
- Crossref fills blank fields before payload mapping.
- Crossref does not overwrite non-empty incoming fields by default.
- Crossref title/year conflict creates `verification.crossref.status:
  "conflict"` and tag `qiongli:metadata-conflict`.
- No DOI creates `verification.crossref.status: "skipped"` and tag
  `qiongli:metadata-unverified`.
- `verify_crossref: false` skips lookup.
- Crossref provider failure creates `verification.crossref.status:
  "unavailable"` without failing
  the whole dry run.

### Companion Tests

- `upsertItems` receives review tags and preserves them in create/update payloads.
- `toCompactItem` returns no local paths and includes tags/collections needed
  for source search.

### Artifact And Docs Tests

- MCPB manifest packages new schema fields and Zotero source docs.
- Companion XPI artifact still excludes tests, local paths, and fixture secrets.
- English and Chinese docs explain:
  - Zotero is explicit opt-in for main search.
  - Imported records are candidates needing review.
  - Crossref DOI verification is registry metadata, not human verification.

## Rollout

1. Add schema flags and tests.
2. Implement Zotero source inclusion in `qiongli_literature_search`.
3. Implement local match annotation and diagnostics.
4. Implement Crossref verifier helper for upsert.
5. Add default review tags and review status.
6. Update import-file fallback metadata.
7. Update English/Chinese docs and skill text.

## Acceptance Criteria

- Main search behavior is unchanged unless `include_zotero: true`.
- With `include_zotero: true`, local Zotero results can appear in search output.
- Existing local Zotero matches are surfaced on external provider results.
- Zotero writes default to dry-run and add review-state tags when writing.
- DOI-bearing imports run Crossref verification by default unless disabled.
- Crossref fills only blank fields under the default policy.
- Metadata conflicts are visible in response payloads and tags.
- No local attachment paths or secrets appear in packaged artifacts or responses.
- Tests cover default-off search behavior, opt-in Zotero search, Crossref
  enrichment, conflict tagging, and fallback import files.
