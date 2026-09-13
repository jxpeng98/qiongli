# 本地桌面开发与打包

本指南是 Qiongli 2 Svelte/Tauri 桌面应用的维护者快速路径，覆盖源码开发、原生运行、
本地安装包组装，以及“源码包”和“可发布产品”之间的区别。

## 先确认正在运行哪一种构建

| 构建 | 原生服务 | 是否写入真实穷理状态 | packaged-product authority | 用途 |
|---|---:|---:|---:|---|
| 浏览器 fixture | 否 | 否 | 否 | 快速调整布局、响应式与本地化 |
| `cargo run` 源码 App | 是 | 明确预览/确认后才会写入 | 否 | 开发原生 UI、CLI、项目与服务 |
| 本地源码包 | 是 | 明确预览/确认后才会写入 | 否 | 检查打包结构和目标平台启动 |
| product-controlled acceptance App | 是 | 只写入隔离的测试 home | 临时测试 authority | 自动验收安装、更新和集成 |
| 正式提升的发布包 | 是 | 明确预览/确认后才会写入 | Production 或 Community Alpha authority | 分发给测试者或用户 |

普通源码构建按设计不会携带 packaged-product authority。因此，看到
`Confirmation is unavailable because this source build has no packaged-product authority`、
Apply operations unavailable、客户端安装或更新 unavailable 都属于预期结果。不要为让源码
构建看起来像正式产品而增加绕过逻辑。

## macOS 一条命令构建

如果只需要在当前 Mac 上测试完整 App，不需要 Windows/Linux、发布验收、签名公证或安装包，
在仓库根目录运行：

```bash
pnpm desktop:macos
```

首次使用前仍需执行一次 `pnpm install --frozen-lockfile`。命令会依次构建 Svelte 静态资源和
当前 Mac 的 Rust executable，然后生成本地 ad-hoc 签名的：

```text
dist/macos/Qiongli.app
```

App 包会同时包含 `Contents/MacOS/Qiongli` 和同版本的
`Contents/MacOS/qiongli-cli`。在 **关于 → 穷理 CLI** 中，可以查看内置版本、受管
`~/.local/bin/qiongli` 目标，以及当前观察到的 `PATH` 是否仍被旧 pip/npm 命令遮蔽。
普通源码包只能检查这些状态；由于没有 packaged-product authority，它不会写入用户 CLI
目录。

它不会运行跨平台门禁、安全扫描、release composer、notarization 或 product-control
验收。直接构建并打开可使用：

```bash
pnpm desktop:macos:open
```

该命令固定使用 Cargo release profile，并启用 Qiongli 的 `custom-protocol` feature，让
Tauri 从 executable 内提供嵌入式 Svelte 资源，而不是连接 `http://127.0.0.1:1420`。这个
`.app` 是本地源码 App，可以验证真实 Tauri/Svelte 和原生服务，但不携带
packaged-product authority，不能作为发布包。下文较长的 composer 和签名步骤只用于发布
结构、更新链与分发验收。

## macOS 一条命令测试安装功能

普通源码 App 按设计不能安装 Qiongli Skills 或客户端插件，因为它没有签名的产品授权。
如果需要测试这些操作，同时避免改动真实的 `~/.codex` 和 `~/.claude`，运行：

```bash
pnpm desktop:macos:acceptance:open
```

该命令要求 Git 工作区保持干净，避免包含未提交代码的 App 仍显示上一个提交的构建身份。
它会使用临时开发 authority 构建同一套内嵌 Svelte 应用，组装并 ad-hoc 签名一个
禁止发布的验收包；随后自动完成 Skills 的 materialize/verify/refresh，以及 Codex 和
Claude Code 的 install/verify/repair/remove。它还会在单独 HOME 中创建 9 个受支持的
穷理 1.x 表面（8 个客户端集成表面，加上旧版 provider 配置），完整执行预览、暂存、
验证、清理与收尾，验证 provider 设置已经转换、旧版表面已经归零，并重新验证两个
2.x 客户端安装。所有检查通过后，测试 App 才会写入：

```text
dist/macos-acceptance/current/extracted/Qiongli.app
```

自动破坏性检查使用 `dist/macos-acceptance/current/automated-home`。打开的进程使用独立且
干净的 `dist/macos-acceptance/current/manual-home` 作为 `HOME`，因此只会发现测试用的
Codex 和 Claude 目录，不能把集成状态写入真实用户目录。之后可以在 App 内按正常 preview 和
confirmation 流程再次手动测试安装。验收证据位于同一测试目录下的
`qiongli-packaged-product-acceptance.receipt.json`。
也可以在同一个隔离 App 的 **关于 → 穷理 CLI** 中预览并安装内置原生 CLI；受管目标位于
`manual-home/.local/bin`，因此不会替换真实 HOME 中的 pip/npm 安装。

