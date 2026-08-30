# REL-913 installation lifecycle acceptance design

## Boundary

REL-913 composes current owners instead of creating another lifecycle layer:

```text
three-target candidate acceptance -> clean install / verify / uninstall
packaged-product acceptance       -> content upgrade / repair / restart
macOS packaged update helper      -> versioned activation / health rollback
shared Rust transactions          -> cross-platform receipt and rollback rules
```

The exact-head Native CI Acceptance run is the evidence boundary. Ordinary PR
CI remains Slice evidence and cannot accept the release task.

## Candidate lifecycle matrix

Convert `lite-alpha-candidate-acceptance` to the existing three-runner target
matrix while keeping one job owner and one example. Each runner builds an
ephemeral test-signed candidate from its checked-out exact source and executes
both Codex and Claude Code paths in isolated homes.

Before preview, each target home contains three fixed canaries:

- a user-owned project directory and academic text file outside product-owned
  install roots;
- a global v2 state canary below the configured root but outside native payload,
  Plugin source, and Host registration paths; and
- an unrelated home/Host sibling entry.

The example records their SHA-256 values, checks them after every failed or
successful lifecycle boundary, and emits only `passed` fields. Existing
candidate identities, approval requirements, error codes, and file formats do
not change.

## Versioned macOS update fixture

Keep `macos_native_update_journey.sh` as the packaged helper owner. For each
success and rollback journey:

1. Extract two copies of the exact current ad-hoc-signed package.
2. Turn only the old copy into an ephemeral predecessor fixture by deriving the
   immediately preceding alpha fixture version, updating bundle version fields,
   adding a bounded predecessor marker, and ad-hoc re-signing that copy.
3. Retain the untouched exact current package as the staged target.
4. Seed project, global-state, and unmanaged-state canaries and record digests.
5. Run the real packaged update helper with empty `PATH` and isolated `HOME`.
6. For success, require the staged inode/current bundle version, generation 2,
   healthy signature, and complete cleanup.
7. For failed health, require the predecessor inode/version, generation 1,
   healthy predecessor signature, and complete cleanup.
8. Require every canary digest to remain unchanged in both journeys.

This fixture proves versioned replacement mechanics without claiming that the
ephemeral predecessor was published or production-signed.

## Focused contract

Reuse and rename existing tests so a shared `rel_913` filter selects the five
roadmap behaviors through their normal owners:

| Behavior | Existing owner |
|---|---|
| Clean install and uninstall | `ManagedNativePayloadExecutor` plus candidate acceptance |
| Repair | `ManagedNativePayloadExecutor` plus packaged-product repair |
| Upgrade | packaged previous-content reconciliation plus successful update-helper replacement |
| Rollback | reconciliation rollback plus failed-health application restoration |
| Preservation | candidate and update fixtures' project/global/unmanaged canaries |

Only missing version and canary assertions are added. No duplicate umbrella
test or new test framework is introduced.

## Evidence and compatibility

Candidate and update receipts stay schema v1 with additive check fields. The
final repository acceptance document binds the exact source and workflow runs
to the inspected receipts; it stores no binaries, local paths, Host state, or
project content.

Existing public CLI, App, MCP, Plugin, Skill, project, and Host contracts remain
unchanged. Candidate acceptance still uses ephemeral in-memory keys and every
artifact remains `publication_allowed: false`.

## Risk and rollback

- A canary mismatch, version mismatch, missing target receipt, or incomplete
  cleanup fails Acceptance and leaves `REL-913` proposed.
- The predecessor copy is confined to a private runner fixture; the exact staged
  current package is never modified.
- The implementation can be reverted as workflow/example/script/test changes.
  No normal user home or published artifact is mutated.
