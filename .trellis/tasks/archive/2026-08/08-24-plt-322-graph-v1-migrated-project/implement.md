# Implementation plan

## 1. Activate and branch

- After explicit approval of this final plan, run `task.py start` and create
  `feat/plt-322-graph-v1-migrated-project` from `2.x`.
- Load the product, native, App API, Desktop, and shared Trellis specs before
  editing.

## 2. Build the Qiongli research project

- Create `RESEARCH/asset-pricing-capm-ff3/` from canonical Qiongli templates.
- Initialize it with the repository's 1.19 CLI, set the subject to `finance`,
  and capture only the redacted status needed to prove the legacy project
  boundary; `.qiongli/` remains ignored and migration-excluded.
- Complete the Idea Funnel and boundary review, then the selected A/B/C/I/F4
  task artifacts named in `design.md`.
- Add verified notes, retrieval limits, extraction rows, and bibliography
  entries only for sources actually inspected; keep literature coverage
  explicitly targeted rather than systematic.
- Keep claims provisional or `gap_note` until the analysis run exists.

Checkpoint: canonical tables retain their exact headers and the source contains
no 2.x manifest, committed private runtime/raw input, or absolute path.

Initialization commands:

```bash
.venv/bin/python -m qiongli.cli project init --project-dir RESEARCH/asset-pricing-capm-ff3
.venv/bin/python -m qiongli.cli project set-subject finance --project-dir RESEARCH/asset-pricing-capm-ff3
.venv/bin/python -m qiongli.cli project status --project-dir RESEARCH/asset-pricing-capm-ff3
```

## 3. Implement and run the empirical analysis

- Add one PEP 723 Python entrypoint and create its script lock with `uv`.
- Download the two official ZIPs, inspect and pin their SHA-256 values, parse
  their monthly value/equal-weighted sections, and run the locked CAPM/FF3
  specifications.
- Write deterministic model, summary, diagnostic, and provenance outputs.
- Run the entrypoint twice and compare machine-readable output digests.
- Complete the evidence ledger, result interpretation, performance profile,
  reproducibility audit, and Q1/Q2/Q4 quality-gate evidence from observed
  outputs only.

Focused checks:

```bash
uv lock --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py
uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py
uv run --script RESEARCH/asset-pricing-capm-ff3/analysis/run_analysis.py --check
```

Checkpoint: 25 portfolios x 2 models x 2 weighting schemes are present, all
reported sample dates agree, diagnostics are recorded, and the second run is
digest-identical for deterministic outputs.

## 4. Add the bounded migrated-project acceptance

- Add the explicit root acceptance command, Node coordinator, and one Desktop
  acceptance test described in `design.md`.
- Reuse native migration/Graph/App commands, App API schemas, and existing
  Desktop Graph functions; add no new public product schema.
- Make missing source, dirty source, skipped control, digest drift, query
  mismatch, source mutation, and path leakage fail closed.
- Keep existing synthetic readiness cases as required negative controls.

Focused checks:

```bash
cargo test --locked --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-project academic_graph_readiness
pnpm --dir packages/qiongli-desktop exec vitest run src/lib/features/academic-graph/readiness.test.ts src/lib/features/academic-graph/representative-migrated-project.acceptance.test.ts
node scripts/plt322_migrated_graph_acceptance.mjs --source RESEARCH/asset-pricing-capm-ff3 --receipt /tmp/plt322-receipt.json
```

Checkpoint: the real project passes all semantic, deterministic, query,
inspection, presentation, and source-retention assertions; every omitted
required check makes the command fail.

## 5. Full task-scope verification

- Run format and the affected native project/App tests.
- Run App API tests, Desktop tests/check/build, the capability contract check,
  and the program-roadmap generator tests.
- Run `git diff --check` and scan generated evidence for absolute paths or
  forbidden private material.
- Run `trellis-check`; fix all task-scope findings and rerun invalidated checks.

Planned Slice commands:

```bash
cargo fmt --all --manifest-path packages/qiongli-native/Cargo.toml -- --check
cargo test --locked --manifest-path packages/qiongli-native/Cargo.toml -p qiongli-project
pnpm --dir packages/qiongli-app-api test
pnpm --dir packages/qiongli-desktop test
pnpm --dir packages/qiongli-desktop check
pnpm --dir packages/qiongli-desktop build
python3 scripts/validate_capability_contract.py
python3 tooling/scripts/update_program_roadmap.py --check
git diff --check
```

## 6. Exact-source acceptance and closeout

- Apply any required spec update, then commit the product/research source.
- Run the real migrated-project command from that exact clean commit and retain
  only the redacted receipt.
- Push the branch and obtain the exact-head Native CI Slice result.
- Only after both pass, add the PLT-322 acceptance note, set the Program Ledger
  row to `accepted`, regenerate the current index, and commit the evidence-only
  closeout.
- Open the PR against `2.x`; merge only after required checks pass. Do not start
  candidate packaging, promotion, publication, or `PILOT-903`.

Rollback point: if the real run or Slice CI fails, leave the ledger proposed,
remove no source project, and report the exact stable blocker.
