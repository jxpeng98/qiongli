# Qiongli Native Workspace

This workspace is the canonical Rust-native product source for Qiongli 2.x.
It contains the product application, `apps/qiongli`, plus the accepted content,
config, runtime, and isolated Windows-security service boundaries. The content
crate defines the versioned resource-pack manifest, frozen profile projections,
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

## R1 command contract

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
