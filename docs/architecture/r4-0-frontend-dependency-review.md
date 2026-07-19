# R4-0 frontend dependency review

- Review date: 2026-07-19
- Scope: direct production and test dependencies introduced by the R4-0
  Tauri/Svelte presentation cutover
- Resolution authority: `pnpm-lock.yaml` and `packages/qiongli-native/Cargo.lock`

## Accepted production dependencies

| Concern | Package | Exact version | License | Upstream | Decision |
|---|---|---:|---|---|---|
| Native WebView shell | `tauri` | 2.11.5 | Apache-2.0 OR MIT | `tauri-apps/tauri` | Accepted; Rust owns the two custom commands and no shell or filesystem plugin is enabled. |
| Tauri build integration | `tauri-build` | 2.6.3 | Apache-2.0 OR MIT | `tauri-apps/tauri` | Accepted; embeds the static frontend at build time. |
| UI runtime | `svelte` | 5.56.6 | MIT | `sveltejs/svelte` | Accepted; presentation state only. |
| Static routing/build | `@sveltejs/kit` | 2.70.1 | MIT | `sveltejs/kit` | Accepted with `adapter-static`, SSR disabled, and SPA fallback. |
| Accessible primitives | `bits-ui` | 2.18.1 | MIT | `huntabyte/bits-ui` | Accepted for the confirmation dialog and future headless controls. |
| Icons | `@lucide/svelte` | 1.25.0 | ISC | `lucide-icons/lucide` | Accepted; individual icon imports and text labels for critical actions. |
| Runtime DTO validation | `zod` | 4.4.3 | MIT | `colinhacks/zod` | Accepted in the framework-neutral App client. |
| Tauri JS invoke bridge | `@tauri-apps/api` | 2.11.1 | Apache-2.0 OR MIT | `tauri-apps/tauri` | Accepted; only `invoke` is imported. |

Tailwind CSS 4.3.3 is accepted as a build-time styling dependency under MIT.
It emits static CSS and is not an installed runtime service.

## Accepted build and test dependencies

`@sveltejs/adapter-static` 3.0.10, Vite 8.1.5, TypeScript 5.9.3,
Vitest 4.1.10, Svelte Testing Library 5.4.2, and jsdom 29.1.1 are pinned in
the lockfile. They are used only while building or testing and do not appear as
Node processes in the installed product.

## Deferred dependencies

TanStack Query/Form/Table/Virtual and Cytoscape are approved role allocations
in the roadmap but are intentionally not installed in R4-0. They enter only
when a shipped feature needs their owning concern. This avoids a second cache,
form model, table abstraction, or graph authority before R4A/R4B.

## Boundary and payload review

- The capability manifest grants the main window no Tauri plugin permissions.
  The WebView can invoke only the two application commands registered in Rust.
- No Tauri shell, filesystem, process, HTTP, opener, clipboard, or SQL plugin is
  present in the frontend dependency graph.
- The CSP denies remote scripts, frames, objects, forms, and arbitrary network
  access. Static application code is embedded with the Rust product.
- The current static SPA output is below 0.5 MiB uncompressed. This is a
  development observation, not a permanent budget exemption; release receipts
  must continue to bind the exact asset tree.
- Direct dependencies use permissive licenses compatible with this repository.
  Transitive identities are locked and remain subject to the normal release
  license and vulnerability gates.

## Re-review triggers

Re-review is required when adding a Tauri plugin or permission, remote content,
a second router/state authority/design system, a new custom accessibility
primitive, Cytoscape, any TanStack package, or a direct dependency that ships
runtime code in the installed application.
