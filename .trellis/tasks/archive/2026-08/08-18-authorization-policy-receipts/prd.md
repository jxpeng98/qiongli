# Publish authorization policy and receipts

## Goal

Publish one versioned, machine-checkable governance contract for `GOV-410`
through `GOV-412`: who may request, execute, and authorize sensitive actions;
which authority transitions are explicitly non-transitive; and the minimum
redacted receipt shape that binds each decision to exact scope and evidence.

The contract must make repository and release automation safer without changing
the current App, CLI, Plugin/Skills, MCP, GitHub settings, or publication state.

## Background and confirmed facts

- The master roadmap owns the requirements in Section 19.1 through 19.4 and the
  live Program Ledger lists `GOV-410`, `GOV-411`, and `GOV-412` as the next three
  proposed GOV items.
- The roadmap already distinguishes research, repository, and publication
  planes; eight minimum roles; a minimum action matrix; non-transitive authority;
  and the required redacted receipt fields
  (`docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md:911`).
- Native operations already use action-specific preview, digest, approval, and
  receipt controls. The Community Alpha release path has a specialized
  `NativePublicationAuthorizationV1`
  (`packages/qiongli-native/crates/qiongli-platform/src/distribution.rs:348`).
- That release type is an executable exact-release-set authorization. It is not
  a general authorization matrix or receipt for research and repository planes.
- ADR 0216 applies Rust-generated authority to changed public App IPC, MCP, and
  CLI JSON. The new governance files remain policy-as-code only; exposing this
  receipt through a public product boundary later requires a Rust-owned schema
  and compatibility record.
- `GOV-413` through `GOV-418` separately own branch protection, checklists,
  exact-head rules, distinct publication decisions, exceptional paths, and
  policy enforcement. They must not be reported as complete here.

## Requirements

### R0 — Product-spine and editability preflight

- Before implementing governance policy, re-run the current-source checks for
  App/CLI, Full MCP, Codex/Claude Plugin bundles, standalone Skills, and the
  receipt-owned Workflow/Skill edit -> reconcile -> Ready -> reset path.
- Prove editability through executable state transitions and installed output,
  not only editable metadata or file presence.
- If a check exposes a real gap, repair it at the existing shared owner, add the
  smallest focused regression, and requalify affected package inputs before
  continuing. If no gap exists, record the exact evidence and do not rewrite
  working product code.

### R1 — Versioned authorization matrix

- Add one closed, versioned JSON policy with exactly the three authorization
  planes and the eight roadmap roles.
- Cover canonical research mutation, destructive research mutation,
  restricted-data movement, local edit, stage, commit, push, PR open/update,
  merge, and release publication.
- For every action, record its plane, allowed executor roles, required
  authorizer roles, exact binding requirements, evidence requirements, and
  deny-by-default behavior.
- An Agent or CI principal may execute an already-authorized mechanical action
  and emit evidence, but must never appear as an authorizer.

### R2 — Non-transitive authority

- Encode explicit negative transitions, including:
  `edit -> commit`, `commit -> push`, `push -> merge`, and
  `CI green -> release publication`.
- Also retain the roadmap's adjacent safety boundaries for
  `preview -> apply`, `edit -> stage`, `stage -> commit`,
  `PR open/update -> merge`, and `merge -> release publication`.
- Validation must reject a missing, duplicate, reordered, unknown, or positive
  implication. No success signal may be interpreted as an authorization.

### R3 — Redacted receipt schema

- Add one Draft 2020-12 JSON Schema with closed fields for schema/record version,
  opaque authorization ID, action, object scope, actor role, authorizer role,
  project/source revision, plan digest, artifact digests, data classification,
  decision, constraints, reason code, issue time, expiry, and evidence refs.
- Require at least one plan or artifact digest and a finite expiry.
- Restrict actions and roles to the policy inventory; exclude Agent/CI from the
  authorizer role enum.
- Include one synthetic redacted example and no credentials, prompts,
  conversations, raw research content, secrets, tokens, or machine-absolute
  paths.
- The receipt is immutable evidence, not a bearer credential and not executable
  authorization by itself.

### R4 — Deterministic validation and CI

- Add one Python-standard-library validator that checks exact keys, ordered
  closed inventories, unique IDs, role/action references, non-transitive pairs,
  canonical repository paths, the receipt schema contract, and its example.
- Add one focused `unittest` module covering the valid repository state and the
  minimum negative mutations needed to prove the above boundaries.
- Run both through the existing `Evaluation Truth` workflow.
- Update the product-control Trellis spec with the executable contract.

### R5 — Truthful roadmap evidence

- Mark only `GOV-410`, `GOV-411`, and `GOV-412` active during implementation.
- Mark them accepted only after exact implementation-head CI passes, using the
  implementation commit and run ID as Program Ledger evidence.

## Acceptance criteria

- [ ] Current-source tests prove the CLI, MCP, Plugin/Skills bundles, and
      Workflow/Skill edit/reconcile/reset path remain complete and fail closed.
- [ ] One validator accepts the repository policy and schema with no errors.
- [ ] The matrix contains exactly the planned planes, roles, actions, and core
      non-transitive rules, with Agent/CI excluded from authorizer roles.
- [ ] The schema is Draft 2020-12, denies unknown fields, binds every required
      receipt field, requires a digest and expiry, and validates its synthetic
      redacted example.
- [ ] Tests reject missing/reordered inventory, unknown roles/actions, Agent/CI
      self-authorization, absent core non-transitive pairs, widened implication,
      missing digest/expiry, unknown receipt fields, and unsafe evidence paths.
- [ ] Evaluation Truth invokes the validator and focused tests.
- [ ] Existing architecture, ledger, and roadmap-index checks remain green.
- [ ] No product wire shape, runtime behavior, Host profile, GitHub protection,
      tag, release, package, or user data changes.
- [ ] Exact-head CI evidence is recorded before `GOV-410` through `GOV-412` are
      accepted.

## Out of scope

- Runtime authorization engines, App UI, CLI/MCP receipt exposure, identity
  federation, credential storage, or migration of existing specialized native
  authorization types.
- GitHub branch protection, CODEOWNERS, pre-push/release checklist automation,
  release/announcement separation, denial/revocation workflows, and full
  self-authorization policy enforcement (`GOV-413` through `GOV-418`).
- Any real research mutation, restricted-data transfer, Git push/merge, tag,
  release publication, or public announcement.

## Risks and constraints

- A documentation-only matrix could drift; CI therefore validates one canonical
  machine-readable policy and schema.
- A general schema could be mistaken for runtime authority; its record and spec
  must state that it is evidence-only until an action-specific consumer verifies
  current scope, revision, digest, decision, and expiry.
- Do not add a JSON Schema dependency or a speculative authorization framework;
  the closed v1 contract is small enough for repository-standard-library checks.
