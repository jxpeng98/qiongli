# Qiongli R5C Completion Review and Pre-Beta Readiness Plan

Status: in progress — N0 preparation is complete; the C0-C5 completion review
starts only after both live-host receipts are accepted

Date: July 26, 2026

Parent plans:

- `docs/superpowers/plans/2026-07-25-qiongli-r5c-cross-surface-continuity.md`
- `docs/superpowers/plans/2026-07-26-qiongli-r5c-c5-packaged-acceptance.md`

## Outcome

Close the remaining real-host C5 observations, perform one evidence-backed
completion review across C0-C5, and freeze the pre-Beta distribution sequence.
This stage prepares promotion work; it does not publish a release, claim Beta,
retire the R5B legacy source, or expand current macOS acceptance to untested
targets.

## Entry gate

Do not start the R5C completion review until:

1. the exact accepted `2.0.0-alpha.2` package remains available;
2. both isolated Codex and Claude Code profiles have user-controlled
   authentication;
3. the current 2.x Plugin, Skill, and Full MCP remain visible after a fresh
   host process; and
4. the host-visible acceptance profile contains the same bounded
   three-project fixture required by C5.

Authentication material, prompts, responses, conversations, tool bodies,
project identifiers, and absolute paths remain outside every receipt.

Readiness snapshot:

- the package-bound host fixture preparation landed in `6192cd20`;
- fixture `r5c-c5-host-driven-v1` is prepared in the accepted App's isolated
  manual profile at project revision 2;
- the three-project App/CLI/Full MCP parity checks pass and preparation is
  idempotent;
- the package-bound validator now checks the exact binary, product source,
  fixture revision, and host-specific installed plugin digest; and
- only user-controlled authentication plus one real handoff in each host
  remains before this plan's completion review may start.

## Batch N0 — Close C5 live-host acceptance

Preparation already completed:

1. The bounded three-project fixture was created in the host-visible isolated
   profile without copying real user projects.
2. The canonical preparation receipt is bound to the accepted product receipt,
   exact copied binary, fixture digest, and project revision.
3. Re-running preparation verifies the same state and does not create another
   fixture.

Remaining live sequence:

1. Authenticate Codex and Claude Code inside their isolated profiles through
   each host's own login flow. Do not copy a token or real host configuration.
2. Restart the accepted App, Codex, and Claude Code.
3. In each host, complete one revision-bound Qiongli handoff through Full MCP:
   - read the declared project revision and evidence;
   - submit a host-owned candidate;
   - reject stale revision, checkpoint-digest mismatch, undeclared evidence,
     and unknown fields; and
   - advance only after explicit artifact approval.
4. Return to the packaged App and copied CLI and confirm exact project revision
   and checkpoint parity.
5. Append only path-redacted host verdicts and bounded counts to the C5 record.
6. Mark C5 complete only after both host receipts independently pass.

Receipt validation:

```bash
bash scripts/validate_macos_acceptance_host_receipt.sh \
  --receipt /absolute/path/to/codex-receipt.json
bash scripts/validate_macos_acceptance_host_receipt.sh \
  --receipt /absolute/path/to/claude-code-receipt.json
```

Focused checks:

```bash
codex plugin list
codex mcp list
claude plugin list
claude plugin details qiongli-next@qiongli-local
claude mcp list
git diff --check
```

Run those commands against the isolated profiles and exact installed clients.
No retired live-provider test or broad cybersecurity scan belongs to this
batch.

## Batch N1 — R5C C0-C5 completion review

Build one review ledger that maps each accepted claim to its source commit,
focused tests, package identity, and path-redacted receipt.

Review boundaries:

- C0 native baseline and package identity;
- C1 capture delivery, replay, acknowledgement, and deduplication;
- C2 assignment, resolution, lineage, and conflict handling;
- C3 catalog, incremental Portfolio, archive/restore, deletion, and rebuild;
- C4 App API, Desktop routes, localization, focus, narrow layout, and restart
  invalidation;
- C5 copied package, three-project continuity, App/CLI/Full-MCP parity, current
  2.x installation, restart discovery, and live host handoffs.

The review must identify unsupported or unknown surfaces explicitly. It must
not convert source tests, fixture hosts, registration files, or MCP health
checks into live-session claims.

Exit criteria:

- every C0-C5 completion gate has one exact evidence owner;
- no unresolved P0/P1 data-loss, migration, installer, or host-integration
  defect remains;
- no current readiness depends on a recognized 1.x source;
- no Qiongli-owned model credential, provider request, model response, or
  model-CLI launch exists in the accepted product path; and
- R5C is marked complete in the parent plan and roadmap.

## Batch N2 — Freeze the pre-Beta release contract

Create a target-and-claim matrix before implementing distribution:

| Target | Current evidence | Pre-Beta action | Claim before acceptance |
|---|---|---|---|
| macOS arm64 | Local ad-hoc packaged acceptance | Developer ID, notarization, update-chain, and clean-machine qualification | Local engineering acceptance only |
| macOS x86_64 | Not accepted | Native build and startup on Intel hardware, then the same package lifecycle | Unsupported/unknown |
| Windows x86_64 | Not accepted | Native build, Authenticode decision, portable package, installer lifecycle | Unsupported/unknown |
| Linux x86_64 | Not accepted | Native App/CLI package and clean-machine lifecycle | Unsupported/unknown |

Freeze:

- artifact naming and immutable versioned URLs;
- per-target SHA-256 identities;
- SBOM and provenance formats;
- signing, notarization, and timestamp requirements;
- signed update metadata and atomic rollback;
- state-preserving install, upgrade, repair, and removal semantics; and
- the rule that direct release assets precede package-manager projections.

No package manager may advertise an asset before the corresponding direct
artifact and clean-machine receipt exist.

## Batch N3 — Execute distribution qualification in dependency order

1. Qualify the macOS arm64 Developer ID/notarized candidate without changing
   its already accepted product semantics.
2. Run clean-machine install, first launch, current 2.x integration install,
   restart, upgrade, repair, removal, and user-state preservation.
3. Produce native macOS x86_64 evidence on Intel hardware; do not use Rosetta as
   the architecture claim.
4. Produce and qualify Windows x86_64 and Linux x86_64 direct artifacts.
5. Generate Homebrew, Scoop, and WinGet projections only from the finalized
   immutable release metadata.
6. Publish direct assets first. External manager listings remain
   `pending publication` until independently discoverable and accepted.

R5B legacy Python/Node source retirement remains after Beta acceptance and does
not move into this plan.

## Receipt and review rules

- Commit only path-redacted receipts, plans, schemas, tests, and source.
- Never commit generated Apps, homes, credentials, logs, screenshots, private
  project data, prompts, responses, or host conversations.
- Bind every receipt to the exact source and artifact digests.
- Separate source, registration, activation, MCP attachment, live session,
  signing, notarization, clean-machine installation, and public availability.
- Keep `publication_allowed: false` until the relevant promotion gate passes.

## Commit checkpoints

1. `test(acceptance): close live host continuity`
2. `docs(roadmap): close r5c continuity`
3. `docs(release): freeze pre-beta qualification`
4. `build(macos): qualify trusted arm64 package`
5. target-specific build and acceptance commits after their native evidence
   exists

Each checkpoint must be independently reviewable. A failed target disables only
that target claim and must not fall back to a legacy runtime, a direct model
backend, Rosetta, or an unverified latest asset.
