# Qiongli R5C C5 Packaged Acceptance Execution Plan

Status: in progress — C5.1 and C5.2 passed; C5.3 packaged App and real-host
isolated installation/restart passed; live Codex and Claude handoffs await
system 2.x registration verification

Date: July 26, 2026

Target branch: `feat/r4b-ui-localization-polish`

Baseline: C4.5 source qualification commits `fd1e166a` and `dedfc7b4`

Parent plan:
`docs/superpowers/plans/2026-07-25-qiongli-r5c-cross-surface-continuity.md`

## Outcome

Qualify the complete R5C continuity path with one copied, ad-hoc-signed macOS
App. Its isolated home proves clean current-2.x install/migration, restart, and
three-project continuity. Live host-driven execution runs through already
authenticated system Codex and Claude Code profiles whose current Qiongli
registrations match that isolated accepted Plugin, without adding a Qiongli
model backend.

C5 is non-publishing engineering acceptance. It does not create a release,
notarize an App, copy Host credentials, or claim Windows, Linux, remote,
Marketplace, Beta, or public distribution readiness. Automated preparation
does not modify the real user home; only the explicitly approved current 2.x
system integration installation may do so for live acceptance.

## Frozen acceptance rules

- Build only from a clean committed head. The package and every receipt bind
  the exact source commit and content identities.
- Use `dist/macos-acceptance/current/manual-home` and disposable project roots.
  No fixture may write into the maintainer's real Codex, Claude, Qiongli, or
  project directories.
- Install only the current 2.x managed projections. Recognized 1.x content may
  appear only in the dedicated migration fixture and must be converted or
  reported as unresolved migration input, never as a current installation.
- Qiongli remains a native shell and control plane. It stores no provider
  credential, opens no model conversation, calls no provider API, and launches
  no model CLI as part of its own service path.
- Real client evidence is not inferred from files. Package source,
  registration, activation, MCP attachment, and a live host session remain
  separate facts.
- Authentication stays in each existing system Host profile. No login is
  required in the isolated installation profile, and there is no Qiongli OAuth
  flow. Credentials, prompts, candidate bodies, responses, conversations, tool
  bodies, registration paths, absolute paths, and project IDs never enter the
  acceptance receipt.
- A fixture or mocked host proves protocol behavior only. It cannot substitute
  for the required real Codex and Claude Code observations.
- No broad cybersecurity scan is part of C5. Security checks remain limited to
  isolation, package identity, path containment, current-product ownership,
  receipt binding, and absence of Qiongli-owned model transport.

## Acceptance fixture

Create three disposable registered projects with deterministic exact
relationships:

1. Project A and Project B share one source and one concept.
2. Project B and Project C share one method.
3. One reviewed capture lineage connects A to B without fuzzy label merging.
4. A second capture remains offline through one process restart before replay.
5. A divergent or unbound capture requires explicit assignment and
   item-scoped resolution.
6. One project is archived and restored before the final catalog rebuild.

Record only bounded counts and opaque or hashed evidence. The receipt may state
that expected shared identities and lineage were observed, but it must not
contain source text, artifact bodies, roots, or private record payloads.

## Batches

### C5.1 — Clean-head package and isolated product lifecycle

1. Commit the C4.5 acceptance record and verify `git status --short` is empty.
2. Run:

   ```bash
   pnpm desktop:macos:acceptance
   ```

3. Require the existing product-controlled harness to:
   - build the embedded Svelte/Tauri App from the committed head;
   - compose and ad-hoc-sign the non-publishing package;
   - materialize, verify, refresh, and remove Skills in the automated home;
   - install, verify, repair, restart, and remove current Codex and Claude Code
     projections;
   - migrate the dedicated nine-surface 1.x fixture to 2.x and leave no
     recognized current 1.x installation; and
   - publish the extracted App only after every automated check passes.
4. Validate
   `dist/macos-acceptance/current/qiongli-packaged-product-acceptance.receipt.json`
   against the exact source head, package digest, App digest, signing mode,
   isolated roots, current `2.0.0-alpha.2` content, and
   `publication_allowed: false`.

If the existing receipt does not expose enough bounded C1-C4 observations,
extend the acceptance schema and rejection fixtures before continuing. Do not
fill missing evidence by hand.

### C5.2 — Three-project packaged continuity

Extend the packaged acceptance example only where required to drive the
accepted native services through the copied CLI, App boundary, and Full MCP:

1. create/register the three deterministic projects and rebuild the portfolio;
2. enqueue one offline delivery, stop the process, restart from the same
   isolated home, then retry and acknowledge the same envelope identity;
3. exercise exact replay and duplicate suppression without creating another
   academic capture;
4. assign the divergent/unbound capture, select every required resolution
   disposition explicitly, preview, confirm, and preserve source-to-child
   lineage;
5. archive and restore one project, remove all derived portfolio state, and
   prove deterministic reconstruction from canonical registered artifacts;
6. compare copied CLI and App views after restart for delivery, assignment,
   resolution, catalog, query, and timeline state; compare Full MCP only for
   its public project-list and academic-graph portfolio contract, without
   expanding that contract to C1-C3 mutation or timeline tools; and
7. assert that every fixture root remains under the dedicated acceptance
   directory and that every canonical project digest is unchanged by derived
   deletion.

The automated receipt records source/package identities, bounded fixture
counts, state transitions, reconstruction equivalence, and path-redacted
parity verdicts. It does not claim a live model client.

### C5.3 — Isolated installation proof and system-Host restart

First run the accepted package with its isolated manual home:

