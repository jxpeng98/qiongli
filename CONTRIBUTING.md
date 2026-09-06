# Contributing to Qiongli

Start with the
[Qiongli 2 master roadmap](docs/superpowers/roadmaps/2026-08-02-qiongli-2-research-harness-master-roadmap.md).
It owns product direction, ordering, and the current execution horizon. The
[program ledger](docs/superpowers/roadmaps/qiongli-program-ledger-v1.json) owns
task state and accepted evidence. The current implementation plan selects a
bounded user outcome from that roadmap; existing `.trellis/` records remain
project knowledge without requiring Trellis skills or a task lifecycle.

## Four delivery lanes

| Lane | Use it for | Required gate |
|---|---|---|
| Daily development | Editing one bounded behavior | The smallest **Focused** check that can falsify the change |
| Pull request | Integrating a frozen exact head | PR review plus required exact-head **Slice** checks |
| Build | Proving one named source/target artifact | The named build and runtime checks for that target |
| Release | Qualifying immutable public bytes | Explicit **Acceptance**, release authorization, and independent verification |

Passing one lane does not authorize the next. In particular, a green PR or
successful build is not release approval.

## Daily development

The main Agent can plan, implement and check a bounded change directly. Use one
short execution plan for work spanning sessions; keep existing design/spec notes
where they still explain a contract. Routine fixes require no new task, duplicate
PRD/design/JSONL set, phase approval or journal. Delegate only independent work
with clear ownership when that reduces elapsed time; no fixed role chain is required.

Deliver the smallest complete behavior and record its focused result before
continuing. Once interfaces are stable, native persistence, App interaction and
tool binding can advance independently toward the same integration journey.
An unavailable live adapter or release receipt blocks its own readiness claim,
not offline implementation. Track remaining integration gaps in the current plan.

Run only affected checks while editing. Security, authorization, schema, path,
ownership, and data-loss changes still require their negative checks immediately.
Before committing or pushing, follow
[the delivery checklist](.github/delivery-checklists.md).

Keep changes in their canonical boundary:

- academic content: `content/`;
- native Qiongli 2 source: `packages/qiongli-native/`;
- stable command wrappers: `scripts/`, with implementations in `tooling/scripts/`;
- generated plugin/package payloads: regenerate through the supported
  materialization workflow; never edit generated copies directly.

## Pull requests

Use the [PR template](.github/pull_request_template.md) and target `2.x` from a
working branch. Keep the PR draft while its head is moving. When ready:

- runtime, workflow, action, fixture, mixed, unknown, or empty changes run the
  full Linux, macOS, and Windows Native CI Slice;
- clearly non-runtime documentation, process, and evidence-only changes keep the
  required contexts but use their lightweight report path;
- every head change invalidates earlier exact-head CI and review evidence.

## Builds

Use [Local Desktop Development and Packaging](docs/development/local-desktop-build.md)
for the macOS, Windows, and Linux build loop. Record the exact source and named
target. Cross-compilation proves compilation only; it does not claim target-native
runtime, signing, installer, or release acceptance.

## Releases

Release work starts only from an explicit release task after merge to `2.x`.
Follow the Release section of the delivery checklist and run the existing owner:

```bash
./scripts/release_ready.sh --version <version> --staging-dir <external-dir>
```

CI supplies evidence. A named human release decision authorizes publication, and
a separate decision authorizes announcement.
