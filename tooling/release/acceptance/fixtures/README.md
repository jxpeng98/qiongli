# Host-Driven Acceptance Fixtures

`alpha2-host-driven-v1.json` is the fixed, canonical fixture contract for the
Codex and Claude Code Alpha.2 host sessions.

`r5c-c5-host-driven-v1.json` is the package-bound C5 fixture. It uses project
revision 2 from the isolated three-project continuity lifecycle and anchors its
two synthetic facts to the graph records that are observable through the
declared Full MCP project-read tools.

It contains only source-fact and source-anchor digests, required project-read
tools, observable checkpoint transition kinds, candidate requirements, and
minimum fail-closed rejection counts. It does not contain an expected prose
answer, project ID/path, prompt, candidate body, model response, conversation
ID, provider credential, or tool result.

Validate the fixture without starting a model host:

```sh
pnpm acceptance:host:preflight
```

This produces a non-publishing `fixture-ready-manual-host-required` preflight
summary. It is not a host acceptance receipt.

For the R5C C5 package-bound flow, first build the accepted App, then commit the
acceptance helper and run:

```sh
pnpm acceptance:host:c5:preflight
pnpm desktop:macos:acceptance:host-prepare
```

The preparation command never rebuilds the product and never touches the real
user home. It validates the exact accepted product receipt and binary, installs
both managed Plugin sources and registrations into `manual-home` without Host
authentication or activation, and creates the same three-project fixture. Its
canonical `qiongli-packaged-host-fixture.receipt.json` contains only hashes,
counts, ordinals, revisions, and verdicts. Re-running the command validates the
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

For C5, follow
`tooling/release/acceptance/fixtures/r5c-c5-live-host-runbook.md`, record a
canonical observation conforming to
`r5c-c5-host-observation.schema.json`, and let the package-bound composer derive
the product, Plugin, fixture-fact, and evidence-audit bindings. The isolated
home proves installation and fixture preparation without login. The live
handoff runs in an already authenticated system Host whose current 2.x
registration is passed explicitly:

```sh
bash scripts/compose_macos_acceptance_host_receipt.sh \
  --observation /absolute/path/to/canonical-observation.json \
  --system-registration /absolute/path/to/qiongli-next-registration.json
```

Then use the stronger package-bound validator:

```sh
pnpm acceptance:host:c5:receipt -- \
  --receipt /absolute/path/to/qiongli-c5-host-acceptance.receipt.json \
  --system-registration /absolute/path/to/qiongli-next-registration.json
```

In addition to the receipt contract, this binds the receipt to the accepted
product receipt, exact App binary, prepared fixture revision, product version
and source commit, the isolated installed Plugin content digest, and the same
current 2.x Plugin digest in the system Codex or Claude Code registration. The
validator requires the Host's standard registration suffix outside the
acceptance root, so the isolated registration cannot be reused as system
evidence. The registration path and Host authentication state never enter the
validation output, which remains path-redacted and non-publishing.