```bash
pnpm desktop:macos:acceptance:open
```

Manual observations:

1. In **About**, verify the packaged source identity and
   `2.0.0-alpha.2`; confirm no source-build authority warning is shown.
2. In **Client Integrations**, install the current `qiongli-next`
   integration and verify source, Marketplace/registration, Skills, and Full
   MCP attachment separately. No `qiongli` legacy source may satisfy current
   readiness.
3. Quit and reopen the App with the same isolated home. Confirm the current
   integration and all three registered projects are rediscovered.
4. Restart Codex and Claude Code against the isolated profile only to confirm
   that current Plugin and Skills installation remains discoverable. Do not log
   either isolated Host in.

Then use the existing authenticated system Host profiles:

5. Migrate or reinstall each system integration to current
   `qiongli-next@2.0.0-alpha.2`, restart the normal Host, and verify its
   registration version and Plugin content digest match the isolated accepted
   installation.
6. In each real Host, run one revision-bound Qiongli handoff that reads project
   evidence through Full MCP, returns a host-owned candidate, and advances only
   after exact checkpoint/evidence validation and explicit artifact approval.
7. Return to the App and copied CLI. Confirm the accepted checkpoint and
   project revision agree, while prompts, responses, provider credentials, and
   conversation state are absent.
8. Exercise the packaged Captures, Portfolio, and Timeline routes after
   restart, including offline replay, explicit resolution, archive/restore,
   derived deletion, full rebuild, English/Chinese switching, focus
   restoration, and a narrow window.

If either existing system Host is not authenticated, cannot migrate to the
matching current 2.x registration, or cannot load the live integration, record
the exact external prerequisite as pending and keep C5 open. Do not relabel
automated protocol evidence as real-client acceptance.

Execution record on July 26, 2026:

- the accepted App showed `2.0.0-alpha.2`, the exact clean source commit, and
  verified packaged-product authority;
- App-driven current 2.x installation completed in the manual acceptance home,
  and Codex and Claude managed receipts survived App restart;
- a fresh Codex `0.144.6` process discovered, installed, and enabled
  `qiongli-next@personal`; its Full MCP registration was enabled;
- a fresh Claude Code `2.1.216` process discovered and enabled
  `qiongli-next@qiongli-local`, exposed the workflow Skill, and reported the
  Plugin MCP as connected;
- both isolated Host profiles reported `not logged in`, which is now the
  expected install-only state; no live model call was attempted there and no
  real revision-bound handoff is yet claimed; and
- the path-redacted partial record is
  `docs/superpowers/acceptance/2026-07-26-qiongli-r5c-c5-packaged-acceptance.md`.

### C5.4 — Receipt, review, and hand-off

Create one path-redacted C5 acceptance record that references, without copying
private contents:

- source commit, package/App/CLI/Plugin/Skills/Full-MCP identities;
- automated product-control receipt and three-project continuity receipt;
- bounded project, delivery, replay, acknowledgement, assignment, resolution,
  catalog, query, timeline, archive, restore, and rebuild counts;
- App/CLI/Full-MCP parity after restart;
- separate Codex and Claude source, registration, activation, attachment, and
  live-session observations;
- zero Qiongli provider credential, provider request, direct model response,
  and model-CLI launch verdicts;
- isolation and path-redaction verdicts;
- known limitations and `publication_allowed: false`.

Review the receipt against the package and source head, then commit only the
path-redacted acceptance record and roadmap closure. Generated Apps, homes,
credentials, logs, and disposable projects remain ignored local evidence and
must not enter Git.

## Focused gates

Before packaging:

```bash
pnpm --dir packages/qiongli-app-api check
pnpm --dir packages/qiongli-app-api test
pnpm --dir packages/qiongli-desktop check
pnpm --dir packages/qiongli-desktop test
pnpm --dir packages/qiongli-desktop build
cargo fmt --manifest-path packages/qiongli-native/Cargo.toml --all -- --check
cargo clippy --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked -- -D warnings
cargo check --manifest-path packages/qiongli-native/Cargo.toml \
  --workspace --all-targets --all-features --locked
git diff --check
```

Then run the clean-head packaged command and any focused tests added to the
acceptance example. Do not rerun unrelated legacy or broad security suites.

## Completion gate

C5 is complete only when:

1. one clean committed head produces the accepted copied macOS App and exact
   path-redacted receipts;
2. automated current 2.x Skills, Codex, and Claude Code install/verify/restart
   checks pass without touching the real home;
3. the three-project continuity fixture survives process restart, replay,
   duplicate suppression, assignment, resolution, archive/restore, derived
   deletion, and deterministic rebuild;
4. copied CLI, packaged App, and Full MCP agree after restart;
5. current 2.x Plugin and Skills remain discoverable after real Codex and
   Claude Code restart, each system registration matches the accepted isolated
   Plugin digest, and each Host completes one revision-bound handoff;
6. Qiongli owns no model credential, provider request, model response, or
   model-CLI launch in those workflows;
7. automated fixture writes remain inside the isolated home and disposable
   project roots; manually approved system writes are limited to current 2.x
   integration registration and managed Plugin/Skill projection; and
8. the receipt makes no public distribution, production signing,
   notarization, Tier 1, cloud relay, Marketplace, Beta, or Stable claim.

After C5, the next planning boundary is R5C completion review and the
pre-Beta distribution sequence. R5B legacy source retirement remains
post-Beta and does not move ahead of that review.

The proposed follow-on plan is
`docs/superpowers/plans/2026-07-26-qiongli-r5c-completion-review-pre-beta.md`.
