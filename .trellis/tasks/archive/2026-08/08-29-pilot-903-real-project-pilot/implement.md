# Implementation plan

## 1. Activate the task and freeze the boundary

- After explicit approval of this final plan, run `task.py start` on
  `feat/pilot-903-real-project-pilot` from `2.x`.
- Re-read the product-control and affected native specs before any code edit.
- Record the clean base/source identity and confirm the asset-pricing source has
  no uncommitted drift.

## 2. Prepare exact current-source pilot inputs

- Build the native `qiongli` binary once from the clean task source.
- Record its SHA-256, current embedded Skill/product version, and Codex CLI
  version.
- Create a private temporary root and materialize the existing Codex Qiongli
  Skill under its conventional `.agents/skills` path.
- Use the existing project migration preview/apply commands with an isolated
  `QIONGLI_CONFIG_HOME`; reuse preview identities exactly for apply.

Checkpoint: the migrated project is the only project in isolated state, the
source inventory digest is unchanged, and no normal user config was written.

## 3. Run the Full MCP and ephemeral Codex pilot

- Verify the exact binary's Full MCP `initialize`, tool registry, and
  `qiongli_orchestrator_route` response before model execution.
- Launch one bounded `codex exec --ephemeral --ignore-user-config` run with a
  read-only workspace and explicit current-source Full MCP stdio config.
- Require `$qiongli` routing and a truthful single-agent host descriptor.
- Complete `doctor -> start -> read -> submit -> next` to terminal, including
  one project read and one bounded non-empty Graph query.
- Keep raw Host output only under the private temporary root. Derive a bounded
  observation and verify the durable Qiongli checkpoint independently.

Checkpoint: the terminal checkpoint is revision/digest bound, authenticated
evidence was used, and no artifact/capture apply occurred.

## 4. Derive evidence and rollback

- Compose one path-redacted pilot receipt from exact binary, Skill, source,
  migration, Graph, Host, and checkpoint facts.
- Fail closed on unknown fields or any credential, prompt/response, candidate or
  tool body, conversation/session identifier, temporary path, or absolute path.
- Run the supported migration rollback preview/apply, verify destination and
  registration removal, and compare the source inventory digest again.
- Delete temporary raw output only after supported rollback and evidence
  validation succeed.

Checkpoint: the normal Host/Qiongli state and source project are unchanged.

## 5. Fix only a reproduced essential-path defect

- If all existing owners work, add no product code or duplicate test harness.
- If the pilot fails because of current-source product logic, trace every caller,
  patch the shared owner once, and add one focused regression beside the existing
  owner tests. Rerun only evidence invalidated by that change.
- Operational/authentication unavailability is recorded as a blocker, not
  converted into simulated PASS evidence.

## 6. Verify, accept, and merge

- Commit the product/evidence mechanism before deriving exact-source evidence.
- Run the focused native test, real pilot, roadmap generator check, privacy scan,
  task validation, and `git diff --check`.
- Push the branch and run one exact-head Slice CI. Avoid a second full run for an
  evidence-only closeout commit when no product/package input changed.
- After a passing pilot and CI, add the acceptance note, set `PILOT-903` to
  `accepted`, regenerate the current program index, commit the evidence-only
  closeout, open the PR against `2.x`, and merge after required checks pass.
- Archive the Trellis task only after the merged identity is known.

Rollback point: any failed Host, privacy, source-retention, or migration rollback
gate leaves the ledger proposed and prevents merge of an acceptance claim.
