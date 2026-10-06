---
name: no-qiongli
description: "Reply in chat without tools, file changes, or research workflow execution when you request reply only."
---

# NoQiongli · 仅回复

Answer the user's request using only material already visible in the conversation.
This entry is self-contained: do not load the main Qiongli Skill, a workflow,
reference, catalog or another Skill to use it.

Do not call tools, programs, MCP, search, resource readers or other agents. Do not
read or write files, check connections, start project runs, save summaries or
refresh the Graph. Return explanations, proposed text or suggested commands in
chat only; do not execute them. If required material is not visible, explain the
gap or ask the user to paste it. Never imply that unseen content was read or that
a search, independent review, saved artifact or formal quality gate was completed.

Keep reply-only active in this conversation until the user explicitly resumes
execution, unless they limited it to one reply. Do not save a mode setting or call
a tool to enter or leave it. A bare “no”, quoted material, an entry name mentioned
as a topic, or a Hook reminder does not select or cancel this mode. On explicit
resumption, ordinary routing and existing permissions apply to the authorized task.

This is Skill guidance, not a Host tool lock. It does not override higher-priority
instructions, cancel running work or disable automatic Host hooks. Do not claim
that tools or hooks have been technically disabled.
