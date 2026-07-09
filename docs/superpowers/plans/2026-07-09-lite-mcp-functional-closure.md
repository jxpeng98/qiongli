# Lite MCP Functional Closure Implementation Plan

> **For agentic workers:** Execute this plan task by task with tests-first
> development. Keep each commit independently reviewable, but do not merge or
> release the branch until the complete Gate 0 suite is green.

**Goal:** Make every Marketplace Lite MCP capability advertised in
`content/mcp-contracts/lite-tools.json` dispatchable, behaviorally real,
secret-safe, and covered by automated tests before Qiongli builds the unified
platform control plane.

**Roadmap:**
`docs/superpowers/roadmaps/2026-07-09-unified-platform-roadmap.md`

**Release target:** `v1.18.0-beta.3`.

**Architecture:** Keep Rust Lite and Python Full as separate runtime profiles.
Refactor Rust Lite around injectable runtime dependencies so provider and
Companion behavior can be tested against loopback mock servers without live
network access. Preserve existing MCP tool names and public status vocabulary.
Use shared semantic projections for Gate 0 conformance; common response-envelope
design remains Capability Contract v2 work.

**Tech stack:** Rust, Cargo, `serde`, `serde_json`, blocking `reqwest` with
`rustls-tls`, `quick-xml`, loopback HTTP fixtures, Python 3.12 `unittest`, stdio
JSON-RPC MCP, existing contract fixtures, and the legacy Node MCPB suite as a
behavioral reference only.

## Execution Status — July 9, 2026

Implementation Tasks 1 through 10 and the complete local Gate 0 acceptance are
complete on `dev`. The result is ready for review, but it is not a published
release: no commit, tag, release, or external marketplace mutation is part of
this execution turn. The separate `v1.18.0-beta.3` release task must still bump
the product version, generate the release receipt, and enforce an honest native
target policy before publication.

Implemented evidence so far:

- all twelve contract tools have handlers and committed safe-call fixtures;
- Rust fmt and Clippy pass with warnings denied;
- the Rust `--all-targets` suite passes provider, orchestration, wizard,
  protocol, Zotero, and preview tests;
- Lite/Full contract, redaction, input-error, evidence, and task-identity
  projections pass;
- legacy Node reference tests pass 119/119;
- packaging, target identity, real plugin startup, twelve-call smoke, release
  index, and release automation tests pass 97/97;
- the complete Python regression suite passes 1316/1316;
- materialized-distribution auditing and the maintainer beta smoke pass;
- the strict validator reports 6203 passed, 0 failed, and 0 warnings;
- the documentation build and quick release-preflight path pass.

Local Gate 0 status: **ready for review**. Public release status: **not ready**
until the release-specific version, receipt, and native-target decisions above
are completed. A generic cross-platform native claim remains prohibited; the
full native build, signing, provenance, and install-time variant matrix belongs
to Roadmap Stage 4.

---

## Verified Starting State

The following was verified against `dev` on July 9, 2026:

- The Rust test suite passes thirteen existing integration tests.
- Rust Clippy passes with warnings denied.
- Rust formatting check currently fails on existing formatting drift.
- Selected Python contract and artifact tests pass but do not call every Lite
  tool or execute provider search.
- The legacy Node MCPB reference suite passes 119 tests and covers request
  shapes, PubMed two-step search, limits, deduplication, partial failure,
  redacted errors, wizard security, Zotero probing, and multiple stdio framing
  forms.
- Direct calls to the current Rust binary show:
  - `qiongli_configure_provider` returns JSON-RPC `-32601`;
  - `qiongli_open_config_wizard` returns JSON-RPC `-32601`;
  - `qiongli_literature_search` returns `status: ok`, empty results, and
    `diagnostics.status: not_run`;
  - `qiongli_zotero_status` returns a fixed `fallback_only` response without a
    network probe;
  - saving `openalex.email` returns `unsupported provider field`.
- Plugin and MCPB builders package the current-host binary while source
  compatibility metadata claims more than one operating system.

These passing structural tests are not release evidence for functional closure.

## Gate 0 Decisions

The following decisions are fixed for this plan:

1. Implement `qiongli_configure_provider` and
   `qiongli_open_config_wizard`; do not withdraw them from the Lite contract.
2. Preserve all current MCP tool names and the provider config file at
   `QIONGLI_CONFIG_HOME/providers.json`.
3. Preserve the Zotero public status vocabulary:
   `ok`, `companion_missing`, `fallback_only`, and `disabled`.
4. Preserve Python Full top-level search status semantics for shared behavior:
   `ok`, `warning`, and `error`. A partial provider failure uses top-level
   `warning` plus `diagnostics.status: partial`.
5. Compare Lite and Full through a shared semantic projection in Gate 0.
   Do not require identical outer envelopes.
6. Normalize optional record fields by omitting absent values in the shared
   projection. Do not freeze cross-runtime `null` behavior until Capability
   Contract v2.
7. General searches default to 25 results per provider. Review and systematic
   review searches default to 50. Explicit per-provider limits are bounded to
   1 through 200. `total_limit` applies after deduplication.
8. `limit` remains a compatibility alias for `per_provider_limit`.
9. Gate 0 supports the Lite arguments explicitly listed in the revised tool
   contract. Advanced domain-specific deep search, citation expansion, and
   review-grade coverage diagnostics remain Full or legacy-reference behavior.
10. Unsupported Lite arguments must be rejected or removed from public docs;
    they must not be silently ignored.
11. Lite does not add Zotero search, collection management, item writes, local
    agent launch, arbitrary shell execution, or project guidance writes.
