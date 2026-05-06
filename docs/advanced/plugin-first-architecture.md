# Plugin-First Architecture

Research Skills is distributed as one plugin package, not as dozens of separate academic-skill plugins.

## Definitions

| Layer | Path | Role |
|-------|------|------|
| Main plugin | `plugins/research-skills/` | Install, discovery, version, marketplace, and platform entrypoints. |
| Portable skill package | `research-paper-workflow/` | The cross-client runtime skill loaded by Codex, Claude Code, and Gemini. |
| Source skill specs | `skills/` | Canonical academic capability specs maintained by this repository. |
| Workflow commands | `research-paper-workflow/workflows/` and `plugins/research-skills/commands/` | User entrypoints such as `/paper`, `/lit-review`, and `/code-build`. |

## Plugin vs Skill

A **plugin** is the distribution container. It owns manifests, marketplace metadata, command wrappers, bundled skills, and platform-specific install surfaces.

A **skill** is the reusable execution contract. It tells the model how to perform a specific research task, what inputs are required, what artifact to write, and what quality bar must be met.

This repo keeps one plugin, `research-skills`, and ships one portable skill package, `research-paper-workflow`, inside it. The 71 academic skill specs remain internal capability cards under `skills/`; they are synchronized into the portable package during release.

## Source Of Truth

- Edit source skills in `skills/`.
- Edit portable package shell files such as `research-paper-workflow/SKILL.md`, `workflows/`, and `references/` directly.
- Run `bash scripts/sync_skill_package.sh --target all` before testing artifacts or publishing.
- Do not edit `plugins/research-skills/skills/research-paper-workflow/` by hand; it is a distribution copy.

## Platform Surfaces

| Platform | Manifest | Runtime entry |
|----------|----------|---------------|
| Codex | `plugins/research-skills/.codex-plugin/plugin.json` and `.agents/plugins/marketplace.json` | `skills/research-paper-workflow/` |
| Claude Code | `plugins/research-skills/.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json` | `commands/*.md` plus `skills/research-paper-workflow/` |
| Gemini | `plugins/research-skills/gemini-extension.json` | `skills/research-paper-workflow/` |

The thin command wrappers under `plugins/research-skills/commands/` intentionally contain no workflow logic. They only load `research-paper-workflow` and point to `skills/research-paper-workflow/workflows/<command>.md`.

## Quality Contract

Every canonical skill should pass:

```bash
python3 scripts/audit_skill_sections.py --strict
```

The maintained contract is in `docs/maintainer/skill-quality-contract.md`; the runtime scholarly-output rubric is bundled at `research-paper-workflow/references/academic-output-rubric.md`.
