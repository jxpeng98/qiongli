# DeepSeek Harness and external Agent coordination

Qiongli can be loaded into DeepSeek Harness as a Cordis bundle containing the
canonical `qiongli` and `no-qiongli` Skills and native Full MCP. The coordinating
Host can also prepare and collect bounded proposals from Codex, Claude Code,
DeepSeek Harness and Antigravity CLI. The Host executes and supervises the child
process; Qiongli validates the result against the original task and sources.

## Build and load the DeepSeek bundle

This is an opt-in development artifact, not a published package or an addition to
the default release matrix. Use the existing native content exporter and archive
verifier. Build from a clean source commit; use that same commit for the binary
and content export. From the repository root, with the current native binary built:

```sh
QIONGLI_NATIVE_SOURCE_COMMIT="$(git rev-parse HEAD)" cargo run --locked \
  --manifest-path packages/qiongli-native/Cargo.toml -p qiongli \
  --example export_marketplace_content -- /absolute/new-content-directory
python3 tooling/scripts/native_marketplace_plugins.py \
  --content-dir /absolute/new-content-directory --out-dir /absolute/new-bundle-directory \
  --version 2.0.0 --commit "$(git rev-parse HEAD)" \
  --binary packages/qiongli-native/target/debug/qiongli \
  --target aarch64-apple-darwin --platform deepseek
```

Use the actual executable version and target; Linux x64 and Windows x64 layouts
are also supported by the generator. Each output directory must be new. Existing
source/byte/target/receipt checks remain enforced. Default packaging continues to
build only Codex and Claude; no release or upload is performed.

Extract the generated archive. Its `plugins/qiongli-macos-arm64` directory
contains `package.json` and the bundle patch (the name differs by target/channel).
Install that directory with DeepSeek's own plugin manager into the chosen
profile, which requires its documented pnpm dependency:

```sh
dsh plugin --profile web add /absolute/extracted/plugins/qiongli-macos-arm64
dsh --profile web --dump-config
dsh web
```

The bundle adds Skills and one `mcp-qiongli` row; it does not select a model,
change authentication or replace the profile's other configuration. Use only one
Qiongli bundle per profile. Inspect the actual Skill catalog and MCP tool list,
then call `qiongli_config_status` to verify the live connection. For the current
shared MCP schemas, DeepSeek uses route platform `unknown` and Host family
`other-local`, with actual versions and observed component states. A loaded
Plugin is not project-write approval. Removal uses the same Host manager:
`dsh plugin --profile web remove dsh-qiongli-macos-arm64`.

## Dispatch and collect an external proposal

The coordinator obtains its handoff and authenticated source reads from an
existing Full MCP run. Save that handoff and a JSON packet containing only
`scope` and `sourceText`; the source snapshot must be authorized for the selected
external Host. Run:

```sh
qiongli agent claude prepare --handoff handoff.json --packet packet.json --json
```

Replace `claude` with `codex`, `deepseek` or `antigravity` as requested. Execute the
returned argv using the current Host's execution tool, feed `stdin` unchanged,
and merge any returned `env` overrides into only that child process. Use an
authorized isolated working directory, a bounded deadline and separate stdout
and stderr files. Wait for the actual process exit. Then collect:

```sh
qiongli agent claude collect --handoff handoff.json --packet packet.json \
  --events stdout.json --status completed --exit-code 0 --json
```

Supply the observed status and exit code, never the model's interpretation.
Failed, cancelled, timed-out, truncated, mismatched and unsuccessful streams
cannot become completed delegation results. Do not copy a result between Host
adapters. The coordinator reconciles findings, rechecks current source bindings
and submits through its own MCP process. Existing preview/approval/CAS owns all
research writes. No adapter changes the user's saved model, reasoning effort or
account; no direct provider API, Agent daemon, persistent resume or silent fallback
is introduced.

| Adapter | Required protocol | Per-run constraint |
| --- | --- | --- |
| Codex | `codex exec --json` with thread/turn events | Ephemeral read-only sandbox |
| Claude Code | `claude --print --output-format json` success envelope and session ID | Built-in tools and MCP disabled; no saved session |
| DeepSeek | `dsh --profile headless --json` session, turn-end and final events | `DSH_PERMISSION_MODE=read-only` |
| Antigravity | `agy --input-format stream-json --output-format stream-json` | One input message; plan mode; slash expansion disabled |

These are proposal restrictions, not complete isolation from configured hooks or
all Host tools. Tool inventory and policy remain Host-owned. A permission prompt,
missing login or unavailable protocol is a visible gap; never auto-approve it or
silently substitute a different model/Host. Reap interrupted processes before
starting an explicitly authorized fresh run.

## Protocol references and limits

The DeepSeek protocol was checked against official source
[`477b4f420553e8a52c2fbccc464d7561b239c443`](https://github.com/deepseek-ai/deepseek-harness/tree/477b4f420553e8a52c2fbccc464d7561b239c443),
including the [headless runner](https://github.com/deepseek-ai/deepseek-harness/blob/477b4f420553e8a52c2fbccc464d7561b239c443/packages/bundle/headless/README.md),
[bundle contract](https://deepseek-harness.github.io/deepseek-harness/en/develop/basic/publish)
and [Skill provider](https://deepseek-harness.github.io/deepseek-harness/en/reference/subsystems/skills).
The observed npm `@deepseek-ai/dsh@0.1.5-rc.3` supports bundles but its headless
help does **not** expose `--json`. It cannot satisfy this external collector;
use a configured build supporting the documented protocol and check its help.
Plain-text output is deliberately insufficient to identify a completed run.

The other protocol references are the official
[Claude Code CLI](https://code.claude.com/docs/en/cli-reference) and
[Antigravity headless interface](https://antigravity.google/docs/cli/headless/).
Protocol checks and synthetic tests do not establish installed-Host, real
research, two-Host collaboration or cross-platform release acceptance. Exact
observations and remaining checks belong to the current execution plan.
