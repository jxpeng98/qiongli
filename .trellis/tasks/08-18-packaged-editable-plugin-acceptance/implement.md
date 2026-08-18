# Implementation: editable content and connected Research Graph

## Ordered checklist

- [x] Add a failing `qiongli-project` regression proving every extracted
  semantic node is reachable from the project through exact source containment
  and that repeated rebuilds retain identical identities.
- [x] Add the minimum `artifact -> semantic record` structural edges inside the
  existing `AcademicGraphService::rebuild` path.
- [x] Update the canonical Workflow and academic context maintainer with the
  graph-readable artifact, stable-ID, contribution-claim, and refresh/readiness
  handoff rules; regenerate existing embedded/package locks only through the
  repository's current scripts.
- [x] Make the Research Library selected-project action explicitly name and
  open the existing knowledge graph; keep the dedicated Cytoscape route as the
  only project graph implementation.
- [x] Extend the existing packaged product acceptance through customized edit,
  update-required, explicit Skills/Plugin reconcile, Customized Ready, reset,
  explicit reconcile, and Canonical Ready before cleanup.
- [x] Add connected graph parity to the same packaged fixture across App, CLI,
  and Full MCP and emit explicit receipt booleans.
- [x] Strengthen existing isolated real Codex and Claude tests to assert the
  official Host caches contain customized Skill bytes and the exact variant
  receipt while MCP/manifests remain canonical.
- [x] Run focused native, content, App API, Desktop, and capability checks.
- [x] Run full required format, lint, test, build, and evaluation gates.
- [x] Build the exact-source local macOS package, run automated packaged
  acceptance, and manually inspect Library -> connected Graph -> source anchor
  plus Workflow edit/reset if the desktop session is available.
- [ ] Commit, push, open the PR, repair required CI failures, record exact-head
  evidence, and merge only after all required checks pass.
- [ ] Resume the already planned GOV-405 through GOV-407 task from latest `2.x`.

## Focused validation

```bash
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-project academic_graph --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-content --test workflow_overrides --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test codex_plugin_bundle --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test claude_plugin_bundle --locked
pnpm --dir packages/qiongli-app-api test
pnpm --dir packages/qiongli-desktop test
.venv/bin/python scripts/validate_capability_contract.py
```

The two real-Host ignored tests run only with isolated temporary homes and the
already installed official Host CLIs.

## Final validation

```bash
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
pnpm --dir packages/qiongli-desktop check
pnpm --dir packages/qiongli-desktop build
.venv/bin/python evals/runner/run_suite.py
git diff --check
```

Run the repository's existing exact-source macOS package/acceptance command;
do not introduce another packaging path.

## Acceptance evidence (2026-08-18)

- Exact product commit: `5e7991289aa31cc1d976d942e44ebbbd2d1ab975`.
  The schema-3 receipt is `accepted-ad-hoc-nonpublishing`, keeps
  `publication_allowed: false`, and reports every required check as `true`,
  including connected graph parity and Workflow edit/reconcile/reset.
- The exact App started in the clean acceptance home and reported the same
  source commit. An App-confirmed project refresh advanced the manual fixture
  to `r2`; Library's `Open knowledge graph` action opened 7 nodes, 7 edges,
  and 1 connected component in topology layout v2.
- Selecting the research-question node produced a 2-node, 1-edge neighborhood
  and exposed `context/research_state.md` plus the exact
  `field:main_question_or_thesis` anchor. Zoom, fit, minimap, focus, and the
  synchronized node-table entry remained available without renderer fallback.
- The exact App editor saved a harmless `workflow/SKILL.md` marker as receipt-
  bound variant `r1`, then restored canonical bytes at revision `r2`; the
  canonical 27,273-byte resource disabled both save and restore again.
- Full gates passed: Rust format, Clippy with warnings denied, full workspace
  tests, 247 Desktop tests, Svelte check, production build, Capability
  Contract V2, all 12 Evaluation Truth cases, and `git diff --check`.

## Review gates

- A graph edge must be structural unless its existing extractor supplies an
  explicit source-backed scholarly relation.
- No normal Codex/Claude profile may be read or mutated by automated tests.
- Customized bytes may replace only allowed Workflow/Skill Markdown resources.
- Package evidence must bind the exact source commit and remain non-publishing.
- A passing canonical test is not sufficient evidence for customized Host
  activation or graph connectivity.

## Rollback points

- Structural graph continuity is one projection change and can be reverted
  without changing project artifacts or schemas.
- Workflow content is regenerated through existing locks; revert source and
  generated bytes together.
- Packaged acceptance changes are evidence-only and do not create a second
  runtime owner.
