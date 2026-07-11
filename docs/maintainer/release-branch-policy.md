# Release Branch Policy

The accepted Python-led 1.x line is now frozen. This repository uses `2.x` for
Rust-native development, keeps `release/1.x-python` as the accepted 1.x
compatibility oracle and critical-fix line, and retains `dev` as the recorded
post-release handoff branch. `main` remains the legacy stable release branch.

## Branch Roles

| Branch | Role | Allowed changes |
|--------|------|-----------------|
| `2.x` | Active Rust-native development, integration, and 2.x prerelease source | Native Rust workspace and product features, contract/resource loaders, native CLI/UI/MCP/orchestrator work, installers, tests, docs, CI, and 2.x release tooling. Python and Node may be used only as frozen oracle or build-time test inputs, never as production runtime dependencies. |
| `dev` | Accepted 1.x handoff and post-release baseline integration endpoint | A8 baseline evidence, branch governance, documentation, tests, and handoff metadata. No new 1.x product features and no Rust-native product implementation. |
| `release/1.x-python` | Accepted 1.x tag, compatibility oracle, and critical-fix-only maintenance line | Approved security or release-breakage corrections through pull requests, plus the minimum tests, release metadata, and documentation required for those corrections. No normal features. |
| `main` | Legacy stable release source | Stable release evidence and explicitly approved emergency maintenance. No normal 1.x feature development. |

Open native feature pull requests against `2.x`. Use `dev` only for the A8
handoff and cross-line governance after the final 1.x beta. Do not merge native
implementation back into `dev`, `main`, or `release/1.x-python`.

## 1.x Maintenance Governance

`release/1.x-python` points at the accepted annotated tag
`v1.19.0-beta.1` (`8d2e99866ce4c4efb8b3b5e0265c0c1f89a36b0f`). It is not a
moving copy of `dev`. Because it was cut at the accepted tag, it does **not**
contain the A8 workflow-filter changes committed later on `dev`; do not claim
that those later workflow definitions are present on the frozen branch.

