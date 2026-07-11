# Repository Source Code Standard

Qiongli's repository engineering policy is Task RC1. It governs the product's
source code and maintainer tooling; it does not judge the scientific validity
of analysis code produced for a paper. Academic analysis-code requirements
remain under AC1 and the Stage I workflow.

The canonical machine policy is
`tooling/quality/repository-source-code-contract.yaml`. Keep it under
`tooling/quality/`: repository policy must not be materialized into skills,
plugins, package payloads, or marketplace catalogs.

## Native-foundation enforcement

The first enforcing phase is intentionally narrow. It protects
`packages/qiongli-native/` while the Rust-native platform is bootstrapped:

- the workspace has one product member and binary at `apps/qiongli`;
- package, release-channel, toolchain, lockfile, and lint identity stay fixed;
- native source cannot use symlinks or gitlinks, commit executable, oversized,
  or binary build output, or escape the workspace through Cargo path
  dependencies;
- high-confidence credentials and machine-specific absolute paths are blocked;
- B2a production Rust cannot launch external processes, so a CI-installed
  Python, Node.js, Cargo, or agent CLI cannot hide a startup dependency;
- the initial product remains dependency-free, and Clippy resolves aliases when
  rejecting `std::process::Command::new` in production targets; the product
  crate root uses `forbid`, so a local lint allowance cannot weaken the gate;
- repository Cargo config, compiler-flag/wrapper environment overrides, and
  build scripts are rejected during B2a so `--cap-lints` cannot silence it;
- all native changes select locked Rust formatting, clippy, and test gates; and
- native-foundation findings cannot be hidden in the legacy-debt baseline.

Python, JavaScript/TypeScript, Shell, and PowerShell profiles remain planned for
the pre-beta RC1 phase. This policy does not authorize a repository-wide
reformat or claim complete secret, license, SBOM, or supply-chain coverage.

## Commands

Use the event-aware merge base in CI:

```bash
python scripts/validate_repository_source.py --base-ref <base-commit>
```

Check an explicit local change or the complete repository tree:

```bash
python scripts/validate_repository_source.py \
  --changed-file packages/qiongli-native/apps/qiongli/src/main.rs
python scripts/validate_repository_source.py --full-tree --json
```

Native changes must also pass:

CI pins Rust `1.97.0`. When running locally, either enter
`packages/qiongli-native/` so its `rust-toolchain.toml` override applies, or
activate the same toolchain before using these repository-root commands.

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

Exit status `0` means pass, `1` means blocking findings, and `2` means the
policy, path input, or Git comparison could not be evaluated safely. JSON mode
writes one deterministic report to stdout.

## Findings and baseline policy

Rule IDs are stable `RSC-*` identifiers. A finding fingerprint binds its rule,
canonical path, and source blob digest without recording the matched secret or
machine-local value.

`tooling/quality/repository-source-code-baseline.json` is reserved for exact,
pre-existing non-native debt. Every future entry must identify one regular
file, one rule, an owner, rationale, compensating check, unexpired deadline,
and matching fingerprint. Glob suppressions, directory suppressions, expired
entries, fingerprint drift, and all suppressions under
`packages/qiongli-native/` fail closed.

Generated distribution payloads remain governed by
`scripts/check_generated_payload_edits.py`, which reads the canonical generated
roots from the repository source-layout module. Do not duplicate or broaden
those roots in RC1.
