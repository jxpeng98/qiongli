# Using Agent Skills

This page's current entry targets 2.x. Follow the
[installation guide](cli-2x.md#install-and-upgrade-bundled-content), then start a
new Host session and check the tools actually available.

## Ask for the research task

Try “Read this paper and explain its findings and evidence limits,” “Build a
literature review from these sources,” or “Summarize this completed stage with
sources and changes.” A narrow request loads the relevant guidance; having a
project does not start the entire lifecycle. J2 polish preserves numbers,
citations, terminology and causal limits.

The current development source also makes design guidance depend on the study's
method and agreed protocol. Roles no longer impose a fixed number of papers,
rivals or robustness checks. State the decision you need, the available material
and any requirements that must stay in force. Formal deliverables still require
their evidence and checks; a preregistration draft is not a registered study.
These changes are included in beta.6. Update the CLI, then refresh the Plugin.

New Codex Plugin builds expose `$qiongli` and 20 workflow shortcuts, including
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

<details>
<summary>Legacy 1.x client and runtime instructions</summary>

The retained instructions below apply to older installations. Their Python,
bootstrap and CLI requirements do not apply to native 2.x.


Qiongli installs one agent-facing skill system, but each client exposes it differently. Use this page after installation when you need to know what to type inside Codex, Claude Code, Antigravity, Hermes, or the shell.

## Naming Model

| Name | Meaning | Where it appears |
|---|---|---|
| `qiongli` | Public plugin, CLI, and user-visible skill name | Skillsplace, npm, PyPI, Codex `/skills`, shell commands |
| `qiongli-workflow` | Portable skill package directory | `~/.codex/skills/`, `~/.claude/skills/`, `~/.gemini/antigravity/skills/`, `~/.hermes/skills/`, plugin payloads |
| `skills/*/*.md` | Internal academic capability cards | Repository source and orchestrator injection |

Most users should look for `qiongli`, not `research-paper-workflow`. The directory name `qiongli-workflow` remains for compatibility with existing installers and release artifacts.

## Client Entry Points

| Client | Discover | Invoke | Notes |
|---|---|---|---|
| Codex | `/skills` | `$qiongli`, `$qiongli-lit-review`, `$qiongli-academic-write`, or another generated Qiongli workflow wrapper | Codex does not expose Qiongli workflows as custom slash commands. Plugin installs generate thin wrapper skills that route to the same canonical workflows as Claude slash commands. Restart Codex after installing or upgrading. |
| Claude Code | Plugin UI or `/plugin` commands | `/paper`, `/lit-review`, `/paper-write`, `/code-build`, or natural language asking for Qiongli | Plugin installs command wrappers plus the portable skill package. |
| Shell | `qiongli check` | npm: `qiongli install`, `qiongli update`, `qiongli project ...`; full runtime: `qiongli doctor`, `qiongli task-run`, `python3 -m bridges.orchestrator ...` | npm/npx is Python-free asset management. Full runtime commands require `pipx install qiongli` and Python 3.12+. |

## Runtime Architecture Flow

This is the end-to-end path from a user request to runtime choice, preview, execution, and durable outputs.

```mermaid
flowchart TB
    subgraph Entrypoints["Entry surfaces"]
        Request["Academic request<br/>topic, paper type, constraints"]
        Client["Client skill/plugin<br/>Codex, Claude Code,<br/>Claude Desktop/Web"]
        Npm["npm/npx asset manager<br/>install, setup, update,<br/>refresh, upgrade, check"]
        RuntimeCli["Full runtime CLI/MCP<br/>pipx, pip, bootstrap full"]
        Request --> Client
        Request --> Npm
        Request --> RuntimeCli
    end

    subgraph ProjectState["Project usage state"]
        Manifest{"Project manifest exists?<br/>.qiongli/guidance_manifest.yaml"}
        Auto["Implicit project state<br/>active_subject: auto"]
        Configured["Configured project state<br/>subject, venue profiles,<br/>method lenses, strictness"]
        LocalGuidance["Human guidance<br/>.qiongli/local_guidance.md<br/>.qiongli/guidance.d/*.md"]
        Npm --> Manifest
        RuntimeCli --> Manifest
        Manifest -->|no| Auto
        Manifest -->|yes| Configured
        Auto --> LocalGuidance
        Configured --> LocalGuidance
    end

    subgraph Routing["Task routing"]
        Contract["Task contract<br/>Task ID, stage, outputs,<br/>evidence rules, quality gates"]
        Subject["Resolve subject context<br/>explicit domain > project state<br/>> temporary inference > core"]
        Runtime{"Smallest runtime<br/>that fits the job"}
        Client --> Contract
        RuntimeCli --> Contract
        LocalGuidance --> Subject
        Contract --> Subject
        Subject --> Runtime
    end

    subgraph RuntimeChoice["Runtime choices"]
        SkillOnly["Skill/plugin only<br/>read guidance,<br/>draft or review artifacts"]
        Provider["Literature provider<br/>MCPB or bundled Node MCP<br/>status, search, evidence export"]
        Preview["Full runtime preview<br/>doctor, task-plan,<br/>task-run without agents"]
        Execute{"run_agents == true<br/>and doctor passes?"}
        Agents["Controlled agent run<br/>controller, primary,<br/>reviewer, verifier"]
        Runtime --> SkillOnly
        Runtime --> Provider
        Runtime --> Preview
        Preview --> Execute
        Execute -->|no| PreviewResult["Preview result<br/>safe plan and runtime notes"]
        Execute -->|yes| Agents
    end

    subgraph Outputs["Outputs and learning loop"]
        Formal["Formal outputs<br/>RESEARCH/[topic]/..."]
        Trace["Trace bundle<br/>.qiongli/trace/runs/&lt;run_id&gt;/"]
        Proposal["Guidance proposal<br/>manifest patch plus local notes"]
        Apply{"guidance_mode"}
        SkillOnly --> Formal
        Provider --> Formal
        PreviewResult --> Trace
        Agents --> Formal
        Agents --> Trace
        Trace --> Proposal
        Proposal --> Apply
        Apply -->|propose| LocalGuidance
        Apply -->|apply| Manifest
    end
```

The important boundary is that installation does not imply execution. npm/npx installs and refreshes client assets, while the full runtime has to be installed and called explicitly before `doctor`, MCP orchestration, provider setup, or agent execution can run.

## Codex Usage

After installing from Skillsplace, npm, PyPI, or `qiongli upgrade --target codex`, restart Codex and check:

```text
/skills
```

You should see `qiongli`. Plugin-first installs also generate thin workflow wrapper skills such as `qiongli-lit-review`, `qiongli-academic-write`, `qiongli-paper-read`, and `qiongli-proofread`. Invoke the main skill or a wrapper with a concrete research task:

```text
$qiongli plan a systematic review on retrieval augmented generation in education
$qiongli-lit-review retrieval augmented generation in education
$qiongli-academic-write related work for my CHI paper
$qiongli design an empirical study about ai writing support in universities
$qiongli prepare a submission checklist for my CHI paper
```

Do not expect `/qiongli` or `/lit-review` to work in Codex. Slash commands are reserved for Codex built-ins and installed slash surfaces that Codex itself exposes. Qiongli's Codex-facing entries are skill invocation forms. The generated wrappers are intentionally thin: `$qiongli-lit-review` routes to the same `workflows/lit-review.md` source used by Claude Code `/lit-review`.

If `/skills` only shows `research-paper-workflow`, you are seeing an older global install. Run the current installer or upgrade path, restart Codex, and check again:

```bash
qiongli upgrade --target codex --overwrite
```

Current qiongli installers remove confirmed legacy `research-paper-workflow` global skill directories during upgrade. Use `qiongli clean --globals --dry-run` first if you want to preview global cleanup separately.

## Claude Desktop / Claude.ai Usage

Claude Desktop should expose Qiongli as the installed `qiongli` skill or direct
plugin entry. Start with a natural academic request such as:

```text
Use Qiongli to plan a literature review on retrieval augmented generation in education.
Use Qiongli to read this DOI and extract the claim, method, evidence, and limits.
Use Qiongli to prepare a rebuttal matrix for these reviewer comments.
```

When direct plugin command wrappers are visible, `/qiongli` is the unified entry
router. It delegates to the same workflow files used by the narrower stage
commands. The Qiongli Literature Provider MCPB or bundled literature MCP adds
provider search tools; the skill instructions alone do not imply
`provider_connected` literature search.

## Claude Code Usage

Claude Code can expose Qiongli through workflow entry markdowns. The common user-facing commands are:

| Command | Use when |
|---|---|
| `/qiongli` | You want the unified Qiongli entry to choose the right workflow. |
| `/paper` | You want the guided paper workflow and paper-type routing. |
| `/lit-review` | You need literature search, screening, extraction, or synthesis. |
| `/paper-read` | You want deep analysis of one paper. |
| `/find-gap` | You need to identify and rank research gaps. |
| `/study-design` | You need empirical, qualitative, or mixed-method study design. |
| `/paper-write` | You want manuscript drafting from an existing research workspace. |
| `/code-build` | You need strict academic code specification, planning, execution, review, and reproducibility checks. |
| `/submission-prep` | You need journal or conference submission packaging. |

These slash workflows are convenience entrypoints. They all route into the same Qiongli task contract and skill package.

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

## Shell And Orchestrator Usage

Use the shell CLI when you need to inspect, upgrade, validate, or run explicit task IDs:

```bash
qiongli check
qiongli upgrade --target all
qiongli doctor --project-dir .
```

Use the orchestrator when you want explicit task planning or multi-agent execution:

```bash
python3 -m bridges.orchestrator task-plan \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd .

python3 -m bridges.orchestrator task-run \
  --task-id F3 \
  --paper-type empirical \
  --topic ai-in-education \
  --cwd . \
  --triad
```

## Recommended Flow

1. Install through Skillsplace for one-client usage, or use `qiongli upgrade --target all` for global multi-client usage.
2. Restart the target client so its skill registry and workflow discovery refresh.
3. In Codex, confirm `/skills` shows `qiongli` and invoke `$qiongli`.
4. In Claude Code, use `/paper`, `/lit-review`, `/paper-write`, or `/code-build`.
5. For repeatable task execution, use `qiongli doctor` and `python3 -m bridges.orchestrator task-plan|task-run`.

Qiongli writes research artifacts under `RESEARCH/[topic]/` when a workflow or orchestrator task produces durable outputs. Project-local integration files are only written when you explicitly run `qiongli init` or choose project install parts.
</details>
