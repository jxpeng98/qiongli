# Qiongli 2.0.0-alpha.4 Private Candidate Receipt

Status: accepted as private exact-source three-target test evidence;
`publication_allowed=false`

Date: September 1, 2026

Target branch: `2.x`

## Accepted outcome

Qiongli `2.0.0-alpha.4` has one verified private tester candidate for the
closed target set of macOS arm64, Windows x86_64, and Linux x86_64. All five
candidate assets were rebuilt from the same merged product source, passed
target-native packaging and startup checks, and were aggregated without
entering the publication-authorization environment.

This is an authenticated GitHub Actions artifact for private testing. It is
not a Git tag, GitHub Release, update-channel entry, package-manager release,
public download, or announcement. Installation and replacement remain manual.

## Exact evidence chain

| Evidence | Accepted value |
|---|---|
| Product source `S` | `48b8302cb05d8d28d0af7cd845529ab4b0eaae7d` |
| Preparation review | [PR #169](https://github.com/jxpeng98/qiongli/pull/169), merged into `2.x` as `S` |
| Local release readiness | `release_ready.sh` completed for `2.0.0-alpha.4` at `S` in dry-run mode; `publication_allowed=false` and `publication_performed=false` |
| Native CI `N` | [run 33460271739, attempt 1](https://github.com/jxpeng98/qiongli/actions/runs/33460271739): `success`, `head_sha=S` |
| Promotion `P` | [run 33461498315, attempt 1](https://github.com/jxpeng98/qiongli/actions/runs/33461498315/attempts/1): `success`, `head_sha=S`, with preflight bound to `N` |
| Candidate artifact | ID `9783762095`; `qiongli-community-alpha-candidate-48b8302cb05d8d28d0af7cd845529ab4b0eaae7d` |
| Artifact archive | 53,010,163 bytes; GitHub digest `sha256:dffd9509778f93c0a8932c511ade5f477c2224492fef762710dc668167954c9a` |
| Retention | Created `2026-09-01T02:30:45Z`; expires `2026-09-04T02:30:41Z` |
| Candidate status | `fresh-three-target-nonpublishing-candidate`; `publication_allowed=false` |
| Candidate-set digest | `1fa699c4617308e6c54b1c4b02f433e710d4103e1658000f4f643fa48cb3a069` |
| Candidate receipt file SHA-256 | `f94fee8efe412e7f3f65ea298ceb47e4f272db94a082c97f5dd52b163482fcdb` |
| Publication authorization | Job `99716385138` was skipped with no steps; the protected environment was not entered |

The later evidence commit `E` that adds this receipt is not the product source
and must never be used as the identity of these candidate bytes. The candidate
remains bound only to `S`.

## Candidate assets

| Target | Asset | Bytes | SHA-256 |
|---|---|---:|---|
| macOS arm64 | `Qiongli-2.0.0-alpha.4-macOS-arm64.zip` | 10,403,833 | `bf7bb501cd5c806fbe5b76b6878dffab0620562832bcb2fb393fab9842aa7963` |
| macOS arm64 | `Qiongli-2.0.0-alpha.4-macOS-arm64.dmg` | 10,522,474 | `0ecef5c9851349a16530c437efb8b3951dab4b6f33117ed0479e2f8954ad27d8` |
| Windows x86_64 | `Qiongli-2.0.0-alpha.4-Windows-x64.zip` | 27,157,324 | `d2d39a35f0e98706b5c56163eb1ddad106cb322d8372c90cb22a7bd1e2f2f2fd` |
| Linux x86_64 | `Qiongli-2.0.0-alpha.4-Linux-x64.AppImage` | 11,672,056 | `0a75ece173a12053ea219decc674543290ddabf7b2c8b85334962df67d03f2db` |
| Linux x86_64 | `Qiongli-2.0.0-alpha.4-Linux-x64.zip` | 38,552,761 | `4b99de8dc193e5460093815dc8a0cca6f8bd83d036bd8955d3b648faa5f2d150` |

The candidate receipt contains exactly three ordered targets and five public
assets. Every target records `fresh-exact-source-target-native-build`,
`raw_ci_artifact_reused=false`, and `publication_allowed=false`.

## Verification receipt

The aggregate was downloaded to a private temporary directory before expiry;
the local path is intentionally omitted. Standard file metadata and
`shasum -a 256` matched every declared asset byte count and digest.

The downloaded public assets, target receipts, manifests, and promotion
records were then reconstructed into the repository's existing three-target
promotion inputs. The exact-source `native_community_alpha_promotion`
aggregator accepted all canonical records, target identities, file inventories,
sizes, and SHA-256 values and reproduced the same candidate-set digest. A
recursive byte comparison between the downloaded and independently
reaggregated candidate directories reported no difference.

Native CI `N` succeeded for the native boundary, Lite compatibility, Linux,
macOS and Windows Rust foundations, all three non-publishing desktop packages,
all three installation lifecycles, and packaged macOS product control.
Promotion `P` then succeeded for exact-head preflight, all three fresh target
rebuilds, and non-publishing aggregation.

## Trust boundary and non-claims

- macOS artifacts are ad-hoc signed and not notarized; per-app **Open Anyway**
  may be required.
- The Windows portable ZIP is unsigned and may be warned about or blocked by
  host security policy.
- The Linux artifacts require the runtime facilities declared by the package.
- No production signing, notarization, Authenticode, public-download,
  automatic-update, package-manager, Stable, or milestone-exit claim is made.
- No tag, GitHub Release, public upload, update-channel mutation, publication
  authorization, or announcement was created or requested.
- `GOV-413` remains `blocked`. `GOV-417`--`GOV-418`, `PLT-401`--`PLT-408`, and
  `SEC-401`--`SEC-405` remain `proposed`; this candidate changes none of those
  M1 states.
- Alpha 3 evidence remains historical and does not qualify Alpha 4. Alpha 1
  remains the latest public Qiongli 2 prerelease.

Expiry, source movement, or any asset change invalidates reuse of this
candidate. A replacement must repeat the exact-source Native CI, three-target
promotion, download, and verification chain; receipts must not be rewritten to
make older bytes appear current.