12. Provider and Zotero endpoint overrides are injected by tests, not accepted
    from MCP tool arguments. Production provider endpoints stay internal to the
    binary to avoid an arbitrary-URL or SSRF surface.
13. The full five-target build and signing matrix remains a later roadmap stage,
    but `beta.3` must stop presenting an unidentified current-host binary as a
    generic multi-platform artifact.

## Target Lite Behavior

### Tool Surface

| Tool | Gate 0 behavior | Side effect |
|---|---|---|
| `qiongli_config_status` | Report config path, redacted provider status, capability mode, missing fields, and next action | Read local config/env |
| `qiongli_save_provider_config` | Save one supported field and return path/status without echoing the value | Local config write |
| `qiongli_configure_provider` | Start a tokenized loopback setup page and return its URL | Loopback listener, optional config write |
| `qiongli_open_config_wizard` | Compatibility alias of `qiongli_configure_provider` | Same as alias target |
| `qiongli_literature_status` | Report configured providers and implemented provider capabilities | Read config/env |
| `qiongli_search_plan` | Return provider/native routing plan without performing native search | Read config/env |
| `qiongli_literature_search` | Execute configured provider calls, normalize, deduplicate, diagnose, and limit results | Provider network calls |
| `qiongli_literature_export_evidence` | Return an auditable snapshot of supplied search evidence | No filesystem write |
| `qiongli_zotero_status` | Probe loopback Connector and Companion and report existing public status vocabulary | Loopback network calls |
| `qiongli_zotero_export_import_files` | Return CSL JSON, RIS, BibTeX, and import report content | No direct Zotero write |
| `qiongli_orchestrator_route` | Return preview-only routing and an explicit Full upgrade recommendation for execution | None |
| `qiongli_task_plan` | Return a preview-only task packet without agents or project writes | None |

### Provider Configuration

| Provider | Activation field | Optional fields | No-credential behavior |
|---|---|---|---|
| OpenAlex | `api_key` | `email` | Report missing; do not call |
| Semantic Scholar | `api_key` | none | Report missing; do not call |
| Crossref | `email` | none | Report missing in the current shared contract; do not call |
| PubMed | `api_key` | none | Report missing in the current shared contract; do not call |
| arXiv | none | none | Enabled and configured by default |

OpenAlex email alone must not mark OpenAlex configured. The Rust resolver must
support the same standard, legacy, and MCPB environment aliases used by Python
Full for the fields in this table.

### Literature Search

- Construct one bounded HTTP client with connect and total request timeouts.
- Disable redirects for credential-bearing provider and Zotero requests. A 3xx
  response becomes a sanitized provider diagnostic rather than forwarding an
  API key across origins.
- Execute configured providers concurrently or with another bounded strategy;
  return results in deterministic provider and record order.
- Use OpenAlex `api_key` and optional `mailto` query values.
- Use Semantic Scholar `x-api-key` header.
- Use Crossref `mailto` query value.
- Implement PubMed as `esearch.fcgi` followed by `esummary.fcgi`.
- Use the arXiv Atom search endpoint without credentials.
- Deduplicate by normalized DOI first. Fall back to normalized title plus year
  when DOI is missing.
- Merge the `providers` list deterministically and preserve one stable primary
  `provider`.
- Apply `total_limit` after deduplication.
- Sanitize diagnostics into stable kinds such as `timeout`, `http_error`,
  `decode_error`, and `transport_error`. Never serialize a credential-bearing
  request URL.
- Return successful provider results even when another provider fails.

### Configuration Wizard

- Bind only to `127.0.0.1`; accept `localhost` as an input alias and normalize
  it to `127.0.0.1`.
- Return the URL to the MCP caller. Do not require or promise automatic browser
  launch.
- Generate an unguessable one-time token with an operating-system random source.
- Require the token on GET and POST; invalid or absent tokens return 403.
- Limit request and form body sizes.
- Save through the same provider config module used by MCP config tools.
- Stop after a successful save and expire an unused session after a bounded
  timeout.
- Never put provider values in the URL, tool output, stdout, stderr, or access
  logs.

### Zotero Status

- Read the existing local-enabled and connector URL settings.
- Accept only loopback HTTP(S) endpoints; do not follow redirects.
- Probe the Zotero Connector endpoint and then `/qiongli/ping` with a short
  timeout.
- Return:
  - `disabled` when the local bridge is explicitly disabled;
  - `fallback_only` when Zotero is not reachable;
  - `companion_missing` when Zotero responds but Qiongli Companion does not;
  - `ok` when the Qiongli Companion responds successfully.
- Keep import-file fallback available in every state except a future explicitly
  unsupported environment.
- Return only allowlisted Companion metadata such as version; do not forward
  arbitrary local response fields.

## File Plan

### Canonical contracts and fixtures

- Modify `content/mcp-contracts/lite-tools.json`.
- Modify `content/mcp-contracts/provider-config.schema.json`.
- Modify `content/mcp-contracts/literature-result.schema.json` only after the
  optional-field policy is tested.
- Modify `content/mcp-contracts/literature-diagnostics.schema.json` to match the
  shared semantic projection rather than a runtime-specific envelope.
- Create `content/mcp-contracts/fixtures/lite-tool-smoke-calls.json`.
- Create `content/mcp-contracts/fixtures/expected-search-response.json`.
- Create provider error and partial-failure fixtures as needed.

### Rust Lite runtime

