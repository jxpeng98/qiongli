# Qiongli Native Workspace

This workspace is the canonical Rust-native product source for Qiongli 2.x.
It contains the product application, `apps/qiongli`, plus the accepted content,
config, runtime, platform-trust, and isolated Windows-security service
boundaries. The content crate defines the versioned resource-pack manifest,
frozen profile projections,
and bounded canonical source collector. It compiles those collected bytes into
an unsigned deterministic `.qlpack` core, verifies and loads that core entirely
in memory, and can atomically materialize a verified profile through a trusted
target capability.

## Dependency direction

The product dependency graph is one-way:

```text
apps/qiongli -> service crates -> contracts/platform primitives
```

Libraries must never depend on `apps/qiongli`, and the application remains a
thin mode dispatcher. Future service crates are added only with their first
real contract and tests. Production crates must not require Python, Node.js,
Cargo, or another language runtime to start.

`qiongli-content` freezes three projections: `skill-only`,
`marketplace-lite` (alias `lite`), and `full`. Its synthetic golden fixture
tests the format contract without binding the crate to the uncommitted working
copy of canonical content. FND-202B collects only the 12 allowlisted roots under
`content/`, normalizes and sorts portable paths, and rejects links, path
collisions, unsupported file types, and bounded count/size violations.
FND-202C writes a 20-byte versioned header, an RFC 8785 canonical manifest, and
the sorted uncompressed payload. SHA-256 entry digests feed a domain-separated
content root; SHA-256 over the entire unsigned core produces `pack_sha256`.
Input enumeration order, filesystem metadata, build paths, and wall-clock time
do not enter the result. FND-202D requires an expected whole-core SHA-256,
enforces bounded pack/manifest/entry/payload/path limits, rejects noncanonical
manifests, incompatible format/profile declarations, paths outside the
canonical roots, path/resource-kind mismatches, portable path collisions,
trailing payload, content-root drift, and entry-digest drift, then exposes
borrowed immutable bytes through an explicit profile projection. It accepts no
output path and performs no filesystem writes.

FND-202E keeps the write boundary separate from loading. A caller can request a
unique target inside an atomically created private temporary container (`0700`
on Unix) or can explicitly approve an absolute normalized target at a trusted
CLI, UI, or installer boundary. Explicit Unix targets reject group- or
world-writable ancestors; the private temporary factory permits only sticky
system temporary ancestors before its owner-only container. Model-generated
and MCP tool arguments must not cross that approval boundary. Materialization
refuses linked/reparse-point ancestors, unmanaged existing contents, receipt or
file drift, hard-linked managed files on Unix, and concurrent target ownership.
The create-new lock is
pinned to its file identity so an earlier owner does not unlink a replacement.
The selected profile and canonical `.qiongli-materialization.json` receipt are
written into a unique sibling staging directory that remains `0700` while
files are atomically created as `0600` on Unix. Files are closed before final
modes are applied; directories become `0755` only after writing completes. The
tree is then verified with bounded canonical-source/profile paths and logical
`0644`/`0755` modes before the prior managed tree is renamed to a backup and the
staging tree is promoted.
A promotion failure restores the backup before returning; handled pre-commit
failures remove their staging and lock artifacts. A failure after promotion is
reported explicitly as committed-with-cleanup-failure, including any remaining
backup cleanup path, instead of being indistinguishable from a pre-commit
failure.

FND-202F turns that content pipeline into a self-contained product resource.
The committed `qiongli-core.lock.json` freezes the accepted 1.19 metadata, 418
entries, content-root SHA-256, and whole-pack SHA-256. The `qiongli` Cargo build
script collects canonical sources, deterministically rebuilds the pack, and
fails closed unless both identities match the canonical lock. It writes only
the verified `.qlpack` and expected digest under Cargo `OUT_DIR`; normal builds
never rewrite tracked sources. `content/**` is fixed to LF through repository
attributes so checkout line-ending policy cannot create platform-specific pack
identities.

The application embeds both outputs at compile time. `EmbeddedContent` verifies
the static bytes against the expected digest and exposes profile inspection,
profile-scoped resource reads, and materialization through the existing target
capability. The canonical executable performs this integrity check at startup.
An integration test copies the executable outside the checkout and starts it
with an empty `PATH`, demonstrating that runtime content access does not read
the source tree or launch Python, Node.js, Cargo, or another external runtime.
Building Qiongli from source still requires the pinned Rust toolchain and the
canonical source tree; the distributed executable does not.

