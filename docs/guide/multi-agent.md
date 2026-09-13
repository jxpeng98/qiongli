# Work with other agents

Qiongli 2 uses the agents available in your Host, such as Codex or Claude Code.
The Plugin supplies research guidance and tools; your Host supplies the model and
any subagent or communication tools. Installing Qiongli does not start a team.

## Request a bounded review

After [installing the Plugin](cli-2x.md#first-use), ask:

> Have an independent agent review this methods section against the supplied
> study protocol. Return source-linked findings before editing the manuscript.

The coordinator gives the reviewer the relevant material, the decision to check
and a clear return format. An independent review requires an actual agent result.
One conversation taking several roles is self-review. If your Host cannot run a
subagent, Qiongli should explain the limit and continue only the work it can do.

For parallel work, give each agent separate candidate files or output areas.
One coordinator reconciles the results and presents proposed changes for approval.
Agents must not overwrite the same canonical research records concurrently.

## Ask another Host to contribute

Use the bundled `templates/agent-handoff.md` and
`templates/agent-review-packet.md` to carry the task, permitted sources, revisions
and exact candidate between Hosts. Transfer them through an authorized tool or
manually. A prepared packet is still awaiting review until a result comes back.
Compare it with the current source before accepting any edits.

There is no native `qiongli team-run` or Python orchestrator prerequisite in this
2.x flow. Automatic cross-Host scheduling and shared write authority are not
provided by the handoff templates.

See [collaboration and optional hooks](../advanced/agent-skill-collaboration.md)
for source tracking, continuity and the reply-only entry.
