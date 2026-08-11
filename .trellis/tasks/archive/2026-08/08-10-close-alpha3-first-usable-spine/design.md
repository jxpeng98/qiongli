# Design

## Boundary

Use the existing product spine; add no new service or registry:

`content contracts -> native services -> App API / CLI / MCP -> Desktop / Host -> package evidence`

The current App work stays on `fix/alpha3-app-usability`. After it lands, any
Zotero implementation starts from the updated `2.x` head so review does not mix
the App customization contract with the Zotero MCP contract. This Trellis task
remains the single Alpha 3 execution record and updates its branch metadata at
that transition.

## App Slice

- Provider controls consume the native provider field model and existing
  preview/apply paths.
- `ProjectArtifactViewer.svelte` has one scroll owner for the content pane.
- Content customization returns bounded embedded source previews. Only project
  local guidance is editable, stored by `ProjectStateService` with containment,
  lock, atomic write, size limit, and expected-content digest checks.
- The App API version changes with the new intents/events and stays synchronized
  with the Rust fixture and browser transport.

## Zotero Slice

- Extend the existing native `CompanionClient`; do not port or duplicate Zotero
  business logic from JavaScript.
- Accept only `http://127.0.0.1` / `localhost` loopback endpoints, bounded JSON,
  bounded timeouts, endpoint contract `2`, and redacted errors.
- Add search and upsert tool IDs to the canonical v2 profile/schema and native
  registry together. Reuse the Companion's one-shot dry-run receipt unchanged.
- Keep import-file export as the explicit unavailable fallback.

## Claim And Package Slice

- Release-note sources, generated note templates, CLI examples, Skills, and MCP
  profiles are checked against the native registry before packaging.
- Use the existing `desktop:macos:acceptance` receipt. It is the only integrated
  local gate; exact-head CI owns the full cross-platform matrix.
- The candidate remains non-publishing. A6-A9 can later qualify public claims
  without reopening product work unless they reproduce an essential-path P0.

## Rollback

- App: remove the version-17 intents/events and local-guidance write as one
  contract slice; existing managed content remains untouched.
- Zotero: remove the added native tool registrations/schemas; existing status
  and import-file fallback remain available.
- Release: hold the candidate and retain Alpha 1 publicly. Never replace a tag
  or asset in place.
