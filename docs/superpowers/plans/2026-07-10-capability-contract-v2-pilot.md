# Capability Contract v2 Pilot Execution Plan

## Goal

Start Stage 1 of the unified-platform roadmap with one complete, low-risk
vertical slice. The pilot makes `qiongli_literature_export_evidence` consume a
versioned capability record, canonical input/output schemas, semantic error
classes, and Lite/Full golden conformance without changing its public name or
claiming that the remaining tools are already migrated.

## Baseline

- Rust Marketplace Lite advertises 12 public tool names.
- Python Full advertises 22 public tool names.
- Ten names overlap; two Zotero names are Lite-only and twelve names are
  Full-only.
- `qiongli_open_config_wizard` is a compatibility alias, so the completed v2
  registry should contain 23 canonical capability records expanded to 24
  public names.
- Only four of the ten overlapping input schemas are currently identical.
- Lite and Full use different wire-level error carriers and structured output
  envelopes. Contract v2 must record and migrate these differences instead of
  hiding them behind name-only parity.

## Pilot Selection

`qiongli_literature_export_evidence` is the first slice because it:

- has no network, config-write, project-write, process, or agent side effect;
- already has a Lite/Full semantic projection test;
- exercises input schema, output schema, compatibility arguments, error
  semantics, redaction, and runtime declaration alignment;
- can be tested deterministically without live providers or user data.

## Files

Create:

- `content/mcp-contracts/v2/registry.json`
- `content/mcp-contracts/v2/registry.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_literature_export_evidence.input.schema.json`
- `content/mcp-contracts/v2/schemas/qiongli_literature_export_evidence.output.schema.json`
- `tooling/scripts/validate_capability_contract.py`
- `scripts/validate_capability_contract.py`
- `tests/test_capability_contract_v2.py`

Modify:

- `content/mcp-contracts/lite-tools.json`
- `content/mcp-contracts/fixtures/lite-tool-smoke-calls.json`
- `packages/qiongli-lite-mcp/src/mcp/server.rs`
- `packages/python-qiongli/src/qiongli/bridges/literature_mcp_tools.py`
- `packages/python-qiongli/src/qiongli/bridges/mcp_tool_handlers.py`
- `.github/workflows/ci.yml`
- `docs/superpowers/roadmaps/2026-07-09-unified-platform-roadmap.md`

## Task 1: Registry And Schemas

- [x] Add a Draft 2020-12 registry schema with explicit profiles, maturity,
  lifecycle, errors, side effects, security, and smoke evidence.
- [x] Add a pilot registry that reports `1 / 23` canonical and `1 / 24` public
  name coverage instead of claiming Stage 1 completion.
- [x] Add canonical input and output schemas for evidence export.
- [x] Preserve `query_plan`, `search_results`, and `search_diagnostics` as
  declared deprecated argument aliases.

## Task 2: Runtime Alignment

- [x] Make Lite and Full advertise the canonical pilot input schema.
- [x] Make both runtimes normalize compatibility arguments to canonical output
  fields.
- [x] Keep Lite's JSON-RPC `-32602` and Full's tool-result carrier during the
  compatibility window, but map both to the semantic `invalid_arguments` class.
- [x] Keep the shared output core stable while allowing Lite's `status` and
  Full's `exported_at` compatibility fields.

## Task 3: Validation And CI

- [x] Validate registry shape, counts, names, profiles, schema references,
  taxonomy references, side-effect policy, security, smoke evidence, and runtime
  declaration drift.
- [x] Add machine-readable validator output.
- [x] Add golden Lite/Full calls, alias normalization, output validation,
  semantic error classification, and negative schema tests.
- [x] Add the validator to required CI.
- [ ] Add the validator to release preflight after the pilot remains green on
  `dev` and the registry expands beyond one tool.

## Acceptance

- `python scripts/validate_capability_contract.py` exits zero.
- Both runtimes advertise the same pilot input schema.
- The same canonical or compatibility-alias call produces schema-valid outputs
  with equivalent common fields in Lite and Full.
- Unknown arguments are classified as `invalid_arguments` without requiring a
  shared wire carrier in this pilot.
- Registry and output negative fixtures fail with actionable paths.
- Existing Lite behavior, Full behavior, MCPB, release, and strict repository
  tests remain green.

## Next Migration Order

1. Configuration status and save/setup tools, including the wizard tool alias.
2. Literature status and search-plan schemas.
3. Literature search inputs, result envelope, diagnostics, and provider errors.
4. Preview routing and task-plan tools.
5. Zotero Lite tools with explicit Full availability status.
6. Full-only orchestration, subject, experience, lifecycle, and execution tools.

The registry becomes `complete` only when all 23 canonical records, all 24
public names, runtime declarations, smoke evidence, schemas or explicit
migration status, and profile availability are covered.
