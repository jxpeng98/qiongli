# ADR 0240: DeepSeek Latest Release and Terminal Installation

- Status: Accepted
- Date: 2026-10-09
- Task: CLI-402
- Authority: the maintainer requests the normal `dsh plugin --profile desktop
  add qiongli@<latest exact version>` flow, less scrolling and fewer repeated inputs.
- Supersedes the DeepSeek CLI-version binding, package command arguments and
  captured package-process behavior in ADR 0238, and the default per-stage success
  lines/full diagnostic fields in ADR 0239. Other Hosts retain those contracts.

Resolve the public npm `qiongli/latest` metadata before approval using bounded
HTTPS with no redirects. Accept only canonical stable SemVer and the expected
DSH package identity. Failure refuses this Host rather than silently falling
back to a cached/CLI version. Freeze the exact version in the installation plan
and digest; later tag changes cannot change approved argv. Query metadata each
installation; the official manager continues to own download caching.

Keep PATH discovery and digest-locked executable resolution. A Desktop-installed
`dsh` symlink may resolve to the official executable inside the application;
show that identity without replacing it with a different npm/global CLI. Default
to `desktop`, require it to be initialized, and allow explicit `web`/custom profile
selection. Do not silently redirect a Desktop installation to web.

After explicit trust and executable/profile CAS recheck, run literal argv
`plugin --profile <profile> add qiongli@<version>`. Do not add registry or
ignore-scripts flags. DSH owns package registry, dependency and script policies.
Package commands inherit terminal streams, environment
and caller working directory, with reviewed HOME/DSH_HOME roots propagated.
No capture limit or package-process deadline interrupts native prompts; Ctrl-C
retains foreground process-group behavior. The read-only version probe and profile initialization remain
bounded and captured; `--dump-config` must not print profile contents. Qiongli suspends its progress line before terminal handoff;
it neither captures, filters, persists nor replays DSH's native output. Other
Hosts retain bounded capture/redaction. Native DSH rendering is not controlled by
Qiongli's `--plain`/`--verbose` flags.

Compact approval displays exact version/argv, executable, profile directory and
language, plus the effect on bundle registration and DSH-managed policy. Verbose
adds full plan/digest diagnostics and the twelve stage results. Failures always
identify the stage; ordinary failures continue the batch, input/output interruption
stops it. Single-Host DSH installs reuse valid stored language or locale without
a separate prompt; explicit `--language` overrides it. Language writes retain
their CAS owner. Corrupt preferences refuse before installation.

Postconditions check selected-profile bundle registration, the approved package
version/entry-point declarations and a bounded reconstruction of that version's
source pack from installed files. Validate manifest paths, sizes, hashes,
canonical manifest, root and pack digest through the shared resource-pack owner.
The normal pnpm package-root link is allowed; links inside its DSH resource tree
are refused. This checks source-content consistency, not registry origin,
generated JavaScript equivalence or native binary attestation. Download/package
integrity remains DSH/pnpm's responsibility; the release packaging owner retains
full projection qualification. Never compare a different release to the running
CLI's embedded pack or claim live session tools from installation alone.

Validation uses local HTTP, synthetic DSH and isolated package fixtures, including
cross-version content, tampering, stale/declined approvals, failures and compact
output. The user's latest Mac failure remains unreproduced. Rollback restores the
previous installer; it does not undo manager-owned changes to an installed profile.
