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
copy of canonical content. FND-202B collects only the 14 allowlisted roots under
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
The committed `qiongli-core.lock.json` freezes the accepted 1.19 metadata, 420
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

## R3C Codex personal marketplace adapter

`qiongli-platform` now contains the first `CodexLocal` host adapter. It derives
only the documented current-user personal marketplace and a fixed private
Qiongli source location, validates that source through its complete native
plugin-bundle receipt, and exposes a deterministic one-operation
`RegisterPluginSource` preview. The plan requires the exact
`filesystem-write`, `client-config-change`, and `host-trust` approval vector and
retains `install-or-enable-plugin` as an outstanding client action.

The registration executor preserves unrelated marketplace fields and entries,
uses private canonical receipts and a root-scoped journal, and supports apply,
verify, absent-entry repair, remove, and rollback. Conflicting unreceipted
`qiongli` entries, malformed or oversized documents, source or receipt drift,
linked/insecure paths, partial approval, and ambiguous rollback fail closed.
The embedded `marketplace-lite` projection contains the canonical Codex
metadata template; it is not itself treated as an installable plugin package.

The source executable adds the side-effect-free command:

```text
qiongli install codex status
```

It returns symbolic paths and typed source, marketplace, and registration
states without creating directories or exposing the user home. General install
status labels the Codex adapter engine ready, while production `launch_grant`,
`preview`, and `apply` remain `unavailable`. Registration does not write or
claim the Codex plugin cache, enablement state, Desktop installation, MCP
activation, public Marketplace publication, or cloud availability.

## R3D native Codex plugin package

`qiongli-platform` now composes a complete target-specific Codex plugin from
the verified embedded Marketplace Lite projection, a verified `PluginBundle`
launch grant, and the grant-matched native Qiongli executable. The generated
root contains canonical `skills/qiongli-workflow` content,
`.codex-plugin/plugin.json`, `.mcp.json`, `bin/qiongli[.exe]`, and a canonical
receipt covering every other file, mode, size, digest, artifact identity,
signed-grant payload, and resource-pack identity.

The root MCP declaration launches that plugin-local executable as
`qiongli mcp serve --profile marketplace-lite --transport stdio`. It never
names Python, Node, Rust, Cargo, a package manager, or a user-global Qiongli
command. Composition uses a private sibling stage, a target lock, no-replace
promotion, and committed verification. Existing targets, unmanaged data,
links, reparse points, hard links, permission drift, extra or missing files,
receipt drift, and signed binary or content mismatch fail closed.

The normal Rust test copies the product binary, builds the full 420-entry
embedded projection into the generated plugin layout, and launches the bundled
MCP with an empty `PATH`. A separate explicit acceptance test uses an isolated
home and `CODEX_HOME`, validates the generated root with Plugin Creator, asks
the real Codex CLI to install it from the isolated personal marketplace,
verifies Codex's cache and enablement record, and launches the cached MCP with
an empty `PATH`. It does not modify the developer's normal Codex state.

The source executable still has no production signing grant or public package
command, so general `launch_grant`, `preview`, and `apply` status remains
`unavailable`. R3D proves local package composition and client activation; it
does not publish a public Marketplace entry, make a local executable available
to cloud/web runtimes, ship an alpha, add package upgrade/rollback, or complete
Claude, UI, Full MCP, agent, or orchestrator execution work.

## R3E Claude Code local integration

`qiongli-platform` now composes the same verified native Marketplace Lite
content into a target-specific Claude Code plugin. The generated package
contains `.claude-plugin/plugin.json`, `.mcp.json`,
`skills/qiongli-workflow`, `bin/qiongli[.exe]`, and a canonical receipt binding
the signed grant, artifact, embedded pack, binary, modes, sizes, digests, and
complete package tree.

The preferred direct target is `<claude-config>/skills/qiongli`. Discovery
accepts only an exact verified package, reports symbolic state through
`qiongli install claude status`, and treats unreceipted or drifted content as a
conflict. Verified removal first moves the exact package into a
transaction-owned quarantine and never adopts or overwrites unmanaged data.

The alternative local marketplace remains below Qiongli-managed data. Its
receipt-backed adapter provides preview, apply, verify, absent-entry repair,
remove, and rollback while preserving unrelated state. Claude Code—not
Qiongli—adds that marketplace, installs or enables the plugin, and owns its
registry, settings, and versioned cache. The plugin MCP command uses
`${CLAUDE_PLUGIN_ROOT}/bin/qiongli[.exe]` and needs no Python, Node, Rust,
package manager, checkout, or global Qiongli command at runtime.

