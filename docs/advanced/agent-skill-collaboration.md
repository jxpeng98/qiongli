# Agents, collaboration and optional hooks

Qiongli 2 keeps the model in your chosen Host. Qiongli supplies research
contracts, source-bound handoffs and project tools. Installing the CLI-exported Plugin gives
you Skills and Full MCP; it does not create another agent or change your model.

## Reply only

Beta.6 includes the independent **`no-qiongli`** Skill.
In Codex, select it as `$no-qiongli`; in other Hosts, select `no-qiongli` from the
installed Skills. You can also say **“reply only”** or **“no tools”** naturally.
The entry answers directly without loading the main Skill or research workflow:

> Reply only for this conversation: explain the paragraph below. Do not use tools
> or read or save files.

Codex and Claude Plugin packages place it at `skills/no-qiongli/SKILL.md`, beside
the research entry. Update the Plugin and reload its Skills to discover it.
Skills-only exports include `workflow/no-qiongli/SKILL.md`; that self-contained
directory can be installed as a Skill through your Host. The export itself does
not register a Host or connect MCP. There is no `qiongli no-qiongli` CLI command.

The model answers from visible conversation content, skipping resource reads,
search, MCP, agents, project checks and Graph updates. If material is missing,
it asks for an excerpt or explains the limit. The choice stays active until you
explicitly resume execution; “for this reply only” limits it to one response.
It also applies when using a workflow shortcut. Ordinary requests keep their
existing routing, and “no” in a quoted passage does not select this mode.

This is a Skill instruction, not a CLI flag or a Host permission lock. It does
not change higher-priority Host rules, cancel running work or disable automatic
Host hooks. To remove this Plugin's context hooks, separately use
`qiongli install plugin --hooks off` and complete its installation confirmations.

## Choose the smallest useful collaboration

| Need | Use | Completion evidence |
|---|---|---|
| Independent methodological judgment | A fresh native subagent reviewing the same bounded sources | Actual reviewer result, execution identity and sources read |
| Draft plus review | A writer, then a reviewer of the exact candidate | Candidate identity, findings and coordinator's reconciliation |
| Independent components | Native subagents with separate output/file scopes | Returned components and an integration check |
| Another Host's review or edits | A portable handoff and returned review/candidate | Matching task, source version and candidate; unresolved conflicts remain visible |

Ask naturally, for example: “Use an independent agent to check whether this
results section respects our study design. Return findings before changing the
manuscript.” Small edits can stay in one conversation. There is no required
agent count or discussion round count.

Qiongli's `model-collaborator` guidance uses the Host's actual delegation tools.
The coordinator gives each participant an outcome, source scope, file ownership
and return format, then collects real results. A role label or a queued task
is not an independent completed review. If no reviewer can run, that requirement
stays open; same-conversation self-review is labelled accordingly.

Native tool availability depends on the running Host, configuration and model
integration. Preserve the user's model settings. See the official
[Codex subagent guide](https://learn.chatgpt.com/docs/agent-configuration/subagents)
and [Claude Code subagent guide](https://code.claude.com/docs/en/sub-agents).

## Work with another Host

Use the bundled `templates/agent-handoff.md` to send the requested outcome,
permitted source excerpts/files, observed source hashes or revision, exact
candidate and allowed actions. The other Host returns
`templates/agent-review-packet.md` or candidate edits. Use an available,
authorized communication tool, or transfer the packet manually. Prepared packets
remain **awaiting external review** until a result returns.

The coordinator compares the returned source/candidate identity with the current
project, resolves disagreements against evidence and previews any accepted
changes. Stale reviews need reconciliation or a new review. One coordinator owns
canonical research writes; separate candidate files or worktrees avoid concurrent
overwrites. Keep claim/decision IDs, citekeys, source anchors, design limitations
and previous stage-summary links across the exchange. A review is not new
primary evidence for the Graph.

Do not transfer private conversations, credentials or approval tokens. Existing
Full MCP runs are bound to their starting Host and authenticated reads; a packet
does not transfer that checkpoint or its authority. Automatic cross-Host task
claiming, concurrent canonical writes and crash recovery remain later work under
ADR 0218. Portable review/edit exchange is available without those features.

## Optional context hooks

A hook can remind the model what to recheck after compaction, resume or child-agent
startup. The CLI provides `qiongli hooks context`: a native command
that reads an event JSON object from stdin and returns a brief context reminder.
It needs no Python, Node, MCP connection or model call. It does not read project
files or transcripts, save summaries, approve writes or keep a completed task
running. Hooks are optional; ordinary Skill routing remains sufficient.

In beta.6, `qiongli install` offers context hooks
when you choose Plugin. They default to off on first install; later installs and
upgrades preserve your choice. You can also choose explicitly:

```sh
qiongli install plugin --hooks context
qiongli install plugin --hooks off
```

The preview shows the events and command before you confirm file changes. The
configuration lives in the Plugin manifest and calls its bundled native binary;
no separate script runtime or MCP service is needed. Each Host has its own
choice and installation. `--hooks off` removes this Plugin's reminder entries.
It leaves manual Host settings intact. Avoid enabling the same reminder through
both the Plugin and a manual configuration.

After registration, Codex still requires you to review and trust the commands in
`/hooks` or its Hook settings. A changed definition can require trust again.
Claude Code needs 2.1.139 or newer for command hooks with `args`; the guide
checks this before writing files. Check the Plugin entries in `/hooks`. Reload the Plugin or start a new session, then
verify that resume/compaction or child startup delivers a reminder. Export and
registration checks do not prove event delivery. To inspect exported configuration,
use `qiongli app plugin-source-status --target codex --destination /absolute/path/qiongli-next`:
`source.context_hooks` reports the receipt-verified choice; `host_state` remains
`not-verified`. Use `--target claude` for Claude Code.

If you prefer manual Host configuration, the following example uses an absolute
native binary path; replace it with yours, preserving
shell quoting. On Windows use the absolute `qiongli.exe` path and quote/escape it
for the Host's command shell and JSON.

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "resume|compact",
        "hooks": [
          {
            "type": "command",
            "command": "\"/absolute/path/qiongli\" hooks context",
            "timeout": 5
          }
        ]
      }
    ],
    "SubagentStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"/absolute/path/qiongli\" hooks context",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

For Codex, merge these entries into the appropriate `.codex/hooks.json` and
review/trust them in `/hooks`; project-local configuration also needs project
trust. For Claude Code, merge the `hooks` entries into the appropriate
`.claude/settings.json`. Keep other entries intact and avoid registering this
same reminder at multiple scopes. Configuration and event support follow the Host's own
[Codex hooks documentation](https://learn.chatgpt.com/docs/hooks) or
[Claude Code hooks documentation](https://code.claude.com/docs/en/hooks).

Check `qiongli hooks context --help` first: an older CLI may not
contain this command. To inspect the protocol locally, pipe a SessionStart
object with `source: "resume"` into the command and close stdin. It returns
`hookSpecificOutput.additionalContext`; unsupported events return `{}`. Invalid
or over-64-KiB input exits 1 without printing its contents. The Hook result is
only a reminder, never evidence of an independent review or saved research.

This implementation uses command hooks only. Do not assume all Hosts implement
the same prompt/agent/MCP hook handlers. Confirm delivery in the actual Host;
protocol tests alone do not qualify a live installation. The historical
[observed capability matrix](/guide/agent-host-capability-matrix) keeps its
original evidence scope.
