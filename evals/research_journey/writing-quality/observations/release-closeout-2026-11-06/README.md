# Writing release closeout — November 6, 2026

This increment addresses the three remaining evidence gaps from the selectable
writing styles work: a real saved-preference restart, a whole-article human
assessment with actual reading/editing time, and an Antigravity return with a
Host-confirmed model identity. The user also requests release preparation and
regenerable build-cache cleanup. Local preparation does not establish publication
or program acceptance.

## Actual saved project and recovery

An isolated native project saves the public Tarlao2022 source packet, both complete
retained Chinese versions and a concise project preference at semantic revision 3.
The explanatory version remains available; a paragraph-only choice must not
replace the project preference. All saves use the existing preview, reviewed
digest, academic approval and filesystem approval owners. Missing approvals and
a changed handoff under the old digest refuse without changing the project or
configuration.

The first real read-only session exposes a runtime gap: Graph snapshot discovery
rebuilds the derived portfolio catalog and fails under read-only permissions.
Reading a historical stage summary alone cannot establish the current preference.
The implementation now exposes the existing revision-bound registered-artifact
reader as `project context read` for research state and stage handoff. It performs
no Graph/catalog rebuild or project/config write. Current source/manifest checks,
safe-path limits and refusal of stale revisions remain with the existing owner.

The original failed recovery is retained. An initial override attempt is excluded:
the coordinator ran a Graph diagnostic concurrently, creating derived catalog
state and invalidating that run's preservation assertion. Fresh independent
retries use the new direct read route and the user's unchanged configured model.
Their actual replies and checks are recorded in [observation.json](observation.json).
Both retries pass their scoped checks using `gpt-6-astra` / `xhigh` in 115.7 and
119.5 seconds. The [ordinary continuation](recovery-fixed.md) recovers concise;
the [local override](override-fixed.md) uses explanatory for one paragraph while
retaining the default and both versions. Each encounters one initial outer-output
truncation and completes separate/paged rereads. Native responses reconstruct the
exact saved summary and source bytes; project/config fingerprints and temporary
authentication cleanup pass. Coordinator source review finds no required meaning
repair. These are read-only isolated CLI sessions, not installed-Plugin acceptance.

## Complete bilingual comparison and Antigravity

The [anonymous comparison](anonymous-comparison.md) contains a complete analytical
example based on the nine-page public Doshi and Hauser (2024) article, with abstract,
introduction, related research/positioning and discussion in both languages.
This is a reused public source, not a new held-out paper, official translation or
private manuscript. The original and return remain separate and unchanged.

Antigravity 1.5.2 completes a real polishing turn in 73.1 seconds. Its actual Host
log identifies **Gemini 4 Argon (Medium)**; the invocation supplies no model or
effort override and preserves the existing settings. The stream is complete with
no external tool calls. This establishes model-labelled execution, not a quality
verdict or a comparison between different models.

M/N were randomized once after both texts were complete. The ordinary reader
reports a small overall difference and a slight preference for N. After identity
disclosure, the reader confirms completing both languages and describes M's
English as slightly more formal and N's as comparatively more informal. Actual
reading/editing duration was not reported. The frozen packet and every binding
verify before unsealing: N is the original; M is the Antigravity return.

The [human review and adjudication](human-review.md) separate overall preference,
register, source meaning and mechanical constraints. The previously blinded
model review and coordinator source-text check identify no required scientific
repair. Mechanical counting does find M's Chinese exceeds the 2400 limit (2458).
Separate local refinements pass: M has 2395 Chinese code points with its English
unchanged; N has three optional expression edits. The reviewed originals and
comparison remain byte-identical. Revised examples are not yet human-reviewed or
adopted into a project; the ordinary-reader response establishes no expert or
timed-efficiency claim.

## Validation and remaining release work

The observation and existing execution plan retain exact checks, initial failures
and retries. The first changed-content build correctly rejects its stale resource
lock; the supported owner regenerates it from canonical commit `1062203c`.
Initial draft/test inputs created under group-writable permissions are refused;
private fixture permissions resolve the input issue without relaxing the native
reader. Release-note verification naming is corrected without weakening its
validator.

Strict research-standard validation passes 6211/6211 with no warnings; 11 portable
routing checks, one context-parser test, two real CLI save/resume journeys, two
embedded-pack tests and 53 evaluation-helper/version tests pass. Format checks
pass. Existing write-approval, stale-digest and semantic-drift negative cases are
retained. The current execution plan owns program/index and package checks.

The release candidate is 2.6.0. Current-target archive/package installation checks
and cache cleanup are recorded separately at integration. Other target platforms,
remote distribution and publication require their own exact-source evidence and
authority. Full/Lite tools, research schemas, configured models and existing
write approvals remain unchanged.


The [release preflight record](release-preflight.json) adds Rust 1.99 compatibility
and bounded test synchronization repairs. It preserves the original workspace
permission failures, a corrected private-checkout run and the four controlled
process-namespace retries. The combined application evidence covers 268 passing
cases with three existing manual skips; 138 platform cases and final strict
Clippy pass. This is not a single full-suite result or a claim that the shared
host permits managed executable replacement. The first incomplete private-cache
copy and its corrected retry are recorded as coordinator work, not product faults.


The next release-mode CLI run on `38219e48` passes 46 cases and fails the legacy
inventory count: the isolated fixture inherits the live `CODEX_HOME`. Removing
only the inherited Host roots makes that unchanged case pass. The shared test
helpers now clear `CODEX_HOME` and `CLAUDE_CONFIG_DIR`; explicit custom-root tests
still set their own overrides. The complete release CLI and MCP suites then pass
47/47 and 9/9 with the actual parent environment retained. Format passes. This is
a test isolation repair; production discovery and custom Host paths are unchanged.
The previous release build was terminated (exit 143) before test results or assets;
its log records no compiler error and does not establish the termination cause.


### Local 2.6.0 candidate and cache cleanup

Clean source `1a1b6164` passes the supported release owner for
Linux ARM64: format, strict Clippy, 47 CLI and 9 MCP tests, extracted archive
smoke, clean npm/wheel installs and both Host Plugin packages. The five artifacts,
manifest and checksum list agree. The same extracted binary additionally passes
both existing saved-handoff/summary recovery journeys, including current-context
reads and stale-revision/semantic-drift refusal. Exact details are in
[release-qualification.json](release-qualification.json).

The first packaging attempt lacks `auditwheel`; the system Python also lacks
`ensurepip`. These environment failures remain recorded. The final retry uses
the existing Python and task-local auditwheel/patchelf/PyYAML tools. The host
build then correctly fails the glibc 2.35 audit because of two versioned glibc
2.39 symbols. The successful candidate uses an isolated Ubuntu 22.04/glibc 2.35
environment with CI Rust 1.97, no network or credentials, and its own target cache.
Host system packages and project dependencies remain unchanged.

After builders stop, cleanup removes only ignored debug dependency/incremental
directories in the original and private checkout: 8,816,647,564
logical bytes, with 8,545,734,656 bytes of observed disk space recovered.
All 45 retained release, executable, project and
evaluation files match their pre-cleanup hashes. Other target trees and user
configuration remain intact. The read-only cache-budget check passes afterward.

This is an unpublished current-target candidate. The subsequent ordinary-reader
feedback and verified unsealing are recorded above; timing remains unreported.
The next release increment is exact-source qualification across the advertised
targets under an authorized release workflow. No program acceptance, remote push,
tag, publication or announcement is implied. Later bookkeeping changes do not
change the named artifact source or reuse its evidence for different bytes.
