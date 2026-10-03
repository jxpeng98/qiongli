# Install Qiongli

Choose one CLI installation method, then run `qiongli install` to connect your Host. The current version is **2.2.0**.

## Direct download {#standalone-binary-download}

The standalone program includes research Skills, templates and Lite/Full MCP. No separate Python, Node.js or Rust installation is needed.

| Platform | Download |
|---|---|
| macOS Apple Silicon (ARM64, macOS 11+) | [tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [zip](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-x86_64-pc-windows-msvc.zip) |
| Linux x64 (glibc 2.35+) | [tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64 (glibc 2.35+) | [tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-aarch64-unknown-linux-gnu.tar.gz) |

Choose the platform archive under the release's **Assets**. **Source code** archives need a build. Binary packages do not cover Intel macOS, Windows ARM64, 32-bit systems or musl Linux. Windows includes the C runtime.

### Verify and run

Download [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/SHA256SUMS) from the same release and compare the SHA-256 for your exact filename. Use `shasum -a 256 <filename>` on macOS, `sha256sum <filename>` on Linux, or `Get-FileHash <filename> -Algorithm SHA256` in PowerShell.

If the hashes match, extract into a new directory and run there:

```sh
./qiongli --version
./qiongli install
```

In PowerShell, use `.\qiongli.exe --version` and `.\qiongli.exe install`. To use the command from any directory, add the extracted directory to your user PATH and open a new terminal.

## Package managers {#package-managers}

Choose one. npm and PyPI include the native executable; Cargo builds from source. Each provides `qiongli` and `ql`.

| Channel | Requirement | Install command |
|---|---|---|
| npm | Node.js 18+ | `npm install --global qiongli@latest` |
| PyPI | Python 3.9+; use a virtual environment | `python -m pip install --upgrade "qiongli==2.2.0"` |
| Cargo | Rust 1.97+ and a native linker | `cargo install qiongli --version 2.2.0 --locked` |

Pinning the pip version prevents an incompatible platform from silently selecting 1.x. npm prereleases use `qiongli@next`, which may differ from stable.

## Connect your Host {#first-use}

```sh
qiongli --version
qiongli install
qiongli mcp check
```

Choose Plugin and your Host, review the file plan and registration, and confirm each step. The Plugin includes Skills, the program and Full MCP.

Open a new Host session. Ask it to list Qiongli tools and call `qiongli_config_status`. `mcp check` checks only the local protocol. Then try a research request from the [quickstart](../quickstart.md).

See [Plugin setup](../advanced/plugin-installation.md) for Host selection, language, hooks and DeepSeek profiles. Already installed? Follow the [upgrade guide](upgrade.md).
