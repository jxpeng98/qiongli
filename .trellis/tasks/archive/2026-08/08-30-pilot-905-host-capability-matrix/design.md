# PILOT-905 design

## Boundary

Project one small, immutable evidence graph:

`accepted receipts -> machine matrix -> bilingual docs -> Program Ledger`

The accepted receipts remain authoritative. The matrix is a projection, not a
new Host adapter, product capability registry, benchmark, or release candidate.

## Machine contract

The JSON receipt uses:

- a closed ordered Host list;
- a closed ordered capability list;
- three statuses: `observed-present`, `observed-absent`, `not-observed`;
- source-bound evidence records with repository-relative path and SHA-256; and
- one cell per Host/capability pair with zero or more evidence IDs.

`observed-present` means the cited receipt directly demonstrated the capability.
`observed-absent` means the cited receipt directly demonstrated its absence.
`not-observed` means no accepted receipt proves either presence or absence. It
does not mean unsupported.

The existing receipts support two evidence records:

1. August 24 Codex/Claude client compatibility at source `192ad24f...`.
2. PILOT-903 Codex model/Graph journey at source `d0b4113...`.

Codex CLI cells may cite either exact observation. Claude Code cells cite only
the compatibility observation. Other Hosts have no evidence IDs and remain
`not-observed`. Exact model identity remains `not-recorded` for both observed
Hosts.

## User-facing projection

English and Chinese pages render the same compact matrix with three symbols and
then list exact evidence identities. They explicitly explain that:

- client installation/protocol compatibility is not authenticated model use;
- one Host result does not transfer to another Host;
- one source-bound result does not qualify a changed release candidate; and
- `not-observed` is an evidence gap, not a negative capability claim.

The docs landing pages add one direct link. No generator is introduced because
the matrix is small and changes only when a new accepted receipt lands.

## Validation

One standard-library unittest loads the JSON and verifies:

- exact Host and capability inventories;
- exact status vocabulary and complete rectangular coverage;
- evidence IDs, file existence, SHA-256, source commit, and Host/version shape;
- evidence on every observed cell and no evidence on `not-observed` cells;
- model identity remains explicit rather than inferred; and
- the two public docs expose all rows, statuses, and the machine receipt path.

The existing docs build, Program Ledger generator, task validator, diff check,
and required source CI close the Slice. No package, live-Host, or promotion job
is valid evidence for this docs/evidence-only task.

## Rollback

Revert the matrix receipt, bilingual pages, links, focused test, and ledger
entry together. Existing accepted receipts and product behavior remain
unchanged.

