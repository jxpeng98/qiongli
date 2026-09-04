# Technical Design -- platform capacity and bounds baseline

## Boundary

This task adds a development-only measurement path over existing native owners:

```text
test-only deterministic fixtures
        -> existing project/Desktop services
        -> raw samples + fixture identity
        -> two JSON receipt parts per target
        -> one manual Native CI artifact per Linux/macOS/Windows target
```

No runtime service, public CLI, App route, persisted product schema, dependency,
or automatic gate changes. The three roadmap IDs stay in one Trellis task
because `PLT-402` and `PLT-403` consume the fixture corpus created for
`PLT-401`; splitting them would duplicate setup and evidence.

## Test-only owners

- A `qiongli-project` unit-test module owns Library, Capture, Graph, Portfolio,
  refresh, import/export, percentile, and resident-memory fixtures. Locating it
  under the crate root lets tests use the existing crate-private limits and
  pure builders without exporting new production APIs.
- A `qiongli` unit-test module owns native Desktop startup, App snapshot, and
  serialized IPC payload measurements through the existing `pub(crate)`
  Desktop entrypoints.
- Both modules write distinct JSON files to one required output directory and
  bind them with the same source/target/run identity. Cargo can run both with
  one workspace test filter; no aggregator or benchmark framework is needed.

## Fixture profiles

Profiles are deterministic and independent by subsystem:

| Surface | Small | Medium | Product limit |
| --- | ---: | ---: | ---: |
| Research Library projects | 3 | 64 | 512 |
| Capture documents | 8 | 128 | 1,024 |
| Graph nodes / edges | 64 / 64 | 1,024 / 1,024 | 4,096 / 4,096 |
| Portfolio projects | 3 | 64 | 512 |
| Portable project files | 8 | 128 | 1,024 |

Portfolio node, edge, and occurrence ceilings are exercised by separate exact
and one-over cases at 16,384, 32,768, and 65,536. They are not multiplied into
every other limit fixture. IDs, timestamps, labels, contents, and ordering are
fixed; absolute temporary paths and timing values are excluded from the
canonical fixture identity.

Fixture roots live under a unique temporary directory, never use the user's
normal config root, and are removed after each test process. Failure to prepare
or cleanly validate a fixture fails the run rather than reducing its size.

## Measurements

- Use `std::time::Instant` and one warm-up phase followed by 20 measured samples.
- Sort integer samples and use nearest-rank P50/P95. Retain raw samples in the
  receipt so percentile calculations remain auditable.
- Time the actual existing owners for project snapshot, refresh, Capture inbox
  load, Graph projection/index build and query, Portfolio rebuild, portable
  export/import, Desktop startup validation, and App snapshot construction.
- Record serialized App snapshot bytes as the IPC payload observation.
- Record absolute resident-set observations at defined fixture/operation
  checkpoints. Linux reads `/proc/self/statm`, macOS invokes `/bin/ps` without a
  shell, and Windows invokes PowerShell without a profile using only the current
  numeric process ID. Missing or malformed memory evidence fails that target's
  receipt.

Measurements are descriptive. No threshold, pass/fail performance claim, or
cross-target comparison is added before `PLT-407`.

## Receipt contract

Each target artifact contains a project receipt part and a Desktop receipt part.
Both include:

- receipt version and observation-only status;
- exact 40-character source commit;
- `std::env::consts::OS` and architecture;
- Rust version, sample count, unit, profile counts, and fixture SHA-256;
- raw samples plus calculated P50/P95;
- no host name, user name, absolute path, environment dump, credential, or
  research content.

Receipt validation rejects a missing source, unsupported target label,
non-positive sample, missing named operation, profile mismatch, percentile
mismatch, or divergent source/fixture identity.

## Manual three-target run

The existing `.github/workflows/native-ci.yml` Rust foundation matrix receives
two steps guarded by `github.event_name == 'workflow_dispatch'`: run the ignored
release-mode capacity tests, then upload the target receipt directory. Pull
request events continue to skip both steps. A dispatch against the feature
branch also keeps all existing `refs/heads/2.x` package and candidate jobs
skipped.

After one exact-source run passes on Linux, macOS, and Windows, a concise
acceptance record stores the observed values and the exact run/source IDs. The
Program Ledger may then mark `PLT-401`--`PLT-403` accepted. A later evidence-only
closeout commit does not rewrite the measured product source.

## Compatibility, risk, and rollback

- Test-only modules cannot change shipped behavior or public wire contracts.
- Hosted-runner noise is retained as labelled observation, not hidden behind a
  false SLO. `PLT-407` owns any later budget decision.
- Product-limit generation can be slow or memory-heavy, so it remains ignored
  outside explicit manual runs and must fit the existing 30-minute matrix job.
- Reverting the test modules, manual-only workflow steps, acceptance record,
  and ledger transition removes the baseline without migrating or deleting user
  state.