旧版研究项目不会通过扫描整个 `HOME` 自动猜测。请为每个项目显式选择源目录和目标目录，
先运行 `qiongli project migrate preview`，再使用它返回的摘要执行绑定 digest 的
`apply`。源项目保持不变，只有新建的 2.x 项目会被注册。

1.x 替换事务只使用
`dist/macos-acceptance/current/legacy-migration-home`；这个目录不会作为手动 UI 的
`HOME` 打开，因此旧版夹具不会再出现在截图或交互验证使用的客户端状态中。

该包通过验收输出目录与普通源码包区分，只使用 ad-hoc 签名，明确记录
`publication_allowed: false`，安装 grant 在一小时后失效。过期后重新运行命令即可。
不要把它复制到 `/Applications`、通过 Finder 单独打开或分发；这些方式会丢失隔离启动
环境，或错误地把非发布验收证据当成产品包。

## 当前主机的 Claude Desktop Full MCPB

Claude Desktop 可以通过 `.mcpb` Desktop Extension 启动本地二进制 MCP。为当前操作系统和
架构构建单独标识的穷里 Full 包：

```bash
pnpm mcpb:pack:full
```

该命令会构建 Rust `qiongli` release executable，并使用空 `PATH` 和隔离配置目录实际启动
它，验证由 Lite、项目与宿主编排组成的 30 个工具清单，随后写入：

```text
dist/qiongli-full-runtime-2.0.0-alpha.2.mcpb
dist/qiongli-full-runtime-2.0.0-alpha.2.receipt.json
```

在 **Claude Desktop → Settings → Extensions → Advanced settings → Install
Extension…** 中手动选择 MCPB。安装、信任、启用、重启、实时挂接和工具批准均由宿主负责；
本地构建回执只证明包内容、目标身份和运行时清单。因此回执固定记录
`publication_allowed: false`，也不会宣称 Claude Desktop 已连接。