When a future content baseline is explicitly accepted, regenerate and review
the lock with native tooling before rebuilding the application:

```bash
cd packages/qiongli-native
cargo run -p qiongli-content --example update_qiongli_core_lock --locked
```

The expected digest establishes integrity only when it comes from a trusted
embedding or authenticated descriptor. Publisher signatures, trusted-key and
revocation policy, running-product compatibility enforcement, public UI/MCP
wiring, host installation, and adversarial same-user handle-relative
filesystem hardening remain separate successor work.

The version 1 header is `QLPACK\0\0`, followed by a little-endian `u32` format
version and a little-endian `u64` manifest length. Payload offsets start after
the manifest. The content-root preimage is
`qiongli:resource-pack:content-root:v1`, one NUL byte, the little-endian `u64`
length of the canonical entry-array JSON, and those canonical bytes. The
manifest schema keeps numeric fields within the JCS safe-integer range.

The existing `packages/qiongli-lite-mcp` crate remains a migration oracle and
compatibility package. Native functionality will be extracted into shared
workspace crates rather than copied into a second implementation.

## R2A shared Lite boundary

`qiongli-runtime` owns the first extracted Lite boundaries: a strict parser for
the 12 public Contract v2 Lite definitions (11 canonical typed identities) and
bounded newline/Content-Length stdio framing. The config-wizard compatibility
name resolves to the same typed handler identity as the canonical name. Runtime
errors expose only stable reason codes, and framing limits messages to 8 MiB
and headers to 64 KiB.

The canonical app enables the embedded-content adapter and proves that the
registry loads through the already verified `marketplace-lite` projection. The
old Lite package uses the same parser, identity resolver, and framing code
through thin compatibility modules. This checkpoint does not add an MCP mode
to the native executable and does not migrate provider, evidence, Zotero, or
orchestration-preview behavior.

## R2B shared provider kernel

`qiongli-runtime` now owns the five-provider access/status model, bounded HTTP
policy, response normalization, concurrent search, diagnostics, deduplication,
cooperative cancellation, and deterministic search planning. Canonical search
requests cap queries at 4,096 bytes, per-provider results at 200, and combined
results at 1,000. Provider HTTP disables redirects and implicit proxy discovery,
uses 3-second connection and 15-second request timeouts, and reads at most
4 MiB per response.

The optional native-config adapter resolves secret references only through
`SecretStore`; it does not read legacy provider files or environment aliases.
The old Lite package retains those compatibility inputs at its edge, converts
resolved values into the shared in-memory access model, and re-exports the
shared provider/search implementations. Access values are zeroizing and
non-serializable; public status and failures remain redacted.

R2B still does not add `qiongli mcp serve`, provider-management CLI/UI, a
production secure-store backend, evidence/Zotero extraction, or an installable
native release. Canonical MCP availability remains closed until binary-level
initialize, tools/list, and tools/call tests pass.

## R2C shared evidence and Zotero services

`qiongli-runtime` now owns bounded literature evidence snapshots and the
accepted Zotero surface. Evidence input is capped at 2 MiB, 1,000 results, 32
container levels, and 100,000 JSON values. Credential-bearing keys are removed
recursively before the deterministic snapshot is returned; the compatibility
`cwd` is validated but never copied into output.

Zotero export validates at most 1,000 normalized literature records and returns
only selected CSL-JSON, RIS, BibTeX, and import-report contents in memory. The
combined result is capped at 2 MiB. RIS folds control and line-separator input,
while BibTeX escapes syntax-bearing characters so reference text cannot inject
new records or fields.

The shared probe accepts only bounded HTTP(S) loopback roots, normalizes them
to fixed `/connector/ping` and `/qiongli/ping` calls, disables redirects and
implicit proxy discovery, applies a five-second production timeout, and reads
at most 32 KiB from the companion response. It returns status and bounded
version metadata only. The old Lite package retains the historical
`QIONGLI_ZOTERO_*` environment mapping at its edge and otherwise re-exports
the shared implementation.

