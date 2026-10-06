# ADR 0230: Manual local macOS verification

- Status: Accepted
- Date: 2026-09-15
- Task ID: `CLI-410`
- Owners: Qiongli maintainers
- Authority: the maintainer explicitly moved all macOS checks to manual local
  execution with retained receipts, preserving automated Linux/Windows Actions.
- Supersedes the hosted macOS execution requirement in ADRs 0220/0221/0227;
  target coverage, exact-source evidence and publication authority remain required.

## Decision

No workflow in the current source schedules a hosted or self-hosted macOS job.
Native, legacy diagnostic, installation, Cargo and retained Desktop checks run
on macOS manually through their existing owners. Linux and Windows continue
on GitHub Actions. Removing a job is not evidence that its checks passed.

Ordinary native distribution runs build and qualify Linux/Windows assets.
The maintainer builds macOS at the same clean commit, assembles all three
targets locally with the existing packet owner, and runs its `qualify-macos`
mode. That mode checks combined npm/wheel installs, both Plugins and actual
Cargo archives, then binds the observed receipt to the complete packet. Missing,
failed, stale or different-source local evidence blocks publication.

Under separate release authority, the maintainer uploads only the verified
packet and checksum files to a draft Release at the existing immutable tag.
The existing publisher verifies that draft, dispatches Linux/Windows combined
installation checks and requires successful receipts for the exact manifest
digest. It rechecks the draft and source refs before publishing. A successful
two-target build alone never qualifies a release. Published releases remain
immutable; registry environments, channels and authority stay unchanged.

Draft discovery needs push access. Only the small Linux download job receives
`contents: write`; it checks out no repository and executes no candidate code.
Installation jobs retain read-only permissions. See GitHub's
[draft visibility rule](https://docs.github.com/en/rest/releases/releases#list-releases).

Retained Community Alpha target rebuilds remain available on Linux/Windows;
three-target aggregation and its separately authorized promotion run locally
with the existing Rust examples. They must still require macOS evidence.

## Verification and rollout

Keep negative checks for stale source/packet, missing platform evidence,
failed installs, wrong tag/main, unverified attachments and attempts to replace
published releases. Document the manual commands and receipt locations in the
[local macOS guide](../../development/local-macos-checks.md).

The remote `2.x` ruleset 18800504 still required the macOS status context when
read on September 15. A separately authorized remote synchronization must
remove only that retired context, retaining Linux/Windows, the boundary check,
Evaluation Truth and the PR/ref protections. Do not add a dummy green macOS job.
Local changes do not update remote maintenance branches, rules or published tags.

Rollback requires an explicit policy change before restoring hosted macOS jobs;
failure of a local check must never trigger an automatic cloud fallback.
