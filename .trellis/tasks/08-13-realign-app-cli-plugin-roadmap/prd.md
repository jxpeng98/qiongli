# Realign roadmap for App CLI and Plugin priorities

## Goal

Make the next Qiongli 2 delivery sequence explicit and evidence-bound:

1. first, close the packaged App's bundled CLI and Plugin effectiveness path;
2. second, improve the bundled Plugin's academic quality with executable gates;
3. only then resume unrelated M1 and later roadmap work.

This task changes planning authority and deploys the two bounded child tasks. It
does not claim Alpha 3 publication, Stable readiness, or target-branch
integration.

## Confirmed Baseline

- The packaged App already owns native CLI preview, receipt-owned
  install/update/remove, PATH configuration, and a fresh login-shell version
  test. That path needs regression evidence, not a replacement installer.
- Packaged Plugin control already verifies the embedded product, materializes
  receipt-owned Codex and Claude sources, registers their local marketplace,
  and probes the official clients. It currently stops at
  `installed-host-action-required` and asks the user to copy official Host CLI
  commands.
- The accepted trust boundary forbids direct Host-cache mutation,
  undocumented UI automation, and inferring activation from copied files or
  registration alone.
- The user selected one confirmation: the App may execute only the fixed,
  target-matched official Host CLI operations shown by the approved preview,
  then must obtain fresh positive evidence before reporting Ready.
- The canonical Skill audit currently scans 82 Skills and reports 74 complete.
  Eight Coursework/Dissertation Skills have explicit contract gaps, while the
  checked-in 71/71 report is stale.
- The academic-quality runner currently averages scores declared by its 12 YAML
  fixtures. Those numbers are not executed observations. Evaluation Truth V1
  already provides typed artifact assertions and a fail-closed success
  predicate that can be reused.
- The working head contains local EVAL-401 through EVAL-407 work, but it is 19
  commits ahead of `origin/2.x`; roadmap wording must not present it as accepted
  on the target branch.

Detailed evidence is recorded in
`research/current-priority-audit.md`.

## Requirements

### R1. Make the roadmap truthful and current

- Update the master roadmap's status, verified baseline, immediate dependency
  sequence, M0/M1 execution language, and first-90-days projection.
- Preserve M0 release qualification and M2-M7 scientific-trust work as later
  gates; move them out of the immediate lane rather than deleting them.
- Correct the stale 232/233 Task-ID count without creating a new master-roadmap
  checklist ID or duplicating the roadmap backlog into Trellis. ADR 0213's
  `ARC-213` is decision-log metadata, not a new roadmap backlog item.
- Update the narrow product-control spec so future tasks inherit the same
  activation-before-quality order and evidence boundary.

### R2. Deploy two ordered child tasks

- P0: `.trellis/tasks/08-13-close-app-cli-plugin-activation` owns the App,
  official Host CLI, and Ready vertical.
- P1: `.trellis/tasks/08-13-make-plugin-quality-executable` owns Evaluation
  Truth V1 academic fixtures and the bounded canonical Skill repairs.
- P1 must not start until P0 is accepted or explicitly deferred with evidence.
- Only one Trellis implementation task may be active at a time.

### R3. Freeze the P0 product outcome

- Existing native CLI lifecycle behavior remains the owner; no second CLI
  installer or command framework is introduced.
- Because accepted ADR 0206 is immutable and currently leaves Host actions to
  the user, P0 must record the user's new decision in a superseding ADR rather
  than editing the frozen decision in place.
- One confirmed integration preview may run only native, fixed Codex/Claude
  command plans for the selected target, scope, expected version, and managed
  source.
- Ready requires receipt-owned App state plus fresh official Host evidence:
  exact Plugin identity/version enabled, exact managed/cached bundle identity,
  and Full MCP attached. Claude must also expose the expected
  `qiongli-workflow` component; Codex Skill presence is proven only through the
  exact activated bundle identity because its Plugin CLI has no component
  inventory command.
- No plan may claim that a model session invoked the Skill unless a separate
  fresh Host-session receipt observes that event.
- Timeout, non-zero exit, oversized/malformed output, version/source/cache
  mismatch, or missing Skill/MCP evidence must not produce Ready.

### R4. Freeze the P1 quality outcome

- Remove self-declared dimension averages as the academic-quality authority.
- Reuse Evaluation Truth V1 typed assertions against case-owned inputs and
  captured artifact findings; do not add a second evaluator framework.
- Repair only failures exposed by the executable corpus and the current eight
  canonical Skill audit rows.
- Regenerate the Skill quality report and verify staged Codex/Claude Plugin
  payloads from canonical sources.
- Model-dependent `claude plugin eval` ablation remains an optional observed
  lane, not a deterministic CI or completion gate for this slice.

### R5. Keep all claims evidence-bound

- Local tests, exact-head CI, packaged acceptance, live Host evidence, and
  publication authorization remain separate claim classes.
- A failed or unobserved child gate remains open; documentation must not convert
  it into accepted status.

## Acceptance Criteria

- [x] The master roadmap and product-control spec consistently show P0 App
      CLI/Plugin effectiveness before P1 Plugin quality and broader M1 work.
- [x] The roadmap retains 233 unique long-term Task IDs and introduces no new
      checklist ID for this Trellis execution slice.
- [x] The roadmap states that EVAL-401 through EVAL-407 are local working-head
      evidence until integrated into `2.x`.
- [x] Both child tasks contain converged `prd.md`, `design.md`, and
      `implement.md` files with explicit dependencies, non-goals, validation,
      and rollback boundaries.
- [x] The P0 plan binds one approval to fixed official Host commands and fresh
      Ready evidence without arbitrary shell execution or direct cache writes.
- [x] The P0 plan records the material trust-boundary change in a new ADR while
      retaining ADR 0206's cache, ownership, conflict, and Host-policy rules.
- [x] The P1 plan replaces fixture-declared scores, targets the current eight
      canonical Skill gaps, and makes no unsupported model-quality claim.
- [ ] The parent closes before P0 is activated; P1 remains queued until P0 is
      accepted or explicitly deferred.
- [x] No edit represents the local package as publicly qualified, Stable, or
      accepted on a branch where the evidence is absent.

## Out of Scope

- Implementing either child task inside this planning task.
- Public Alpha 3 publication, signing, notarization, package-manager release,
  or protected release authorization.
- New Hosts, providers, domain packs, generic command execution, or broad UI
  redesign.
- Typed Research Kernel, Evidence v2, reproducibility, Restricted/Offline mode,
  and post-Stable collaboration implementation.
