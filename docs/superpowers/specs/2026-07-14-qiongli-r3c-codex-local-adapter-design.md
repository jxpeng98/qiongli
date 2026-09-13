# Qiongli 2.0 R3C Codex Local Adapter Design

Date: 2026-07-14  
Status: frozen for implementation  
Roadmap slice: `R3C / INT-201`  
Scope: current-user Codex personal marketplace source registration

## Outcome

R3C adds the first real host adapter above the R3A install contracts and the
R3B managed-resource transaction. It registers one receipt-verified local
Qiongli plugin source in the current user's Codex personal marketplace.

The adapter supports read-only discovery and preview plus approval-gated
apply, verify, repair, remove, and rollback. Registration means only that the
source is available in the ChatGPT desktop plugin directory. It does not mean
that the desktop client installed, cached, enabled, trusted, or activated the
plugin.

## Documented Codex Boundary

The implementation follows the current official Codex plugin contract:

- the personal marketplace is `~/.agents/plugins/marketplace.json`;
- local entries use a `./`-prefixed source path relative to the marketplace
  root;
- the ChatGPT desktop app performs installation and places its installed copy
  below `~/.codex/plugins/cache/...`;
- plugin enablement is client-owned state in `~/.codex/config.toml`; and
- installed plugins may contribute a manifest, skills, and MCP configuration.

References:

- <https://learn.chatgpt.com/docs/build-plugins>
- <https://learn.chatgpt.com/docs/extend/mcp>

Qiongli therefore owns neither the Codex plugin cache nor Codex enablement
state. R3C never writes either location. Public Marketplace submission and
review are separate release work.

## Fixed User Layout

The adapter accepts one caller-supplied, already resolved current-user home.
It does not consult model input, MCP parameters, arbitrary environment
overrides, or repository configuration.

The v1 layout is fixed:

| Purpose | Symbolic path |
| --- | --- |
| Personal marketplace | `<user-home>/.agents/plugins/marketplace.json` |
| Qiongli adapter state root | `<user-home>/.qiongli/plugins/codex` |
| R3B-managed plugin source | `<user-home>/.qiongli/plugins/codex/qiongli` |
| Marketplace source value | `./.qiongli/plugins/codex/qiongli` |

Only a user-scoped `CodexLocal` / `DesktopLocal` / Lite target is supported.
Repository marketplaces, Codex cloud, ChatGPT web local-file access, public
Marketplace publishing, arbitrary source paths, and upgrades or replacement
of a conflicting `qiongli` entry are rejected in this slice.

## Plugin Source Trust

The source must already be a healthy R3B materialization:

1. its parent is the fixed private Qiongli adapter state root;
2. `.qiongli-materialization.json` parses and verifies against the complete
   materialized tree;
3. the receipt profile is `marketplace-lite`;
4. `.codex-plugin/plugin.json` is a regular, receipt-covered file;
5. the manifest name is exactly `qiongli`, its version is valid SemVer, and
   its skills path is local and traversal-free; and
6. the canonical materialization receipt digest becomes the immutable plugin
   source digest in the install plan and registration receipt.

R3C adds a skills-only Codex manifest to the canonical embedded content. Lite
MCP wiring is deliberately not claimed by this registration slice. A later
adapter batch will bind the installed native executable into the plugin MCP
contract without adding Python, Node, or Rust runtime dependencies.

## Discovery And Preview

Discovery is read-only. It returns symbolic locations and typed states for:

- plugin source: `missing`, `ready`, or `invalid`;
- personal marketplace: `missing` or `ready`;
- registration: `absent`, `registered`, `conflict`, `drifted`, or
  `recovery-required`.

No serialized result, error, receipt, or debug representation contains the
absolute user home.

Preview requires a verified `CodexLocal` Lite launch grant and a healthy
source. It creates one deterministic `RegisterPluginSource` operation against
`CodexPersonalMarketplace`. The operation binds:

- the exact source receipt digest;
- the exact canonical marketplace entry digest;
- the observed missing or managed state digest;
- one ownership marker for install ID `qiongli-codex-user`; and
- the inverse exact-digest `RemoveManagedEntry` operation.

The exact approval vector is:

