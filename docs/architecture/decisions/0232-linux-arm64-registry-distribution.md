# ADR 0232: Linux ARM64 registry distribution

- Status: Accepted
- Date: 2026-10-03
- Task ID: CLI-410
- Owners: Qiongli maintainers
- Authority: maintainer request to repair pip/npm installation on Linux and Windows,
  with the reported Ubuntu 24.04 aarch64 installation selecting only 1.x.
- Supersedes: ADR 0220's three-target roster and ADR 0223's six-archive count only.
  ADR 0231's hosted verification and separate publication authority remain.

## Decision

Add `aarch64-unknown-linux-gnu` to the existing canonical native target registry.
Keep macOS ARM64, Linux x64 and Windows x64. One npm package bundles all four
executables; four Python wheels use platform selection. The Linux ARM64 wheel
uses `manylinux_2_35_aarch64`, qualified with auditwheel and a native Ubuntu 22.04
ARM64 build/install job. The combined package is installed on all four targets.
No source fallback, runtime downloader, package identity or user-model change
is introduced. Intel macOS, Windows ARM64, 32-bit Linux and musl remain outside
this precompiled package roster.

The same registry drives the Node launcher, Python tooling host detection and
the existing DSH projection. Native Marketplace Plugins add the matching ARM64
pair. New combined release manifests use schema 2 and require all four receipts
and their artifacts, including exact wheel architecture and executable bytes.
Immutable unversioned packets through 2.1.1 retain their original three-target
verification; later versions cannot omit the new schema to weaken the gate.

pip filters incompatible releases before choosing a version. The observed
unconstrained ARM64 lookup selects 1.17.0 while x64/Windows/macOS select 2.1.1.
Document `qiongli>=2,<3` for users requiring native 2.x so unsupported targets
fail explicitly. Source implementation does not alter already published wheels;
the fix must ship in a new version through the existing release owners.

## Verification and rollback

Exercise actual ARM64 pip/npm installs, both aliases, embedded resources and
Lite/Full MCP. Retain Windows command-shim and other target-native CI checks.
Test missing target receipts, swapped executable architectures, changed wheel
bytes, unsupported dispatch, and old three-target packet verification. These
development checks do not establish hosted or published candidate acceptance.
Rollback uses a previously published compatible package; never relabel x64
wheels as ARM64 or replace an immutable release. Research preview/approval/CAS,
Host registration and configured models remain with their existing owners.
