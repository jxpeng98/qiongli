# Technical Design — Alpha 4 private candidate freeze

## Outcome

Produce one private `2.0.0-alpha.4` tester candidate for the closed target set:

- macOS arm64;
- Windows x86_64;
- Linux x86_64.

The candidate is an expiring authenticated GitHub Actions artifact with
`publication_allowed=false`. This work changes release identity and evidence,
not product behavior. It creates no tag, GitHub Release, update-channel entry,
publication authorization, or announcement.

## Existing owners

Reuse the current owners without adding a parallel release path:

- `scripts/sync_versions.py` for version-bearing files;
- `update_qiongli_core_lock` for the deterministic embedded-content lock;
- `scripts/verify_release_tag_version.sh` for cross-file version agreement;
- `scripts/release_ready.sh` for the local native release dry-run;
- `.github/workflows/native-ci.yml` for exact-source three-platform acceptance;
- `.github/workflows/native-community-alpha-promotion.yml` for candidate
  aggregation with publication disabled;
- the existing acceptance-document pattern for the path-redacted closeout
  receipt.

No new dependency, workflow, schema, builder, installer, role, or storage
service is justified.

## Evidence flow

1. Record clean preparation base commit `B` and prepare the Alpha 4 version,
   content lock, release notes, and changelog on a short-lived branch.
2. Merge the reviewed change into `2.x`; record the clean remote head as product
   source `S`.
3. Run local release readiness against `S` without publishing.
4. Explicitly dispatch Native CI for `S`; record successful run `N` only when
   every required Linux, macOS, Windows, Lite, package, packaged-product, and
   candidate job succeeds for the same head SHA.
5. Let Native CI dispatch Community Alpha promotion for `S` with publication
   authorization left false; record successful run/attempt `P`.
6. Download candidate set `C` before its three-day retention expires and verify
   its closed inventory, byte sizes, SHA-256 values, source/version fields, and
   candidate-set digest in a private temporary directory.
7. Add an evidence-only closeout receipt at commit `E`. The receipt references
   `S`, `N`, `P`, and `C`; `E` is never represented as the source that produced
   the candidate bytes.

Each failed or stale link stops the chain. A rerun creates new evidence and
cannot inherit acceptance from an older source or candidate.

## Identity contracts

- The checked-in `qiongli-core.lock.json` keeps its existing lock-generation
  meaning: its `source_commit` records preparation base `B` used by the current
  lock procedure. It is distinct from native product source `S`.
- Native package, lifecycle, promotion, and candidate receipts must bind exact
  product source `S`, which must equal the then-current remote `2.x` head.
- Candidate version is exactly `2.0.0-alpha.4` across Cargo, Cargo.lock, Plugin
  manifests, Skill registry/workflow metadata, MCPB metadata, embedded content,
  release notes, and changelog.
- Candidate targets are a closed three-item set; missing or additional targets
  fail verification.
- `publication_allowed` is exactly `false`. The protected authorization job
  must remain skipped.
- Alpha 1 public assets and channels remain untouched. Alpha 3 evidence remains
  historical and cannot qualify Alpha 4.

## Failure and rollback

- Before merge: fix or abandon the release-preparation branch.
- After merge but before a valid candidate: correct the source through a new
  reviewed commit and restart exact-source qualification from the new `S`.
- After candidate creation: discard an invalid/expired artifact and rerun; do
  not edit receipts to make old bytes appear current.
- If an evidence-only receipt is wrong, revert or correct that receipt without
  rewriting `2.x` history, tags, or releases.

Because nothing is published, candidate rollback is deletion/expiry plus a new
exact-source run; no public rollback mechanism is needed.

## Risks and controls

- Version drift: run the existing sync and version-contract checks.
- Embedded-content drift: regenerate the checked-in lock and run its focused
  Rust tests.
- Head movement: compare `S` with remote `2.x`, Native CI `head_sha`, and
  promotion input before accepting any run.
- Platform failure: require the existing closed job set; do not replace a
  failed native target with local inference.
- Artifact expiry: download and verify within the existing three-day retention
  window.
- Authority leakage: keep the promotion authorization input false and confirm
  the protected authorization job was skipped.
- Closeout ambiguity: record both product source `S` and later evidence commit
  `E` explicitly.
