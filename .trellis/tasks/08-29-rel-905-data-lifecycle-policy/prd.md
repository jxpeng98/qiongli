# REL-905 data lifecycle policy

## Goal

Publish the authoritative data ownership, backup, export, uninstall, and end-of-support policy, including the existing 90-day post-Stable 1.x support window.

## Requirements

- Publish one user-facing policy that distinguishes user-owned project data,
  Qiongli-managed global state, operating-system credential storage,
  Host-managed integrations, and external-provider data.
- State the exact native 2.x project and global-state backup boundaries, and
  distinguish a complete backup from the privacy-filtered portable project
  export.
- Document the supported portable export entry point and the data classes it
  intentionally excludes.
- Separate removal of receipt-owned binaries, Plugins, Skills, and client
  entries from intentional deletion of retained projects, global state, and
  credentials. No uninstall path may be described as deleting user data.
- Publish the existing 1.x maintenance boundary: `v1.19.0-beta.1` is the final
  planned feature-bearing 1.x release, its planned support window ends 90 days
  after Qiongli 2 Stable, and Alpha/Beta or this policy do not start that clock.
- Keep English and Simplified Chinese policy pages discoverable from the Guide
  indexes and VitePress sidebars.
- Back every normative product claim with current repository behavior or the
  existing release-branch policy; do not promise a future backup, purge, cloud
  retention, or credential-export feature.

## Acceptance Criteria

- [ ] English and Simplified Chinese pages cover ownership, backup, export,
      uninstall/deletion, and end-of-support.
- [ ] The policy names `<project>/.qiongli/v2`,
      `<user-home>/.config/qiongli/v2`, `$QIONGLI_CONFIG_HOME/v2`, and the
      approved `qiongli project export` flow without exposing private paths.
- [ ] Portable export is explicitly not described as a complete backup and its
      credential, session/chat/transcript, client-state, cache/build, and
      private-runtime exclusions are documented.
- [ ] Removal preserves projects, global state, credentials, and frozen 1.x
      sources unless the user performs a separate deliberate cleanup.
- [ ] The 90-day post-Stable 1.x rule matches the maintainer release policy and
      does not invent a calendar end date.
- [ ] A focused standard-library test protects the policy, navigation, and
      support-window invariants; VitePress builds successfully.
- [ ] Exact-head Slice CI passes without candidate packaging, promotion, or
      publication.

## Notes

- Non-goals: new backup/restore implementation, full-data export, automated
  purge, remote-service retention terms, REL-906 source retirement, release
  qualification, promotion, and publication.
