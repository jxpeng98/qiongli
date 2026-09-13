# Implementation Plan — Qiongli 2.0.0-alpha.5 release

> Superseded on 2026-09-03. Steps 4–7 were intentionally not run because the
> maintainer classified Alpha 5 as an internal candidate and resumed 2.x
> development.

## 1. Freeze the release identity

- Record clean synchronized base `B`; confirm Alpha 5 has no local/remote tag or
  GitHub release.
- Create `build/qiongli-2-alpha5` from `2.x`.
- Run `python3 scripts/sync_versions.py 2.0.0-alpha.5`.
- Regenerate `qiongli-core.lock.json` with the existing Rust example and
  `QIONGLI_NATIVE_SOURCE_COMMIT=B`.
- Add `tooling/release/v2.0.0-alpha.5.md` and update `CHANGELOG.md` with the
  limited public tester scope and explicit nonclaims.

## 2. Validate and merge the freeze PR

Run the existing owners closest to the changed files:

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
  --root . --tag v2.0.0-alpha.5

.venv/bin/python tooling/scripts/check_native_release_literals.py
.venv/bin/python scripts/validate_capability_contract.py
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli-content --locked
cargo test --manifest-path packages/qiongli-native/Cargo.toml \
  -p qiongli --test embedded_pack --locked
.venv/bin/python tooling/scripts/update_program_roadmap.py --check
.venv/bin/python .trellis/scripts/task.py validate \
  .trellis/tasks/09-01-release-qiongli-2-alpha5
git diff --check
```

- Commit, push, open one PR against `2.x`, wait for the required checks, and
  merge the reviewed exact head.

## 3. Qualify the merged source

- Synchronize clean local `2.x` and record its exact head as `S`.
- Run `./scripts/release_ready.sh --version 2.0.0-alpha.5 --skip-bump
  --skip-note-gen --staging-dir <fresh-private-temp>`.
- Explicitly dispatch full Native CI on `2.x`; require every target-native and
  release-acceptance job to succeed with `head_sha=S` and record run `N`.
- Use the resulting promotion or explicitly dispatch it with `S` and `N`.
  Require the five-asset exact candidate `C` and record run/attempt `P`.
- Download `C` into a fresh private directory and verify its canonical receipt,
  target order, sizes, SHA-256 values, source/version, and candidate-set digest.

## 4. Authorize and sign offline

- Confirm the existing release private key is securely exported locally before
  requesting authorization. If it is unavailable, stop here without changing
  public state.
- Dispatch Community Alpha promotion for `S` and `N` with publication
  authorization requested; approve only the exact protected-environment
  deployment and download authorization `A` from the same run.
- In a clean checkout at `S`, run the existing release example `prepare`,
  `authorize`, and `verify --require-authorization` against `C`, the checked-in
  authority, Cargo.lock, and `A`.
- Unset `QIONGLI_ALPHA_RELEASE_PRIVATE_KEY_HEX` immediately after `prepare`.

## 5. Publish the immutable prerelease

- Create draft `v2.0.0-alpha.5` targeting `S`, upload the verified release
  directory, and use its bilingual generated notes.
- Inspect draft tag target, prerelease flags, exact asset names, byte sizes, and
  GitHub-reported SHA-256 digests. Do not publish a partial or extra set.
- Mark the same draft public and prerelease without replacing any byte.

## 6. Verify from the public boundary

- Download all public release assets into a second fresh private directory.
- Compare the public inventory and digests to the signed release directory;
  run the existing offline verifier with required authorization.
- Extract the downloaded macOS ZIP and run `Qiongli --startup-check` with a
  disposable home. Confirm the same-source Windows and Linux promotion startup
  jobs passed.
- Confirm `gh release view` reports non-draft prerelease `v2.0.0-alpha.5`
  targeting `S` and that every public URL resolves.

## 7. Close evidence and stop

- Add one path-redacted Alpha 5 acceptance receipt and update only ledger states
  supported by the recorded evidence; regenerate the current program index.
- Validate the receipt, roadmap, task, public asset inventory, and diff.
- Commit the evidence-only closeout, open and merge its PR to `2.x`, archive the
  Trellis task, record the session, and leave local `2.x` clean and synchronized.
- Stop further roadmap development after the public test release is verified.

## Stop conditions

- Do not request protected authorization before the offline key is available.
- Do not reuse Alpha 3/4 candidates, stale CI, expired authorization, or a
  candidate from another run attempt.
- Do not publish if any platform, closed-inventory, signature, checksum,
  authorization, or public-download check fails.
- Do not rotate keys, add automatic updates, announce, or close unsupported
  roadmap work as part of this task.