An explicit acceptance test isolates both `HOME` and `CLAUDE_CONFIG_DIR`.
Claude Code `2.1.206` strictly validates and discovers the skills-directory
package, adds and installs the local marketplace package, copies the verified
tree into its isolated cache, launches all 12 Lite MCP tools with an empty
`PATH`, and removes the isolated plugin and marketplace. The normal gate keeps
this external-client test ignored and passes all 199 normal native Rust tests,
all 69 focused Lite compatibility tests, strict host Clippy, and Windows MSVC
workspace check/Clippy.

Production grants, mutating install commands, Claude Desktop, cloud/web
execution, public marketplace publication, managed in-place upgrade, release
artifacts, UI, Full MCP, agent execution, and orchestrator execution remain
unavailable.

## R3F native desktop manager

The canonical executable now has one Rust-native desktop mode:

```text
qiongli ui
```

`qiongli-ui` owns only bounded presentation models, typed intents and events,
stock egui views, and the eframe application. The product composition root
builds a read-only snapshot from the verified embedded pack, redacted global
configuration, the 12-tool Lite MCP contract, and Codex and Claude Code local
discovery. The snapshot contains typed states, counts, symbolic locations, and
fixed remediation codes; it does not contain concrete paths, email addresses,
API keys, secret references, environment values, or raw service documents.

The window provides Overview, Skills, MCP, Providers, Integrations, and
Diagnostics views. Refresh and provider/integration previews cross the typed
service boundary. Provider preview accepts only a transient public contact
email and clears it after submission. The production adapter validates that
input but keeps config writes and every install confirmation unavailable.
Closing the window does not persist eframe state.

The UI pins eframe, egui, and egui_kittest 0.35.0, using native wgpu and
AccessKit without a webview, JavaScript frontend, glow renderer, or persistence
feature. Headless AccessKit tests cover all six views, keyboard activation,
labelled input, non-echoing feedback, confirmation/cancel, progress and
recovery states, narrow and normal widths, and 100%, 150%, and 200% scale.
These tests do not replace later packaged-window and target screen-reader
acceptance.

R3F remains a source-build alpha boundary. It does not install, remove, update,
or activate plugins; write provider configuration or secrets; launch or
supervise MCP from the window; package a desktop application; or claim Claude
Desktop, cloud/web, public marketplace, Full MCP, agent, orchestrator, updater,
signing, or release readiness. Building from source requires the pinned Rust
toolchain; a future packaged executable must not require that toolchain at
runtime.

## R3G current-target artifact staging

`qiongli-platform` can now assemble the canonical current-target executable
into a verified native artifact staging tree:

```text
qiongli-<version-channel-profile-os-arch-package>/
  .qiongli-native-artifact.json
  bin/qiongli[.exe]
```

The typed identity fixes product, version, channel, Lite profile, host OS and
architecture, and `portable-archive` package kind. The canonical RFC 8785 JSON
manifest records `assembled-unpublished` status, the binary's logical mode,
size, and SHA-256 digest, the caller-supplied verified resource-pack identity,
and a domain-separated artifact content root.

Composition validates a bounded regular source executable and an explicitly
approved canonical target, uses an owner-private sibling stage and target lock,
writes with create-new semantics and fixed logical modes, syncs the result,
promotes without replacement, and verifies the committed tree. Verification
rejects links/reparse points, hard links, ownership or mode drift, missing or
extra entries, non-canonical metadata, identity drift, resource-pack drift,
and binary/content-root tampering through fixed path-redacted reason codes.

Current-target application tests run only the committed artifact-local binary
from outside the checkout with an empty `PATH`. They verify `--version`, content
inspection, MCP initialization, the exact 12-tool Lite contract, and a bounded
read-only tool call. The Windows fixture uses the same owner-only directory
primitive required by the production validator.

R3G produces an unpacked staging tree, not the final portable archive. It does
not add compression, signing, notarization, checksums, SBOM, provenance,
installation, updater, publication, packaged-window startup, cross-target
packaging, or clean-machine release acceptance.

## R3H deterministic portable archive

`qiongli-platform` can wrap one verified R3G current-target staging tree in a
canonical portable archive named `<artifact-id>.zip`. The accepted container is
a strict store-only ZIP with exactly these entries in fixed order:

```text
<artifact-id>/
<artifact-id>/.qiongli-native-artifact.json
<artifact-id>/bin/
<artifact-id>/bin/qiongli[.exe]
```

