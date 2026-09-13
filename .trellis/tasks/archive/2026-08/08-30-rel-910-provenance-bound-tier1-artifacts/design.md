# REL-910 design

## Boundary

Reuse the existing release path and correct only its control-plane identity:

`exact-source Native CI -> promotion run attempt -> target-native rebuilds ->`
`target receipts -> aggregate candidate artifact`

No new builder, artifact format, release ledger, signing service, or package
workflow is introduced.

## Provenance binding

The exact-head preflight keeps validating the user-supplied Native CI run ID
against workflow name, source SHA, completion, and success. That run is
qualification evidence, not the builder.

The preflight output named `build_run_url` instead comes from the active
promotion runtime:

```text
$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/
$GITHUB_RUN_ID/attempts/$GITHUB_RUN_ATTEMPT
```

All target jobs receive that immutable attempt URL and the same source commit.
The existing Rust owner embeds both values in each target promotion, rejects
cross-target disagreement, records exact asset/evidence sizes and SHA-256
digests, and derives `candidate_set_sha256` from canonical candidate content.
The later SLSA projection therefore names the build attempt that actually
created the artifacts.

## Authorization separation

Add the workflow-dispatch boolean `request_publication_authorization`, default
`false`. Gate only `authorize-candidate` on this value.

- `false`: exact-source rebuild and aggregate jobs complete; the workflow is
  green and the candidate remains `publication_allowed: false`.
- `true`: the existing protected Environment job runs unchanged and emits its
  short-lived read-only authorization receipt after maintainer approval.

Native CI's acceptance dispatch uses the safe default. Actual release work may
explicitly request authorization later; REL-910 does not do so.

## Acceptance evidence

After merging the implementation, dispatch Native CI on the exact current
`2.x` source. The successful dispatch produces a single downstream promotion
run. Download its aggregate candidate artifact to a private temporary
directory and verify:

- candidate canonical JSON and content digest;
- exact source and build-attempt URL;
- ordered macOS, Windows, and Linux target identities;
- exact five-file public inventory;
- each asset/evidence path, bounded size, and SHA-256; and
- absence of publication authorization/publication mutation.

Commit only a compact repository receipt with public source/run identities and
digests. Do not commit built binaries or runner-local paths.

## Validation and rollback

The existing `tests/test_branch_policy.py` is the focused workflow owner. The
existing `qiongli-platform` Community Alpha unit tests remain the candidate
schema owner. Exact-head Native CI and the downstream three-target run provide
the real platform proof.

Rollback is one workflow/test revert. Existing candidate schema and prior
artifacts remain readable; no public release, user state, Host profile, or
private key is mutated.
