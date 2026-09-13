# Architecture

Qiongli 2 delivers research guidance and tools through a Rust CLI, Plugin/Skills
and Lite/Full MCP. Your Host owns the model, authentication and conversation.
Qiongli manages research records, source links, project operations and installation
receipts. The default CLI build does not require the retained Desktop application.

## One set of sources, several entry points

`content/` holds shared Skills, workflows, templates and public MCP contracts.
`qiongli-content` builds the embedded resource pack. Native services under
`packages/qiongli-native/` use that pack and own project state, revisions, previews,
approvals, Graph and MCP dispatch. CLI and Host adapters call those same services.

GitHub archives, npm and PyPI carry the same executable for a given version and
target; Cargo builds it from source. Native package builders share the product
description, while each channel supplies its own launcher and installation guide.
The old Python/npm product sources remain compatibility references.

A CLI-exported Plugin includes Full MCP and the shared Skills. The native
Marketplace platform Plugin includes Lite MCP. The Host loads the Plugin cache
and starts its stdio process; no separate MCP package or terminal is normally needed.
See [Plugin contents](advanced/plugin-first-architecture.md).

## Research changes and evidence

CLI, Full MCP and the retained App use the same project services and revision
rules. The Graph projects saved research records, retaining claim IDs, citekeys
and source locations. It does not turn a summary or reviewer opinion into new
primary evidence. See [the Graph example](examples/research-graph.md).

Full MCP adds project operations to Lite's bounded literature and Zotero tools.
Its `qiongli_project_capture_apply` write rechecks the preview and requires the
matching plan digest and `approve_filesystem_write=true`. In-process ToolHost
remains read-only and rejects that write. One approved operation is not permission
for unrestricted edits. Zotero direct access uses the local Companion; import-file
contents remain available when the Companion cannot be reached.

Subagents run through actual Host tools. Cross-Host packets carry bounded sources
and candidates to one coordinator; they do not transfer project write authority.
The [collaboration guide](advanced/agent-skill-collaboration.md) describes the
available exchange and the automation still outside its scope.

## Decisions and maintenance

Accepted ADRs under `docs/architecture/decisions/` own the architecture. ADR 0218
sets the CLI-first, Host-owned execution direction; ADRs 0219–0223 define native
distribution, ADR 0224 defines approved Host registration, and ADR 0227 makes
integrated `main` the stable source while `2.x` remains the prerelease source.
A merge does not establish release or live-Host acceptance.

Tauri/Svelte, App API and earlier ACP/All Chat work remain in maintenance or deferred
scope under their accepted decisions. They are not prerequisites for CLI use.
When entry points disagree, fix their shared owner and rebuild affected outputs.
See [repository structure](development/repository-structure.md) and
[editing conventions](conventions.md); do not rewrite accepted decision history.
