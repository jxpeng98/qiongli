# Qiongli REL-913 Installation Lifecycle Acceptance

Status: accepted at Acceptance tier

Date: August 30, 2026

Target branch: `2.x`

Implementation and recovery pull requests: `#156`, `#157`, `#158`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Initial implementation merge | `6b08c9eb044459b36b6cf539ffed1a775120bc16` |
| Windows acceptance-path merge | `328ad7cf14b74047fbe22bdfd00b27010a1b30e9` |
| Windows state-root fix | `bad1677b8987b5bd99ff374239c0442d7d926b11` |
| Accepted product source | `ca0a4a5d530cf53c14d51968387a2aefe19dc630` |
| Root-fix exact-head Slice CI | run `33310061192`: success |
| Exact-source Native Acceptance | run `33310992152`: success |
| Three-target promotion | run `33311931096`, attempt `1`: success |
| Aggregate candidate artifact | `qiongli-community-alpha-candidate-ca0a4a5d530cf53c14d51968387a2aefe19dc630` |
| Aggregate candidate artifact ID | `9732482162` |
| Candidate-set content SHA-256 | `00ddd1cf44212f308f759e73a813d9bb71729fd6524f878a0bb216a50cf95ee7` |
| Candidate receipt file SHA-256 | `4137b25f09c603b6cfc6132164d67e7e5b700bb9582cd41c0b917ab682538151` |

The explicit Acceptance run and the downstream promotion are distinct. The
promotion rebuilt every target from the same accepted source and used only
inputs uploaded by promotion attempt `33311931096/1`.

## Recovery and root cause

The first explicit Acceptance run `33306251259` and the retry
`33307884631` failed only the Windows Candidate job at
`candidate-acceptance-mcp-call-invalid`. A bounded diagnostic run
`33308791893` classified the underlying failure as
`candidate-acceptance-mcp-config-unavailable` after Managed Skills had been
installed.

Managed-content materialization created the shared `config/v2` directory
directly. On Windows that inherited a broad ACL, while `GlobalSettingsStore`
correctly requires its state root to be owner-only. PR `#158` routed directory
preparation through the existing configuration owner before registry mutation
and added one cross-platform regression test. Exact-head run `33310061192`
then passed Linux, macOS, and Windows before the fix was merged.

## Exact-source workflow result

Native Acceptance run `33310992152` completed successfully:

| Job | Result |
|---|---|
| Native `2.x` change boundary | success |
| R2 Lite compatibility, Linux | success |
| Rust native foundation, Linux | success |
| Rust native foundation, macOS | success |
| Rust native foundation, Windows | success |
| Candidate installation lifecycle, Linux | success |
| Candidate installation lifecycle, macOS | success |
| Candidate installation lifecycle, Windows | success |
| Packaged product control acceptance, macOS | success |
| Non-publishing desktop package, Linux AppImage | success |
| Non-publishing desktop package, macOS application | success |
| Non-publishing desktop package, Windows portable | success |
| Dispatch exact Community Alpha promotion | success |

Promotion run `33311931096` also completed successfully:

| Job | Result |
|---|---|
| Verify exact `2.x` head | success |
| Fresh Community Alpha target, macOS arm64 | success |
| Fresh Community Alpha target, Windows x86_64 | success |
| Fresh Community Alpha target, Linux x86_64 | success |
| Aggregate non-publishing three-target candidate | success |
| Authorize exact Community Alpha candidate | skipped |

The skipped authorization job confirms that this acceptance did not enter the
protected publication environment.

## Candidate lifecycle receipts

| Target | Artifact ID | Receipt SHA-256 | Candidate archive SHA-256 | Result |
|---|---:|---|---|---|
| Linux x86_64 | `9732083012` | `59f1b8e10f865bd1ccbe9f6cd112947342498ce8cef4833aec462e8d89bab196` | `5584f664c397c1a8ac9d173d4daefe116c0e658235bd1f908cc7b1ac651f81b0` | passed |
| macOS arm64 | `9732047765` | `e6e0b66ef8df42bdd7f362efaa9bfc2b8496efe93b7801e1a34bc015f3b48bd3` | `24d0aa6440d93e4ba49f05ee3c1d3164ab7640ce1bc01b45f534c675cbeac5b7` | passed |
| Windows x86_64 | `9732138774` | `b4b3d9f3113c52ce2952002127823c43fb8daa19108bcf054b4399356d2f9f4d` | `ad4a392f52411db2a66104268e5d91080ca9d87ad63b3b8558610a4683a602fa` | passed |