- Modify `packages/qiongli-lite-mcp/Cargo.toml` and `Cargo.lock`.
- Create `packages/qiongli-lite-mcp/src/mcp/protocol.rs`.
- Modify `packages/qiongli-lite-mcp/src/mcp/mod.rs`.
- Modify `packages/qiongli-lite-mcp/src/mcp/server.rs`.
- Create `packages/qiongli-lite-mcp/src/config/wizard.rs`.
- Modify `packages/qiongli-lite-mcp/src/config/mod.rs`.
- Modify `packages/qiongli-lite-mcp/src/config/provider_config.rs`.
- Modify `packages/qiongli-lite-mcp/src/providers/search.rs`.
- Create `packages/qiongli-lite-mcp/src/providers/runtime.rs` or an equivalent
  dependency-injection boundary.
- Modify all five files under `packages/qiongli-lite-mcp/src/providers/`.
- Modify `packages/qiongli-lite-mcp/src/searchplan.rs`.
- Modify `packages/qiongli-lite-mcp/src/zotero/companion.rs`.
- Modify `packages/qiongli-lite-mcp/src/orchestrator/preview.rs`.
- Modify `packages/qiongli-lite-mcp/src/main.rs`.

### Tests and validation

- Modify existing Rust tests under `packages/qiongli-lite-mcp/tests/`.
- Create `packages/qiongli-lite-mcp/tests/mcp_protocol.rs`.
- Create `packages/qiongli-lite-mcp/tests/provider_http.rs`.
- Create `packages/qiongli-lite-mcp/tests/search_orchestration.rs`.
- Create `packages/qiongli-lite-mcp/tests/config_wizard.rs`.
- Modify `tests/test_lite_mcp_contract.py`.
- Modify `tests/test_lite_full_mcp_parity.py`.
- Modify `tests/test_mcp_contract_fixtures.py`.
- Create `tests/test_lite_mcp_behavior_contract.py` as a
  `unittest.TestCase` suite.
- Modify `tests/test_literature_mcpb_artifact.py`.
- Modify `tests/test_plugin_distribution_contract.py`.
- Modify `tooling/scripts/validate_marketplace_install.py`.
- Modify `.github/workflows/ci.yml`.
- Modify release preflight scripts only if CI and local release entrypoints would
  otherwise enforce different Rust gates.

### Packaging and documentation

- Modify `tooling/scripts/build_lite_mcp.py`.
- Modify `tooling/scripts/build_plugin_artifacts.py`.
- Modify `tooling/scripts/build_literature_mcpb.py`.
- Modify release artifact metadata generation for current-host identity.
- Modify `packages/qiongli-literature-mcpb/manifest.json` through
  runtime-specific staging overlays where Lite and legacy Node metadata differ.
- Modify `packages/qiongli-literature-mcpb/README.md`.
- Modify `docs/advanced/cross-platform-mcp.md`.
- Modify `docs/advanced/mcp-zotero-integration.md`.
- Modify relevant install docs, `README.md`, and `README_CN.md`.
- Update release notes and acceptance receipts only during the release task.

Do not edit generated workflow, plugin, npm payload, or Python payload trees.

## Task Dependency Graph

```text
Task 1: Formatting and truth-test baseline
       |
       v
Task 2: Injectable runtime dependencies and provider config
       |                         \
       v                          v
Task 3: Provider HTTP clients    Task 6: Config wizard
       |                          |
       v                          |
Task 4: Search orchestration      |
       |                          |
       v                          |
Task 5: MCP search behavior <-----+
       |
       +------> Task 7: Zotero probe and preview truth
       |                          |
       v                          v
Task 8: Cross-runtime conformance and dispatcher smoke
       |
       v
Task 9: CI, validators, and artifact truth
       |
       v
Task 10: Documentation, version map, and release acceptance
```

## Suggested Execution Cadence

This is an illustrative ten-engineering-day sequence for two implementers plus
one reviewer. Gate completion depends on evidence, not the calendar.

| Days | Runtime track | Contract/release track | Review checkpoint |
|---|---|---|---|
| 1-2 | Tasks 1-2: formatting, runtime dependencies, config | Safe-call fixtures and Python RED tests | Config and dependency-injection design |
| 3-4 | Task 3: five provider request behaviors | Task 6 wizard tests and security review | Auth placement, redirects, PubMed two-step |
| 5-6 | Task 4: orchestration, dedupe, limits | Task 7 Zotero and preview tests | Partial-failure and loopback boundaries |
| 7 | Task 5: MCP search wiring | Contract projection and schema corrections | No false-success responses |
| 8 | Task 8: stdio and cross-runtime conformance | Safe-call binary harness | All twelve tools dispatchable |
| 9 | Task 9: CI, validator, artifact identity | Release-policy and legacy overlay tests | CI/local preflight equivalence |
| 10 | Task 10: full acceptance and docs | Version map and receipt | Gate 0 go/no-go |

With one implementer, execute the dependency graph sequentially. With three or
more implementers, do not parallelize Tasks 3-5 across incompatible server or
config abstractions; first merge Task 2's runtime dependency boundary.

## Task 1: Normalize Rust Formatting And Lock Failing Truth Tests

**Files:**

- Modify Rust source and tests mechanically with `cargo fmt`.
- Modify `tests/test_lite_mcp_contract.py`.
- Modify `tests/test_mcp_contract_fixtures.py`.
- Create `tests/test_lite_mcp_behavior_contract.py`.
- Create `content/mcp-contracts/fixtures/lite-tool-smoke-calls.json`.

- [ ] **Step 1: Record the baseline before formatting**

Run:

```bash
cargo fmt --manifest-path packages/qiongli-lite-mcp/Cargo.toml -- --check
cargo clippy --manifest-path packages/qiongli-lite-mcp/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml
```

