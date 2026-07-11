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

The CTR validation command currently covers two static inventories. CTR-201A
is the merged, validation-backed derived-inventory slice. CTR-201B captures the
parser-declared Python Full CLI surface: 46 canonical and 49 public command
paths, five console entrypoints, 164 non-help actions, and 27 defaults that
resolve to the current working directory. Help coverage is limited to authored
parser metadata; the formatted help output, runtime behavior, JSON output, exit
codes, dry-run behavior, error classes, and npm compatibility are not complete.

Accordingly, CTR-201 remains in progress and FND-202 is not implemented. The
next contract slices are CTR-201C for the orchestrator and CTR-201D for content
and materialized-tree closure.

```bash
python scripts/validate_repository_source.py --base-ref <base-commit>
python scripts/extract_ctr_201_cli_inventory.py --check
python scripts/validate_ctr_201_inventory.py
python -m unittest tests.test_ctr_201_inventory \
  tests.test_ctr_201_cli_inventory -v
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

The complete repository engineering policy and exception rules are documented
in `docs/development/repository-source-code-standard.md`. These RC1 rules are
separate from the AC1 academic analysis-code standard.
