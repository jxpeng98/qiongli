# Extend Qiongli

Start with the smallest change that solves a real research or installation problem.
Find the source that owns the behaviour, then update its consumers through the
existing build. Do not add another workflow or runtime when an existing one can
handle the task.

## Choose the right source

| Change | Source |
|---|---|
| Task outputs and research requirements | `content/standards/` |
| Reusable research guidance | `content/skills/`, `content/roles/` and `content/templates/` |
| Main Skill, shortcuts and stage references | `content/workflow/` |
| Plugin metadata | `content/distribution/plugins.yaml` |
| Public MCP names and schemas | `content/mcp-contracts/` |
| Native CLI, MCP, project, Graph or installation behaviour | `packages/qiongli-native/` |
| Native package assembly and release checks | `tooling/scripts/native_*.py` and the relevant workflow |
| Documentation | `docs/` and its Chinese counterpart under `docs/zh/` |

The Python and old npm product trees remain compatibility sources. They are not
the implementation path for a 2.x runtime fix. Generated Plugins, installed caches
and embedded packs must be rebuilt rather than edited directly.

## Keep guidance proportionate

A Skill should state the requested outcome, evidence requirements and boundaries,
then let the model choose a suitable method. A narrow answer need not run the
whole paper lifecycle. Protocol requirements, source attribution, independent
review when required, and approval for project writes remain binding.

Reuse the main Skill and shared stage reference for shortcuts. For a new internal
Skill, update its registry entry and affected task mapping. Add only the templates
and examples needed to explain the distinct task. See [subject guidance](subject-packaging-model.md)
for changes that belong in a profile instead.

## Check and integrate

Follow the repository's `AGENTS.md`, `CONTRIBUTING.md` and current bounded plan.
Run the closest relevant content, CLI or packaging checks; keep refusal and data-loss
cases. For documentation, build the site with `npm run docs:build` and check the
commands against the current CLI. Update both languages together.

Review the diff, record checks and remaining gaps once in the plan and ledger,
and integrate the scoped commit locally. A passing test does not establish live
Host or release acceptance. Publication follows the separate
[release policy](../maintainer/release-branch-policy.md).
