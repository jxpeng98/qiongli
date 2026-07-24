# Qiongli 1.x To 2.x Replacement Migration Execution Plan

Status: implemented in source; packaged macOS and deferred manual-host
qualification remain

Date: July 24, 2026

Target branch: `feat/r4b-ui-localization-polish`

Architecture authority:
[ADR 0212](../../architecture/decisions/0212-qiongli-1x-replacement-migration.md)

Depends on:

- R3Q packaged-product control and receipt-owned client installation;
- R4A native project migration and project-state service;
- R4 host-driven Plugin + Full MCP architecture; and
- accepted frozen 1.x baseline `v1.19.0-beta.1`.

## Goal

Replace the current indefinite 1.x/2.x coexistence behavior with one bounded
migration path:

- detect supported Qiongli 1.x installations and data;
- explain exactly what can be migrated, regenerated, removed, or needs review;
- preserve supported user academic data and settings;
- install and activate a verified Qiongli 2 product;
- remove recognized active 1.x surfaces only after 2.x verification;
- finish with one Qiongli 2 product, one set of client integrations, and one
  truthful version/build identity.

The migration path never runs Qiongli 1.x code and never copies 1.x generated
runtime assets into the Qiongli 2 product.

Implementation note (July 24, 2026): the automatic transaction covers nine
bounded local surfaces—eight Codex/Claude integration records plus the legacy
provider document. Research projects remain a separate explicit
source/destination copy migration because no bounded global location can prove
which directories are projects. The Desktop migration card and CLI
documentation direct users to that existing digest-bound workflow.

## Product outcome

The normal integration journey becomes:

```text
No Qiongli
  -> Install Qiongli 2
  -> Activate in host
  -> Connected

Qiongli 1.x detected
  -> Review migration
  -> Stage and verify Qiongli 2
  -> Activate in host
  -> Remove verified 1.x surfaces
  -> Migration complete
```

`Mixed ownership` is removed as a successful steady state. Temporary overlap
is represented as a migration phase, not as an ownership classification.

## Scope and data policy

| Surface | Detection | 2.x action | 1.x cleanup gate |
|---|---|---|---|
| Codex `qiongli` plugin and marketplace entry | Known source, marker, entry, and host registration | Materialize verified Qiongli 2 plugin and registration | 2.x source/receipt verified and host action completed |
| Claude Code `qiongli` plugin and local marketplace | Known source, marker, marketplace entry, and registration | Materialize verified Qiongli 2 plugin and registration | 2.x source/receipt verified and host action completed |
| Standalone global Skills | Known workflow manifests under supported client roots | Do not copy; use Skills bundled with the 2.x plugin | Plugin Skills verified |
| Standalone MCP entries | Exact accepted 1.x managed blocks/objects | Do not copy; use the 2.x plugin Full MCP declaration | New declaration verified and host restart acknowledged |
| Python/npm CLI wrappers outside the nine managed surfaces | Not automatically scanned or uninstalled | Do not import or invoke; use the packaged native App/CLI | User/package-manager cleanup remains separate |
| Global provider settings | Supported schema and fields | Normalize supported non-secret values | 2.x config reread succeeds |
| Literature-provider secrets | Recognized provider/field pairs | Import into Keychain only after explicit approval | Redacted Keychain lookup succeeds |
| Direct model credential or unknown provider field | Bounded provider document is classified `review-required` | Do not import into host-driven Qiongli and do not delete implicitly | Explicit manual resolution |
| Existing research projects | Existing native migration preview contract | Copy/register bounded academic files and rebuild native state | Project counts/digests and restart verification pass |
| Caches, logs, downloads, generated traces outside managed surfaces | Not automatically scanned | Do not migrate or read | Preserve |
| Modified or unknown content | Drift/unknown-child/config-conflict evidence | No automatic conversion | Manual resolution required |

Project migration remains copy-based at the data boundary so a failed
transaction cannot destroy research work. It has a separate preview and
receipt: after that project transaction commits, only native 2.x project state
is registered and read by the product. Completing the installation migration
does not claim that arbitrary projects elsewhere on disk have been migrated.

## Migration contract

### `LegacyMigrationInventoryV1`

Add one platform-owned inventory containing:

- detected source release or bounded `unknown-1x`;
- target client and surface type;
- redacted symbolic location;
- ownership evidence and content digest;
- classification: `user-data`, `supported-setting`, `secret`,
  `generated-installation`, `host-registration`, `ephemeral`, or `unknown`;
