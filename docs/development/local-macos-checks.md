# Local macOS checks

macOS checks run manually on the maintainer's Mac (ADR 0230). Linux and Windows
remain on Actions. Keep the source commit, commands, logs and generated receipts;
an absent or failed local run is pending evidence. No self-hosted runner is needed.

## Development

Use the affected checks from `CONTRIBUTING.md`. For native CLI changes, run from
`packages/qiongli-native/` to use pinned Rust 1.97.0:

```bash
set -euo pipefail
check_out=$(mktemp -d /private/tmp/qiongli-macos-check.XXXXXX)
git rev-parse HEAD > "$check_out/source-commit.txt"
git status --short > "$check_out/worktree-status.txt"
cargo fmt --all --check 2>&1 | tee "$check_out/fmt.log"
cargo test --workspace --exclude qiongli-ui --all-targets --no-default-features --locked \
  2>&1 | tee "$check_out/tests.log"
cargo clippy --workspace --exclude qiongli-ui --all-targets --no-default-features --locked -- -D warnings \
  2>&1 | tee "$check_out/clippy.log"
```

Dirty-source logs are development evidence only. A release requires the exact
clean commit. Reuse passing checks while their inputs remain unchanged.

For retained legacy/CTR diagnostics, use the existing `tests.test_ctr_201_*`
inventory modules and `tests.test_release_version_contract`,
`tests.test_native_release_dry_run`, `tests.test_release_note_versions` from the
repository root. Retain their real failures. Checkout bootstrap coverage uses
`tests.test_bootstrap_qiongli` and `tests.test_install_qiongli`; disposable
fixtures must not replace the user's HOME, Plugin sources or installed CLI.

## Native release: local Mac plus Actions

All commands refer to one reviewed version and clean source commit. Build and
test first; tag synchronization, draft upload and publication require their own
authorization. The following example uses the compatible 2.0.1 main candidate;
do not version-label the development v2 migration as that patch.

1. On clean `main` (stable) or `2.x` (prerelease), qualify macOS through the
   existing owner into a new directory outside the checkout:

   ```bash
   bash scripts/release_ready.sh --version 2.0.1 --cli-github \
     --staging-dir /private/tmp/qiongli-2.0.1-macos
   ```

   This produces `assets/release-manifest.json`, hashes, archives and local
   install receipts. A prior commit's receipt cannot qualify a changed candidate.

2. After authorized source synchronization, the ordinary **Native CLI distribution**
   run automatically qualifies Linux/Windows. Download its two successful target
   artifacts into a fresh parent, then copy the macOS assets beside them:

   ```bash
   gh run download <run-id> --repo jxpeng98/qiongli --name x86_64-unknown-linux-gnu \
     --dir /private/tmp/qiongli-2.0.1-targets/x86_64-unknown-linux-gnu
   gh run download <run-id> --repo jxpeng98/qiongli --name x86_64-pc-windows-msvc \
     --dir /private/tmp/qiongli-2.0.1-targets/x86_64-pc-windows-msvc
   cp -R /private/tmp/qiongli-2.0.1-macos/assets \
     /private/tmp/qiongli-2.0.1-targets/aarch64-apple-darwin
   ```

   Confirm the run is successful at the intended commit. Assembly also rejects
   mixed source/version, missing targets and substituted assets.

3. Assemble and qualify the complete packet **on the Mac**:

   ```bash
   python3 tooling/scripts/native_release_assets.py assemble \
     --root /private/tmp/qiongli-2.0.1-targets \
     --out /private/tmp/qiongli-2.0.1-combined/assets \
     --version 2.0.1 --commit "$(git rev-parse HEAD)"
   python3 tooling/scripts/native_release_assets.py qualify-macos \
     --root /private/tmp/qiongli-2.0.1-combined/assets \
     --out /private/tmp/qiongli-2.0.1-local-install \
     --version 2.0.1 --commit "$(git rev-parse HEAD)"
   ```

   The second command installs the actual combined npm/wheel, tests both bundled
   Plugins, stages Cargo, runs its dry-run and installs its actual archives.
   It writes `install-check.json` in the requested output and embeds a
   `local_macos` receipt in the packet manifest only after success. It regenerates
   `SHA256SUMS`. Keep those receipts and Cargo logs. Shared temporary ancestors
   remain disallowed for Plugin exports; the probe owner uses private checkout
   fixtures. No real Host registration occurs.

4. Under separate upload authority, create a **draft** Release at the existing
   immutable tag (`gh release create --verify-tag --draft --notes-file ...`).
   Upload exactly the filenames in `release-manifest.json`'s `artifacts` plus
   `release-manifest.json` and `SHA256SUMS`. Do not upload the installation work
   directories or use `--clobber`. Do not publish the draft from the web UI.

5. Dispatch the existing authorized publisher at that tag:

   ```bash
   gh workflow run release-automation.yml --ref v2.0.1 \
     -f mode=post -f tag=v2.0.1 -f create_release=true
   ```

   It verifies the local Mac receipt, checks the same combined packet on
   Linux/Windows and checks all three wheel metadata records on Linux. Successful
   run receipts must match the manifest digest. Only then can it publish the
   unchanged draft and dispatch the existing npm/PyPI/Cargo jobs. Missing local
   evidence or a green build-only run cannot substitute for this handoff.

6. After publication, perform macOS public-download and registry installation
   checks locally. For Cargo, install the exact version into a new private root
   (`cargo install qiongli --version <version> --locked --root <root> --bin qiongli
   --bin ql`) and run `scripts/native_registry_install_check.py --cargo-only
   --cargo-installed <root>/bin/qiongli --version <version> --out-dir <new-output>`.
   Linux/Windows public Cargo checks remain automatic. Publication success does
   not mark the unrun macOS public checks or live Host session as passed.

## Retained Desktop and Community Alpha

Use [local Desktop development and packaging](local-desktop-build.md), including
its package acceptance, `pnpm desktop:macos:acceptance:open`, signing and update
journeys. For automated receipt collection without opening the App, use its
underlying `native_packaged_product_acceptance` Rust example. Retain the
`accepted-ad-hoc-nonpublishing` status, exact `product_source_commit`,
`publication_allowed: false`, and every individual check; do not infer them
from a successful build. Capacity probes remain explicitly scoped.

The removed hosted macOS steps remain local obligations when those features are
being qualified: `native_desktop_package`, missing-production-credential refusal,
`macos_native_sign_notarize.sh --test-only-ad-hoc` / `--community-alpha`,
`codesign` and mounted-DMG verification, `macos_native_update_journey.sh`, and
`native_candidate_acceptance`. Keep their existing preview and permission rules.

The retained **Native Community Alpha Promotion** workflow now produces only
Linux/Windows target receipts and archives. Use the existing
`native_community_alpha_promotion target` owner on the Mac, download the two
matching remote target sets, and run its `aggregate` mode with **all three**
directories and exact source. The subsequent
`native_community_alpha_release authorize-candidate` operation still needs its
separate publication decision. Automatic aggregation/promotion has been removed;
two target results do not establish three-target acceptance.

## Remote rollout

Local merges do not update GitHub. The September 15 ruleset read showed
`Rust native foundation (macOS)` still required by ruleset 18800504. At authorized
synchronization, remove only that retired required context while keeping the
other checks and PR/ref protections. Carry the workflow changes to any maintained
remote branch that still contains macOS jobs; changes on `main` do not rewrite
`release/1.x-python` or historical tags.