The writer fixes UTF-8 names, the 1980-01-01 DOS timestamp, logical modes,
local and central records, CRC-32 values, offsets, and an empty comment. The
bounded parser rejects compression, encryption, ZIP64, data descriptors,
extra fields, comments, multiple disks, unknown or reordered entries, trailing
bytes, and any mismatch with the canonical R3G manifest or caller-supplied
verified resource pack.

Composition uses an explicitly approved canonical target, an owner-private
create-new sibling file, target locking, persistence sync, no-replace
promotion, and committed verification. Extraction parses and verifies the
complete envelope before mutation, never joins an archive-controlled path, and
commits only the fixed manifest and executable payload through the existing
R3G private staging path.

Current-target application tests compose two byte-identical archives, extract
one into an isolated root, and run only its executable outside the checkout
with an empty runtime `PATH`. They verify `--version`, embedded content, MCP
initialization, the exact 12-tool Lite contract, and one bounded read-only tool
call. Structural, content, source-drift, link, lock, and destination-conflict
failures preserve existing caller data and return fixed path-redacted reason
codes.

R3H is an unsigned, unpublished transport-container gate. It does not provide
signing, notarization, checksum sidecars, SBOM, provenance, launch grants,
installation, updater behavior, publication, packaged-window startup,
cross-target packaging, or clean-machine release acceptance.

## R3I verified native-payload installation

`qiongli-platform` can now bind one verified R3H current-target archive to an
R3A signed launch grant, deterministic `InstallPlanV1`, explicit
`filesystem-write` approval, and caller-approved `QiongliManagedData` root.
The additive `InstallNativePayload` action fixes the canonical artifact leaf
and binds the archive, manifest, resource-pack, artifact-content-root, binary,
and signed-grant digests before persistent mutation.

The managed root uses an archive-derived install identity and contains only
portable, caller-independent records:

```text
<managed-root>/
  <artifact-id>/
    .qiongli-native-artifact.json
    bin/qiongli[.exe]
  .qiongli-native-payload-<archive-sha256>.json
```

Apply and absent-target repair run through the accepted R3H parser and R3G
no-replace commit path, re-verify the installed tree, and atomically persist a
strict bounded owner-private receipt. Identical apply, read-only verify,
present-healthy repair, remove, rollback, and terminal lifecycle replay have
explicit dispositions. Remove and rollback first move a verified tree into an
identity-checked private quarantine; failures before the durable state point
restore it, while ambiguous outcomes retain a journal and return
`install-recovery-required`.

Current-target acceptance creates an explicit test-signed launch grant,
installs the verified archive into an isolated managed root, and runs only the
installed executable outside the checkout and archive trees with an empty
runtime `PATH`. It verifies `--version`, embedded content, MCP initialization,
the exact 12-tool Lite contract, and one bounded read-only tool call. Fault,
tamper, linked-state, drift, and foreign-destination tests preserve existing
caller data and use fixed path-redacted errors.

R3I is a shared installation service, not a production installer or release.
The source build cannot manufacture a production launch grant. Production key
provisioning, production-signed release metadata, automatic managed-root
discovery, client activation, public Marketplace installation, updater
behavior, notarization, SBOM, provenance, packaged-window startup, cross-target
output, and clean-machine release acceptance remain later gates.

## R3J signed native release envelope

`qiongli-platform` now authorizes one exact R3H current-target archive through
a strict bounded canonical `NativeReleaseEnvelopeV1`. The envelope binds the
complete artifact identity, release channel and generation, validity interval,
archive filename, size and digest, R3G manifest, embedded resource pack,
artifact content root, executable digest, and complete R3A signed launch grant.

Release-envelope Ed25519 keys and launch-grant keys are separate Rust trust
roles. A trusted release public key has a bounded key ID plus an inclusive
minimum and optional exclusive maximum release generation. At most sixteen
unique release keys are accepted, and there is no unsigned, caller-supplied, or
cross-role fallback. The repository exposes only deterministic envelope and
domain-separated signing-preimage construction; it contains no production
private-key API or material.

Successful verification returns a private token retaining the caller-approved
archive target, newly verified archive, and independently verified launch
grant. Native-payload preview, apply, and repair require that token. The plan
action, semantic digest, and canonical receipt bind its signed-payload digest,
and the executor re-verifies and compares the retained archive immediately
before extraction. Receipt-backed verify, remove, rollback, and recovery remain
available without the release source.

