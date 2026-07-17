# Qiongli R2C Evidence And Zotero Runtime Design

Status: approved for execution

Date: July 14, 2026

Scope: bounded evidence snapshots and the accepted read-only/in-memory Zotero
surface in the shared Rust runtime

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

Move the accepted Marketplace Lite evidence snapshot and Zotero behavior into
`qiongli-runtime`. Keep `qiongli-lite-mcp` as a protocol and legacy-environment
adapter over those shared services.

This batch implements exactly three domain operations:

1. build a redacted, auditable literature evidence snapshot in memory;
2. probe the fixed Zotero Desktop and Qiongli Companion loopback endpoints; and
3. generate selected CSL-JSON, RIS, BibTeX, and import-report contents in
   memory.

It does not write files, mutate a Zotero library, install the companion, search
local Zotero content, or expose native MCP mode in the canonical executable.

## Ownership Boundary

`qiongli-runtime` owns:

- evidence input validation, compatibility aliases, bounds, recursive
  credential-key redaction, and snapshot construction;
- Zotero record and format validation, deterministic serialization, escaping,
  and output bounds;
- loopback URL validation, bounded HTTP probing, status projection, and the
  import-file fallback description; and
- typed, sanitized errors that do not echo peer-controlled values.

The old Lite package owns only:

- mapping the historical `QIONGLI_ZOTERO_*` environment values into the shared
  typed client;
- mapping shared validation errors to the existing JSON-RPC error transport;
  and
- producing the MCP content envelope and its existing defense-in-depth output
  redaction.

The canonical 2.x path does not read Zotero settings from process environment.
A future native settings/UI batch may add a typed Zotero setting; it must call
the same runtime service instead of creating another client.

## Evidence Snapshot Contract

The runtime accepts the Contract v2 canonical fields and the three deprecated
aliases. Supplying both a canonical field and its alias is rejected as
ambiguous. `cwd` remains a validated, ignored compatibility field and is never
copied into the snapshot.

| Boundary | Value |
|---|---:|
| query | optional string, at most 4,096 UTF-8 bytes |
| results | at most 1,000 objects |
| JSON nesting | at most 32 containers |
| traversed JSON values | at most 100,000 |
| accepted snapshot input | at most 2 MiB when serialized |

`provider_status`, `search_plan`, `diagnostics`, and every result must retain
their non-credential evidence fields. Keys representing passwords, secrets,
credentials, authorization, cookies, private keys, access keys, API keys, and
tokens are removed recursively. Benign fields such as `token_budget` and
`public_key` remain. The snapshot is deterministic and omits wall-clock time;
callers may add an external receipt time without changing evidence identity.

The operation is side-effect free. Errors identify only a static reason and
must not contain query text, unknown key names, credential values, or nested
payload content.

## Zotero Import-File Contract

The exporter accepts normalized `LiteratureResult` records and an optional,
unique selection from these exact names:

- `references.json`;
- `references.ris`;
- `bibliography.bib`; and
- `zotero-import-report.md`.

An absent or empty format selection means all four formats for compatibility.
The exporter validates before generation:

| Boundary | Value |
|---|---:|
| records | at most 1,000 |
| serialized record input | at most 2 MiB |
| title or venue | at most 16 KiB each |
| DOI | at most 2 KiB |
| provider identifiers | at most 16 values and 128 bytes each |
| combined generated files | at most 2 MiB |

Titles must be non-empty. RIS values fold control/newline characters so a
record cannot inject fields. BibTeX values escape syntax-bearing characters.
CSL-JSON is emitted through `serde_json`; every output is deterministic and
held in memory. A returned `files` object is not evidence that a user imported
those files into Zotero.

## Zotero Probe Contract

The production default is `http://127.0.0.1:23119`. An explicit base URL is
accepted only when it:

- uses HTTP or HTTPS;
- has no username or password;
- resolves syntactically to `localhost` or a literal loopback address; and
- can act as a base URL.

The client normalizes to the origin root and calls only `/connector/ping` and,
after connector success, `/qiongli/ping`. It disables redirects and implicit
proxy discovery, uses a five-second production timeout, accepts only bounded
test timeouts from 1 millisecond through 30 seconds, and reads at most 32 KiB
from the companion response. It returns availability, HTTP status, and bounded
version strings only; it does not return endpoint URLs, response bodies,
library data, or credentials.

An unavailable connector returns `fallback_only`; a connector without the
Qiongli endpoint returns `companion_missing`; both probes succeeding returns
`ok`. Disabled probing performs no network call and still describes the
in-memory import-file fallback.

## Compatibility Extraction

The old `zotero::export` module becomes a re-export of the shared exporter. The
old `zotero::companion` module re-exports shared client/status types and keeps
only `probe_zotero_from_env` plus its boolean compatibility parser. Evidence
construction in the old server delegates to the shared input parser and
snapshot builder.

Existing Lite public tool names and structured output shapes remain unchanged.
Invalid enum values continue to map to static JSON-RPC messages and never echo
attacker-controlled input.

## Verification

Native tests cover canonical/alias parsing, ambiguity, bounds, recursive
redaction, deterministic snapshots, record validation, selected formats,
RIS/BibTeX injection resistance, output bounds, loopback URL rules, disabled
status, redirects, malformed/oversized JSON, and timeouts. Existing Lite tests
remain on the compatibility paths and prove MCP result shapes.

The required gate is native format, locked workspace check, strict Clippy,
all native tests, Windows MSVC cross-target check/Clippy, and focused Lite
protocol/server/evidence/Zotero tests. No live Zotero instance, Python suite,
Node suite, or scholarly service is required.

## Nonclaims

R2C does not implement or claim:

- file writes or automatic Zotero imports;
- Zotero library search, item updates, collection management, notes,
  attachments, or Crossref verification;
- a production native Zotero settings/UI surface;
- installation or activation of the Zotero Companion;
- canonical binary MCP initialize, tools/list, or tools/call availability;
- orchestration, installer, desktop UI, packaging, or release readiness; or
- completion of R2.

## Acceptance Criteria

R2C is complete only when:

1. shared Rust code owns all evidence snapshot, Zotero serialization, and
   loopback probing behavior used by the Lite entrypoint;
2. direct runtime calls enforce the declared bounds and redaction without
   relying on the MCP envelope;
3. generated RIS and BibTeX cannot gain fields through embedded newlines or
   unescaped syntax;
4. the probe cannot follow redirects, use an implicit proxy, contact a
   non-loopback URL, or return response content;
5. old Lite modules contain only re-exports and the named environment adapter;
6. focused compatibility behavior remains green;
7. local native and Windows gates pass without Python or Node; and
8. exact-head GitHub jobs pass before the roadmap records completion.

## Approval Record

The user instructed continuation under the accepted accelerated roadmap on
July 14, 2026. This authorizes R2C on the existing rolling branch and Draft PR
without expanding the public product claims.