R2C does not write import files, mutate or search a Zotero library, install the
companion, add native Zotero settings/UI, or expose MCP mode in the canonical
binary. Returned import-file contents prove generation only, not a completed
Zotero import.

## R2D shared orchestration previews and typed dispatch

Every canonical Lite tool identity now projects exhaustively into a typed
config, literature, Zotero, or orchestration handler target. The two
orchestration targets dispatch only bounded route and task-plan previews. They
return the accepted Marketplace Lite safety flags with agent execution, shell
execution, and project writes disabled.

Route requests are capped at 4,096 bytes and accept only the six Contract v2
platform values. Task IDs and paper types are capped at 256 bytes, topics at
4,096 bytes, and current trimming behavior is retained. Unknown, missing,
mistyped, blank, unsupported, and oversized input returns static errors without
echoing private values. The old Lite preview module is now a shared-runtime
re-export and its server dispatches through the same typed projection.

The returned `qiongli mcp serve --transport stdio` value remains compatibility
guidance for the frozen 1.x Full runtime. R2D does not add that command or any
MCP mode to the native executable, launch agents or processes, write project
state, or provide native provider-secret configuration.

## R2E canonical native Lite MCP vertical slice

The canonical product binary now composes the embedded Marketplace Lite
registry, bounded line and `Content-Length` framing, JSON-RPC/MCP envelopes,
typed dispatch, and the shared R2 domain services through one closed command:

```text
qiongli mcp serve --profile lite --transport stdio
```

`marketplace-lite` is the exact profile alias. Both options are required;
unknown, duplicate, Full-profile, and non-stdio values fail before serving.
Once selected, stdout contains MCP responses only. The server supports
initialize, initialized notifications, ping, tools/list, and tools/call for the
12 frozen Lite public names.

Search-plan and literature-search inputs are now parsed in `qiongli-runtime`,
so the canonical server and old Lite compatibility adapter use the same alias,
provider, mode, and limit boundaries. Native global settings are converted to
the shared in-memory provider model. Secret references use only the explicit
`SecretStore` boundary; R2E has no production secure-store backend and never
falls back to environment credentials or the 1.x plaintext provider file.

Provider-status results use `<managed-native-config>` instead of a local path.
Valid provider-save and wizard calls return a fixed unavailable tool error
after strict validation, without writing config, opening a listener, launching
a browser, or echoing the supplied value. Zotero status is explicitly disabled
with import-file fallback and performs no loopback probe. Route and task tools
remain preview-only.

The copied-binary acceptance runs with an empty `PATH` and proves initialize,
all 12 listed names, bounded safe calls, secret/path redaction, and clean EOF
without Python or Node. This establishes the native development vertical
slice only. Signed launch grants, plugin/Desktop activation, target packaging,
provider-secret mutation, Full MCP, agents, UI, installer, and release
readiness remain R3 or later work.

## R3A install-plan and Lite launch-grant contracts

`qiongli-platform` now owns the first installation trust boundary. It validates
the exact product/version/channel/profile/OS/architecture/installer identity,
then verifies a bounded Ed25519-signed Lite launch grant against public keys
already trusted by the product composition layer. The signature covers
domain-separated canonical JSON plus binary and embedded-pack SHA-256 values,
allowed `cli`/`lite-mcp` modes, local Codex/Claude Code scopes, validity times,
and an anti-replay generation. The verified capability token retains the mode
and scope checked by the verifier, so it cannot be reused to plan a different
integration.

The same crate defines a bounded, strict `InstallPlanV1` preview with closed
target and symbolic-root vocabularies, typed materialize/plugin-source/Lite-MCP
operations, observed state, postconditions, inverse operations, approvals, and
host-owned outstanding actions. Plans reject traversal, nonportable paths,
unknown roots, duplicate or unsorted identities, mismatched ownership, stale
targets, non-Lite profiles, altered semantic digests, and operations without a
matching inverse. Plan ID and display timestamps do not affect the canonical
semantic digest; every semantic field does.

The source-built executable reports this boundary through:

```text
qiongli install status
```

