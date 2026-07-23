# Alpha.2 Host-Driven Acceptance Fixture

`alpha2-host-driven-v1.json` is the fixed, canonical fixture contract for the
Codex and Claude Code Alpha.2 host sessions.

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
