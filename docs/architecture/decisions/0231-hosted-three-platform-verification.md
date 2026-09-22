# ADR 0231: Hosted three-platform verification

- Status: Accepted
- Date: 2026-09-22
- Task ID: `CLI-410`
- Owners: Qiongli maintainers
- Supersedes: ADR 0230 (retained unchanged as historical evidence)
- Authority: the maintainer explicitly restored hosted three-platform execution.

## Decision

Restore the previous standard GitHub-hosted macOS, Linux and Windows build,
installation, Cargo and retained Desktop workflows. Standard hosted runners for
public repositories are free under [GitHub Actions billing](https://docs.github.com/en/billing/concepts/product-billing/github-actions).
The local-only policy's mandatory macOS receipt and maintainer-uploaded draft
handoff are retired. Local checks remain optional through existing owners.

The existing release owner again assembles all three target artifacts, verifies
combined installations, and publishes only after exact-source CI and separate
release authority. Community Alpha again requires the complete three-target
candidate and its protected authorization gate. Missing or failing evidence
never becomes a pass. Published releases remain immutable; no replacement of
assets, tags or protected refs is authorized by this decision.

Preserve ADR 0229: 2.0.1 retains actual v1 status and legacy Next Plugin IDs;
2.1 owns the deferred migration. Keep frozen source/content boundaries, user
models, approval/CAS, upgrade/retry/rollback and historical receipt verification.

## Verification and rollout

Restore the existing structural and release tests, including platform coverage,
exact-source checks and publication refusal cases. Local configuration changes
neither run hosted CI nor update remote rules or publish a release. Any prepared
local-only CI branches remain historical and must not be submitted. A fresh
packet is not qualified merely by restoring its tooling.
