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
artifacts. CTR-201E canonical extraction uses Python `3.12` and a Node engine
meeting `>=18`; CI supplies Node 20 only to the Ubuntu full tier for the
authenticated, parse-only npm dispatch oracle. CTR-201F canonical extraction
is likewise pinned to Python `3.12` in the Ubuntu full tier. Windows and macOS
validate its checked, portable artifact and bindings without re-executing the
accepted orchestrator runtime.

The CTR validation command covers the CTR-201A master ledger and the
CTR-201B/C/D/E/F child inventories. CTR-201A is the merged,
validation-backed derived-inventory slice. CTR-201B captures the
parser-declared Python Full CLI surface: 46 canonical and 49 public command
paths, five console entrypoints, 164 non-help actions, and 27 defaults that
resolve to the current working directory. Help coverage is limited to authored
parser metadata in CTR-201B; CTR-201E supplies the separate runtime-inventory
classification and must not be confused with Full handler runtime parity.
CTR-201C captures the accepted-source `DECLARED/STATIC` orchestrator control
contract and compatibility boundary: 13 stages, 76 tasks, 104 required
dependency edges, three runtime agent IDs, nine functional agent IDs, 82
routing skill IDs, 11 logical MCP capabilities, four quality gates, five
built-in profiles, and the B1/H3 team and worker configurations. This inventory
does not by itself establish orchestrator runtime parity or semantic execution.
The 82 values are unique routing IDs rather than an installable-skill count,
and the 11 values are logical capability IDs rather than public MCP tool names.

CTR-201D captures the accepted 377-file canonical content tree, its closed
resource-kind partition, the `skill-only`, `marketplace-lite` (`lite` alias),
and `full` source projections, and three reproducible 1.x materialized skill
subtrees. It does not claim parity for published plugin/archive wrappers,
install activation, or a Rust resource pack.

CTR-201E is merged and implements the accepted-source Full CLI
runtime-inventory-freeze slice. Its gate classifies the 49 public command paths
and five console entrypoints already fixed by CTR-201B across formatted help,
stdout/stderr, JSON, exit codes, normalized error classes, dry-run and
side-effect behavior, zero-argument behavior, aliases, and legacy npm dispatch.
The Python capture is audit-hook isolated from network and real user-state
mutation; the Node evidence executes only a source-audited parse-only module and
does not claim an operating-system network sandbox. Both are deterministic,
digest-bound, and backed by semantic and negative/mutation tests. See
`docs/development/ctr-201-cli-runtime-freeze.md` for the exact boundary.

The checked corpus contains 118 cases. Successful handler behavior is captured
only for the bounded cases named by the artifact. Every other executable
handler dimension is linked to `CTR-201E-D001` or `CTR-201E-D002`, while npm
handler parity is linked to `CTR-201E-D003`; all three decisions are scoped to
CTR-201 inventory completion and owned downstream by `LEG-201`. Do not infer
Full handler parity from `cli.completion_ready=true`.

CTR-201F closes the accepted-source orchestrator runtime inventory with bounded,
deterministic fixtures and explicit inventory-only dispositions. Its 44 cases
(one A8 oracle plus 43 bounded cases) close six behavior dimensions with six
decisions. It records the accepted control flow for profiles, task planning,
MCP routing, topic-less `code_build`, solo/duo/triad routing, review and
revision, and worker/team success and failure branches. State/resume evidence
is limited to bridge session-command passthrough, experience replay advice, and
the accepted absence of durable task/team resume or cancel APIs. Missing real
agent execution, strict topic `code_build`, native worker dispatch, semantic
quality evaluation, and public cancellation remain explicit boundaries rather
than inferred capabilities. Running its checked validator on three
operating-system families proves portable validation, not cross-platform
orchestrator runtime parity. Its six dispositions retain their declared AGT,
ORC, CFG, DOM, GOV, MCP, and conditional LEG downstream owners.

CTR-201E and CTR-201F are migration-engineering work items, not canonical
academic Task IDs; do not add them to the research workflow task catalog. Their
checked artifacts are oracle and disposition evidence rather than Rust
implementation evidence. CTR-201F closes the parent source-oracle inventory;
it does not implement CTR-202, FND-202, a Rust orchestrator, or a production
agent backend.
CTR-201D does not establish archive or published-package parity, but that
evidence is an unassigned downstream governance boundary rather than a CTR-201
exit dependency. CTR-202 owns completion of Capability Contract v2 and follows
CTR-201. The current CTR-202 working-tree candidate closes the concrete issues
found by the July 12 review and passes focused local complete-mode validation;
it still requires commit, review, and exact-head CI. SEC/CTR child IDs remain
evidence labels rather than mandatory PR shapes or canonical academic Task IDs.
FND-202 is a separate successor; FND-202A now defines the native manifest and
profile projection contract. See
`docs/development/ctr-202-capability-contract-v2.md` for the exact profile
matrix, safety strategy, and claim boundary.

```bash
python scripts/extract_ctr_201_cli_inventory.py --check
python scripts/extract_ctr_201_orchestrator_inventory.py --check
python scripts/extract_ctr_201_content_inventory.py --check
python scripts/extract_ctr_201_cli_runtime_inventory.py --check
python scripts/extract_ctr_201_orchestrator_runtime_inventory.py --check
python scripts/validate_ctr_201_inventory.py
python scripts/validate_capability_contract.py
python -m unittest tests.test_ctr_201_inventory \
  tests.test_ctr_201_cli_inventory \
  tests.test_ctr_201_orchestrator_inventory \
  tests.test_ctr_201_content_inventory \
  tests.test_ctr_201_cli_runtime_inventory \
  tests.test_ctr_201_orchestrator_runtime_inventory -v
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
```

The former RC1 repository checks and AC1 academic-code design remain available
as optional review guidance. They are not required migration, CI, or release
gates. Use language-native checks, relevant tests, and normal review for the
changed behavior.
