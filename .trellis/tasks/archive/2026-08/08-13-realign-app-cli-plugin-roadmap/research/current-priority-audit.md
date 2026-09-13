# Current Priority Audit

Date: 2026-08-13

## Executive finding

The first priority is not missing bundle generation. The packaged App already
contains the native CLI and target-specific `qiongli-next` Plugin payloads. The
remaining user-outcome gap is that App-controlled installation stops after
materialization and registration: Host-owned Plugin installation/enablement is
shown as a command, so the App truthfully remains at
`installed-host-action-required` until a separate Host action and probe succeed.

The second priority is not missing Plugin structure. Current structural and
capability checks are substantial, but the quality baseline is incomplete and
the academic-quality score is not an execution result: the runner averages
scores declared by its case YAML rather than running the Plugin and grading its
artifacts.

## Priority 1 — App-bundled CLI and Plugin effectiveness

### CLI chain

The App's About route already exposes:

1. `preview-cli-install` and digest-bound confirmation;
2. receipt-owned installation/update/removal of the bundled native binary;
3. bounded login-profile PATH configuration;
4. a fresh login-shell resolution and version test.

Owners and evidence:

- `packages/qiongli-desktop/src/routes/about/+page.svelte`
- `packages/qiongli-native/apps/qiongli/src/cli_install.rs`
- `packages/qiongli-native/apps/qiongli/src/desktop.rs`
- Alpha 3 A3 receipt in
  `docs/superpowers/acceptance/2026-08-01-qiongli-alpha3-readiness.md`

The CLI path therefore needs one packaged end-to-end acceptance journey and
any reproduced focused fix; it does not need another installer.

### Plugin chain

The packaged-product path already:

1. verifies product control, executable identity, embedded pack, signed launch
   grants, target and source;
2. materializes the target-specific Plugin source;
3. registers the local Codex or Claude Code marketplace/source;
4. preserves ownership, rollback and exact removal receipts;
5. probes the official client CLI for Plugin activation, cache identity and MCP
   attachment.

The missing in-App transition is between steps 3 and 5. Current code deliberately
returns `ClientActionRequired`, and the Desktop displays these official actions:

- Codex: `codex plugin add --json qiongli-next@personal`
- Claude Code: add the local marketplace, then
  `claude plugin install qiongli-next@qiongli-local --scope user`

Current installed client help confirms these commands remain supported. The
Desktop already has a bounded process/probe helper for resolved Codex and Claude
executables, but it uses that helper only for read-only probes.

Relevant owners:

- `docs/architecture/decisions/0206-declarative-install-plan-and-client-trust.md`
- `packages/qiongli-native/crates/qiongli-platform/src/product_control.rs`
- `packages/qiongli-native/apps/qiongli/src/desktop.rs`
- `packages/qiongli-native/apps/qiongli/src/desktop_api.rs`
- `packages/qiongli-app-api/src/schema.ts`
- `packages/qiongli-desktop/src/routes/client-integrations/+page.svelte`

The accepted ADR forbids cache mutation, undocumented UI automation and false
activation claims. It does not currently authorize App invocation of the
documented client CLI. The user has now selected that behavior; because accepted
ADR 0206 is frozen, implementation requires a new superseding ADR that retains
its ownership, conflict, cache and Host-policy boundaries.

## Priority 2 — Plugin quality

Current checks on this branch:

- capability contract: passes;
- canonical Skill structure: 82 scanned, 74 complete;
- eight Coursework/Dissertation Skills retain missing insufficient-input and
  claim-strength constraints; five also lack explicit no-fabrication wording;
- Skill/contract alignment focused tests: pass;
- academic-quality runner: reports 12 cases and nine dimension averages, but
  `tooling/scripts/run_academic_quality_evals.py` only averages each case's
  declared `expected_dimensions`.

The checked-in `docs/maintainer/skill-quality-gap-report.md` is stale at 71/71
and must not be used as the current quality authority.

Claude Code `2.1.222` exposes a native `claude plugin eval` command with plugin
versus no-plugin ablation, graders, thresholds and cost bounds. It is useful as
an optional observed-quality lane, but it is model-dependent and cannot replace
deterministic repository checks. The portable minimum is still roadmap item
`EVAL-409`: convert declared-score cases into executable inputs and expected
findings, then improve canonical Plugin content against those failures.

## Roadmap consequence

The immediate execution projection should become:

1. **Activation vertical:** packaged App -> bundled CLI install/PATH/test ->
   bundled Plugin materialize/register -> confirmed App execution of a fixed
   official Host action -> fresh probe -> exact Plugin/cache identity and Full
   MCP observed ready; Claude also exposes the Skill component, while Codex
   live Skill invocation remains a separate Host-session claim.
2. **Plugin quality vertical:** current structural baseline -> executable
   inputs/findings -> one bounded content-improvement batch -> materialized
   Plugin parity -> optional real-Host ablation receipt.
3. Resume the wider M1 governance, scale, threat-model and later trust milestones
   after these two product outcomes, without deleting their dependencies or
   claiming M0 release qualification.

## Product decision — resolved

The user selected one-confirmation App execution: after confirming the existing
integration-install preview, the App executes only the fixed official Codex or
Claude client CLI actions and then performs a fresh positive Ready probe.
Unknown commands, model/UI-supplied arguments, cache mutation and inferred
activation remain forbidden.
