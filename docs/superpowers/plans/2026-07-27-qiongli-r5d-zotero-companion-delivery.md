# Qiongli R5D Zotero Companion Delivery Plan

Status: D0–D3 complete locally — D4 in progress

Date: July 27, 2026

Target branch: `feat/r4b-ui-localization-polish`

Baseline: desktop migration and integration repair commit `4e04ba55`

Roadmap:
`docs/superpowers/roadmaps/2026-07-13-qiongli-2-accelerated-rust-migration-roadmap.md`

## Goal

Make the existing Qiongli Zotero Companion a truthful, product-controlled local
integration without embedding Zotero, mutating a Zotero profile silently, or
turning Qiongli into a second reference-library authority.

Qiongli owns:

- deterministic Companion XPI assembly and release identity;
- bounded Zotero Desktop, Connector, Companion, version, and endpoint
  observations;
- a user-confirmed handoff to Zotero's own Plugins interface;
- post-install and post-restart verification; and
- import-file fallback whenever live local integration is unavailable.

Zotero owns plugin trust confirmation, plugin activation, plugin updates after
installation, and the user's reference library.

## Product sequence

```text
inspect Zotero and Companion without mutation
  -> verify the bundled XPI identity
  -> explain plugin authority and requested local access
  -> open Zotero and reveal the exact XPI
  -> user confirms installation in Zotero Plugins
  -> verify Connector and /qiongli/ping after restart
  -> enable explicit dry-run search/write planning
  -> retain import-file fallback at every unavailable state
```

No state may report `ready` from an XPI file, profile directory, install
receipt, or successful handoff alone. `ready` requires a live, bounded,
loopback-only Companion response with the supported endpoint version.

## D0 — Status and compatibility contract

Status: complete locally; awaiting integration commit

- add one strict top-level Zotero integration projection to the native Desktop
  snapshot and App API;
- distinguish `not-observed`, Zotero unavailable, Companion missing,
  Companion incompatible, update available, restart required, ready, disabled,
  and not observable;
- expose Connector and Companion observations separately;
- expose installed Companion version, available bundled version, observed
  endpoint version, and the supported endpoint version without returning an
  endpoint URL or response body;
- keep CSL-JSON, RIS, BibTeX, and import-report fallback availability visible;
- reserve explicit verify and prepare-install capabilities without performing
  either operation during an ordinary snapshot; and
- reject unknown fields, invalid versions, unbounded formats, and contradictory
  states at both Rust and TypeScript boundaries.

Acceptance:

- ordinary snapshot construction performs no loopback request;
- the new contract is schema-versioned and fixture-backed;
- `not-observed` is neutral rather than an installation failure;
- fallback remains available before Zotero is installed or observed; and
- no Zotero profile path crosses the App API.

Implementation evidence:

- `qiongli-runtime` owns the supported Companion endpoint version and derives
  disabled, Zotero-not-running, Companion-missing, incompatible, and ready
  states from bounded observations;
- Native Desktop exposes a neutral, read-only, top-level Zotero projection
  without probing loopback during ordinary snapshot construction;
- App API schema version 9 carries the strict projection and path-free Desktop
  handoff actions across the Rust and TypeScript boundary;
- Rust rejects contradictory ready, missing, incompatible, and not-observed
  evidence, and TypeScript mirrors the same high-value contradictions;
- the canonical Rust fixture, development transport, and frontend contract all
  use the same fallback formats and endpoint version; and
- targeted Runtime, UI, Native Desktop, App API, Desktop test, typecheck, and
  production-build verification passed on July 27, 2026.

## D1 — Product-controlled Companion artifact

Status: complete locally; awaiting integration commit

- make the XPI build deterministic and include its version, compatibility
  floor/ceiling, endpoint version, size, and SHA-256 in a strict manifest;
- include the verified XPI and manifest in every advertised desktop package;
- reject missing, changed, oversized, or incompatible Companion artifacts;
- keep XPI version independent from the Qiongli App version while binding both
  identities into the package receipt; and
- add package inventory and copied-artifact acceptance without Python or Node
  at runtime.

Implementation evidence:

- Rust and Python independently assemble the same canonical, deterministic XPI
  and strict artifact manifest from the four bounded Companion sources;
- the source build embeds and verifies the artifact without Python or Node at
  runtime and materializes it under `Qiongli.app/Contents/Resources/Zotero`;
- macOS, Windows, and Linux Desktop package manifests require the XPI and
  artifact manifest, bind both digests and the independent Companion version,
  and reject missing or changed entries;
- Desktop package receipts, packaged-product acceptance, release evidence, and
  Linux AppImage validation carry the same Companion identity; and
- a source-built macOS App passed artifact verification, strict code-signature
  verification, native startup check, and product-version inspection on
  July 27, 2026.

## D2 — Desktop installation handoff

