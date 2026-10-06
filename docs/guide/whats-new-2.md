# What changed in 2.x

Qiongli 2.x uses one Rust-native program for the CLI, research guidance and Lite/Full MCP. Keep your chosen Host and model settings.

## Compared with 1.x

| Area | Late 1.x | 2.x |
|---|---|---|
| Installation | Python full runtime and npm asset entry had different roles; native Lite packages already existed | Extract-and-run binary; npm, PyPI and Cargo share the CLI contract |
| Plugin | Native Lite and Python Full shipped separately | Both Lite and Full run natively |
| Daily use | More installation options and prescribed sequences | Guided installation; research guidance follows the current task |
| Records | Workflow files and literature citation discovery | Adds structured records with revision checks, Research Graph and stage handoffs |

## Added in 2.3.0

- Append reviewed retrieval history while preserving earlier attempts and source bindings.
- Discover and read saved notes, source packets and retrieval history through CLI and Full MCP, with revision, digest and safe-path checks.
- Full MCP now exposes 35 tools; Lite remains at 15. Literature tools declare their read-only and external-access behavior without granting approval.

See the [2.3.0 release notes](https://github.com/jxpeng98/qiongli/releases/tag/v2.3.0)
and [upgrade and rollback](upgrade.md) for the new receipt compatibility limits.

## Updated in 2.2.1

This patch reorganizes the English and Chinese guides and moves native release
qualification to Ubuntu 24.04 runners. Research commands and saved project
formats are unchanged. See the [2.2.1 release notes](https://github.com/jxpeng98/qiongli/releases/tag/v2.2.1)
for the package scope and [installation](install.md) for current downloads.

## Added in 2.2.0

- Read public HTTPS full text with provenance and page or section locations.
- Save reviewed stage summaries, paper notes and source packets; append notes while preserving existing text.
- Improved reading, writing, synthesis and evidence-review guidance.
- Native Linux ARM64 packages; Lite exposes 15 tools and Full exposes 33.

Version 2.2.0 did not save `retrieval_manifest.csv`; 2.3.0 adds reviewed persistence. Public full-text reading does not include paywall bypass or private-library access. Graph links canonical records; research conclusions still need source review.

See the [2.2.0 release notes](https://github.com/jxpeng98/qiongli/releases/tag/v2.2.0) for scope, [installation](install.md) and [upgrade and rollback](upgrade.md) for setup, or the [1.x reference](../reference/cli.md) for old commands.
