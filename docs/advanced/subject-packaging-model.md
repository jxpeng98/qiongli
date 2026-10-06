# Subjects and shared research guidance

Qiongli uses one shared research workflow. Subject guidance adds relevant methods,
terminology and reporting expectations without replacing that workflow.
You do not need a separate CLI installation for every discipline.

## Tell the Host what matters

Install the [Plugin](../guide/cli-2x.md#first-use), then describe the field,
research question, available material and any agreed protocol. For example:

> This is an economics study using staggered difference-in-differences. Review
> the identification assumptions and the diagnostics justified by this design.

The model can read relevant bundled profiles and apply them to the task.
A profile is guidance, not evidence or current journal policy. Verify external
requirements when they affect a decision. Saving project decisions retains the
existing approval and revision checks.

The 1.x `project set-subject`, `project set-venue` and `--domain` examples are not
native 2.x CLI commands. Use the Host to state research context and `qiongli project
--help` for supported project operations.

## What package terms mean

`core` supplies shared guidance. A subject layer can add profiles or focused
instructions. `complete` and `focused` describe content coverage; composite packs
combine named subject layers. These are content concepts, not extra runtimes or
a promise that every subject combination is published through every channel.
Use `qiongli content --json` to inspect the profiles in your installed build.

## Maintain a subject layer

Edit `content/subjects/`, its `catalog.yaml`, relevant
`content/skills/domain-profiles/` and `content/venue-profiles/`. Reuse a shared
Skill when the difference can be expressed as a method or reporting rule.
Add a separate Skill only for a distinct, reusable task.

The existing materializer supports compatibility package checks; native release
payloads follow the native build contract. Do not edit generated Plugin caches or
assume a content materialization command publishes a native release.
See [extension guidance](extend-qiongli.md) and [repository structure](../development/repository-structure.md).
