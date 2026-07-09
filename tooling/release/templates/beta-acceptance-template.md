# Release Acceptance Receipt — {{TAG}}

- Date: {{DATE}}
- Release Tag: {{TAG}}
- Commit: {{COMMIT}}
- CI Status: {{CI_STATUS}}

## Component Version Map

{{COMPONENT_VERSION_MAP}}

## Publish Preconditions (Preflight)

- [ ] `python3 scripts/validate_research_standard.py --strict`
- [ ] `python3 -m unittest tests.test_orchestrator_workflows -v`
- [ ] `./scripts/run_beta_smoke.sh`

{{SUBJECT_RUNTIME_EVIDENCE}}

## Publish Actions

- [ ] `./scripts/release_automation.sh publish --tag {{TAG}} ...`
- [ ] Release automation pushed the release branch and tag.
- [ ] Tag-triggered PyPI and npm publish workflows completed.

## Post-Release Verification

- [ ] Remote branch/tag consistency verified.
- [ ] GitHub Actions branch checks and tag publish workflows verified for release commit.
- [ ] GitHub Release page exists and notes are attached.
- [ ] Rollback path validated (`tooling/release/rollback.md`).

## Collaboration Validation (Codex / Claude)

- [ ] `parallel` Codex/Claude path works or records unavailable workers safely.
- [ ] `task-run` stage routing works with capability map fallback.
- [ ] Profile overrides (`--draft-profile` / `--review-profile` / `--triad-profile`) are effective.

## Sign-off

- Owner:
- Reviewer:
- Notes:
