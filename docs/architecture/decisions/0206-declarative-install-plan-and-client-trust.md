# ADR 0206: Declarative Install Plan and Client Trust Boundaries

- Status: Accepted
- Date: 2026-07-11
- Task ID: `ARC-201F`
- Owners: Qiongli maintainers
- Decision scope: Qiongli 2 local installation and client integration trust

## Decision drivers

- One install model must serve the CLI, desktop UI, doctor, repair, upgrade,
  removal, and rollback flows without each surface inferring different paths.
- Qiongli must preserve unmanaged user files and must be able to prove which
  files and config entries it owns.
- Codex, ChatGPT, Claude, and other clients retain their own marketplace,
  activation, trust, approval, and administrator-policy boundaries.
- Local installation must use documented source or configuration locations and
  must remain resilient when a host changes its private cache layout.
- A local native binary cannot make files or processes appear in a hosted or
  cloud execution environment.
- Every mutation must be previewable, auditable, verifiable, and reversible.

## Context

Qiongli 1.x has multiple installers and client-specific path inference. That
model is unsuitable for a single native 2.x product because the CLI and desktop
application could disagree about ownership, activation, repair, or removal. It
also creates pressure to treat a host-managed plugin cache as an installation
API, overwrite a user's existing configuration, or infer that a successful
local copy means the plugin is active in the host.

The supported surfaces have different public integration mechanisms. Some
accept a personal or repository plugin source, some support a skills directory,
and some require an import or marketplace action that remains controlled by the
host. Cloud sessions do not share the local machine's filesystem. These are
product boundaries, not installer inconveniences to bypass.

## Decision

### One target-aware typed plan

All local installation changes are represented by a versioned, serializable,
typed `InstallPlan`. The planner receives an explicit product artifact and an
explicit target descriptor; it must not infer a target from whichever client or
machine happens to be available during apply.

At minimum, a plan records:

- plan schema version, plan ID, creation time, and product identity;
- target host, host surface, installation scope, execution profile, operating
  system, architecture, and adapter version;
- the documented source/config roots the adapter is permitted to use;
- typed operations, their preconditions, expected prior state, content hashes,
  postconditions, and inverse operations;
- ownership markers, backups, approval requirements, and any host action that
  remains outstanding;
- a deterministic semantic preview digest that apply must revalidate before
  mutation.

The semantic digest covers the plan schema, product/artifact identity, target,
allowed roots, normalized operations, preconditions, observed-state digests,
postconditions, inverse operations, approvals required, and signed launch
grant. It excludes the random plan ID, creation/display timestamps, localized
copy, and later receipt fields. The plan separately records an expiry. An
approval binds the semantic digest and expiry. Apply rejects an expired plan,
revalidates every observed-state digest and precondition, and recomputes the
digest from canonical plan bytes immediately before mutation. The digest
detects drift; it does not grant write authority or replace path/policy checks.

Typed operations distinguish materializing Qiongli-owned resources, registering
a documented plugin/marketplace source, updating a supported MCP or client
configuration entry, and removing a previously managed entry. Arbitrary path
writes and untyped shell snippets are not valid install operations.

### One transaction lifecycle

The same plan and target model drives `preview`, `apply`, `verify`, `rollback`,
and `remove`:

1. `preview` resolves supported paths, reads current state, detects conflicts,
   and emits the complete proposed change set without mutation.
2. `apply` rechecks expiry, semantic preview digest, observed state, and every
   precondition, stages writes, atomically commits them where the platform
   permits, and writes a receipt.
3. `verify` compares installed state with the receipt, expected content hashes,
   registrations, and declared host-action status. It does not equate
   registration with enablement or runtime activation.
4. `rollback` executes the recorded inverse operations in reverse order and
   restores transaction backups after a failed or explicitly reversed apply.
5. `remove` deletes only entries whose current ownership and content hashes
   match a Qiongli receipt, then verifies that no managed residue remains.

Repair and upgrade are later compositions of these primitives: they must
produce a new preview and transaction rather than mutate state through a second
installer path.

### Managed ownership and conflict behavior

Every Qiongli-managed file or structured config entry has an ownership marker
and a cryptographic content hash recorded in the transaction receipt. A marker
is stored only in a format and location supported by the target adapter; when
the host format cannot carry ownership metadata, Qiongli uses its own receipt
store and addresses the entry by a stable, adapter-defined key.

Apply fails closed before any mutation when:

- a destination exists without a matching Qiongli ownership marker;
- a previously managed destination no longer matches its recorded hash;
- another product or Qiongli installation claims the same structured entry;
- a path escapes an allowed root, traverses a symlink to another root, or
  changes after preview; or
- the current client/adapter contract cannot identify a documented write path.

Conflicts are reported and are never overwritten by apply, repair, upgrade, or
remove. The user must resolve the conflict or choose a different supported
scope and generate a new preview. A generic `--force` flag cannot weaken this
rule.

### Host-owned trust and activation

Adapters may write only official, documented source or configuration paths for
the selected host and surface. Qiongli never writes a Codex, Claude, ChatGPT,
or other host's private plugin or marketplace cache directly, never automates
undocumented UI actions, and never treats a discovered cache path as public
API.

The receipt and UI expose installation states separately: `materialized`,
`registered`, `enabled`, `trusted`, `active`, `unsupported`, and `remote-only`.
Qiongli may verify a state only when the host exposes a documented mechanism.
Host-controlled installation, enablement, restart, marketplace review, MCP
approval, trust prompt, workspace policy, and administrator approval remain
host-controlled. The installer reports the required user or administrator
action and does not bypass or falsely mark it complete.

### Local and remote boundary

The `InstallPlan` is limited to a filesystem and clients reachable on the local
machine. It must not claim that a local Qiongli binary, plugin, skill, or MCP was
installed into ChatGPT web, Codex cloud, Claude cloud, or another hosted worker.
Host-supported content upload may be represented by a separate adapter only
when its public contract is implemented and tested; a production remote MCP or
hosted service belongs to `REM-201` and its separate threat model.

## Alternatives considered

### Keep one imperative installer per client

Rejected. The installers would continue to disagree about paths, ownership,
dry-run behavior, and rollback, and the desktop UI could not safely reuse the
CLI's operations.

### Copy directly into discovered host caches

Rejected. Cache paths are host-owned implementation details, can change without
notice, and bypass the host's source, trust, and lifecycle model.

### Overwrite conflicts after confirmation or `--force`

Rejected. A confirmation does not establish ownership and cannot make removal
or rollback safe. Users resolve unmanaged or externally modified state before a
new plan is generated.

### Treat materialization or registration as activation

Rejected. Host enablement, trust, restart, marketplace, and administrator
policies can still prevent use. Combining these states would produce false
success receipts.

### Use the local installer to provision cloud sessions

Rejected. Local filesystem access does not cross into hosted workers. Cloud
execution requires an explicit host upload mechanism or the future `REM-201`
remote service.

## Consequences

- CLI and desktop integration management share one planner, executor, receipt,
  and verification vocabulary.
- Every supported adapter must publish an allowlisted path and capability
  contract; unsupported or undocumented clients fail without mutation.
- Preview and apply may require an extra discovery pass, and conflict handling
  is deliberately conservative.
- Receipts and managed markers become durable compatibility contracts and need
  schema migration and corruption handling.
- Qiongli can reliably repair or remove its own state while preserving user and
  third-party state.
- Some installations remain pending until the user or administrator completes
  a host-owned action; the UI must communicate that distinction clearly.

## Security and privacy

- The executor canonicalizes and revalidates each path immediately before use,
  rejects traversal and symlink escapes, and restricts writes to the adapter's
  allowlisted roots.
