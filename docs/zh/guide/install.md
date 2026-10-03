# 安装穷理

选择一种 CLI 安装方式，再运行 `qiongli install` 接入 Host。当前版本为 **2.2.0**。

## 直接下载 {#standalone-binary-download}

独立程序自带研究 Skills、模板和 Lite/Full MCP，无需另装 Python、Node.js 或 Rust。

| 平台 | 下载 |
|---|---|
| macOS Apple Silicon（ARM64，macOS 11+） | [tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-aarch64-apple-darwin.tar.gz) |
| Windows x64 | [zip](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-x86_64-pc-windows-msvc.zip) |
| Linux x64（glibc 2.35+） | [tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-x86_64-unknown-linux-gnu.tar.gz) |
| Linux ARM64（glibc 2.35+） | [tar.gz](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/qiongli-2.2.0-aarch64-unknown-linux-gnu.tar.gz) |

从 Release 的 **Assets** 选择平台包。**Source code** 是源码。Intel macOS、Windows ARM64、32 位系统和 musl Linux 不在二进制包支持范围内。Windows 已内置 C 运行库。

### 核对文件并运行

下载同一版本的 [SHA256SUMS](https://github.com/jxpeng98/qiongli/releases/download/v2.2.0/SHA256SUMS)，核对对应文件的 SHA-256。macOS 用 `shasum -a 256 <文件名>`，Linux 用 `sha256sum <文件名>`，PowerShell 用 `Get-FileHash <文件名> -Algorithm SHA256`。

核对一致后解压到新目录，在该目录运行：

```sh
./qiongli --version
./qiongli install
```

PowerShell 使用 `.\qiongli.exe --version` 和 `.\qiongli.exe install`。需要在任意目录使用时，把解压目录加入用户 PATH，然后新开终端。

## 包管理器 {#package-managers}

任选一种。npm 和 PyPI 自带原生程序，Cargo 从源码编译。安装后均提供 `qiongli` 和 `ql`。

| 渠道 | 前提 | 安装命令 |
|---|---|---|
| npm | Node.js 18+ | `npm install --global qiongli@latest` |
| PyPI | Python 3.9+；在虚拟环境中运行 | `python -m pip install --upgrade "qiongli==2.2.0"` |
| Cargo | Rust 1.97+ 与本机链接器 | `cargo install qiongli --version 2.2.0 --locked` |

pip 指定版本可避免在平台不匹配时意外安装 1.x。预发布 npm 使用 `qiongli@next`，版本和稳定版可能不同。

## 接入 Host {#first-use}

```sh
qiongli --version
qiongli install
qiongli mcp check
```

选择 Plugin 和 Host，审阅文件计划与注册操作，分别确认。Plugin 已包含 Skills、程序和 Full MCP。

新开 Host 会话，请它列出穷理工具并调用 `qiongli_config_status`。`mcp check` 只检查本地协议。接着按[快速开始](../quickstart.md)提出研究请求。

指定 Host、语言、Hook 或 DeepSeek profile，见 [Plugin 配置](../advanced/plugin-installation.md)。已安装用户见[升级指南](upgrade.md)。
