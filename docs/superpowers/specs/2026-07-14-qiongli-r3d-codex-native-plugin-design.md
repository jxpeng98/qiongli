# Qiongli 2.0 R3D Codex Native Plugin Design

Date: 2026-07-14  
Status: frozen for implementation  
Roadmap slice: `R3D / INT-201`  
Scope: native Codex plugin composition and isolated client activation evidence

## Outcome

R3D turns the receipt-verified R3C local source into a complete Codex plugin
package. The package contains the target-native Qiongli executable, the
Marketplace Lite skill projection, and a root MCP declaration that launches
that executable directly.

A user of this package does not need Python, Node, Rust, Cargo, npm, or pip.
Codex remains responsible for copying the package into its cache and recording
plugin enablement. Qiongli does not write Codex cache or enablement files.

R3D proves the local Codex vertical. It does not publish a public Marketplace
listing, support Codex cloud or ChatGPT web access to a local executable, or
claim that an unreleased source build is a production installer.

## Current Codex Contract

The design follows the current Codex plugin contract confirmed on 2026-07-14:

- the plugin root contains `.codex-plugin/plugin.json`;
- a plugin may declare `"skills": "./skills/"`;
- a root `.mcp.json` is selected with
  `"mcpServers": "./.mcp.json"`;
- plugin-owned MCP commands are resolved from the installed plugin package;
- the personal marketplace may reference a local plugin source;
- Codex owns `~/.codex/plugins/cache` and plugin enablement in
  `~/.codex/config.toml`.

R3C continues to own personal-marketplace source registration. R3D owns the
source package that R3C registers. A clean-client acceptance test invokes the
real Codex CLI against an isolated home and `CODEX_HOME`; it never changes the
developer's normal Codex state.

## Canonical Package Layout

Each artifact is platform-specific and has one root directory named
`qiongli`:

```text
qiongli/
  .codex-plugin/plugin.json
  .mcp.json
  .qiongli-codex-plugin-bundle.json
  bin/qiongli              # Unix
  bin/qiongli.exe          # Windows
  skills/
    qiongli-workflow/
      SKILL.md
      ...Marketplace Lite content...
```

There is one native executable, not a second MCP-only runtime. The MCP entry is
platform-specific only at the executable suffix:

```json
{
  "mcpServers": {
    "qiongli": {
      "command": "./bin/qiongli",
      "args": [
        "mcp",
        "serve",
        "--profile",
        "marketplace-lite",
        "--transport",
        "stdio"
      ],
      "cwd": ".",
      "startup_timeout_sec": 20,
      "tool_timeout_sec": 60
    }
  }
}
```

Windows uses `./bin/qiongli.exe`. No manifest command names a language runtime,
shell, package manager, checkout path, or user-global Qiongli command.

## Single-source Content Projection

The composer consumes an already verified `marketplace-lite` resource pack.
It does not copy a tracked plugin mirror.

The canonical `.codex-plugin/plugin.json` resource is a metadata template. The
composer preserves its descriptive metadata, then deterministically sets:

- `version` from the signed artifact identity;
- `skills` to `./skills/`; and
- `mcpServers` to `./.mcp.json`.

The template itself is not copied into the skill payload. The canonical
`workflow/SKILL.md` becomes `skills/qiongli-workflow/SKILL.md`; all other
`workflow/` resources lose that prefix, and the remaining Marketplace Lite
resources keep their canonical relative paths below
`skills/qiongli-workflow/`.

The composer rejects output-path collisions and non-portable paths. Logical
resource modes are preserved. The native executable is the only generated
executable outside the content projection.

## Trust Inputs

Composition requires all of the following:

1. a verified resource pack;
2. a verified signed launch grant authorizing `LiteMcp` and `CodexLocal`;
3. a Lite `PluginBundle` artifact for the current target OS and architecture;
4. a regular target-native executable whose SHA-256 equals the grant; and
5. an explicitly approved absolute output target named `qiongli`.

