# CTR-201E accepted-source Full CLI runtime inventory freeze

Status: **merged through protected PR #57; historical slice complete**

CTR-201E is an engineering child of the Qiongli 2 migration task `CTR-201`.
It freezes the observable and explicitly dispositioned Full CLI runtime
inventory from the accepted `v1.19.0-beta.1` source boundary. Later Rust
implementations can compare against captured cases and can identify every
uncaptured handler dimension without treating it as parity evidence. It does
not implement the Rust Full CLI, establish Full handler runtime parity, or add
a canonical academic research Task ID.

## Dependencies and program order

CTR-201E depends on:

- the accepted `v1.19.0-beta.1` tag, peeled commit, and immutable migration
  baseline;
- CTR-201A's derived semantic ledger and source bindings; and
- CTR-201B's closed static CLI inventory of 46 canonical command paths, 49
  public command paths, five console entrypoints, 164 non-help actions, and 27
  current-working-directory defaults.

CTR-201E must not declare `CTR-201` as its dependency: the parent exit gate
depends on its required children, so that declaration would create a cycle.
CTR-201E closed only the CLI-runtime inventory slice. At the time this child
artifact was recorded, CTR-201 remained in progress pending the
accepted-source orchestrator-runtime requirement. CTR-201F subsequently closed
that remaining source-oracle requirement; the CTR-201E child artifact retains
its historical slice-local status unchanged.

CTR-201D does not establish archive-member, published ZIP/TAR, or plugin-wrapper
parity. That evidence remains an unassigned downstream governance boundary; it
is not a CTR-201E deliverable or a CTR-201 parent exit dependency.

After CTR-201 closes, `CTR-202` and `FND-202` are separate successors under the
current roadmap:

- CTR-202 expands and completes Capability Contract v2;
- FND-202 builds the deterministic embedded resource pack after FND-201 and
  CTR-201.

Neither successor is completion evidence for the other, and neither has been
implemented by CTR-201E.

## Capture contract

The runtime corpus must use the public paths and entrypoints already fixed by
CTR-201B rather than discovering a different command surface. For every
applicable public command path or entrypoint, the closed coverage matrix must
classify:

- formatted help and zero-argument behavior;
- stdout and stderr separately;
- JSON output where the accepted command exposes it;
- process exit code;
- normalized error class;
- dry-run and observable side-effect behavior;
- public aliases and console-entrypoint equivalence; and
- the legacy npm dispatch boundary.

Every cell must be `captured`, `not-applicable`, or linked to an explicit
approved disposition. Missing cells and free-form unclassified exceptions are
failures. `Not-applicable` must state a machine-checkable reason; it must not be
used to hide an unsafe or inconvenient scenario.

The approved inventory-only decisions are machine-bound in the artifact:

- `CTR-201E-D001` transfers bounded stateful handler fixtures to `LEG-201`;
- `CTR-201E-D002` transfers network, browser, listener, secret-bearing,
  download, and self-update fixtures to `LEG-201`; and
- `CTR-201E-D003` transfers npm handler stdout, exit, and side-effect parity to
  `LEG-201` while retaining the captured `parseArgv` dispatch map.

These decisions close missing inventory classification, not runtime parity.

Runtime capture must:

- bind the accepted tag, peeled commit, source hashes, pinned Python version,
  and the accepted npm parser's minimum Node engine requirement;
- use fixed locale, timezone, environment allowlist, temporary HOME, and
  temporary working/project directories;
- normalize approved nondeterminism such as temporary roots and timestamps
  without erasing semantically meaningful output;
- deny network, subprocess, browser, and out-of-sandbox write attempts in the
  isolated Python worker;
- execute only the authenticated, source-audited `args.mjs` parser in the Node
  worker, reject capability-bearing imports or calls before execution, and
  verify that the accepted npm tree is unchanged afterward; this is a bounded
  parse-only control, not an operating-system network sandbox claim;
- never write a real user home, client configuration, plugin cache,
  Marketplace directory, or repository-generated payload; and
- record side-effect observations from before/after filesystem manifests rather
  than relying only on a command's dry-run label.