Expected before implementation: fmt fails on existing drift; Clippy and the
thirteen current tests pass.

- [ ] **Step 2: Apply only mechanical Rust formatting**

Run:

```bash
cargo fmt --manifest-path packages/qiongli-lite-mcp/Cargo.toml
```

Review the diff to confirm there are no behavior changes.

- [ ] **Step 3: Add safe-call fixture records for all twelve tools**

Each record in `lite-tool-smoke-calls.json` must include:

- tool name;
- arguments;
- expected response class: success, input error, or bounded local result;
- whether config, network, or a loopback listener is allowed;
- fields that must never appear in serialized output.

Use deliberate invalid input to prove dispatch without causing side effects
where appropriate. Examples:

- config save omits `value` and expects an input error;
- configuration wizard uses a non-loopback host and expects a host-policy error;
- literature search omits `query` and expects an input error;
- task plan omits required fields and expects an input error.

The expected result must distinguish a handler-level input error from JSON-RPC
`-32601 Tool not found`.

- [ ] **Step 4: Add RED behavior tests**

Add `unittest.TestCase` methods that:

- build and call the Rust binary;
- prove both wizard names currently return `-32601`;
- prove a valid search currently returns `not_run` and empty results;
- prove every declared tool has a safe-call fixture;
- prove fixture secrets do not appear in responses.

Do not add top-level pytest-style functions unless the project formally adds
pytest to CI. The command below must actually discover the new tests.

Run:

```bash
python3 -m unittest \
  tests.test_lite_mcp_contract \
  tests.test_mcp_contract_fixtures \
  tests.test_lite_mcp_behavior_contract -v
```

Expected: RED on missing wizard handlers and no-op search.

- [ ] **Step 5: Commit formatting and closure tests together only if branch
      policy permits a red intermediate commit**

Preferred commit after the first implementation task turns the relevant tests
green:

```bash
git commit -m "test(lite-mcp): expose functional closure gaps"
```

## Task 2: Add Injectable Runtime Dependencies And Align Provider Config

**Files:**

- Modify `content/mcp-contracts/provider-config.schema.json`.
- Modify `packages/qiongli-lite-mcp/src/config/provider_config.rs`.
- Create `packages/qiongli-lite-mcp/src/providers/runtime.rs` or equivalent.
- Modify `packages/qiongli-lite-mcp/src/providers/mod.rs`.
- Modify `packages/qiongli-lite-mcp/src/mcp/server.rs`.
- Modify `packages/qiongli-lite-mcp/tests/provider_config.rs`.
- Modify Python config parity tests using `unittest.TestCase`.

- [ ] **Step 1: Add RED provider config tests**

Cover:

- `openalex.email` can be saved and read;
- OpenAlex email alone does not mark OpenAlex configured;
- OpenAlex API key does mark it configured;
- all documented `QIONGLI_MCPB_*` aliases resolve correctly;
- arXiv remains configured without credentials;
- default result limit reads `QIONGLI_MCPB_DEFAULT_LIMIT` and clamps invalid
  values according to the Gate 0 decision;
- redacted summaries never contain raw values;
- Unix writes remain mode `0600`;
- Rust and Python share activation-versus-optional field semantics.

Use isolated `QIONGLI_CONFIG_HOME` and serialize environment-mutating Rust tests
with the existing mutex pattern.

- [ ] **Step 2: Introduce an internal runtime config type**

Add a non-serializable internal representation for provider values and search
defaults. Do not derive `Debug` or `Serialize` for a structure that contains
credentials.

Separate:

- supported fields;
- activation-required fields;
- optional request metadata;
- environment aliases;
- redacted status.

- [ ] **Step 3: Add injectable runtime dependencies**

Introduce a `RuntimeDeps` structure or equivalent containing:

- bounded HTTP client;
- immutable provider endpoint set;
- config resolver;
- clock/random/session helpers where tests require determinism.

`McpServer::new()` uses production dependencies. Tests construct a server with
loopback endpoints. Do not expose provider endpoint override arguments in MCP
schemas.

- [ ] **Step 4: Build the shared HTTP client safely**

Configure connect and total timeouts, a stable user agent, response size bounds
where practical, and redirect denial. Ensure formatted errors cannot include
credential-bearing query strings or header values.

- [ ] **Step 5: Run GREEN**

```bash
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --test provider_config

python3 -m unittest \
  tests.test_lite_mcp_behavior_contract \
  tests.test_mcp_tool_handlers -v
```

Keep new parity cases in `tests/test_lite_mcp_behavior_contract.py`. The
existing top-level pytest-style functions in `tests/test_provider_config.py`
are not discovered by this `unittest` command and must not be used as evidence
for Gate 0 unless pytest is formally added to CI.

- [ ] **Step 6: Commit**

```bash
git commit -m "fix(lite-mcp): align provider configuration behavior"
```

## Task 3: Complete Provider HTTP Request Behavior

**Files:**

- Modify all five Rust provider client files.
- Modify `packages/qiongli-lite-mcp/src/providers/search.rs`.
- Create `packages/qiongli-lite-mcp/tests/provider_http.rs`.
- Add request and response fixtures under `content/mcp-contracts/fixtures/` only
  when they describe shared behavior.

- [ ] **Step 1: Add a loopback mock HTTP harness**

Use an in-process Rust test server or a dev-only mock HTTP dependency. Tests must
run without public network access and must capture method, path, query, headers,
and body.

- [ ] **Step 2: Add RED request-shape tests**

Cover:

