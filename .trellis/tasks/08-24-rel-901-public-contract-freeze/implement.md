# REL-901 Implementation Plan

## 1. Freeze The Existing Policy

- Extend `tooling/architecture/public-schema-policy.json` with the closed global
  compatibility window and three family freeze rows.
- Do not change App, MCP, CLI, project, or global-state runtime schemas.

Focused check: parse the JSON and inspect only the new keys.

Rollback point: revert the policy-only edit if an identity cannot be bound to
an existing source owner.

## 2. Enforce The Freeze

- Extend `tooling/scripts/validate_public_schema_policy.py` using only the
  standard library and existing repository-path safeguards.
- Validate exact structure/values, App Rust/TypeScript version agreement, and
  MCP registry/schema root identity.
- Extend `tests/test_public_schema_policy.py` with the smallest mutation matrix
  that proves global and family freeze fields fail closed.

Focused checks:

```bash
python tooling/scripts/validate_public_schema_policy.py
python -m unittest tests.test_public_schema_policy -v
```

Rollback point: keep one shared validator path; do not add a second release
validator if the existing policy validator can own the rule.

## 3. Keep Cross-Surface Contracts Green

Run:

```bash
python3 scripts/validate_capability_contract.py
python3 tooling/scripts/update_program_roadmap.py --check
git diff --check
```

Confirm no files under App API, native runtime, MCP contracts, or generated
distribution trees changed.

## 4. Record The Executable Contract

- Update the product-control public-schema spec with the release-freeze rule.
- Commit the product/policy source, push it, and obtain exact-head Evaluation
  Truth plus required Native CI Slice evidence.
- Only after both pass, add one acceptance note, set `REL-901` to `accepted`,
  regenerate the current program index, and commit the evidence-only closeout.
- Open a PR against `2.x`; merge only after required checks pass.

Do not start `REL-902`, `REL-903`, candidate packaging, promotion, publication,
or legacy-path retirement in this task.