1. `filesystem-write`;
2. `client-config-change`; and
3. `host-trust`.

`host-trust` is required because the plan records
`install-or-enable-plugin` as an outstanding client action. The executor never
performs that action and the receipt preserves it as outstanding.

## Marketplace Merge Rules

The adapter accepts a missing document or a bounded JSON object whose
`plugins` member is absent or an array. It preserves unknown top-level fields,
the `interface` object, and all unrelated plugin entries.

The managed entry contains only documented marketplace fields:

```json
{
  "name": "qiongli",
  "source": {
    "source": "local",
    "path": "./.qiongli/plugins/codex/qiongli"
  },
  "policy": {
    "installation": "AVAILABLE",
    "authentication": "ON_INSTALL"
  },
  "category": "Education"
}
```

An existing entry named `qiongli` is accepted only when an active Qiongli
registration receipt exists and the entry digest, source digest, and ownership
all match. Otherwise preview and apply fail closed with a conflict. R3C never
adopts or overwrites an unreceipted entry.

## Transaction And Receipts

The private adapter state root contains:

- `.qiongli-codex-registration.json`: canonical active/lifecycle state;
- `.qiongli-codex-registration-journal.json`: one root-scoped immutable
  transaction journal; and
- `.qiongli-codex-registration.lock`: a current-process/user lock.

Apply uses compare-and-swap semantics:

1. revalidate the verified plan, exact approval, source receipt, home and
   state-root identity, and current marketplace digest;
2. acquire the private adapter lock;
3. persist and sync a journal containing the bounded previous marketplace
   document and expected next digest;
4. atomically replace the marketplace document;
5. re-read and verify the exact entry and document digest;
6. atomically commit the canonical registration state; and
7. remove the journal.

If a post-activation state commit fails, the in-process transaction restores
the journaled prior document. If safe restoration cannot be proven, it retains
the journal and returns `recovery-required`. A surviving journal blocks every
mutation; automatic crash recovery is not introduced in R3C.

Verify revalidates the source, receipt, exact marketplace entry, and current
document. Repair restores only a missing receipt-owned entry. Remove and
rollback delete only the exact receipt-owned entry and write distinct lifecycle
receipts. Any entry drift or ambiguous ownership is preserved and rejected.

## Security And Compatibility

- Reads and writes are bounded to 1 MiB documents and receipts.
- Existing path components, source files, marketplace files, and state files
  may not be symlinks or platform reparse points.
- Unix objects must be current-user-owned and not writable by group or other;
  private state files/directories use `0600`/`0700`.
- Windows-created private state uses the existing protected-DACL boundary.
- Writes use private staging/recovery files, atomic replacement, directory
  synchronization where supported, and post-commit verification.
- Unknown marketplace content is preserved semantically; malformed documents,
  non-array `plugins`, duplicate `qiongli` entries, and conflicting entries
  fail closed.
- No subprocess, shell, Codex CLI, network request, plugin cache write, or
  client config enablement occurs in the adapter.

## CLI Truthfulness

`qiongli install codex status` exposes only read-only discovery with symbolic
locations. The ordinary source build continues to report production launch
grant, preview, and apply as unavailable. `qiongli install status` may report
that the Codex adapter engine exists, but must not report an installed or
active plugin without receipt and client evidence.

## Acceptance Gate

R3C is complete when Rust tests prove:

1. discovery is side-effect free and does not leak absolute paths;
2. preview is deterministic and binds the exact source and observed state;
3. missing or partial approval is rejected;
4. apply preserves unrelated marketplace fields and entries;
5. identical replay is idempotent;
6. verify detects source, receipt, entry, and document drift;
7. repair restores only a receipt-owned missing entry;
8. remove and rollback delete only an exact receipt-owned entry;
9. malformed, duplicate, linked, oversized, conflicting, and recovery states
   fail closed;
10. the canonical embedded `marketplace-lite` projection contains a valid
    skills-only Codex manifest; and
11. local full Rust, boundary, focused Lite, Linux, macOS, and real Windows CI
    gates pass on the exact implementation head.

R3C completion is not evidence of Desktop installation, cache creation,
enablement, MCP activation, Marketplace publication, cloud availability,
packaging, UI, updater, or release readiness.