- OpenAlex path, query, API key, optional mailto, and result limit;
- Semantic Scholar path, fields, limit, and `x-api-key`;
- Crossref path, query, rows, and mailto;
- PubMed ESearch term/retmax/API key followed by ESummary IDs/API key;
- arXiv search query and max results;
- percent encoding and Unicode query text;
- non-2xx responses;
- redirect refusal;
- timeout;
- malformed JSON and XML;
- sanitized errors with test credentials absent.

- [ ] **Step 3: Implement request behavior**

Keep parsing functions independently testable with committed fixtures. Convert
transport failures to stable internal error kinds before they reach MCP output.

For PubMed, do not pass an ESearch response to the existing ESummary parser.
Implement the two calls and an empty-ID fast path.

- [ ] **Step 4: Use the Node suite as an oracle, not a dependency**

Review the equivalent passing Node tests, select the shared Marketplace Lite
subset, and express those cases as Rust tests or shared fixtures. Do not call
Node from the Rust runtime and do not make Node files canonical contracts.

- [ ] **Step 5: Run GREEN**

```bash
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --test providers --test provider_http

npm --prefix packages/qiongli-literature-mcpb test
```

Expected: Rust request tests and the legacy reference suite pass without live
provider calls.

- [ ] **Step 6: Commit**

```bash
git commit -m "fix(lite-mcp): complete provider HTTP clients"
```

## Task 4: Implement Deterministic Search Orchestration

**Files:**

- Modify `packages/qiongli-lite-mcp/src/providers/search.rs`.
- Modify `packages/qiongli-lite-mcp/src/providers/mod.rs`.
- Modify the runtime dependency boundary.
- Create `packages/qiongli-lite-mcp/tests/search_orchestration.rs`.
- Create `content/mcp-contracts/fixtures/expected-search-response.json`.

- [ ] **Step 1: Add RED orchestration tests**

Cover:

1. configured providers execute;
2. unconfigured providers do not execute;
3. arXiv executes without credentials;
4. calls are bounded and a slow provider cannot block indefinitely;
5. output ordering remains deterministic even if calls complete out of order;
6. `limit` and `per_provider_limit` resolve to the same per-provider behavior;
7. general default is 25 and review default is 50;
8. explicit per-provider limits clamp to 1 through 200;
9. `total_limit` applies after deduplication;
10. equal normalized DOI records merge;
11. missing-DOI title/year records merge;
12. merged records contain deterministic unique `providers`;
13. one failed provider yields top-level `warning` and diagnostic `partial`;
14. all failed providers yield top-level `error` and diagnostic `failed`;
15. errors and serialized output contain no configured secrets.

- [ ] **Step 2: Define internal output and diagnostics types**

Represent provider results and diagnostics separately. Diagnostics must include
one record per attempted provider with status, count, and a sanitized error kind
or warning. Do not keep the current split where `provider_counts` contradicts
the committed diagnostics schema.

- [ ] **Step 3: Implement bounded fan-out**

Use threads or an equivalent bounded blocking strategy suitable for the current
runtime. Stable output order must be independent of completion order.

- [ ] **Step 4: Implement deduplication and limits**

Normalize DOI prefixes and case. Normalize fallback titles without erasing
meaningful Unicode. Prefer non-empty metadata deterministically and merge
provenance.

- [ ] **Step 5: Run GREEN**

```bash
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --test search_orchestration
```

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(lite-mcp): orchestrate literature search"
```

## Task 5: Wire Search Into MCP And Enforce Real Tool Results

**Files:**

- Modify `content/mcp-contracts/lite-tools.json`.
- Modify result and diagnostics contracts or add Gate 0 projection fixtures.
- Modify `packages/qiongli-lite-mcp/src/mcp/server.rs`.
- Modify `packages/qiongli-lite-mcp/tests/mcp_server.rs`.
- Modify `tests/test_lite_mcp_contract.py`.
- Modify `tests/test_lite_mcp_behavior_contract.py`.

- [ ] **Step 1: Freeze the supported Lite search arguments**

List every supported argument explicitly. Set `additionalProperties` according
to the implemented validation behavior. Include only arguments whose semantics
are tested. Unsupported advanced arguments must not be documented as working.

- [ ] **Step 2: Add RED MCP handler tests**

With injected loopback providers, call `qiongli_literature_search` and require:

- a non-empty normalized result;
- an executed search plan;
- real provider diagnostics;
- no `not_run` success;
- correct `ok`, `warning`, and `error` behavior;
- invalid or unsupported arguments return a structured tool error rather than
  a protocol crash.

- [ ] **Step 3: Replace `empty_search_output()` in the handler**

Call the Task 4 orchestrator through injected runtime dependencies. Do not keep
an unused `SearchInput` merely to build a plan.

- [ ] **Step 4: Define a Gate 0 semantic projection**

For conformance, project runtime responses to:

- status class;
- search plan routing fields;
- provider diagnostic status/count/error kind;
- normalized records with absent optional fields removed;
- deduplication provenance.

Do not require Rust, Python, and legacy Node outer envelopes to match.

- [ ] **Step 5: Run GREEN**

```bash
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --test mcp_server --test search_orchestration

python3 -m unittest \
  tests.test_lite_mcp_contract \
  tests.test_lite_mcp_behavior_contract -v
