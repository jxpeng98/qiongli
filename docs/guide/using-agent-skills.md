# Using Agent Skills

For Qiongli 2, follow the
[installation guide](cli-2x.md#install-and-upgrade-bundled-content), then start a
new Host session and check the tools actually available.

## Ask for the research task

Try “Read this paper and explain its findings and evidence limits,” “Build a
literature review from these sources,” or “Summarize this completed stage with
sources and changes.” A narrow request loads the relevant guidance; having a
project does not start the entire lifecycle. J2 polish preserves numbers,
citations, terminology and causal limits.

Design guidance follows the study's method and agreed protocol. There is no
fixed quota of papers, rival explanations or robustness checks. State the decision you need, the available material
and any requirements that must stay in force. Formal deliverables still require
their evidence and checks; a preregistration draft is not a registered study.
If you use an older build, update the CLI and refresh the Plugin.

Codex Plugins from beta.6 expose `$qiongli` and 20 workflow shortcuts, including
`$qiongli-paper-read`, `$qiongli-lit-review` and `$qiongli-stage-close`. Each reads
the shared Skill before its workflow. The 82 internal skill cards are not wrapped
separately. Claude retains one main Skill and also accepts natural-language
requests. Update an older cache before expecting the new entries to appear.

## Tools and saved changes

| Entry | Boundary |
|---|---|
| Standalone Skills | Use the Host's available tools and authorized materials; do not assume MCP is connected |
| Native Marketplace Plugin | Starts bundled Lite MCP with 14 tools |
| CLI-exported local Plugin | Starts bundled Full MCP with 32 tools; the user retains their Host and model |
| CLI | `qiongli doctor`, `qiongli project` and `qiongli help`; no Python runtime is needed |

Registration and cache verification do not prove that session tools have loaded.
If required MCP tools are missing, check the connection. Continue independent
work supported by available materials, but never imitate tool results or bypass
approval by directly editing a registered project. Writes retain previews,
explicit approval and current revision checks. Stage summaries preserve original
files; any deletion remains the user's own action.

[Graph and research continuity](cli-2x.md#research-graph) · [Task recipes](task-recipes.md)

## Journal fit and pre-submission review

Tell Qiongli which decision you need:

- “I have chosen this journal. Check the requirements for an initial research
  article and suggest changes to this draft.” A5 verifies the applicable rules
  and maps them to manuscript locations and proposed changes.
- “Read this draft and recommend suitable journals.” H5 starts with the paper's
  contribution, methods and evidence, then compares eligibility, readers, costs
  and other constraints that matter to you.
- “Review this manuscript's methods and claims.” H3 provides a critique; H4
  focuses on supported blockers. Neither automatically rewrites or submits it.

Decision-relevant journal rules include their sources, checked dates and article
type/stage. Local profiles help discovery but do not establish current policy.
With only an abstract or unavailable sources, the answer stays provisional and
identifies what is missing. There is no fixed number of recommendations, no
promised acceptance, and no scientific flaw inferred merely from a venue mismatch.

Proposed edits separate presentation and reporting from new analysis, data or
author commitments. You can request a short answer without creating a project;
formal work retains the existing artifacts and write confirmations. Multiple
review lenses in one model are self-review, not independent reviewers. Confidential
journal assignments also depend on that journal's AI-use and confidentiality rules.
