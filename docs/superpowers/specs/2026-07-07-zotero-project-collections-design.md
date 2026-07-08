# Zotero Project Collections Design

## Goal

When Qiongli writes references to local Zotero through the companion, references
should be grouped into project-specific Zotero collections instead of landing in
one mixed library-level pool.

## Current State

The MCPB already accepts `collection_path` and `review_collection_path`, and it
passes `collection_path` to `POST /qiongli/upsertItems`. The companion exposes a
`/qiongli/collections` endpoint, but the upsert path does not currently resolve,
create, or attach items to collections.

## Design

Collection selection uses this priority:

1. Explicit `collection_path`.
2. Explicit `review_collection_path`.
3. Configured default review collection path.
4. Configured default collection path.
5. A derived project collection path from `project_title`, `research_title`, or
   `topic`.
6. No collection, preserving existing behavior.

Derived paths use the root `Qiongli` plus a slug from the supplied title, for
example `Qiongli/platform-governance`. Slugs are lower-case ASCII, hyphen
separated, capped to a small number of meaningful tokens, and never override an
explicit collection path.

The MCPB owns conversation/project context. It resolves the final
`collection_path` before posting to the companion. The companion owns Zotero
library mutation: it resolves nested collection paths, creates missing
collections, and adds both new and existing items to the resolved collection.

## Companion Behavior

`upsertItems` should:

- Treat `dry_run: true` as non-mutating and report the intended
  `collection_path`.
- Create nested collections when `dry_run: false` and the path is missing.
- Add created items to the target collection.
- Add unchanged or updated duplicate items to the target collection if they are
  not already members.
- Include collection path/key details in each result where applicable.

## MCPB Behavior

`qiongli_zotero_upsert_references` should:

- Expose `project_title`, `research_title`, and `topic` as optional schema
  fields.
- Derive `Qiongli/<slug>` only when no explicit or configured collection path is
  available.
- Send the resolved `collection_path` to the companion.

## Testing

Use TDD against the existing Node test suites:

- Companion unit tests for dry-run reporting, nested collection creation, and
  attaching duplicate items to collections.
- Bootstrap VM tests for Zotero runtime collection APIs.
- MCPB tests for schema exposure and derived `collection_path` payloads.
