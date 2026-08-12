# Native Runtime

The Qiongli 2 executable, CLI, Desktop service, Full MCP, project state, and
embedded resources live under `packages/qiongli-native/`.

## Local Pattern

- `apps/qiongli/src/command.rs` owns public CLI parsing and help.
- `apps/qiongli/src/desktop.rs` owns the shared App service; Tauri commands in
  `desktop/tauri_adapter.rs` adapt it instead of duplicating product logic.
- `crates/qiongli-runtime/src/contract.rs` and `apps/qiongli/src/mcp.rs` own the
  native MCP registry and dispatch boundary.
- [Full MCP profile routing](./full-mcp-profile-routing.md) defines how Full may
  reuse Lite validation without returning a Lite profile result.
- `crates/qiongli-project/src/service.rs` owns project mutations and revision
  checks; App, CLI, and Full MCP route through that service.
- `crates/qiongli-runtime/src/zotero/companion.rs` owns the loopback Companion
  boundary. Only loopback endpoints may be contacted.

Public writes use preview, digest-bound approval, revalidation, and fail-closed
errors. `qiongli_project_capture_apply` is a real Full MCP project write and
must never be described as read-only. ToolHost remains read-only in-process.

The native 2.x runtime must not fall back to Python or Node in production.
Legacy packages can provide migration evidence but are not runtime dependencies.

## Pre-Development Checklist

- Trace all App, CLI, MCP, and ToolHost callers of the shared owner.
- Check `content/mcp-contracts/` and affected Skills for the same public name.
- Preserve redaction, loopback-only networking, ownership, and revision checks.

## Quality Check

- Run the closest crate or integration test first.
- For contract changes, verify tool registry, dispatch, schemas, and docs agree.
- Before exact-head CI, run Rust format and the affected workspace tests.
- Confirm public CLI examples exist in the parser and `--help` output.
