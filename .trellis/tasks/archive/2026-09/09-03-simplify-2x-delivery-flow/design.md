# Technical Design — lightweight Qiongli 2.x delivery

## Authority flow

```text
Master roadmap -> one bounded Trellis task -> Daily / PR / named Build
merged 2.x + explicit release task         -> Release
```

Completion in one lane never authorizes or implies entry into the next lane.
The program ledger records state/evidence alongside this flow but does not set
the immediate work order.

## Existing owners

- `.trellis/workflow.md`: task phases and default verification tier.
- `.trellis/config.yaml` plus `.codex/agents/trellis-*.toml`: Codex supervisor,
  Implement, and Check dispatch.
- `CONTRIBUTING.md`: human entrypoint and lane routing.
- `.github/delivery-checklists.md` and PR template: commit, push, PR, and release
  authority boundaries.
- `scripts/check_2x_native_change_boundary.sh`: whether a ready PR needs native
  matrix execution.
- `docs/development/local-desktop-build.md`: local and target build commands.
- the master roadmap: long-term order and current horizon.
- the program ledger/index: exact state and evidence.

No new workflow, validator, schema, dependency, or Agent role is introduced.

## Native-matrix classification

Pure documentation/process paths may skip the expensive Rust matrix because
they cannot change native source or packaged resources. The existing required
job names still report success through their lightweight branch.

Runtime source, content, tests, scripts, GitHub workflows/actions, mixed diffs,
unknown files, and empty diffs require the matrix. Frozen paths remain rejected
before classification.

## Compatibility and safety

- Keep the four required Native CI context identities unchanged.
- Keep explicit `workflow_dispatch` full regardless of path classification.
- Keep the authorization validator's four ordered checklist stages and required
  safety markers.
- Keep historical evidence immutable and distinguish the Alpha 5 product source
  from later documentation commits.

## Rollback

Reverting this task restores the earlier documentation and conservative matrix
classification. No persisted product state, package format, release asset, tag,
or external service is changed.
