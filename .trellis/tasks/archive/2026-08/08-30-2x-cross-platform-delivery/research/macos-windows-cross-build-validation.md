# macOS-to-Windows cross-build validation

Validation date: 2026-08-31

## Integration audit

- The working branch is `2.x`; remote `origin/2.x` already contains the Windows
  config-root repair from PR 158.
- Windows repair commits `b219f6d6` and merged `8490254e` have the same stable
  patch ID: `0bfcbc5c02e5affab94398f12777eb055414246f`.
- Remote-only `457addc5` temporarily enabled candidate diagnostics on its topic
  branch. It is not product work to merge after the repair and acceptance were
  completed.

## Host and tools

- Host: Apple Silicon macOS (`arm64`, Darwin).
- Workspace: `packages/qiongli-native`, Rust `1.97.0` from its pinned toolchain.
- Windows target: `x86_64-pc-windows-msvc`.
- Cross-builder: third-party `cargo-xwin 0.23.1`; Microsoft SDK licence use was
  explicitly approved before the target/SDK setup.
- Guest: Parallels 27, Microsoft Windows 11 Enterprise 10.0.26100, ARM64;
  Parallels Tools 26.3.0. Windows x64 executables ran through x64 emulation.

## Commands and results

From `packages/qiongli-native/`:

```bash
cargo test --workspace --all-targets --all-features --locked
cargo xwin build --workspace --release --target x86_64-pc-windows-msvc --locked
cargo xwin test --workspace --no-run --all-features --target x86_64-pc-windows-msvc --locked
```

- macOS workspace tests: passed; only external Codex/Claude CLI tests were
  ignored by their declared prerequisites.
- Windows release cross-build: passed in 2m39s.
- Windows test compilation: passed in 3m07s; 36 test executables were emitted
  and were not run on macOS.
- Tauri generated `apps/qiongli/gen/schemas/windows-schema.json`; its SHA-256 is
  `623c82e1cf0b1093b61b4b0900f8e2f06fbef272b8e5c24cc90dde35b063da97`,
  identical to the current desktop and macOS schemas.

## Release artifact identities

Output root: `packages/qiongli-native/target/x86_64-pc-windows-msvc/release/`

| Artifact | Type | SHA-256 |
| --- | --- | --- |
| `qiongli.exe` | PE32+ console x86-64 | `57ea8be2ed899ece5f4b14cf7adc627a2c88590e3599591a3f338781dc17eb8c` |
| `qiongli-desktop.exe` | PE32+ GUI x86-64 | `39a78e8b4da5b30407b2a06b6411c514de70719203e4189aef283caa79bf6335` |
| `qiongli-update-helper.exe` | PE32+ console x86-64 | `aa6f823d9c7e86834ee211fc339a521eb500f45b9ba133f591357d78d3b63f10` |

The hashes matched after transfer to the guest-local
`C:\QiongliCrossSmoke\11f213a4` directory.

## Windows guest smoke

Passed under the logged-in Windows user:

- `qiongli.exe --version` returned `qiongli 2.0.0-alpha.3`;
- `qiongli.exe content list` loaded the embedded verified content pack;
- `qiongli-desktop.exe --internal-startup-check` exited successfully;
- an isolated `QIONGLI_CONFIG_HOME` persisted `default_profile=full` at
  revision 1 and a fresh process read the same state;
- `qiongli.exe doctor` completed with non-blocking attention for unavailable
  local integrations/secure store and platform-limited update recovery;
- a stale revision-0 update failed with `revision-conflict`, and the following
  status remained revision 1 with profile `full`.

## Residual risk and nonclaims

Validated on Windows 11 Arm with x64 emulation; native Windows x64 hardware was
not verified. The run did not visually inspect the full GUI/menu journey, run
network/update transactions, exercise the update helper, validate an installer
on a clean machine, sign artifacts, or publish anything. The development smoke
does not replace the exact-head native Windows CI Slice or explicit candidate
Acceptance.
