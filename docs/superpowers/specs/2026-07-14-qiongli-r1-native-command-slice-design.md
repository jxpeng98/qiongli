# Qiongli R1 Native Command Slice Design

Status: approved for execution

Date: July 14, 2026

Scope: final R1 command composition over the accepted embedded-content and
global-config services

Branch: `feat/2x-native-alpha1`

PR: rolling Draft PR #63

## Decision

The canonical `qiongli` Rust executable will expose the first useful native
command surface without creating another service implementation. The app crate
parses trusted local CLI intent, constructs already accepted content/config
capabilities, invokes those services, and renders stable redacted results.

This batch closes the R1 command deliverable. It does not begin provider,
Lite MCP, desktop, installer, credential, project-state, or migration work.

## Command Contract

The accepted grammar is:

```text
qiongli --help
qiongli --version
qiongli content --help
qiongli content list
qiongli content materialize --profile <profile> --target <absolute-path>
qiongli config --help
qiongli config show
qiongli config set --expected-revision <revision> --default-profile <profile>
qiongli status
qiongli doctor
```

`profile` accepts the canonical `skill-only`, `marketplace-lite`, and `full`
identifiers plus the already frozen `lite` alias. Materialization and config
set options may appear in either order, but duplicates, omissions, unknown
options, trailing arguments, invalid UTF-8 control tokens, and malformed
revisions fail as usage errors.

The CLI is the explicitly trusted boundary allowed to create an approved
caller-selected `MaterializationTarget`. No MCP/model-sourced path reaches
that capability in this batch.

`config set` deliberately changes only `default_profile`. It loads the full
current typed settings, preserves every provider field, requires an explicit
expected revision, and delegates the mutation to `GlobalSettingsStore`.
Credentials and raw secret values are not accepted by this command.

## Output And Exit Contract

Help and version retain their text output. Every successful data command emits
one newline-terminated JSON object with `schema_version: 1` and a stable
`command` discriminator.

- `content list` reports verified pack identity and canonical profile
  projections;
- `content materialize` reports profile, authorization, entry count, and
  content identities, but not the selected filesystem path;
- `config show` wraps only `RedactedConfigStatus`;
- `config set` reports the committed revision, selected profile, and whether
  post-commit cleanup is required;
- `status` combines verified embedded-content identity with redacted config
  status; and
- `doctor` returns explicit non-secret checks for embedded content, global
  config, and the currently unavailable secure store.

Exit codes are:

| Code | Meaning |
|---|---|
| `0` | command completed; doctor found no blocking config condition |
| `1` | an operation failed, or doctor found a blocking condition |
| `2` | command grammar or option value is invalid |

Doctor treats `missing` and `ready` config as non-blocking because typed
defaults remain usable. Invalid, future, insecure, busy, recovery-required, or
write-unsupported config is blocking. The unavailable secure store is reported
as a non-blocking limitation until credential-backed providers enter scope.

## Environment And Root Resolution

Commands that touch config resolve `QIONGLI_CONFIG_HOME` through the accepted
`qiongli-config` root validator. Default resolution uses `HOME` on Unix and
`USERPROFILE` on Windows, with a Windows `HOMEDRIVE` plus `HOMEPATH` fallback.
Missing, relative, traversal-bearing, device-namespace, or otherwise invalid
roots fail closed.

Help, version, and embedded-content listing do not require a home directory.
All supported commands must work with an empty `PATH`; production code must not
launch Python, Node.js, Cargo, a shell, or another Qiongli process.

## Privacy And Error Contract

Usage errors contain only static messages and the relevant static usage block.
Operation failures emit only `error: <allowlisted-reason-code>` on stderr.

The command adapter must not render:

- concrete config or materialization paths;
- raw command arguments or environment-variable values;
- SIDs, usernames, emails, secret references, or credential values;
- document bytes, managed-receipt details that contain paths, or raw I/O/Win32
  text; or
- `Debug`/`Display` output from path-bearing materialization errors.

`MaterializationError` therefore gains a path-free stable `reason_code()`
adapter. Existing detailed error variants remain available inside the trusted
service and tests; only the public CLI rendering is redacted.

## App Boundary

`main.rs` remains a thin process adapter. Parsing, command composition, output
models, config-root construction, and redacted error mapping live in the app
library so they are testable without duplicating content or config logic.

The app may depend directly on `qiongli-content`, `qiongli-config`, `serde`, and
`serde_json`. It does not add a CLI framework, async runtime, home-directory
package, process launcher, networking dependency, or platform integration
crate in this batch.

## Nonclaims

This command slice does not implement:

- provider configuration beyond default profile selection;
- credential input, keychain/vault storage, or plaintext fallback;
- resource read/export commands beyond profile listing/materialization;
- Lite or Full MCP framing and dispatch;
- agents, ToolHost, orchestration, project state, or 1.x migration;
- Codex/Claude discovery or installation;
- desktop UI, packaging, signing, updater, or clean-machine release acceptance;
  or
- Windows ACL/hard-link hardening for content materialization.

## Acceptance Criteria

The batch is complete only when:

1. every accepted grammar path has parser and binary-level tests;
2. content list and materialization work from the embedded pack without a
   checkout or external runtime path;
3. materialization uses only an explicitly approved target capability and
   failed commands do not mutate an unapproved target;
4. config show/set preserve redaction, provider fields, optimistic revision,
   owner-only persistence, and stale-write rejection on Unix and Windows;
5. status and doctor are read-only, deterministic, and path/secret redacted;
6. argument, environment, config, and materialization canaries never appear in
   public errors or JSON;
7. local native boundary, format, locked check, Clippy, and workspace tests
   pass without Python or Node suites;
8. Windows cross-target check and Clippy pass before push; and
9. exact-head boundary, Linux, macOS, and Windows GitHub jobs pass before the
   roadmap and Draft PR claim R1 command completion.

## Approval Record

The user instructed continuation with the next planned batch on July 14, 2026.
That accepts this already-roadmapped R1 command composition boundary, but does
not expand it to R2 Lite runtime or R3 UI/installation work.
