# Content And Distribution

Canonical academic content and public capability contracts live under
`content/`. Plugin directories, installed Skills, embedded packs, and release
payloads are generated outputs.

## Local Pattern

- Edit workflow/Skill source under `content/workflow/` and `content/skills/`.
- Edit Plugin metadata in `content/distribution/plugins.yaml`.
- Edit MCP public profiles and schemas under `content/mcp-contracts/`.
- Materialize payloads through `tooling/scripts/`; do not patch `dist/`,
  installed client directories, or generated plugin trees.
- A Skill may name a tool only when the selected MCP profile exposes it. If a
  runtime cannot provide the operation, the Skill must define a truthful safe
  fallback instead of assuming another product line is installed.

User-edited Plugin/Skill variants are managed project/user outputs. They do not
replace canonical content and must retain preview, receipt, and exact-removal
boundaries.

## Skill instruction ownership

The root Skill owns the explicit conversational reply-only choice. Resolve it
before any resource read, workflow or MCP check; the unified router and generated
Codex entry template retain an inline guard so they need no read to honor it.
It lasts until explicitly resumed unless scoped to one reply. This is not a Host
tool lock, global setting or automatic Hook disablement. Keep ordinary routing and
permission gates unchanged outside this choice; no extra runtime mode is required.

`content/workflow/no-qiongli/SKILL.md` is the self-contained reply-only entry.
Local native and Marketplace projectors place its exact bytes at
`skills/no-qiongli/SKILL.md` for both Hosts; standalone profiles retain the
`workflow/no-qiongli/SKILL.md` source path. Allow only this extra Plugin file,
not arbitrary siblings or scripts. Older packs without it keep their projection.

The root Skill selects the requested outcome and relevant resources. Stage F
writing workflows, cards and roles reference
`content/workflow/references/stage-F-writing.md` for their shared writing
contract; they must not duplicate mandatory chunk sequences or paragraph quotas.
`self-critique.md` owns review convergence. A standard/deep label alone does not
impose a minimum pass count. Explicit protocols, saved-run limits and required
independent review remain binding; the retained 1.x controller is unchanged.

Stage C design cards, workflows, role perspectives and the core digest share
`content/workflow/references/stage-C-design.md`. They retain method/protocol
requirements without imposing generic sample, rival or robustness quotas. The
preregistration card owns C5 and truthful collection/access/registration status;
a draft never establishes registration or access permission. Literature review
guidance retains the existing search-quality contract and protocol boundaries;
record counts alone do not justify broadening a search.

A5 venue analysis and H1/H3/H4/H5 share
`content/workflow/references/stage-H-submission.md` for venue evidence, manuscript
fit, adaptation and review boundaries. Verify requirements for the actual venue,
article type and submission stage; local profiles remain discovery aids. Preserve
uncertainty, source locations and proportionate remedies rather than universal
candidate/flaw quotas or assumed reviewer preferences. A5 adapts to a selected
target; H5 inspects an existing draft before recommending targets.

Preserve canonical artifact paths, evidence anchors, stable IDs and formal gates.
A narrow chat answer need not generate the full formal artifact set. Writing
freedom does not waive tool availability, preview/approval/CAS or user-only file
deletion. Validate shared-reference reachability and projection bytes; use
isolated behavioral trials for response quality, since text assertions alone
cannot establish model behavior or cross-model performance.

## Pre-Development Checklist

- Identify the canonical source and every generated consumer.
- Compare Skill tool names with the v2 registry and native tool registries.
- Decide whether the change affects embedded-pack or release inputs.

## Quality Check

- `python3 scripts/validate_capability_contract.py`
- Run the closest materialization or payload audit only when its inputs changed.
- Confirm generated outputs were not edited directly.

The retained `audit_distribution_payloads.py` prunes name-excluded directories
before hashing, while preserving its file/suffix rules and deterministic results.
Included-directory scan errors fail the audit; partial inventories cannot pass.
Its separate generated-tree symlink guard still inspects excluded subtrees.

## CLI registry packages

`native_cli_release.py` owns standalone GitHub CLI archives and their generated,
target-specific README. Each archive carries the executable with embedded content,
README and LICENSE; the assembly owner retains all three platform archives and
their hashes in the release packet. Direct-download instructions must identify
the platform asset, checksum verification, extraction, executable name and PATH
option without requiring a package manager or confusing GitHub source archives
with runnable binaries. Published archives remain immutable.

