# Plugin-First Architecture

Qiongli stable is distributed as one plugin package, not as dozens of separate academic-skill plugins.

## Definitions

| Layer | Path | Role |
|-------|------|------|
| Main plugin | `packages/qiongli-plugin/` | Source manifests, discovery commands, version metadata, and platform entrypoints. |
| Next Codex plugin | `packages/qiongli-next-plugin/` | Git-backed prerelease Codex plugin source for Skillsplace beta installs. Its `.codex-plugin/plugin.json` name is `qiongli-next`. |
| Portable skill package | `qiongli-workflow/` | The cross-client runtime skill loaded by Codex, Claude Code, and Gemini. |
| Source skill specs | `content/skills/` | Canonical academic capability specs maintained by this repository. |
| Workflow commands | `qiongli-workflow/workflows/` and `packages/qiongli-plugin/commands/` | User entrypoints such as `/paper`, `/lit-review`, and `/code-build`. |

## Plugin vs Skill

A **plugin** is the distribution container. It owns manifests, command wrappers, bundled skills, and platform-specific install surfaces.

A **skill** is the reusable execution contract. It tells the model how to perform a specific research task, what inputs are required, what artifact to write, and what quality bar must be met.

This repo keeps the stable plugin as `qiongli` and ships one portable skill package directory, `qiongli-workflow`, inside release artifacts. The portable package declares the user-visible skill name `qiongli`; the directory name stays `qiongli-workflow` so existing install paths, release artifacts, and compatibility checks remain stable. The 71 academic skill specs remain internal capability cards under `skills/`; they are materialized into the portable package during release.

The `qiongli-next` Codex plugin is a tracked prerelease exception for Git-backed Skillsplace installs. It contains only the `core/complete` package plus the bundled literature MCP runtime, and it does not publish subject-specific plugin variants.

## Source Of Truth

- Edit source skills in `content/skills/`.
- Edit portable package shell files such as `content/workflow/SKILL.md`, `workflows/`, and `references/` directly.
- Run `python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force` before testing staged artifacts.
- Run `python3 scripts/materialize_distribution_payloads.py --target next-plugin --in-place` only when maintaining the tracked `packages/qiongli-next-plugin/` beta source directory.
- Release automation materializes payloads in a release staging workspace before publishing and does not write generated payloads back to the source checkout.
- Do not edit generated payloads such as `plugins/qiongli/`; they are rebuilt from canonical source and are not tracked in the clean development checkout.

## Platform Surfaces

| Platform | Manifest | Runtime entry |
|----------|----------|---------------|
| Codex | `packages/qiongli-plugin/.codex-plugin/plugin.json` plus bundled `packages/qiongli-plugin/.mcp.json`; public catalog entry in `jxpeng98/skillsplace` | `skills/qiongli-workflow/`, visible as `qiongli` in `/skills`, invoked as `$qiongli`; bundled Node literature-provider MCP runtime under `mcp/qiongli-literature-provider/` |
| Codex beta | `packages/qiongli-next-plugin/.codex-plugin/plugin.json` plus bundled `packages/qiongli-next-plugin/.mcp.json`; beta catalog entry in `jxpeng98/skillsplace` may point at this Git subdirectory on a beta tag | `skills/qiongli-workflow/`, visible as `qiongli-next` in `/skills`, invoked as `$qiongli-next`; bundled Node literature-provider MCP runtime exposed as `mcpServers.qiongli-next` |
| Claude Code | `packages/qiongli-plugin/.claude-plugin/plugin.json`; public catalog entry in `jxpeng98/skillsplace` | `commands/*.md` plus `skills/qiongli-workflow/`; bundled Node literature-provider MCP runtime under `mcp/qiongli-literature-provider/` |
| Gemini | `packages/qiongli-plugin/gemini-extension.json` | `skills/qiongli-workflow/` |

The shared Skillsplace repository is the public marketplace source of truth. Stable entries point to this repository's `packages/qiongli-plugin` subdirectory through git-subdir entries. The beta Codex entry can point to `packages/qiongli-next-plugin` on a beta tag. This repository owns the plugin manifests and source used to generate release payloads, not duplicate public marketplace catalog state.

The thin command wrappers under `packages/qiongli-plugin/commands/` intentionally contain no workflow logic. They only load `qiongli-workflow` and point to `skills/qiongli-workflow/workflows/<command>.md`.

## Upgrade Compatibility

The plugin-first package does not replace the legacy global install in place. Native plugin installation and bootstrap / `rsk` installation use separate surfaces:

- Plugin bundle: managed by the client plugin or extension system.
- Global skill install: `~/.codex/skills/qiongli-workflow`, `~/.claude/skills/qiongli-workflow`, and `~/.gemini/skills/qiongli-workflow`, managed by `rsk` or bootstrap.
- Global slash discovery: `~/.claude/commands/*.md` and `~/.gemini/workflows/*.md`, managed by `rsk`.

For ordinary skill-only client-native usage and Codex/Claude Code bundled literature-provider MCP usage, the plugin bundle is enough. For the full Python-backed MCP server, CLI commands, validators, `doctor`, release tooling, or `bridges.orchestrator`, users still need the local `qiongli` CLI/runtime and should keep the global install aligned with:

```bash
qiongli upgrade --target all --doctor
```

When migrating fully to the plugin and removing old global skill directories or slash discovery, preview cleanup first:

```bash
qiongli clean --globals --dry-run
```

## Quality Contract

Every canonical skill should pass:

```bash
python3 scripts/audit_skill_sections.py --strict
```

The maintained contract is in `docs/maintainer/skill-quality-contract.md`; the runtime scholarly-output rubric is bundled at `qiongli-workflow/references/academic-output-rubric.md`.
