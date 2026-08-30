# Qiongli REL-910 Provenance-Bound Tier 1 Artifacts Acceptance

Status: accepted at Acceptance tier

Date: August 30, 2026

Target branch: `2.x`

Implementation pull request: `#153`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Implementation commit | `4df813e2b9a4f16fca652ce723c423c1d1724c22` |
| Accepted product source | `cf23c6f1286fadd8b5af0f0ccfd3e2aaab09d072` |
| Exact-source Native CI | run `33287603009`: success |
| Three-target promotion | run `33288394060`, attempt `1`: success |
| Build run URL | `https://github.com/jxpeng98/qiongli/actions/runs/33288394060/attempts/1` |
| Candidate artifact | `qiongli-community-alpha-candidate-cf23c6f1286fadd8b5af0f0ccfd3e2aaab09d072` |
| Candidate artifact ID | `9725401138` |
| Candidate-set content SHA-256 | `8e4d368f7c5f4248d604240cac835a2bb73cffaea04b51dc2b55d5fec8c751ff` |
| Candidate receipt file SHA-256 | `709bddb36b741d725c12e01c6cd40470693f9ed3dee7ff5d8214567c1fbf7f5e` |

The qualifying Native CI run and the artifact-building promotion run are distinct. Every
target receipt and the aggregate candidate identify promotion attempt `33288394060/1` as
the builder invocation.

## Workflow result

Native CI run `33287603009` completed successfully for the exact accepted product source.
Its Linux, macOS, and Windows native jobs, non-publishing package jobs, packaged-product
control, Lite compatibility, and downstream promotion dispatch all passed.

Promotion run `33288394060` rebuilt all three targets from the same source and completed
successfully:

| Job | Result |
|---|---|
| Verify exact `2.x` head | success |
| Fresh Community Alpha target, macOS arm64 | success |
| Fresh Community Alpha target, Windows x86_64 | success |
| Fresh Community Alpha target, Linux x86_64 | success |
| Aggregate non-publishing three-target candidate | success |
| Authorize exact Community Alpha candidate | skipped |

The skipped authorization job confirms that candidate generation did not enter the
protected `community-alpha-publication` Environment.

## Public asset inventory

| Target | File | Bytes | SHA-256 |
|---|---|---:|---|
| macOS arm64 | `Qiongli-2.0.0-alpha.3-macOS-arm64.zip` | 10,415,416 | `6f6c1a9ad3ee5a4e3b060e4824681075e05c1442faa2e0183e6b316479bbea1e` |
| macOS arm64 | `Qiongli-2.0.0-alpha.3-macOS-arm64.dmg` | 10,537,322 | `cc4dcda2f377c898f19c5bdd3bf6f8e7bb4bb8f6963dee9fc16904fc0433ad1b` |
| Windows x86_64 | `Qiongli-2.0.0-alpha.3-Windows-x64.zip` | 27,166,028 | `1202831fa6074aa5953c38018fe27e9ad17fad2a96c19e642482ae2b8cb479fc` |
| Linux x86_64 | `Qiongli-2.0.0-alpha.3-Linux-x64.AppImage` | 11,696,632 | `7ecacd66b08bb584c676c00f8b548591fa8a0569dafcf9dd83b74271d0b849cf` |
| Linux x86_64 | `Qiongli-2.0.0-alpha.3-Linux-x64.zip` | 38,655,961 | `d4c6641a63a2a693335e005a3e2c0e46dd0fbbc038ac7dd0203e531812e2cd5f` |

The downloaded candidate contained exactly these five public files.

## Evidence inventory

| Target | Evidence file | Bytes | SHA-256 |
|---|---|---:|---|
| macOS arm64 | `qiongli-desktop-package.manifest.json` | 3,418 | `e97ed71ba8940970359012d5d7733e09dbfe6d9d0ba084c9f8492c87f91519a4` |
| macOS arm64 | `qiongli-desktop-package.receipt.json` | 699 | `2017c0144587c352b763b598cbc7ea24b82368970d36c7c27da1312c3a26b6aa` |
| macOS arm64 | `qiongli-macos-unsigned-acceptance.receipt.json` | 1,156 | `bcb440cd93427c777e4abaa8e576d387724bcb07873c0a4a2e9bbc8e88e65464` |
| macOS arm64 | `qiongli-macos-signing.receipt.json` | 2,654 | `ba0529b3f4477bd9c47674b14ff99941c2f5f7aa1e1f78f68d5964838ffd9127` |
| Windows x86_64 | `qiongli-desktop-package.manifest.json` | 3,204 | `1d2eeac7a7b9d04adee04c890050376e80507650027097b545afd7b20c844472` |
| Windows x86_64 | `qiongli-desktop-package.receipt.json` | 692 | `78ec6f35fbd9bb3d2ce2127666450f7af39b1f579debd9cdf20fad7b2a3ca3d5` |
| Linux x86_64 | `qiongli-desktop-package.manifest.json` | 3,407 | `f68ab947f30d0637f2fc875497a8aa11892fdd4a5fea0d4b4f92cd888ff59060` |
| Linux x86_64 | `qiongli-desktop-package.receipt.json` | 690 | `c28696c07d9723d637493e5f29dd359573d3c9ef856b8f101e8ea29fe80cc52c` |
| Linux x86_64 | `qiongli-linux-appimage.receipt.json` | 1,248 | `49ca8ab60262bac7ee0b1b30b195595d3e6dc461e72cb16d60ffab499633c353` |

Each target also retained its canonical promotion receipt:

| Target | Bytes | SHA-256 |
|---|---:|---|
| macOS arm64 | 2,257 | `c563006b56e79688f6b4bfb58fae001d954f909e6fa33042bd2dc3696b4d1281` |
| Windows x86_64 | 1,721 | `fa0e297d864fdf97292f67e3b5283e72757ee0d56a9b75756f1221311b11f184` |
| Linux x86_64 | 2,081 | `2970f40b39e08f8ae3cf3e65ce40b697845d0142f6d67800939f5547fabef63b` |

## Downloaded-byte verification

The aggregate artifact was downloaded by exact run and artifact name into a private
temporary directory. A dependency-free verifier confirmed:

- canonical JSON for the candidate and all three target promotion receipts;
- candidate content digest `8e4d368f...751ff`;
- source `cf23c6f...9d072`, version `2.0.0-alpha.3`, and attempt URL
  `33288394060/attempts/1` on the candidate and every target;
- ordered targets `macos/aarch64`, `windows/x86-64`, and `linux/x86-64`;
- the exact five-asset, nine-evidence-file, and three-promotion-receipt inventory;
- every recorded file size and SHA-256 against downloaded bytes; and
- `publication_allowed: false` at aggregate and target levels.

The focused branch-policy suite passed 19 tests, the closest Rust Community Alpha suite
passed 6 tests, exact-source cross-platform Actions passed, and the implementation PR's
required Slice checks passed.

## Nonclaims

This acceptance establishes fully provenance-bound artifacts, not byte-identical
reproducibility. It does not claim Developer ID/notarization, Authenticode/timestamping,
production signing, package-manager publication, lifecycle acceptance, independent public
download verification, revocation, Stable eligibility, a Git tag, or a GitHub Release.
