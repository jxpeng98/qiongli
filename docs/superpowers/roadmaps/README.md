# Qiongli Roadmap Authority

This directory separates current program sequencing from historical migration
records.

The public [GitHub Projects roadmap](https://github.com/users/jxpeng98/projects/1)
mirrors release Milestones and bounded Epic Issues for collaboration. It is a
derived view: exact task state and acceptance continue to follow the authority
rules below.

## Current authority

- [Qiongli 2 Research Harness Master Roadmap](2026-08-02-qiongli-2-research-harness-master-roadmap.md)
  is the sequencing authority for work after the current Alpha 3 release
  closure. Its program-governance section also defines the authorization,
  branch/push/PR/merge and release-publication policy; documenting a workflow
  does not itself grant permission to execute it.
- [Qiongli 2.0.0-alpha.3 Completion and Release Plan](../plans/2026-08-01-qiongli-alpha3-completion-and-release.md)
  remains the execution authority for the Alpha 3 A5-A9 release gates.
- [Qiongli 2.0.0-alpha.3 Acceptance Ledger](../acceptance/2026-08-01-qiongli-alpha3-readiness.md)
  remains the evidence authority for that release.
- Accepted files under `docs/architecture/decisions/` remain architecture
  authority. A roadmap may sequence an ADR follow-up, but it does not silently
  rewrite an accepted decision.

## Historical roadmaps

The dated Rust migration, unified-platform, Marketplace Lite, adaptive-subject,
and UI migration roadmaps remain useful design and acceptance history. They no
longer determine post-Alpha-3 priority when they conflict with the master
roadmap above.

Historical completion must be established from an acceptance ledger, exact
commit, or CI receipt. Unchecked Markdown boxes in an old plan are not an
authoritative backlog.