```

- [ ] **Step 6: Commit**

```bash
git commit -m "fix(lite-mcp): execute provider search from MCP"
```

## Task 6: Implement The Tokenized Loopback Configuration Wizard

**Files:**

- Modify `packages/qiongli-lite-mcp/Cargo.toml` and `Cargo.lock`.
- Create `packages/qiongli-lite-mcp/src/config/wizard.rs`.
- Modify `packages/qiongli-lite-mcp/src/config/mod.rs`.
- Modify `packages/qiongli-lite-mcp/src/mcp/server.rs`.
- Create `packages/qiongli-lite-mcp/tests/config_wizard.rs`.
- Modify Python black-box tests.

- [ ] **Step 1: Add RED wizard tests**

Cover:

- non-loopback hosts are rejected;
- `localhost` normalizes to `127.0.0.1`;
- port zero selects an available local port;
- the returned URL contains a random token but no secret;
- missing and incorrect tokens receive 403;
- correct GET returns a form for only supported fields;
- oversized POST receives an error;
- correct POST saves values through provider config;
- the session stops after save;
- an unused session expires;
- neither stdout, stderr, HTML confirmation, nor MCP result contains the saved
  secret;
- both MCP tool names call the same implementation.

- [ ] **Step 2: Add the minimal dependencies**

Use an OS-backed random source and a small loopback HTTP implementation. Review
new dependencies for license, binary-size effect, and unsupported platform
features. Avoid a general web framework.

- [ ] **Step 3: Implement a background session lifecycle**

The MCP tool returns immediately with a URL while a bounded background listener
serves the form. Ensure repeated calls create isolated sessions and process
shutdown cannot hang on wizard threads.

- [ ] **Step 4: Centralize dispatch truth**

Refactor tool lookup into a central handler registry or equivalent mapping.
Add a Rust test asserting every name loaded from `lite-tools.json` resolves to a
handler, including compatibility aliases.

- [ ] **Step 5: Run GREEN**

```bash
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --test config_wizard --test mcp_server

python3 -m unittest \
  tests.test_lite_mcp_contract \
  tests.test_lite_mcp_behavior_contract -v
```

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(lite-mcp): add loopback provider setup wizard"
```

## Task 7: Probe Zotero Honestly And Keep Preview Tools Bounded

**Files:**

- Modify `packages/qiongli-lite-mcp/src/zotero/companion.rs`.
- Modify `packages/qiongli-lite-mcp/src/mcp/server.rs`.
- Modify `packages/qiongli-lite-mcp/src/orchestrator/preview.rs`.
- Modify `packages/qiongli-lite-mcp/tests/zotero_companion.rs`.
- Modify `packages/qiongli-lite-mcp/tests/orchestrator_preview.rs`.

- [ ] **Step 1: Add RED Zotero tests**

Using a loopback fake server, cover:

- local bridge disabled;
- Zotero unreachable;
- Connector succeeds but Companion is missing;
- Connector and `/qiongli/ping` both succeed;
- non-loopback URL rejection;
- redirect rejection;
- timeout;
- malformed response;
- allowlisted Companion version metadata;
- import-file fallback in all supported states.

- [ ] **Step 2: Implement loopback-only probes**

Reuse bounded HTTP policy without exposing provider credentials. Preserve
`ok`, `companion_missing`, `fallback_only`, and `disabled` exactly.

- [ ] **Step 3: Add preview truth tests**

Require route and task-plan responses to include:

- `mode: preview`;
- `runtime_profile: marketplace_lite`;
- `run_agents_allowed: false`;
- `project_writes_allowed: false`;
- a structured Full upgrade recommendation for execution requests;
- no shell command or agent launch behavior.

Validate required task-plan inputs and avoid implying that a static plan was
executed.

- [ ] **Step 4: Run GREEN**

```bash
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --test zotero_companion --test orchestrator_preview --test mcp_server
```

- [ ] **Step 5: Commit**

```bash
git commit -m "fix(lite-mcp): report Zotero and preview capabilities honestly"
```

## Task 8: Add Stdio, Dispatcher, And Cross-Runtime Conformance Gates

**Files:**

- Create `packages/qiongli-lite-mcp/src/mcp/protocol.rs`.
- Modify `packages/qiongli-lite-mcp/src/main.rs`.
- Create `packages/qiongli-lite-mcp/tests/mcp_protocol.rs`.
- Modify `tests/test_lite_full_mcp_parity.py`.
- Modify `tests/test_mcp_contract_fixtures.py`.
- Modify `tests/test_lite_mcp_behavior_contract.py`.

- [ ] **Step 1: Add host-framing tests**

Use the existing plugin and MCPB host expectations to test newline-delimited
JSON-RPC and any Content-Length framing still required by supported direct
Desktop/MCPB hosts. Define and test response framing for each supported input
form. Do not claim a framing mode that is not exercised by a supported client
or acceptance fixture.

- [ ] **Step 2: Execute every safe-call fixture**

Run all twelve tools through MCP. Assert:

- no declared tool returns `-32601`;
- input errors are tool-handler errors with the expected error kind;
- successful responses contain MCP `content` and `structuredContent`;
- forbidden side effects do not occur;
- no canary secret occurs in output.

- [ ] **Step 3: Replace name-only parity with semantic conformance**

Keep the existing name subset assertion, then add shared projections for:

- provider config status and optional/required field semantics;
- search-plan routing modes;
- evidence-export record count and provenance;
- normalized search record required fields;
- redaction and error classes;
- preview task-plan identity.

Treat Rust/Python outer envelope differences as expected until Stage 1. Do not
force Node's nullable record fields or diagnostics array shape into the shared
projection.

- [ ] **Step 4: Add real schema validation support for tests**

If a third-party JSON Schema validator is added, make it a test extra and ensure
CI installs that extra. Do not add a runtime dependency merely for tests.
Validate only schemas that match the Gate 0 projection; update misleading
schemas before enforcing them.

- [ ] **Step 5: Run GREEN**