- proposed action: `convert`, `regenerate`, `remove-after-verify`, `preserve`,
  or `review`;
- blockers and required client/user actions.

The inventory reuses the current client adapters but stops mapping legacy
presence directly to `Mixed` ownership or current Skills health.

### `LegacyMigrationPlanV1`

The preview binds:

- exact inventory revision and item digests;
- current application product version and source commit;
- target Qiongli 2 resource-pack identity;
- ordered prepare, verify, client-action, cleanup, and compensation operations;
- approvals for filesystem mutation, host integration, secret import, and
  final cleanup; and
- a plan digest and bounded expiry.

Any change to an input path, shared config file, host state, project revision,
or packaged product invalidates the preview.

### `LegacyMigrationReceiptV1`

The durable receipt records:

- state: `detected`, `preview-ready`, `staged`,
  `awaiting-client-activation`, `verification-required`, `cleanup-ready`,
  `complete`, or `recovery-required`;
- source classifications and redacted action counts;
- Qiongli 2 product version, source commit, resource-pack digest, and client
  registration receipts;
- cleanup result codes;
- health-window and restart evidence; and
- unresolved items that prevented completion.

Receipts never store a secret, raw config body, absolute private path, project
body, host conversation, or copied legacy runtime.

## Transaction order

1. **Inspect:** discover bounded 1.x surfaces and current 2.x state.
2. **Preview:** classify every item and produce the exact migration plan.
3. **Approve:** require separate approval groups for normal writes, host
   integration, secret transfer, and destructive cleanup.
4. **Prepare:** migrate supported provider settings into staged 2.x state and
   materialize fresh Qiongli 2 integration bytes. Migrate each user-selected
   project through the separate project command.
5. **Verify staged 2.x:** verify product authority, receipts, Skills, MCP
   declaration and configuration reload.
6. **Activate host:** give exact Codex/Claude client actions. Observe the host
   state when supported; otherwise stop for explicit user confirmation.
7. **Restart health:** verify the packaged App and the selected client after
   restart without invoking a model or requiring a live provider.
8. **Cleanup:** remove only proven 1.x entries and sources with a fresh
   compare-and-swap check.
9. **Commit:** record `complete`, refresh inventory, and assert that no
   recognized active 1.x surface remains.
10. **Close health window:** remove transaction-only compensation copies.

If any step fails before commit, compensation restores the changed legacy
registrations/config entries and removes only receipt-owned staged 2.x bytes.
If compensation cannot be proven, the workflow enters `recovery-required`.

## Implementation batches

### Batch M0 — Architecture, release identity, and fixture separation

Purpose: remove the misleading release and acceptance assumptions before
adding mutation.

Changes:

- accept ADR 0212 and make this plan the Alpha.2 migration authority;
- advance the complete product identity to `2.0.0-alpha.2` in one coordinated
  change across Cargo, application metadata, embedded control, update
  metadata, scripts, receipts, fixtures, and tests;
- replace Alpha.1-named local build scripts with version-derived names where
  the script is not a historical release artifact;
- display product version and packaged source commit separately;
- split automated destructive acceptance homes from clean manual UI homes;
- remove legacy canary creation from the ordinary packaged-product lifecycle
  and retain it only in dedicated migration fixtures.

Acceptance:

- the packaged App, CLI, embedded plugin, and receipts agree on Alpha.2;
- the App displays an exact short build identity;
- a clean manual home contains no legacy canary or stale marketplace entry.

### Batch M1 — Typed legacy inventory and truthful UI state

Purpose: detect migration inputs without assigning false current health.

Changes:

- add `LegacyMigrationInventoryV1` and supported Codex/Claude/global/project
  detectors;
- derive ownership proof from markers, structured entries, receipts, and the
  frozen baseline rather than path names alone;
- map plugin-bundled Skills to current Skills readiness;
- separate catalog availability, package installation, host activation,
  session observation, and MCP declaration/attachment;
- replace `Mixed ownership` with `Migration available`,
  `Migration in progress`, `Review required`, or `Qiongli 2 current`.

Acceptance:

- current Qiongli 2 plus a legacy fixture reports migration work, not a stale
  installed version;
- bundled Skills are Ready when their verified plugin source is Ready;
- `MCP declared` and `MCP attached` cannot share one ambiguous Ready badge.

### Batch M2 — Preview, staged conversion, and compensation

Purpose: implement a non-destructive migration transaction.

Changes:

- add plan/receipt schemas and digest verification;
- link users to the existing explicit native project migration for supported
  academic content without adding home-directory discovery;
