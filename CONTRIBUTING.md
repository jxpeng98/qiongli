# Contributing to Qiongli

Keep changes inside their canonical repository boundary. Academic content is
edited under `content/`; native Qiongli 2 source is edited under
`packages/qiongli-native/`; root `scripts/` files remain stable wrappers around
implementations under `tooling/scripts/`. Generated plugin and package payloads
must be produced through the supported materialization workflow rather than
edited directly.

Before submitting native-source changes, run:

CI uses Rust `1.97.0`; select that toolchain locally or run Cargo from
`packages/qiongli-native/` so the workspace override is active.

```bash
python scripts/validate_repository_source.py --base-ref <base-commit>
python scripts/validate_ctr_201_inventory.py
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

The complete repository engineering policy and exception rules are documented
in `docs/development/repository-source-code-standard.md`. These RC1 rules are
separate from the AC1 academic analysis-code standard.
