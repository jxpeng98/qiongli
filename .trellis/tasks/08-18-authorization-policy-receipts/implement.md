# Implementation Plan

## 0. Re-audit CLI, MCP, Plugin, and Skill editability

- [x] Trace canonical content through private Workflow/Skill edit, receipt-owned
      variant storage, standalone Skills and Codex/Claude Plugin materialization,
      managed reconcile, fresh Host Ready observation, Full MCP, and reset.
- [x] Run current-source App API/Desktop, WorkflowVariantStore, content override,
      CLI, MCP, Codex/Claude bundle, Capability Contract v2, and isolated packaged
      vertical checks.
- [x] If any link fails, repair the shared owner and add one focused regression;
      otherwise record the passing evidence and leave product code unchanged.

## 1. Activate the bounded slice

- [x] Review this PRD/design/plan, then run `task.py start` only after fresh
      user approval.
- [x] Create `feat/authorization-policy-receipts` from the current local `2.x`
      head, including the two prior Trellis closeout commits.
- [x] Load product-control and shared Trellis specs with `trellis-before-dev`.
- [x] Mark only `GOV-410`, `GOV-411`, and `GOV-412` active and regenerate the
      current index.

## 2. Publish the policy and receipt schema

- [x] Add one closed v1 authorization matrix for the three planes, eight roles,
      bounded action inventory, required bindings/evidence, and exact
      non-transitive rules.
- [x] Add one closed Draft 2020-12 receipt schema with a synthetic redacted
      example, exact role/action enums, digest binding, and finite expiry.
- [x] Preserve existing native authorization types and every product/public wire
      shape unchanged.

## 3. Enforce the contract

- [x] Add one Python-standard-library validator, reusing the existing canonical
      repository-path helper.
- [x] Add one focused table-driven `unittest` module for valid state and the
      PRD's fail-closed mutations.
- [x] Run both in the existing Evaluation Truth workflow.

Focused checks:

```bash
python tooling/scripts/validate_authorization_policy.py
python -m unittest tests.test_authorization_policy tests.test_arc_201_adrs -v
python tooling/scripts/update_program_roadmap.py --check
git diff --check
```

## 4. Record the executable rule

- [x] Add the product-control authorization policy spec and link it from the
      control index.
- [x] Run `trellis-check`; verify the current ADR/public-schema/ledger checks
      remain green.
- [x] Confirm no product, package, Host, release, or user-data input changed, so
      no packaged-product rebuild is required by the evidence ladder.

## 5. Integrate exact evidence

- [ ] Commit the policy implementation, push the feature branch, open a PR, and
      resolve exact-head CI/review failures.
- [ ] After exact implementation CI passes, mark `GOV-410` through `GOV-412`
      accepted with the implementation commit and Evaluation Truth run; regenerate
      the index and rerun focused checks.
- [ ] Merge only after final exact-head required checks pass, then archive and
      journal the task before selecting `GOV-413`.

## Risk and rollback points

- The change is governance-only; revert removes the gate without mutating
  product or external state.
- Do not widen this task into branch settings, runtime enforcement, or release
  publication.
- Unknown roles, actions, fields, implications, paths, and receipt values fail
  closed.
