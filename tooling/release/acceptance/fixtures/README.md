# Host-Driven Acceptance Fixtures

`alpha2-host-driven-v1.json` is the fixed, canonical fixture contract for the
Codex and Claude Code Alpha.2 host sessions.

`r5c-c5-host-driven-v1.json` is the package-bound C5 fixture. It uses project
revision 2 from the isolated three-project continuity lifecycle and anchors its
two synthetic facts under
`RESEARCH/r5c-c5-host-acceptance/sources.md`.

It contains only source-fact and source-anchor digests, the required
project-read tool, checkpoint transition kinds, and candidate requirements. It
does not contain an expected prose answer, project ID/path, prompt, candidate
body, model response, conversation ID, provider credential, or tool result.

Validate the fixture without starting a model host:

```sh
pnpm acceptance:host:preflight
```

This produces a non-publishing `fixture-ready-manual-host-required` preflight
summary. It is not a host acceptance receipt.

For the R5C C5 package-bound flow, first build and manually install the accepted
App and both host plugins in its isolated `manual-home`, then commit the
acceptance helper and run:

```sh
pnpm acceptance:host:c5:preflight
pnpm desktop:macos:acceptance:host-prepare
```

The preparation command never rebuilds the product and never touches the real
user home. It validates the exact accepted product receipt and binary before
creating the same three-project fixture in `manual-home`. Its canonical
`qiongli-packaged-host-fixture.receipt.json` contains only hashes, counts,
ordinals, revisions, and verdicts. Re-running the command validates the
existing fixture instead of creating another one.

After a separately approved real-host session writes a canonical receipt,
validate it against the fixed fixture with:

```sh
cargo run --locked \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --example native_host_acceptance_contract \
  -- receipt \
  tooling/release/acceptance/fixtures/alpha2-host-driven-v1.json \
  /path/to/qiongli-alpha2-host-acceptance.receipt.json
```

The receipt contract accepts only exact build/host/plugin/protocol identities,
hashes, counts, fixed tool IDs, checkpoint transitions, review result, and
zero-direct-execution verdicts. Unknown fields and non-canonical JSON are
rejected.

For C5, use the stronger package-bound validator:

```sh
pnpm acceptance:host:c5:receipt -- \
  --receipt /absolute/path/to/qiongli-c5-host-acceptance.receipt.json
```

In addition to the receipt contract, this binds the receipt to the accepted
product receipt, exact App binary, prepared fixture revision, product version
and source commit, plus the installed plugin content digest for the receipt's
Codex or Claude Code host family. The validation output remains path-redacted
and non-publishing.
