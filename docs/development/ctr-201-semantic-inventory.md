# CTR-201A derived semantic inventory

Status: **in progress**

CTR-201A derives a mutable 2.x semantic inventory from the immutable
`v1.19.0-beta.1` migration baseline. It does not edit or regenerate the A8
manifest, its oracle fixtures, the 2.x branch-point evidence, or ADR 0201-0207.

## Delivered by this slice

- Bind the derived inventory to the accepted tag, peeled commit, baseline
  corpus digest, 377-file `content/` tree digest, and all three frozen oracle
  fixture digests.
- Record the observed Python Full, Rust Lite, and legacy Node MCP public-name
  surfaces separately from the 23-canonical / 24-public target contract.
- Preserve the two Node-only Zotero names as explicit legacy-only evidence
  pending `LEG-201` disposition.
- Record the current Contract v2 pilot coverage and the known CLI,
  orchestrator, profile, and materialization gaps.
- Validate the inventory, its canonical payload digest, frozen anchors,
  duplicate identifiers, portable paths, counts, and completion status with a
  fail-closed command.

## Not delivered by this slice

- CTR-201 is not complete. Contract v2, CLI, orchestrator, and content-profile
  coverage remain incomplete.
- FND-202 is not implemented. The native binary does not yet compile, embed,
  inspect, list, read, or materialize a `.qlpack` resource pack.
- The inventory does not claim parity with the Python materializer or write to
  a Codex, Claude, ChatGPT, Marketplace, or host-cache location.
- The legacy-only names are recorded, not accepted into the 2.x public
  contract; their disposition belongs to `LEG-201`.

## Evidence snapshot

| Evidence | Confirmed state |
|---|---:|
| Frozen A8 content tree | 377 files |
| Frozen runtime oracles | 3 fixtures, 5 cases each |
| Contract v2 pilot | 6 canonical names, 7 public names |
| Target contract | 23 canonical names, 24 public names |
| Observed oracle public-name union | 26 names |
| Node-only legacy names | 2 names |

The checked inventory is the machine-readable source for exact paths, hashes,
runtime surfaces, and gap identifiers. This document intentionally avoids
duplicating those long lists.

The frozen Python oracle does contain an `align` success, an installer dry-run
with exit code zero, and a duo-mode orchestration preview. CTR-201A records
those observations while marking the complete command, exit-code, dry-run, and
solo/duo/triad matrices as not yet fully captured.

## Validation

Run:

```bash
python3 scripts/generate_migration_baseline.py verify
python3 scripts/validate_ctr_201_inventory.py
python3 -m unittest tests.test_ctr_201_inventory -v
python3 scripts/check_frozen_migration_baseline.py --base-ref origin/2.x
python3 scripts/check_frozen_2x_architecture_baseline.py --base-ref origin/2.x
python3 scripts/validate_repository_source.py --base-ref origin/2.x
```

PR and release notes must describe this work as advancing CTR-201 through its
derived semantic-inventory slice. They must not describe CTR-201 or FND-202 as
complete.
