# ADR 0214: Receipt-Owned Local Workflow Variants

- Status: Accepted
- Date: 2026-08-18
- Task ID: `ARC-214`
- Owners: Qiongli maintainers
- Decision scope: local customization of packaged Workflow and Skill Markdown
- Supersedes in part: ADR 0213's canonical-only Skill-byte assumption
- Retains: ADR 0205's immutable deterministic resource pack and ADR 0213's
  fixed official Host CLI, Host-owned cache, and fresh-Ready boundaries

## Context

The packaged App can install and verify exact canonical Skills and Codex/Claude
Plugin bundles, but its content-customization surface could not change the
instructions those managed destinations actually consume. Editing an installed
tree directly correctly produced drift and broke exact update, removal, cache,
and Ready evidence.

Users need bounded instruction customization without gaining authority to edit
Plugin identity, MCP configuration, executable arguments, schemas, binaries,
canonical repository content, or Host caches.

## Decision

### Canonical content remains immutable

The embedded resource pack remains the sole canonical parent. One local variant
may override only existing UTF-8 Markdown resources projected as
`workflow/SKILL.md` or `skills/**/*.md`. Plugin manifests, MCP declarations,
tool schemas, standards, roles, templates, executable content, and arbitrary
paths remain read only.

Each override is bound to its canonical path and base digest. Per-file and total
byte limits, control-character rules, contained regular-file checks, and the
canonical parent pack identity fail closed.

### One private receipt owns the variant

The native configuration owner stores one verified local variant under the
private Qiongli v2 state root. Its canonical JSON receipt binds the parent pack,
resource digests, deterministic variant digest, monotonic revision, and exact
file inventory.

Replace and reset operations use preview, compare-and-swap inputs, a digest-bound
confirmation token, private staging, atomic promotion, and exact reload. Stale
drafts, link substitution, drift, invalid content, and cleanup ambiguity do not
write managed destinations.

Multiple named variants, sync, collaboration, sharing, and a general editor are
outside this decision.

### Managed outputs derive from the selected variant

Standalone Skills and Codex/Claude Plugin composition accept the verified
variant as an optional projection input. Their receipts retain the canonical
pack identity and add the exact variant digest. Runtime binaries, MCP contracts,
and Plugin manifests stay byte-identical to the canonical projection.

Canonical compose entry points remain thin no-variant wrappers. Existing
canonical receipts remain readable. Verification and exact removal compare the
identity recorded by each destination rather than silently adopting the newest
variant.

### Editing never silently activates content

Saving a variant changes only the private variant owner. Installed standalone
Skills and Host integrations become update- or repair-required until the user
approves reconciliation.

Existing receipt-owned outputs are replaced through the current staged
reconciliation journal. The same transaction updates dependent registration
receipts and managed standalone Skills. It never edits Host caches. After
activation, the App runs only ADR 0213's fixed official Codex or Claude repair
plan and discards prior observations.

If the official Host plan or exact verification fails, the native reconciliation
rolls back receipt-owned outputs when recovery is still safe. Unmanaged and
drifted destinations remain untouched.

### Exact selected content owns Ready

Ready requires all of the following from the same fresh verification boundary:

- the installed source and registration bind the selected canonical or local
  variant receipt;
- the Host cache bundle receipt exactly matches the managed source receipt;
- Plugin identity, version, enablement, scope, Skill inventory where exposed,
  and Full MCP evidence satisfy ADR 0213; and
- the App labels current content provenance as Canonical or Customized.

Command success, a stored draft, an older canonical receipt, or stale Host
evidence is not Ready. Reset selects canonical derivation and requires the same
explicit reconciliation before canonical content becomes active again.

## Alternatives considered

### Edit installed Plugin or Skills trees directly

Rejected because it converts managed content into drift and weakens safe update,
removal, and cache comparison.

### Mutate canonical embedded content or generated Plugin mirrors

Rejected because local preferences would change signed product inputs and
destroy reproducible packaging.

### Permit Plugin manifest or MCP editing

Rejected because that expands execution authority instead of customizing
research instructions.

### Write refreshed bytes into Host caches

Rejected because Host caches remain private Host state. Only the official Host
CLI may install or refresh them.

## Consequences

- App edits affect the content that managed Skills and Plugins can actually
  consume after one explicit reconciliation.
- Canonical product inputs remain reproducible and independently verifiable.
- Receipts and transactions gain one optional variant identity and one private
  variant state owner.
- A saved edit may temporarily show repair-required; this is intentional and
  prevents silent behavioral changes.
- Wider authoring, sharing, and synchronization features remain deferred.

## Security and privacy

- Only bounded Markdown text crosses the editing boundary.
- No arbitrary path, command, Plugin identity, MCP declaration, binary, secret,
  prompt history, or Host output is stored in the variant receipt.
- Preview and confirmation revalidate revision, base digest, variant digest,
  destination ownership, fixed Host plan, and prepared transaction journal.
- Cleanup and rollback operate only on exact transaction-owned paths and
  receipt-verified managed destinations.

## Rollback

Reset the verified variant, then explicitly reconcile the affected destinations
through the same transaction and official Host flow. Reverting product code
does not delete an existing local variant or unmanaged content; older binaries
fail closed if they cannot verify the newer state.

## Acceptance tests

1. Allowed Markdown replace/reset is deterministic, revision-bound, size-bound,
   link-safe, and preserves unrelated files.
2. Standalone Skills and both Plugin bundles contain the selected instruction
   bytes while manifests, MCP declarations, and packaged binaries remain exact.
3. A stored variant marks canonical managed destinations repair-required and
   never rewrites them before confirmation.
4. Reconciliation updates receipt-owned source, registration, standalone Skills,
   and then the Host cache only through fixed official Host CLI commands.
5. Fresh managed/cache receipt equality plus Plugin/Skill/MCP probes is required
   before Customized Ready; reset repeats the journey to Canonical Ready.
6. Stale preview, invalid content, receipt drift, command failure, or observation
   mismatch stays non-Ready and preserves unrelated state.