The npm surface is compatibility evidence for the accepted 1.x boundary. Its
capture does not mean Node is part of the Qiongli 2 production runtime.

## Implemented surface

The implementation adds the following source-controlled evidence and gates
without changing the immutable A8 oracle:

- `tooling/migration/ctr-201-cli-runtime.json`;
- `tooling/migration/ctr-201-cli-runtime.schema.json`;
- `tooling/scripts/extract_ctr_201_cli_runtime_inventory.py` plus a stable root
  wrapper under `scripts/`;
- `tests/test_ctr_201_cli_runtime_inventory.py`; and
- a digest/count/status binding from the mutable CTR-201 master ledger.

The exact artifact structure is owned by its closed schema. Schema validation
alone is insufficient: a semantic validator must enforce cross-record equality,
source identity, complete coverage, canonical digest, portable paths, and the
parent-ledger binding. An independently fixed case-manifest root binds the
ordered digest of all 118 invocation/outcome/effect records, so synchronized
changes to a case and the overall payload digest still fail closed.

The checked corpus contains 118 cases. It captures 245 entrypoint-by-public-path
help observations, 49 invalid-usage observations, five zero-argument group
boundaries, five console-entrypoint root-help and `align` pairs, two safe JSON
handler boundaries, one representative domain error, two accepted A8 cases,
and the accepted npm `parseArgv` dispatch mapping. Handler scenarios that would
need authenticated services, user-state writes, downloads, browser/listener
control, or a separately bounded project fixture remain explicit dispositions;
they are not silently treated as executed behavior or Full CLI parity.
Concretely, 39 executable public-path behavior/stream/exit/error/side-effect
dimensions remain decision-bound rather than successful-handler captures; 11
JSON-capable paths and 10 dry-run paths are likewise dispositioned. The master
ledger's CLI `completion_ready` flag therefore means inventory closure only.

## Exit gate

CTR-201E may be described as complete only when all of the following pass:

1. the artifact contains exactly the CTR-201B-declared 49 public command paths
   and five console entrypoints, with no duplicate or unknown path;
2. every required behavior dimension has a captured value, a valid
   not-applicable reason, or an approved disposition;
3. two isolated extractions under the declared Python 3.12 and Node `>=18`
   parser boundaries produce byte-identical canonical output and the same
   payload digest;
4. the checked extractor rejects accepted-source, dependency, environment, and
   command-surface drift;
5. schema and semantic validation reject missing/duplicate cases, altered
   streams or exit codes, invalid error classes, source/digest drift, forbidden
   host writes, and undeclared nondeterminism;
6. the slice-local child binding reports the CLI runtime inventory ready while
   retaining its historical `CTR-201: in-progress` and
   `FND-202: not-implemented` boundary; the mutable master ledger may advance
   only through a later, separately validated child; and
7. protected PR #57 exact-head CI ran canonical extraction in the controlled
   Ubuntu full tier with Python 3.12 and Node 20, then validated the portable
   artifact and parent binding on the declared operating-system matrix.

Running a schema or validator on Linux, macOS, and Windows proves portable
validation only. Tier 1 runtime parity may be claimed only if the runtime corpus
itself is reproduced or replayed on all three operating-system families with
matching, explicitly normalized results.

## Claim boundary

Allowed completion language:

> CTR-201E captures and explicitly dispositions the declared accepted-source
> Full CLI runtime inventory, closing that inventory slice. It is bounded
> oracle and decision evidence, not Full handler parity or a Rust
> implementation. This child did not complete CTR-201 by itself; CTR-201F later
> closed the remaining source-oracle requirement. CTR-202 is not complete, and
> FND-202 is not implemented.

Do not describe CTR-201E as any of the following:

- a Rust Full CLI implementation or migration;
- Full CLI parity, Tier 1 runtime parity, or zero-dependency runtime acceptance;
- completion of CTR-201 by CTR-201E alone, or Capability Contract v2 / CTR-202;
- implementation of FND-202 or an embedded resource pack; or
- an installable or published Qiongli 2 alpha, plugin, Marketplace package,
  release artifact, tag, or registry publication.
