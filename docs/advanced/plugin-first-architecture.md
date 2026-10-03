# How the Plugin connects Qiongli

For Codex or Claude Code, the Plugin is the simplest way to connect research
Skills and MCP. Start with `qiongli install plugin`; the installer guides you
through file export and official Host registration.

## What gets installed

| Surface | Guidance | Program and tools |
|---|---|---|
| CLI-exported Plugin | Shared Skills, workflows and references | Bundled native executable; Full MCP, 33 tools |
| Native Marketplace platform Plugin | Shared Skills, workflows and references | Bundled native executable; Lite MCP, 15 tools |
| Standalone Skills | Exported guidance and references | No MCP connection or automatic Host registration |

Both MCP profiles run without Python or Node. A package manager may have its own
requirements: npm needs Node, PyPI needs Python, and Cargo builds with Rust.
The [standalone binary](../guide/cli-2x.md#standalone-binary-download) needs none
of those language runtimes on its supported platform.

The Host loads Skills from its registered Plugin cache and starts MCP when needed.
You do not need to copy the Plugin into `~/.agents/skills` or run a background
MCP terminal. A Skills-only export must be installed through your Host separately.

## Refresh and verify

After upgrading the CLI through its original channel, run
`qiongli install plugin` again. It reuses the registered source directory.
The file preview and Host registration each require confirmation. Known Codex
Plugin conflicts can follow the confirmed migration flow; other conflicts need
manual review. See the
[installation guide](../guide/cli-2x.md#first-use) for the exact boundary.

Run `qiongli doctor` and `qiongli install list`, then start a new Host session and
check its actual Qiongli tools. Registration alone does not verify session tools,
provider access or Hook execution. Optional hooks need separate Host trust.

## Where maintainers edit

Shared guidance lives in `content/`; native implementation lives in
`packages/qiongli-native/`. Plugin exports and registry packages are generated
from those sources. A release staging workspace holds generated files; packaging
does not write generated payloads back to the source checkout.

See [repository structure](../development/repository-structure.md) and
[release policy](../maintainer/release-branch-policy.md) before changing packaging.