Acceptance uses distinct deterministic test-only release and launch keys. It
rejects noncanonical and oversized input, unknown fields, removed or
out-of-window keys, wrong key roles, stale generations, invalid time or
channel, signature and metadata tampering, payload mismatch, invalid attached
grants, archive drift, and alternate valid release tokens before managed
mutation. The installed current-target executable still runs CLI and the exact
12-tool Lite MCP contract outside the checkout with an empty runtime `PATH`.

R3J is a release-verification boundary, not the published Lite Alpha.1. The
accepted implementation remains test-signed and does not provide production
release-key provisioning, an executable CLI/UI install intent, automatic root
discovery, client activation, public publication, updater behavior, OS signing
or notarization, checksum sidecars, SBOM, provenance, cross-target artifacts,
or clean-machine release acceptance.

## R3K embedded release authority and native install CLI

The canonical product can now embed one strict public release-authority policy
at build time. `NativeReleaseAuthorityV1` fixes the channel, minimum release and
launch-grant generations, and bounded sorted Ed25519 public-key sets for the two
independent R3J trust roles. Release keys retain explicit generation windows;
key IDs and public-key bytes cannot overlap across roles. The policy channel
must agree with the compiled product version.

Release builds provide the byte-canonical public policy through
`QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE`. The build script validates and embeds
the policy or fails the build. The variable is a build input only: the installed
binary has no runtime environment, command-line, file, network, or development
key override. Ordinary source builds embed an empty sentinel and truthfully
report release authority, signed-release preview, and apply as unavailable.

An authority-injected current-target product exposes:

```text
qiongli install native preview \
  --release <release.json> \
  --archive <archive> \
  --managed-root <existing-private-absolute-directory> \
  --target <codex|claude>

qiongli install native apply <same source options> \
  --expected-plan-digest <preview-sha256> \
  --approve-filesystem-write

qiongli install native verify \
  --managed-root <existing-private-absolute-directory> \
  --install-id <native-payload-id>

qiongli install native remove <same receipt options> \
  --approve-filesystem-write
```

Preview and apply derive the compiled current Lite portable target and repeat
the complete R3J release/archive/grant verification plus R3I plan verification.
Preview emits a path-redacted `mutation: none` summary. Apply reconstructs that
plan and requires both its exact semantic digest and explicit filesystem-write
approval. Verify and remove use the canonical owned receipt, so they remain
available without the release source or embedded authority; remove still
requires explicit approval and preserves R3I drift, quarantine, and recovery
rules.

R3K requires an already existing owner-private managed root. It does not choose
or provision production key values, sign or download a release, create or
discover the root, expose repair, activate Codex or Claude, write desktop state,
start a packaged window, publish an artifact or Marketplace entry, provide an
updater, sign/notarize an OS package, produce checksum/SBOM/provenance outputs,
or claim clean-machine Lite Alpha.1 readiness.

## R3L client activation coordination and desktop intent

The accepted Codex and Claude Code registration adapters now share one closed
`ClientActivationCoordinator`. Discovery returns a path-redacted capability
handle for exactly one target. Preview accepts only an already verified Lite
PluginBundle launch grant, re-verifies its trusted key, generation, target
scope, mode, artifact and plan, and requires exactly `filesystem-write`,
`client-config-change`, and `host-trust` approval. A private process-local
binding prevents a preview from one discovered handle being applied through a
different handle, even when both name the same client family.

The coordinator preserves target-specific apply/replay, verify, repair,
remove/replay, rollback/replay, recovery, and unrelated-entry behavior. It
verifies immediately after apply or repair and attempts the accepted adapter
rollback if a fresh mutation unexpectedly fails verification. Public discovery,
commit, verification, lifecycle, Debug, and error output remain path-redacted;
client-owned plugin installation, cache, enablement, reload, and trust prompts
remain explicit host actions.

The desktop service may receive at most one prepared trusted activation session
per local target from a release/installer boundary. A confirmable preview shows
the exact lowercase plan digest and the three approval labels, while the
verified plan remains application-owned behind one OS-random 128-bit operation
token. Confirm applies only that pending plan and refreshes the validated
snapshot; cancel, wrong token, stale token, malformed preview, and missing
session paths fail closed. Ordinary source builds receive no session, report
`apply: false`, and retain a truthful blocked integration preview.

The canonical binary also exposes the non-mutating preflight:

```text
qiongli ui --startup-check
```

