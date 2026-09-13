# Implementation Plan

## 1. Start From One Verified Candidate

- [x] Start this Trellis task only after explicit approval of the latest plan.
- [x] Download artifact `9082833758` from promotion run `31439930097` into a
      fresh private temporary directory.
- [x] Verify the aggregate and macOS receipts, source commit, version, candidate
      set digest, executable/content/Plugin digests, and
      `publication_allowed=false` before launching any byte.
- [x] Record candidate identity and retain only bounded digest evidence.

Primary retrieval command:

```bash
gh run download 31439930097 --repo jxpeng98/qiongli \
  --name qiongli-community-alpha-candidate-cced60826ac4d7dad596669103a7e15b61868e81 \
  --dir <fresh-private-directory>
```

## 2. Pass Existing Real-Client Isolation Gates

- [x] Resolve absolute Codex and Claude Code executables and record their
      versions.
- [x] Run the existing Codex ignored test with Plugin Creator validation.
- [x] Run the existing Claude Code ignored test.
- [x] Run the existing combined candidate acceptance only if it can consume or
      truthfully reproduce the exact frozen candidate; otherwise use the two
      exact-source test journeys and downloaded package receipts.
- [x] Verify isolated install/cache/Lite MCP/removal evidence and no normal-home
      changes. Do not add another test harness.

Existing commands:

```bash
QIONGLI_CODEX_BIN=<absolute-codex> \
QIONGLI_PLUGIN_VALIDATOR=<absolute-validate_plugin.py> \
QIONGLI_PLUGIN_VALIDATOR_PYTHON=<absolute-python-with-pyyaml> \
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --test codex_plugin_bundle --locked \
  real_codex_clean_client_installs_enables_caches_and_launches_bundle -- \
  --ignored --exact --nocapture

QIONGLI_CLAUDE_BIN=<absolute-claude> \
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli --test claude_plugin_bundle --locked \
  real_claude_clean_client_discovers_and_installs_both_local_forms -- \
  --ignored --exact --nocapture
```

## 3. Preview And Apply Exact System Integrations

- [x] Snapshot bounded digests and private rollback copies for Qiongli-owned
      Codex and Claude registrations/sources only.
- [x] Discover and run the candidate's existing preview command for each Host;
      review target containment, replacement set, and rollback receipt.
- [x] Apply Codex and Claude integrations from the verified candidate.
- [x] Verify version plus executable, Plugin, embedded-content, and registration
      digests. Confirm unrelated plugins/configuration are unchanged.
- [x] Both Hosts bound the same candidate, so rollback was not required; retain
      the bounded rollback backup and receipt authority.

## 4. Qualify Codex In A Fresh Host

- [x] Restart/open a fresh authenticated Codex process and directly observe the
      enabled `qiongli-next` Plugin, bundled Qiongli Skill, and connected Full
      MCP.
- [x] Use the Skill and Qiongli tools to inspect readiness, run one safe query,
      inspect an existing project, and return a source-grounded result.
- [x] Keep the run ephemeral/read-only, cap scope and spend, and record only the
      bounded Host observation.

## 5. Qualify Claude Code In A Fresh Host

- [x] Restart/open a fresh Claude Code process and directly
      observe the same exact Plugin, Skill, and Full MCP contract.
- [ ] Run the same bounded research journey with no session persistence and no
      unrelated tool/write permissions. Deferred: external model transmission
      was not authorized.
- [x] Record the bounded static Host observation and compare candidate/content/tool
      identity with Codex without claiming a live receipt.

## 6. Qualify Zotero Without User-Library Risk

- [x] Create an isolated Zotero profile/data root and install the candidate's
      bundled Companion XPI.
- [x] Start Zotero, require endpoint contract `2`, and verify Full MCP status.
- [x] Exercise bounded search, dry-run upsert, one receipt-bound apply into a
      disposable collection, replay rejection, and cleanup/removal.
- [x] Stop the Companion and verify CSL JSON/RIS/BibTeX/report fallback.
- [x] Remove the disposable profile only after its evidence is complete.

## 7. Reuse The Existing Host Receipt Contract

- [ ] If the packaged R5C C5 fixture is available, run the existing rejection
      probes and valid revision-2 handoff once in Codex and once in Claude Code.
      Codex completed; Claude was not authorized.
- [x] Compose and validate the Codex path-redacted receipt with the existing
      schema and scripts. Do not claim A7 because the Full route defect required
      a bypass and Claude did not complete the runbook.

## 8. Fix Only A Reproduced Essential Blocker

- [x] If all gates pass, make no product-code change. Not applicable: a P0
      reproduced.
- [x] If an essential-path P0 reproduces, trace all callers, fix the shared
      owner once, and add one smallest focused regression.
- [ ] For a product/package-input change, freeze a new exact source, run
      exact-head CI/package evidence, then rerun only invalidated Host/Zotero
      slices. Deferred until Alpha 3 release qualification resumes.

## 9. Record The Honest Result

- [x] Update the Alpha readiness ledger and master roadmap with exact candidate,
      client versions, bounded receipt identifiers, and one disposition:
      `qualified`, `partial`, or `blocked`.
- [x] Keep CLI size, signing, A8 approval, and Community Alpha publication open.
- [x] Run relevant receipt validators, focused tests if code changed,
      `git diff --check`, and the Trellis completion gate. Affected checks pass;
      the full research-standard strict audit retains four pre-existing M0
      release-document gaps (`Release Notes`, `Validation Evidence`, `Publish
      Steps`, and `rollback.md`) rather than inventing publication material.
- [x] Verify rollback/removal authority before archiving the task.

## Quality Evidence

- `cargo fmt --all -- --check`: passed.
- `cargo clippy -p qiongli --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo test -p qiongli --test mcp_stdio`: 7 passed.
- Codex and Claude deterministic Plugin bundle tests: passed.
- `python3 scripts/validate_capability_contract.py`: passed.
- `python3 -m unittest tests.test_solo_role_gate_audit`: 14 passed.
- `git diff --check` and `node --check scripts/r5d_zotero_acceptance.mjs`: passed.
- Research-standard strict audit: 6,200 passed; four unrelated M0 release-doc
  requirements remain open and are not waived.
