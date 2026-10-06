# ADR 0229: Plugin v1 transition and 2.1 cutoff

- Status: Accepted
- Date: 2026-09-15
- Task ID: `CLI-403 / CLI-410`
- Owners: Qiongli maintainers
- Authority: the maintainer explicitly retains a transition in 2.0.1 and places
  removal of the Plugin source-status v1 interface in 2.1.
- Supersedes ADR 0228's release timing and immediate v2/stable-identity switch;
  its verified migration, cache retention and approval boundaries remain valid.

## Decision

`v1` here means the public CLI `plugin-source-status` interface. It does not
mean the entire Python 1.x product, MCP v1, or every historic receipt format.

| Release | Public source-status | Local / public Plugin identity | npm channel |
|---|---|---|---|
| Published 2.0.0 | Existing v1 | Existing Next identity | Existing stable routing |
| 2.0.1 transition | Continue v1 output and semantics | Retain Next local and platform IDs | Stable uses latest; prereleases use next |
| Planned 2.1 | Remove v1 output support after named consumer/upgrade checks; use v2 | Stable Qiongli; Alpha/Beta Qiongli Next | Stable latest; prereleases next |

The compatible 2.0.1 branch excludes ADR 0228's runtime change. No fake v1
projection may relabel an actual Qiongli installation as Next. Installed
package/executable versions own behavior; a mutable registry tag is never an
independent runtime identity setting. Published artifacts remain immutable.

The 2.0.1 release owner must observe its actual packaged v1 behavior and bind
that result to release evidence. Packet verification rejects missing or wrong
transition evidence for this new candidate while retaining historical packet
verification. Stable source/tag, target-native checks, asset hashes and official
publisher boundaries remain unchanged.

## Upgrade and retirement boundary

2.1 retirement removes v1 output support, not the ability to read verified old
installation receipts or migrate previous Plugin IDs. Keep each receipt's
existing predecessor support, immutable identity projection and transaction
checks. Keep historical schemas and fixtures as compatibility evidence.

Before shipping the cutoff, update supported consumers and verify old Next to
new stable Qiongli, repeat operations, cancelled approvals, stale configuration,
partial registration failure, retry and restoration. Preserve source/cache files,
research data and user model settings. Claude's manual conflict handling must
be explicit and checked; a Beta-only test cannot qualify the stable identity.
External v1-only scripts require documented adaptation to v2. The maintainer's
version decision does not represent completed consumer or three-platform tests.

## Recovery and release authority

2.0.1 retains the existing update/recovery path. For the later identity migration,
switching back may require exporting the previous CLI's Plugin and re-enabling
its ID; changing the binary alone does not undo Host registration. Keep this
limitation visible. Never delete old caches to simulate a successful migration.

Local implementation and review do not authorize push, tag, publication,
external catalog edits, live user Host registration or announcement. A release
decision must name the final source, verified target assets and channels.

## Acceptance checks

1. 2.0.1 actual CLI output and exported/packaged Plugin identity preserve v1/Next.
2. v2, mixed identities, absent transition evidence and substituted artifacts
   cannot qualify as this 2.0.1 patch; historical packets remain verifiable.
3. Stable/prerelease publication routing remains unchanged and independently
   checked against the exact source, version and complete target set.
4. 2.1 v1 retirement retains old receipt readers and requires its own supported
   consumer, installation, cancellation and recovery evidence.

Execution and observed completion belong to the current bounded plan and
program ledger, not this accepted policy decision.