```bash
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml --all-targets

python3 -m unittest \
  tests.test_lite_mcp_contract \
  tests.test_lite_mcp_behavior_contract \
  tests.test_lite_full_mcp_parity \
  tests.test_mcp_contract_fixtures \
  tests.test_mcp_tool_surface_parity -v
```

- [ ] **Step 6: Commit**

```bash
git commit -m "test(mcp): enforce Lite functional conformance"
```

## Task 9: Enforce Rust CI, Marketplace Calls, And Artifact Truth

**Files:**

- Modify `.github/workflows/ci.yml`.
- Modify release preflight scripts where necessary.
- Modify `tooling/scripts/validate_marketplace_install.py`.
- Modify `tests/test_plugin_distribution_contract.py`.
- Modify `tests/test_lite_mcp_binary_artifacts.py`.
- Modify `tooling/scripts/build_lite_mcp.py`.
- Modify plugin and MCPB builders.
- Modify release artifact metadata tests.

- [ ] **Step 1: Add a dedicated Rust quality job**

Pin or document the Rust toolchain and run:

```bash
cargo fmt --manifest-path packages/qiongli-lite-mcp/Cargo.toml -- --check
cargo clippy --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --all-targets --locked -- -D warnings
cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --all-targets --locked
cargo build --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --release --locked
```

Run at least `cargo test --locked` on Windows so wizard, path, permissions, and
network code compile and execute outside Linux. Full native artifact coverage
remains Stage 4 of the roadmap.

- [ ] **Step 2: Add Rust gates to local release preflight**

CI and `release_preflight.sh` must not disagree about whether Rust tests are
required for a Lite release. Keep quick and release tiers explicit.

- [ ] **Step 3: Expand marketplace activation from list-only to safe calls**

After `initialize` and `tools/list`, execute the committed safe-call fixtures or
a release-safe subset derived from them. Use an isolated
`QIONGLI_CONFIG_HOME`. Fail if a tool is missing, undispatchable, leaks a canary
secret, or violates its expected response class.

Do not make live provider availability a release gate.

- [ ] **Step 4: Add a current-host identity record**

Until the Stage 4 native matrix exists, stage machine-readable metadata beside
each native binary containing at least:

- Rust target triple;
- operating system and architecture;
- component version;
- binary filename;
- checksum if already available in the release path.

The validator must compare the record to the built executable and artifact
policy.

- [ ] **Step 5: Stop generic multi-platform claims in `beta.3`**

Choose and implement one honest beta policy:

- publish current-target-qualified plugin/MCPB asset identities; or
- scope staged compatibility metadata and release notes to the actual host
  target; or
- do not publish generic native assets until the native matrix exists.

The plan recommends target-qualified beta assets. Do not leave source MCPB
compatibility listing Darwin, Linux, and Windows unchanged in a package that
contains only one current-host executable.

- [ ] **Step 6: Preserve legacy Node manifest behavior separately**

`build_literature_mcpb.py --legacy-node` currently reuses the source manifest.
Use runtime-specific staging overlays or a separate legacy manifest view so
narrowing Rust Lite settings does not accidentally remove legacy Node settings.

- [ ] **Step 7: Run GREEN**

```bash
python3 -m unittest \
  tests.test_plugin_distribution_contract \
  tests.test_lite_mcp_binary_artifacts \
  tests.test_literature_mcpb_artifact \
  tests.test_release_downloads -v

./scripts/release_preflight.sh --quick \
  --materialize-out /tmp/qiongli-gate0-preflight
```

- [ ] **Step 8: Commit**

```bash
git commit -m "ci(lite-mcp): enforce behavior and native artifact truth"
```

## Task 10: Align Documentation, Component Versions, And Release Evidence

**Files:**

- Modify MCP and install documentation listed in the file plan.
- Modify MCPB source/staging metadata and tests.
- Modify the old Rust Lite roadmap status where implementation evidence now
  exists.
- Modify changelog and release notes only for the release commit.
- Modify release acceptance receipt generation or contents as needed.

- [ ] **Step 1: Audit every public Lite claim**

Search for claims about:

- browser configuration;
- five-provider search;
- query variants and domain deep search;
- default and maximum limits;
- deduplication and diagnostics;
- Zotero probing, search, and writes;
- preview versus execution;
- supported operating systems and architectures.

Every remaining claim must point to a passing Gate 0 behavior test. Narrow or
remove claims that belong to Full or legacy Node.

- [ ] **Step 2: Update MCPB metadata through runtime overlays**

The Rust Lite MCPB should expose only settings the Rust runtime consumes. The
legacy Node artifact may retain its own supported settings through a separate
overlay. Tests must prove the two manifests are intentionally different.

- [ ] **Step 3: Record product and component version mapping**

Qiongli product version and Rust/MCPB component versions are currently updated
through different paths. The release artifact manifest and acceptance receipt
must record:

- product version;
- Lite MCP crate version;
- MCPB package version;
- contract schema version;
- native target triple.

Do not silently assume `sync_versions.py` updates the Rust crate or MCPB when it
does not.

- [ ] **Step 4: Run the complete local acceptance suite**

```bash
cargo fmt --manifest-path packages/qiongli-lite-mcp/Cargo.toml -- --check

cargo clippy --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --all-targets --locked -- -D warnings

cargo test --manifest-path packages/qiongli-lite-mcp/Cargo.toml \
  --all-targets --locked

npm --prefix packages/qiongli-literature-mcpb test

python3 -m unittest discover -s tests -v

python3 scripts/materialize_distribution_payloads.py \
  --target all \
  --out /tmp/qiongli-gate0-dist \
  --force

python3 scripts/audit_distribution_payloads.py \
  --root /tmp/qiongli-gate0-dist

python3 scripts/validate_research_standard.py \
  --root /tmp/qiongli-gate0-dist \
  --strict

./scripts/run_beta_smoke.sh
```

