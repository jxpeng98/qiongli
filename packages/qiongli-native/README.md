# Qiongli Native Workspace

This workspace is the canonical Rust-native product source for Qiongli 2.x.
It currently contains one product application, `apps/qiongli`, and deliberately
does not add empty service crates whose public APIs have not been accepted.

## Dependency direction

The product dependency graph is one-way:

```text
apps/qiongli -> service crates -> contracts/platform primitives
```

Libraries must never depend on `apps/qiongli`, and the application remains a
thin mode dispatcher. Future service crates are added only with their first
real contract and tests. Production crates must not require Python, Node.js,
Cargo, or another language runtime to start.

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

Run the foundation gates from the native workspace so Rustup applies the
workspace-local pinned toolchain:

```bash
cd packages/qiongli-native
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```
