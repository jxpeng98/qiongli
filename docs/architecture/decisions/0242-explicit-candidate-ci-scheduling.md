# ADR 0242: Explicit Candidate CI Scheduling

- Status: Accepted
- Date: 2026-11-13
- Task ID: `CLI-410`
- Owners: Qiongli maintainers
- Authority: the maintainer requests lower macOS CI usage and then authorizes
  remote synchronization and release, ending after publication-task dispatch.
- Supersedes: ADR 0227's main-push distribution/Cargo scheduling only.

Keep standard hosted Linux, macOS and Windows verification under ADR 0231 and
the four release targets under ADR 0232. Full distribution and Cargo
qualification run for explicit candidates and releases, not branch pushes.
The existing Release Automation owner dispatches distribution at the immutable
tag, avoiding an earlier duplicate build of the same main commit. Cargo keeps
pre-publication archive verification and post-publication registry installation.

Runtime PRs retain their three-platform tests and the existing required context
names. For non-runtime changes, lightweight Linux jobs report that native work
was skipped; they do not supply macOS/Windows runtime evidence. Boundary failures
remain blocking. Remote protection is unchanged.

Manual Native CI defaults to the CLI profile. The full profile explicitly opts
into retained Desktop, capacity, candidate lifecycle and Community Alpha work.
Its existing complete-platform dependencies and protected promotion gates remain
mandatory. This changes scheduling, not target support or acceptance criteria.

Distribution/Cargo Actions artifacts retain seven days. An expired candidate
must be regenerated when its artifacts are needed. Published assets, immutable
tags and existing valid caches are not removed by this policy.

Stable publication still requires the frozen remote main head, matching tag and
version, reviewed notes, successful exact-source distribution, verified downloads
and registry-specific credentials/environments. No local or Linux-only result
substitutes for those checks. A successful dispatch reports handoff only; a
maintainer request to stop monitoring does not make unobserved uploads successful.

Focused routing, target-roster, publication-refusal and workflow-syntax checks
cover the scheduling change. Actual hosted duration/storage effects require a
subsequent observation and must not be presented as already measured savings.
