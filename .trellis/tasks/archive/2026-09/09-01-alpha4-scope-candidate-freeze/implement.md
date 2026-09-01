# Implementation Plan — Alpha 4 private candidate freeze

## 1. Prepare the reviewed Alpha 4 identity

- Confirm local `2.x` is clean and synchronized with `origin/2.x`; record base
  commit `B` and confirm no `v2.0.0-alpha.4` tag or GitHub Release exists.
- Create `build/alpha4-private-candidate` from that exact base.
- Run `python3 scripts/sync_versions.py 2.0.0-alpha.4`.
- Regenerate `qiongli-core.lock.json` with the existing Rust example and
  `QIONGLI_NATIVE_SOURCE_COMMIT=B`.
- Add `tooling/release/v2.0.0-alpha.4.md` and update `CHANGELOG.md`.
- State the closed three-target set, manual replacement model, short artifact
  retention, `publication_allowed=false`, and all public/M1/Stable non-claims.
- Do not change any M1 ledger task state.

## 2. Validate the preparation diff

Run the smallest existing checks that own the changed contracts:

```bash
.venv/bin/python -m unittest \
  tests.test_sync_versions \
  tests.test_release_version_contract \
  tests.test_release_note_versions \
  tests.test_release_automation \
  tests.test_release_upload_assets \
  tests.test_release_local_install_check \
  tests.test_branch_policy -v

bash scripts/verify_release_tag_version.sh \
  --root . --tag v2.0.0-alpha.4

.venv/bin/python tooling/scripts/check_native_release_literals.py
.venv/bin/python scripts/validate_capability_contract.py

cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-content --locked

cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test embedded_pack --locked

.venv/bin/python tooling/scripts/update_program_roadmap.py --check
.venv/bin/python .trellis/scripts/task.py validate \
  .trellis/tasks/09-01-alpha4-scope-candidate-freeze
git diff --check
```

If these checks expose an existing hard-coded release assumption, make only the
minimum owner-level correction and its focused regression test.

## 3. Review and merge the version-preparation PR

- Review the final diff for generated-file completeness and authority wording.
- Commit with `build(release): prepare 2.0.0-alpha.4 candidate`.
- Push the branch and open one PR targeting `2.x`.
- Mark it ready once, wait for the repository's required Evaluation and native
  checks, and merge only the reviewed exact head.
- Do not create a tag, Release, update entry, or publication request.

## 4. Qualify the exact merged product source

- Synchronize local `2.x` after merge, require a clean worktree, and record the
  exact remote head as product source `S`.
- Create an external temporary staging directory with `mktemp -d` and run:

```bash
./scripts/release_ready.sh \
  --version 2.0.0-alpha.4 \
  --skip-bump \
  --skip-note-gen \
  --staging-dir <temporary-directory>
```

- Explicitly dispatch `.github/workflows/native-ci.yml` on `2.x`.
- Select only a Native CI run whose `head_sha` equals `S`; wait for every
  required job to succeed and record run `N`.
- Select the automatically dispatched Community Alpha promotion whose
  `source_commit` equals `S` and whose qualifying run is `N`; wait for success
  and record run/attempt `P`.
- Confirm the `Authorize exact Community Alpha candidate` job is skipped and no
  publication environment was entered.

Any source movement or failed required job invalidates the attempt. Fix through
a reviewed source change if necessary, then restart from the new exact `S`.

## 5. Verify and record the private candidate

- Download `qiongli-community-alpha-candidate-S` into a private temporary
  directory before the existing three-day retention deadline.
- Use the aggregate receipt and standard hash tooling to verify:
  - version `2.0.0-alpha.4`;
  - product source `S`, Native run `N`, and promotion attempt `P`;
  - exactly macOS arm64, Windows x86_64, and Linux x86_64;
  - canonical file inventory and non-zero sizes;
  - each file SHA-256 and the candidate-set digest;
  - `publication_allowed=false`.
- Add
  `docs/superpowers/acceptance/2026-09-01-qiongli-alpha4-private-candidate.md`
  with path-redacted values for `S`, `N`, `P`, target files, sizes, hashes,
  digest, retention, claims, and non-claims.
- Keep all unfinished M1 task states unchanged, regenerate no ledger data unless
  an independently accepted ledger change exists, and confirm the generated
  program index remains current.
- Submit the receipt as an evidence-only closeout PR, merge it, archive this
  Trellis task, update the developer journal, and finish with clean synchronized
  local `2.x`.

## Stop conditions

- Never substitute an older Alpha 3 run, partial platform result, or expired
  candidate for current evidence.
- Never request or infer publication/announcement authorization.
- Never reuse a failed candidate; produce a fresh exact-source run.
- Stop before any tag, GitHub Release, public upload, update-channel mutation,
  production signing, or announcement.
