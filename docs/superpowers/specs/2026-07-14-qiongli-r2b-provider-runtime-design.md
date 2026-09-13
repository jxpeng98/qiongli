# Qiongli R2B Provider Runtime Design

Status: approved for execution

Date: July 14, 2026

Scope: provider access/status and bounded literature search in the shared Rust
runtime

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

Move the accepted Rust Lite provider clients, normalization, multi-provider
search, and deterministic search planning into `qiongli-runtime`. Keep the old
`qiongli-lite-mcp` package as a compatibility consumer through thin adapters.

The native 2.x settings model remains authoritative. R2B does not copy the old
Lite plaintext `providers.json` model into the native platform. The old config
resolver remains only at the compatibility edge and converts resolved values
into the shared runtime's in-memory provider-access type.

This batch establishes reusable provider behavior but does not expose a public
MCP mode or provider-management command in the canonical executable.

## Dependency And Ownership Boundary

`qiongli-runtime` owns:

- canonical provider identities and aliases;
- redacted provider availability/status projection;
- bounded search request validation;
- HTTP policy, provider requests, response decoding, and normalization;
- multi-provider fan-out, diagnostics, deduplication, and final bounds;
- cooperative cancellation at request boundaries; and
- deterministic search-plan generation.

`qiongli-config` continues to own native settings persistence, secret
references, and the `SecretStore` boundary. The runtime may optionally adapt
those settings through a `native-config` feature. The feature-light old Lite
consumer does not pull embedded content or native persistence into its binary.

The intended dependency direction is:

```text
qiongli-config --optional--> qiongli-runtime --> apps/qiongli
                                  ^
                                  |
                       qiongli-lite-mcp adapters
```

No dependency points from the native workspace back to the old Lite package.

## Provider Access And Status

The shared registry freezes these provider identities and execution order:

1. OpenAlex;
2. Semantic Scholar;
3. Crossref;
4. PubMed; and
5. arXiv.

Aliases are parsed once in the runtime. Access values are process-local,
non-serializable, and must not implement a secret-revealing `Debug`. Secret
strings use zeroizing storage. Public status contains only provider identity,
enabled state, and one of these readiness states:

- `disabled`;
- `ready`;
- `needs_secret`;
- `needs_public_setting`; or
- `secret_store_unavailable`.

When native settings contain a secret reference, the runtime resolves it only
through the injected `SecretStore`. It does not read environment variables or
fall back to plaintext. Missing and unavailable secret storage remain distinct
redacted outcomes. The compatibility adapter may receive already-resolved
legacy values, but that behavior stays outside the native configuration path.

Errors and serialized status must never contain secret values, secret
references, request headers, query text, response bodies, or endpoint values.

## Bounded Search Contract

The canonical request validates before network work:

| Field | Contract |
|---|---|
| query | trimmed, non-empty UTF-8, at most 4,096 bytes |
| mode | `auto`, `topic`, `review`, or `systematic_review` |
| providers | canonical identities or accepted aliases; no duplicates |
| per-provider limit | 1 through 200 |
| total limit | 1 through 1,000 |

The compatibility `SearchInput` surface is retained in the shared crate for
old call sites, including its historical limit-clamping behavior, but it feeds
the same provider implementations and orchestration path. It is not a second
provider kernel.

Search runs configured providers concurrently, keeps canonical output order,
reports failures per provider, deduplicates DOI first and normalized
title-plus-year second, and applies the total bound after deduplication. A
partial result is successful with diagnostics; failure of every attempted
provider remains a typed search failure.

## Network And Cancellation Policy

Production endpoints remain compile-time constants. Endpoint injection is
available only through a hidden test/compatibility constructor and is never
accepted from MCP input, config files, or environment variables.

The shared HTTP client freezes the already accepted Lite protections:

| Boundary | Value |
|---|---:|
| connect timeout | 3 seconds |
| request timeout | 15 seconds |
| maximum response body | 4 MiB |
| redirects | disabled |
| implicit system/environment proxy discovery | disabled |

HTTP, decode, invalid-endpoint, timeout, transport, and cancellation failures
are typed and sanitized. Provider credentials use only their provider-specific
header or query placement and never appear in diagnostics.

Proxy routing can be added later only through an explicit typed and redacted
native setting. R2B does not allow process environment to redirect credentialed
provider requests.

TLS uses bundled Rustls roots on non-Windows targets and the operating
system's SChannel through `native-tls` on Windows. This keeps Windows binaries
free of a user-installed TLS runtime and keeps the declared Windows
cross-target gate buildable without importing a foreign C SDK.

Cancellation is cooperative. The runtime checks a shared cancellation token
before scheduling work, before and after each request, and between PubMed's
search and summary calls. Blocking HTTP already in flight is not forcibly
aborted; it remains bounded by the 15-second request timeout. R2B makes no
stronger cancellation claim.

## Compatibility Extraction

The provider and search modules in `qiongli-lite-mcp` become re-exports or
small constructors over `qiongli-runtime`. Existing focused tests remain on
the old public paths and prove request shape, auth placement, redirect policy,
timeouts, response bounds, parsers, ordering, deduplication, partial failure,
and MCP-facing behavior.

Legacy provider config persistence, setup wizard behavior, and environment
aliases stay in the old package for now. Only the resolved in-memory access
values cross into the shared kernel.

## CI And Verification

The native workspace must pass format, locked check, strict Clippy, all-target
tests, and the Windows MSVC cross-target check. Native unit tests cover typed
identities, request bounds, secret-store adaptation, redaction, cancellation,
normalization, deduplication, and search planning without live provider calls.

The parallel Linux Lite compatibility job expands to the focused provider
HTTP, parser, search-orchestration, literature-planning, and search-plan tests.
No test depends on live scholarly services. Python and Node suites remain out
of scope for the accelerated migration gate.

## Nonclaims

R2B does not implement or claim:

- a canonical `qiongli mcp serve` mode or native initialize/tools calls;
- native provider config CLI/UI or a production secure-store implementation;
- import of plaintext legacy provider secrets into native settings;
- immediate interruption of blocking HTTP already in flight;
- evidence export, Zotero, orchestrator execution, install integration, UI,
  packaging, or release readiness; or
- completion of R2.

## Acceptance Criteria

R2B is complete only when:

1. `qiongli-runtime` owns the only provider HTTP, parser, normalization,
   orchestration, deduplication, and search-plan implementations used by Rust;
2. native settings adapt through `SecretStore` without plaintext fallback;
3. public status and every provider failure remain credential-free;
4. canonical requests reject every declared invalid bound before networking;
5. timeouts, response limits, redirect denial, and cooperative cancellation are
   tested;
6. the old Lite provider modules are compatibility adapters over the shared
   runtime;
7. focused old Lite behavior remains green without live services;
8. local native and Windows gates pass without Python or Node; and
9. exact-head GitHub jobs pass before the roadmap records completion.

## Approval Record

The user instructed continuation under the accepted accelerated roadmap on
July 14, 2026. This authorizes R2B on the existing rolling branch and Draft PR
without expanding the product's public availability claims.
