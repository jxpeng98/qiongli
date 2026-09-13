# CTR-201 accepted-source semantic inventory

Status: **source-oracle inventory complete**

The CTR-201A master ledger derives a mutable 2.x semantic inventory from the
immutable `v1.19.0-beta.1` migration baseline. CTR-201B-F close its required
CLI, orchestrator, and content children. None of these artifacts edits or
regenerates the A8 manifest, its oracle fixtures, the 2.x branch-point
evidence, or ADR 0201-0207.

## Closed inventory

- CTR-201A binds the master ledger to the accepted tag, peeled commit, baseline
  corpus digest, 377-file `content/` tree digest, and all three frozen oracle
  fixture digests.
- CTR-201B freezes the accepted-source static Full CLI command and parser
  contract.
- CTR-201C freezes the accepted-source declared/static orchestrator control
  contract.
- CTR-201D closes the canonical content tree and the three reproducible
  materialized skill-subtree projections.
- CTR-201E, merged through protected PR #57, freezes the accepted-source Full
  CLI runtime inventory and explicitly dispositions unexecuted handler cells.
- CTR-201F closes the accepted-source orchestrator runtime inventory with
  deterministic bounded fixtures and explicit dispositions for unavailable,
  unsafe, or inapplicable runtime dimensions.

The master validator rejects missing child bindings, unclassified required
gaps, schema or payload drift, duplicate identifiers, non-portable paths,
secret-shaped data, and synchronized artifact tampering. A required runtime
dimension may be closed by captured evidence or a machine-bound approved
disposition; omission is not completion.

## Evidence snapshot

| Evidence | Confirmed state |
|---|---:|
| Frozen A8 content tree | 377 files |
| Frozen runtime oracles | 3 fixtures, 5 cases each |
| Static Full CLI | 46 canonical paths, 49 public paths, 5 entrypoints |
| Canonical content | 377 files, 3 materialized profiles |
| Orchestrator runtime freeze | 44 cases: 1 A8 + 43 bounded; 6 dimensions; 6 dispositions |
| Contract v2 pilot | 6 canonical names, 7 public names |
| Target contract | 23 canonical names, 24 public names |
| Observed oracle public-name union | 26 names |
| Node-only legacy names | 2 names |

The checked artifacts are the machine-readable source for exact paths, hashes,
case counts, outcomes, and disposition IDs. This document intentionally avoids
duplicating their long matrices.

## Completion boundary

CTR-201 completion means the accepted-source MCP, CLI, content, and
orchestrator inventory is normalized, digest-bound, and free of unclassified
required gaps. It does not mean that the Rust implementation conforms to that
inventory.

In particular, CTR-201F does not launch real Codex, Claude, or Antigravity
agents and does not establish strict single-agent solo enforcement, public
cancellation, real session resume, native worker dispatch, semantic
quality-gate execution, or cross-platform orchestrator runtime parity. Its
Windows and macOS gates validate the checked portable artifact; canonical
runtime re-extraction runs only in the Ubuntu full tier with Python 3.12.

CTR-202 and FND-202 are separate successors now unblocked by CTR-201. Contract
v2 remains a pilot, and the deterministic Rust resource pack is not
implemented. CTR-201E's three CLI dispositions remain `LEG-201` work.
CTR-201F's six decisions bind their actual downstream owners across AGT, ORC,
CFG, DOM, GOV, MCP, and, only where declared, LEG tasks; they must not be
collapsed into a generic `LEG-201` assignment.

CTR-201 does not publish a 2.x alpha, implement a Rust CLI or orchestrator,
write to a Codex, Claude, ChatGPT, Marketplace, or host-cache location, or
establish archive/plugin-wrapper parity.

## Validation

Run:

```bash
python3 scripts/generate_migration_baseline.py verify
python3 scripts/extract_ctr_201_cli_runtime_inventory.py --check
python3 scripts/extract_ctr_201_orchestrator_runtime_inventory.py --check
python3 scripts/validate_ctr_201_inventory.py
python3 -m unittest tests.test_ctr_201_inventory \
  tests.test_ctr_201_orchestrator_runtime_inventory -v
python3 scripts/check_frozen_migration_baseline.py --base-ref origin/2.x
python3 scripts/check_frozen_2x_architecture_baseline.py --base-ref origin/2.x
```

The RC1 repository-source validator may still be run manually as an optional
diagnostic, but it is not part of the CTR-201, migration, CI, or release gate.

PR descriptions must distinguish checked-tree implementation, exact-head CI,
and protected-branch integration. Allowed completion language is:

> CTR-201F closes the accepted-source orchestrator runtime inventory and, with
> CTR-201A-E, completes the CTR-201 source-oracle gate. This is bounded oracle
> and disposition evidence, not real-agent parity, Rust implementation,
> cross-platform runtime parity, CTR-202 completion, FND-202 implementation, or
> publication evidence.
