# Plugin-First Architecture

Qiongli is distributed as one plugin package, not as dozens of separate academic-skill plugins.

## Definitions

| Layer | Path | Role |
|-------|------|------|
| Main plugin | `plugins/qiongli/` | Install, discovery, version, marketplace, and platform entrypoints. |
| Portable skill package | `qiongli-workflow/` | The cross-client runtime skill loaded by Codex, Claude Code, and Gemini. |
| Source skill specs | `skills/` | Canonical academic capability specs maintained by this repository. |
| Workflow commands | `qiongli-workflow/workflows/` and `plugins/qiongli/commands/` | User entrypoints such as `/paper`, `/lit-review`, and `/code-build`. |

## Plugin vs Skill

A **plugin** is the distribution container. It owns manifests, marketplace metadata, command wrappers, bundled skills, and platform-specific install surfaces.

A **skill** is the reusable execution contract. It tells the model how to perform a specific research task, what inputs are required, what artifact to write, and what quality bar must be met.

This repo keeps one plugin, `qiongli`, and ships one portable skill package, `qiongli-workflow`, inside it. The 71 academic skill specs remain internal capability cards under `skills/`; they are synchronized into the portable package during release.

## Source Of Truth

- Edit source skills in `skills/`.
- Edit portable package shell files such as `qiongli-workflow/SKILL.md`, `workflows/`, and `references/` directly.
- Run `bash scripts/sync_skill_package.sh --target all` before testing artifacts or publishing.
- Do not edit `plugins/qiongli/skills/qiongli-workflow/` by hand; it is a distribution copy.

## Platform Surfaces

| Platform | Manifest | Runtime entry |
|----------|----------|---------------|
| Codex | `plugins/qiongli/.codex-plugin/plugin.json` and `.agents/plugins/marketplace.json` | `skills/qiongli-workflow/` |
| Claude Code | `plugins/qiongli/.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json` | `commands/*.md` plus `skills/qiongli-workflow/` |
| Gemini | `plugins/qiongli/gemini-extension.json` | `skills/qiongli-workflow/` |

The thin command wrappers under `plugins/qiongli/commands/` intentionally contain no workflow logic. They only load `qiongli-workflow` and point to `skills/qiongli-workflow/workflows/<command>.md`.

## Upgrade Compatibility

The plugin-first package does not replace the legacy global install in place. Native plugin installation and bootstrap / `rsk` installation use separate surfaces:

- Plugin bundle: managed by the client plugin or extension system.
- Global skill install: `~/.codex/skills/qiongli-workflow`, `~/.claude/skills/qiongli-workflow`, and `~/.gemini/skills/qiongli-workflow`, managed by `rsk` or bootstrap.
- Global slash discovery: `~/.claude/commands/*.md` and `~/.gemini/workflows/*.md`, managed by `rsk`.

For ordinary client-native usage, the plugin bundle is enough. For CLI commands, validators, `doctor`, release tooling, or `bridges.orchestrator`, users still need the `full` runtime and should keep the global install aligned with:

```bash
qiongli upgrade --target all --doctor
```

When migrating fully to the plugin and removing old slash discovery, preview cleanup first:

```bash
qiongli clean --globals --dry-run
```

## Quality Contract

Every canonical skill should pass:

```bash
python3 scripts/audit_skill_sections.py --strict
```

The maintained contract is in `docs/maintainer/skill-quality-contract.md`; the runtime scholarly-output rubric is bundled at `qiongli-workflow/references/academic-output-rubric.md`.