Standalone binaries require no separately installed language runtime or package
manager. The Windows release owner sets `target-feature=+crt-static` only for
the explicit target and checks the final PE imports with the build machine's
LLVM tools, rejecting non-system DLLs. Extracted CLI/Lite/Full MCP smoke checks
run with empty PATH on every target. New native release packets require that
evidence and the Windows import list. LLVM is a build check, not a user runtime
dependency; supported OS libraries remain required (Linux x64: glibc 2.35+).
Public Windows alpha.8 still imports `VCRUNTIME140.dll`; download instructions
must disclose that exception until a newly qualified version replaces the link.

`release_version.py` owns SemVer/Git/npm and PEP 440 version projections;
prerelease npm publication uses `next`. `native_registry_packages.py` owns fixed
OS/CPU dispatch, executable bytes and platform wheels. The three-platform
`native-cli-distribution.yml` builds and installs on each target, assembles one
npm package and three wheels, and tests the combined package on each target.
`native_release_assets.py` refuses mixed source/version, missing targets and
changed bytes. Registry jobs reuse existing workflow filenames/environments and
require a successful exact-source distribution run. No App upload is part of this lane; product approval/CAS and managed trust remain unchanged.

The CLI Cargo manifest owns the shared product description. Generated standalone,
npm, PyPI and Cargo READMEs reuse it while keeping channel-specific installation,
runtime prerequisites and removal instructions. npm carries its Node launcher,
wheels their Python launcher, and Cargo its Rust source closure; none packages
another channel's launcher. Build/staging rejects mixed workspace, executable or
embedded-content versions. SemVer and PEP 440 spellings remain distinct projections
of one release; channel tags do not identify immutable bytes.

Cargo uses the staged workspace and existing archive install checker (ADR 0221).
`publish-cargo.yml` runs native source verification on three systems, publishes
only on a qualified native GitHub Release, and checks public registry installs.
Cargo uses exact SemVer and both `qiongli`/`ql`; only the staged manifests permit
publication. Credentials and successful registry resolution are separate gates.
Staged manifests normalize CRLF input and write LF explicitly on every platform.
Cargo uploads run through GitHub Actions with the maintainer-configured
`CARGO_REGISTRY_TOKEN` repository or `crates-io` environment secret. Missing
credentials fail the publication job; local upload is not a fallback. Manual
`verify_public` dispatch checks an already published version without uploading.
The beta.1 local bootstrap remains historical evidence. Trusted Publishing is
deferred until the maintainer requests that migration.

User-approved local Plugin sources (ADR 0222) reuse the native Codex/Claude
bundle projectors, include the current executable and use a dedicated
`qiongli-cli-local` marketplace. They are derived exports, not canonical content
or signed products. Their `user-local-host-full-mcp` receipts contain no signed
grant digest; signed bundle APIs reject them. Local receipt schema 4 follows the
executable channel in the deferred 2.1 migration; ADR 0229 keeps the 2.0.1
transition on its existing Next/v1 contract. Schemas 2/3 keep their historical
Next identities and remain verifiable after public v1 output is retired. Source
updates/removal require the expected receipt inside the existing bundle transaction. Host registration,
cache refresh and live readiness remain separately observed actions.
ADR 0224's terminal `install/upgrade plugin` flow now offers official Codex/Claude
registration after a second, exact-plan confirmation. It may replace only the
selected local Plugin's completely verified cache; unrelated enabled Qiongli
Plugins and changed files block that step. The terminal Codex migration may now
disable explicitly listed legacy/platform Qiongli entries after the separate Host
confirmation. It uses the official `config/batchWrite` API with an observed user
configuration version, preserving old source/cache files and other settings.
Unknown identities, non-user entries, unsupported APIs and changed configurations
fail closed. A failed subsequent registration leaves a disclosed disabled previous
Plugin that the user can re-enable; no automatic deletion or rollback edits are
performed. Claude keeps its manual conflict handling. Lower-level source plan/apply commands
still only export files. CLI package updates stay with the original installer.
Optional local context hooks are projected inline into the selected Host manifest
and bound to its receipt and native binary. Installation previews the configuration
and preserves the choice on updates. No Python/Node wrapper, global settings edit
or automatic Hook trust is added. Public Marketplace and signed bundles keep their
existing defaults; standalone Skills exports do not install Hook configuration.

Public Marketplace Plugins (ADR 0223) use `native_marketplace_plugins.py` and
the CLI's `export_marketplace_content` example. Shared research resources retain
the exact `marketplace-lite` pack bytes. Each Codex/Claude archive bundles its
qualified target's CLI and directly starts Lite MCP (14 tools) without Node,
npm, Python, a shell bridge or executable downloads. Full MCP remains available
through explicit CLI/local Plugin configuration; packaging does not expand tools.

