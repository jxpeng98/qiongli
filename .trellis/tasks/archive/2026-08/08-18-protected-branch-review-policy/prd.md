# Configure protected branch and review ownership

## Goal

Advance `GOV-413` as far as the current single-maintainer repository can safely
support: declare review ownership for security, schema, migration, release,
research-Gate, and authorization paths; make the active `2.x` ruleset and its
required checks version-controlled and machine-verifiable; and require
`Evaluation Truth V1` without making `2.x` impossible to update.

The task must report independent review as blocked until a second eligible human
reviewer exists. It must not claim that a CODEOWNER file is equivalent to an
enforced approval.

## Background and confirmed facts

- The Program Ledger lists `GOV-413` as the next proposed governance item.
- GitHub ruleset `18800504`, `2.x protected native development`, actively
  targets only `refs/heads/2.x` with no bypass actor.
- The ruleset already blocks deletion and non-fast-forward updates, requires a
  PR, dismisses stale reviews, resolves review threads, and requires the native
  change-boundary plus Linux/macOS/Windows Rust checks.
- It currently requires zero approvals, does not require CODEOWNER review, and
  does not require `Evaluation Truth V1`.
- `.github/CODEOWNERS` does not exist.
- `jxpeng98` is the only collaborator with write/admin permission; there are no
  pending collaborator invitations. The repository is public and user-owned,
  so team-based required reviewers are unavailable.
- GitHub does not allow a PR author to approve their own PR. Enabling one
  approval or required CODEOWNER review now would deadlock maintainer-authored
  changes because the ruleset has no bypass.
- PR #131 proved the protected PR path and all four existing required checks on
  the current repository state.

## Requirements

1. Add one `.github/CODEOWNERS` file that maps the six named sensitive domains
   to the current write-authorized maintainer and protects the CODEOWNERS and
   reviewer-policy files themselves.
2. Add one closed, versioned repository-review policy record bound to
   `jxpeng98/qiongli`, branch `2.x`, ruleset `18800504`, the six domains, the
   exact required checks, and the current review-enforcement state.
3. Reuse the existing authorization-policy validator and test module to reject
   unknown fields/domains, missing ownership, unsafe or nonexistent repository
   paths, weakened branch rules, missing checks, malformed CODEOWNERS lines, and
   false claims of enforced independent review.
4. Keep `Evaluation Truth V1` as the single CI owner by extending its existing
   authorization validation; add no dependency or second governance framework.
5. Update the live ruleset only to add `Evaluation Truth V1` to the required
   checks. Preserve its branch target, PR rule, deletion/non-fast-forward
   protection, native checks, merge methods, stale-review behavior, thread
   resolution, and empty bypass list.
6. Keep `required_approving_review_count=0` and
   `require_code_owner_review=false` while only one eligible reviewer exists.
7. Record `GOV-413` as `blocked`, with repository evidence and an exact blocker:
   a second independent write-authorized human must be nominated and added
   before approval and CODEOWNER enforcement can be enabled.
8. Preserve App, CLI, MCP, Plugin/Skills, project, release, package, and user data
   unchanged. No package rebuild is caused by this policy-only slice.

## Acceptance criteria

- [x] `.github/CODEOWNERS` contains ordered, parseable ownership entries for all
      six sensitive domains and for its own policy surface.
- [x] The versioned review policy validates locally and fails closed under
      focused mutation tests.
- [x] `python tooling/scripts/validate_authorization_policy.py` and
      `python -m unittest tests.test_authorization_policy -v` pass.
- [x] Evaluation Truth runs the extended validator and focused tests unchanged
      through its existing command path.
- [x] A post-update GitHub API read shows ruleset `18800504` active on `2.x`, no
      bypass, all prior protections intact, and `Evaluation Truth V1` included
      in the exact required-check set.
- [x] The API read still shows zero required approvals and no required
      CODEOWNER review; no maintainer lockout is introduced.
- [x] The generated roadmap index is current and truthfully shows `GOV-413`
      blocked rather than accepted.
- [x] Exact-head CI passes before merge through the protected PR path.

## Out of scope

- Adding or inviting a collaborator without a user-nominated GitHub identity.
- Enabling required approvals, CODEOWNER approval, last-push approval, or a
  bypass actor while the repository has only one eligible human.
- Moving the repository into an organization or creating a reviewer team.
- `GOV-414` checklists, `GOV-415` history policy, release authorization changes,
  and product/Graph/Plugin behavior.
- Changing allowed merge methods or unrelated repository settings.

## Deferred activation

When a second independent human with write permission is available, a separate
reviewed change may add that identity as an owner, set the approval count to at
least one, enable required CODEOWNER review, verify a non-stale approval on an
exact revision, and then move `GOV-413` from `blocked` to `accepted`.
