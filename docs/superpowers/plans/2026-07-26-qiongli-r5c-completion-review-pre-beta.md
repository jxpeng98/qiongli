# Qiongli R5C Completion Review and Pre-Beta Readiness Plan

Status: in progress — N0 tooling and package-bound preparation are complete;
system Host 2.x registration verification and two live receipts remain

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
2. the existing system Codex and Claude Code Hosts remain authenticated through
   their own account flows;
3. both system Hosts have a current 2.x Qiongli registration matching the
   isolated accepted Plugin digest, and the Plugin, Skill, and Full MCP remain
   visible after a fresh Host process; and
4. the host-visible acceptance profile contains the same bounded
   three-project fixture required by C5.

Authentication material, prompts, responses, conversations, tool bodies,
project identifiers, and absolute paths remain outside every receipt.

Readiness snapshot:

- the package-bound host fixture preparation landed in `6192cd20`;
- the observable-transition contract, graph-backed fixture, rejection probes,
  and package-bound receipt composer landed in `9884f494`;
- the isolated-install/system-Host split and current 2.x system registration
  binding landed in `71207f29`;
- fixture `r5c-c5-host-driven-v1` is prepared in the accepted App's isolated
  manual profile at project revision 2 with SHA-256
  `28dcd6a4f7ba34822503f2b6611dc9b887de34fbd0817836541dcac8dd418a9a`;
- the three-project App/CLI/Full MCP parity checks pass and preparation is
  idempotent;
- the package-bound validator now checks the exact binary, product source,
  fixture revision, and host-specific installed plugin digest; and
- the C0-C4 completion preflight at `24eb9359` reconfirmed all accepted commit
  ancestry, 156 project tests, 122 App-library tests, 19 App API tests, 116
  Desktop tests, the production Desktop build, and the focused Rust source
  gates; and
- only system 2.x registration migration/verification plus one real handoff in
  each already authenticated Host remains before this plan's completion review
  may start.

The reusable, explicitly non-final ledger is
`docs/superpowers/acceptance/2026-07-26-qiongli-r5c-c0-c4-completion-preflight.md`.

## Batch N0 — Close C5 live-host acceptance

Preparation already completed:

1. The bounded three-project fixture was created in the isolated installation
   profile without copying real user projects or authentication.
2. The canonical preparation receipt is bound to the accepted product receipt,
   exact copied binary, fixture digest, and project revision.
3. Re-running preparation verifies the same state and does not create another
   fixture.

### N0.1 — Codex live observation

1. Confirm the normal Codex `0.144.6` Host is already authenticated. Do not
   copy its credentials or log in the isolated profile.
2. Migrate or reinstall the system integration, then start a fresh Codex
   process and verify its current `2.0.0-alpha.2` registration, Plugin, Skill,
   and Full MCP attachment.
3. Execute the exact triad and rejection sequence in
   `tooling/release/acceptance/fixtures/r5c-c5-live-host-runbook.md`.
4. Compose and validate the Codex package-bound receipt with the explicit
   system registration path.
5. Confirm App/copied-CLI parity, cancel the bounded acceptance run, and remove
   the temporary observation.

Exit gate: one valid Codex receipt exists; project semantic revision remains
`2`; no private host material is retained by Qiongli.

### N0.2 — Claude Code live observation

Repeat N0.1 in the existing authenticated system Claude Code `2.1.216` profile.
Do not reuse a Codex conversation, candidate, evidence reference, checkpoint,
observation, or receipt.

Exit gate: one independently valid Claude Code receipt exists with the same
fixture and product identities and the Claude-specific installed Plugin
digest.

### N0.3 — C5 closure

1. Validate both generated receipts against the exact accepted root.
2. Re-run App and copied-CLI project parity after both acceptance runs.
3. Confirm both acceptance runs are terminally cancelled and no semantic
   revision changed.
4. Update the C5 acceptance record with only receipt digests and bounded
   verdicts.
5. Mark C5 complete only if every negative observation was fail-closed and the
   continuous three-transition chain passed in both hosts.

Receipt composition:

```bash
bash scripts/compose_macos_acceptance_host_receipt.sh \
  --observation /absolute/path/to/canonical-observation.json \
  --system-registration /absolute/path/to/qiongli-next-registration.json
```

Receipt validation:

```bash
bash scripts/validate_macos_acceptance_host_receipt.sh \
  --receipt /absolute/path/to/codex-receipt.json \
  --system-registration /absolute/path/to/codex-registration.json
bash scripts/validate_macos_acceptance_host_receipt.sh \
  --receipt /absolute/path/to/claude-code-receipt.json \
  --system-registration /absolute/path/to/claude-code-registration.json
```

Focused host checks:

```bash
codex plugin list
codex mcp list
claude plugin list
claude plugin details qiongli-next@qiongli-local
claude mcp list
git diff --check
```

Run those commands against the existing system profiles and exact installed
clients. The isolated profile remains install-only. No retired live-provider
test or broad cybersecurity scan belongs to this batch.

### N0.4 — Start the completion review

Only after N0.3 passes, freeze both receipt digests and begin Batch N1. If one
system Host lacks a matching current 2.x registration or fails acceptance, keep
C5 and the R5C completion review open; do not substitute fixture-only or
MCP-health evidence.

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