现有 `qiongli-literature-provider-*.mcpb` 继续保持 Marketplace Lite。两个 MCPB 都不会
激活 Claude Web、Codex Cloud 或其他远程 Worker。Claude Desktop 的官方安装说明见
[Getting Started with Local MCP Servers on Claude
Desktop](https://support.claude.com/en/articles/10949351-getting-started-with-local-mcp-servers-on-claude-desktop)。

## 环境要求

使用原生 CI 已验证的版本：

- Node.js 24；
- pnpm 11.13.1，由根目录 `packageManager` 固定；
- Rust 1.97.0，并包含 `rustfmt`、`clippy`，由
  `packages/qiongli-native/rust-toolchain.toml` 固定；
- Tauri 2 在当前平台所需的 WebView 和原生编译依赖。

下文支持的命令不要求全局安装 Tauri CLI。macOS 需要 Xcode Command Line Tools；
Windows 需要 MSVC C++ Build Tools、Windows SDK 与 WebView2。Debian/Ubuntu 可使用与
CI 一致的依赖：

```bash
sudo apt-get install --no-install-recommends \
  libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

在仓库根目录安装锁定的前端依赖并核对工具版本：

```bash
pnpm install --frozen-lockfile
node --version
pnpm --version
(cd packages/qiongli-native && rustc --version)
```

## Svelte UI 快速开发

使用只读开发 transport 启动 Svelte：

```bash
pnpm --dir packages/qiongli-desktop dev
```

打开 `http://127.0.0.1:1420/?fixture=source-read-only`。fixture 提供有类型且确定的
样例数据，不会调用原生命令或写入项目状态。没有 fixture 参数的普通浏览器页面缺少
Tauri IPC bridge，不能代替完整原生宿主。

这个模式适合开发样式、紧凑布局、响应式、本地化和组件状态。完成前仍需在完整 App 中
验证原生行为。

## 运行完整源码 App

先生成静态 Svelte 资源，再运行 canonical Rust executable：

```bash
pnpm desktop:build
cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --features custom-protocol \
  --locked
```

`qiongli` 是 package 的默认 binary；没有 CLI 参数时，`cargo run` 会直接打开桌面窗口。
修改 Svelte 后先重新执行 `pnpm desktop:build`，再重启原生 App；Rust 变更会由 Cargo
自动重新编译。

源码 App 会使用真实原生服务，也会发现本机实际客户端和项目。任何写入仍必须经过 App
的 preview 和 confirmation。无需打开窗口即可检查同一套原生状态：

```bash
cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --locked \
  -- doctor
```

构建成功后，可直接运行 `packages/qiongli-native/target/debug/qiongli`，Windows 对应
`qiongli.exe`。运行已生成的 executable 不依赖 Node 或 pnpm；只有重新构建嵌入式前端
时才需要这些工具。

## 验证桌面变更

开发时先运行与变更最相关的检查：

```bash
pnpm desktop:check
pnpm desktop:test
pnpm desktop:build
cargo test \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --locked
```

提交原生变更前，再运行 `CONTRIBUTING.md` 中的完整原生门禁；提交文档变更前还应运行
`pnpm docs:build`。

## 组装本地源码包

Qiongli 不把通用 Tauri bundler 作为权威打包边界：
`packages/qiongli-native/apps/qiongli/tauri.conf.json` 中的 `bundle.active` 保持关闭。
仓库自己的 native composer 会把 canonical runtime、轻量桌面 launcher、update
helper、嵌入式资源包、应用 metadata、目标平台与源码 commit 绑定到一份经过验证的
archive 和 receipt 中。

需要可信源码绑定时，应使用干净且已经提交的 checkout。在 macOS 或 Linux 的仓库根目录
执行：

```bash
set -euo pipefail

REPO_ROOT="$(pwd -P)"
SOURCE_COMMIT="$(git rev-parse HEAD)"
PACKAGE_PARENT="$(mktemp -d "${TMPDIR:-/tmp}/qiongli-local-package.XXXXXX")"
PACKAGE_ROOT="$PACKAGE_PARENT/artifact"
TARGET_DIR="$REPO_ROOT/packages/qiongli-native/target/release"

pnpm desktop:build
QIONGLI_NATIVE_SOURCE_COMMIT="$SOURCE_COMMIT" cargo build \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --release \
  --bins \
  --features custom-protocol \
  --locked

QIONGLI_NATIVE_SOURCE_COMMIT="$SOURCE_COMMIT" cargo run \
  --manifest-path packages/qiongli-native/Cargo.toml \
  --package qiongli \
  --example native_desktop_package \
  --release \
  --locked \
  -- \
  --canonical "$TARGET_DIR/qiongli" \
  --launcher "$TARGET_DIR/qiongli-desktop" \
  --update-helper "$TARGET_DIR/qiongli-update-helper" \
  --output "$PACKAGE_ROOT" \
  --source-commit "$SOURCE_COMMIT"

printf 'Local package: %s\n' "$PACKAGE_ROOT"
```

输出必须是 checkout 之外、尚不存在的绝对路径。composer 会拒绝已经存在的 output
directory，因此这里只预先创建它的 private parent。

Windows PowerShell 使用同一个 composer，但输入文件带 `.exe`：

```powershell
$RepoRoot = (Get-Location).Path
$SourceCommit = (git rev-parse HEAD).Trim()
$PackageParent = Join-Path $env:TEMP ("qiongli-local-package-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $PackageParent | Out-Null
$PackageRoot = Join-Path $PackageParent "artifact"
$TargetDir = Join-Path $RepoRoot "packages\qiongli-native\target\release"
$env:QIONGLI_NATIVE_SOURCE_COMMIT = $SourceCommit

pnpm desktop:build
cargo build `
  --manifest-path packages/qiongli-native/Cargo.toml `
  --package qiongli --release --bins --features custom-protocol --locked
cargo run `
  --manifest-path packages/qiongli-native/Cargo.toml `
  --package qiongli --example native_desktop_package --release --locked -- `
  --canonical "$TargetDir\qiongli.exe" `
  --launcher "$TargetDir\qiongli-desktop.exe" `
  --update-helper "$TargetDir\qiongli-update-helper.exe" `
  --output "$PackageRoot" `
  --source-commit "$SourceCommit"

Remove-Item Env:QIONGLI_NATIVE_SOURCE_COMMIT
Write-Output "Local package: $PackageRoot"
```

package directory 只包含：

- 当前目标平台的 archive；
- `qiongli-desktop-package.manifest.json`；
- `status: assembled-unpublished` 的
  `qiongli-desktop-package.receipt.json`。

原生输出按宿主平台区分：

| 宿主 | composer 输出 | 本地启动形式 |
|---|---|---|
| macOS | `Qiongli-<version>-macOS-<arch>.source.zip` | 按下节把已验收的 arm64 source package 转为 ad-hoc 测试 ZIP/DMG |
| Windows | `Qiongli-<version>-Windows-x64.zip` | 完整解压 `Qiongli` 目录并运行 `Qiongli.exe` |
| Linux | `Qiongli-<version>-Linux-x64.zip` | 解压并运行 `Qiongli.AppDir/AppRun`；CI 另行完成使用固定工具的 AppImage 转换 |

### 生成 macOS ad-hoc 测试 DMG

在受支持的 macOS arm64 主机上，把经过验证的 source package 转换成本地 ad-hoc
签名的测试 ZIP 和 DMG：

```bash
PACKAGE_SHA256="$(plutil -extract package_sha256 raw \
  "$PACKAGE_ROOT/qiongli-desktop-package.receipt.json")"
SIGNED_ROOT="$PACKAGE_PARENT/ad-hoc"

tooling/scripts/macos_native_sign_notarize.sh \
  --artifact-dir "$PACKAGE_ROOT" \
  --expected-source-commit "$SOURCE_COMMIT" \
  --expected-package-sha256 "$PACKAGE_SHA256" \
  --output-dir "$SIGNED_ROOT" \
  --test-only-ad-hoc
```

生成的 ZIP 和 DMG 只是禁止发布的工程验证证据，不能发给用户，也不能附加到 release。
`--community-alpha` 和 `--production` 属于受控 promotion/signing 流程，不用于普通本地
开发。

## 准备 Alpha.2 宿主验收夹具

无需启动 Codex、Claude Code、Claude Desktop 或模型提供方即可运行离线预检：

```bash
pnpm acceptance:host:preflight
```

该命令会验证固定 canonical fixture、source-fact 与 source-anchor 摘要、必需的
project-read 工具、schema 2 candidate contract 以及 checkpoint transition 顺序。输出
状态是 `fixture-ready-manual-host-required`，并且
`publication_allowed: false`；它不是一份已经通过的宿主验收收据。

原生收据验证方式记录在
`tooling/release/acceptance/fixtures/README.md`。之后的手动宿主会话必须提供精确的
host、adapter、Plugin、binary 和 protocol identity、checkpoint 摘要和计数，以及
direct-model/model-CLI 均为零的判定。收据不能包含 prompt、candidate body、model
response、conversation ID、project ID/path、provider credential 或 tool result。

## 本地包不能证明什么

本地源码包是自包含的：在目标机器启动时不需要 checkout、Rust、Node.js、pnpm、
Cargo、Python 或另一份 Qiongli 安装；目标操作系统仍需提供自己的 WebView 和窗口设施。

但是，自包含不等于拥有发布授权。没有嵌入 release authority 和 product control 时，
integration Apply operations、客户端 plugin 安装和自动更新仍然 unavailable。仅限 macOS
的 `native_packaged_product_acceptance` example 会使用临时密钥和隔离 home 验证这些路径；
它是自动验收 harness，不是可分发 App。

关于分发类型、支持目标、系统信任提示、签名、公证与发布验收，请继续阅读
[原生桌面 Alpha 安装包](/zh/advanced/native-desktop-alpha)。

## 常见错误

| 现象 | 原因与处理 |
|---|---|
| 浏览器页面无法读取原生状态 | 使用 `?fixture=source-read-only`，或运行完整 Tauri App |
| `cargo run` 中没有出现最新 Svelte 变更 | 执行 `pnpm desktop:build` 后重启 App |
| release App 打开后只有空白窗口 | 构建 `qiongli` 时加入 `--features custom-protocol`；`pnpm desktop:macos` 已自动处理 |
| 缺少 `frontendDist` 或静态资源 | 在仓库根目录执行 `pnpm install --frozen-lockfile` 和 `pnpm desktop:build` |
| Rust 使用了错误编译器 | 通过 native manifest/workspace 运行 Cargo，以选择固定的 `rust-toolchain.toml` |
| `desktop-package-source-commit-unbound` | release build 和 composer 必须使用相同的 `QIONGLI_NATIVE_SOURCE_COMMIT` |
| `desktop-package-output-invalid` | 使用 checkout 之外、existing private parent 下尚不存在的绝对输出路径 |
| Apply 或 update unavailable | 普通源码构建或没有 product authority 的源码包中的预期结果 |
| Codex/Claude Code Missing 或 Unavailable | 运行 `qiongli doctor`，并分别检查 client discovery、plugin source、registration、activation 与 MCP attachment；它们不是同一个状态 |
