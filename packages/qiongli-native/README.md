# Qiongli Native Workspace

This workspace is the canonical Rust-native product source for Qiongli 2.x.
It contains the product application, `apps/qiongli`, plus the first real shared
service contract in `crates/qiongli-content`. The content crate defines the
versioned resource-pack manifest, frozen profile projections, and bounded
canonical source collector. It now compiles those collected bytes into an
unsigned deterministic `.qlpack` core and verifies/loads that core entirely in
memory; it does not yet sign or materialize pack bytes.

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

The expected digest establishes integrity only when it comes from a trusted
embedding or authenticated descriptor. Publisher signatures, trusted-key and
revocation policy, running-product compatibility enforcement, runtime
embedding, and atomic materialization remain separate successor work.

The version 1 header is `QLPACK\0\0`, followed by a little-endian `u32` format
version and a little-endian `u64` manifest length. Payload offsets start after
the manifest. The content-root preimage is
`qiongli:resource-pack:content-root:v1`, one NUL byte, the little-endian `u64`
length of the canonical entry-array JSON, and those canonical bytes. The
manifest schema keeps numeric fields within the JCS safe-integer range.

The existing `packages/qiongli-lite-mcp` crate remains a migration oracle and
compatibility package. Native functionality will be extracted into shared
workspace crates rather than copied into a second implementation.

## B2a command contract

The bootstrap executable intentionally supports only `--version` and
`-h|--help`. A bare invocation, unknown command or option, and extra token
returns exit code 2 with a redacted usage error. UI, MCP, doctor, installer,
agent, and orchestration modes are added only with their real service contract;
they are not placeholder commands in this slice.

The CLI contract tests also clear `PATH` before starting the binary. This
prevents developer or CI installations of Python, Node.js, Cargo, or other
tools from masking a required startup dependency.

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
