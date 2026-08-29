# REL-905 data lifecycle policy design

## Boundary

REL-905 is a public-policy Slice, not a storage feature. It publishes the
contract already implemented by the native project, config, credential,
portable export, managed integration, and release-governance owners.

## Authority and placement

- `docs/guide/data-lifecycle.md` is the English user-facing authority.
- `docs/zh/guide/data-lifecycle.md` is its Simplified Chinese counterpart.
- Guide indexes and VitePress sidebars make both pages discoverable.
- `docs/maintainer/release-branch-policy.md` remains the authority for the 1.x
  maintenance branch; the lifecycle policy repeats its user-relevant window
  without changing it.

No JSON schema, App API field, CLI command, or storage migration is added.

## Policy model

The document uses one ownership table and four operational sections:

1. Back up complete project roots and the resolved global `v2` state root while
   Qiongli and attached agents are stopped; treat credentials as a separate
   operating-system store.
2. Use portable project export only for bounded transfer. It intentionally
   omits private runtime, credentials, client state, sessions, transcripts,
   caches, and build output, so it is not a complete backup.
3. Treat uninstall/removal as receipt-owned product cleanup. Retained user data
   is deleted only by a separate deliberate user action after verification.
4. Start the existing 90-day 1.x countdown only when Qiongli 2 Stable is
   actually published; prereleases and REL-905 do not start it.

External Agent hosts and research providers retain their own data policies;
REL-905 does not make claims on their behalf.

## Executable guard

One dependency-free unittest reads the English/Chinese pages, Guide indexes,
VitePress config, and maintainer policy. It checks the required topics, exact
symbolic paths/command, portable-export warning, removal preservation, and
support-window source. Evaluation Truth runs that test on every `2.x` PR.

## Compatibility and rollback

This change is documentation and CI-only. It changes no runtime, public schema,
storage bytes, installer behavior, or release authorization. Rollback is a
normal revert of the policy, navigation, test, and workflow line.
