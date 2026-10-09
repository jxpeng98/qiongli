# ADR 0241: Unified Native Provider Setup

- Status: Accepted
- Date: 2026-10-09
- Task: CLI-402
- Owners: Qiongli maintainers / CLI

## Context

The native Plugin exposes ConfigureProvider and SaveProvider, but the shared
runtime returns capability-unavailable. Documentation promises a local browser
page that only retained runtimes provide. A new CLI/Plugin user therefore cannot
configure literature credentials through the supported CLI-first entrypoint.

## Decision

CLI `provider setup [provider] [--no-open]` and CLI-hosted Lite/Full MCP
`qiongli_configure_provider` (including its compatibility alias) use one short-lived
loopback page. MCP returns its URL and never launches a browser. CLI optionally
opens the browser and waits for a confirmed save or expiry. `provider status` and
`doctor` inspect metadata offline; `provider test <provider>
--confirm-network-request` makes one bounded request to that provider through the
existing runtime, returning only connectivity diagnostics. Empty results do not
mean connection failure. No tool is added or removed.

The 192-bit bearer URL binds an IPv4 loopback listener, selected provider and
ten-minute lifetime. Host/Origin/fetch-site, request size/deadline, duplicate
headers/fields and selected-provider checks precede any mutation. GET never writes.
A page submission requires explicit approval and the revision that was displayed.
One successful save consumes the listener. Error pages never echo submitted values.
Responses disable caching, framing and referrer disclosure. Passwords stay on this
page; the native raw-value Save tool returns setup-required guidance without
writing. This specifies the native implementation of the existing tool contract;
retained legacy runtimes are not changed.

Reuse the existing provider preview/confirmation transaction and global settings
CAS owner. Change only the selected provider. Blank fields keep existing values;
removal is explicit. Removing a key also disables its provider. New/rotated keys
use fresh opaque references, store/read-back verification and failed-commit
compensation, leaving the old reference valid until settings commit. Old key
cleanup happens after commit; a cleanup failure is reported as committed with
cleanup required rather than pretending to roll back a completed save.

Preserve ADR 0204: ordinary settings contain only opaque references. The current
macOS Keychain adapter is reused. No plaintext/environment fallback, new vault,
Windows/Linux credential backend or model credential ownership is introduced;
unavailable stores refuse secret writes explicitly. Public-only configuration
remains usable on those platforms. Provider metadata and bounded credential-loader
caches refresh on settings revision changes, including after a page save in the
same MCP process. Discovery/status never resolves credentials.

## Validation and limits

Verify actual isolated HTTP/CLI/MCP flows, alias reuse and fresh same-session
metadata. Negative cases cover missing/wrong/expired tokens, Host/Origin, duplicate
fields/headers, oversized/slow requests, unapproved/unsupported writes, stale
revisions, unavailable stores, read-back/store/commit/cleanup failures and canary
redaction. Preserve unrelated settings and prior credentials across failed CAS.
Reuse closest existing runtime/CLI tests and strict formatting/Clippy.

Fake-store/loopback evidence is not macOS Keychain, real Host/session or online
provider acceptance. A stored credential and a successful bounded online test are
different observations. Changes require new local integration, not publication.
The separate standalone Lite package retains its existing unavailable setup
response; capability-gated Skill guidance preserves that fallback. Regenerate the
local content lock through its existing owner when changing the shared guidance.