- Files are staged with owner-only permissions where they may contain sensitive
  references; structured writes use atomic replacement where supported.
- Receipts contain identities, hashes, paths, operation results, and redacted
  diagnostics, but never API keys, tokens, environment values, or copied secret
  contents.
- Secrets are referenced through the configuration/keychain boundary and are
  not embedded in a plugin payload or install plan.
- Plan and receipt parsing is fail-closed for unknown operation types, schema
  versions, target identities, or ownership markers.
- Host trust prompts, administrator policy, signature checks, and marketplace
  controls are additive security boundaries and cannot be suppressed by an
  adapter.

## Rollback

Before the first mutation, apply persists a transaction journal and the minimum
backups needed for its inverse operations. If any commit or verification step
fails, it stops further work and rolls back already committed operations in
reverse order. A rollback refuses to delete or restore over content whose
ownership/hash changed after apply; it reports that conflict for manual
recovery instead of causing a second data-loss event.

Rolling back this architecture decision means disabling affected adapters and
leaving their receipts intact for diagnosis. It does not authorize returning to
cache mutation or unmanaged overwrites. A replacement installer design must
first demonstrate equivalent preview, ownership, conflict, removal, and
host-trust guarantees.

## Acceptance tests

- Schema tests reject unknown operation kinds, incomplete target identities,
  unversioned plans, and plans without inverse operations or postconditions.
- Determinism tests produce the same normalized preview and digest for the same
  artifact, target descriptor, and filesystem fixture even when plan ID,
  display time, or locale differs; a semantic field change changes the digest.
- Expiry and stale-state tests reject replayed approval, changed observed state,
  and recomputed plans with altered operations before any write.
- Preview tests prove that no filesystem or client configuration mutation
  occurs.
- Apply/verify tests cover fresh install for every advertised client, scope,
  profile, OS, and architecture tuple.
- Conflict tests cover unmanaged destinations, modified managed files,
  duplicate structured entries, stale previews, path traversal, symlink escape,
  and path replacement between preview and apply; every case performs zero
  writes.
- Failure-injection tests interrupt each operation boundary and prove that
  rollback restores the pre-transaction state or emits a non-destructive
  conflict with a recovery receipt.
- Remove tests prove that only matching Qiongli-owned entries are deleted and
  that unmanaged files, user edits, and other products remain untouched.
- Boundary tests fail any attempted write to known host cache fixtures and fail
  adapters that lack a documented source/config path.
- Client tests distinguish registration, enablement, trust, and active runtime;
  a pending host action can never produce an `active` receipt.
- Cloud tests prove that local targets cannot be mapped to a hosted worker and
  return a structured `remote-only` result referencing `REM-201`.
- Secret scans prove plans, previews, receipts, logs, crash fixtures, and error
  messages contain no credential values.

## Follow-up tasks

- `PLT-201`: define the versioned `InstallPlan`, target, operation, receipt, and
  status schemas plus deterministic preview fixtures.
- `PLT-202`: implement the transaction executor, managed markers, repair,
  removal, failure injection, and rollback tests.
- `INT-201`: implement Codex local adapters against documented source/config
  contracts and real activation receipts.
- `INT-202`: implement Claude Code skills-directory and marketplace adapters
  with preserved trust and approval semantics.
- `CFG-201` and `CFG-202`: provide the receipt/state store, atomic writes,
  migration, backup, and secret-reference boundaries.
- `UI-201` through `UI-203`: expose preview, conflict, pending host action,
  verification, removal, and rollback states without duplicating installer
  logic.
- `REM-201`: separately design hosted/cloud MCP delivery, authentication,
  tenancy, policy, and operations; it is not an extension of local apply.

## Primary references

- [OpenAI: Build plugins](https://developers.openai.com/codex/plugins/build)
- [Anthropic: Plugins reference](https://code.claude.com/docs/en/plugins-reference)
- [Anthropic: Desktop application](https://code.claude.com/docs/en/desktop)
