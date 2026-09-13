# Qiongli PLT-401--PLT-403 Platform Capacity Acceptance

Status: accepted as observation-only Tier 1 capacity evidence

Date: September 4, 2026

Target branch: `codex/platform-capacity-bounds`

Publication allowed: false

## Exact identities

| Identity | Accepted value |
|---|---|
| Product source | `3ab9d3f5c2d4e035068c3ea839caaa7e6f97c9d3` |
| Native CI | [run `33876294769`](https://github.com/jxpeng98/qiongli/actions/runs/33876294769): success |
| Rust | `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| Receipt contract | `qiongli-platform-capacity/v1`; `observation-only`; one warm-up plus 20 samples |
| Project fixture | `f4701b3e229e2721210fe24a52b6ae614d97b3c87d7630e9a237e22aceef1175` |
| Desktop fixture | `a695f21da3fcc33636df4a645aa2cdbfef0b4d4c7d930e10cda0f01bd6474124` |

| Target | Native foundation | Artifact |
|---|---:|---|
| Linux x86_64 | 28m05s | `qiongli-capacity-Linux-3ab9d3f5c2d4e035068c3ea839caaa7e6f97c9d3` |
| macOS aarch64 | 32m16s | `qiongli-capacity-macOS-3ab9d3f5c2d4e035068c3ea839caaa7e6f97c9d3` |
| Windows x86_64 | 46m24s | `qiongli-capacity-Windows-3ab9d3f5c2d4e035068c3ea839caaa7e6f97c9d3` |

## Fixture profiles

| Surface | Small | Medium | Product limit |
|---|---:|---:|---:|
| Research Library projects | 3 | 64 | 512 |
| Capture documents | 8 | 128 | 1,024 |
| Graph nodes / edges | 64 / 64 | 1,024 / 1,024 | 4,096 / 4,096 |
| Portfolio projects | 3 | 64 | 512 |
| Portable project files | 8 | 128 | 1,024 |

The independent Portfolio ceilings are 16,384 nodes, 32,768 edges, and 65,536
occurrences. Exact-limit cases passed and each `limit + 1` case retained its
native fail-closed error class.

## Project observations

Each target cell is `P50 / P95`. Timing values are nanoseconds and resident
memory values are bytes.

| Profile | Operation | Metric | Linux x86_64 | macOS aarch64 | Windows x86_64 |
|---|---|---|---:|---:|---:|
| small | project_snapshot | elapsed_time (nanoseconds) | 547666 / 556358 | 589333 / 614083 | 4135900 / 4345100 |
| small | project_snapshot | resident_memory (bytes) | 7036928 / 7036928 | 5324800 / 5324800 | 6955008 / 6971392 |
| small | project_refresh | elapsed_time (nanoseconds) | 236348 / 248465 | 282208 / 337250 | 1833400 / 2039700 |
| small | project_refresh | resident_memory (bytes) | 7036928 / 7036928 | 5439488 / 5439488 | 7032832 / 7032832 |
| small | capture_load | elapsed_time (nanoseconds) | 661523 / 677066 | 724166 / 1091625 | 4517400 / 4915200 |
| small | capture_load | resident_memory (bytes) | 8151040 / 8151040 | 6881280 / 6897664 | 7553024 / 7593984 |
| small | graph_build | elapsed_time (nanoseconds) | 105475 / 119576 | 117084 / 125375 | 250300 / 293900 |
| small | graph_build | resident_memory (bytes) | 14381056 / 14381056 | 21331968 / 21331968 | 13770752 / 13774848 |
| small | graph_query | elapsed_time (nanoseconds) | 52177 / 60478 | 50958 / 60125 | 125300 / 139500 |
| small | graph_query | resident_memory (bytes) | 14381056 / 14381056 | 21512192 / 21512192 | 13860864 / 13860864 |
| small | portfolio_rebuild | elapsed_time (nanoseconds) | 194015 / 204381 | 223875 / 246958 | 440700 / 502000 |
| small | portfolio_rebuild | resident_memory (bytes) | 24162304 / 24162304 | 27820032 / 27820032 | 15380480 / 15388672 |
| small | portable_export | elapsed_time (nanoseconds) | 5067353 / 5547890 | 8029250 / 9971292 | 216330700 / 552074300 |
| small | portable_export | resident_memory (bytes) | 24178688 / 24178688 | 29343744 / 29343744 | 16310272 / 16310272 |
| small | portable_import | elapsed_time (nanoseconds) | 7855895 / 8343141 | 15723125 / 25149833 | 408488900 / 1039380800 |
| small | portable_import | resident_memory (bytes) | 24178688 / 24178688 | 29425664 / 29425664 | 16453632 / 16478208 |
| medium | project_snapshot | elapsed_time (nanoseconds) | 11135134 / 11233959 | 10853958 / 11661667 | 73661000 / 84333800 |
| medium | project_snapshot | resident_memory (bytes) | 7036928 / 7036928 | 5619712 / 5619712 | 7057408 / 7069696 |
| medium | project_refresh | elapsed_time (nanoseconds) | 424133 / 437804 | 388000 / 405875 | 2211200 / 2332000 |
| medium | project_refresh | resident_memory (bytes) | 7036928 / 7036928 | 5619712 / 5619712 | 7258112 / 7270400 |
| medium | capture_load | elapsed_time (nanoseconds) | 10417002 / 10484932 | 10503041 / 13360750 | 60928400 / 62494300 |
| medium | capture_load | resident_memory (bytes) | 8282112 / 8282112 | 7094272 / 7127040 | 7614464 / 7774208 |
| medium | graph_build | elapsed_time (nanoseconds) | 1808590 / 1840107 | 1660542 / 1700417 | 3644000 / 3920900 |
| medium | graph_build | resident_memory (bytes) | 15097856 / 15097856 | 22429696 / 22429696 | 13275136 / 13578240 |
| medium | graph_query | elapsed_time (nanoseconds) | 249957 / 261385 | 222791 / 245500 | 426500 / 466200 |
| medium | graph_query | resident_memory (bytes) | 15097856 / 15097856 | 22577152 / 22577152 | 14938112 / 14938112 |
| medium | portfolio_rebuild | elapsed_time (nanoseconds) | 3383547 / 3454841 | 3511792 / 3786750 | 5940800 / 6231800 |
| medium | portfolio_rebuild | resident_memory (bytes) | 24162304 / 24162304 | 28114944 / 28114944 | 16400384 / 16707584 |
| medium | portable_export | elapsed_time (nanoseconds) | 48782272 / 66665877 | 119093417 / 140376750 | 1928814500 / 2796715600 |
| medium | portable_export | resident_memory (bytes) | 24244224 / 24244224 | 29507584 / 29507584 | 16756736 / 16756736 |
| medium | portable_import | elapsed_time (nanoseconds) | 53631094 / 54800745 | 97888208 / 129494750 | 2160720100 / 2852552100 |
| medium | portable_import | resident_memory (bytes) | 24248320 / 24248320 | 29540352 / 29540352 | 16822272 / 16822272 |
| product-limit | project_snapshot | elapsed_time (nanoseconds) | 90874575 / 91195337 | 87819958 / 88976542 | 607031500 / 624099200 |
| product-limit | project_snapshot | resident_memory (bytes) | 7593984 / 7593984 | 6258688 / 6291456 | 7348224 / 7380992 |
| product-limit | project_refresh | elapsed_time (nanoseconds) | 1748252 / 1766148 | 1156833 / 1231250 | 4357900 / 4464600 |
| product-limit | project_refresh | resident_memory (bytes) | 7598080 / 7598080 | 6291456 / 6291456 | 7405568 / 7446528 |
| product-limit | capture_load | elapsed_time (nanoseconds) | 84090316 / 84327996 | 81877000 / 86392125 | 486503700 / 503425900 |
| product-limit | capture_load | resident_memory (bytes) | 8282112 / 8282112 | 7438336 / 7438336 | 7692288 / 7786496 |
| product-limit | graph_build | elapsed_time (nanoseconds) | 8350032 / 8846022 | 6949833 / 7220500 | 16468200 / 17080800 |
| product-limit | graph_build | resident_memory (bytes) | 21708800 / 21708800 | 25493504 / 25493504 | 13377536 / 13647872 |
| product-limit | graph_query | elapsed_time (nanoseconds) | 941204 / 961845 | 709875 / 822292 | 1356300 / 1474100 |
| product-limit | graph_query | resident_memory (bytes) | 21708800 / 21708800 | 25919488 / 25919488 | 19836928 / 19836928 |
| product-limit | portfolio_rebuild | elapsed_time (nanoseconds) | 28010199 / 28310010 | 28187041 / 29158958 | 50415300 / 51840900 |
| product-limit | portfolio_rebuild | resident_memory (bytes) | 24178688 / 24178688 | 29261824 / 29278208 | 16482304 / 16715776 |
| product-limit | portable_export | elapsed_time (nanoseconds) | 390459034 / 399765507 | 832400541 / 923567083 | 12747187700 / 17739703500 |
| product-limit | portable_export | resident_memory (bytes) | 24248320 / 24248320 | 29638656 / 29638656 | 16445440 / 18100224 |
| product-limit | portable_import | elapsed_time (nanoseconds) | 405512471 / 426799119 | 898344250 / 1023508875 | 13512194200 / 18295649200 |
| product-limit | portable_import | resident_memory (bytes) | 24248320 / 24248320 | 26705920 / 26705920 | 16424960 / 17063936 |

## Desktop observations

Each target cell is `P50 / P95` in the unit shown.

| Profile | Metric | Linux x86_64 | macOS aarch64 | Windows x86_64 |
|---|---|---:|---:|---:|
| small | native_startup_validation (nanoseconds) | 10611348 / 10716443 | 19307958 / 19636292 | 20307100 / 20635800 |
| small | app_snapshot (nanoseconds) | 9436821 / 9640791 | 18289417 / 18900666 | 17447900 / 18865700 |
| small | serialized_ipc_payload (bytes) | 16375 / 16375 | 16383 / 16383 | 16396 / 16396 |
| medium | native_startup_validation (nanoseconds) | 33545733 / 33757845 | 34766916 / 35271750 | 94850800 / 102197400 |
| medium | app_snapshot (nanoseconds) | 32326740 / 32614262 | 33981875 / 35680959 | 91525700 / 99600100 |
| medium | serialized_ipc_payload (bytes) | 84025 / 84025 | 84033 / 84033 | 84046 / 84046 |
| product-limit | native_startup_validation (nanoseconds) | 199343590 / 199904105 | 150557500 / 152477458 | 638436300 / 656175700 |
| product-limit | app_snapshot (nanoseconds) | 198566468 / 200221742 | 150620083 / 177606917 | 638001800 / 668019200 |
| product-limit | serialized_ipc_payload (bytes) | 580858 / 580858 | 580866 / 580866 | 580879 / 580879 |

## Verification

- All six JSON receipts match the exact source, run, schema, sample count,
  Rust version, and target identities above.
- Every receipt retains 20 raw samples; every recorded P50/P95 value was
  recomputed with the nearest-rank rule.
- Project and Desktop fixture identities are deterministic and identical across
  all three targets.
- Native formatting, workspace check, Clippy, full tests, R4A mobility, and R2
  Lite compatibility passed. Package, candidate, signing, promotion, and
  publication jobs remained skipped on the feature branch.

## Nonclaims

These hosted-runner measurements are observations, not SLOs or performance
budgets. They do not accept PLT-404--PLT-408, qualify a release, publish an
artifact, or make cross-target performance claims.
