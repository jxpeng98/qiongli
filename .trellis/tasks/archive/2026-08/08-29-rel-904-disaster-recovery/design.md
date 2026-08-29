# REL-904 disaster recovery design

## Boundary

REL-904 reuses the native project and Portfolio recovery owners. The only
behavior change is inside the existing approved Portfolio derived-state reset;
no App, CLI, MCP, schema, or package contract changes.

## Failure map

| Failure | Existing owner | Recovery proof |
|---|---|---|
| Interrupted migration | `ProjectStateService` migration recovery | Resume committed receipt and register once |
| Missing index | `IncrementalPortfolioService::reconcile` | Rebuild absent Portfolio catalog |
| Corrupted derived state | Existing `delete-derived-state` operation | Explicit reset, then clean rebuild |
| Lost registration | `preview_register` / `apply` | Re-register existing portable manifest |
| Partial update | `PortfolioCatalogStore::rebuild` | Replay one durable transaction journal |

## Corrupt-state flow

1. `preview_delete_derived_state` reads the current Research Library revision.
2. A valid or missing catalog keeps the existing preview identity. Only
   `InvalidPortfolioCatalog` is represented as absent/resettable derived state.
3. Apply re-runs the same preview and requires the exact digest plus
   `derived-state-write` approval.
4. `PortfolioCatalogStore::delete` uses the normal validated deletion path for
   valid state. For `InvalidPortfolioCatalog` with no expected catalog identity,
   it clears only the fixed private catalog contents while retaining the held
   catalog lock.
5. A later reconcile reconstructs the catalog from registered canonical
   projects.

## Safety and compatibility

- Unsafe paths, symlinks that violate private-root validation, lock contention,
  and transaction recovery errors are not downgraded to corruption.
- Cleanup never follows unknown symlinks and never touches the Research Library
  or project roots.
- Keeping the lock file/root during cleanup avoids the Windows open-handle
  removal problem and preserves serialization.
- No public JSON field, command, reason code, or schema version changes.

## Rollback

The production change is isolated to corrupt derived-state deletion and can be
reverted normally. Canonical data is never a deletion target, so rollback does
not require data restoration.

## Evidence boundary

Focused and Slice evidence qualify only `REL-904`. They do not qualify backup
policy, packaging, promotion, publication, release authorization, `REL-905`, or
1.x retirement.
