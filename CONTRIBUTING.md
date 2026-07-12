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
CTR-201C extraction is pinned to Python `3.12`; CTR-201D requires Python
`3.12+`, with CI regeneration fixed to `3.12`. Both use PyYAML `6.0.3`; use
the locked workspace environment when regenerating or checking their
artifacts.

The CTR validation command currently covers four static inventories. CTR-201A
is the merged, validation-backed derived-inventory slice. CTR-201B captures the
parser-declared Python Full CLI surface: 46 canonical and 49 public command
paths, five console entrypoints, 164 non-help actions, and 27 defaults that
resolve to the current working directory. Help coverage is limited to authored
parser metadata for CTR-201B; the formatted help output, runtime behavior, JSON
output, exit codes, dry-run behavior, error classes, and npm compatibility are
not complete.
CTR-201C captures the accepted-source `DECLARED/STATIC` orchestrator control
contract and compatibility boundary: 13 stages, 76 tasks, 104 required
dependency edges, three runtime agent IDs, nine functional agent IDs, 82
routing skill IDs, 11 logical MCP capabilities, four quality gates, five
built-in profiles, and the B1/H3 team and worker configurations. This inventory
does not establish orchestrator runtime parity or semantic execution. The 82
values are unique routing IDs rather than an installable-skill count, and the
11 values are logical capability IDs rather than public MCP tool names.

CTR-201D captures the accepted 377-file canonical content tree, its closed
resource-kind partition, the `skill-only`, `marketplace-lite` (`lite` alias),
and `full` source projections, and three reproducible 1.x materialized skill
subtrees. It does not claim parity for published plugin/archive wrappers,
install activation, or a Rust resource pack.

Accordingly, CTR-201 remains in progress because Contract v2, CLI runtime, and
orchestrator runtime coverage are still incomplete; FND-202 is not implemented.

```bash
python scripts/validate_repository_source.py --base-ref <base-commit>
python scripts/extract_ctr_201_cli_inventory.py --check
python scripts/extract_ctr_201_orchestrator_inventory.py --check
python scripts/extract_ctr_201_content_inventory.py --check
python scripts/validate_ctr_201_inventory.py
python -m unittest tests.test_ctr_201_inventory \
  tests.test_ctr_201_cli_inventory \
  tests.test_ctr_201_orchestrator_inventory \
  tests.test_ctr_201_content_inventory -v
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

The complete repository engineering policy and exception rules are documented
in `docs/development/repository-source-code-standard.md`. These RC1 rules are
separate from the AC1 academic analysis-code standard.