Every receipt binds source `ca0a4a5d...c630`, reports
`publication_allowed: false`, and passes clean install, version, Lite MCP,
embedded Skills, Codex-local lifecycle, Claude-Code-local lifecycle, failure
compensation, digest/approval rejection, verification, uninstall, and exact
preservation of project, global v2, unrelated, and unmanaged Host state.

## Packaged product and update evidence

| Evidence | Artifact ID | File SHA-256 | Result |
|---|---:|---|---|
| Packaged-product receipt | `9732113524` | `194ccdcfd875abd7a73015c609a793d6ede47efb5eb5a95e2661155c6e8d0dc0` | `accepted-ad-hoc-nonpublishing` |
| macOS native update receipt | `9732079042` | `c443f9ff8b3dccd02766cc0b92e48d30fc07579ae165f8bb8f17484ae20d49c3` | `passed-test-only` |

The packaged-product receipt binds source `ca0a4a5d...c630` and passes:

- Codex and Claude install/verify/remove plus Plugin reconciliation/removal;
- Skill materialization, verification, refresh, and standalone all-target checks;
- Lite MCP self-test;
- project App/CLI/library/Full MCP parity; and
- connected Graph App/CLI/Full MCP parity across three projects and restart.

The macOS update receipt proves a healthy atomic replacement from the derived
`2.0.0-alpha.2` predecessor fixture to `2.0.0-alpha.3`, failed-health rollback
to that predecessor, last-known-good restoration, transaction cleanup, and
project/global/unmanaged-state preservation.

## Aggregate candidate inventory

| Target | File | Bytes | SHA-256 |
|---|---|---:|---|
| macOS arm64 | `Qiongli-2.0.0-alpha.3-macOS-arm64.zip` | 10,400,433 | `86e387f0ad85b7b6e832422561d655649300e28b698f26287ff98d176f01d9ef` |
| macOS arm64 | `Qiongli-2.0.0-alpha.3-macOS-arm64.dmg` | 10,518,378 | `415b8598643591e14bf5589685daf1c4cae8fd6abcf54b1627e7fb086d78003e` |
| Windows x86_64 | `Qiongli-2.0.0-alpha.3-Windows-x64.zip` | 27,121,996 | `b73414197014619cf553c6f3b4ae7a05459acd14bb353c8091a8a63d7eb7e6c0` |
| Linux x86_64 | `Qiongli-2.0.0-alpha.3-Linux-x64.AppImage` | 11,684,344 | `2838c71092f62a88dfd73e8dc26184b9f0fb0d44d7c36536affa2f3c1a1bbee9` |
| Linux x86_64 | `Qiongli-2.0.0-alpha.3-Linux-x64.zip` | 38,547,009 | `f398ce60b0c5c7b26eb8e04cac7bdcffd85d8a306152a2e729903e6450beecac` |

The aggregate receipt reports `fresh-three-target-nonpublishing-candidate`,
`raw_ci_artifact_reused: false`, and `publication_allowed: false` for every
target.

## Downloaded-byte verification

The candidate receipts, packaged-product receipt, macOS update receipt, and
aggregate candidate were downloaded by exact run and artifact name into a
private temporary directory. Local verification confirmed:

- all three candidate receipts bind the accepted source and pass the complete
  recorded lifecycle/preservation check set;
- the packaged-product and update receipt file digests above match downloaded
  bytes;
- sorted compact canonical candidate content hashes to
  `00ddd1cf...95ee7` with no trailing newline;
- all five aggregate asset sizes and SHA-256 values match downloaded bytes;
- all target evidence file digests match the aggregate receipt; and
- source, version, build-attempt URL, target order, and non-publication policy
  agree across the aggregate and three target receipts.

## Nonclaims

The Candidate receipts exercise isolated Codex and Claude Code Host layouts,
not installed external client binaries; their real-client gates correctly say
`not-run` because no external clients were supplied to this workflow. The
separate PILOT-903 receipt remains the external Codex-model pilot evidence.

This acceptance does not claim a physical clean machine, an interactively
displayed desktop window, a published predecessor binary, network update
selection, Developer ID/notarization, Authenticode/timestamping, production
signing, package-manager behavior, public download verification, Stable
eligibility, a Git tag, a GitHub Release, or publication. The aggregate
artifact's internal `public/` directory is packaging layout only and was not
published.
