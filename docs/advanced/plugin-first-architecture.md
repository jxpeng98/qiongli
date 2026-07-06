# Plugin-First Architecture

Qiongli stable is distributed as one plugin package, not as dozens of separate academic-skill plugins.

Plugin-first describes the marketplace distribution architecture. It is not the full-product architecture. For full local Qiongli, the CLI full profile can generate a client-native local plugin bundle. Marketplace plugins remain client-native lite installs, with bundled Node literature MCP fallback where supported.

## Definitions

| Layer | Path | Role |
|-------|------|------|
| Plugin metadata | `content/distribution/plugins.yaml` | Source names, descriptions, prompts, keywords, and platform enablement for stable and prerelease plugins. |
| Plugin generator | `tooling/scripts/build_plugin_artifacts.py` | Generates manifests, command wrappers, bundled MCP manifests, and platform payloads. |
| Portable skill package | `qiongli-workflow/` | The cross-client runtime skill loaded by Codex, Claude Code, Antigravity, and Hermes. |
| Source skill specs | `content/skills/` | Canonical academic capability specs maintained by this repository. |
| Workflow commands | `content/workflow/workflows/` | User entrypoints such as `/paper`, `/lit-review`, and `/code-build`; plugin command files are generated from these workflows. |

## Plugin vs Skill

A **plugin** is the distribution container. It owns manifests, command wrappers, bundled skills, and platform-specific install surfaces.

A **skill** is the reusable execution contract. It tells the model how to perform a specific research task, what inputs are required, what artifact to write, and what quality bar must be met.

This repo keeps the stable plugin as `qiongli` and ships one portable skill package directory, `qiongli-workflow`, inside release artifacts. The portable package declares the user-visible skill name `qiongli`; the directory name stays `qiongli-workflow` so existing install paths, release artifacts, and compatibility checks remain stable. The academic skill specs remain internal capability cards under `content/skills/`; they are materialized into the portable package during release.

The `qiongli-next` plugin is generated from the same canonical sources with prerelease metadata from `content/distribution/plugins.yaml`. It contains only the `core/complete` package plus the bundled literature MCP runtime, and it does not publish subject-specific plugin variants.

## Source Of Truth

- Edit source skills in `content/skills/`.
- Edit portable package shell files such as `content/workflow/SKILL.md`, `workflows/`, and `references/` directly.
- Edit plugin names, prompts, keywords, and platform enablement in `content/distribution/plugins.yaml`.
- Edit generated manifest or wrapper structure in `tooling/scripts/build_plugin_artifacts.py`.
- Run `python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force` before testing staged artifacts.
- Release automation materializes payloads in a release staging workspace before publishing and does not write generated payloads back to the source checkout.
- Do not edit generated payloads such as `plugins/qiongli/`, `plugins/qiongli-next/`, `packages/qiongli-plugin/`, or `packages/qiongli-next-plugin/`; they are rebuilt from canonical source and are not tracked in the clean development checkout.

## Platform Surfaces

| Platform | Manifest | Runtime entry |
|----------|----------|---------------|
| Codex | generated `.codex-plugin/plugin.json` plus bundled `.mcp.json` in `plugins/qiongli/` or `plugins/qiongli-next/` | `skills/qiongli-workflow/`, visible as `qiongli` or `qiongli-next` in `/skills`, invoked as `$qiongli` or `$qiongli-next`; bundled Node literature-provider MCP runtime under `mcp/qiongli-literature-provider/` |
| Claude Code | generated `.claude-plugin/plugin.json` in the plugin payload | `commands/*.md` plus `skills/qiongli-workflow/`; bundled Node literature-provider MCP runtime under `mcp/qiongli-literature-provider/` |
| Claude Desktop direct plugin | root `plugin.json` plus generated `.claude-plugin/plugin.json` in `qiongli-claude-desktop-plugin-<tag>.zip` | installed `qiongli` entry, generated workflow wrappers including `/qiongli`, `skills/qiongli-workflow/`, and bundled lightweight literature MCP runtime |
| Claude Desktop/Web fallback skill ZIP | uploaded skill package only | installed `qiongli` skill; no bundled provider tools unless the user also installs the Qiongli Literature Provider MCPB |

The shared Skillsplace repository is the public marketplace source of truth. This repository owns the generator inputs and release artifacts used by marketplace entries, not duplicate public marketplace catalog state.

Generated command wrappers intentionally contain no workflow logic. They only load the generated skill package and point to `skills/qiongli-workflow/workflows/<command>.md`. The `/qiongli` wrapper points to `workflows/qiongli.md`, which is the unified entry router and delegates to the narrower canonical workflows.

## Upgrade Compatibility

The plugin-first package does not replace the legacy global install in place. Native plugin installation and bootstrap / `rsk` installation use separate surfaces:

- Plugin bundle: managed by the client plugin or extension system.
- Global skill install: `~/.codex/skills/qiongli-workflow`, `~/.claude/skills/qiongli-workflow`, `~/.gemini/antigravity/skills/qiongli-workflow`, and `~/.hermes/skills/qiongli-workflow`, managed by `rsk` or bootstrap.
- Global slash discovery: `~/.claude/commands/*.md`, managed by `rsk`.

For ordinary skill-only client-native usage and Codex/Claude Code bundled literature-provider MCP usage, the marketplace plugin bundle is enough. Those marketplace artifacts keep the bundled Node literature MCP. From v1.9.0 onward, the default full local Qiongli path installs a client-native local plugin bundle backed by the full Python MCP:

```bash
qiongli install --target all
qiongli mcp doctor --json
```

`qiongli upgrade` also defaults to this plugin-first path. After the new plugin install succeeds, it cleans legacy global skill directories, Claude Code slash discovery links, and Codex/Claude standalone MCP config. If you are manually cleaning older installs, preview cleanup first:

```bash
qiongli clean --globals --dry-run
```

`qiongli check` discovers the current surface before reporting versions. Codex, Claude Code, and Antigravity local full installs are reported as `surface=plugin`; Hermes managed MCP config is reported as `surface=mcp`; old global skill directories remain visible as `surface=legacy_skill`. `qiongli doctor --cwd .` keeps its runtime/orchestrator role and appends a non-fatal client integration summary from the same discovery layer.

For Codex personal marketplace troubleshooting, remember that the plugin list and plugin detail read are separate steps. A `qiongli` entry can appear as available if `~/.agents/plugins/marketplace.json` is valid, while the details page can still fail if the referenced local plugin has an invalid `.codex-plugin/plugin.json`, invalid `skills/qiongli-workflow/SKILL.md` YAML frontmatter, or a missing plugin directory.

## Quality Contract

Every canonical skill should pass:

```bash
python3 scripts/audit_skill_sections.py --strict
```

The maintained contract is in `docs/maintainer/skill-quality-contract.md`; the runtime scholarly-output rubric is bundled at `qiongli-workflow/references/academic-output-rubric.md`.