ADRs 0228/0229 reserve explicit `qiongli-<platform>` stable identities for the
2.1 migration; the 2.0.1 transition retains `qiongli-next-<platform>`, as do
Alpha/Beta releases. There is no automatic OS selection in a generic Host
manifest. `marketplace-plugins.json`
maps all six archives to Host, target, digest, plugin path and immutable
`<host>/<target>/v<version>` distribution ref. External marketplace catalogs must
consume that mapping and present the platform choices before public rollout.
Do not point an unqualified generic entry at one platform's binary.

Native Codex local exports and Marketplace archives generate workflow Skills from
`workflow/workflows/*.md` using the shared
`workflow/references/codex-workflow-wrapper.md` template. Preserve each canonical
single-line YAML description and the stable `qiongli-<workflow>` name. Exclude
the duplicate `qiongli` router; the main `$qiongli` Skill remains available.
Skill cards, references and templates do not get independent wrappers. Each
entry loads the shared Skill before its workflow, preserving Host tool limits,
request scope and preview/approval/CAS. Wrapper paths permit only bounded
lowercase slug names and `SKILL.md`; receipt validation covers their bytes.
Claude retains the main research Skill and the independent reply-only entry.
Old packs without the template
retain their historical projection, so existing archives remain verifiable.

Schema-2 archive receipts bind the target, executable and resource bytes. The
release owner verifies the executable against the corresponding CLI/npm bytes,
requires the same embedded pack across targets and binds target-native empty-PATH
CLI/MCP checks to each archive. The final combined-package matrix exercises the
extracted MCP manifest on each system. Preserve executable mode 0755 in tarballs
and Git distribution trees; research resources use 0644. Unknown platforms,
wrong executable formats, missing/changed bytes, altered profiles or permissions,
and incomplete/index-mismatched packets fail qualification.

Schema-1 alpha.8 npm-bridge archives remain verifiable with their historical
names and contents. New builds emit only schema 2. Public changes require a new
version and six qualified immutable distributions; existing alpha.8 refs and
assets remain unchanged. No signed grant, managed activation, Host registration,
private research access or model configuration change is implied by an archive.

Beta.2 permits exactly one npm `postinstall` script: the canonical terminal-only
installation review launcher. Asset verification checks its command and bytes;
arbitrary scripts remain rejected. It performs no download or cleanup, tolerates
cancellation, and skips non-terminal streams. `--ignore-scripts` remains supported.
Python wheels and Cargo retain standard installation; the installed native CLI
owns their subsequent interactive review.

An explicit `release-automation.yml` post dispatch at an immutable native
release tag qualifies the same tag through Native CLI distribution, verifies
its packet, creates the matching GitHub release, then dispatches the existing registry
workflows at that tag. Dispatch uploads are opt-in and tag-only; existing
credential environments and exact-source CI gates still apply. GITHUB_TOKEN
release events do not chain jobs, so publisher dispatch is explicit. The local
Agent may end after submission when requested; no public success is inferred.

ADR 0227 places native stable source on `main` and keeps prerelease development
on `2.x`. Stable publication requires the tag to equal the frozen remote main
head, uses GitHub latest and npm latest, and retains matching PyPI/Cargo versions.
Main pushes qualify builds without publishing. Native Marketplace archives also
accept stable SemVer. ADR 0229 retains Next IDs and v1 output in 2.0.1; ADR
0228's stable qiongli IDs/MCP keys and v2 output move to the planned 2.1 cutoff.
Alpha/Beta keep qiongli-next. New 2.0.1 packets must carry observed transition
evidence; previously published projections remain byte-verifiable. The legacy
TestPyPI builder is restricted to `release/1.x-python`; native PyPI assets never
use the frozen Python package. A main merge does not update that maintenance
branch, promote an external catalog or establish product acceptance.

## Current user documentation

`docs/` and `docs/zh/` describe native 2.x; retained 1.x operational guides are
explicitly labelled and excluded from normal site search. Historical ADRs and
acceptance records keep their original evidence scope. Main integration does not
change a published version or establish stable acceptance.

`tooling/scripts/generate_skill_docs.py` owns the current bilingual Skills guide.
It reuses the frozen registry reader and table renderer as build-time tooling,
without changing the 1.x product. Update its prose or canonical registry metadata,
then regenerate both reference pages. Keep channel package README generation in
the existing native packaging owner.