It truthfully returns `launch_grant`, `preview`, and `apply` as `unavailable`
and labels `codex-local` and `claude-code-local` as `contract-only`. Tests
generate ephemeral signing keys; the repository contains no signing private
key or caller-selectable trust root. This checkpoint does not discover or
write host paths, register a plugin/MCP entry, mutate a private cache, create
receipts, or claim installation, activation, packaging, or alpha.1 readiness.
Those remain R3 successor gates.

## R3B managed resource transaction vertical

`qiongli-platform` now contains the first executor behind the R3A trust
boundary. It accepts only an already verified signed plan, an exact local
approval token, one explicitly approved owner-only `QiongliManagedData` root,
and bytes from a verified embedded resource pack. The initial executable
subset is deliberately one Marketplace Lite resource directory with a missing
precondition; arbitrary paths, caller-provided bytes, client configuration,
multi-operation plans, managed overwrite, and host actions are rejected before
persistent mutation.

Fresh apply writes one root-scoped immutable journal before delegating the
atomic resource tree commit to `qiongli-content`, verifies the complete tree,
and commits a canonical owner-only platform receipt last. Read-only verify
binds the active platform receipt to the canonical materialization receipt.
Repair restores only an absent target with the same semantic plan and install
identity; a present but drifted target remains a conflict. Remove and rollback
identity-pin and reverify the exact managed target, quarantine it, commit a
distinct lifecycle receipt, and then clean the quarantine. Proven pre-commit
failures restore absence or the active target; uncertain materializer
ownership, state-commit results, or rollback retain data and the journal and
fail closed.

`qiongli install status` reports receipt contract version `1` and the
transaction engine as `grant-and-approval-gated`. The source binary still has
no production launch grant, approved root, or executable plan, so
`launch_grant`, `preview`, and `apply` remain `unavailable`. R3B does not
discover or register Codex/Claude, write MCP/client configuration, activate a
plugin, perform an in-place upgrade, install into Marketplace/Desktop/cloud,
or produce a package or release.

## R1 command contract (retained)

The native executable composes the verified embedded pack and versioned global
config service through this first useful command surface:

```text
qiongli --help
qiongli --version
qiongli content --help
qiongli content list
qiongli content materialize --profile <profile> --target <absolute-path>
qiongli config --help
qiongli config show
qiongli config set --expected-revision <revision> --default-profile <profile>
qiongli status
qiongli doctor
```

Data commands emit a newline-terminated JSON object with `schema_version: 1`.
Usage failures return exit code 2, operation failures return exit code 1, and
public errors contain only allowlisted reason codes. Materialization paths,
config roots, environment values, provider identifiers, and document bytes are
not rendered. `config set` changes only the default profile, preserves provider
settings, and requires an optimistic expected revision. This `doctor` command
reports only the R1 embedded-content, global-config, and secure-store foundation;
it does not claim provider, MCP, installer, agent, or orchestration readiness.

Binary contract tests clear `PATH` before supported commands and also copy the
executable outside the checkout before listing and materializing embedded
content. This prevents developer or CI installations of Python, Node.js, Cargo,
or other tools—and a source-relative working directory—from masking a required
runtime dependency.

## Version and toolchain

`Cargo.toml` is the single native product-version source. The release channel
is explicit workspace metadata and must agree with the SemVer prerelease.
`rust-toolchain.toml` pins the build toolchain used by the Tier 1 CI matrix.

REL-201 exposes that identity through `scripts/release_version.py` and a
non-publishing dry-run. The dry-run writes a release plan, notes, a planned-only
target identity, and rollback metadata to an explicit directory outside the
checkout:

```bash
OUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-native-release.XXXXXX")"
python3 scripts/native_release_dry_run.py \
  --tag v2.0.0-alpha.1 \
  --out-dir "$OUT_DIR" \
  --json
```

This is planning evidence, not permission to publish. It does not build an
installer, modify Git, publish PyPI/npm or Marketplace records, or claim the
target-native, signing, SBOM, provenance, updater, and rollback gates owned by
later roadmap tasks.
The direct command leaves source-ref and cleanliness unassessed. Use
`scripts/release_automation.sh pre` from a clean `2.x` checkout when the plan
must carry an eligible source binding.

Run the foundation gates from the native workspace so Rustup applies the
workspace-local pinned toolchain:

```bash
cd packages/qiongli-native
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```
