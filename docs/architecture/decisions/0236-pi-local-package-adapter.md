# ADR 0236: Pi Local Package Adapter

- Status: Accepted
- Date: 2026-10-07
- Task: CLI-409
- Authority: the maintainer requests Pi support after the all-Host installer.
- Extends ADR 0235's detected Host set; its sequential approvals and failure
  boundaries remain unchanged. Other Host adapters and signed targets are unchanged.

Support the official Pi coding agent through terminal `--target pi`, Host choice
7 and detected `install/update/upgrade all`. Preserve 3/both and 6/all. Require
Pi 0.99.0 or newer: that published release introduced built-in MCP and
`registerMcpServer`. A CLI's presence alone does not establish compatibility.

Project a local Pi package through the shared native bundle transaction. The
`user-local-pi-full-mcp` kind and `.qiongli-pi-plugin-bundle.json` receipt separate
this schema-4 layout from Codex and AGY; historical receipts remain verifiable.
Its explicit `package.json` declares two Skills, `qiongli` and `no-qiongli`, and
one dependency-free JavaScript extension. Internal workflows, variants and
language metadata reuse the shared content owner. The extension registers the
bundled executable's absolute path as Full MCP with deferred discovery. Pi owns
transport and model execution; no third-party bridge, model selection, auth
migration, Qiongli daemon or standalone MCP configuration is added. The root
Skill uses the existing `other-local` descriptor and preview/approval/CAS rules.

Default to `~/qiongli-pi`; respect `PI_CODING_AGENT_DIR`, and reuse an existing
verified local source declared in user settings, resolving Pi's relative paths
against its agent directory. Bind executable, source, content, variants, language
and profile hashes to each terminal preview. Recheck under the existing write
guard before applying. Only the official `pi install <source>` writes Host
settings after separate trust confirmation. On Unix a fixed `/bin/sh` script
sets umask 077 and execs the resolved Pi executable with separate argv elements;
new profile files remain private without changing the parent's mask or existing
permissions. The same bounded launcher retains process/output limits. Existing
unsafe profiles refuse rather than being chmodded. Already registered unfiltered paths
need only an approved source update. Verify the resulting declaration and
preservation of unrelated settings/packages, plus unchanged standalone MCP
configuration. Refuse unknown/changed source files, competing Qiongli packages,
filtered target resources, disabled built-in MCP and conflicting standalone MCP
entries. Do not print settings contents or reset existing choices.

Pi loads local packages in place. Keep the source until `pi remove <source>`
removes its declaration. Source updates/removal retain receipt-bound transaction
and data-loss checks. Registration is not live readiness: project overrides,
resource filters and a replacement MCP extension can alter session behavior.
Use session `/mcp` and an actual tool call for connection evidence; shell-level
`pi mcp list` does not load extensions. Signed products, App plan schemas, context
hooks and child-agent execution protocols gain no Pi target in this increment.

Verification covers package projection, relative-source reuse, filters/conflicts,
unrelated/model preservation, receipt CAS/drift and existing Host regressions.
An isolated official Pi loader/session may verify Skills, MCP discovery and a
read-only native call without model credentials. This does not qualify an LLM
research journey, every operating system, or a release. Rollback removes the
Pi installer entry and adapter; installed source directories remain user data.

Sources: official [Pi packages](https://pi.dev/docs/latest/packages),
[MCP](https://pi.dev/docs/latest/mcp),
[environment](https://pi.dev/docs/latest/environment-variables), and
[release history](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/CHANGELOG.md).
