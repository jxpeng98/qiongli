# Qiongli 2.0 R3F Native Desktop Manager Execution Plan

Date: 2026-07-14

Status: accepted at exact implementation-and-CI head `4d706033`

Branch: `feat/2x-native-alpha1`

Rolling PR: `#63`

Design:
`docs/superpowers/specs/2026-07-14-qiongli-r3f-native-desktop-manager-design.md`

## Batch 1 — Pin The Native UI Boundary

- add `qiongli-ui` to the native workspace;
- pin `eframe`, `egui`, and `egui_kittest` `0.35.0`;
- enable eframe AccessKit, native wgpu, fonts, Wayland, and X11 support;
- keep persistence, web-target, JavaScript-frontend, and webview features absent;
- do not call the native clipboard or link-opening facilities pulled in by
  eframe's window integration; and
- expose a native window configuration with bounded initial/minimum size.

Gate: locked dependency graph, host check, and Windows MSVC cross-check.

## Batch 2 — Add View Models And Typed Service Boundary

- define the bounded redacted `DesktopSnapshotV1`;
- define fixed section, status, provider, integration, doctor, capability,
  intent, event, operation-token, and preview types;
- prevent private input intents from implementing Debug or serialization;
- add a trait-object desktop service boundary with no concrete service types;
- validate snapshot ordering and stable identifiers with pure Rust tests.

Gate: model, redaction, bounds, and no-secret-debug tests.

## Batch 3 — Implement The Blocking Shell Prototype

- implement Overview, Skills, MCP, Providers, Integrations, and Diagnostics;
- apply a token-driven, high-contrast, flat academic visual system;
- use stock semantic widgets, labelled inputs, visible focus, adjacent errors,
  and disabled-state explanations;
- implement responsive side/top navigation and scale-safe scroll regions;
- add preview, confirmation/cancel, progress, blocked, and recovery feedback.

Gate: `egui_kittest` AccessKit, navigation, form, dialog, width, and scale tests.

## Batch 4 — Compose Real Read-only Services

- build a snapshot from the verified embedded pack and target identity;
- map redacted global-config and provider status without reading secret values;
- expose the exact native Lite MCP tool count and command shape;
- discover Codex and Claude Code through their accepted read-only adapters;
- map doctor and recovery states to fixed public codes;
- prove collection creates no user, host, temporary, or UI-persistence state.

Gate: isolated-home snapshot and side-effect tests.

## Batch 5 — Add The Canonical Desktop Mode

- parse `qiongli ui` without launching a window during parser tests;
- compose the UI service and eframe app only in the product entry point;
- return `desktop-ui-start-failed` for renderer/window startup failure;
- keep CLI, MCP, and content modes free from renderer initialization;
- document the source-build UI limitations truthfully.

Gate: CLI parsing, headless-mode, empty-`PATH`, and composition tests.

## Batch 6 — Acceptance And Rolling PR

- run format, locked workspace check, strict Clippy, all native tests, focused
  Lite compatibility, and Windows MSVC check/Clippy;
- commit and push cohesive checkpoints on the same rolling branch;
- monitor Native CI and Cloudflare on each accepted exact head;
- update the accelerated roadmap, native README, and Draft PR #63 with facts,
  evidence, next batch, rollback, and non-claims.

R3F closes only after exact-head CI. Manual packaged-window and assistive-
technology receipts remain later alpha.1 gates and may not be inferred from
the headless AccessKit test harness.

## Acceptance Record

R3F is accepted at design checkpoint `59f968c6`, implementation checkpoint
`66398b2c`, and exact implementation-and-CI head `4d706033`.

Local acceptance for the final tree proves:

- all 210 normal all-target, all-feature native Rust tests pass; the Codex and
  Claude external-client tests remain ignored in the default workspace gate;
- all eight headless `qiongli-ui` AccessKit tests pass;
- the focused Claude bundle tests pass 3/3 and the focused Codex bundle tests
  pass 2/2, with one external-client test ignored in each suite;
- strict host and Windows MSVC Clippy pass; and
- the test profile strips final debug information while retaining the 128 MiB
  plugin-composer source-binary limit.

The first implementation-head run, Native CI `29349701112`, showed that the
Linux debug test executable exceeded that existing bound after adding the UI
stack. Checkpoint `4d706033` fixes only the test artifact profile; it does not
increase the production limit or change runtime behavior.

Remote acceptance for exact head
`4d706033d093783a8c6214c1bc492b1540bffb61` passed in Native CI run
`29351331008`: boundary 6s, focused Lite 39s, macOS 4m45s, Linux 4m51s, and
Windows 6m16s. Cloudflare Pages also passed.

This acceptance proves a source-build native window mode and headless
accessibility tree. It does not prove packaged-window startup, manual
VoiceOver/Narrator/Linux assistive technology, signing, installation, updater,
or clean-machine desktop release readiness.
