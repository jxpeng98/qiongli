# Capability Contract v2 Literature Planning Batch

## Goal

Continue Stage 1 after the configuration batch by migrating the two shared,
read-only literature-planning capabilities:

- `qiongli_literature_status`
- `qiongli_search_plan`

The batch makes provider readiness and hybrid search planning truthful across
Marketplace Lite and Full without performing provider or platform-native
search. The rich hybrid plan already exposed by Full is the canonical target;
Lite must converge on that shape instead of reducing Full behavior.

## Baseline Drift

- Literature status shared only `providers` and `capability_mode`; Lite exposed
  simplified `provider_capabilities`, while Full exposed richer `capabilities`
  and omitted `status` and `missing`.
- Lite accepted no `cwd` compatibility context for literature status.
- Lite exposed a six-field search plan while Full exposed provider queries,
  native queries, full-text candidates, provenance, execution order,
  instructions, merge policy, and limitations.
- Full accepted unknown or mistyped search-plan arguments and allowed an empty
  query; the two runtimes advertised different schemas.
- Lite converted provider-config read failures into `strategy_only` instead of
  returning `tool_error`.
- Provider readiness did not distinguish configured-but-disabled providers
  consistently from active providers.

## Contract Work

- [x] Add canonical input and output schemas for both tools.
- [x] Raise registry coverage to `6 / 23` canonical records and `7 / 24`
  public names while keeping the registry in `pilot` mode.
- [x] Record bounded local config reads as the only side effect.
- [x] Mark research queries and generated query routes as sensitive research
  context that may be returned to the caller but must not be copied into
  shared logs automatically.
- [x] Give both tools referenced, identity-checked smoke calls.

## Runtime Alignment

- [x] Accept typed `cwd` for literature status in both runtimes and reject
  unknown arguments.
- [x] Return the same status core, active provider set, activation-field
  status, rich provider-capability schema, and redacted provider state. Each
  runtime reports only capabilities it actually implements; Lite capability
  lists must remain truthful subsets of Full rather than overclaiming parity.
- [x] Advertise the same strict search-plan schema, including bounded legacy
  aliases that normalize to canonical snake-case inputs.
- [x] Preserve the output compatibility window by emitting deprecated
  `fromYear` / `toYear` filters alongside canonical `from_year` / `to_year`.
- [x] Return the complete hybrid plan: provider/native/full-text routes,
  provenance, execution sequence, instructions, merge policy, and
  limitations.
- [x] Treat conflicting aliases, empty queries, invalid years, invalid array
  items, and unknown fields as semantic `invalid_arguments`.
- [x] Treat malformed or unreadable provider configuration as `tool_error` in
  both runtimes; never silently downgrade it to `strategy_only`.

## Security And Boundaries

- [x] Never return provider credentials, environment values, malformed config
  content, or local config paths from these tools or their errors.
- [x] Keep both tools network-free and process-free; platform-native queries
  are instructions for the active agent, not actions performed by MCP.
- [x] Keep canonical source under `content/mcp-contracts/`; do not modify
  academic `content/standards/`, repository-code `tooling/quality/`, generated
  plugin payloads, release archives, external marketplace catalogs, or
  lockfiles.
- [x] Begin release-preflight validation of the versioned capability contract
  now that the pilot has expanded beyond one capability.

## Verification

- [x] Registry, schemas, runtime declarations, descriptions, smoke identity,
  side effects, security paths, and MCPB manifest metadata validate.
- [x] Lite and Full golden status and hybrid-plan calls validate against the
  canonical schemas and share the same canonical projection.
- [x] Provider-only, hybrid, native-only, and strategy-only planning modes have
  deterministic tests without live provider calls.
- [x] Alias normalization, conflict, empty-query, year-range, unknown-field,
  malformed-config, and secret-canary negative paths are covered.
- [x] Rust format, clippy, and all-target tests pass; Python, MCPB, repository,
  materialized-distribution, and boundary regressions pass before handoff.

## Acceptance

- Registry coverage is honestly reported as `6 / 23` and `7 / 24`.
- Both runtimes advertise exact canonical input schemas for the batch.
- Literature status reports active providers without exposing credentials or
  local paths.
- Search-plan output is complete, schema-valid, provenance-aware, and performs
  no network or native-search action.
- Corrupted provider config is a redacted `tool_error`, not a successful
  strategy-only plan.

## Next Migration

After this batch remains green, migrate `qiongli_literature_search`, including
its inputs, result envelope, diagnostics, provider failures, timeout behavior,
and network side-effect contract.
