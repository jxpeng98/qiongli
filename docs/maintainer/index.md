# Maintainer

This section is for people changing the system itself rather than only operating it.

## Maintainer Path

- [Maintainer workflow](/maintainer/claude-overview)
- [Architecture](/architecture)
- [Conventions](/conventions)
- [Local Desktop Development and Packaging](/development/local-desktop-build)
- [Repository Structure](/development/repository-structure)
- [Naming Policy](/maintainer/naming-policy)
- [Release Branch Policy](/maintainer/release-branch-policy)
- [Extend Qiongli](/advanced/extend-qiongli)
- [Publish native packages](/advanced/publish-pypi)

## Day-to-day maintenance

Find the source that owns the behaviour, run affected checks and update both
languages. Native work integrates on `2.x`; reviewed changes advance `main` when
authorized. Desktop information remains for maintenance. The old Python branch
has its separate compatibility and critical-fix scope.
