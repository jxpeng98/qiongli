# Release Branch Policy

This repository uses `dev` for active development and keeps `main` as the stable release branch.

## Branch Roles

| Branch | Role | Allowed changes |
|--------|------|-----------------|
| `dev` | Active development and integration | Features, fixes, prerelease plugin packaging work, docs, tests, and CI hardening. |
| `main` | Stable release source | Release-prep commits, stable tags, postflight acceptance receipts, and emergency fixes that are ready to publish. |

Open normal pull requests against `dev`. Merge `dev` into `main` only when the next release candidate has passed the release gates and the plugin package is ready for a stable release.

## Official Plugin Linkage

The official plugin marketplace entry should point at the stable repository identity:

- Repository: `https://github.com/jxpeng98/qiongli`
- Codex plugin catalog: `.agents/plugins/marketplace.json`
- Codex manifest: `plugins/qiongli/.codex-plugin/plugin.json`
- Claude Code marketplace: `.claude-plugin/marketplace.json`
- Claude Code manifest: `plugins/qiongli/.claude-plugin/plugin.json`
- Gemini extension manifest: `plugins/qiongli/gemini-extension.json`

The official plugin marketplace should track `main` and release tags, not `dev`. Use `dev` for local marketplace testing and prerelease validation before the official entry is updated.

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

4. For prerelease packaging checks, build marketplace artifacts from the intended tag:

```bash
python3 scripts/build_marketplace_artifacts.py --tag v0.7.0-beta.2 --dist-dir dist
```

5. Merge to `main` only after CI, install checks, and release preflight pass.

## Stable Release Rule

Only `main` should create stable release tags and official marketplace artifacts. The release automation enforces this by requiring publish mode to run from the primary branch. Keep beta and release-candidate work on `dev` until it is ready to become a stable release.