The resource-pack digest, target OS and architecture, artifact identity,
binary digest, and authorized integration scope must agree. Any disagreement
fails closed before writing the package.

The source build has no production launch grant. R3D exposes a library
composition boundary and test/evidence path; it does not add an install command
that could bypass release signing.

## Bundle Receipt

`.qiongli-codex-plugin-bundle.json` is canonical RFC 8785 JSON and binds:

- receipt schema and package kind;
- signed artifact identity;
- signed launch-grant payload digest;
- resource-pack ID, version, source commit, pack digest, and content root;
- target OS and architecture;
- binary path and digest;
- plugin manifest and MCP declaration digests;
- every package file's relative path, logical mode, size, and SHA-256; and
- a domain-separated package content-root digest.

The receipt excludes only itself. Verification rejects missing or extra files
and directories, links or Windows reparse points, hard-linked managed files,
non-canonical receipts, path or mode drift, oversized input, and any digest
mismatch.

R3C source discovery changes to require this bundle receipt. Its registration
receipt binds the verified plugin-bundle receipt and content-root digests, not
the earlier raw materialization receipt.

## Filesystem Transaction

The approved target capability is created only at a trusted CLI, UI, installer,
release, or test boundary. Model or MCP input must never mint it.

Composition:

1. validates the absolute portable target and secure existing parent chain;
2. validates the source executable without following links;
3. acquires a target-scoped private lock;
4. refuses an existing target in R3D instead of replacing unmanaged data;
5. writes a private sibling staging directory with create-new files;
6. verifies the complete staged tree against the receipt;
7. promotes the directory with no-replace semantics; and
8. verifies the committed tree before returning.

Unix files use `0644`, the bundled executable and canonical executable
resources use `0755`, and directories use `0755` only after staging is
complete. Windows creation uses the existing current-user-only security
boundary and rejects reparse points and hard links.

R3D deliberately does not add in-place package upgrade or repair. Later
release/install transactions may stage a new verified package and use the
existing managed-resource lifecycle for replacement and rollback.

## Client-owned Activation Evidence

The acceptance harness performs this sequence in a fresh isolated user root:

1. compose and verify a plugin package using an ephemeral test signing key;
2. validate the package with the current Plugin Creator validator;
3. write only the isolated personal marketplace entry;
4. run `codex plugin add qiongli@personal` with the isolated environment;
5. confirm Codex lists the installed plugin and created its cache copy;
6. confirm Codex enablement state exists in the isolated config; and
7. launch the cached `.mcp.json` command with an empty `PATH`, then complete
   MCP `initialize` and `tools/list`.

The evidence records the Codex CLI version, package receipt digest, cache path
shape, MCP server identity, and exact Lite tool list. It must not record home
paths, secrets, signing private keys, or normal user configuration.

Because the Codex CLI is external to the Rust workspace, this real-client test
is an explicit acceptance gate rather than a required test on runners where
Codex is unavailable. Deterministic package and direct MCP tests remain normal
Rust tests on Linux, macOS, and Windows.

## Failure And Non-claim Rules

- Package composition success is not Codex installation success.
- R3C registration success is not Codex cache or enablement success.
- Codex CLI installation success is not public Marketplace publication.
- Local activation does not make a local executable available to cloud or web
  runtimes.
- A test-signed bundle is acceptance evidence only and must never be released.
- No package contains credentials, API keys, environment snapshots, user
  config, or an embedded private signing key.

## Exit Gate

R3D is complete when:

- the deterministic composer and strict verifier pass on Tier 1 targets;
- R3C accepts only a verified native plugin bundle;
- Plugin Creator validates the generated package;
- the bundled binary serves the canonical Lite MCP with an empty `PATH`;
- the real Codex CLI installs, enables, caches, and launches the package in an
  isolated clean-client environment;
- the native workspace gate and exact-head CI are green; and
- the rolling Draft PR states the local-only limits and does not claim public
  Marketplace, cloud, release, UI, Claude, update, or rollback completion.