Status: complete locally; awaiting integration commit

- add a Zotero integration card with Refresh, Prepare installation, Reveal XPI,
  Open Zotero, and Verify actions;
- never copy directly into a Zotero profile or extension database;
- show plugin version, supported Zotero versions, digest, requested local
  access, and restart consequences before handoff;
- use an opaque, receipt-bound staged artifact rather than exposing arbitrary
  filesystem selection to the WebView; and
- report `restart-required` until a compatible live Companion is observed.

Implementation evidence:

- Client Integrations now presents a dedicated Zotero card with separate local
  Refresh, approval-gated Prepare installation, receipt-owned Reveal XPI,
  detected-app Open Zotero, and explicit live Verify actions;
- the Rust service stages the exact embedded and verified XPI under
  Qiongli-owned `v2/zotero/companion` state with a canonical receipt, atomic
  persistence, idempotence, drift rejection, and no Zotero profile write;
- the WebView sends no path for any Zotero action; native code resolves only
  the verified staged XPI and an allowlisted detected Zotero application;
- Refresh performs local disk/application observation only, while Verify alone
  performs the bounded 900 ms loopback Companion probe;
- the card discloses the independent Companion version, Zotero compatibility
  range, exact artifact digest and size, requested local access, Zotero-owned
  confirmation/restart boundary, and import-file fallback;
- unsupported Zotero versions are distinct from missing, stopped, and
  incompatible Companion states and cannot prepare an XPI; and
- Rust staging, UI model, Native Desktop fixture/handoff, TypeScript contract,
  Svelte typecheck, 116 Desktop tests, and the production frontend build passed
  on July 27, 2026.

## D3 — Full MCP and explicit library operation qualification

Status: complete locally; awaiting integration commit

- connect the Desktop observation to the existing native
  `qiongli_zotero_status` contract;
- qualify local search, collections, tags, notes, and attachment summaries;
- require dry-run before every reference upsert;
- require explicit write intent for non-dry-run operations;
- preserve Zotero-authored fields under the selected update policy; and
- fall back to import files without claiming a library write occurred.

Implementation evidence:

- Companion `0.3.0` and the App, Runtime, artifact builder, and Full MCP now
  require endpoint contract `2`; a live `0.2.2`/endpoint-1 Companion is exposed
  as a bundled update rather than as a valid connection;
- both direct Full MCP Zotero tools and the Zotero source inside literature
  search probe the endpoint contract before operating;
- bounded local search qualifies DOI, title, citekey, creator, year, tag, and
  collection filters and returns creators, tags, collection paths, note
  summaries, and attachment summaries without paths by default;
- every mutation requires an immediately preceding, one-shot, five-minute,
  plan-bound dry-run receipt plus `write_intent: "apply"`; direct, replayed,
  expired, or changed-plan writes are refused before mutation;
- upserts are bounded to 100 items, preserve non-empty Zotero-authored fields
  under the default policy, append only missing tags, and do not duplicate a
  matching child note;
- unavailable or incompatible live integration returns truthful CSL-JSON, RIS,
  BibTeX, and report fallback rather than claiming a Zotero write; and
- 26 Companion tests, 139 passing Full MCP tests with three platform skips,
  Runtime/UI/Desktop compatibility tests, 21 App API tests, and deterministic
  artifact tests passed on July 27, 2026.

## D4 — Packaged acceptance and release readiness

Status: automated source-App acceptance complete; clean-commit packaged run and
Zotero-owned manual gates pending

- use an isolated Zotero profile and disposable library;
- cover missing Zotero, not running, missing Companion, incompatible endpoint,
  install handoff, restart, ready, update, disable, and removal;
- prove no profile mutation before Zotero's own confirmation;
- prove XPI/package/endpoint identities and import-file fallback after restart;
- run the accepted search and dry-run/write lifecycle against disposable data;
  and
- keep all evidence non-publishing until the parent Beta gates permit release.

Automated implementation evidence:

- the release artifact builder emits a Mozilla-style
  `qiongli-zotero-companion-updates.json` only when given an explicit release
  tag, binding Companion version, Zotero compatibility, immutable XPI URL, and
  XPI SHA-256; the release target registry uploads it beside the XPI while
  prerelease App updates remain App-staged until the latest stable release
  advances the public update URL;
- `pnpm acceptance:zotero` validates the exact Companion resources inside a
  macOS App, runs `app snapshot` under a disposable HOME, proves no Zotero
  profile directory is created, and reruns the native state matrix,
  deterministic artifact checks, Companion lifecycle, and Full MCP
  qualification;
- the source App built on July 27, 2026 carried Companion `0.3.0`, endpoint
  contract `2`, a 65,639-byte XPI, and matching XPI digest
  `77fff3a2841571a7f15b519b753f6b20eaf4c93492fea59c3b01cdfd8ca0c17c`;
