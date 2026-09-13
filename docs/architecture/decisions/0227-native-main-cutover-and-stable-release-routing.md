# ADR 0227: Native main cutover and stable release routing

- Status: Accepted
- Date: 2026-09-13
- Task ID: `CLI-410`
- Owners: Qiongli maintainers
- Authority: the maintainer requests consolidation of 2.x into main, followed
  by checks and review of the Python release/build lane.
- Supersedes the main-as-legacy branch policy and the prerelease-only scope of
  ADRs 0219/0220/0223 for native CLI distribution. Accepted architecture and
  migration baselines remain historical evidence and are not rewritten.

`main` carries the integrated Rust-native product and stable release source.
`2.x` remains the development and prerelease line. Preserve `dev` as historical
handoff and `release/1.x-python` as the critical-fix-only compatibility line;
neither receives the native implementation. Review outstanding branches for
unique behavior before integration; generated distribution refs and stale
bookkeeping are not feature branches to merge into source.

Reuse the existing native CLI, npm, wheel and Cargo owners. Local qualification
accepts clean main or 2.x. Native CI and Evaluation Truth cover PRs to either;
main pushes qualify three-platform CLI distribution and Cargo source installs.
Legacy CI targets dev and the Python maintenance branch. The legacy TestPyPI
builder is restricted to that maintenance branch; native wheels continue to
package the native executable. Main workflow changes are not a maintenance
branch backport.

Publication still requires an explicit Actions dispatch at an immutable tag,
matching version/source, reviewed notes, successful three-platform qualification
and verified assets. Stable tags must equal remote main when publication starts;
keep main frozen during that release. The existing publisher marks stable
GitHub Releases as latest and chooses npm latest; Alpha/Beta use prerelease and
npm next. PyPI and Cargo retain their existing normalized version and credential
gates. Local invocation, a wrong main/tag, failed CI and mismatched assets refuse
publication. A merge or build success does not publish or accept the program.

Stable Marketplace archives retain the existing qiongli-next target IDs and MCP
server key so installation identity does not change merely with release channel.
Stable display names use Qiongli; prerelease names and immutable archive
projections stay unchanged. Every archive still binds its native executable,
source pack, Host and target, with no language runtime dependency. External
catalog promotion is separately scoped; no existing ref or asset is replaced.

Focused checks cover stable/prerelease routing, rejected publication conditions,
stable Plugin archive round trips and legacy isolation. After merging, qualify
the actual current-target standalone CLI, npm/wheel installs and Plugin archives.
Cross-platform CI, live Host/Hook/Graph journeys and formal stable acceptance
retain their own evidence requirements. This decision does not mark them passed.
If integration fails, retain the previous main commit and diagnose the merged
candidate; use a reviewed revert if needed, never rewrite published history or
research data. Release rollback preserves prior assets and user data.