- [ ] **Step 5: Run hygiene checks**

- Confirm no secrets, tokens, cookies, local absolute paths, or test config
  values appear in tracked files or staged artifacts.
- Confirm no generated plugin/payload directories were edited as sources.
- Confirm no external marketplace catalog was copied into this repository.
- Confirm the diff contains only intended canonical sources, tests, tooling,
  docs, and release evidence.

- [ ] **Step 6: Commit**

```bash
git commit -m "docs(mcp): record Lite functional closure"
```

## Recommended Commit Sequence

1. `test(lite-mcp): expose functional closure gaps`
2. `fix(lite-mcp): align provider configuration behavior`
3. `fix(lite-mcp): complete provider HTTP clients`
4. `feat(lite-mcp): orchestrate literature search`
5. `fix(lite-mcp): execute provider search from MCP`
6. `feat(lite-mcp): add loopback provider setup wizard`
7. `fix(lite-mcp): report Zotero and preview capabilities honestly`
8. `test(mcp): enforce Lite functional conformance`
9. `ci(lite-mcp): enforce behavior and native artifact truth`
10. `docs(mcp): record Lite functional closure`

Tasks 3 through 5 form one dependency chain. Task 6 may proceed in parallel
after Task 2. Task 7 may proceed in parallel after the bounded HTTP client from
Task 2 exists. Tasks 8 through 10 remain sequential integration gates.

## Review Focus By Commit

| Commit area | Primary review focus |
|---|---|
| Config | Optional versus activation-required fields, aliases, redaction, permissions |
| Provider HTTP | Auth placement, redirect policy, PubMed two-step, sanitized errors |
| Search orchestration | Bounded concurrency, deterministic order, deduplication, limits, partial failures |
| Wizard | Loopback binding, token entropy, TTL, request bounds, thread shutdown, secret handling |
| Zotero | Loopback validation, public status compatibility, timeout, no direct writes |
| Conformance | Tests execute rather than false-green discovery, shared projection boundaries |
| CI and release | Locked Rust build, safe tool calls, honest native target identity |
| Docs | Claims match tests, Lite/Full/legacy boundaries remain explicit |

## Rollback Strategy

- Keep provider config at version 1 and make fields additive so an older runtime
  can read files written by `beta.3`.
- Keep each behavior slice in a separate commit so search, wizard, Zotero, CI,
  and docs can be reverted independently before release.
- If provider execution must be disabled, return an explicit `warning` or
  `error` with `strategy_only` guidance. Never restore an empty successful
  `not_run` response.
- If the wizard cannot meet security gates, remove both wizard names from the
  Lite contract, Rust and MCPB declarations, overlap expectations, and user
  docs in the same rollback. Do not leave listed-but-undispatchable tools.
- If Zotero probing must be disabled, return the existing `disabled` or
  `fallback_only` state with an explicit reason. Do not fabricate observation.
- Preserve the explicit legacy Node artifact as a manual rollback path until
  Rust behavioral parity and native release gates have passed their documented
  retirement window.
- If target-qualified beta packaging cannot be completed, do not publish a
  generic multi-platform native asset.

## Definition Of Done

Gate 0 is complete only when:

- all twelve declared Lite tools resolve to handlers;
- all twelve safe-call fixtures pass through the built MCP binary;
- neither wizard alias returns `-32601`;
- valid fixture-backed literature search calls real provider adapters and never
  report `not_run`;
- all five providers have request-shape, response, timeout, and sanitized-error
  tests;
- PubMed uses ESearch followed by ESummary;
- config fields, activation semantics, environment aliases, and default limits
  match the Gate 0 contract;
- successful, partial, and failed search states are truthful;
- DOI and title/year deduplication, provider provenance, per-provider limits,
  and post-dedup total limits are tested;
- wizard loopback, token, body limit, expiry, shutdown, and redaction tests pass;
- Zotero `ok`, `companion_missing`, `fallback_only`, and `disabled` states come
  from bounded probes or explicit configuration;
- preview tools explicitly prohibit agents, shell execution, and project writes;
- shared Lite/Full provider-config, search-plan, normalized-record, redaction,
  and error projections pass without forcing identical envelopes;
- Rust fmt, Clippy, test, and release build are explicit CI and local preflight
  gates;
- marketplace validation executes safe tool calls, not only `tools/list`;
- every `beta.3` native artifact has an honest target identity and compatibility
  claim;
- product, crate, MCPB, contract, and native target versions are recorded
  together in release evidence;
- legacy Node remains explicitly labeled and its separate manifest behavior is
  preserved;
- full Python MCP and installer regression tests pass;
- docs and manifests make no Lite claim without a matching passing test;
- no secrets, machine-local paths, generated-source edits, or marketplace
  catalog files enter the change.

## Non-Goals

This implementation plan does not include:

- Capability Contract v2 code generation;
- Product or Platform Target v2;
- the unified platform compiler;
- the complete five-target artifact, signing, checksum, provenance, and SBOM
  matrix;
- a Rust rewrite of Python Full;
- a shared Rust provider subprocess for Python Full;
- hosted or remote MCP;
- automatic system-browser launch;
- OS keychain integration or config encryption;
- arbitrary provider endpoint input;
- advanced finance/economics deep search;
- citation/reference graph expansion;
- Zotero local search, collections, notes, or writes;
- local agent or shell execution from Lite;
- changes to canonical academic Task IDs or workflow artifact contracts.
