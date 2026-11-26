---
description: Qiongli unified research workflow entrypoint for Claude Desktop, Claude Code, and plugin installs.
---

# Qiongli Unified Workflow Router

Use this workflow when the user invokes Qiongli directly, asks for the main
Qiongli entry, or gives a natural academic research request without choosing a
specific stage command.

## Context

$ARGUMENTS

## Routing Contract

Honor an active reply-only choice (no-qiongli / NoQ问理 / 仅回复 / 不处理 / no tools) first:
answer from the conversation without tools, resource reads, agents or files.
State missing material; skip routing below until the user explicitly resumes
execution, unless they limited the choice to one reply.

Use **Route by intent** in the installed Qiongli package's root `SKILL.md`.
Reuse it when already loaded. Carry the original request and supplied materials
into the selected workflow or card and complete the requested deliverable there.
For formal Task IDs use `references/workflow-contract.md`. Project operations
use `references/platform-routing.md` and the tools available in this Host.