It validates the embedded content, desktop service, redacted snapshot, UI app
state, and linked window entrypoint, emits versioned path-free JSON, opens no
window, and starts no subprocess. Acceptance runs the command from a copied
current-target artifact outside the checkout with an empty `PATH`, so it cannot
silently depend on Python, Node.js, Cargo, or another language runtime.

R3L itself does not assemble the R3K portable payload and target-specific
PluginBundle into a release candidate. The R3M successor below composes that
local candidate journey; production signing, publication, client-owned
enablement, cloud surfaces, updater behavior, and the remaining acceptance
gates stay separate.

## R3M signed candidate product journey

Release builds additionally provide the exact lowercase 40- or 64-character
source object ID through `QIONGLI_NATIVE_SOURCE_COMMIT`. The build validates and
embeds that value beside the public release authority. Neither value is read
from the runtime environment, and neither contains private signing material.
Source builds without both inputs fail candidate preview/apply closed.

The authenticated candidate is exactly one same-directory three-file set with
derived filenames: the portable archive, canonical signed candidate JSON, and
signed release notes. Product commands accept those files and a client target,
but no managed root or plugin-source path:

```text
qiongli install candidate preview \
  --candidate <artifact-id>.candidate.json \
  --archive <artifact-id>.zip \
  --release-notes <artifact-id>.release-notes.md \
  --target <codex|claude>

qiongli install candidate apply <same file and target options> \
  --expected-approval-digest <preview-sha256> \
  --approve-filesystem-write \
  --approve-client-config-change \
  --approve-host-trust

qiongli install candidate verify \
  --target <codex|claude> \
  --install-id <native-payload-id>

qiongli install candidate remove <same target and install ID> \
  --approve-filesystem-write \
  --approve-client-config-change
```

Preview performs no write and emits only versioned, symbolic-path output. Its
approval digest binds both the signed candidate digest and selected target.
Apply re-verifies the complete candidate and derives the fixed owner-private
payload root `<user-home>/.qiongli/native/payloads` plus the fixed Codex or
Claude Code source path. It then applies and immediately verifies payload,
source, and registration receipts as one closed identity chain. A fresh later
failure compensates only fresh earlier steps in reverse order.

Verify and remove require no candidate, authority, source-commit input, or
unexpired release. They reopen only the fixed current-user paths and require
the payload, PluginBundle, and registration receipts to agree on target,
artifact, binary, pack, grant, and source identities. Remove verifies first and
then removes registration, source, and payload in reverse order. A partial
remove fails with recovery-required evidence rather than deleting unverified
state.

The native manager can open the same verified candidate session without a
second installer implementation:

```text
qiongli ui \
  --candidate <artifact-id>.candidate.json \
  --archive <artifact-id>.zip \
  --release-notes <artifact-id>.release-notes.md \
  --target <codex|claude>
```

Its operation token owns the verified in-memory candidate, shows the same
target-bound approval digest and three approvals, and calls the same local
apply pipeline on confirmation. Qiongli still does not drive client UI or
modify client-owned cache, enablement, reload, or trust state; those remain
explicit host actions. This implementation does not itself create a tag,
publish Alpha.1, provision production private keys, or claim Claude Desktop,
Codex/ChatGPT Marketplace bypass, cloud execution, updater, notarization,
SBOM, provenance, or cross-target readiness.

### Non-publishing candidate acceptance

Maintainers and Native CI can assemble and exercise an ephemeral test-signed
candidate with the repository example:

```text
cargo run --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --example native_candidate_acceptance \
  --locked \
  -- \
  --output <absolute-new-directory-outside-checkout> \
  --source-commit <exact-40-or-64-character-lowercase-object-id>
```

The output parent must already exist and pass the same trusted-path rules as a
release target. The harness creates distinct ephemeral release and launch keys
in memory, persists only the canonical public authority, builds one
authority/source-bound Release binary, normalizes Cargo output into a fresh
owner-private single-link artifact source, and verifies that the candidate
directory contains exactly the archive, signed candidate JSON, and signed
release notes. The normalized source is removed immediately after composition.

Acceptance extracts and runs only that binary from outside the checkout with
an empty `PATH` and isolated homes. It covers version, embedded skills,
materialization, UI startup preflight, the public Lite MCP tool list, Codex and
Claude Code preview/apply/verify/remove, wrong and partial approvals, fresh-step
compensation, and unrelated-state preservation. The generated
`acceptance-evidence.json` is explicitly non-publishing. Real-client UI,
displayed-window, and production-signing gates remain `not-run` with reasons
until their external environments and maintainer authority are provided.

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
