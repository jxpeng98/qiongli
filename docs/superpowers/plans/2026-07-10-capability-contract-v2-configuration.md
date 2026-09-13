# Capability Contract v2 Configuration Batch

## Goal

Continue Stage 1 after the evidence-export pilot by migrating the shared
provider-configuration surface as one security-sensitive vertical slice:

- `qiongli_config_status`
- `qiongli_save_provider_config`
- `qiongli_configure_provider`
- `qiongli_open_config_wizard` as a compatibility alias

The batch must define truthful side effects and secret handling, align Lite and
Full semantics where compatibility permits, and leave wire-level error carrier
convergence for the declared compatibility window.

## Baseline Drift

- Lite rejected `cwd` for config status while Full used it for project-local
  lookup.
- Full ignored unknown arguments and coerced invalid values for the three
  configuration handlers.
- Config status reported two missing activation fields in Full and four in
  Lite.
- Save output normalization, `saved`, warning behavior, and invalid-field error
  classification differed.
- Full provider writes were non-atomic and could overwrite malformed config.
- Full config status could return arbitrary `search` values inside an otherwise
  redacted payload.
- Wizard input errors were reported as generic tool failures, Full lacked
  Lite's bounded listener controls, and the alias was not modeled canonically.

## Contract Work

- [x] Add canonical input/output schemas for the three canonical tools.
- [x] Model `qiongli_open_config_wizard` only as a compatibility alias.
- [x] Raise registry coverage to `4 / 23` canonical records and `5 / 24` public
  names without changing pilot status.
- [x] Record local reads, local config writes, loopback listener startup, and
  deferred browser-form writes explicitly.
- [x] Mark the save value as secret-bearing input and the wizard URL as
  sensitive control output.
- [x] Require every canonical and alias public name to have referenced smoke
  evidence.

## Runtime Alignment

- [x] Make both runtimes advertise the same configuration input schemas.
- [x] Accept `cwd` as a typed compatibility context in Lite and reject unknown
  or mistyped Full arguments.
- [x] Use all four activation-required provider fields for status `missing` and
  a shared next-action priority.
- [x] Return canonical provider/field names and `saved: true` after writes.
- [x] Classify caller-controlled schema/provider/field failures as semantic
  `invalid_arguments`; preserve runtime/IO failures as `tool_error`.
- [x] Add `status` to Full configuration outputs and reuse an active Full wizard
  session across the canonical and alias names.

## Security Hardening

- [x] Make malformed or unreadable global provider JSON fail closed.
- [x] Write provider configuration through a same-directory temporary file,
  flush/fsync it, and atomically replace the destination. Use mode `0600` on
  Unix; on Windows, Rust Lite applies a protected DACL that grants only the
  current user full control. The legacy Node MCPB fails closed and remains
  read-only on Windows because it cannot enforce that DACL.
- [x] Keep the prior file and clean temporary state if replacement fails.
- [x] Honor `enabled: false` end to end: omit disabled credentials from command
  environments, report `strategy_only` when no provider is active, and never
  fall back to a legacy network search after an explicit provider opt-out.
- [x] Resolve `QIONGLI_CONFIG_HOME` identically across runtimes: accept an
  fully qualified absolute path or portable `~` / `~/...` home notation,
  reject rooted or drive-prefixed home suffixes and other relative values, use
  the platform user home by default, and never fall back to the process working
  directory.
- [x] Allowlist typed public search settings in redacted status output.
- [x] Bound the Full wizard to loopback, ten minutes, 16 KiB default request
  bodies, one successful submission, and fixed secret-free errors.
- [x] Add `no-store`, `nosniff`, and no-referrer response policy.

## Verification

- [x] Registry structure, schema references, runtime declarations, alias
  inheritance, smoke identity, lifecycle, security, and side effects validate.
- [x] Lite and Full golden config-status/save/wizard outputs pass canonical
  schemas.
- [x] Unknown, mistyped, unsupported, and out-of-range inputs share the
  `invalid_arguments` semantic class without requiring one wire carrier.
- [x] Secret canaries do not appear in success output, errors, status,
  diagnostics, or wizard responses.
- [x] Malformed-file, config-home resolution, atomic replacement, permission,
  TTL, body-limit, and single-use negative paths have focused tests.
- [x] Complete the repository-wide and materialized-distribution regression
  suite before handoff.

Rust Lite Windows writes create the temporary file under a protected
current-user-only DACL and verify the persisted DACL after replacement. The
remaining release gate is execution of the DACL, reparse-point, and fail-closed
cases on the Windows CI runner; the legacy Node MCPB does not write on Windows.

## Acceptance

- Registry coverage is honestly reported as `4 / 23` and `5 / 24`.
- `qiongli_open_config_wizard` has no duplicated capability record or schemas.
- Both runtimes advertise exact canonical input schemas for the batch.
- Successful outputs validate against the canonical output schemas.
- Invalid caller input cannot write config or start a non-loopback listener.
- Provider secrets never appear in MCP responses or wizard rejection bodies.
- Existing corrupted provider configuration is never silently replaced.

## Next Migration

After this batch remains green, continue with `qiongli_literature_status` and
`qiongli_search_plan`, then migrate literature search inputs, diagnostics,
provider failures, and result envelopes.