- the resulting ignored receipt is
  `dist/macos/qiongli-r5d-zotero-acceptance.receipt.json`, is explicitly
  non-publishing, and records that this dirty source App has no Desktop package
  manifest binding;
- the same receipt binds the source App XPI byte-for-byte to the independently
  built release XPI and records update-manifest digest
  `95116c93994de318f642cab2a958ab34fa7772e766c855c391e9207a860fc623`;
- `desktop:macos:acceptance` now runs the same R5D verifier against its
  clean-commit accepted App and retains the separate receipt beside the main
  packaged-product acceptance receipt; and
- automated evidence covers missing/stopped/missing-Companion/incompatible/
  update/ready/disabled/removal state transitions, XPI staging without profile
  mutation, endpoint shutdown, bounded search, one-shot approved writes,
  duplicate preservation, and import-file fallback.

Manual Zotero gate:

1. Commit the intended R5D source, then run
   `pnpm desktop:macos:acceptance:open`. Confirm the R5D receipt reports
   `desktopPackageManifestBound: true`.
2. Quit the ordinary Zotero session. Launch
   `/Applications/Zotero.app/Contents/MacOS/zotero -P`, create a dedicated
   `Qiongli-R5D-Acceptance` profile, and give it a new disposable data
   directory. Do not enable sync or sign in. Zotero notes that a new profile
   may otherwise reuse an existing data directory, so verify the directory in
   Settings → Advanced → Files and Folders before continuing:
   <https://www.zotero.org/support/kb/multiple_profiles>.
3. With that profile running and no Companion installed, use Refresh and Verify
   in Qiongli. The App must distinguish installed/stopped Zotero from
   Companion missing and must keep import-file fallback visible.
4. Select Prepare installation and inspect the preview. It must show Companion
   `0.3.0`, endpoint `2`, Zotero `8.0`–`9.0.*`, the exact XPI digest and size,
   local access disclosure, and restart consequence. Cancel once and confirm
   that the disposable Zotero profile is byte-for-byte unchanged.
5. For the update gate, download the published legacy fixture
   `qiongli-zotero-companion-0.2.2.xpi` from
   <https://github.com/jxpeng98/qiongli/releases/download/v1.17.0/qiongli-zotero-companion-0.2.2.xpi>
   and verify SHA-256
   `1b6308b4fc92f4992a4202813b2a30df0fc9ab3cf6a36098bc3b5f85fffa7012`.
   Install it only in the disposable profile, restart, and run Verify. Qiongli
   must show an incompatible legacy endpoint as `update-required`, never
   Ready.
6. Confirm Prepare, use Reveal XPI, and install the revealed `0.3.0` file
   through Zotero Tools → Plugins. Zotero's current plugin documentation
   requires the `.xpi` to be dragged into that window and warns that plugins
   have full local access: <https://www.zotero.org/support/plugins>. Qiongli
   must remain `restart-required`; staging alone must never report Ready.
7. Restart only the disposable Zotero profile and run Verify. Ready requires a
   live Companion `0.3.0` response with endpoint `2`.
8. In the disposable library, exercise one title search, one dry-run upsert,
   and one approved write using the immediately preceding receipt. Confirm a
   replay or changed plan is rejected, a duplicate keeps curated metadata, a
   matching child note is not duplicated, and attachment results expose no
   path by default.
9. Disable the Companion in Zotero, restart, and verify Qiongli leaves Ready.
   Re-enable and restart to prove recovery. Finally remove the Companion,
   restart, and verify Companion Missing plus import-file fallback.
10. Preserve the displayed states, then delete the disposable Zotero profile
    and data directory. Run the manual recorder with every gate identifier
    reported by
    `pnpm acceptance:zotero:manual-record -- --list-gates`, including
    `--confirm disposable-profile-removed`. The recorder requires the clean
    packaged automated receipt, exact source commit, package manifest, XPI,
    and update-manifest identities; it refuses partial confirmations and
    remains non-publishing. Do not publish either receipt while the parent Beta
    gates remain open.

## Nonclaims

R5D does not:

- bundle or redistribute Zotero Desktop;
- install an extension silently;
- edit Zotero's SQLite database or profile metadata directly;
- treat an extension file as activation evidence;
- call a model provider;
- upload a Zotero library; or
- replace Zotero as the authority for references, collections, notes, or
  attachments.

## Completion gate

R5D is complete when:

1. every advertised desktop package carries one verified Companion artifact;
2. installation requires Zotero-owned user confirmation;
3. App, CLI, and Full MCP agree on the same live compatibility observation;
4. incompatible, missing, stopped, and unobservable states remain distinct;
5. explicit writes pass dry-run, approval, restart, and duplicate qualification;
6. import files remain usable without Zotero or the Companion; and
7. packaged acceptance proves install, update, verification, disable, removal,
   and fallback without mutating a real user library.
