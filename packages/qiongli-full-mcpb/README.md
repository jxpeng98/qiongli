# Qiongli Full Research Runtime MCPB

This package is the local Claude Desktop adapter for the Qiongli Full MCP
runtime. It bundles the current-host Rust executable and launches only:

```text
qiongli mcp serve --profile full --transport stdio
```

Claude Desktop owns the model login, conversation, model execution, extension
installation, trust, enablement, and tool approvals. Qiongli owns registered
research projects, revision-bound project tools, Academic Graph inspection,
host-driven orchestration handoffs, evidence validation, checkpoints, and
explicitly approved project mutations.

The bundle does not contain a model provider credential, provider endpoint,
model selection, shell command, project path, or automatic direct-model
fallback. Qiongli never launches Claude from this extension.

## Local Installation

Build the non-publishing package for the current machine:

```bash
pnpm mcpb:pack:full
```

In Claude Desktop, open **Settings → Extensions → Advanced settings → Install
Extension…** and select the generated `qiongli-full-runtime-*.mcpb`.

The local builder emits a target identity and a receipt next to the package.
The package is valid only for the recorded operating system and architecture.
Its local receipt has `publication_allowed: false`; release publication needs
a separately approved multi-target build and distribution contract.

## Runtime Boundary

The Full MCP exposes the Lite provider surface, registered-project reads,
capture and Academic Graph tools, and host-driven orchestration controls. The
calling host advances model-backed work through start/read/submit/next
handoffs. Candidate content remains untrusted until Qiongli validates the
project revision, checkpoint generation, document digest, evidence references,
role gate, and schema.

The existing `qiongli-literature-provider-*.mcpb` remains a smaller Marketplace
Lite adapter. Installing it does not install or activate this Full runtime.

Local installation does not activate Claude Web, Codex Cloud, or another
remote worker. Those surfaces require a separately supported remote MCP,
repository bundle, or host-specific deployment contract.
