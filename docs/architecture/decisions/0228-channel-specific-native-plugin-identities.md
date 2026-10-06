# ADR 0228: Channel-specific native Plugin identities

- Status: Accepted
- Date: 2026-09-15
- Task ID: `CLI-403 / CLI-410`
- Owners: Qiongli maintainers
- Authority: the maintainer requests Qiongli for stable installations and Next
  for Beta/npm next installations, including migration of existing Plugin IDs.
- Supersedes ADR 0227's shared stable/prerelease Plugin identity policy for the
  CLI-local and public Marketplace lanes. Signed Desktop integrations retain
  their maintenance contract.

The validated executable/package version owns the identity. Stable versions
use `qiongli` and display Qiongli; Alpha/Beta use `qiongli-next` and display
Qiongli Next. Existing npm publication routing already maps these versions to
latest/next. A mutable npm tag is not a second runtime setting: installing an
explicit version produces that version's Plugin identity.

Local Plugins keep the `qiongli-cli-local` marketplace and use
`qiongli@qiongli-cli-local` or `qiongli-next@qiongli-cli-local`. New default
directories and MCP keys follow the channel. Verified existing `qiongli` and
`qiongli-next` source directories may be reused without a directory rename.
Workflow shortcut names, research records, accounts, models and tool profiles
do not change. There remains one selected active Qiongli channel per Host.

Reuse the existing two-step file and Host confirmations. Codex previews and
disables the exact previous enabled Qiongli IDs through its version-checked
configuration API before installing the selected ID. Previous caches remain;
the source directory changes only under its separately approved file plan.
Cancellation leaves Host configuration unchanged, although the separately
approved source export can hide the old ID from the catalog. Read the exact
sibling ID from Codex's official user configuration as well as its inventory,
so a retry still disables that old entry under the same version/hash checks.
If registration fails after disable, report it and retain retry recovery.
Switching back to a removed catalog ID requires exporting the previous CLI's
Plugin before re-enabling it. Claude continues to require manual disabling of
the conflicting entry. Unknown sources,
modified files, wrong scope, stale configuration and changed plans fail closed.

Local bundle receipt schema 4 binds the channel-specific manifest, marketplace
entry and MCP key. Schemas 2/3 retain their historical Next identity, including
the published stable 2.0.0 source, and remain verifiable before receipt-bound
update/removal. Signed maintenance bundles keep their Next identity.

The CLI `plugin-source-status` contract moves from v1 to v2, classified
`migratable-breaking`: v2 reports either observed Plugin ID, or the selected
channel when the source is absent. Preserve the v1 schema and fixture. Existing
CLI/Host consumers read verified legacy sources and both identity strings;
the producer generates the v2 schema and fixture from Rust. Managed-operation
plans, approval digests and project/MCP contracts retain their existing versions.

Public Marketplace receipt schema 3 uses `qiongli-<platform>` IDs, `qiongli`
MCP keys and `qiongli-<host>-plugin-...` archives for stable releases; prereleases
retain their Next equivalents. Verification still reconstructs immutable
schema-1/2 projections, including old stable Next filenames and index entries.
Duplicate old/new archives and mixed identities are rejected. External catalogs
consume the generated platform index; catalog changes require separate scope.

Checks cover stable/Alpha/Beta names, old receipts and archives, mismatched
identities, complete file hashes, approval refusal, drift and stale config.
This is a source change for a subsequent release; published 2.0.0 assets,
tags, installed user Plugins and accepted program evidence are not rewritten.
Publication and installed-session acceptance remain separately scoped.
