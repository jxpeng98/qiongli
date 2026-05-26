# Release Branch Policy

This repository uses `dev` for active development and keeps `main` as the stable release branch.

## Branch Roles

| Branch | Role | Allowed changes |
|--------|------|-----------------|
| `dev` | Active development, integration, and beta release source | Features, fixes, prerelease plugin packaging work, beta release-prep commits, beta tags, docs, tests, and CI hardening. |
| `main` | Stable release source | Stable release-prep commits, stable tags, postflight acceptance receipts, and emergency fixes that are ready to publish. |

Open normal pull requests against `dev`. Beta releases may publish from `dev` after the release gates pass. Merge `dev` into `main` only when the next release candidate has passed the release gates and the plugin package is ready for a stable release.

## Official Plugin Linkage

The official public marketplace entry lives in `jxpeng98/skillsplace` and should point at the stable Qiongli plugin payload:

- Marketplace repository: `https://github.com/jxpeng98/skillsplace`
- Qiongli repository: `https://github.com/jxpeng98/qiongli`
- Plugin subdirectory: `plugins/qiongli`
- Codex manifest: `plugins/qiongli/.codex-plugin/plugin.json`
- Claude Code manifest: `plugins/qiongli/.claude-plugin/plugin.json`
- Gemini extension manifest: `plugins/qiongli/gemini-extension.json`

The Skillsplace catalog should track `main` and release tags, not `dev`. Use `dev` for local plugin packaging tests and prerelease validation before the shared marketplace entry is updated. This repository no longer carries Codex or Claude marketplace catalog files; it only owns the plugin payload and platform manifests.

## Development Flow

1. Start feature and packaging work on `dev`.
2. Keep the portable skill package synchronized before validation:

```bash
bash scripts/sync_skill_package.sh --target all
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

5. Publish beta releases from `dev` when the release-prep commit and preflight evidence are ready:

```bash
./scripts/release_automation.sh publish --version 0.8.0b1 --skip-bump --from-tag v0.7.0-beta.2
```

6. Merge to `main` only after CI, install checks, and release preflight pass for a stable release candidate.

## Stable Release Rule

Only `main` should create stable release tags and public plugin artifacts, and the shared Skillsplace entry should only be advanced after those release gates pass. The release automation enforces stable publish mode from the primary branch. Prerelease tags may publish from `dev`; postflight then checks that the beta tag commit is reachable from `dev` and queries CI on `dev`. Keep release-candidate work on `dev` until it is ready to become a stable release.
