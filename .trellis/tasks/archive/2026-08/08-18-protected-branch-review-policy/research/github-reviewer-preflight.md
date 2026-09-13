# GOV-413 GitHub reviewer preflight

Date: 2026-08-18

## Repository facts

- Repository: `jxpeng98/qiongli`, public, user-owned; default branch `main`.
- Target development branch: `2.x`.
- Active ruleset: `18800504`, `2.x protected native development`.
- Target: exactly `refs/heads/2.x`; bypass actors: none.
- Rules: deletion blocked, non-fast-forward blocked, PR required, stale reviews
  dismissed, review threads resolved, strict status checks required.
- Required checks before this task:
  - `Native 2.x change boundary`
  - `Rust native foundation (Linux)`
  - `Rust native foundation (macOS)`
  - `Rust native foundation (Windows)`
- Review settings: zero approvals, CODEOWNER review false, last-push approval
  false.
- Eligible collaborators: only `jxpeng98` (admin/write); pending invitations:
  none.
- `.github/CODEOWNERS`: absent.
- The legacy branch-protection endpoint returns 404 because protection is owned
  by the active ruleset.

## Observed proof

- A direct push of Trellis closeout commits to `2.x` was rejected with GH013:
  changes must be made through a PR and four required checks were expected.
- PR #131 passed the four ruleset checks and merged through the protected path
  as merge commit `c50dbd3489130e0ceee2313d881f22709629dd20`.

## GitHub constraints

- GitHub states that pull-request authors cannot approve their own pull
  requests:
  https://docs.github.com/en/pull-requests/how-tos/review-pull-requests/approving-a-pull-request-with-required-reviews
- CODEOWNERS must be users or teams with write access, and required CODEOWNER
  review blocks matching changes until an owner approves:
  https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners
- Repository-level required reviewer teams are unavailable on user-owned
  repositories:
  https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets

## Conclusion

Path ownership and stronger required CI can be configured now. Independent
approval cannot be safely enforced until a second write-authorized human is
added; enabling it now would lock the only maintainer out because the ruleset
has no bypass. `GOV-413` must therefore remain blocked after this bounded slice.

## Implemented evidence

- PR #132 merged through the protected `2.x` path as
  `2807af5b6377728b0fb8ebce2979509b6a7e2d1f`.
- Exact-head Evaluation Truth run `32170444416` passed.
- Exact-head Native CI run `32170444458` passed, including the Linux, macOS,
  and Windows Rust foundations and the native change boundary.
- Ruleset `18800504` readback retained the exact branch target, empty bypass
  list, pull-request rule, deletion/non-fast-forward protections, stale-review
  dismissal, review-thread resolution, and four prior native checks.
- The only live ruleset addition is required check `Evaluation Truth V1` with
  GitHub Actions integration ID `15368`.
- Required approvals remain zero, CODEOWNER review remains disabled, and the
  Program Ledger continues to report `GOV-413` as blocked on a second eligible
  human reviewer.
