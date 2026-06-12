# Release Branch Policy

This repository uses `dev` for active development and keeps `main` as the stable release branch.

## Branch Roles

| Branch | Role | Allowed changes |
|--------|------|-----------------|
| `dev` | Active development, integration, and beta release source | Features, fixes, prerelease plugin packaging work, beta release-prep commits, beta tags, docs, tests, and CI hardening. |
| `main` | Stable release source | Stable release-prep commits, stable tags, postflight acceptance receipts, and emergency fixes that are ready to publish. |

Open normal pull requests against `dev`. Beta releases may publish from `dev` after the release gates pass. Merge `dev` into `main` only when the next release candidate has passed the release gates and the plugin package is ready for a stable release.

## Official Plugin Linkage

The official public marketplace entry lives in `jxpeng98/skillsplace` and should point at the stable generated Qiongli plugin payload:

- Marketplace repository: `https://github.com/jxpeng98/skillsplace`
- Qiongli repository: `https://github.com/jxpeng98/qiongli`
- Stable Codex artifact: `qiongli-core-codex-plugin-<tag>.tar.gz`
- Stable Claude Code artifact: `qiongli-core-claude-plugin-<tag>.tar.gz` or `.zip`
- Stable Gemini artifact: `qiongli-gemini-extension-<tag>.tar.gz`
- Stable generated payload root: `plugins/qiongli/`

The stable Skillsplace catalog entries should track `main` and stable release tags, not `dev`. Use `dev` for local plugin packaging tests and prerelease validation before the shared marketplace entry is updated. This repository no longer carries Codex or Claude marketplace catalog files; it owns the plugin manifests and materializes the release payload from canonical source.

Prerelease tags publish the `qiongli-next` testing channel instead of the full stable marketplace matrix. The generated next artifacts are:

- `qiongli-next-codex-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.tar.gz`
- `qiongli-next-claude-plugin-<tag>.zip`
- `qiongli-next-claude-desktop-skill-core-<tag>.zip`

The `qiongli-next` Codex and Claude Code plugin artifacts install only the `core/complete` skill package and keep the bundled zero-dependency Node literature MCP runtime. They do not publish subject plugin variants. The Claude plugin ZIP contains the same plugin payload as the Claude tarball for upload flows that reject `.tar.gz`. Claude Desktop testing uses the focused core ZIP plus the separate `qiongli-literature-provider-<version>.mcpb` release asset. The Skillsplace catalog may expose a separate `qiongli-next` entry for beta testing while stable `qiongli` and subject entries continue to point at stable artifacts.

This repository does not track stable or beta plugin payload directories. `plugins/qiongli/`, `plugins/qiongli-next/`, `packages/qiongli-plugin/`, and `packages/qiongli-next-plugin/` are generated shapes. Change `content/workflow/`, `content/distribution/plugins.yaml`, or `tooling/scripts/build_plugin_artifacts.py`, then materialize into a staging directory for validation.

## Development Flow

1. Start feature and packaging work on `dev`.
2. Materialize the portable skill package into a staging directory before artifact validation:

```bash
python3 scripts/materialize_distribution_payloads.py --target all --out /tmp/qiongli-dist --force
```

3. Run the normal validation set on `dev`:

```bash
python3 scripts/validate_research_standard.py --strict
python3 -m unittest discover -s tests -v
```

4. For prerelease packaging checks, build plugin artifacts from the intended tag:

```bash
python3 scripts/build_plugin_artifacts.py --tag v0.7.0-beta.2 --dist-dir dist
```

For beta tags this command should produce only `qiongli-next` core artifacts; for stable tags it should preserve the full `qiongli`, `qiongli-core`, subject, Desktop subject, and Gemini artifact structure.

5. Publish beta releases from `dev` when the release-prep commit and preflight evidence are ready:

```bash
./scripts/release_automation.sh publish --version 0.8.0b1 --skip-bump --from-tag v0.7.0-beta.2
```

`publish` pushes the release-prep commit first and waits for the required branch checks (`CI` and
`Checkout Install Check`) on that same commit before it creates or pushes the beta tag. The tag then
triggers registry publish workflows, and the GitHub prerelease is created only after those tag
publish workflows pass.

6. Merge to `main` only after CI, install checks, and release preflight pass for a stable release candidate.

## Stable Release Rule

Only `main` should create stable release tags and public plugin artifacts, and the shared Skillsplace entry should only be advanced after those release gates pass. The release automation enforces stable publish mode from the primary branch and waits for required branch checks before tag creation. Prerelease tags may publish from `dev`; publish mode first gates on `dev` CI/checks, then creates the beta tag, then waits for tag publish workflows. Keep release-candidate work on `dev` until it is ready to become a stable release.

Beta is not mandatory for every stable release. Use beta when the release changes high-risk surfaces such as release automation, package payloads, installers, package metadata, CI, or publish workflows. Low-risk docs and small fixes may publish directly from `main` as stable. When stable ships without a matching beta, npm `latest` advances and npm `next` intentionally remains on the previous beta; `next` means latest prerelease validation build, not a channel that must always be newer than stable.