The repository ruleset for this line is
[ruleset 18797579](https://github.com/jxpeng98/qiongli/rules/18797579). It
requires pull requests for changes to `release/1.x-python`; direct maintenance
pushes are not the operating procedure. It also blocks deletion and
non-fast-forward updates and has no bypass actors. The server-side ruleset is
the source of enforcement, while this document records the review policy.

A 1.x maintenance pull request is allowed only when all of the following hold:

1. the change fixes a critical security issue or release breakage and names the
   exception explicitly;
2. the failure is reproduced against the accepted tag and the smallest safe
   patch is proposed through a pull request;
3. the pull request includes focused regression tests, full applicable release
   gates, and artifact or rollback evidence proportional to the change;
4. the same behavior is forward-ported to Rust 2.x, or the pull request records
   equivalence evidence showing why the Rust line is unaffected and names any
   follow-up owner;
5. the change does not add a 1.x feature or silently move the frozen oracle.

The planned 1.x support window ends **90 days after Qiongli 2 stable** unless a
later, explicit support decision supersedes it. The security and
release-breakage exception remains subject to a release-owner decision; this
window is not permission to resume feature development.

## 2.x Native Branch Governance

`2.x` is created from the exact clean A8 handoff commit after the normalized
1.x baseline is frozen. It owns all subsequent native implementation and 2.x
release work. The `CI` and `Checkout Install Check` workflows accept pushes and
pull requests targeting `2.x` in the A8 handoff commit. The generated-payload
guard resolves a pull request's `github.base_ref`; on push events it uses the
event's prior commit and falls back safely to the first available parent/root,
instead of comparing every branch to `origin/dev`.

The same comparison base anchors the frozen-baseline guard. If the comparison
base already contains
`tooling/migration/baselines/v1.19.0-beta.1/manifest.json`, CI rejects every
change below that versioned baseline and every change to its 1.x baseline plan
or JSON Schemas. This catches a synchronized rewrite of an oracle and its
manifest even when offline verification would otherwise see internally
consistent hashes. A missing anchor is allowed only for the one-time A8 commit
and the first `2.x` push from that exact commit; after the anchor exists in
branch history, new conformance evidence must use a new versioned path rather
than rewrite the frozen 1.x evidence. The accepted A8 commit is therefore the
CI and branch-point trust anchor, while asset-backed `capture --check` is the
reproduction gate for the initial runtime outcomes.

After the initial `2.x` push and its first `CI` and `Checkout Install Check`
runs pass, add an active-development server-side ruleset that requires pull
requests and those exact checks, blocks deletion and non-fast-forward updates,
and has no bypass actors. Audit the existing `dev` protection at the same
handoff and record both ruleset identities or any corrective action. The
immutable guard is preventive only when its workflow is required; without
branch protection, a direct push could move the branch before a failing
workflow reports the violation.

Production code on `2.x` must be Rust-native and dependency-free for end users.
Frozen Python Full, Rust Lite, and Node MCPB results remain compatibility
oracles and test evidence; they are not allowed to become hidden production
dependencies.

## Official Plugin Linkage

The official public marketplace entry lives in `jxpeng98/skillsplace` and should point at the stable generated Qiongli plugin payload:

- Marketplace repository: `https://github.com/jxpeng98/skillsplace`
- Qiongli repository: `https://github.com/jxpeng98/qiongli`
- Stable Codex artifact: `qiongli-core-codex-plugin-<tag>.tar.gz`
- Stable Claude Code artifact: `qiongli-core-claude-plugin-<tag>.tar.gz` or `.zip`
- Stable generated payload root: `plugins/qiongli/`

The stable Skillsplace catalog entries should track `main` and stable release
tags, not `dev`. Use `2.x` for native plugin packaging tests and prerelease
validation after the A8 handoff; use `dev` only to preserve the recorded A8
baseline and governance evidence. This repository no longer carries Codex or
Claude marketplace catalog files; it owns the plugin manifests and materializes
the release payload from canonical source.

Prerelease tags publish the `qiongli-next` testing channel instead of the full stable marketplace matrix. The generated next artifacts are:

- `qiongli-next-codex-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.zip`
- `qiongli-next-claude-desktop-skill-core-<tag>.zip`

The `qiongli-next` Codex and Claude Code plugin artifacts install only the `core/complete` skill package and keep the bundled Rust Lite literature MCP runtime. They do not publish subject plugin variants. The Claude plugin ZIP contains the same plugin payload as the Claude tarball for upload flows that reject `.tar.gz`. Claude Desktop testing uses the focused core ZIP plus the separate `qiongli-literature-provider-<version>.mcpb` release asset. The Skillsplace catalog may expose a separate `qiongli-next` entry for beta testing while stable `qiongli` and subject entries continue to point at stable artifacts.

This repository does not track stable or beta plugin payload directories. `plugins/qiongli/`, `plugins/qiongli-next/`, `packages/qiongli-plugin/`, and `packages/qiongli-next-plugin/` are generated shapes. Change `content/workflow/`, `content/distribution/plugins.yaml`, or `tooling/scripts/build_plugin_artifacts.py`, then materialize into a staging directory for validation.

## Platform dist refs

Codex and Claude marketplace installs from `jxpeng98/skillsplace` use Git subdirectory sources when the reviewed plugin payload is generated at release time. Keep the existing release tag, GitHub Release, PyPI, npm, and archive artifact flow unchanged, and publish separate orphan branch refs for each platform:

```text
refs/heads/codex/v<version>
refs/heads/claude/v<version>
```

## Codex dist refs

Each Codex dist ref must contain only the generated plugin payload tree needed by the Codex marketplace entry:

```text
plugins/qiongli/.codex-plugin/plugin.json
plugins/qiongli-next/.codex-plugin/plugin.json
```

Codex refs must not include `.claude-plugin/`; Claude refs carry that metadata separately.

## Claude dist refs

Each Claude dist ref must contain only the generated plugin payload tree needed by the Claude marketplace entry:

```text
plugins/qiongli/.claude-plugin/plugin.json
plugins/qiongli-next/.claude-plugin/plugin.json
```

Claude refs must not include `.codex-plugin/` or `.mcp.json`. The release postflight automatically publishes the platform dist refs after it materializes the release staging payload and builds the existing plugin artifacts. Stable marketplace installs publish `plugins/qiongli` to `codex/v<stable-version>` and `claude/v<stable-version>`; prerelease installs publish `plugins/qiongli-next` to `codex/v<prerelease-version>` and `claude/v<prerelease-version>`.

Use `scripts/publish-codex-dist-ref.mjs` manually only when backfilling an existing release or intentionally repairing a dist ref from a verified staging directory:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
node scripts/publish-codex-dist-ref.mjs --channel codex --version 1.3.0 --slug qiongli --source /tmp/qiongli-dist/plugins/qiongli
node scripts/publish-codex-dist-ref.mjs --channel claude --version 1.3.0 --slug qiongli --source /tmp/qiongli-dist/plugins/qiongli
node scripts/publish-codex-dist-ref.mjs --channel codex --version 1.5.0-beta.1 --slug qiongli-next --source /tmp/qiongli-dist/plugins/qiongli-next
node scripts/publish-codex-dist-ref.mjs --channel claude --version 1.5.0-beta.1 --slug qiongli-next --source /tmp/qiongli-dist/plugins/qiongli-next
```

The publisher validates the channel-specific manifest, bundled MCP entrypoint, portable skill package, and version fields before it writes the orphan ref. Re-run with `--force` only when intentionally replacing an existing platform `v<version>` payload.

## Development Flow

1. After A8 records the branch point, start all native feature and packaging
   work on `2.x` and open pull requests back to `2.x`.
2. Run `CI` and `Checkout Install Check` for the exact pull-request commit.
   Native changes must also provide equivalence evidence against the frozen A8
   baseline where the migrated surface already has an oracle.
3. Materialize legacy portable payloads only into a staging directory for
   comparison or artifact validation:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

4. Keep the frozen 1.x source and baseline read-only during ordinary 2.x work.
   The immutable CI surface includes the versioned baseline directory,
   `qiongli-1x-baseline-plan.json`, `baseline-plan.schema.json`,
   `baseline-manifest.schema.json`, and `oracle-fixture.schema.json`. Run the
   applicable legacy validators as compatibility evidence, not as a production
   dependency:

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
```

5. Route any 1.x security or release-breakage exception to
   `release/1.x-python` under the PR-only policy above. Do not use `dev` as a
   feature-bearing 1.x release source.
6. Do not publish a 2.x alpha until the B1 release-tooling work explicitly
   supports alpha syntax, native artifacts, channels, acceptance, and rollback.

## Stable Release Rule

The accepted `v1.19.0-beta.1` tag is the final planned feature-bearing
Python-led 1.x beta. `main` remains the legacy stable source, but no routine 1.x
feature or release-candidate work should move there. An exceptional 1.x release
requires the maintenance decision, PR evidence, forward-port/equivalence
evidence, and release gates defined above; do not bypass the current release
automation's branch checks.

The 2.x stable and prerelease rules are established on `2.x` as native release
tooling lands. Shared Skillsplace entries advance only after the corresponding
native release gates and artifact acceptance pass.