- add supported global-setting conversion;
- add explicit Keychain import for literature-provider secrets and fail closed
  on obsolete direct-model credentials or unknown provider fields;
- materialize Qiongli 2 plugin/Skills/MCP from the embedded pack;
- add fault-injected compensation before any legacy cleanup.

Acceptance:

- no legacy-generated executable or plugin byte is copied into 2.x;
- project and supported-setting results are deterministic;
- failed staging leaves the working 1.x integration untouched.

### Batch M3 — Client activation and verified legacy cleanup

Purpose: complete the cutover without deleting the only working integration.

Changes:

- add exact Codex and Claude activation/deactivation instructions and supported
  observation adapters;
- stop at `awaiting-client-activation` when the client owns an unobservable
  action;
- structurally remove exact 1.x marketplace/MCP entries;
- remove proven 1.x plugin sources and standalone Skills after Qiongli 2
  activation and restart verification;
- block automatic cleanup for drifted, symlinked, or unknown content.

Acceptance:

- completed migration leaves only one active Qiongli integration per client;
- concurrent shared-config changes invalidate cleanup instead of overwriting
  them;
- an activation failure retains or restores the working 1.x integration.

### Batch M4 — Desktop and CLI migration experience

Purpose: give users one understandable upgrade journey.

Changes:

- add a top-level "Qiongli 1.x detected" migration card;
- show Preserve, Convert, Replace, Remove, and Review item counts;
- provide Preview, Continue, Retry verification, Finish cleanup, and Recovery
  actions;
- keep internal paths and plugin slugs in expandable diagnostics;
- add `qiongli migrate-1x inspect|preview|apply|continue|status|recover`;
- never label a migration as complete while host activation is pending.

Acceptance:

- UI and CLI consume the same snapshot and receipt;
- restarting the App resumes the exact pending state;
- completion shows Qiongli 2 version/build and no legacy warning.

### Batch M5 — Acceptance and release gate

Purpose: qualify Alpha.2 without preserving legacy as an active product mode.

Fixtures:

- clean machine;
- 1.x Skills-only;
- Codex plugin-only;
- Claude plugin-only;
- full 1.x plugin plus standalone MCP;
- duplicate Skills/plugin surfaces;
- multiple projects and supported provider configuration;
- direct-model credential present;
- unknown file in a legacy-named directory;
- malformed/symlinked/non-private legacy locations;
- interrupted staging, activation, cleanup, and restart; and
- already-completed idempotent rerun.

Release gates:

- focused Rust unit/integration tests and frontend component/flow tests pass;
- packaged macOS acceptance passes in separate automated and manual homes;
- a completed migration receipt proves no recognized active 1.x surface;
- no Python, Node, npm, pip, or Qiongli 1.x process is required;
- no formal cybersecurity scan is introduced;
- real client activation remains manual where the host cannot expose
  authoritative state.

## Files and modules expected to change

- `qiongli-platform`: inventory, migration plans, filesystem transactions,
  client config structural edits, receipts, and recovery;
- `qiongli-project`: project migration aggregation and result digests;
- `qiongli-config`: supported 1.x setting conversion and Keychain import
  coordination;
- `qiongli-content`: current plugin Skills health and Alpha.2 identity;
- `apps/qiongli`: application service, CLI commands, desktop DTOs, and
  migration operations;
- `apps/qiongli-ui`: migration card, separated integration states, confirmation
  and recovery flows;
- packaged-product and candidate acceptance examples;
- macOS local build/acceptance scripts and release receipts; and
- active roadmap, CLI/build documentation, and migration fixtures.

## Explicit non-goals

- executing or embedding the Python/Node 1.x runtime;
- preserving dual active plugin or standalone Skills surfaces after commit;
- automatically interpreting arbitrary user modifications;
- importing direct model-provider credentials into the host-driven product;
- deleting unproven files because their path contains `qiongli`;
- remote/cloud migration; and
- Windows/Linux interactive release qualification in this macOS Alpha.2
  batch.

## Completion definition

This plan is complete when:

1. the packaged product and every embedded/receipt identity report
   `2.0.0-alpha.2` plus the exact source commit;
2. a clean install has no legacy concepts in the primary UI;
3. every supported 1.x fixture reaches either `complete` or a precise
   item-scoped `review-required` state;
4. completed migration leaves one verified Qiongli 2 integration and no
   recognized active 1.x surface;
5. user projects and approved settings retain their accepted meaning;
6. unknown content is never silently deleted; and
7. packaged macOS acceptance and the deferred manual host checklist pass.
